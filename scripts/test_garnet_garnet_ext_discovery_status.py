#!/usr/bin/env python3
"""Regression tests for the .GARNET discovery status gate (S81)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_garnet_ext_discovery_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_garnet_ext_discovery_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
gd = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_garnet_ext_discovery_status"] = gd
SPEC.loader.exec_module(gd)


class DiscoveryTests(unittest.TestCase):
    def test_collector_is_case_insensitive(self) -> None:
        self.assertTrue(gd.read_status().collector_case_insensitive)

    def test_old_case_sensitive_compare_removed(self) -> None:
        self.assertTrue(gd.read_status().no_case_sensitive_compare)

    def test_cap_manifest_reuses_shared_collector(self) -> None:
        self.assertTrue(gd.read_status().cap_manifest_reuses_collector)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(gd.main(["--gate", "--format", "json"]), 0)

    def test_markdown_names_the_four_findings(self) -> None:
        md = gd.render_markdown(gd.read_status())
        self.assertIn("WIN-S33/S36/S37/S46", md)
        self.assertIn("Windows-proof-pending", md)


if __name__ == "__main__":
    unittest.main()
