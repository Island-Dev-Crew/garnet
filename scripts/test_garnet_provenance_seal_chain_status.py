#!/usr/bin/env python3
"""Regression tests for the S97 provenance seal-chain status gate."""
from __future__ import annotations

import importlib.util
import io
import sys
import unittest
from contextlib import redirect_stdout
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_provenance_seal_chain_status.py")
provenance_status = None
LOAD_ERROR: Exception | None = None

if SCRIPT.is_file():
    SPEC = importlib.util.spec_from_file_location("garnet_provenance_seal_chain_status", SCRIPT)
    assert SPEC is not None and SPEC.loader is not None
    provenance_status = importlib.util.module_from_spec(SPEC)
    sys.modules["garnet_provenance_seal_chain_status"] = provenance_status
    try:
        SPEC.loader.exec_module(provenance_status)
    except Exception as exc:  # pragma: no cover - reported by the first test.
        LOAD_ERROR = exc
else:
    LOAD_ERROR = FileNotFoundError(SCRIPT)


class ProvenanceSealChainStatusTests(unittest.TestCase):
    def status(self):
        if LOAD_ERROR is not None:
            self.fail(f"could not load S97 status reporter: {LOAD_ERROR}")
        assert provenance_status is not None
        return provenance_status.read_status()

    def test_gate_passes_on_real_repo(self) -> None:
        if LOAD_ERROR is not None:
            self.fail(f"could not load S97 status reporter: {LOAD_ERROR}")
        assert provenance_status is not None
        with redirect_stdout(io.StringIO()):
            self.assertEqual(provenance_status.main(["--gate", "--format", "json"]), 0)

    def test_source_inventory_is_present(self) -> None:
        status = self.status()
        self.assertEqual(status.schema, "garnet.provenance_seal_chain/v1")
        self.assertTrue(status.rust_test_present)
        self.assertTrue(status.cli_flag_present)
        self.assertTrue(status.chain_builder_present)
        self.assertTrue(status.readiness_lane_present)

    def test_scope_text_stays_calibrated(self) -> None:
        status = self.status()
        self.assertIn("self-declared", status.scope_summary)
        self.assertIn("not independently verified", status.scope_summary)
        assert provenance_status is not None
        md = provenance_status.render_markdown(status)
        self.assertIn("does not prove the model executed the prompt", md)


if __name__ == "__main__":
    unittest.main()
