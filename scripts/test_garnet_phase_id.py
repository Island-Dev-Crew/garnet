#!/usr/bin/env python3
"""Regression tests for the Garnet phase-id collision-prevention helper."""
from __future__ import annotations

import importlib.util
import subprocess
import sys
from pathlib import Path
import unittest

SCRIPT = Path(__file__).with_name("garnet_phase_id.py")
SPEC = importlib.util.spec_from_file_location("garnet_phase_id", SCRIPT)
assert SPEC is not None
phase_mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_phase_id"] = phase_mod
SPEC.loader.exec_module(phase_mod)


class NextPhaseIdTests(unittest.TestCase):
    def test_global_max_successor(self) -> None:
        self.assertEqual(
            phase_mod.next_phase_id(["4BH", "4BI", "6BS"]), "6BT"
        )

    def test_normalizes_case_and_whitespace(self) -> None:
        self.assertEqual(
            phase_mod.next_phase_id([" 6bs ", "4bh", "Phase 6BR"]), "6BT"
        )

    def test_single_letter_and_rollover(self) -> None:
        self.assertEqual(phase_mod.next_phase_id(["6Z"]), "6AA")
        self.assertEqual(phase_mod.next_phase_id(["6BZ"]), "6CA")
        self.assertEqual(phase_mod.next_phase_id(["4Z", "6ZZ"]), "6AAA")

    def test_empty_defaults_to_1a(self) -> None:
        self.assertEqual(phase_mod.next_phase_id([]), "1A")

    def test_is_used_normalizes(self) -> None:
        used = phase_mod.normalize_ids(["4BI", "6BS"])
        self.assertIn("6BS", used)
        self.assertIn("4BI", used)

    def test_cli_emits_single_token(self) -> None:
        proc = subprocess.run(
            [sys.executable, str(SCRIPT)],
            capture_output=True,
            text=True,
            cwd=str(SCRIPT.resolve().parents[1]),
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        out = proc.stdout
        self.assertTrue(out.endswith("\n"), repr(out))
        token = out.strip()
        self.assertRegex(token, r"^[0-9]+[A-Z]+$")
        self.assertEqual(len(out.splitlines()), 1)

    def test_cli_check_rejects_used_id(self) -> None:
        proc = subprocess.run(
            [sys.executable, str(SCRIPT), "--check", "6BS"],
            capture_output=True,
            text=True,
            cwd=str(SCRIPT.resolve().parents[1]),
        )
        # 6BS is a real merged phase (PR #162); --check must reject it.
        self.assertNotEqual(proc.returncode, 0)

    def test_cli_check_accepts_unused_id(self) -> None:
        proc = subprocess.run(
            [sys.executable, str(SCRIPT), "--check", "9ZZZ"],
            capture_output=True,
            text=True,
            cwd=str(SCRIPT.resolve().parents[1]),
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)


if __name__ == "__main__":
    unittest.main()
