#!/usr/bin/env python3
"""Regression tests for the 12-domain / 7-novel proof matrix (S48)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_proof_matrix.py")
SPEC = importlib.util.spec_from_file_location("garnet_proof_matrix", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
proof_matrix = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_proof_matrix"] = proof_matrix
SPEC.loader.exec_module(proof_matrix)


class ProofMatrixTests(unittest.TestCase):
    def test_exactly_twelve_domains_all_present(self) -> None:
        m = proof_matrix.read_proof_matrix()
        self.assertEqual(len(m.domains), 12, "the matrix must cover 12 domains")
        self.assertTrue(m.all_domains_present, "every domain example must exist")

    def test_exactly_seven_contributions_all_exercised(self) -> None:
        m = proof_matrix.read_proof_matrix()
        self.assertEqual(len(m.contributions), 7, "Paper VI has 7 novel contributions")
        self.assertTrue(
            m.every_contribution_exercised,
            "every contribution must be exercised by existing in-repo evidence",
        )

    def test_paper_vi_scorecard_is_verbatim(self) -> None:
        m = proof_matrix.read_proof_matrix()
        # Honesty anchor — must not soften.
        self.assertEqual(
            m.paper_vi_scorecard,
            "4 supported, 2 partial (downgraded honestly), 0 refuted, 1 pending-infra",
        )

    def test_contribution_titles_are_the_canonical_seven(self) -> None:
        titles = {c.title for c in proof_matrix.read_proof_matrix().contributions}
        for needle in [
            "LLM-native syntax",
            "Type spectrum",
            "Compiler-as-agent",
            "Kind-aware allocation",
            "Error bridging",
            "Hot-reload",
            "Reproducible builds",
        ]:
            self.assertTrue(
                any(needle in t for t in titles),
                f"missing canonical contribution: {needle}",
            )

    def test_reuses_studio_domain_matrix_cases(self) -> None:
        # The 12 domains come from CORE_12_CASES (single source of truth).
        cases = proof_matrix._load_domain_cases()
        self.assertEqual([c.id for c in cases][0], "mvp_01_os_simulator")

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(proof_matrix.main(["--gate", "--format", "json"]), 0)

    def test_markdown_quotes_scorecard(self) -> None:
        md = proof_matrix.render_markdown(proof_matrix.read_proof_matrix())
        self.assertIn("Paper VI scorecard", md)
        self.assertIn("0 refuted, 1 pending-infra", md)


if __name__ == "__main__":
    unittest.main()
