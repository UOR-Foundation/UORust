# Prism factory

This directory is the upstream Prism factory (PrismPM's `prism-factory/`),
installed with this tool's graph. It decides, for every obligation of the
tool's gate, its writers and its certified export, whether it is already
proved, must run, is refuted, or is blocked, and in what order to run what is
left. The decisions are the upstream's Prism model, checked by the Lean kernel
and exported by LexLean to certified Rust; the host only reads the tree, runs
what the plan routes to `observe`, and records receipts.

What is this tool's own is its graph and nothing else. `upstream.toml` names
the revision every other file is taken from and the SHA-256 of each; `graph.py
check` (inside the `factory-model` obligation) refuses a file that drifted, and
`graph.py upstream DIR` compares the component with upstream's at that revision.

## Layout

| Path | What it is | Upstream |
| --- | --- | --- |
| `model/src/Factory/Engine.lex.tex` | Obligations, identity, closures, routing, priority, and their theorems | identical |
| `std/` | LexLean's standard library (`Buffer`, `Sha256`, `Text`) the engine uses | identical (its lock is the SDK compiler's) |
| `model/src/Factory.lex.tex` | The roots `plan` and `reweigh` and their text protocol | derived by `graph.py derive`: upstream's definitions of the two roots, over this graph |
| `model/src/Factory/UORust.lex.tex` | This tool's graph and its theorems | written by `graph.py write` from `graph.toml` and the Justfile |
| `graph.toml` | The graph's one source: each obligation's recipe, kind, inputs and needs | the tool's |
| `certified/` | The production bundle of the two roots: certified Rust packages, certificates, attestation | exported here by the SDK's LexLean |
| `host/` | The thin host (`prism-factory` CLI): upstream's `request`, `plan`, `run`, `plan-request`, `reweigh` | upstream's code for those commands |
| `check.sh` | The `factory-model` obligation | the tool's, after upstream's |
| `factory.sh` | The entry: `routes`, `run`, `start`, `status`, and the host's commands | the tool's, after upstream's |
| `sdk.sh` | Runs an obligation's command in the SDK image `prismpm.lock` selects | the tool's |
| `written.sh` | A writer's obligation: passes only when the writer changes nothing it writes | the tool's |

The release workflow, the cut and the forge's delegation of upstream
(`Workflow.lex.tex`, `Delegation.lex.tex`, `PrismPM.lex.tex`, `jobs/`, the
`actions-*` and `forge-*` commands) are PrismPM's own release and are not
installed.

## Identity

An obligation's identity is upstream's, unchanged: the SHA-256 (in hex) of

```text
prism-factory/obligation/1
name <name>
kind hermetic|fetch
environment <environment>
run <run>
input <prefix>                 one line per declared input, in order (`!prefix` excludes, `@key` is a context)
needs <name> <identity>        one line per need, in order
closure <sha256 of the closure>
```

The closure is one line `file <mode>:<object> <path>` for each tracked entry
covered by the declared inputs, in Git tree order. An entry is covered when
the longest inclusion it starts with is longer than the longest exclusion
(`!prefix`) it starts with. Changing a file an obligation declares, an input,
the command, the environment or any upstream identity changes the identity.
Nothing else does. `host/tests/core.rs` holds the plan's identities against
Python's `hashlib` over the same text.

## The graph

Every obligation runs on the host (`environment host`): an obligation that
needs the SDK runs its recipe through `sdk.sh`, which reads the image from
`prismpm.lock`, and both are declared inputs, so the image's digest is part of
the identity. A compiling obligation fetches locked crates, so it is a `fetch`
obligation as upstream's are: a failure is observed again, never refuted.

<!-- graph:begin (graph.py write) -->
| Obligation | Kind | Needs | Reads | Not touched by |
| --- | --- | --- | --- | --- |
| `factory-model` | hermetic | nothing | `prismpm.lock`, `prism-factory/sdk.sh`, `prism-factory/`, `Justfile` | anything outside prism-factory/, the Justfile and the SDK lock |
| `factory-upstream` | fetch | nothing | `prism-factory/` | anything outside prism-factory/ |
| `fmt-check` | hermetic | nothing | `prismpm.lock`, `prism-factory/sdk.sh`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `crates/`, `!crates/uorust/core/`, `!crates/uorust/std/`, `!crates/uorust/crates/`, `!crates/uorust/ledger/`, `!crates/uorust/tests/fixtures/`, `crates/uorust/tests/fixtures/nist-sha256/`, `xtask/`, `app/lexlean.lock`, `.github/dependabot.yml`, `.github/workflows/bootstrap.yml`, `.github/workflows/prismpm.yml`, `.github/workflows/template-update.yml`, `rustfmt.toml` | the model, the app, the registers, the fixtures, scripts, the site, the documents |
| `model` | fetch | nothing | `prismpm.lock`, `prism-factory/sdk.sh`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.devcontainer/`, `.github/`, `AGENTS.md`, `CONFORMANCE.md`, `README.md`, `VERIFICATION.md`, `template.lock`, `template-contract.json`, `crates/`, `xtask/`, `model/`, `LANES.md`, `TEMPLATE-CONTRACT.md`, `TEMPLATE-VERIFICATION.md` | the app, scripts, the site, the brand, frontier.toml, lift-verdicts.toml |
| `template-check` | fetch | nothing | `prismpm.lock`, `prism-factory/sdk.sh`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.devcontainer/`, `.github/`, `AGENTS.md`, `CONFORMANCE.md`, `README.md`, `VERIFICATION.md`, `template.lock`, `template-contract.json`, `crates/`, `xtask/`, `model/`, `Justfile` | the app, scripts, the site, the brand, frontier.toml, lift-verdicts.toml |
| `self-model-check` | hermetic | nothing | `prismpm.lock`, `prism-factory/sdk.sh`, `model/`, `app/`, `brand/`, `scripts/self-model/`, `scripts/experience/`, `scripts/mir/ledger.py`, `crates/uorust/src/`, `crates/uorust/core/frontier.toml`, `crates/uorust/std/frontier.toml`, `crates/uorust/crates/`, `crates/certified/manifest.toml`, `frontier.toml`, `site/app.json`, `.github/workflows/`, `Cargo.toml`, `README.md` | the other crates, the fixtures, xtask/, the lock files, lift-verdicts.toml |
| `lint` | fetch | nothing | `prismpm.lock`, `prism-factory/sdk.sh`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `crates/`, `!crates/uorust/core/`, `!crates/uorust/std/`, `!crates/uorust/crates/`, `!crates/uorust/ledger/`, `!crates/uorust/tests/fixtures/`, `crates/uorust/tests/fixtures/nist-sha256/`, `xtask/`, `app/lexlean.lock`, `.github/dependabot.yml`, `.github/workflows/bootstrap.yml`, `.github/workflows/prismpm.yml`, `.github/workflows/template-update.yml`, `clippy.toml` | the model, the app, the registers, the fixtures, scripts, the site, the documents |
| `test` | fetch | nothing | `prismpm.lock`, `prism-factory/sdk.sh`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `crates/`, `xtask/`, `frontier.toml`, `lift-verdicts.toml`, `app/lexlean.lock`, `.github/`, `model/`, `features/`, `app/`, `Justfile`, `CONFORMANCE.md`, `README.md`, `scripts/ci/` | the site, the brand, scripts other than scripts/ci/, the documents other than CONFORMANCE.md and README.md |
| `deny` | fetch | nothing | `prismpm.lock`, `prism-factory/sdk.sh`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `crates/`, `!crates/uorust/core/`, `!crates/uorust/std/`, `!crates/uorust/crates/`, `!crates/uorust/ledger/`, `!crates/uorust/tests/fixtures/`, `crates/uorust/tests/fixtures/nist-sha256/`, `xtask/`, `app/lexlean.lock`, `.github/dependabot.yml`, `.github/workflows/bootstrap.yml`, `.github/workflows/prismpm.yml`, `.github/workflows/template-update.yml`, `deny.toml` | the model, the app, the registers, the fixtures, scripts, the site, the documents |
| `app` | fetch | nothing | `prismpm.lock`, `prism-factory/sdk.sh`, `app/` | everything outside app/ and the SDK lock |
| `uorust` | fetch | nothing | `prismpm.lock`, `prism-factory/sdk.sh`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `crates/`, `xtask/`, `frontier.toml`, `lift-verdicts.toml`, `app/lexlean.lock`, `.github/` | the model, the app, scripts, the site, the documents |
| `vv` | hermetic | `fmt-check`, `model`, `template-check`, `self-model-check`, `lint`, `test`, `deny`, `app`, `uorust` | `Justfile` | everything its parts leave untouched, and the Justfile's other recipes only through its own input |
| `prove-fixture` | fetch | nothing | `prismpm.lock`, `prism-factory/sdk.sh`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `crates/`, `!crates/uorust/core/`, `!crates/uorust/std/`, `!crates/uorust/crates/`, `!crates/uorust/ledger/`, `!crates/uorust/tests/fixtures/`, `crates/uorust/tests/fixtures/nist-sha256/`, `xtask/`, `app/lexlean.lock`, `.github/dependabot.yml`, `.github/workflows/bootstrap.yml`, `.github/workflows/prismpm.yml`, `.github/workflows/template-update.yml`, `crates/uorust/tests/fixtures/mir-path/`, `prism-factory/written.sh` | the model, the app, the other fixtures, the registers, scripts, the site, the documents |
| `prove` | fetch | nothing | `prismpm.lock`, `prism-factory/sdk.sh`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `crates/`, `xtask/`, `frontier.toml`, `lift-verdicts.toml`, `app/lexlean.lock`, `.github/`, `prism-factory/written.sh` | the model, the app, scripts, the site, the documents |
| `certified` | fetch | `prove` | `prismpm.lock`, `prism-factory/sdk.sh`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `crates/`, `xtask/`, `frontier.toml`, `lift-verdicts.toml`, `app/lexlean.lock`, `.github/`, `Justfile`, `prism-factory/written.sh` | the model, the app, scripts, the site, the documents |
<!-- graph:end -->

## Running it

```sh
just factory-plan            # each obligation's route and how many are left (prism-factory/factory.sh routes)
just factory                 # plan and run to the fixed point; `just factory NAME...` runs only those
just factory-start           # the same, detached; its log and status are written beside the receipts
just factory-status          # `running PID TIME`, or `done CODE established N proved N refuted N blocked N observe N`
```

`run` copies each obligation's closure, and nothing else, into a fresh
directory outside any repository (`FACTORY_WORK`) and runs it there, so an
undeclared input fails the run instead of producing a false receipt. A
receipt is written as each obligation ends, so an interrupted run loses only
the obligation in flight, and a second run on the same tree runs nothing.
The receipts live in `FACTORY_RECEIPTS`, a directory that outlives the
worktree and every target directory. `factory.sh keep BRANCH` would add them
to a receipts branch of the forge, upstream's store; it pushes, so it runs
only where the repository's owner grants that push.

## Changing the graph

```sh
python3 prism-factory/graph.py write                     # graph.toml and the Justfile -> UORust.lex.tex and the table
cd prism-factory/model && lexlean lock && lexlean verify --all
lexlean production targets --diagnostic-format json > /tmp/targets.json
python3 ../graph.py request /tmp/targets.json            # only when the SDK's compiler changed
lexlean production build --request ../production-request.json
rm -rf ../certified && cp -r .lexlean/production/<result> ../certified
```

The graph is compiled into the certified `plan` root, as upstream's is, so a
change of an obligation's recipe, inputs or needs is a new export.

## Limits

- Obligations run with the host's network and user, as upstream's `host`
  obligations do. The closure is enforced; network and uid isolation are not.
  A compiling obligation reads the SDK container's cargo cache, which is why it
  is a `fetch` obligation.
- A receipt is trusted because of who can write the receipt store. A receipt
  established on a workstation counts on the forge only when a listed
  maintainer signed it for exactly its identity (upstream's delegation model).
  No signing key exists here and none is made, and the forge's workflows keep
  no receipts, so a forge run would establish every obligation its event asks
  for again. What is carried by content regardless of receipts is carried
  there too: the certified packages in `crates/certified/` and their export
  identities (`uorust certify production --carry`).
- `generate.yml` and `certified.yml` are not routed through this entry: on the
  forge, with no kept receipts, the plan leaves every obligation, so routing
  them would establish the same work again; the forge starts neither on a
  pull request.
