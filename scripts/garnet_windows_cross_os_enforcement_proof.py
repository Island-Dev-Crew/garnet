#!/usr/bin/env python3
"""Windows + WSL Stage V trap proof recorder/status gate (S106 Phase 1).

This script does not add enforcement behavior. It records whether this Windows
machine can reproduce the already-merged Stage V trap gates:

- `@max_depth` VM/interp trap parity (S99/S101)
- `@caps` host-authority trap parity for env/proc/fs/net (S100/S101)
- the S92 program-entry `@caps(proc)` laundering trap on the VM

The WSL row is deliberately labeled execution/portability only. It is not Linux
seccomp, Wasmtime, or OS-sandbox enforcement proof.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shlex
import subprocess
import sys
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Iterable

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = "garnet.windows_cross_os_enforcement_proof/v1"
WINDOWS_PROOF = (
    Path("proofs") / "windows" / "enforcement" / "windows-enforcement-proof.json"
)
WSL_PROOF = (
    Path("proofs") / "linux" / "execution" / "wsl-execution-portability-proof.json"
)

REQUIRED_TRAPS = [
    "@max_depth",
    "@caps(env)",
    "@caps(proc)",
    "@caps(fs)",
    "@caps(net)",
    "S92 program-entry @caps(proc)",
]

WINDOWS_SCOPE = "Windows enforcement proof"
WSL_SCOPE = "WSL execution/portability, not enforcement"
NAMED_DEFERRED = [
    "WSL is not Linux seccomp enforcement",
    "WSL is not OS-sandbox enforcement",
    "Wasmtime fuel / @bounded runtime enforcement remains out of scope",
    "memory/time/@mailbox runtime ceilings remain out of scope",
]


@dataclass
class CommandEvidence:
    name: str
    argv: list[str]
    exit_code: int
    stdout_log: str
    stderr_log: str
    stdout_sha256: str
    stderr_sha256: str


@dataclass
class ProofRecord:
    schema: str
    platform: str
    tier: str
    honesty_scope: str
    required_traps: list[str]
    commands: list[dict]
    ok: bool
    generated_at: str | None = None
    git_head: str | None = None
    git_status_short: str | None = None
    named_deferred: list[str] = field(default_factory=list)
    notes: list[str] = field(default_factory=list)


@dataclass
class S106Status:
    schema: str
    windows: ProofRecord
    wsl: ProofRecord
    ok: bool


def _sha256(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8", errors="replace")).hexdigest()


def _run(args: list[str], *, cwd: Path = ROOT) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=cwd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=False,
    )


def _git_text(args: list[str]) -> str:
    result = _run(["git", *args])
    return result.stdout.strip() if result.returncode == 0 else ""


def _command_set(platform: str) -> list[tuple[str, list[str]]]:
    python_cmd = "python3" if platform == "wsl" else sys.executable
    return [
        (
            "s101_gate",
            [
                python_cmd,
                "scripts/garnet_vm_interp_enforcement_parity_status.py",
                "--gate",
                "--format",
                "json",
            ],
        ),
        (
            "bounded_enforcement",
            [
                "cargo",
                "test",
                "-p",
                "garnet-cli",
                "--test",
                "bounded_enforcement",
                "--",
                "--nocapture",
            ],
        ),
        (
            "caps_enforcement",
            [
                "cargo",
                "test",
                "-p",
                "garnet-cli",
                "--test",
                "caps_enforcement",
                "--",
                "--nocapture",
            ],
        ),
    ]


def _wsl_root() -> str:
    result = _run(["wsl.exe", "-e", "wslpath", "-a", str(ROOT)])
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or "wslpath failed")
    return result.stdout.strip()


def _run_wsl(argv: list[str], wsl_root: str) -> subprocess.CompletedProcess[str]:
    quoted = " ".join(shlex.quote(part) for part in argv)
    command = f"cd {shlex.quote(wsl_root)} && {quoted}"
    return _run(["wsl.exe", "-e", "sh", "-lc", command])


def _record_commands(platform: str, out_dir: Path) -> list[CommandEvidence]:
    out_dir.mkdir(parents=True, exist_ok=True)
    wsl_root = _wsl_root() if platform == "wsl" else ""
    records: list[CommandEvidence] = []
    for name, argv in _command_set(platform):
        result = _run_wsl(argv, wsl_root) if platform == "wsl" else _run(argv)
        stdout_name = f"{name}.stdout.log"
        stderr_name = f"{name}.stderr.log"
        (out_dir / stdout_name).write_text(result.stdout, encoding="utf-8")
        (out_dir / stderr_name).write_text(result.stderr, encoding="utf-8")
        records.append(
            CommandEvidence(
                name=name,
                argv=argv,
                exit_code=result.returncode,
                stdout_log=stdout_name,
                stderr_log=stderr_name,
                stdout_sha256=_sha256(result.stdout),
                stderr_sha256=_sha256(result.stderr),
            )
        )
    return records


def _proof_filename(platform: str) -> str:
    if platform == "windows":
        return "windows-enforcement-proof.json"
    if platform == "wsl":
        return "wsl-execution-portability-proof.json"
    raise ValueError(f"unknown platform: {platform}")


def _markdown_filename(platform: str) -> str:
    return _proof_filename(platform).replace(".json", ".md")


def _manifest(out_dir: Path) -> None:
    lines: list[str] = []
    for path in sorted(p for p in out_dir.rglob("*") if p.is_file()):
        if path.name == "MANIFEST.sha256":
            continue
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        rel = path.relative_to(out_dir).as_posix()
        lines.append(f"{digest}  {rel}")
    (out_dir / "MANIFEST.sha256").write_text("\n".join(lines) + "\n", encoding="utf-8")


def build_record(platform: str, out_dir: Path) -> ProofRecord:
    if platform not in {"windows", "wsl"}:
        raise ValueError("--platform must be windows or wsl")
    commands = _record_commands(platform, out_dir)
    tier = "enforcement-proof" if platform == "windows" else "execution-portability"
    scope = WINDOWS_SCOPE if platform == "windows" else WSL_SCOPE
    ok = all(command.exit_code == 0 for command in commands)
    return ProofRecord(
        schema=SCHEMA,
        platform=platform,
        tier=tier,
        honesty_scope=scope,
        required_traps=REQUIRED_TRAPS,
        commands=[asdict(command) for command in commands],
        ok=ok,
        generated_at=datetime.now(timezone.utc).isoformat(),
        git_head=_git_text(["rev-parse", "HEAD"]),
        git_status_short=_git_text(["status", "--short"]),
        named_deferred=NAMED_DEFERRED,
        notes=[],
    )


def write_record(record: ProofRecord, out_dir: Path) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    record_path = out_dir / _proof_filename(record.platform)
    record_path.write_text(json.dumps(asdict(record), indent=2) + "\n", encoding="utf-8")
    (out_dir / _markdown_filename(record.platform)).write_text(
        render_record_markdown(record), encoding="utf-8"
    )
    _manifest(out_dir)


def _missing_record(platform: str, path: Path) -> ProofRecord:
    tier = "enforcement-proof" if platform == "windows" else "execution-portability"
    scope = WINDOWS_SCOPE if platform == "windows" else WSL_SCOPE
    return ProofRecord(
        schema=SCHEMA,
        platform=platform,
        tier=tier,
        honesty_scope=scope,
        required_traps=[],
        commands=[],
        ok=False,
        named_deferred=NAMED_DEFERRED,
        notes=[f"missing proof record: {path.as_posix()}"],
    )


def _load_record(path: Path, platform: str) -> ProofRecord:
    if not path.is_file():
        return _missing_record(platform, path)
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        record = _missing_record(platform, path)
        record.notes = [f"invalid JSON: {exc}"]
        return record
    record = ProofRecord(
        schema=raw.get("schema", ""),
        platform=raw.get("platform", ""),
        tier=raw.get("tier", ""),
        honesty_scope=raw.get("honesty_scope", ""),
        required_traps=list(raw.get("required_traps", [])),
        commands=list(raw.get("commands", [])),
        ok=bool(raw.get("ok", False)),
        generated_at=raw.get("generated_at"),
        git_head=raw.get("git_head"),
        git_status_short=raw.get("git_status_short"),
        named_deferred=list(raw.get("named_deferred", [])),
        notes=list(raw.get("notes", [])),
    )
    return _validate_record(record, platform)


def _validate_record(record: ProofRecord, expected_platform: str) -> ProofRecord:
    notes = list(record.notes)
    if record.schema != SCHEMA:
        notes.append(f"schema mismatch: {record.schema}")
    if record.platform != expected_platform:
        notes.append(f"platform mismatch: {record.platform}")
    expected_tier = (
        "enforcement-proof" if expected_platform == "windows" else "execution-portability"
    )
    if record.tier != expected_tier:
        if expected_platform == "wsl":
            notes.append("WSL row must be execution/portability, not enforcement")
        else:
            notes.append(f"Windows row must be {expected_tier}")
    missing_traps = [trap for trap in REQUIRED_TRAPS if trap not in record.required_traps]
    if missing_traps:
        notes.append(f"missing trap labels: {', '.join(missing_traps)}")
    failed = [
        str(command.get("name", "<unknown>"))
        for command in record.commands
        if int(command.get("exit_code", 1)) != 0
    ]
    if failed:
        notes.append(f"failed commands: {', '.join(failed)}")
    if len(record.commands) < 3:
        notes.append("expected s101_gate, bounded_enforcement, and caps_enforcement")
    if expected_platform == "wsl" and "not enforcement" not in record.honesty_scope:
        notes.append("WSL honesty scope must say not enforcement")
    record.notes = notes
    record.ok = record.ok and not notes
    return record


def read_status(root: Path = ROOT) -> S106Status:
    windows = _load_record(root / WINDOWS_PROOF, "windows")
    wsl = _load_record(root / WSL_PROOF, "wsl")
    return S106Status(
        schema=SCHEMA,
        windows=windows,
        wsl=wsl,
        ok=windows.ok and wsl.ok,
    )


def _commands_summary(commands: Iterable[dict]) -> str:
    parts = []
    for command in commands:
        status = "pass" if int(command.get("exit_code", 1)) == 0 else "FAIL"
        parts.append(f"{command.get('name', '<unknown>')}={status}")
    return ", ".join(parts) or "none"


def render_record_markdown(record: ProofRecord) -> str:
    return "\n".join(
        [
            f"# Garnet S106 {record.platform} Stage V trap proof",
            "",
            f"_Schema {record.schema}._",
            "",
            f"- tier: `{record.tier}`",
            f"- honesty scope: {record.honesty_scope}",
            f"- git head: `{record.git_head or 'unknown'}`",
            f"- commands: {_commands_summary(record.commands)}",
            f"- required traps: {', '.join(record.required_traps) or 'none'}",
            f"- verdict: {'ok' if record.ok else 'NOT OK'}",
            "",
            "Named deferred boundaries:",
            *[f"- {item}" for item in (record.named_deferred or NAMED_DEFERRED)],
            "",
        ]
    )


def render_markdown(status: S106Status) -> str:
    return "\n".join(
        [
            "# Garnet S106 Windows cross-OS enforcement proof",
            "",
            f"_Schema {status.schema}._",
            "",
            f"- Windows Stage V trap proof: {'ok' if status.windows.ok else 'NO'}",
            f"- WSL execution/portability, not enforcement: {'ok' if status.wsl.ok else 'NO'}",
            f"- Windows tier: `{status.windows.tier}`",
            f"- WSL tier: `{status.wsl.tier}`",
            f"- traps checked: {', '.join(REQUIRED_TRAPS)}",
            "",
            "The Windows row re-proves the S101 Stage V trap gate on this Windows box. "
            "The WSL row is execution/portability, not enforcement: it is not Linux "
            "seccomp, not Wasmtime fuel, and not OS-sandbox enforcement.",
            "",
            "Remaining named-deferred boundaries:",
            *[f"- {item}" for item in NAMED_DEFERRED],
            "",
            "Notes:",
            *[f"- windows: {note}" for note in status.windows.notes],
            *[f"- wsl: {note}" for note in status.wsl.notes],
            "",
        ]
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument("--gate", action="store_true")
    parser.add_argument("--record-proof", action="store_true")
    parser.add_argument("--platform", choices=["windows", "wsl"], default="windows")
    parser.add_argument("--out", type=Path)
    args = parser.parse_args(list(argv) if argv is not None else None)

    if args.record_proof:
        out_dir = args.out
        if out_dir is None:
            out_dir = (
                ROOT / "proofs" / "windows" / "enforcement"
                if args.platform == "windows"
                else ROOT / "proofs" / "linux" / "execution"
            )
        record = build_record(args.platform, out_dir)
        write_record(record, out_dir)
        print(
            render_record_markdown(record)
            if args.format == "md"
            else json.dumps(asdict(record), indent=2)
        )
        return 0 if record.ok else 1

    status = read_status()
    print(
        render_markdown(status)
        if args.format == "md"
        else json.dumps(asdict(status), indent=2)
    )
    if args.gate and not status.ok:
        print(f"S106 Windows/WSL proof gate FAILED: {asdict(status)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
