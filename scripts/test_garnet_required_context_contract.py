#!/usr/bin/env python3
"""Schema tests for the declarative producer inventory."""
from __future__ import annotations

import importlib.util
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path


PATH = Path(__file__).with_name("garnet_required_context_contract.py")
SPEC = importlib.util.spec_from_file_location("_producer_contract_test", PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot load {PATH}")
contract = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = contract
SPEC.loader.exec_module(contract)


class ContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.path = Path(self.temp.name) / "inventory.json"
        self.value = {
            "schema": contract.SCHEMA,
            "target_branch": "main",
            "optional_contexts": ["Future"],
            "producers": [
                {
                    "context": "Static",
                    "workflow": ".github/workflows/checks.yml",
                    "event": "pull_request",
                    "job": "static",
                },
                {
                    "context": "Future",
                    "workflow": ".github/workflows/future.yaml",
                    "event": "pull_request_target",
                    "job": "future",
                    "matrix": {"os": "ubuntu-latest"},
                },
            ],
        }

    def tearDown(self) -> None:
        self.temp.cleanup()

    def load(self):
        self.path.write_text(json.dumps(self.value), encoding="utf-8")
        return contract.load_inventory(self.path)

    def assertProblem(self, fragment: str) -> None:  # noqa: N802
        problems = self.load().problems
        self.assertTrue(any(fragment in item for item in problems), problems)

    def test_exact_inventory_loads(self) -> None:
        inventory = self.load()
        self.assertEqual(inventory.problems, [])
        self.assertEqual(inventory.optional_contexts, {"Future"})
        self.assertEqual(inventory.producers[1].matrix, ("os", "ubuntu-latest"))

    def test_duplicate_json_and_contexts_fail_closed(self) -> None:
        self.path.write_text(
            '{"schema":"a","schema":"b"}', encoding="utf-8"
        )
        self.assertIn("duplicate JSON key", contract.load_inventory(self.path).problems[0])
        self.value["producers"].append(dict(self.value["producers"][0]))
        self.assertProblem("duplicate inventory context")

    def test_identity_and_matrix_shapes_are_exact(self) -> None:
        mutations = (
            ("workflow", ".github/workflows/Checks.yml", "lowercase YAML"),
            ("workflow", ".github/workflows/checks.YML", "lowercase YAML"),
            ("workflow", ".github/workflows/../evil.yml", "lowercase YAML"),
            ("event", "push", "approved pull-request event"),
            ("job", "bad job", "job id is not canonical"),
        )
        for key, value, message in mutations:
            with self.subTest(key=key):
                original = self.value["producers"][0][key]
                self.value["producers"][0][key] = value
                self.assertProblem(message)
                self.value["producers"][0][key] = original
        self.value["producers"][1]["matrix"] = {"os": "linux", "arch": "x64"}
        self.assertProblem("exactly one binding")
        self.value["producers"][1]["matrix"] = ["linux"]
        self.assertProblem("exactly one binding")

    def test_branch_schema_and_keys_are_exact(self) -> None:
        self.value["target_branch"] = "main\nattacker"
        self.assertProblem("must be 'main'")
        self.value["target_branch"] = "main"
        self.value["schema"] = "garnet.required-context-producers/v0"
        self.assertProblem("schema must be")
        self.value["schema"] = contract.SCHEMA
        self.value["unknown"] = True
        self.assertProblem("top-level keys are not exact")
        del self.value["unknown"]
        self.value["producers"][0]["unknown"] = True
        self.assertProblem("keys are not exact")

    def test_context_and_matrix_member_are_static_canonical_text(self) -> None:
        for context in (
            " Static",
            "Static\nInjected",
            "Sta\u200btic",
            "Cafe\u0301",
            "\ud800",
            "Static\u2028Check",
            "Static\u00a0Check",
        ):
            with self.subTest(context=repr(context)):
                self.value["producers"][0]["context"] = context
                self.assertProblem("context is not canonical text")
        self.value["producers"][0]["context"] = "Static"
        for member in (" ", "linux x64", "${{ github.token }}"):
            with self.subTest(member=member):
                self.value["producers"][1]["matrix"] = {"os": member}
                self.assertProblem("matrix binding is invalid")

    def test_malformed_missing_large_and_nested_duplicate_files_fail(self) -> None:
        self.path.write_text("{", encoding="utf-8")
        self.assertIn("cannot read", contract.load_inventory(self.path).problems[0])
        self.path.write_text('{"outer":{"key":1,"key":2}}', encoding="utf-8")
        self.assertIn("duplicate JSON key", contract.load_inventory(self.path).problems[0])
        self.path.unlink()
        self.assertIn("cannot inspect", contract.load_inventory(self.path).problems[0])
        self.path.write_bytes(b"x" * (contract.MAX_INVENTORY_BYTES + 1))
        self.assertIn("size limit", contract.load_inventory(self.path).problems[0])

    def test_directory_and_symlink_inventory_fail(self) -> None:
        directory = Path(self.temp.name) / "directory"
        directory.mkdir()
        self.assertIn("not a regular file", contract.load_inventory(directory).problems[0])
        target = Path(self.temp.name) / "target.json"
        target.write_text(json.dumps(self.value), encoding="utf-8")
        link = Path(self.temp.name) / "link.json"
        try:
            os.symlink(target, link)
        except OSError as exc:
            self.skipTest(f"symlink creation unavailable: {exc}")
        self.assertIn("symlink/reparse", contract.load_inventory(link).problems[0])

    def test_optional_context_must_name_a_producer(self) -> None:
        self.value["optional_contexts"] = ["Unknown"]
        self.assertProblem("absent from producers")


if __name__ == "__main__":
    unittest.main()
