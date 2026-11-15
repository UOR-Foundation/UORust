"""The attempt ledger of the MIR path's CI workloads: the adapter that hashes.

An attempt is (module, translator κ, toolchain κ), Engine::Attempt in
app/src/Engine.lex.tex; for a crate of crates.io the module is
`<name>@<version>`. A module's dump is fixed by the toolchain and its
translation by the translator, so an attempt the ledger holds is never made
again: Engine::staleModules keeps the attempts the ledger does not hold, and
a tick whose attempts are all held is idle and settles (Engine::settled).
The model decides those rules over its fixtures; this script computes the
two κ and applies the same rule to the ledger file:

- the translator κ is the SHA-256 of every file that fixes a row from this
  repository (the translator's sources, the embedding's LexLean project in
  crates/uorust/assets, its manifest, the lockfile, the dump script, the
  differential harness and the register's own workflow), each named by its
  path and its own SHA-256;
- the toolchain κ is the SHA-256 of rust-toolchain.toml and the SDK image
  prismpm.lock selects, which fix the compiler, rust-src and the build.

    python3 -I scripts/mir/ledger.py stale REGISTER MODULES.json
    python3 -I scripts/mir/ledger.py record REGISTER SHARDS_DIR
    python3 -I scripts/mir/ledger.py rank-count REGISTERS BUDGET_SECONDS
    python3 -I scripts/mir/ledger.py plan-crates REGISTERS RANKED.json EVENT BUDGET_SECONDS
    python3 -I scripts/mir/ledger.py record-crates REGISTERS TIMES_DIR
    python3 -I scripts/mir/ledger.py mir-key

`mir-key` prints the key a library's MIR dump is kept under, as a CI
artifact: the toolchain κ and the digest of the dump script, the two inputs
that fix the dump's bytes. `just core-mir` fetches the dumps CI made under
the current key, so a lane's census reads the very MIR CI's does.

The ledger of a register is crates/uorust/ledger/<its directory's name>.toml:
core.toml, std.toml, and crates.toml for the crates of crates.io.
"""
import hashlib, json, os, subprocess, sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
# Every file that fixes a row's columns: the translator, its manifest and
# lockfile, the dump script, the embedding's LexLean project (the evaluator
# and its datatype, crates/uorust/assets) and the differential check's
# harness. The workload's own workflow, whose steps run them, is added per
# register (`workflow`).
TRANSLATOR = ["crates/uorust/src", "crates/uorust/assets", "crates/uorust/Cargo.toml", "Cargo.lock",
              "scripts/mir/dump-lib.sh", "scripts/mir/harness.sh"]
TOOLCHAIN = ["rust-toolchain.toml"]


def sha(data):
    return hashlib.sha256(data).hexdigest()


def tree_kappa(paths):
    files = subprocess.run(["git", "-C", ROOT, "ls-files", "--", *paths], check=True,
                           capture_output=True, text=True).stdout.split()
    lines = [f"{f}\0{sha(open(os.path.join(ROOT, f), 'rb').read())}" for f in sorted(files)]
    return sha("\n".join(lines).encode())


def toolchain_kappa():
    lock = open(os.path.join(ROOT, "prismpm.lock"), encoding="utf-8").read()
    image = lock.split('"sdk_image":"', 1)[1].split('"', 1)[0]
    return sha(f"{tree_kappa(TOOLCHAIN)}\0{image}".encode())


DUMP = ["scripts/mir/dump-lib.sh", "crates/uorust/src/layouts.rs"]


def mir_key():
    """The key of a library's MIR dump: (toolchain κ, dump-script digest)."""
    return f"{toolchain_kappa()[:16]}-{tree_kappa(DUMP)[:16]}"


def ledger_path(register):
    """crates/uorust/ledger/<register>.toml: beside the registers, not in
    them, whose every other file is a module's."""
    return os.path.join(ROOT, "crates/uorust/ledger", os.path.basename(os.path.normpath(register)) + ".toml")


def read_ledger(register):
    """The attempts the ledger holds, as (module, translator, toolchain)."""
    path = ledger_path(register)
    if not os.path.exists(path):
        return set()
    import tomllib
    doc = tomllib.loads(open(path, encoding="utf-8").read())
    return {(a["module"], a["translator"], a["toolchain"]) for a in doc.get("attempt", [])}


def read_seconds(register):
    """The seconds each attempt took, by module, where it was measured."""
    path = ledger_path(register)
    if not os.path.exists(path):
        return {}
    import tomllib
    doc = tomllib.loads(open(path, encoding="utf-8").read())
    return {a["module"]: a["seconds"] for a in doc.get("attempt", []) if "seconds" in a}


def write_ledger(register, attempts, seconds=None):
    seconds = read_seconds(register) if seconds is None else seconds
    os.makedirs(os.path.dirname(ledger_path(register)), exist_ok=True)
    rows = "".join(f'[[attempt]]\nmodule = {json.dumps(m)}\ntranslator = "{t}"\ntoolchain = "{c}"\n'
                   + (f"seconds = {seconds[m]}\n" if m in seconds else "") + "\n"
                   for m, t, c in sorted(attempts))
    head = ("# The attempt ledger (written by scripts/mir/ledger.py; do not edit): each\n"
            "# (module, translator κ, toolchain κ) the workload translated. An attempt\n"
            "# held here is not made again (Engine::staleModules in app/src/Engine.lex.tex).\n\n")
    open(ledger_path(register), "w", encoding="utf-8").write((head + rows).rstrip("\n") + "\n")


def workflow(register):
    """The workflow that keeps the register: .github/workflows/<its directory's name>.yml."""
    return f".github/workflows/{os.path.basename(os.path.normpath(register))}.yml"


def current(register, module):
    return (module, tree_kappa(TRANSLATOR + [workflow(register)]), toolchain_kappa())


def stale(attempts, ledger):
    """Engine::staleModules: the attempts the ledger does not hold."""
    return [a for a in attempts if a not in ledger]


def record(register, modules, seconds=None):
    """The ledger with the current attempt of each module, the older
    attempts of those modules replaced, and the seconds each attempt took
    where `seconds` measured it."""
    ledger = read_ledger(register)
    done = {m for m in modules}
    kept = {a for a in ledger if a[0] not in done}
    held = {m: t for m, t in read_seconds(register).items() if m not in done}
    write_ledger(register, kept | {current(register, m) for m in modules}, {**held, **(seconds or {})})


def version_order(version):
    import re
    return [int(p) if p.isdigit() else -1 for p in re.split(r"[.+-]", version)]


def held_crates(registers):
    """The crates REGISTERS holds a register for, each at the newest version
    its ledger records."""
    held = {}
    for module, _translator, _toolchain in read_ledger(registers):
        name, _, version = module.partition("@")
        if os.path.isfile(os.path.join(registers, name, "frontier.toml")) \
                and (name not in held or version_order(version) > version_order(held[name])):
            held[name] = version
    return held


def estimate(registers):
    """The seconds a crate's attempt is expected to take: its own last
    measured attempt, at any version, or else the mean of the attempts the
    ledger measured; None when it measured none."""
    measured = read_seconds(registers)
    by_name = {}
    for module, took in sorted(measured.items(), key=lambda kv: version_order(kv[0].partition("@")[2])):
        by_name[module.partition("@")[0]] = took
    mean = sum(measured.values()) / len(measured) if measured else None
    return lambda name: by_name.get(name, mean)


def plan_crates(registers, ranked, push, budget):
    """The crates a run lifts, one shard each: on a push, the held crates
    whose attempt is stale; otherwise those first, then the ranking's crates
    the ledger has not attempted, in rank order, while the expected seconds
    of them all fit `budget`. With nothing measured, one new crate is taken,
    whose attempt then measures the rest."""
    ledger = read_ledger(registers)
    held = held_crates(registers)
    take = lambda c: current(registers, f"{c['name']}@{c['version']}") not in ledger
    old = [c for c in ({"name": n, "version": v} for n, v in sorted(held.items())) if take(c)]
    if push:
        return old
    expect = estimate(registers)
    spent = sum(expect(c["name"]) or 0 for c in old)
    out = list(old)
    for c in ranked:
        if c["name"] in held or not take(c):
            continue
        cost = expect(c["name"])
        if cost is None:
            return out + [c]
        if spent + cost > budget:
            break
        spent += cost
        out.append(c)
    return out


def rank_count(registers, budget):
    """How many of the ranking a plan can read from: every crate the ledger
    attempted, and as many more as `budget` holds at the fastest measured
    attempt (one, with nothing measured)."""
    attempted = {m.partition("@")[0] for m, _t, _c in read_ledger(registers)}
    measured = [t for t in read_seconds(registers).values() if t > 0]
    return len(attempted) + (int(budget // min(measured)) if measured else 1)


cmd = sys.argv[1]
if cmd == "--library":
    pass
elif cmd == "stale":
    register, modules = sys.argv[2], json.load(open(sys.argv[3]))
    print(json.dumps([a[0] for a in stale([current(register, m) for m in modules], read_ledger(register))]))
elif cmd == "record":
    register, shards = sys.argv[2], sys.argv[3]
    names = sorted(f[:-len(".toml")] for f in os.listdir(shards) if f.endswith(".toml")) if os.path.isdir(shards) else []
    record(register, names)
    print(f"ledger: {len(names)} attempts recorded in {ledger_path(register)}")
elif cmd == "rank-count":
    print(rank_count(sys.argv[2], float(sys.argv[3])))
elif cmd == "plan-crates":
    registers, ranked, event, budget = sys.argv[2], json.load(open(sys.argv[3])), sys.argv[4], float(sys.argv[5])
    print(json.dumps(plan_crates(registers, ranked, event == "push", budget)))
elif cmd == "record-crates":
    # TIMES holds a file per attempt the shards made, named <name>@<version>
    # and holding the seconds it took; a crate that did not build is an
    # attempt too, made again when its κ moves.
    registers, times = sys.argv[2], sys.argv[3]
    seconds = {f: int(open(os.path.join(times, f)).read().strip())
               for f in (os.listdir(times) if os.path.isdir(times) else []) if "@" in f}
    record(registers, sorted(seconds), seconds)
    print(f"ledger: {len(seconds)} attempts recorded in {ledger_path(registers)}")
elif cmd == "mir-key":
    print(mir_key())
else:
    sys.exit(f"ledger: unknown command {cmd}")
