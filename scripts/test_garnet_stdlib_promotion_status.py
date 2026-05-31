#!/usr/bin/env python3
"""Regression tests for the stdlib promotion-wave status reporter (S76)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_stdlib_promotion_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_stdlib_promotion_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
pr = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_stdlib_promotion_status"] = pr
SPEC.loader.exec_module(pr)


class PromotionTests(unittest.TestCase):
    def test_all_core_primitives_now_stable(self) -> None:
        r = pr.read_status()
        self.assertEqual(r.core_experimental, [], f"core still experimental: {r.core_experimental}")
        self.assertGreater(r.core_stable, 0)
        self.assertEqual(r.core_stable, r.core_total)

    def test_std_kept_experimental_proves_scoped_not_blanket(self) -> None:
        # The wave must NOT have flipped std::*; that proves it was principled.
        self.assertGreater(pr.read_status().std_experimental_count, 0)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(pr.main(["--gate", "--format", "json"]), 0)

    def test_promoted_families_are_core_layer(self) -> None:
        for fam in pr.PROMOTED_FAMILIES:
            self.assertTrue(fam.startswith("core::"), f"{fam} is not core-layer")

    def test_markdown_states_not_warning_suppression(self) -> None:
        md = pr.render_markdown(pr.read_status())
        self.assertIn("not warning-suppression", md)
        self.assertIn("KEEPS std::* experimental", md)


if __name__ == "__main__":
    unittest.main()
