#!/usr/bin/env python3
"""Regression tests for the converter LLM feasibility reporter."""
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_converter_llm_feasibility.py")


class GarnetConverterLlmFeasibilityTests(unittest.TestCase):
    def test_json_is_feasible_for_advisory_assist_only(self) -> None:
        output = subprocess.check_output(
            [sys.executable, str(SCRIPT), "--format", "json"],
            text=True,
        )
        data = json.loads(output)

        self.assertEqual("advisory-feasible", data["status"])
        self.assertFalse(data["conversion_active"])
        self.assertFalse(data["autonomous_conversion_feasible"])
        self.assertFalse(data["provider_required"])
        self.assertFalse(data["model_required"])
        self.assertFalse(data["network_required"])
        self.assertEqual("provider-neutral advisory planning", data["recommended_first_lane"])
        self.assertIn("not active LLM conversion", data["current_truth"])
        self.assertIn("secure advisory implementation", data["blocking_gates"])
        self.assertIn("human audit before unquarantine", data["required_gates"])

    def test_reports_user_requested_language_surface(self) -> None:
        output = subprocess.check_output(
            [sys.executable, str(SCRIPT), "--format", "json"],
            text=True,
        )
        data = json.loads(output)

        active = {item["id"] for item in data["active_languages"]}
        planned = {item["id"] for item in data["planned_languages"]}
        coverage = {item["id"]: item for item in data["planned_language_assist_coverage"]}

        self.assertGreaterEqual(active, {"rust", "ruby", "python", "go"})
        self.assertGreaterEqual(
            planned,
            {
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
            },
        )
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
            self.assertEqual("planned-assist-required", coverage[language]["status"])
            self.assertFalse(coverage[language]["deterministic_converter_available"])
            self.assertFalse(coverage[language]["llm_conversion_active"])

        native = {item["id"] for item in data["native_boundary_languages"]}
        self.assertGreaterEqual(native, {"c", "cpp", "objective_c", "assembly", "cuda", "platform_specific"})
        self.assertIn("source language classifier", data["recommended_pipeline"])
        self.assertIn("human-approved candidate", data["recommended_pipeline"])

    def test_markdown_answers_feasibility_without_overclaiming(self) -> None:
        rendered = subprocess.check_output([sys.executable, str(SCRIPT)], text=True)

        self.assertIn("Garnet Converter LLM Feasibility", rendered)
        self.assertIn("Advisory assist is feasible", rendered)
        self.assertIn("Autonomous LLM conversion is not feasible yet", rendered)
        self.assertIn("Kotlin, Shell, SQL, Other", rendered)
        self.assertIn("Native Boundary Coverage", rendered)
        self.assertIn("source language classifier", rendered)
        self.assertIn("not active LLM conversion", rendered)
        self.assertIn("human audit before unquarantine", rendered)

    def test_output_dir_writes_manifested_feasibility_report(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            out_dir = Path(temp) / "feasibility"
            subprocess.check_call(
                [
                    sys.executable,
                    str(SCRIPT),
                    "--output-dir",
                    str(out_dir),
                ],
                stdout=subprocess.DEVNULL,
            )

            self.assertTrue((out_dir / "garnet-converter-llm-feasibility.json").exists())
            self.assertTrue((out_dir / "garnet-converter-llm-feasibility.md").exists())
            manifest = out_dir / "MANIFEST.sha256"
            self.assertTrue(manifest.exists())
            verify = subprocess.check_output(
                ["shasum", "-a", "256", "-c", manifest.name],
                cwd=out_dir,
                text=True,
            )

        self.assertIn("garnet-converter-llm-feasibility.json: OK", verify)
        self.assertIn("garnet-converter-llm-feasibility.md: OK", verify)


if __name__ == "__main__":
    unittest.main()
