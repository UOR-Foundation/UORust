//! Width-derived boundaries between mathematical source integers and native words.
use crate::lex::{Decl, Expr};
use crate::lift::Ty;

pub(super) struct Converted {
    pub(super) value: String,
    /// Ordered range obligations. Later checks use the bounds established
    /// by earlier ones; every check belongs to the same native candidate.
    pub(super) valid: Vec<String>,
    pub(super) declarations: Vec<Decl>,
}

/// Decode a native bit pattern into SDK Int64 using only total fixed-width
/// bit operations. Balanced composition keeps the semantic JSON depth bounded.
fn signed_source(value: &str, bits: u32) -> Converted {
    let modulus = super::int::pow2(bits);
    let half = super::int::pow2(bits - 1);
    let mut parts = vec![if bits < 64 {
        format!("(if (codecWord <? {half}) then (0 : Int64) else (-{modulus} : Int64))")
    } else {
        "(0 : Int64)".into()
    }];
    for bit in 0..bits {
        let divisor = super::int::pow2(bit);
        let mask = if bit == 63 {
            i64::MIN.to_string()
        } else {
            (1u64 << bit).to_string()
        };
        parts.push(format!("(if ((prim remainder (prim quotient codecWord {divisor} 0 : Nat) 2 0 : Nat) =? 0) then (0 : Int64) else ({mask} : Int64))"));
    }
    while parts.len() > 1 {
        parts = parts
            .chunks(2)
            .map(|pair| match pair {
                [a, b] => format!("(prim bit_or {a} {b} : Int64)"),
                [a] => a.clone(),
                _ => unreachable!(),
            })
            .collect();
    }
    let body = format!(
        "(let codecWord : Nat := (prim remainder raw {modulus} 0 : Nat); {})",
        parts[0]
    );
    let name = format!(
        "mirSignedSource{}",
        crate::project::sha(format!("{bits}\0{body}").as_bytes())
    );
    Converted {
        value: format!("({name} {value})"),
        valid: Vec::new(),
        declarations: vec![Decl::Definition {
            name,
            type_parameters: Vec::new(),
            parameters: vec![("raw".into(), "Nat".into())],
            result: "Int64".into(),
            body: Expr::Flat(body),
            axioms: vec!["Quot.sound".into(), "propext".into()],
            recursive: None,
            termination: None,
            mutual: None,
        }],
    }
}

fn source_width(ty: &Ty, widths: &[(u32, Vec<String>)]) -> Option<u32> {
    match ty {
        Ty::Integer(Some(bits)) => Some(*bits),
        Ty::Integer(None) => Some(widths.last().map_or(64, |(bits, _)| *bits)),
        _ => None,
    }
}

fn nonnegative(value: &str) -> String {
    format!("((not (prim less_than {value} (0 : Int) : Bool)) = true)")
}

fn range(value: &str, limit: &str) -> String {
    format!("(((not (prim less_than {value} (0 : Int) : Bool)) && (prim less_than {value} ({limit} : Int) : Bool)) = true)")
}

fn under(hypotheses: &[String], goal: String) -> String {
    hypotheses
        .iter()
        .rev()
        .fold(goal, |body, hyp| format!("({hyp} -> {body})"))
}

pub(super) fn scalar(
    value: &str,
    from: &Ty,
    to: &Ty,
    widths: &[(u32, Vec<String>)],
) -> Option<Converted> {
    if from.same(to) {
        return Some(Converted {
            value: value.into(),
            valid: Vec::new(),
            declarations: Vec::new(),
        });
    }
    if let (Ty::Signed(Some(bits)), Ty::Int(target)) = (from, to) {
        return (bits == target && [8, 16, 32, 64].contains(bits))
            .then(|| signed_source(value, *bits));
    }
    // Source fixed-width signed values share Int64 storage. Normalize
    // through the SDK's exact conversion, then reuse the same word codec
    // as mathematical source integers. The declared Rust width still binds
    // the conversion; this does not admit the reverse Int64 boundary.
    if let Ty::Int(bits) = from {
        return scalar(
            &format!("(prim to_int {value} : Int)"),
            &Ty::Integer(Some(*bits)),
            to,
            widths,
        );
    }
    if let Ty::Signed(_) = from {
        let word = super::int::Int::of(from, widths)?;
        if source_width(to, widths)? != word.held {
            return None;
        }
        return Some(Converted {
            value: crate::registered::word_integer(value, word.held, true),
            valid: Vec::new(),
            declarations: Vec::new(),
        });
    }
    let Ty::Signed(_) = to else {
        return None;
    };
    let word = super::int::Int::of(to, widths)?;
    if source_width(from, widths)? != word.held || word.held > 128 {
        return None;
    }
    let modulus = super::int::pow2(word.held);
    let limb = super::int::pow2(64);
    let negative = format!("((prim less_than {value} (0 : Int) : Bool) = true)");
    let mut valid = Vec::new();
    let mut branch = |magnitude: String, premise: String| {
        let positive = nonnegative(&magnitude);
        valid.push(under(std::slice::from_ref(&premise), positive.clone()));
        let residue = format!("(prim remainder {magnitude} ({modulus} : Int) (0 : Int) : Int)");
        let bounded = range(&residue, &modulus);
        valid.push(under(&[premise.clone(), positive], bounded.clone()));
        let hyps = [premise, bounded];
        let low = if word.held <= 64 {
            residue.clone()
        } else {
            format!("(prim remainder {residue} ({limb} : Int) (0 : Int) : Int)")
        };
        valid.push(under(&hyps, range(&low, &limb)));
        let low = format!("(prim checked_convert {low} : (Option UInt64))");
        if word.held <= 64 {
            format!("(match {low} | Option.none => 0 | Option.some lowWord => (prim to_nat lowWord : Nat))")
        } else {
            let high = format!("(prim quotient {residue} ({limb} : Int) (0 : Int) : Int)");
            valid.push(under(&hyps, range(&high, &limb)));
            let high = format!("(prim checked_convert {high} : (Option UInt64))");
            format!("(match {low} | Option.none => 0 | Option.some lowWord => (match {high} | Option.none => 0 | Option.some highWord => ((prim to_nat lowWord : Nat) + (prim multiply {limb} (prim to_nat highWord : Nat) : Nat))))")
        }
    };
    let positive = branch(value.into(), nonnegative(value));
    let magnitude = branch(format!("(prim negate {value} : Int)"), negative);
    Some(Converted {value:format!("(if (prim less_than {value} (0 : Int) : Bool) then (prim remainder (prim subtract {modulus} {magnitude} : Nat) {modulus} 0 : Nat) else {positive})"),valid,declarations: Vec::new()})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_codecs_refuse_unrelated_or_mismatched_slots_ue_56() {
        let widths = [(64, Vec::new())];
        for (from, to) in [
            (Ty::Bool, Ty::Signed(Some(128))),
            (Ty::Int(32), Ty::Signed(Some(64))),
            (Ty::Signed(Some(32)), Ty::Int(64)),
            (Ty::Signed(Some(64)), Ty::Integer(Some(128))),
            (Ty::Integer(Some(128)), Ty::Signed(Some(64))),
        ] {
            assert!(scalar("value", &from, &to, &widths).is_none());
        }
    }
}
