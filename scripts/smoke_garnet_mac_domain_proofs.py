#!/usr/bin/env python3
"""Record the S107 Mac-native S105 domain proof bundle.

This recorder runs the six S105 demonstrator domains through the local Garnet
binary on macOS and writes a manifest-backed committed proof bundle. It records
acceptance as sealed only when the existing `agent-loop` emits the four trust
artifacts. Refusal/report domains intentionally record `sealed=false`; negative
proofs must not fabricate a seal.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import re
import subprocess
import sys
import tempfile
from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "garnet-cli" / "tests" / "fixtures" / "ultrapunch"
SUMMARY_NAME = "garnet-mac-domain-proofs.json"
MARKDOWN_NAME = "garnet-mac-domain-proofs.md"
MANIFEST_NAME = "MANIFEST.sha256"
SCHEMA = "garnet.mac_domain_proofs.v1"
DEFAULT_ROOT = ROOT / "proofs" / "mac" / "domains"
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


def timestamp_slug(now: datetime | None = None) -> str:
    return (now or datetime.now(UTC)).strftime("%Y%m%d-%H%M%S")


def default_output_dir(now: datetime | None = None) -> Path:
    return DEFAULT_ROOT / f"mac-domain-proofs-{timestamp_slug(now)}"


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


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


def _run(
    *,
    command_id: str,
    command: list[str],
    display_args: list[str],
    bundle_dir: Path,
    cwd: Path,
    expected_failure: bool = False,
) -> CommandRecord:
    stdout_rel = Path("commands") / f"{command_id}-stdout.txt"
    stderr_rel = Path("commands") / f"{command_id}-stderr.txt"
    completed = subprocess.run(
        command,
        cwd=cwd,
        text=True,
        encoding="utf-8",
        errors="replace",
        capture_output=True,
        check=False,
    )
    _write_text(bundle_dir / stdout_rel, completed.stdout)
    _write_text(bundle_dir / stderr_rel, completed.stderr)
    ok = completed.returncode != 0 if expected_failure else completed.returncode == 0
    return CommandRecord(
        id=command_id,
        display_args=display_args,
        exit_code=completed.returncode,
        stdout_file=stdout_rel.as_posix(),
        stderr_file=stderr_rel.as_posix(),
        expected_failure=expected_failure,
        status="passed" if ok else "failed",
    )


def _garnet_display(garnet: list[str]) -> list[str]:
    return [Path(garnet[0]).name or garnet[0], *garnet[1:]]


def _read_command_text(bundle_dir: Path, command: CommandRecord) -> str:
    out = (bundle_dir / command.stdout_file).read_text(encoding="utf-8")
    err = (bundle_dir / command.stderr_file).read_text(encoding="utf-8")
    return out + err


def _write_cap_manifest(
    *,
    domain_dir: Path,
    commands: list[CommandRecord],
    garnet: list[str],
    source: Path,
    bundle_dir: Path,
    cwd: Path,
    command_id: str,
) -> bool:
    command = _run(
        command_id=command_id,
        command=[*garnet, "caps", str(source)],
        display_args=[*_garnet_display(garnet), "caps", _repo_relative(source)],
        bundle_dir=bundle_dir,
        cwd=cwd,
    )
    commands.append(command)
    if command.status != "passed":
        return False
    _write_text(domain_dir / "capability_manifest.json", _read_command_text(bundle_dir, command))
    return True


def _agent_loop_command(
    garnet: list[str],
    *,
    proposal: Path,
    record_dir: Path,
    seal_out: Path,
    attest: bool,
) -> list[str]:
    command = [
        *garnet,
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
    garnet: list[str],
    *,
    proposal: Path,
    record_name: str,
    seal_name: str,
    attest: bool,
) -> list[str]:
    display = [
        *_garnet_display(garnet),
        "agent-loop",
        "--baseline",
        "garnet-cli/tests/fixtures/ultrapunch/baseline.garnet",
        "--proposal",
        _repo_relative(proposal),
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


def _sealed(record_dir: Path, seal_out: Path) -> bool:
    return (record_dir / "seal.json").is_file() or seal_out.is_file()


def _domain_base(domain_id: str, label: str, verdict: str) -> dict[str, object]:
    return {
        "id": domain_id,
        "label": label,
        "verdict": verdict,
        "status": "failed",
        "source_included": False,
        "provider_api_called": False,
        "commands": [],
        "artifacts": [],
        "sealed": False,
        "seal_expected": False,
        "honest_scope": [
            "accepted on capability + depth evidence only when explicitly accepted",
            "refusal/report domains must not be sealed",
            "not seccomp proof",
            "not OS-sandbox proof",
            "not Wasmtime fuel proof",
            "not production or v1.0 readiness",
        ],
    }


def _record_agent_loop_domain(
    *,
    domain_id: str,
    label: str,
    proposal: Path,
    expected_failure: bool,
    expected_marker: str,
    verdict: str,
    garnet: list[str],
    bundle_dir: Path,
    cwd: Path,
    attest: bool = False,
) -> dict[str, object]:
    domain_dir = bundle_dir / "domains" / domain_id
    record_dir = domain_dir / "record"
    seal_out = domain_dir / "seal-out.json"
    commands: list[CommandRecord] = []
    data = _domain_base(domain_id, label, verdict)

    _write_cap_manifest(
        domain_dir=domain_dir,
        commands=commands,
        garnet=garnet,
        source=proposal,
        bundle_dir=bundle_dir,
        cwd=cwd,
        command_id=f"{domain_id}-caps",
    )
    loop = _run(
        command_id=f"{domain_id}-agent-loop",
        command=_agent_loop_command(
            garnet,
            proposal=proposal,
            record_dir=record_dir,
            seal_out=seal_out,
            attest=attest,
        ),
        display_args=_agent_loop_display(
            garnet,
            proposal=proposal,
            record_name=f"domains/{domain_id}/record",
            seal_name=f"domains/{domain_id}/seal-out.json",
            attest=attest,
        ),
        bundle_dir=bundle_dir,
        cwd=cwd,
        expected_failure=expected_failure,
    )
    commands.append(loop)
    loop_text = _read_command_text(bundle_dir, loop)
    marker_ok = expected_marker in loop_text
    sealed = _sealed(record_dir, seal_out)
    artifacts = sorted(p.name for p in record_dir.iterdir() if p.is_file()) if record_dir.exists() else []

    if not expected_failure and loop.status == "passed":
        verify = _run(
            command_id=f"{domain_id}-caps-log-verify",
            command=[*garnet, "caps-log", "--verify", str(record_dir / "transparency_log.jsonl")],
            display_args=[*_garnet_display(garnet), "caps-log", "--verify", f"domains/{domain_id}/record/transparency_log.jsonl"],
            bundle_dir=bundle_dir,
            cwd=cwd,
        )
        commands.append(verify)
        marker_ok = marker_ok and verify.status == "passed"
        data["seal_expected"] = True
        ok = (
            sorted(artifacts) == sorted([*ACCEPT_ARTIFACTS, "run_output.txt"])
            and sealed
            and marker_ok
        )
    else:
        ok = loop.status == "passed" and marker_ok and not sealed

    data.update(
        {
            "status": "passed" if ok else "failed",
            "commands": [asdict(command) for command in commands],
            "artifacts": artifacts,
            "sealed": sealed,
            "proposal": _repo_relative(proposal),
        }
    )
    return data


def _record_direct_diff_domain(
    *,
    domain_id: str,
    label: str,
    old: Path,
    new: Path,
    expected_marker: str,
    verdict: str,
    garnet: list[str],
    bundle_dir: Path,
    cwd: Path,
) -> dict[str, object]:
    domain_dir = bundle_dir / "domains" / domain_id
    commands: list[CommandRecord] = []
    data = _domain_base(domain_id, label, verdict)
    _write_cap_manifest(
        domain_dir=domain_dir,
        commands=commands,
        garnet=garnet,
        source=new,
        bundle_dir=bundle_dir,
        cwd=cwd,
        command_id=f"{domain_id}-caps",
    )
    diff = _run(
        command_id=f"{domain_id}-diff-caps",
        command=[*garnet, "diff-caps", str(old), str(new)],
        display_args=[*_garnet_display(garnet), "diff-caps", _repo_relative(old), _repo_relative(new)],
        bundle_dir=bundle_dir,
        cwd=cwd,
        expected_failure=True,
    )
    commands.append(diff)
    diff_text = _read_command_text(bundle_dir, diff)
    _write_text(domain_dir / "diff_caps.txt", diff_text)
    _write_text(
        domain_dir / "decision.md",
        f"# Domain decision: REFUSED\n\n{verdict}. No seal is produced for this refusal.\n",
    )
    ok = diff.status == "passed" and expected_marker in diff_text
    data.update(
        {
            "status": "passed" if ok else "failed",
            "commands": [asdict(command) for command in commands],
            "artifacts": sorted(p.name for p in domain_dir.iterdir() if p.is_file()),
            "sealed": False,
            "old": _repo_relative(old),
            "new": _repo_relative(new),
        }
    )
    return data


def _record_pr_review_domain(*, garnet: list[str], bundle_dir: Path, cwd: Path) -> dict[str, object]:
    before = ROOT / "examples" / "wedge_pr_review" / "before.garnet"
    after = ROOT / "examples" / "wedge_pr_review" / "after.garnet"
    domain_dir = bundle_dir / "domains" / "pr_review_collapse"
    commands: list[CommandRecord] = []
    data = _domain_base(
        "pr_review_collapse",
        "Agent-authored PR-review collapse",
        "diff-caps hard-fails the authority-widening merge gate",
    )
    checks_ok = True
    for label, source in (("before", before), ("after", after)):
        check = _run(
            command_id=f"pr-review-check-{label}",
            command=[*garnet, "check", str(source)],
            display_args=[*_garnet_display(garnet), "check", _repo_relative(source)],
            bundle_dir=bundle_dir,
            cwd=cwd,
        )
        commands.append(check)
        checks_ok = checks_ok and check.status == "passed" and "0 diagnostics" in _read_command_text(bundle_dir, check)
    _write_cap_manifest(
        domain_dir=domain_dir,
        commands=commands,
        garnet=garnet,
        source=after,
        bundle_dir=bundle_dir,
        cwd=cwd,
        command_id="pr-review-caps-after",
    )
    diff = _run(
        command_id="pr-review-diff-caps",
        command=[*garnet, "diff-caps", str(before), str(after)],
        display_args=[*_garnet_display(garnet), "diff-caps", _repo_relative(before), _repo_relative(after)],
        bundle_dir=bundle_dir,
        cwd=cwd,
        expected_failure=True,
    )
    commands.append(diff)
    diff_text = _read_command_text(bundle_dir, diff)
    _write_text(domain_dir / "diff_caps.txt", diff_text)
    _write_text(
        domain_dir / "decision.md",
        "# Domain decision: REFUSED\n\nThe checker stays clean, but `diff-caps` reports an authority expansion. No seal is produced.\n",
    )
    ok = checks_ok and diff.status == "passed" and "AUTHORITY EXPANDED" in diff_text
    data.update(
        {
            "status": "passed" if ok else "failed",
            "commands": [asdict(command) for command in commands],
            "artifacts": sorted(p.name for p in domain_dir.iterdir() if p.is_file()),
            "sealed": False,
            "old": _repo_relative(before),
            "new": _repo_relative(after),
        }
    )
    return data


def _record_mcp_domain(*, garnet: list[str], bundle_dir: Path, cwd: Path) -> dict[str, object]:
    source = ROOT / "examples" / "mcp" / "agent_toolset.mcpcaps"
    domain_dir = bundle_dir / "domains" / "mcp_tool_authority_creep"
    commands: list[CommandRecord] = []
    data = _domain_base(
        "mcp_tool_authority_creep",
        "MCP tool-set authority-creep lens",
        "`mcp-caps` reports high-authority tool declarations; this is a report, not an enforcement trap",
    )
    human = _run(
        command_id="mcp-caps-human",
        command=[*garnet, "mcp-caps", str(source)],
        display_args=[*_garnet_display(garnet), "mcp-caps", _repo_relative(source)],
        bundle_dir=bundle_dir,
        cwd=cwd,
    )
    commands.append(human)
    json_cmd = _run(
        command_id="mcp-caps-json",
        command=[*garnet, "mcp-caps", "--format", "json", str(source)],
        display_args=[*_garnet_display(garnet), "mcp-caps", "--format", "json", _repo_relative(source)],
        bundle_dir=bundle_dir,
        cwd=cwd,
    )
    commands.append(json_cmd)
    human_text = _read_command_text(bundle_dir, human)
    json_text = _read_command_text(bundle_dir, json_cmd)
    _write_text(domain_dir / "mcp_caps.txt", human_text)
    _write_text(domain_dir / "mcp_caps.json", json_text)
    _write_text(
        domain_dir / "decision.md",
        "# Domain decision: REPORTED\n\n`mcp-caps` makes high-authority tool declarations reviewable. No seal or hard-fail is claimed.\n",
    )
    ok = (
        human.status == "passed"
        and json_cmd.status == "passed"
        and "high-authority" in human_text
        and "aggregate authority:" in human_text
        and '"enforced":false' in json_text.replace(" ", "")
    )
    data.update(
        {
            "status": "passed" if ok else "failed",
            "commands": [asdict(command) for command in commands],
            "artifacts": sorted(p.name for p in domain_dir.iterdir() if p.is_file()),
            "sealed": False,
            "source": _repo_relative(source),
            "enforced": False,
        }
    )
    return data


def record_mac_domains(*, garnet: list[str], output_dir: Path, format_: str) -> int:
    output_dir = output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="garnet-mac-domain-proof-") as tmp:
        cwd = Path(tmp)
        domains = [
            _record_agent_loop_domain(
                domain_id="data_pipeline_net_egress",
                label="Data-pipeline net-egress widening",
                proposal=FIXTURES / "reject_widen.garnet",
                expected_failure=True,
                expected_marker="AUTHORITY EXPANDED",
                verdict="capability widening refused at diff-caps; no seal",
                garnet=garnet,
                bundle_dir=output_dir,
                cwd=cwd,
            ),
            _record_direct_diff_domain(
                domain_id="supply_chain_proc_escalation",
                label="Supply-chain installer proc-escalation",
                old=FIXTURES / "supply_chain_base.garnet",
                new=FIXTURES / "supply_chain_proc_escalation.garnet",
                expected_marker="caps GAINED:  proc",
                verdict="declared subprocess authority addition refused by diff-caps; no seal",
                garnet=garnet,
                bundle_dir=output_dir,
                cwd=cwd,
            ),
            _record_agent_loop_domain(
                domain_id="config_processor_depth_trap",
                label="Config processor recursion-depth trap",
                proposal=FIXTURES / "reject_overdepth.garnet",
                expected_failure=True,
                expected_marker="@max_depth(4) exceeded",
                verdict="capability-clean proposal refused by enforced @max_depth trap; no seal",
                garnet=garnet,
                bundle_dir=output_dir,
                cwd=cwd,
            ),
            _record_agent_loop_domain(
                domain_id="accept_provenance_dossier",
                label="Accept-path provenance dossier",
                proposal=FIXTURES / "accept_proposal.garnet",
                expected_failure=False,
                expected_marker="ACCEPTED on capability+depth evidence",
                verdict="accepted on capability + depth evidence; four trust artifacts plus decision emitted",
                garnet=garnet,
                bundle_dir=output_dir,
                cwd=cwd,
                attest=True,
            ),
            _record_pr_review_domain(garnet=garnet, bundle_dir=output_dir, cwd=cwd),
            _record_mcp_domain(garnet=garnet, bundle_dir=output_dir, cwd=cwd),
        ]

    status = "passed" if all(domain["status"] == "passed" for domain in domains) else "failed"
    summary = {
        "schema": SCHEMA,
        "created_at": datetime.now(UTC).isoformat(timespec="seconds"),
        "platform": "macos",
        "host_platform": platform.platform(),
        "arch": platform.machine(),
        "evidence_tier": "macos-native-domain-execution",
        "status": status,
        "source_included": False,
        "provider_api_called": False,
        "domain_count": len(domains),
        "passed_domains": sum(1 for domain in domains if domain["status"] == "passed"),
        "failed_domains": sum(1 for domain in domains if domain["status"] != "passed"),
        "commands_recorded": sum(len(domain["commands"]) for domain in domains),
        "garnet_command": garnet,
        "domains": domains,
        "cross_os_role": "S107 Mac-Codex row for S109 consolidation",
        "honest_scope": [
            "macOS-native execution proof only; not Windows or Linux completion",
            "negative proofs intentionally have no seal",
            "mcp-caps is a static report, not an MCP-host-enforced gate",
            "not seccomp proof",
            "not OS-sandbox proof on macOS",
            "not Wasmtime fuel proof",
            "not production or v1.0 readiness",
        ],
    }
    _write_text(output_dir / SUMMARY_NAME, json.dumps(summary, indent=2, sort_keys=True) + "\n")
    _write_text(output_dir / MARKDOWN_NAME, render_markdown(summary))
    _write_manifest(output_dir)

    verified = verify_bundle(output_dir / SUMMARY_NAME)
    if format_ == "json":
        print(json.dumps(summary, indent=2, sort_keys=True))
    else:
        print(render_markdown(summary), end="")
    if status == "passed" and not verified:
        print("mac-domain-proofs: bundle verification failed after write", file=sys.stderr)
        return 1
    return 0 if status == "passed" else 1


def verify_bundle(summary_path: Path) -> bool:
    try:
        data = json.loads(summary_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return False
    bundle_dir = summary_path.parent
    manifest = _manifest_entries(bundle_dir)
    if manifest is None:
        return False
    required = {SUMMARY_NAME, MARKDOWN_NAME}
    required.update(
        {
            "domains/data_pipeline_net_egress/record/decision.md",
            "domains/supply_chain_proc_escalation/decision.md",
            "domains/config_processor_depth_trap/record/decision.md",
            "domains/accept_provenance_dossier/record/decision.md",
            "domains/pr_review_collapse/decision.md",
            "domains/mcp_tool_authority_creep/decision.md",
        }
    )
    if not required.issubset(manifest):
        return False
    if data.get("schema") != SCHEMA or data.get("status") != "passed":
        return False
    if data.get("platform") != "macos" or data.get("source_included") is not False:
        return False
    if data.get("provider_api_called") is not False or data.get("domain_count") != 6:
        return False
    domains = data.get("domains")
    if not isinstance(domains, list) or len(domains) != 6:
        return False
    by_id = {domain.get("id"): domain for domain in domains if isinstance(domain, dict)}
    if set(by_id) != {
        "data_pipeline_net_egress",
        "supply_chain_proc_escalation",
        "config_processor_depth_trap",
        "accept_provenance_dossier",
        "pr_review_collapse",
        "mcp_tool_authority_creep",
    }:
        return False
    for domain in by_id.values():
        if domain.get("status") != "passed":
            return False
        commands = domain.get("commands")
        if not isinstance(commands, list) or not commands:
            return False
        for command in commands:
            if not isinstance(command, dict) or command.get("status") != "passed":
                return False
            if command.get("stdout_file") not in manifest or command.get("stderr_file") not in manifest:
                return False
    accept = by_id["accept_provenance_dossier"]
    if accept.get("sealed") is not True or accept.get("seal_expected") is not True:
        return False
    if not set(ACCEPT_ARTIFACTS).issubset(set(accept.get("artifacts", []))):
        return False
    for domain_id in set(by_id) - {"accept_provenance_dossier"}:
        if by_id[domain_id].get("sealed") is not False:
            return False
    mcp = by_id["mcp_tool_authority_creep"]
    if mcp.get("enforced") is not False:
        return False
    return True


def render_markdown(data: dict[str, object]) -> str:
    lines = [
        "# Garnet Mac Domain Proofs",
        "",
        f"- Status: `{data.get('status')}`",
        f"- Platform: `{data.get('platform')} {data.get('arch')}`",
        f"- Domains: `{data.get('passed_domains')}/{data.get('domain_count')}`",
        f"- Commands recorded: `{data.get('commands_recorded')}`",
        f"- Cross-OS role: `{data.get('cross_os_role')}`",
        "",
        "| Domain | Status | Sealed | Verdict |",
        "| --- | --- | --- | --- |",
    ]
    for domain in data.get("domains", []):
        if not isinstance(domain, dict):
            continue
        lines.append(
            f"| `{domain.get('id')}` | `{domain.get('status')}` | "
            f"`{str(domain.get('sealed')).lower()}` | {domain.get('verdict')} |"
        )
    lines.extend(
        [
            "",
            "## Honest Scope",
            "",
            "- Mac row only; Windows/Linux completion waits for their committed rows.",
            "- Negative proofs have no seal by design.",
            "- `mcp-caps` is static report evidence, not MCP-host enforcement.",
            "- No seccomp, OS-sandbox, Wasmtime fuel, production, or v1.0 claim is made.",
            "",
        ]
    )
    return "\n".join(lines)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--garnet", required=True, help="Path to the Garnet CLI binary")
    parser.add_argument("--output-dir", type=Path, default=None)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument("--verify", type=Path, help="Verify an existing summary JSON instead of recording")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    if args.verify:
        ok = verify_bundle(args.verify)
        print("mac-domain-proofs: verified" if ok else "mac-domain-proofs: verification FAILED")
        return 0 if ok else 1
    output_dir = args.output_dir or default_output_dir()
    garnet = str(Path(args.garnet).resolve()) if Path(args.garnet).exists() else args.garnet
    return record_mac_domains(garnet=[garnet], output_dir=output_dir, format_=args.format)


if __name__ == "__main__":
    raise SystemExit(main())
