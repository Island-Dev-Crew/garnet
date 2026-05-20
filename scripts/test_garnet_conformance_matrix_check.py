#!/usr/bin/env python3
"""Regression tests for garnet_conformance_matrix_check.py."""
from __future__ import annotations

import importlib.util
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_conformance_matrix_check.py")
SPEC = importlib.util.spec_from_file_location("garnet_conformance_matrix_check", SCRIPT)
assert SPEC is not None
mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_conformance_matrix_check"] = mod
SPEC.loader.exec_module(mod)


_HEADER = """\
# Garnet Mini-Spec Conformance Matrix

## Status legend

| Symbol | Meaning |
|---|---|
| ✅ | Implemented. |

## Rows

| Mini-Spec | Status | Evidence | Notes |
|---|---|---|---|
"""


def _write_matrix(rows: list[str], tmp_root: Path) -> Path:
    md = _HEADER + "\n".join(rows) + "\n"
    p = tmp_root / "matrix.md"
    p.write_text(md, encoding="utf-8")
    return p


class GarnetConformanceMatrixCheckTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)

    def test_empty_matrix_is_clean(self) -> None:
        matrix = _write_matrix([], self.root)
        result = mod.check_matrix(matrix)
        self.assertEqual(0, len(result.findings))
        self.assertEqual(0, result.rows_with_paths)

    def test_row_with_resolvable_path_passes(self) -> None:
        # Create a file inside the repo root that the matrix references.
        # We point at the script itself as the universally-resolvable target.
        repo_relative = "scripts/garnet_conformance_matrix_check.py"
        rows = [
            f"| §1.1 Lex | ✅ | `{repo_relative}` | sanity check |",
        ]
        matrix = _write_matrix(rows, self.root)
        result = mod.check_matrix(matrix)
        self.assertEqual(
            0, len(result.findings), f"unexpected findings: {result.findings}"
        )

    def test_row_with_unresolvable_path_is_flagged(self) -> None:
        rows = [
            "| §2.1 Bogus | ✅ | `garnet-imaginary/does/not/exist.rs` | should fail |",
        ]
        matrix = _write_matrix(rows, self.root)
        result = mod.check_matrix(matrix)
        self.assertEqual(1, len(result.findings))
        self.assertEqual(
            "garnet-imaginary/does/not/exist.rs", result.findings[0].missing_path
        )

    def test_grammar_reference_is_not_classified_as_path(self) -> None:
        # `do...end` and `KwTry/KwRescue` look path-ish but are prose. The
        # check must not flag them.
        rows = [
            "| §5.4 Blocks | ✅ | `do...end` | grammar reference |",
            "| §7.2 Try | ✅ | `KwTry/KwRescue/KwEnsure/KwRaise` | enum names |",
        ]
        matrix = _write_matrix(rows, self.root)
        result = mod.check_matrix(matrix)
        self.assertEqual(0, len(result.findings))

    def test_default_exit_code_is_advisory(self) -> None:
        # Build a matrix with one guaranteed-missing path and run the CLI.
        rows = [
            "| §99 Missing | ✅ | `garnet-imaginary/x.rs` | should appear in findings |",
        ]
        matrix = _write_matrix(rows, self.root)
        cp = subprocess.run(
            [sys.executable, str(SCRIPT), "--matrix", str(matrix)],
            capture_output=True,
            text=True,
        )
        self.assertEqual(0, cp.returncode, cp.stderr)
        self.assertIn("Unresolved paths: 1", cp.stdout)

    def test_strict_exit_code_is_one_on_findings(self) -> None:
        rows = [
            "| §99 Missing | ✅ | `garnet-imaginary/x.rs` | should appear in findings |",
        ]
        matrix = _write_matrix(rows, self.root)
        cp = subprocess.run(
            [sys.executable, str(SCRIPT), "--matrix", str(matrix), "--strict"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(1, cp.returncode)

    def test_strict_exit_code_is_zero_when_clean(self) -> None:
        matrix = _write_matrix([], self.root)
        cp = subprocess.run(
            [sys.executable, str(SCRIPT), "--matrix", str(matrix), "--strict"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(0, cp.returncode, cp.stderr)


if __name__ == "__main__":
    unittest.main()
