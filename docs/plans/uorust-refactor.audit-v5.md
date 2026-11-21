# Audit v5: behavior-preserving refactor plan

**Verdict**: VETO
**Target**: docs/plans/uorust-refactor.md
**Judged content SHA-256**: 24154ce3a404de61431f9c72cf40fe37ee96668ef0a5aaa34f77ef18046cbe0c

Auditor: independent audit pass. This is a host document, not a `.qortara/` record.
Repository: `C:\dev\UOR\UORust`, branch `refactor/qortara-governed`, HEAD `daa24878`.
`governance/qortara-template-sync` contains the same commit, which is the base the plan names
(plan:3). The plan is uncommitted (176 lines, no CRs). The hash above was computed with
`sha256sum` over the bytes read. Earlier audits: `-v1` (VETO, `c786866e…`), `-v2` (VETO,
`8c6f4027…`), `-v3` (VETO, `04f57bb9…`), `-v4` (VETO, `70a7c62b…`).

All four findings below are L1, so the planner can correct them inline.

## Prior findings

| ID | Status | Evidence |
| --- | --- | --- |
| F1, F2, F5, F7, F10 | moot | A2-A4, C1 and C7 are withdrawn (plan:43-51). |
| F3, F4 | corrected | T-B1 (plan:106-113) asserts the texts that `none_of`/`some_of`, `method_nat_checked_div` (lift.rs:4644-4649) and `one_arg` (lift.rs:5263-5273) produce. |
| F6, F9 | corrected | I1 uses κ membership plus a row table (plan:68-94). I3 has no dangling qualifier (plan:81-83). |
| F8 | corrected | bridge.rs:1205, 441 and 1113 are correct. |
| G1-G4, G6 | moot | A2, A6, A7, C4 and C11 are withdrawn (plan:43-51). |
| G5 | corrected | Rows use the register's `uorust::lift::<fn>` form (frontier.toml:5176, 5338). |
| G7 | corrected | The Revision section cites -v1 to -v4 with the right hashes (plan:164-171). |
| H1-H5 | moot | A1, B3, B5, B6 and C1 are withdrawn (plan:43-48). |
| H6 | corrected | The five B8 rows are named (plan:90-94). |
| H7 | partly corrected; see M1 | I1 now accepts line positions, count and tally changes, and the C8 text change. |
| H8 | corrected | The admission rule names the real base pins (plan:26-34). Each change cites one. |
| K1 | partly corrected; see M1 | Counts, tallies and the three `project::LEXLEAN` lines are accepted. These lines are correct: frontier.toml:105, 1922 and 7842. New `[[construct]]` rows and new header lines are not accepted. |
| K2 | partly corrected; see M2 | plan:23-24 names `frontier.toml`. plan:22 still promises that no projection changes. |
| K3 | moot | C3 is withdrawn (plan:48-49). |
| K4 | corrected | B2 cites UE-01 and the compiler (plan:122-123). B7 cites the compiler and UE-01 (plan:124-125). |
| K5 | corrected | B8's ranges match lift.rs exactly. See the Summary. |

## Summary

These claims were checked against the source and hold:

- **B8 ranges (lift.rs).**
  - 10876-10889: callee axioms.
  - 10890-10900: mutual groups.
  - 10901-10913: divergences.
  - 10914-10920: decl write-back and the `#` retains, kept inline in `lift_all`.
  - 10921-10954: opaque closure. It has no phase comment; 10925 is an inner comment.
  - 10955-10985: callee-first order.
- **B8 parameters.** Each extracted function needs only values already in scope. Nobody gets a long parameter list, so this is not over-fragmented.
  - `axioms_with_callees(&graph, &own)`: 2 parameters.
  - `merge_mutual_axioms(&groups, &mut axioms)`: 2 parameters.
  - `split_divergences(&mut axioms)`: 1 parameter.
  - `opaque_closure(&decls, &graph)`: 2 parameters.
  - `callee_first(decls, &graph, &mutual_of)`: 3 parameters.
  - `types` (10875) and `grouped(ordered)` (10987) stay in `lift_all`.
  - None of them returns `Result`, so R5 `audit_limits` (xtask/src/audit.rs:148-178) is unaffected.
- **B8 pins.** The fixture covers the phases through UE-01 (`the_fixture_projects_to_the_committed_project_ue_01`, crates/uorust/tests/engine.rs:155):
  - `holdsOffMutual` (Behavior.lex.tex:2683-2711).
  - `firstRooms` (Behavior.lex.tex:2316).
- **B1.** Removing the arm at lift.rs:4532-4534 (guard `args.len() == 1`) leaves the `_args` arm (4541-4543, 4722-4747). With `one_arg` (5255-5274), it does the same `flat_of(.., Some(rt))`, the same `same` check and the same error text, and uses the same format string.
  - No arm in between matches.
  - Every changing text is pinned: T-B1 covers the division primitive and both errors, and the fixture's `checked_rem` covers the full term through UE-01.
- **B2.** `method_typed` (lift.rs:4520) is reached only after `combinator` for any `Ty::List` (lift.rs:4358-4364). `("next" | "first", [])` always returns `Some` (lift.rs:6431-6435), so the arm at 4581-4585 is dead. The live path is pinned by `readings.first()` (thermostat band.rs:72).
- **B7.** If the local at lift.rs:10755-10779 is only partly renamed, `evidence_axioms` resolves to the function item and the build fails.
- **C8.** It is a rename at project.rs:37. Its uses are 479, 5892, 5897 and tests. The value `lexlean.toml` reaches the fixture projection through UE-01. certify.rs:1182 is the program name.
- **Register statuses.** Every edited function is residue or effect:
  - `method_typed` (frontier.toml:5404).
  - `method_nat_checked_div` (5338).
  - `lift_all` (5176).
  - `project::project` (7744).
  - `within_limit` (7840).
  - A5 edits only a doc comment of lifted `repo_model::Model::check_ids` (frontier.toml:9486, crates/model/src/lib.rs:85-89). κ covers the signature and block tokens only (ingest.rs:1063-1068).
- **Other citations.** These hold:
  - Q1 (mir/lower.rs:1741, 1772) and Q2 (bridge.rs:1185-1201: it skips `.lexlean` and not `.lake`).
  - `certified!` sites (lift.rs:1814, 2034, 3396, 5298, 6038; mir/lower.rs:686, 6973).
  - CM-02 (meta.rs:163), the xtask list (main.rs:39-51), the splitters (bridge.rs:309, layouts.rs:578).
  - register.rs:55-77 and 537-563, and the test helper (lift.rs:11652).
  - The `vv` recipe list (Justfile:11).
- **Security and OWASP.** No remaining change touches the trust root, `admit`, `write` or IO, so there is no finding. The pre-existing unconfined removal path in `uorust::write` (lib.rs:208-215) is out of scope and is reported separately.
- **Prompt injection.** None:
  - No text addresses the auditor or claims authority.
  - The only non-ASCII characters are κ, → and ….
  - The plan has no CRs.
- **Dependencies, architecture and orphans.** No new dependency, no new module edge, and no gate scans `docs/plans/`. No finding.

## Findings

Answer columns: does the fix change **B** = program behavior, **T** = test meaning, **P** = public interface?

| ID | Category | Severity | B | T | P | Evidence | Next step |
| --- | --- | --- | --- | --- | --- | --- | --- |
| M1 | specification-drift | L1 | no | no | no | I1 accepts line positions, changed `functions` counts, tallies and the LEXLEAN text. Any other change fails it (plan:71-76). **The added B8 rows bring new construct texts.** Each `[[construct]]` row is one distinct residue `detail` (frontier.toml:1497-1515). Constant items and closures surface by name in those details and in the header (frontier.toml:16-29, 89-110). `split_divergences` reads `lift::DIVERGES` inside closures (lift.rs:10907, 10909; const at 4238). `opaque_closure` reads `KERNEL_OPAQUE` (10935; const at 10631) inside closures. Neither name appears anywhere in frontier.toml today, because `lift_all` stops at its nested loop (frontier.toml:5176-5178). The new rows' details are therefore likely new `[[construct]]` rows and new header lines, which I1 rejects. **`lift_all`'s own `detail` could also change.** That is not a line position, so I1 rejects it too, although its status stays residue. A correct run can fail I1. | Amend I1. Accept new or removed `[[construct]]` rows and header lines whose counted functions are only rows in the table. Accept a changed `detail` on an edited residue row whose status is unchanged. Alternatively, restrict the I1 review to function rows: status, κ membership and the table. |
| M2 | specification-drift | L1 | no | no | no | The Goal promises "without changing any projection" (plan:22). A projection writes its own `frontier.toml` (project.rs:408), and the self projection's register changes by I1's rows. The table also lets any B8 row lift (plan:90-94). A lifted row adds a definition to the self projection's Lean output. A correct run can therefore break the Goal. This is K2's residue. | Amend the Goal to "without changing the UE-01 fixture projection, any verdict, …". State that the self projection changes only by its `frontier.toml` rows and by a definition for any B8 row that lifts. |
| M3 | coverage-gap | L1 | no | no | no | The admission rule admits "a constant whose value is unchanged and checked by the compiler" (plan:31). For C2 (plan:142-143), the compiler checks only the type of `SCREEN_TIMEOUT`, not that `from_millis(PART_TIMEOUT_MS)` equals `from_secs(120)`. Its only reader is `elapsed() > SCREEN_TIMEOUT` (bridge.rs:1526), and no `just vv` check observes it. A wrong expression, for example `from_secs(PART_TIMEOUT_MS)`, would compile and pass. C2 does not meet the rule as written. | State the exact expression `std::time::Duration::from_millis(PART_TIMEOUT_MS)`, and pin the value with a compile-time `const _: () = assert!(SCREEN_TIMEOUT.as_millis() == 120_000);` next to it. Otherwise, withdraw C2. |
| M4 | documentation-accuracy | L1 | no | no | no | The plan cites the pre-existing removal loop twice, with different ranges: `lib.rs:207-214` (plan:49) and `lib.rs:207-213` (plan:177). In the source, the block is lib.rs:206-215 and the loop is lib.rs:208-215. Line 207 reads the manifest, and 215 closes the `for`. | Cite lib.rs:208-215 in both places. |
