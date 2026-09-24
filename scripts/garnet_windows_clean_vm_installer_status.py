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
import os
import re
import stat
import sys
from dataclasses import asdict, dataclass, replace
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = "garnet.windows_studio.clean_vm_installer_proof.v1"
PROOF_FILE = "windows-clean-vm-installer-proof.json"
# A bundle committed here is read on every host (T5a, path (a)); the Desktop
# dogfood root is only a local fallback when the repo carries no bundle.
COMMITTED_BUNDLES_REL = Path("proofs/windows/studio-clean-vm")
X64_GUEST_ARCHES = frozenset({"x64", "x86_64", "amd64"})
# Committed bundles are named <YYYYMMDD-HHMM>-<host>. Any entry whose name
# starts with a timestamp is a candidate; the newest decides, and it must carry
# the full name or it is reported (fail closed), never skipped.
BUNDLE_NAME = re.compile(r"^[0-9]{8}-[0-9]{4}")
BUNDLE_FULL_NAME = re.compile(r"[0-9]{8}-[0-9]{4}-[A-Za-z0-9][A-Za-z0-9._-]*")
# An evidence file name: plain segments joined by dots. No colon (an NTFS
# alternate stream), no trailing dot or space (a Win32 alias), no separator.
EVIDENCE_NAME = re.compile(r"[A-Za-z0-9][A-Za-z0-9_-]*(?:\.[A-Za-z0-9_-]+)*")
RECORDER_OUTPUTS = frozenset(
    {PROOF_FILE, "MANIFEST.sha256", "windows-clean-vm-installer-status.json", "windows-clean-vm-installer-status.md"}
)
REQUIRED_GATE_IDS = frozenset(
    {"installer-artifact", "fresh-guest", "install-log", "studio-smoke", "launch-screenshot", "claim-boundary"}
)

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
    proof_source: str
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
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return {}
    return data if isinstance(data, dict) else {}


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


def _load_proof(path: Path) -> ProofRecord:
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise ValueError("the proof record is not a JSON object")
    return _proof_from_data(data)


RECORD_STRING_FIELDS = (
    "schema", "created_at", "mode", "installer_path", "installer_sha256", "vm_name",
    "guest_os", "guest_arch", "install_log", "studio_smoke_json", "screenshot",
)
GATE_FIELDS = frozenset({"id", "label", "status", "evidence"})


def _committed_record_problem(data: object) -> str | None:
    """Check a committed record's JSON shape strictly, before any field is used."""
    if not isinstance(data, dict):
        return "the proof record is not a JSON object"
    for key in RECORD_STRING_FIELDS:
        if not isinstance(data.get(key), str):
            return f"{key} is missing or not a string"
    if data.get("verified") is not True:
        return "verified is not the literal true"
    gates = data.get("gates")
    if not isinstance(gates, list) or not all(
        isinstance(gate, dict) and set(gate) == GATE_FIELDS and all(isinstance(value, str) for value in gate.values())
        for gate in gates
    ):
        return "the gates are malformed"
    claims = data.get("forbidden_claims")
    if not isinstance(claims, list) or not all(isinstance(claim, str) for claim in claims):
        return "forbidden_claims is malformed"
    return None


def _proof_from_data(data: dict) -> ProofRecord:
    gates = [SmokeGate(**gate) for gate in data.get("gates", [])]
    return ProofRecord(
        schema=data.get("schema", ""),
        created_at=data.get("created_at", ""),
        mode=data.get("mode", ""),
        verified=data.get("verified") is True,
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


def _read_bundle_file(path: Path, seen: set[tuple[int, int]]) -> tuple[str | None, bytes]:
    """Open one bundle file once and read it from that handle.

    The checks run on the open handle whose bytes are returned, so nothing can
    be swapped between check and read: it must still be the file lstat saw, a
    regular file with exactly one link, not opened through a link, and not the
    same file as another bundle entry (`seen` holds the identities read so far).
    """
    try:
        before = os.lstat(path)
    except FileNotFoundError:
        return f"{path.name} is missing", b""
    if stat.S_ISLNK(before.st_mode) or _is_link(path) or not stat.S_ISREG(before.st_mode):
        return f"{path.name} is not a regular file; a bundle holds only regular files", b""
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_BINARY", 0)
    fd = os.open(path, flags)
    try:
        info = os.fstat(fd)
        if not stat.S_ISREG(info.st_mode) or (info.st_dev, info.st_ino) != (before.st_dev, before.st_ino):
            return f"{path.name} changed while it was being read", b""
        if info.st_nlink != 1:
            return f"{path.name} has {info.st_nlink} links; a bundle file has exactly one", b""
        identity = (info.st_dev, info.st_ino)
        if info.st_ino and identity in seen:
            return f"{path.name} is the same file as another bundle entry", b""
        seen.add(identity)
        chunks = []
        while True:
            chunk = os.read(fd, 1 << 20)
            if not chunk:
                break
            chunks.append(chunk)
        return None, b"".join(chunks)
    finally:
        os.close(fd)


def _verified_manifest(bundle: Path) -> tuple[str | None, dict[str, bytes]]:
    """Verify MANIFEST.sha256 against the bundle in one read.

    Returns (problem, contents). On success, contents maps each listed name to
    the exact bytes that were hashed, so every later check reads only verified
    bytes and a file that appears afterwards is never evidence. Every entry is
    a regular file with exactly one link (git never checks out a hard link).
    """
    seen: set[tuple[int, int]] = set()
    problem, manifest_bytes = _read_bundle_file(bundle / "MANIFEST.sha256", seen)
    if problem:
        return problem, {}
    listed: dict[str, str] = {}
    for line in manifest_bytes.decode("utf-8").splitlines():
        digest, sep, name = line.partition("  ")
        if not sep or not re.fullmatch(r"[0-9a-f]{64}", digest) or "/" in name or "\\" in name:
            return f"malformed manifest line: {line!r}", {}
        if name in listed:
            return f"the manifest lists {name} twice", {}
        listed[name] = digest
    entries = list(bundle.iterdir())
    for path in entries:
        info = os.lstat(path)
        if _is_link(path) or not stat.S_ISREG(info.st_mode):
            return f"{path.name} is not a regular file; a bundle holds only regular files", {}
        if info.st_nlink != 1:
            return f"{path.name} has {info.st_nlink} links; a bundle file has exactly one", {}
    present = {path.name for path in entries if path.name != "MANIFEST.sha256"}
    if set(listed) != present:
        return f"manifest lists {sorted(listed)} but the bundle holds {sorted(present)}", {}
    contents: dict[str, bytes] = {}
    for name, digest in listed.items():
        problem, data = _read_bundle_file(bundle / name, seen)
        if problem:
            return problem, {}
        if hashlib.sha256(data).hexdigest() != digest:
            return f"{name} does not match its manifest digest", {}
        contents[name] = data
    return None, contents


def _committed_bundle_problem(
    bundle: Path, repo: Path, proof: ProofRecord, verified: dict[str, bytes]
) -> str | None:
    """Return why a committed bundle cannot back the claim, or None if it can.

    `verified` is the manifest-verified content of the bundle (_verified_manifest).
    """
    if proof.schema != SCHEMA or proof.mode != "clean-vm" or not proof.verified:
        return "the proof is not a verified clean-vm record of the current schema"
    if proof.guest_arch.lower() not in X64_GUEST_ARCHES:
        return f"guest architecture {proof.guest_arch!r} is not x64"
    if not re.fullmatch(r"[0-9a-f]{64}", proof.installer_sha256):
        return "the installer SHA-256 is missing"
    gate_ids = [gate.id for gate in proof.gates]
    if sorted(gate_ids) != sorted(REQUIRED_GATE_IDS) or any(gate.status != "pass" for gate in proof.gates):
        return "the record does not carry each required gate exactly once, passing"
    # The gate labels are the recorder's summary; check the facts behind them.
    if not proof.vm_name.strip() or not proof.guest_os.strip():
        return "the fresh guest's VM name and OS are not recorded"
    if not proof.installer_path.strip():
        return "the installer path is not recorded"
    missing_claims = [claim for claim in forbidden_claims() if claim not in proof.forbidden_claims]
    if missing_claims:
        return f"the claim boundary is missing {missing_claims[0]!r}"
    # Each recorded path must name the bundle directly: proofs/windows/
    # studio-clean-vm/<bundle>/<file>. The bundle path is already link-free
    # (_confinement_problem), so no link can sit anywhere along the path.
    bundle_parts = (COMMITTED_BUNDLES_REL / bundle.name).parts
    bundle_dir = bundle.resolve()
    # Only files the manifest verified can be evidence, named exactly as the
    # directory listed them: no stream, case or short-name alias, and nothing
    # that appeared after verification.
    inventory = set(verified) - RECORDER_OUTPUTS
    names = [Path(text.replace("\\", "/")).name for text in (proof.install_log, proof.studio_smoke_json, proof.screenshot)]
    if len(set(names)) != len(names):
        return "the install log, smoke record and screenshot must be three different files"
    for name in names:
        if not EVIDENCE_NAME.fullmatch(name) or name not in inventory:
            return f"{name!r} is not a verified evidence file of the bundle"
    for field, text in (
        ("install_log", proof.install_log),
        ("studio_smoke_json", proof.studio_smoke_json),
        ("screenshot", proof.screenshot),
    ):
        relative = Path(text.replace("\\", "/"))
        candidate = repo / relative
        if (
            not text
            or relative.is_absolute()
            or relative.parts[:-1] != bundle_parts
            or _is_link(candidate)
            or not candidate.is_file()
            or candidate.resolve().parent != bundle_dir
        ):
            return f"{field} must name a file directly inside {'/'.join(bundle_parts)}"
    smoke = _strict_json_bytes(verified[Path(proof.studio_smoke_json.replace("\\", "/")).name])
    if not isinstance(smoke, dict) or not (
        smoke.get("status") == "passed"
        and smoke.get("source_included") is False
        and smoke.get("provider_api_called") is False
    ):
        return "studio-smoke.json in the bundle does not show a passing smoke"
    return None


def _is_link(path: Path) -> bool:
    return path.is_symlink() or bool(getattr(path, "is_junction", lambda: False)())


def _confinement_problem(repo: Path, path: Path) -> str | None:
    """Committed evidence must be real directories inside the repo's proof root."""
    current = repo
    for part in path.relative_to(repo).parts:
        current = current / part
        if _is_link(current):
            return f"{current.relative_to(repo).as_posix()} is a link; committed evidence must be real files in the repository"
    if not path.resolve().is_relative_to((repo / COMMITTED_BUNDLES_REL).resolve()):
        return "the bundle resolves outside proofs/windows/studio-clean-vm"
    return None


def _broken_proof(problem: str) -> ProofRecord:
    return replace(_empty_proof(), gates=[_integrity_gate(problem)])


def _reject_duplicate_keys(pairs: list[tuple[str, object]]) -> dict:
    keys = [key for key, _ in pairs]
    if len(keys) != len(set(keys)):
        raise ValueError("duplicate JSON key")
    return dict(pairs)


def _strict_json_bytes(data: bytes) -> object:
    """Committed evidence JSON: strict UTF-8, and no key may repeat."""
    return json.loads(data.decode("utf-8"), object_pairs_hook=_reject_duplicate_keys)


def latest_committed_proof(repo: Path | None = None) -> tuple[ProofRecord, str] | None:
    """Read the newest committed bundle (by its <YYYYMMDD-HHMM>-<host> name), checked in full.

    Links on the path to the proof root are refused before the root's absence
    is trusted. The newest bundle entry decides: if it is not a real directory,
    lacks its record, or fails any check, it is reported unverified, and neither
    an older bundle nor local Desktop evidence stands in for it.
    """
    repo = repo or ROOT
    root = repo / COMMITTED_BUNDLES_REL
    root_source = f"committed:{COMMITTED_BUNDLES_REL.as_posix()}"
    try:
        # Walk every ancestor with lstat, which reports errors instead of
        # hiding them: only a missing path means "no committed evidence".
        current = repo
        for part in COMMITTED_BUNDLES_REL.parts:
            current = current / part
            try:
                info = os.lstat(current)
            except FileNotFoundError:
                return None
            label = current.relative_to(repo).as_posix()
            if stat.S_ISLNK(info.st_mode) or _is_link(current):
                return _broken_proof(f"{label} is a link"), root_source
            if not stat.S_ISDIR(info.st_mode):
                return _broken_proof(f"{label} is not a directory"), root_source
            # Listing it is the readability test: on Windows os.access checks
            # only attributes, not the permission to list a directory.
            try:
                with os.scandir(current):
                    pass
            except PermissionError:
                return _broken_proof(f"{label} is not a readable directory"), root_source
        entries = sorted(path for path in root.iterdir() if BUNDLE_NAME.match(path.name))
        if not entries:
            return None
        bundle = entries[-1]
        source = f"committed:{(COMMITTED_BUNDLES_REL / bundle.name).as_posix()}"
        if _is_link(bundle) or not bundle.is_dir():
            return _broken_proof("the newest bundle entry is not a directory"), source
        if not BUNDLE_FULL_NAME.fullmatch(bundle.name):
            return _broken_proof("the newest bundle is not named <YYYYMMDD-HHMM>-<host>"), source
        problem = _confinement_problem(repo, bundle)
        if problem:
            return _broken_proof(problem), source
        # Verify the manifest first; the record and the evidence are then read
        # only from the bytes it verified.
        problem, verified = _verified_manifest(bundle)
        if problem:
            return _broken_proof(problem), source
        if PROOF_FILE not in verified:
            return _broken_proof("the newest bundle has no proof record"), source
        data = _strict_json_bytes(verified[PROOF_FILE])
        problem = _committed_record_problem(data)
        if problem:
            return _broken_proof(problem), source
        proof = _proof_from_data(data)
        problem = _committed_bundle_problem(bundle, repo, proof, verified)
    except (OSError, UnicodeError, ValueError, TypeError) as error:
        return _broken_proof(f"unreadable committed evidence: {error}"), root_source
    if problem:
        proof = replace(proof, verified=False, gates=[*proof.gates, _integrity_gate(problem)])
    return proof, source


def _integrity_gate(problem: str) -> SmokeGate:
    return SmokeGate("committed-bundle", "Committed Bundle Integrity", "blocked", problem)


def _empty_proof() -> ProofRecord:
    return ProofRecord(SCHEMA, "", "", False, "", "", "", "", "", "", "", "", [], forbidden_claims())


def latest_proof(evidence_root: Path | None = None) -> ProofRecord | None:
    return _locate_proof(evidence_root)[0]


def _locate_proof(evidence_root: Path | None) -> tuple[ProofRecord | None, str]:
    if evidence_root is None:
        committed = latest_committed_proof()
        if committed is not None:
            return committed
    root = evidence_root or default_evidence_root()
    if not root.exists():
        return None, "none"
    candidates = sorted(root.glob(f"*/{PROOF_FILE}"), key=lambda path: path.stat().st_mtime)
    if not candidates:
        return None, "none"
    return _load_proof(candidates[-1]), f"local:{candidates[-1].parent}"


def blocked_by(proof: ProofRecord | None) -> list[str]:
    if proof and proof.verified:
        return []
    if proof:
        gate_blockers = {
            "installer-artifact": "unsigned NSIS installer artifact digest",
            "fresh-guest": "clean Windows VM guest identity",
            "install-log": "installer run log from inside the guest",
            "studio-smoke": "installed Studio --studio-smoke JSON",
            "launch-screenshot": "installed app launch screenshot",
            "claim-boundary": "claim boundary evidence",
            "committed-bundle": "committed clean-VM bundle integrity",
        }
        return [
            gate_blockers.get(gate.id, gate.label)
            for gate in proof.gates
            if gate.status != "pass"
        ]
    return [
        "clean Windows VM guest identity",
        "unsigned NSIS installer artifact digest",
        "installer run log from inside the guest",
        "installed Studio --studio-smoke JSON",
        "installed app launch screenshot",
    ]


def read_status(evidence_root: Path | None = None) -> WindowsCleanVmInstallerStatus:
    proof, proof_source = _locate_proof(evidence_root)
    verified = bool(proof and proof.verified)
    return WindowsCleanVmInstallerStatus(
        source=str(ROOT),
        status="clean-vm-proof-verified" if verified else "proof-contract-ready-clean-vm-open",
        default_evidence_root=str(evidence_root or default_evidence_root()),
        clean_vm_verified=verified,
        proof_source=proof_source,
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
        blocked_by=blocked_by(proof),
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
        f"Proof source: `{status.proof_source}`",
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
