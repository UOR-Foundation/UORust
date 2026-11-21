# Audit v8: behavior-preserving refactor plan

**Verdict**: PASS
**Target**: docs/plans/uorust-refactor.md
**Judged content SHA-256**: 96d6058764e3dfb0ecb7d5a6139da03dfa7232ec2f4e432674512a98c5830674
**Corrected**: P1

Auditor: independent audit pass. This is a host document, not a `.qortara/` record.
Repository: `C:\dev\UOR\UORust`, branch `refactor/qortara-governed`, HEAD `daa24878`. HEAD is also the
tip of `governance/qortara-template-sync` and `origin/governance/qortara-template-sync`, which is the
base named at plan:3. The plan is uncommitted: 186 lines, no CRs. The hash was computed with
`sha256sum` over the bytes read. Earlier audits are `-v1` .. `-v7`, all VETO.

## Prior findings

| ID | Status | Evidence |
| --- | --- | --- |
| P1 | corrected | plan:60 now cites `crates/conformance/tests/ratchet.rs:104-108`. Those lines are `#[test]` (104), `fn the_hand_written_translator_only_shrinks_ue_18()` (105) and its body through the closing brace (108). The Revision entry (plan:184-186) records the change. |

## Summary

**Prompt injection: none.** No text in the plan addresses the auditor, claims approval or asks for a verdict. The words "VETO" and "verdict" appear only as references to earlier audits and to UE-17. The only non-ASCII characters are κ, → and …, and there are no CRs.

**Security and OWASP: no finding.** No change touches IO, the bootstrap trust root, the manifest removal loop (lib.rs:208-215) or `copy_tree`. No change adds a dependency, a workflow or an unsafe block.

**Infrastructure alignment.** Every citation was opened at HEAD:

- **A5:** crates/model/src/lib.rs:85-89 holds the doubled clause "belong to the repository that has those classes" at lines 87-88.
- **B1:** the arms are at lift.rs:4531-4534 and 4541-4543. `method_nat_checked_div` runs 4635-4665 and `method_nat_checked_div_args` runs 4722-4747. `one_arg` is at 5255-5274.
- **B2:** the `first` arm is lift.rs:4581-4585. `combinator` is called at 4358-4364, and its `("next" | "first", [])` arm is at 6431-6435.
- **B7:** the local is at lift.rs:10755, 10758 and 10779. The function call at 10859 is outside the local's scope, otherwise the base would not compile.
- **Test locations:** the test helper `lift` is at 11652, and the neighbouring checked-operation tests are at 12352-12368.
- **C2:** bridge.rs:441 (`PART_TIMEOUT_MS = 120_000`), 1113 (`LONGEST_STEP`) and 1205 (`from_secs(120)`).
- **C8:** project.rs:37 is the file constant and certify.rs:1182 is the program. The uses are at 479, 5892, 5897 and in tests at 6914, 7161, 7190, 7204, 7217 and 7387. frontier.toml:105, 1922 and 7842 hold the `project::LEXLEAN` text.
- **Other citations, all confirmed:**
  - register.rs:55-77, 201 and 537-563
  - certify.rs:48 and 3786-3810
  - the `certified!` sites at lift.rs:1814, 2034, 3396, 5298, 6038 and mir/lower.rs:686, 6973
  - mir/lower.rs:1741 and 1772
  - bridge.rs:1185-1201 and 309
  - layouts.rs:578, meta.rs:163, xtask/src/main.rs:39-51 and ingest.rs:1065
  - project.rs:408, 3034, 3115 and 3131
  - project.rs:7138, a test tying `ProductFunctions` to the register's function count
  - lib.rs:87-91 and 208-215
  - the fixture's `.first()` at thermo-core/src/band.rs:72 and `checked_rem` at roster.rs:496

**Admission rule.** Each change meets it:

- **A5:** the edit is in the second paragraph of the doc comment. `summary` reads only the first paragraph (ingest.rs:956-966), and the full `doc` is otherwise used only for `is_empty` (project.rs:3032). κ hashes signature and block tokens without attributes (ingest.rs:1064-1067), so lifted `repo_model::Model::check_ids` (frontier.toml:9484-9487) keeps its κ.
- **B1:** the two paths are equivalent by construction. Both call `flat_of(arg, …, Some(rt))`, run the same `same` check with the same text, and use the same prim selection and rendering. For an argument count other than one, the base already takes the `_args` arm. T-B1 pins this and withdraws B1 if any expected text differs on the base. `Ty::Nat(_).render()` is `Nat` (lift.rs:168), consistent with T-B1's expected strings.
- **B2:** every `List` receiver reaches `combinator` before `method_typed` (lift.rs:4358-4370), and `combinator` always returns `Some` for `("first", [])`. `method_typed` has no other caller.
- **B7:** this is a local rename that the compiler checks.
- **C2:** the expression `Duration::from_millis(PART_TIMEOUT_MS)` matches `LONGEST_STEP`. The `const _` assertion fixes the value at 120 000 ms. The new line is 94 columns, within `max_width = 100` (rustfmt.toml).
- **C8:** this is a private rename. Renamed lines stay at 95 columns or less.

**Register and invariants.**

- **I3 holds.** Every edited function is residue: `method_nat_checked_div` (frontier.toml:5338), `method_typed` (5404), `lift_all` (5176), `project::project` (7744) and `project::within_limit` (7841). `check_ids` (lifted) changes only its doc.
- **Edited functions none of whose MIR is certified.** None of the edited functions is in `crates/certified/roots`. The certified callees `flat_lift`, `some_of`, `none_of` and `opt_of` are not edited, and they stay used.
- **Line shifts are expected.** B1 and B2 shift the source positions embedded in the residue details of unedited `applied_path` and `match_rest` (frontier.toml:4680 and 5244). These are not status changes, so I1's function-row review accepts them as written.

**Gates in `just vv` (Justfile:11).**

- **fmt-check:** passes, per the column checks above.
- **model:**
  - R5 inspects only `Result<` lines outside tests (xtask/src/audit.rs:148-170).
  - R4 is not triggered.
- **template-check:**
  - No template-owned file is edited.
  - CONFORMANCE.md is unaffected, because no ID or scenario changes.
- **self-model-check:** facets.py reads only module docs of project.rs and lift.rs (facets.py:223, 473). The app's cited spans are in main.rs, crates/uorust/src/lib.rs:50, route.rs and model/ids.toml, and none of those is edited.
- **lint `-D warnings`:** no item is left unused, and the `const _` was compile-checked by audit v7.
- **missing_docs:** an unnamed `const _` needs no doc. Every renamed item keeps its doc.
- **test:**
  - UE-01 (engine.rs:155) is pinned by the fixture's `.first()` and `checked_rem`.
  - UE-16 (engine.rs:983, 1294, 1382) runs against the regenerated register.
  - UE-17 digests cover the rendered translation of the verdict functions, which have no closures.
  - UE-18 (ratchet.rs:104-108) passes because B1 lowers the uncertified `uorust::lift::` count (certify.rs:3793-3797). Test functions add no row.
  - CM-02 is triggered only by a test name ending in an ID slug, and the plan names none.
- **deny:** no dependency change.
- **app:** no change.
- **uorust:** the fixture projection is unchanged. The self projection is generated, as the Goal states (plan:24-29).

**Simplicity, orphans and drift.**

- **Orphans:** B1 leaves no caller of the removed function (its only call is lift.rs:4533). B2 leaves `flat_lift` and `opt_of` used elsewhere.
- **Candidate count:** the tally 6 + 17 + 1 + 7 = 31 matches plan:45.
- **Goal:** the six changes named in the Goal match the six under Phases.
- **Verification:** the section matches the `vv` recipe parts.

**Documentation accuracy.** The plan's descriptions of each change, of the withdrawn items and of the out-of-scope defects (CM-02 whitespace at meta.rs:163, the xtask task list and the `->` splitters) match the code as opened.

## Findings

None.
