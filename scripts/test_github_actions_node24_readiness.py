#!/usr/bin/env python3
"""Regression checks for GitHub-hosted Node 24-compatible workflow actions."""
from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = ROOT / ".github" / "workflows"

MINIMUM_NODE24_ACTION_MAJORS = {
    "actions/checkout": 6,
    "actions/setup-python": 6,
    "actions/cache": 5,
    "actions/upload-artifact": 6,
    "actions/download-artifact": 8,
    "github/codeql-action/init": 4,
    "github/codeql-action/autobuild": 4,
    "github/codeql-action/analyze": 4,
}

USES_RE = re.compile(r"uses:\s*[\"']?((?:actions/[a-z-]+)|(?:github/codeql-action/[a-z-]+))@v([0-9]+)")


class GithubActionsNode24ReadinessTests(unittest.TestCase):
    def test_actions_use_node24_capable_majors(self) -> None:
        failures: list[str] = []
        scanned = 0

        for workflow in sorted(WORKFLOWS.glob("*.yml")):
            text = workflow.read_text(encoding="utf-8")
            for match in USES_RE.finditer(text):
                action = match.group(1)
                major = int(match.group(2))
                minimum = MINIMUM_NODE24_ACTION_MAJORS.get(action)
                if minimum is None:
                    continue
                scanned += 1
                if major < minimum:
                    failures.append(
                        f"{workflow.relative_to(ROOT)} pins {action}@v{major}; expected v{minimum}+",
                    )

        self.assertGreater(scanned, 0, "expected to scan at least one Node 24-gated workflow action")
        self.assertEqual([], failures)

    def test_no_workflow_pins_explicit_node20_action_variants(self) -> None:
        offenders = []
        for workflow in sorted(WORKFLOWS.glob("*.yml")):
            text = workflow.read_text(encoding="utf-8")
            if "node20" in text.lower():
                offenders.append(str(workflow.relative_to(ROOT)))

        self.assertEqual([], offenders)


if __name__ == "__main__":
    unittest.main()
