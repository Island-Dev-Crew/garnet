#!/usr/bin/env python3
"""Regression tests for the seal source-hash determinism gate (S82)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_seal_determinism_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_seal_determinism_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
sd = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_seal_determinism_status"] = sd
SPEC.loader.exec_module(sd)


class SealDeterminismTests(unittest.TestCase):
    def test_gitattributes_pins_garnet_lf(self) -> None:
        self.assertTrue(sd.read_status().gitattributes_pins_garnet_lf)

    def test_manifest_normalizes_source_eol(self) -> None:
        self.assertTrue(sd.read_status().manifest_normalizes_source_eol)

    def test_contract_documented(self) -> None:
        self.assertTrue(sd.read_status().contract_documented)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(sd.main(["--gate", "--format", "json"]), 0)

    def test_markdown_names_win_s38(self) -> None:
        md = sd.render_markdown(sd.read_status())
        self.assertIn("WIN-S38-001", md)
        self.assertIn("Windows-proof-pending", md)


if __name__ == "__main__":
    unittest.main()
