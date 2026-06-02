#!/usr/bin/env python3
"""Regression tests for the agent-acceptance loop status gate (S102)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_agent_loop_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_agent_loop_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
al = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_agent_loop_status"] = al
SPEC.loader.exec_module(al)


class AgentLoopStatusTests(unittest.TestCase):
    def test_three_stage_gate(self) -> None:
        self.assertTrue(al.read_status().three_stage_gate)

    def test_rule2_widening_refused(self) -> None:
        self.assertTrue(al.read_status().rule2_widening_refused)

    def test_rule3_provenance_recorded(self) -> None:
        self.assertTrue(al.read_status().rule3_provenance_recorded)

    def test_honesty_anchor_present(self) -> None:
        self.assertTrue(al.read_status().honesty_anchor_present)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(al.main(["--gate", "--format", "json"]), 0)

    def test_markdown_states_capability_depth_only(self) -> None:
        md = al.render_markdown(al.read_status())
        self.assertIn("capability+depth evidence", md)
        self.assertIn("declared-not-enforced", md)
        self.assertIn("simulated", md)


if __name__ == "__main__":
    unittest.main()
