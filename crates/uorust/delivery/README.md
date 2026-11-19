# Structural delivery preflight

`guard.py` reads `model/delivery-policy.json`, a complete-registry work plan,
exact source/tool/MIR/SDK/verdict bindings and retained campaign receipts. It
rejects individual function or operation repair queues, partial structural
scope, unbound authority bytes and unchanged failed work. Executable changes
to the manual semantic registry are rejected against the pinned implementation
baseline; test-vector and explanatory-only changes do not count as semantics. Intact verified
receipts are reusable; report and artifact mutations refuse reuse.

The caller must compute bindings from the actual inputs and run the gate before
proof work. After actual SDK acceptance, `record` preserves an immutable report
and the artifact hashes. Completion additionally requires every function, the
complete release gate and source-bound adoption. The guard does not create SDK
proofs or supply missing compiler semantics.

Run `python3 -I crates/uorust/delivery/test_guard.py`. The UE-62 engine regression
executes the same positive and negative cases in the repository acceptance gate.
A CLI caller supplies `--policy`, `--plan`, `--binding`, and `--history`; malformed
plans and failed bindings exit with a typed DG diagnostic before work begins.

This follows the existing certified Prism Factory identity/dependency runtime
and Verdant's model provenance, dependency lightcones and exact-input transition
reuse. The consumer campaign installs this preflight at its entry point; its
workspace instructions also prohibit bypassing it or resuming a manual leaf queue.

For proof-stage reuse, run the actual SDK build and pass its unmodified JSON to
`sdk_receipt`, with the SDK pin digest, verified history and acceptance tree.
The build, source, semantic and module identities must match an original real
verification. The history's original binding must hash to its own key and bind
the same SDK; its report and artifacts must remain intact. A cached proof bundle
must be present. Otherwise run the actual verifier. Always retain the original
receipt provenance and run the actual SDK project check. A successful build is
never itself a proof. New history rows preserve their input binding explicitly.
