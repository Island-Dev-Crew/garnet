#!/usr/bin/env python3
"""Record and verify a WSL Linux Garnet Studio Xvfb runtime-start proof.

This is an S117 package-pipeline increment. It proves that this Windows box can
drive WSL to start the extracted Linux Tauri Studio binary under `xvfb-run` and
observe that it remains alive until the harness timeout. It is not Linux desktop
GUI launch proof, not a clean Linux install proof, not privileged package
install proof, and not Linux seccomp or OS-sandbox enforcement.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import time
from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = "garnet.studio.linux_wsl_xvfb_runtime.v1"
SUMMARY_NAME = "garnet-studio-linux-wsl-xvfb-runtime.json"
MARKDOWN_NAME = "garnet-studio-linux-wsl-xvfb-runtime.md"
MANIFEST_NAME = "MANIFEST.sha256"
DEFAULT_PROOF_ROOT = ROOT / "proofs" / "linux" / "execution" / "studio-xvfb-runtime"
EXPECTED_TIMEOUT_EXIT_CODE = 124
DEFAULT_TIMEOUT_SECONDS = 8
REQUIRED_COMMANDS = [
    "wsl-uname",
    "xvfb-tooling",
    "xvfb-runtime-start",
]
REQUIRED_HONEST_SCOPE = [
    "WSL Xvfb runtime-start evidence only",
    "not Linux desktop GUI launch proof",
    "not Linux seccomp or OS-sandbox enforcement",
    "not clean Linux install proof",
    "not privileged system package install proof",
]
FORBIDDEN_CLAIMS = [
    "Linux desktop GUI launch proof verified",
    "Linux seccomp enforced",
    "OS-sandbox enforcement verified",
    "clean Linux install proof verified",
    "privileged system package install proof verified",
    "production readiness verified",
    "v1.0 readiness verified",
]
TOOLING_KEYS = ["xvfb-run", "timeout"]
DISPLAY_KEYS = ["DISPLAY", "WAYLAND_DISPLAY", "XDG_RUNTIME_DIR"]


@dataclass(frozen=True)
class CommandRecord:
    id: str
    display_args: list[str]
    exit_code: int
    stdout_file: str
    stderr_file: str
    status: str
    runtime_seconds: float


@dataclass(frozen=True)
class SourceBinary:
    package_format: str
    bundle: str
    summary: str
    relative_path: str
    absolute_path: Path


@dataclass(frozen=True)
class LinuxWslXvfbEvidence:
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


def _normalize_command_log(text: str) -> str:
    lines = [line.rstrip() for line in text.splitlines()]
    while lines and lines[-1] == "":
        lines.pop()
    return "\n".join(lines) + "\n" if lines else ""


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


def _shell_quote(value: str) -> str:
    return "'" + value.replace("'", "'\"'\"'") + "'"


def _run_wsl(
    *,
    command_id: str,
    shell: str,
    display_args: list[str],
    output_dir: Path,
    wsl_root: str,
    expected_exit_code: int = 0,
) -> CommandRecord:
    started = time.monotonic()
    completed = _run_host(["wsl.exe", "-e", "sh", "-lc", f"cd {_shell_quote(wsl_root)} && {shell}"])
    elapsed = round(time.monotonic() - started, 3)
    stdout_rel = Path("commands") / f"{command_id}-stdout.txt"
    stderr_rel = Path("commands") / f"{command_id}-stderr.txt"
    stdout_text = _normalize_command_log(completed.stdout)
    stderr_text = _normalize_command_log(completed.stderr)
    _write_text(output_dir / stdout_rel, stdout_text)
    _write_text(output_dir / stderr_rel, stderr_text)
    if command_id == "xvfb-runtime-start":
        _write_text(output_dir / "runtime" / "xvfb-runtime-start-stdout.txt", stdout_text)
        _write_text(output_dir / "runtime" / "xvfb-runtime-start-stderr.txt", stderr_text)
    return CommandRecord(
        id=command_id,
        display_args=display_args,
        exit_code=completed.returncode,
        stdout_file=stdout_rel.as_posix(),
        stderr_file=stderr_rel.as_posix(),
        status="passed" if completed.returncode == expected_exit_code else "failed",
        runtime_seconds=elapsed,
    )


def _read_json(path: Path) -> dict | None:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None


def _verified_source_summaries(root: Path) -> list[tuple[str, Path]]:
    import smoke_garnet_studio_linux_wsl_deb_install
    import smoke_garnet_studio_linux_wsl_rpm

    candidates: list[tuple[str, Path]] = []
    for summary in sorted(
        (root / "proofs" / "linux" / "execution" / "studio-rpm-package").glob(
            f"*/{smoke_garnet_studio_linux_wsl_rpm.SUMMARY_NAME}"
        )
    ):
        if smoke_garnet_studio_linux_wsl_rpm.verify_bundle(summary):
            candidates.append(("rpm", summary))
    for summary in sorted(
        (root / "proofs" / "linux" / "execution" / "studio-package-install").glob(
            f"*/{smoke_garnet_studio_linux_wsl_deb_install.SUMMARY_NAME}"
        )
    ):
        if smoke_garnet_studio_linux_wsl_deb_install.verify_bundle(summary):
            candidates.append(("deb", summary))
    return candidates


def _find_source_binary(root: Path = ROOT) -> SourceBinary | None:
    for package_format, summary in reversed(_verified_source_summaries(root)):
        data = _read_json(summary)
        if not isinstance(data, dict):
            continue
        extracted = data.get("extracted_binary", {})
        relative = extracted.get("path") if isinstance(extracted, dict) else None
        if not isinstance(relative, str) or not relative:
            continue
        absolute = root / Path(relative)
        if absolute.is_file():
            return SourceBinary(
                package_format=package_format,
                bundle=summary.parent.as_posix(),
                summary=summary.as_posix(),
                relative_path=relative,
                absolute_path=absolute,
            )
    return None


def _parse_tooling(stdout: str) -> dict[str, str]:
    values = {key: "" for key in [*TOOLING_KEYS, *DISPLAY_KEYS]}
    for line in stdout.splitlines():
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        if key in values:
            values[key] = value
    return values


def _xvfb_tooling_shell() -> str:
    return """
for tool in xvfb-run timeout; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "$tool missing"
    exit 12
  fi
  printf "%s=%s\\n" "$tool" "$(command -v "$tool")"
done
printf "DISPLAY=%s\\n" "${DISPLAY:-}"
printf "WAYLAND_DISPLAY=%s\\n" "${WAYLAND_DISPLAY:-}"
printf "XDG_RUNTIME_DIR=%s\\n" "${XDG_RUNTIME_DIR:-}"
"""


def record_proof(output_dir: Path | None = None, timeout_seconds: int = DEFAULT_TIMEOUT_SECONDS) -> int:
    stamp = _timestamp()
    bundle = output_dir or DEFAULT_PROOF_ROOT / f"linux-wsl-xvfb-runtime-{stamp}"
    bundle.mkdir(parents=True, exist_ok=True)
    wsl_root = _wsl_root()
    source_binary = _find_source_binary(ROOT)

    commands = [
        _run_wsl(
            command_id="wsl-uname",
            shell="uname -a",
            display_args=["wsl.exe", "-e", "uname", "-a"],
            output_dir=bundle,
            wsl_root=wsl_root,
        ),
        _run_wsl(
            command_id="xvfb-tooling",
            shell=_xvfb_tooling_shell(),
            display_args=["sh", "-lc", "verify xvfb-run timeout display vars"],
            output_dir=bundle,
            wsl_root=wsl_root,
        ),
    ]
    if source_binary is not None:
        binary_arg = _shell_quote(f"./{source_binary.relative_path}")
        commands.append(
            _run_wsl(
                command_id="xvfb-runtime-start",
                shell=f"timeout {int(timeout_seconds)}s xvfb-run -a {binary_arg}",
                display_args=[
                    "timeout",
                    f"{int(timeout_seconds)}s",
                    "xvfb-run",
                    "-a",
                    f"./{source_binary.relative_path}",
                ],
                output_dir=bundle,
                wsl_root=wsl_root,
                expected_exit_code=EXPECTED_TIMEOUT_EXIT_CODE,
            )
        )
    else:
        stdout_rel = Path("commands") / "xvfb-runtime-start-stdout.txt"
        stderr_rel = Path("commands") / "xvfb-runtime-start-stderr.txt"
        _write_text(bundle / stdout_rel, "")
        _write_text(bundle / stderr_rel, "No verified extracted Linux Studio binary was found.\n")
        _write_text(bundle / "runtime" / "xvfb-runtime-start-stdout.txt", "")
        _write_text(bundle / "runtime" / "xvfb-runtime-start-stderr.txt", "No verified extracted Linux Studio binary was found.\n")
        commands.append(
            CommandRecord(
                id="xvfb-runtime-start",
                display_args=["timeout", f"{int(timeout_seconds)}s", "xvfb-run", "-a", "<missing-binary>"],
                exit_code=127,
                stdout_file=stdout_rel.as_posix(),
                stderr_file=stderr_rel.as_posix(),
                status="failed",
                runtime_seconds=0.0,
            )
        )

    runtime_command = next(command for command in commands if command.id == "xvfb-runtime-start")
    tooling = _parse_tooling((bundle / "commands" / "xvfb-tooling-stdout.txt").read_text(encoding="utf-8", errors="replace"))
    runtime_passed = runtime_command.exit_code == EXPECTED_TIMEOUT_EXIT_CODE and runtime_command.status == "passed"
    data = {
        "schema": SCHEMA,
        "generated_at": datetime.now(UTC).isoformat(timespec="seconds"),
        "status": "passed" if source_binary is not None and runtime_passed and all(c.status == "passed" for c in commands) else "failed",
        "platform": "linux",
        "evidence_tier": "wsl-linux-xvfb-runtime-start-smoke",
        "wsl_is_enforcement": False,
        "source_included": False,
        "provider_api_called": False,
        "linux_enforcement_proven": False,
        "desktop_gui_launch_proven": False,
        "linux_desktop_gui_launch_proven": False,
        "clean_linux_install_proven": False,
        "privileged_system_install_proven": False,
        "xvfb_runtime_start_proven": bool(runtime_passed),
        "expected_timeout_exit_code": EXPECTED_TIMEOUT_EXIT_CODE,
        "timeout_seconds": int(timeout_seconds),
        "runtime_seconds": runtime_command.runtime_seconds,
        "source_package_proof": {
            "format": source_binary.package_format if source_binary else None,
            "bundle": source_binary.bundle if source_binary else None,
            "summary": source_binary.summary if source_binary else None,
        },
        "extracted_binary": {
            "path": source_binary.relative_path if source_binary else None,
            "sha256": _sha256(source_binary.absolute_path) if source_binary else None,
        },
        "xvfb_tooling": tooling,
        "runtime_start": {
            "exit_code": runtime_command.exit_code,
            "expected_exit_code": EXPECTED_TIMEOUT_EXIT_CODE,
            "status": runtime_command.status,
            "stdout_file": "runtime/xvfb-runtime-start-stdout.txt",
            "stderr_file": "runtime/xvfb-runtime-start-stderr.txt",
        },
        "commands": [asdict(command) for command in commands],
        "honest_scope": [
            "WSL Xvfb runtime-start evidence only",
            "not Linux desktop GUI launch proof",
            "not Linux seccomp or OS-sandbox enforcement",
            "not clean Linux install proof",
            "not privileged system package install proof",
            "not signed, production, or v1.0 readiness",
        ],
    }
    _write_text(bundle / SUMMARY_NAME, json.dumps(data, indent=2) + "\n")
    _write_text(bundle / MARKDOWN_NAME, render_markdown(data))
    _write_manifest(bundle)
    print(render_markdown(data), end="")
    if data["status"] == "passed" and not verify_bundle(bundle / SUMMARY_NAME):
        print("linux-wsl-xvfb-runtime proof failed post-write verification", file=sys.stderr)
        return 1
    return 0 if data["status"] == "passed" else 1


def verify_bundle(summary_path: Path) -> bool:
    data = _read_json(summary_path)
    if not isinstance(data, dict):
        return False
    bundle = summary_path.parent
    manifest = _manifest_entries(bundle)
    if manifest is None:
        return False
    if SUMMARY_NAME not in manifest or MARKDOWN_NAME not in manifest:
        return False
    if data.get("schema") != SCHEMA or data.get("status") != "passed":
        return False
    if data.get("platform") != "linux" or data.get("evidence_tier") != "wsl-linux-xvfb-runtime-start-smoke":
        return False
    false_keys = [
        "wsl_is_enforcement",
        "source_included",
        "provider_api_called",
        "linux_enforcement_proven",
        "desktop_gui_launch_proven",
        "linux_desktop_gui_launch_proven",
        "clean_linux_install_proven",
        "privileged_system_install_proven",
    ]
    for key in false_keys:
        if data.get(key) is not False:
            return False
    if data.get("xvfb_runtime_start_proven") is not True:
        return False
    if int(data.get("expected_timeout_exit_code", -1)) != EXPECTED_TIMEOUT_EXIT_CODE:
        return False
    if int(data.get("timeout_seconds", 0)) <= 0:
        return False
    if float(data.get("runtime_seconds", 0.0)) <= 0.0:
        return False
    scope = " ".join(str(item) for item in data.get("honest_scope", []))
    if not all(anchor in scope for anchor in REQUIRED_HONEST_SCOPE):
        return False
    all_text = json.dumps(data, sort_keys=True)
    if any(claim in all_text for claim in FORBIDDEN_CLAIMS):
        return False

    source_proof = data.get("source_package_proof", {})
    if not isinstance(source_proof, dict) or source_proof.get("format") not in {"deb", "rpm"}:
        return False
    if not isinstance(source_proof.get("summary"), str) or not source_proof.get("summary"):
        return False

    binary = data.get("extracted_binary", {})
    if not isinstance(binary, dict):
        return False
    if not isinstance(binary.get("path"), str) or not binary.get("path"):
        return False
    if len(str(binary.get("sha256", ""))) != 64:
        return False

    tooling = data.get("xvfb_tooling", {})
    if not isinstance(tooling, dict):
        return False
    for tool in TOOLING_KEYS:
        if not isinstance(tooling.get(tool), str) or not tooling.get(tool):
            return False
    for key in DISPLAY_KEYS:
        if not isinstance(tooling.get(key, ""), str):
            return False

    runtime = data.get("runtime_start", {})
    if (
        not isinstance(runtime, dict)
        or runtime.get("status") != "passed"
        or int(runtime.get("exit_code", -1)) != EXPECTED_TIMEOUT_EXIT_CODE
        or int(runtime.get("expected_exit_code", -1)) != EXPECTED_TIMEOUT_EXIT_CODE
    ):
        return False
    for path_key in ("stdout_file", "stderr_file"):
        rel = runtime.get(path_key)
        if not isinstance(rel, str) or rel not in manifest:
            return False

    commands = data.get("commands")
    if not isinstance(commands, list):
        return False
    command_ids = [command.get("id") for command in commands if isinstance(command, dict)]
    if command_ids != REQUIRED_COMMANDS:
        return False
    for command in commands:
        if not isinstance(command, dict) or command.get("status") != "passed":
            return False
        expected = EXPECTED_TIMEOUT_EXIT_CODE if command.get("id") == "xvfb-runtime-start" else 0
        if int(command.get("exit_code", -1)) != expected:
            return False
        stdout_file = command.get("stdout_file")
        stderr_file = command.get("stderr_file")
        if not isinstance(stdout_file, str) or not isinstance(stderr_file, str):
            return False
        if stdout_file not in manifest or stderr_file not in manifest:
            return False
    return True


def read_committed_evidence(root: Path = ROOT) -> LinuxWslXvfbEvidence:
    if (root / SUMMARY_NAME).is_file():
        candidates = [root / SUMMARY_NAME]
    else:
        candidates = sorted(
            (root / "proofs" / "linux" / "execution" / "studio-xvfb-runtime").glob(f"*/{SUMMARY_NAME}")
        )
        if not candidates:
            candidates = sorted(root.glob(f"**/{SUMMARY_NAME}"))
    for summary in reversed(candidates):
        if verify_bundle(summary):
            data = json.loads(summary.read_text(encoding="utf-8"))
            source = data.get("source_package_proof", {})
            binary = data.get("extracted_binary", {})
            return LinuxWslXvfbEvidence(
                status="verified",
                verified=True,
                reason=(
                    "WSL Xvfb runtime-start verified at "
                    f"`{summary.as_posix()}` (source {source.get('format')} proof "
                    f"{source.get('summary')}, extracted binary {binary.get('path')}, "
                    f"timeout exit {data.get('expected_timeout_exit_code')})."
                ),
                bundle=summary.parent.as_posix(),
                deferred=[
                    "not Linux desktop GUI launch proof",
                    "not Linux seccomp or OS-sandbox enforcement",
                    "not clean Linux install proof",
                    "not privileged system package install proof",
                    "not signed, production, or v1.0 readiness",
                ],
            )
    return LinuxWslXvfbEvidence(
        status="missing",
        verified=False,
        reason="No committed WSL Linux Xvfb runtime-start proof bundle verified.",
        bundle=None,
        deferred=[
            "record with `scripts/smoke_garnet_studio_linux_wsl_xvfb.py --record`",
            "Linux desktop GUI launch and clean Linux install remain separate gates",
        ],
    )


def render_markdown(data: dict) -> str:
    source = data.get("source_package_proof", {})
    binary = data.get("extracted_binary", {})
    tooling = data.get("xvfb_tooling", {})
    runtime = data.get("runtime_start", {})
    lines = [
        "# Garnet Studio Linux WSL Xvfb Runtime Proof",
        "",
        f"- schema: `{data.get('schema')}`",
        f"- status: `{data.get('status')}`",
        f"- evidence tier: `{data.get('evidence_tier')}`",
        f"- source package proof: `{source.get('format')}` `{source.get('summary')}`",
        f"- extracted binary: `{binary.get('path')}`",
        f"- extracted binary sha256: `{binary.get('sha256')}`",
        f"- timeout_seconds: `{data.get('timeout_seconds')}`",
        f"- runtime_seconds: `{data.get('runtime_seconds')}`",
        f"- runtime exit code: `{runtime.get('exit_code')}`",
        f"- expected timeout exit code: `{runtime.get('expected_exit_code')}`",
        "",
        "## Xvfb Tooling",
        "",
        "| Tool / variable | Value |",
        "| --- | --- |",
    ]
    for key in [*TOOLING_KEYS, *DISPLAY_KEYS]:
        lines.append(f"| `{key}` | `{tooling.get(key, '')}` |")
    lines.extend(
        [
            "",
            "## Commands",
            "",
            "| Command | Exit | Status |",
            "| --- | ---: | --- |",
        ]
    )
    for command in data.get("commands", []):
        lines.append(f"| `{command.get('id')}` | `{command.get('exit_code')}` | `{command.get('status')}` |")
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
    parser.add_argument("--record", action="store_true", help="record WSL Linux Xvfb runtime-start proof")
    parser.add_argument("--gate", action="store_true", help="verify committed proof bundle")
    parser.add_argument("--output-dir", type=Path)
    parser.add_argument("--timeout-seconds", type=int, default=DEFAULT_TIMEOUT_SECONDS)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    if args.record:
        return record_proof(args.output_dir, timeout_seconds=args.timeout_seconds)
    evidence = read_committed_evidence()
    if args.format == "md":
        print(
            "\n".join(
                [
                    "# Garnet Studio Linux WSL Xvfb Runtime Proof Status",
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
