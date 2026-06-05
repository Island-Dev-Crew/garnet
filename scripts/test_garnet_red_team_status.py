#!/usr/bin/env python3
"""Regression tests for the kernel red-team gate (S114)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_red_team_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_red_team_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
rt = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_red_team_status"] = rt
SPEC.loader.exec_module(rt)


class RedTeamTests(unittest.TestCase):
    def test_high_hole_fixed_with_regressions(self) -> None:
        r = rt.read_status()
        self.assertTrue(r.high_hole_fixed)
        self.assertTrue(r.regression_tests_present)

    def test_low_holes_recorded(self) -> None:
        self.assertTrue(rt.read_status().low_holes_recorded)

    def test_held_and_deferred_recorded(self) -> None:
        self.assertTrue(rt.read_status().held_and_deferred_recorded)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(rt.main(["--gate", "--format", "json"]), 0)

    def test_markdown_states_one_high_two_low(self) -> None:
        md = rt.render_markdown(rt.read_status())
        self.assertIn("One HIGH enforced-ceiling hole found + fixed", md)
        self.assertIn("Named-deferred ceilings unchanged", md)


if __name__ == "__main__":
    unittest.main()
