# Audit v2: behavior-preserving refactor plan

**Verdict**: VETO
**Target**: docs/plans/uorust-refactor.md
**Judged content SHA-256**: 8c6f4027b014279ba2756a0c469f2db3a4eb4ff1a70b1f800afca5e72f850e3a

Auditor: independent audit pass. This is a host document, not a `.qortara/` record.
Repository: `C:\dev\UOR\UORust`, branch `refactor/qortara-governed`, HEAD
`daa24878f17cb56cc530d468c6c87227e5ba7989`. That is also the tip of
`governance/qortara-template-sync`, the base the plan names. The plan file is
uncommitted. The previous audit is `docs/plans/uorust-refactor.audit-v1.md`
(VETO, judged `c786866e…`).

## Prior findings

| ID | Corrected | Evidence |
| --- | --- | --- |
| F1 | yes, but see G1 and G2 | T-A1 (plan:76-87) now plants 13 defects, one for each of the 13 top-level conjuncts at xtask/src/bootstrap.rs:527-553. A3 and A4 are withdrawn (plan:96-100). The plan now admits nothing in bootstrap.rs:227-248 or 492-508. |
| F2 | yes | C7 is withdrawn (plan:153-155), so the embed command that could not be run is gone. |
| F3 | yes | T-B1 asserts the arity error and the type error (plan:116-119). These match `one_arg` (lift.rs:5264-5273) and `method_nat_checked_div` (lift.rs:4645-4649). |
| F4 | yes | The expected strings match the code. `none_of`/`some_of` are at lift.rs:5240-5248. `Ty::Nat(_)` renders as `Nat` (lift.rs:168). `u64` maps to `Ty::Nat(Some(64))` (lift.rs:1239, 2200), and `i128` maps to `Ty::Integer(Some(128))` (lift.rs:1246), so `same` fails. The term template is at lift.rs:4737-4743. `Residue::Form` is a variant of `pub(crate) enum Residue` (lift.rs:30), and the test helper `lift` (lift.rs:11652) returns `Result<String, Residue>`, so it can be matched. |
| F5 | yes | The helper moves to `register::quoted` (register.rs:574). The only crate item register.rs imports is `crate::UorustError` (register.rs:33). error.rs imports only `std` (error.rs:7-8). certify already imports register (certify.rs:39). No cycle is added. |
| F6 | yes, with residual inaccuracy (G5) | I1 is restated as "no lifted κ lost" (plan:46-52). The expected-status table is present (plan:57-69). |
| F7 | yes | The same as F2: C7 is withdrawn. |
| F8 | yes | C2 cites bridge.rs:1205 for `SCREEN_TIMEOUT` (`from_secs(120)`), bridge.rs:441 for `PART_TIMEOUT_MS`, and bridge.rs:1113 for `LONGEST_STEP`. All three are correct. |
| F9 | yes | I3 (plan:55) no longer has the qualifier that pointed at nothing. |
| F10 | yes | A3 is withdrawn, so the miscounted error texts are moot. |

## Summary

These claims were re-checked against the source and hold:

- **Infrastructure claims.**
  - The `certified!` sites: lift.rs:1814, 2034, 3396, 5298, 6038 and mir/lower.rs:686, 6973.
  - κ is computed over the signature and body tokens only, without attributes (ingest.rs:1065, register.rs:4-6). So A5's doc-comment edit to the lifted `repo_model::Model::check_ids` does not move its κ.
  - UE-16 `admit` skips an absent κ (register.rs:539-563).
  - Q1 `let _ = what;` is at mir/lower.rs:1772. Q2 `copy_tree` is at bridge.rs:1185-1201.
  - The CM-02 whitespace defect is at meta.rs:163, and the xtask task list is at main.rs:39-51.
- **B1 and B2.**
  - The `*_args` arm (lift.rs:4541-4543, 4722-4747) produces the same term and errors as `method_nat_checked_div` (lift.rs:4635-4665).
  - `method_typed` has one caller (lift.rs:4370), after `combinator` (lift.rs:4358-4364).
  - The `("next" | "first", [])` arm (lift.rs:6431-6435) always returns `Some`.
- **B5 and B6.** The ranges at mir/lower.rs:1678-1692 and 1721-1735 match, and so do the axiom-pair sites at 1555, 1691, 1734, 1790 and 4548.
- **B7 and B8.** The `evidence_axioms` shadow is at lift.rs:10755-10779, the four phase comments are inside `lift_all` (10642-10997), and the lifted `lift::evidence_axioms` is at lift.rs:11084.
- **C1, C3, C6, C8 and C9.** Every site exists. The duplicated values are byte-identical.
- **Register status.** Every function whose tokens change is `[[residue]]` or `[[effect]]` in frontier.toml, so I3 holds:
  - `render_conformance`, `xtask::bootstrap::audit`, `docker_credentials_are_confined`
  - `xtask::audit::{collect, audit_limits, audit_deferral}`, `scaffold::new_capability`, `check_honesty`
  - `method_typed`, `combinator`, `lift_all`, the four `Fun<'_, '_>` methods (all 69 `Fun` rows are residue)
  - `register::quoted` and the four wrappers and their callers, `uorust::write`
  - `uorust::{run, certify, read_register}`, `registry::{fetch, embed_crate, layouts}`
  - `embed::{embed_with, embed_linked}`, `layouts::{top_split, home_of, run}`, `bridge::elaborate`
  - `project::{project, within_limit}`

  The lifted helpers the plan reuses (`flat_some`, `opt_of`, `project::sha`, `project::toml_string`, `lift::evidence_axioms`) are untouched.
- **Expected-status table.** The table is plausible, except for the naming and completeness problems in G5.
- **Prompt injection.** None found. The plan contains no directives to the auditor and no hidden characters: the only non-ASCII code points are U+2026, U+2192 and U+03BA.
- **Orphan file.** No repository gate scans `docs/plans/`. `audit_deferral` reads `crates/`, `xtask/` and root `*.md` (xtask/src/audit.rs:204-218). The R2 checks read fixed root documents (meta.rs:176-181, 206). The plan has no deferral markers. No finding.
- **Not verifiable.** The workflow manifest digest `sha256:cd8adea1…` (plan:5) appears nowhere in the repository. The plan itself declares the governance seal NOT_ESTABLISHED, so this is not raised as a finding.

The verdict is still VETO. T-A1 is still not sufficient to catch a weakened extraction of the devcontainer predicate, because it covers neither the absent-key branches nor the call-site wiring. Two other changes are admitted although their failure paths have no test that `just vv` runs, which is the plan's own reason for withdrawing A3, A4 and C7.

## Findings

Answer columns: does the fix change **B** = program behavior, **T** = test meaning, **P** = public interface?

| ID | Category | Severity | B | T | P | Evidence | Next step |
| --- | --- | --- | --- | --- | --- | --- | --- |
| G1 | security-l3 | L3 | no | yes | no | Three conjuncts reject an *absent or ill-typed* key through `is_none_or` or a negated `as_str().is_some_and`: `mounts` (bootstrap.rs:526, 534), `initializeCommand` (bootstrap.rs:542-544) and `runArgs` (bootstrap.rs:545-551). All 13 T-A1 plants (plan:79-85) keep these keys present and well-typed. They remove one mount, change the command text, or drop the init argument. An extraction that turned `is_none_or` into `is_some_and`, or `!…is_some_and` into `…is_none_or`, would therefore accept a devcontainer with no `mounts`, no `runArgs` or no `initializeCommand`, and T-A1 and `just vv` would both pass. `docker_credentials_are_confined` (bootstrap.rs:114-116) still needs the config mount text and `.docker/config.json`, but nothing else requires the docker.sock mount or `prismpm-devcontainer-init`. | Amend T-A1. Add one plant each for: `mounts` removed, `mounts` not an array, `runArgs` removed, `runArgs` not an array, `initializeCommand` removed, `initializeCommand` not a string. Each must assert false. |
| G2 | security-l3 | L3 | no | yes | no | T-A1 tests only the extracted function. The call site in `audit` is guarded only by the positive run, `just model` → `validate` → `bootstrap::audit` (justfile:20-21, xtask/src/main.rs:107-112). The plan claims "Error text unchanged" (plan:95) but never verifies it. The call site is unverified in four ways: (a) a dropped call; (b) the wrong `image` argument, for example the devcontainer's own `image` instead of the one from `prismpm.lock` (bootstrap.rs:527); (c) a changed error text (bootstrap.rs:555); (d) a move relative to `audit_policy_files` and the `bootstrap.yml` check (bootstrap.rs:523, 558-563). Any of these passes `just vv`. A3 and A4 were withdrawn for exactly this reason: their error paths are "reachable only through `audit`" (plan:96-100). AGENTS.md R5 also requires a negative test for every public failure. | Amend A2. Make the extracted unit return `Result<(), Fail>` carrying the exact text. T-A1 then asserts `Err` with that text for each plant, and `Ok` when the repository image is passed in. The call site becomes a single `?` directly after `audit_policy_files`. Also add a test that runs `audit` on a temporary copy of the root, with a planted devcontainer and a consistently re-hashed `template.lock`, and asserts the exact error. Otherwise, withdraw A2 on the same grounds as A3 and A4. |
| G3 | coverage-gap | L2 | no | yes | no | A7 rewrites the detection itself, the two `ASSERTIVE.iter().find(...)` calls inside `check_honesty` (meta.rs:192, 216). No test plants an assertive word next to an open ID or a cited authority in a root document. `the_meta_gate_is_falsifiable_cm_02` (crates/conformance/tests/bdd.rs:172-199) plants only the missing-test CM-02 violation. `the_assertive_vocabulary_is_recognised` (meta.rs:256-268) calls `ASSERTIVE` directly and never `check_honesty`. A faulty `assertive()` helper, for example one that always returns `None`, would silently disable both R2 checks and still pass `just vv`, because the repository's documents are clean. That contradicts the admission rule at plan:22-23. | Amend the plan. Add a test that calls `check_honesty` on a scratch root whose README line pairs an open ID, and then an authority ID, with an assertive word, and asserts both R2 violation texts. Otherwise, withdraw A7. |
| G4 | coverage-gap | L2 | no | yes | no | Three changes alter code that produces diagnostic text. I2 (plan:53-54) promises no byte change in that text, but no test pins it. **A6** changes the `rel` path in the `audit_deferral` violations (audit.rs:224-228), which have no negative test, as the plan itself says at plan:167. It also changes `Source.rel` from `collect` (audit.rs:68-72). `any_other_error_type_is_still_refused` (audit.rs:348-359) checks only `contains("R5")`. **C4** changes the `--link {l}: not <crate>=<dumps>` errors (main.rs:620-622, 668-672). **C11** changes the `UorustError::Io` mapping (main.rs:776-782, 799-804). The CLI exercises none of these error paths. The plan applies the "`just vv` can verify it" rule to withdraw A3, A4 and C7, but not to these three changes. | Amend the plan. Add tests that pin each error text: an `audit_deferral` scratch root with a marker, asserting the relative path in the violation; a `links()` unit test with a malformed `--link`; a missing-file `read_text` test. Otherwise, record A6, C4 and C11 as NO REFACTOR REQUIRED with that reason. |
| G5 | specification-drift | L1 | no | no | no | The expected-status table (plan:57-69) has three problems. (1) It names `main::links` and `main::read_text`, but rows of the binary are registered under `uorust::`: `uorust::run`, `uorust::read_register` and `uorust::certify` (frontier.toml:9336, 9276, 8651). The I1 diff review would look for rows that never appear. (2) The "Removed" list holds only `method_nat_checked_div`. C1 does not say whether `certify::toml_quoted`, `certify::profile::quoted`, `embed::census_string` and `mir::core::toml_quote` (all residue, frontier.toml:3466, 3424, 3886, 5944) are deleted or become delegates. If they are deleted, I1's "every changed/new row accounted for" cannot be met as written. (3) It gives no expected status for re-keyed rows (edited residue and effect functions), which I1 says are checked too. | Amend the plan text. Use the registered paths (`uorust::links`, `uorust::read_text`, `uorust::mir::lower::Fun<'_, '_>::decrease_theorem`). State whether the C1 wrappers are removed, and list them under Removed if they are. Add "re-keyed rows keep their status". |
| G6 | infrastructure-mismatch | L1 | no | no | no | T-A1 cites `bootstrap.rs:691-694` for the "each plant changes the input" assertion (plan:86-87). Lines 689-695 are the path-filter plant in the list. The assertion is `assert_ne!(plant, bootstrap, "every plant must change the workflow")` at bootstrap.rs:707-710. | Amend the citation to bootstrap.rs:707-710. |
| G7 | documentation-accuracy | L1 | no | no | no | The Revision section (plan:182) says v2 answers `docs/plans/uorust-refactor.audit.md` (VETO, `c786866e…`). That audit is now `docs/plans/uorust-refactor.audit-v1.md`, and `uorust-refactor.audit.md` is this v2 audit. The pointer resolves to the wrong document. | Amend the Revision section to cite `docs/plans/uorust-refactor.audit-v1.md`. |
