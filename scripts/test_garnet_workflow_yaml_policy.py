#!/usr/bin/env python3
"""Adversarial tests for the immutable typed workflow YAML boundary."""
from __future__ import annotations

import importlib.util, subprocess, sys, tempfile, unittest
from pathlib import Path


PATH = Path(__file__).with_name("garnet_workflow_yaml_policy.py")
SPEC = importlib.util.spec_from_file_location("_workflow_yaml_policy_test", PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot load {PATH}")
policy = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = policy
SPEC.loader.exec_module(policy)

WORKFLOW = """name: Checks
on:
  pull_request: {}
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - run: |
          printf '\"name\": &anchor !tag --- {x: y}'
plain-null: null
quoted-null: "null"
"""


class WorkflowYamlPolicyTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / ".github/workflows").mkdir(parents=True)
        self.git("init", "-q")
        self.git("config", "user.email", "fixture@example.invalid")
        self.git("config", "user.name", "Fixture")
        self.git("config", "core.autocrlf", "false")
        self.stage(WORKFLOW)

    def git(self, *args: str) -> str:
        return subprocess.run(["git", *args], cwd=self.root, check=True,
                              capture_output=True, text=True).stdout.strip()

    def stage(self, content: str | bytes | None, name: str = "checks.yml") -> None:
        path = self.root / ".github/workflows" / name
        if content is None:
            path.unlink(missing_ok=True)
        else:
            path.write_bytes(content if isinstance(content, bytes) else content.encode())
        self.git("add", "-A")

    def test_real_index_and_scalar_style_are_preserved(self) -> None:
        self.assertEqual(sys.flags.isolated, 1)
        self.assertEqual(policy.YAML_ORIGIN.name, "__init__.py")
        live = policy.workflow_documents(PATH.parents[1])
        self.assertEqual((live.problems, len(live.documents)), ((), 11))
        result = policy.workflow_documents(self.root)
        self.assertEqual(result.problems, ())
        document = result.documents[0]
        values = dict(document.root.items)
        self.assertEqual(values["plain-null"], policy.WorkflowScalar("null", None))
        self.assertEqual(values["quoted-null"], policy.WorkflowScalar("null", '"'))
        self.assertRegex(document.object_id, r"^[0-9a-f]{40,64}$")

    def test_exact_commit_ignores_changed_index(self) -> None:
        self.git("commit", "-qm", "fixture")
        commit = self.git("rev-parse", "HEAD")
        self.stage("name: [broken\n")
        result = policy.workflow_documents(self.root, treeish=commit)
        self.assertEqual((result.problems, len(result.documents)), ((), 1))

    def test_ambiguous_or_noncanonical_yaml_fails_all_or_zero(self) -> None:
        variants: list[str | bytes | None] = [
            None, b"\xff", b"\xef\xbb\xbf" + WORKFLOW.encode(), WORKFLOW.replace("\n", "\r\n"),
            WORKFLOW.replace("  check:", "\tcheck:"), "name: [broken\n",
            WORKFLOW.replace("name: Checks", "name: One\nname: Two"),
            WORKFLOW.replace("name: Checks", '"name": Checks'),
            WORKFLOW.replace("name: Checks", "name: &label Checks"),
            "base: &base\n  name: Checks\ncopy:\n  <<: *base\n",
            WORKFLOW.replace("name: Checks", "name: !evil Checks"),
            "---\n" + WORKFLOW, WORKFLOW + "---\nname: second\n",
            WORKFLOW.replace("pull_request: {}", "pull_request: {types: [closed]}"),
            "name: Checks\n" + (" " * 130) + "deep: value\n",
            "name: " + ("x" * 4_097) + "\n",
            "name: Checks\nflow: " + ("[" * 513) + "\n",
        ]
        for variant in variants:
            with self.subTest(variant=repr(variant)[:60]):
                self.stage(variant)
                result = policy.workflow_documents(self.root)
                self.assertTrue(result.problems)
                self.assertEqual(result.documents, ())

    def test_one_invalid_file_erases_valid_ast_output(self) -> None:
        self.stage(b"name: [broken\n", "bad.yml")
        result = policy.workflow_documents(self.root)
        self.assertTrue(result.problems)
        self.assertEqual(result.documents, ())


if __name__ == "__main__":
    unittest.main()
