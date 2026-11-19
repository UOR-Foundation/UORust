# Template repository verification

## Native feedback regressions (UE-40 through UE-44)

These checks use the immutable SDK selected by `prismpm.lock`. Each row was
registered with its scenario and an executable regression before its repair.
They are focused development evidence; release acceptance remains `just vv`.

| ID | Reproduced defect | Restored check |
| --- | --- | --- |
| UE-40 | A renamed unique workspace type did not lift, while an imported name could bind a function's generic parameter. | `renamed_unique_types_preserve_imports_and_generic_shadowing_ue_40` checks the original type, generic shadowing, and actual SDK verification. |
| UE-41 | A compiler-derived signed structure field was admitted into a native tuple with a different integer representation. | `native_aggregates_check_field_representations_ue_41` refuses incompatible tuple, structure, variant, array and option cases with typed field diagnostics; the compatible projection passes SDK type checking. |
| UE-42 | Native loop bodies transporting lock-admitted registry values produced no candidates. | `native_registry_carriers_require_locked_readings_ue_42` proves all five carrier functions through the SDK, rejects unmodeled JSON equality, and rejects a changed archive checksum. |
| UE-43 | Native trials checked unrelated modules and retained no structured SDK result. | `native_verification_scopes_feedback_and_reuses_decisions_ue_43` proves three candidates, refuses an overflow candidate, records only Behavior and Prelude, and replays identical verdicts with zero verifier runs. |
| UE-44 | Lean rejected a countdown carrying an invariant Boolean with `Unknown identifier x1`. | `native_loop_state_preserves_invariants_in_evidence_ue_44` proves the compiler-derived function using one changing loop-state parameter. The SDK checks the unreduced decrease obligation exactly before proving it. |

Run these regressions with `cargo test --locked -p uorust --test engine native_`.
The existing policy-hint regression also exercises corrected hints and unchanged
replay. Native trials now run SDK type checking before Lean verification and
write the actual SDK JSON to `uorust-proof-result.json` in their project-owned
scratch directory. Neither this feedback nor prior axiom hints confer proof
status: a changed translation digest still requires the SDK's successful verdict.

The MIR fixture was refreshed by the actual prover after loop-state lowering
changed its translations. Eight translations were proved and one refused in
three verifier runs; two unchanged proved rows were retained. In particular,
`walk::any_above`, previously refused for its invariant capture, now proves.
The fixture's overflow refusal remains a refusal. CLI and engine replay checks
require zero new verifier runs for the resulting register.

The complete gate is not represented as passed by these checks. The prior full
run's self-lift alias regression was repaired separately, and the translator's
UE-18 certification boundary still requires complete release acceptance.

## Collection transport and proof-history reuse (UE-45 and UE-46)

`native_hash_collections_reuse_unordered_carriers_ue_45` initially found no
native standard-map candidate. Its repair proves five compiler-derived
functions through the SDK: default and custom-hasher maps and sets, plus a
workspace homonym. Hasher inspection, iteration order and an unsupported
element type remain refused. This adds passive carriers, without a model for
hash-builder operations.

`native_verdicts_survive_compiler_input_selections_ue_46` initially reduced
the actual eleven-row fixture register to one row when given one compiler
body. It now retains all proved and refused rows across full, single-body and
full selections, with zero verifier runs throughout. A different digest for a
live translation still invalidates its previous decision; the policy-hint
regression requires actual SDK verification after that change. Historical
cache size is not a count of currently lifted functions: the current frontier
remains the authority for eligibility.

## Ordinary native float scopes (UE-47)

The initial `native_workspace_floats_share_source_readings_ue_47` run lacked
the ordinary f32 transport candidate. MIR target scopes now receive the same
core-derived primitive formats as source lifting, and core/std facade paths
resolve their supported inherent methods. Eight actual-SDK proofs cover float
transport, arithmetic, finite tests, bit conversions and a negative-zero
literal. Primitive headers retain their bit carriers despite a qualified
workspace homonym. `native_workspace_floats_preserve_root_names_ue_47` checks
a root declaration named f64 through its own Boolean field and SDK type
checking. Unsupported trigonometry remains unlifted. The existing literal
conformance test compares f64 positive and negative zero with Rust's actual
bit patterns; it does not assume a particular printed form of the model.

This repository's `just vv` instantiates the universal verification policy in
`VERIFICATION.md` for the template itself, which is a Prism application: the
template contract checker in `app/`.

| `just` recipe | Enforces | ID classes |
| --- | --- | --- |
| `just fmt-check` | the diff is reviewable | --- |
| `just model` | R1, R4, R5 and the bootstrap trust root, in one `xtask validate` | `CM-01` |
| `just template-check` | immutable SDK selection and template drift, checked by the SDK | --- |
| `just lint` | Clippy at `-D warnings`, every target, every optional feature | --- |
| `just test` | the workspace suite, including the R3 meta-gate | `CM-02`, `CM-03` |
| `just deny` | R6 over the dependency graph | --- |
| `just app` | the Prism application: LexLean format and lock, then `prismpm verify`, which builds the holo and checks it | `SM-01`, `SM-02` |

Each check runs once per `just vv`. `just bdd` runs only the meta-gate while a
scenario is being written; `just test` already includes it.

The register holds UORust's claims (`SM-01` to `SM-04` for its own model,
and the engine's rows), so the meta-gate checks real rows, real scenarios and
real tests. It becomes an error as soon as an ID, scenario, or test is added
without the other two.

## Planted defects

| Gate | Planted defect | Result |
| --- | --- | --- |
| `check-model` | `CONFORMANCE.md` disagrees with the register | rejected |
| `audit-deferral` | a deferral marker in a crate and in the gate's own source | both rejected |
| honesty meta-gate | an ID with no test | rejected |
| `self-model-check` | a hand edit of a generated module (a stakeholder renamed in `app/src/Architecture.lex.tex`) | rejected |
| `self_model_sm_01` | a route dropped from `app/src/Routes.lex.tex`, or a vector of `model/routes.toml` missing from the application | both rejected |
| `prismpm verify` | an acceptance vector set without the exact guest-allocation boundary | rejected (PP2009) |
| UE-18 ratchet (`the_hand_written_translator_only_shrinks_ue_18`, `just ratchet`, the pre-commit hook) | a register with one more uncertified translator function than its parent's (`a_grown_translator_is_refused_ue_18`; in `certify`, a new residue function and a certified function whose κ changed) | rejected, naming `uorust::mir::lower::planted: residue (mir): mir::Rvalue::Use` |
| UE-18 differential and fixpoint (`certify agree`, `just certified-gate`) | a hand-written output one byte apart from the certified one's (`the_differential_refuses_one_byte_apart`) | rejected, naming both κ |
| UE-18 certified sources (`certify sync`) | a package source whose SHA-256 is not the one the bundle's result records | rejected |
| UE-18 certified dispatch (`certified!`, `--cfg=uorust_lift`) | a hand-written body whose certified dispatch the lift reads (`the_lift_reads_no_certified_dispatch`) | rejected: the dispatch statement is left out of the lifted syntax |
| `audit-bootstrap` | a floating/copied action, mutable SDK tag, changed universal file, narrowed boundary, shallow/credential-retaining checkout, copied project renderer, omitted standards lock, acceptance bypass, deploy-job signing privilege, incomplete release lifecycle, or a bootstrap that drops a native architecture from main or tags, cancels or replaces a main run, or skips runs by path, condition or matrix exclusion | all rejected |

`audit-deferral` reads every crate and `xtask`, including itself. Its token
construction therefore cannot exempt the very gate in which a deferral could
otherwise be hidden.

## One pull-request run, complete native coverage on main and tags

Every instance inherits this bootstrap. Before this change, each pull request
and each main push ran the complete `just vv` on both native runners, and
nothing cancelled a superseded pull-request run. Those runner minutes are
shared across the organization (UOR-Foundation/PrismPM#187). Repeating a
deterministic gate on the same inputs adds time, not evidence
(UOR-Foundation/PrismPM#210, UOR-Foundation/prism-code#25).

A pull request now runs the complete gate once, natively on amd64, and a newer
push to the same pull request cancels its older run. Every main push and every
tag runs the complete gate on both native architectures. Each non-pull-request
run has a concurrency group of its own. GitHub keeps at most one pending run
per group, so a shared main group would replace a queued commit's run before
it produced a verdict. The pull request's arm64 evidence therefore arrives on
the merge commit, before any tag of it publishes.

`native_coverage_is_complete`, part of `audit-bootstrap`, keeps that saving
sound. It requires the main and tag triggers, one of exactly two runner
matrices, and either no concurrency or exactly the pull-request-only
cancellation. The two admissible matrices are both architectures on every
event, or amd64 on pull requests and both elsewhere. The gate also rejects job
conditions, `continue-on-error`, matrix inclusions or exclusions, and path,
branch or tag filters.

The new test, `dropped_native_coverage_plant_is_rejected`, failed against the
previous workflow bytes: `assertion failed: native_coverage_is_complete(bootstrap)`.
It passes against the changed workflow, and rejects eight planted defects:
- arm64 removed from main and tags;
- the two matrix legs swapped;
- the tag trigger removed;
- unconditional cancellation;
- one shared main group;
- a `paths-ignore` filter;
- a job condition that skips arm64;
- a matrix exclusion of arm64.

Running both architectures on every event without concurrency stays
admissible, because it is stronger than the policy.

All 15 `xtask` tests, including both invoked Node cases, passed with the pinned
Rust 1.97.1 on `x86_64-pc-windows-gnu`, locked. Formatting passed, and
all-target Clippy with warnings denied passed for `xtask`. These ran outside
the SDK image, because no Docker host was available. Workspace Clippy on that
target also reports `result_large_err` in `repo-model` on unchanged `main`; it
is not part of this change. The change's own pull-request run on GitHub
evaluated both expressions and started exactly one job,
`acceptance / ubuntu-24.04`. That job then stopped before the gate, as every
template run does, because the template commits no `prismpm.lock`. The main,
tag and cancellation behavior have not yet run on a hosted runner.

## Platform-indexed SDK bootstrap

The `/2` SDK lock renderer delegates index and inventory validation to the
SDK-owned `platform-lock.mjs` helper. Transport extracts inventory and standards
files from each exact `linux/amd64` and `linux/arm64` image child using Docker
without executing the foreign architecture. The template native audit compares
the detected platform's complete inventory and bytes; the legacy `/1` branch
keeps its original exact comparison.

All 10 template `xtask` tests and all-target Clippy passed, locked and offline,
inside the existing PrismPM SDK as UID 1000. The new negative test rejects a
swapped native inventory, missing platform, and legacy architecture drift.
Shell and Node syntax checks passed. An isolated ignored Cargo target was used
because older root-owned fingerprints prevented reuse of the default target;
those unrelated artifacts were not changed. These focused tests do not claim
the template's full `just vv` has accepted an unpublished candidate SDK. A real
immutable multi-platform candidate and policy-input commit are still required
before generating and committing the new locks.

The initial standards binding adds two executable Node cases, also invoked
by the ordinary `xtask` test suite: absent and matching locks preserve the
exact supplied bytes; a differing existing lock is rejected without changing
it. The complete 11-test `xtask` suite, all-target Clippy and shell/Node syntax
checks passed inside the SDK as UID 1000. This is byte-binding evidence, not a
claim that the project has adopted or conforms to every referenced standard.

## Private Buildx state with read-only credentials

Foundry bootstrap run `35104180230` failed on both runner architectures before
the repository gate: Buildx tried to create `.docker/buildx` inside the
read-only credentials mount. The exact amd64 SDK candidate
`ghcr.io/uor-foundation/prismpm-sdk-candidate@sha256:c2e0e50437e13d9b2e382d3af4ae7a962b469721b9b80f14f215d9254e8ed78f`
reproduced `mkdir /tmp/prismpm-home/.docker/buildx: read-only file system`
under UID 1000, with network disabled, an empty test-only credentials file
mounted read-only, the socket's group, and the same private `/tmp` tmpfs.
No owner credentials were read or mounted.

With `BUILDX_CONFIG=/tmp/prismpm-buildx` and `umask 077`, actual `docker buildx
version` and `docker buildx inspect --bootstrap` passed. The state directory
was mode 0700, credentials stayed read-only and byte-identical, and no state
was created inside the credential directory. The default Docker driver was
inspected; no builder was created, selected, or removed. The disposable
container's state vanished when it exited. This is native amd64 execution,
not a claim that the updated hosted ARM job has already passed.

The native guard's planted missing-state regression failed before the fix.
It now rejects missing/duplicate configuration, paths inside either the
credentials directory or its parent, missing tmpfs and permissive creation.
An additional real-workflow mutation pair reproduced that the initial guard
accepted an appended `umask 022` before Buildx and a `umask 077` moved after
inspection. Both now fail: the scoped container-script check requires exactly
one umask directive, exactly 077, before its first Buildx command. The final
12-test xtask suite, both invoked Node tests, formatting and Clippy passed
again; the actual Docker workflow was already correct and remained unchanged.
All 20 workspace Rust tests (including all 12 xtask tests), both invoked
Node tests, locked offline Clippy, all-features/all-target checks, formatting,
model comparison and YAML/embedded-shell syntax passed under that exact SDK.
The YAML check used the SDK's existing `js-yaml`; no tooling was installed.
Raw RED/GREEN logs are retained under `target/template-buildx-*`.

The template remains unbound: these component checks do not replace its
required complete `just vv`, accept a production SDK, or waive universal
policy. Downstream repositories must receive the changed workflow and policy
through their normal reviewed template update.

## SDK-owned dependency maintenance

Dependabot uses canonical JSON-form YAML, checked structurally by
`audit-dependency-updates` within `audit-bootstrap`. Duplicate keys, malformed
values, missing or broader exclusions, and disabled maintenance are rejected.
Only the SHA-pinned `UOR-Foundation/PrismPM/action` and byte-bound
`.github/workflows/bootstrap.yml` use the reviewed SDK/template update flow.
Other Actions and Cargo dependencies retain their weekly schedules and groups;
both now request Conventional Commit prefixes.

GitHub documents [dependency ignores and path exclusions](https://docs.github.com/en/code-security/reference/supply-chain-security/dependabot-options-reference).
The upstream [SHA-pinned subpath parser](https://github.com/dependabot/dependabot-core/blob/dc25739e082c6d97399b476ad4eb06b5559895e7/github_actions/lib/dependabot/github_actions/file_parser.rb#L148)
confirms the exact action dependency name. The existing SDK's `js-yaml` 4.3.2
independently parsed the configuration and preserved the prior schedules and
groups; this inspection is not a new SDK API or general YAML conformance claim.

The previous configuration failed the new gate. A canonical broad `*` ignore
also failed the actual command; restoring the exact action passed. Negative
tests cover omitted/wrong/extra exclusions, disabled or duplicate ecosystems,
removed Cargo coverage, altered scheduling, and duplicate JSON keys.
The byte-wise SHA formatter preserves exact empty-string and `abc` digests
with unchanged sha2 0.10.9; the same correction fixes sha2 0.11 compilation in
the separately verified Foundry dependency update. No dependency was changed
here. Logs are retained under `target/template-dependabot-*` and
`target/template-maintenance-*`.

In the exact native AMD64 SDK above, all 22 workspace Rust tests and both
invoked Node tests, formatting, Clippy, all-feature/all-target compilation,
model/limit/deferral audits and dependency checks passed. Full `just vv`
passes the new policy check, then rejects the template's existing absent
`prismpm.lock`. No universal policy bytes changed or release gate was waived.

## Each check once, and the R3 scaffold

`just vv` used to run `check-model` and `audit-bootstrap` twice (once in
`template-check`, again inside `xtask validate`), the `repo-conformance` tests
twice (`test`, then `bdd`), and the workspace through two compilers that differ
only in features (`lint` at the default features, then `features`, a
`cargo check --all-features`). Repeating a deterministic check on the same
inputs adds time, not evidence. Now `model` is the one `xtask validate`,
`template-check` is only the SDK's two checks, `lint` is Clippy with
`--all-features` (so an optional feature is still compiled, with its tests, and
now also linted), and `test` compiles the default features. Every Cargo
invocation passes `--locked`, so no part of the gate can resolve a dependency
the lock does not name (R6). The `cargo vv` alias, which ran `xtask validate`
and so was not the gate its name claimed, is removed.

`cargo xtask new-capability <ID> <level> <suite> <statement>` (`just
new-capability`) writes the register row, the tagged scenario and a test named
`<suite>_<id>` that fails until the capability is built, then regenerates
`CONFORMANCE.md`. Its tests run the honesty meta-gate on a scratch copy of the
register: two scaffolded capabilities leave it clean, and a lowercase ID, a
missing hyphen or number, an unknown level, an uppercase or path-escaping
suite, an empty or two-line statement, too few arguments and a duplicate ID
are all refused with the register byte-identical and no suite file written.

On a copy of this repository, `just new-capability CX-01 build parsing ...`
produced files that passed `cargo fmt --check` and all-feature Clippy, and
`cargo test` then failed exactly at `parsing_cx_01`, as R3 requires. The 18
`xtask` tests, all-target all-feature Clippy, formatting and `audit-deferral`
passed with the pinned Rust 1.97.1 on `x86_64-unknown-linux-gnu`, locked,
outside the SDK image; the SDK checks run in the pull request's bootstrap job.

## UORust's route table

`app/` is UORust's own Prism model. Its application, modeled in
`app/src/Routes.lex.tex` on the `prismpm/text-application/3` profile with a
workbench (`app/src/Guide.lex.tex`, `app/src/Frame.lex.tex`), is the route
table of `model/routes.toml`: a request is one Rust fact's kinds, one per line,
and the answer is the one route they all name, `refused unknown-fact`,
`refused ambiguous`, or `error` for an empty, oversized, non-UTF-8 or
blank-line request. The theorems `routeTableIsTotal`, `routeFactsAreDistinct`,
`routeTableRefusesAmbiguity` and `routeIndexesArePositions` are decided over
the whole table. `self_model_sm_01` and `self_model_sm_02` read the model as
text and hold the table, `routeOf`, the acceptance vectors and the theorem
statements to `model/routes.toml`.

Constraints met on the way, each a refusal of the pinned SDK: a request of
exactly `guestAllocationMaximum` bytes must be a vector (PP2009); a text/3
command line needs an outcome named `error`, and the workbench purpose must
equal its description; generated code refuses a `List` used as an
intermediate value and a static list constant on the runtime path, so the
table is answered by an `if` chain (`routeOf`) and the list (`routes`) serves
the theorems; generated Rust has one namespace, so no two definitions share a
name across modules; and every definition states its exact axioms.

Locally, in the locked SDK: `lexlean verify --all` verifies all 19 modules,
including every decide theorem, and `prismpm --project app check` passes with
every facet (PP2012, PP2014 to PP2022, PP2101). `prismpm --project app verify`
builds and replays the holo and then stops in the SDK's pinned Hologram
oracle (`PP5301: attached portable View asset closure/order is not
canonical`): the oracle expects a View of exactly `app.css`, `app.js` and
`index.html`, and the SDK's own build attaches the brand files beside them.

## Prism accounts: the remembered passkey

The sign-in page (`identity/id.mjs`) remembers which passkey this device last
signed in with, so the browser asks for that one and skips its chooser, and
it names the device's authenticator ("Secured by Windows Hello"). The hint
must never decide that a device has no passkey: if it could, someone with an
account whose remembered passkey was removed would be sent to Create account
and end up with a second account.

| Gate | Planted defect | Result |
| --- | --- | --- |
| `npm run test:identity`, "a remembered passkey that is gone asks for any passkey, never straight to a new account" | the branch that forgets a remembered passkey which did not answer, removed from `run()` | rejected: `locator.waitFor: Timeout 10000ms exceeded` waiting for `#go-label` to read `Sign in`; restored, 20 of 20 pass |
| `npm run test:identity`, "a click the bar is redrawn under still opens the panel" | the frame before this fix, which opened the panel it was clicked on even after the bar was redrawn (found on holo-search: a first click right after load did nothing) | rejected: `not ok 2`, waiting for `#account-panel:popover-open`; fixed, 21 of 21 pass |
| same test, after the account panel became a centred dialog and the button reads "Continue with Windows Hello" | the same branch removed again | rejected: the button read `Create account` where `Continue with Windows Hello` was required; restored, 21 of 21 pass |

## The workspace page

The standard page became the workspace: a header row, the app's surface
beside a rail for its result, actions and history, and a spec that opens
beside the window (the frame's window and bar narrow by `--prism-aside`)
and loads only when opened. Plain dark replaced immersive as the default
theme. The page gates were run on three apps built with these files, as
`frame-sync.mjs --pin` gives them (Leap year, Luhn check, Legal chess, each
with its own holo and LexLean source):

| Gate | Result |
| --- | --- |
| `node bootstrap/screens.mjs site screens`, six size classes in three themes, the spec open and closed | every class passes in every theme, all three apps |
| `lhci autorun`, three runs each | 100 in all four categories, CLS 0, LCP about 1.21 s; 105 KiB (Leap year, Luhn) and 122,701 of 122,880 bytes (Legal chess, whose live page was already within 1 KiB of the budget) |
| `npm run test:identity`, `npm run test:try` | 21 of 21, 1 of 1 |

The screens gate's new check, that the workspace fits the screen, was first
written as "the page does not scroll". A planted defect showed it could not
fail: the page hides its overflow, so a workspace taller than the screen is
cut off rather than scrolled.

| Gate | Planted defect | Result |
| --- | --- | --- |
| `screens.mjs`, the workspace fits the screen (scroll only) | `main` made 4 rem taller than the screen | passed: the defect went unseen |
| same, checking that `main` ends inside the screen and holds its content | the same 4 rem | rejected in all 18 cases: `the workspace does not fit the screen: the page would scroll or cut it off`; restored, the three apps pass |

Lighthouse found two more before the change was done: the display's type
shrank once the script set its width (CLS 0.009 on Luhn check, fixed by
setting the first width in the hashed stylesheet), and `app.mjs` crossed the
unminified-JavaScript allowance (2,458 bytes against 2,048, fixed by moving
the spec into `site/spec.mjs`, loaded on first open, and shortening comments).

## A newer frame may add a file

`frame-sync.mjs --pin` judged a target commit's lock by this checkout's own
frame list, so the first frame to add files (`site/spec.css`, `site/spec.mjs`)
could not be pinned by any app that had an older script. The target is now
judged by its own frame: its `frame-sync.mjs`, matched to its lock, names the
files, read as text and never run, and each must sit under `bootstrap/` or
`site/` (or be `lighthouserc.json`). `--check` is unchanged. Run against real
commits and against mirrors served with `PRISM_TEMPLATE_RAW`:

| Case | Result |
| --- | --- |
| the old logic with the old fourteen-file list pins `128d80d` | refused: `frame.lock at 128d80d… is not a prism/frame-lock/1 lock of this frame` (the defect) |
| the new logic with the old list pins `128d80d` | pinned; `--check` then passes, 16 files |
| the new logic pins back to `76bdeb0` | pinned; `site/spec.css` and `site/spec.mjs` reported as no longer part of the frame |
| a mirror serves `site/app.mjs` with one line added | refused: its digest against the lock's; nothing written |
| a mirror's lock names a file its frame does not | refused: the lock's list against the frame's; nothing written |
| a mirror's frame and lock both add `.github/workflows/x.yml` | refused: `names files outside the page`; nothing written |
| an untouched mirror | pinned |

## Stable MIR identities and bounded proof work

UE-28 uses actual compiler MIR for three modules declaring the same method
name. The earlier allocator retained only two candidates; the repaired
allocator retains all three and preserves the last method's declaration when
only that method's dump is selected.

UE-29 plans trials by dependency components. The negative cases reject a zero
budget, an absent dependency, and a recursive component larger than the budget.
Its SDK regression verifies three countdown methods in single-function trials;
each first discovers its actual axiom set and then verifies that declaration.
A second run must reuse the unchanged proved digests without a verifier run.

UE-30 combines locked-source number readings with generic registry aliases.
Before the repair, expansion of `BigRational` to `Ratio<BigInt>` erased its
registered model and the identity-function assertion failed. The repaired
projection lifts that identity as the stored integer pair. Replacing the
locked rational archive checksum with zeros must leave it unlifted.

The development profile optimizes the projector itself, as it already does
its certified translator. The SDK gate still runs every check with debug
assertions and overflow checking enabled. This removes repeated unoptimized
whole-workspace traversals from proof and integration tests.

Saved state-threading, cell-field and trait-dictionary changes retain their
positive and counterexample tests. Unsupported-type fixtures use `UnsafeCell`
instead of newly readable cells. Embedding join fixtures use a display-trait
bound instead of newly readable marker bounds, so they still exercise the
embedding fallback. Their closure, source-site and per-function regression
assertions remain in place.


The UE-29 diagnostic regression also places a failure on a theorem's opening
line, before its `name` field. The previous lookup blamed the preceding
definition (`a` instead of `b`); the repaired lookup names the declaration
that opens on that line. Actual SDK failures on adjacent native obligations
exposed this case. The prover prints the full SDK diagnostics while retaining
compact refusal causes in the verdict register.

Engine assertions share the immutable checkout's self projection. The
byte-determinism regression explicitly takes two fresh projections, and
mutated fixture checkouts always project afresh, so sharing cannot mask those
negative cases.


## Scoped types and metadata MIR collection

UE-31 resolves qualified field and signature types before reducing their
paths. Its fixture distinguishes numeric and Boolean homonyms, an imported
bare type, a nested generic argument and an unresolved qualified path.
The previous projection refused the valid numeric getter. The corrected
projection lifts the four valid functions and keeps the unresolved one out.
Existing path diagnostics retain the exact declaration sites; a qualified
constructor whose body reading remains unsupported stays refused.

UE-32 compares the pinned compiler's complete pre-LLVM MIR inventory and
bytes under native emission and the production metadata flags. Private,
generic, closure and impl bodies are present, and the production target
emits metadata without a native archive. Dropping forced MIR encoding gives
an incomplete inventory and is detected. Independent Cargo library and
binary probes also succeed with these flags.


## Early propagation with mutable state (UE-48)

The registered source fixture first failed because every `?` in a body with
threaded mutable parameters was refused. The repaired projection returns the
current parameter values on both success and early failure. A fallible lifted
callee is unpacked before its result is matched, so its updates survive failure.
Result constructors use the return context without broadening float inference.
Option comparisons bind both operands at their concrete type before matching;
this prevents an untyped `None` in the generated Lean match.

Six reference Rust tests exercise Option and Result success and failure, a
state-changing fallible callee, and two mutable parameters. The actual pinned
SDK verifies all generated acceptance roots. A planted defect replaces the
updated state by the original parameter on the Option failure path: the same
SDK proves the option-failure acceptance proposition false and rejects it.
The fixture also confirms mutable aliases and an unmodeled bool-to-u8 error
conversion remain unlifted. All 421 library tests pass, including the existing
float inference and borrow-escape refusals. Complete release acceptance remains
separate from these focused checks.


## Locked numeric observations and machine proof campaigns (UE-49, UE-50)

UE-49 first failed on the absent native integer zero observation. The numeric
readings now share stored-ratio accessor terms between source and MIR lowering.
The exact checksum-admitted num-bigint 0.4.8 and num-rational 0.4.2 sources define
the predicates, signs, absolute values, accessors and identity constructors.
The compiler must resolve the declared crate and trait; a spelling alone does
not select a reading. Unsupported operations and wrong checksums stay refused.

The actual SDK proved 18 compiler-derived numeric functions. Nineteen closed
acceptance vectors over the emitted functions also pass the SDK. Rust reference
vectors confirm that a raw ratio 0/0 is both zero and one, and that absolute
value can preserve a negative denominator while negating only the numerator.
These representation-sensitive cases prevent replacing stored pairs with an
assumed normalized rational. Axiom policies come from the actual verifier's
observations; the tests reuse UORust's digest-bound native decisions rather than
repeating completed proof batches when acceptance vectors change.

UE-50 first failed because prove accepted no machine-output option. With
`prove --json`, UORust runs the proof campaign, writes its final verdict-backed
projection, then measures the register itself. Its JSON binds the frontier,
verdict register, projection manifest and LexLean lock to exact bytes and records
the running executable's digest. The report distinguishes an open frontier,
translation closure, and full-project SDK acceptance. A cached fixture campaign
uses zero verifier calls; an unowned output directory is refused and its planted
file is preserved. The caller need not parse prose, reconstruct the function
counts, or rerun a separate projection to obtain the final work queue.


## Width-derived constructors and semantic proof-cache context (UE-51, UE-52)

UE-51 first refused unsigned BigInt construction. The project-owned Prelude now
provides a structural binary fold over a declared bit count. Work is bounded by
the primitive word width, including u128::MAX; it does not count up to the value.
Unsigned source constructors and signed/unsigned native constructors share this
reading, with a width-derived two's-complement adjustment. Native rational
integer construction shares the existing stored-pair reading.

Thirteen native functions and 32 closed SDK boundary vectors pass, covering every
primitive integer width, pointer-sized integers, signed minima and all-one bit
patterns. Reference Rust vectors confirm the same boundary values. The SDK
preflight also checks the shared model directly before expensive proof runs.

UE-52 reproduced a stale-cache defect: changing a source helper from identity to
negation left its native caller's own tokens unchanged, and the old tool reused
the caller's decision without SDK work. Native digests now bind the source/type
context, bundled semantics, the pinned SDK lock and the transitive native
translation closure. The actual regression verifies the changed context, then
replays it with zero verifier calls. A planted Bool-as-Nat definition in the
owned shared module is rejected by the SDK as a typed campaign failure. It is
not misrecorded as a refusal of each unrelated native function. Existing legacy
fixture decisions are refreshed through the actual prover under the new binding;
no cached status or SDK object is edited to make it pass.


## Reachable native proof obligations (UE-53)

The planted regression supplied two native loops, one valid and one with an
unchecked u8 increment, beside an unrelated false source assertion. Before
the repair, both native functions were refused. The actual SDK now proves
the valid function and refuses the overflow obligation. The separate full
projection still contains the source assertion and fails SDK verification.

Native trials retain all candidate declarations and their transitive declaration
and type dependencies. Mutual groups remain whole; quoted text cannot introduce
references or delimit declarations. Missing roots and unsupported declaration
shapes are typed tool failures. This changes only trial inputs, and the proof
strategy is bound into native cache identities. A module-wide failure is split
by recursive components; an inconclusive final component returns a tool failure
with its raw SDK receipt rather than inventing per-function counterexamples.

The regression was red with zero proved and two refused, then passed with one
proved and one refused. The lexer regression also covers quoted braces, escaped
quotes, transitive type references and mutual siblings. These checks are focused
development evidence; complete release acceptance remains `just vv`.

## Exact native feedback module identity (UE-54)

A consumer projection also contains library projects. Looking up the first path
ending in Behavior.lex.tex selected an earlier library file for root SDK source
locations. Concrete overflow diagnostics therefore appeared unattributed and
triggered repeated batch splitting. The planted regression returned libraryClaim
instead of holdsNative before the repair. Feedback now reads the exact root
src/Behavior.lex.tex, the same file the native SDK invocation verifies. Library
modules remain independent inputs and cannot supply a root declaration's owner.

## Native reference-alias match evidence (UE-55)

The previous frozen producer refuses the paired enum, copied Option and replaced
enum fixtures with `ProjectionElem::Downcast`: a field read outside a match.
The compiler uses different temporary references for the discriminant and field
access even when both read the same immutable value. Native bindings now key by
the exact value term and its complete type. Copies share match evidence; a
functional update changes the key and cannot reuse a stale constructor binder.

Three compiler-derived functions pass actual SDK proof trials. Nine closed SDK
vectors agree with executable Rust reference cases, including replacement after
a prior match. The overflowing fourth function remains refused. A planted
assertion that replacement returns its stale value is rejected by the SDK
because its proposition is false. The regression also requires current native
candidate eligibility, so historical cached proofs cannot hide a translation
regression. These are focused development checks; complete `just vv` remains
the release gate.

## Native readonly shared owners (UE-57)

The compiler-derived regression initially fails with `mir::Ty: std::sync::Arc<Wide>`.
Native lowering now uses the source's existing content representation for the
standard Rc and Arc paths. Six native functions covering construction, cloning
and readonly field access pass actual SDK proof trials. Twenty-four closed SDK
vectors agree with executable Rust cases at zero, one, an intermediate value
and u32::MAX. Reference counts, pointer identity and shared Mutex contents
remain typed residue. Type tests reject unrelated same-named owners and RefCell
contents, and require both lock-admitted numeric readings before accepting an
expanded Arc<Ratio<BigInt>>. Complete `just vv` remains the release gate.

## Native unary numeric models (UE-58)

The regression initially refuses the compiler's i128 checked_neg call. One
width-derived rule now handles primitive checked negation, wrapping negation,
unsigned magnitude and signum. Guarded Option results materialize through the
existing read operation when returned. Exact BigInt and raw Ratio negation
reuse the lock-admitted numeric carriers; only the stored numerator is negated.

Six functions pass actual SDK proof trials. Twenty-two closed SDK vectors
cover primitive minima, maxima, zero and minus one, plus exact integer and raw
rational cases with a negative denominator. Executable Rust confirms the
stored rational representation. An unsupported unwrap stays typed residue.
A planted unsigned owner in the signed checked-negation MIR is rejected,
and type checks reject mismatched owners, result types, unrelated paths and
pointer widths that differ across declared targets. Complete `just vv`
remains the release gate.

## Blocked-context proof history (UE-59)

An actual Factory refresh erased a complete-context proof for a caller whose
partial compiler view could not translate a transitive callee. The next full
campaign reproduced the identical proof in two redundant verifier runs.
The regression obtains SDK proofs for a three-function chain, omits the leaf
MIR and demonstrates that the still-listed, blocked caller has a different
partial-context digest. Before repair, that digest erased the caller's proof.

Only a translation currently eligible for verification can invalidate a cached
decision. The blocked caller remains unadmitted; all original decisions survive,
and restoring the complete view replays with zero verifier calls. The existing
selection-history regression also passes, and the source-dependency regression
confirms that changed eligible meanings still require actual SDK verification.
No verdict status or digest is rewritten. Complete `just vv` remains the release
gate.

## Mathematical integer ABI codecs (UE-56)

The previous frozen producer refuses all six compiler-derived i128 and isize
field-read, aggregate and source-call fixtures at their representation boundary.
Width-derived codecs now bridge native words and matching mathematical source
integers. Conversions split signed magnitudes into checked UInt64 limbs, with
ordered range obligations proved by the actual SDK. Source-call results bind
once before conversion so each range check stays independent of the call body.

All six native functions pass actual SDK proof trials. Twenty-four closed SDK
vectors agree with executable Rust reference cases at minimum, maximum, zero
and negative values. Fixed Int64 representations, unrelated field types and
mismatched widths remain refused. A planted Bool field in the source schema
rejects the corresponding native integer read. These are focused development
checks; complete `just vv` remains the release gate.

## Shared native hash-map reads (UE-60)

The compiler-derived regression initially fails at `HashMap::get`. Four table
rows now reuse the existing linear lookup, membership and length models on
the unordered logical carrier. Map patterns retain their ordered/unordered
flag. Primitive equality and the source's admitted derived comparisons are
shared; byte-identical source/native comparison declarations emit once while
the native proof context still binds the complete helper definitions.

Seven native functions pass actual SDK proof trials. Twenty-eight closed SDK
vectors agree with executable Rust cases for numeric, borrowed string and
derived keys, both entry orders, hit/miss and empty/nonempty maps. A source
comparison in the same fixture exercises shared-helper emission. Custom
equality, hasher observations, iteration, floats and mismatched map kinds
remain refused. Replacing successful lookup results with a different value
produces LLV7002 and the SDK diagnostic that the proposition is false. The
model's Rust vectors are regenerated from std.toml and executed directly.
These are focused development checks; complete `just vv` remains the release
gate.

## Native function values and option mapping (UE-61)

The compiler-derived regression first fails on the constant function item
`flip`. Immutable workspace function items now retain their admitted signatures
and dependency closure, with explicit lambda parameters for their application.
Standard Rc/Arc AsRef, Borrow and Deref adapters reuse the admitted content
representation only for exact owners, traits and targets. A single Option::map
table row applies these pure function values; mutable callbacks remain residue.

Five mapped native functions pass actual SDK proof trials. Fifteen closed SDK
vectors agree with executable Rust for None, Boolean negation and shared values
at zero and u32::MAX. Custom adapter owners, mutable callbacks, incompatible
targets, unrelated traits and shared Mutex contents remain refused. A planted
wrong borrow result produces LLV7002 and the diagnostic that the proposition
is false. Generated Rust table vectors execute the same Option mapping model.
The interrupted test run retained its bound native decisions for replay. These
are focused development checks; complete `just vv` remains the release gate.

## Structural delivery safeguards (UE-62)

The initial regression fails because no delivery preflight exists. The policy
now requires the complete registry, structural dependency families and an
authority bound to actual model bytes. The campaign calls this guard before
proof work. Positive and negative cases cover leaf plans, manual strategies,
partial scope, wrong authority, missing source bindings, unchanged failures,
UI work, changed reports and artifacts, baseline loss, and false completion.
Twenty positive/negative cases pass. A preserved report remains usable after
the original report path is replaced. An actual planted leaf plan is rejected
at the consumer entry point without changing native or SDK logs. Identical
verified work replays in 1.8 seconds without running either verifier. Manual
semantic-registry changes and reductions of the original source/registry are
also rejected.

The consumer entry point also binds actual source, compiler MIR, embeddings,
SDK, producer and verdict bytes. It preserves actual verified receipts and
checks their manifests before reuse. Workspace instructions persist the user's
structural delivery requirement without modifying immutable repository policy.
This guard enforces campaign discipline; it does not create missing Rust
semantics or change the all-functions finish line. Full `just vv` remains open.

## Compositional compiler control-flow regions (UE-63)

The compiler-derived regression initially refuses nested control flow because
function-sized loop continuations create cyclic helper calls. Natural regions
now derive from compiler edges and dominating headers. Inner regions return
explicit exit packets; their callers own continuations. Conservative MIR write
analysis keeps unchanged frame values in their caller and transports changed
live state. Region outcomes also preserve early returns and distinct exits.

The structural analysis uses one linear-size reachability set per loop header,
with an acyclic fast path, rather than a quadratic dominator matrix. Existing
non-nested lowering retains its emitted representation and proof identities.
No standard-operation model row changes.

Four nested, triple, early-return and multi-exit functions pass actual SDK
proofs. All 128 closed SDK results agree with an executable Rust oracle. An
overflowing fifth function remains refused, and live mutable-borrow transport
remains typed residue. Independent CFG cases reject irreducible entries, missing
blocks and an absent entry. A planted lost Boolean exit state is rejected by
LLV7002 because an oracle proposition is false. Exact match-path evidence is
retained in every recursive decrease obligation; no termination gate is weakened.
These are development checks; full `just vv`, translator certification and
consumer adoption remain separate acceptance requirements.

## UE-64: shared source model import

The model, scenario and regression were registered first. The red compiler
fixture stopped at `Option::unwrap_or_default`; the native path had no access to
the source translator's existing model. Native calls now bind compiler-owned
standard inherent paths and exact immutable source/native carrier types before
importing the source declarations and their dependencies. Recursive nominal
schemas are checked for mutable cells. Panic-default and divergent readings
cannot pass this interface. No executable row of `mir/std.toml` changed.

`native_calls_import_source_semantic_models_ue_64` passed in the pinned SDK:
four native proofs; eleven executable-Rust differential vectors across Option,
Result and Boolean receivers; four definitional equalities to the existing opaque
string model. UTF-8 runtime primitives do not reduce under kernel `decide`, so
those four equalities establish model transport rather than claiming a computed
Rust differential. An incorrect oracle result (255 changed to 254 in the vector
itself) was rejected with LLV7002 / `is false`. Partial unwraps, Unicode-divergent
readings, mutable inputs, incompatible carriers and custom owners are refused.
Two unit regressions check exact bindings and recursive schema closure.

The complete library suite detected missing codec/source entries in the tool's
source-code digest. Both are now included; the source inventory gate remains
intact. Evidence: `uorust-source-model-import-red.log`,
`uorust-source-model-import-final-checks.log`, and
`uorust-source-model-import-acceptance.log` in the delivery runtime. Focused
acceptance does not replace `just vv`, translator certification or consumer
adoption. No contract completion is claimed from this capability.

### UE-62: SDK proof dependency closure reuse

The extended model/scenario and failing receipt-selection tests preceded the
implementation. All 31 guard cases pass. The guard selects an original real
verification only at equal actual SDK build/source/semantic/module identities,
with a hash-bound original SDK pin, intact report/artifacts and a cached proof
bundle. It never calculates SDK semantic identities or converts build success
into verification. The mandatory complete-campaign preflight remains intact.

The actual consumer SDK emitted unchanged identities after recompiling the
producer. Its original 12-module verification was reused with explicit
provenance; the real SDK project check and Factory refresh passed, with zero
native and kernel verifier calls. An actual SDK build of a planted false
assertion succeeded but changed all three content identities; it could not
select the earlier verification. Its proof was not admitted. The SDK acceptance
resume took 62.9 seconds; this excludes prior native projection and interrupted
setup and is not a fresh-campaign throughput result. The consumer remains at
634/2,704 and no additional lifts are claimed.

The earlier complete library suite passed 433 tests. The actual SDK refreshed
one fixture receipt whose declaration changed through source-model import;
all 16 CLI tests then replayed successfully, and conformance meta-tests and
strict all-target Clippy passed. The fixture verdict was copied from the actual
tool-written SDK result. No verdict, digest or axiom policy was fabricated.

### UE-64: compiler callable environments

The extended model/scenario and red compiler fixture preceded implementation.
A single reifier derives callback signatures and environment applications from
compiler bodies for the existing source and native standard model interfaces.
It binds the environment once and declares that exact SDK lambda capture;
mutable captures, memory signatures and incompatible carriers stay refused.
No standard-operation formula or executable registry row was added.

The actual SDK proved four fixture functions (captured predicates, mapped
values, lazy defaults and a function item), and all 21 executable-Rust vectors
passed. Rust's overflow counterexample panics and the SDK refuses its callback;
the verdict register contains four proved and one refused function. A planted
wrong closed callback result is rejected with LLV7002 / `is false`. The SDK also
rejected the earlier missing capture metadata with LLT4001 before recording any
function verdict, demonstrating that malformed model transport is fail-closed.
Evidence: `uorust-model-callables-red.log`,
`uorust-model-callables-final-sdk.log`, and
`uorust-model-callables-acceptance.log`. Library, CLI, conformance, strict Clippy,
model/template and lock checks passed. Consumer lifting gain remains subject to
the complete guarded campaign. Full release acceptance remains open.

## UE-65: shared source type readings

The model, scenario and failing compiler test were registered first. Native
lowering now tries its original ABI reading, then imports a compatible closed
source reading. The fallback accepts admitted SDK identities and their
containers; ABI-sensitive integers/characters, callables, type variables and
unregistered nominal schemas stay out. Shared-cell allocation references retain
identity rather than exposing their contents. No executable std registry row or
new standard-operation semantics was added.

The actual SDK proved four fixture functions and twelve transport/container
properties. Rust checks the same properties while a RefCell's mutable borrow is
live, demonstrating that transport never reads a snapshot of its contents.
Identity properties use reflexivity: the opaque SDK reference types deliberately
provide no decidable equality. Shared contents, writes, locks, counts and pointer
comparisons remain typed refusals. The SDK rejects a planted wrong container
observation with LLV7002 / `is false`. The primitive ABI/state-boundary unit test
passes; the older readonly-owner test now distinguishes opaque allocation
identity from the still-refused plain RefCell contents.

Evidence: `uorust-type-import-red.log`, `uorust-type-import-final-sdk.log`,
`uorust-type-import-acceptance.log`, and `uorust-type-import-final-checks.log`.
The complete library pass checked 433 unaffected tests, and its one changed
expectation passed on replay (434 total); all 16 CLI tests, conformance tests,
strict Clippy and model/template/lock gates passed. Consumer gain is measured
only by the complete guarded campaign; release acceptance remains open.

## UE-66: compiler composition with source generic dependencies

The compiler path now imports complete source-lifted generic definitions through
`lift::generic_call`, the source binder itself. Type arguments and coherent trait
methods retain their existing signatures and dependency names. It does not add
standard-operation models or supply an implementation for an unknown method.
Native words may pass parametrically without acquiring source integer semantics.
Source definitions with recorded divergences are excluded from this interface.

The initial regression failed at `generic_dependencies::word` with
`mir::TerminatorKind::Call: transport`. The restored regression has five actual
native SDK proofs and 32 computed-Rust comparison vectors covering unsigned and
signed transport, option payloads, coherent dictionaries and captured callbacks.
The SDK rejects a planted expected result of 254 for Rust's actual 255 with
LLV7002 (`is false`). The final focused SDK run passed in 78.72 seconds.

A separate planted schema defect admitted a dictionary call on a struct holding
`Cell<bool>`. Its negative assertion failed (`shared`) before the shared schema
check was applied. The final run keeps it unlifted alongside unavailable method
implementations, incompatible signed return carriers, partial panic conventions
and mutable references. The common binder's unit test also refuses two different
word widths bound to one type parameter, a missing coherent dictionary entry and
a method with a different result type; it retains the accepted method dependency.

435 library tests, all 16 CLI tests, conformance meta-tests, model/template/lock
checks and strict all-target Clippy passed during focused acceptance. These are
review evidence, not a claim that the complete `just vv` release gate passed.
Unused trait implementation entries are kept out of the global native proof
context: the chosen method and its source meaning are already bound by the
translated term and dependency identities.

## UE-67: parametric compiler bodies and product state

Native scopes now import the function's declared type parameters, including
parameters whose bounds its body does not use. Reading that parameter does not
supply a method implementation. Parametric trait dispatch without an admitted
dictionary and reachable generic drop glue stay typed residue.

Generic product values use the SDK's built-in, right-nested products. The common
emitter binds generated definitions, theorems and region-exit types by the type
variables they and their dependencies use. It supplies explicit function and
constructor type arguments, preserves literals, and retains generic binders when
applying the SDK's learned axiom policy. Type-parameter names share the source
binder's collision rule. This also admits generic source product returns through
the existing source translator.

The initial compiler regression failed on `mir::Ty: T`. Four compiler-generated
generic bodies now have actual SDK proved verdicts: transport, product return,
nested natural loops with an exit-state type, and transport with an unused Clone
bound. Forty computed-Rust comparison vectors cover the first three and generic
source products at Boolean and unsigned-word instantiations. A planted expected
255-to-254 result is rejected by LLV7002 (`is false`). The final focused SDK run
passed in 125.90 seconds.

A trait method deliberately named `clone` exposed an inadmissible fallback to a
same-named value adapter: the `mimic` negative failed before the general
parametric-dispatch guard. The restored run keeps both custom and standard
unresolved trait calls unlifted, and confirms that implicit generic drop glue is
reported as a drop-effect gap. No standard-operation registry row was added.
The binder unit additionally checks that quoted type-variable/function names do
not add generic dependencies to literal or unrelated declarations.

436 library tests and all 16 CLI tests passed, with conformance meta-tests,
strict all-target Clippy, model, template, SDK lock and generated self-model
checks. This remains focused review evidence; complete release acceptance and
translator certification are separate open gates.

## UE-68: rooted native model identities and original decision retention

A native proof previously bound every source definition and type scope. Adding
an unrelated source definition changed an unchanged candidate's digest. The
initial regression failed with two different digests for that one model.

The additional `model_context_sha256` observation now binds the existing emitted
proof-declaration closure, reached native/type/schema/signature readings, and the
pinned SDK/assets. It is an internal producer binding, not an SDK semantic ID.
Legacy records still require their original exact digest before the prover may
learn this optional observation. Learning it retains the original digest,
proved/refused status, diagnostic and axiom evidence. Later equal rooted models
reuse that original record; changed reached meanings still require the SDK.

The actual SDK regression established one proved and one refused decision, then
removed only the optional observation to reproduce a legacy register. Legacy
metadata learning and an unrelated source change each took zero verifier calls;
all original decision fields stayed intact. Changing a reached helper triggered
a new actual SDK proof. A reached schema change altered the model identity, and
a planted wrong rooted binding prevented admission even at the original legacy
digest. Malformed and non-string observations are also refused. The regression
passed in 116.31 seconds.

The CLI fixture now checks the same transition: only optional observations may
appear during migration, original evidence remains byte-identical after removing
those lines, and the following replay is byte-identical with zero verifier calls.
437 library tests, all 16 CLI tests, conformance meta-tests, strict all-target
Clippy, model/template/SDK-lock and generated self-model checks passed. No cached
proof status, diagnostic or legacy digest was fabricated or manually reassigned.
Complete `just vv` and translator certification remain separate open gates.

## UE-69: native product views preserve unavailable fields

The initial real-compiler regression failed at `mir::Ty: Holder`: an unknown
service field prevented access to an independent Boolean field. The common
reader now retains complete declared field order in SDK products. It composes
views along the existing source dependency graph and retains unavailable fields
as abstract values whose identities bind their source readings. Recursive edges
stay abstract, and incomplete shapes are never guessed. Source admission and
existing closed nominal schemas retain their prior representation.

Five actual native SDK proofs passed, including a nested field access, a call
between compiler bodies, and an exclusive update preserving its opaque payload.
48 executable-Rust comparisons passed; the service implementation panics if
called, so these field operations cannot obtain a result by invoking it. A
planted incorrect Boolean result was rejected by the actual SDK. The first
proof and differential run took 161.79 seconds. Unavailable service operations,
parametric drop glue, unresolved custom trait dispatch and incomplete generic
structs stay residue. A separate custom inherent-call check confirms a blocked
dependency cannot acquire the meaning of a same-named standard adapter.

A unit regression checks nested dependency composition, retained field count
and order, source-bound opaque identities, mutable-reference detection, cyclic
edge abstraction and incomplete-shape refusal. The existing code-identity gate
rejected the new module until it was included in the bound production sources.
The delivery preflight and frozen standard semantic registry remain intact.

Final focused checks passed: 438 library tests, all 16 CLI tests, conformance
meta-tests, both struct-view engine regressions, strict all-target Clippy,
model/template/SDK-lock and generated self-model checks. The CLI register was
refreshed from actual tool-written SDK decisions, retaining ten proved and one
refused status and recording rooted observations for all eleven decisions. The
replay now takes zero verifier calls and keeps existing rooted evidence exactly;
legacy migration remains covered independently by UE-68. Full `just vv`,
translator certification and consumer adoption remain open release boundaries.

## UE-70: typed storage boundaries for native mutations

The full product-view campaign stopped at an SDK link-type mismatch: a field
update replaced an abstract stored value with a concrete value. The prior
636-function consumer checkpoint remains the latest accepted projection; the
failed run claims no additional lift. Its log and SDK result are retained.

A minimal MIR regression reproduced the cause: place-update recursion discarded
the incoming value type and accepted overwriting an opaque slot with a Boolean.
The shared recursion now retains that type through nested field, element and
option updates. At the storage boundary it reuses the existing representation
codec, including its actual range obligations, or returns a typed residue before
SDK scheduling. No standard-operation semantics or SDK behavior was added.

The negative regression now passes. Two actual native SDK proofs and 24
executable-Rust comparisons passed for direct and nested signed 128-bit stores,
including minimum, maximum and large positive values. Comparisons inspect the
stored mathematical integer, rather than only a returned word. A planted wrong
stored value was rejected by the SDK. The proof and comparison run passed in
124.67 seconds. Library, CLI and dependency checks are recorded with this
checkpoint; complete release acceptance and translator certification remain open.

Final focused dependency checks passed: all 439 library tests, all 16 CLI tests,
conformance meta-tests, both product-view regressions, strict all-target Clippy,
model/template/SDK-lock and generated self-model checks. The product-view
regressions replayed with retained native decisions and passed in 47.35 seconds.

## UE-71: parametric map carriers and explicit ABI transport

The real SDK regression first failed at an unbound `K`: generic map entries
created global tuple schemas without type binders. Entry rendering, dependencies
and the existing source/standard semantic bindings now share SDK products when
parameters are present. Closed entries retain their original tuple ABI.

A second actual SDK check rejected a concrete caller passing that closed tuple
ABI to a generic product ABI. The common generic-call binder now transports
admitted list, product, option and result carriers through explicit structural
helpers in both directions. Opaque type arguments retain their own carrier;
unsupported callable conversions remain typed residue. Compiler bindings import
those same helper declarations. The transport changes no key, value or ordering
and supplies no comparison implementation or operation semantics.

Eight actual native SDK decisions and 39 executable-Rust comparisons passed
across Boolean and natural-valued maps, source and compiler reads, ordered and
unordered maps, concrete calls of generic functions and both directions of
transport. Unavailable generic key comparison and generic drop glue stay residue.
The first comparison used equality of a nominal tuple list, for which the SDK
provides no decidable-equality instance. Closed outputs are instead checked by
matching both entries, their primitive fields and the empty tail. No instance,
axiom or model ABI was added to make the test pass. A planted incorrect length
is rejected by the actual SDK. The final comparison run passed in 48.14 seconds.

The unit regression checks the exact retained closed rendering/tag, absence of
unbound nominal generic entries, both transport directions and their explicit
pure declarations. All 440 library tests, 16 CLI tests, conformance meta-tests,
strict all-target Clippy, model/template/SDK-lock and generated self-model checks
passed. Full release acceptance, translator certification and adoption remain
open; these are supporting checks rather than a completed consumer contract.

## UE-72: shared admitted source constant dependencies

The registered regression first failed on `left::VALUE`: its source definition
was already admitted, but native constant operands had no binding to that
meaning. The compiler context now distinguishes actual constant declarations
from function items and imports exact, nondivergent source definitions through
their target and module paths. Ambiguous printed identities stay unbound, even
when only one competing declaration has an admitted model. The existing source
and native type readers and immutable-value interface must agree; incompatible
signed carriers and unavailable initializers remain typed operand residue.
No second initializer evaluator or standard-operation semantics were added.

Three actual native SDK decisions and twelve executable Rust comparisons cover
Boolean values, unsigned values and equal constant names in distinct modules.
Changing one initializer rejects its previous native decision while retaining
the unrelated module's rooted decision. A planted incorrect result is refused
by the actual SDK with LLV7002. The initial comparison notation omitted the
outer equality parentheses; the SDK rejected that malformed test, and the
restored fixture passed in 55.13 seconds using intact native proof receipts.
This focused evidence does not establish additional consumer lifts, full
release acceptance, translator certification or final adoption.

The focused dependency checks passed: 440 library tests, all 16 CLI tests,
conformance meta-tests, strict all-target Clippy, model/template/SDK-lock and
generated self-model checks. The manual standard semantic registry is unchanged.
