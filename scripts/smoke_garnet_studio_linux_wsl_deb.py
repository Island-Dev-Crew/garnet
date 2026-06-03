#!/usr/bin/env python3
"""Record and verify a WSL Linux Garnet Studio DEB package proof.

This is an S117 package-pipeline increment. It proves that this Windows box can
drive WSL to build the Tauri Linux `.deb`, inspect it with `dpkg-deb`, and run
the Linux binary's non-GUI `--studio-smoke` command. It is not Linux desktop GUI
install/launch proof and it is not Linux seccomp or OS-sandbox enforcement.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = "garnet.studio.linux_wsl_deb.v1"
SUMMARY_NAME = "garnet-studio-linux-wsl-deb.json"
MARKDOWN_NAME = "garnet-studio-linux-wsl-deb.md"
MANIFEST_NAME = "MANIFEST.sha256"
DEFAULT_PROOF_ROOT = ROOT / "proofs" / "linux" / "execution" / "studio-package"
REQUIRED_COMMANDS = [
    "wsl-uname",
    "npm-install",
    "npm-build",
    "tauri-build-deb",
    "studio-smoke",
    "dpkg-info",
    "dpkg-contents",
]
REQUIRED_HONEST_SCOPE = [
    "not Linux desktop GUI launch proof",
    "not Linux seccomp or OS-sandbox enforcement",
    "not clean Linux install proof",
]
FORBIDDEN_CLAIMS = [
    "Linux desktop GUI launch proof verified",
    "Linux seccomp enforced",
    "OS-sandbox enforcement verified",
    "clean Linux install proof verified",
    "production readiness verified",
    "v1.0 readiness verified",
]


@dataclass(frozen=True)
class CommandRecord:
    id: str
    display_args: list[str]
    exit_code: int
    stdout_file: str
    stderr_file: str
    status: str


@dataclass(frozen=True)
class LinuxWslDebEvidence:
    status: str
    verified: bool
    reason: str
    bundle: str | None
    deferred: list[str]


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8", newline="\n")


def _timestamp() -> str:
    return datetime.now(UTC).strftime("%Y%m%d-%H%M%S")


def _write_manifest(directory: Path) -> None:
    entries: list[str] = []
    for path in sorted(p for p in directory.rglob("*") if p.is_file()):
        if path.name == MANIFEST_NAME:
            continue
        entries.append(f"{_sha256(path)}  {path.relative_to(directory).as_posix()}")
    _write_text(directory / MANIFEST_NAME, "\n".join(entries) + "\n")


def _manifest_entries(directory: Path) -> dict[str, str] | None:
    manifest = directory / MANIFEST_NAME
    if not manifest.is_file():
        return None
    entries: dict[str, str] = {}
    for line in manifest.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        try:
            digest, relative = line.split("  ", 1)
        except ValueError:
            return None
        target = directory / Path(relative)
        if not target.is_file() or _sha256(target) != digest:
            return None
        entries[relative.replace("\\", "/")] = digest
    return entries


def _run_host(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        capture_output=True,
        check=False,
    )


def _wsl_root() -> str:
    completed = _run_host(["wsl.exe", "-e", "wslpath", "-a", str(ROOT)])
    if completed.returncode != 0:
        raise RuntimeError(completed.stderr.strip() or "wslpath failed")
    return completed.stdout.strip()


def _run_wsl(
    *,
    command_id: str,
    shell: str,
    display_args: list[str],
    output_dir: Path,
    wsl_root: str,
) -> CommandRecord:
    completed = _run_host(["wsl.exe", "-e", "sh", "-lc", f"cd '{wsl_root}' && {shell}"])
    stdout_rel = Path("commands") / f"{command_id}-stdout.txt"
    stderr_rel = Path("commands") / f"{command_id}-stderr.txt"
    _write_text(output_dir / stdout_rel, completed.stdout)
    _write_text(output_dir / stderr_rel, completed.stderr)
    return CommandRecord(
        id=command_id,
        display_args=display_args,
        exit_code=completed.returncode,
        stdout_file=stdout_rel.as_posix(),
        stderr_file=stderr_rel.as_posix(),
        status="passed" if completed.returncode == 0 else "failed",
    )


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace") if path.is_file() else ""


def _find_deb() -> Path | None:
    bundle_dir = ROOT / "target" / "release" / "bundle" / "deb"
    candidates = sorted(bundle_dir.glob("*.deb"))
    return candidates[0] if candidates else None


def _linux_binary() -> Path:
    return ROOT / "target" / "release" / "garnet-studio"


def record_proof(output_dir: Path | None = None) -> int:
    bundle = output_dir or DEFAULT_PROOF_ROOT / f"linux-wsl-deb-package-{_timestamp()}"
    bundle.mkdir(parents=True, exist_ok=True)
    wsl_root = _wsl_root()
    commands = [
        _run_wsl(
            command_id="wsl-uname",
            shell="uname -a",
            display_args=["wsl.exe", "-e", "uname", "-a"],
            output_dir=bundle,
            wsl_root=wsl_root,
        ),
        _run_wsl(
            command_id="npm-install",
            shell="cd apps/garnet-studio && npm install --include=optional",
            display_args=["npm", "install", "--include=optional"],
            output_dir=bundle,
            wsl_root=wsl_root,
        ),
        _run_wsl(
            command_id="npm-build",
            shell="cd apps/garnet-studio && npm run build",
            display_args=["npm", "run", "build"],
            output_dir=bundle,
            wsl_root=wsl_root,
        ),
        _run_wsl(
            command_id="tauri-build-deb",
            shell="cd apps/garnet-studio && npm exec -- tauri build --bundles deb",
            display_args=["npm", "exec", "--", "tauri", "build", "--bundles", "deb"],
            output_dir=bundle,
            wsl_root=wsl_root,
        ),
        _run_wsl(
            command_id="studio-smoke",
            shell="./target/release/garnet-studio --studio-smoke",
            display_args=["target/release/garnet-studio", "--studio-smoke"],
            output_dir=bundle,
            wsl_root=wsl_root,
        ),
        _run_wsl(
            command_id="dpkg-info",
            shell="dpkg-deb --info target/release/bundle/deb/*.deb",
            display_args=["dpkg-deb", "--info", "target/release/bundle/deb/*.deb"],
            output_dir=bundle,
            wsl_root=wsl_root,
        ),
        _run_wsl(
            command_id="dpkg-contents",
            shell="dpkg-deb --contents target/release/bundle/deb/*.deb",
            display_args=["dpkg-deb", "--contents", "target/release/bundle/deb/*.deb"],
            output_dir=bundle,
            wsl_root=wsl_root,
        ),
    ]
    dpkg_info = _read_text(bundle / "commands" / "dpkg-info-stdout.txt")
    dpkg_contents = _read_text(bundle / "commands" / "dpkg-contents-stdout.txt")
    _write_text(bundle / "package" / "dpkg-info.txt", dpkg_info)
    _write_text(bundle / "package" / "dpkg-contents.txt", dpkg_contents)

    deb = _find_deb()
    binary = _linux_binary()
    contains_binary = "usr/bin/garnet-studio" in dpkg_contents
    contains_desktop = ".desktop" in dpkg_contents
    all_commands_passed = all(command.status == "passed" for command in commands)
    package_ok = deb is not None and contains_binary and contains_desktop
    binary_ok = binary.is_file()
    data = {
        "schema": SCHEMA,
        "generated_at": datetime.now(UTC).isoformat(timespec="seconds"),
        "status": "passed" if all_commands_passed and package_ok and binary_ok else "failed",
        "platform": "linux",
        "evidence_tier": "wsl-linux-package-build-command-smoke",
        "wsl_is_enforcement": False,
        "source_included": False,
        "provider_api_called": False,
        "desktop_gui_launch_proven": False,
        "package_install_proven": False,
        "package": {
            "format": "deb",
            "path": deb.relative_to(ROOT).as_posix() if deb else None,
            "sha256": _sha256(deb) if deb else None,
            "size_bytes": deb.stat().st_size if deb else 0,
            "architecture": "amd64" if "Architecture: amd64" in dpkg_info else "unknown",
            "contains_binary": contains_binary,
            "contains_desktop_file": contains_desktop,
        },
        "binary": {
            "path": binary.relative_to(ROOT).as_posix(),
            "sha256": _sha256(binary) if binary.is_file() else None,
            "studio_smoke_status": "passed"
            if next((command for command in commands if command.id == "studio-smoke"), None).status == "passed"
            else "failed",
        },
        "commands": [asdict(command) for command in commands],
        "honest_scope": [
            "WSL is Linux package build and command-smoke evidence only",
            "not Linux desktop GUI launch proof",
            "not Linux seccomp or OS-sandbox enforcement",
            "not clean Linux install proof",
            "not signed, production, or v1.0 readiness",
        ],
    }
    _write_text(bundle / SUMMARY_NAME, json.dumps(data, indent=2) + "\n")
    _write_text(bundle / MARKDOWN_NAME, render_markdown(data))
    _write_manifest(bundle)
    print(render_markdown(data), end="")
    if data["status"] == "passed" and not verify_bundle(bundle / SUMMARY_NAME):
        print("linux-wsl-deb proof failed post-write verification", file=sys.stderr)
        return 1
    return 0 if data["status"] == "passed" else 1


def verify_bundle(summary_path: Path) -> bool:
    try:
        data = json.loads(summary_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return False
    bundle = summary_path.parent
    manifest = _manifest_entries(bundle)
    if manifest is None:
        return False
    if SUMMARY_NAME not in manifest or MARKDOWN_NAME not in manifest:
        return False
    if data.get("schema") != SCHEMA or data.get("status") != "passed":
        return False
    if data.get("platform") != "linux" or data.get("evidence_tier") != "wsl-linux-package-build-command-smoke":
        return False
    if data.get("wsl_is_enforcement") is not False:
        return False
    for key in ["source_included", "provider_api_called", "desktop_gui_launch_proven", "package_install_proven"]:
        if data.get(key) is not False:
            return False
    scope = " ".join(str(item) for item in data.get("honest_scope", []))
    if not all(anchor in scope for anchor in REQUIRED_HONEST_SCOPE):
        return False
    all_text = json.dumps(data, sort_keys=True)
    if any(claim in all_text for claim in FORBIDDEN_CLAIMS):
        return False
    package = data.get("package", {})
    if (
        package.get("format") != "deb"
        or package.get("architecture") != "amd64"
        or package.get("contains_binary") is not True
        or package.get("contains_desktop_file") is not True
        or not isinstance(package.get("sha256"), str)
        or len(package.get("sha256", "")) != 64
        or int(package.get("size_bytes", 0)) <= 0
    ):
        return False
    binary = data.get("binary", {})
    if binary.get("studio_smoke_status") != "passed" or len(str(binary.get("sha256", ""))) != 64:
        return False
    commands = data.get("commands")
    if not isinstance(commands, list):
        return False
    command_ids = [command.get("id") for command in commands if isinstance(command, dict)]
    if command_ids != REQUIRED_COMMANDS:
        return False
    for command in commands:
        if command.get("status") != "passed" or int(command.get("exit_code", 1)) != 0:
            return False
        stdout_file = command.get("stdout_file")
        stderr_file = command.get("stderr_file")
        if not isinstance(stdout_file, str) or not isinstance(stderr_file, str):
            return False
        if stdout_file not in manifest or stderr_file not in manifest:
            return False
    return "package/dpkg-info.txt" in manifest and "package/dpkg-contents.txt" in manifest


def read_committed_evidence(root: Path = ROOT) -> LinuxWslDebEvidence:
    candidates: list[Path]
    if (root / SUMMARY_NAME).is_file():
        candidates = [root / SUMMARY_NAME]
    else:
        candidates = sorted((root / "proofs" / "linux" / "execution" / "studio-package").glob(f"*/{SUMMARY_NAME}"))
        if not candidates:
            candidates = sorted(root.glob(f"**/{SUMMARY_NAME}"))
    for summary in reversed(candidates):
        if verify_bundle(summary):
            data = json.loads(summary.read_text(encoding="utf-8"))
            package = data.get("package", {})
            return LinuxWslDebEvidence(
                status="verified",
                verified=True,
                reason=(
                    "WSL Linux Tauri .deb package build and non-GUI studio-smoke "
                    f"verified at `{summary.as_posix()}` ({package.get('path')}, sha256 {package.get('sha256')})."
                ),
                bundle=summary.parent.as_posix(),
                deferred=[
                    "not Linux desktop GUI launch proof",
                    "not Linux seccomp or OS-sandbox enforcement",
                    "not clean Linux install proof",
                    "not signed, production, or v1.0 readiness",
                ],
            )
    return LinuxWslDebEvidence(
        status="missing",
        verified=False,
        reason="No committed WSL Linux Tauri .deb proof bundle verified.",
        bundle=None,
        deferred=[
            "record with `scripts/smoke_garnet_studio_linux_wsl_deb.py --record`",
            "Linux desktop GUI launch and package install remain separate gates",
        ],
    )


def render_markdown(data: dict) -> str:
    package = data.get("package", {})
    binary = data.get("binary", {})
    lines = [
        "# Garnet Studio Linux WSL DEB Package Proof",
        "",
        f"- schema: `{data.get('schema')}`",
        f"- status: `{data.get('status')}`",
        f"- evidence tier: `{data.get('evidence_tier')}`",
        f"- package: `{package.get('path')}`",
        f"- package sha256: `{package.get('sha256')}`",
        f"- package architecture: `{package.get('architecture')}`",
        f"- contains binary: `{str(package.get('contains_binary')).lower()}`",
        f"- contains desktop file: `{str(package.get('contains_desktop_file')).lower()}`",
        f"- binary: `{binary.get('path')}`",
        f"- binary sha256: `{binary.get('sha256')}`",
        f"- studio smoke: `{binary.get('studio_smoke_status')}`",
        "",
        "## Commands",
        "",
        "| Command | Status |",
        "| --- | --- |",
    ]
    for command in data.get("commands", []):
        lines.append(f"| `{command.get('id')}` | `{command.get('status')}` |")
    lines.extend(
        [
            "",
            "## Honest Scope",
            "",
            *[f"- {item}" for item in data.get("honest_scope", [])],
            "",
        ]
    )
    return "\n".join(lines)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--record", action="store_true", help="record WSL Linux .deb package proof")
    parser.add_argument("--gate", action="store_true", help="verify committed proof bundle")
    parser.add_argument("--output-dir", type=Path)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    if args.record:
        return record_proof(args.output_dir)
    evidence = read_committed_evidence()
    if args.format == "md":
        print(
            "\n".join(
                [
                    "# Garnet Studio Linux WSL DEB Package Proof Status",
                    "",
                    f"- status: `{evidence.status}`",
                    f"- verified: `{str(evidence.verified).lower()}`",
                    f"- reason: {evidence.reason}",
                    "",
                    "## Deferred",
                    "",
                    *[f"- {item}" for item in evidence.deferred],
                    "",
                ]
            )
        )
    else:
        print(json.dumps(asdict(evidence), indent=2))
    if args.gate and not evidence.verified:
        print(evidence.reason, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
