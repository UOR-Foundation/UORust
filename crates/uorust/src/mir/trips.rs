//! How many times each block of a body runs at most, from its loops' exits
//! (UE-19): the differential check's fuel bound.
//!
//! A loop is a back edge's natural loop. Its trips are bounded by an exit
//! whose condition the loop's own writes drive to it: a counter that every
//! trip makes larger, while it stays below a constant (`while i < 256`, the
//! loop a `const fn` fills a table with), runs at most that constant's
//! trips; one that every trip halves (`>>= k`, `/= c`), while it stays
//! above zero, at most its width's. A loop no exit bounds so has no count,
//! and the reason is said: the check does not run a sample it cannot bound.

use std::collections::{BTreeMap, BTreeSet};

use super::ir::{IrBody, IrConstant, IrOperand, IrPlace, IrRvalue, IrStatement, IrTerminator};
use super::spec::MirBinOp as B;

/// The most runs of each block per entry to the body, or why a loop has
/// no bound.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Runs {
    Bounded(Vec<u128>),
    Unbounded(String),
}

/// The runs of `b`'s blocks, of those `reached` (its blocks control reaches
/// without unwinding), on a target of `width`-bit pointers.
pub(crate) fn runs(b: &IrBody, reached: &BTreeSet<usize>, width: u32) -> Runs {
    let succ = |i: usize| -> Vec<usize> {
        b.blocks
            .get(i)
            .map(|block| block.terminator.successors())
            .unwrap_or_default()
            .into_iter()
            .filter(|n| reached.contains(n))
            .collect()
    };
    // Back edges: an edge to a block on the depth-first path to it.
    let mut back: Vec<(usize, usize)> = Vec::new();
    let mut state: BTreeMap<usize, bool> = BTreeMap::new(); // true: on the path
    let mut stack: Vec<(usize, Vec<usize>)> = vec![(0, succ(0))];
    state.insert(0, true);
    while let Some((at, next)) = stack.last_mut() {
        let at = *at;
        match next.pop() {
            Some(n) => match state.get(&n) {
                Some(true) => back.push((at, n)),
                Some(false) => {}
                None => {
                    state.insert(n, true);
                    stack.push((n, succ(n)));
                }
            },
            None => {
                state.insert(at, false);
                stack.pop();
            }
        }
    }
    // Each header's natural loop: the header and every block that reaches
    // one of its latches without passing it.
    let mut preds: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for &i in reached {
        for n in succ(i) {
            preds.entry(n).or_default().push(i);
        }
    }
    let mut loops: BTreeMap<usize, (BTreeSet<usize>, BTreeSet<usize>)> = BTreeMap::new();
    for (latch, header) in back {
        let (body, latches) = loops.entry(header).or_default();
        latches.insert(latch);
        body.insert(header);
        let mut work = vec![latch];
        while let Some(x) = work.pop() {
            if body.insert(x) {
                work.extend(preds.get(&x).into_iter().flatten().copied());
            }
        }
    }
    let mut runs = vec![1u128; b.blocks.len()];
    for (header, (body, latches)) in &loops {
        // A loop entered only through its head: each entry to it is one
        // run of the blocks around it, which bound its entries.
        let entered_elsewhere = body.iter().any(|i| {
            i != header
                && preds
                    .get(i)
                    .is_some_and(|ps| ps.iter().any(|p| !body.contains(p)))
        });
        if entered_elsewhere {
            return Runs::Unbounded(format!(
                "a loop (at bb{header}) is entered other than through its head"
            ));
        }
        match trips(b, *header, body, latches, width) {
            Some(n) => {
                for &i in body {
                    runs[i] = runs[i].saturating_mul(n.saturating_add(1));
                }
            }
            None => {
                return Runs::Unbounded(format!(
                    "a loop (at bb{header}) exits on no counter that every trip grows below a constant or halves above one"
                ))
            }
        }
    }
    Runs::Bounded(runs)
}

/// A local read whole.
fn local_of(o: &IrOperand) -> Option<usize> {
    match o {
        IrOperand::Copy(p) | IrOperand::Move(p) if p.projection.is_empty() => Some(p.local),
        _ => None,
    }
}

fn int_of(o: &IrOperand) -> Option<i128> {
    match o {
        IrOperand::Constant(IrConstant::Int(n, _)) => Some(*n),
        _ => None,
    }
}

/// The width of an unsigned integer type, as MIR prints it.
fn unsigned_bits(ty: &str, width: u32) -> Option<u32> {
    match ty {
        "usize" => Some(width),
        _ => ty.strip_prefix('u')?.parse().ok(),
    }
}

/// How a trip writes a counter.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Step {
    Grows,
    Halves,
}

/// The most trips of the loop at `header` (blocks `body`, back edges from
/// `latches`): the least bound of its exits, or none where no exit bounds
/// it.
fn trips(
    b: &IrBody,
    header: usize,
    body: &BTreeSet<usize>,
    latches: &BTreeSet<usize>,
    width: u32,
) -> Option<u128> {
    let assignments = |local: usize| -> Vec<(usize, &IrRvalue)> {
        body.iter()
            .flat_map(|&i| {
                b.blocks[i].statements.iter().filter_map(move |s| match s {
                    IrStatement::Assign(
                        IrPlace {
                            local: l,
                            projection,
                        },
                        rv,
                    ) if *l == local && projection.is_empty() => Some((i, rv)),
                    _ => None,
                })
            })
            .collect()
    };
    // How every write of `counter` in the loop steps it, where each is one
    // step of one kind, and the blocks that write it.
    // A counter a call returns into, or a borrow reaches, is written other
    // than by the loop's steps.
    let aliased = |counter: usize| -> bool {
        b.blocks.iter().any(|block| {
            block.statements.iter().any(|s| {
                matches!(s, IrStatement::Assign(_, IrRvalue::Ref(_, p) | IrRvalue::RawPtr(_, p))
                    if p.local == counter)
            }) || matches!(&block.terminator, IrTerminator::Call { destination, .. }
                if destination.local == counter)
        })
    };
    // How every write of `counter` in the loop steps it, where each is one
    // step of one kind, the blocks that write it, and the most a growing
    // step adds.
    let steps = |counter: usize| -> Option<(Step, BTreeSet<usize>, i128)> {
        if aliased(counter) {
            return None;
        }
        let writes = assignments(counter);
        let mut kind: Option<Step> = None;
        let mut blocks = BTreeSet::new();
        let mut most = 0i128;
        for (i, rv) in writes {
            let step = match rv {
                IrRvalue::Binary(B::Add | B::AddUnchecked, x, k)
                    if local_of(x) == Some(counter) && int_of(k).is_some_and(|k| k >= 1) =>
                {
                    most = most.max(int_of(k).unwrap_or(0));
                    Step::Grows
                }
                IrRvalue::Binary(B::Shr | B::ShrUnchecked, x, k)
                    if local_of(x) == Some(counter) && int_of(k).is_some_and(|k| k >= 1) =>
                {
                    Step::Halves
                }
                IrRvalue::Binary(B::Div, x, c)
                    if local_of(x) == Some(counter) && int_of(c).is_some_and(|c| c >= 2) =>
                {
                    Step::Halves
                }
                // `t = AddWithOverflow(i, k); .. i = move (t.0)`.
                IrRvalue::Use(IrOperand::Move(p) | IrOperand::Copy(p))
                    if p.projection.len() == 1 =>
                {
                    let checked = assignments(p.local).into_iter().all(|(_, rv)| {
                        matches!(rv, IrRvalue::Binary(B::AddWithOverflow, x, k)
                            if local_of(x) == Some(counter) && int_of(k).is_some_and(|k| k >= 1))
                    }) && !assignments(p.local).is_empty();
                    if !checked {
                        return None;
                    }
                    // An overflowing add panics: it never wraps below.
                    Step::Grows
                }
                _ => return None,
            };
            if kind.is_some_and(|k| k != step) {
                return None;
            }
            kind = Some(step);
            blocks.insert(i);
        }
        Some((kind?, blocks, most))
    };
    // Whether every trip passes a block of `writes`: no cycle through the
    // header avoids them.
    let every_trip = |writes: &BTreeSet<usize>| -> bool {
        if writes.contains(&header) {
            return true;
        }
        let mut seen = BTreeSet::new();
        let mut work: Vec<usize> = b.blocks[header]
            .terminator
            .successors()
            .into_iter()
            .filter(|n| body.contains(n))
            .collect();
        while let Some(x) = work.pop() {
            if writes.contains(&x) || !seen.insert(x) {
                continue;
            }
            if latches.contains(&x) || x == header {
                return false;
            }
            work.extend(
                b.blocks[x]
                    .terminator
                    .successors()
                    .into_iter()
                    .filter(|n| body.contains(n) && *n != header),
            );
        }
        // A latch is reached only through a write.
        !latches.iter().any(|l| seen.contains(l))
    };
    let mut best: Option<u128> = None;
    for &x in body {
        let IrTerminator::Switch(op, arms, otherwise) = &b.blocks[x].terminator else {
            continue;
        };
        let inside = |t: &usize| body.contains(t);
        // A boolean exit: one arm (`0`, false) and `otherwise` (true), one
        // of them leaving the loop.
        let [(0, on_false)] = arms.as_slice() else {
            continue;
        };
        let continues_when = match (inside(on_false), inside(otherwise)) {
            (false, true) => true,
            (true, false) => false,
            _ => continue,
        };
        let Some(c) = local_of(op) else { continue };
        let statements = &b.blocks[x].statements;
        let writes = |s: &IrStatement, local: usize| {
            matches!(s, IrStatement::Assign(IrPlace { local: l, projection }, _)
                if *l == local && projection.is_empty())
        };
        let Some(j) = statements.iter().rposition(|s| writes(s, c)) else {
            continue;
        };
        let IrStatement::Assign(_, IrRvalue::Binary(cmp, l, r)) = &statements[j] else {
            continue;
        };
        // A compared local the block copies another into, as MIR compares
        // a temporary: the local it copies, unwritten since.
        let origin = |t: usize| -> usize {
            let Some(k) = statements[..j].iter().rposition(|s| writes(s, t)) else {
                return t;
            };
            match &statements[k] {
                IrStatement::Assign(_, IrRvalue::Use(IrOperand::Copy(p) | IrOperand::Move(p)))
                    if p.projection.is_empty()
                        && !statements[k + 1..j].iter().any(|s| writes(s, p.local)) =>
                {
                    p.local
                }
                _ => t,
            }
        };
        let (l, r) = (local_of(l).map(origin), local_of(r).map(origin));
        let (li, ri) = match &statements[j] {
            IrStatement::Assign(_, IrRvalue::Binary(_, a, b)) => (int_of(a), int_of(b)),
            _ => continue,
        };
        // The condition the loop continues under, `counter REL constant`.
        let (counter, rel, n) = match (l, ri, li, r) {
            (Some(i), Some(n), _, _) => (i, *cmp, n),
            (_, _, Some(n), Some(i)) => (
                i,
                match cmp {
                    B::Lt => B::Gt,
                    B::Le => B::Ge,
                    B::Gt => B::Lt,
                    B::Ge => B::Le,
                    other => *other,
                },
                n,
            ),
            _ => continue,
        };
        let rel = if continues_when {
            rel
        } else {
            match rel {
                B::Lt => B::Ge,
                B::Le => B::Gt,
                B::Gt => B::Le,
                B::Ge => B::Lt,
                B::Eq => B::Ne,
                B::Ne => B::Eq,
                other => other,
            }
        };
        let Some(bits) = b.locals.get(counter).and_then(|t| unsigned_bits(t, width)) else {
            continue;
        };
        let Some((step, writes, most)) = steps(counter) else {
            continue;
        };
        // Every trip writes the counter and tests it.
        if !every_trip(&writes) || !every_trip(&BTreeSet::from([x])) {
            continue;
        }
        // A counter below `n` grown by at most `most` stays below its
        // type's end, so it never wraps back below `n`.
        let end = 1u128.checked_shl(bits).unwrap_or(u128::MAX);
        let no_wrap =
            |last: i128| u128::try_from(last.saturating_add(most)).is_ok_and(|top| top < end);
        let bound = match (step, rel) {
            (Step::Grows, B::Lt) if n >= 1 && no_wrap(n - 1) => Some(n as u128),
            (Step::Grows, B::Le) if n >= 0 && no_wrap(n) => Some(n as u128 + 1),
            // Halving reaches zero within its width's steps and stays.
            (Step::Halves, B::Gt) if n >= 0 => Some(u128::from(bits) + 1),
            (Step::Halves, B::Ge) if n >= 1 => Some(u128::from(bits) + 1),
            (Step::Halves, B::Ne) if n == 0 => Some(u128::from(bits) + 1),
            _ => None,
        };
        if let Some(t) = bound {
            best = Some(best.map_or(t, |b| b.min(t)));
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(text: &str) -> IrBody {
        crate::mir::parse::bodies(text).into_iter().next().unwrap()
    }

    fn all(b: &IrBody) -> BTreeSet<usize> {
        (0..b.blocks.len()).collect()
    }

    /// A `const fn` table loop runs its constant's trips.
    #[test]
    fn a_counter_below_a_constant_bounds_its_loop() {
        let b = body(
            "fn f() -> usize {\n    let mut _0: usize;\n    let mut _1: usize;\n    let mut _2: bool;\n\n    bb0: {\n        _1 = const 0_usize;\n        goto -> bb1;\n    }\n\n    bb1: {\n        _2 = Lt(copy _1, const 256_usize);\n        switchInt(move _2) -> [0: bb3, otherwise: bb2];\n    }\n\n    bb2: {\n        _1 = Add(copy _1, const 1_usize);\n        goto -> bb1;\n    }\n\n    bb3: {\n        _0 = copy _1;\n        return;\n    }\n}\n",
        );
        assert_eq!(runs(&b, &all(&b), 64), Runs::Bounded(vec![1, 257, 257, 1]));
    }

    /// A counter compared through the temporary MIR copies it into, as
    /// `while i < 256` compiles.
    #[test]
    fn a_counter_compared_through_a_copy_bounds_its_loop() {
        let b = body(
            "fn f() -> usize {\n    let mut _0: usize;\n    let mut _1: usize;\n    let mut _2: bool;\n    let mut _3: usize;\n\n    bb0: {\n        _1 = const 0_usize;\n        goto -> bb1;\n    }\n\n    bb1: {\n        _3 = copy _1;\n        _2 = Lt(move _3, const 256_usize);\n        switchInt(move _2) -> [0: bb3, otherwise: bb2];\n    }\n\n    bb2: {\n        _1 = Add(copy _1, const 1_usize);\n        goto -> bb1;\n    }\n\n    bb3: {\n        _0 = copy _1;\n        return;\n    }\n}\n",
        );
        assert_eq!(runs(&b, &all(&b), 64), Runs::Bounded(vec![1, 257, 257, 1]));
    }

    /// A counter halved while above zero runs at most its width's trips.
    #[test]
    fn a_halved_counter_bounds_its_loop_by_its_width() {
        let b = body(
            "fn f(_1: u8) -> u8 {\n    let mut _0: u8;\n    let mut _2: bool;\n\n    bb0: {\n        goto -> bb1;\n    }\n\n    bb1: {\n        _2 = Gt(copy _1, const 0_u8);\n        switchInt(move _2) -> [0: bb3, otherwise: bb2];\n    }\n\n    bb2: {\n        _1 = Shr(copy _1, const 1_i32);\n        goto -> bb1;\n    }\n\n    bb3: {\n        _0 = copy _1;\n        return;\n    }\n}\n",
        );
        assert_eq!(runs(&b, &all(&b), 64), Runs::Bounded(vec![1, 10, 10, 1]));
    }

    /// A loop whose exit its trips do not drive is said to have no bound.
    #[test]
    fn a_loop_no_exit_bounds_is_unbounded() {
        let b = body(
            "fn f(_1: u8) -> u8 {\n    let mut _0: u8;\n    let mut _2: bool;\n\n    bb0: {\n        goto -> bb1;\n    }\n\n    bb1: {\n        _2 = Lt(copy _1, const 9_u8);\n        switchInt(move _2) -> [0: bb3, otherwise: bb2];\n    }\n\n    bb2: {\n        _1 = Sub(copy _1, const 1_u8);\n        goto -> bb1;\n    }\n\n    bb3: {\n        _0 = copy _1;\n        return;\n    }\n}\n",
        );
        assert!(matches!(runs(&b, &all(&b), 64), Runs::Unbounded(_)));
    }
}
