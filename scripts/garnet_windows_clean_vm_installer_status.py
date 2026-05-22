#!/usr/bin/env python3
"""Report and record Windows clean-VM installer proof for Garnet Studio.

This script is intentionally evidence-first. It can record a clean Windows VM
proof bundle from an already-produced installer/log/screenshot set, and it can
summarize the latest bundle for Studio and MIT-readiness panels. It does not
run an installer on the current machine and it does not upgrade unsigned NSIS
proof into signed MSI, winget, or clean-machine completion without the required
fresh-guest evidence.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = "garnet.windows_studio.clean_vm_installer_proof.v1"

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")


@dataclass(frozen=True)
class PackageTarget:
    id: str
    platform: str
    architecture: str
    rust_target: str
    package_surface: str
    status: str
    proof_required: str


@dataclass(frozen=True)
class SmokeGate:
    id: str
    label: str
    status: str
    evidence: str


@dataclass(frozen=True)
class ProofRecord:
    schema: str
    created_at: str
    mode: str
    verified: bool
    installer_path: str
    installer_sha256: str
    vm_name: str
    guest_os: str
    guest_arch: str
    install_log: str
    studio_smoke_json: str
    screenshot: str
    gates: list[SmokeGate]
    forbidden_claims: list[str]


@dataclass(frozen=True)
class WindowsCleanVmInstallerStatus:
    source: str
    status: str
    default_evidence_root: str
    clean_vm_verified: bool
    current_truth: list[str]
    package_targets: list[PackageTarget]
    required_gates: list[SmokeGate]
    latest_proof: ProofRecord | None
    blocked_by: list[str]
    forbidden_claims: list[str]


def default_evidence_root(home: Path | None = None) -> Path:
    base = home or Path.home()
    return base / "Desktop" / "dogfood" / "garnet-studio-windows-clean-vm"


def timestamp_slug(now: datetime | None = None) -> str:
    current = now or datetime.now(timezone.utc)
    return current.strftime("%Y%m%d-%H%M%S")


def package_targets() -> list[PackageTarget]:
    return [
        PackageTarget(
            id="studio-windows-x64-nsis",
            platform="Windows",
            architecture="x64",
            rust_target="x86_64-pc-windows-msvc",
            package_surface="Tauri NSIS setup executable",
            status="first-clean-vm-target",
            proof_required="Build on Windows, install unsigned NSIS in a fresh x64 VM, launch, and run --studio-smoke.",
        ),
        PackageTarget(
            id="studio-windows-arm64-nsis",
            platform="Windows",
            architecture="ARM64",
            rust_target="aarch64-pc-windows-msvc",
            package_surface="Tauri NSIS setup executable",
            status="planned-after-x64-proof",
            proof_required="Install ARM64 Rust/MSVC components, build with --target aarch64-pc-windows-msvc, and smoke on Windows ARM64 hardware or VM.",
        ),
        PackageTarget(
            id="studio-windows-x86-nsis",
            platform="Windows",
            architecture="32-bit x86",
            rust_target="i686-pc-windows-msvc",
            package_surface="Tauri NSIS setup executable",
            status="deferred-until-user-demand",
            proof_required="Only add when product demand justifies WebView2, installer, and clean-VM QA for 32-bit Windows.",
        ),
        PackageTarget(
            id="studio-linux-x64",
            platform="Linux",
            architecture="x64",
            rust_target="x86_64-unknown-linux-gnu",
            package_surface="AppImage, .deb, or .rpm decision pending",
            status="runtime-open",
            proof_required="Launch the Tauri shell in a Linux desktop session and run CLI plus advisory evidence smoke.",
        ),
        PackageTarget(
            id="studio-linux-arm64",
            platform="Linux",
            architecture="ARM64",
            rust_target="aarch64-unknown-linux-gnu",
            package_surface="source/PWA shell first; package later",
            status="planned-after-x64-linux-proof",
            proof_required="Select package surface after x64 Linux launch proof and validate GUI dependencies on ARM64.",
        ),
        PackageTarget(
            id="studio-macos-reference",
            platform="macOS",
            architecture="Apple Silicon and Intel",
            rust_target="aarch64-apple-darwin / x86_64-apple-darwin",
            package_surface="SwiftUI Studio reference app, not the Windows/Linux Tauri shell",
            status="separate-apple-lane",
            proof_required="Keep macOS notarization and DMG evidence separate from Windows/Linux Studio claims.",
        ),
    ]


def required_gates() -> list[SmokeGate]:
    return [
        SmokeGate(
            id="installer-artifact",
            label="Installer artifact exists and is SHA-256 identified",
            status="required",
            evidence="Unsigned NSIS setup executable path plus SHA-256 digest.",
        ),
        SmokeGate(
            id="fresh-guest",
            label="Fresh Windows guest identity is recorded",
            status="required",
            evidence="VM name, guest OS, guest architecture, and clean-VM mode.",
        ),
        SmokeGate(
            id="install-log",
            label="Installer run log is preserved",
            status="required",
            evidence="Install transcript or command log from inside the guest.",
        ),
        SmokeGate(
            id="studio-smoke",
            label="Installed Studio writes no-GUI smoke evidence",
            status="required",
            evidence="studio-smoke.json with status=passed, source_included=false, provider_api_called=false.",
        ),
        SmokeGate(
            id="launch-screenshot",
            label="Installed Studio launch screenshot is preserved",
            status="required",
            evidence="Screenshot from the clean VM after installed app launch.",
        ),
        SmokeGate(
            id="claim-boundary",
            label="Unsigned installer proof remains separate from signed MSI and winget",
            status="required",
            evidence="Proof bundle forbidden_claims keeps signed MSI, winget, and Linux package claims open.",
        ),
    ]


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _path_status(path_text: str, label: str) -> SmokeGate:
    if not path_text:
        return SmokeGate(label, label.replace("-", " ").title(), "blocked", "missing path")
    path = Path(path_text)
    if path.exists():
        return SmokeGate(label, label.replace("-", " ").title(), "pass", str(path))
    return SmokeGate(label, label.replace("-", " ").title(), "blocked", f"missing file: {path}")


def _load_smoke_json(path_text: str) -> dict[str, object]:
    if not path_text:
        return {}
    path = Path(path_text)
    if not path.exists():
        return {}
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return {}


def build_proof_record(
    *,
    mode: str,
    installer: Path | None,
    vm_name: str,
    guest_os: str,
    guest_arch: str,
    install_log: Path | None,
    studio_smoke_json: Path | None,
    screenshot: Path | None,
    now: datetime | None = None,
) -> ProofRecord:
    installer_path = str(installer) if installer else ""
    install_log_path = str(install_log) if install_log else ""
    smoke_path = str(studio_smoke_json) if studio_smoke_json else ""
    screenshot_path = str(screenshot) if screenshot else ""
    smoke = _load_smoke_json(smoke_path)
    installer_hash = sha256_file(installer) if installer and installer.exists() else ""

    installer_gate = _path_status(installer_path, "installer-artifact")
    fresh_guest_gate = SmokeGate(
        "fresh-guest",
        "Fresh Guest",
        "pass" if mode == "clean-vm" and guest_os and guest_arch else "blocked",
        f"mode={mode}; vm={vm_name or '(missing)'}; os={guest_os or '(missing)'}; arch={guest_arch or '(missing)'}",
    )
    install_log_gate = _path_status(install_log_path, "install-log")
    smoke_passed = (
        smoke.get("status") == "passed"
        and smoke.get("source_included") is False
        and smoke.get("provider_api_called") is False
    )
    smoke_gate = SmokeGate(
        "studio-smoke",
        "Studio Smoke",
        "pass" if smoke_passed else "blocked",
        smoke_path or "missing studio-smoke.json",
    )
    screenshot_gate = _path_status(screenshot_path, "launch-screenshot")
    claim_gate = SmokeGate(
        "claim-boundary",
        "Claim Boundary",
        "pass",
        "signed MSI, winget, Linux package, and provider-backed conversion remain forbidden claims.",
    )
    gates = [
        installer_gate,
        fresh_guest_gate,
        install_log_gate,
        smoke_gate,
        screenshot_gate,
        claim_gate,
    ]
    verified = all(gate.status == "pass" for gate in gates)
    return ProofRecord(
        schema=SCHEMA,
        created_at=(now or datetime.now(timezone.utc)).isoformat(),
        mode=mode,
        verified=verified,
        installer_path=installer_path,
        installer_sha256=installer_hash,
        vm_name=vm_name,
        guest_os=guest_os,
        guest_arch=guest_arch,
        install_log=install_log_path,
        studio_smoke_json=smoke_path,
        screenshot=screenshot_path,
        gates=gates,
        forbidden_claims=forbidden_claims(),
    )


def forbidden_claims() -> list[str]:
    return [
        "signed Windows MSI is available",
        "winget install path is verified",
        "Windows clean-machine proof exists without clean-VM evidence",
        "Linux Studio package is verified",
        "provider-backed conversion is active",
    ]


def _write_manifest(directory: Path) -> None:
    lines = []
    for path in sorted(directory.iterdir()):
        if not path.is_file() or path.name == "MANIFEST.sha256":
            continue
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        lines.append(f"{digest}  {path.name}")
    (directory / "MANIFEST.sha256").write_text("\n".join(lines) + "\n", encoding="utf-8")


def write_proof(record: ProofRecord, output_dir: Path) -> Path:
    output_dir.mkdir(parents=True, exist_ok=True)
    path = output_dir / "windows-clean-vm-installer-proof.json"
    path.write_text(json.dumps(asdict(record), indent=2) + "\n", encoding="utf-8")
    _write_manifest(output_dir)
    return path


def latest_proof(evidence_root: Path | None = None) -> ProofRecord | None:
    root = evidence_root or default_evidence_root()
    if not root.exists():
        return None
    candidates = sorted(root.glob("*/windows-clean-vm-installer-proof.json"), key=lambda path: path.stat().st_mtime)
    if not candidates:
        return None
    data = json.loads(candidates[-1].read_text(encoding="utf-8"))
    gates = [SmokeGate(**gate) for gate in data.get("gates", [])]
    return ProofRecord(
        schema=data.get("schema", ""),
        created_at=data.get("created_at", ""),
        mode=data.get("mode", ""),
        verified=bool(data.get("verified", False)),
        installer_path=data.get("installer_path", ""),
        installer_sha256=data.get("installer_sha256", ""),
        vm_name=data.get("vm_name", ""),
        guest_os=data.get("guest_os", ""),
        guest_arch=data.get("guest_arch", ""),
        install_log=data.get("install_log", ""),
        studio_smoke_json=data.get("studio_smoke_json", ""),
        screenshot=data.get("screenshot", ""),
        gates=gates,
        forbidden_claims=list(data.get("forbidden_claims", [])),
    )


def read_status(evidence_root: Path | None = None) -> WindowsCleanVmInstallerStatus:
    proof = latest_proof(evidence_root)
    verified = bool(proof and proof.verified)
    return WindowsCleanVmInstallerStatus(
        source=str(ROOT),
        status="clean-vm-proof-verified" if verified else "proof-contract-ready-clean-vm-open",
        default_evidence_root=str(evidence_root or default_evidence_root()),
        clean_vm_verified=verified,
        current_truth=[
            "Windows x64 is the first Studio installer proof target because current Tauri NSIS evidence is x64-local.",
            "Windows ARM64 is a reasonable follow-up target, but it needs its own Rust/MSVC target install, build, and clean-machine smoke.",
            "Windows 32-bit remains deferred until user demand justifies separate WebView2 and installer QA.",
            "Linux Studio package format remains open until a Linux desktop launch proves the shell runtime.",
            "macOS Studio remains the separate SwiftUI Apple reference lane, not a Tauri port claim.",
            "This script records or reports installer proof; it does not run installers on the current host.",
        ],
        package_targets=package_targets(),
        required_gates=required_gates(),
        latest_proof=proof,
        blocked_by=[]
        if verified
        else [
            "clean Windows VM guest identity",
            "unsigned NSIS installer artifact digest",
            "installer run log from inside the guest",
            "installed Studio --studio-smoke JSON",
            "installed app launch screenshot",
        ],
        forbidden_claims=forbidden_claims(),
    )


def render_markdown(status: WindowsCleanVmInstallerStatus) -> str:
    lines = [
        "# Garnet Windows Studio Clean-VM Installer Status",
        "",
        f"Source: `{status.source}`",
        f"Status: `{status.status}`",
        f"Default evidence root: `{status.default_evidence_root}`",
        f"Clean VM verified: `{str(status.clean_vm_verified).lower()}`",
        "",
        "## Current Truth",
        "",
        *[f"- {item}" for item in status.current_truth],
        "",
        "## Package Target Posture",
        "",
        "| Target | Platform | Architecture | Rust target | Surface | Status |",
        "| --- | --- | --- | --- | --- | --- |",
    ]
    for target in status.package_targets:
        lines.append(
            f"| {target.id} | {target.platform} | {target.architecture} | `{target.rust_target}` | {target.package_surface} | `{target.status}` |"
        )
    lines.extend(
        [
            "",
            "## Required Gates",
            "",
            "| Gate | Status | Evidence |",
            "| --- | --- | --- |",
        ]
    )
    gates = status.latest_proof.gates if status.latest_proof else status.required_gates
    for gate in gates:
        lines.append(f"| {gate.label} | `{gate.status}` | {gate.evidence} |")
    if status.latest_proof:
        lines.extend(
            [
                "",
                "## Latest Proof",
                "",
                f"- Mode: `{status.latest_proof.mode}`",
                f"- VM: `{status.latest_proof.vm_name or '(not recorded)'}`",
                f"- Guest: `{status.latest_proof.guest_os or '(not recorded)'}` / `{status.latest_proof.guest_arch or '(not recorded)'}`",
                f"- Installer SHA-256: `{status.latest_proof.installer_sha256 or '(missing)'}`",
            ]
        )
    lines.extend(
        [
            "",
            "## Blocked By",
            "",
            *([f"- {item}" for item in status.blocked_by] or ["- None"]),
            "",
            "## Forbidden Claims",
            "",
            *[f"- {item}" for item in status.forbidden_claims],
        ]
    )
    return "\n".join(lines) + "\n"


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=("markdown", "json"), default="markdown")
    parser.add_argument("--output-dir", type=Path, help="write status/proof JSON and MANIFEST.sha256")
    parser.add_argument("--evidence-root", type=Path, help="read latest proof from this root")
    parser.add_argument("--record-proof", action="store_true", help="record a proof bundle from supplied evidence files")
    parser.add_argument("--mode", choices=("current-host", "clean-vm"), default="current-host")
    parser.add_argument("--installer", type=Path)
    parser.add_argument("--vm-name", default="")
    parser.add_argument("--guest-os", default="")
    parser.add_argument("--guest-arch", default="")
    parser.add_argument("--install-log", type=Path)
    parser.add_argument("--studio-smoke-json", type=Path)
    parser.add_argument("--screenshot", type=Path)
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    output_dir = args.output_dir
    if args.record_proof:
        if output_dir is None:
            output_dir = default_evidence_root() / f"garnet-studio-windows-clean-vm-{timestamp_slug()}"
        record = build_proof_record(
            mode=args.mode,
            installer=args.installer,
            vm_name=args.vm_name,
            guest_os=args.guest_os,
            guest_arch=args.guest_arch,
            install_log=args.install_log,
            studio_smoke_json=args.studio_smoke_json,
            screenshot=args.screenshot,
        )
        write_proof(record, output_dir)
        args.evidence_root = output_dir.parent

    status = read_status(args.evidence_root)
    if output_dir:
        output_dir.mkdir(parents=True, exist_ok=True)
        (output_dir / "windows-clean-vm-installer-status.json").write_text(
            json.dumps(asdict(status), indent=2) + "\n",
            encoding="utf-8",
        )
        (output_dir / "windows-clean-vm-installer-status.md").write_text(
            render_markdown(status),
            encoding="utf-8",
        )
        _write_manifest(output_dir)

    if args.format == "json":
        print(json.dumps(asdict(status), indent=2))
    else:
        print(render_markdown(status), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
