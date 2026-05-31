#!/usr/bin/env python3
"""Regression tests for the Paper VI Exp 3 status reporter (S71)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_paper_vi_exp3_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_paper_vi_exp3_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
ex = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_paper_vi_exp3_status"] = ex
SPEC.loader.exec_module(ex)


class Exp3StatusTests(unittest.TestCase):
    def test_harness_well_formed_with_ten_snapshots(self) -> None:
        r = ex.read_status(run_harness=False)
        self.assertTrue(r.harness_present)
        self.assertEqual(r.snapshot_count, 10)
        self.assertTrue(r.lane_scripts_present and r.aggregate_present)

    def test_provider_backed_is_pending_infra(self) -> None:
        r = ex.read_status(run_harness=False)
        self.assertIn("pending-infra", r.provider_backed_status)

    def test_recorded_outcome_is_honest_partial(self) -> None:
        r = ex.read_status(run_harness=False)
        self.assertIn("partial", r.recorded_outcome["h3a"])
        self.assertIn("6.5%", r.recorded_outcome["h3a"])
        self.assertIn("pass", r.recorded_outcome["h3b"])
        self.assertIn("pass", r.recorded_outcome["h3c"])

    def test_c3_revision_quote_is_verbatim_in_exec_doc(self) -> None:
        # The honesty anchor must stay in sync with the source execution doc.
        self.assertTrue(ex.read_status(run_harness=False).c3_revision_in_doc)

    def test_gate_passes_running_harness_provider_free(self) -> None:
        # Actually runs both lanes (harness-only) + aggregate; all exit 0.
        self.assertEqual(ex.main(["--gate", "--format", "json"]), 0)

    def test_markdown_states_no_remeasure_no_llm(self) -> None:
        md = ex.render_markdown(ex.read_status(run_harness=False))
        self.assertIn("NOT re-measured here", md)
        self.assertIn("no LLM is called", md)


if __name__ == "__main__":
    unittest.main()
