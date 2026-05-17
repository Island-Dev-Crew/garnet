#!/usr/bin/env python3
"""Report the Windows/Linux Garnet Studio MVP contract.

This is the target-platform counterpart to the Mac-side continuation pulse. It
does not claim a completed desktop shell. It records the first Windows/Linux
Studio slice as a verified contract: action inventory, safe command
construction, Desktop dogfood evidence defaults, copy-truth taxonomy, and
packaging gates that a future Tauri/PWA shell must satisfy.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Sequence

ROOT = Path(__file__).resolve().parents[1]

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
        "status": "contract-only",
        "required_runtime_proof": [
            "launch on Windows",
            "launch on Linux",
            "locate or bundle garnet",
            "parse/check/run local examples",
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
        advisory = _require_language(language, ACTIVE_CONVERSION + ADVISORY_PLANNING, "language")
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
        advisory = _require_language(language, ACTIVE_CONVERSION + ADVISORY_PLANNING, "language")
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
    raise CommandContractError(f"unknown Studio action: {action_id}")


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
            status="contracted-not-yet-shell-wired",
        )
        for plan in samples
    ]


def read_status() -> WindowsLinuxStudioStatus:
    return WindowsLinuxStudioStatus(
        source=str(ROOT),
        status="contract-only-runtime-proof-open",
        current_truth=[
            "origin/main is newer than PR #140; live main remains the source of truth",
            "macOS SwiftUI Studio remains the native Apple reference app",
            "Windows/Linux Studio runtime proof is not complete until it launches on target systems",
            "the first shell must wrap existing CLI, docs/PWA, advisory scripts, and dogfood gates",
            "Tauri is a candidate dependency, not an adopted dependency in this slice",
            "CLI Health maps to the existing `garnet version` probe unless a real health subcommand is added",
        ],
        least_new_dependency_decision=(
            "Start with this no-new-dependency command/evidence contract. Add Tauri v2 only "
            "after dependency review and Windows/Linux build verification; otherwise keep a PWA-first shell."
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
                "screenshots once a shell exists",
            ],
        ),
        actions=_sample_actions(),
        packaging_gates=[
            PackagingGate(
                id="windows_msvc_build",
                platform="Windows",
                status="open",
                next_evidence="cargo build --release --target x86_64-pc-windows-msvc for CLI plus future shell",
                forbidden_claim="Windows packaged Studio build is complete",
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
                id="linux_package_choice",
                platform="Linux",
                status="open",
                next_evidence="choose AppImage-first, .deb/.rpm, Flatpak, or source/PWA shell after target smoke",
                forbidden_claim="Linux Studio package is verified",
            ),
        ],
        next_slices=[
            "Minimal shell scaffold wired to these command plans",
            "CLI health plus parse/check/run action execution",
            "Active convert and advisory planning UI with source omitted by default",
            "Desktop dogfood evidence browser plus manifest checks",
            "Windows package smoke and copy review",
            "Linux package smoke and copy review",
        ],
        user_assistance_needed=[
            "Provide a real Windows machine or Windows VM for runtime launch evidence",
            "Provide a Linux VM/container with GUI or AppImage-capable desktop session for runtime launch evidence",
            "Provide signing credentials only when ready to verify signed MSI claims",
            "Confirm whether Tauri v2 is acceptable after this no-new-dependency contract passes",
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
