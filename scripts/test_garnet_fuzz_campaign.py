#!/usr/bin/env python3
"""Regression tests for the fuzz campaign inventory (S59)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_fuzz_campaign.py")
SPEC = importlib.util.spec_from_file_location("garnet_fuzz_campaign", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
fz = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_fuzz_campaign"] = fz
SPEC.loader.exec_module(fz)


class FuzzCampaignTests(unittest.TestCase):
    def test_harness_present_declared_and_wired(self) -> None:
        c = fz.read_campaign()
        self.assertTrue(c.target_file_present, "fuzz target file missing")
        self.assertTrue(c.declared_in_cargo, "target not declared in fuzz Cargo.toml")
        self.assertTrue(c.workflow_wired, "target not wired into fuzz-nightly.yml")
        self.assertTrue(c.ok)

    def test_seed_corpus_grew(self) -> None:
        # 8 original S20 seeds + the S59 newer-construct seeds.
        self.assertGreaterEqual(fz.read_campaign().seed_count, 13)

    def test_no_bug_claim(self) -> None:
        c = fz.read_campaign()
        self.assertIn("no bug-found", c.note.lower())
        self.assertIn("not run here", c.note.lower())

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(fz.main(["--gate", "--format", "json"]), 0)

    def test_markdown_states_no_bug_claims(self) -> None:
        md = fz.render_markdown(fz.read_campaign())
        self.assertIn("no bug claims", md)
        self.assertIn("Fuzz harness wired", md)


if __name__ == "__main__":
    unittest.main()
