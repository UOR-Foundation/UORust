//! A typed import of source semantic models, with compiler-owned call identity.
//!
//! Source and native carriers must agree exactly, rather than merely rendering
//! to the same Lean type. In particular, signed words and Unicode scalars are
//! not source integers and characters. Mutable inputs and trait dispatch need
//! their own state/dictionary interface and cannot enter this value interface.
use std::collections::{BTreeMap, BTreeSet};

use crate::lex::Decl;
use crate::lift::{self, Candidate, Ty, Types};

pub(super) struct Imported {
    pub(super) term: String,
    pub(super) decls: Vec<Decl>,
    pub(super) types: BTreeSet<Ty>,
}

fn source_type(text: &str, types: &Types) -> Option<Ty> {
    let ty: syn::Type = syn::parse_str(&text.replace("::<", "<")).ok()?;
    types.ty(&crate::convert::ty(&ty)).ok()
}

/// Reuse closed source readings only after the native ABI has declined them.
/// Imported SDK identities are not snapshots of the allocation they identify.
pub(super) fn type_model(text: &str, types: &Types) -> Option<Ty> {
    fn compatible(ty: &Ty) -> bool {
        match ty {
            // These carriers differ from native words/scalars, or require
            // the separately signature-bound callable interface.
            Ty::Int(_)
            | Ty::Integer(_)
            | Ty::Char
            | Ty::Signed(_)
            | Ty::Scalar
            | Ty::Fn(..)
            | Ty::Param(_) => false,
            Ty::Named(name) => name.starts_with("Prelude::"),
            Ty::Option(t) | Ty::List(t) | Ty::Set(t, _) => compatible(t),
            Ty::Map(a, b, _) | Ty::Outcome(a, b) => compatible(a) && compatible(b),
            Ty::Pair(ts) | Ty::Tuple(ts) => ts.iter().all(compatible),
            _ => true,
        }
    }
    let reading = source_type(text, types)?;
    compatible(&reading).then_some(reading)
}

/// A standard inherent owner, never a workspace trait or lookalike type.
fn inherent(callee: &str) -> Option<(String, String)> {
    let parts = super::parse::split_path(callee);
    let method = super::lower::erase(parts.last()?);
    if parts.len() < 2 || syn::parse_str::<syn::Ident>(&method).is_err() {
        return None;
    }
    if !matches!(parts[0], "std" | "core" | "alloc")
        && !(parts.len() == 2 && lift::PRIMITIVE_TYPES.contains(&parts[0]))
    {
        return None;
    }
    let owner = parts[parts.len() - 2];
    let owner = match owner
        .strip_prefix("<impl ")
        .and_then(|s| s.strip_suffix('>'))
    {
        Some(ty) => ty.to_string(),
        None => parts[..parts.len() - 1].join("::"),
    };
    Some((owner, method))
}

/// Nominal values retain their source schema at the MIR boundary. Inspect the
/// dependency closure so a recursive container neither hides a mutable cell
/// nor loses coverage merely because its immutable payload has a name.
fn value_interface(ty: &Ty, types: &Types, seen: &mut BTreeSet<String>) -> bool {
    interface(ty, types, seen, false)
}

fn interface(ty: &Ty, types: &Types, seen: &mut BTreeSet<String>, parametric: bool) -> bool {
    match ty {
        Ty::Signed(_) | Ty::Scalar | Ty::Param(_) => parametric,
        Ty::Named(name) => {
            if !seen.insert(name.clone()) {
                return true;
            }
            if types
                .cells
                .get(name)
                .is_some_and(|fields| !fields.is_empty())
            {
                return false;
            }
            if name.starts_with("Prelude::") {
                // The source reader admits SDK-owned value schemas; allocation
                // references carry identity rather than mutable contents.
                return true;
            }
            if let Some(fields) = types.structs.get(name) {
                fields
                    .iter()
                    .all(|(_, t)| interface(t, types, seen, parametric))
            } else if let Some(variants) = types.datas.get(name) {
                variants.iter().all(|(_, fields)| {
                    fields
                        .iter()
                        .all(|(_, t)| interface(t, types, seen, parametric))
                })
            } else {
                types.enums.contains_key(name)
            }
        }
        Ty::Option(t) | Ty::List(t) | Ty::Set(t, _) => interface(t, types, seen, parametric),
        Ty::Outcome(a, b) | Ty::Map(a, b, _) => {
            interface(a, types, seen, parametric) && interface(b, types, seen, parametric)
        }
        Ty::Tuple(ts) | Ty::Pair(ts) => ts.iter().all(|t| interface(t, types, seen, parametric)),
        Ty::Fn(ins, out) => {
            ins.iter().all(|t| interface(t, types, seen, parametric))
                && interface(out, types, seen, parametric)
        }
        _ => true,
    }
}

/// Captured values must have an immutable schema in this shared interface.
pub(super) fn callable_environment(ty: &Ty, types: &Types) -> bool {
    value_interface(ty, types, &mut BTreeSet::new())
}

/// A generic type parameter can transport a native word without interpreting
/// it as a source integer. Its schema must still expose no mutable contents.
pub(super) fn generic_environment(ty: &Ty, types: &Types) -> bool {
    interface(ty, types, &mut BTreeSet::new(), true)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply(
    callee: &str,
    args: &[(String, Ty)],
    texts: &[Option<String>],
    result: &Ty,
    types: &Types,
    widths: &[(u32, Vec<String>)],
    taken: &BTreeSet<String>,
    name: &str,
) -> Option<Imported> {
    let (owner, method) = inherent(callee)?;
    let owner_type = source_type(&format!("&{owner}"), types)?;
    if args.len() != texts.len() {
        return None;
    }
    for ((_, native), text) in args.iter().zip(texts) {
        if !matches!(native, Ty::Fn(..)) && source_type(text.as_deref()?, types)? != *native {
            return None;
        }
    }
    if args
        .iter()
        .map(|(_, t)| t)
        .chain(std::iter::once(result))
        .any(|t| !value_interface(t, types, &mut BTreeSet::new()))
    {
        return None;
    }
    let params: Vec<_> = args
        .iter()
        .enumerate()
        .map(|(i, (_, t))| {
            let name = format!("modelArg{i}");
            (name.clone(), name, t.clone())
        })
        .collect();
    let arguments: Vec<_> = params
        .iter()
        .enumerate()
        .map(|(position, (n, _, t))| match t {
            // A source combinator expects closure syntax. The function carrier
            // is already immutable: eta expansion preserves its value semantics.
            Ty::Fn(ins, _) => {
                let zero = texts[position]
                    .as_deref()
                    .and_then(|text| syn::parse_str::<syn::TypeFnPtr>(text).ok())
                    .is_some_and(|signature| signature.inputs.is_empty());
                if zero && ins.as_slice() == [Ty::Unit] {
                    return format!("|| {n}(())");
                }
                let vars = (0..ins.len())
                    .map(|i| format!("input{i}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("|{vars}| {n}({vars})")
            }
            _ => n.clone(),
        })
        .collect();
    let owner_syntax: syn::Type = syn::parse_str(&owner.replace("::<", "<")).ok()?;
    let syn::Type::Path(owner_path) = owner_syntax else {
        return None;
    };
    if owner_path.qself.is_some() {
        return None;
    }
    let owner_name = owner_path.path.segments.last()?.ident.to_string();
    let mut calls = Vec::new();
    if args
        .first()
        .is_some_and(|(_, receiver)| *receiver == owner_type)
    {
        calls.push(format!("modelArg0.{method}({})", arguments[1..].join(", ")));
    }
    // The source binder owns the constructor's meaning. This bridge changes
    // only the invocation form after checking the compiler's owner and ABI.
    calls.push(format!("{owner_name}::{method}({})", arguments.join(", ")));
    calls.into_iter().find_map(|call| {
        let syntax: syn::Block = syn::parse_str(&format!("{{ {call} }}")).ok()?;
        let block = crate::convert::block(&syntax);
        let candidate = Candidate {
            rust: name.into(),
            name: name.into(),
            called: name.into(),
            generic: None,
            params: params.clone(),
            result: result.clone(),
            block: &block,
            imports: BTreeMap::from([(owner_name.clone(), owner.clone())]),
            aliases: BTreeMap::new(),
            homonyms: BTreeMap::new(),
            formatter: None,
            seen: Default::default(),
            threads: Vec::new(),
        };
        let lifted = lift::lift_models(&[candidate], types, widths, taken);
        if !lifted.failed.is_empty() || !lifted.divergences.is_empty() || lifted.decls.is_empty() {
            return None;
        }
        let mut decls = crate::order::decls(&lifted.ordered, types);
        decls.extend(lifted.decls);
        Some(Imported {
            term: format!(
                "({})",
                std::iter::once(name)
                    .chain(args.iter().map(|(v, _)| v.as_str()))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            decls,
            types: lifted.types,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_semantic_calls_include_associated_forms_ue_81() {
        let types = Types::default();
        let list = Ty::List(Box::new(Ty::Nat(Some(8))));
        let map = Ty::Map(Box::new(Ty::Nat(Some(8))), Box::new(Ty::Nat(Some(8))), true);
        for (path, result) in [
            ("std::string::String::new", Ty::Str),
            ("std::vec::Vec::<u8>::new", list),
            ("std::collections::BTreeMap::<u8,u8>::new", map),
        ] {
            let imported = apply(
                path,
                &[],
                &[],
                &result,
                &types,
                &[],
                &BTreeSet::new(),
                "importedFactory",
            );
            assert!(
                imported.is_some(),
                "{path}: existing source meaning was not imported"
            );
        }
        assert!(apply(
            "workspace::String::new",
            &[],
            &[],
            &Ty::Str,
            &types,
            &[],
            &BTreeSet::new(),
            "fake"
        )
        .is_none());
    }

    #[test]
    fn source_type_readings_keep_native_abi_and_state_boundaries_ue_65() {
        let mut types = Types::default();
        let rc = "std::rc::Rc<std::cell::RefCell<u32>>";
        assert_eq!(
            type_model(rc, &types),
            Some(Ty::Named("Prelude::RcCellRef".into()))
        );
        assert_eq!(
            super::super::lower::ty(rc, &types).unwrap(),
            Ty::Named("Prelude::RcCellRef".into())
        );
        assert_eq!(
            super::super::lower::ty("i8", &types).unwrap(),
            Ty::Signed(Some(8))
        );
        assert_eq!(super::super::lower::ty("char", &types).unwrap(), Ty::Scalar);
        for text in [
            "i8",
            "i128",
            "char",
            "std::cell::RefCell<u32>",
            "&std::cell::RefCell<u32>",
            "std::sync::Arc<std::sync::Mutex<u32>>",
            "dyn Fn(bool) -> bool",
            "custom::Rc<std::cell::RefCell<u32>>",
        ] {
            assert!(type_model(text, &types).is_none(), "{text}");
        }
        types.generic.insert("T".into(), Ty::Param("T".into()));
        assert!(type_model("T", &types).is_none());
        types
            .structs
            .insert("Local".into(), vec![("flag".into(), Ty::Bool)]);
        assert!(type_model("Local", &types).is_none());
    }

    #[test]
    fn source_model_import_checks_recursive_schema_closure_ue_64() {
        let mut types = Types::default();
        types.structs.insert(
            "Payload".into(),
            vec![
                (
                    "next".into(),
                    Ty::Option(Box::new(Ty::Named("Payload".into()))),
                ),
                ("flag".into(), Ty::Bool),
            ],
        );
        assert!(value_interface(
            &Ty::Named("Payload".into()),
            &types,
            &mut BTreeSet::new()
        ));
        let optional = Ty::Option(Box::new(Ty::Named("Payload".into())));
        let call = |types: &Types| {
            apply(
                "std::option::Option::<Payload>::is_some",
                &[("value".into(), optional.clone())],
                &[Some("Option<Payload>".into())],
                &Ty::Bool,
                types,
                &[(64, Vec::new())],
                &BTreeSet::new(),
                "imported",
            )
        };
        assert!(call(&types).is_some());
        types.cells.insert(
            "Payload".into(),
            BTreeMap::from([("flag".into(), lift::CellKind::Cell)]),
        );
        assert!(
            call(&types).is_none(),
            "recursive mutable content is not an immutable snapshot"
        );
    }

    #[test]
    fn source_model_import_requires_exact_compiler_bindings_ue_64() {
        let types = Types::default();
        let widths = [(64, Vec::new())];
        let taken = BTreeSet::new();
        let optional = Ty::Option(Box::new(Ty::Nat(Some(8))));
        let invoke = |path: &str, text: &str, input: Ty, result: Ty| {
            apply(
                path,
                &[("value".into(), input)],
                &[Some(text.into())],
                &result,
                &types,
                &widths,
                &taken,
                "imported",
            )
        };
        assert!(invoke(
            "std::option::Option::<u8>::unwrap_or_default",
            "Option<u8>",
            optional.clone(),
            Ty::Nat(Some(8))
        )
        .is_some());
        for (path, text, input, result) in [
            (
                "std::option::Option::<u8>::unwrap",
                "Option<u8>",
                optional.clone(),
                Ty::Nat(Some(8)),
            ),
            (
                "std::option::Option::<u8>::unwrap_or_default",
                "&mut Option<u8>",
                optional.clone(),
                Ty::Nat(Some(8)),
            ),
            (
                "local::Option::<u8>::unwrap_or_default",
                "Option<u8>",
                optional.clone(),
                Ty::Nat(Some(8)),
            ),
            (
                "<Option<u8> as local::Trait>::unwrap_or_default",
                "Option<u8>",
                optional.clone(),
                Ty::Nat(Some(8)),
            ),
            (
                "std::vec::Vec::<u8>::unwrap_or_default",
                "Option<u8>",
                optional,
                Ty::Nat(Some(8)),
            ),
            (
                "std::option::Option::<i8>::unwrap_or_default",
                "Option<i8>",
                Ty::Option(Box::new(Ty::Signed(Some(8)))),
                Ty::Signed(Some(8)),
            ),
            (
                "std::str::<impl str>::to_lowercase",
                "&str",
                Ty::Str,
                Ty::Str,
            ),
            (
                "std::str::<impl str>::to_ascii_lowercase",
                "&str",
                Ty::Str,
                Ty::Nat(Some(8)),
            ),
        ] {
            assert!(invoke(path, text, input, result).is_none(), "{path} {text}");
        }
    }
}
