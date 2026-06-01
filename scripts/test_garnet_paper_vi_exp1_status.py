#!/usr/bin/env python3
"""Regression tests for the S94 Paper VI Exp 1 provider-gated harness."""
from __future__ import annotations

import importlib.util
import io
import sys
import unittest
from contextlib import redirect_stdout
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_paper_vi_exp1_status.py")
exp1 = None
LOAD_ERROR: Exception | None = None

if SCRIPT.is_file():
    SPEC = importlib.util.spec_from_file_location("garnet_paper_vi_exp1_status", SCRIPT)
    assert SPEC is not None and SPEC.loader is not None
    exp1 = importlib.util.module_from_spec(SPEC)
    sys.modules["garnet_paper_vi_exp1_status"] = exp1
    try:
        SPEC.loader.exec_module(exp1)
    except Exception as exc:  # pragma: no cover - reported by the first test.
        LOAD_ERROR = exc
else:
    LOAD_ERROR = FileNotFoundError(SCRIPT)


class PaperViExp1StatusTests(unittest.TestCase):
    def status(self):
        if LOAD_ERROR is not None:
            self.fail(f"could not load S94 status reporter: {LOAD_ERROR}")
        assert exp1 is not None
        return exp1.read_status()

    def test_gate_passes_on_real_repo(self) -> None:
        if LOAD_ERROR is not None:
            self.fail(f"could not load S94 status reporter: {LOAD_ERROR}")
        assert exp1 is not None
        with redirect_stdout(io.StringIO()):
            self.assertEqual(exp1.main(["--gate", "--format", "json"]), 0)

    def test_seed_corpus_is_present_but_not_full_benchmark(self) -> None:
        status = self.status()
        self.assertGreaterEqual(status.seed_task_count, 3)
        self.assertEqual(status.full_corpus_status, "seed-only")

    def test_provider_free_run_is_pending_not_measured(self) -> None:
        status = self.status()
        self.assertTrue(status.provider_free_run_ok)
        self.assertEqual(status.provider_free_status, "pending-infra")
        self.assertFalse(status.provider_free_claims_measurement)

    def test_fixture_run_scores_pass_and_fail_without_network(self) -> None:
        status = self.status()
        self.assertTrue(status.fixture_run_ok)
        self.assertEqual(status.fixture_measured_rows, 6)
        self.assertEqual(status.fixture_pass_rows, 3)

    def test_real_provider_path_stays_credential_gated(self) -> None:
        status = self.status()
        self.assertEqual(status.provider_backed_status, "pending-credentials")
        self.assertTrue(status.provider_flag_present)

    def test_markdown_preserves_no_measurement_boundary(self) -> None:
        status = self.status()
        assert exp1 is not None
        md = exp1.render_markdown(status)
        self.assertIn("No provider-backed pass@1 measurement is claimed", md)
        self.assertIn("seed-only", md)


if __name__ == "__main__":
    unittest.main()
