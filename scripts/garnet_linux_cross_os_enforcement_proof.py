#!/usr/bin/env python3
"""Record and verify the S108 Linux cross-OS enforcement proof.

S108 is the independent Linux row for S109 consolidation. It reruns the already
merged Stage-V enforcement gates on a real Linux host and, where seccomp tooling
is present, attempts the S92/S46 named-deferred OS-policy apply path as a Linux
only datapoint. This is not a Windows/macOS sandbox claim and not full S109
completion by itself.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import shlex
import subprocess
import sys
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = "garnet.linux_cross_os_enforcement_proof.v1"
SUMMARY_NAME = "garnet-linux-cross-os-enforcement-proof.json"
MARKDOWN_NAME = "garnet-linux-cross-os-enforcement-proof.md"
MANIFEST_NAME = "MANIFEST.sha256"
DEFAULT_ROOT = ROOT / "proofs" / "linux" / "enforcement"
REQUIRED_COMMANDS = {
    "linux-s101-gate",
    "linux-bounded-enforcement",
    "linux-caps-enforcement",
    "linux-seccomp-apply",
}
REQUIRED_TRAPS = {
    "max_depth",
    "caps",
    "s92_program_entry_proc",
}


@dataclass(frozen=True)
class LinuxCrossOsEvidence:
    verified: bool
    bundle_json: Path | None
    reason: str
    deferred: list[str]


def timestamp_slug(now: datetime | None = None) -> str:
    return (now or datetime.now(UTC)).strftime("%Y%m%d-%H%M%S")


def default_output_dir(now: datetime | None = None) -> Path:
    return DEFAULT_ROOT / f"utm-linux-enforcement-{timestamp_slug(now)}"


def _sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _sha256(path: Path) -> str:
    return _sha256_bytes(path.read_bytes())


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8", newline="\n")


def _bundle_relative(bundle_dir: Path, path: Path) -> str:
    return path.resolve().relative_to(bundle_dir.resolve()).as_posix()


def _repo_relative_display(path: Path) -> str:
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
        match = re.fullmatch(r"([0-9a-f]{64})  ([^\r\n]+)", line)
        if not match:
            return None
        digest, relative = match.groups()
        target = bundle_dir / Path(relative)
        if not target.is_file() or _sha256(target) != digest:
            return None
        entries[relative.replace("\\", "/")] = digest
    return entries


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


def _ssh_command(
    ssh_target: str,
    remote_root: str,
    command: list[str],
) -> subprocess.CompletedProcess[str]:
    quoted = " ".join(shlex.quote(part) for part in command)
    remote = f'. "$HOME/.cargo/env" 2>/dev/null || true; cd {shlex.quote(remote_root)} && {quoted}'
    return _run(
        [
            "ssh",
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            ssh_target,
            remote,
        ]
    )


def _ssh_text(ssh_target: str, command: str) -> str:
    result = _run(
        [
            "ssh",
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            ssh_target,
            command,
        ]
    )
    return result.stdout.strip() if result.returncode == 0 else ""


def _record_command(
    *,
    command_id: str,
    display_args: list[str],
    completed: subprocess.CompletedProcess[str],
    bundle_dir: Path,
) -> dict[str, object]:
    stdout_rel = Path("commands") / f"{command_id}-stdout.txt"
    stderr_rel = Path("commands") / f"{command_id}-stderr.txt"
    _write_text(bundle_dir / stdout_rel, completed.stdout)
    _write_text(bundle_dir / stderr_rel, completed.stderr)
    return {
        "id": command_id,
        "display_args": display_args,
        "exit_code": completed.returncode,
        "stdout_file": stdout_rel.as_posix(),
        "stderr_file": stderr_rel.as_posix(),
        "status": "passed" if completed.returncode == 0 else "failed",
    }


def _linux_stage_v_commands(
    ssh_target: str,
    remote_root: str,
    bundle_dir: Path,
) -> list[dict[str, object]]:
    command_specs = [
        (
            "linux-s101-gate",
            [
                "python3",
                "scripts/garnet_vm_interp_enforcement_parity_status.py",
                "--gate",
                "--format",
                "json",
            ],
        ),
        (
            "linux-bounded-enforcement",
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
            "linux-caps-enforcement",
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
    records: list[dict[str, object]] = []
    for command_id, command in command_specs:
        completed = _ssh_command(ssh_target, remote_root, command)
        records.append(
            _record_command(
                command_id=command_id,
                display_args=command,
                completed=completed,
                bundle_dir=bundle_dir,
            )
        )
    return records


def _seccomp_command(
    ssh_target: str,
    remote_root: str,
    remote_garnet: str,
    bundle_dir: Path,
) -> tuple[dict[str, object], dict[str, object]]:
    proof_script = (
        "for run in 1 2 3; do "
        'echo "== deterministic seccomp run $run/3 =="; '
        f"env GARNET={shlex.quote(remote_garnet)} tools/seccomp-apply/prove.sh || exit $?; "
        "done"
    )
    command = ["bash", "-lc", proof_script]
    completed = _ssh_command(ssh_target, remote_root, command)
    record = _record_command(
        command_id="linux-seccomp-apply",
        display_args=command,
        completed=completed,
        bundle_dir=bundle_dir,
    )
    stdout = completed.stdout
    seccomp = {
        "attempted": True,
        "status": "proven" if completed.returncode == 0 else "failed",
        "denied_socket_trapped": "BLOCKED" in stdout and "Operation not permitted" in stdout,
        "allowed_socket_policy_driven": "ALLOWED" in stdout and "policy-driven" in stdout,
        "deterministic_denied_runs": stdout.count("BLOCKED"),
    }
    return record, seccomp


def record_remote(
    *,
    ssh_target: str,
    remote_root: str,
    remote_garnet: str,
    bundle_dir: Path,
) -> Path:
    bundle_dir.mkdir(parents=True, exist_ok=True)
    commands = _linux_stage_v_commands(ssh_target, remote_root, bundle_dir)
    seccomp_record, seccomp = _seccomp_command(
        ssh_target,
        remote_root,
        remote_garnet,
        bundle_dir,
    )
    commands.append(seccomp_record)
    environment = {
        "kind": "utm-debian-12-arm64",
        "ssh_target": ssh_target,
        "remote_root": remote_root,
        "kernel": _ssh_text(ssh_target, "uname -a"),
        "git_head": _ssh_text(ssh_target, f"cd {shlex.quote(remote_root)} && git rev-parse HEAD"),
        "git_status_short": _ssh_text(
            ssh_target,
            f"cd {shlex.quote(remote_root)} && git status --short --branch",
        ),
    }
    summary: dict[str, object] = {
        "schema": SCHEMA,
        "platform": "linux",
        "cross_os_role": "S108 Linux row for S109 consolidation",
        "status": "passed" if all(command.get("exit_code") == 0 for command in commands) else "failed",
        "tier": "linux-enforcement-proof",
        "generated_at": datetime.now(UTC).isoformat(),
        "environment": environment,
        "stage_v_traps": [
            {"trap": "max_depth", "status": "passed" if commands[0]["exit_code"] == 0 else "failed"},
            {"trap": "caps", "status": "passed" if commands[2]["exit_code"] == 0 else "failed"},
            {
                "trap": "s92_program_entry_proc",
                "status": "passed" if commands[2]["exit_code"] == 0 else "failed",
            },
        ],
        "seccomp": seccomp,
        "commands": commands,
        "honest_scope": [
            "This is the independent Linux S108 enforcement row for S109 consolidation.",
            "Linux seccomp is Linux-only evidence, not Windows/macOS OS-sandbox enforcement.",
            "This is not full S109 completion; S109 still needs a separate consolidation gate update.",
            "No Wasmtime fuel, production, or v1.0 claim is made.",
        ],
    }
    summary_path = bundle_dir / SUMMARY_NAME
    _write_text(summary_path, json.dumps(summary, indent=2) + "\n")
    _write_text(bundle_dir / MARKDOWN_NAME, render_markdown(summary))
    _write_manifest(bundle_dir)
    return summary_path


def render_markdown(data: dict[str, object]) -> str:
    environment = data.get("environment", {})
    seccomp = data.get("seccomp", {})
    lines = [
        "# S108 Linux Cross-OS Enforcement Proof",
        "",
        f"- Schema: `{data.get('schema')}`",
        f"- Status: `{data.get('status')}`",
        f"- Tier: `{data.get('tier')}`",
        f"- Cross-OS role: `{data.get('cross_os_role')}`",
    ]
    if isinstance(environment, dict):
        lines.extend(
            [
                f"- Environment: `{environment.get('kind', 'unknown')}`",
                f"- Kernel: `{environment.get('kernel', 'unknown')}`",
                f"- Git head: `{environment.get('git_head', 'unknown')}`",
            ]
        )
    if isinstance(seccomp, dict):
        lines.extend(
            [
                f"- Seccomp attempted: `{str(seccomp.get('attempted')).lower()}`",
                f"- Seccomp status: `{seccomp.get('status')}`",
                f"- Denied socket trapped: `{str(seccomp.get('denied_socket_trapped')).lower()}`",
                f"- Declared net socket policy-driven: `{str(seccomp.get('allowed_socket_policy_driven')).lower()}`",
                f"- Deterministic denied runs: `{seccomp.get('deterministic_denied_runs')}`",
            ]
        )
    lines.extend(["", "## Stage-V Traps", ""])
    traps = data.get("stage_v_traps", [])
    if isinstance(traps, list):
        for row in traps:
            if isinstance(row, dict):
                lines.append(f"- `{row.get('trap')}`: `{row.get('status')}`")
    lines.extend(["", "## Honest Scope", ""])
    scope = data.get("honest_scope", [])
    if isinstance(scope, list):
        for item in scope:
            lines.append(f"- {item}")
    return "\n".join(lines) + "\n"


def _command_files_ok(bundle_dir: Path, manifest_entries: dict[str, str], command: dict[str, object]) -> bool:
    if command.get("exit_code") != 0 or command.get("status") != "passed":
        return False
    for key in ("stdout_file", "stderr_file"):
        value = command.get(key)
        if not isinstance(value, str) or value not in manifest_entries:
            return False
        path = bundle_dir / Path(value)
        if not path.is_file():
            return False
    return True


def verify_bundle(summary: Path) -> bool:
    try:
        data = json.loads(summary.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return False
    bundle_dir = summary.parent
    manifest_entries = _manifest_entries(bundle_dir)
    if manifest_entries is None:
        return False
    if SUMMARY_NAME not in manifest_entries or MARKDOWN_NAME not in manifest_entries:
        return False
    commands = data.get("commands", [])
    if not isinstance(commands, list):
        return False
    by_id = {command.get("id"): command for command in commands if isinstance(command, dict)}
    if set(by_id) != REQUIRED_COMMANDS:
        return False
    if not all(_command_files_ok(bundle_dir, manifest_entries, command) for command in by_id.values()):
        return False
    traps = data.get("stage_v_traps", [])
    if not isinstance(traps, list):
        return False
    trap_status = {row.get("trap"): row.get("status") for row in traps if isinstance(row, dict)}
    if set(trap_status) != REQUIRED_TRAPS or any(status != "passed" for status in trap_status.values()):
        return False
    seccomp = data.get("seccomp", {})
    if not isinstance(seccomp, dict):
        return False
    scope = data.get("honest_scope", [])
    scope_text = "\n".join(item for item in scope if isinstance(item, str))
    return (
        data.get("schema") == SCHEMA
        and data.get("platform") == "linux"
        and data.get("status") == "passed"
        and data.get("tier") == "linux-enforcement-proof"
        and data.get("cross_os_role") == "S108 Linux row for S109 consolidation"
        and seccomp.get("attempted") is True
        and seccomp.get("status") == "proven"
        and seccomp.get("denied_socket_trapped") is True
        and seccomp.get("allowed_socket_policy_driven") is True
        and isinstance(seccomp.get("deterministic_denied_runs"), int)
        and seccomp.get("deterministic_denied_runs") >= 3
        and "independent Linux S108 enforcement row" in scope_text
        and "not Windows/macOS OS-sandbox enforcement" in scope_text
        and "not full S109 completion" in scope_text
        and "No Wasmtime fuel, production, or v1.0 claim" in scope_text
    )


def _verified_linux_enforcement_under(root: Path) -> Path | None:
    if not root.exists():
        return None
    candidates = sorted(
        root.rglob(SUMMARY_NAME),
        key=lambda path: path.stat().st_mtime,
        reverse=True,
    )
    for candidate in candidates:
        if verify_bundle(candidate):
            return candidate
    return None


def read_committed_evidence(root: Path = ROOT) -> LinuxCrossOsEvidence:
    proof = _verified_linux_enforcement_under(root / "proofs" / "linux" / "enforcement")
    if proof is None:
        return LinuxCrossOsEvidence(
            False,
            None,
            "No committed S108 Linux enforcement proof bundle exists yet.",
            [
                "independent Linux S108 enforcement row",
                "Linux seccomp datapoint where present",
            ],
        )
    return LinuxCrossOsEvidence(
        True,
        proof,
        (
            f"S108 Linux enforcement proof verified at `{_repo_relative_display(proof)}`. "
            "It records Stage-V max-depth/caps traps on Linux plus a deterministic "
            "Linux-only seccomp apply datapoint."
        ),
        [
            "not Windows/macOS OS-sandbox enforcement",
            "not full S109 completion",
            "No Wasmtime fuel, production, or v1.0 claim",
        ],
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--record", action="store_true")
    parser.add_argument("--verify", type=Path)
    parser.add_argument("--gate", action="store_true")
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument("--out-dir", type=Path, default=None)
    parser.add_argument("--ssh-target", default="debian@192.168.64.4")
    parser.add_argument("--remote-root", default="/home/debian/garnet")
    parser.add_argument("--remote-garnet", default="target/debug/garnet")
    args = parser.parse_args(list(argv) if argv is not None else None)

    if args.record:
        out_dir = args.out_dir or default_output_dir()
        summary = record_remote(
            ssh_target=args.ssh_target,
            remote_root=args.remote_root,
            remote_garnet=args.remote_garnet,
            bundle_dir=out_dir,
        )
        ok = verify_bundle(summary)
        print(summary)
        return 0 if ok else 1

    if args.verify is not None:
        ok = verify_bundle(args.verify)
        print("linux-cross-os-enforcement: verified" if ok else "linux-cross-os-enforcement: FAILED")
        return 0 if ok else 1

    evidence = read_committed_evidence(ROOT)
    payload = {
        "schema": SCHEMA,
        "verified": evidence.verified,
        "bundle_json": _repo_relative_display(evidence.bundle_json) if evidence.bundle_json else None,
        "reason": evidence.reason,
        "deferred": evidence.deferred,
    }
    if args.format == "md":
        print("\n".join(["# S108 Linux Enforcement Evidence", "", evidence.reason, ""]))
    else:
        print(json.dumps(payload, indent=2))
    if args.gate and not evidence.verified:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
