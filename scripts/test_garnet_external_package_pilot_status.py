#!/usr/bin/env python3
"""Regression tests for the external package pilot status reporter (S77)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_external_package_pilot_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_external_package_pilot_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
ep = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_external_package_pilot_status"] = ep
SPEC.loader.exec_module(ep)


class PilotStatusTests(unittest.TestCase):
    def test_pilot_test_present_and_complete(self) -> None:
        r = ep.read_status()
        self.assertTrue(r.pilot_test_present, f"missing markers: {r.missing_markers}")
        self.assertEqual(r.missing_markers, [])

    def test_registry_infra_and_slopguard_grounded(self) -> None:
        r = ep.read_status()
        self.assertTrue(r.registry_infra_present)
        self.assertTrue(r.slopguard_present)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(ep.main(["--gate", "--format", "json"]), 0)

    def test_doc_present(self) -> None:
        self.assertTrue(ep.read_status().doc_present)

    def test_markdown_states_local_stub_not_live_ecosystem(self) -> None:
        md = ep.render_markdown(ep.read_status())
        self.assertIn("NOT a live public ecosystem", md)
        self.assertIn("not a", md.lower())


if __name__ == "__main__":
    unittest.main()
