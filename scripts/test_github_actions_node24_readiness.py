#!/usr/bin/env python3
"""Regression checks for GitHub-hosted Node 24-compatible workflow actions."""
from __future__ import annotations

import re
import json
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = ROOT / ".github" / "workflows"
PIN_MANIFEST = ROOT / ".github" / "rulesets" / "external-action-pins.json"

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

USES_RE = re.compile(
    r"uses:\s*[\"']?((?:actions/[a-z-]+)|(?:github/codeql-action/[a-z-]+))@([0-9a-f]{40})"
)


def node24_pin_majors(manifest: Path = PIN_MANIFEST) -> dict[tuple[str, str], int]:
    value = json.loads(manifest.read_text(encoding="utf-8"))
    rows = value.get("entries", []) if isinstance(value, dict) else []
    result: dict[tuple[str, str], int] = {}
    for row in rows:
        if not isinstance(row, dict):
            continue
        action, commit, source_ref = (
            row.get("action"), row.get("commit"), row.get("source_ref")
        )
        match = re.fullmatch(r"v([0-9]+)", source_ref or "")
        if isinstance(action, str) and isinstance(commit, str) and match:
            result[(action, commit)] = int(match.group(1))
    return result


def workflow_files(workflows: Path) -> list[Path]:
    return sorted([*workflows.glob("*.yml"), *workflows.glob("*.yaml")])


def workflow_label(workflow: Path, display_base: Path) -> str:
    return str(workflow.relative_to(display_base))


def scan_action_pins(
    workflows: Path,
    *,
    display_base: Path | None = None,
    pin_majors: dict[tuple[str, str], int] | None = None,
) -> tuple[int, list[str]]:
    failures: list[str] = []
    scanned = 0
    label_base = display_base or workflows

    majors = node24_pin_majors() if pin_majors is None else pin_majors
    for workflow in workflow_files(workflows):
        text = workflow.read_text(encoding="utf-8")
        for match in USES_RE.finditer(text):
            action = match.group(1)
            commit = match.group(2)
            major = majors.get((action, commit))
            minimum = MINIMUM_NODE24_ACTION_MAJORS.get(action)
            if minimum is None:
                continue
            scanned += 1
            if major is None:
                failures.append(
                    f"{workflow_label(workflow, label_base)} pins unreviewed {action}@{commit}",
                )
            elif major < minimum:
                failures.append(
                    f"{workflow_label(workflow, label_base)} pins {action}@{commit} "
                    f"(reviewed v{major}); expected v{minimum}+",
                )

    return scanned, failures


class GithubActionsNode24ReadinessTests(unittest.TestCase):
    def test_yaml_extension_workflows_are_scanned(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            workflows = Path(temp_dir)
            (workflows / "node24.yaml").write_text(
                "name: yaml workflow\njobs:\n  test:\n    steps:\n"
                f"      - uses: actions/checkout@{'5' * 40}\n",
                encoding="utf-8",
            )

            scanned, failures = scan_action_pins(
                workflows, pin_majors={("actions/checkout", "5" * 40): 5}
            )

        self.assertEqual(1, scanned)
        self.assertEqual(
            [
                f"node24.yaml pins actions/checkout@{'5' * 40} "
                "(reviewed v5); expected v6+"
            ],
            failures,
        )

    def test_actions_use_node24_capable_majors(self) -> None:
        scanned, failures = scan_action_pins(WORKFLOWS, display_base=ROOT)

        self.assertGreater(scanned, 0, "expected to scan at least one Node 24-gated workflow action")
        self.assertEqual([], failures)

    def test_no_workflow_pins_explicit_node20_action_variants(self) -> None:
        offenders = []
        for workflow in workflow_files(WORKFLOWS):
            text = workflow.read_text(encoding="utf-8")
            if "node20" in text.lower():
                offenders.append(str(workflow.relative_to(ROOT)))

        self.assertEqual([], offenders)


if __name__ == "__main__":
    unittest.main()
