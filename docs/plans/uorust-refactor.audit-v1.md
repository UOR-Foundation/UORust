# Audit: behavior-preserving refactor plan

**Verdict**: VETO
**Target**: docs/plans/uorust-refactor.md
**Judged content SHA-256**: c786866e01885b2593fa72be997987b77d49f47c025b481dc8f03eba7af85ca3

Auditor: independent Qortara audit pass (host document, not a `.qortara/` record).
Repository state: `C:\dev\UOR\UORust`, branch `refactor/qortara-governed`, HEAD
`daa24878f17cb56cc530d468c6c87227e5ba7989`, which matches the plan's stated base.

## Summary

Most of the plan's load-bearing claims hold when checked against the source:

- B1 holds. `method_nat_checked_div` (lift.rs:4636-4665) and the `*_args` arm
  (lift.rs:4541-4543 -> 4723-4747) with `one_arg` (lift.rs:5255-5274) emit the
  same term. They produce the same type-error text (`argument of another type to
  .{name}`) and the same arity error (`method call .{name}`).
- B2 holds. `method_typed` has exactly one caller, `method_on` (lift.rs:4370).
  `method_on` first calls `combinator` for every `Ty::List` receiver
  (lift.rs:4358-4364), and the `("next" | "first", [])` arm (lift.rs:6431) always
  returns `Some`.
- A1 holds. Every suite is collected from `model.ids.id` itself
  (codegen.rs:69-71), so `rows.is_empty()` (codegen.rs:75-77) cannot fire.
- C1 holds. The four helpers are byte-identical:
  `toml::Value::String(..).to_string()` at certify.rs:2400, profile.rs:306,
  register.rs:574, embed.rs:3885 and mir/core.rs:2538. `project::toml_string`
  (project.rs:5245) uses different escaping.
- C3 holds. Merging the blocks at lib.rs:203-215 keeps `admit` before any removal
  or write.
- C6 holds. `project::sha` (project.rs:668-673) is the same
  `Sha256::digest(..)` -> `{b:02x}` as registry.rs:119-122, and the crates.io
  checksum comparison is unchanged.
- κ holds. It is defined at register.rs:55-77 as the domain, the crate path and
  the tokens. UE-16 `admit` skips an absent κ (register.rs:543).
- Every function the plan edits is `[[residue]]` or `[[effect]]` in
  `frontier.toml`, and the lifted helpers it reuses are left untouched
  (`flat_some`, `opt_of`, `project::sha`, `project::toml_string`).
- The `certified!` sites, the CM-02 whitespace, the xtask task list and Q1/Q2 are
  all cited correctly.

The plan file itself is acceptable to the repository's gates:
- `audit_deferral` scans only `crates/`, `xtask/` and root `*.md`
  (xtask/src/audit.rs:205-218).
- The R2 assertive-word checks read only fixed root documents
  (crates/conformance/src/meta.rs:176-181, 206).
- The plan contains no deferral marker.

No prompt injection was found.

The verdict is still VETO, because of the findings below:
- one L3: the trust-root refactor (A2-A4) has a safety net with large holes;
- four L2: verification and test-specification defects;
- five L1: plan-text defects.

## Findings

Answer columns: does fixing it change **B** = program behavior, **T** = test
meaning, **P** = public interface?

| ID | Category | Severity | B | T | P | Evidence | Next step |
| --- | --- | --- | --- | --- | --- | --- | --- |
| F1 | security-l3 | L3 | no | yes | no | A2 moves a trust-root predicate with 13 conjuncts (xtask/src/bootstrap.rs:527-553: `image`, `build`, `dockerFile`, `features`, `containerUser`, `remoteUser`, `overrideCommand`, the docker.sock mount, the config mount, `initializeCommand`, the `runArgs` init, `curl`, `stable`). T-A1 (plan:59-62) plants defects for only 4 of them. If the extraction dropped or inverted any of the other 9, T-A1 and `just vv` would still pass, because the only other check is the positive run on the repository's own devcontainer. A3 (bootstrap.rs:227-248) and A4 (bootstrap.rs:492-497, 503-508) also refactor trust-root parsing with no negative test at all, and the plan's own risk line (plan:125) says so. For example, an A4 helper that skipped the canonical-JSON comparison would pass. | Amend the plan text. T-A1 must plant one defect per conjunct. Add negative tests for A3/A4: non-canonical prismpm.lock and template.lock, a missing array, a non-string element. Each must assert the exact error text. |
| F2 | test-failure | L2 | no | yes | no | The C7 embed equivalence check, "`cargo run -p uorust -- embed` on the thermostat fixture" (plan:122-124), cannot be run as written. `embed` requires `--crate`, `--mir` (module shards), `--library` (rust-src) and `--out` (crates/uorust/src/main.rs:372-388), and documents its crate as `core`, `std`. `crates/uorust/tests/fixtures/thermostat` is a Cargo workspace, and the plan produces no shards for it. The check also misses the second caller of `embed_project` (bridge.rs:583). | Amend the plan text. Pin the bytes of `embed_project` (pure, embed.rs:3894) with a test before C7, or give the exact command sequence including the MIR dump step. |
| F3 | coverage-gap | L2 | no | yes | no | B1 claims that the errors are the same (plan:87-89), and I2 covers error text. T-B1 (plan:83-84) checks only the success term. Neither the wrong-argument-type error (lift.rs:4646-4649 vs 5268-5272) nor the arity error (`method call .checked_div`, lift.rs:5265-5266) is exercised, so a regression in either one passes `just vv`. | Amend the plan text. T-B1 must also assert both error texts for Nat `checked_div`/`checked_rem`. |
| F4 | documentation-accuracy | L2 | no | yes | no | T-B1's expected term `(if (x =? 0) then none else some(prim quotient\|remainder r x 0 : Nat))` (plan:84) is not what the code emits. The arm emits `none_of(rt)` = `(ctor Option.none [<ty>])` (lift.rs:5245-5247) and `some_of(rt, "(prim quotient r x 0 : Nat)")` (lift.rs:4740-4742). A test written to the plan's string fails on correct code. | Amend the plan text with the exact expected substrings. |
| F5 | macro-architecture | L1 | no | no | no | C1 puts the shared helper at certify.rs:2400. certify already depends on `register` (certify.rs:39) and `mir` (certify.rs:37-38). Neither register.rs nor mir/core.rs refers to certify today. C1 would create the module cycles register <-> certify and mir <-> certify. | Amend the plan text: put the helper in a leaf module that the users already depend on, for example register.rs, which certify already imports. |
| F6 | specification-drift | L1 | no | no | no | I1 (plan:46-48) requires the honesty header to stay at `344 ... lifted` (frontier.toml:8). The plan adds about 12 new functions and removes one: `cell`, `devcontainer_uses_locked_sdk`, the A3 and A4 helpers, `rel`, the A7 lookup, the B5 method, four B8 phases, `links` and `read_text`, while `method_nat_checked_div` goes. It states no expected lift status for any of them. A pure `&str` helper such as `cell` uses `replace`/`trim`, which the lift handles (lift.rs:3977, 4097). If one lifts, the count becomes 345 and I1 fails even though the change preserves behavior. The total of 1503 also changes. | Amend the plan text. Restate I1 as "no `[[lifted]]` κ lost or turned unlifted", and give the expected status of each new function. |
| F7 | specification-drift | L1 | no | no | no | The goal applies a candidate only when `just vv` can verify it (plan:22-23). The risks section says C7's embed side has no byte golden and depends on review plus a manual diff (plan:122-124). The plan therefore includes a change its own admission rule excludes. | Amend the plan text: withdraw C7's embed side, or make it verifiable through `just vv` (see F2). |
| F8 | infrastructure-mismatch | L1 | no | no | no | C2 cites `bridge.rs:1113` for `SCREEN_TIMEOUT`. Line 1113 is `LONGEST_STEP`, which is already `from_millis(PART_TIMEOUT_MS)`. `SCREEN_TIMEOUT` is at bridge.rs:1205 (`from_secs(120)`). | Amend the plan text: cite bridge.rs:1205. |
| F9 | documentation-accuracy | L1 | no | no | no | I3 says "except where noted" (plan:52), but no change is noted as an exception, so the qualifier points at nothing. | Amend the plan text: remove the qualifier or name the exception. |
| F10 | documentation-accuracy | L1 | no | no | no | A3 says "both error texts" (plan:71-72). The three arrays carry six error texts (bootstrap.rs:229, 234, 239, 241, 245, 247). | Amend the plan text: "each array's two error texts, unchanged". |

## Checks with no finding

- Prompt injection: the plan contains no text directed at the auditor.
- OWASP/security for C3 (admit before removal) and C6 (checksum integrity): the
  checks are preserved, and nothing is weakened.
- C6 dependency direction (registry -> project): project does not depend on
  registry, and embed and mir::core already call `project::sha`.
- Orphan file: no repository gate scans `docs/plans/`, and the file has no
  deferral markers or R2 assertive words over registered IDs.
- C8: `LEXLEAN` in project.rs:37 is used only by residue functions (`project`,
  `within_limit`) and by the tests. The certify.rs:1182 `LEXLEAN` is the program
  name, so the rename is justified.
- C9: the sites are effect or residue. `layouts::EXTERN` (layouts.rs:598) is used
  only by residue and effect functions.
