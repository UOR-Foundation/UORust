//! Rigid compiler type-flow facts anchored in existing source view schemas.
use std::collections::{BTreeMap, BTreeSet};

use super::ir::{IrBody, IrOperand, IrPlace, IrProjection, IrRvalue, IrStatement};
use crate::lift::{Lift, Residue, Ty, Types};
use quote::ToTokens;

fn value_type(text: &str) -> Option<String> {
    let mut parsed = syn::parse_str::<syn::Type>(text).ok()?;
    if super::views::borrows_mutably(&crate::convert::ty(&parsed)) {
        return None;
    }
    loop {
        parsed = match parsed {
            syn::Type::Reference(r) if r.mutability.is_none() => *r.elem,
            syn::Type::Reference(_) | syn::Type::Ptr(_) => return None,
            syn::Type::Paren(p) => *p.elem,
            other => return Some(other.to_token_stream().to_string()),
        };
    }
}

fn read(text: &str, types: &Types, inferred: &BTreeMap<String, Ty>) -> Option<Ty> {
    super::lower::ty(super::lower::mut_target(text).unwrap_or(text), types)
        .ok()
        .or_else(|| inferred.get(&value_type(text)?).cloned())
}

fn field(types: &Types, value: &Ty, index: usize) -> Option<Ty> {
    match value {
        Ty::Tuple(items) | Ty::Pair(items) => items.get(index).cloned(),
        Ty::Named(name) => types
            .structs
            .get(name)?
            .get(index)
            .map(|(_, ty)| ty.clone()),
        _ => None,
    }
}

fn place(
    body: &IrBody,
    at: &IrPlace,
    types: &Types,
    inferred: &BTreeMap<String, Ty>,
) -> Option<(String, Option<Ty>)> {
    let mut text = body.locals.get(at.local)?.clone();
    let mut model = read(&text, types, inferred);
    for projection in &at.projection {
        match projection {
            IrProjection::Deref => {
                let parsed = syn::parse_str::<syn::Type>(&text).ok()?;
                let syn::Type::Reference(reference) = parsed else {
                    return None;
                };
                // A known mutable container still has its declared field schema;
                // an unread mutable value cannot acquire an abstract carrier.
                text = reference.elem.to_token_stream().to_string();
            }
            IrProjection::Field(index, printed) => {
                model = model.as_ref().and_then(|ty| field(types, ty, *index));
                text.clone_from(printed);
            }
            _ => return None,
        }
    }
    Some((text, model))
}

fn constrain(
    a: &(String, Option<Ty>),
    b: &(String, Option<Ty>),
    inferred: &mut BTreeMap<String, Ty>,
) -> Lift<bool> {
    let (Some(left), Some(right)) = (value_type(&a.0), value_type(&b.0)) else {
        return Ok(false);
    };
    if left != right {
        return Ok(false);
    }
    let known = match (&a.1, &b.1) {
        (Some(a), Some(b))
            if a != b && (!a.parameters().is_empty() || !b.parameters().is_empty()) =>
        {
            return Err(Residue::Type(
                "conflicting source-bound compiler carrier identities".into(),
            ));
        }
        (Some(a), _) | (_, Some(a)) if !a.parameters().is_empty() => a,
        _ => return Ok(false),
    };
    if let Some(old) = inferred.get(&left) {
        if old != known {
            return Err(Residue::Type(
                "conflicting source-bound compiler carrier identities".into(),
            ));
        }
        return Ok(false);
    }
    inferred.insert(left, known.clone());
    Ok(true)
}

fn transported(value: &IrRvalue) -> Option<&IrPlace> {
    match value {
        IrRvalue::Use(IrOperand::Copy(p) | IrOperand::Move(p))
        | IrRvalue::CopyForDeref(p)
        | IrRvalue::Cast(_, IrOperand::Copy(p) | IrOperand::Move(p), _)
        | IrRvalue::Ref(super::spec::MirBorrowKind::Shared, p) => Some(p),
        _ => None,
    }
}

/// Solve only transport constraints. An unknown operation supplies no equation
/// and an unanchored type stays unread. No opaque sort or default is invented.
pub(crate) fn infer(body: &IrBody, types: &Types) -> Lift<BTreeMap<String, Ty>> {
    let mut reached = BTreeSet::new();
    let mut pending = vec![0];
    while let Some(index) = pending.pop() {
        if !reached.insert(index) {
            continue;
        }
        if let Some(block) = body.blocks.get(index) {
            pending.extend(block.terminator.successors());
        }
    }
    let mut inferred = BTreeMap::new();
    // rustc has resolved the type of every local and field. Equal normalized
    // compiler types are already one constraint node; collecting rigid source
    // anchors once computes their transitive closure without iteration.
    let empty = BTreeMap::new();
    for index in &reached {
        let Some(block) = body.blocks.get(*index) else {
            continue;
        };
        for statement in &block.statements {
            let IrStatement::Assign(destination, value) = statement else {
                continue;
            };
            let Some(source) = transported(value) else {
                continue;
            };
            if let (Some(a), Some(b)) = (
                place(body, destination, types, &empty),
                place(body, source, types, &empty),
            ) {
                constrain(&a, &b, &mut inferred)?;
            }
        }
    }
    Ok(inferred)
}

/// Apply the inferred frame through the same native type reader used by both
/// signatures and bodies. Compiler spelling variants share one rigid reading.
pub(crate) fn bindings(
    body: &IrBody,
    types: &Types,
    inferred: &BTreeMap<String, Ty>,
) -> BTreeMap<String, Ty> {
    let mut scoped = types.generic.clone();
    let mut printed: Vec<&str> = body.locals.iter().map(String::as_str).collect();
    for block in &body.blocks {
        for statement in &block.statements {
            if let IrStatement::Assign(destination, value) = statement {
                printed.extend(destination.projection.iter().filter_map(|p| {
                    if let IrProjection::Field(_, text) = p {
                        Some(text.as_str())
                    } else {
                        None
                    }
                }));
                if let Some(source) = transported(value) {
                    printed.extend(source.projection.iter().filter_map(|p| {
                        if let IrProjection::Field(_, text) = p {
                            Some(text.as_str())
                        } else {
                            None
                        }
                    }));
                }
            }
        }
    }
    for text in printed {
        if let Some(model) = value_type(text).and_then(|key| inferred.get(&key)) {
            if super::lower::ty(text, types).is_err() {
                scoped.insert(text.trim().to_string(), model.clone());
            }
            if let Some(inner) = super::lower::referent(text) {
                if super::lower::ty(&inner, types).is_err() {
                    scoped.insert(inner, model.clone());
                }
            }
        }
    }
    scoped
}

#[cfg(test)]
mod tests {
    use super::*;
    fn scope() -> Types {
        let mut types = Types::default();
        types.native_views.insert(
            "Holder".into(),
            Ty::Pair(vec![
                Ty::Bool,
                Ty::Param("SourceField".into()),
                Ty::Param("AnotherField".into()),
            ]),
        );
        types
    }
    fn body(text: &str) -> IrBody {
        super::super::parse::bodies(text).remove(0)
    }

    #[test]
    fn source_anchors_constrain_forward_value_flow_without_new_sorts_ue_86() {
        let body=body("fn carried(_1: &Holder) -> &dyn Service {\n let mut _0: &dyn Service;\n let mut _2: &dyn Service;\n bb0: {\n goto -> bb2;\n }\n bb1: {\n _0 = copy _2;\n return;\n }\n bb2: {\n _2 = copy ((*_1).1: &dyn Service);\n goto -> bb1;\n }\n}\n");
        let types = scope();
        let inferred = infer(&body, &types).unwrap();
        assert_eq!(
            inferred.values().cloned().collect::<Vec<_>>(),
            vec![Ty::Param("SourceField".into())]
        );
        let bindings = bindings(&body, &types, &inferred);
        let mut scoped = types.clone();
        scoped.generic = bindings;
        assert_eq!(
            super::super::lower::ty(&body.locals[0], &scoped).unwrap(),
            Ty::Param("SourceField".into())
        );
        assert!(types.generic.is_empty(), "original scope was mutated");
        assert_eq!(
            infer(&body, &types).unwrap(),
            inferred,
            "constraints are not deterministic"
        );
    }

    #[test]
    fn conflicting_unknown_carriers_and_unanchored_or_mutable_types_stay_open_ue_86() {
        let text="fn carried(_1: &Holder) -> &dyn Service {\n let mut _0: &dyn Service;\n bb0: {\n _0 = copy ((*_1).1: &dyn Service);\n _0 = copy ((*_1).2: &dyn Service);\n return;\n }\n}\n";
        assert!(infer(&body(text), &scope())
            .unwrap_err()
            .detail()
            .contains("conflicting"));
        let unanchored=body("fn carried(_1: &dyn Service) -> &dyn Service {\n let mut _0: &dyn Service;\n bb0: {\n _0 = copy _1;\n return;\n }\n}\n");
        assert!(infer(&unanchored, &scope()).unwrap().is_empty());
        assert!(value_type("&mut dyn Service").is_none());
        assert!(value_type("Option<&mut dyn Service>").is_none());
        assert!(value_type("*const dyn Service").is_none());
        assert_ne!(
            value_type("&dyn left::Service"),
            value_type("&dyn right::Service")
        );
    }
}
