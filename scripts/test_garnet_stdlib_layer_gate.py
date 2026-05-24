#!/usr/bin/env python3
"""Regression tests for garnet_stdlib_layer_gate.py."""
from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_stdlib_layer_gate.py")
SPEC = importlib.util.spec_from_file_location("garnet_stdlib_layer_gate", SCRIPT)
assert SPEC is not None
mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_stdlib_layer_gate"] = mod
SPEC.loader.exec_module(mod)


SAMPLE = r'''
    fn build_prims() -> Vec<PrimMeta> {
        vec![
            p("time", "now_ms", 0, RequiredCaps::time(), Layer::Std, Stability::Stable,
              "Monotonic clock."),
            p("str", "split", 2, RequiredCaps::none(), Layer::Core, Stability::Stable,
              "Split a string."),
            p("core::iter", "map", 2, RequiredCaps::none(), Layer::Core, Stability::Experimental,
              "Map over a sequence."),
            p("std::env", "get", 1, RequiredCaps::env(), Layer::Std, Stability::Experimental,
              "Read an env var."),
            p("legacy", "old", 0, RequiredCaps::none(), Layer::Std, Stability::Deprecated,
              "An old primitive."),
        ]
    }
'''


class ParseRegistryTests(unittest.TestCase):
    def test_extracts_module_name_layer_stability(self) -> None:
        prims = mod.parse_registry(SAMPLE)
        self.assertEqual(len(prims), 5)
        by_q = {p["qualified"]: p for p in prims}
        self.assertEqual(by_q["core::iter::map"]["layer"], "Core")
        self.assertEqual(by_q["core::iter::map"]["stability"], "Experimental")
        self.assertEqual(by_q["std::env::get"]["layer"], "Std")
        self.assertEqual(by_q["time::now_ms"]["stability"], "Stable")

    def test_ignores_non_prim_text(self) -> None:
        # Prose mentioning Layer::Core or the word "map" must not be parsed as
        # a primitive — only `p("..","..",..)` calls are.
        noise = 'A doc comment mentioning Layer::Core and Stability::Stable.\n'
        self.assertEqual(mod.parse_registry(noise), [])


class SummarizeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.prims = mod.parse_registry(SAMPLE)

    def test_counts_by_layer_and_tier(self) -> None:
        s = mod.summarize(self.prims, doc_exists=True)
        self.assertEqual(s.total, 5)
        self.assertEqual(s.by_layer["Core"], 2)
        self.assertEqual(s.by_layer["Std"], 3)
        self.assertEqual(s.stability_breakdown["Experimental"], 2)
        self.assertEqual(s.stability_breakdown["Deprecated"], 1)

    def test_explicit_stability_is_full_when_all_tagged(self) -> None:
        s = mod.summarize(self.prims, doc_exists=True)
        self.assertEqual(s.explicit_stability_percent, 100.0)

    def test_deprecated_listed_with_removal_target(self) -> None:
        s = mod.summarize(self.prims, doc_exists=True)
        self.assertEqual(len(s.deprecated), 1)
        self.assertEqual(s.deprecated[0]["primitive"], "legacy::old")
        self.assertEqual(s.deprecated[0]["removal_target"], "next major")

    def test_count_gate_fails_below_fifty(self) -> None:
        s = mod.summarize(self.prims, doc_exists=True)
        self.assertFalse(s.meets_count_gate)  # only 5 prims
        self.assertTrue(s.meets_stability_gate)  # but 100% tagged
        self.assertFalse(s.ok)

    def test_doc_absence_fails_gate(self) -> None:
        s = mod.summarize(self.prims, doc_exists=False)
        self.assertFalse(s.layer_policy_doc_exists)
        self.assertFalse(s.ok)


class LiveRegistryTests(unittest.TestCase):
    def test_live_registry_meets_s17_gate(self) -> None:
        s = mod.read_status()
        self.assertGreaterEqual(s.total, 50, "S17 requires >= 50 primitives")
        self.assertGreaterEqual(
            s.explicit_stability_percent, 95.0, "S17 requires >= 95% explicit @stability"
        )
        self.assertTrue(s.layer_policy_doc_exists, "Layer Policy doc must exist")
        self.assertTrue(s.ok)

    def test_cli_json_roundtrips_and_exits_zero(self) -> None:
        proc = subprocess.run(
            [sys.executable, str(SCRIPT), "--format", "json"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        data = json.loads(proc.stdout)
        self.assertGreaterEqual(data["total"], 50)


if __name__ == "__main__":
    unittest.main()
