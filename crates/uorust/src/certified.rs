//! The certified translator's adapters, written by `uorust certify sync`
//! (`just certified`): for each certified root whose parameter and result
//! types convert, a function over references to the hand-written one's
//! arguments that runs the certified package (crates/certified) on them. A
//! hand-written function runs its certified realization through
//! [`certified!`], its first statement; a site whose root this commit does
//! not certify has a stub that answers `None`, so the hand-written body runs.
//! `--translator hand` runs every hand-written body, the differential's second
//! path. A certified root that overflows its target's width stops with a
//! panic, which the differential sees as a disagreement.

#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, Ordering};

static HAND: AtomicBool = AtomicBool::new(false);

/// Run the hand-written translator in place of the certified one: the
/// differential's second path.
pub fn use_hand(hand: bool) {
    HAND.store(hand, Ordering::Relaxed);
}

/// Whether the hand-written translator runs.
pub(crate) fn hand() -> bool {
    HAND.load(Ordering::Relaxed)
}

/// `certified!(adapter, args...)`, a hand-written function's first
/// statement under `#[cfg(not(uorust_lift))]`: return what the adapter
/// answers, unless the hand-written translator runs. The lift's dump sets
/// `uorust_lift`, so the self register lifts the hand-written body alone.
macro_rules! certified {
    ($adapter:ident $(, $arg:expr)* $(,)?) => {
        if !$crate::certified::hand() {
            if let Some(v) = $crate::certified::$adapter($(&$arg),*) {
                return v;
            }
        }
    };
}
pub(crate) use certified;

/// `uorust::lex::members`, certified as `Behavior::members`.
#[allow(clippy::all)]
pub(crate) fn lex_members(
    p0: &&str,
    p1: &&[std::string::String],
    p2: &&[std::string::String],
    p3: &bool,
) -> Option<std::string::String> {
    Some(
        uorust_certified::members::run(
            uorust_certified::members::Str::lit((*p0)),
            {
                let mut l = uorust_certified::members::List::nil();
                for x in (*p1).iter().rev() {
                    l = uorust_certified::members::List::cons(
                        uorust_certified::members::Str::lit(x),
                        l,
                    );
                }
                l
            },
            {
                let mut l = uorust_certified::members::List::nil();
                for x in (*p2).iter().rev() {
                    l = uorust_certified::members::List::cons(
                        uorust_certified::members::Str::lit(x),
                        l,
                    );
                }
                l
            },
            *p3,
        )
        .text()
        .to_string(),
    )
}

/// `uorust::lift::Types::accessor`, certified as `Behavior::typesAccessor`.
#[allow(clippy::all)]
pub(crate) fn lift_Types_accessor(p0: &&str, p1: &&str) -> Option<std::string::String> {
    Some(
        uorust_certified::types_accessor::run(
            uorust_certified::types_accessor::Str::lit((*p0)),
            uorust_certified::types_accessor::Str::lit((*p1)),
        )
        .text()
        .to_string(),
    )
}

/// `uorust::lift::equality_name`, certified as `Behavior::equalityName`.
#[allow(clippy::all)]
pub(crate) fn lift_equality_name(p0: &&str) -> Option<std::string::String> {
    Some(
        uorust_certified::equality_name::run(uorust_certified::equality_name::Str::lit((*p0)))
            .text()
            .to_string(),
    )
}

/// `uorust::lift::evidence_axioms`, certified as `Behavior::evidenceAxioms`.
#[allow(clippy::all)]
pub(crate) fn lift_evidence_axioms() -> Option<std::vec::Vec<std::string::String>> {
    Some({
        let mut v = Vec::new();
        let mut c = uorust_certified::evidence_axioms::run();
        while let Some((h, t)) = c.uncons() {
            v.push(h.text().to_string());
            c = t;
        }
        v
    })
}

/// `uorust::lift::evidence_proof`, certified as `Behavior::evidenceProof`.
#[allow(clippy::all)]
pub(crate) fn lift_evidence_proof(p0: &&[std::string::String]) -> Option<std::string::String> {
    Some(
        uorust_certified::evidence_proof::run({
            let mut l = uorust_certified::evidence_proof::List::nil();
            for x in (*p0).iter().rev() {
                l = uorust_certified::evidence_proof::List::cons(
                    uorust_certified::evidence_proof::Str::lit(x),
                    l,
                );
            }
            l
        })
        .text()
        .to_string(),
    )
}

/// `uorust::lift::first_free`, certified as `Behavior::firstFree`.
#[allow(clippy::all)]
pub(crate) fn lift_first_free(
    p0: &&str,
    p1: &&[std::string::String],
) -> Option<std::string::String> {
    Some(
        match uorust_certified::first_free::run(uorust_certified::first_free::Str::lit((*p0)), {
            let mut l = uorust_certified::first_free::List::nil();
            for x in (*p1).iter().rev() {
                l = uorust_certified::first_free::List::cons(
                    uorust_certified::first_free::Str::lit(x),
                    l,
                );
            }
            l
        }) {
            Ok(v) => v,
            Err(_) => panic!("uorust::lift::first_free: the certified root overflowed"),
        }
        .text()
        .to_string(),
    )
}

/// `uorust::lift::mentions_word`, certified as `Behavior::mentionsWord`.
#[allow(clippy::all)]
pub(crate) fn lift_mentions_word(p0: &&str, p1: &&str) -> Option<bool> {
    Some(
        match uorust_certified::mentions_word::run(
            uorust_certified::mentions_word::Str::lit((*p0)),
            uorust_certified::mentions_word::Str::lit((*p1)),
        ) {
            Ok(v) => v,
            Err(_) => panic!("uorust::lift::mentions_word: the certified root overflowed"),
        },
    )
}

/// `uorust::lift::pointer_width`, certified as `Behavior::pointerWidth`.
#[allow(clippy::all)]
pub(crate) fn lift_pointer_width(p0: &&str) -> Option<std::option::Option<u32>> {
    Some(match uorust_certified::pointer_width::run(uorust_certified::pointer_width::Str::lit((*p0))) { Ok(v) => v, Err(_) => panic!("uorust::lift::pointer_width: the certified root overflowed") }.map(|x| u32::try_from(x).expect("a natural the hand-written type holds")))
}

/// `uorust::lift::unicode_property`, certified as `Behavior::unicodeProperty`.
#[allow(clippy::all)]
pub(crate) fn lift_unicode_property(p0: &&str) -> Option<std::option::Option<std::string::String>> {
    Some(
        uorust_certified::unicode_property::run(uorust_certified::unicode_property::Str::lit(
            (*p0),
        ))
        .map(|x| x.text().to_string()),
    )
}

/// `uorust::mir::lower::address`, certified as `Behavior::address`.
#[allow(clippy::all)]
pub(crate) fn mir_lower_address(p0: &&str) -> Option<std::string::String> {
    Some(
        uorust_certified::address::run(uorust_certified::address::Str::lit((*p0)))
            .text()
            .to_string(),
    )
}

/// `uorust::mir::lower::decoded`, certified as `Behavior::decoded`.
#[allow(clippy::all)]
pub(crate) fn mir_lower_decoded(p0: &&str) -> Option<std::string::String> {
    Some(
        uorust_certified::decoded::run(uorust_certified::decoded::Str::lit((*p0)))
            .text()
            .to_string(),
    )
}

/// `uorust::mir::lower::implication`, certified as `Behavior::implication`.
#[allow(clippy::all)]
pub(crate) fn mir_lower_implication(
    p0: &&[std::string::String],
    p1: &&str,
) -> Option<std::string::String> {
    Some(
        uorust_certified::implication::run(
            {
                let mut l = uorust_certified::implication::List::nil();
                for x in (*p0).iter().rev() {
                    l = uorust_certified::implication::List::cons(
                        uorust_certified::implication::Str::lit(x),
                        l,
                    );
                }
                l
            },
            uorust_certified::implication::Str::lit((*p1)),
        )
        .text()
        .to_string(),
    )
}

/// `uorust::mir::lower::is_const_argument`, certified as `Behavior::isConstArgument`.
#[allow(clippy::all)]
pub(crate) fn mir_lower_is_const_argument(p0: &&str) -> Option<bool> {
    Some(
        match uorust_certified::is_const_argument::run(
            uorust_certified::is_const_argument::Str::lit((*p0)),
        ) {
            Ok(v) => v,
            Err(_) => {
                panic!("uorust::mir::lower::is_const_argument: the certified root overflowed")
            }
        },
    )
}

/// `uorust::mir::lower::pointer_value`, certified as `Behavior::pointerValue`.
#[allow(clippy::all)]
pub(crate) fn mir_lower_pointer_value(
    p0: &&str,
    p1: &&str,
    p2: &&str,
) -> Option<std::string::String> {
    Some(
        uorust_certified::pointer_value::run(
            uorust_certified::pointer_value::Str::lit((*p0)),
            uorust_certified::pointer_value::Str::lit((*p1)),
            uorust_certified::pointer_value::Str::lit((*p2)),
        )
        .text()
        .to_string(),
    )
}

/// `uorust::mir::lower::referent`, certified as `Behavior::referent`.
#[allow(clippy::all)]
pub(crate) fn mir_lower_referent(p0: &&str) -> Option<std::option::Option<std::string::String>> {
    Some(
        match uorust_certified::referent::run(uorust_certified::referent::Str::lit((*p0))) {
            Ok(v) => v,
            Err(_) => panic!("uorust::mir::lower::referent: the certified root overflowed"),
        }
        .map(|x| x.text().to_string()),
    )
}

/// `uorust::mir::lower::scalar_value`, certified as `Behavior::scalarValue`.
#[allow(clippy::all)]
pub(crate) fn mir_lower_scalar_value(p0: &&str) -> Option<std::string::String> {
    Some(
        uorust_certified::scalar_value::run(uorust_certified::scalar_value::Str::lit((*p0)))
            .text()
            .to_string(),
    )
}

/// `uorust::mir::lower::uninhabited`, certified as `Behavior::uninhabited`.
#[allow(clippy::all)]
pub(crate) fn mir_lower_uninhabited(p0: &&str) -> Option<bool> {
    Some(
        match uorust_certified::uninhabited::run(uorust_certified::uninhabited::Str::lit((*p0))) {
            Ok(v) => v,
            Err(_) => panic!("uorust::mir::lower::uninhabited: the certified root overflowed"),
        },
    )
}

/// `uorust::mir::memory::axioms`, certified as `Behavior::mirMemoryAxioms`.
#[allow(clippy::all)]
pub(crate) fn mir_memory_axioms() -> Option<std::vec::Vec<std::string::String>> {
    Some({
        let mut v = Vec::new();
        let mut c = uorust_certified::mir_memory_axioms::run();
        while let Some((h, t)) = c.uncons() {
            v.push(h.text().to_string());
            c = t;
        }
        v
    })
}

/// `uorust::mir::memory::of`, certified as `Behavior::of`.
#[allow(clippy::all)]
pub(crate) fn mir_memory_of(p0: &&str) -> Option<std::string::String> {
    Some(
        uorust_certified::of::run(uorust_certified::of::Str::lit((*p0)))
            .text()
            .to_string(),
    )
}

/// Not certified at this commit: the hand-written body runs.
pub(crate) fn f<A0: ?Sized, R>(_: &A0) -> Option<R> {
    None
}

#[cfg(test)]
mod tests {
    // The switch the differential runs the hand-written translator through.
    #[test]
    fn use_hand_selects_the_hand_written_translator() {
        super::use_hand(true);
        assert!(super::hand());
        super::use_hand(false);
        assert!(!super::hand());
    }
}
