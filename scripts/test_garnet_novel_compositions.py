#!/usr/bin/env python3
"""Regression tests for smoke_garnet_novel_compositions.py."""
from __future__ import annotations

import importlib.util
import os
import subprocess
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("smoke_garnet_novel_compositions.py")
ROOT = SCRIPT.resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("smoke_garnet_novel_compositions", SCRIPT)
assert SPEC is not None
mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["smoke_garnet_novel_compositions"] = mod
SPEC.loader.exec_module(mod)


def _garnet_binary() -> Path | None:
    exe = "garnet.exe" if os.name == "nt" else "garnet"
    for profile in ("release", "debug"):
        candidate = ROOT / "target" / profile / exe
        if candidate.exists():
            return candidate
    return None


class CaseInventoryTests(unittest.TestCase):
    def test_cases_are_well_formed(self) -> None:
        self.assertGreaterEqual(len(mod.NOVEL_CASES), 3)
        for case in mod.NOVEL_CASES:
            self.assertTrue((ROOT / case.file).exists(), f"missing program: {case.file}")
            self.assertGreaterEqual(
                len(case.contributions), 3, f"{case.id} should fuse >= 3 contributions"
            )
            self.assertTrue(case.run_expect, f"{case.id} needs an expected run line")

    def test_each_case_fuses_multiple_distinct_contributions(self) -> None:
        for case in mod.NOVEL_CASES:
            self.assertEqual(
                len(set(case.contributions)),
                len(case.contributions),
                f"{case.id} lists a duplicate contribution",
            )


class EvaluateCaseTests(unittest.TestCase):
    def setUp(self) -> None:
        self.case = mod.NOVEL_CASES[0]

    def test_pass_when_clean_and_expected_present(self) -> None:
        r = mod.evaluate_case(
            self.case, 0, "11 functions checked, 0 diagnostics", 0, f"x\n{self.case.run_expect}\n=> 16"
        )
        self.assertTrue(r.ok)
        self.assertTrue(r.check_ok)
        self.assertTrue(r.run_ok)

    def test_check_fails_on_nonzero_exit(self) -> None:
        r = mod.evaluate_case(self.case, 1, "0 diagnostics", 0, self.case.run_expect)
        self.assertFalse(r.check_ok)
        self.assertFalse(r.ok)

    def test_check_fails_when_diagnostics_present(self) -> None:
        # No "0 diagnostics" marker -> the checker reported something.
        r = mod.evaluate_case(self.case, 0, "3 diagnostics", 0, self.case.run_expect)
        self.assertFalse(r.check_ok)

    def test_run_fails_on_wrong_output(self) -> None:
        r = mod.evaluate_case(self.case, 0, "0 diagnostics", 0, "governance: 999")
        self.assertFalse(r.run_ok)
        self.assertFalse(r.ok)

    def test_run_fails_on_nonzero_exit(self) -> None:
        r = mod.evaluate_case(self.case, 0, "0 diagnostics", 1, self.case.run_expect)
        self.assertFalse(r.run_ok)


@unittest.skipUnless(_garnet_binary() is not None, "garnet CLI not built")
class LiveMatrixTests(unittest.TestCase):
    def test_all_novel_programs_check_and_run(self) -> None:
        proc = subprocess.run(
            [sys.executable, str(SCRIPT), "--format", "json"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)


if __name__ == "__main__":
    unittest.main()
