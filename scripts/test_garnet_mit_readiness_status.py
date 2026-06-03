#!/usr/bin/env python3
"""Regression tests for the broader MIT-readiness objective status reporter."""
from __future__ import annotations

import importlib.util
import hashlib
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

SCRIPT = Path(__file__).with_name("garnet_mit_readiness_status.py")
TEST_DOGFOOD_DIR = tempfile.TemporaryDirectory()
TEST_CLEAN_VM_ROOT = tempfile.TemporaryDirectory()
TEST_DOMAIN_MATRIX_ROOT = tempfile.TemporaryDirectory()
os.environ["GARNET_PROMO_VIDEO_DESKTOP_DIR"] = TEST_DOGFOOD_DIR.name
os.environ["GARNET_WINDOWS_CLEAN_VM_EVIDENCE_ROOT"] = TEST_CLEAN_VM_ROOT.name
os.environ["GARNET_STUDIO_DOMAIN_MATRIX_ROOT"] = TEST_DOMAIN_MATRIX_ROOT.name
SPEC = importlib.util.spec_from_file_location("garnet_mit_readiness_status", SCRIPT)
assert SPEC is not None
status_mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_mit_readiness_status"] = status_mod
SPEC.loader.exec_module(status_mod)


def _write_verified_domain_matrix_bundle(root: Path) -> None:
    bundle = root / "garnet-studio-domain-matrix-test"
    bundle.mkdir(parents=True, exist_ok=True)
    commands_dir = bundle / "commands"
    commands_dir.mkdir()
    cases = []
    repo_root = Path(__file__).resolve().parents[1]
    for case_id, relative_file in status_mod.DOMAIN_MATRIX_CASES.items():
        source = repo_root / relative_file
        commands = []
        for step in ("parse", "check", "run"):
            stdout = commands_dir / f"{case_id}-{step}-stdout.txt"
            stderr = commands_dir / f"{case_id}-{step}-stderr.txt"
            stdout.write_text(f"{step} ok\n", encoding="utf-8")
            stderr.write_text(
                "BLAKE3 fingerprint mismatch\n"
                if case_id == "mvp_11_signed_hotreload_mismatch" and step == "run"
                else "",
                encoding="utf-8",
            )
            commands.append(
                {
                    "step": step,
                    "argv": ["garnet", step, f"examples/{case_id}.garnet"],
                    "exit_code": 1 if case_id == "mvp_11_signed_hotreload_mismatch" and step == "run" else 0,
                    "status": "passed",
                    "stdout_file": stdout.relative_to(bundle).as_posix(),
                    "stderr_file": stderr.relative_to(bundle).as_posix(),
                    "expected_failure": case_id == "mvp_11_signed_hotreload_mismatch" and step == "run",
                    "expectation": "nonzero exit with `BLAKE3 fingerprint mismatch`"
                    if case_id == "mvp_11_signed_hotreload_mismatch" and step == "run"
                    else "exit 0",
                }
            )
        cases.append(
            {
                "id": case_id,
                "label": case_id,
                "group": "test",
                "file": str(source),
                "repo_relative_file": relative_file,
                "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
                "status": "passed",
                "commands": commands,
            }
        )
    summary = {
        "schema": "garnet.studio.domain_matrix.v1",
        "created_at": "2026-05-25T00:00:00+00:00",
        "source": str(Path(__file__).resolve().parents[1]),
        "suite": "all",
        "status": "passed",
        "platform": "windows",
        "arch": "AMD64",
        "garnet_command": ["garnet"],
        "case_count": 20,
        "passed_cases": 20,
        "failed_cases": 0,
        "command_count": 60,
        "passed_commands": 60,
        "failed_commands": 0,
        "source_included": False,
        "provider_api_called": False,
        "cases": cases,
    }
    (bundle / "garnet-studio-domain-matrix.json").write_text(
        json.dumps(summary),
        encoding="utf-8",
    )
    (bundle / "garnet-studio-domain-matrix.md").write_text(
        "# Garnet Studio Domain Proof Matrix\n",
        encoding="utf-8",
    )
    _write_manifest(bundle)


def _write_manifest(bundle: Path) -> None:
    lines = []
    for path in sorted(bundle.rglob("*")):
        if not path.is_file() or path.name == "MANIFEST.sha256":
            continue
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        lines.append(f"{digest}  {path.relative_to(bundle).as_posix()}")
    (bundle / "MANIFEST.sha256").write_text("\n".join(lines) + "\n", encoding="utf-8")


def _write_committed_domain_matrix_bundle(repo_root: Path, bundle: Path, platform: str) -> None:
    bundle.mkdir(parents=True, exist_ok=True)
    commands_dir = bundle / "commands"
    commands_dir.mkdir()
    cases = []
    for case_id, relative_file in status_mod.DOMAIN_MATRIX_CASES.items():
        source = repo_root / relative_file
        source.parent.mkdir(parents=True, exist_ok=True)
        source.write_text(f"fn main() -> Int {{ {len(case_id)} }}\n", encoding="utf-8")
        commands = []
        for step in ("parse", "check", "run"):
            stdout = commands_dir / f"{case_id}-{step}-stdout.txt"
            stderr = commands_dir / f"{case_id}-{step}-stderr.txt"
            stdout.write_text(f"{platform} {step} ok\n", encoding="utf-8")
            stderr.write_text(
                "BLAKE3 fingerprint mismatch\n"
                if case_id == "mvp_11_signed_hotreload_mismatch" and step == "run"
                else "",
                encoding="utf-8",
            )
            commands.append(
                {
                    "step": step,
                    "argv": ["garnet", step, relative_file],
                    "exit_code": 1 if case_id == "mvp_11_signed_hotreload_mismatch" and step == "run" else 0,
                    "status": "passed",
                    "stdout_file": stdout.relative_to(bundle).as_posix(),
                    "stderr_file": stderr.relative_to(bundle).as_posix(),
                    "expected_failure": case_id == "mvp_11_signed_hotreload_mismatch" and step == "run",
                    "expectation": "nonzero exit with `BLAKE3 fingerprint mismatch`"
                    if case_id == "mvp_11_signed_hotreload_mismatch" and step == "run"
                    else "exit 0",
                }
            )
        cases.append(
            {
                "id": case_id,
                "label": case_id,
                "group": "test",
                "file": str(source),
                "repo_relative_file": relative_file,
                "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
                "status": "passed",
                "commands": commands,
            }
        )
    summary = {
        "schema": "garnet.studio.domain_matrix.v1",
        "created_at": "2026-06-03T00:00:00+00:00",
        "source": str(repo_root),
        "suite": "all",
        "status": "passed",
        "platform": platform,
        "arch": "x86_64",
        "garnet_command": ["garnet"],
        "case_count": len(status_mod.DOMAIN_MATRIX_CASES),
        "passed_cases": len(status_mod.DOMAIN_MATRIX_CASES),
        "failed_cases": 0,
        "command_count": len(status_mod.DOMAIN_MATRIX_CASES) * 3,
        "passed_commands": len(status_mod.DOMAIN_MATRIX_CASES) * 3,
        "failed_commands": 0,
        "source_included": False,
        "provider_api_called": False,
        "cases": cases,
    }
    (bundle / "garnet-studio-domain-matrix.json").write_text(
        json.dumps(summary, indent=2) + "\n",
        encoding="utf-8",
    )
    (bundle / "garnet-studio-domain-matrix.md").write_text(
        f"# {platform} domain matrix\n",
        encoding="utf-8",
    )
    _write_manifest(bundle)


class GarnetMitReadinessStatusTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        _write_verified_domain_matrix_bundle(Path(TEST_DOMAIN_MATRIX_ROOT.name))

    @classmethod
    def tearDownClass(cls) -> None:
        TEST_DOGFOOD_DIR.cleanup()
        TEST_CLEAN_VM_ROOT.cleanup()
        TEST_DOMAIN_MATRIX_ROOT.cleanup()

    def test_status_distinguishes_plan_completion_from_goal_completion(self) -> None:
        status = status_mod.read_status()
        lanes = {lane.id: lane for lane in status.lanes}

        self.assertEqual("active-partial", status.overall_status)
        self.assertLess(status.completion_percent, 100.0)
        self.assertEqual("verified", lanes["tracked_implementation_plan"].status)
        self.assertEqual(100.0, lanes["tracked_implementation_plan"].completion_percent)
        self.assertEqual("blocked", lanes["developer_id_notarization"].status)
        self.assertEqual("planned", lanes["mobile_distribution"].status)
        self.assertEqual("composition-ready", lanes["promo_video"].status)
        self.assertEqual(50.0, lanes["promo_video"].completion_percent)
        self.assertEqual("active-partial", lanes["llm_assist"].status)
        self.assertLess(lanes["llm_assist"].completion_percent, 100.0)
        self.assertIn("official_packages_seed", lanes)
        self.assertEqual(
            "local-registry-source-ready", lanes["official_packages_seed"].status
        )
        self.assertEqual(85.0, lanes["official_packages_seed"].completion_percent)
        self.assertIn("windows_cross_os_enforcement_phase1", lanes)
        self.assertEqual(
            "verified", lanes["windows_cross_os_enforcement_phase1"].status
        )
        self.assertEqual(
            100.0,
            lanes["windows_cross_os_enforcement_phase1"].completion_percent,
        )
        self.assertIn(
            "S101 Stage V trap gate",
            lanes["windows_cross_os_enforcement_phase1"].evidence,
        )
        self.assertIn(
            "execution/portability, not enforcement",
            lanes["windows_cross_os_enforcement_phase1"].evidence,
        )
        s106_deferred = " ".join(lanes["windows_cross_os_enforcement_phase1"].deferred)
        self.assertIn("not Linux seccomp", s106_deferred)
        self.assertIn("not OS-sandbox enforcement", s106_deferred)
        self.assertIn("Phase 2", s106_deferred)
        self.assertEqual("planned", lanes["broad_converter_frontends"].status)
        self.assertIn("windows_linux_distribution", lanes)
        self.assertEqual(
            "active-partial", lanes["windows_linux_distribution"].status
        )
        self.assertEqual(60.0, lanes["windows_linux_distribution"].completion_percent)
        self.assertLess(
            lanes["windows_linux_distribution"].completion_percent, 100.0
        )
        self.assertTrue(lanes["windows_linux_distribution"].blocked_by)
        self.assertIn("windows_linux_domain_proof_matrix", lanes)
        self.assertEqual("verified", lanes["windows_linux_domain_proof_matrix"].status)
        self.assertEqual(100.0, lanes["windows_linux_domain_proof_matrix"].completion_percent)
        self.assertIn("20 current examples", lanes["windows_linux_domain_proof_matrix"].evidence)
        self.assertIn("Committed Windows bundle", lanes["windows_linux_domain_proof_matrix"].evidence)
        self.assertIn(
            "Committed WSL portability bundle",
            lanes["windows_linux_domain_proof_matrix"].evidence,
        )
        self.assertEqual("verified", lanes["editor_lsp_adoption"].status)
        self.assertEqual(100.0, lanes["editor_lsp_adoption"].completion_percent)
        # S9: Determinism CI cross-machine lane
        self.assertIn("determinism_ci_cross_machine", lanes)
        self.assertEqual("verified", lanes["determinism_ci_cross_machine"].status)
        self.assertEqual(100.0, lanes["determinism_ci_cross_machine"].completion_percent)
        self.assertIn(
            "determinism.yml",
            lanes["determinism_ci_cross_machine"].evidence,
        )
        # Honest deferred list (Windows not in matrix yet, etc.) is non-empty
        self.assertTrue(lanes["determinism_ci_cross_machine"].deferred)
        # S5: Parser fuzz harness lane
        self.assertIn("parser_fuzz_harness", lanes)
        self.assertEqual("verified", lanes["parser_fuzz_harness"].status)
        self.assertEqual(100.0, lanes["parser_fuzz_harness"].completion_percent)
        self.assertIn("parse_input", lanes["parser_fuzz_harness"].evidence)
        self.assertIn("ParseBudget", lanes["parser_fuzz_harness"].evidence)
        self.assertTrue(lanes["parser_fuzz_harness"].deferred)
        # S10: Compiler advisory mode (rules-based)
        self.assertIn("compiler_advisory_rules_based", lanes)
        self.assertEqual("verified", lanes["compiler_advisory_rules_based"].status)
        self.assertEqual(100.0, lanes["compiler_advisory_rules_based"].completion_percent)
        self.assertIn("compiler suggested", lanes["compiler_advisory_rules_based"].evidence)
        self.assertIn("suggest.rs", lanes["compiler_advisory_rules_based"].evidence)
        self.assertTrue(lanes["compiler_advisory_rules_based"].deferred)
        # LLM tier is explicitly deferred; pending-infra anchor preserved.
        deferred_text = " ".join(lanes["compiler_advisory_rules_based"].deferred)
        self.assertIn("LLM", deferred_text)
        # S8: Signed hot-reload BLAKE3 demo lane
        self.assertIn("signed_hot_reload_demo", lanes)
        self.assertEqual("verified", lanes["signed_hot_reload_demo"].status)
        self.assertEqual(100.0, lanes["signed_hot_reload_demo"].completion_percent)
        self.assertIn("BLAKE3 fingerprint", lanes["signed_hot_reload_demo"].evidence)
        self.assertIn("reloaded successfully", lanes["signed_hot_reload_demo"].evidence)
        # Honest deferred: managed-mode reload_signed syntax is NOT in this slice
        s8_deferred = " ".join(lanes["signed_hot_reload_demo"].deferred)
        self.assertIn("actor.reload_signed", s8_deferred)

        # S15: Trivia-preserving CST lane
        self.assertIn("parser_cst_layer", lanes)
        self.assertEqual("verified", lanes["parser_cst_layer"].status)
        self.assertEqual(100.0, lanes["parser_cst_layer"].completion_percent)
        self.assertIn("cst.rs", lanes["parser_cst_layer"].evidence)
        self.assertIn("cst_round_trip.rs", lanes["parser_cst_layer"].evidence)
        self.assertIn("incremental syntax parsing", lanes["parser_cst_layer"].deferred)

    def test_json_exposes_evidence_and_deferred_boundaries(self) -> None:
        output = subprocess.check_output(
            [sys.executable, str(SCRIPT), "--format", "json"],
            env={**os.environ, "GARNET_PROMO_VIDEO_DESKTOP_DIR": TEST_DOGFOOD_DIR.name},
            text=True,
        )
        data = json.loads(output)
        lanes = {lane["id"]: lane for lane in data["lanes"]}

        self.assertIn("tracked implementation plan is complete", data["current_truth"])
        self.assertIn("goal remains active", data["current_truth"])
        self.assertIn("APPLE_DEV_ID_APP", lanes["developer_id_notarization"]["blocked_by"])
        self.assertIn("garnet_studio_notarization_status.py", lanes["developer_id_notarization"]["evidence"])
        self.assertIn("deterministic local context pack", lanes["llm_assist"]["evidence"])
        self.assertIn("advisory-only provider option registry", lanes["llm_assist"]["evidence"])
        self.assertIn("Studio provider-options evidence action", lanes["llm_assist"]["evidence"])
        self.assertIn("provider-neutral advisory bundle", lanes["llm_assist"]["evidence"])
        self.assertIn("official_packages_seed", lanes)
        self.assertEqual(
            "local-registry-source-ready",
            lanes["official_packages_seed"]["status"],
        )
        self.assertIn(
            "garnet_lang_registry_seed", lanes["official_packages_seed"]["evidence"]
        )
        self.assertIn(
            "github.com/garnet-lang",
            " ".join(lanes["official_packages_seed"]["deferred"]),
        )
        self.assertIn("windows_cross_os_enforcement_phase1", lanes)
        self.assertEqual(
            "verified", lanes["windows_cross_os_enforcement_phase1"]["status"]
        )
        self.assertIn(
            "WSL row is labeled `execution/portability, not enforcement`",
            lanes["windows_cross_os_enforcement_phase1"]["evidence"],
        )
        self.assertIn(
            "S103 ultrapunch accept/reject reproduction is Phase 2",
            " ".join(lanes["windows_cross_os_enforcement_phase1"]["deferred"]),
        )
        self.assertIn("compiler_agent_llm_tier", lanes)
        self.assertEqual(
            "feature-gated-source-ready",
            lanes["compiler_agent_llm_tier"]["status"],
        )
        self.assertEqual(85.0, lanes["compiler_agent_llm_tier"]["completion_percent"])
        self.assertIn("garnet-suggest-llm", lanes["compiler_agent_llm_tier"]["evidence"])
        self.assertIn(
            "garnet-cli",
            " ".join(lanes["compiler_agent_llm_tier"]["deferred"]),
        )
        self.assertIn("garnet_promo_video_status.py", lanes["promo_video"]["evidence"])
        self.assertIn("visual identity", lanes["promo_video"]["evidence"])
        self.assertIn("composition source", lanes["promo_video"]["evidence"])
        self.assertIn("rendered artifact", lanes["promo_video"]["blocked_by"])
        self.assertNotIn("HyperFrames or Remotion composition", lanes["promo_video"]["deferred"])
        self.assertIn("JavaScript", lanes["broad_converter_frontends"]["deferred"])
        self.assertIn("Android", lanes["mobile_distribution"]["deferred"])
        self.assertIn("garnet_proof_benchmark_status.py", lanes["proof_empirics"]["evidence"])
        self.assertIn("S2 VM parse/compile/execute harness", lanes["proof_empirics"]["evidence"])
        self.assertIn("1 fuzz harness", lanes["proof_empirics"]["evidence"])
        self.assertIn("nightly fuzz hours require GitHub Actions evidence after merge", lanes["proof_empirics"]["blocked_by"])
        self.assertIn("accumulated nightly fuzz hours", lanes["proof_empirics"]["deferred"])
        self.assertIn("fresh benchmark measurement run", lanes["proof_empirics"]["blocked_by"])
        self.assertIn("formal RustBelt/Iris/Coq mechanization", lanes["proof_empirics"]["deferred"])
        self.assertIn("Tauri v2 shell scaffold", lanes["windows_linux_distribution"]["evidence"])
        self.assertIn("v0.5 readiness reporter parity", lanes["windows_linux_distribution"]["evidence"])
        self.assertIn("Domain Proof Matrix", lanes["windows_linux_distribution"]["evidence"])
        self.assertIn("clean-VM installer proof contract", lanes["windows_linux_distribution"]["evidence"])
        self.assertIn("clean Windows VM", " ".join(lanes["windows_linux_distribution"]["blocked_by"]))
        self.assertIn("windows_linux_domain_proof_matrix", lanes)
        self.assertEqual("verified", lanes["windows_linux_domain_proof_matrix"]["status"])
        self.assertIn("BLAKE3 mismatch rejection", lanes["windows_linux_domain_proof_matrix"]["evidence"])
        self.assertIn("S16 CST-precise LSP surface", lanes["editor_lsp_adoption"]["evidence"])
        self.assertIn("document/workspace symbols", lanes["editor_lsp_adoption"]["evidence"])
        self.assertIn("CST-precise rename", lanes["editor_lsp_adoption"]["evidence"])
        self.assertIn("rules-based quick-fix actions", lanes["editor_lsp_adoption"]["evidence"])
        self.assertIn("semantic tokens", lanes["editor_lsp_adoption"]["evidence"])
        self.assertNotIn("release-backed VSIX smoke after tag", lanes["editor_lsp_adoption"]["deferred"])
        self.assertEqual([], lanes["editor_lsp_adoption"]["deferred"])

    def test_domain_matrix_lane_is_source_present_without_verified_bundle(self) -> None:
        with tempfile.TemporaryDirectory() as repo, tempfile.TemporaryDirectory() as local:
            repo_root = Path(repo)
            (repo_root / "scripts").mkdir()
            (repo_root / "scripts" / "smoke_garnet_studio_domain_matrix.py").write_text(
                "# domain matrix script present\n",
                encoding="utf-8",
            )
            with mock.patch.object(status_mod, "ROOT", repo_root), mock.patch.dict(
                os.environ,
                {"GARNET_STUDIO_DOMAIN_MATRIX_ROOT": local},
            ):
                evidence = status_mod._domain_matrix_evidence()

        self.assertTrue(evidence.source_present)
        self.assertFalse(evidence.verified)
        self.assertIn("no verified", evidence.reason)

    def test_domain_matrix_evidence_accepts_committed_windows_and_wsl_bundles(self) -> None:
        with tempfile.TemporaryDirectory() as temp, tempfile.TemporaryDirectory() as local:
            repo_root = Path(temp)
            (repo_root / "scripts").mkdir()
            (repo_root / "scripts" / "smoke_garnet_studio_domain_matrix.py").write_text(
                "# domain matrix script present\n",
                encoding="utf-8",
            )
            _write_committed_domain_matrix_bundle(
                repo_root,
                repo_root / "proofs" / "windows" / "domains" / "windows-domain-matrix-test",
                "windows",
            )
            _write_committed_domain_matrix_bundle(
                repo_root,
                repo_root / "proofs" / "linux" / "execution" / "domains" / "wsl-domain-matrix-test",
                "linux-wsl",
            )

            with mock.patch.object(status_mod, "ROOT", repo_root), mock.patch.dict(
                os.environ,
                {"GARNET_STUDIO_DOMAIN_MATRIX_ROOT": local},
            ):
                evidence = status_mod._domain_matrix_evidence()

        self.assertTrue(evidence.verified)
        self.assertIn("Committed Windows bundle", evidence.reason)
        self.assertIn("Committed WSL portability bundle", evidence.reason)

    def test_domain_matrix_verifier_rejects_fake_manifest_hashes(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            _write_verified_domain_matrix_bundle(root)
            bundle = root / "garnet-studio-domain-matrix-test"
            (bundle / "MANIFEST.sha256").write_text(
                "f" * 64 + "  garnet-studio-domain-matrix.json\n",
                encoding="utf-8",
            )

            self.assertFalse(
                status_mod._domain_matrix_summary_verified(
                    bundle / "garnet-studio-domain-matrix.json"
                )
            )

    def test_domain_matrix_verifier_rejects_nonzero_regular_run(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            _write_verified_domain_matrix_bundle(root)
            bundle = root / "garnet-studio-domain-matrix-test"
            summary = bundle / "garnet-studio-domain-matrix.json"
            data = json.loads(summary.read_text(encoding="utf-8"))
            case = next(
                case
                for case in data["cases"]
                if case["id"] != "mvp_11_signed_hotreload_mismatch"
            )
            run = next(command for command in case["commands"] if command["step"] == "run")
            run["exit_code"] = 1
            summary.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
            _write_manifest(bundle)

            self.assertFalse(status_mod._domain_matrix_summary_verified(summary))

    def test_domain_matrix_verifier_rejects_mismatch_without_marker(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            _write_verified_domain_matrix_bundle(root)
            bundle = root / "garnet-studio-domain-matrix-test"
            stderr = (
                bundle
                / "commands"
                / "mvp_11_signed_hotreload_mismatch-run-stderr.txt"
            )
            stderr.write_text("generic failure\n", encoding="utf-8")
            _write_manifest(bundle)

            self.assertFalse(
                status_mod._domain_matrix_summary_verified(
                    bundle / "garnet-studio-domain-matrix.json"
                )
            )

    def test_clean_vm_proof_lifts_windows_distribution_lane_without_overclaiming(self) -> None:
        clean_vm_mod = (
            status_mod.garnet_windows_linux_studio_status.garnet_windows_clean_vm_installer_status
        )
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
            clean_vm_record = clean_vm_mod.build_proof_record(
                mode="clean-vm",
                installer=installer,
                vm_name="WindowsSandbox-123",
                guest_os="Windows 11 Enterprise",
                guest_arch="AMD64",
                install_log=install_log,
                studio_smoke_json=smoke,
                screenshot=screenshot,
            )
            clean_vm_mod.write_proof(clean_vm_record, bundle)

            with mock.patch.dict(
                os.environ,
                {
                    "GARNET_PROMO_VIDEO_DESKTOP_DIR": TEST_DOGFOOD_DIR.name,
                    "GARNET_WINDOWS_CLEAN_VM_EVIDENCE_ROOT": str(root),
                },
            ):
                status = status_mod.read_status()

        lane = {lane.id: lane for lane in status.lanes}["windows_linux_distribution"]
        self.assertEqual("active-partial", lane.status)
        self.assertEqual(70.0, lane.completion_percent)
        self.assertIn("verified x64 clean-VM installer proof", lane.evidence)
        self.assertNotIn("clean Windows VM", " ".join(lane.blocked_by))
        self.assertIn("Linux VM/container", " ".join(lane.blocked_by))
        self.assertIn("Windows ARM64 target build/smoke", " ".join(lane.deferred))

    def test_rendered_promo_artifacts_update_objective_blockers(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            artifact_dir = Path(temp) / "garnet-promo-video"
            artifact_dir.mkdir()
            (artifact_dir / "garnet-promo.mp4").write_bytes(b"fake-mp4")
            (artifact_dir / "garnet-promo.webm").write_bytes(b"fake-webm")

            with mock.patch.dict(os.environ, {"GARNET_PROMO_VIDEO_DESKTOP_DIR": temp}):
                status = status_mod.read_status()

        lanes = {lane.id: lane for lane in status.lanes}
        promo_lane = lanes["promo_video"]
        self.assertEqual("rendered-artifact-ready", promo_lane.status)
        self.assertEqual(65.0, promo_lane.completion_percent)
        self.assertIn("rendered MP4/WebM evidence", promo_lane.evidence)
        self.assertNotIn("rendered artifact", promo_lane.blocked_by)
        self.assertIn("visual QA verdict", promo_lane.blocked_by)

    def test_visual_qa_promo_artifacts_update_objective_blockers(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            artifact_dir = Path(temp) / "garnet-promo-video"
            artifact_dir.mkdir()
            (artifact_dir / "garnet-promo.mp4").write_bytes(b"fake-mp4")
            (artifact_dir / "garnet-promo.webm").write_bytes(b"fake-webm")
            qa_dir = Path(temp) / "garnet-promo-video-visual-qa"
            qa_dir.mkdir()
            (qa_dir / "promo-visual-qa-data.json").write_text(
                json.dumps({"status": "visual-qa-ready", "verdict": "pass", "checks": [{"passed": True}]}),
                encoding="utf-8",
            )

            with mock.patch.dict(os.environ, {"GARNET_PROMO_VIDEO_DESKTOP_DIR": temp}):
                status = status_mod.read_status()

        lanes = {lane.id: lane for lane in status.lanes}
        promo_lane = lanes["promo_video"]
        self.assertEqual("visual-qa-ready", promo_lane.status)
        self.assertEqual(80.0, promo_lane.completion_percent)
        self.assertNotIn("visual QA verdict", promo_lane.blocked_by)
        self.assertIn("website-ready export", promo_lane.blocked_by)

    def test_website_export_promo_artifacts_update_objective_blockers(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            artifact_dir = Path(temp) / "garnet-promo-video"
            artifact_dir.mkdir()
            (artifact_dir / "garnet-promo.mp4").write_bytes(b"fake-mp4")
            (artifact_dir / "garnet-promo.webm").write_bytes(b"fake-webm")
            qa_dir = Path(temp) / "garnet-promo-video-visual-qa"
            qa_dir.mkdir()
            (qa_dir / "promo-visual-qa-data.json").write_text(
                json.dumps({"status": "visual-qa-ready", "verdict": "pass", "checks": [{"passed": True}]}),
                encoding="utf-8",
            )
            export_dir = Path(temp) / "garnet-promo-video-website-export"
            export_dir.mkdir()
            (export_dir / "promo-website-export-data.json").write_text(
                json.dumps({"status": "website-export-ready", "verdict": "pass", "checks": [{"passed": True}]}),
                encoding="utf-8",
            )

            with mock.patch.dict(os.environ, {"GARNET_PROMO_VIDEO_DESKTOP_DIR": temp}):
                status = status_mod.read_status()

        lanes = {lane.id: lane for lane in status.lanes}
        promo_lane = lanes["promo_video"]
        self.assertEqual("website-export-ready", promo_lane.status)
        self.assertEqual(90.0, promo_lane.completion_percent)
        self.assertNotIn("website-ready export", promo_lane.blocked_by)
        self.assertIn("public-site embedding and review", promo_lane.blocked_by)

    def test_repo_site_embed_updates_objective_blockers_without_full_completion(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            artifact_dir = Path(temp) / "garnet-promo-video"
            artifact_dir.mkdir()
            (artifact_dir / "garnet-promo.mp4").write_bytes(b"fake-mp4")
            (artifact_dir / "garnet-promo.webm").write_bytes(b"fake-webm")
            qa_dir = Path(temp) / "garnet-promo-video-visual-qa"
            qa_dir.mkdir()
            (qa_dir / "promo-visual-qa-data.json").write_text(
                json.dumps({"status": "visual-qa-ready", "verdict": "pass", "checks": [{"passed": True}]}),
                encoding="utf-8",
            )
            export_dir = Path(temp) / "garnet-promo-video-website-export"
            export_dir.mkdir()
            (export_dir / "promo-website-export-data.json").write_text(
                json.dumps({"status": "website-export-ready", "verdict": "pass", "checks": [{"passed": True}]}),
                encoding="utf-8",
            )
            sync_dir = Path(temp) / "garnet-promo-video-site-sync"
            sync_dir.mkdir()
            (sync_dir / "promo-site-sync-data.json").write_text(
                json.dumps({"status": "public-site-embedded", "verdict": "pass", "checks": [{"passed": True}]}),
                encoding="utf-8",
            )

            with mock.patch.dict(os.environ, {"GARNET_PROMO_VIDEO_DESKTOP_DIR": temp}):
                status = status_mod.read_status()
        lanes = {lane.id: lane for lane in status.lanes}
        promo_lane = lanes["promo_video"]

        self.assertEqual("public-site-embedded", promo_lane.status)
        self.assertEqual(95.0, promo_lane.completion_percent)
        self.assertNotIn("public-site embedding and review", promo_lane.blocked_by)
        self.assertIn("human/aesthetic acceptance review", promo_lane.blocked_by)
        self.assertLess(status.completion_percent, 100.0)
        rendered = status_mod.render_markdown(status)
        self.assertEqual(1, rendered.count("human/aesthetic acceptance review"))

    def test_markdown_is_human_readable_and_honest(self) -> None:
        rendered = subprocess.check_output(
            [sys.executable, str(SCRIPT)],
            env={**os.environ, "GARNET_PROMO_VIDEO_DESKTOP_DIR": TEST_DOGFOOD_DIR.name},
            text=True,
        )

        self.assertIn("not full MIT/productization completion", rendered)
        self.assertIn("Developer ID notarization", rendered)
        self.assertIn("Mobile distribution", rendered)
        self.assertIn("Promo video", rendered)
        self.assertIn("Editor/LSP adoption", rendered)
        self.assertIn("LLM assist", rendered)
        self.assertIn("Broad converter frontends", rendered)

    def test_markdown_survives_cp1252_stdout(self) -> None:
        cp = subprocess.run(
            [sys.executable, str(SCRIPT), "--format", "markdown"],
            env={
                **os.environ,
                "GARNET_PROMO_VIDEO_DESKTOP_DIR": TEST_DOGFOOD_DIR.name,
                "PYTHONIOENCODING": "cp1252",
            },
            capture_output=True,
            text=True,
        )
        self.assertEqual(0, cp.returncode, cp.stderr)
        self.assertIn("Garnet MIT Readiness Objective Status", cp.stdout)

    def test_local_temp_probe_failure_is_quarantined(self) -> None:
        def denied_probe():
            raise PermissionError("denied temp fixture")

        with mock.patch.object(
            status_mod.garnet_promo_video_status, "read_status", denied_probe
        ):
            status = status_mod.read_status()

        lanes = {lane.id: lane for lane in status.lanes}
        self.assertEqual("local", lanes["promo_video"].evidence_class)
        self.assertEqual("planned-contract", lanes["promo_video"].status)
        self.assertIn("local promo probe skipped", lanes["promo_video"].evidence)
        self.assertEqual("active-partial", status.overall_status)

    def test_committed_only_json_excludes_local_lanes_and_source_paths(self) -> None:
        first = subprocess.check_output(
            [sys.executable, str(SCRIPT), "--format", "json", "--committed-only"],
            env={**os.environ, "GARNET_PROMO_VIDEO_DESKTOP_DIR": TEST_DOGFOOD_DIR.name},
            text=True,
        )
        second = subprocess.check_output(
            [sys.executable, str(SCRIPT), "--format", "json", "--committed-only"],
            env={
                **os.environ,
                "GARNET_PROMO_VIDEO_DESKTOP_DIR": TEST_CLEAN_VM_ROOT.name,
                "GARNET_STUDIO_DOMAIN_MATRIX_ROOT": TEST_CLEAN_VM_ROOT.name,
            },
            text=True,
        )
        self.assertEqual(first, second)
        data = json.loads(first)
        self.assertEqual("committed-truth", data["source"])
        self.assertTrue(data["lanes"])
        self.assertTrue(all(lane["evidence_class"] == "committed" for lane in data["lanes"]))
        self.assertNotIn(str(Path.home()), first)
        self.assertNotIn("windows_linux_distribution", {lane["id"] for lane in data["lanes"]})

    def test_no_regression_gate_passes_source_only_floor(self) -> None:
        completed = subprocess.run(
            [sys.executable, str(SCRIPT), "--check-no-regression"],
            env={**os.environ, "GARNET_PROMO_VIDEO_DESKTOP_DIR": TEST_DOGFOOD_DIR.name},
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

        self.assertEqual("", completed.stderr)
        self.assertEqual(0, completed.returncode)

    def test_committed_truth_split_quarantines_local_evidence(self) -> None:
        # S31-PR2: machine-variable lanes are tagged local; the determinism lane is committed.
        status = status_mod.read_status()
        lanes = {lane.id: lane for lane in status.lanes}
        for local_id in (
            "windows_linux_distribution",
            "promo_video",
        ):
            self.assertEqual("local", lanes[local_id].evidence_class, local_id)
        domain_lane = lanes["windows_linux_domain_proof_matrix"]
        if "Committed Windows bundle" in domain_lane.evidence:
            self.assertEqual("committed", domain_lane.evidence_class)
        else:
            self.assertEqual("local", domain_lane.evidence_class)
        self.assertIn("reporter_determinism", lanes)
        self.assertEqual("committed", lanes["reporter_determinism"].evidence_class)
        self.assertEqual("verified", lanes["reporter_determinism"].status)

    def test_no_regression_gate_ignores_local_lanes(self) -> None:
        # S31-PR2: a local lane pinned ABOVE the live value must not trip the gate
        # (this is the cross-machine false-regression the fix removes).
        status = status_mod.read_status()
        with tempfile.TemporaryDirectory() as tmp:
            baseline_path = Path(tmp) / "baseline.json"
            baseline_path.write_text(
                json.dumps(
                    {"lanes": [{"id": "windows_linux_distribution", "completion_percent": 99.0}]}
                ),
                encoding="utf-8",
            )
            regressions, missing = status_mod.check_no_regression(status, baseline_path)
        self.assertEqual([], regressions)
        self.assertEqual([], missing)

    def test_public_site_surfaces_objective_accounting_without_overclaiming(self) -> None:
        docs_dir = Path(__file__).resolve().parents[1] / "docs"
        site = (docs_dir / "index.html").read_text(
            encoding="utf-8"
        )
        status_site = (docs_dir / "status.html").read_text(encoding="utf-8")

        self.assertIn("Objective accounting", site)
        self.assertIn("MIT/productization objective", site)
        self.assertIn("78.0%", site)
        self.assertNotIn("58.1%", site)
        self.assertNotIn("55.8%", site)
        self.assertNotIn("57.9%", site)
        self.assertNotIn("58.6%", site)
        self.assertIn("78.0%", status_site)
        self.assertNotIn("58.1%", status_site)
        self.assertNotIn("55.8%", status_site)
        self.assertNotIn("57.9%", status_site)
        self.assertNotIn("58.6%", status_site)
        self.assertIn("87/87 tracked slices", site)
        self.assertIn("tracked implementation plan is complete", site)
        self.assertIn("not full MIT/productization completion", site)
        self.assertIn("notarization", site)
        self.assertIn("machine-readable preflight status reporter", site)
        self.assertIn("mobile", site)
        self.assertIn("LLM assist", site)
        self.assertIn("verified x64 clean-VM installer proof", site)
        self.assertIn("verified x64 clean-VM installer proof", status_site)

    # S0: --check-no-regression flag.

    def test_check_no_regression_passes_when_baseline_matches(self) -> None:
        # Use the live status as its own baseline: nothing can have regressed.
        status = status_mod.read_status()
        baseline = {
            "lanes": [
                {"id": lane.id, "completion_percent": lane.completion_percent}
                for lane in status.lanes
            ]
        }
        with tempfile.TemporaryDirectory() as temp:
            baseline_path = Path(temp) / "baseline.json"
            baseline_path.write_text(json.dumps(baseline), encoding="utf-8")
            regressions, missing = status_mod.check_no_regression(status, baseline_path)
        self.assertEqual([], regressions)
        self.assertEqual([], missing)

    def test_check_no_regression_detects_regression(self) -> None:
        status = status_mod.read_status()
        target_lane = status.lanes[0]
        baseline = {
            "lanes": [
                {
                    "id": target_lane.id,
                    "completion_percent": target_lane.completion_percent + 25.0,
                }
            ]
        }
        with tempfile.TemporaryDirectory() as temp:
            baseline_path = Path(temp) / "baseline.json"
            baseline_path.write_text(json.dumps(baseline), encoding="utf-8")
            regressions, missing = status_mod.check_no_regression(status, baseline_path)
        self.assertEqual(1, len(regressions))
        self.assertIn(target_lane.id, regressions[0])
        self.assertEqual([], missing)

    def test_check_no_regression_flags_missing_lane(self) -> None:
        status = status_mod.read_status()
        baseline = {
            "lanes": [
                {
                    "id": "ghost_lane_that_was_renamed_or_deleted",
                    "completion_percent": 10.0,
                }
            ]
        }
        with tempfile.TemporaryDirectory() as temp:
            baseline_path = Path(temp) / "baseline.json"
            baseline_path.write_text(json.dumps(baseline), encoding="utf-8")
            regressions, missing = status_mod.check_no_regression(status, baseline_path)
        self.assertEqual([], regressions)
        self.assertEqual(["ghost_lane_that_was_renamed_or_deleted"], missing)

    def test_check_no_regression_missing_baseline_reports_seed_instructions(self) -> None:
        status = status_mod.read_status()
        with tempfile.TemporaryDirectory() as temp:
            baseline_path = Path(temp) / "nonexistent.json"
            regressions, missing = status_mod.check_no_regression(status, baseline_path)
        self.assertEqual([], regressions)
        self.assertEqual(1, len(missing))
        self.assertIn("baseline missing", missing[0])

    def test_check_no_regression_cli_exit_code_on_regression(self) -> None:
        status = status_mod.read_status()
        target_lane = status.lanes[0]
        baseline = {
            "lanes": [
                {
                    "id": target_lane.id,
                    "completion_percent": target_lane.completion_percent + 25.0,
                }
            ]
        }
        with tempfile.TemporaryDirectory() as temp:
            baseline_path = Path(temp) / "baseline.json"
            baseline_path.write_text(json.dumps(baseline), encoding="utf-8")
            cp = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPT),
                    "--check-no-regression",
                    "--baseline",
                    str(baseline_path),
                ],
                env={**os.environ, "GARNET_PROMO_VIDEO_DESKTOP_DIR": TEST_DOGFOOD_DIR.name},
                capture_output=True,
                text=True,
            )
        self.assertEqual(1, cp.returncode)
        self.assertIn("Readiness regression detected", cp.stderr)


if __name__ == "__main__":
    unittest.main()
