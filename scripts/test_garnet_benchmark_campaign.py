#!/usr/bin/env python3
"""Regression tests for the benchmark campaign inventory (S58)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_benchmark_campaign.py")
SPEC = importlib.util.spec_from_file_location("garnet_benchmark_campaign", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
camp = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_benchmark_campaign"] = camp
SPEC.loader.exec_module(camp)


class BenchmarkCampaignTests(unittest.TestCase):
    def test_all_six_benches_present_and_declared(self) -> None:
        c = camp.read_campaign()
        self.assertEqual(len(c.benches), 6)
        for s in c.benches:
            self.assertTrue(s.file_present, f"{s.crate}::{s.name} bench file missing")
            self.assertTrue(s.declared_in_cargo, f"{s.crate}::{s.name} not in Cargo.toml")
        self.assertTrue(c.all_present)

    def test_run_commands_are_per_bench(self) -> None:
        for s in camp.read_campaign().benches:
            self.assertIn(f"--bench {s.name}", s.run_command)
            self.assertIn(s.crate, s.run_command)

    def test_no_measurement_claim(self) -> None:
        c = camp.read_campaign()
        self.assertIn("no measurements are run or claimed", c.note)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(camp.main(["--gate", "--format", "json"]), 0)

    def test_markdown_states_no_measurements(self) -> None:
        md = camp.render_markdown(camp.read_campaign())
        self.assertIn("no measurements claimed", md)
        self.assertIn("All 6 benches present", md)


if __name__ == "__main__":
    unittest.main()
