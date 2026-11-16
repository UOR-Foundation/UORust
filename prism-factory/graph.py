#!/usr/bin/env python3
"""The writer and the checks of the tool's factory component.

    graph.py write            write model/src/Factory/UORust.lex.tex and the README's graph table
    graph.py check            refuse a stale model text or table, an incomplete graph, or drift
                              of an installed file from upstream.toml
    graph.py derive UPSTREAM  write model/src/Factory.lex.tex from upstream's (a prism-factory/ directory)
    graph.py upstream UPSTREAM  compare every installed file with upstream's at the named revision
    graph.py record UPSTREAM REVISION  write upstream.toml: how each installed file stands to
                              upstream's at REVISION, with both SHA-256s
    graph.py request TARGETS  write production-request.json (the roots `plan` and `reweigh`, each
                              pinned to the rust-std descriptor TARGETS, the JSON of
                              `lexlean production targets`, states); `request-check TARGETS` refuses a stale one
    graph.py runs             print each obligation's name and run, one per line (for tests)

The graph's one source is graph.toml with the Justfile's recipe bodies. This
script writes the model text the certified core is exported from; it decides
no route, identity or order (the certified core does).
"""
import hashlib
import json
import pathlib
import re
import shlex
import sys
import tomllib

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent
GRAPH = HERE / "graph.toml"
JUSTFILE = ROOT / "Justfile"
MODULE = HERE / "model/src/Factory/UORust.lex.tex"
ROOT_MODULE = HERE / "model/src/Factory.lex.tex"
README = HERE / "README.md"
UPSTREAM = HERE / "upstream.toml"
REQUEST = HERE / "production-request.json"
ROOTS = ["plan", "reweigh"]
BEGIN, END = "<!-- graph:begin (graph.py write) -->", "<!-- graph:end -->"

# The definitions of upstream's Factory.lex.tex the tool's root module keeps:
# the request protocol, the plan and the reweigh roots. The release workflow,
# the cut and the forge's delegation are PrismPM's own release and are not
# installed; `derive` refuses an upstream that lacks any of these.
KEPT = [
    "Request", "cut", "passes", "readLine", "reversedTree", "reversedPairs", "readRequest",
    "levelOf", "insertByLevel", "schedule", "words", "tagged", "closureFiles", "runsHere",
    "describe", "render", "plan", "outcomeOf", "reweigh",
]
# How each installed file stands to upstream's: byte for byte; derived by
# `derive`; reduced (upstream's code with only the release and forge parts
# left out, as the file's own header says); relocked (upstream's lock with this
# compiler's semantics digest).
INSTALLED = {
    "lexlean-workspace.toml": "identical",
    "model/lakefile.toml": "identical",
    "model/lean-toolchain": "identical",
    "model/lake-manifest.json": "identical",
    "model/lexlean.toml": "identical",
    "model/lexlean.lock": "relocked",
    "model/src/Factory/Engine.lex.tex": "identical",
    "model/src/Factory.lex.tex": "derived",
    "std/lakefile.toml": "identical",
    "std/lean-toolchain": "identical",
    "std/lake-manifest.json": "identical",
    "std/lexlean.toml": "identical",
    "std/lexlean.lock": "relocked",
    "std/src/Buffer.lex.tex": "identical",
    "std/src/Sha256.lex.tex": "identical",
    "std/src/Text.lex.tex": "identical",
    "host/src/lib.rs": "reduced",
    "host/src/main.rs": "reduced",
}
KEPT_IMPORTS = ["LexLeanStd.Buffer", "LexLeanStd.Text", "Factory.Engine", "Factory.UORust"]


class Refused(Exception):
    pass


def justfile_recipes(text):
    """Recipe name -> (parameters with defaults, dependencies, body lines)."""
    assignments = dict(re.findall(r'^([A-Za-z_][A-Za-z0-9_-]*) := "(.*)"$', text, re.M))
    recipes = {}
    lines = text.split("\n")
    for index, line in enumerate(lines):
        match = re.match(r"^([A-Za-z0-9][A-Za-z0-9_-]*)((?: +[*+]?[A-Za-z_]+(?:=\"[^\"]*\")?)*) *:(?!=)(.*)$", line)
        if not match:
            continue
        name, params, deps = match.group(1), match.group(2), match.group(3)
        defaults = {}
        for param in params.split():
            key, _, value = param.lstrip("*+").partition("=")
            defaults[key] = value.strip('"') if value else None
        body = []
        for row in lines[index + 1:]:
            if not row.startswith("    "):
                break
            body.append(row[4:])
        recipes[name] = (defaults, deps.split(), body, assignments)
    return recipes


def recipe_command(recipes, name):
    if name not in recipes:
        raise Refused(f"graph.toml names recipe {name}, which the Justfile does not define")
    defaults, _, body, assignments = recipes[name]
    if body and body[0].startswith("#!"):
        raise Refused(f"recipe {name} is a script recipe; an obligation's run is a line recipe")
    commands = []
    for row in body:
        row = row.strip()
        if not row or row.startswith("#"):
            continue
        row = row.lstrip("@")

        def value(found):
            key = found.group(1).strip()
            if key in defaults:
                if defaults[key] is None:
                    raise Refused(f"recipe {name} takes {key} without a default")
                return defaults[key]
            if key in assignments:
                return assignments[key]
            raise Refused(f"recipe {name}: {{{{{key}}}}} is not a variable or a parameter with a default")

        commands.append(re.sub(r"\{\{([^}]*)\}\}", value, row))
    if not commands:
        raise Refused(f"recipe {name} has no command")
    return commands[0] if len(commands) == 1 else " && ".join(f"({c})" for c in commands)


def leaves(recipes, name):
    """The recipes a dependency-only recipe stands for, in order."""
    _, deps, body, _ = recipes[name]
    if any(row.strip() and not row.strip().startswith("#") for row in body) or not deps:
        return [name]
    return [leaf for dep in deps for leaf in leaves(recipes, dep)]


def load():
    graph = tomllib.loads(GRAPH.read_text(encoding="utf-8"))
    recipes = justfile_recipes(JUSTFILE.read_text(encoding="utf-8"))
    obligations = []
    for row in graph["obligation"]:
        inputs = [item for set_name in row.get("sets", []) for item in graph["sets"][set_name]] + row["inputs"]
        command = recipe_command(recipes, row["recipe"])
        run = f"bash prism-factory/sdk.sh {shlex.quote(command)}" if row["sdk"] else command
        if row.get("writes"):
            run = f"bash prism-factory/written.sh {' '.join(row['writes'])} -- {run}"
        obligations.append({
            "name": row["name"], "kind": row["kind"], "environment": "host", "run": run,
            "inputs": inputs, "needs": row.get("needs", []), "sdk": row["sdk"],
            "writes": row.get("writes", []), "untouched": row["untouched"],
        })
    return graph, recipes, obligations


def validate(graph, recipes, obligations):
    problems = []
    names = [item["name"] for item in obligations]
    if len(set(names)) != len(names):
        problems.append("two obligations share a name")
    seen = set()
    for item in obligations:
        if item["kind"] not in ("hermetic", "fetch"):
            problems.append(f"{item['name']}: kind {item['kind']}")
        for need in item["needs"]:
            if need not in seen:
                problems.append(f"{item['name']} needs {need}, which is not declared before it")
        seen.add(item["name"])
        if "" in item["inputs"]:
            problems.append(f"{item['name']} reads the whole tree; declare what it reads")
        if len(set(item["inputs"])) != len(item["inputs"]):
            problems.append(f"{item['name']} declares an input twice")
        if item["sdk"] and not {"prismpm.lock", "prism-factory/sdk.sh"} <= set(item["inputs"]):
            problems.append(f"{item['name']} runs in the SDK without declaring prismpm.lock and prism-factory/sdk.sh")
        if item["writes"] and "prism-factory/written.sh" not in item["inputs"]:
            problems.append(f"{item['name']} checks a writer without declaring prism-factory/written.sh")
        if "just " in item["run"] and "Justfile" not in item["inputs"]:
            problems.append(f"{item['name']} calls just without declaring the Justfile")
    gate = [leaf for dep in recipes["vv"][1] for leaf in leaves(recipes, dep)]
    vv = next((item for item in obligations if item["name"] == "vv"), None)
    if vv is None or vv["needs"] != gate:
        problems.append(f"the vv obligation must need exactly the gate's parts, in order: {gate}")
    checked = {row["recipe"]: row["checked_by"] for row in graph.get("writer", [])}
    for writer in [leaf for dep in recipes["generate"][1] for leaf in leaves(recipes, dep)]:
        if checked.get(writer) not in names:
            problems.append(f"the writer {writer} of `just generate` has no obligation that checks its output")
    if problems:
        raise Refused("\n".join(problems))


def lex_string(text):
    return json.dumps(text, ensure_ascii=True)


def lex_list(items, kind="String"):
    out = f"(nil {kind})"
    for item in reversed(items):
        out = f"(cons {item} {out})"
    return out


def module_text(obligations):
    def obligation(item):
        kind = f"(ctor Factory.Engine::Kind.{item['kind']})"
        inputs = lex_list([lex_string(i) for i in item["inputs"]])
        needs = lex_list([lex_string(n) for n in item["needs"]])
        return f"(obligation {lex_string(item['name'])} {kind} {lex_string(item['environment'])} {lex_string(item['run'])} {inputs} {needs})"

    graph = lex_list([obligation(item) for item in obligations], "Factory.Engine::Obligation")
    in_sdk = lex_list([lex_string(item["name"]) for item in obligations if item["sdk"]])
    count = len(obligations)
    definitions = [
        ("definition", "obligation", [
            "parameters: [name : String, kind : Factory.Engine::Kind, environment : String, run : String, inputs : (List String), needs : (List String)]",
            "result: Factory.Engine::Obligation",
            "body: (record Factory.Engine::Obligation {name := name, kind := kind, environment := environment, run := run, inputs := inputs, needs := needs})",
            "executable: true"]),
        ("definition", "graph", [
            "parameters: []", "result: (List Factory.Engine::Obligation)", f"body: {graph}", "executable: true"]),
        ("definition", "inSdk", [
            "parameters: []", "result: (List String)", f"body: {in_sdk}", "executable: true"]),
        ("definition", "declaring", [
            "parameters: [input : String, items : (List Factory.Engine::Obligation)]",
            "result: Nat",
            "body: (prim list_fold (fun [input] (sofar : Nat) (item : Factory.Engine::Obligation) => (if (Factory.Engine::member input (project item inputs)) then (sofar + 1) else sofar)) 0 items : Nat)",
            "executable: true"]),
        ("definition", "declaredWhere", [
            "parameters: [input : String, chosen : (List String), items : (List Factory.Engine::Obligation)]",
            "result: Bool",
            "body: (prim list_fold (fun [chosen, input] (sofar : Bool) (item : Factory.Engine::Obligation) => (sofar && ((not (Factory.Engine::member (project item name) chosen)) || (Factory.Engine::member input (project item inputs))))) true items : Bool)",
            "executable: true"]),
        ("theorem", "every_obligation_has_a_distinct_name", [
            "parameters: []", "statement: ((Factory.Engine::distinct (Factory.Engine::names (graph))) = true)",
            "proof: decide{}", "axioms: []"]),
        ("theorem", "every_need_is_declared_before_it_is_needed", [
            "parameters: []", "statement: ((Factory.Engine::needsComeFirst (graph)) = true)",
            "proof: decide{}", "axioms: []"]),
        ("theorem", "every_obligation_declares_its_inputs_and_its_run", [
            "parameters: []", "statement: ((Factory.Engine::allDeclared (graph)) = true)",
            "proof: decide{}", "axioms: []"]),
        ("theorem", "no_obligation_reads_the_whole_tree", [
            "parameters: []", "statement: ((declaring \"\" (graph)) = 0)",
            "proof: decide{}", "axioms: []"]),
        ("theorem", "every_obligation_in_the_sdk_declares_its_image_and_its_launcher", [
            "parameters: []",
            "statement: (((declaredWhere \"prismpm.lock\" (inSdk) (graph)) && (declaredWhere \"prism-factory/sdk.sh\" (inSdk) (graph))) = true)",
            "proof: decide{}", "axioms: []"]),
        ("theorem", f"the_graph_has_{count}_obligations", [
            "parameters: []", f"statement: ((prim length (graph) : Nat) = {count})",
            "proof: decide{}", "axioms: []"]),
    ]
    blocks = []
    for kind, name, fields in definitions:
        body = ",\n".join(f"  {field}" for field in [f"name: {name}"] + fields)
        blocks.append(f"{kind}{{\n{body}\n}}")
    return (
        "\\begin{lexlean}{Factory.UORust}\n"
        "\\useglossary{lexlean.std.nat@1.2.0}\n"
        "\\importmodule{Factory.Engine}\n"
        "\\title{Natural number addition}\n"
        "\\begin{semanticmodule}\n"
        + "\n\n".join(blocks)
        + "\n\\end{semanticmodule}\n\\end{lexlean}\n"
    )


def table_text(obligations):
    rows = ["| Obligation | Kind | Needs | Reads | Not touched by |", "| --- | --- | --- | --- | --- |"]
    for item in obligations:
        reads = ", ".join(f"`{i}`" for i in item["inputs"])
        needs = ", ".join(f"`{n}`" for n in item["needs"]) or "nothing"
        rows.append(f"| `{item['name']}` | {item['kind']} | {needs} | {reads} | {item['untouched']} |")
    return "\n".join(rows) + "\n"


def readme_with(text, table):
    if BEGIN not in text or END not in text:
        raise Refused(f"{README} has no {BEGIN} ... {END} section")
    head, rest = text.split(BEGIN, 1)
    _, tail = rest.split(END, 1)
    return f"{head}{BEGIN}\n{table}{END}{tail}"


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def derive(upstream_text):
    head, rest = upstream_text.split("\\begin{semanticmodule}\n", 1)
    body, tail = rest.split("\\end{semanticmodule}\n", 1)
    blocks = {}
    for block in re.split(r"\n\n+", body.strip("\n")):
        found = re.search(r"^  name: ([A-Za-z_][A-Za-z0-9_]*),$", block, re.M)
        if found:
            blocks[found.group(1)] = block
    missing = [name for name in KEPT if name not in blocks]
    if missing:
        raise Refused(f"upstream's Factory.lex.tex lacks {missing}")
    header = [line for line in head.split("\n") if line and not line.startswith("\\importmodule")]
    imports = [f"\\importmodule{{{name}}}" for name in KEPT_IMPORTS]
    kept = "\n\n".join(blocks[name] for name in KEPT).replace("Factory.PrismPM::graph", "Factory.UORust::graph")
    return "\n".join(header[:2] + imports + header[2:]) + "\n\\begin{semanticmodule}\n" + kept + "\n" + "\\end{semanticmodule}\n" + tail


def request_text(targets_json):
    targets = json.loads(targets_json)
    digest = next(target["digest"] for target in targets["targets"] if target["id"] == "rust-std")
    roots = [{"package": {"name": f"factory_{root}", "version": "0.1.0"}, "root": f"PrismFactory.Factory.{root}",
              "targets": [{"digest": digest, "id": "rust-std"}]} for root in ROOTS]
    return json.dumps({"roots": roots, "spec": "lexlean/production-request/1"}, indent=2) + "\n"


def unlocked(text):
    return re.sub(r'^compiler_semantics = "[0-9a-f]{64}"$', "", text, flags=re.M)


def record_text(source, revision):
    lines = [
        "# The upstream Prism factory this component is installed from, and how each",
        "# installed file stands to upstream's there. Written by `graph.py record`;",
        "# `graph.py check` refuses an installed file whose SHA-256 differs.",
        'repository = "https://github.com/UOR-Foundation/prismpm"',
        f'revision = "{revision}"',
        'directory = "prism-factory"',
    ]
    for path, how in INSTALLED.items():
        lines += ["", "[[file]]", f'path = "{path}"', f'how = "{how}"',
                  f'upstream_sha256 = "{sha256(source / path)}"', f'sha256 = "{sha256(HERE / path)}"']
    return "\n".join(lines) + "\n"


def drift():
    record = tomllib.loads(UPSTREAM.read_text(encoding="utf-8"))
    problems = []
    for row in record["file"]:
        path = HERE / row["path"]
        if not path.is_file():
            problems.append(f"{row['path']}: missing")
        elif sha256(path) != row["sha256"]:
            problems.append(f"{row['path']}: differs from what upstream.toml records ({row['how']} at {record['revision']})")
    return record, problems


def main(argv):
    command = argv[1] if len(argv) > 1 else ""
    if command == "write":
        graph, recipes, obligations = load()
        validate(graph, recipes, obligations)
        MODULE.write_text(module_text(obligations), encoding="utf-8")
        README.write_text(readme_with(README.read_text(encoding="utf-8"), table_text(obligations)), encoding="utf-8")
        print(f"graph.py: wrote {MODULE.relative_to(ROOT)} and the README's table ({len(obligations)} obligations)")
    elif command == "check":
        graph, recipes, obligations = load()
        validate(graph, recipes, obligations)
        problems = []
        if MODULE.read_text(encoding="utf-8") != module_text(obligations):
            problems.append(f"{MODULE.relative_to(ROOT)} is not what graph.py writes; run prism-factory/graph.py write")
        if README.read_text(encoding="utf-8") != readme_with(README.read_text(encoding="utf-8"), table_text(obligations)):
            problems.append("the README's graph table is not what graph.py writes; run prism-factory/graph.py write")
        record, drifted = drift()
        problems += drifted
        if problems:
            raise Refused("\n".join(problems))
        print(f"graph.py: the model text and the table are the graph's ({len(obligations)} obligations); "
              f"{len(record['file'])} installed files are as upstream.toml records at {record['revision'][:12]}")
    elif command == "derive" and len(argv) == 3:
        ROOT_MODULE.write_text(derive((pathlib.Path(argv[2]) / "model/src/Factory.lex.tex").read_text(encoding="utf-8")), encoding="utf-8")
        print(f"graph.py: wrote {ROOT_MODULE.relative_to(ROOT)}")
    elif command == "upstream" and len(argv) == 3:
        source = pathlib.Path(argv[2])
        record, problems = drift()
        for row in record["file"]:
            original = source / row["path"]
            if not original.is_file():
                problems.append(f"{row['path']}: upstream has no such file")
            elif sha256(original) != row["upstream_sha256"]:
                problems.append(f"{row['path']}: upstream's differs from what upstream.toml records")
            elif row["how"] == "identical" and original.read_bytes() != (HERE / row["path"]).read_bytes():
                problems.append(f"{row['path']}: not byte-identical to upstream's")
            elif row["how"] == "relocked" and unlocked(original.read_text(encoding="utf-8")) != unlocked((HERE / row["path"]).read_text(encoding="utf-8")):
                problems.append(f"{row['path']}: differs from upstream's beyond the compiler's semantics digest")
            elif row["how"] == "derived" and derive(original.read_text(encoding="utf-8")) != (HERE / row["path"]).read_text(encoding="utf-8"):
                problems.append(f"{row['path']}: not what graph.py derives from upstream's")
        if problems:
            raise Refused("\n".join(problems))
        print(f"graph.py: {len(record['file'])} installed files hold to upstream {record['repository']} at {record['revision']}")
    elif command == "request" and len(argv) == 3:
        REQUEST.write_text(request_text(pathlib.Path(argv[2]).read_text(encoding="utf-8")), encoding="utf-8")
        print(f"graph.py: wrote {REQUEST.relative_to(ROOT)}")
    elif command == "request-check" and len(argv) == 3:
        if REQUEST.read_text(encoding="utf-8") != request_text(pathlib.Path(argv[2]).read_text(encoding="utf-8")):
            raise Refused(f"{REQUEST.relative_to(ROOT)} does not pin this compiler's rust-std descriptor; run graph.py request")
        print(f"graph.py: {REQUEST.relative_to(ROOT)} pins this compiler's rust-std descriptor")
    elif command == "record" and len(argv) == 4:
        if not re.fullmatch(r"[0-9a-f]{40}", argv[3]):
            raise Refused("the revision is a full commit hash")
        UPSTREAM.write_text(record_text(pathlib.Path(argv[2]), argv[3]), encoding="utf-8")
        print(f"graph.py: wrote {UPSTREAM.relative_to(ROOT)}")
    elif command == "runs":
        _, _, obligations = load()
        for item in obligations:
            print(json.dumps({"name": item["name"], "kind": item["kind"], "run": item["run"],
                              "inputs": item["inputs"], "needs": item["needs"]}))
    else:
        print(__doc__, file=sys.stderr)
        return 64
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main(sys.argv))
    except Refused as refusal:
        print(f"graph.py: {refusal}", file=sys.stderr)
        sys.exit(1)
