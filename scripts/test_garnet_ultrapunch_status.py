#!/usr/bin/env python3
"""Regression tests for the ultrapunch evidence-record gate (S104)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_ultrapunch_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_ultrapunch_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
up = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_ultrapunch_status"] = up
SPEC.loader.exec_module(up)


class UltrapunchStatusTests(unittest.TestCase):
    def test_record_and_artifacts(self) -> None:
        r = up.read_status()
        self.assertTrue(r.record_present)
        self.assertTrue(r.four_artifacts_named)

    def test_two_level_symmetry_explicit(self) -> None:
        self.assertTrue(up.read_status().two_level_symmetry_explicit)

    def test_refusal_and_honesty(self) -> None:
        r = up.read_status()
        self.assertTrue(r.refusal_documented)
        self.assertTrue(r.honesty_anchor_present)

    def test_reproduce_script_and_demo_pinned(self) -> None:
        r = up.read_status()
        self.assertTrue(r.reproduce_script_present)
        self.assertTrue(r.demo_pinned)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(up.main(["--gate", "--format", "json"]), 0)

    def test_markdown_states_capability_depth_only(self) -> None:
        md = up.render_markdown(up.read_status())
        self.assertIn("capability + depth evidence", md)
        self.assertIn("declared-not-enforced", md)


if __name__ == "__main__":
    unittest.main()
