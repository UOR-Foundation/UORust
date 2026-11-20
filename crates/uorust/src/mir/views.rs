//! Native field views over complete source shapes, using SDK products.
use std::collections::{BTreeMap, BTreeSet};

use crate::ast::RustType;
use crate::lift::{self, Ty, Types};

/// Whether a compiler type contains a viewed nominal type, including a view
/// whose fields happen to have closed native readings.
pub(crate) fn touches(text: &str, types: &Types) -> bool {
    if types.native_views.is_empty() {
        return false;
    }
    syn::parse_str::<syn::Type>(text).ok().is_some_and(|t| {
        crate::convert::ty(&t)
            .paths()
            .iter()
            .any(|p| types.native_views.contains_key(&types.resolve(p)))
    })
}

pub(crate) fn borrows_mutably(t: &RustType) -> bool {
    use crate::ast::{GenericArgument, PathArguments, TypeKind};
    match &t.kind {
        TypeKind::Reference { mutable: true, .. } => true,
        TypeKind::Reference { elem, .. }
        | TypeKind::Slice(elem)
        | TypeKind::Array(elem, _)
        | TypeKind::Paren(elem) => borrows_mutably(elem),
        TypeKind::Tuple(ts) => ts.iter().any(borrows_mutably),
        TypeKind::Path { path, .. } => path.segments.iter().any(|s| match &s.arguments {
            PathArguments::AngleBracketed(args) => args
                .iter()
                .any(|a| matches!(a,GenericArgument::Type(t) if borrows_mutably(t))),
            _ => false,
        }),
        _ => false,
    }
}

/// Preserve declaration order and every field, including fields whose meaning
/// is unavailable. Cyclic dependencies stay abstract rather than unfolding.
pub(crate) fn bind(types: &mut Types, shapes: &BTreeMap<String, Vec<(String, RustType)>>) {
    let shapes: BTreeMap<_, _> = shapes
        .iter()
        .filter(|(name, fields)| {
            types.outside.contains_key(*name) && fields.iter().all(|(name, _)| !name.is_empty())
        })
        .collect();
    let graph: BTreeMap<String, BTreeSet<String>> = shapes
        .iter()
        .map(|(name, fields)| {
            let deps = fields
                .iter()
                .flat_map(|(_, t)| t.paths())
                .map(|p| types.resolve(&p))
                .filter(|p| shapes.contains_key(p))
                .collect();
            ((*name).clone(), deps)
        })
        .collect();
    let families = lift::strong_components(&graph);
    let mut pending: BTreeSet<String> = graph.keys().cloned().collect();
    while let Some(name) = pending
        .iter()
        .find(|name| {
            let family = families.iter().find(|f| f.contains(*name));
            graph[*name]
                .iter()
                .all(|dep| !pending.contains(dep) || family.is_some_and(|f| f.contains(dep)))
        })
        .cloned()
    {
        let family = families.iter().find(|f| f.contains(&name));
        let fields = shapes[&name];
        let mut view = Vec::new();
        for (index, (field, written)) in fields.iter().enumerate() {
            let recursive = written
                .paths()
                .iter()
                .any(|p| family.is_some_and(|f| f.contains(&types.resolve(p))));
            let reading = (!recursive && !borrows_mutably(written))
                .then(|| super::lower::ty(&written.text, types).ok())
                .flatten();
            let ty = reading.unwrap_or_else(|| {
                // Bind opaque slot identities to their actual source reading.
                let identity = crate::project::sha(
                    format!("{name}\0{index}\0{field}\0{written:?}").as_bytes(),
                );
                Ty::Param(format!("OpaqueField{identity}"))
            });
            view.push(ty);
        }
        types.native_views.insert(name.clone(), Ty::Pair(view));
        pending.remove(&name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn written(text: &str) -> RustType {
        crate::convert::ty(&syn::parse_str(text).unwrap())
    }

    #[test]
    fn views_follow_shapes_and_keep_unknown_fields_ue_69() {
        assert!(borrows_mutably(&written("Option<&mut bool>")));
        assert!(!borrows_mutably(&written("Option<&bool>")));
        let mut types = Types::default();
        for n in ["Inner", "Outer", "Cycle", "Incomplete"] {
            types.outside.insert(n.into(), "not source modeled".into());
        }
        let shapes = BTreeMap::from([
            (
                "Inner".into(),
                vec![
                    ("flag".into(), written("bool")),
                    ("service".into(), written("Box<dyn Service>")),
                ],
            ),
            (
                "Outer".into(),
                vec![
                    ("inner".into(), written("Inner")),
                    ("word".into(), written("u8")),
                ],
            ),
            (
                "Cycle".into(),
                vec![
                    ("next".into(), written("Option<Box<Cycle>>")),
                    ("flag".into(), written("bool")),
                ],
            ),
            (
                "Incomplete".into(),
                vec![lift::unread_shape(lift::PARAMETRIC_STRUCT)],
            ),
        ]);
        bind(&mut types, &shapes);
        let inner = types.native_views["Inner"].clone();
        assert!(
            matches!(&inner, Ty::Pair(ts) if ts.len()==2 && ts[0]==Ty::Bool && matches!(ts[1],Ty::Param(_)))
        );
        assert_eq!(
            types.native_views["Outer"],
            Ty::Pair(vec![inner.clone(), Ty::Nat(Some(8))])
        );
        assert!(
            matches!(&types.native_views["Cycle"], Ty::Pair(ts) if matches!(ts[0],Ty::Param(_)) && ts[1]==Ty::Bool)
        );
        assert!(!types.native_views.contains_key("Incomplete"));
        assert!(touches("&Outer", &types));
        assert!(!touches("&bool", &types));
        let mut changed = shapes.clone();
        changed.get_mut("Inner").unwrap()[1].1 = written("Box<dyn AnotherService>");
        types.native_views.clear();
        bind(&mut types, &changed);
        assert_ne!(
            types.native_views["Inner"], inner,
            "opaque reading changed without its identity"
        );
    }
}
