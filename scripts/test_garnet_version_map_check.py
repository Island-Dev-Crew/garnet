#!/usr/bin/env python3
"""Regression tests for the v0.8 version-map source-of-truth check (S70)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_version_map_check.py")
SPEC = importlib.util.spec_from_file_location("garnet_version_map_check", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
vm = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_version_map_check"] = vm
SPEC.loader.exec_module(vm)


class VersionMapCheckTests(unittest.TestCase):
    def test_version_map_present_with_all_anchors(self) -> None:
        r = vm.read_readiness()
        self.assertTrue(r.version_map_present)
        self.assertEqual(r.missing_map_anchors, [], f"missing: {r.missing_map_anchors}")

    def test_contract_points_at_source_of_truth(self) -> None:
        self.assertTrue(vm.read_readiness().contract_points_at_map)

    def test_no_superseded_bold_band_cells_in_contract(self) -> None:
        r = vm.read_readiness()
        self.assertEqual(
            r.forbidden_cells_in_contract, [], f"drifted: {r.forbidden_cells_in_contract}"
        )

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(vm.main(["--gate", "--format", "json"]), 0)

    def test_markdown_states_single_cut_at_s80(self) -> None:
        md = vm.render_markdown(vm.read_readiness())
        self.assertIn("one `v0.8.0` tag at the", md)
        self.assertIn("cuts no tag", md)


if __name__ == "__main__":
    unittest.main()
