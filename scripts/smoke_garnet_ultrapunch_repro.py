#!/usr/bin/env python3
"""Record a manifest-backed S110 ultrapunch reproduction bundle.

The canonical S104 script proves the loop and deletes its temp directory. This
recorder keeps the evidence: ACCEPT emits the four trust artifacts and verifies
the transparency-log chain; REJECT proves both a capability widening and an
over-depth proposal are refused and never sealed.

Honest scope: WSL/Linux rows produced by this script are reproduction /
portability evidence unless a separate real-kernel enforcement proof says
otherwise. This script does not prove seccomp, OS-sandbox application, Wasmtime
fuel, production readiness, or v1.0 readiness.
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
FIXTURES = ROOT / "garnet-cli" / "tests" / "fixtures" / "ultrapunch"
SUMMARY_NAME = "garnet-ultrapunch-repro.json"
MARKDOWN_NAME = "garnet-ultrapunch-repro.md"
MANIFEST_NAME = "MANIFEST.sha256"
SCHEMA = "garnet.ultrapunch.repro.v1"
ACCEPT_ARTIFACTS = [
    "capability_manifest.json",
    "diff_caps.txt",
    "seal.json",
    "transparency_log.jsonl",
    "decision.md",
]


@dataclass(frozen=True)
class CommandRecord:
    id: str
    display_args: list[str]
    exit_code: int
    stdout_file: str
    stderr_file: str
    expected_failure: bool
    status: str


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _repo_relative(path: Path) -> str:
    try:
        return path.resolve().relative_to(ROOT.resolve()).as_posix()
    except ValueError:
        return path.name


def _bundle_relative(bundle_dir: Path, path: Path) -> str:
    return path.resolve().relative_to(bundle_dir.resolve()).as_posix()


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8", newline="\n")


def _run(
    *,
    command_id: str,
    command: list[str],
    display_args: list[str],
    bundle_dir: Path,
    expected_failure: bool = False,
) -> CommandRecord:
    completed = subprocess.run(
        command,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        capture_output=True,
        check=False,
    )
    stdout = Path("commands") / f"{command_id}-stdout.txt"
    stderr = Path("commands") / f"{command_id}-stderr.txt"
    _write_text(bundle_dir / stdout, completed.stdout)
    _write_text(bundle_dir / stderr, completed.stderr)
    outcome_ok = completed.returncode != 0 if expected_failure else completed.returncode == 0
    return CommandRecord(
        id=command_id,
        display_args=display_args,
        exit_code=completed.returncode,
        stdout_file=stdout.as_posix(),
        stderr_file=stderr.as_posix(),
        expected_failure=expected_failure,
        status="passed" if outcome_ok else "failed",
    )


def _garnet_display(garnet_cmd: list[str]) -> list[str]:
    if not garnet_cmd:
        return ["garnet"]
    first = Path(garnet_cmd[0]).name or garnet_cmd[0]
    return [first, *garnet_cmd[1:]]


def _agent_loop_command(
    garnet_cmd: list[str],
    *,
    proposal: Path,
    record_dir: Path,
    seal_out: Path,
    attest: bool,
) -> list[str]:
    command = [
        *garnet_cmd,
        "agent-loop",
        "--baseline",
        str(FIXTURES / "baseline.garnet"),
        "--proposal",
        str(proposal),
        "--seal-out",
        str(seal_out),
        "--record-dir",
        str(record_dir),
    ]
    if attest:
        command.extend(
            [
                "--attest",
                "agent=scripted-agent-v1",
                "--attest",
                "model=simulated",
                "--gate-version",
                "dogfood-gate-v1",
            ]
        )
    return command


def _agent_loop_display(
    garnet_cmd: list[str],
    *,
    proposal_name: str,
    record_name: str,
    seal_name: str,
    attest: bool,
) -> list[str]:
    display = [
        *_garnet_display(garnet_cmd),
        "agent-loop",
        "--baseline",
        "garnet-cli/tests/fixtures/ultrapunch/baseline.garnet",
        "--proposal",
        f"garnet-cli/tests/fixtures/ultrapunch/{proposal_name}",
        "--seal-out",
        seal_name,
        "--record-dir",
        record_name,
    ]
    if attest:
        display.extend(
            [
                "--attest",
                "agent=scripted-agent-v1",
                "--attest",
                "model=simulated",
                "--gate-version",
                "dogfood-gate-v1",
            ]
        )
    return display


def _seal_present(record_dir: Path, seal_out: Path) -> bool:
    return (record_dir / "seal.json").is_file() or seal_out.is_file()


def _write_manifest(bundle_dir: Path) -> None:
    entries: list[str] = []
    for path in sorted(p for p in bundle_dir.rglob("*") if p.is_file()):
        if path.name == MANIFEST_NAME:
            continue
        relative = _bundle_relative(bundle_dir, path)
        entries.append(f"{_sha256(path)}  {relative}")
    _write_text(bundle_dir / MANIFEST_NAME, "\n".join(entries) + "\n")


def _manifest_entries(bundle_dir: Path) -> dict[str, str] | None:
    manifest = bundle_dir / MANIFEST_NAME
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
        target = bundle_dir / Path(relative)
        if not target.is_file() or _sha256(target) != digest:
            return None
        entries[relative.replace("\\", "/")] = digest
    return entries


def verify_bundle(summary_path: Path) -> bool:
    try:
        data = json.loads(summary_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return False
    bundle_dir = summary_path.parent
    manifest = _manifest_entries(bundle_dir)
    if manifest is None:
        return False
    required = {
        SUMMARY_NAME,
        MARKDOWN_NAME,
        "accept/capability_manifest.json",
        "accept/diff_caps.txt",
        "accept/seal.json",
        "accept/transparency_log.jsonl",
        "accept/decision.md",
    }
    if not required.issubset(manifest):
        return False
    commands = data.get("commands")
    if not isinstance(commands, list) or len(commands) != 4:
        return False
    for command in commands:
        if not isinstance(command, dict):
            return False
        if command.get("status") != "passed":
            return False
        stdout_file = command.get("stdout_file")
        stderr_file = command.get("stderr_file")
        if not isinstance(stdout_file, str) or not isinstance(stderr_file, str):
            return False
        if stdout_file not in manifest or stderr_file not in manifest:
            return False
    accept = data.get("accept", {})
    reject_widen = data.get("reject_widen", {})
    reject_overdepth = data.get("reject_overdepth", {})
    return (
        data.get("schema") == SCHEMA
        and data.get("status") == "passed"
        and data.get("source_included") is False
        and data.get("provider_api_called") is False
        and accept.get("chain_verified") is True
        and sorted(accept.get("artifacts", [])) == sorted(ACCEPT_ARTIFACTS)
        and accept.get("sealed") is True
        and reject_widen.get("refused") is True
        and reject_widen.get("sealed") is False
        and reject_overdepth.get("refused") is True
        and reject_overdepth.get("sealed") is False
    )


def render_markdown(data: dict) -> str:
    return "\n".join(
        [
            "# Garnet S110 Ultrapunch Reproduction",
            "",
            f"- platform: `{data['platform']}`",
            f"- evidence tier: `{data['evidence_tier']}`",
            f"- status: `{data['status']}`",
            f"- accept artifacts retained: {', '.join(data['accept']['artifacts'])}",
            f"- transparency log verified: {'yes' if data['accept']['chain_verified'] else 'NO'}",
            f"- widening refused and never sealed: {'yes' if data['reject_widen']['refused'] and not data['reject_widen']['sealed'] else 'NO'}",
            f"- over-depth refused and never sealed: {'yes' if data['reject_overdepth']['refused'] and not data['reject_overdepth']['sealed'] else 'NO'}",
            "",
            "Honest scope: accepted on capability + depth evidence only. WSL/Linux rows "
            "from this recorder are portability-repro evidence unless paired with a "
            "separate real-kernel enforcement proof. This is not seccomp, OS-sandbox, "
            "Wasmtime fuel, production, or v1.0 proof.",
            "",
        ]
    )


def record_repro(
    *,
    platform: str,
    evidence_tier: str,
    garnet_cmd: list[str],
    output_dir: Path,
    format_: str,
) -> int:
    output_dir.mkdir(parents=True, exist_ok=True)

    accept_dir = output_dir / "accept"
    reject_widen_dir = output_dir / "reject-widen"
    reject_overdepth_dir = output_dir / "reject-overdepth"
    accept_seal_out = output_dir / "accept-seal.json"
    reject_widen_seal_out = output_dir / "reject-widen-seal.json"
    reject_overdepth_seal_out = output_dir / "reject-overdepth-seal.json"

    commands: list[CommandRecord] = []
    commands.append(
        _run(
            command_id="accept-agent-loop",
            command=_agent_loop_command(
                garnet_cmd,
                proposal=FIXTURES / "accept_proposal.garnet",
                record_dir=accept_dir,
                seal_out=accept_seal_out,
                attest=True,
            ),
            display_args=_agent_loop_display(
                garnet_cmd,
                proposal_name="accept_proposal.garnet",
                record_name="accept",
                seal_name="accept-seal.json",
                attest=True,
            ),
            bundle_dir=output_dir,
        )
    )

    accept_artifacts = [name for name in ACCEPT_ARTIFACTS if (accept_dir / name).is_file()]
    chain_command = [
        *garnet_cmd,
        "caps-log",
        "--verify",
        str(accept_dir / "transparency_log.jsonl"),
    ]
    commands.append(
        _run(
            command_id="accept-caps-log-verify",
            command=chain_command,
            display_args=[
                *_garnet_display(garnet_cmd),
                "caps-log",
                "--verify",
                "accept/transparency_log.jsonl",
            ],
            bundle_dir=output_dir,
        )
    )

    commands.append(
        _run(
            command_id="reject-widen-agent-loop",
            command=_agent_loop_command(
                garnet_cmd,
                proposal=FIXTURES / "reject_widen.garnet",
                record_dir=reject_widen_dir,
                seal_out=reject_widen_seal_out,
                attest=False,
            ),
            display_args=_agent_loop_display(
                garnet_cmd,
                proposal_name="reject_widen.garnet",
                record_name="reject-widen",
                seal_name="reject-widen-seal.json",
                attest=False,
            ),
            bundle_dir=output_dir,
            expected_failure=True,
        )
    )

    commands.append(
        _run(
            command_id="reject-overdepth-agent-loop",
            command=_agent_loop_command(
                garnet_cmd,
                proposal=FIXTURES / "reject_overdepth.garnet",
                record_dir=reject_overdepth_dir,
                seal_out=reject_overdepth_seal_out,
                attest=False,
            ),
            display_args=_agent_loop_display(
                garnet_cmd,
                proposal_name="reject_overdepth.garnet",
                record_name="reject-overdepth",
                seal_name="reject-overdepth-seal.json",
                attest=False,
            ),
            bundle_dir=output_dir,
            expected_failure=True,
        )
    )

    source_files = []
    for name in [
        "baseline.garnet",
        "accept_proposal.garnet",
        "reject_widen.garnet",
        "reject_overdepth.garnet",
    ]:
        path = FIXTURES / name
        source_files.append({"path": _repo_relative(path), "sha256": _sha256(path)})

    accept_ok = (
        commands[0].status == "passed"
        and commands[1].status == "passed"
        and sorted(accept_artifacts) == sorted(ACCEPT_ARTIFACTS)
        and _seal_present(accept_dir, accept_seal_out)
    )
    reject_widen_sealed = _seal_present(reject_widen_dir, reject_widen_seal_out)
    reject_overdepth_sealed = _seal_present(reject_overdepth_dir, reject_overdepth_seal_out)
    reject_widen_ok = commands[2].status == "passed" and not reject_widen_sealed
    reject_overdepth_ok = commands[3].status == "passed" and not reject_overdepth_sealed
    ok = accept_ok and reject_widen_ok and reject_overdepth_ok

    data = {
        "schema": SCHEMA,
        "generated_at": datetime.now(UTC).isoformat(timespec="seconds"),
        "platform": platform,
        "evidence_tier": evidence_tier,
        "status": "passed" if ok else "failed",
        "source_included": False,
        "provider_api_called": False,
        "source_files": source_files,
        "command_count": len(commands),
        "passed_commands": sum(1 for command in commands if command.status == "passed"),
        "failed_commands": sum(1 for command in commands if command.status != "passed"),
        "accept": {
            "proposal": "accept_proposal.garnet",
            "artifacts": accept_artifacts,
            "chain_verified": commands[1].status == "passed",
            "sealed": _seal_present(accept_dir, accept_seal_out),
        },
        "reject_widen": {
            "proposal": "reject_widen.garnet",
            "refused": commands[2].status == "passed",
            "sealed": reject_widen_sealed,
            "expected_stage": "diff-caps",
        },
        "reject_overdepth": {
            "proposal": "reject_overdepth.garnet",
            "refused": commands[3].status == "passed",
            "sealed": reject_overdepth_sealed,
            "expected_stage": "enforced-kernel",
        },
        "commands": [asdict(command) for command in commands],
        "honest_scope": [
            "accepted on capability + depth evidence only",
            "WSL/Linux rows are portability-repro evidence unless separately paired with real-kernel enforcement",
            "not seccomp proof",
            "not OS-sandbox proof",
            "not Wasmtime fuel proof",
            "not production or v1.0 readiness",
        ],
    }

    _write_text(output_dir / SUMMARY_NAME, json.dumps(data, indent=2) + "\n")
    _write_text(output_dir / MARKDOWN_NAME, render_markdown(data))
    _write_manifest(output_dir)

    if format_ == "md":
        print(render_markdown(data), end="")
    else:
        print(json.dumps(data, indent=2))
    if ok and not verify_bundle(output_dir / SUMMARY_NAME):
        print("ultrapunch-repro: bundle verification failed after write", file=sys.stderr)
        return 1
    return 0 if ok else 1


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--platform", choices=["windows", "linux", "macos"], required=True)
    parser.add_argument("--evidence-tier")
    parser.add_argument("--garnet", required=True, help="Path to the garnet binary or command")
    parser.add_argument(
        "--garnet-arg",
        action="append",
        default=[],
        help="Additional argument inserted after --garnet before the Garnet subcommand; used by tests.",
    )
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    tier = args.evidence_tier
    if not tier:
        tier = "windows-local-repro" if args.platform == "windows" else "portability-repro"
    return record_repro(
        platform=args.platform,
        evidence_tier=tier,
        garnet_cmd=[args.garnet, *args.garnet_arg],
        output_dir=args.output_dir,
        format_=args.format,
    )


if __name__ == "__main__":
    raise SystemExit(main())
