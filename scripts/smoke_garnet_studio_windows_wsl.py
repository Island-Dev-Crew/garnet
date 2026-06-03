#!/usr/bin/env python3
"""Record Windows Studio smoke and WSL command-contract evidence.

This is a productization proof increment, not a Linux enforcement proof. The
Windows row exercises the Tauri Studio `--studio-smoke` mode. The WSL row only
replays the repo-owned command/status contract in WSL and is always labeled as
execution/portability evidence, not Linux seccomp, OS-sandbox, Wasmtime fuel, or
desktop GUI proof.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import re
import shutil
import subprocess
import sys
from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SUMMARY_NAME = "garnet-studio-windows-wsl-smoke.json"
MARKDOWN_NAME = "garnet-studio-windows-wsl-smoke.md"
MANIFEST_NAME = "MANIFEST.sha256"
SCHEMA = "garnet.studio.windows_wsl_smoke.v1"
WINDOWS_ROOT = ROOT / "proofs" / "windows" / "studio"
WSL_ROOT = ROOT / "proofs" / "linux" / "execution" / "studio"


@dataclass(frozen=True)
class CommandRecord:
    id: str
    display_args: list[str]
    exit_code: int
    stdout_file: str
    stderr_file: str
    status: str


@dataclass(frozen=True)
class StudioSmokeEvidence:
    verified: bool
    windows_summary: Path | None
    wsl_summary: Path | None
    reason: str


def timestamp_slug(now: datetime | None = None) -> str:
    current = now or datetime.now(UTC)
    return current.strftime("%Y%m%d-%H%M%S")


def default_windows_output_dir(now: datetime | None = None) -> Path:
    return WINDOWS_ROOT / f"windows-studio-smoke-{timestamp_slug(now)}"


def default_wsl_output_dir(now: datetime | None = None) -> Path:
    return WSL_ROOT / f"wsl-studio-command-contract-{timestamp_slug(now)}"


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8", newline="\n")


def _bundle_relative(bundle_dir: Path, path: Path) -> str:
    return path.resolve().relative_to(bundle_dir.resolve()).as_posix()


def _write_manifest(bundle_dir: Path) -> None:
    entries: list[str] = []
    for path in sorted(p for p in bundle_dir.rglob("*") if p.is_file()):
        if path.name == MANIFEST_NAME:
            continue
        entries.append(f"{_sha256(path)}  {_bundle_relative(bundle_dir, path)}")
    _write_text(bundle_dir / MANIFEST_NAME, "\n".join(entries) + "\n")


def _manifest_entries(bundle_dir: Path) -> dict[str, str] | None:
    manifest = bundle_dir / MANIFEST_NAME
    if not manifest.is_file():
        return None
    entries: dict[str, str] = {}
    try:
        lines = manifest.read_text(encoding="utf-8").splitlines()
    except OSError:
        return None
    for line in lines:
        if not line.strip():
            continue
        match = re.fullmatch(r"([0-9a-f]{64})  ([^\r\n]+)", line)
        if not match:
            return None
        digest, relative = match.groups()
        relative_path = Path(relative)
        if relative_path.is_absolute() or ".." in relative_path.parts:
            return None
        target = bundle_dir / relative_path
        if not target.is_file() or _sha256(target) != digest:
            return None
        entries[relative.replace("\\", "/")] = digest
    return entries


def _run_command(
    *,
    command_id: str,
    command: list[str],
    display_args: list[str],
    bundle_dir: Path,
    cwd: Path = ROOT,
    env: dict[str, str] | None = None,
) -> CommandRecord:
    stdout_file = Path("commands") / f"{command_id}-stdout.txt"
    stderr_file = Path("commands") / f"{command_id}-stderr.txt"
    try:
        completed = subprocess.run(
            command,
            cwd=cwd,
            env=env,
            text=True,
            encoding="utf-8",
            errors="replace",
            capture_output=True,
            check=False,
        )
        stdout = completed.stdout
        stderr = completed.stderr
        exit_code = completed.returncode
    except FileNotFoundError as exc:
        stdout = ""
        stderr = f"missing executable: {command[0]} ({exc})\n"
        exit_code = 127
    _write_text(bundle_dir / stdout_file, stdout)
    _write_text(bundle_dir / stderr_file, stderr)
    return CommandRecord(
        id=command_id,
        display_args=display_args,
        exit_code=exit_code,
        stdout_file=stdout_file.as_posix(),
        stderr_file=stderr_file.as_posix(),
        status="passed" if exit_code == 0 else "failed",
    )


def _repo_relative(path: Path) -> str:
    try:
        return path.resolve().relative_to(ROOT.resolve()).as_posix()
    except ValueError:
        return path.as_posix()


def _studio_binary() -> Path:
    name = "garnet-studio.exe" if os.name == "nt" else "garnet-studio"
    candidates = [
        ROOT / "target" / "release" / name,
        ROOT / "apps" / "garnet-studio" / "src-tauri" / "target" / "release" / name,
    ]
    for candidate in candidates:
        if candidate.is_file():
            return candidate
    return candidates[0]


def _host_command(name: str) -> str:
    candidates = [name]
    if os.name == "nt":
        candidates = [f"{name}.cmd", f"{name}.exe", name]
    for candidate in candidates:
        resolved = shutil.which(candidate)
        if resolved:
            return resolved
    return candidates[0]


def _desktop_studio_root() -> Path:
    return Path.home() / "Desktop" / "dogfood" / "garnet-studio-windows-linux"


def _latest_studio_smoke_bundle(before: set[Path]) -> Path | None:
    root = _desktop_studio_root()
    if not root.exists():
        return None
    candidates = sorted(
        (
            path
            for path in root.glob("garnet-studio-windows-linux-smoke-*")
            if path.is_dir() and path not in before
        ),
        key=lambda path: path.stat().st_mtime,
        reverse=True,
    )
    for candidate in candidates:
        if (candidate / "studio-smoke.json").is_file():
            return candidate
    return None


def _existing_smoke_bundles() -> set[Path]:
    root = _desktop_studio_root()
    if not root.exists():
        return set()
    return {path for path in root.glob("garnet-studio-windows-linux-smoke-*") if path.is_dir()}


def _copy_smoke_payload(bundle_dir: Path, smoke_bundle: Path | None) -> dict[str, object]:
    if smoke_bundle is None:
        return {
            "bundle_path": "",
            "bundle_found": False,
            "studio_smoke_json": "",
            "studio_smoke_sha256": "",
        }
    smoke_json = smoke_bundle / "studio-smoke.json"
    if not smoke_json.is_file():
        return {
            "bundle_path": str(smoke_bundle),
            "bundle_found": False,
            "studio_smoke_json": "",
            "studio_smoke_sha256": "",
        }
    target = bundle_dir / "studio-smoke.json"
    target.write_bytes(smoke_json.read_bytes())
    return {
        "bundle_path": str(smoke_bundle),
        "bundle_found": True,
        "studio_smoke_json": "studio-smoke.json",
        "studio_smoke_sha256": _sha256(target),
    }


def _record_summary(bundle_dir: Path, data: dict[str, object]) -> Path:
    _write_text(bundle_dir / SUMMARY_NAME, json.dumps(data, indent=2, sort_keys=True) + "\n")
    _write_text(bundle_dir / MARKDOWN_NAME, render_markdown(data))
    _write_manifest(bundle_dir)
    return bundle_dir / SUMMARY_NAME


def _common_scope() -> list[str]:
    return [
        "Windows `--studio-smoke` is Tauri backend smoke evidence, not signed/package-manager proof",
        "WSL rows are execution/portability evidence only, not Linux seccomp or OS-sandbox enforcement",
        "No Linux desktop GUI launch, AppImage/deb/rpm package, Wasmtime fuel, production, or v1.0 claim is made",
        "Source is not included in the evidence bundle and no provider API is called",
    ]


def record_windows(*, output_dir: Path, build: bool = True) -> Path:
    output_dir.mkdir(parents=True, exist_ok=True)
    commands: list[CommandRecord] = []
    app_dir = ROOT / "apps" / "garnet-studio"
    env = os.environ.copy()
    env["GARNET_REPO"] = str(ROOT)
    npm = _host_command("npm")
    cargo = _host_command("cargo")

    if build:
        if not (app_dir / "node_modules").is_dir():
            commands.append(
                _run_command(
                    command_id="npm-install",
                    command=[npm, "install"],
                    display_args=["npm", "install"],
                    bundle_dir=output_dir,
                    cwd=app_dir,
                    env=env,
                )
            )
        commands.append(
            _run_command(
                command_id="npm-build",
                command=[npm, "run", "build"],
                display_args=["npm", "run", "build"],
                bundle_dir=output_dir,
                cwd=app_dir,
                env=env,
            )
        )
        commands.append(
            _run_command(
                command_id="cargo-build-studio-release",
                command=[
                    cargo,
                    "build",
                    "--manifest-path",
                    str(ROOT / "apps" / "garnet-studio" / "src-tauri" / "Cargo.toml"),
                    "--release",
                ],
                display_args=[
                    "cargo",
                    "build",
                    "--manifest-path",
                    "apps/garnet-studio/src-tauri/Cargo.toml",
                    "--release",
                ],
                bundle_dir=output_dir,
                env=env,
            )
        )

    binary = _studio_binary()
    before = _existing_smoke_bundles()
    if binary.is_file():
        commands.append(
            _run_command(
                command_id="studio-smoke",
                command=[str(binary), "--studio-smoke"],
                display_args=[_repo_relative(binary), "--studio-smoke"],
                bundle_dir=output_dir,
                env=env,
            )
        )
    else:
        _write_text(output_dir / "commands" / "studio-smoke-stdout.txt", "")
        _write_text(
            output_dir / "commands" / "studio-smoke-stderr.txt",
            f"missing Studio release binary: {binary}\n",
        )
        commands.append(
            CommandRecord(
                id="studio-smoke",
                display_args=[_repo_relative(binary), "--studio-smoke"],
                exit_code=1,
                stdout_file="commands/studio-smoke-stdout.txt",
                stderr_file="commands/studio-smoke-stderr.txt",
                status="failed",
            )
        )
    smoke_bundle = _latest_studio_smoke_bundle(before)
    smoke_payload = _copy_smoke_payload(output_dir, smoke_bundle)
    passed = all(command.status == "passed" for command in commands) and smoke_payload["bundle_found"] is True
    data: dict[str, object] = {
        "schema": SCHEMA,
        "status": "passed" if passed else "failed",
        "created_at": datetime.now(UTC).isoformat(),
        "target_platform": "windows",
        "platform_tier": "windows-local-tauri-studio-smoke",
        "source_included": False,
        "provider_api_called": False,
        "windows_studio_smoke_claimed": True,
        "wsl_execution_portability_claimed": False,
        "linux_enforcement_claimed": False,
        "linux_desktop_gui_claimed": False,
        "signed_msi_claimed": False,
        "winget_claimed": False,
        "windows_arm64_claimed": False,
        "binary": _repo_relative(binary),
        "binary_sha256": _sha256(binary) if binary.is_file() else "",
        "studio_smoke": smoke_payload,
        "commands": [asdict(command) for command in commands],
        "honest_scope": _common_scope(),
    }
    return _record_summary(output_dir, data)


def _wsl_repo_root() -> str:
    completed = subprocess.run(
        ["wsl.exe", "-e", "wslpath", "-a", str(ROOT)],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        raise RuntimeError(completed.stderr.strip() or completed.stdout.strip() or "wslpath failed")
    return completed.stdout.strip()


def record_wsl(*, output_dir: Path) -> Path:
    output_dir.mkdir(parents=True, exist_ok=True)
    commands: list[CommandRecord] = []
    wsl_root = _wsl_repo_root()
    commands.append(
        _run_command(
            command_id="wsl-uname",
            command=["wsl.exe", "-e", "bash", "-lc", "uname -a"],
            display_args=["wsl.exe", "-e", "bash", "-lc", "uname -a"],
            bundle_dir=output_dir,
        )
    )
    commands.append(
        _run_command(
            command_id="wsl-studio-status-json",
            command=[
                "wsl.exe",
                "-e",
                "bash",
                "-lc",
                f"cd {sh_quote(wsl_root)} && python3 scripts/garnet_windows_linux_studio_status.py --format json",
            ],
            display_args=[
                "wsl.exe",
                "-e",
                "bash",
                "-lc",
                "cd <repo> && python3 scripts/garnet_windows_linux_studio_status.py --format json",
            ],
            bundle_dir=output_dir,
        )
    )
    commands.append(
        _run_command(
            command_id="wsl-studio-status-tests",
            command=[
                "wsl.exe",
                "-e",
                "bash",
                "-lc",
                f"cd {sh_quote(wsl_root)} && python3 scripts/test_garnet_windows_linux_studio_status.py",
            ],
            display_args=[
                "wsl.exe",
                "-e",
                "bash",
                "-lc",
                "cd <repo> && python3 scripts/test_garnet_windows_linux_studio_status.py",
            ],
            bundle_dir=output_dir,
        )
    )
    passed = all(command.status == "passed" for command in commands)
    data: dict[str, object] = {
        "schema": SCHEMA,
        "status": "passed" if passed else "failed",
        "created_at": datetime.now(UTC).isoformat(),
        "target_platform": "wsl",
        "platform_tier": "execution/portability, not enforcement",
        "source_included": False,
        "provider_api_called": False,
        "windows_studio_smoke_claimed": False,
        "wsl_execution_portability_claimed": True,
        "linux_enforcement_claimed": False,
        "linux_desktop_gui_claimed": False,
        "signed_msi_claimed": False,
        "winget_claimed": False,
        "windows_arm64_claimed": False,
        "host_platform": platform.platform(),
        "wsl_repo_root": wsl_root,
        "commands": [asdict(command) for command in commands],
        "honest_scope": _common_scope(),
    }
    return _record_summary(output_dir, data)


def sh_quote(value: str) -> str:
    return "'" + value.replace("'", "'\"'\"'") + "'"


FORBIDDEN_WSL_CLAIMS = [
    "seccomp enforced",
    "os-sandbox enforced",
    "linux desktop gui passed",
    "linux package verified",
    "wasmtime fuel enforced",
    "production ready",
    "v1.0 ready",
]


def verify_summary(path: Path, *, expected_platform: str) -> bool:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return False
    bundle_dir = path.parent
    manifest = _manifest_entries(bundle_dir)
    if manifest is None:
        return False
    if not {SUMMARY_NAME, MARKDOWN_NAME}.issubset(manifest):
        return False
    commands = data.get("commands")
    if not isinstance(commands, list) or not commands:
        return False
    for command in commands:
        if not isinstance(command, dict) or command.get("status") != "passed":
            return False
        stdout_file = command.get("stdout_file")
        stderr_file = command.get("stderr_file")
        if not isinstance(stdout_file, str) or not isinstance(stderr_file, str):
            return False
        if stdout_file not in manifest or stderr_file not in manifest:
            return False
    common_ok = (
        data.get("schema") == SCHEMA
        and data.get("status") == "passed"
        and data.get("target_platform") == expected_platform
        and data.get("source_included") is False
        and data.get("provider_api_called") is False
        and data.get("linux_enforcement_claimed") is False
        and data.get("linux_desktop_gui_claimed") is False
        and data.get("signed_msi_claimed") is False
        and data.get("winget_claimed") is False
        and data.get("windows_arm64_claimed") is False
    )
    if not common_ok:
        return False
    if expected_platform == "windows":
        command_text = " ".join(
            " ".join(command.get("display_args", []))
            for command in commands
            if isinstance(command, dict)
        )
        return (
            data.get("platform_tier") == "windows-local-tauri-studio-smoke"
            and data.get("windows_studio_smoke_claimed") is True
            and data.get("wsl_execution_portability_claimed") is False
            and "--studio-smoke" in command_text
            and "studio-smoke.json" in manifest
        )
    if expected_platform == "wsl":
        text = json.dumps(data, sort_keys=True).lower()
        return (
            data.get("platform_tier") == "execution/portability, not enforcement"
            and data.get("windows_studio_smoke_claimed") is False
            and data.get("wsl_execution_portability_claimed") is True
            and all(claim not in text for claim in FORBIDDEN_WSL_CLAIMS)
        )
    return False


def _verified_summary_under(root: Path, *, expected_platform: str) -> Path | None:
    if not root.exists():
        return None
    candidates = sorted(
        root.rglob(SUMMARY_NAME),
        key=lambda path: path.stat().st_mtime,
        reverse=True,
    )
    for candidate in candidates:
        if verify_summary(candidate, expected_platform=expected_platform):
            return candidate
    return None


def _repo_display(path: Path) -> str:
    try:
        return path.resolve().relative_to(ROOT.resolve()).as_posix()
    except ValueError:
        return path.as_posix()


def read_committed_evidence(repo_root: Path = ROOT) -> StudioSmokeEvidence:
    windows = _verified_summary_under(repo_root / "proofs" / "windows" / "studio", expected_platform="windows")
    wsl = _verified_summary_under(repo_root / "proofs" / "linux" / "execution" / "studio", expected_platform="wsl")
    if windows is None or wsl is None:
        return StudioSmokeEvidence(
            False,
            windows,
            wsl,
            (
                "No complete committed Windows + WSL Studio smoke pair exists yet. "
                "Expected a Windows Tauri `--studio-smoke` bundle and a WSL command-contract "
                "bundle under `proofs/windows/studio/` and `proofs/linux/execution/studio/`."
            ),
        )
    return StudioSmokeEvidence(
        True,
        windows,
        wsl,
        (
            f"Committed Windows Studio smoke bundle: `{_repo_display(windows)}`. "
            f"Committed WSL command-contract portability bundle: `{_repo_display(wsl)}`. "
            "The WSL row is execution/portability evidence only, not Linux seccomp, "
            "OS-sandbox enforcement, Wasmtime fuel, Linux desktop GUI, or native Linux package proof."
        ),
    )


def render_markdown(data: dict[str, object]) -> str:
    lines = [
        "# Garnet Studio Windows/WSL Smoke Proof",
        "",
        f"- Status: `{data.get('status')}`",
        f"- Target platform: `{data.get('target_platform')}`",
        f"- Platform tier: `{data.get('platform_tier')}`",
        f"- Source included: `{str(data.get('source_included')).lower()}`",
        f"- Provider API called: `{str(data.get('provider_api_called')).lower()}`",
        "",
        "## Commands",
        "",
        "| Command | Exit | Status |",
        "| --- | ---: | --- |",
    ]
    for command in data.get("commands", []):
        if isinstance(command, dict):
            lines.append(
                f"| `{' '.join(command.get('display_args', []))}` | {command.get('exit_code')} | `{command.get('status')}` |"
            )
    lines.extend(
        [
            "",
            "## Honest Scope",
            "",
            *[f"- {scope}" for scope in data.get("honest_scope", []) if isinstance(scope, str)],
        ]
    )
    return "\n".join(lines) + "\n"


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--record", action="store_true", help="record Windows and WSL evidence")
    parser.add_argument("--gate", action="store_true", help="verify committed Windows and WSL evidence")
    parser.add_argument("--windows-output-dir", type=Path)
    parser.add_argument("--wsl-output-dir", type=Path)
    parser.add_argument("--skip-windows", action="store_true")
    parser.add_argument("--skip-wsl", action="store_true")
    parser.add_argument("--no-build", action="store_true", help="run the existing Studio binary without rebuilding")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    if args.record:
        written: list[Path] = []
        if not args.skip_windows:
            written.append(
                record_windows(
                    output_dir=args.windows_output_dir or default_windows_output_dir(),
                    build=not args.no_build,
                )
            )
        if not args.skip_wsl:
            if shutil.which("wsl.exe") is None:
                print("WSL is not available; cannot record WSL portability evidence.", file=sys.stderr)
                return 1
            written.append(record_wsl(output_dir=args.wsl_output_dir or default_wsl_output_dir()))
        for path in written:
            print(path)
        return 0 if all(verify_summary(path, expected_platform=json.loads(path.read_text(encoding="utf-8"))["target_platform"]) for path in written) else 1

    evidence = read_committed_evidence()
    if args.gate:
        print(json.dumps(asdict(evidence), indent=2, default=str))
        return 0 if evidence.verified else 1
    print(evidence.reason)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
