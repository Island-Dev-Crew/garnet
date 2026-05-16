#!/usr/bin/env python3
"""Regression tests for the Garnet-aware assist context pack."""
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = Path(__file__).with_name("garnet_assist_context_pack.py")


class GarnetAssistContextPackTests(unittest.TestCase):
    def test_json_reports_context_without_enabling_llm_conversion(self) -> None:
        output = subprocess.check_output(
            [sys.executable, str(SCRIPT), "--format", "json"],
            text=True,
        )
        data = json.loads(output)

        self.assertEqual("active-context-pack", data["status"])
        self.assertFalse(data["provider_required"])
        self.assertFalse(data["model_required"])
        self.assertFalse(data["network_required"])
        self.assertFalse(data["enabled_by_default"])
        self.assertFalse(data["llm_conversion_active"])
        self.assertEqual("planned-contract", data["assist_contract_status"])
        self.assertIn("not active conversion today", data["current_truth"])

        self.assertEqual(["Go", "Python", "Ruby", "Rust"], sorted(data["active_languages"]))
        self.assertIn("JavaScript", data["planned_languages"])
        self.assertIn("TypeScript", data["planned_languages"])
        self.assertIn("C++", data["planned_languages"])

    def test_context_documents_are_real_and_hashed(self) -> None:
        output = subprocess.check_output(
            [sys.executable, str(SCRIPT), "--format", "json"],
            text=True,
        )
        data = json.loads(output)
        docs = {item["path"]: item for item in data["context_documents"]}

        required = {
            "CURRENT_STATE.md",
            "README.md",
            "C_Language_Specification/GARNET_v1_0_Mini_Spec.md",
            "C_Language_Specification/GARNET_v0_4_2_Conformance_Matrix.md",
            "F_Project_Management/DOGFOOD/GARNET_v0_5_DOGFOOD_READINESS_PHASE_LOG.md",
        }
        self.assertTrue(required.issubset(docs))

        for item in docs.values():
            self.assertTrue(item["exists"], item["path"])
            self.assertGreater(item["bytes"], 0, item["path"])
            self.assertEqual(64, len(item["sha256"]), item["path"])
            self.assertIn(item["role"], {"current-truth", "public-entry", "spec", "conformance", "dogfood"})

    def test_pack_preserves_analysis_targets_and_required_gates(self) -> None:
        output = subprocess.check_output(
            [sys.executable, str(SCRIPT), "--format", "json"],
            text=True,
        )
        data = json.loads(output)

        for target in (
            "safe-mode ownership candidates",
            "memory declarations",
            "CapCaps/capability boundaries",
            "actor/orchestration mappings",
            "migration risk inventory",
        ):
            self.assertIn(target, data["analysis_targets"])

        for gate in (
            "lineage per emitted node",
            "@sandbox default",
            "migrate_todo evidence",
            "garnet check",
            "dogfood readiness bundle",
            "human audit before unquarantine",
        ):
            self.assertIn(gate, data["required_gates"])

        self.assertIn("deterministic converter output remains authoritative", data["system_boundaries"])
        self.assertIn("source code is not executed during analysis", data["system_boundaries"])

    def test_json_includes_provider_neutral_prompt_contract(self) -> None:
        output = subprocess.check_output(
            [sys.executable, str(SCRIPT), "--format", "json"],
            text=True,
        )
        data = json.loads(output)
        prompt = data["prompt_pack"]

        self.assertEqual("provider-neutral-assist-prompt", prompt["status"])
        self.assertFalse(prompt["provider_required"])
        self.assertFalse(prompt["network_required"])
        self.assertFalse(prompt["conversion_active"])
        self.assertIn("assist plan JSON", prompt["required_inputs"])
        self.assertIn("candidate Garnet output or migrate_todo evidence", prompt["required_output_sections"])
        self.assertIn("Do not execute source code.", prompt["system_prompt"])
        self.assertIn("Never claim conversion is active.", prompt["system_prompt"])
        self.assertIn("lineage per emitted node", prompt["system_prompt"])
        self.assertIn("human audit before unquarantine", prompt["user_prompt_template"])

    def test_markdown_is_human_readable_and_honest(self) -> None:
        rendered = subprocess.check_output([sys.executable, str(SCRIPT)], text=True)

        self.assertIn("Garnet Assist Context Pack", rendered)
        self.assertIn("not active conversion today", rendered)
        self.assertIn("provider-optional", rendered)
        self.assertIn("Rust, Ruby, Python, Go", rendered)
        self.assertIn("JavaScript", rendered)
        self.assertIn("safe-mode ownership candidates", rendered)
        self.assertIn("dogfood readiness bundle", rendered)

    def test_output_dir_writes_manifested_pack(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            out_dir = Path(temp) / "context-pack"
            subprocess.check_call(
                [
                    sys.executable,
                    str(SCRIPT),
                    "--output-dir",
                    str(out_dir),
                ],
                stdout=subprocess.DEVNULL,
            )

            self.assertTrue((out_dir / "garnet-assist-context-pack.json").exists())
            self.assertTrue((out_dir / "garnet-assist-context-pack.md").exists())
            self.assertTrue((out_dir / "garnet-assist-prompt-pack.md").exists())
            manifest = out_dir / "MANIFEST.sha256"
            self.assertTrue(manifest.exists())
            verify = subprocess.check_output(
                ["shasum", "-a", "256", "-c", str(manifest.name)],
                cwd=out_dir,
                text=True,
            )
            self.assertIn("garnet-assist-context-pack.json: OK", verify)
            self.assertIn("garnet-assist-context-pack.md: OK", verify)
            self.assertIn("garnet-assist-prompt-pack.md: OK", verify)


if __name__ == "__main__":
    unittest.main()
