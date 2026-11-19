#!/usr/bin/env python3
"""Describe supported UORust release archives from checked SHA256SUMS bytes.

Usage: python3 -I scripts/release/platforms.py DIST TAG --manifest|--windows-note

This describes availability; it does not authenticate the producer. Check
archive build attestations separately with `gh attestation verify`.
"""

import argparse
import hashlib
import json
import re
from pathlib import Path

ARCHIVES = (
    ("x86_64-unknown-linux-gnu", ".tar.gz", True),
    ("aarch64-unknown-linux-gnu", ".tar.gz", True),
    ("x86_64-apple-darwin", ".tar.gz", True),
    ("aarch64-apple-darwin", ".tar.gz", True),
    ("x86_64-pc-windows-msvc", ".zip", False),
)
DIGEST_ROW = re.compile(r"([0-9a-fA-F]{64}) [ *]([A-Za-z0-9_.-]+)")
TAG = re.compile(r"v[0-9]+\.[0-9]+\.[0-9]+(?:[-+][A-Za-z0-9_.+-]+)?")


def describe_release(dist: Path, tag: str):
    if not TAG.fullmatch(tag):
        raise ValueError("release tag must be v<semantic-version>")
    rows = (dist / "SHA256SUMS").read_text(encoding="utf-8").splitlines()
    entries = {}
    for row in rows:
        match = DIGEST_ROW.fullmatch(row)
        if not match:
            raise ValueError("invalid SHA256SUMS entry")
        digest, filename = match.groups()
        if filename in entries:
            raise ValueError("duplicate release archive identity")
        entries[filename] = digest.lower()
    expected = {f"uorust-{target}{suffix}" for target, suffix, _ in ARCHIVES}
    if set(entries) - (expected | {"platforms.json"}):
        raise ValueError("unknown archive identity in SHA256SUMS")
    # No unlisted binary may ride along in release.yml's `gh release ... dist/*`.
    # In particular, an optional Windows .zip without its .sha256 sidecar
    # must be refused, not published as a misleading unverified asset.
    staged_archives = {p.name for p in dist.iterdir() if p.is_file()
                       and (p.name.endswith(".tar.gz") or p.name.endswith(".zip"))}
    unlisted_archives = staged_archives - set(entries)
    if unlisted_archives:
        raise ValueError(f"unlisted release archives: {sorted(unlisted_archives)}")
    platforms = []
    for target, suffix, required in ARCHIVES:
        filename = f"uorust-{target}{suffix}"
        digest = entries.get(filename)
        if required and not digest:
            raise ValueError(f"required release archive is absent: {filename}")
        if digest:
            artifact = dist / filename
            if not artifact.is_file():
                raise ValueError(f"listed release archive is missing: {filename}")
            actual = hashlib.sha256(artifact.read_bytes()).hexdigest()
            if actual != digest:
                raise ValueError(f"SHA256 mismatch for listed archive: {filename}")
        platforms.append({
            "target": target,
            "required": required,
            "status": "included" if digest else "not-included",
            "archive": filename if digest else None,
            "sha256": digest,
        })
    manifest = {"schema": "uorust/release-platforms/1", "tag": tag, "platforms": platforms}
    if "platforms.json" in entries:
        path = dist / "platforms.json"
        if not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest() != entries["platforms.json"]:
            raise ValueError("release-platform manifest checksum mismatch")
        if json.loads(path.read_text(encoding="utf-8")) != manifest:
            raise ValueError("release-platform manifest disagrees with archives")
    return manifest


def windows_note(manifest):
    windows = next(p for p in manifest["platforms"] if "windows" in p["target"])
    if windows["status"] == "included":
        return ("Windows: this release includes the x64 Windows ZIP, which install.ps1 "
                "checks against SHA256SUMS (ARM64 runs x64 under emulation).")
    return ("Windows: no Windows archive was published for this release. "
            "install.ps1 refuses a missing archive; build from source if needed.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("dist", type=Path)
    parser.add_argument("tag")
    choices = parser.add_mutually_exclusive_group(required=True)
    choices.add_argument("--manifest", action="store_true")
    choices.add_argument("--windows-note", action="store_true")
    args = parser.parse_args()
    result = describe_release(args.dist, args.tag)
    if args.manifest:
        print(json.dumps(result, indent=2, sort_keys=True))
    else:
        print(windows_note(result))


if __name__ == "__main__":
    main()
