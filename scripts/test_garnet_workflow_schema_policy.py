#!/usr/bin/env python3
"""Adversarial tests for the fail-closed workflow projection."""
from __future__ import annotations
import importlib.util, sys, unittest
from dataclasses import FrozenInstanceError
from pathlib import Path
from unittest.mock import patch
PATH = Path(__file__).with_name("garnet_workflow_schema_policy.py")
SPEC = importlib.util.spec_from_file_location("_workflow_schema_policy_test", PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot load {PATH}")
policy = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = policy
SPEC.loader.exec_module(policy)
WORKFLOW = """name: Checks
on:
  pull_request:
    branches: [main]
  workflow_dispatch: {}
jobs:
  static:
    name: Static check
    runs-on: ubuntu-latest
    steps:
      - run: echo static
  matrix:
    name: Matrix (${{ matrix.os }})
    needs: static
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest]
    steps:
      - uses: actions/checkout@v6
"""
def snapshot(*contents: str) -> object:
    documents = tuple(policy.yaml_policy.WorkflowDocument(
        f".github/workflows/fixture-{index}.yml", "100644", f"{index + 1:040x}",
        policy.yaml_policy._document(content.encode()),
    ) for index, content in enumerate(contents))
    return policy.yaml_policy.WorkflowYamlSnapshot(documents, ())
class WorkflowSchemaPolicyTests(unittest.TestCase):
    def test_current_index_projects_33_ordered_contexts(self) -> None:
        result = policy.workflow_projection(PATH.parents[1])
        contexts = [item.context for workflow in result.workflows for item in workflow.contexts]
        self.assertEqual((result.problems, len(result.workflows), len(contexts)), ((), 11, 33))
        self.assertIn("cargo test (windows-latest)", contexts)
        self.assertIn("Publish VSIX release assets", contexts)
        with self.assertRaises(FrozenInstanceError):
            result.workflows[0].events = ()
    def test_projected_sources_filters_steps_and_scalar_style_survive(self) -> None:
        conditional = WORKFLOW.replace("- run: echo static", "- if: always()\n        run: echo static")
        workflow = policy.project_snapshot(snapshot(conditional)).workflows[0]
        pull, matrix = workflow.events[0], workflow.jobs[1]
        self.assertIs(pull.source, dict(dict(workflow.source.root.items)["on"].items)["pull_request"])
        self.assertIs(matrix.source, dict(dict(workflow.source.root.items)["jobs"].items)["matrix"])
        self.assertEqual(dict(workflow.jobs[0].steps[0].items)["if"].value, "always()")
        self.assertEqual((pull.filters[0][0], pull.filters[0][1][0].style), ("branches", None))
        self.assertEqual(tuple(item.value for item in matrix.needs), ("static",))
        self.assertEqual([item.context for item in workflow.contexts],
                         ["Static check", "Matrix (ubuntu-latest)", "Matrix (macos-latest)"])
        self.assertEqual(workflow.contexts[-1].binding[1].value, "macos-latest")
        self.assertEqual(policy.project_snapshot(snapshot(WORKFLOW.replace("name: Checks", 'name: "false"'))).problems, ())
    def test_projection_never_reopens_document_paths(self) -> None:
        frozen = snapshot(WORKFLOW)
        with patch("builtins.open", side_effect=AssertionError("reopened path")):
            self.assertEqual(policy.project_snapshot(frozen).problems, ())
    def test_schema_security_and_scalar_tricks_fail_all_or_zero(self) -> None:
        r = WORKFLOW.replace
        variants = (
            r("name: Checks", "name: false"), r("  pull_request:\n    branches: [main]", "  pull_request: main"),
            r("jobs:", "bogus: true\njobs:", 1), r("jobs:", "permissions: write-all\njobs:", 1),
            r("jobs:", "permissions:\n  id-token: read\njobs:", 1), r("jobs:", "permissions:\n  models: write\njobs:", 1),
            r("    runs-on: ubuntu-latest", "    permissions:\n      checks: write\n    runs-on: ubuntu-latest", 1),
            r("    runs-on: ubuntu-latest", "    continue-on-error: true\n    runs-on: ubuntu-latest", 1),
            r("        os: [ubuntu-latest, macos-latest]", "        os: [ubuntu-latest]\n        arch: [x64]"),
            r("Matrix (${{ matrix.os }})", "Matrix (${{ github.ref }})"),
            r("name: Static check", "name: Static ${{ github.ref"), r("name: Static check", "name: " + "x" * 257),
            r("    needs: static", "    needs: missing"),
            r("    runs-on: ubuntu-latest", "    needs: matrix\n    runs-on: ubuntu-latest", 1),
            r("    steps:\n      - run: echo static", "    steps: echo static"),
            r("      - run: echo static", "      - name: inert"), r("      - run: echo static", "      - run: echo static\n        uses: actions/checkout@v6"), r("      - run: echo static", "      - run: echo static\n        with:\n          x: y"),
            r("      - run: echo static", "      - bogus: true"), r("      - run: echo static", "      - run:\n          bad: true"), r("      - uses: actions/checkout@v6", "      - uses: nonsense"),
            r("      - run: echo static", "      - run: echo static\n        continue-on-error: true"), r("      - uses: actions/checkout@v6", "      - uses: actions/checkout@v6\n        shell: bash"), r("echo static", "echo ${{ github.ref"), r("echo static", "echo }} ${{ github.ref }}"),
            r("jobs:", "env: [bad]\njobs:", 1), r("jobs:", "concurrency:\n  bogus: value\njobs:", 1),
            r("    runs-on: ubuntu-latest", "    env: [bad]\n    runs-on: ubuntu-latest", 1),
            r("    runs-on: ubuntu-latest", "    timeout-minutes:\n      bad: true\n    runs-on: ubuntu-latest", 1),
            r("  workflow_dispatch: {}", "  workflow_dispatch:\n    inputs:\n      mode:\n        required: nope"), r("  workflow_dispatch: {}", "  workflow_dispatch:\n    inputs:\n      mode:\n        type: choice"), r("  workflow_dispatch: {}", "  schedule:\n    - cron: never"),
        )
        for variant in variants:
            with self.subTest(variant=variant[:60]):
                result = policy.project_snapshot(snapshot(WORKFLOW, variant))
                self.assertTrue(result.problems)
                self.assertEqual(result.workflows, ())
    def test_duplicate_context_occurrences_are_not_deduplicated(self) -> None:
        duplicate = WORKFLOW.replace("  matrix:\n", "  duplicate:\n    name: Static check\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo duplicate\n  matrix:\n", 1)
        contexts = policy.project_snapshot(snapshot(duplicate)).workflows[0].contexts
        self.assertEqual([item.context for item in contexts].count("Static check"), 2)
if __name__ == "__main__":
    unittest.main()
