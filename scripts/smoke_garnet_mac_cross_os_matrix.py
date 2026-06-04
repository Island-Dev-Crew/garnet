#!/usr/bin/env python3
"""Record the S109 Mac row of the cross-OS trap matrix.

This is a Mac-row consolidation proof, not the full S109 completion claim. It
compares the committed Windows/WSL baselines with the committed Mac S107 domain
proof, then runs the Mac Stage-V trap gates locally. WSL remains
execution/portability evidence only; an independent Linux enforcement row is
still required before the matrix is cross-OS-complete.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import garnet_windows_cross_os_enforcement_proof  # noqa: E402
import smoke_garnet_mac_domain_proofs  # noqa: E402

SCHEMA = "garnet.mac_cross_os_matrix.v1"
SUMMARY_NAME = "garnet-mac-cross-os-matrix.json"
MARKDOWN_NAME = "garnet-mac-cross-os-matrix.md"
MANIFEST_NAME = "MANIFEST.sha256"
DEFAULT_ROOT = ROOT / "proofs" / "mac" / "matrix"
WINDOWS_ULTRAPUNCH_ROOT = ROOT / "proofs" / "windows" / "ultrapunch"
MAC_DOMAIN_ROOT = ROOT / "proofs" / "mac" / "domains"
REQUIRED_TRAPS = ["max_depth", "caps", "diff_caps_reject"]


@dataclass(frozen=True)
class CommandRecord:
    id: str
    display_args: list[str]
    exit_code: int
    stdout_file: str
    stderr_file: str
    status: str


def timestamp_slug(now: datetime | None = None) -> str:
    return (now or datetime.now(UTC)).strftime("%Y%m%d-%H%M%S")


def default_output_dir(now: datetime | None = None) -> Path:
    return DEFAULT_ROOT / f"mac-cross-os-matrix-{timestamp_slug(now)}"


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
        match = re.fullmatch(r"([0-9a-f]{64})  ([^\r\n]+)", line)
        if not match:
            return None
        digest, relative = match.groups()
        target = bundle_dir / Path(relative)
        if not target.is_file() or _sha256(target) != digest:
            return None
        entries[relative.replace("\\", "/")] = digest
    return entries


def _run_command(command_id: str, command: list[str], bundle_dir: Path) -> CommandRecord:
    stdout_rel = Path("commands") / f"{command_id}-stdout.txt"
    stderr_rel = Path("commands") / f"{command_id}-stderr.txt"
    completed = subprocess.run(
        command,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        capture_output=True,
        check=False,
    )
    _write_text(bundle_dir / stdout_rel, completed.stdout)
    _write_text(bundle_dir / stderr_rel, completed.stderr)
    return CommandRecord(
        id=command_id,
        display_args=command,
        exit_code=completed.returncode,
        stdout_file=stdout_rel.as_posix(),
        stderr_file=stderr_rel.as_posix(),
        status="passed" if completed.returncode == 0 else "failed",
    )


def _mac_stage_v_commands(bundle_dir: Path) -> list[CommandRecord]:
    return [
        _run_command(
            "mac-s101-gate",
            [
                sys.executable,
                "scripts/garnet_vm_interp_enforcement_parity_status.py",
                "--gate",
                "--format",
                "json",
            ],
            bundle_dir,
        ),
        _run_command(
            "mac-bounded-enforcement",
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
            bundle_dir,
        ),
        _run_command(
            "mac-caps-enforcement",
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
            bundle_dir,
        ),
    ]


def _latest_verified_mac_domain(root: Path) -> Path | None:
    if not root.exists():
        return None
    candidates = sorted(
        root.rglob(smoke_garnet_mac_domain_proofs.SUMMARY_NAME),
        key=lambda path: path.stat().st_mtime,
        reverse=True,
    )
    for candidate in candidates:
        if smoke_garnet_mac_domain_proofs.verify_bundle(candidate):
            return candidate
    return None


def _latest_windows_ultrapunch(root: Path) -> Path | None:
    if not root.exists():
        return None
    candidates = sorted(
        root.rglob("garnet-ultrapunch-repro.json"),
        key=lambda path: path.stat().st_mtime,
        reverse=True,
    )
    for candidate in candidates:
        if _windows_ultrapunch_verified(candidate):
            return candidate
    return None


def _windows_ultrapunch_verified(summary: Path) -> bool:
    try:
        data = json.loads(summary.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return False
    bundle = summary.parent
    return (
        data.get("schema") == "garnet.ultrapunch.repro.v1"
        and data.get("platform") == "windows"
        and data.get("status") == "passed"
        and (bundle / "accept" / "capability_manifest.json").is_file()
        and (bundle / "accept" / "transparency_log.jsonl").is_file()
        and (bundle / "accept" / "seal.json").is_file()
        and (bundle / "reject-widen" / "decision.md").is_file()
        and (bundle / "reject-overdepth" / "run_trap.txt").is_file()
        and not (bundle / "reject-widen" / "seal.json").is_file()
    )


def _load_json(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def _text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def _bytes_equal(left: Path, right: Path) -> bool:
    return left.read_bytes() == right.read_bytes()


def _accept_domain(mac_data: dict[str, object]) -> dict[str, object] | None:
    domains = mac_data.get("domains")
    if not isinstance(domains, list):
        return None
    for domain in domains:
        if isinstance(domain, dict) and domain.get("id") == "accept_provenance_dossier":
            return domain
    return None


def _domain_by_id(mac_data: dict[str, object], domain_id: str) -> dict[str, object] | None:
    domains = mac_data.get("domains")
    if not isinstance(domains, list):
        return None
    for domain in domains:
        if isinstance(domain, dict) and domain.get("id") == domain_id:
            return domain
    return None


def _seal_field_comparison(windows_seal: Path, mac_seal: Path) -> dict[str, object]:
    left = _load_json(windows_seal)
    right = _load_json(mac_seal)
    left_pred = left.get("predicate", {})
    right_pred = right.get("predicate", {})
    if not isinstance(left_pred, dict) or not isinstance(right_pred, dict):
        return {"status": "failed", "reason": "seal predicate missing"}
    left_manifest = left_pred.get("build_manifest", {})
    right_manifest = right_pred.get("build_manifest", {})
    if not isinstance(left_manifest, dict) or not isinstance(right_manifest, dict):
        return {"status": "failed", "reason": "build manifest missing"}
    fields = {
        "source_blake3": left_pred.get("source_blake3") == right_pred.get("source_blake3"),
        "source_hash": left_manifest.get("source_hash") == right_manifest.get("source_hash"),
        "ast_hash": left_manifest.get("ast_hash") == right_manifest.get("ast_hash"),
        "capability_manifest": left_pred.get("capability_manifest")
        == right_pred.get("capability_manifest"),
        "attestation": left_pred.get("attestation") == right_pred.get("attestation"),
    }
    full_equal = _bytes_equal(windows_seal, mac_seal)
    return {
        "status": "passed" if all(fields.values()) else "failed",
        "full_json_byte_equal": full_equal,
        "field_equal": fields,
        "windows_prelude_hash": left_manifest.get("prelude_hash"),
        "mac_prelude_hash": right_manifest.get("prelude_hash"),
        "delta": (
            "Full seal JSON differs because the prelude_hash field differs across "
            "the older Windows baseline checkout and the current Mac proof; the "
            "OS-independent subject, AST, capability manifest, and attestation fields match."
            if not full_equal
            else "Full seal JSON is byte-identical."
        ),
    }


def _diff_caps_tail_equal(windows_diff: Path, mac_diff: Path) -> bool:
    def body(path: Path) -> str:
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
        return "\n".join(lines[1:]).strip()

    return body(windows_diff) == body(mac_diff)


def _byte_comparisons(windows_ultrapunch: Path, mac_domain: Path) -> list[dict[str, object]]:
    win_bundle = windows_ultrapunch.parent
    mac_bundle = mac_domain.parent
    win_accept = win_bundle / "accept"
    mac_accept = mac_bundle / "domains" / "accept_provenance_dossier" / "record"
    comparisons = [
        {
            "id": "accept_capability_manifest",
            "expected_os_independent": True,
            "windows_path": _repo_relative(win_accept / "capability_manifest.json"),
            "mac_path": _repo_relative(mac_accept / "capability_manifest.json"),
            "byte_equal": _bytes_equal(
                win_accept / "capability_manifest.json",
                mac_accept / "capability_manifest.json",
            ),
            "delta": "Must be byte-identical; declared surface is OS-independent.",
        },
        {
            "id": "accept_transparency_log",
            "expected_os_independent": True,
            "windows_path": _repo_relative(win_accept / "transparency_log.jsonl"),
            "mac_path": _repo_relative(mac_accept / "transparency_log.jsonl"),
            "byte_equal": _bytes_equal(
                win_accept / "transparency_log.jsonl",
                mac_accept / "transparency_log.jsonl",
            ),
            "delta": "Must be byte-identical for a one-entry local chain over the same accepted proposal.",
        },
        {
            "id": "accept_diff_caps",
            "expected_os_independent": False,
            "windows_path": _repo_relative(win_accept / "diff_caps.txt"),
            "mac_path": _repo_relative(mac_accept / "diff_caps.txt"),
            "byte_equal": _bytes_equal(win_accept / "diff_caps.txt", mac_accept / "diff_caps.txt"),
            "normalized_body_equal": _diff_caps_tail_equal(
                win_accept / "diff_caps.txt", mac_accept / "diff_caps.txt"
            ),
            "delta": "Full text includes absolute OS paths; the path-independent verdict body must match.",
        },
    ]
    seal = _seal_field_comparison(win_accept / "seal.json", mac_accept / "seal.json")
    comparisons.append(
        {
            "id": "accept_seal",
            "expected_os_independent": False,
            "windows_path": _repo_relative(win_accept / "seal.json"),
            "mac_path": _repo_relative(mac_accept / "seal.json"),
            **seal,
        }
    )
    return comparisons


def _trap_rows(
    *,
    windows_status: garnet_windows_cross_os_enforcement_proof.S106Status,
    windows_ultrapunch: Path,
    mac_data: dict[str, object],
    commands: list[CommandRecord],
) -> list[dict[str, object]]:
    command_status = {command.id: command.status == "passed" for command in commands}
    windows_commands = {
        command.get("name"): int(command.get("exit_code", 1)) == 0
        for command in windows_status.windows.commands
    }
    win_bundle = windows_ultrapunch.parent
    rows = [
        {
            "trap": "max_depth",
            "windows": {
                "status": windows_status.windows.ok and windows_commands.get("bounded_enforcement", False),
                "evidence": _repo_relative(ROOT / "proofs" / "windows" / "enforcement" / "bounded_enforcement.stdout.log"),
            },
            "mac": {
                "status": command_status.get("mac-bounded-enforcement", False)
                and (_domain_by_id(mac_data, "config_processor_depth_trap") or {}).get("status") == "passed",
                "evidence": _repo_relative(
                    MAC_DOMAIN_ROOT
                    / mac_data.get("_bundle_name", "")
                    / "domains"
                    / "config_processor_depth_trap"
                    / "record"
                    / "run_trap.txt"
                ),
            },
            "wsl": {
                "status": windows_status.wsl.ok,
                "tier": windows_status.wsl.tier,
                "honest_scope": "execution/portability only, not Linux enforcement",
            },
        },
        {
            "trap": "caps",
            "windows": {
                "status": windows_status.windows.ok and windows_commands.get("caps_enforcement", False),
                "evidence": _repo_relative(ROOT / "proofs" / "windows" / "enforcement" / "caps_enforcement.stdout.log"),
            },
            "mac": {
                "status": command_status.get("mac-caps-enforcement", False)
                and (_domain_by_id(mac_data, "data_pipeline_net_egress") or {}).get("status") == "passed",
                "evidence": "mac-bounded/caps test logs plus S107 capability-widening refusal",
            },
            "wsl": {
                "status": windows_status.wsl.ok,
                "tier": windows_status.wsl.tier,
                "honest_scope": "execution/portability only, not Linux enforcement",
            },
        },
        {
            "trap": "diff_caps_reject",
            "windows": {
                "status": (win_bundle / "reject-widen" / "decision.md").is_file()
                and not (win_bundle / "reject-widen" / "seal.json").exists(),
                "evidence": _repo_relative(win_bundle / "reject-widen" / "decision.md"),
            },
            "mac": {
                "status": all(
                    (_domain_by_id(mac_data, domain_id) or {}).get("status") == "passed"
                    and (_domain_by_id(mac_data, domain_id) or {}).get("sealed") is False
                    for domain_id in (
                        "data_pipeline_net_egress",
                        "supply_chain_proc_escalation",
                        "pr_review_collapse",
                    )
                ),
                "evidence": "S107 Mac refused net/proc/PR-review diff-caps widening domains with no seal",
            },
            "wsl": {
                "status": windows_status.wsl.ok,
                "tier": windows_status.wsl.tier,
                "honest_scope": "execution/portability only, not Linux enforcement",
            },
        },
    ]
    for row in rows:
        row["status"] = (
            "passed"
            if row["windows"]["status"] and row["mac"]["status"] and row["wsl"]["status"]
            else "failed"
        )
    return rows


def build_summary(
    *,
    root: Path,
    bundle_dir: Path,
    commands: list[CommandRecord],
) -> dict[str, object]:
    windows_status = garnet_windows_cross_os_enforcement_proof.read_status(root)
    windows_ultrapunch = _latest_windows_ultrapunch(root / "proofs" / "windows" / "ultrapunch")
    mac_domain = _latest_verified_mac_domain(root / "proofs" / "mac" / "domains")
    if windows_ultrapunch is None:
        raise RuntimeError("missing verified Windows ultrapunch baseline under proofs/windows/ultrapunch")
    if mac_domain is None:
        raise RuntimeError("missing verified Mac S107 domain proof under proofs/mac/domains")

    mac_data = _load_json(mac_domain)
    mac_data["_bundle_name"] = mac_domain.parent.name
    comparisons = _byte_comparisons(windows_ultrapunch, mac_domain)
    rows = _trap_rows(
        windows_status=windows_status,
        windows_ultrapunch=windows_ultrapunch,
        mac_data=mac_data,
        commands=commands,
    )
    mac_rows_complete = all(row["status"] == "passed" for row in rows)
    required_byte_comparisons = [
        comparison for comparison in comparisons if comparison.get("expected_os_independent") is True
    ]
    required_byte_ok = all(comparison.get("byte_equal") is True for comparison in required_byte_comparisons)
    command_ok = all(command.status == "passed" for command in commands)
    ok = windows_status.ok and command_ok and mac_rows_complete and required_byte_ok
    return {
        "schema": SCHEMA,
        "created_at": datetime.now(UTC).isoformat(timespec="seconds"),
        "platform": "macos",
        "status": "passed" if ok else "failed",
        "mac_rows_complete": mac_rows_complete,
        "cross_os_complete": False,
        "cross_os_complete_reason": (
            "Independent Linux S108 enforcement row is not present; WSL is recorded "
            "as execution/portability only and is not Linux seccomp or OS-sandbox enforcement."
        ),
        "windows_baseline": _repo_relative(
            root / "proofs/windows/enforcement/windows-enforcement-proof.json"
        ),
        "windows_ultrapunch_baseline": _repo_relative(windows_ultrapunch),
        "mac_domain_baseline": _repo_relative(mac_domain),
        "commands": [asdict(command) for command in commands],
        "trap_rows": rows,
        "byte_comparisons": comparisons,
        "honest_scope": [
            "This is the Mac row for S109 consolidation, not full S109 completion.",
            "WSL remains execution/portability evidence only, not Linux seccomp enforcement.",
            "No macOS OS-sandbox enforcement, Wasmtime fuel, production, or v1.0 claim is made.",
            "Only explicitly marked OS-independent artifacts are required to be byte-identical.",
        ],
    }


def record_matrix(*, output_dir: Path, format_: str) -> int:
    output_dir.mkdir(parents=True, exist_ok=True)
    commands = _mac_stage_v_commands(output_dir)
    summary = build_summary(root=ROOT, bundle_dir=output_dir, commands=commands)
    _write_text(output_dir / SUMMARY_NAME, json.dumps(summary, indent=2, sort_keys=True) + "\n")
    _write_text(output_dir / MARKDOWN_NAME, render_markdown(summary))
    _write_manifest(output_dir)
    verified = verify_bundle(output_dir / SUMMARY_NAME)
    if format_ == "json":
        print(json.dumps(summary, indent=2, sort_keys=True))
    else:
        print(render_markdown(summary), end="")
    if summary["status"] == "passed" and not verified:
        print("mac-cross-os-matrix: bundle verification failed after write", file=sys.stderr)
        return 1
    return 0 if summary["status"] == "passed" else 1


def verify_bundle(summary_path: Path) -> bool:
    try:
        data = json.loads(summary_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return False
    bundle_dir = summary_path.parent
    manifest = _manifest_entries(bundle_dir)
    if manifest is None:
        return False
    if SUMMARY_NAME not in manifest or MARKDOWN_NAME not in manifest:
        return False
    if data.get("schema") != SCHEMA or data.get("platform") != "macos":
        return False
    if data.get("status") != "passed" or data.get("mac_rows_complete") is not True:
        return False
    if data.get("cross_os_complete") is not False:
        return False
    scope = " ".join(data.get("honest_scope", []))
    if "not Linux seccomp" not in scope or "not full S109 completion" not in scope:
        return False
    commands = data.get("commands")
    if not isinstance(commands, list) or len(commands) < 3:
        return False
    for command in commands:
        if not isinstance(command, dict) or command.get("status") != "passed":
            return False
        if command.get("stdout_file") not in manifest or command.get("stderr_file") not in manifest:
            return False
    rows = data.get("trap_rows")
    if not isinstance(rows, list) or {row.get("trap") for row in rows if isinstance(row, dict)} != set(REQUIRED_TRAPS):
        return False
    if any(row.get("status") != "passed" for row in rows if isinstance(row, dict)):
        return False
    comparisons = data.get("byte_comparisons")
    if not isinstance(comparisons, list):
        return False
    by_id = {item.get("id"): item for item in comparisons if isinstance(item, dict)}
    for required in ("accept_capability_manifest", "accept_transparency_log"):
        if by_id.get(required, {}).get("byte_equal") is not True:
            return False
    seal = by_id.get("accept_seal", {})
    if seal.get("full_json_byte_equal") is True:
        return False
    if seal.get("status") != "passed" or "prelude_hash" not in str(seal.get("delta", "")):
        return False
    diff_caps = by_id.get("accept_diff_caps", {})
    if diff_caps.get("byte_equal") is True or diff_caps.get("normalized_body_equal") is not True:
        return False
    return True


def render_markdown(data: dict[str, object]) -> str:
    lines = [
        "# Garnet Mac Cross-OS Matrix Row",
        "",
        f"- Status: `{data.get('status')}`",
        f"- Mac rows complete: `{str(data.get('mac_rows_complete')).lower()}`",
        f"- Cross-OS complete: `{str(data.get('cross_os_complete')).lower()}`",
        f"- Reason: {data.get('cross_os_complete_reason')}",
        "",
        "## Trap Rows",
        "",
        "| Trap | Status | Windows | Mac | WSL |",
        "| --- | --- | --- | --- | --- |",
    ]
    for row in data.get("trap_rows", []):
        if not isinstance(row, dict):
            continue
        lines.append(
            f"| `{row.get('trap')}` | `{row.get('status')}` | "
            f"`{str(row.get('windows', {}).get('status')).lower()}` | "
            f"`{str(row.get('mac', {}).get('status')).lower()}` | "
            f"`{row.get('wsl', {}).get('tier')}` |"
        )
    lines.extend(["", "## Byte Comparisons", "", "| Artifact | Byte Equal | Delta |", "| --- | --- | --- |"])
    for comparison in data.get("byte_comparisons", []):
        if not isinstance(comparison, dict):
            continue
        lines.append(
            f"| `{comparison.get('id')}` | `{str(comparison.get('byte_equal')).lower()}` | "
            f"{comparison.get('delta', '')} |"
        )
    lines.extend(
        [
            "",
            "## Honest Scope",
            "",
            "- This is the Mac row for S109 consolidation, not full S109 completion.",
            "- WSL remains execution/portability only, not Linux seccomp or OS-sandbox enforcement.",
            "- No macOS OS-sandbox enforcement, Wasmtime fuel, production, or v1.0 claim is made.",
            "",
        ]
    )
    return "\n".join(lines)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, default=None)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument("--verify", type=Path, help="Verify an existing summary JSON instead of recording")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    if args.verify:
        ok = verify_bundle(args.verify)
        print("mac-cross-os-matrix: verified" if ok else "mac-cross-os-matrix: verification FAILED")
        return 0 if ok else 1
    return record_matrix(output_dir=args.output_dir or default_output_dir(), format_=args.format)


if __name__ == "__main__":
    raise SystemExit(main())
