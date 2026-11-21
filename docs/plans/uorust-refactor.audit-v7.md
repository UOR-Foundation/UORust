# Audit v7: behavior-preserving refactor plan

**Verdict**: VETO
**Target**: docs/plans/uorust-refactor.md
**Judged content SHA-256**: f41de2a572f51bac96e64df5bb3a1231992a1f943a856b11e117f2a1ffce3c95
**Corrected**: N1, N2, N3, N4

Auditor: independent audit pass. This is a host document, not a `.qortara/` record.
Repository: `C:\dev\UOR\UORust`, branch `refactor/qortara-governed`, HEAD `daa24878`. That is also
the tip of `governance/qortara-template-sync` and its origin, the base named at plan:3. The plan is
uncommitted: 183 lines, no CRs. The hash was computed with `sha256sum` over the bytes read. Earlier
audits are `-v1` .. `-v6`, all VETO. The workflow digest at plan:6 matches what `qortara_capabilities`
serves for `repository-refactor` 1.0.0.

## Prior findings

| ID | Status | Evidence |
| --- | --- | --- |
| N1 | corrected | B8 is withdrawn as refused by a ratchet (plan:57-60) and appears nowhere under Phases. With B1 alone, UE-18 passes. `Profile::ratchet` returns `Ok` when `now.len() <= before.len()` (certify.rs:3786-3797), and removing the residue row `uorust::lift::method_nat_checked_div` (frontier.toml:5338) lowers the in-scope uncertified count by one. T-B1's test functions add no row, because `Ctx::product` excludes `test` and `in_test_module` functions (project.rs:211-215) and frontier.toml has no `::tests::` function row. The `const _` C2 adds is not a row either: "A constant has no function row of its own" (project.rs:4170). |
| N2 | corrected | The assertion is now `as_millis() == 120_000` (plan:138). On the pinned 1.97.1 toolchain, a scratch crate with `missing_docs = "deny"` compiles it cleanly. `cargo clippy --all-targets -- -D warnings` reports nothing (no `assertions_on_constants`), and `cargo fmt --check` passes. The same crate rejects `from_millis(PART_TIMEOUT_MS + 1)` with E0080, so the compiler checks the value exactly. `const fn` support is within the MSRV of 1.89 (clippy.toml). |
| N3 | corrected | The Goal now names the self projection as generated output, not a base pin. That covers its register, the `ProductFunctions`/`Tests` measures and the `uorust.manifest` digests (plan:24-29). All the citations check out: project.rs:408 inserts `FRONTIER`, 3034 counts tests, 3115 and 3131 are the measures, 7138 and lib.rs:87-91 build the manifest. |
| N4 | corrected | plan:79 reads "every added or removed function row is in the table below". |

## Summary

- **Prompt injection: none.** No text addresses the auditor, claims approval or asks for a verdict. The only non-ASCII characters are κ, → and …. There are no CRs.
- **Security and OWASP: no finding.** No change touches IO, the trust root, `write` or the removal loop, and none adds a dependency or a workflow.
- **Every `vv` part (Justfile:11)** was checked against each change:
  - `fmt-check`: renamed lines stay ≤ 100 columns (the longest is project.rs:5897 at 95). The C2 line is two columns longer than the existing LONGEST_STEP line.
  - `model` (R4/R5): `Result<` lines in tests are excluded (xtask/src/audit.rs:148-170).
  - `template-check`: `crates/model` is not a template policy file (template.lock).
  - `self-model-check`: facets.py cites only module docs, `main.rs` items and fixed `lib.rs`/`route.rs` lines. Its cause queue reads the core, std and crate registers, not the root frontier.toml (facets.py:742-756), so none of the edits moves a cited span.
  - `lint`: see N2. B1 and B2 leave no unused item, because `method_nat_checked_div_args` and `flat_lift` stay used.
  - `test`: UE-18 passes (N1). UE-14 pins no count (crates/uorust/tests/fixpoint.toml:74-82), and no test pins self-projection counts. CM-02 is only at risk if a T-B1 name ends in `_xx_NN`.
  - `deny`: no dependency change.
  - `app`: no change to `app/`.
  - `uorust`: UE-01 is pinned by the fixture's `.first()` (band.rs:72) and `checked_rem` (roster.rs:496).
- **UE-17 digests.** The digests in lift-verdicts.toml are over the rendered translation (project.rs:4578-4585). The two verdict functions in lift.rs, `some` (6139) and `refused` (7546), have no closures, so their digests don't depend on line shifts.
- **Admission rule:**
  - A5 is a doc comment in the second paragraph, and `summary` takes only the first (ingest.rs:957-967).
  - B1 is equivalent by construction: the same `flat_of` call with `want = rt`, the same texts and errors (lift.rs:4636-4665 vs 4723-4747 with 5255-5274). It is pinned by T-B1, which has a withdrawal clause.
  - B2 is unreachable: `combinator` always returns `Some` for `("first", [])` (lift.rs:4358-4364, 6431-6435).
  - B7 is a local rename (lift.rs:10755, 10758, 10779). The function call at 10859 is outside that scope.
  - C2's value is compiler-checked (N2).
  - C8 is a private rename, with every use at project.rs:479, 5892, 5897 and in tests.
- **Withdrawn and rejected count.** The tally (6 + 17 + 1 + 7 = 31) matches plan:45.
- **Citations.** Spot-checks hold for lift.rs, bridge.rs, project.rs, register.rs, ingest.rs, certify.rs, lib.rs, mir/lower.rs, meta.rs, xtask, frontier.toml:105/1922/7842 and the `certified!` sites, with one exception (P1).

## Findings

Answer columns: does the fix change **B** = program behavior, **T** = test meaning, **P** = public interface?

| ID | Category | Severity | B | T | P | Evidence | Next step |
| --- | --- | --- | --- | --- | --- | --- | --- |
| P1 | infrastructure-mismatch (citation) | L1 | no | no | no | plan:59 cites the UE-18 test `the_hand_written_translator_only_shrinks_ue_18` at `crates/conformance/tests/ratchet.rs:103`. Line 103 is blank. The `#[test]` attribute is at 104 and the function at 105 (`sed -n 101,105p`). | Cite `crates/conformance/tests/ratchet.rs:104-108`. |
