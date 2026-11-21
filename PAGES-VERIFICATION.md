# Pages repository identity: planted-defect verification

**Scope:** UORust [Pages repair PR #39](https://github.com/UOR-Foundation/UORust/pull/39), after the transfer from `afflom/uorust` to `UOR-Foundation/UORust`.

**Production baseline:** [run #44853624994](https://github.com/UOR-Foundation/UORust/actions/runs/44853624994) on merged PR #38 successfully built and signed the holo, then `bootstrap/page.mjs` rejected `site/app.json.repository = https://github.com/afflom/uorust` while `GITHUB_REPOSITORY=UOR-Foundation/UORust`. Upload and publish were correctly skipped.

## Negative controls for the *new* early CI preflights

`scripts/ci/test_pages_identity.py` extracts the **actual** Node `-e` script from **each** of `.github/workflows/pages.yml` and `.github/workflows/try-page.yml`, and runs it against independently created temporary `site/app.json` fixtures. The test requires exactly one preflight in each workflow and identical guard bodies. It runs in both workflow jobs before the preflight, and via `crates/conformance/tests/pages_identity.rs` in `cargo test --workspace` inside mandatory `just vv`. This uses the factory graph's existing `test` obligation and declared `crates/` and `scripts/ci/` inputs, without adding a second, unmodeled dependency.

| Case | Input value of `repository` | Required outcome |
| --- | --- | --- |
| Planted transferred-repo defect | `https://github.com/afflom/uorust` | Exit nonzero; diagnostic `site/app.json repository mismatch: https://github.com/afflom/uorust != https://github.com/UOR-Foundation/UORust` |
| Restored correct control | `https://github.com/UOR-Foundation/UORust` | Exit 0 |
| Missing property | Absent | Exit nonzero with `repository mismatch` |
| Malformed property | Numeric `123` or empty string | Exit nonzero with `repository mismatch` |
| Absent/duplicate preflight | Workflow source mutated or guard removed | `workflow_guard` fails before running fixture |
| Plant `if: false` after guard script | Identity step disabled | Complete-step verifier refuses `trailing configuration` |
| Plant `continue-on-error: true` after script | Guard failure ignored | Complete-step verifier refuses `trailing configuration` |
| Plant `if: false` before `shell: bash` | Identity step disabled | Complete-step verifier refuses `step configuration` |

**Reproduce:** `python3 -I -m unittest discover -s scripts/ci -p 'test_pages_identity.py' -v`. The mandatory `just vv` invokes the same tests through `cargo test --workspace` and the Rust conformance bridge (no new `vv` recipe or factory obligation). The planted wrong-repository fixture must **fail the workflow's extracted preflight**; the enclosing regression test passes only when that refusal and the restored success are both observed.

**Evidence boundary:** These controlled tests establish behavior of the checked-in workflow guard. They do **not** attest to production publication. The next `main` Pages run must separately pass holo/provenance, image stamping, page assembly, Lighthouse, responsive screenshots, upload and deployment, followed by a live URL check. Do not weaken `bootstrap/page.mjs` or any signed provenance check.
