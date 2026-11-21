# Plan: behavior-preserving refactor of UORust's hand-written code (v8)

Base: `daa24878` (branch `governance/qortara-template-sync`). Workflow:
Qortara `repository-refactor` 1.0.0, manifest digest as served by
`qortara_capabilities` (workflow_id `repository-refactor`):
`sha256:cd8adea18a3bd55895bcb202b4aacce88f99c975e6a632ea0de3b40b73784271`
(not stored in this repository). Governance seal: NOT_ESTABLISHED (no native
Qortara runtime; no recorded gate).

## Open questions

- Q1. `mir::lower::Fun::obligation` takes `what` and discards it
  (`crates/uorust/src/mir/lower.rs:1741`, `:1772` `let _ = what;`). Not in this
  plan until the owner says it is not a placeholder.
- Q2. `bridge::copy_tree` copies the SDK runtime without skipping `.lake`
  (`crates/uorust/src/bridge.rs:1185-1201`); recorded, not changed.

## Goal

Remove a duplicated rule, a dead arm, a shadowing name, a misleading constant
name, an independent copy of a timeout and a garbled doc from the hand-written
code, without changing the UE-01 fixture
projection, any UE-17 verdict digest, CONFORMANCE.md, any error text, any
incumbent lift status or any certified root. Generated output that may change:
`frontier.toml`, as I1 accounts for, and the self projection (`project.rs:408`,
generated from this repository's own source: its register, its measures such
as `ProductFunctions` and `Tests`, `project.rs:3034, 3115, 3131, 7138`, and the
`uorust.manifest` digests, `lib.rs:87-91`); the self projection is not a base
pin.

**Admission rule:** a candidate is applied only when every behavior it can
change is pinned against base bytes or asserted text by a check `just vv`
runs: UE-01 (`the_fixture_projects_to_the_committed_project_ue_01`, the
thermostat fixture's committed projection), the UE-17 verdict digests
(`lift-verdicts.toml`), CONFORMANCE.md, or unit/engine assertions; or it is a
doc comment / a constant whose value is unchanged and checked by the compiler.
UE-02 (two runs agree), UE-14 (drift from the hand model) and `frontier.toml`
equality compare against output the same binary writes, so they are not base
pins. A candidate with any unpinned path is withdrawn.

## Scope

Mode: **explicit** (owner-declared: the whole hand-written repository:
`xtask/`, `crates/model/`, `crates/conformance/`, `crates/uorust/src/`). The
reconnaissance of that scope produced 31 candidates; six meet the admission
rule and are listed under Phases. The rest are withdrawn:

- Unpinned paths (audits v2/v3): A1 `cell()` (authority/claim rows render
  empty here), A2/A3/A4 bootstrap trust-root parsing, A6 `rel()`, A7 R2
  lookup, B3 `combinator` (`.last()` on a list is never lifted), B5 decrease
  tail (the shrinking branch never runs), B6 axiom const (two of five sites
  unpinned), C1 quoting helper (embed/mir::core/certify call sites have no
  byte pins), C3 merged `manifest.is_file()` blocks (the removal of an earlier
  projection's files, `lib.rs:208-215`, is checked by no test), C4 `--link` parser, C6 crates.io digest, C7 lake files, C9
  library-crate list, C11 `main` IO mapping, the `bridge`/`layouts` splitter
  merge.
- Refused by a repository ratchet: B8 split of `lift_all` (five new
  uncertified `uorust::lift::` functions raise the count UE-18
  `the_hand_written_translator_only_shrinks_ue_18` refuses,
  `crates/conformance/tests/ratchet.rs:104-108`, `certify.rs:48, 3786-3810`).
- Rejected by reconnaissance (cost or contract): renaming/moving lifted
  functions (κ includes the crate path, `register.rs:55-77, 201`), splitting
  `lift.rs`, merging `try_in_*`, merging `copy_tree`s, unifying hex/sha checks
  in the bootstrap trust root, sharing Python helpers across `-I`-isolated
  scripts, CI credential scripts.
- Never in scope: generated code (`crates/certified/**`,
  `crates/uorust/src/certified.rs`, `prism-factory/**`, `CONFORMANCE.md`,
  template-owned `bootstrap/render*`); anything behind `certified!`
  (lift.rs:1814, 2034, 3396, 5298, 6038; mir/lower.rs:686, 6973); certified
  roots; `lift-verdicts.toml` rows.
- Behavior-changing defects found (reported separately, not fixed here): CM-02
  message whitespace (`crates/conformance/src/meta.rs:163`), xtask task list
  omitting `proof-reuse`/`proof-keep` (`xtask/src/main.rs:39-51`), `->`
  miscounted by the naive splitters (`bridge.rs:309`, `layouts.rs:578`).

Invariants:
- I1. `frontier.toml` regenerated with `just register`. No incumbent
  `[[lifted]]` κ is lost or turns non-lifted; every edited function keeps its
  base status; every added or removed function row is in the table below; the honesty
  header's lifted count does not change. The I1 review is
  over function rows (`[[lifted]]`, `[[residue]]`, `[[effect]]`, `[[foreign]]`,
  each keyed by a function): any function row added, removed or changing status
  other than in the table fails I1. A residue `detail` of an edited function
  may change only in its source position or the C8 rename. `[[construct]]`
  rows and the header's tallies and cause lines may change freely (counts;
  positions; `project::LEXLEAN` becoming
  `project::LEXLEAN_TOML` at frontier.toml:105, 1922, 7842); they carry no
  lift status.
  (UE-16 cannot see a lifted κ that disappears, `register.rs:537-563`, so this
  is checked by reading the diff.)
- I2. No byte change in the UE-01 fixture projection, the UE-17 verdict
  digests, CONFORMANCE.md, or any asserted error text.
- I3. Only residue/effect functions change tokens. No lifted function's tokens
  change (A5 edits a doc comment of lifted `repo_model::Model::check_ids`;
  doc comments are outside κ, `ingest.rs:1065`).

Register rows added or removed:

| Row | Change | Expected status |
|---|---|---|
| `uorust::lift::method_nat_checked_div` (B1) | removed | was residue |

## Phases

### P1: model (slice A)

- A5 `crates/model/src/lib.rs:85-89`: repeated clause in a doc comment.

### P2: translator core `lift.rs` (slice B)

Tests first (written and passing on the base before B1), next to
`lift.rs:12352-12368`, using the test helper `lift` (`:11652`):
- T-B1: `fn q(a: u64, b: u64) -> Option<u64> { a.checked_div(b) }` lifts, and
  the output contains `(prim quotient a b 0 : Nat)`, `(ctor Option.none [Nat])`
  and `(ctor Option.some [Nat] (prim quotient a b 0 : Nat))`; the same with
  `checked_rem`/`remainder`; `a.checked_div()` on `a: u64` is
  `Err(Residue::Form(t))` with `t == "method call .checked_div"`;
  `a.checked_div(b)` with `a: u64, b: i128` is `Err(Residue::Form(t))` with
  `t == "argument of another type to .checked_div"`. If any text differs on the
  base, B1 is withdrawn.

Changes (all in residue functions):
- B1 `lift.rs:4531-4534, 4635-4665`: drop `method_nat_checked_div`; the
  `*_args` arm (`:4541-4543`, `:4722-4747`) with `one_arg` (`:5255-5274`)
  produces the same text and errors. Pinned by T-B1 and the fixture's use
  through UE-01.
- B2 `lift.rs:4581-4585`: drop the `(Ty::List(t), "first")` arm, unreachable
  because `method_on` calls `combinator` first (`:4358-4364`, `:6431-6435`).
  Pinned by UE-01 (the fixture's `.first()`, thermostat `band.rs:72`) and the
  compiler.
- B7 `lift.rs:10755-10779`: rename the local set shadowing `evidence_axioms()`.
  Pinned by the compiler and UE-01.

### P3: rest of `crates/uorust/src` (slice C)

- C2 `bridge.rs:1205`: `SCREEN_TIMEOUT` becomes
  `Duration::from_millis(PART_TIMEOUT_MS)` (`:441`), as `LONGEST_STEP`
  (`:1113`) already is, with `const _: () = assert!(SCREEN_TIMEOUT.as_millis() == 120_000);`
  beside it, so the compiler checks the value is exactly unchanged (120 000 ms).
- C8 `project.rs:37`: `LEXLEAN` → `LEXLEAN_TOML` (the file, not the program
  `certify.rs:1182`); value unchanged.

## Risks and limits

- Most reconnaissance candidates are left undone: the repository's pinned
  tests, not reading, are the evidence bar, and most duplicated helpers sit on
  paths no test pins. Adding those pins is separate test work.
- Native Qortara gates are not recorded; this plan and its audits are host
  documents, not `.qortara/` records.

## Verification

In the pinned SDK image (as `bootstrap.yml` runs it): T-B1 on the base first;
then after all changes `just register`, the I1 diff review against the table,
and `just vv` twice on the final tree (fmt-check, model, template-check,
self-model-check, lint `-D warnings`, test, deny, app, uorust).

## Revision

- v2 answered `docs/plans/uorust-refactor.audit-v1.md` (VETO on `c786866e…`).
- v3 answered `docs/plans/uorust-refactor.audit-v2.md` (VETO on `8c6f4027…`).
- v4 answers `docs/plans/uorust-refactor.audit-v3.md` (VETO on `04f57bb9…`):
  H1 (A1 withdrawn), H2 (B3 withdrawn), H3 (B5 withdrawn), H4 (B6 withdrawn),
  H5 (C1 withdrawn), H6 (B5 row gone; B8 rows named), H7 (I1 accepts
  line-position-only row changes), H8 (admission rule names the real base
  pins).
- v5 answers `docs/plans/uorust-refactor.audit-v4.md` (VETO on `70a7c62b…`):
  K1 (I1 accepts the count, tally and C8 text changes that follow from the
  listed rows), K2 (Goal names `frontier.toml` as the one changed generated
  file), K3 (C3 withdrawn), K4 (B2/B7 pins named; C3 gone), K5 (B8 phases
  match the code's comments; the opaque closure and `:10914-10920` placed).
  The pre-existing removal loop not confining manifest paths (`lib.rs:208-215`)
  is reported separately.
- v6 corrects inline the all-L1 VETO `docs/plans/uorust-refactor.audit-v5.md`
  (on `24154ce3…`): M1 (I1 reviews function rows; construct rows and header
  tallies may change), M2 (Goal scoped to the UE-01 fixture projection; self
  projection changes named), M3 (C2 expression stated, compile-time value
  assertion), M4 (removal loop at `lib.rs:208-215`).
- v7 corrects inline the all-L1 VETO `docs/plans/uorust-refactor.audit-v6.md`
  (on `025172ca…`): N1 (B8 withdrawn: UE-18 refuses a rise in uncertified
  `uorust::lift::` functions), N2 (C2 asserts `as_millis() == 120_000`), N3
  (Goal names the self projection as generated, not a base pin), N4 (I1 says
  "function row"). Confirmed corrected by audit v6: M1, M4.
- v8 corrects inline the all-L1 VETO `docs/plans/uorust-refactor.audit-v7.md`
  (on `f41de2a5…`): P1 (UE-18 test cited at `ratchet.rs:104-108`). Confirmed
  corrected by audit v7: N1, N2, N3, N4.
