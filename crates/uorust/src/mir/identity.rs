//! Import compiler-qualified calls through existing Rust type identities.
use crate::names::TypeIdentity;
use quote::ToTokens;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub(crate) struct Method {
    owner: TypeIdentity,
    trait_identity: TypeIdentity,
    name: String,
}
impl Method {
    pub(crate) fn new(
        owner: &str,
        trait_ref: &str,
        name: &str,
        read: impl Fn(&str) -> Option<TypeIdentity>,
    ) -> Option<Self> {
        Some(Self {
            owner: read(owner)?,
            trait_identity: read(trait_ref)?,
            name: name.to_string(),
        })
    }
}
pub(crate) type Bindings = BTreeMap<String, BTreeMap<String, super::lower::Callee>>;

pub(crate) fn qualified(path: &str) -> bool {
    path.starts_with('<') && path.contains(" as ")
}

/// Shared definition index; each query still carries its source type scope.
pub(crate) struct Index<'a> {
    methods: BTreeMap<String, Vec<&'a super::lower::Callee>>,
}
impl<'a> Index<'a> {
    pub(crate) fn new(callees: &'a BTreeMap<String, super::lower::Callee>) -> Self {
        let mut methods: BTreeMap<String, Vec<_>> = BTreeMap::new();
        for callee in callees.values() {
            if let Some(method) = &callee.method {
                methods.entry(method.name.clone()).or_default().push(callee);
            }
        }
        Self { methods }
    }
    pub(crate) fn resolve(
        &self,
        callee: &str,
        read: impl Fn(&str) -> Option<TypeIdentity>,
    ) -> Option<&'a super::lower::Callee> {
        let expr = syn::parse_str::<syn::ExprPath>(callee).ok()?;
        let qualified = expr.qself?;
        if qualified.position == 0 || qualified.position + 1 != expr.path.segments.len() {
            return None;
        }
        let mut path = expr.path;
        let name = path.segments.pop()?;
        if !matches!(name.arguments, syn::PathArguments::None) {
            return None;
        }
        let candidates = self.methods.get(&name.ident.to_string())?;
        let owner = read(&qualified.ty.to_token_stream().to_string())?;
        let reference = path
            .segments
            .iter()
            .map(|s| s.to_token_stream().to_string())
            .collect::<Vec<_>>()
            .join("::");
        let trait_identity = read(&reference)?;
        let mut matching = candidates.iter().copied().filter(|callee| {
            let method = callee.method.as_ref().expect("indexed method");
            method.owner.matches(&owner) && method.trait_identity.matches(&trait_identity)
        });
        let selected = matching.next()?;
        matching.next().is_none().then_some(selected)
    }
}

/// Function identities reached directly or carried as values by compiler IR.
pub(crate) fn references(body: &super::IrBody) -> std::collections::BTreeSet<String> {
    use super::ir::{IrConstant, IrOperand, IrRvalue, IrStatement, IrTerminator};
    fn operand(value: &IrOperand, out: &mut std::collections::BTreeSet<String>) {
        if let IrOperand::Constant(IrConstant::Path(path)) = value {
            out.insert(path.clone());
        }
    }
    let mut out = std::collections::BTreeSet::new();
    for block in &body.blocks {
        for statement in &block.statements {
            if let IrStatement::Assign(_, value) = statement {
                match value {
                    IrRvalue::Use(v) | IrRvalue::Unary(_, v) | IrRvalue::Cast(_, v, _) => {
                        operand(v, &mut out)
                    }
                    IrRvalue::Binary(_, a, b) => {
                        operand(a, &mut out);
                        operand(b, &mut out);
                    }
                    IrRvalue::Aggregate(_, values) => {
                        for (_, v) in values {
                            operand(v, &mut out);
                        }
                    }
                    _ => (),
                }
            }
        }
        match &block.terminator {
            IrTerminator::Call { callee, args, .. } => {
                out.insert(callee.clone());
                for arg in args {
                    operand(arg, &mut out);
                }
            }
            IrTerminator::Switch(v, _, _) | IrTerminator::Assert { condition: v, .. } => {
                operand(v, &mut out)
            }
            _ => (),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn compiler_reference_closures_include_function_values_ue_79() {
        let text="fn caller(_1: u8) -> u8 {\n    let mut _0: u8;\n    let mut _2: fn(u8) -> u8;\n\n    bb0: {\n        _2 = const <Mark as Trait<u8>>::method;\n        _0 = <Mark as Other>::method(copy _1, const <Mark as Third>::function) -> [return: bb1, unwind continue];\n    }\n\n    bb1: {\n        return;\n    }\n}\n";
        let bodies = crate::mir::parse::bodies(text);
        assert_eq!(bodies.len(), 1);
        let refs = super::references(&bodies[0]);
        assert_eq!(
            refs,
            [
                "<Mark as Trait<u8>>::method",
                "<Mark as Other>::method",
                "<Mark as Third>::function"
            ]
            .into_iter()
            .map(str::to_string)
            .collect()
        );
    }
}
