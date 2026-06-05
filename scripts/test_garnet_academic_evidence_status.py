#!/usr/bin/env python3
"""Regression tests for the academic evidence-package gate (S118)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_academic_evidence_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_academic_evidence_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
ae = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_academic_evidence_status"] = ae
SPEC.loader.exec_module(ae)


class AcademicEvidenceTests(unittest.TestCase):
    def test_package_present_and_structured(self) -> None:
        r = ae.read_status()
        self.assertTrue(r.doc_present)
        self.assertTrue(r.has_contribution)
        self.assertTrue(r.has_refuse_section)

    def test_anchors_and_artifacts(self) -> None:
        r = ae.read_status()
        self.assertTrue(r.anchors_present)
        self.assertTrue(r.artifacts_cited)

    def test_every_sourced_pointer_resolves(self) -> None:
        r = ae.read_status()
        self.assertEqual(
            r.sourced_pointers_missing,
            0,
            f"dangling sources: {ae._missing_pointers()}",
        )
        self.assertGreaterEqual(r.sourced_pointers_total, 12)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(ae.main(["--gate", "--format", "json"]), 0)

    def test_markdown_carries_the_honesty_anchor(self) -> None:
        md = ae.render_markdown(ae.read_status())
        self.assertIn("no production / 1.0 claim", md)


if __name__ == "__main__":
    unittest.main()
