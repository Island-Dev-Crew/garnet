#!/usr/bin/env python3
"""Regression tests for the Windows/Linux Studio MVP contract."""
from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from datetime import datetime, timezone
from pathlib import Path
from unittest import mock

TEST_CLEAN_VM_ROOT = tempfile.TemporaryDirectory()
os.environ["GARNET_WINDOWS_CLEAN_VM_EVIDENCE_ROOT"] = TEST_CLEAN_VM_ROOT.name
SCRIPT = Path(__file__).with_name("garnet_windows_linux_studio_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_windows_linux_studio_status", SCRIPT)
assert SPEC is not None
status_mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_windows_linux_studio_status"] = status_mod
SPEC.loader.exec_module(status_mod)


class GarnetWindowsLinuxStudioStatusTests(unittest.TestCase):
    @classmethod
    def tearDownClass(cls) -> None:
        TEST_CLEAN_VM_ROOT.cleanup()

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
            },
            action_ids,
        )
        health = next(action for action in status.actions if action.id == "cli_health")
        self.assertEqual(["garnet", "version"], health.current_command)
        self.assertIn("garnet version", " ".join(status.current_truth))
        provider = next(action for action in status.actions if action.id == "provider_options")
        self.assertIn("--output-dir", provider.current_command)
        self.assertIn("garnet_converter_llm_feasibility.py", " ".join(provider.current_command))
        self.assertFalse(provider.source_included_by_default)
        domain = next(action for action in status.actions if action.id == "domain_proof_matrix")
        self.assertIn("smoke_garnet_studio_domain_matrix.py", " ".join(domain.current_command))
        self.assertFalse(domain.source_included_by_default)

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

    def test_advisory_actions_reject_active_conversion_languages(self) -> None:
        for action_id in ["assist_plan", "advisory_bundle"]:
            with self.assertRaises(status_mod.CommandContractError):
                status_mod.build_command_plan(
                    action_id,
                    language="Rust",
                    source=Path("sample.rs"),
                )

    def test_release_readiness_plans_call_repo_native_reporters(self) -> None:
        evidence = Path("dogfood")
        plan = status_mod.build_command_plan(
            "mit_deck_preview",
            evidence_dir=evidence,
            python_executable="python3",
        )

        self.assertEqual("MIT Deck Preview", plan.label)
        self.assertEqual("python3", plan.argv[0])
        self.assertIn("garnet_mit_deck_preview.py", plan.argv[1])
        self.assertIn("--output-dir", plan.argv)
        self.assertIn("dogfood", plan.argv)
        self.assertIn("html", plan.argv)
        self.assertFalse(plan.calls_provider_apis)
        self.assertFalse(plan.source_included_by_default)

        domain = status_mod.build_command_plan(
            "domain_proof_matrix",
            evidence_dir=evidence,
            python_executable="python3",
        )
        self.assertEqual("Domain Proof Matrix", domain.label)
        self.assertIn("smoke_garnet_studio_domain_matrix.py", domain.argv[1])
        self.assertTrue(domain.executes_source_code)
        self.assertFalse(domain.calls_provider_apis)
        self.assertFalse(domain.source_included_by_default)

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

    def test_clean_vm_proof_updates_unsigned_nsis_gate_without_overclaiming(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            bundle = root / "proof"
            installer = root / "Garnet Studio_0.1.0_x64-setup.exe"
            install_log = root / "install.log"
            smoke = root / "studio-smoke.json"
            screenshot = root / "launch.png"
            installer.write_bytes(b"fake installer")
            install_log.write_text("InstallerExitCode=0\n", encoding="utf-8")
            smoke.write_text(
                json.dumps(
                    {
                        "status": "passed",
                        "source_included": False,
                        "provider_api_called": False,
                    }
                ),
                encoding="utf-8",
            )
            screenshot.write_bytes(b"fake png")
            clean_vm_record = status_mod.garnet_windows_clean_vm_installer_status.build_proof_record(
                mode="clean-vm",
                installer=installer,
                vm_name="WindowsSandbox-123",
                guest_os="Windows 11 Enterprise",
                guest_arch="AMD64",
                install_log=install_log,
                studio_smoke_json=smoke,
                screenshot=screenshot,
            )
            status_mod.garnet_windows_clean_vm_installer_status.write_proof(clean_vm_record, bundle)
            linux_xvfb_missing = status_mod.smoke_garnet_studio_linux_wsl_xvfb.LinuxWslXvfbEvidence(
                status="missing",
                verified=False,
                reason="No committed WSL Linux Xvfb runtime-start proof bundle verified.",
                bundle=None,
                deferred=["record WSL Linux Xvfb proof bundle"],
            )
            linux_xvfb_window_missing = (
                status_mod.smoke_garnet_studio_linux_wsl_xvfb_window.LinuxWslXvfbWindowEvidence(
                    status="missing",
                    verified=False,
                    reason="No committed WSL Linux Xvfb virtual-display window-capture proof bundle verified.",
                    bundle=None,
                    deferred=["record WSL Linux Xvfb window-capture proof bundle"],
                )
            )

            with mock.patch.object(
                status_mod.smoke_garnet_studio_linux_wsl_xvfb,
                "read_committed_evidence",
                return_value=linux_xvfb_missing,
            ), mock.patch.object(
                status_mod.smoke_garnet_studio_linux_wsl_xvfb_window,
                "read_committed_evidence",
                return_value=linux_xvfb_window_missing,
            ):
                status = status_mod.read_status(clean_vm_evidence_root=root)

        self.assertEqual(
            "tauri-v2-shell-v0-5-readiness-parity-windows-clean-vm-verified-wsl-deb-rpm-extract-verified-linux-gui-still-open",
            status.status,
        )
        gate = next(gate for gate in status.packaging_gates if gate.id == "windows_unsigned_nsis")
        self.assertEqual("clean-vm-proof-verified", gate.status)
        linux_gate = next(gate for gate in status.packaging_gates if gate.id == "linux_package_choice")
        self.assertEqual("wsl-deb-rpm-extract-command-smoke-verified", linux_gate.status)
        self.assertIn("real Linux desktop", linux_gate.next_evidence)
        self.assertIn("Linux desktop GUI package install/launch", linux_gate.forbidden_claim)
        self.assertNotIn(
            "Run the unsigned NSIS installer in a clean Windows VM for installer/runtime launch evidence",
            status.user_assistance_needed,
        )
        self.assertIn("Linux VM/container", " ".join(status.user_assistance_needed))

    def test_linux_wsl_deb_package_gate_marks_package_build_not_gui_completion(self) -> None:
        evidence = status_mod.smoke_garnet_studio_linux_wsl_deb.LinuxWslDebEvidence(
            status="verified",
            verified=True,
            reason="WSL Linux Tauri .deb package build and non-GUI studio-smoke verified.",
            bundle="proofs/linux/execution/studio-package/linux-wsl-deb-test",
            deferred=[
                "not Linux desktop GUI launch proof",
                "not Linux seccomp or OS-sandbox enforcement",
            ],
        )
        install_evidence = status_mod.smoke_garnet_studio_linux_wsl_deb_install.LinuxWslDebInstallEvidence(
            status="verified",
            verified=True,
            reason="WSL Linux Tauri .deb extract and extracted-binary non-GUI studio-smoke verified.",
            bundle="proofs/linux/execution/studio-package-install/linux-wsl-deb-install-test",
            deferred=[
                "not Linux desktop GUI launch proof",
                "not clean Linux install proof",
            ],
        )
        rpm_missing = status_mod.smoke_garnet_studio_linux_wsl_rpm.LinuxWslRpmEvidence(
            status="missing",
            verified=False,
            reason="No committed WSL Linux Tauri .rpm package proof bundle verified.",
            bundle=None,
            deferred=["record WSL Linux `.rpm` proof bundle"],
        )
        xvfb_missing = status_mod.smoke_garnet_studio_linux_wsl_xvfb.LinuxWslXvfbEvidence(
            status="missing",
            verified=False,
            reason="No committed WSL Linux Xvfb runtime-start proof bundle verified.",
            bundle=None,
            deferred=["record WSL Linux Xvfb proof bundle"],
        )
        xvfb_window_missing = status_mod.smoke_garnet_studio_linux_wsl_xvfb_window.LinuxWslXvfbWindowEvidence(
            status="missing",
            verified=False,
            reason="No committed WSL Linux Xvfb virtual-display window-capture proof bundle verified.",
            bundle=None,
            deferred=["record WSL Linux Xvfb window-capture proof bundle"],
        )
        with mock.patch.object(
            status_mod.smoke_garnet_studio_linux_wsl_deb,
            "read_committed_evidence",
            return_value=evidence,
        ), mock.patch.object(
            status_mod.smoke_garnet_studio_linux_wsl_deb_install,
            "read_committed_evidence",
            return_value=install_evidence,
        ), mock.patch.object(
            status_mod.smoke_garnet_studio_linux_wsl_rpm,
            "read_committed_evidence",
            return_value=rpm_missing,
        ), mock.patch.object(
            status_mod.smoke_garnet_studio_linux_wsl_xvfb,
            "read_committed_evidence",
            return_value=xvfb_missing,
        ), mock.patch.object(
            status_mod.smoke_garnet_studio_linux_wsl_xvfb_window,
            "read_committed_evidence",
            return_value=xvfb_window_missing,
        ):
            status = status_mod.read_status()

        linux_gate = next(gate for gate in status.packaging_gates if gate.id == "linux_package_choice")
        self.assertEqual("wsl-deb-extract-command-smoke-verified", linux_gate.status)
        self.assertIn("Linux desktop GUI", linux_gate.forbidden_claim)
        self.assertIn("WSL Linux `.deb` package build", " ".join(status.current_truth))
        self.assertIn("WSL Linux `.deb` package extract", " ".join(status.current_truth))
        self.assertIn("Linux desktop GUI", " ".join(status.next_slices))
        self.assertIn("Linux VM/container", " ".join(status.user_assistance_needed))

    def test_linux_wsl_rpm_package_gate_marks_extract_not_gui_completion(self) -> None:
        evidence = status_mod.smoke_garnet_studio_linux_wsl_deb.LinuxWslDebEvidence(
            status="verified",
            verified=True,
            reason="WSL Linux Tauri .deb package build and non-GUI studio-smoke verified.",
            bundle="proofs/linux/execution/studio-package/linux-wsl-deb-test",
            deferred=["not Linux desktop GUI launch proof"],
        )
        install_evidence = status_mod.smoke_garnet_studio_linux_wsl_deb_install.LinuxWslDebInstallEvidence(
            status="verified",
            verified=True,
            reason="WSL Linux Tauri .deb extract and extracted-binary non-GUI studio-smoke verified.",
            bundle="proofs/linux/execution/studio-package-install/linux-wsl-deb-install-test",
            deferred=["not Linux desktop GUI launch proof"],
        )
        rpm_evidence = status_mod.smoke_garnet_studio_linux_wsl_rpm.LinuxWslRpmEvidence(
            status="verified",
            verified=True,
            reason="WSL Linux Tauri .rpm extract and extracted-binary non-GUI studio-smoke verified.",
            bundle="proofs/linux/execution/studio-rpm-package/linux-wsl-rpm-test",
            deferred=[
                "not Linux desktop GUI launch proof",
                "not clean Linux install proof",
            ],
        )
        xvfb_missing = status_mod.smoke_garnet_studio_linux_wsl_xvfb.LinuxWslXvfbEvidence(
            status="missing",
            verified=False,
            reason="No committed WSL Linux Xvfb runtime-start proof bundle verified.",
            bundle=None,
            deferred=["record WSL Linux Xvfb proof bundle"],
        )
        xvfb_window_missing = status_mod.smoke_garnet_studio_linux_wsl_xvfb_window.LinuxWslXvfbWindowEvidence(
            status="missing",
            verified=False,
            reason="No committed WSL Linux Xvfb virtual-display window-capture proof bundle verified.",
            bundle=None,
            deferred=["record WSL Linux Xvfb window-capture proof bundle"],
        )
        with mock.patch.object(
            status_mod.smoke_garnet_studio_linux_wsl_deb,
            "read_committed_evidence",
            return_value=evidence,
        ), mock.patch.object(
            status_mod.smoke_garnet_studio_linux_wsl_deb_install,
            "read_committed_evidence",
            return_value=install_evidence,
        ), mock.patch.object(
            status_mod.smoke_garnet_studio_linux_wsl_rpm,
            "read_committed_evidence",
            return_value=rpm_evidence,
        ), mock.patch.object(
            status_mod.smoke_garnet_studio_linux_wsl_xvfb,
            "read_committed_evidence",
            return_value=xvfb_missing,
        ), mock.patch.object(
            status_mod.smoke_garnet_studio_linux_wsl_xvfb_window,
            "read_committed_evidence",
            return_value=xvfb_window_missing,
        ):
            status = status_mod.read_status()

        linux_gate = next(gate for gate in status.packaging_gates if gate.id == "linux_package_choice")
        self.assertEqual("wsl-deb-rpm-extract-command-smoke-verified", linux_gate.status)
        self.assertIn("real Linux desktop", linux_gate.next_evidence)
        self.assertIn("Linux desktop GUI", linux_gate.forbidden_claim)
        truth = " ".join(status.current_truth)
        self.assertIn("WSL Linux `.rpm` package extract", truth)
        self.assertIn("not complete", truth)

    def test_linux_wsl_xvfb_runtime_gate_marks_runtime_start_not_gui_completion(self) -> None:
        evidence = status_mod.smoke_garnet_studio_linux_wsl_deb.LinuxWslDebEvidence(
            status="verified",
            verified=True,
            reason="WSL Linux Tauri .deb package build and non-GUI studio-smoke verified.",
            bundle="proofs/linux/execution/studio-package/linux-wsl-deb-test",
            deferred=["not Linux desktop GUI launch proof"],
        )
        install_evidence = status_mod.smoke_garnet_studio_linux_wsl_deb_install.LinuxWslDebInstallEvidence(
            status="verified",
            verified=True,
            reason="WSL Linux Tauri .deb extract and extracted-binary non-GUI studio-smoke verified.",
            bundle="proofs/linux/execution/studio-package-install/linux-wsl-deb-install-test",
            deferred=["not Linux desktop GUI launch proof"],
        )
        rpm_evidence = status_mod.smoke_garnet_studio_linux_wsl_rpm.LinuxWslRpmEvidence(
            status="verified",
            verified=True,
            reason="WSL Linux Tauri .rpm extract and extracted-binary non-GUI studio-smoke verified.",
            bundle="proofs/linux/execution/studio-rpm-package/linux-wsl-rpm-test",
            deferred=["not Linux desktop GUI launch proof"],
        )
        xvfb_evidence = status_mod.smoke_garnet_studio_linux_wsl_xvfb.LinuxWslXvfbEvidence(
            status="verified",
            verified=True,
            reason="WSL Xvfb runtime-start verified with timeout exit 124.",
            bundle="proofs/linux/execution/studio-xvfb-runtime/linux-wsl-xvfb-test",
            deferred=[
                "not Linux desktop GUI launch proof",
                "not Linux seccomp or OS-sandbox enforcement",
            ],
        )
        xvfb_window_missing = status_mod.smoke_garnet_studio_linux_wsl_xvfb_window.LinuxWslXvfbWindowEvidence(
            status="missing",
            verified=False,
            reason="No committed WSL Linux Xvfb virtual-display window-capture proof bundle verified.",
            bundle=None,
            deferred=["record WSL Linux Xvfb window-capture proof bundle"],
        )
        with mock.patch.object(
            status_mod.smoke_garnet_studio_linux_wsl_deb,
            "read_committed_evidence",
            return_value=evidence,
        ), mock.patch.object(
            status_mod.smoke_garnet_studio_linux_wsl_deb_install,
            "read_committed_evidence",
            return_value=install_evidence,
        ), mock.patch.object(
            status_mod.smoke_garnet_studio_linux_wsl_rpm,
            "read_committed_evidence",
            return_value=rpm_evidence,
        ), mock.patch.object(
            status_mod.smoke_garnet_studio_linux_wsl_xvfb,
            "read_committed_evidence",
            return_value=xvfb_evidence,
        ), mock.patch.object(
            status_mod.smoke_garnet_studio_linux_wsl_xvfb_window,
            "read_committed_evidence",
            return_value=xvfb_window_missing,
        ):
            status = status_mod.read_status()

        linux_gate = next(gate for gate in status.packaging_gates if gate.id == "linux_package_choice")
        self.assertEqual("wsl-deb-rpm-xvfb-runtime-start-verified", linux_gate.status)
        self.assertIn("real Linux desktop", linux_gate.next_evidence)
        self.assertIn("Linux desktop GUI", linux_gate.forbidden_claim)
        truth = " ".join(status.current_truth)
        self.assertIn("WSL Linux Xvfb runtime-start", truth)
        self.assertIn("not Linux desktop GUI launch proof", truth)
        self.assertIn("Linux VM/container", " ".join(status.user_assistance_needed))

    def test_linux_wsl_xvfb_window_capture_gate_marks_virtual_display_not_desktop_completion(self) -> None:
        evidence = status_mod.smoke_garnet_studio_linux_wsl_deb.LinuxWslDebEvidence(
            status="verified",
            verified=True,
            reason="WSL Linux Tauri .deb package build and non-GUI studio-smoke verified.",
            bundle="proofs/linux/execution/studio-package/linux-wsl-deb-test",
            deferred=["not Linux desktop GUI launch proof"],
        )
        install_evidence = status_mod.smoke_garnet_studio_linux_wsl_deb_install.LinuxWslDebInstallEvidence(
            status="verified",
            verified=True,
            reason="WSL Linux Tauri .deb extract and extracted-binary non-GUI studio-smoke verified.",
            bundle="proofs/linux/execution/studio-package-install/linux-wsl-deb-install-test",
            deferred=["not Linux desktop GUI launch proof"],
        )
        rpm_evidence = status_mod.smoke_garnet_studio_linux_wsl_rpm.LinuxWslRpmEvidence(
            status="verified",
            verified=True,
            reason="WSL Linux Tauri .rpm extract and extracted-binary non-GUI studio-smoke verified.",
            bundle="proofs/linux/execution/studio-rpm-package/linux-wsl-rpm-test",
            deferred=["not Linux desktop GUI launch proof"],
        )
        xvfb_evidence = status_mod.smoke_garnet_studio_linux_wsl_xvfb.LinuxWslXvfbEvidence(
            status="verified",
            verified=True,
            reason="WSL Xvfb runtime-start verified with timeout exit 124.",
            bundle="proofs/linux/execution/studio-xvfb-runtime/linux-wsl-xvfb-test",
            deferred=["not Linux desktop GUI launch proof"],
        )
        window_evidence = status_mod.smoke_garnet_studio_linux_wsl_xvfb_window.LinuxWslXvfbWindowEvidence(
            status="verified",
            verified=True,
            reason="WSL Xvfb virtual-display window capture verified with screenshot and xwininfo.",
            bundle="proofs/linux/execution/studio-xvfb-window-capture/linux-wsl-xvfb-window-test",
            deferred=[
                "not Linux desktop GUI launch proof",
                "not Linux seccomp or OS-sandbox enforcement",
            ],
        )
        with mock.patch.object(
            status_mod.smoke_garnet_studio_linux_wsl_deb,
            "read_committed_evidence",
            return_value=evidence,
        ), mock.patch.object(
            status_mod.smoke_garnet_studio_linux_wsl_deb_install,
            "read_committed_evidence",
            return_value=install_evidence,
        ), mock.patch.object(
            status_mod.smoke_garnet_studio_linux_wsl_rpm,
            "read_committed_evidence",
            return_value=rpm_evidence,
        ), mock.patch.object(
            status_mod.smoke_garnet_studio_linux_wsl_xvfb,
            "read_committed_evidence",
            return_value=xvfb_evidence,
        ), mock.patch.object(
            status_mod.smoke_garnet_studio_linux_wsl_xvfb_window,
            "read_committed_evidence",
            return_value=window_evidence,
        ):
            status = status_mod.read_status()

        linux_gate = next(gate for gate in status.packaging_gates if gate.id == "linux_package_choice")
        self.assertEqual("wsl-deb-rpm-xvfb-window-capture-verified", linux_gate.status)
        self.assertIn("real Linux desktop", linux_gate.next_evidence)
        self.assertIn("Linux desktop GUI", linux_gate.forbidden_claim)
        truth = " ".join(status.current_truth)
        self.assertIn("WSL Linux Xvfb virtual-display window capture", truth)
        self.assertIn("not Linux desktop GUI launch proof", truth)
        self.assertIn("Linux VM/container", " ".join(status.user_assistance_needed))

    def test_json_and_markdown_preserve_not_completed_boundary(self) -> None:
        output = subprocess.check_output(
            [sys.executable, str(SCRIPT), "--format", "json"],
            text=True,
        )
        data = json.loads(output)
        self.assertEqual(
            "tauri-v2-shell-v0-5-readiness-parity-windows-clean-vm-contract-open-wsl-deb-rpm-xvfb-window-capture-verified-linux-desktop-still-open",
            data["status"],
        )
        truth = " ".join(data["current_truth"])
        self.assertIn("Tauri v2 is now adopted", truth)
        self.assertIn("v0.5 reporters", truth)
        self.assertIn("repo-owned evidence contract", truth)
        self.assertIn("WSL Linux `.deb` package build", truth)
        self.assertIn("WSL Linux `.deb` package extract", truth)
        self.assertIn("WSL Linux `.rpm` package extract", truth)
        self.assertIn("WSL Linux Xvfb runtime-start", truth)
        self.assertIn("WSL Linux Xvfb virtual-display window capture", truth)
        self.assertIn("Windows ARM64 follows after x64 proof", truth)
        self.assertIn("Domain Proof Matrix", truth)
        self.assertIn("Linux runtime proof is not complete", " ".join(data["current_truth"]))
        self.assertFalse(data["safety_contract"]["calls_provider_apis_by_default"])
        self.assertFalse(data["safety_contract"]["includes_source_by_default"])

        markdown = subprocess.check_output([sys.executable, str(SCRIPT)], text=True)
        self.assertIn("Garnet Windows/Linux Studio Status", markdown)
        self.assertIn("Tauri v2 is accepted", markdown)
        self.assertIn("MIT Deck Preview", markdown)
        self.assertIn("windows_target_architecture_matrix", markdown)
        self.assertIn("signed Windows MSI for Studio", json.dumps(data["safety_contract"]["forbidden_claims"]))


if __name__ == "__main__":
    unittest.main()
