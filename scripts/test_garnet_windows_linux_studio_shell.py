#!/usr/bin/env python3
"""Regression tests for the Windows/Linux Tauri Studio scaffold."""
from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
APP = ROOT / "apps" / "garnet-studio"


class GarnetWindowsLinuxStudioShellTests(unittest.TestCase):
    def test_tauri_v2_scaffold_uses_minimal_frontend_dependencies(self) -> None:
        package = json.loads((APP / "package.json").read_text(encoding="utf-8"))
        self.assertEqual("garnet-studio", package["name"])
        self.assertIn("@tauri-apps/api", package["dependencies"])
        self.assertIn("@tauri-apps/cli", package["devDependencies"])
        self.assertNotIn("@tauri-apps/plugin-shell", package["dependencies"])
        self.assertNotIn("@tauri-apps/plugin-opener", package["dependencies"])

        config = json.loads((APP / "src-tauri" / "tauri.conf.json").read_text(encoding="utf-8"))
        self.assertEqual("Garnet Studio", config["productName"])
        self.assertFalse(config["app"]["withGlobalTauri"])
        self.assertEqual(["nsis"], config["bundle"]["targets"])
        self.assertNotIn("fonts.googleapis.com", config["app"]["security"]["csp"])

    def test_backend_exposes_required_actions_without_shell_plugin_permission(self) -> None:
        backend = (APP / "src-tauri" / "src" / "commands.rs").read_text(encoding="utf-8")
        lib = (APP / "src-tauri" / "src" / "lib.rs").read_text(encoding="utf-8")
        cargo = (APP / "src-tauri" / "Cargo.toml").read_text(encoding="utf-8")
        capability = json.loads(
            (APP / "src-tauri" / "capabilities" / "default.json").read_text(encoding="utf-8")
        )

        for command in [
            "cli_health",
            "cli_parse",
            "cli_check",
            "cli_run",
            "cli_convert",
            "advisory_assist_plan",
            "advisory_bundle",
            "advisory_review",
            "advisory_handoff",
            "objective_pulse",
            "agentic_dogfood_matrix",
            "domain_proof_matrix",
            "windows_linux_studio_status",
            "converter_status",
            "provider_options",
            "mit_demo_route",
            "mit_deck_outline",
            "mit_deck_preview",
            "mac_continuation_pulse",
            "proof_benchmark_status",
            "benchmark_no_run",
            "notarization_status",
            "windows_vm_installer_status",
            "create_evidence_bundle",
        ]:
            self.assertIn(command, backend + lib)

        self.assertNotIn("tauri-plugin-shell", cargo)
        self.assertEqual(["core:default"], capability["permissions"])

    def test_release_readiness_panel_matches_v05_truth_without_completion_claims(self) -> None:
        frontend = (APP / "index.html").read_text(encoding="utf-8")
        main = (APP / "src" / "main.ts").read_text(encoding="utf-8")
        backend = (APP / "src-tauri" / "src" / "commands.rs").read_text(encoding="utf-8")

        for copy in [
            "Release / Readiness",
            "Windows/Linux Status",
            "Domain Proof Matrix",
            "Converter Fit Matrix",
            "Provider Options",
            "Demo Route",
            "Deck Outline",
            "Deck Preview",
            "Continuation Pulse",
            "Proof / Benchmark Status",
            "Benchmark No-Run",
            "Notarization Status",
            "Windows VM Installer",
        ]:
            self.assertIn(copy, frontend)

        for command in [
            "windows_linux_studio_status",
            "domain_proof_matrix",
            "converter_status",
            "provider_options",
            "mit_demo_route",
            "mit_deck_outline",
            "mit_deck_preview",
            "mac_continuation_pulse",
            "proof_benchmark_status",
            "benchmark_no_run",
            "notarization_status",
            "windows_vm_installer_status",
        ]:
            self.assertIn(command, main)

        self.assertIn("garnet_windows_linux_studio_status.py", backend)
        self.assertIn("smoke_garnet_studio_domain_matrix.py", backend)
        self.assertIn("garnet_converter_llm_feasibility.py", backend)
        self.assertIn("garnet_mit_deck_preview.py", backend)
        self.assertIn("garnet_studio_notarization_status.py", backend)
        self.assertIn("garnet_windows_clean_vm_installer_status.py", backend)
        self.assertIn("Linux launch", frontend)
        self.assertIn("Clean Windows VM proof bundle", frontend)
        self.assertIn("x64 clean-VM proof first", frontend)
        self.assertNotIn("Windows/Linux desktop runtime complete", frontend)
        self.assertNotIn("provider-backed conversion is active", frontend)

    def test_converter_direction_and_command_shape_match_repo_truth(self) -> None:
        frontend = (APP / "index.html").read_text(encoding="utf-8")
        backend = (APP / "src-tauri" / "src" / "commands.rs").read_text(encoding="utf-8")

        self.assertIn("Rust/Ruby/Python/Go to Garnet", frontend)
        self.assertIn("Convert to Garnet", frontend)
        self.assertNotIn("Convert Garnet source to", frontend)
        self.assertIn('"convert".to_string()', backend)
        self.assertIn('"--out".to_string()', backend)
        self.assertNotIn('"--to"', backend)
        self.assertIn("ACTIVE_CONVERSION", backend)
        self.assertIn("advisory_plan_rejects_active_conversion_languages", backend)
        self.assertIn("normalize_language(&language, ADVISORY_LANGUAGES)", backend)

    def test_evidence_and_smoke_contracts_are_windows_linux_specific(self) -> None:
        paths = (APP / "src-tauri" / "src" / "paths.rs").read_text(encoding="utf-8")
        evidence = (APP / "src-tauri" / "src" / "evidence.rs").read_text(encoding="utf-8")
        main = (APP / "src-tauri" / "src" / "main.rs").read_text(encoding="utf-8")
        lib = (APP / "src-tauri" / "src" / "lib.rs").read_text(encoding="utf-8")

        self.assertIn('"garnet-studio-windows-linux"', paths)
        self.assertIn("MANIFEST.sha256", evidence)
        self.assertIn('"source_included": false', evidence)
        self.assertIn('"provider_api_called": false', lib)
        self.assertIn("--studio-smoke", main)
        self.assertIn("--studio-domain-proof-smoke", main)
        self.assertIn("run_domain_proof_smoke", lib)
        self.assertIn("domain-proof-shell-smoke", lib)
        self.assertIn("non_wsl_linux_desktop_claimed", lib)
        self.assertIn("not Linux seccomp or OS-sandbox enforcement", lib)


if __name__ == "__main__":
    unittest.main()
