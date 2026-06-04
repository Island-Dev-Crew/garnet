#!/usr/bin/env python3
"""Record and verify the full S109 cross-OS trap parity matrix.

This is the post-S108 consolidation gate. It does not rerun or redefine the
Windows/Mac/Linux row proofs; it verifies the committed row evidence and records
the remaining Linux `diff-caps` rejection datapoint from the Mac #2 UTM Debian
guest. WSL remains execution/portability evidence only and is never counted as
Linux enforcement.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import shlex
import subprocess
import sys
from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import garnet_linux_cross_os_enforcement_proof  # noqa: E402
import garnet_windows_cross_os_enforcement_proof  # noqa: E402
import smoke_garnet_mac_cross_os_matrix  # noqa: E402

SCHEMA = "garnet.cross_os_trap_parity_matrix.v1"
SUMMARY_NAME = "garnet-cross-os-trap-parity-matrix.json"
MARKDOWN_NAME = "garnet-cross-os-trap-parity-matrix.md"
MANIFEST_NAME = "MANIFEST.sha256"
DEFAULT_ROOT = ROOT / "proofs" / "cross-os" / "matrix"
REQUIRED_TRAPS = {"max_depth", "caps", "diff_caps_reject"}


@dataclass(frozen=True)
class CommandRecord:
    id: str
    display_args: list[str]
    expected_exit_code: int
    exit_code: int
    stdout_file: str
    stderr_file: str
    status: str


def timestamp_slug(now: datetime | None = None) -> str:
    return (now or datetime.now(UTC)).strftime("%Y%m%d-%H%M%S")


def default_output_dir(now: datetime | None = None) -> Path:
    return DEFAULT_ROOT / f"cross-os-trap-parity-{timestamp_slug(now)}"


def _sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _sha256(path: Path) -> str:
    return _sha256_bytes(path.read_bytes())


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8", newline="\n")


def _bundle_relative(bundle_dir: Path, path: Path) -> str:
    return path.resolve().relative_to(bundle_dir.resolve()).as_posix()


def _repo_relative(path: Path) -> str:
    try:
        return path.resolve().relative_to(ROOT.resolve()).as_posix()
    except ValueError:
        return path.as_posix()


def _write_manifest(bundle_dir: Path) -> None:
    lines: list[str] = []
    for path in sorted(p for p in bundle_dir.rglob("*") if p.is_file()):
        if path.name == MANIFEST_NAME:
            continue
        lines.append(f"{_sha256(path)}  {_bundle_relative(bundle_dir, path)}")
    _write_text(bundle_dir / MANIFEST_NAME, "\n".join(lines) + "\n")


def _manifest_entries(bundle_dir: Path) -> dict[str, str] | None:
    manifest = bundle_dir / MANIFEST_NAME
    if not manifest.is_file():
        return None
    entries: dict[str, str] = {}
    for line in manifest.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        digest, sep, relative = line.partition("  ")
        if sep != "  " or len(digest) != 64:
            return None
        target = bundle_dir / relative
        if not target.is_file() or _sha256(target) != digest:
            return None
        entries[relative.replace("\\", "/")] = digest
    return entries


def _latest_verified_mac_matrix() -> Path | None:
    root = ROOT / "proofs" / "mac" / "matrix"
    if not root.exists():
        return None
    candidates = sorted(
        root.rglob(smoke_garnet_mac_cross_os_matrix.SUMMARY_NAME),
        key=lambda path: path.stat().st_mtime,
        reverse=True,
    )
    for candidate in candidates:
        if smoke_garnet_mac_cross_os_matrix.verify_bundle(candidate):
            return candidate
    return None


def _run_linux_diff_caps(command: str, bundle_dir: Path) -> CommandRecord:
    stdout_rel = Path("commands") / "linux-diff-caps-reject-stdout.txt"
    stderr_rel = Path("commands") / "linux-diff-caps-reject-stderr.txt"
    completed = subprocess.run(
        command,
        cwd=ROOT,
        shell=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        capture_output=True,
        check=False,
    )
    _write_text(bundle_dir / stdout_rel, completed.stdout)
    _write_text(bundle_dir / stderr_rel, completed.stderr)
    ok = completed.returncode == 1 and "AUTHORITY EXPANDED" in completed.stdout
    return CommandRecord(
        id="linux-diff-caps-reject",
        display_args=["sh", "-lc", command],
        expected_exit_code=1,
        exit_code=completed.returncode,
        stdout_file=stdout_rel.as_posix(),
        stderr_file=stderr_rel.as_posix(),
        status="passed" if ok else "failed",
    )


def _commands_by_name(commands: list[dict]) -> dict[str, bool]:
    return {str(item.get("name")): int(item.get("exit_code", 1)) == 0 for item in commands}


def _matrix_row_os_status(mac_data: dict[str, object], trap: str, os_name: str) -> bool:
    rows = mac_data.get("trap_rows")
    if not isinstance(rows, list):
        return False
    for row in rows:
        if isinstance(row, dict) and row.get("trap") == trap:
            os_row = row.get(os_name)
            return (
                row.get("status") == "passed"
                and isinstance(os_row, dict)
                and os_row.get("status") is True
            )
    return False


def _linux_stage_v_status(linux_data: dict[str, object], trap: str) -> bool:
    traps = linux_data.get("stage_v_traps")
    if not isinstance(traps, list):
        return False
    for item in traps:
        if isinstance(item, dict) and item.get("trap") == trap:
            return item.get("status") == "passed"
    return False


def _trap_rows(
    *,
    windows_status: garnet_windows_cross_os_enforcement_proof.S106Status,
    mac_data: dict[str, object],
    linux_data: dict[str, object],
    linux_diff_caps: CommandRecord,
) -> list[dict[str, object]]:
    windows_commands = _commands_by_name(windows_status.windows.commands)
    wsl_ok = windows_status.wsl.ok
    rows = [
        {
            "trap": "max_depth",
            "windows": {
                "status": windows_status.windows.ok and windows_commands.get("bounded_enforcement", False),
                "tier": "enforcement",
                "evidence": "proofs/windows/enforcement/bounded_enforcement.stdout.log",
            },
            "mac": {
                "status": _matrix_row_os_status(mac_data, "max_depth", "mac"),
                "tier": "enforcement",
                "evidence": mac_data.get("mac_domain_baseline"),
            },
            "linux": {
                "status": _linux_stage_v_status(linux_data, "max_depth"),
                "tier": "enforcement",
                "evidence": _repo_relative(
                    ROOT
                    / "proofs"
                    / "linux"
                    / "enforcement"
                    / "utm-linux-enforcement-20260604-s108"
                    / "garnet-linux-cross-os-enforcement-proof.json"
                ),
            },
            "wsl": {
                "status": wsl_ok,
                "tier": windows_status.wsl.tier,
                "excluded_from_linux_enforcement": True,
            },
        },
        {
            "trap": "caps",
            "windows": {
                "status": windows_status.windows.ok and windows_commands.get("caps_enforcement", False),
                "tier": "enforcement",
                "evidence": "proofs/windows/enforcement/caps_enforcement.stdout.log",
            },
            "mac": {
                "status": _matrix_row_os_status(mac_data, "caps", "mac"),
                "tier": "enforcement",
                "evidence": mac_data.get("mac_domain_baseline"),
            },
            "linux": {
                "status": _linux_stage_v_status(linux_data, "caps")
                and linux_data.get("seccomp", {}).get("denied_socket_trapped") is True,
                "tier": "enforcement+linux-seccomp",
                "evidence": _repo_relative(
                    ROOT
                    / "proofs"
                    / "linux"
                    / "enforcement"
                    / "utm-linux-enforcement-20260604-s108"
                    / "garnet-linux-cross-os-enforcement-proof.json"
                ),
            },
            "wsl": {
                "status": wsl_ok,
                "tier": windows_status.wsl.tier,
                "excluded_from_linux_enforcement": True,
            },
        },
        {
            "trap": "diff_caps_reject",
            "windows": {
                "status": _matrix_row_os_status(mac_data, "diff_caps_reject", "windows"),
                "tier": "enforcement",
                "evidence": "proofs/windows/ultrapunch",
            },
            "mac": {
                "status": _matrix_row_os_status(mac_data, "diff_caps_reject", "mac"),
                "tier": "enforcement",
                "evidence": mac_data.get("mac_domain_baseline"),
            },
            "linux": {
                "status": linux_diff_caps.status == "passed",
                "tier": "enforcement",
                "evidence": linux_diff_caps.stdout_file,
            },
            "wsl": {
                "status": wsl_ok,
                "tier": windows_status.wsl.tier,
                "excluded_from_linux_enforcement": True,
            },
        },
    ]
    for row in rows:
        row["status"] = (
            "passed"
            if row["windows"]["status"] and row["mac"]["status"] and row["linux"]["status"]
            else "failed"
        )
    return rows


def _load_json(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def build_summary(*, bundle_dir: Path, commands: list[CommandRecord]) -> dict[str, object]:
    windows_status = garnet_windows_cross_os_enforcement_proof.read_status(ROOT)
    linux_evidence = garnet_linux_cross_os_enforcement_proof.read_committed_evidence(ROOT)
    if not linux_evidence.verified or linux_evidence.bundle_json is None:
        raise RuntimeError(linux_evidence.reason)
    mac_matrix = _latest_verified_mac_matrix()
    if mac_matrix is None:
        raise RuntimeError("missing verified Mac S109 matrix row under proofs/mac/matrix")

    linux_data = _load_json(linux_evidence.bundle_json)
    mac_data = _load_json(mac_matrix)
    linux_diff_caps = next(command for command in commands if command.id == "linux-diff-caps-reject")
    rows = _trap_rows(
        windows_status=windows_status,
        mac_data=mac_data,
        linux_data=linux_data,
        linux_diff_caps=linux_diff_caps,
    )
    byte_comparisons = mac_data.get("byte_comparisons", [])
    required_byte_ok = all(
        item.get("byte_equal") is True
        for item in byte_comparisons
        if isinstance(item, dict) and item.get("expected_os_independent") is True
    )
    cross_os_complete = all(row.get("status") == "passed" for row in rows) and required_byte_ok
    return {
        "schema": SCHEMA,
        "created_at": datetime.now(UTC).isoformat(timespec="seconds"),
        "status": "passed" if cross_os_complete else "failed",
        "cross_os_complete": cross_os_complete,
        "windows_baseline": _repo_relative(ROOT / "proofs" / "windows" / "enforcement" / "windows-enforcement-proof.json"),
        "mac_matrix_baseline": _repo_relative(mac_matrix),
        "linux_enforcement_baseline": _repo_relative(linux_evidence.bundle_json),
        "commands": [asdict(command) for command in commands],
        "trap_rows": rows,
        "byte_comparisons": byte_comparisons,
        "honest_scope": [
            "Full S109 cross-OS trap parity requires committed Windows, Mac, and Linux rows.",
            "WSL remains execution/portability evidence and is excluded from Linux enforcement.",
            "Linux seccomp is Linux-only evidence, not Windows/macOS OS-sandbox enforcement.",
            "No Wasmtime fuel, production, release, tag, S120, or v1.0 claim is made.",
        ],
    }


def record_matrix(*, output_dir: Path, linux_diff_caps_command: str, format_: str) -> int:
    output_dir.mkdir(parents=True, exist_ok=True)
    command = _run_linux_diff_caps(linux_diff_caps_command, output_dir)
    summary = build_summary(bundle_dir=output_dir, commands=[command])
    _write_text(output_dir / SUMMARY_NAME, json.dumps(summary, indent=2, sort_keys=True) + "\n")
    _write_text(output_dir / MARKDOWN_NAME, render_markdown(summary))
    _write_manifest(output_dir)
    verified = verify_bundle(output_dir / SUMMARY_NAME)
    if format_ == "json":
        print(json.dumps(summary, indent=2, sort_keys=True))
    else:
        print(render_markdown(summary), end="")
    if summary["status"] == "passed" and not verified:
        print("cross-os-trap-parity: bundle verification failed after write", file=sys.stderr)
        return 1
    return 0 if summary["status"] == "passed" else 1


def verify_bundle(summary_path: Path) -> bool:
    try:
        data = json.loads(summary_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return False
    bundle_dir = summary_path.parent
    manifest = _manifest_entries(bundle_dir)
    if manifest is None or SUMMARY_NAME not in manifest or MARKDOWN_NAME not in manifest:
        return False
    if data.get("schema") != SCHEMA or data.get("status") != "passed":
        return False
    if data.get("cross_os_complete") is not True:
        return False
    scope = " ".join(data.get("honest_scope", []))
    if "excluded from Linux enforcement" not in scope or "not Windows/macOS OS-sandbox" not in scope:
        return False
    if "production" not in scope or "S120" not in scope or "v1.0" not in scope:
        return False
    commands = data.get("commands")
    if not isinstance(commands, list) or len(commands) != 1:
        return False
    command = commands[0]
    if (
        not isinstance(command, dict)
        or command.get("id") != "linux-diff-caps-reject"
        or command.get("expected_exit_code") != 1
        or command.get("exit_code") != 1
        or command.get("status") != "passed"
        or command.get("stdout_file") not in manifest
        or command.get("stderr_file") not in manifest
    ):
        return False
    stdout = (bundle_dir / str(command["stdout_file"])).read_text(encoding="utf-8", errors="replace")
    if "AUTHORITY EXPANDED" not in stdout:
        return False
    rows = data.get("trap_rows")
    if not isinstance(rows, list) or {row.get("trap") for row in rows if isinstance(row, dict)} != REQUIRED_TRAPS:
        return False
    for row in rows:
        if not isinstance(row, dict) or row.get("status") != "passed":
            return False
        for os_name in ("windows", "mac", "linux"):
            if row.get(os_name, {}).get("status") is not True:
                return False
        if row.get("wsl", {}).get("excluded_from_linux_enforcement") is not True:
            return False
    comparisons = data.get("byte_comparisons")
    if not isinstance(comparisons, list):
        return False
    for item in comparisons:
        if isinstance(item, dict) and item.get("expected_os_independent") is True:
            if item.get("byte_equal") is not True:
                return False
    return True


def read_committed_evidence(root: Path = ROOT) -> Path | None:
    evidence_root = root / "proofs" / "cross-os" / "matrix"
    if not evidence_root.exists():
        return None
    candidates = sorted(
        evidence_root.rglob(SUMMARY_NAME),
        key=lambda path: path.stat().st_mtime,
        reverse=True,
    )
    for candidate in candidates:
        if verify_bundle(candidate):
            return candidate
    return None


def render_markdown(data: dict[str, object]) -> str:
    lines = [
        "# Garnet S109 Cross-OS Trap Parity Matrix",
        "",
        f"- Status: `{data.get('status')}`",
        f"- Cross-OS complete: `{str(data.get('cross_os_complete')).lower()}`",
        "",
        "## Trap Rows",
        "",
        "| Trap | Status | Windows | Mac | Linux | WSL Treatment |",
        "| --- | --- | --- | --- | --- | --- |",
    ]
    for row in data.get("trap_rows", []):
        if not isinstance(row, dict):
            continue
        lines.append(
            f"| `{row.get('trap')}` | `{row.get('status')}` | "
            f"`{str(row.get('windows', {}).get('status')).lower()}` | "
            f"`{str(row.get('mac', {}).get('status')).lower()}` | "
            f"`{str(row.get('linux', {}).get('status')).lower()}` | "
            f"`{row.get('wsl', {}).get('tier')}; excluded={str(row.get('wsl', {}).get('excluded_from_linux_enforcement')).lower()}` |"
        )
    lines.extend(["", "## Byte Comparisons", "", "| Artifact | Byte Equal | Delta |", "| --- | --- | --- |"])
    for comparison in data.get("byte_comparisons", []):
        if not isinstance(comparison, dict):
            continue
        byte_equal = comparison.get("byte_equal")
        if byte_equal is None:
            byte_equal = comparison.get("full_json_byte_equal")
        lines.append(
            f"| `{comparison.get('id')}` | `{str(byte_equal).lower()}` | "
            f"{comparison.get('delta', '')} |"
        )
    lines.extend(["", "## Honest Scope", ""])
    lines.extend(f"- {item}" for item in data.get("honest_scope", []) if isinstance(item, str))
    lines.append("")
    return "\n".join(lines)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--record", action="store_true")
    parser.add_argument("--output-dir", type=Path, default=None)
    parser.add_argument("--linux-diff-caps-command")
    parser.add_argument("--verify", type=Path)
    parser.add_argument("--gate", action="store_true")
    parser.add_argument("--format", choices=["json", "md"], default="json")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    if args.record:
        if not args.linux_diff_caps_command:
            print("--record requires --linux-diff-caps-command", file=sys.stderr)
            return 2
        return record_matrix(
            output_dir=args.output_dir or default_output_dir(),
            linux_diff_caps_command=args.linux_diff_caps_command,
            format_=args.format,
        )
    if args.verify is not None:
        ok = verify_bundle(args.verify)
        print("cross-os-trap-parity: verified" if ok else "cross-os-trap-parity: FAILED")
        return 0 if ok else 1
    committed = read_committed_evidence(ROOT)
    payload = {
        "schema": SCHEMA,
        "verified": committed is not None,
        "bundle_json": _repo_relative(committed) if committed else None,
    }
    print(json.dumps(payload, indent=2) if args.format == "json" else render_markdown(payload))
    return 0 if (committed is not None or not args.gate) else 1


if __name__ == "__main__":
    raise SystemExit(main())
