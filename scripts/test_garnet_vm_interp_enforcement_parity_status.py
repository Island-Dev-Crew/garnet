#!/usr/bin/env python3
"""Regression tests for the VM/interp enforcement-parity gate (S101)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_vm_interp_enforcement_parity_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_vm_interp_enforcement_parity_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
ep = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_vm_interp_enforcement_parity_status"] = ep
SPEC.loader.exec_module(ep)


class EnforcementParityTests(unittest.TestCase):
    def test_max_depth_trap_parity(self) -> None:
        # S99: the VM traps on @max_depth identically to the interpreter.
        self.assertTrue(ep.read_status().max_depth_trap_parity)

    def test_caps_trap_parity(self) -> None:
        # S100: the VM traps on @caps (incl. the S92 entry gate) identically.
        self.assertTrue(ep.read_status().caps_trap_parity)

    def test_both_ceilings_enforced(self) -> None:
        self.assertEqual(ep.read_status().enforced_ceilings, ["@max_depth", "@caps"])

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(ep.main(["--gate", "--format", "json"]), 0)

    def test_markdown_names_deferred_and_seam_closed(self) -> None:
        md = ep.render_markdown(ep.read_status())
        self.assertIn("CLOSED", md)
        self.assertIn("@bounded (Wasmtime fuel)", md)
        self.assertIn("not a proof of total backend equivalence", md)


if __name__ == "__main__":
    unittest.main()
