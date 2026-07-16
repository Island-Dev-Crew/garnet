#!/usr/bin/env python3
"""Validate Garnet's single, CI-enforced Rust MSRV contract.

This reporter is deliberately local and deterministic. It reads manifests,
current documentation, and existing required workflow contexts; it performs no
network access and does not infer a floor from the ambient toolchain.
"""
from __future__ import annotations

import argparse
import json
import shutil
import sys
import tomllib
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MSRV = "1.95"
EXCLUDED_ACTIVE_MANIFESTS = (
    "apps/garnet-studio/src-tauri/Cargo.toml",
    "garnet-parser-v0.3/fuzz/Cargo.toml",
)
CURRENT_SURFACES = {
    "README.md": ("Rust 1.95+",),
    "CONTRIBUTING.md": ("**Rust** 1.95+",),
    "FAQ.md": ("Rust 1.95+",),
    "docs/getting-started.html": ("Rust 1.95+",),
    "docs/index.html": ("Rust 1.95+",),
    "garnet-parser-v0.3/README.md": ("**Rust version:** 1.95+",),
}
STALE_CURRENT_MARKERS = ("Rust 1.75+", "1.75+ per", "tested on 1.94.1")
CI_PATH = ".github/workflows/ci.yml"
STUDIO_CI_PATH = ".github/workflows/macos-studio.yml"
ROOT_AGENT_PATH = "AGENTS.md"
ROOT_CI_COMMAND = (
    "cargo +1.95.0 check --workspace --all-targets --all-features --locked"
)
STUDIO_CI_COMMAND = (
    "cargo +1.95.0 check --locked --manifest-path "
    "apps/garnet-studio/src-tauri/Cargo.toml --all-targets"
)
INSTALL_COMMAND = "rustup toolchain install 1.95.0 --profile minimal"
REPORTER_TEST_COMMAND = "python3 -I scripts/test_garnet_msrv_status.py"
REPORTER_GATE_COMMAND = "python3 -I scripts/garnet_msrv_status.py --gate"
AGENT_ANCHOR = "Cargo `rust-version = \"1.95\"` is the single workspace MSRV"


@dataclass
class MsrvStatus:
    schema: str
    msrv: str
    workspace_member_count: int
    workspace_members_inheriting: int
    excluded_manifests_declaring: int
    current_surfaces_aligned: bool
    stable_tracking_preserved: bool
    exact_msrv_ci_check: bool
    studio_exact_msrv_ci_check: bool
    reporter_ci_wired: bool
    rust_toolchain_file_absent: bool
    procedural_contract_present: bool
    findings: list[str] = field(default_factory=list)
    ok: bool = False


def _read(path: Path, findings: list[str], label: str) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except OSError as exc:
        findings.append(f"{label} is unreadable: {exc}")
        return ""


def _toml(path: Path, findings: list[str], label: str) -> dict[str, object]:
    text = _read(path, findings, label)
    if not text:
        return {}
    try:
        value = tomllib.loads(text)
    except tomllib.TOMLDecodeError as exc:
        findings.append(f"{label} is invalid TOML: {exc}")
        return {}
    if not isinstance(value, dict):
        findings.append(f"{label} root must be a table")
        return {}
    return value


def _package(manifest: dict[str, object], findings: list[str], label: str) -> dict[str, object]:
    package = manifest.get("package", {})
    if not isinstance(package, dict):
        findings.append(f"{label} is missing [package]")
        return {}
    return package


def _inherits_msrv(value: object) -> bool:
    return isinstance(value, dict) and value.get("workspace") is True


def _workspace_members(root_manifest: dict[str, object], findings: list[str]) -> list[str]:
    workspace = root_manifest.get("workspace", {})
    if not isinstance(workspace, dict):
        findings.append("root Cargo.toml is missing [workspace]")
        return []
    members = workspace.get("members", [])
    if not isinstance(members, list) or not all(isinstance(item, str) for item in members):
        findings.append("root Cargo.toml workspace.members must be a string array")
        return []
    return list(members)


def read_status(root: Path = ROOT) -> MsrvStatus:
    findings: list[str] = []
    root_manifest = _toml(root / "Cargo.toml", findings, "root Cargo.toml")
    workspace = root_manifest.get("workspace", {})
    if not isinstance(workspace, dict):
        workspace = {}
    workspace_package = workspace.get("package", {})
    if not isinstance(workspace_package, dict):
        workspace_package = {}
    if workspace_package.get("rust-version") != MSRV:
        findings.append(
            f'root [workspace.package] rust-version must be exactly "{MSRV}"'
        )

    members = _workspace_members(root_manifest, findings)
    inheriting = 0
    for member in members:
        rel = f"{member}/Cargo.toml"
        manifest = _toml(root / rel, findings, rel)
        package = _package(manifest, findings, rel)
        if _inherits_msrv(package.get("rust-version")):
            inheriting += 1
        else:
            findings.append(
                f"{rel} must inherit the workspace MSRV with rust-version.workspace = true"
            )

    excluded_declaring = 0
    for rel in EXCLUDED_ACTIVE_MANIFESTS:
        manifest = _toml(root / rel, findings, rel)
        package = _package(manifest, findings, rel)
        if package.get("rust-version") == MSRV:
            excluded_declaring += 1
        else:
            findings.append(f'{rel} must declare rust-version = "{MSRV}" directly')

    surfaces_aligned = True
    for rel, required in CURRENT_SURFACES.items():
        text = _read(root / rel, findings, rel)
        missing = [marker for marker in required if marker not in text]
        stale = [marker for marker in STALE_CURRENT_MARKERS if marker in text]
        if missing or stale:
            surfaces_aligned = False
        if missing:
            findings.append(f"{rel} is missing current MSRV marker(s): {missing}")
        if stale:
            findings.append(f"{rel} carries stale current MSRV marker(s): {stale}")

    ci = _read(root / CI_PATH, findings, CI_PATH)
    studio_ci = _read(root / STUDIO_CI_PATH, findings, STUDIO_CI_PATH)
    stable_tracking = (
        "dtolnay/rust-toolchain@stable" in ci
        and "dtolnay/rust-toolchain@stable" in studio_ci
    )
    if not stable_tracking:
        findings.append("moving stable CI tracking must remain present in both workflows")

    exact_ci = INSTALL_COMMAND in ci and ROOT_CI_COMMAND in ci
    if not exact_ci:
        findings.append("ci.yml is missing the exact Rust 1.95 workspace check")
    studio_exact_ci = INSTALL_COMMAND in studio_ci and STUDIO_CI_COMMAND in studio_ci
    if not studio_exact_ci:
        findings.append("macos-studio.yml is missing the exact Rust 1.95 Studio check")
    reporter_ci_wired = (
        REPORTER_TEST_COMMAND in ci and REPORTER_GATE_COMMAND in ci
    )
    if not reporter_ci_wired:
        findings.append("ci.yml is missing the MSRV reporter test/gate")

    toolchain_absent = not (root / "rust-toolchain.toml").exists() and not (
        root / "rust-toolchain"
    ).exists()
    if not toolchain_absent:
        findings.append("the moving-stable policy forbids a repository rust-toolchain pin")

    agent_text = _read(root / ROOT_AGENT_PATH, findings, ROOT_AGENT_PATH)
    procedural_contract = AGENT_ANCHOR in agent_text
    if not procedural_contract:
        findings.append("AGENTS.md is missing the procedural MSRV contract")

    return MsrvStatus(
        schema="garnet.msrv_status/v1",
        msrv=MSRV,
        workspace_member_count=len(members),
        workspace_members_inheriting=inheriting,
        excluded_manifests_declaring=excluded_declaring,
        current_surfaces_aligned=surfaces_aligned,
        stable_tracking_preserved=stable_tracking,
        exact_msrv_ci_check=exact_ci,
        studio_exact_msrv_ci_check=studio_exact_ci,
        reporter_ci_wired=reporter_ci_wired,
        rust_toolchain_file_absent=toolchain_absent,
        procedural_contract_present=procedural_contract,
        findings=findings,
        ok=not findings,
    )


def copy_contract_surface(source: Path, destination: Path) -> None:
    """Copy only the deterministic contract surface for mutation tests."""
    root_manifest = tomllib.loads((source / "Cargo.toml").read_text(encoding="utf-8"))
    members = root_manifest["workspace"]["members"]
    paths = {
        "Cargo.toml",
        *[f"{member}/Cargo.toml" for member in members],
        *EXCLUDED_ACTIVE_MANIFESTS,
        *CURRENT_SURFACES,
        CI_PATH,
        STUDIO_CI_PATH,
        ROOT_AGENT_PATH,
    }
    for rel in sorted(paths):
        target = destination / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source / rel, target)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless every MSRV contract surface agrees",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    status = read_status(args.root.resolve())
    print(json.dumps(asdict(status), indent=2, sort_keys=True))
    if args.gate and not status.ok:
        print(
            "garnet-msrv gate FAILED: " + "; ".join(status.findings),
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
