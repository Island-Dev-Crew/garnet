#!/usr/bin/env python3
"""Regression tests for garnet_memory_eviction_status.py."""
from __future__ import annotations

import importlib.util
import subprocess
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_memory_eviction_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_memory_eviction_status", SCRIPT)
assert SPEC is not None
mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_memory_eviction_status"] = mod
SPEC.loader.exec_module(mod)


class GarnetMemoryEvictionStatusTests(unittest.TestCase):
    def test_reads_live_bench_file_with_complete_coverage(self) -> None:
        status = mod.read_status()
        self.assertTrue(
            status.bench_file_present,
            "S6 bench file must exist at garnet-memory-v0.3/benches/eviction.rs",
        )
        self.assertTrue(
            status.cargo_entry_present,
            "Cargo.toml must declare a `[[bench]] name = \"eviction\"` entry",
        )
        self.assertTrue(
            status.coverage_complete,
            "All four Mnemos kinds (working/episodic/semantic/procedural) must be "
            "exercised with both naive_fifo and policy_score branches; got "
            f"{[k for k in status.kinds]}",
        )
        kinds = {k.kind for k in status.kinds}
        self.assertEqual({"working", "episodic", "semantic", "procedural"}, kinds)

    def test_cli_markdown_default(self) -> None:
        cp = subprocess.run(
            [sys.executable, str(SCRIPT)],
            capture_output=True,
            text=True,
        )
        self.assertEqual(0, cp.returncode, cp.stderr)
        self.assertIn(
            "Memory Eviction Benchmark Status", cp.stdout
        )
        for kind in ("working", "episodic", "semantic", "procedural"):
            self.assertIn(kind, cp.stdout)

    def test_cli_json_round_trip(self) -> None:
        import json
        cp = subprocess.run(
            [sys.executable, str(SCRIPT), "--format", "json"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(0, cp.returncode, cp.stderr)
        data = json.loads(cp.stdout)
        self.assertTrue(data["bench_file_present"])
        self.assertTrue(data["cargo_entry_present"])
        self.assertTrue(data["coverage_complete"])
        self.assertEqual(4, len(data["kinds"]))


if __name__ == "__main__":
    unittest.main()
