#!/usr/bin/env python3
"""Regression tests for the S95 Paper VI Exp 3 5K-LOC harness gate."""
from __future__ import annotations

import importlib.util
import io
import sys
import unittest
from contextlib import redirect_stdout
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_paper_vi_exp3_5k_status.py")
exp3_5k = None
LOAD_ERROR: Exception | None = None

if SCRIPT.is_file():
    SPEC = importlib.util.spec_from_file_location("garnet_paper_vi_exp3_5k_status", SCRIPT)
    assert SPEC is not None and SPEC.loader is not None
    exp3_5k = importlib.util.module_from_spec(SPEC)
    sys.modules["garnet_paper_vi_exp3_5k_status"] = exp3_5k
    try:
        SPEC.loader.exec_module(exp3_5k)
    except Exception as exc:  # pragma: no cover - reported by the first test.
        LOAD_ERROR = exc
else:
    LOAD_ERROR = FileNotFoundError(SCRIPT)


class PaperViExp3FiveKStatusTests(unittest.TestCase):
    def status(self):
        if LOAD_ERROR is not None:
            self.fail(f"could not load S95 status reporter: {LOAD_ERROR}")
        assert exp3_5k is not None
        return exp3_5k.read_status()

    def test_gate_passes_on_real_repo(self) -> None:
        if LOAD_ERROR is not None:
            self.fail(f"could not load S95 status reporter: {LOAD_ERROR}")
        assert exp3_5k is not None
        with redirect_stdout(io.StringIO()):
            self.assertEqual(exp3_5k.main(["--gate", "--format", "json"]), 0)

    def test_generated_corpus_has_ten_snapshots_at_5k_loc(self) -> None:
        status = self.status()
        self.assertEqual(status.snapshot_count, 10)
        self.assertGreaterEqual(status.min_snapshot_loc, 5000)
        self.assertGreaterEqual(status.total_generated_loc, 50000)

    def test_provider_free_rows_cover_both_lanes(self) -> None:
        status = self.status()
        self.assertTrue(status.provider_free_run_ok)
        self.assertEqual(status.stateless_rows, 10)
        self.assertEqual(status.history_aware_rows, 10)

    def test_h3a_stays_pending_without_measured_rows(self) -> None:
        status = self.status()
        self.assertEqual(status.measured_rows, 0)
        self.assertEqual(status.h3a_status, "pending-provider-rerun")
        self.assertIn("6.5% partial stands", status.analysis_summary)

    def test_markdown_preserves_no_remeasurement_boundary(self) -> None:
        status = self.status()
        assert exp3_5k is not None
        md = exp3_5k.render_markdown(status)
        self.assertIn("No 5K h3a measurement is claimed", md)
        self.assertIn("pending-provider-rerun", md)


if __name__ == "__main__":
    unittest.main()
