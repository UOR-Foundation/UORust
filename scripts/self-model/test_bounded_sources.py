"""Test the real facets generator's bounded, provenance-faithful list emitter.

Pure functions are extracted from the generator's AST so no model artifacts
are written. The locked-SDK just vv still owns semantic acceptance.
"""
import ast
import re
import unittest
from pathlib import Path

GENERATOR = Path(__file__).with_name("facets.py")
NAMES = {"q", "defn", "source_list", "bounded_source_definitions"}
CONSTANTS = {"SOURCE_LIST_TYPE", "SOURCE_LIST_RUN_SIZE"}


def isolated_generator():
    tree = ast.parse(GENERATOR.read_text(encoding="utf-8"), filename=str(GENERATOR))
    nodes = [node for node in tree.body if (
        isinstance(node, ast.FunctionDef) and node.name in NAMES
        or isinstance(node, ast.Assign) and any(
            isinstance(t, ast.Name) and t.id in CONSTANTS for t in node.targets)
    )]
    assert {n.name for n in nodes if isinstance(n, ast.FunctionDef)} == NAMES
    ns = {"SOURCES": {}}
    exec(compile(ast.fix_missing_locations(ast.Module(body=nodes, type_ignores=[])),
                 str(GENERATOR), "exec"), ns)
    return ns


def fixture(count):
    return [(f"Rec{i:04}", f"src/{i}.toml", i + 1, i + 3, f"{i:064x}")
            for i in range(count)]


def evaluated_records(source, final):
    """Evaluate the generated acyclic list/append graph, not arbitrary terms."""
    definitions = {}
    for match in re.finditer(
        r"definition\{\n  name: ([A-Za-z0-9]+),\n  parameters: \[\],\n"
        r"  result: \(List Architecture::Source\),\n  body: ([^\n]+)\n\}", source
    ):
        name, body = match.groups()
        assert name not in definitions, f"duplicate declaration {name}"
        definitions[name] = body
    assert final in definitions, f"missing public {final}"
    memo, stack = {}, set()

    def resolve(name):
        if name in memo:
            return memo[name]
        assert name not in stack, f"dependency cycle at {name}"
        stack.add(name)
        body = definitions[name]
        append = re.fullmatch(
            r"\(prim append \(([A-Za-z0-9]+)\) \(([A-Za-z0-9]+)\)"
            r" : \(List Architecture::Source\)\)", body
        )
        if append:
            values = resolve(append[1]) + resolve(append[2])
        elif re.fullmatch(r"\([A-Za-z0-9]+\)", body):
            values = resolve(body[1:-1])
        else:
            values = re.findall(r'record := "(Rec\d{4})"', body)
            assert body.count("(cons (record Architecture::Source") == len(values)
        stack.remove(name)
        memo[name] = values
        return values

    return resolve(final), definitions


class BoundedSourceListTests(unittest.TestCase):
    def setUp(self):
        self.ns = isolated_generator()

    def test_boundary_counts_preserve_order_and_provenance(self):
        for count in (0, 1, 16, 17, 32, 33, 255, 256, 500, 1000):
            with self.subTest(count=count):
                records = fixture(count)
                self.ns["SOURCES"]["Quality"] = records
                output = self.ns["bounded_source_definitions"]("Quality", "sourcesOfMeasures")
                actual, declarations = evaluated_records(output, "sourcesOfMeasures")
                self.assertEqual(actual, [x[0] for x in records])
                self.assertEqual(output.count('record := "Rec'), count)
                for rec, path, first, last, kappa in records:
                    self.assertIn(
                        f'record := "{rec}", path := "{path}", firstLine := {first}, '
                        f'lastLine := {last}, kappa := "{kappa}"', output)
                self.assertLessEqual(max((v.count("(cons (") for v in declarations.values()),
                                         default=0), 16)
                self.assertLessEqual(max((v.count("(prim append") for v in declarations.values()),
                                         default=0), 1)
                self.assertEqual(output.count("  name: sourcesOfMeasures,\n"), 1)

    def test_old_500_record_source_is_overdepth(self):
        self.ns["SOURCES"]["Quality"] = fixture(500)
        original = self.ns["source_list"]("Quality")
        self.assertGreater(original.count("(cons (record Architecture::Source"), 256)
        revised = self.ns["bounded_source_definitions"]("Quality", "sourcesOfMeasures")
        _, declarations = evaluated_records(revised, "sourcesOfMeasures")
        self.assertTrue(all(body.count("(cons (") <= 16 for body in declarations.values()))

    def test_tampering_changes_generated_evidence(self):
        records = fixture(35)
        self.ns["SOURCES"]["Quality"] = records
        initial = self.ns["bounded_source_definitions"]("Quality", "sourcesOfMeasures")
        changed = records.copy()
        name, path, first, last, _ = changed[18]
        changed[18] = (name, path, first, last, "f" * 64)
        self.ns["SOURCES"]["Quality"] = changed
        self.assertNotEqual(initial,
                            self.ns["bounded_source_definitions"]("Quality", "sourcesOfMeasures"))

    def test_quality_uses_bounded_emitter(self):
        source = GENERATOR.read_text(encoding="utf-8")
        self.assertIn('bounded_source_definitions(Qm, "sourcesOfMeasures")', source)
        self.assertNotIn('defn("sourcesOfMeasures", "(List Architecture::Source)", source_list(Qm))',
                         source)


if __name__ == "__main__":
    unittest.main()
