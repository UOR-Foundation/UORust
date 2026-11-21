# Audit v6: behavior-preserving refactor plan

**Verdict**: VETO
**Target**: docs/plans/uorust-refactor.md
**Judged content SHA-256**: 025172ca8e9aba687f725c038d5d6be26cf9fa850f1b55732a27bb2962c36518
**Corrected**: M1, M4

Auditor: independent audit pass. This is a host document, not a `.qortara/` record.
Repository: `C:\dev\UOR\UORust`, branch `refactor/qortara-governed`, HEAD `daa24878`
(`governance/qortara-template-sync`, the base the plan names at plan:3, contains the same commit).
The plan is uncommitted (191 lines, no CRs). The hash was computed with `sha256sum` over the bytes
read. Earlier audits: `-v1` .. `-v5` (all VETO). Unless noted, line citations verified by `-v5` at this
HEAD were re-spot-checked and hold (lift.rs:4531-4545, 4581-4585, 10755-10779, 10876-10985, 11652,
12352-12368; project.rs:37; bridge.rs:441, 1113, 1205).

## Prior findings

| ID | Status | Evidence |
| --- | --- | --- |
| M1 | corrected | I1 now reviews function rows only (plan:75-77) and frees `[[construct]]` rows, header tallies and cause lines (plan:79-83). A correct run satisfies it: `lift_all` keeps its outer `loop` with the nested `for c in candidates` (lift.rs:10649, 10678), so it stays residue, and its detail is the position-free CFG check text (mir/lower.rs:1116-1119, frontier.toml:5178). `method_typed`'s detail is position-free too (frontier.toml:5406). It still catches a lost or demoted incumbent `[[lifted]]` κ, because a lifted row that disappears or changes table is a function row changing outside the table. A wording residue is N4. |
| M2 | not corrected; see N3 | The Goal now names the UE-01 fixture projection (plan:22). But the self-projection carve-out ("only by its own register and by a Lean definition", plan:25-27) is still too narrow. |
| M3 | not corrected; see N2 | The expression is now stated (plan:149-152) and `SCREEN_TIMEOUT` is a `const` (bridge.rs:1205). The assertion compiles on 1.97.1 (rust-toolchain.toml), and `Duration::from_millis` and `as_secs` are `const fn`. But `as_secs()` truncates, so the assertion does not pin the value. |
| M4 | corrected | plan:52 and plan:185 both cite lib.rs:208-215. In the source, 208 is `for line in listed.lines()` and 215 closes that `for`. |

## Summary

- **Prompt injection: none.** No text addresses the auditor, claims authority or asks for a verdict. The only non-ASCII characters are κ (U+03BA), → (U+2192) and … (U+2026). There are no CRs.
- **Security and OWASP: no finding.** A5, B1, B2, B7, B8, C2 and C8 don't touch the trust root, `admit`, `write`, the removal loop or any IO path.
- **A5** edits only the second paragraph of the `check_ids` doc. `summary` takes the first paragraph only (ingest.rs:957-967), and κ excludes doc comments, so the self projection doesn't move.
- **C8** has uses only at project.rs:479, 5892, 5897 and in tests. All of them sit in residue functions (frontier.toml:7744, 7840).
- **B1, B2 and B7** hold as `-v5` found.
- **B8 parameters** are proportionate.
- **New blocker (N1).** The gate this plan relies on refuses B8. UE-18 (`the_hand_written_translator_only_shrinks_ue_18`) is part of `cargo test --workspace`, so it runs under `just vv`, and it counts uncertified `uorust::lift::` functions.
- **Infrastructure.** Consts aren't register rows: `LONGEST_STEP` has none, so `const _` adds none. No new dependency, module edge or workflow change.

## Findings

Answer columns: does the fix change **B** = program behavior, **T** = test meaning, **P** = public interface?

| ID | Category | Severity | B | T | P | Evidence | Next step |
| --- | --- | --- | --- | --- | --- | --- | --- |
| N1 | infrastructure-mismatch | L1 | no | no | no | UE-18 (crates/conformance/tests/ratchet.rs:103-108; `certify::Profile::ratchet`, certify.rs:3786-3810) refuses when more functions in `SCOPE` are uncertified under the head than under the base. `SCOPE` includes `uorust::lift::` (certify.rs:48), and a lifted κ missing from `crates/certified/manifest.toml` counts as uncertified ("lifted as …, not yet certified", certify.rs:3757-3769). B8 adds five `uorust::lift::` functions and B1 removes one, so the count rises by four whether or not the new rows lift. The base is HEAD while the work tree changes `frontier.toml`, else HEAD~1 (ratchet.rs:43-51). The comparison therefore runs locally too, and `just vv` fails after `just register`. The certified manifest is out of scope (plan:60-64). No audit or plan revision accounts for UE-18. | Withdraw B8. Alternatively, restrict it so the in-scope uncertified count does not rise (with B1, at most one new function). Name UE-18 in the admission rule and in Verification. |
| N2 | coverage-gap | L1 | no | no | no | `const _: () = assert!(SCREEN_TIMEOUT.as_secs() == 120);` (plan:151) accepts any value in [120 s, 121 s). On toolchain 1.97.1, `from_millis(PART_TIMEOUT_MS + 999)` passes that assertion and prints `120.999s`. So "the compiler checks the value is unchanged" (plan:152) is false, and C2 still doesn't meet the admission rule's "constant whose value is unchanged and checked by the compiler" (plan:34). | Use `assert!(SCREEN_TIMEOUT.as_millis() == 120_000)`. It compiles on 1.97.1, and on the same toolchain it rejects the +999 ms variant with E0080. |
| N3 | specification-drift | L1 | no | no | no | The Goal says the self projection may change "only by its own register and by a Lean definition for a new B8 row that lifts" (plan:25-27). The self projection also states the `ProductFunctions` measure from `lift.functions` (project.rs:3112-3116, 7138), which rises by 4 (+5 B8, -1 B1). It states the `Tests` measure (project.rs:3034, 3131), which rises with T-B1's added test functions. And `uorust.manifest` holds the digest of every projected file (lib.rs:87-91), so it changes with `frontier.toml`. A correct run breaks the Goal as written. This is M2's residue. | State that the self projection's `frontier.toml`, its `uorust.manifest` digests, the `ProductFunctions`/`LiftedFunctions`/`Tests` measure values and, for a B8 row that lifts, its definition may change. Alternatively, describe the self projection as not a base pin and drop the "only". |
| N4 | documentation-accuracy | L1 | no | no | no | I1 first requires "every added or removed row is in the table below" (plan:73), which with no qualifier covers `[[construct]]` rows. It then says construct rows "may change freely" (plan:79-83). New construct rows from the B8 functions aren't in the table, so the two sentences conflict. A reviewer can fail a correct run on the first. | Change plan:73 to "every added or removed function row is in the table below". |
