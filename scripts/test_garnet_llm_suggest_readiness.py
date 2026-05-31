#!/usr/bin/env python3
"""Regression tests for the LLM-suggest readiness reporter (S69)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_llm_suggest_readiness.py")
SPEC = importlib.util.spec_from_file_location("garnet_llm_suggest_readiness", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
ls = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_llm_suggest_readiness"] = ls
SPEC.loader.exec_module(ls)


class LlmSuggestReadinessTests(unittest.TestCase):
    def test_rules_tier_ids_present_in_suggest_rs(self) -> None:
        r = ls.read_readiness()
        self.assertEqual(r.missing_rule_ids, [], f"missing: {r.missing_rule_ids}")
        self.assertTrue(r.rules_tier_present and r.rules_tier_ready)
        self.assertEqual(len(r.rules_tier_ids), 3)

    def test_llm_tier_is_pending_infra(self) -> None:
        r = ls.read_readiness()
        self.assertIn("pending-infra", r.llm_tier_status)

    def test_scorecard_is_verbatim(self) -> None:
        r = ls.read_readiness()
        self.assertEqual(
            r.paper_vi_scorecard,
            "4 supported, 2 partial (downgraded honestly), 0 refuted, 1 pending-infra",
        )

    def test_gate_passes_on_real_repo(self) -> None:
        # The rules tier is present; the LLM tier is NOT gated (pending-infra).
        self.assertEqual(ls.main(["--gate", "--format", "json"]), 0)

    def test_markdown_states_no_model_called(self) -> None:
        md = ls.render_markdown(ls.read_readiness())
        self.assertIn("no model is called", md)
        self.assertIn("pending-infra", md)


if __name__ == "__main__":
    unittest.main()
