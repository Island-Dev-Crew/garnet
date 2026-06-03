#!/usr/bin/env python3
"""Record and verify a WSLg Garnet Studio system install/launch proof.

This is an S117 package-pipeline increment. It proves that this Windows box can
drive WSL to install the Tauri Linux `.deb` as a privileged system package, run
the installed binary's non-GUI smoke command, and observe a WSLg/X11 window from
the installed Linux Studio binary. It is still WSL/WSLg portability evidence:
not a clean Linux install, not non-WSL desktop Linux proof, and not Linux
seccomp or OS-sandbox enforcement.
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
SCHEMA = "garnet.studio.linux_wslg_system_install_launch.v1"
SUMMARY_NAME = "garnet-studio-linux-wslg-system-install-launch.json"
MARKDOWN_NAME = "garnet-studio-linux-wslg-system-install-launch.md"
MANIFEST_NAME = "MANIFEST.sha256"
DEFAULT_PROOF_ROOT = ROOT / "proofs" / "linux" / "execution" / "studio-wslg-system-install"
PACKAGE_NAME = "garnet-studio"
INSTALLED_BINARY = "/usr/bin/garnet-studio"
REQUIRED_COMMANDS = [
    "wsl-uname",
    "wslg-env",
    "npm-install",
    "npm-build",
    "tauri-build-deb",
    "dpkg-status-before",
    "dpkg-install",
    "dpkg-status-after-install",
    "installed-binary-ls",
    "installed-studio-smoke",
    "wslg-launch",
    "dpkg-remove",
    "dpkg-status-after-remove",
]
REQUIRED_HONEST_SCOPE = [
    "not Linux desktop GUI proof outside WSLg",
    "not Linux seccomp or OS-sandbox enforcement",
    "not clean Linux install proof",
    "not signed, production, or v1.0 readiness",
]
FORBIDDEN_CLAIMS = [
    "Linux desktop GUI proof outside WSLg verified",
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
class LinuxWslgInstallLaunchEvidence:
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
        if "\\" in relative:
            return None
        target = directory / Path(relative)
        if not target.is_file() or _sha256(target) != digest:
            return None
        entries[relative] = digest
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


def _package_name_from_info(text: str) -> str:
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.lower().startswith("package:"):
            return stripped.split(":", 1)[1].strip()
    return PACKAGE_NAME


def _package_arch_from_info(text: str) -> str:
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.lower().startswith("architecture:"):
            return stripped.split(":", 1)[1].strip()
    return "unknown"


def _privileged(command: str) -> str:
    return f'if [ "$(id -u)" -eq 0 ]; then {command}; else sudo -n sh -lc {json.dumps(command)}; fi'


def _status_command(package_name: str) -> str:
    return f"dpkg -s {package_name} || true"


def _launch_command(stage_rel: str) -> str:
    stdout_rel = f"{stage_rel}/app-stdout.txt"
    stderr_rel = f"{stage_rel}/app-stderr.txt"
    pid_rel = f"{stage_rel}/studio.pid"
    process_rel = f"{stage_rel}/process.txt"
    xwininfo_rel = f"{stage_rel}/xwininfo.txt"
    xwininfo_stderr_rel = f"{stage_rel}/xwininfo.stderr"
    return " ".join(
        [
            "set -eu;",
            f"mkdir -p {stage_rel};",
            f"rm -f {stdout_rel} {stderr_rel} {pid_rel} {process_rel} {xwininfo_rel} {xwininfo_stderr_rel};",
            "export GDK_BACKEND=x11;",
            "export WEBKIT_DISABLE_DMABUF_RENDERER=1;",
            "export WEBKIT_DISABLE_COMPOSITING_MODE=1;",
            "export NO_AT_BRIDGE=1;",
            f"({INSTALLED_BINARY} > {stdout_rel} 2> {stderr_rel} & echo $! > {pid_rel});",
            "sleep 6;",
            f"pid=$(cat {pid_rel});",
            f"if kill -0 \"$pid\" 2>/dev/null; then ps -p \"$pid\" -o pid=,comm=,args= > {process_rel}; else cat {stderr_rel}; exit 1; fi;",
            f"xwininfo -root -tree > {xwininfo_rel} 2> {xwininfo_stderr_rel} || true;",
            f"cat {process_rel};",
            f"cat {xwininfo_rel};",
            f"if ! grep -Eiq 'Garnet Studio|garnet-studio|webkit|tauri' {xwininfo_rel}; then cat {xwininfo_stderr_rel}; kill \"$pid\" 2>/dev/null || true; wait \"$pid\" 2>/dev/null || true; exit 1; fi;",
            "kill \"$pid\" 2>/dev/null || true;",
            "wait \"$pid\" 2>/dev/null || true;",
            f"echo '--app-stdout--'; cat {stdout_rel};",
            f"echo '--app-stderr--'; cat {stderr_rel};",
        ]
    )


def record_proof(output_dir: Path | None = None) -> int:
    stamp = _timestamp()
    bundle = output_dir or DEFAULT_PROOF_ROOT / f"linux-wslg-system-install-{stamp}"
    bundle.mkdir(parents=True, exist_ok=True)
    wsl_root = _wsl_root()
    stage_rel = f"target/linux-wslg-system-install-{stamp}"

    commands: list[CommandRecord] = [
        _run_wsl(
            command_id="wsl-uname",
            shell="uname -a",
            display_args=["wsl.exe", "-e", "uname", "-a"],
            output_dir=bundle,
            wsl_root=wsl_root,
        ),
        _run_wsl(
            command_id="wslg-env",
            shell=(
                "printf 'DISPLAY=%s\\nWAYLAND_DISPLAY=%s\\nXDG_RUNTIME_DIR=%s\\n' "
                "\"${DISPLAY:-}\" \"${WAYLAND_DISPLAY:-}\" \"${XDG_RUNTIME_DIR:-}\"; "
                "test -n \"${DISPLAY:-}\"; test -n \"${WAYLAND_DISPLAY:-}\"; command -v xwininfo"
            ),
            display_args=["env", "DISPLAY/WAYLAND_DISPLAY", "command -v xwininfo"],
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
    ]

    deb = _find_deb()
    package_info_text = ""
    package_name = PACKAGE_NAME
    package_arch = "unknown"
    if deb is not None:
        info_completed = _run_host(["wsl.exe", "-e", "sh", "-lc", f"cd '{wsl_root}' && dpkg-deb --info target/release/bundle/deb/*.deb"])
        package_info_text = info_completed.stdout
        package_name = _package_name_from_info(package_info_text)
        package_arch = _package_arch_from_info(package_info_text)
        _write_text(bundle / "package" / "dpkg-info.txt", package_info_text)
        _write_text(bundle / "package" / "dpkg-info.stderr", info_completed.stderr)

    commands.extend(
        [
            _run_wsl(
                command_id="dpkg-status-before",
                shell=_status_command(package_name),
                display_args=["dpkg", "-s", package_name],
                output_dir=bundle,
                wsl_root=wsl_root,
            ),
            _run_wsl(
                command_id="dpkg-install",
                shell=_privileged("dpkg -i target/release/bundle/deb/*.deb"),
                display_args=["dpkg", "-i", "target/release/bundle/deb/*.deb"],
                output_dir=bundle,
                wsl_root=wsl_root,
            ),
            _run_wsl(
                command_id="dpkg-status-after-install",
                shell=_status_command(package_name),
                display_args=["dpkg", "-s", package_name],
                output_dir=bundle,
                wsl_root=wsl_root,
            ),
            _run_wsl(
                command_id="installed-binary-ls",
                shell=f"ls -l {INSTALLED_BINARY}",
                display_args=["ls", "-l", INSTALLED_BINARY],
                output_dir=bundle,
                wsl_root=wsl_root,
            ),
            _run_wsl(
                command_id="installed-studio-smoke",
                shell=f"{INSTALLED_BINARY} --studio-smoke",
                display_args=[INSTALLED_BINARY, "--studio-smoke"],
                output_dir=bundle,
                wsl_root=wsl_root,
            ),
            _run_wsl(
                command_id="wslg-launch",
                shell=_launch_command(stage_rel),
                display_args=[INSTALLED_BINARY, "(WSLg launch probe)"],
                output_dir=bundle,
                wsl_root=wsl_root,
            ),
            _run_wsl(
                command_id="dpkg-remove",
                shell=_privileged(f"dpkg -r {package_name}"),
                display_args=["dpkg", "-r", package_name],
                output_dir=bundle,
                wsl_root=wsl_root,
            ),
            _run_wsl(
                command_id="dpkg-status-after-remove",
                shell=_status_command(package_name),
                display_args=["dpkg", "-s", package_name],
                output_dir=bundle,
                wsl_root=wsl_root,
            ),
        ]
    )

    for command_name, target_name in [
        ("dpkg-status-before", "dpkg-status-before.txt"),
        ("dpkg-status-after-install", "dpkg-status-after-install.txt"),
        ("dpkg-status-after-remove", "dpkg-status-after-remove.txt"),
    ]:
        _write_text(
            bundle / "package" / target_name,
            _read_text(bundle / "commands" / f"{command_name}-stdout.txt")
            + _read_text(bundle / "commands" / f"{command_name}-stderr.txt"),
        )
    _write_text(
        bundle / "installed" / "studio-smoke.json",
        _read_text(bundle / "commands" / "installed-studio-smoke-stdout.txt"),
    )
    launch_stage = ROOT / stage_rel
    for source, target in [
        ("app-stdout.txt", "wslg-launch-stdout.txt"),
        ("app-stderr.txt", "wslg-launch-stderr.txt"),
        ("process.txt", "process.txt"),
        ("xwininfo.txt", "xwininfo.txt"),
        ("xwininfo.stderr", "xwininfo.stderr"),
    ]:
        _write_text(bundle / "launch" / target, _read_text(launch_stage / source))

    status_before = _read_text(bundle / "package" / "dpkg-status-before.txt")
    status_after_install = _read_text(bundle / "package" / "dpkg-status-after-install.txt")
    status_after_remove = _read_text(bundle / "package" / "dpkg-status-after-remove.txt")
    installed_smoke = _read_text(bundle / "installed" / "studio-smoke.json")
    process_text = _read_text(bundle / "launch" / "process.txt")
    xwininfo_text = _read_text(bundle / "launch" / "xwininfo.txt")

    all_commands_passed = all(command.status == "passed" for command in commands)
    absent_before = "not installed" in status_before or "no information is available" in status_before
    install_ok = "Status: install ok installed" in status_after_install
    removed_ok = "not installed" in status_after_remove or "no information is available" in status_after_remove
    smoke_lower = installed_smoke.lower()
    smoke_ok = ("passed" in smoke_lower) or ('"status"' in installed_smoke and "ok" in smoke_lower)
    process_observed = "garnet-studio" in process_text.lower()
    window_observed = any(token in xwininfo_text.lower() for token in ["garnet studio", "garnet-studio", "webkit", "tauri"])
    wslg_env = _read_text(bundle / "commands" / "wslg-env-stdout.txt")
    display = ""
    wayland_display = ""
    xdg_runtime_dir = ""
    for line in wslg_env.splitlines():
        if line.startswith("DISPLAY="):
            display = line.split("=", 1)[1]
        elif line.startswith("WAYLAND_DISPLAY="):
            wayland_display = line.split("=", 1)[1]
        elif line.startswith("XDG_RUNTIME_DIR="):
            xdg_runtime_dir = line.split("=", 1)[1]

    passed = (
        all_commands_passed
        and deb is not None
        and absent_before
        and install_ok
        and smoke_ok
        and process_observed
        and window_observed
        and removed_ok
    )
    data = {
        "schema": SCHEMA,
        "recorded_at": datetime.now(UTC).isoformat(),
        "status": "passed" if passed else "failed",
        "platform": "linux",
        "evidence_tier": "wslg-system-package-install-launch",
        "wsl_is_enforcement": False,
        "source_included": False,
        "provider_api_called": False,
        "desktop_gui_launch_proven": False,
        "wslg_gui_launch_proven": bool(process_observed and window_observed),
        "clean_linux_install_proven": False,
        "privileged_system_install_proven": bool(install_ok),
        "linux_enforcement_proven": False,
        "package": {
            "format": "deb",
            "path": str(deb.relative_to(ROOT)).replace("\\", "/") if deb else None,
            "sha256": _sha256(deb) if deb and deb.is_file() else None,
            "size_bytes": deb.stat().st_size if deb and deb.is_file() else 0,
            "package_name": package_name,
            "architecture": package_arch,
        },
        "install": {
            "method": "dpkg -i",
            "installed_binary": INSTALLED_BINARY,
            "package_absent_before_record": bool(absent_before),
            "package_status_before_file": "package/dpkg-status-before.txt",
            "package_status_after_install_file": "package/dpkg-status-after-install.txt",
            "package_removed_after_record": bool(removed_ok),
            "package_status_after_remove_file": "package/dpkg-status-after-remove.txt",
        },
        "wslg": {
            "display": display,
            "wayland_display": wayland_display,
            "xdg_runtime_dir": xdg_runtime_dir,
            "launch_status": "passed" if process_observed and window_observed else "failed",
            "process_observed": bool(process_observed),
            "window_observed": bool(window_observed),
            "launch_stdout_file": "launch/wslg-launch-stdout.txt",
            "launch_stderr_file": "launch/wslg-launch-stderr.txt",
            "process_file": "launch/process.txt",
            "window_file": "launch/xwininfo.txt",
        },
        "commands": [asdict(command) for command in commands],
        "honest_scope": [
            "WSLg is WSL package install and GUI-launch evidence only",
            "not Linux desktop GUI proof outside WSLg",
            "not Linux seccomp or OS-sandbox enforcement",
            "not clean Linux install proof",
            "not signed, production, or v1.0 readiness",
        ],
    }
    summary = bundle / SUMMARY_NAME
    _write_text(summary, json.dumps(data, indent=2) + "\n")
    _write_text(bundle / MARKDOWN_NAME, render_markdown(data))
    _write_manifest(bundle)
    return 0 if verify_bundle(summary) else 1


def _command_files_exist(bundle: Path, data: dict) -> bool:
    commands = data.get("commands", [])
    if not isinstance(commands, list):
        return False
    by_id = {command.get("id"): command for command in commands if isinstance(command, dict)}
    for command_id in REQUIRED_COMMANDS:
        command = by_id.get(command_id)
        if not command or command.get("status") != "passed":
            return False
        for key in ("stdout_file", "stderr_file"):
            rel = command.get(key)
            if not isinstance(rel, str) or not (bundle / rel).is_file():
                return False
    return True


def verify_bundle(summary_path: Path) -> bool:
    if not summary_path.is_file():
        return False
    try:
        data = json.loads(summary_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return False
    bundle = summary_path.parent
    manifest = _manifest_entries(bundle)
    if not manifest:
        return False
    if data.get("schema") != SCHEMA or data.get("status") != "passed":
        return False
    if data.get("wsl_is_enforcement") is not False:
        return False
    if data.get("source_included") is not False or data.get("provider_api_called") is not False:
        return False
    if data.get("desktop_gui_launch_proven") is not False:
        return False
    if data.get("clean_linux_install_proven") is not False or data.get("linux_enforcement_proven") is not False:
        return False
    if data.get("privileged_system_install_proven") is not True or data.get("wslg_gui_launch_proven") is not True:
        return False
    scope = data.get("honest_scope", [])
    if not isinstance(scope, list) or not all(item in scope for item in REQUIRED_HONEST_SCOPE):
        return False
    joined_scope = "\n".join(str(item) for item in scope)
    if any(claim.lower() in joined_scope.lower() for claim in FORBIDDEN_CLAIMS):
        return False
    package = data.get("package", {})
    if not isinstance(package, dict) or package.get("format") != "deb":
        return False
    if package.get("package_name") != PACKAGE_NAME:
        return False
    install = data.get("install", {})
    if not isinstance(install, dict):
        return False
    if install.get("package_absent_before_record") is not True:
        return False
    if install.get("package_removed_after_record") is not True:
        return False
    if install.get("installed_binary") != INSTALLED_BINARY:
        return False
    wslg = data.get("wslg", {})
    if not isinstance(wslg, dict) or wslg.get("launch_status") != "passed":
        return False
    if wslg.get("process_observed") is not True or wslg.get("window_observed") is not True:
        return False
    for rel in [
        "package/dpkg-status-before.txt",
        "package/dpkg-status-after-install.txt",
        "package/dpkg-status-after-remove.txt",
        "installed/studio-smoke.json",
        str(wslg.get("process_file", "")),
        str(wslg.get("window_file", "")),
    ]:
        if not rel or not (bundle / rel).is_file():
            return False
    status_before = _read_text(bundle / "package" / "dpkg-status-before.txt")
    status_after_install = _read_text(bundle / "package" / "dpkg-status-after-install.txt")
    status_after_remove = _read_text(bundle / "package" / "dpkg-status-after-remove.txt")
    smoke = _read_text(bundle / "installed" / "studio-smoke.json")
    process = _read_text(bundle / str(wslg["process_file"]))
    window = _read_text(bundle / str(wslg["window_file"]))
    if "not installed" not in status_before and "no information is available" not in status_before:
        return False
    if "Status: install ok installed" not in status_after_install:
        return False
    if "not installed" not in status_after_remove and "no information is available" not in status_after_remove:
        return False
    smoke_lower = smoke.lower()
    if "passed" not in smoke_lower and ('"status"' not in smoke or "ok" not in smoke_lower):
        return False
    if "garnet-studio" not in process.lower():
        return False
    if not any(token in window.lower() for token in ["garnet studio", "garnet-studio", "webkit", "tauri"]):
        return False
    return _command_files_exist(bundle, data)


def read_committed_evidence(root: Path = ROOT) -> LinuxWslgInstallLaunchEvidence:
    if (root / SUMMARY_NAME).is_file():
        summaries = [root / SUMMARY_NAME]
    else:
        summaries = sorted(
            (root / "proofs" / "linux" / "execution" / "studio-wslg-system-install").glob(f"*/{SUMMARY_NAME}")
        )
        if not summaries:
            summaries = sorted(root.glob(f"**/{SUMMARY_NAME}"))
    for summary in reversed(summaries):
        if verify_bundle(summary):
            data = json.loads(summary.read_text(encoding="utf-8"))
            package = data.get("package", {})
            wslg = data.get("wslg", {})
            return LinuxWslgInstallLaunchEvidence(
                status="verified",
                verified=True,
                reason=(
                    "WSLg system package install and installed-binary GUI launch verified at "
                    f"`{summary}` ({package.get('package_name', PACKAGE_NAME)} {package.get('architecture', 'unknown')}, "
                    f"display {wslg.get('display', '')}, wayland {wslg.get('wayland_display', '')})."
                ),
                bundle=str(summary),
                deferred=[
                    "not Linux desktop GUI proof outside WSLg",
                    "not clean Linux install proof",
                    "not Linux seccomp or OS-sandbox enforcement",
                    "not signed, production, or v1.0 readiness",
                ],
            )
    return LinuxWslgInstallLaunchEvidence(
        status="failed",
        verified=False,
        reason="Committed WSLg system install/launch bundles exist, but none verify.",
        bundle=None,
        deferred=[
            "not Linux desktop GUI proof outside WSLg",
            "not clean Linux install proof",
            "not Linux seccomp or OS-sandbox enforcement",
        ],
    )


def render_markdown(data: dict) -> str:
    package = data.get("package", {})
    wslg = data.get("wslg", {})
    honest_scope = "\n".join(f"- {item}" for item in data.get("honest_scope", []))
    commands = "\n".join(
        f"- `{command['id']}`: {command['status']} (exit {command['exit_code']})"
        for command in data.get("commands", [])
    )
    return (
        "# Garnet Studio Linux WSLg System Install Proof\n\n"
        f"- Schema: `{data.get('schema')}`\n"
        f"- Status: `{data.get('status')}`\n"
        f"- Evidence tier: `{data.get('evidence_tier')}`\n"
        f"- Package: `{package.get('path')}`\n"
        f"- Package name: `{package.get('package_name')}`\n"
        f"- Installed binary: `{data.get('install', {}).get('installed_binary')}`\n"
        f"- WSLg display: `{wslg.get('display')}` / `{wslg.get('wayland_display')}`\n"
        f"- Process observed: `{wslg.get('process_observed')}`\n"
        f"- Window observed: `{wslg.get('window_observed')}`\n\n"
        "## Honest Scope\n\n"
        f"{honest_scope}\n\n"
        "## Commands\n\n"
        f"{commands}\n"
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--record", action="store_true", help="record a fresh WSLg system install proof")
    parser.add_argument("--gate", action="store_true", help="verify committed proof evidence")
    parser.add_argument("--format", choices=["text", "json"], default="text")
    parser.add_argument("--output-dir", type=Path, help="override proof output directory for --record")
    args = parser.parse_args(argv)

    if args.record:
        return record_proof(args.output_dir)

    evidence = read_committed_evidence()
    if args.format == "json":
        print(json.dumps(asdict(evidence), indent=2))
    else:
        print(f"{evidence.status}: {evidence.reason}")
    if args.gate and not evidence.verified:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
