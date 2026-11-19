"""Static negative controls for failed workflow_run installer fanout.

Checks workflow source. Hosted GitHub event behavior is verified separately.
"""
import re
import unittest
from pathlib import Path

WORKFLOW = Path(__file__).resolve().parents[2] / ".github/workflows/install.yml"
INSTALLERS = {
    "package", "release-archive", "install-sh", "install-ps1",
    "binstall", "cargo-install", "homebrew", "nix",
}
SUCCESS_ONLY = (
    "github.event_name != 'workflow_run' || "
    "github.event.workflow_run.conclusion == 'success'"
)
REFUSAL = (
    "github.event_name == 'workflow_run' && "
    "github.event.workflow_run.conclusion != 'success'"
)


def job_blocks(source):
    blocks = {}
    current = None
    in_jobs = False
    for line in source.splitlines():
        if line == "jobs:":
            in_jobs = True
            continue
        if not in_jobs:
            continue
        match = re.fullmatch(r"  ([a-z][a-z0-9-]*):", line)
        if match:
            current = match.group(1)
            blocks[current] = []
        elif current is not None:
            blocks[current].append(line)
    return {name: "\n".join(lines) for name, lines in blocks.items()}


def verify(source):
    blocks = job_blocks(source)
    if set(blocks) != INSTALLERS | {"upstream-refusal"}:
        raise ValueError("missing or additional installer/refusal job")
    for name in INSTALLERS:
        conditions = re.findall(r"^    if: (.+)$", blocks[name], re.M)
        if conditions != ["${{ " + SUCCESS_ONLY + " }}"]:
            raise ValueError(f"{name} can run after a failed upstream workflow")
    refusal = blocks["upstream-refusal"]
    if re.findall(r"^    if: (.+)$", refusal, re.M) != [
        "${{ " + REFUSAL + " }}"
    ] or "          exit 1" not in refusal:
        raise ValueError("failed upstream can appear as passing installation")
    return True


class InstallUpstreamGateTest(unittest.TestCase):
    def test_exact_workflow_guards_every_installer_and_reports_refusal(self):
        self.assertTrue(verify(WORKFLOW.read_text()))

    def test_planted_unconditional_installer_is_rejected(self):
        source = WORKFLOW.read_text()
        guarded = "    if: ${{ " + SUCCESS_ONLY + " }}\n"
        self.assertIn(guarded, source)
        with self.assertRaisesRegex(ValueError, "can run after"):
            verify(source.replace(guarded, "", 1))

    def test_planted_false_green_is_rejected(self):
        source = WORKFLOW.read_text()
        self.assertIn("          exit 1", source)
        with self.assertRaisesRegex(ValueError, "passing installation"):
            verify(source.replace("          exit 1", "          exit 0", 1))


if __name__ == "__main__":
    unittest.main()
