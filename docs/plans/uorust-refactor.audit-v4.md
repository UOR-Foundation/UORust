# Audit v4: behavior-preserving refactor plan

**Verdict**: VETO
**Target**: docs/plans/uorust-refactor.md
**Judged content SHA-256**: 70a7c62b3d189ce7929e585a9d047a3364deb757595edf15998b0bd46e1819d1

Auditor: independent audit pass. This is a host document, not a `.qortara/` record.
Repository: `C:\dev\UOR\UORust`, branch `refactor/qortara-governed`, HEAD `daa24878`.
`governance/qortara-template-sync` resolves to the same commit
(`daa24878f17cb56cc530d468c6c87227e5ba7989`), the base the plan names (plan:3). The plan is
uncommitted (156 lines). The hash above was computed with `sha256sum` over the bytes read.
Earlier audits: `uorust-refactor.audit-v1.md` (VETO, `c786866e…`), `-v2.md` (VETO,
`8c6f4027…`), `-v3.md` (VETO, `04f57bb9…`).

## Prior findings

| ID | Status | Evidence |
| --- | --- | --- |
| F1 | moot | A2, A3, A4 withdrawn (plan:43). |
| F2 | moot | C7 withdrawn (plan:47). |
| F3 | corrected | T-B1 asserts both error texts (plan:103-106). They match `one_arg` (lift.rs:5264-5272) and `method_nat_checked_div` (lift.rs:4645-4649). |
| F4 | corrected | Expected substrings (plan:101-103) match `none_of`/`some_of` (lift.rs:5240-5248) and the `_args` arm (lift.rs:4738-4744). |
| F5 | moot | C1 withdrawn (plan:46). |
| F6 | corrected | I1 is stated as κ membership plus a row table (plan:66-88). |
| F7 | moot | C7 withdrawn. |
| F8 | corrected | C2 cites bridge.rs:1205, 441, 1113. All three are correct. |
| F9 | corrected | I3 has no dangling qualifier (plan:76-78). |
| F10 | moot | A3 withdrawn. |
| G1, G2, G6 | moot | A2 withdrawn. |
| G3 | moot | A7 withdrawn (plan:44). |
| G4 | moot | A6, C4, C11 withdrawn (plan:44-48). |
| G5 | corrected | `main::` names are gone, C1 is withdrawn, the B5 row is gone, and the B8 rows are named (plan:84-88). |
| G7 | corrected | The Revision section cites audit-v1, -v2 and -v3 with the right hashes (plan:150-156). |
| H1 | moot | A1 withdrawn (plan:42). |
| H2 | moot | B3 withdrawn (plan:44). |
| H3 | moot | B5 withdrawn (plan:45). |
| H4 | moot | B6 withdrawn (plan:45-46). |
| H5 | moot | C1 withdrawn (plan:46). |
| H6 | corrected | The four B8 rows use the `uorust::lift::<free fn>` form the register uses (for example `uorust::lift::lift_all`, frontier.toml:5176, and `uorust::lift::method_nat_checked_div`, frontier.toml:5338). |
| H7 | partly corrected; see K1 | I1 now accepts rows whose *only* change is a line position (plan:70-71). C8, B1 and B8 also change register text that is not a line position. |
| H8 | partly corrected; see K4 | The admission rule names the real base pins (plan:25-33). Not every change cites its pin, and C3 cites one that does not cover it. |

## Summary

These claims were re-checked against the source and hold:

- **Citations.**
  - `certified!` sites: lift.rs:1814, 2034, 3396, 5298, 6038; mir/lower.rs:686, 6973.
  - Q1 (mir/lower.rs:1741, 1772) and Q2 (bridge.rs:1185-1187).
  - CM-02 (meta.rs:163), the xtask task list (main.rs:39-51), the splitters (bridge.rs:309, layouts.rs:578).
  - κ (register.rs:55-57, 201), `admit` (register.rs:537), the token source (ingest.rs:1065-1068).
  - The test helper `lift(src, &types) -> Result<String, Residue>` (lift.rs:11652) and the test neighbourhood (lift.rs:12352-12368).
  - The `vv` recipe list (Justfile:11), matching plan:146. `bootstrap.yml` runs `exec just vv` in the image locked in `prismpm.lock` (bootstrap.yml:78-118).
- **Register status.** Every edited function is residue or effect:
  - `method_typed` (frontier.toml:5404), `method_nat_checked_div` (5338), `lift_all` (5176), `uorust::write` (effect, 9341), `project::project` (7744), `project::within_limit` (7840).
  - `repo_model::Model::check_ids` (A5) is lifted (9486), but A5 edits only its doc comment, which is outside the tokens (ingest.rs:1065).
  - No edited function has a `lift-verdicts.toml` row.
- **Row naming.** Closures are not function rows (only 8 `function =` rows contain "closure", all named fns). B8 therefore adds exactly the four function rows.
- **A5.** The doubled clause is real (model/src/lib.rs:86-88). Doc only.
- **B1.** Pinned by T-B1 and UE-01:
  - For one argument, the `_args` arm with `one_arg` produces the same text and errors. No intervening arm matches `checked_div`/`checked_rem` (lift.rs:4531-4543).
  - The fixture's `total.checked_rem(rooms)` (thermo-core/src/roster.rs:496) renders at thermostat-prism/src/Behavior.lex.tex:1846.
  - `checked_div` and the error texts are pinned only by T-B1, which is adequate.
- **B2.** The arm is unreachable:
  - `method_typed` has one caller, `method_on` (lift.rs:4370), after `combinator` for every `Ty::List` (lift.rs:4358-4364).
  - `("next" | "first", [])` returns `Some` with identical text (lift.rs:6431-6435).
  - The combinator path is pinned by UE-01 through `readings.first()` (band.rs:72, rendered at Behavior.lex.tex:790).
- **B7.** A missed rename of the local `evidence_axioms` (lift.rs:10755-10779) would hit the function item and fail to compile.
- **B8.** All four extracted phases reach UE-01 bytes:
  - Callee and group axioms: `holdsOffMutual` members carry `axioms: [Quot.sound, propext]` (Behavior.lex.tex:2683-2711).
  - Divergence propagation: `filed_name` and `same_file` (thermostat-prism/frontier.toml:463, 710).
  - Opaque closure: `firstRooms` calls the well-founded `firstRoomsLoop1` (Behavior.lex.tex:2307-2319). It has a Rust closed instance (roster.rs:990) that the projection omits only because of the transitive closure (project.rs:4202).
  - Order: decl order in Behavior.lex.tex.
  - Correction to audit-v3: its `SizeOfPhase` example comes from project.rs:5006, not from `lift_all`. The `holdsOffMutual` evidence above replaces it.
- **C2.** `from_millis(120_000)` equals `from_secs(120)`. `SCREEN_TIMEOUT` is used only at bridge.rs:1526.
- **C8.** Compiler-checked rename. The value is unchanged. Uses: project.rs:479, 5892, 5897, 6914, 7161.
- **Security and OWASP.** C3 keeps `admit` before any removal or write (lib.rs:203-205). This is pinned by UE-16: `Regressed`, and `frontier.toml` is unchanged (engine.rs:1433-1445). No check is weakened. No finding.
  - Outside the plan: the removal loop deletes `out.join(path)` for each manifest line (lib.rs:207-213) without confining `path` to `out`. This is pre-existing and not changed by C3. The owner may want it recorded.
- **Prompt injection.** None. No text addresses the auditor. There are no CRs. The only non-ASCII characters are κ, → and ….
- **Dependencies, architecture, orphans.** No new dependency and no new module edge. No gate scans `docs/plans/`. No finding.

The verdict is VETO:
- C3 changes a path that no `just vv` check pins. Under the plan's own rule (plan:32-33), C3 must be withdrawn.
- I1 cannot pass literally on a correct execution.
- There are three text defects.

## Findings

Answer columns: does the fix change **B** = program behavior, **T** = test meaning, **P** = public interface?

| ID | Category | Severity | B | T | P | Evidence | Next step |
| --- | --- | --- | --- | --- | --- | --- | --- |
| K1 | specification-drift | L1 | no | no | no | I1 accepts unlisted changes only when "the only change is a source line position" (plan:70-71). The changes also rewrite other register text. **C8:** `within_limit`'s residue detail reads `mir::Operand::Constant: the constant item project::LEXLEAN` (frontier.toml:7842). The same text is a `[[construct]]` row (frontier.toml:1922) and a header line (frontier.toml:105). All three become `…LEXLEAN_TOML`. **B1:** removing `method_nat_checked_div` (detail `mir::Ty: core::fmt::rt::Argument<'_>`, frontier.toml:5340) lowers that construct's `functions = 61` (frontier.toml:496-499) and the header's residue-by-cause tallies (frontier.toml:8-15). **B8:** four new residue rows raise construct counts and may add per-closure header lines (the `mir::AggregateKind::Closure` list, frontier.toml:15-80). None of these is a line-position change or a table row, so I1 fails as written. This is H7's residue. | Restate I1: review only function rows (status and κ membership against the table). State that `[[construct]]` rows, their counts, header tallies and `detail` text are expected to change. |
| K2 | specification-drift | L1 | no | no | no | The Goal promises no change to "any output, error text, generated file, lift status" (plan:22-23). `frontier.toml` is generated ("written by uorust", frontier.toml:1), and the plan itself adds and removes rows in it (plan:80-88). The self projection under `target/uorust-self` also changes. | Amend the Goal: no change to the UE-01 fixture projection, CONFORMANCE.md, verdict digests, asserted error text or any incumbent row's status. `frontier.toml` changes only as I1 states. |
| K3 | coverage-gap | L2 | no | yes | no | C3 merges the two `manifest.is_file()` blocks (lib.rs:203-215). That moves the loop that deletes the earlier projection's listed files (lib.rs:207-214). No `just vv` check observes that deletion. UE-06 rewrites with identical files and checks currency (engine.rs:396-399), so a dropped or misplaced loop still passes, because the files are overwritten anyway. UE-16 stops at `admit` before the loop (engine.rs:1433-1445). UE-01 as the plan defines it calls only `uorust::check` (engine.rs:155-168). The `uorust` and `register` recipes `rm -rf` the output first (Justfile:66, 71, 83). The CLI test writes into a fresh directory (cli.rs:24-37). The plan's rule withdraws a candidate with an unpinned path (plan:32-33). | Before C3, add an engine test: `write` a projection with files {A, B}; `write` one with {A} into the same directory; assert that B is gone, that A and the manifest are current, and that a foreign directory is still refused. Otherwise withdraw C3. |
| K4 | documentation-accuracy | L1 | no | no | no | H8's next step required each change to cite the base-pinned check that covers it. **C3** cites UE-01 (plan:129), but UE-01 as defined (plan:27) never calls `uorust::write` (engine.rs:155-168). **B2** and **B7** cite nothing (plan:114-116). Their actual pins are UE-01 (list `.first()`: band.rs:72 → Behavior.lex.tex:790; `lift_all` output) and the compiler. | Cite UE-01 (fixture `.first()`) for B2, and the compiler plus UE-01 for B7. For C3, drop UE-01 and cite UE-06, UE-16 and the K3 test. |
| K5 | documentation-accuracy | L1 | no | no | no | B8 says it extracts "the four commented phases of `lift_all`" into the four tabled functions (plan:117-120). The commented phases are callee axioms (lift.rs:10876), mutual-group axioms (10890), divergence (10901) and order (10955). The plan merges the first two into `axioms_with_callees` and adds `opaque_closure`. That block (lift.rs:10920-10953) has no phase comment. The axiom write-back into `decls` and the `#` retains (lift.rs:10913-10919) are assigned to no function. | State the lift.rs line range each extracted function takes. Say where lift.rs:10913-10919 stays: in `lift_all` between `divergences_from_axioms` and `opaque_closure`. |
