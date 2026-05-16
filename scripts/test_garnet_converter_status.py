#!/usr/bin/env python3
"""Regression tests for the Garnet converter capability reporter."""
from __future__ import annotations

import json
import subprocess
import sys
import unittest
from pathlib import Path

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
        self.assertEqual("stylized-migration-assistant", data["converter_scope"])
        self.assertTrue(data["trust_boundaries"]["sandbox_on_by_default"])
        self.assertTrue(data["trust_boundaries"]["lineage_required"])
        self.assertFalse(data["trust_boundaries"]["source_execution_allowed"])

        for language in ("javascript", "typescript", "swift", "java", "c", "cpp", "csharp", "perl"):
            self.assertIn(language, planned)
            self.assertEqual("planned", planned[language]["status"])

        llm = data["llm_assist"]
        self.assertEqual("proposed-gated", llm["status"])
        self.assertTrue(llm["advisory_only"])
        self.assertTrue(llm["requires_lineage"])
        self.assertTrue(llm["requires_sandbox"])
        self.assertTrue(llm["requires_garnet_check"])
        self.assertFalse(llm["enabled_by_default"])

    def test_markdown_is_user_facing_and_honest(self) -> None:
        rendered = subprocess.check_output(
            [sys.executable, str(SCRIPT)],
            text=True,
        )

        self.assertIn("Rust, Ruby, Python, and Go", rendered)
        self.assertIn("not a full transpiler", rendered)
        self.assertIn("LLM assist is proposed only as a gated advisory lane", rendered)
        self.assertIn("JavaScript / TypeScript", rendered)


if __name__ == "__main__":
    unittest.main()
