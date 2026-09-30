#!/usr/bin/env python3
"""Regression tests for the agentic dogfood matrix inventory."""
from __future__ import annotations

import ast
import importlib.util
import json
import os
import shlex
import shutil
import subprocess
import sys
import tempfile
import unittest
from collections import Counter
from pathlib import Path
from textwrap import dedent
from unittest import mock

SCRIPT = Path(__file__).with_name("run_agentic_dogfood_matrix.py")
SPEC = importlib.util.spec_from_file_location("run_agentic_dogfood_matrix", SCRIPT)
assert SPEC is not None
matrix = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["run_agentic_dogfood_matrix"] = matrix
SPEC.loader.exec_module(matrix)


class AgenticDogfoodMatrixTests(unittest.TestCase):
    def test_default_garnet_binary_name_is_platform_aware(self) -> None:
        expected = "garnet.exe" if sys.platform.startswith("win") else "garnet"
        self.assertEqual(expected, matrix.executable_name("garnet"))

    def _fake_garnet_path(self) -> Path:
        return Path(sys.executable)

    def _fake_result(self, probe: object) -> object:
        return matrix.ProbeResult(
            probe=probe,
            status="passed",
            exit_code=0,
            duration_ms=1,
            stdout_log="/tmp/stdout.log",
            stderr_log="/tmp/stderr.log",
            stdout_excerpt="",
            stderr_excerpt="",
        )

    def _inventory_results(self, probes: list[object]) -> list[object]:
        results = []
        original_run = matrix.run

        def fake_run(cmd: list[str], cwd: Path, timeout: int = 120, env: dict[str, str] | None = None) -> object:
            return subprocess.CompletedProcess(cmd, 0, "inventory probe stub\n", "")

        for probe in probes:
            if isinstance(probe, matrix.Probe):
                results.append(self._fake_result(probe))
            else:
                matrix.run = fake_run
                try:
                    results.append(probe())
                finally:
                    matrix.run = original_run
        return results

    def test_probe_inventory_includes_agent_recovery_domain(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["agent recovery and diagnostics"], 4)
        self.assertIn("report-converter-adoption-status", ids)
        self.assertIn("check-malformed-agent-source", ids)
        self.assertIn("check-missing-agent-source", ids)
        self.assertIn("eval-unknown-agent-symbol", ids)
        self.assertIn("verify-missing-release-manifest", ids)

    def test_converter_status_probe_guards_intelligent_assist_contract(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)

        probe = next(
            probe
            for probe in probes
            if isinstance(probe, matrix.Probe)
            and probe.id == "report-converter-adoption-status"
        )

        self.assertIn("planned-contract", probe.expected_stdout)
        self.assertIn("CapCaps/capability boundaries", probe.expected_stdout)
        self.assertIn("provider_required", probe.expected_stdout)

    def test_probe_inventory_includes_mit_readiness_accounting(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["MIT readiness accounting"], 3)
        self.assertIn("report-mit-readiness-plan-complete", ids)
        self.assertIn("report-mit-readiness-open-productization", ids)
        self.assertIn("report-mit-readiness-assist-and-frontends", ids)

    def test_probe_inventory_includes_mac_side_continuation_accounting(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["Mac-side continuation accounting"], 1)
        self.assertIn("report-mac-side-continuation-boundaries", ids)

    def test_probe_inventory_includes_github_actions_node24_readiness(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["CI action runtime"], 1)
        self.assertIn("report-github-actions-node24-readiness", ids)
        probe = next(result.probe for result in results if result.probe.id == "report-github-actions-node24-readiness")
        self.assertIn("Ran 3 tests", probe.expected_stderr)

    def test_probe_inventory_includes_proof_benchmark_empirics(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["proof benchmark empirics"], 2)
        self.assertIn("report-proof-benchmark-status", ids)
        self.assertIn("report-benchmark-no-run-status", ids)

    def test_probe_inventory_includes_mit_demo_route(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["MIT demo route"], 3)
        self.assertIn("report-mit-demo-route-current-truth", ids)
        self.assertIn("report-mit-demo-route-blocked-gates", ids)
        self.assertIn("report-mit-demo-route-output-manifest", ids)

    def test_probe_inventory_includes_mit_deck_outline(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["MIT deck outline"], 3)
        self.assertIn("report-mit-deck-outline-current-truth", ids)
        self.assertIn("report-mit-deck-outline-blocked-gates", ids)
        self.assertIn("report-mit-deck-outline-output-manifest", ids)

    def test_probe_inventory_includes_promo_video_readiness_contract(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["promo video readiness"], 9)
        self.assertIn("report-promo-video-current-truth", ids)
        self.assertIn("report-promo-video-required-gates", ids)
        self.assertIn("report-promo-video-source-lock", ids)
        self.assertIn("report-promo-video-composition-source", ids)
        self.assertIn("report-promo-video-render-harness-contract", ids)
        self.assertIn("report-promo-video-visual-qa-harness-contract", ids)
        self.assertIn("report-promo-video-website-export-harness-contract", ids)
        self.assertIn("report-promo-video-site-sync-harness-contract", ids)
        self.assertIn("report-promo-video-output-manifest", ids)

    def test_probe_inventory_includes_repo_site_adoption_surface(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["repo/site adoption surface"], 3)
        self.assertIn("report-adoption-surface-active-truth", ids)
        self.assertIn("report-adoption-surface-planned-frontends", ids)
        self.assertIn("report-adoption-surface-use-cases", ids)

    def test_probe_inventory_includes_assist_context_pack(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["converter intelligent assist"], 4)
        self.assertIn("report-assist-context-current-truth", ids)
        self.assertIn("report-assist-context-required-gates", ids)
        self.assertIn("report-assist-context-documents", ids)
        self.assertIn("report-assist-context-prompt-pack", ids)

    def test_probe_inventory_includes_converter_assist_plan(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["converter assist planning"], 10)
        self.assertIn("report-assist-plan-typescript-current-truth", ids)
        self.assertIn("report-assist-plan-typescript-risks", ids)
        self.assertIn("report-assist-plan-javascript-risks", ids)
        self.assertIn("report-assist-plan-swift-risks", ids)
        self.assertIn("report-assist-plan-java-risks", ids)
        self.assertIn("report-assist-plan-c-risks", ids)
        self.assertIn("report-assist-plan-cpp-risks", ids)
        self.assertIn("report-assist-plan-csharp-risks", ids)
        self.assertIn("report-assist-plan-perl-risks", ids)
        self.assertIn("report-assist-plan-output-manifest", ids)

    def test_probe_inventory_includes_converter_llm_feasibility_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["converter LLM feasibility"], 5)
        self.assertIn("report-converter-llm-feasibility-current-truth", ids)
        self.assertIn("report-converter-llm-feasibility-language-coverage", ids)
        self.assertIn("report-converter-llm-feasibility-provider-options", ids)
        self.assertIn("report-converter-llm-feasibility-blockers", ids)
        self.assertIn("report-converter-llm-feasibility-output-manifest", ids)

    def test_probe_inventory_includes_converter_provider_options_studio_ux(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["converter provider options UX"], 3)
        self.assertIn("report-studio-provider-options-action", ids)
        self.assertIn("report-studio-provider-options-runner", ids)
        self.assertIn("report-studio-provider-options-desktop-evidence", ids)

    def test_probe_inventory_includes_windows_linux_studio_shell(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["Windows/Linux Studio shell"], 5)
        self.assertIn("report-windows-linux-studio-tauri-scaffold", ids)
        self.assertIn("report-windows-linux-studio-command-contract", ids)
        self.assertIn("report-windows-linux-studio-v05-readiness-parity", ids)
        self.assertIn("report-windows-linux-studio-advisory-boundary", ids)
        self.assertIn("report-windows-linux-studio-evidence-smoke", ids)

    def test_probe_inventory_includes_windows_clean_vm_installer_proof(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["Windows clean VM installer proof"], 3)
        self.assertIn("report-windows-clean-vm-installer-status-script", ids)
        self.assertIn("report-windows-clean-vm-proof-boundary", ids)
        self.assertIn("report-windows-clean-vm-studio-action", ids)

    def _clean_vm_status_probe_result(
        self, status: dict[str, object], env: dict[str, str] | None = None
    ) -> subprocess.CompletedProcess[str]:
        """Run the clean-VM status probe against a stand-in reporter that prints `status`."""
        real_script = str(matrix.ROOT / "scripts" / "garnet_windows_clean_vm_installer_status.py")
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            probe = next(
                probe
                for probe in probes
                if isinstance(probe, matrix.Probe) and probe.id == "report-windows-clean-vm-installer-status-script"
            )
            fake = work / "fake_clean_vm_status.py"
            fake.write_text(f"import json\nprint(json.dumps({status!r}))\n", encoding="utf-8")
            code = probe.command[-1]
            self.assertIn(repr(real_script), code)
            command = [*probe.command[:-1], code.replace(repr(real_script), repr(str(fake)))]
            return subprocess.run(command, capture_output=True, text=True, timeout=60, env={**os.environ, **(env or {})})

    def _clean_vm_status(self, **changes: object) -> dict[str, object]:
        status: dict[str, object] = {
            "status": "clean-vm-proof-verified",
            "clean_vm_verified": True,
            "proof_source": "committed:proofs/windows/studio-clean-vm/20260926-0349-NUCBOX_M2PRO_S",
            "package_targets": [
                {"id": "studio-windows-x64-nsis", "rust_target": "x86_64-pc-windows-msvc", "status": "first-clean-vm-target"},
                {"id": "studio-windows-arm64-nsis", "rust_target": "aarch64-pc-windows-msvc", "status": "planned-after-x64-proof"},
                {"id": "studio-windows-x86-nsis", "rust_target": "i686-pc-windows-msvc", "status": "deferred-until-user-demand"},
            ],
        }
        status.update(changes)
        return status

    def test_clean_vm_status_probe_passes_only_on_the_reviewed_committed_bundle(self) -> None:
        result = self._clean_vm_status_probe_result(self._clean_vm_status())
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertIn("windows clean vm installer status contract present", result.stdout)

    def test_clean_vm_status_probe_fails_when_the_proof_is_missing_or_unverified(self) -> None:
        for label, changes in (
            ("no proof", {"status": "proof-contract-ready-clean-vm-open", "clean_vm_verified": False, "proof_source": "none"}),
            (
                "failing committed bundle",
                {
                    "status": "proof-contract-ready-clean-vm-open",
                    "clean_vm_verified": False,
                    "proof_source": "committed:proofs/windows/studio-clean-vm/20260926-0349-NUCBOX_M2PRO_S",
                },
            ),
            ("verified flag but open status", {"status": "proof-contract-ready-clean-vm-open"}),
            ("truthy non-boolean flag", {"clean_vm_verified": "true"}),
        ):
            with self.subTest(label):
                self.assertNotEqual(0, self._clean_vm_status_probe_result(self._clean_vm_status(**changes)).returncode)

    def test_clean_vm_status_probe_fails_for_any_other_proof_source(self) -> None:
        for source in (
            "local:/home/runner/Desktop/dogfood/garnet-studio-windows-clean-vm",
            "committed:proofs/windows/studio-clean-vm/20270101-0000-NEWER_HOST",
            "committed:proofs/windows/studio-clean-vm",
        ):
            with self.subTest(source):
                result = self._clean_vm_status_probe_result(self._clean_vm_status(proof_source=source))
                self.assertNotEqual(0, result.returncode)

    def test_clean_vm_status_probe_holds_under_python_optimization(self) -> None:
        # PYTHONOPTIMIZE strips `assert`; the matrix runner passes the ambient environment through.
        optimized = {"PYTHONOPTIMIZE": "1"}
        unverified = self._clean_vm_status(
            status="proof-contract-ready-clean-vm-open", clean_vm_verified=False, proof_source="none"
        )
        self.assertNotEqual(0, self._clean_vm_status_probe_result(unverified, optimized).returncode)
        other = self._clean_vm_status(proof_source="committed:proofs/windows/studio-clean-vm/20270101-0000-NEWER_HOST")
        self.assertNotEqual(0, self._clean_vm_status_probe_result(other, optimized).returncode)
        reviewed = self._clean_vm_status_probe_result(self._clean_vm_status(), optimized)
        self.assertEqual(0, reviewed.returncode, reviewed.stderr)

    def test_probe_inventory_includes_converter_advisory_bundle(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["converter advisory bundle"], 4)
        self.assertIn("report-converter-advisory-bundle-current-truth", ids)
        self.assertIn("report-converter-advisory-bundle-omits-source", ids)
        self.assertIn("report-converter-advisory-bundle-include-source-gate", ids)
        self.assertIn("report-converter-advisory-bundle-output-manifest", ids)

    def test_probe_inventory_includes_converter_advisory_bundle_studio_ux(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["converter advisory bundle UX"], 3)
        self.assertIn("report-studio-advisory-bundle-action", ids)
        self.assertIn("report-studio-advisory-bundle-runner", ids)
        self.assertIn("report-studio-advisory-bundle-default-privacy", ids)

    def test_probe_inventory_includes_converter_advisory_review_studio_ux(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["converter advisory review UX"], 3)
        self.assertIn("report-studio-advisory-review-action", ids)
        self.assertIn("report-studio-advisory-review-runner", ids)
        self.assertIn("report-studio-advisory-review-desktop-evidence", ids)

    def test_probe_inventory_includes_converter_advisory_handoff_studio_ux(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["converter advisory handoff UX"], 3)
        self.assertIn("report-studio-advisory-handoff-action", ids)
        self.assertIn("report-studio-advisory-handoff-runner", ids)
        self.assertIn("report-studio-advisory-handoff-desktop-evidence", ids)

    def test_probe_inventory_includes_studio_objective_pulse_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["MIT objective pulse UX"], 3)
        self.assertIn("report-studio-objective-pulse-action", ids)
        self.assertIn("report-studio-objective-pulse-runner", ids)
        self.assertIn("report-studio-objective-pulse-truth-copy", ids)

    def test_probe_inventory_includes_studio_mac_continuation_pulse_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["Mac continuation pulse UX"], 3)
        self.assertIn("report-studio-mac-continuation-pulse-action", ids)
        self.assertIn("report-studio-mac-continuation-pulse-runner", ids)
        self.assertIn("report-studio-mac-continuation-pulse-truth-copy", ids)

    def test_probe_inventory_includes_studio_mit_demo_route_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["MIT demo route UX"], 3)
        self.assertIn("report-studio-mit-demo-route-action", ids)
        self.assertIn("report-studio-mit-demo-route-runner", ids)
        self.assertIn("report-studio-mit-demo-route-desktop-evidence", ids)

    def test_probe_inventory_includes_studio_mit_deck_outline_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["MIT deck outline UX"], 3)
        self.assertIn("report-studio-mit-deck-outline-action", ids)
        self.assertIn("report-studio-mit-deck-outline-runner", ids)
        self.assertIn("report-studio-mit-deck-outline-desktop-evidence", ids)

    def test_probe_inventory_includes_mit_deck_preview_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["MIT deck preview"], 4)
        self.assertIn("report-mit-deck-preview-current-truth", ids)
        self.assertIn("report-mit-deck-preview-html", ids)
        self.assertIn("report-mit-deck-preview-output-contract", ids)
        self.assertIn("report-mit-deck-preview-browser-smoke-harness", ids)

    def test_probe_inventory_includes_studio_mit_deck_preview_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["MIT deck preview UX"], 3)
        self.assertIn("report-studio-mit-deck-preview-action", ids)
        self.assertIn("report-studio-mit-deck-preview-runner", ids)
        self.assertIn("report-studio-mit-deck-preview-desktop-evidence", ids)

    def test_probe_inventory_includes_studio_mit_deck_preview_smoke_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["MIT deck preview smoke"], 5)
        self.assertIn("report-studio-mit-deck-preview-smoke-command", ids)
        self.assertIn("report-studio-mit-deck-preview-smoke-output-contract", ids)
        self.assertIn("report-studio-mit-deck-preview-smoke-manifest-verification", ids)
        self.assertIn("report-studio-mit-deck-preview-dmg-smoke", ids)
        self.assertIn("report-studio-mit-deck-preview-dmg-manifest-log", ids)

    def test_probe_inventory_includes_converter_advisory_review_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["converter advisory review"], 3)
        self.assertIn("report-converter-advisory-review-current-truth", ids)
        self.assertIn("report-converter-advisory-review-source-included-block", ids)
        self.assertIn("report-converter-advisory-review-output-manifest", ids)

    def test_probe_inventory_includes_converter_advisory_handoff_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["converter advisory handoff"], 3)
        self.assertIn("report-converter-advisory-handoff-current-truth", ids)
        self.assertIn("report-converter-advisory-handoff-source-included-block", ids)
        self.assertIn("report-converter-advisory-handoff-output-manifest", ids)

    def test_probe_inventory_includes_web_pwa_offline_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["web/PWA productization"], 3)
        self.assertIn("smoke-web-pwa-offline-handler", ids)
        self.assertIn("smoke-web-pwa-local-readiness", ids)
        self.assertIn("smoke-web-pwa-browser-offline", ids)

    def test_probe_inventory_includes_signed_release_provenance(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["signed release provenance"], 3)
        self.assertIn("release-keygen-build-verify-signature", ids)
        self.assertIn("release-unsigned-manifest-requires-signature", ids)
        self.assertIn("release-signed-manifest-tamper-detection", ids)

    def test_probe_inventory_includes_macos_notarization_readiness(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["macOS notarization readiness"], 3)
        self.assertIn("report-notarization-status-blockers", ids)
        self.assertIn("report-notarization-status-redaction", ids)
        self.assertIn("report-notarization-status-missing-bundle", ids)

    def test_signed_release_probe_does_not_persist_generated_private_keys(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            if sys.platform == "win32":
                fake_garnet = work / "fake-garnet.cmd"
                fake_garnet.write_text(
                    dedent(
                        """\
                        @echo off
                        if "%1"=="keygen" (
                          echo private test key>"%2"
                          echo generated Ed25519 signing keypair
                          exit /b 0
                        )
                        if "%1"=="build" (
                          echo {"signature":"test","signer_pubkey":"test"}>"%5.manifest.json"
                          echo signed_by test
                          exit /b 0
                        )
                        if "%1"=="verify" (
                          echo signature valid
                          exit /b 0
                        )
                        exit /b 1
                        """
                    ),
                    encoding="utf-8",
                )
            else:
                fake_garnet = work / "fake-garnet"
                fake_garnet.write_text(
                    dedent(
                        """\
                        #!/usr/bin/env sh
                        case "$1" in
                          keygen)
                            printf 'private test key\\n' > "$2"
                            echo 'generated Ed25519 signing keypair'
                            ;;
                          build)
                            source="${5:-$3}"
                            manifest="${source}.manifest.json"
                            printf '{"signature":"test","signer_pubkey":"test"}\\n' > "$manifest"
                            echo 'signed_by test'
                            ;;
                          verify)
                            echo 'signature valid'
                            ;;
                        esac
                        """
                    ),
                    encoding="utf-8",
                )
            fake_garnet.chmod(0o755)
            fixtures = matrix.prepare_fixtures(work)

            result = matrix.build_signed_release_probe(fake_garnet, work, fixtures["build_source"])

            self.assertTrue(result.passed)
            self.assertEqual([], list((work / "signed-release").glob("*.key")))

    def test_probe_inventory_covers_developer_experience_repair(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["developer experience"], 3)
        self.assertIn("fmt-repair-dirty-agent", ids)

    def test_probe_inventory_covers_memory_declaration_analysis(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["agent memory and analysis"], 3)
        self.assertIn("parse-advertised-log-analyzer-memory", ids)

    def test_probe_inventory_covers_signed_memory_persistence_integrity(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["memory persistence integrity"], 3)
        self.assertIn("memory-signed-cache-roundtrip", ids)
        self.assertIn("memory-signed-cache-tamper-rejection", ids)
        self.assertIn("memory-signed-cache-foreign-key-rejection", ids)

    def test_packaged_matrix_skips_source_workspace_memory_integrity_probes(self) -> None:
        original_root = matrix.ROOT
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp) / "dogfood"
            packaged_resources = Path(temp) / "Garnet Studio.app" / "Contents" / "Resources"
            packaged_resources.mkdir(parents=True)
            matrix.ROOT = packaged_resources
            try:
                probes = matrix.memory_persistence_integrity_probes(work)
                results = [probe() for probe in probes]
            finally:
                matrix.ROOT = original_root

        self.assertEqual([result.status for result in results], ["skipped", "skipped", "skipped"])
        self.assertTrue(all(result.passed for result in results))
        self.assertEqual(matrix.score(results)["skipped"], 3)
        self.assertIn("Cargo.toml", results[0].stdout_excerpt)
        self.assertIn("source workspace", results[0].probe.notes)

    def test_packaged_matrix_skips_source_workspace_benchmark_no_run_probe(self) -> None:
        original_root = matrix.ROOT
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp) / "dogfood"
            packaged_resources = Path(temp) / "Garnet Studio.app" / "Contents" / "Resources"
            packaged_resources.mkdir(parents=True)
            matrix.ROOT = packaged_resources
            try:
                probes = matrix.benchmark_no_run_probes(work)
                results = [probe() for probe in probes]
            finally:
                matrix.ROOT = original_root

        self.assertEqual([result.status for result in results], ["skipped"])
        self.assertTrue(results[0].passed)
        self.assertEqual(matrix.score(results)["skipped"], 1)
        self.assertIn("Cargo.toml", results[0].stdout_excerpt)
        self.assertIn("benchmark compile boundary", results[0].probe.notes)

    def test_probe_inventory_covers_agent_toolbelt_examples(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["agent toolbelt examples"], 5)
        self.assertIn("run-agent-toolbelt-triage-router", ids)
        self.assertIn("run-agent-toolbelt-capability-budget", ids)
        self.assertIn("run-agent-toolbelt-memory-recall", ids)
        self.assertIn("run-agent-toolbelt-release-gate", ids)
        self.assertIn("run-agent-toolbelt-repair-planner", ids)

    def test_probe_inventory_covers_agent_adversarial_boundaries(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["agent adversarial boundaries"], 3)
        self.assertIn("reject-agent-depth-budget-bomb", ids)
        self.assertIn("reject-agent-main-without-caps", ids)
        self.assertIn("reject-agent-safe-var-mutation", ids)

    def test_probe_inventory_covers_source_app_workbench_smoke(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            garnet = Path("/tmp/garnet-target/debug/garnet")
            probes = matrix.probe_set(garnet, work, fixtures, include_app_workbench=True)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["macOS app workbench"], 3)
        self.assertIn("app-self-test", ids)
        self.assertIn("app-xctest", ids)
        self.assertIn("app-smoke-test", ids)

    def test_probe_inventory_covers_canonical_project_templates(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["project scaffolding"], 3)
        self.assertIn("template-cli-run-and-test", ids)
        self.assertIn("template-web-api-run-and-test", ids)
        self.assertIn("template-agent-orchestrator-run-and-test", ids)

    def test_domain_coverage_marks_undercovered_agentic_surfaces(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(self._fake_garnet_path(), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        coverage = {item["domain"]: item for item in matrix.domain_coverage(results)}

        self.assertEqual(coverage["web/PWA productization"]["probe_count"], 3)
        self.assertEqual(coverage["web/PWA productization"]["target_probe_count"], 3)
        self.assertEqual(coverage["web/PWA productization"]["status"], "adequate")
        self.assertEqual(coverage["project scaffolding"]["status"], "adequate")
        self.assertEqual(coverage["developer experience"]["status"], "adequate")
        self.assertEqual(coverage["agent toolbelt examples"]["status"], "adequate")
        self.assertEqual(coverage["agent memory and analysis"]["status"], "adequate")
        self.assertEqual(coverage["memory persistence integrity"]["status"], "adequate")
        self.assertEqual(coverage["agent recovery and diagnostics"]["status"], "adequate")
        self.assertEqual(coverage["agent adversarial boundaries"]["status"], "adequate")
        self.assertEqual(coverage["MIT readiness accounting"]["status"], "adequate")
        self.assertEqual(coverage["converter intelligent assist"]["status"], "adequate")
        self.assertEqual(coverage["converter assist planning"]["status"], "adequate")
        self.assertEqual(coverage["converter LLM feasibility"]["status"], "adequate")
        self.assertEqual(coverage["converter advisory bundle"]["status"], "adequate")
        self.assertEqual(coverage["converter advisory review"]["status"], "adequate")
        self.assertEqual(coverage["converter advisory handoff"]["status"], "adequate")
        self.assertEqual(coverage["converter advisory bundle UX"]["status"], "adequate")
        self.assertEqual(coverage["converter advisory review UX"]["status"], "adequate")
        self.assertEqual(coverage["converter advisory handoff UX"]["status"], "adequate")
        self.assertEqual(coverage["MIT objective pulse UX"]["status"], "adequate")
        self.assertEqual(coverage["MIT demo route UX"]["status"], "adequate")
        self.assertEqual(coverage["MIT deck outline"]["status"], "adequate")
        self.assertEqual(coverage["MIT deck outline UX"]["status"], "adequate")
        self.assertEqual(coverage["MIT deck preview"]["status"], "adequate")
        self.assertEqual(coverage["MIT deck preview UX"]["status"], "adequate")
        self.assertEqual(coverage["MIT deck preview smoke"]["status"], "adequate")
        self.assertEqual(coverage["signed release provenance"]["status"], "adequate")
        self.assertEqual(coverage["macOS notarization readiness"]["status"], "adequate")

    def test_write_outputs_persists_domain_coverage(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            probe = matrix.Probe(
                "one-web-probe",
                "web/PWA productization",
                "one probe should still expose coverage debt",
                ["/bin/true"],
                True,
            )
            result = self._fake_result(probe)
            matrix.write_outputs(
                work,
                [result],
                {
                    "repo": "/tmp/repo",
                    "head": "abc123",
                    "branch": "test",
                    "garnet": "/bin/garnet",
                    "app_workbench": "skipped",
                    "artifact_dir": str(work),
                },
            )

            data = (work / "dogfood-readiness-data.json").read_text(encoding="utf-8")
            report = (work / "dogfood-readiness-report.md").read_text(encoding="utf-8")

        self.assertIn('"domain_coverage"', data)
        self.assertIn('"needs-expansion"', data)
        self.assertIn("## Domain Coverage Adequacy", report)
        self.assertIn("web/PWA productization", report)


class ChildInterpreterEnvironmentTests(unittest.TestCase):
    """The runner, not the caller's environment, decides how a probe's checks run."""

    def _run_command(
        self, command: list[str], marker: str, ambient: dict[str, str], env: dict[str, str] | None = None
    ) -> object:
        probe = matrix.Probe(
            "child-interpreter-environment",
            "runner isolation",
            "a probe's checks run as written, whatever the caller's environment holds",
            command,
            True,
            (marker,),
            env=env or {},
        )
        with tempfile.TemporaryDirectory() as temp, mock.patch.dict(os.environ, ambient):
            result = matrix.run_probe(probe, Path(temp))
            self.full_stdout = Path(result.stdout_log).read_text(encoding="utf-8")
            return result

    def _run_probe(self, code: str, marker: str, ambient: dict[str, str], env: dict[str, str] | None = None) -> object:
        return self._run_command([sys.executable, "-c", code], marker, ambient, env)

    def test_ambient_optimization_cannot_strip_a_failing_probe_check(self) -> None:
        failing = "assert 1 == 2, 'probe check failed'\nprint('probe check passed')\n"
        for level in ("1", "2"):
            with self.subTest(PYTHONOPTIMIZE=level):
                result = self._run_probe(failing, "probe check passed", {"PYTHONOPTIMIZE": level})
                self.assertEqual("failed", result.status, result.stdout_excerpt)
                self.assertIn("probe check failed", result.stderr_excerpt)
            with self.subTest(PYTHONOPTIMIZE=level, path="run() without a probe env, as the builder probes call it"):
                with tempfile.TemporaryDirectory() as temp, mock.patch.dict(os.environ, {"PYTHONOPTIMIZE": level}):
                    completed = matrix.run([sys.executable, "-c", failing], Path(temp))
                self.assertNotEqual(0, completed.returncode, completed.stdout)
                self.assertNotIn("probe check passed", completed.stdout)
        passing = "assert 1 == 1, 'probe check failed'\nprint('probe check passed')\n"
        self.assertEqual("passed", self._run_probe(passing, "probe check passed", {"PYTHONOPTIMIZE": "1"}).status)

    def test_ambient_optimization_does_not_reach_a_probes_own_python_child(self) -> None:
        code = (
            "import subprocess, sys\n"
            "rc = subprocess.run([sys.executable, '-c', 'assert 1 == 2']).returncode\n"
            "if rc == 0:\n"
            "    sys.exit('the grandchild assert was stripped')\n"
            "print('grandchild assert held')\n"
        )
        result = self._run_probe(code, "grandchild assert held", {"PYTHONOPTIMIZE": "1"})
        self.assertEqual("passed", result.status, result.stderr_excerpt)

    def test_pythonpath_shim_cannot_redirect_a_probe_import(self) -> None:
        with tempfile.TemporaryDirectory() as shim_temp:
            shim = Path(shim_temp)
            sentinel = shim / "shim-imported"
            (shim / "json.py").write_text(
                "from pathlib import Path\n"
                f"Path({str(sentinel)!r}).write_text('imported')\n"
                "def loads(text):\n"
                "    return {'verified': True}\n",
                encoding="utf-8",
            )
            code = (
                "import json, sys\n"
                "if json.loads('{\"verified\": false}')['verified'] is not True:\n"
                "    sys.exit('proof not verified')\n"
                "print('proof verified')\n"
            )
            result = self._run_probe(code, "proof verified", {"PYTHONPATH": str(shim)})
            self.assertEqual("failed", result.status, result.stdout_excerpt)
            self.assertIn("proof not verified", result.stderr_excerpt)
            self.assertFalse(sentinel.exists(), "the PYTHONPATH shim module was imported")

    def test_user_site_reached_through_home_cannot_inject_startup_code(self) -> None:
        with tempfile.TemporaryDirectory() as home_temp:
            home = Path(home_temp).resolve()
            ambient = {"HOME": str(home), "APPDATA": str(home / "AppData")}
            lookup_env = {name: value for name, value in os.environ.items() if not name.startswith("PYTHON")}
            lookup_env.update(ambient)
            enabled, user_site = subprocess.run(
                [sys.executable, "-c", "import site\nprint(site.ENABLE_USER_SITE)\nprint(site.getusersitepackages())\n"],
                capture_output=True,
                text=True,
                check=True,
                env=lookup_env,
            ).stdout.splitlines()
            user_site_path = Path(user_site).resolve()
            if enabled != "True" or home not in user_site_path.parents:
                self.skipTest(f"this interpreter's user site is not under HOME: {enabled} {user_site}")
            user_site_path.mkdir(parents=True)
            sentinel = home / "user-site-loaded"
            (user_site_path / "zz_garnet_matrix_shim.pth").write_text(
                f"import pathlib, sys; pathlib.Path({str(sentinel)!r}).write_text('loaded'); sys.exit = lambda *args: None\n",
                encoding="utf-8",
            )
            code = "import sys\nsys.exit('proof not verified')\nprint('proof verified')\n"
            result = self._run_probe(code, "proof verified", ambient)
            self.assertEqual("failed", result.status, result.stdout_excerpt)
            self.assertFalse(sentinel.exists(), "a .pth file in the HOME user site ran in the probe")

    # The 50 variables https://docs.python.org/3.14/using/cmdline.html documents (read 2026-09-30),
    # PYTHONTHREADDEBUG from earlier versions, and one undocumented name, so the prefix rule is tested.
    DOCUMENTED_PYTHON_VARIABLES = (
        "PYTHONASYNCIODEBUG", "PYTHONBREAKPOINT", "PYTHONCASEOK", "PYTHONCOERCECLOCALE", "PYTHONDEBUG",
        "PYTHONDEVMODE", "PYTHONDONTWRITEBYTECODE", "PYTHONDUMPREFS", "PYTHONDUMPREFSFILE",
        "PYTHONEXECUTABLE", "PYTHONFAULTHANDLER", "PYTHONHASHSEED", "PYTHONHOME", "PYTHONINSPECT",
        "PYTHONINTMAXSTRDIGITS", "PYTHONIOENCODING", "PYTHONLEGACYWINDOWSFSENCODING",
        "PYTHONLEGACYWINDOWSSTDIO", "PYTHONMALLOC", "PYTHONMALLOCSTATS", "PYTHONNODEBUGRANGES",
        "PYTHONNOUSERSITE", "PYTHONOPTIMIZE", "PYTHONPATH", "PYTHONPERFSUPPORT", "PYTHONPLATLIBDIR",
        "PYTHONPROFILEIMPORTTIME", "PYTHONPYCACHEPREFIX", "PYTHONSAFEPATH", "PYTHONSTARTUP",
        "PYTHONTRACEMALLOC", "PYTHONUNBUFFERED", "PYTHONUSERBASE", "PYTHONUTF8", "PYTHONVERBOSE",
        "PYTHONWARNDEFAULTENCODING", "PYTHONWARNINGS", "PYTHON_BASIC_REPL", "PYTHON_COLORS",
        "PYTHON_CONTEXT_AWARE_WARNINGS", "PYTHON_CPU_COUNT", "PYTHON_DISABLE_REMOTE_DEBUG",
        "PYTHON_FROZEN_MODULES", "PYTHON_GIL", "PYTHON_HISTORY", "PYTHON_JIT", "PYTHON_PERF_JIT_SUPPORT",
        "PYTHON_PRESITE", "PYTHON_THREAD_INHERIT_CONTEXT", "PYTHON_TLBC",
        "PYTHONTHREADDEBUG", "PYTHONGARNETMATRIXUNDOCUMENTED",
    )
    # The 24 NODE_* variables node's CLI documentation lists (doc/api/cli.md at v22.22.3 and main,
    # read 2026-09-30), plus one it does not document.
    NODE_VARIABLES = (
        "NODE_COMPILE_CACHE", "NODE_COMPILE_CACHE_PORTABLE", "NODE_COMPILE_CACHE_READONLY", "NODE_DEBUG",
        "NODE_DEBUG_NATIVE", "NODE_DISABLE_COLORS", "NODE_DISABLE_COMPILE_CACHE", "NODE_EXTRA_CA_CERTS",
        "NODE_ICU_DATA", "NODE_NO_WARNINGS", "NODE_OPTIONS", "NODE_PATH", "NODE_PENDING_DEPRECATION",
        "NODE_PENDING_PIPE_INSTANCES", "NODE_PRESERVE_SYMLINKS", "NODE_REDIRECT_WARNINGS",
        "NODE_REPL_EXTERNAL_MODULE", "NODE_REPL_HISTORY", "NODE_SKIP_PLATFORM_CHECK", "NODE_TEST_CONTEXT",
        "NODE_TLS_REJECT_UNAUTHORIZED", "NODE_USE_ENV_PROXY", "NODE_USE_SYSTEM_CA", "NODE_V8_COVERAGE",
        "NODE_GARNET_MATRIX_UNDOCUMENTED",
    )
    # What bash and sh read at startup: bash's own BASH* names (an exported function is
    # BASH_FUNC_<name>%%), plus one it does not define, and the unprefixed names bash's shell.c
    # reads before running a script (bash 3.2 and 5.x): ENV, SHELLOPTS, POSIXLY_CORRECT,
    # POSIX_PEDANTIC, SSH_CLIENT and SSH2_CLIENT.
    SHELL_VARIABLES = (
        "BASH_ENV", "BASHOPTS", "BASH_COMPAT", "BASH_XTRACEFD", "BASH_LOADABLES_PATH", "BASH_ARGV0",
        "BASH_FUNC_garnet_matrix_probe%%", "BASH_GARNET_MATRIX_UNDOCUMENTED",
        "ENV", "SHELLOPTS", "POSIXLY_CORRECT", "POSIX_PEDANTIC", "SSH_CLIENT", "SSH2_CLIENT",
    )

    def test_child_environment_carries_no_ambient_interpreter_variable(self) -> None:
        watched = (*self.DOCUMENTED_PYTHON_VARIABLES, *self.NODE_VARIABLES, *self.SHELL_VARIABLES)
        ambient = {name: "x" for name in watched}
        ambient["GARNET_MATRIX_UNRELATED"] = "kept"
        code = (
            "import json, os, sys\n"
            f"watched = set({sorted(watched)!r})\n"
            "seen = {name: value for name, value in os.environ.items()\n"
            "        if name in watched or name.startswith(('PYTHON', 'NODE_', 'BASH'))}\n"
            "print(json.dumps({'seen': seen, 'unrelated': os.environ.get('GARNET_MATRIX_UNRELATED'),"
            " 'optimize': sys.flags.optimize, 'no_user_site': sys.flags.no_user_site}, sort_keys=True))\n"
        )
        result = self._run_probe(code, '"unrelated": "kept"', ambient)
        self.assertEqual("passed", result.status, result.stderr_excerpt)
        data = json.loads(self.full_stdout.strip().splitlines()[-1])
        self.assertEqual(
            {"PYTHONNOUSERSITE": "1", "PYTHONUTF8": "1" if sys.flags.utf8_mode else "0"},
            data["seen"],
        )
        self.assertEqual(0, data["optimize"])
        self.assertEqual(1, data["no_user_site"])

    def test_child_encodes_pipes_in_the_runners_utf8_mode(self) -> None:
        opposite = "0" if sys.flags.utf8_mode else "1"
        code = "import sys\nprint('utf8_mode=%d' % sys.flags.utf8_mode)\n"
        result = self._run_probe(code, f"utf8_mode={sys.flags.utf8_mode}", {"PYTHONUTF8": opposite})
        self.assertEqual("passed", result.status, result.stdout_excerpt)

    def test_child_utf8_mode_follows_the_runner_in_either_mode(self) -> None:
        # The host decides the runner's own mode, so run the runner's module under both.
        code = (
            "import importlib.util, sys\n"
            f"spec = importlib.util.spec_from_file_location('run_agentic_dogfood_matrix', {str(SCRIPT)!r})\n"
            "module = importlib.util.module_from_spec(spec)\n"
            "sys.modules['run_agentic_dogfood_matrix'] = module\n"
            "spec.loader.exec_module(module)\n"
            "print(module.child_environment()['PYTHONUTF8'])\n"
        )
        for mode in ("0", "1"):
            with self.subTest(runner_utf8_mode=mode):
                completed = subprocess.run(
                    [sys.executable, "-I", "-X", f"utf8={mode}", "-c", code],
                    capture_output=True,
                    text=True,
                    timeout=60,
                )
                self.assertEqual(0, completed.returncode, completed.stderr)
                self.assertEqual(mode, completed.stdout.strip())

    @unittest.skipUnless(shutil.which("bash"), "bash is not on PATH")
    def test_bash_startup_file_cannot_forge_a_shell_probe(self) -> None:
        bash = shutil.which("bash")
        with tempfile.TemporaryDirectory() as script_temp:
            scripts = Path(script_temp)
            (scripts / "forge.sh").write_text("echo 'shell check passed'\nexit 0\n", encoding="utf-8")
            (scripts / "optimize.sh").write_text("export PYTHONOPTIMIZE=2\n", encoding="utf-8")
            (scripts / "check.sh").write_text("exit 1\n", encoding="utf-8")
            (scripts / "python_check.sh").write_text(
                f"{shlex.quote(sys.executable)} -c 'assert 1 == 2' || exit 1\necho 'shell check passed'\n",
                encoding="utf-8",
            )
            for label, startup, script in (
                ("startup file forges the marker", "forge.sh", "check.sh"),
                ("startup file re-exports PYTHONOPTIMIZE to a Python check", "optimize.sh", "python_check.sh"),
            ):
                with self.subTest(label):
                    result = self._run_command(
                        [bash, str(scripts / script)], "shell check passed", {"BASH_ENV": str(scripts / startup)}
                    )
                    self.assertEqual("failed", result.status, result.stdout_excerpt)

    @unittest.skipUnless(shutil.which("bash"), "bash is not on PATH")
    def test_exported_bash_function_cannot_replace_a_command_in_a_shell_probe(self) -> None:
        with tempfile.TemporaryDirectory() as script_temp:
            script = Path(script_temp) / "check.sh"
            script.write_text("false || exit 1\necho 'shell check passed'\n", encoding="utf-8")
            result = self._run_command(
                [shutil.which("bash"), str(script)], "shell check passed", {"BASH_FUNC_false%%": "() {  return 0\n}"}
            )
            self.assertEqual("failed", result.status, result.stdout_excerpt)

    @unittest.skipUnless(shutil.which("bash"), "bash is not on PATH")
    def test_posix_mode_from_the_environment_cannot_change_a_shell_probe(self) -> None:
        # POSIXLY_CORRECT and POSIX_PEDANTIC start bash in POSIX mode, which expands aliases in a script.
        bash = shutil.which("bash")
        script = "check() { return 1; }\nalias check=':'\nif check; then echo 'shell check passed'; else exit 1; fi\n"
        from_python = (
            "import subprocess, sys\n"
            f"sys.exit(subprocess.run([{bash!r}, '-c', {script!r}]).returncode)\n"
        )
        for variable in ("POSIXLY_CORRECT", "POSIX_PEDANTIC"):
            for label, command in (
                ("bash probe", [bash, "-c", script]),
                ("bash started by a Python probe", [sys.executable, "-c", from_python]),
            ):
                with self.subTest(variable=variable, path=label):
                    result = self._run_command(command, "shell check passed", {variable: "1"})
                    self.assertEqual("failed", result.status, result.stdout_excerpt)

    @unittest.skipUnless(shutil.which("bash"), "bash is not on PATH")
    def test_ssh_client_cannot_make_bash_source_a_home_startup_file(self) -> None:
        # A bash built with SSH_SOURCE_BASHRC (macOS /bin/bash, Debian) runs ~/.bashrc before
        # `bash -c` when SSH_CLIENT or SSH2_CLIENT is set and SHLVL is unset or 0.
        bash = shutil.which("bash")
        with tempfile.TemporaryDirectory() as home_temp:
            home = Path(home_temp)
            (home / ".bashrc").write_text("echo 'shell check passed'\nexit 0\n", encoding="utf-8")
            for variable in ("SSH_CLIENT", "SSH2_CLIENT"):
                ambient = {"HOME": str(home), variable: "192.0.2.1 50000 22", "SHLVL": "0"}
                direct = subprocess.run(
                    [bash, "-c", "exit 1"],
                    env={"PATH": os.environ.get("PATH", ""), **ambient},
                    stdin=subprocess.DEVNULL,
                    capture_output=True,
                    text=True,
                    timeout=30,
                )
                if "shell check passed" not in direct.stdout:
                    self.skipTest(f"this bash does not source ~/.bashrc under {variable}")
                with self.subTest(variable=variable):
                    result = self._run_command([bash, "-c", "exit 1"], "shell check passed", ambient)
                    self.assertEqual("failed", result.status, result.stdout_excerpt)

    def test_probes_read_an_empty_standard_input(self) -> None:
        # bash also runs ~/.bashrc when its standard input is a network connection, and an
        # interactive interpreter would read the caller's terminal; probes get /dev/null instead.
        code = (
            "import importlib.util, os, sys\n"
            f"spec = importlib.util.spec_from_file_location('run_agentic_dogfood_matrix', {str(SCRIPT)!r})\n"
            "module = importlib.util.module_from_spec(spec)\n"
            "sys.modules['run_agentic_dogfood_matrix'] = module\n"
            "spec.loader.exec_module(module)\n"
            "probe = 'import os, sys; print(os.path.samestat(os.fstat(0), os.stat(os.devnull)), repr(sys.stdin.read()))'\n"
            "print(module.run([sys.executable, '-c', probe], module.ROOT).stdout.strip())\n"
        )
        completed = subprocess.run(
            [sys.executable, "-I", "-c", code], input="caller input\n", capture_output=True, text=True, timeout=60
        )
        self.assertEqual(0, completed.returncode, completed.stderr)
        self.assertEqual("True ''", completed.stdout.strip())

    @unittest.skipUnless(shutil.which("node"), "node is not on PATH")
    def test_node_options_cannot_preload_code_into_a_node_probe(self) -> None:
        with tempfile.TemporaryDirectory() as script_temp:
            preload = Path(script_temp) / "preload.js"
            preload.write_text("console.log('node check passed');\nprocess.exit(0);\n", encoding="utf-8")
            result = self._run_command(
                [shutil.which("node"), "-e", "process.exit(1)"],
                "node check passed",
                {"NODE_OPTIONS": f'--require "{preload.as_posix()}"'},
            )
            self.assertEqual("failed", result.status, result.stdout_excerpt)

    def test_probe_env_still_sets_an_interpreter_variable_explicitly(self) -> None:
        code = "import sys\nprint('optimize=%d' % sys.flags.optimize)\n"
        result = self._run_probe(code, "optimize=1", {"PYTHONOPTIMIZE": "2"}, env={"PYTHONOPTIMIZE": "1"})
        self.assertEqual("passed", result.status, result.stdout_excerpt)

    def test_runner_itself_has_no_assert_statement(self) -> None:
        # The runner's own logic must not depend on `assert`, which `-O` strips from the runner.
        tree = ast.parse(SCRIPT.read_text(encoding="utf-8"))
        self.assertEqual([], [node.lineno for node in ast.walk(tree) if isinstance(node, ast.Assert)])


if __name__ == "__main__":
    unittest.main()
