#!/usr/bin/env python3
"""Record Studio Release / Readiness shell proof on Windows and WSL.

This is a Windows-lane productization proof. It exercises the Tauri Studio
binary's `--studio-release-readiness-smoke` command, which routes through the
same Rust command wrappers as the UI's Release / Readiness panel. The WSL row is
execution/portability evidence only; it is not Linux seccomp, OS-sandbox, or
clean non-WSL Linux desktop evidence.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SUMMARY_NAME = "garnet-studio-release-readiness-shell-proof.json"
MARKDOWN_NAME = "garnet-studio-release-readiness-shell-proof.md"
MANIFEST_NAME = "MANIFEST.sha256"
SCHEMA = "garnet.studio.release_readiness_shell_proof.v1"
WINDOWS_ROOT = ROOT / "proofs" / "windows" / "studio-release-readiness-shell"
WSL_ROOT = ROOT / "proofs" / "linux" / "execution" / "studio-release-readiness-shell"
PAYLOAD_NAME = "release-readiness-shell-smoke.json"
REQUIRED_COMMAND_IDS = {
    "windows-linux-studio-status",
    "objective-pulse",
    "converter-status",
    "windows-vm-installer-status",
}


@dataclass(frozen=True)
class CommandRecord:
    id: str
    display_args: list[str]
    exit_code: int
    stdout_file: str
    stderr_file: str
    status: str


@dataclass(frozen=True)
class ReleaseReadinessShellEvidence:
    verified: bool
    windows_summary: Path | None
    wsl_summary: Path | None
    reason: str


def timestamp_slug(now: datetime | None = None) -> str:
    return (now or datetime.now(UTC)).strftime("%Y%m%d-%H%M%S")


def default_windows_output_dir(now: datetime | None = None) -> Path:
    return WINDOWS_ROOT / f"win-{timestamp_slug(now)}"


def default_wsl_output_dir(now: datetime | None = None) -> Path:
    return WSL_ROOT / f"wsl-{timestamp_slug(now)}"


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


def _host_command(name: str) -> str:
    candidates = [name]
    if os.name == "nt":
        candidates = [f"{name}.cmd", f"{name}.exe", name]
    for candidate in candidates:
        resolved = shutil.which(candidate)
        if resolved:
            return resolved
    return candidates[0]


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


def _extract_evidence_path(stdout: str) -> str:
    for line in stdout.splitlines():
        if line.startswith("evidence="):
            return line.split("=", 1)[1].strip()
    return ""


def _copy_tree_payload(source: Path, target: Path) -> str:
    if not source.is_dir():
        return ""
    if target.exists():
        raise RuntimeError(f"payload target already exists: {target}")
    shutil.copytree(source, target)
    return target.name


def _copied_payload_summary(bundle_dir: Path, payload_dir: str) -> dict[str, object]:
    if not payload_dir:
        return {
            "copied": False,
            "payload_dir": "",
            "payload_json": "",
            "payload_status": "",
            "payload_mode": "",
            "verified_command_ids": [],
        }
    payload = bundle_dir / payload_dir
    payload_json = payload / PAYLOAD_NAME
    try:
        data = json.loads(payload_json.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {
            "copied": True,
            "payload_dir": payload_dir,
            "payload_json": "",
            "payload_status": "",
            "payload_mode": "",
            "verified_command_ids": [],
        }
    commands = data.get("release_readiness_commands", [])
    verified_ids = [
        command.get("id")
        for command in commands
        if isinstance(command, dict)
        and command.get("success") is True
        and command.get("exit_code") == 0
        and command.get("stdout_has_expected_heading") is True
    ]
    return {
        "copied": True,
        "payload_dir": payload_dir,
        "payload_json": f"{payload_dir}/{PAYLOAD_NAME}",
        "payload_status": data.get("status", ""),
        "payload_mode": data.get("mode", ""),
        "verified_command_ids": sorted(str(item) for item in verified_ids),
    }


def _record_summary(bundle_dir: Path, data: dict[str, object]) -> Path:
    _write_text(bundle_dir / SUMMARY_NAME, json.dumps(data, indent=2, sort_keys=True) + "\n")
    _write_text(bundle_dir / MARKDOWN_NAME, render_markdown(data))
    _write_manifest(bundle_dir)
    return bundle_dir / SUMMARY_NAME


def _common_scope() -> list[str]:
    return [
        "Studio release/readiness shell proof exercises the Tauri command wrappers behind the Release / Readiness panel.",
        "WSL rows are execution/portability evidence only, not Linux seccomp or OS-sandbox enforcement.",
        "This is not clean/non-WSL Linux desktop GUI install/launch proof.",
        "No signed MSI, winget, Windows ARM64, production, or v1.0 claim is made.",
        "Source is not included in the evidence bundle and no provider API is called.",
    ]


def _payload_ok(payload: dict[str, object]) -> bool:
    if payload.get("payload_status") != "passed" or payload.get("payload_mode") != "studio-release-readiness-smoke":
        return False
    return REQUIRED_COMMAND_IDS.issubset(set(payload.get("verified_command_ids", [])))


def record_windows(*, output_dir: Path, build: bool = True) -> Path:
    output_dir.mkdir(parents=True, exist_ok=True)
    commands: list[CommandRecord] = []
    env = os.environ.copy()
    env["GARNET_REPO"] = str(ROOT)
    app_dir = ROOT / "apps" / "garnet-studio"
    npm = _host_command("npm")
    cargo = _host_command("cargo")

    if build:
        commands.append(
            _run_command(
                command_id="cargo-build-garnet-cli",
                command=[cargo, "build", "-p", "garnet-cli", "--release"],
                display_args=["cargo", "build", "-p", "garnet-cli", "--release"],
                bundle_dir=output_dir,
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
    commands.append(
        _run_command(
            command_id="studio-release-readiness-smoke",
            command=[str(binary), "--studio-release-readiness-smoke"],
            display_args=[_repo_relative(binary), "--studio-release-readiness-smoke"],
            bundle_dir=output_dir,
            env=env,
        )
    )
    stdout = (output_dir / "commands" / "studio-release-readiness-smoke-stdout.txt").read_text(
        encoding="utf-8"
    )
    payload_path = Path(_extract_evidence_path(stdout))
    payload_dir = _copy_tree_payload(payload_path, output_dir / "studio-payload")
    payload = _copied_payload_summary(output_dir, payload_dir)
    passed = all(command.status == "passed" for command in commands) and _payload_ok(payload)
    data: dict[str, object] = {
        "schema": SCHEMA,
        "status": "passed" if passed else "failed",
        "created_at": datetime.now(UTC).isoformat(),
        "target_platform": "windows",
        "platform_tier": "windows-local-tauri-release-readiness-shell-proof",
        "source_included": False,
        "provider_api_called": False,
        "release_readiness_shell_proven": _payload_ok(payload),
        "studio_command_path_proven": True,
        "wsl_execution_portability_claimed": False,
        "wsl_is_enforcement": False,
        "linux_enforcement_proven": False,
        "linux_desktop_gui_claimed": False,
        "clean_linux_install_proven": False,
        "non_wsl_linux_desktop_proven": False,
        "signed_msi_claimed": False,
        "winget_claimed": False,
        "windows_arm64_claimed": False,
        "binary": _repo_relative(binary),
        "binary_sha256": _sha256(binary) if binary.is_file() else "",
        "studio_payload": payload,
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


def _windows_path_from_wsl_repo_path(wsl_path: str, wsl_root: str) -> Path:
    if not wsl_path.startswith(wsl_root.rstrip("/") + "/"):
        return Path()
    relative = wsl_path[len(wsl_root.rstrip("/") + "/") :]
    return ROOT / Path(*relative.split("/"))


def sh_quote(value: str) -> str:
    return "'" + value.replace("'", "'\"'\"'") + "'"


def record_wsl(*, output_dir: Path, build: bool = True) -> Path:
    output_dir.mkdir(parents=True, exist_ok=True)
    commands: list[CommandRecord] = []
    wsl_root = _wsl_repo_root()
    stamp = timestamp_slug()
    home = f"{wsl_root}/target/wsl-release-readiness-shell-home-{stamp}"
    commands.append(
        _run_command(
            command_id="wsl-uname",
            command=["wsl.exe", "-e", "bash", "-lc", "uname -a"],
            display_args=["wsl.exe", "-e", "bash", "-lc", "uname -a"],
            bundle_dir=output_dir,
        )
    )
    if build:
        commands.append(
            _run_command(
                command_id="wsl-cargo-build-garnet-cli",
                command=[
                    "wsl.exe",
                    "-e",
                    "bash",
                    "-lc",
                    f"cd {sh_quote(wsl_root)} && cargo build -p garnet-cli --release",
                ],
                display_args=[
                    "wsl.exe",
                    "-e",
                    "bash",
                    "-lc",
                    "cd <repo> && cargo build -p garnet-cli --release",
                ],
                bundle_dir=output_dir,
            )
        )
        commands.append(
            _run_command(
                command_id="wsl-cargo-build-studio-release",
                command=[
                    "wsl.exe",
                    "-e",
                    "bash",
                    "-lc",
                    (
                        f"cd {sh_quote(wsl_root)} && "
                        "cargo build --manifest-path apps/garnet-studio/src-tauri/Cargo.toml --release"
                    ),
                ],
                display_args=[
                    "wsl.exe",
                    "-e",
                    "bash",
                    "-lc",
                    "cd <repo> && cargo build --manifest-path apps/garnet-studio/src-tauri/Cargo.toml --release",
                ],
                bundle_dir=output_dir,
            )
        )
    smoke_script = (
        f"cd {sh_quote(wsl_root)} && mkdir -p {sh_quote(home + '/Desktop')} && "
        f"HOME={sh_quote(home)} GARNET_REPO={sh_quote(wsl_root)} "
        "./target/release/garnet-studio --studio-release-readiness-smoke"
    )
    commands.append(
        _run_command(
            command_id="wsl-studio-release-readiness-smoke",
            command=["wsl.exe", "-e", "bash", "-lc", smoke_script],
            display_args=[
                "wsl.exe",
                "-e",
                "bash",
                "-lc",
                "cd <repo> && HOME=<repo>/target/wsl-release-readiness-shell-home-* ./target/release/garnet-studio --studio-release-readiness-smoke",
            ],
            bundle_dir=output_dir,
        )
    )
    stdout = (output_dir / "commands" / "wsl-studio-release-readiness-smoke-stdout.txt").read_text(
        encoding="utf-8"
    )
    payload_path = _windows_path_from_wsl_repo_path(_extract_evidence_path(stdout), wsl_root)
    payload_dir = _copy_tree_payload(payload_path, output_dir / "studio-payload")
    payload = _copied_payload_summary(output_dir, payload_dir)
    passed = all(command.status == "passed" for command in commands) and _payload_ok(payload)
    data: dict[str, object] = {
        "schema": SCHEMA,
        "status": "passed" if passed else "failed",
        "created_at": datetime.now(UTC).isoformat(),
        "target_platform": "wsl",
        "platform_tier": "execution/portability, not enforcement",
        "source_included": False,
        "provider_api_called": False,
        "release_readiness_shell_proven": _payload_ok(payload),
        "studio_command_path_proven": True,
        "wsl_execution_portability_claimed": True,
        "wsl_is_enforcement": False,
        "linux_enforcement_proven": False,
        "linux_desktop_gui_claimed": False,
        "clean_linux_install_proven": False,
        "non_wsl_linux_desktop_proven": False,
        "signed_msi_claimed": False,
        "winget_claimed": False,
        "windows_arm64_claimed": False,
        "host_platform": os.environ.get("OS", ""),
        "wsl_repo_root": wsl_root,
        "studio_payload": payload,
        "commands": [asdict(command) for command in commands],
        "honest_scope": _common_scope(),
    }
    return _record_summary(output_dir, data)


FORBIDDEN_CLAIMS = [
    "seccomp enforced",
    "os-sandbox enforced",
    "clean linux desktop passed",
    "non-wsl linux desktop passed",
    "production ready",
    "v1.0 ready",
    "winget verified",
    "signed msi verified",
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
    if not {SUMMARY_NAME, MARKDOWN_NAME, f"studio-payload/{PAYLOAD_NAME}"}.issubset(manifest):
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
    try:
        payload = json.loads((bundle_dir / "studio-payload" / PAYLOAD_NAME).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return False
    payload_commands = payload.get("release_readiness_commands")
    if not isinstance(payload_commands, list):
        return False
    verified_ids = {
        command.get("id")
        for command in payload_commands
        if isinstance(command, dict)
        and command.get("success") is True
        and command.get("exit_code") == 0
        and command.get("stdout_has_expected_heading") is True
    }
    common_ok = (
        data.get("schema") == SCHEMA
        and data.get("status") == "passed"
        and data.get("target_platform") == expected_platform
        and data.get("source_included") is False
        and data.get("provider_api_called") is False
        and data.get("release_readiness_shell_proven") is True
        and data.get("studio_command_path_proven") is True
        and data.get("wsl_is_enforcement") is False
        and data.get("linux_enforcement_proven") is False
        and data.get("linux_desktop_gui_claimed") is False
        and data.get("clean_linux_install_proven") is False
        and data.get("non_wsl_linux_desktop_proven") is False
        and data.get("signed_msi_claimed") is False
        and data.get("winget_claimed") is False
        and data.get("windows_arm64_claimed") is False
        and payload.get("status") == "passed"
        and payload.get("mode") == "studio-release-readiness-smoke"
        and REQUIRED_COMMAND_IDS.issubset(verified_ids)
    )
    if not common_ok:
        return False
    text = json.dumps(data, sort_keys=True).lower()
    if any(claim in text for claim in FORBIDDEN_CLAIMS):
        return False
    if expected_platform == "windows":
        return (
            data.get("platform_tier") == "windows-local-tauri-release-readiness-shell-proof"
            and data.get("wsl_execution_portability_claimed") is False
        )
    if expected_platform == "wsl":
        return (
            data.get("platform_tier") == "execution/portability, not enforcement"
            and data.get("wsl_execution_portability_claimed") is True
        )
    return False


def _verified_summary_under(root: Path, *, expected_platform: str) -> Path | None:
    if not root.exists():
        return None
    candidates = sorted(root.rglob(SUMMARY_NAME), key=lambda path: path.stat().st_mtime, reverse=True)
    for candidate in candidates:
        if verify_summary(candidate, expected_platform=expected_platform):
            return candidate
    return None


def _repo_display(path: Path) -> str:
    try:
        return path.resolve().relative_to(ROOT.resolve()).as_posix()
    except ValueError:
        return path.as_posix()


def read_committed_evidence(repo_root: Path = ROOT) -> ReleaseReadinessShellEvidence:
    windows = _verified_summary_under(
        repo_root / "proofs" / "windows" / "studio-release-readiness-shell",
        expected_platform="windows",
    )
    wsl = _verified_summary_under(
        repo_root / "proofs" / "linux" / "execution" / "studio-release-readiness-shell",
        expected_platform="wsl",
    )
    if windows is None or wsl is None:
        return ReleaseReadinessShellEvidence(
            False,
            windows,
            wsl,
            (
                "No complete committed Windows + WSL Studio release/readiness shell proof pair exists yet. "
                "Expected Windows Tauri `--studio-release-readiness-smoke` and WSL portability bundles."
            ),
        )
    return ReleaseReadinessShellEvidence(
        True,
        windows,
        wsl,
        (
            f"Committed Windows Studio Release / Readiness shell proof: `{_repo_display(windows)}`. "
            f"Committed WSL Studio Release / Readiness portability proof: `{_repo_display(wsl)}`. "
            "The WSL row is execution/portability evidence only, not Linux seccomp, "
            "OS-sandbox enforcement, clean/non-WSL Linux desktop GUI proof, production, or v1.0 readiness."
        ),
    )


def render_markdown(data: dict[str, object]) -> str:
    lines = [
        "# Garnet Studio Release / Readiness Shell Proof",
        "",
        f"- Status: `{data.get('status')}`",
        f"- Target platform: `{data.get('target_platform')}`",
        f"- Platform tier: `{data.get('platform_tier')}`",
        f"- Release/readiness shell proven: `{str(data.get('release_readiness_shell_proven')).lower()}`",
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
    payload = data.get("studio_payload", {})
    if isinstance(payload, dict):
        lines.extend(
            [
                "",
                "## Studio Payload",
                "",
                f"- Payload: `{payload.get('payload_json', '')}`",
                f"- Mode: `{payload.get('payload_mode', '')}`",
                f"- Verified reporter commands: `{', '.join(payload.get('verified_command_ids', []))}`",
            ]
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


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--record", action="store_true", help="record Windows and WSL evidence")
    parser.add_argument("--gate", action="store_true", help="verify committed Windows and WSL evidence")
    parser.add_argument("--windows-output-dir", type=Path)
    parser.add_argument("--wsl-output-dir", type=Path)
    parser.add_argument("--skip-windows", action="store_true")
    parser.add_argument("--skip-wsl", action="store_true")
    parser.add_argument("--no-build", action="store_true", help="run existing binaries without rebuilding")
    parser.add_argument("--format", choices=("text", "json"), default="text")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    summaries: list[Path] = []
    if args.record:
        if not args.skip_windows:
            summaries.append(
                record_windows(
                    output_dir=args.windows_output_dir or default_windows_output_dir(),
                    build=not args.no_build,
                )
            )
        if not args.skip_wsl:
            summaries.append(
                record_wsl(
                    output_dir=args.wsl_output_dir or default_wsl_output_dir(),
                    build=not args.no_build,
                )
            )
    evidence = read_committed_evidence(ROOT)
    ok = evidence.verified
    if args.format == "json":
        print(
            json.dumps(
                {
                    "ok": ok,
                    "recorded": [str(path) for path in summaries],
                    "windows_summary": str(evidence.windows_summary) if evidence.windows_summary else "",
                    "wsl_summary": str(evidence.wsl_summary) if evidence.wsl_summary else "",
                    "reason": evidence.reason,
                },
                indent=2,
            )
        )
    else:
        print(("ok: " if ok else "not ok: ") + evidence.reason)
        for summary in summaries:
            print(f"recorded: {summary}")
    if args.gate and not ok:
        return 1
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
