#!/usr/bin/env python3
"""Regression tests for the post-tag release-truth status gate (S83)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_release_truth_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_release_truth_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
rt = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_release_truth_status"] = rt
SPEC.loader.exec_module(rt)


class ReleaseTruthTests(unittest.TestCase):
    def test_cut_doc_records_tag_cut_by_jon(self) -> None:
        self.assertTrue(rt.read_status().cut_doc_records_tag_cut)

    def test_readiness_only_truth_coexists(self) -> None:
        self.assertTrue(rt.read_status().cut_doc_keeps_readiness_only_truth)

    def test_changelog_records_the_cut(self) -> None:
        self.assertTrue(rt.read_status().changelog_records_cut)

    def test_ledger_s80_merged_with_cut_record(self) -> None:
        self.assertTrue(rt.read_status().ledger_s80_merged_with_cut_record)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(rt.main(["--gate", "--format", "json"]), 0)


if __name__ == "__main__":
    unittest.main()
