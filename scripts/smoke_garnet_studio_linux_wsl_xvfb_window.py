#!/usr/bin/env python3
"""Record and verify a WSL Linux Garnet Studio Xvfb window-capture proof.

This is an S117 package-pipeline increment. It proves that this Windows box can
drive WSL to start the extracted Linux Tauri Studio binary under `xvfb-run`,
observe the X11 window tree, and capture a virtual-display screenshot artifact.
It is still WSL execution/portability evidence: not Linux desktop GUI launch
proof, not a clean Linux install proof, not privileged package install proof,
and not Linux seccomp or OS-sandbox enforcement.
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
SCHEMA = "garnet.studio.linux_wsl_xvfb_window_capture.v1"
SUMMARY_NAME = "garnet-studio-linux-wsl-xvfb-window-capture.json"
MARKDOWN_NAME = "garnet-studio-linux-wsl-xvfb-window-capture.md"
MANIFEST_NAME = "MANIFEST.sha256"
DEFAULT_PROOF_ROOT = ROOT / "proofs" / "linux" / "execution" / "studio-xvfb-window-capture"
MIN_SCREENSHOT_BYTES = 256
REQUIRED_COMMANDS = [
    "wsl-uname",
    "x11-capture-tooling",
    "xvfb-window-capture",
]
REQUIRED_HONEST_SCOPE = [
    "WSL Xvfb virtual-display window-capture evidence only",
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
TOOLING_KEYS = ["xvfb-run", "xwininfo", "xdpyinfo", "xwd", "convert", "identify"]


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
class LinuxWslXvfbWindowEvidence:
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


def _wsl_path(path: Path) -> str:
    completed = _run_host(["wsl.exe", "-e", "wslpath", "-a", str(path)])
    if completed.returncode != 0:
        raise RuntimeError(completed.stderr.strip() or f"wslpath failed for {path}")
    return completed.stdout.strip()


def _wsl_root() -> str:
    return _wsl_path(ROOT)


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
    _write_text(output_dir / stdout_rel, _normalize_command_log(completed.stdout))
    _write_text(output_dir / stderr_rel, _normalize_command_log(completed.stderr))
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


def _source_runtime_summary(root: Path = ROOT) -> Path | None:
    import smoke_garnet_studio_linux_wsl_xvfb

    candidates = sorted(
        (root / "proofs" / "linux" / "execution" / "studio-xvfb-runtime").glob(
            f"*/{smoke_garnet_studio_linux_wsl_xvfb.SUMMARY_NAME}"
        )
    )
    for summary in reversed(candidates):
        if smoke_garnet_studio_linux_wsl_xvfb.verify_bundle(summary):
            return summary
    return None


def _source_binary_from_runtime(root: Path = ROOT) -> tuple[dict, Path] | None:
    summary = _source_runtime_summary(root)
    if summary is None:
        return None
    data = _read_json(summary)
    if not isinstance(data, dict):
        return None
    binary = data.get("extracted_binary", {})
    if not isinstance(binary, dict):
        return None
    relative = binary.get("path")
    if not isinstance(relative, str) or not relative:
        return None
    absolute = root / Path(relative)
    if not absolute.is_file():
        return None
    return data, absolute


def _parse_tooling(stdout: str) -> dict[str, str]:
    values = {key: "" for key in TOOLING_KEYS}
    for line in stdout.splitlines():
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        if key in values:
            values[key] = value
    return values


def _x11_tooling_shell() -> str:
    tools = " ".join(TOOLING_KEYS)
    return f"""
for tool in {tools}; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "$tool missing"
    exit 12
  fi
  printf "%s=%s\\n" "$tool" "$(command -v "$tool")"
done
"""


def _capture_shell(binary_relative: str, capture_dir_wsl: str) -> str:
    binary = f"./{binary_relative}"
    return f"""
set -eu
capture_dir={_shell_quote(capture_dir_wsl)}
binary={_shell_quote(binary)}
mkdir -p "$capture_dir"
rm -f "$capture_dir"/*
xvfb-run -a -s "-screen 0 1280x1024x24" \
  env GDK_BACKEND=x11 \
      LIBGL_ALWAYS_SOFTWARE=1 \
      WEBKIT_DISABLE_DMABUF_RENDERER=1 \
      WEBKIT_DISABLE_COMPOSITING_MODE=1 \
      NO_AT_BRIDGE=1 \
      GARNET_CAPTURE_DIR="$capture_dir" \
      GARNET_STUDIO_BINARY="$binary" \
  sh -c '
  set -u
  "$GARNET_STUDIO_BINARY" > "$GARNET_CAPTURE_DIR/studio-stdout.txt" 2> "$GARNET_CAPTURE_DIR/studio-stderr.txt" &
  pid=$!
  echo "$pid" > "$GARNET_CAPTURE_DIR/studio.pid"
  sleep 1
  for attempt in 1 2 3 4 5; do
    xwininfo -root -tree > "$GARNET_CAPTURE_DIR/xwininfo.txt" 2> "$GARNET_CAPTURE_DIR/xwininfo.stderr" || true
    if grep -Eiq "Garnet|Studio|garnet-studio|WebKit|Tauri" "$GARNET_CAPTURE_DIR/xwininfo.txt"; then
      break
    fi
    sleep 1
  done
  xdpyinfo > "$GARNET_CAPTURE_DIR/xdpyinfo.txt" 2> "$GARNET_CAPTURE_DIR/xdpyinfo.stderr" || true
  xwd -root -silent -out "$GARNET_CAPTURE_DIR/screenshot.xwd" 2> "$GARNET_CAPTURE_DIR/xwd.stderr" || true
  convert "$GARNET_CAPTURE_DIR/screenshot.xwd" "$GARNET_CAPTURE_DIR/screenshot.png" > "$GARNET_CAPTURE_DIR/convert.stdout" 2> "$GARNET_CAPTURE_DIR/convert.stderr" || true
  identify "$GARNET_CAPTURE_DIR/screenshot.png" > "$GARNET_CAPTURE_DIR/identify.txt" 2> "$GARNET_CAPTURE_DIR/identify.stderr" || true
  kill "$pid" >/dev/null 2>&1 || true
  wait "$pid" >/dev/null 2>&1 || true
  test -s "$GARNET_CAPTURE_DIR/xwininfo.txt"
  test -s "$GARNET_CAPTURE_DIR/xdpyinfo.txt"
  test -s "$GARNET_CAPTURE_DIR/screenshot.png"
  test -s "$GARNET_CAPTURE_DIR/identify.txt"
'
"""


def _copy_capture_aliases(bundle: Path) -> None:
    capture = bundle / "capture"
    for name in ["xwininfo.txt", "xdpyinfo.txt", "identify.txt", "screenshot.png"]:
        target = capture / name
        if not target.exists():
            if target.suffix == ".png":
                target.write_bytes(b"")
            else:
                _write_text(target, "")


def _normalize_capture_texts(capture: Path) -> None:
    for target in [
        capture / "convert.stderr",
        capture / "convert.stdout",
        capture / "identify.stderr",
        capture / "identify.txt",
        capture / "studio-stderr.txt",
        capture / "studio-stdout.txt",
        capture / "studio.pid",
        capture / "xdpyinfo.stderr",
        capture / "xdpyinfo.txt",
        capture / "xwd.stderr",
        capture / "xwininfo.stderr",
        capture / "xwininfo.txt",
    ]:
        if target.is_file():
            _write_text(
                target,
                _normalize_command_log(target.read_text(encoding="utf-8", errors="replace")),
            )


def record_proof(output_dir: Path | None = None) -> int:
    stamp = _timestamp()
    bundle = output_dir or DEFAULT_PROOF_ROOT / f"linux-wsl-xvfb-window-capture-{stamp}"
    bundle.mkdir(parents=True, exist_ok=True)
    capture = bundle / "capture"
    capture.mkdir(parents=True, exist_ok=True)
    wsl_root = _wsl_root()
    source = _source_binary_from_runtime(ROOT)

    commands = [
        _run_wsl(
            command_id="wsl-uname",
            shell="uname -a",
            display_args=["wsl.exe", "-e", "uname", "-a"],
            output_dir=bundle,
            wsl_root=wsl_root,
        ),
        _run_wsl(
            command_id="x11-capture-tooling",
            shell=_x11_tooling_shell(),
            display_args=["sh", "-lc", "verify xvfb-run xwininfo xdpyinfo xwd convert identify"],
            output_dir=bundle,
            wsl_root=wsl_root,
        ),
    ]

    if source is not None:
        runtime_data, binary_path = source
        binary_relative = binary_path.relative_to(ROOT).as_posix()
        commands.append(
            _run_wsl(
                command_id="xvfb-window-capture",
                shell=_capture_shell(binary_relative, _wsl_path(capture)),
                display_args=["xvfb-run", "-a", f"./{binary_relative}", "capture-x11-window-tree-and-screenshot"],
                output_dir=bundle,
                wsl_root=wsl_root,
            )
        )
        source_package = runtime_data.get("source_package_proof", {})
        source_runtime_summary = _source_runtime_summary(ROOT)
    else:
        runtime_data = {}
        source_package = {}
        source_runtime_summary = None
        _write_text(capture / "xwininfo.txt", "")
        _write_text(capture / "xdpyinfo.txt", "")
        _write_text(capture / "identify.txt", "")
        (capture / "screenshot.png").write_bytes(b"")
        commands.append(
            CommandRecord(
                id="xvfb-window-capture",
                display_args=["xvfb-run", "-a", "<missing-binary>", "capture-x11-window-tree-and-screenshot"],
                exit_code=127,
                stdout_file="commands/xvfb-window-capture-stdout.txt",
                stderr_file="commands/xvfb-window-capture-stderr.txt",
                status="failed",
                runtime_seconds=0.0,
            )
        )
        _write_text(bundle / "commands" / "xvfb-window-capture-stdout.txt", "")
        _write_text(bundle / "commands" / "xvfb-window-capture-stderr.txt", "No verified Xvfb runtime-start source proof was found.\n")

    _copy_capture_aliases(bundle)
    _normalize_capture_texts(capture)
    tooling = _parse_tooling((bundle / "commands" / "x11-capture-tooling-stdout.txt").read_text(encoding="utf-8", errors="replace"))
    screenshot = capture / "screenshot.png"
    xwininfo = capture / "xwininfo.txt"
    xdpyinfo = capture / "xdpyinfo.txt"
    identify = capture / "identify.txt"
    window_capture_passed = (
        commands[-1].status == "passed"
        and xwininfo.stat().st_size > 0
        and xdpyinfo.stat().st_size > 0
        and identify.stat().st_size > 0
        and screenshot.stat().st_size >= MIN_SCREENSHOT_BYTES
    )
    binary = runtime_data.get("extracted_binary", {}) if isinstance(runtime_data, dict) else {}
    data = {
        "schema": SCHEMA,
        "generated_at": datetime.now(UTC).isoformat(timespec="seconds"),
        "status": "passed" if source is not None and window_capture_passed and all(c.status == "passed" for c in commands) else "failed",
        "platform": "linux",
        "evidence_tier": "wsl-linux-xvfb-virtual-display-window-capture",
        "wsl_is_enforcement": False,
        "source_included": False,
        "provider_api_called": False,
        "linux_enforcement_proven": False,
        "desktop_gui_launch_proven": False,
        "linux_desktop_gui_launch_proven": False,
        "clean_linux_install_proven": False,
        "privileged_system_install_proven": False,
        "virtual_display_window_capture_proven": bool(window_capture_passed),
        "source_runtime_proof": {
            "summary": source_runtime_summary.as_posix() if source_runtime_summary else None,
        },
        "source_package_proof": {
            "format": source_package.get("format") if isinstance(source_package, dict) else None,
            "summary": source_package.get("summary") if isinstance(source_package, dict) else None,
        },
        "extracted_binary": {
            "path": binary.get("path") if isinstance(binary, dict) else None,
            "sha256": binary.get("sha256") if isinstance(binary, dict) else None,
        },
        "x11_tooling": tooling,
        "window_capture": {
            "status": "passed" if window_capture_passed else "failed",
            "window_tree_file": "capture/xwininfo.txt",
            "display_info_file": "capture/xdpyinfo.txt",
            "screenshot_file": "capture/screenshot.png",
            "screenshot_sha256": _sha256(screenshot) if screenshot.is_file() else None,
            "screenshot_bytes": screenshot.stat().st_size if screenshot.is_file() else 0,
            "identify_file": "capture/identify.txt",
        },
        "commands": [asdict(command) for command in commands],
        "honest_scope": [
            "WSL Xvfb virtual-display window-capture evidence only",
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
        print("linux-wsl-xvfb-window proof failed post-write verification", file=sys.stderr)
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
    if data.get("platform") != "linux" or data.get("evidence_tier") != "wsl-linux-xvfb-virtual-display-window-capture":
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
    if data.get("virtual_display_window_capture_proven") is not True:
        return False
    scope = " ".join(str(item) for item in data.get("honest_scope", []))
    if not all(anchor in scope for anchor in REQUIRED_HONEST_SCOPE):
        return False
    all_text = json.dumps(data, sort_keys=True)
    if any(claim in all_text for claim in FORBIDDEN_CLAIMS):
        return False

    source_runtime = data.get("source_runtime_proof", {})
    if not isinstance(source_runtime, dict) or not isinstance(source_runtime.get("summary"), str):
        return False
    source_package = data.get("source_package_proof", {})
    if not isinstance(source_package, dict) or source_package.get("format") not in {"deb", "rpm"}:
        return False
    binary = data.get("extracted_binary", {})
    if not isinstance(binary, dict):
        return False
    if not isinstance(binary.get("path"), str) or len(str(binary.get("sha256", ""))) != 64:
        return False

    tooling = data.get("x11_tooling", {})
    if not isinstance(tooling, dict):
        return False
    for tool in TOOLING_KEYS:
        if not isinstance(tooling.get(tool), str) or not tooling.get(tool):
            return False

    capture = data.get("window_capture", {})
    if not isinstance(capture, dict) or capture.get("status") != "passed":
        return False
    required_capture_files = [
        capture.get("window_tree_file"),
        capture.get("display_info_file"),
        capture.get("screenshot_file"),
        capture.get("identify_file"),
    ]
    for rel in required_capture_files:
        if not isinstance(rel, str) or rel not in manifest:
            return False
    screenshot_rel = capture.get("screenshot_file")
    screenshot = bundle / str(screenshot_rel)
    if not screenshot.is_file() or screenshot.stat().st_size < MIN_SCREENSHOT_BYTES:
        return False
    if capture.get("screenshot_sha256") != _sha256(screenshot):
        return False
    if int(capture.get("screenshot_bytes", 0)) != screenshot.stat().st_size:
        return False
    window_tree = (bundle / str(capture.get("window_tree_file"))).read_text(encoding="utf-8", errors="replace")
    if not any(marker in window_tree.lower() for marker in ["garnet", "studio", "webkit", "tauri"]):
        return False
    identify = (bundle / str(capture.get("identify_file"))).read_text(encoding="utf-8", errors="replace")
    if "png" not in identify.lower():
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
        if int(command.get("exit_code", -1)) != 0:
            return False
        stdout_file = command.get("stdout_file")
        stderr_file = command.get("stderr_file")
        if not isinstance(stdout_file, str) or not isinstance(stderr_file, str):
            return False
        if stdout_file not in manifest or stderr_file not in manifest:
            return False
    return True


def read_committed_evidence(root: Path = ROOT) -> LinuxWslXvfbWindowEvidence:
    if (root / SUMMARY_NAME).is_file():
        candidates = [root / SUMMARY_NAME]
    else:
        candidates = sorted(
            (root / "proofs" / "linux" / "execution" / "studio-xvfb-window-capture").glob(f"*/{SUMMARY_NAME}")
        )
        if not candidates:
            candidates = sorted(root.glob(f"**/{SUMMARY_NAME}"))
    for summary in reversed(candidates):
        if verify_bundle(summary):
            data = json.loads(summary.read_text(encoding="utf-8"))
            capture = data.get("window_capture", {})
            return LinuxWslXvfbWindowEvidence(
                status="verified",
                verified=True,
                reason=(
                    "WSL Xvfb virtual-display window capture verified at "
                    f"`{summary.as_posix()}` (screenshot {capture.get('screenshot_file')}, "
                    f"{capture.get('screenshot_bytes')} bytes)."
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
    return LinuxWslXvfbWindowEvidence(
        status="missing",
        verified=False,
        reason="No committed WSL Linux Xvfb virtual-display window-capture proof bundle verified.",
        bundle=None,
        deferred=[
            "record with `scripts/smoke_garnet_studio_linux_wsl_xvfb_window.py --record`",
            "Linux desktop GUI launch and clean Linux install remain separate gates",
        ],
    )


def render_markdown(data: dict) -> str:
    source = data.get("source_package_proof", {})
    binary = data.get("extracted_binary", {})
    capture = data.get("window_capture", {})
    tooling = data.get("x11_tooling", {})
    lines = [
        "# Garnet Studio Linux WSL Xvfb Window Capture Proof",
        "",
        f"- schema: `{data.get('schema')}`",
        f"- status: `{data.get('status')}`",
        f"- evidence tier: `{data.get('evidence_tier')}`",
        f"- source package proof: `{source.get('format')}` `{source.get('summary')}`",
        f"- extracted binary: `{binary.get('path')}`",
        f"- screenshot: `{capture.get('screenshot_file')}`",
        f"- screenshot sha256: `{capture.get('screenshot_sha256')}`",
        f"- screenshot bytes: `{capture.get('screenshot_bytes')}`",
        f"- window tree: `{capture.get('window_tree_file')}`",
        f"- display info: `{capture.get('display_info_file')}`",
        "",
        "## X11 Tooling",
        "",
        "| Tool | Path |",
        "| --- | --- |",
    ]
    for key in TOOLING_KEYS:
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
    parser.add_argument("--record", action="store_true", help="record WSL Linux Xvfb virtual-display window proof")
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
                    "# Garnet Studio Linux WSL Xvfb Window Capture Proof Status",
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
