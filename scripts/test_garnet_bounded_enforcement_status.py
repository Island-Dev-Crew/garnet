#!/usr/bin/env python3
"""Regression tests for the @max_depth enforcement status gate (S89)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_bounded_enforcement_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_bounded_enforcement_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
be = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_bounded_enforcement_status"] = be
SPEC.loader.exec_module(be)


class BoundedEnforcementTests(unittest.TestCase):
    def test_interpreter_reads_max_depth(self) -> None:
        self.assertTrue(be.read_status().interp_reads_max_depth)

    def test_interpreter_traps_on_exceed(self) -> None:
        self.assertTrue(be.read_status().interp_traps_on_exceed)

    def test_vm_traps_on_exceed(self) -> None:
        # S99: the VM enforces the same @max_depth ceiling with the same message.
        self.assertTrue(be.read_status().vm_traps_on_exceed)

    def test_enforcement_tests_present(self) -> None:
        self.assertTrue(be.read_status().enforcement_tests_present)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(be.main(["--gate", "--format", "json"]), 0)

    def test_markdown_states_one_enforced_ceiling(self) -> None:
        md = be.render_markdown(be.read_status())
        self.assertIn("ONE enforced ceiling", md)
        self.assertIn("declared-not-enforced", md)


if __name__ == "__main__":
    unittest.main()
