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
    match ty {
        Ty::Signed(_) | Ty::Scalar | Ty::Param(_) => false,
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
            if let Some(fields) = types.structs.get(name) {
                fields.iter().all(|(_, t)| value_interface(t, types, seen))
            } else if let Some(variants) = types.datas.get(name) {
                variants
                    .iter()
                    .all(|(_, fields)| fields.iter().all(|(_, t)| value_interface(t, types, seen)))
            } else {
                types.enums.contains_key(name)
            }
        }
        Ty::Option(t) | Ty::List(t) | Ty::Set(t, _) => value_interface(t, types, seen),
        Ty::Outcome(a, b) | Ty::Map(a, b, _) => {
            value_interface(a, types, seen) && value_interface(b, types, seen)
        }
        Ty::Tuple(ts) | Ty::Pair(ts) => ts.iter().all(|t| value_interface(t, types, seen)),
        Ty::Fn(ins, out) => {
            ins.iter().all(|t| value_interface(t, types, seen)) && value_interface(out, types, seen)
        }
        _ => true,
    }
}

/// Captured values must have an immutable schema in this shared interface.
pub(super) fn callable_environment(ty: &Ty, types: &Types) -> bool {
    value_interface(ty, types, &mut BTreeSet::new())
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
    let (_, receiver) = args.first()?;
    if args.len() != texts.len() || source_type(&format!("&{owner}"), types)? != *receiver {
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
    let arguments = params
        .iter()
        .enumerate()
        .skip(1)
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
        .collect::<Vec<_>>()
        .join(", ");
    let syntax: syn::Block =
        syn::parse_str(&format!("{{ modelArg0.{method}({arguments}) }}")).ok()?;
    let block = crate::convert::block(&syntax);
    let candidate = Candidate {
        rust: name.into(),
        name: name.into(),
        called: name.into(),
        generic: None,
        params,
        result: result.clone(),
        block: &block,
        imports: BTreeMap::new(),
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
            "({name} {})",
            args.iter()
                .map(|(v, _)| v.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        ),
        decls,
        types: lifted.types,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
