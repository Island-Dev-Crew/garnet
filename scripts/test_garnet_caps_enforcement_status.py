#!/usr/bin/env python3
"""Regression tests for the @caps enforcement status gate (S90)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_caps_enforcement_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_caps_enforcement_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
ce = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_caps_enforcement_status"] = ce
SPEC.loader.exec_module(ce)


class CapsEnforcementTests(unittest.TestCase):
    def test_interpreter_has_require_capability(self) -> None:
        self.assertTrue(ce.read_status().interp_has_require_capability)

    def test_env_proc_fs_bridges_gated(self) -> None:
        r = ce.read_status()
        self.assertEqual(r.missing_gates, [], f"missing: {r.missing_gates}")
        self.assertEqual(sorted(r.bridges_gated), ["env", "fs", "net", "proc"])

    def test_program_entry_frame_present(self) -> None:
        self.assertTrue(ce.read_status().program_entry_frame_present)

    def test_vm_entry_caps_frame_present(self) -> None:
        # S100: the VM installs the same program-entry caps frame (no --vm laundering).
        self.assertTrue(ce.read_status().vm_entry_caps_frame_present)

    def test_enforcement_tests_present(self) -> None:
        self.assertTrue(ce.read_status().enforcement_tests_present)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(ce.main(["--gate", "--format", "json"]), 0)

    def test_markdown_states_host_authority_only(self) -> None:
        md = ce.render_markdown(ce.read_status())
        self.assertIn("Host-authority surfaces only", md)
        self.assertIn("pure computation", md)
        # S100 retired the "VM does not yet enforce" claim: both backends now enforce.
        self.assertNotIn("does not yet enforce", md)
        self.assertIn("the VM (S100) enforce", md)


if __name__ == "__main__":
    unittest.main()
