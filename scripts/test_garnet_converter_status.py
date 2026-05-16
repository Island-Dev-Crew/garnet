#!/usr/bin/env python3
"""Regression tests for the Garnet converter capability reporter."""
from __future__ import annotations

import json
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = Path(__file__).with_name("garnet_converter_status.py")


class GarnetConverterStatusTests(unittest.TestCase):
    def test_json_reports_active_planned_and_gated_llm_lanes(self) -> None:
        output = subprocess.check_output(
            [sys.executable, str(SCRIPT), "--format", "json"],
            text=True,
        )
        data = json.loads(output)

        active = {item["id"]: item for item in data["active_languages"]}
        planned = {item["id"]: item for item in data["planned_languages"]}

        self.assertEqual(["go", "python", "ruby", "rust"], sorted(active))
        self.assertEqual("bidirectional-advisory-migration-surface", data["converter_scope"])
        self.assertTrue(data["trust_boundaries"]["sandbox_on_by_default"])
        self.assertTrue(data["trust_boundaries"]["lineage_required"])
        self.assertFalse(data["trust_boundaries"]["source_execution_allowed"])

        for language in (
            "javascript",
            "typescript",
            "swift",
            "java",
            "c",
            "cpp",
            "csharp",
            "perl",
            "kotlin",
            "shell",
            "sql",
            "other",
        ):
            self.assertIn(language, planned)
            self.assertEqual("planned", planned[language]["status"])

        native = {item["id"]: item for item in data["native_boundary_languages"]}
        for language in ("c", "cpp", "objective_c", "assembly", "cuda", "platform_specific"):
            self.assertIn(language, native)
            self.assertEqual("native-boundary-recommended", native[language]["status"])
            self.assertIn("FFI", native[language]["notes"])

        backend = data["backend_lowering_strategy"]
        self.assertEqual("planned-two-way-architecture", backend["status"])
        self.assertIn("Wasm", backend["planned_targets"])
        self.assertIn("LLVM-style native targets", backend["planned_targets"])
        self.assertIn("Do not claim source-to-source conversion preserves low-level fidelity.", backend["guardrails"])

        llm = data["llm_assist"]
        self.assertEqual("proposed-gated", llm["status"])
        self.assertTrue(llm["advisory_only"])
        self.assertTrue(llm["requires_lineage"])
        self.assertTrue(llm["requires_sandbox"])
        self.assertTrue(llm["requires_garnet_check"])
        self.assertFalse(llm["enabled_by_default"])

        contract = data["intelligent_assist_contract"]
        self.assertEqual("planned-contract", contract["status"])
        self.assertFalse(contract["provider_required"])
        self.assertFalse(contract["model_required"])
        self.assertIn("CURRENT_STATE.md", contract["required_context"])
        self.assertIn(
            "C_Language_Specification/GARNET_v1_0_Mini_Spec.md",
            contract["required_context"],
        )
        self.assertIn("safe-mode ownership candidates", contract["analysis_targets"])
        self.assertIn("memory declarations", contract["analysis_targets"])
        self.assertIn("CapCaps/capability boundaries", contract["analysis_targets"])
        self.assertIn("lineage per emitted node", contract["required_gates"])
        self.assertIn("@sandbox default", contract["required_gates"])
        self.assertIn("dogfood readiness bundle", contract["required_gates"])
        self.assertIn("source language classifier", contract["pipeline"])
        self.assertIn("risk inventory", contract["pipeline"])
        self.assertIn("human-approved candidate", contract["pipeline"])

    def test_markdown_is_user_facing_and_honest(self) -> None:
        rendered = subprocess.check_output(
            [sys.executable, str(SCRIPT)],
            text=True,
        )

        self.assertIn("Rust, Ruby, Python, and Go", rendered)
        self.assertIn("not a full transpiler", rendered)
        self.assertIn("Best-fit imports", rendered)
        self.assertIn("Bad direct-conversion fits", rendered)
        self.assertIn("Native boundary recommended", rendered)
        self.assertIn("Wasm", rendered)
        self.assertIn("LLVM-style native targets", rendered)
        self.assertIn("LLM assist is proposed only as a gated advisory lane", rendered)
        self.assertIn("JavaScript / TypeScript", rendered)
        self.assertIn("Kotlin", rendered)
        self.assertIn("Shell", rendered)
        self.assertIn("SQL", rendered)
        self.assertIn("Other", rendered)
        self.assertIn("Planned Garnet-Aware Assist Contract", rendered)
        self.assertIn("safe-mode ownership candidates", rendered)

    def test_docs_converter_section_matches_current_truth(self) -> None:
        site = (ROOT / "docs" / "index.html").read_text(encoding="utf-8")

        self.assertIn("Rust · Ruby · Python · Go", site)
        self.assertIn("migration assistant, not a full transpiler", site)
        self.assertIn("Active conversion", site)
        self.assertIn("Advisory planning", site)
        self.assertIn("Native boundary", site)
        self.assertIn("Kotlin", site)
        self.assertIn("Shell", site)
        self.assertIn("SQL", site)
        self.assertIn("Other", site)
        self.assertIn(
            "source classifier -> risk inventory -> Garnet context -> advisory plan -> review handoff -> human-approved candidate -> garnet check/test/dogfood",
            site,
        )
        self.assertIn("safe-mode", site)
        self.assertIn("memory", site)
        self.assertIn("CapCaps", site)
        self.assertIn("JavaScript", site)
        self.assertIn("TypeScript", site)
        self.assertIn("C++", site)
        self.assertNotIn("LLM assist: available", site)

    def test_strategy_doc_records_best_bad_and_two_way_architecture(self) -> None:
        strategy = (ROOT / "F_Project_Management" / "GARNET_CONVERTER_AND_PLATFORM_STRATEGY.md").read_text(
            encoding="utf-8"
        )

        self.assertIn("best-fit imports", strategy.lower())
        self.assertIn("bad direct-conversion fits", strategy.lower())
        self.assertIn("native modules or FFI", strategy)
        self.assertIn("Garnet lowers out to Wasm and LLVM-style native targets", strategy)
        self.assertIn("Kimi", strategy)
        self.assertIn("xAI", strategy)
        self.assertIn("local 1.58-bit", strategy)


if __name__ == "__main__":
    unittest.main()
