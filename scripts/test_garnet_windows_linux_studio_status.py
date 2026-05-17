#!/usr/bin/env python3
"""Regression tests for the Windows/Linux Studio MVP contract."""
from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from datetime import datetime, timezone
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_windows_linux_studio_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_windows_linux_studio_status", SCRIPT)
assert SPEC is not None
status_mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_windows_linux_studio_status"] = status_mod
SPEC.loader.exec_module(status_mod)


class GarnetWindowsLinuxStudioStatusTests(unittest.TestCase):
    def test_taxonomy_matches_handoff_copy_truth(self) -> None:
        status = status_mod.read_status()
        self.assertEqual(["Rust", "Ruby", "Python", "Go"], status.taxonomy.active_conversion)
        self.assertEqual(
            [
                "JavaScript",
                "TypeScript",
                "Swift",
                "Java",
                "C",
                "C++",
                "C#",
                "Perl",
                "Kotlin",
                "Shell",
                "SQL",
                "Other",
            ],
            status.taxonomy.advisory_planning,
        )
        self.assertEqual(
            ["C", "C++", "Objective-C", "Assembly", "CUDA", "platform-specific code"],
            status.taxonomy.native_boundary_recommended,
        )
        self.assertEqual(
            ["Wasm", "LLVM-style native targets", "native package toolchains"],
            status.taxonomy.future_backend_lowering,
        )

    def test_action_inventory_covers_required_windows_linux_mvp_actions(self) -> None:
        status = status_mod.read_status()
        action_ids = {action.id for action in status.actions}
        self.assertEqual(
            {
                "cli_health",
                "parse",
                "check",
                "run",
                "convert",
                "assist_plan",
                "advisory_bundle",
                "advisory_review",
                "advisory_handoff",
                "objective_pulse",
                "agentic_dogfood_matrix",
            },
            action_ids,
        )
        health = next(action for action in status.actions if action.id == "cli_health")
        self.assertEqual(["garnet", "version"], health.current_command)
        self.assertIn("garnet version", " ".join(status.current_truth))

    def test_command_plans_are_argument_vectors_and_preserve_default_no_source_handoff(self) -> None:
        source = Path("sample.ts")
        output = Path("dogfood")
        bundle = status_mod.build_command_plan(
            "advisory_bundle",
            language="TypeScript",
            source=source,
            evidence_dir=output,
            python_executable="python3",
        )
        self.assertIsInstance(bundle.argv, list)
        self.assertNotIn("--include-source", bundle.argv)
        self.assertFalse(bundle.source_included_by_default)
        self.assertFalse(bundle.calls_provider_apis)
        self.assertFalse(bundle.executes_source_code)

        review = status_mod.build_command_plan(
            "advisory_review",
            bundle_dir=Path("bundle"),
            evidence_dir=output,
            python_executable="python3",
        )
        handoff = status_mod.build_command_plan(
            "advisory_handoff",
            bundle_dir=Path("bundle"),
            review_dir=Path("review"),
            evidence_dir=output,
            python_executable="python3",
        )
        for plan in [bundle, review, handoff]:
            self.assertEqual("python3", plan.argv[0])
            self.assertTrue(all(isinstance(part, str) for part in plan.argv))
            self.assertNotIn(";", plan.argv)

    def test_convert_rejects_advisory_language_for_active_converter(self) -> None:
        with self.assertRaises(status_mod.CommandContractError):
            status_mod.build_command_plan("convert", language="TypeScript", source=Path("sample.ts"))

        convert = status_mod.build_command_plan(
            "convert",
            language="Python",
            source=Path("sample.py"),
            evidence_dir=Path("dogfood"),
        )
        self.assertEqual(["garnet", "convert", "python", "sample.py", "--out", "dogfood"], convert.argv)

    def test_evidence_bundle_creation_writes_manifest_without_source(self) -> None:
        fixed = datetime(2026, 5, 17, 12, 0, 0, tzinfo=timezone.utc)
        with tempfile.TemporaryDirectory() as temp:
            bundle = status_mod.create_evidence_bundle(Path(temp), now=fixed)
            self.assertEqual("garnet-studio-windows-linux-20260517-120000", bundle.name)
            contract_path = bundle / "garnet-windows-linux-studio-evidence-contract.json"
            manifest_path = bundle / "MANIFEST.sha256"
            self.assertTrue(contract_path.exists())
            self.assertTrue(manifest_path.exists())
            data = json.loads(contract_path.read_text(encoding="utf-8"))
            self.assertFalse(data["include_source_by_default"])
            self.assertFalse(data["source_included"])
            self.assertIn("garnet-windows-linux-studio-evidence-contract.json", manifest_path.read_text())

    def test_json_and_markdown_preserve_not_completed_boundary(self) -> None:
        output = subprocess.check_output(
            [sys.executable, str(SCRIPT), "--format", "json"],
            text=True,
        )
        data = json.loads(output)
        self.assertEqual("contract-only-runtime-proof-open", data["status"])
        self.assertIn("Windows/Linux Studio runtime proof is not complete", " ".join(data["current_truth"]))
        self.assertFalse(data["safety_contract"]["calls_provider_apis_by_default"])
        self.assertFalse(data["safety_contract"]["includes_source_by_default"])

        markdown = subprocess.check_output([sys.executable, str(SCRIPT)], text=True)
        self.assertIn("Garnet Windows/Linux Studio Status", markdown)
        self.assertIn("Tauri is a candidate dependency", markdown)
        self.assertIn("signed Windows MSI for Studio", json.dumps(data["safety_contract"]["forbidden_claims"]))


if __name__ == "__main__":
    unittest.main()
