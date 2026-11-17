//! Read unconditional numeric parameter macros without evaluating macro code.
//!
//! Only a single empty matcher and a literal transcriber are admitted. Resolve
//! absolute external paths through Cargo's dependency aliases, rather than a
//! global macro-name table: another crate may export the same name.
use std::collections::BTreeMap;
use std::path::Path;

use proc_macro2::{Delimiter, TokenTree};
use serde_json::Value;
use syn::visit_mut::VisitMut;

use crate::facts::{Package, Syntax, TargetKind};

fn numeric(e: &syn::Expr) -> bool {
    match e {
        syn::Expr::Lit(l) => matches!(l.lit, syn::Lit::Int(_) | syn::Lit::Float(_)),
        syn::Expr::Unary(u) if matches!(u.op, syn::UnOp::Neg(_)) => {
            matches!(&*u.expr, syn::Expr::Lit(l) if matches!(l.lit, syn::Lit::Int(_) | syn::Lit::Float(_)))
        }
        _ => false,
    }
}

fn exported_literals(file: &syn::File) -> BTreeMap<String, syn::Expr> {
    if file
        .attrs
        .iter()
        .any(|a| a.path().is_ident("cfg") || a.path().is_ident("cfg_attr"))
    {
        return BTreeMap::new();
    }
    let mut out = BTreeMap::new();
    for item in &file.items {
        let syn::Item::Macro(m) = item else { continue };
        let Some(name) = &m.ident else { continue };
        if !m.mac.path.is_ident("macro_rules")
            || !m.attrs.iter().any(|a| a.path().is_ident("macro_export"))
            || m.attrs
                .iter()
                .any(|a| a.path().is_ident("cfg") || a.path().is_ident("cfg_attr"))
        {
            continue;
        }
        let tokens: Vec<_> = m.mac.tokens.clone().into_iter().collect();
        let [TokenTree::Group(matcher), TokenTree::Punct(eq), TokenTree::Punct(gt), TokenTree::Group(body), tail @ ..] =
            tokens.as_slice()
        else {
            continue;
        };
        if matcher.delimiter() != Delimiter::Parenthesis
            || !matcher.stream().is_empty()
            || eq.as_char() != '='
            || gt.as_char() != '>'
            || body.delimiter() != Delimiter::Brace
            || !matches!(tail, [] | [TokenTree::Punct(_)])
            || tail
                .first()
                .is_some_and(|t| !matches!(t, TokenTree::Punct(p) if p.as_char() == ';'))
        {
            continue;
        }
        if let Ok(value) = syn::parse2::<syn::Expr>(body.stream()) {
            if numeric(&value) {
                out.insert(name.to_string(), value);
            }
        }
    }
    out
}

struct Expand<'a> {
    libraries: &'a BTreeMap<String, BTreeMap<String, syn::Expr>>,
    aliases: BTreeMap<String, String>,
    changed: bool,
}

impl VisitMut for Expand<'_> {
    fn visit_expr_mut(&mut self, expr: &mut syn::Expr) {
        if let syn::Expr::Macro(m) = expr {
            let path = &m.mac.path;
            if path.leading_colon.is_some()
                && path.segments.len() == 2
                && path
                    .segments
                    .iter()
                    .all(|s| matches!(s.arguments, syn::PathArguments::None))
                && m.mac.tokens.is_empty()
            {
                let alias = path.segments[0].ident.to_string();
                let name = path.segments[1].ident.to_string();
                if let Some(value) = self
                    .aliases
                    .get(&alias)
                    .and_then(|package| self.libraries.get(package))
                    .and_then(|macros| macros.get(&name))
                {
                    *expr = value.clone();
                    self.changed = true;
                    return;
                }
            }
        }
        syn::visit_mut::visit_expr_mut(self, expr);
    }
}

pub(crate) fn expand(root: &Path, packages: &[Package], metadata: &Value, syntax: &mut Syntax) {
    let libraries: BTreeMap<_, _> = packages
        .iter()
        .filter_map(|p| {
            let target = p.targets.iter().find(|t| t.kind == TargetKind::Lib)?;
            let text = std::fs::read_to_string(root.join(&target.src)).ok()?;
            let parsed = syn::parse_file(&text).ok()?;
            Some((p.name.clone(), exported_literals(&parsed)))
        })
        .collect();
    for package in packages {
        let Some(meta) = metadata
            .get("packages")
            .and_then(Value::as_array)
            .and_then(|ps| {
                ps.iter()
                    .find(|p| p.get("name").and_then(Value::as_str) == Some(package.name.as_str()))
            })
        else {
            continue;
        };
        let aliases = meta
            .get("dependencies")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(|d| d.get("kind").is_none_or(Value::is_null))
            .filter_map(|d| {
                let name = d.get("name")?.as_str()?;
                let dependency = packages.iter().find(|p| p.name == name)?;
                let declared = Path::new(d.get("path")?.as_str()?).canonicalize().ok()?;
                if declared != root.join(&dependency.dir).canonicalize().ok()? {
                    return None;
                }
                let alias = d
                    .get("rename")
                    .and_then(Value::as_str)
                    .unwrap_or(name)
                    .replace('-', "_");
                Some((alias, name.to_string()))
            })
            .collect();
        let mut visitor = Expand {
            libraries: &libraries,
            aliases,
            changed: false,
        };
        for file in &package.files {
            let Ok(text) = std::fs::read_to_string(root.join(&file.rel)) else {
                continue;
            };
            let Ok(mut parsed) = syn::parse_file(&text) else {
                continue;
            };
            visitor.changed = false;
            visitor.visit_file_mut(&mut parsed);
            if !visitor.changed {
                continue;
            }
            let (_, mut expanded) = crate::ingest::source(
                &parsed,
                file.rel.clone(),
                file.target,
                file.module.clone(),
                &package.targets[file.target].kind,
                file.test,
            );
            let original = &syntax.files[&file.rel];
            // Constant questions, like function questions, keep their actual
            // source tokens. Expansion changes only what the lift evaluates.
            for (after, before) in expanded.consts.iter_mut().zip(&original.consts) {
                after.4.clone_from(&before.4);
            }
            syntax.files.insert(file.rel.clone(), expanded);
        }
    }
}
