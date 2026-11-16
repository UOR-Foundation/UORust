#!/usr/bin/env python3
"""An independent oracle of the factory's identity protocol (prism-factory/README.md):
reads the graph (`graph.py runs`, one JSON object per line) on argv[1] and a request's
`tree` lines on stdin, and prints `name identity` per obligation, from Python's hashlib."""
import hashlib, json, sys

def reach(inp, path):
    prefix = inp[1:] if inp.startswith("!") else inp
    if not path.startswith(prefix):
        return 0
    if path.startswith("@"):
        return len(prefix.encode()) + 1 if prefix.startswith("@") else 0
    return len(prefix.encode()) + 1

def covered(inputs, path):
    include = max([reach(i, path) for i in inputs if not i.startswith("!")] or [0])
    exclude = max([reach(i, path) for i in inputs if i.startswith("!")] or [0])
    return exclude < include

tree = []
for line in sys.stdin.read().splitlines():
    tag, obj, path = line.split(" ", 2)
    if tag == "tree":
        tree.append((obj, path))
known = {}
for row in open(sys.argv[1], encoding="utf-8"):
    item = json.loads(row)
    closure = "".join(f"file {obj} {path}\n" for obj, path in tree if covered(item["inputs"], path))
    text = "prism-factory/obligation/1\n" + f"name {item['name']}\n" + f"kind {item['kind']}\n" + "environment host\n"
    text += f"run {item['run']}\n" + "".join(f"input {i}\n" for i in item["inputs"])
    text += "".join(f"needs {n} {known[n]}\n" for n in item["needs"])
    text += f"closure {hashlib.sha256(closure.encode()).hexdigest()}\n"
    known[item["name"]] = hashlib.sha256(text.encode()).hexdigest()
    print(item["name"], known[item["name"]])
