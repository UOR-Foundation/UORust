"""Planted negative controls for release-platform availability reporting."""
import importlib.util
import hashlib
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parent / "platforms.py"
spec = importlib.util.spec_from_file_location("uorust_platforms", SCRIPT)
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)


class PlatformReportTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.dist = Path(self.tmp.name)
        self.filenames = [f"uorust-{target}{suffix}" for target, suffix, _ in mod.ARCHIVES]
        self._write_archives(self.filenames[:4])

    def _write_archives(self, names):
        for p in self.dist.iterdir():
            p.unlink()
        rows = []
        for name in names:
            data = ("tested archive: " + name).encode()
            (self.dist / name).write_bytes(data)
            rows.append(f"{hashlib.sha256(data).hexdigest()}  {name}")
        (self.dist / "SHA256SUMS").write_text("\n".join(rows) + "\n")

    def test_four_required_archives_are_included_and_windows_is_not(self):
        manifest = mod.describe_release(self.dist, "v0.4.0")
        self.assertEqual([x["status"] for x in manifest["platforms"]],
                         ["included"] * 4 + ["not-included"])
        self.assertIn("no Windows archive", mod.windows_note(manifest))

    def test_windows_present_is_listed_as_included(self):
        self._write_archives(self.filenames)
        manifest = mod.describe_release(self.dist, "v0.4.1")
        self.assertEqual(manifest["platforms"][-1]["status"], "included")
        self.assertIn("this release includes", mod.windows_note(manifest))

    def test_negative_missing_required_archive_refused(self):
        self._write_archives(self.filenames[:3])
        with self.assertRaisesRegex(ValueError, "required release archive"):
            mod.describe_release(self.dist, "v0.4.0")

    def test_negative_tampered_archive_refused(self):
        (self.dist / self.filenames[0]).write_bytes(b"tampered")
        with self.assertRaisesRegex(ValueError, "SHA256 mismatch"):
            mod.describe_release(self.dist, "v0.4.0")

    def test_negative_unlisted_windows_zip_refused(self):
        (self.dist / self.filenames[-1]).write_bytes(b"unlisted optional windows archive")
        with self.assertRaisesRegex(ValueError, "unlisted release archives"):
            mod.describe_release(self.dist, "v0.4.0")

    def test_negative_digest_without_archive_refused(self):
        (self.dist / self.filenames[2]).unlink()
        with self.assertRaisesRegex(ValueError, "listed release archive is missing"):
            mod.describe_release(self.dist, "v0.4.0")

    def test_negative_unexpected_and_duplicate_identity_refused(self):
        rows = (self.dist / "SHA256SUMS").read_text()
        (self.dist / "SHA256SUMS").write_text(rows + rows.splitlines()[0] + "\n")
        with self.assertRaisesRegex(ValueError, "duplicate"):
            mod.describe_release(self.dist, "v0.4.0")
        self._write_archives(self.filenames[:4])
        (self.dist / "SHA256SUMS").write_text(
            (self.dist / "SHA256SUMS").read_text() + "0" * 64 + "  uorust-extra.zip\n")
        with self.assertRaisesRegex(ValueError, "unknown archive"):
            mod.describe_release(self.dist, "v0.4.0")

    def test_platform_manifest_is_checked_after_checksum_append(self):
        manifest = mod.describe_release(self.dist, "v0.4.0")
        path = self.dist / "platforms.json"
        path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        with (self.dist / "SHA256SUMS").open("a") as f:
            f.write(f"{digest}  platforms.json\n")
        self.assertIn("no Windows archive", mod.windows_note(mod.describe_release(self.dist, "v0.4.0")))
        path.write_text(path.read_text().replace("not-included", "included"))
        with self.assertRaisesRegex(ValueError, "manifest checksum mismatch"):
            mod.describe_release(self.dist, "v0.4.0")

    def test_workflow_uses_checked_manifest_to_select_windows_wording(self):
        release = (Path(__file__).resolve().parents[2] / ".github/workflows/release.yml").read_text()
        self.assertIn("--manifest >dist/platforms.json", release)
        self.assertIn("sha256sum platforms.json >>SHA256SUMS", release)
        self.assertIn("--windows-note", release)
        self.assertIn("$windows_note", release)
        self.assertNotIn("(Windows: install.ps1), cargo binstall", release)

    def test_invalid_version_refused(self):
        for tag in ("main", "v0.4.0/../", "0.4.0", "v0.4.0 new"):
            with self.subTest(tag=tag), self.assertRaisesRegex(ValueError, "release tag"):
                mod.describe_release(self.dist, tag)

    def test_cli_outputs_parseable_manifest_and_explanatory_note(self):
        result = subprocess.run([sys.executable, "-I", str(SCRIPT), str(self.dist), "v0.4.0", "--manifest"],
                                capture_output=True, text=True, check=True)
        manifest = json.loads(result.stdout)
        self.assertEqual(manifest["schema"], "uorust/release-platforms/1")
        note = subprocess.run([sys.executable, "-I", str(SCRIPT), str(self.dist), "v0.4.0", "--windows-note"],
                              capture_output=True, text=True, check=True).stdout
        self.assertIn("no Windows archive", note)


if __name__ == "__main__":
    unittest.main()
