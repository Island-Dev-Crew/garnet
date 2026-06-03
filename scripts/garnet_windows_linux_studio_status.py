#!/usr/bin/env python3
"""Report the Windows/Linux Garnet Studio MVP status.

This is the target-platform counterpart to the Mac-side continuation pulse. It
records the current Windows/Linux Studio source truth: the original command and
evidence contract, the Tauri v2 shell scaffold, and the v0.5 readiness reporter
parity surface. It still does not claim signed Windows distribution, winget,
Linux runtime proof, or a completed cross-platform release.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Sequence

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import garnet_windows_clean_vm_installer_status  # noqa: E402
import smoke_garnet_studio_linux_wsl_deb  # noqa: E402
import smoke_garnet_studio_linux_wsl_deb_install  # noqa: E402
import smoke_garnet_studio_linux_wsl_rpm  # noqa: E402
import smoke_garnet_studio_linux_wsl_xvfb  # noqa: E402
import smoke_garnet_studio_linux_wsl_xvfb_window  # noqa: E402
import smoke_garnet_studio_linux_wslg_install_launch  # noqa: E402

ACTIVE_CONVERSION = ["Rust", "Ruby", "Python", "Go"]
ADVISORY_PLANNING = [
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
]
NATIVE_BOUNDARY_RECOMMENDED = [
    "C",
    "C++",
    "Objective-C",
    "Assembly",
    "CUDA",
    "platform-specific code",
]
FUTURE_BACKEND_LOWERING = [
    "Wasm",
    "LLVM-style native targets",
    "native package toolchains",
]
CLEAN_VM_EVIDENCE_ROOT_ENV = "GARNET_WINDOWS_CLEAN_VM_EVIDENCE_ROOT"


@dataclass(frozen=True)
class LanguageTaxonomy:
    active_conversion: list[str]
    advisory_planning: list[str]
    native_boundary_recommended: list[str]
    future_backend_lowering: list[str]


@dataclass(frozen=True)
class SafetyContract:
    calls_provider_apis_by_default: bool
    executes_source_code: bool
    includes_source_by_default: bool
    marks_advisory_output_safe: bool
    required_handoff_gates: list[str]
    forbidden_claims: list[str]


@dataclass(frozen=True)
class StudioAction:
    id: str
    label: str
    group: str
    implementation_surface: str
    current_command: list[str]
    evidence_required: bool
    source_included_by_default: bool
    status: str


@dataclass(frozen=True)
class EvidenceContract:
    default_root: str
    bundle_prefix: str
    created_by_default: bool
    source_included_by_default: bool
    required_files: list[str]


@dataclass(frozen=True)
class PackagingGate:
    id: str
    platform: str
    status: str
    next_evidence: str
    forbidden_claim: str


@dataclass(frozen=True)
class WindowsLinuxStudioStatus:
    source: str
    status: str
    current_truth: list[str]
    least_new_dependency_decision: str
    taxonomy: LanguageTaxonomy
    safety_contract: SafetyContract
    evidence_contract: EvidenceContract
    actions: list[StudioAction]
    packaging_gates: list[PackagingGate]
    next_slices: list[str]
    user_assistance_needed: list[str]


@dataclass(frozen=True)
class CommandPlan:
    action_id: str
    label: str
    argv: list[str]
    working_directory: str
    source_included_by_default: bool
    calls_provider_apis: bool
    executes_source_code: bool


class CommandContractError(ValueError):
    """Raised when a requested Studio command would break the MVP contract."""


def default_dogfood_root(home: Path | None = None) -> Path:
    base = home or Path.home()
    return base / "Desktop" / "dogfood" / "garnet-studio-windows-linux"


def timestamp_slug(now: datetime | None = None) -> str:
    current = now or datetime.now(timezone.utc)
    return current.strftime("%Y%m%d-%H%M%S")


def create_evidence_bundle(
    output_root: Path | None = None,
    now: datetime | None = None,
) -> Path:
    root = output_root or default_dogfood_root()
    bundle = root / f"garnet-studio-windows-linux-{timestamp_slug(now)}"
    bundle.mkdir(parents=True, exist_ok=True)
    contract = {
        "created_at": (now or datetime.now(timezone.utc)).isoformat(),
        "source": str(ROOT),
        "include_source_by_default": False,
        "source_included": False,
        "status": "tauri-v2-shell-v0-5-readiness-parity",
        "required_runtime_proof": [
            "launch on Windows",
            "launch on Linux",
            "locate or bundle garnet",
            "parse/check/run local examples",
            "invoke the Domain Proof Matrix",
            "write advisory handoff evidence without source by default",
            "invoke agentic dogfood matrix",
        ],
    }
    (bundle / "garnet-windows-linux-studio-evidence-contract.json").write_text(
        json.dumps(contract, indent=2) + "\n",
        encoding="utf-8",
    )
    _write_manifest(bundle)
    return bundle


def _write_manifest(directory: Path) -> None:
    lines = []
    for path in sorted(directory.iterdir()):
        if not path.is_file() or path.name == "MANIFEST.sha256":
            continue
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        lines.append(f"{digest}  {path.name}")
    (directory / "MANIFEST.sha256").write_text("\n".join(lines) + "\n", encoding="utf-8")


def _script(name: str, repo_root: Path) -> str:
    return str(repo_root / "scripts" / name)


def _clean_vm_evidence_root_from_env() -> Path | None:
    configured = os.environ.get(CLEAN_VM_EVIDENCE_ROOT_ENV)
    if not configured:
        return None
    return Path(configured).expanduser()


def _require_path(value: Path | None, field: str) -> str:
    if value is None:
        raise CommandContractError(f"{field} is required")
    return str(value)


def _require_language(language: str | None, allowed: Sequence[str], field: str) -> str:
    if language is None:
        raise CommandContractError(f"{field} is required")
    normalized = language.strip()
    allowed_ids = {item.lower(): item.lower() for item in allowed}
    if normalized.lower() not in allowed_ids:
        raise CommandContractError(f"{normalized!r} is not allowed for this action")
    return normalized.lower()


def build_command_plan(
    action_id: str,
    *,
    source: Path | None = None,
    language: str | None = None,
    evidence_dir: Path | None = None,
    bundle_dir: Path | None = None,
    review_dir: Path | None = None,
    garnet_executable: str = "garnet",
    python_executable: str = sys.executable,
    repo_root: Path = ROOT,
    include_source: bool = False,
) -> CommandPlan:
    workdir = str(repo_root)
    evidence = str(evidence_dir or default_dogfood_root())

    if action_id == "cli_health":
        return CommandPlan(action_id, "CLI Health", [garnet_executable, "version"], workdir, False, False, False)
    if action_id == "parse":
        return CommandPlan(
            action_id,
            "Parse",
            [garnet_executable, "parse", _require_path(source, "source")],
            workdir,
            False,
            False,
            False,
        )
    if action_id == "check":
        return CommandPlan(
            action_id,
            "Check",
            [garnet_executable, "check", _require_path(source, "source")],
            workdir,
            False,
            False,
            False,
        )
    if action_id == "run":
        return CommandPlan(
            action_id,
            "Run",
            [garnet_executable, "run", _require_path(source, "source")],
            workdir,
            False,
            False,
            True,
        )
    if action_id == "convert":
        active = _require_language(language, ACTIVE_CONVERSION, "language")
        return CommandPlan(
            action_id,
            "Convert",
            [garnet_executable, "convert", active, _require_path(source, "source"), "--out", evidence],
            workdir,
            False,
            False,
            False,
        )
    if action_id == "assist_plan":
        advisory = _require_language(language, ADVISORY_PLANNING, "language")
        return CommandPlan(
            action_id,
            "Assist Plan",
            [
                python_executable,
                _script("garnet_converter_assist_plan.py", repo_root),
                "--language",
                advisory,
                "--source",
                _require_path(source, "source"),
                "--output-dir",
                evidence,
            ],
            workdir,
            False,
            False,
            False,
        )
    if action_id == "advisory_bundle":
        advisory = _require_language(language, ADVISORY_PLANNING, "language")
        argv = [
            python_executable,
            _script("garnet_converter_advisory_bundle.py", repo_root),
            "--language",
            advisory,
            "--source",
            _require_path(source, "source"),
            "--output-dir",
            evidence,
        ]
        if include_source:
            argv.append("--include-source")
        return CommandPlan(action_id, "Advisory Bundle", argv, workdir, False, False, False)
    if action_id == "advisory_review":
        return CommandPlan(
            action_id,
            "Advisory Review",
            [
                python_executable,
                _script("garnet_converter_advisory_review.py", repo_root),
                "--bundle-dir",
                _require_path(bundle_dir, "bundle_dir"),
                "--output-dir",
                evidence,
            ],
            workdir,
            False,
            False,
            False,
        )
    if action_id == "advisory_handoff":
        return CommandPlan(
            action_id,
            "Advisory Handoff",
            [
                python_executable,
                _script("garnet_converter_advisory_handoff.py", repo_root),
                "--bundle-dir",
                _require_path(bundle_dir, "bundle_dir"),
                "--review-dir",
                _require_path(review_dir, "review_dir"),
                "--output-dir",
                evidence,
            ],
            workdir,
            False,
            False,
            False,
        )
    if action_id == "objective_pulse":
        return CommandPlan(
            action_id,
            "Objective Pulse",
            [python_executable, _script("garnet_mit_readiness_status.py", repo_root)],
            workdir,
            False,
            False,
            False,
        )
    if action_id == "agentic_dogfood_matrix":
        return CommandPlan(
            action_id,
            "Agentic Dogfood Matrix",
            [
                python_executable,
                _script("run_agentic_dogfood_matrix.py", repo_root),
                "--copy-to-desktop",
                "--strict",
            ],
            workdir,
            False,
            False,
            False,
        )
    if action_id == "domain_proof_matrix":
        return _report_command_plan(
            action_id,
            "Domain Proof Matrix",
            "smoke_garnet_studio_domain_matrix.py",
            "markdown",
            evidence,
            workdir,
            python_executable,
            repo_root,
            executes_source_code=True,
        )
    if action_id == "windows_linux_studio_status":
        return CommandPlan(
            action_id,
            "Windows/Linux Studio Status",
            [
                python_executable,
                _script("garnet_windows_linux_studio_status.py", repo_root),
                "--format",
                "markdown",
            ],
            workdir,
            False,
            False,
            False,
        )
    if action_id == "converter_status":
        return CommandPlan(
            action_id,
            "Converter Fit Matrix",
            [
                python_executable,
                _script("garnet_converter_status.py", repo_root),
                "--format",
                "markdown",
            ],
            workdir,
            False,
            False,
            False,
        )
    if action_id == "provider_options":
        return _report_command_plan(
            action_id,
            "Provider Options",
            "garnet_converter_llm_feasibility.py",
            "markdown",
            evidence,
            workdir,
            python_executable,
            repo_root,
        )
    if action_id == "mit_demo_route":
        return _report_command_plan(
            action_id,
            "MIT Demo Route",
            "garnet_mit_demo_route.py",
            "markdown",
            evidence,
            workdir,
            python_executable,
            repo_root,
        )
    if action_id == "mit_deck_outline":
        return _report_command_plan(
            action_id,
            "MIT Deck Outline",
            "garnet_mit_deck_outline.py",
            "markdown",
            evidence,
            workdir,
            python_executable,
            repo_root,
        )
    if action_id == "mit_deck_preview":
        return _report_command_plan(
            action_id,
            "MIT Deck Preview",
            "garnet_mit_deck_preview.py",
            "html",
            evidence,
            workdir,
            python_executable,
            repo_root,
        )
    if action_id == "mac_continuation_pulse":
        return CommandPlan(
            action_id,
            "Mac Continuation Pulse",
            [
                python_executable,
                _script("garnet_mac_side_continuation_status.py", repo_root),
                "--format",
                "markdown",
            ],
            workdir,
            False,
            False,
            False,
        )
    if action_id == "proof_benchmark_status":
        return _report_command_plan(
            action_id,
            "Proof / Benchmark Status",
            "garnet_proof_benchmark_status.py",
            "markdown",
            evidence,
            workdir,
            python_executable,
            repo_root,
        )
    if action_id == "benchmark_no_run":
        return _report_command_plan(
            action_id,
            "Benchmark No-Run",
            "garnet_benchmark_no_run.py",
            "markdown",
            evidence,
            workdir,
            python_executable,
            repo_root,
        )
    if action_id == "notarization_status":
        return CommandPlan(
            action_id,
            "Notarization Status",
            [
                python_executable,
                _script("garnet_studio_notarization_status.py", repo_root),
                "--format",
                "markdown",
            ],
            workdir,
            False,
            False,
            False,
        )
    if action_id == "windows_vm_installer_status":
        return CommandPlan(
            action_id,
            "Windows VM Installer Status",
            [
                python_executable,
                _script("garnet_windows_clean_vm_installer_status.py", repo_root),
                "--format",
                "markdown",
            ],
            workdir,
            False,
            False,
            False,
        )
    raise CommandContractError(f"unknown Studio action: {action_id}")


def _report_command_plan(
    action_id: str,
    label: str,
    script_name: str,
    output_format: str,
    evidence: str,
    workdir: str,
    python_executable: str,
    repo_root: Path,
    executes_source_code: bool = False,
) -> CommandPlan:
    return CommandPlan(
        action_id,
        label,
        [
            python_executable,
            _script(script_name, repo_root),
            "--output-dir",
            evidence,
            "--format",
            output_format,
        ],
        workdir,
        False,
        False,
        executes_source_code,
    )


def _sample_actions() -> list[StudioAction]:
    source = ROOT / "examples" / "mvp_01_os_simulator.garnet"
    foreign_source = ROOT / "tmp" / "sample.py"
    evidence = default_dogfood_root() / "sample"
    bundle = evidence / "bundle"
    review = evidence / "review"
    samples = [
        build_command_plan("cli_health"),
        build_command_plan("parse", source=source),
        build_command_plan("check", source=source),
        build_command_plan("run", source=source),
        build_command_plan("convert", language="python", source=foreign_source, evidence_dir=evidence),
        build_command_plan("assist_plan", language="typescript", source=foreign_source, evidence_dir=evidence),
        build_command_plan("advisory_bundle", language="typescript", source=foreign_source, evidence_dir=evidence),
        build_command_plan("advisory_review", bundle_dir=bundle, evidence_dir=evidence),
        build_command_plan("advisory_handoff", bundle_dir=bundle, review_dir=review, evidence_dir=evidence),
        build_command_plan("objective_pulse"),
        build_command_plan("agentic_dogfood_matrix"),
        build_command_plan("domain_proof_matrix", evidence_dir=evidence),
        build_command_plan("windows_linux_studio_status"),
        build_command_plan("converter_status"),
        build_command_plan("provider_options", evidence_dir=evidence),
        build_command_plan("mit_demo_route", evidence_dir=evidence),
        build_command_plan("mit_deck_outline", evidence_dir=evidence),
        build_command_plan("mit_deck_preview", evidence_dir=evidence),
        build_command_plan("mac_continuation_pulse"),
        build_command_plan("proof_benchmark_status", evidence_dir=evidence),
        build_command_plan("benchmark_no_run", evidence_dir=evidence),
        build_command_plan("notarization_status"),
        build_command_plan("windows_vm_installer_status"),
    ]
    groups = {
        "cli_health": "runtime",
        "parse": "active CLI",
        "check": "active CLI",
        "run": "active CLI",
        "convert": "active conversion",
        "assist_plan": "advisory planning",
        "advisory_bundle": "advisory planning",
        "advisory_review": "advisory planning",
        "advisory_handoff": "advisory planning",
        "objective_pulse": "release evidence",
        "agentic_dogfood_matrix": "dogfood evidence",
        "domain_proof_matrix": "runtime evidence",
        "windows_linux_studio_status": "release evidence",
        "converter_status": "converter evidence",
        "provider_options": "advisory planning",
        "mit_demo_route": "MIT evidence",
        "mit_deck_outline": "MIT evidence",
        "mit_deck_preview": "MIT evidence",
        "mac_continuation_pulse": "platform boundary",
        "proof_benchmark_status": "proof evidence",
        "benchmark_no_run": "proof evidence",
        "notarization_status": "platform boundary",
        "windows_vm_installer_status": "release evidence",
    }
    surfaces = {
        "cli_health": "garnet CLI version probe",
        "parse": "garnet parse",
        "check": "garnet check",
        "run": "garnet run",
        "convert": "garnet convert for Rust/Ruby/Python/Go only",
        "assist_plan": "scripts/garnet_converter_assist_plan.py",
        "advisory_bundle": "scripts/garnet_converter_advisory_bundle.py",
        "advisory_review": "scripts/garnet_converter_advisory_review.py",
        "advisory_handoff": "scripts/garnet_converter_advisory_handoff.py",
        "objective_pulse": "scripts/garnet_mit_readiness_status.py",
        "agentic_dogfood_matrix": "scripts/run_agentic_dogfood_matrix.py",
        "domain_proof_matrix": "scripts/smoke_garnet_studio_domain_matrix.py",
        "windows_linux_studio_status": "scripts/garnet_windows_linux_studio_status.py",
        "converter_status": "scripts/garnet_converter_status.py",
        "provider_options": "scripts/garnet_converter_llm_feasibility.py",
        "mit_demo_route": "scripts/garnet_mit_demo_route.py",
        "mit_deck_outline": "scripts/garnet_mit_deck_outline.py",
        "mit_deck_preview": "scripts/garnet_mit_deck_preview.py",
        "mac_continuation_pulse": "scripts/garnet_mac_side_continuation_status.py",
        "proof_benchmark_status": "scripts/garnet_proof_benchmark_status.py",
        "benchmark_no_run": "scripts/garnet_benchmark_no_run.py",
        "notarization_status": "scripts/garnet_studio_notarization_status.py",
        "windows_vm_installer_status": "scripts/garnet_windows_clean_vm_installer_status.py",
    }
    return [
        StudioAction(
            id=plan.action_id,
            label=plan.label,
            group=groups[plan.action_id],
            implementation_surface=surfaces[plan.action_id],
            current_command=plan.argv,
            evidence_required=plan.action_id not in {"cli_health", "parse", "check", "run"},
            source_included_by_default=plan.source_included_by_default,
            status="tauri-v2-command-wired",
        )
        for plan in samples
    ]


def read_status(clean_vm_evidence_root: Path | None = None) -> WindowsLinuxStudioStatus:
    clean_vm_root = clean_vm_evidence_root or _clean_vm_evidence_root_from_env()
    clean_vm = garnet_windows_clean_vm_installer_status.read_status(clean_vm_root)
    clean_vm_verified = clean_vm.clean_vm_verified
    linux_deb = smoke_garnet_studio_linux_wsl_deb.read_committed_evidence(ROOT)
    linux_deb_install = smoke_garnet_studio_linux_wsl_deb_install.read_committed_evidence(ROOT)
    linux_rpm = smoke_garnet_studio_linux_wsl_rpm.read_committed_evidence(ROOT)
    linux_xvfb = smoke_garnet_studio_linux_wsl_xvfb.read_committed_evidence(ROOT)
    linux_xvfb_window = smoke_garnet_studio_linux_wsl_xvfb_window.read_committed_evidence(ROOT)
    linux_wslg_install = smoke_garnet_studio_linux_wslg_install_launch.read_committed_evidence(ROOT)
    any_linux_package_evidence = (
        linux_deb.verified
        or linux_deb_install.verified
        or linux_rpm.verified
        or linux_xvfb.verified
        or linux_xvfb_window.verified
        or linux_wslg_install.verified
    )
    linux_status_suffix = (
        "wslg-system-install-launch-verified-linux-desktop-still-open"
        if linux_wslg_install.verified
        else "wsl-deb-rpm-xvfb-window-capture-verified-linux-desktop-still-open"
        if linux_xvfb_window.verified and linux_xvfb.verified and linux_deb_install.verified and linux_rpm.verified
        else "wsl-xvfb-window-capture-verified-linux-desktop-still-open"
        if linux_xvfb_window.verified
        else
        "wsl-deb-rpm-xvfb-runtime-verified-linux-desktop-still-open"
        if linux_xvfb.verified and linux_deb_install.verified and linux_rpm.verified
        else "wsl-rpm-xvfb-runtime-verified-linux-desktop-still-open"
        if linux_xvfb.verified and linux_rpm.verified
        else "wsl-xvfb-runtime-verified-linux-desktop-still-open"
        if linux_xvfb.verified
        else
        "wsl-deb-rpm-extract-verified-linux-gui-still-open"
        if linux_deb_install.verified and linux_rpm.verified
        else "wsl-rpm-extract-verified-linux-gui-still-open"
        if linux_rpm.verified
        else "wsl-deb-install-verified-linux-gui-still-open"
        if linux_deb_install.verified
        else "wsl-deb-package-verified-linux-gui-still-open"
        if linux_deb.verified
        else "linux-open"
    )
    status = (
        f"tauri-v2-shell-v0-5-readiness-parity-windows-clean-vm-verified-{linux_status_suffix}"
        if clean_vm_verified
        else f"tauri-v2-shell-v0-5-readiness-parity-windows-clean-vm-contract-open-{linux_status_suffix}"
    )
    clean_vm_truth = (
        [
            "Windows clean-VM installer proof is verified for the unsigned x64 NSIS artifact under `scripts/garnet_windows_clean_vm_installer_status.py`",
            "Windows unsigned x64 clean-machine proof exists; signed MSI, winget, Windows ARM64, and Linux runtime proof remain separate gates",
        ]
        if clean_vm_verified
        else [
            "Windows clean-VM installer proof now has a repo-owned evidence contract and status reporter; it is not verified until a fresh VM bundle is recorded",
            "Windows clean-machine installer proof remains open until the unsigned NSIS artifact is exercised in a fresh VM",
        ]
    )
    linux_package_truth = []
    if linux_deb.verified:
        linux_package_truth.append(
            "WSL Linux `.deb` package build and non-GUI `--studio-smoke` are verified by `scripts/smoke_garnet_studio_linux_wsl_deb.py`; Linux desktop GUI install/launch remains open",
        )
    if linux_deb_install.verified:
        linux_package_truth.append(
            "WSL Linux `.deb` package extract and extracted-binary non-GUI `--studio-smoke` are verified by `scripts/smoke_garnet_studio_linux_wsl_deb_install.py`; clean Linux install and desktop GUI launch remain open",
        )
    if linux_rpm.verified:
        linux_package_truth.append(
            "WSL Linux `.rpm` package extract and extracted-binary non-GUI `--studio-smoke` are verified by `scripts/smoke_garnet_studio_linux_wsl_rpm.py`; clean Linux install, privileged system package install, and desktop GUI launch remain open",
        )
    if linux_xvfb.verified:
        linux_package_truth.append(
            "WSL Linux Xvfb runtime-start is verified by `scripts/smoke_garnet_studio_linux_wsl_xvfb.py`; this proves the extracted Linux Studio process stays alive under a virtual X display until timeout, but it is not Linux desktop GUI launch proof, clean Linux install proof, privileged package install proof, Linux seccomp, or OS-sandbox enforcement",
        )
    if linux_xvfb_window.verified:
        linux_package_truth.append(
            "WSL Linux Xvfb virtual-display window capture is verified by `scripts/smoke_garnet_studio_linux_wsl_xvfb_window.py`; this proves the extracted Linux Studio process creates an observable `Garnet Studio` X11 window tree and screenshot artifact under Xvfb, but it is not Linux desktop GUI launch proof, clean Linux install proof, privileged package install proof, Linux seccomp, or OS-sandbox enforcement",
        )
    if linux_wslg_install.verified:
        linux_package_truth.append(
            "WSLg system package install and installed-binary GUI launch are verified by `scripts/smoke_garnet_studio_linux_wslg_install_launch.py`; this proves a privileged WSL `.deb` install plus WSLg window observation, but it is not clean Linux install proof, non-WSL Linux desktop proof, Linux seccomp, or OS-sandbox enforcement",
        )
    if not linux_package_truth:
        linux_package_truth.append(
            "WSL Linux `.deb` package build proof remains open until `scripts/smoke_garnet_studio_linux_wsl_deb.py --record` verifies the bundle",
        )
    linux_package_gate = PackagingGate(
        id="linux_package_choice",
        platform="Linux",
        status=(
            "wslg-system-install-launch-verified"
            if linux_wslg_install.verified
            else "wsl-deb-rpm-xvfb-window-capture-verified"
            if linux_xvfb_window.verified and linux_xvfb.verified and linux_deb_install.verified and linux_rpm.verified
            else "wsl-xvfb-window-capture-verified"
            if linux_xvfb_window.verified
            else "wsl-deb-rpm-xvfb-runtime-start-verified"
            if linux_xvfb.verified and linux_deb_install.verified and linux_rpm.verified
            else "wsl-rpm-xvfb-runtime-start-verified"
            if linux_xvfb.verified and linux_rpm.verified
            else "wsl-xvfb-runtime-start-verified"
            if linux_xvfb.verified
            else "wsl-deb-rpm-extract-command-smoke-verified"
            if linux_deb_install.verified and linux_rpm.verified
            else "wsl-rpm-extract-command-smoke-verified"
            if linux_rpm.verified
            else "wsl-deb-extract-command-smoke-verified"
            if linux_deb_install.verified
            else "wsl-deb-package-build-smoke-verified"
            if linux_deb.verified
            else "open"
        ),
        next_evidence=(
            "Run/install the package in a non-WSL real Linux desktop session and capture clean GUI launch evidence"
            if linux_wslg_install.verified
            else "Run/install the WSL-built .deb/.rpm in a real Linux desktop session and capture GUI launch evidence"
            if any_linux_package_evidence
            else "choose AppImage-first, .deb/.rpm, Flatpak, or source/PWA shell after target smoke"
        ),
        forbidden_claim=(
            "clean Linux or non-WSL Linux desktop GUI package install/launch is verified"
            if linux_wslg_install.verified
            else "Linux desktop GUI package install/launch is verified"
            if any_linux_package_evidence
            else "Linux Studio package is verified"
        ),
    )
    unsigned_nsis_gate = PackagingGate(
        id="windows_unsigned_nsis",
        platform="Windows",
        status="clean-vm-proof-verified" if clean_vm_verified else "contract-ready-clean-vm-open",
        next_evidence=(
            "Preserve the verified clean Windows VM proof bundle; proceed to signed MSI/AuthentiCode and winget only after signing/public-release evidence exists"
            if clean_vm_verified
            else "Record a clean Windows VM proof bundle with `scripts/garnet_windows_clean_vm_installer_status.py --record-proof --mode clean-vm ...`"
        ),
        forbidden_claim=(
            "signed Windows MSI or winget path is verified"
            if clean_vm_verified
            else "signed or clean-machine Windows installer is verified"
        ),
    )
    linux_runtime_next = (
        "Linux desktop GUI install/launch proof for the WSL-built .deb/.rpm or chosen target package"
        if any_linux_package_evidence
        else "Linux desktop launch proof and first package-format decision"
    )
    next_slices = (
        [
            linux_runtime_next,
            "Windows ARM64 target build/smoke after x64 clean-VM proof",
            "Domain Proof Matrix screenshots/output from the Windows shell and WSL/Linux shell",
            "Active converter end-to-end screenshots from the shell",
            "Advisory bundle/review/handoff evidence walkthrough without source inclusion",
            "Release / Readiness panel screenshot and reporter-output evidence from the Windows shell",
            "Signed Windows MSI/AuthentiCode plan after verified unsigned VM smoke",
            "Website/status copy sync after target smoke evidence",
        ]
        if clean_vm_verified
        else [
            "Windows clean-machine NSIS install and CLI smoke evidence",
            linux_runtime_next,
            "Domain Proof Matrix screenshots/output from the Windows shell and WSL/Linux shell",
            "Active converter end-to-end screenshots from the shell",
            "Advisory bundle/review/handoff evidence walkthrough without source inclusion",
            "Release / Readiness panel screenshot and reporter-output evidence from the Windows shell",
            "Unsigned-to-signed Windows MSI/AuthentiCode plan after VM smoke",
            "Website/status copy sync after target smoke evidence",
        ]
    )
    clean_vm_assistance = (
        []
        if clean_vm_verified
        else [
            *clean_vm.blocked_by,
            "Run the unsigned NSIS installer in a clean Windows VM for installer/runtime launch evidence",
        ]
    )
    return WindowsLinuxStudioStatus(
        source=str(ROOT),
        status=status,
        current_truth=[
            "origin/main is newer than PR #140; live main remains the source of truth",
            "macOS SwiftUI Studio remains the native Apple reference app",
            "Tauri v2 is now adopted for the first Windows/Linux shell scaffold in `apps/garnet-studio`",
            "Windows local source-build proof exists for the Tauri frontend, backend tests, release executable, unsigned NSIS bundle, and `--studio-smoke` evidence",
            *clean_vm_truth,
            *linux_package_truth,
            "Linux runtime proof is not complete until the shell launches in a Linux desktop environment",
            "the shell wraps existing CLI, docs/PWA, advisory scripts, and dogfood gates without duplicating converter logic",
            "CLI Health maps to the existing `garnet version` probe unless a real health subcommand is added",
            "the Domain Proof Matrix runs the current canonical MVP and agentic examples through `garnet parse`, `garnet check`, and `garnet run`, writing manifest-backed evidence without source inclusion",
            "the Release / Readiness panel now exposes the repo-native v0.5 reporters used by the broader MIT/productization story",
            "provider options remain advisory-only; provider-backed conversion is not active",
            "benchmark no-run evidence is compile/status evidence only and does not claim performance measurements",
            "notarization status is a Mac-side preflight boundary, not a Windows completion claim",
            "Windows x64 is the first clean-VM Studio installer target; Windows ARM64 follows after x64 proof; 32-bit Windows is deferred until demand justifies the QA surface",
        ],
        least_new_dependency_decision=(
            "Tauri v2 is accepted for the first shell scaffold. The scaffold keeps webview permissions minimal "
            "(`core:default` only), removes the Tauri shell plugin, and delegates privileged execution to typed Rust commands."
        ),
        taxonomy=LanguageTaxonomy(
            active_conversion=ACTIVE_CONVERSION,
            advisory_planning=ADVISORY_PLANNING,
            native_boundary_recommended=NATIVE_BOUNDARY_RECOMMENDED,
            future_backend_lowering=FUTURE_BACKEND_LOWERING,
        ),
        safety_contract=SafetyContract(
            calls_provider_apis_by_default=False,
            executes_source_code=False,
            includes_source_by_default=False,
            marks_advisory_output_safe=False,
            required_handoff_gates=[
                "lineage",
                "@sandbox",
                "migrate_todo",
                "garnet check",
                "dogfood evidence",
                "human audit",
            ],
            forbidden_claims=[
                "Windows/Linux Studio runtime completion",
                "signed Windows MSI for Studio",
                "Linux Studio package completion",
                "provider-backed conversion is active",
                "advisory output is safe",
                "broad deterministic planned-language frontends are active",
            ],
        ),
        evidence_contract=EvidenceContract(
            default_root=str(default_dogfood_root()),
            bundle_prefix="garnet-studio-windows-linux-",
            created_by_default=True,
            source_included_by_default=False,
            required_files=[
                "garnet-windows-linux-studio-evidence-contract.json",
                "MANIFEST.sha256",
                "command stdout/stderr logs for invoked actions",
                "domain proof matrix JSON/Markdown plus per-command stdout/stderr logs when invoked",
                "screenshots once a shell exists",
                "readiness reporter stdout/stderr logs for invoked release actions",
            ],
        ),
        actions=_sample_actions(),
        packaging_gates=[
            PackagingGate(
                id="windows_msvc_build",
                platform="Windows",
                status="local-pass",
                next_evidence="Preserve PR evidence for cargo build --release, Tauri release executable, and Studio smoke bundle",
                forbidden_claim="Windows packaged Studio build is complete",
            ),
            unsigned_nsis_gate,
            PackagingGate(
                id="windows_target_architecture_matrix",
                platform="Windows",
                status="documented-first-target",
                next_evidence="x64 clean-VM proof first, then ARM64 target build/smoke; 32-bit remains deferred until demand",
                forbidden_claim="all Windows architectures are packaged and verified",
            ),
            PackagingGate(
                id="windows_msi_signing",
                platform="Windows",
                status="open",
                next_evidence="MSI plan, Authenticode signing, signtool verification, and clean-machine smoke",
                forbidden_claim="signed MSI is available before certificate and smoke evidence exist",
            ),
            PackagingGate(
                id="windows_winget",
                platform="Windows",
                status="open",
                next_evidence="winget manifest after a signed public release artifact exists",
                forbidden_claim="winget install path is verified",
            ),
            PackagingGate(
                id=linux_package_gate.id,
                platform=linux_package_gate.platform,
                status=linux_package_gate.status,
                next_evidence=linux_package_gate.next_evidence,
                forbidden_claim=linux_package_gate.forbidden_claim,
            ),
        ],
        next_slices=next_slices,
        user_assistance_needed=[
            *clean_vm_assistance,
            "Provide a Linux VM/container with GUI or AppImage-capable desktop session for runtime launch evidence",
            "Provide signing credentials only when ready to verify signed MSI claims",
        ],
    )


def render_markdown(status: WindowsLinuxStudioStatus) -> str:
    lines = [
        "# Garnet Windows/Linux Studio Status",
        "",
        f"Source: `{status.source}`",
        f"Status: `{status.status}`",
        "",
        "## Current Truth",
        "",
        *[f"- {truth}" for truth in status.current_truth],
        "",
        "## Least-New-Dependency Decision",
        "",
        status.least_new_dependency_decision,
        "",
        "## Language Taxonomy",
        "",
        "| Menu group | Languages |",
        "| --- | --- |",
        f"| Active conversion | {', '.join(status.taxonomy.active_conversion)} |",
        f"| Advisory planning | {', '.join(status.taxonomy.advisory_planning)} |",
        f"| Native boundary recommended | {', '.join(status.taxonomy.native_boundary_recommended)} |",
        f"| Future backend lowering | {', '.join(status.taxonomy.future_backend_lowering)} |",
        "",
        "## Actions",
        "",
        "| Action | Group | Surface | Status |",
        "| --- | --- | --- | --- |",
    ]
    for action in status.actions:
        lines.append(
            f"| {action.label} | {action.group} | `{action.implementation_surface}` | `{action.status}` |"
        )
    lines.extend(
        [
            "",
            "## Safety Contract",
            "",
            f"- Provider APIs by default: `{str(status.safety_contract.calls_provider_apis_by_default).lower()}`",
            f"- Source execution by default: `{str(status.safety_contract.executes_source_code).lower()}`",
            f"- Source included by default: `{str(status.safety_contract.includes_source_by_default).lower()}`",
            f"- Advisory output marked safe: `{str(status.safety_contract.marks_advisory_output_safe).lower()}`",
            f"- Required gates: {', '.join(status.safety_contract.required_handoff_gates)}",
            "",
            "## Evidence Contract",
            "",
            f"- Default root: `{status.evidence_contract.default_root}`",
            f"- Bundle prefix: `{status.evidence_contract.bundle_prefix}`",
            f"- Source included by default: `{str(status.evidence_contract.source_included_by_default).lower()}`",
            "",
            "## Packaging Gates",
            "",
            "| Gate | Platform | Status | Next evidence |",
            "| --- | --- | --- | --- |",
        ]
    )
    for gate in status.packaging_gates:
        lines.append(f"| {gate.id} | {gate.platform} | `{gate.status}` | {gate.next_evidence} |")
    lines.extend(
        [
            "",
            "## User Assistance Needed",
            "",
            *[f"- {item}" for item in status.user_assistance_needed],
        ]
    )
    return "\n".join(lines) + "\n"


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=("markdown", "json"), default="markdown")
    parser.add_argument("--output-dir", type=Path, help="write status JSON/Markdown plus MANIFEST.sha256")
    parser.add_argument(
        "--create-evidence-bundle",
        action="store_true",
        help="create the default Windows/Linux Studio Desktop dogfood bundle contract",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    status = read_status()
    if args.create_evidence_bundle:
        bundle = create_evidence_bundle()
        print(f"Created evidence bundle: {bundle}", file=sys.stderr)
        if args.output_dir is None:
            args.output_dir = bundle
    if args.output_dir:
        args.output_dir.mkdir(parents=True, exist_ok=True)
        (args.output_dir / "garnet-windows-linux-studio-status.json").write_text(
            json.dumps(asdict(status), indent=2) + "\n",
            encoding="utf-8",
        )
        (args.output_dir / "garnet-windows-linux-studio-status.md").write_text(
            render_markdown(status),
            encoding="utf-8",
        )
        _write_manifest(args.output_dir)
    if args.format == "json":
        print(json.dumps(asdict(status), indent=2))
    else:
        print(render_markdown(status), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
