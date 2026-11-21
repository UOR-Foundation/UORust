# Audit v3: behavior-preserving refactor plan

**Verdict**: VETO
**Target**: docs/plans/uorust-refactor.md
**Judged content SHA-256**: 04f57bb9c464d2173bc38149e9fb026a81c893b8626734ed56f14c8348850d72

Auditor: independent audit pass. This is a host document, not a `.qortara/` record.
Repository: `C:\dev\UOR\UORust`, branch `refactor/qortara-governed`, HEAD `daa24878`.
`daa24878` is also the tip of `governance/qortara-template-sync`, the base the plan names
(plan:3). The plan is uncommitted (166 lines). The hash above was computed with `sha256sum`
over the bytes read. Earlier audits: `docs/plans/uorust-refactor.audit-v1.md` (VETO,
`c786866e…`) and `docs/plans/uorust-refactor.audit-v2.md` (VETO, `8c6f4027…`).

## Prior findings

| ID | Status | Evidence |
| --- | --- | --- |
| F1 | moot | A2, A3 and A4 are withdrawn (plan:36-38). |
| F2 | moot | C7 is withdrawn (plan:44-45). |
| F3 | corrected | T-B1 asserts the arity error `method call .checked_div` and the type error `argument of another type to .checked_div` (plan:104-107). These match `one_arg` (lift.rs:5263-5273) and `method_nat_checked_div` (lift.rs:4645-4649). |
| F4 | corrected | The expected substrings (plan:102-103) match `none_of`/`some_of` (lift.rs:5239-5247) and the arm at lift.rs:4737-4743. They also match the pinned fixture line `(ctor Option.none [Nat]) else (ctor Option.some [Nat] (prim remainder ...))` (thermostat-prism/src/Behavior.lex.tex:1846). |
| F5 | corrected | The helper is `register::quoted` (register.rs:574). register.rs imports only `crate::UorustError` (register.rs:33), and certify already imports register (certify.rs:39). No cycle is added. |
| F6 | corrected | I1 is restated (plan:61-66), and the status table is present (plan:73-81). |
| F7 | moot | The same as F2. |
| F8 | corrected | C2 cites bridge.rs:1205, 441 and 1113 correctly (plan:136-137). |
| F9 | corrected | I3 has no qualifier that points at nothing (plan:69-71). |
| F10 | moot | A3 is withdrawn. |
| G1 | moot | A2 is withdrawn (plan:37). |
| G2 | moot | A2 is withdrawn (plan:37). |
| G3 | moot | A7 is withdrawn (plan:40). |
| G4 | moot | A6, C4 and C11 are withdrawn (plan:39-41). |
| G5 | partly corrected; see H6 | The C1 removals are listed (plan:81). "Edited rows keep base status" is stated (plan:62-63). The `main::` names are gone. The B5 row is still not named the way the register names it, which was G5's own next step. |
| G6 | moot | T-A1 is gone along with A2. |
| G7 | corrected | The Revision section cites audit-v1 and audit-v2 (plan:161-162). Both files exist. |

## Summary

These claims were re-checked against the source and hold:

- **Citations.**
  - `certified!` sites: lift.rs:1814, 2034, 3396, 5298, 6038; mir/lower.rs:686, 6973.
  - κ and registers: register.rs:55, 201, 537-563; ingest.rs:1065.
  - Q1 (mir/lower.rs:1741, 1772) and Q2 (bridge.rs:1185-1201).
  - CM-02 (meta.rs:163), the xtask task list (main.rs:39-51) and the splitters (bridge.rs:309, layouts.rs:578).
  - Every B and C line range the plan cites.
- **Test names.**
  - `the_committed_table_is_the_rendering_of_the_register_cm_01` (codegen.rs:142).
  - `check_model_passes_on_this_repository_cm_01` (xtask/tests/cli.rs:15).
  - `every_failure_is_a_typed_variant_ue_06` (engine.rs:344).
  - `a_lifted_kappa_that_turns_unlifted_is_refused_ue_16` (engine.rs:1382).
  - The test helper `lift` (lift.rs:11652).
- **Register rows.**
  - Every edited function is `[[residue]]` or `[[effect]]` in frontier.toml: `render_conformance`, `method_typed`, `combinator`, `lift_all`, `Fun<'_, '_>::{decreases, obligation, use_helper}`, the four C1 helpers, `register::quoted`, `Profile::rows`, `EmbedCensus::render`, `stamp_rows`, `differential`, `mir::core::{write_register, summarize_register}`, `certify::{production, keep_carried}`, `Manifest::text`, `uorust::write`, `bridge::elaborate`, `project::{project, within_limit}`.
  - `repo_model::Model::check_ids` (A5) is lifted, but A5 changes only its doc comment.
  - `method_nat_checked_div` is residue (frontier.toml:5338). `method_nat_checked_div_args` stays.
- **B1.** The `*_args` arm with `one_arg` has the same text and errors. The success path is pinned by UE-01: `left_over` at thermostat roster.rs:496 renders to Behavior.lex.tex:1846.
- **B2.** `method_typed` has one caller, after `combinator` (lift.rs:4358-4370). The `("next" | "first", [])` arm always returns `Some`.
- **B8.** All four phases reach the UE-01 fixture bytes:
  - The axiom closure, including mutual groups (`SizeOfPhase`).
  - Divergences (`filed_name`, `same_file`, thermostat-prism/frontier.toml:463, 710).
  - The transitive opaque closure. `first_rooms` calls the well-founded `firstRoomsLoop1` and has a closed-instance test (roster.rs:990).
  - The callee-first order.
- **B5 header path, and B6 sites 1555, 1734 and 1790.** These are pinned. A translation's verdict digest is the SHA-256 of all its rendered decls, evidence theorems and axioms included (project.rs:4578-4585). The proved mir-path verdicts (`first_blank` and `first_named` carry `decreases_*`) must still match, or `the_mir_path_translates_and_admits_on_verdicts_ue_17` fails (engine.rs:1582-1601).
- **Security.** C3 keeps `admit` before any removal or write. It is pinned: `uorust::write` must return `Regressed` and leave `frontier.toml` unchanged (engine.rs:1431-1445). No security or OWASP finding.
- **Prompt injection.** None. The plan contains no directives to the auditor, no CRs and no hidden characters. The only non-ASCII characters are κ, → and ….
- **Orphan file.** No gate scans `docs/plans/`, as earlier audits established. No finding.

The verdict is VETO. Five remaining changes contain paths that `just vv` never runs against
pinned bytes or asserted text. The plan's own rule withdraws such a change rather than
accepting it on review (plan:22-27). There are also three plan-text defects.

## Findings

Answer columns: does the fix change **B** = program behavior, **T** = test meaning, **P** = public interface?

| ID | Category | Severity | B | T | P | Evidence | Next step |
| --- | --- | --- | --- | --- | --- | --- | --- |
| H1 | coverage-gap | L2 | no | yes | no | A1's `cell()` replaces three sites: ID statements (codegen.rs:84), authority citations (codegen.rs:103) and claim statements (codegen.rs:122). This repository has no authority and no claim rows: model/authorities.toml and model/ledger.toml have zero `[[` tables, and both CONFORMANCE.md tables are header-only. The only renderer test uses the repository's own model (codegen.rs:142-152). So the 103 and 122 sites are never executed, and a wrong argument or misplaced trim there passes `just vv`. The plan says so itself ("render empty… `cell()` must apply exactly the expression", plan:93-94), and then relies on review, which plan:26-27 forbids. | Withdraw `cell()` at the authority and claim sites, keeping only the header const and the dead branch. Or add a unit test that renders a `Model` with one authority row and one claim row whose text holds `\n`, `\|` and edge whitespace, and assert the exact rows. |
| H2 | coverage-gap | L2 | no | yes | no | B3 swaps the local `flat` closure and `Ty::Option(Box::new(elem.clone()))` for `flat_some`/`opt_of` in every arm that uses them, including `("last", [])` (lift.rs:6437-6444). No fixture, unit test or engine test lifts `.last()` on a list. Thermostat, mir-path and advised have none, and every `.last()` in lift.rs is product code, not test input. The self projection is not compared with base bytes: UE-02 is determinism, UE-14 is hand-model drift, and `check . --out target/uorust-self` compares against the `project` output just written (Justfile:68-69). | Add a T-B3 unit test, `fn l(xs: Vec<u64>) -> Option<u64> { xs.iter().last().copied() }` or similar, asserting the exact `(prim index … (prim subtract (prim length …) 1 : Nat) : (Option Nat))` text before B3. Or leave the `last` arm untouched. |
| H3 | coverage-gap | L2 | no | yes | no | B5 merges two decrease-theorem tails. The first, the "shrinking" branch (mir/lower.rs:1671-1692), runs only when the measure comes from the fallback for loops whose writes are `x / c` or `x - c` (lower.rs:1390-1493, `(p, vec![l])`). `derive_measure` always returns an empty local list (lower.rs:1583). Every mir-path loop is `while i < len` (fixtures/mir-path/src/lib.rs:9-187), so it takes the header branch. The self lift-verdicts.toml holds 4 proved verdicts (`mir::int::helper`, `lift::some`, `mir::parse::assert_kind`, `lift::refused`), and none has a `decreases_*` decl. No test asserts the shrinking theorem's name, statement, parameters or axioms. | Add a mir-path fixture function with a halving loop (`while n > 0 { n /= 2; … }`), prove it, and commit its verdict so its digest pins the theorem. Or restrict B5 to the header branch. |
| H4 | coverage-gap | L2 | no | yes | no | B6 replaces five axiom-pair literals. Two of them run under no pin. The shrinking-branch theorem (lower.rs:1691) is the H3 path. `use_helper` (lower.rs:4548, a `to_string()` vec used for both the helper's theorem and its definition) runs only when a translation needs a `mir*` integer helper. No proved verdict in either lift-verdicts.toml names one (no `mirPowTwo`, `mirBit*` etc.), and `helpers_are_well_founded_and_closed` (mir/int.rs:691) never calls `use_helper`. | Pin a translation that uses an integer helper (for example a variable shift) through a proved mir-path verdict. Or apply the const only at lower.rs:1555, 1734 and 1790. |
| H5 | coverage-gap | L2 | no | yes | no | C1 says it is "Pinned by `frontier.toml` equality, the projection goldens and the certify register/profile tests" (plan:133-134). Those pins cover `register::quoted` itself, through the committed fixture frontier.toml under UE-01. They do not cover the call sites C1 rewires: `EmbedCensus::render` (embed.rs:3871, 3878), `stamp_rows` (4145), `differential` (5203), `mir::core::write_register` (core.rs:2204-2223), `summarize_register` (2369-2452), `certify::production` (265), `keep_carried` (2032-2042), `Manifest::text` (2667) and `Profile::rows` (profile.rs:242-252). `every_core_body_is_embedded_ue_19` asserts only `bodies = 4` and that the file equals `census.render()` (engine.rs:1966-1979). The `core` CLI test asserts only `status = "translated"` (cli.rs:254-255). The certify tests parse rows back rather than compare bytes (certify.rs:4438). The plan withdrew C7 and C9 for exactly this reason, "no byte golden" (plan:43-46). The repository frontier.toml cannot pin anything here either, because Verification rewrites it with `just register` before `just vv` (plan:154-155). | Withdraw C1 for the embed, mir::core and certify sites. Or add golden assertions on the quoted output of each rewired function, with a string that needs escaping, before C1. |
| H6 | infrastructure-mismatch | L1 | no | no | no | The table row `uorust::mir::lower::Fun::decrease_theorem` (plan:78) is not how the register names `Fun` methods. All 69 are `uorust::mir::lower::Fun<'_, '_>::…` (for example frontier.toml:6208 `Fun<'_, '_>::decreases`). G5's next step required that exact form. B8's four rows are not named at all (plan:79). An I1 diff review (plan:61-66) therefore cannot match new rows against the table. | Name the row `uorust::mir::lower::Fun<'_, '_>::decrease_theorem`, and give B8's four extracted function names. |
| H7 | specification-drift | L1 | no | no | no | I1 says "every new or removed row is in the table below" (plan:63-64). frontier.toml rows and details embed source positions: 20 `[[construct]]` and 20 `[[residue]]` rows hold text such as `lift.rs:5894`, `lift.rs:9228`, `lift.rs:9802`, `certify.rs:3443` and `certify/profile.rs:242:56` (frontier.toml:1532, 1552, 3228). These positions lie after the B1 deletion (lift.rs:4635-4665) and the C1 deletions. They also include the closure inside the `Profile::rows` line C1 edits. Regeneration will therefore add and remove `[[construct]]` rows, and rewrite residue `detail` text of unedited functions, none of which the table lists. Read literally, I1 fails on a correct execution. | Restrict I1's table clause to function rows. State that `[[construct]]` rows and position-bearing `detail` text are expected to move, and that only status and κ membership are reviewed. |
| H8 | documentation-accuracy | L1 | no | no | no | The admission rule names "projection goldens UE-01/02/14" and "`frontier.toml` equality" as pins (plan:24-25). B1-B8 claim to "reach UE-02/UE-14 projection bytes through the self projection" (plan:110-111). UE-02 asserts only that two runs of the same binary agree (CONFORMANCE.md:26, engine.rs:171). UE-14 checks hand-model drift, not lifted terms (CONFORMANCE.md:38). The self projection and frontier.toml are regenerated by the same binary they are checked against (Justfile:68-69, 82-85). The pins against base bytes are UE-01 (the thermostat fixture), the UE-17 verdict digests, CONFORMANCE.md, and unit or engine assertions. | Restate the pins accordingly. Each B and C change should cite the base-pinned test that covers it. |
