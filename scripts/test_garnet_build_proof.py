#!/usr/bin/env python3
"""Regression tests for the Windows/Linux/macOS build-proof reporter (S47)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_build_proof.py")
SPEC = importlib.util.spec_from_file_location("garnet_build_proof", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
build_proof = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_build_proof"] = build_proof
SPEC.loader.exec_module(build_proof)


class MatrixParsingTests(unittest.TestCase):
    def test_parses_os_array(self) -> None:
        yaml = "    matrix:\n      os: [ubuntu-latest, windows-latest, macos-latest]\n"
        self.assertEqual(
            build_proof._matrix_oses(yaml),
            {"ubuntu-latest", "windows-latest", "macos-latest"},
        )

    def test_quoted_and_multiple_arrays_union(self) -> None:
        yaml = 'os: ["ubuntu-latest"]\nos: [windows-latest, macos-latest]\n'
        self.assertEqual(
            build_proof._matrix_oses(yaml),
            {"ubuntu-latest", "windows-latest", "macos-latest"},
        )

    def test_empty_when_no_matrix(self) -> None:
        self.assertEqual(build_proof._matrix_oses("runs-on: ubuntu-latest\n"), set())


class BuildProofTests(unittest.TestCase):
    def test_real_ci_covers_all_three_oses(self) -> None:
        proof = build_proof.read_build_proof()
        # The repo's ci.yml test job runs cargo test --workspace on all three.
        self.assertTrue(proof.all_behaves, "ci.yml must test all three OSes")
        labels = {p.label for p in proof.oses if p.behaves}
        self.assertEqual(labels, {"Linux", "Windows", "macOS"})

    def test_schema_and_command_are_stable(self) -> None:
        proof = build_proof.read_build_proof()
        self.assertEqual(proof.schema, "garnet.build_proof/v1")
        self.assertEqual(proof.workspace_test_command, "cargo test --workspace")

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(build_proof.main(["--gate", "--format", "json"]), 0)

    def test_markdown_renders_table(self) -> None:
        md = build_proof.render_markdown(build_proof.read_build_proof())
        self.assertIn("| OS | behaves", md)
        self.assertIn("All target OSes behave: yes", md)


if __name__ == "__main__":
    unittest.main()
