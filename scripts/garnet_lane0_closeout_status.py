#!/usr/bin/env python3
"""Fail-closed Lane 0 closeout, evidence-manifest, and ledger-chain verifier.

The gate is intentionally repository-local. It verifies the namespaced
ARCHIPELAGO state and ledger, the exact Lane 0 evidence set, the four frozen
readiness denominators, launch HOLD, and the advisory S6 governance verdict.
It never reads a fork branch, ambient credentials, environment variables, or
remote GitHub state.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import stat
import subprocess
import sys
from dataclasses import asdict, dataclass
from datetime import datetime, timedelta, timezone
from decimal import Decimal
from pathlib import Path, PurePosixPath


ROOT = Path(__file__).resolve().parents[1]
GENESIS_HASH = "0" * 64
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
GIT_OID_RE = re.compile(r"^[0-9a-f]{40}$")
UTC_RE = re.compile(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$")

EXPECTED_EVIDENCE_FILES = (
    "00-environment.json",
    "01-archipelago-contracts.txt",
    "02-first-parent-archive.txt",
    "03-successor-pin-delta.txt",
    "04-truth-freeze.json",
    "05-msrv.json",
    "06-frozen-backlog.json",
    "07-quarterly-watch.json",
    "08-launch-readiness.json",
    "09-mit-readiness.json",
    "10-denominators.json",
    "11-truth-check.txt",
    "12-repository-evidence-integrity.json",
    "13-wv6-pending.json",
    "14-wv7-pending.json",
    "20-python-tests.txt",
    "21-rust-msrv-checks.txt",
    "22-workspace-tests.txt",
    "23-sotu-render.txt",
    "24-pr-body-validation.txt",
    "25-independent-review.md",
    "COMMANDS.json",
)

EXPECTED_DENOMINATORS = (
    {
        "id": "s114_mission",
        "label": "S114 bounded mission",
        "numerator": 19,
        "denominator": 19,
        "percent": 100.0,
        "evidence": "ops/lane0/evidence/10-denominators.json",
    },
    {
        "id": "truth_pulse",
        "label": "Truth pulse",
        "numerator": 65.2,
        "denominator": 70,
        "percent": 93.1,
        "rounded": True,
        "evidence": "ops/lane0/evidence/09-mit-readiness.json",
    },
    {
        "id": "launch_critical",
        "label": "Launch-critical",
        "numerator": 3,
        "denominator": 6,
        "percent": 50.0,
        "evidence": "ops/lane0/evidence/08-launch-readiness.json",
    },
    {
        "id": "launch_ledger",
        "label": "Whole launch ledger",
        "numerator": 3,
        "denominator": 8,
        "percent": 37.5,
        "evidence": "ops/lane0/evidence/08-launch-readiness.json",
    },
)

EXPECTED_NEXT_ACTIONS = (
    "Lane 1: prove fresh, exact-reviewed-head, outcome-verified GOV-009 state and close the live settings/no-bypass readback only after Jon provisions a dedicated admin-authoritative token.",
    "Lane 2A: materialize the hermetic package, connect the live adapter/page, run Playwright traps, promote only from reporter evidence, and prove fail-closed denial.",
    "Lane 2B: add one bounded in-process Garnet tool, raw-byte stdio, a sealed baseline, reject-without-seal proof, and a deterministic Shelf reporter.",
    "Lane 2C: rerun and deterministically report three exact-candidate stress cases exceeding four minutes before restoring APPROVED; current state remains partial.",
)

EXPECTED_LANE0_REVIEW_BOUNDARY = {
    "reviewed_head": "3124ba5ecfa88aa6f2c2c289313860670673cdec",
    "reviewed_head_tree": "d2d3c735cf25b84ef69e0e385c8cfeb35e1af673",
    "reviewed_tree": "98141597d17e13b02cfa228c03cdf0dc2119ad9f",
    "merged_commit": "aa681bacd2e437bfde3cea0ffc1ca75bdb134aac",
    "review_scope": (
        "Independent review ended at reviewed_head. reviewed_tree binds the "
        "final squash content and does not extend or backdate independent "
        "review coverage."
    ),
    "post_review_commits": [
        {
            "commit": "aa14368bde83391506775d835ace8985bb7bc1ed",
            "reviewed": False,
            "purpose": (
                "Final Lane 0 evidence recapture and closeout-state sealing "
                "after the independent review."
            ),
        },
        {
            "commit": "5680fbed4684d57fc3773a1f75f86868c44b7a95",
            "reviewed": False,
            "purpose": (
                "Lane 0 trust-kernel review companion added after the "
                "independent review."
            ),
        },
    ],
}

LANE0_BASE_COMMIT = "231aefa91985e5a0520c493c7f0fc3e54d74efc8"
EXPECTED_LANDED_CHANGED_PATH_COUNT = 87
EXPECTED_POST_REVIEW_ONLY_PATHS = (
    "F_Project_Management/W_TRUST/LANE0_TRUST_KERNEL_REVIEW_2026-07-16.md",
)

EXPECTED_GATE_EVIDENCE = {
    "P0-G1": "20-python-tests.txt",
    "P0-G2": "04-truth-freeze.json",
    "P1-G1": "20-python-tests.txt",
    "P1-G2": "05-msrv.json",
    "P2-G1": "06-frozen-backlog.json",
    "P2-G2": "20-python-tests.txt",
    "P3-G1": "11-truth-check.txt",
    "P3-G2": "12-repository-evidence-integrity.json",
    "P3-G3": "23-sotu-render.txt",
}

EXPECTED_GATE_COMMANDS = {
    "P0-G1": "python3 -I scripts/test_garnet_lane0_truth_freeze_status.py -v",
    "P0-G2": "python3 -I scripts/garnet_lane0_truth_freeze_status.py --gate",
    "P1-G1": "python3 -I -S scripts/test_garnet_msrv_status.py -v",
    "P1-G2": "python3 -I -S scripts/garnet_msrv_status.py --gate",
    "P2-G1": "python3 -I scripts/garnet_frozen_backlog_status.py --gate",
    "P2-G2": "python3 scripts/check-agent-contracts.py",
    "P3-G1": "cargo run -p xtask -- truth --check",
    "P3-G2": "python3 scripts/garnet_evidence_integrity_status.py --format json --gate",
    "P3-G3": "node ops/mission/render-sotu.mjs",
}

EXPECTED_LAUNCH_GATES = (
    ("foundation_integrity", "pass"),
    ("native_linux", "pass"),
    ("s114_acceptance", "accepted-scoped"),
    ("static_playground", "partial"),
    ("live_wasm_playground", "remaining"),
    ("minimum_sealed_shelf", "manual-deferred"),
    ("promo_video", "pending-human"),
    ("launch_fire", "jon-only"),
)

MIT_STATUS_SCORES = {
    "verified": Decimal("1"),
    "active-partial": Decimal("0.5"),
    "rendered-artifact-ready": Decimal("0.65"),
    "visual-qa-ready": Decimal("0.8"),
    "website-export-ready": Decimal("0.9"),
    "public-site-embedded": Decimal("0.95"),
    "composition-ready": Decimal("0.5"),
    "source-locked": Decimal("0.35"),
    "planned-contract": Decimal("0.25"),
    "source-present": Decimal("0.6"),
    "local-registry-source-ready": Decimal("0.85"),
    "feature-gated-source-ready": Decimal("0.85"),
    "provider-gated-harness": Decimal("1"),
    "provider-gated-5k-harness": Decimal("1"),
    "blocked": Decimal("0"),
    "planned": Decimal("0"),
}

ARCHIPELAGO_TOOL = "/private/tmp/archipelago-lane0-tooling-20260715"
EXPECTED_COMMANDS = (
    ("python3 --version", 0, "00-environment.json"),
    ("rustc +1.95.0 --version", 0, "00-environment.json"),
    ("cargo +1.95.0 --version", 0, "00-environment.json"),
    ("node --version", 0, "00-environment.json"),
    ("npm --version", 0, "00-environment.json"),
    ("git --version", 0, "00-environment.json"),
    (
        f"git -C {ARCHIPELAGO_TOOL} rev-parse HEAD",
        0,
        "01-archipelago-contracts.txt",
    ),
    (
        f"python3 {ARCHIPELAGO_TOOL}/scripts/validate_contracts.py ops/lane0/idea.lock.json",
        0,
        "01-archipelago-contracts.txt",
    ),
    (
        f"python3 {ARCHIPELAGO_TOOL}/scripts/validate_contracts.py ops/lane0/plan.lock.json",
        0,
        "01-archipelago-contracts.txt",
    ),
    (
        f"python3 {ARCHIPELAGO_TOOL}/scripts/validate_contracts.py ops/lane0/state.json",
        0,
        "01-archipelago-contracts.txt",
    ),
    (
        "git log --oneline --first-parent d0d4f7cc..1fe7489",
        0,
        "02-first-parent-archive.txt",
    ),
    (
        "git log --oneline --first-parent d0d4f7cc..1fe7489 | wc -l",
        0,
        "02-first-parent-archive.txt",
    ),
    (
        "git log --oneline --first-parent 1fe7489..231aefa",
        0,
        "03-successor-pin-delta.txt",
    ),
    (
        "git rev-list --count --first-parent 1fe7489..231aefa",
        0,
        "03-successor-pin-delta.txt",
    ),
    (
        "git merge-base --is-ancestor 1fe74892c588f912e103742afc9d11e845e8d4e6 231aefa91985e5a0520c493c7f0fc3e54d74efc8",
        0,
        "03-successor-pin-delta.txt",
    ),
    (
        "python3 -I scripts/garnet_lane0_truth_freeze_status.py --gate",
        0,
        "04-truth-freeze.json",
    ),
    (
        "python3 -I -S scripts/garnet_msrv_status.py --gate",
        0,
        "05-msrv.json",
    ),
    (
        "python3 -I scripts/garnet_frozen_backlog_status.py --gate",
        0,
        "06-frozen-backlog.json",
    ),
    (
        "python3 -I scripts/garnet_quarterly_competitive_watch_status.py --as-of 2026-07-16 --gate",
        0,
        "07-quarterly-watch.json",
    ),
    (
        "python3 scripts/garnet_launch_readiness_status.py --format json --gate",
        1,
        "08-launch-readiness.json",
    ),
    (
        "python3 scripts/garnet_mit_readiness_status.py --committed-only --format json",
        0,
        "09-mit-readiness.json",
    ),
    (
        "python3 -I scripts/garnet_lane0_closeout_status.py --write-denominators",
        0,
        "10-denominators.json",
    ),
    ("cargo run -p xtask -- truth --check", 0, "11-truth-check.txt"),
    (
        "python3 scripts/garnet_evidence_integrity_status.py --format json --gate",
        0,
        "12-repository-evidence-integrity.json",
    ),
    (
        "python3 -I scripts/garnet_wv_acceptance_status.py --wv WV-6 --gate",
        1,
        "13-wv6-pending.json",
    ),
    (
        "python3 -I scripts/garnet_wv_acceptance_status.py --wv WV-7 --gate",
        1,
        "14-wv7-pending.json",
    ),
    (
        "python3 -I -S scripts/test_garnet_lane0_closeout_status.py -v",
        0,
        "20-python-tests.txt",
    ),
    (
        "python3 -I scripts/test_garnet_lane0_truth_freeze_status.py -v",
        0,
        "20-python-tests.txt",
    ),
    (
        "python3 -I -S scripts/test_garnet_msrv_status.py -v",
        0,
        "20-python-tests.txt",
    ),
    (
        "python3 -I scripts/test_garnet_frozen_backlog_status.py -v",
        0,
        "20-python-tests.txt",
    ),
    (
        "python3 -I scripts/test_garnet_quarterly_competitive_watch_status.py -v",
        0,
        "20-python-tests.txt",
    ),
    (
        "python3 -I scripts/test_garnet_wv_acceptance_status.py -v",
        0,
        "20-python-tests.txt",
    ),
    (
        "python3 scripts/test_garnet_launch_readiness_status.py",
        0,
        "20-python-tests.txt",
    ),
    ("python3 scripts/check-agent-contracts.py", 0, "20-python-tests.txt"),
    ("python3 scripts/test_check_agent_contracts.py", 0, "20-python-tests.txt"),
    (
        "cargo +1.95.0 check --workspace --all-targets --all-features --locked",
        0,
        "21-rust-msrv-checks.txt",
    ),
    (
        "cargo +1.95.0 test --workspace --no-fail-fast",
        0,
        "21-rust-msrv-checks.txt",
    ),
    (
        "cargo +1.95.0 check --manifest-path apps/garnet-studio/src-tauri/Cargo.toml --all-targets --locked",
        0,
        "21-rust-msrv-checks.txt",
    ),
    ("cargo fmt --all -- --check", 0, "22-workspace-tests.txt"),
    ("cargo test -p garnet-cli new_cmd", 0, "22-workspace-tests.txt"),
    ("cargo test --workspace --no-fail-fast", 0, "22-workspace-tests.txt"),
    ("node ops/mission/render-sotu.mjs", 0, "23-sotu-render.txt"),
    (
        "python3 scripts/check_dogfood_pr_body.py --base 231aefa91985e5a0520c493c7f0fc3e54d74efc8 --head <CANDIDATE_SHA> --body-file ops/lane0/PR_BODY.md",
        0,
        "24-pr-body-validation.txt",
    ),
)

PR_BODY_COMMAND_RE = re.compile(
    r"^python3 scripts/check_dogfood_pr_body\.py "
    r"--base 231aefa91985e5a0520c493c7f0fc3e54d74efc8 "
    r"--head ([0-9a-f]{40}) --body-file ops/lane0/PR_BODY\.md$"
)

TEXT_TRANSCRIPT_OUTPUTS = {
    "01-archipelago-contracts.txt",
    "02-first-parent-archive.txt",
    "03-successor-pin-delta.txt",
    "11-truth-check.txt",
    "20-python-tests.txt",
    "21-rust-msrv-checks.txt",
    "22-workspace-tests.txt",
    "23-sotu-render.txt",
    "24-pr-body-validation.txt",
}


@dataclass(frozen=True)
class CloseoutStatus:
    schema: str
    source: str
    evidence_files: int
    ledger_entries: int
    denominator_count: int
    launch_status: str
    audit_band: int | None
    governance_verdict: str
    stage: str
    findings: list[str]
    ok: bool


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _read_json(path: Path, findings: list[str], label: str) -> dict:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except OSError as exc:
        findings.append(f"{label} is unreadable: {exc}")
        return {}
    except json.JSONDecodeError as exc:
        findings.append(f"{label} is invalid JSON: {exc}")
        return {}
    if not isinstance(value, dict):
        findings.append(f"{label} must be a JSON object")
        return {}
    return value


def read_manifest_entries(manifest: Path) -> list[tuple[str, str]]:
    """Return ``(relative path, digest)`` entries without asserting validity."""
    entries: list[tuple[str, str]] = []
    try:
        lines = manifest.read_text(encoding="utf-8").splitlines()
    except OSError:
        return entries
    for line in lines:
        match = re.fullmatch(r"([0-9a-f]{64})  (.+)", line)
        if match is not None:
            entries.append((match.group(2), match.group(1)))
    return entries


def write_manifest(evidence_dir: Path, manifest: Path) -> None:
    """Write the exact sorted SHA-256 manifest, excluding the manifest itself."""
    evidence_dir.mkdir(parents=True, exist_ok=True)
    names = sorted(
        path.name
        for path in evidence_dir.iterdir()
        if path.name != manifest.name
    )
    lines = [f"{_sha256(evidence_dir / name)}  {name}" for name in names]
    manifest.write_text("\n".join(lines) + "\n", encoding="utf-8")


def verify_manifest(
    evidence_dir: Path, manifest: Path
) -> tuple[list[str], list[tuple[str, str]]]:
    findings: list[str] = []
    if not evidence_dir.is_dir():
        return [f"evidence directory is missing: {evidence_dir}"], []
    if not manifest.is_file() or manifest.is_symlink():
        return [f"manifest is missing or not a regular file: {manifest}"], []

    actual_names = sorted(
        path.name for path in evidence_dir.iterdir() if path.name != manifest.name
    )
    expected_names = sorted(EXPECTED_EVIDENCE_FILES)
    if actual_names != expected_names:
        missing = sorted(set(expected_names) - set(actual_names))
        extra = sorted(set(actual_names) - set(expected_names))
        findings.append(
            "evidence manifest exact coverage failed"
            f" (missing={missing}, extra={extra})"
        )

    try:
        lines = manifest.read_text(encoding="utf-8").splitlines()
    except OSError as exc:
        return [f"manifest is unreadable: {exc}"], []
    entries: list[tuple[str, str]] = []
    seen: set[str] = set()
    for index, line in enumerate(lines, 1):
        match = re.fullmatch(r"([0-9a-f]{64})  ([^\r\n]+)", line)
        if match is None:
            findings.append(f"manifest line {index} is malformed")
            continue
        digest, rel = match.groups()
        pure = PurePosixPath(rel)
        if pure.is_absolute() or ".." in pure.parts or "." in pure.parts:
            findings.append(f"manifest line {index} contains path traversal: {rel}")
            continue
        if len(pure.parts) != 1:
            findings.append(f"manifest line {index} is not a direct evidence file: {rel}")
            continue
        if rel in seen:
            findings.append(f"manifest contains duplicate path: {rel}")
            continue
        seen.add(rel)
        entries.append((rel, digest))

        target = evidence_dir / rel
        try:
            mode = target.lstat().st_mode
        except OSError:
            findings.append(f"manifest target is missing: {rel}")
            continue
        if stat.S_ISLNK(mode):
            findings.append(f"manifest target must not be a symlink: {rel}")
            continue
        if not stat.S_ISREG(mode):
            findings.append(f"manifest target must be a regular file: {rel}")
            continue
        if _sha256(target) != digest:
            findings.append(f"manifest hash mismatch: {rel}")

    manifest_order = [rel for rel, _ in entries]
    if manifest_order != sorted(manifest_order):
        findings.append("manifest entries must be sorted lexicographically by path")

    listed_names = sorted(rel for rel, _ in entries)
    if listed_names != expected_names:
        missing = sorted(set(expected_names) - set(listed_names))
        extra = sorted(set(listed_names) - set(expected_names))
        findings.append(
            "manifest listed paths do not provide exact coverage"
            f" (missing={missing}, extra={extra})"
        )
    return findings, entries


def _parse_utc(value: object) -> datetime | None:
    if not isinstance(value, str) or UTC_RE.fullmatch(value) is None:
        return None
    try:
        return datetime.strptime(value, "%Y-%m-%dT%H:%M:%SZ").replace(
            tzinfo=timezone.utc
        )
    except ValueError:
        return None


def verify_commands(
    path: Path,
) -> tuple[list[str], list[dict], datetime | None, datetime | None]:
    findings: list[str] = []
    data = _read_json(path, findings, "COMMANDS.json")
    if data.get("schema") != "garnet.lane0.commands/v1":
        findings.append("COMMANDS.json schema is invalid")
    entries = data.get("entries")
    if not isinstance(entries, list) or not entries:
        findings.append("COMMANDS.json entries must be a nonempty array")
        return findings, [], None, None
    exact_keys = {
        "command",
        "expectedExit",
        "actualExit",
        "startedAt",
        "endedAt",
        "output",
    }
    actual_inventory: list[tuple[object, object, object]] = []
    starts: list[datetime] = []
    ends: list[datetime] = []
    previous_end: datetime | None = None
    for index, entry in enumerate(entries, 1):
        if not isinstance(entry, dict) or set(entry) != exact_keys:
            findings.append(f"COMMANDS.json entry {index} has an invalid shape")
            continue
        command = entry.get("command")
        if not isinstance(command, str) or not command.strip() or "\n" in command:
            findings.append(f"COMMANDS.json entry {index} has an invalid command")
        elif re.search(
            r"(^|\s)(?:env|printenv)(?:\s|$)|gh\s+auth|Navigata1/garnet(?:\.git)?\s+main",
            command,
            flags=re.IGNORECASE,
        ):
            findings.append(
                f"COMMANDS.json entry {index} contains a forbidden credential/fork probe"
            )
        expected = entry.get("expectedExit")
        actual = entry.get("actualExit")
        if not isinstance(expected, int) or isinstance(expected, bool):
            findings.append(f"COMMANDS.json entry {index} expected exit is invalid")
        if not isinstance(actual, int) or isinstance(actual, bool):
            findings.append(f"COMMANDS.json entry {index} actual exit is invalid")
        elif actual != expected:
            findings.append(
                f"COMMANDS.json entry {index} exit mismatch: expected {expected}, got {actual}"
            )
        started = _parse_utc(entry.get("startedAt"))
        ended = _parse_utc(entry.get("endedAt"))
        if started is None:
            findings.append(f"COMMANDS.json entry {index} startedAt is not UTC")
        else:
            starts.append(started)
        if ended is None:
            findings.append(f"COMMANDS.json entry {index} endedAt is not UTC")
        else:
            ends.append(ended)
        if started is not None and ended is not None and ended < started:
            findings.append(f"COMMANDS.json entry {index} ends before it starts")
        if started is not None and previous_end is not None and started < previous_end:
            findings.append(f"COMMANDS.json entry {index} overlaps prior command capture")
        if ended is not None:
            previous_end = ended
        output = entry.get("output")
        if output not in EXPECTED_EVIDENCE_FILES:
            findings.append(f"COMMANDS.json entry {index} output is not sealed evidence")
        actual_inventory.append((command, expected, output))
    inventory_matches = len(actual_inventory) == len(EXPECTED_COMMANDS)
    for index, (actual_row, expected_row) in enumerate(
        zip(actual_inventory, EXPECTED_COMMANDS, strict=False)
    ):
        if index == len(EXPECTED_COMMANDS) - 1:
            command, expected_exit, output = actual_row
            inventory_matches &= (
                isinstance(command, str)
                and PR_BODY_COMMAND_RE.fullmatch(command) is not None
                and expected_exit == expected_row[1]
                and output == expected_row[2]
            )
        else:
            inventory_matches &= actual_row == expected_row
    if not inventory_matches:
        findings.append(
            "COMMANDS.json must match the exact mandatory command inventory and bindings"
        )
    return (
        findings,
        [entry for entry in entries if isinstance(entry, dict)],
        min(starts) if starts else None,
        max(ends) if ends else None,
    )


def verify_text_transcripts(
    evidence_dir: Path, commands: list[dict]
) -> list[str]:
    findings: list[str] = []
    for output in sorted(TEXT_TRANSCRIPT_OUTPUTS):
        expected_rows = [
            (
                entry.get("command"),
                entry.get("expectedExit"),
                entry.get("actualExit"),
            )
            for entry in commands
            if entry.get("output") == output
        ]
        try:
            text = (evidence_dir / output).read_text(encoding="utf-8")
        except OSError as exc:
            findings.append(f"{output} is unreadable: {exc}")
            continue
        commands_in_text = re.findall(r"(?m)^\$ (.+)$", text)
        expected_exits = [
            int(value)
            for value in re.findall(r"(?m)^expected_exit=(-?\d+)$", text)
        ]
        actual_exits = [
            int(value) for value in re.findall(r"(?m)^actual_exit=(-?\d+)$", text)
        ]
        observed_rows = list(
            zip(commands_in_text, expected_exits, actual_exits, strict=False)
        )
        if observed_rows != expected_rows:
            findings.append(
                f"{output} transcript blocks do not match COMMANDS.json bindings"
            )
    return findings


def _ledger_hash(body: dict) -> str:
    return hashlib.sha256(json.dumps(body, sort_keys=True).encode()).hexdigest()


def _load_ledger(path: Path, findings: list[str]) -> list[dict]:
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError as exc:
        findings.append(f"ledger is unreadable: {exc}")
        return []
    if not lines:
        findings.append("ledger is empty")
        return []
    entries: list[dict] = []
    previous = GENESIS_HASH
    previous_time: datetime | None = None
    for index, line in enumerate(lines, 1):
        try:
            entry = json.loads(line)
        except json.JSONDecodeError as exc:
            findings.append(f"ledger line {index} is invalid JSON: {exc}")
            continue
        if not isinstance(entry, dict):
            findings.append(f"ledger line {index} must be an object")
            continue
        if entry.get("prevHash") != previous:
            findings.append(f"ledger chain break at entry {index}")
        observed_time = _parse_utc(entry.get("at"))
        if observed_time is None:
            findings.append(f"ledger entry {index} has an invalid UTC timestamp")
        elif previous_time is not None and observed_time < previous_time:
            findings.append(f"ledger timestamp order regresses at entry {index}")
        if observed_time is not None:
            previous_time = observed_time
        recorded = entry.get("entryHash")
        body = {key: value for key, value in entry.items() if key != "entryHash"}
        calculated = _ledger_hash(body)
        if recorded != calculated:
            findings.append(f"ledger tampered entry {index}")
        if isinstance(recorded, str):
            previous = recorded
        entries.append(entry)
    return entries


def verify_ledger(
    path: Path,
    manifest_entries: list[tuple[str, str]],
    expected_run_id: str,
    state_gate_evidence: dict[str, str],
    command_start: datetime | None,
    command_end: datetime | None,
    evidence_floor: datetime | None,
    state_loopback_at: datetime | None,
) -> list[str]:
    findings: list[str] = []
    entries = _load_ledger(path, findings)
    if not entries:
        return findings
    if not expected_run_id:
        findings.append("ledger run id is empty")
    for index, entry in enumerate(entries, 1):
        if entry.get("runId") != expected_run_id:
            findings.append(f"ledger entry {index} has the wrong runId")

    events = [entry.get("event") for entry in entries]
    exact_events = [
        "session-start",
        "loopback",
        *(["evidence-sealed"] * len(EXPECTED_EVIDENCE_FILES)),
        *(["gate-evidence"] * len(EXPECTED_GATE_EVIDENCE)),
        "audit-band",
        "governance-verdict",
        "stage-advance",
        "session-close",
    ]
    if events != exact_events:
        findings.append("ledger event sequence is not the exact allowed closeout sequence")

    expected_evidence = dict(manifest_entries)
    evidence_events = [
        entry for entry in entries if entry.get("event") == "evidence-sealed"
    ]
    observed_evidence = {
        str(entry.get("path")): str(entry.get("sha256"))
        for entry in evidence_events
    }
    if observed_evidence != expected_evidence:
        findings.append("ledger evidence SHA events do not exactly match the manifest")
    if [entry.get("path") for entry in evidence_events] != sorted(expected_evidence):
        findings.append("ledger evidence seals must follow sorted manifest order")

    gate_events = [entry for entry in entries if entry.get("event") == "gate-evidence"]
    observed_gates = {
        str(entry.get("gate")): (
            str(entry.get("path")),
            str(entry.get("sha256")),
        )
        for entry in gate_events
    }
    expected_gates = {
        gate: (rel, expected_evidence.get(rel, "")) for gate, rel in state_gate_evidence.items()
    }
    if observed_gates != expected_gates:
        findings.append("ledger gate/evidence SHA events do not match mission state")
    if [entry.get("gate") for entry in gate_events] != list(state_gate_evidence):
        findings.append("ledger gate bindings must follow exact mission gate order")

    expected_shapes = {
        "session-start": {"at", "prevHash", "event", "runId", "entryHash"},
        "loopback": {
            "at",
            "prevHash",
            "event",
            "runId",
            "fromGate",
            "toStage",
            "reason",
            "entryHash",
        },
        "evidence-sealed": {
            "at",
            "prevHash",
            "event",
            "runId",
            "path",
            "sha256",
            "entryHash",
        },
        "gate-evidence": {
            "at",
            "prevHash",
            "event",
            "runId",
            "gate",
            "path",
            "sha256",
            "entryHash",
        },
        "audit-band": {"at", "prevHash", "event", "runId", "band", "entryHash"},
        "governance-verdict": {
            "at",
            "prevHash",
            "event",
            "runId",
            "verdict",
            "waivers",
            "entryHash",
        },
        "stage-advance": {
            "at",
            "prevHash",
            "event",
            "runId",
            "from",
            "to",
            "entryHash",
        },
        "session-close": {
            "at",
            "prevHash",
            "event",
            "runId",
            "nextActions",
            "entryHash",
        },
    }
    for index, entry in enumerate(entries, 1):
        event = entry.get("event")
        if event in expected_shapes and set(entry) != expected_shapes[event]:
            findings.append(f"ledger entry {index} has an invalid {event} shape")

    if len(entries) >= 2:
        loopback = entries[1]
        if (
            loopback.get("event") != "loopback"
            or loopback.get("fromGate") != "G4"
            or loopback.get("toStage") != "S2"
            or not str(loopback.get("reason", "")).strip()
        ):
            findings.append("ledger must record G4 -> S2 immediately after session start")
        if _parse_utc(loopback.get("at")) != state_loopback_at:
            findings.append("ledger loopback timestamp does not match mission state")
    if len(entries) >= 4:
        audit, verdict, stage, close = entries[-4:]
        if audit.get("event") != "audit-band" or audit.get("band") != 3:
            findings.append("ledger must record audit band 3")
        if (
            verdict.get("event") != "governance-verdict"
            or verdict.get("verdict") != "advisory"
            or verdict.get("waivers") != []
        ):
            findings.append("ledger must record S6 advisory with no waivers")
        if (
            stage.get("event") != "stage-advance"
            or stage.get("from") != "S2"
            or stage.get("to") != "S6"
        ):
            findings.append("ledger must record stage advance S2 -> S6")
        if (
            close.get("event") != "session-close"
            or close.get("nextActions") != list(EXPECTED_NEXT_ACTIONS)
        ):
            findings.append("ledger must end with the exact session close actions")

    parsed_times = [_parse_utc(entry.get("at")) for entry in entries]
    if (
        command_start is None
        or not parsed_times
        or parsed_times[0] is None
        or parsed_times[0] != command_start
    ):
        findings.append("ledger session start must equal command capture start")
    seal_times = [
        parsed_times[index]
        for index, entry in enumerate(entries)
        if entry.get("event") in {"evidence-sealed", "gate-evidence", "audit-band",
                                  "governance-verdict", "stage-advance", "session-close"}
    ]
    if command_end is None or evidence_floor is None or any(
        observed is None or observed <= evidence_floor for observed in seal_times
    ):
        findings.append(
            "ledger sealing and closeout timestamps must be strictly after captured evidence"
        )
    return findings


def _verify_denominators(
    value: object, label: str, derived: list[dict[str, object]]
) -> list[str]:
    if not isinstance(value, list):
        return [f"{label} denominators must be an array"]
    if len(value) != 4:
        return [f"{label} must contain exactly four readiness denominators"]
    if value != derived:
        return [f"{label} readiness denominators do not match current reporter evidence"]
    return []


def _verify_lane_state(
    state: dict,
) -> tuple[
    list[str],
    str,
    int | None,
    str,
    str,
    dict[str, str],
    datetime | None,
]:
    findings: list[str] = []
    mission = state.get("mission", {})
    if not isinstance(mission, dict) or mission.get("status") != "complete":
        findings.append("Lane 0 mission status must be complete")
    phases = state.get("phases")
    if not isinstance(phases, list) or [p.get("id") for p in phases if isinstance(p, dict)] != [
        "P0",
        "P1",
        "P2",
        "P3",
    ]:
        findings.append("Lane 0 state must contain exactly P0 through P3")
        phases = []
    state_gate_evidence: dict[str, str] = {}
    state_gate_commands: dict[str, str] = {}
    for phase in phases:
        if phase.get("status") != "done":
            findings.append(f"Lane 0 phase {phase.get('id')} is not done")
        gates = phase.get("gates")
        if not isinstance(gates, list) or not gates:
            findings.append(f"Lane 0 phase {phase.get('id')} has no gates")
            continue
        for gate in gates:
            if not isinstance(gate, dict) or gate.get("status") != "passed":
                findings.append(f"Lane 0 gate {gate.get('id') if isinstance(gate, dict) else '?'} is not passed")
                continue
            evidence = gate.get("evidence")
            if (
                not isinstance(evidence, str)
                or not evidence.startswith("ops/lane0/evidence/")
                or Path(evidence).name not in EXPECTED_EVIDENCE_FILES
            ):
                findings.append(f"Lane 0 gate {gate.get('id')} lacks sealed evidence")
            elif isinstance(gate.get("id"), str):
                state_gate_evidence[gate["id"]] = Path(evidence).name
                if isinstance(gate.get("command"), str):
                    state_gate_commands[gate["id"]] = gate["command"]
            if not isinstance(gate.get("lastRun"), str) or UTC_RE.fullmatch(gate["lastRun"]) is None:
                findings.append(f"Lane 0 gate {gate.get('id')} lacks a UTC lastRun")

    metrics = state.get("metrics")
    metric = None
    if isinstance(metrics, list):
        metric = next(
            (
                item
                for item in metrics
                if isinstance(item, dict)
                and item.get("label") == "Definition-of-done claims verified"
            ),
            None,
        )
    if not isinstance(metric, dict) or metric.get("current") != "4" or metric.get("target") != "4":
        findings.append("Lane 0 definition of done must be 4/4")

    resume = state.get("resume", {})
    if not isinstance(resume, dict):
        resume = {}
    if resume.get("activePhase") is not None:
        findings.append("Lane 0 resume.activePhase must be null at closeout")
    if resume.get("nextActions") != list(EXPECTED_NEXT_ACTIONS):
        findings.append("Lane 0 resume must preserve the exact four next actions")

    archipelago = state.get("archipelago", {})
    if not isinstance(archipelago, dict):
        archipelago = {}
    loop = archipelago.get("loop", {})
    if not isinstance(loop, dict):
        loop = {}
    stage = str(loop.get("stage", ""))
    if stage != "S6":
        findings.append("Lane 0 ARCHIPELAGO stage must be stage S6")
    loopbacks = loop.get("loopbacks")
    valid_loopbacks = (
        [
            item
            for item in loopbacks
            if isinstance(item, dict)
            and item.get("fromGate") == "G4"
            and item.get("toStage") == "S2"
        ]
        if isinstance(loopbacks, list)
        else []
    )
    if len(valid_loopbacks) != 1:
        findings.append("Lane 0 state must record exactly one G4 -> S2 loopback")
        loopback_at = None
    else:
        loopback_at = _parse_utc(valid_loopbacks[0].get("at"))
        if loopback_at is None:
            findings.append("Lane 0 state loopback timestamp must be valid UTC")

    governance = archipelago.get("governance", {})
    if not isinstance(governance, dict):
        governance = {}
    run_id = governance.get("ledgerRunId")
    if not isinstance(run_id, str) or not run_id:
        findings.append("Lane 0 ledgerRunId must be nonempty")
        run_id = ""
    verdict = state.get("closeout", {})
    if not isinstance(verdict, dict):
        verdict = {}
    verdict_name = str(verdict.get("governanceVerdict", ""))
    band = verdict.get("band")
    waivers = verdict.get("waivers")
    if verdict_name != "advisory":
        findings.append("Lane 0 S6 governance verdict must be advisory")
    if band != 3:
        findings.append("Lane 0 audit band must be 3")
        band = None
    if waivers != []:
        findings.append("Lane 0 S6 waivers must be an explicit empty list")
    if state_gate_evidence != EXPECTED_GATE_EVIDENCE:
        findings.append("Lane 0 state gate/evidence bindings are not exact")
    if state_gate_commands != EXPECTED_GATE_COMMANDS:
        findings.append("Lane 0 state gate commands are not exact")
    return (
        findings,
        run_id,
        band,
        verdict_name,
        stage,
        state_gate_evidence,
        loopback_at,
    )


def _verify_audit(path: Path) -> list[str]:
    try:
        text = path.read_text(encoding="utf-8")
    except OSError as exc:
        return [f"Lane 0 audit is unreadable: {exc}"]
    findings: list[str] = []
    lowered = text.lower()
    for phrase in (
        "band 3/5",
        "advisory",
        "g4",
        "s2",
        "waivers: none",
        "playwright",
        "lane 2c",
    ):
        if phrase not in lowered:
            findings.append(f"Lane 0 audit is missing required finding: {phrase}")
    return findings


def _scan_sensitive_evidence(evidence_dir: Path) -> list[str]:
    findings: list[str] = []
    patterns = (
        re.compile(r"github_pat_[A-Za-z0-9_]{20,}"),
        re.compile(r"gh[opsu]_[A-Za-z0-9]{20,}"),
        re.compile(r"(?im)^authorization:\s*(?:bearer|token)\s+\S+"),
        re.compile(r"(?im)^(?:GH_TOKEN|GITHUB_TOKEN|ADMIN_TOKEN)=\S+"),
    )
    for name in EXPECTED_EVIDENCE_FILES:
        path = evidence_dir / name
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        if any(pattern.search(text) for pattern in patterns):
            findings.append(f"sealed evidence contains credential-like material: {name}")
    return findings


def _require_fields(
    data: dict, expected: dict[str, object], label: str, findings: list[str]
) -> None:
    for key, value in expected.items():
        if data.get(key) != value:
            findings.append(f"{label}.{key} must be {value!r}")


def _verify_review(
    path: Path,
) -> tuple[list[str], bool, datetime | None, str | None]:
    try:
        text = path.read_text(encoding="utf-8")
    except OSError as exc:
        return [f"independent review evidence is unreadable: {exc}"], False, None, None
    headings = list(re.finditer(r"(?m)^## Final integrated review\s*$", text))
    if len(headings) != 1:
        return [
            "final integrated review is not APPROVED with zero open Critical/Important findings and complete provenance"
        ], False, None, None
    section_start = headings[0].end()
    next_heading = re.search(r"(?m)^## ", text[section_start:])
    section_end = (
        section_start + next_heading.start() if next_heading is not None else len(text)
    )
    section = text[section_start:section_end]

    field_patterns = {
        "verdict": r"(?m)^Final integrated verdict: \*\*(.+)\*\*$",
        "role": r"(?m)^Reviewer role: (.+)$",
        "range": (
            r"(?m)^Reviewed range: "
            r"`([0-9a-f]{40})\.\.([0-9a-f]{40})`$"
        ),
        "at": r"(?m)^Reviewed at: `([^`]+)`$",
        "critical": r"(?m)^Open Critical findings: (\d+)$",
        "important": r"(?m)^Open Important findings: (\d+)$",
    }
    matches = {
        name: list(re.finditer(pattern, text))
        for name, pattern in field_patterns.items()
    }
    singular = all(
        len(values) == 1
        and section_start <= values[0].start() < section_end
        for values in matches.values()
    )
    verdict = matches["verdict"][0] if singular else None
    role = matches["role"][0] if singular else None
    reviewed_range = matches["range"][0] if singular else None
    reviewed_at = matches["at"][0] if singular else None
    critical = matches["critical"][0] if singular else None
    important = matches["important"][0] if singular else None
    reviewed_time = (
        _parse_utc(reviewed_at.group(1)) if reviewed_at is not None else None
    )
    reviewed_head = (
        reviewed_range.group(2) if reviewed_range is not None else None
    )
    contradictory = (
        re.search(
            r"(?i)\b(?:PENDING|NEEDS PATCH|CHANGES REQUIRED)\b", text
        )
        is not None
        or re.search(r"(?m)^## Fix re-review\s*$", text) is not None
        or re.search(r"(?im)^Fix re-review:", text) is not None
        or re.search(
            r"(?im)^(?:Open )?(?:Critical|Important)(?: [A-Za-z]+)* "
            r"findings:\s*[1-9]\d*$",
            text,
        )
        is not None
    )
    provenance_complete = (
        singular
        and not contradictory
        and role is not None
        and "independent" in role.group(1).lower()
        and reviewed_range is not None
        and reviewed_range.group(1)
        == "231aefa91985e5a0520c493c7f0fc3e54d74efc8"
        and reviewed_at is not None
        and reviewed_time is not None
        and critical is not None
        and important is not None
    )
    approved = (
        provenance_complete
        and verdict is not None
        and verdict.group(1) == "APPROVED"
        and critical.group(1) == "0"
        and important.group(1) == "0"
    )
    if not approved:
        return [
            "final integrated review is not APPROVED with zero open Critical/Important findings and complete provenance"
        ], False, reviewed_time, reviewed_head
    return [], True, reviewed_time, reviewed_head


def _verify_squash_durable_review_marker(
    marker: object,
    evidence_reviewed_head: str | None,
    root: Path,
    *,
    verify_git: bool,
    expected_boundary: dict[str, object] | None = None,
    boundary_label: str = "review",
) -> list[str]:
    """Verify review provenance without requiring pre-squash commit ancestry."""
    if not isinstance(marker, dict):
        return ["squash-durable review marker is missing"]

    findings: list[str] = []
    if marker.get("schema") != "garnet.squash_durable_review_marker/v1":
        findings.append("review marker schema is invalid")
    if marker.get("verdict") != "approved":
        findings.append("review marker verdict must be approved")

    reviewed_head = marker.get("reviewed_head")
    if not isinstance(reviewed_head, str) or GIT_OID_RE.fullmatch(reviewed_head) is None:
        findings.append("review marker reviewed_head must be a full lowercase Git SHA")
    elif reviewed_head != evidence_reviewed_head:
        findings.append(
            "review marker reviewed_head does not match independent review evidence"
        )

    reviewed_head_tree = marker.get("reviewed_head_tree")
    if (
        not isinstance(reviewed_head_tree, str)
        or GIT_OID_RE.fullmatch(reviewed_head_tree) is None
    ):
        findings.append(
            "review marker reviewed_head_tree must be a full lowercase Git tree SHA"
        )

    reviewed_tree = marker.get("reviewed_tree")
    if not isinstance(reviewed_tree, str) or GIT_OID_RE.fullmatch(reviewed_tree) is None:
        findings.append("review marker reviewed_tree must be a full lowercase Git tree SHA")

    merged_commit = marker.get("merged_commit")
    if merged_commit is None:
        findings.append("review marker merged_commit is missing")
    elif not isinstance(merged_commit, str) or GIT_OID_RE.fullmatch(merged_commit) is None:
        findings.append("review marker merged_commit must be a full lowercase Git SHA")

    review_scope = marker.get("review_scope")
    if (
        not isinstance(review_scope, str)
        or "reviewed_head" not in review_scope
        or "does not extend or backdate" not in review_scope
    ):
        findings.append(
            "review marker must state that content proof does not extend or backdate review coverage"
        )

    post_review = marker.get("post_review_commits")
    if not isinstance(post_review, list):
        findings.append("review marker post_review_commits must be a list")
    else:
        for index, entry in enumerate(post_review):
            if not isinstance(entry, dict):
                findings.append(
                    f"review marker post_review_commits[{index}] must be an object"
                )
                continue
            commit = entry.get("commit")
            if not isinstance(commit, str) or GIT_OID_RE.fullmatch(commit) is None:
                findings.append(
                    f"review marker post_review_commits[{index}].commit must be a full lowercase Git SHA"
                )
            if entry.get("reviewed") is not False:
                findings.append(
                    f"review marker post_review_commits[{index}].reviewed must be false"
                )
            if not isinstance(entry.get("purpose"), str) or not entry["purpose"].strip():
                findings.append(
                    f"review marker post_review_commits[{index}].purpose must be nonempty"
                )

    if expected_boundary is not None and any(
        marker.get(key) != value for key, value in expected_boundary.items()
    ):
        findings.append(
            f"review marker does not match the exact {boundary_label} review boundary"
        )

    if findings or not verify_git:
        return findings
    assert isinstance(merged_commit, str)
    assert isinstance(reviewed_tree, str)

    main_ref = None
    for candidate in ("refs/remotes/origin/main", "refs/heads/main"):
        resolved = subprocess.run(
            [
                "git",
                "--no-replace-objects",
                "show-ref",
                "--verify",
                "--quiet",
                candidate,
            ],
            cwd=root,
            capture_output=True,
            text=True,
        )
        if resolved.returncode == 0:
            main_ref = candidate
            break
    if main_ref is None:
        return ["authoritative upstream main ref is unavailable"]

    commit_check = subprocess.run(
        [
            "git",
            "--no-replace-objects",
            "cat-file",
            "-e",
            f"{merged_commit}^{{commit}}",
        ],
        cwd=root,
        capture_output=True,
        text=True,
    )
    if commit_check.returncode != 0:
        return ["review marker merged_commit does not name a commit"]

    first_parent = subprocess.run(
        ["git", "--no-replace-objects", "rev-list", "--first-parent", main_ref],
        cwd=root,
        capture_output=True,
        text=True,
    )
    if first_parent.returncode != 0:
        return ["upstream main first-parent history could not be enumerated"]
    if merged_commit not in first_parent.stdout.splitlines():
        return ["review marker merged_commit is absent from upstream main first-parent history"]

    tree_result = subprocess.run(
        [
            "git",
            "--no-replace-objects",
            "rev-parse",
            f"{merged_commit}^{{tree}}",
        ],
        cwd=root,
        capture_output=True,
        text=True,
    )
    if tree_result.returncode != 0:
        return ["review marker merged_commit tree could not be resolved"]
    if tree_result.stdout.strip() != reviewed_tree:
        return ["review marker reviewed_tree mismatch for merged_commit"]
    return []


def _verify_main_reachable_changed_path_proof(
    root: Path,
    *,
    verify_git: bool,
    base_commit: str = LANE0_BASE_COMMIT,
    merged_commit: str = str(EXPECTED_LANE0_REVIEW_BOUNDARY["merged_commit"]),
    expected_landed_path_count: int = EXPECTED_LANDED_CHANGED_PATH_COUNT,
    post_review_only_paths: tuple[str, ...] = EXPECTED_POST_REVIEW_ONLY_PATHS,
) -> tuple[list[str], int]:
    """Derive the historical candidate count from main-reachable landed content.

    The pre-squash reviewed head remains provenance only. The landed range has
    one disclosed post-review companion, so its exact path set and count let the
    archived 86-path PR-body transcript remain checkable without that discarded
    commit object.
    """
    findings: list[str] = []
    historical_candidate_count = expected_landed_path_count - len(
        post_review_only_paths
    )
    if historical_candidate_count < 0:
        findings.append("post-review-only path count exceeds landed path count")
    if not verify_git:
        return findings, historical_candidate_count

    main_ref = "refs/remotes/origin/main"
    resolved = subprocess.run(
        [
            "git",
            "--no-replace-objects",
            "show-ref",
            "--verify",
            "--quiet",
            main_ref,
        ],
        cwd=root,
        capture_output=True,
        text=True,
    )
    if resolved.returncode != 0:
        return (
            ["authoritative origin main ref is unavailable for changed-path proof"],
            historical_candidate_count,
        )

    first_parent = subprocess.run(
        ["git", "--no-replace-objects", "rev-list", "--first-parent", main_ref],
        cwd=root,
        capture_output=True,
        text=True,
    )
    if first_parent.returncode != 0:
        return (
            [
                "upstream main first-parent history could not be enumerated "
                "for changed-path proof"
            ],
            historical_candidate_count,
        )
    first_parent_commits = first_parent.stdout.splitlines()
    if base_commit not in first_parent_commits:
        findings.append(
            "changed-path base is absent from upstream main first-parent history"
        )
    if merged_commit not in first_parent_commits:
        findings.append(
            "changed-path merged commit is absent from upstream main first-parent history"
        )
    if (
        base_commit in first_parent_commits
        and merged_commit in first_parent_commits
        and first_parent_commits.index(merged_commit)
        >= first_parent_commits.index(base_commit)
    ):
        findings.append("changed-path merged commit does not follow its base on main")
    if findings:
        return findings, historical_candidate_count

    changed = subprocess.run(
        [
            "git",
            "--no-replace-objects",
            "diff",
            "--no-ext-diff",
            "--name-only",
            f"{base_commit}..{merged_commit}",
        ],
        cwd=root,
        capture_output=True,
        text=True,
    )
    if changed.returncode != 0:
        return (
            ["main-reachable changed-path range could not be enumerated"],
            historical_candidate_count,
        )
    landed_paths = tuple(line for line in changed.stdout.splitlines() if line.strip())
    if len(landed_paths) != expected_landed_path_count:
        findings.append(
            "main-reachable changed-path count is not the exact landed count "
            f"({len(landed_paths)} != {expected_landed_path_count})"
        )
    for path in post_review_only_paths:
        if path not in landed_paths:
            findings.append(
                f"post-review-only path is absent from landed range: {path}"
            )
    return findings, historical_candidate_count


def _verify_reporter_evidence(
    evidence_dir: Path,
    main_state: dict,
    commands: list[dict],
    root: Path,
    findings: list[str],
    *,
    verify_git: bool,
) -> list[dict[str, object]]:
    environment = _read_json(
        evidence_dir / "00-environment.json", findings, "00-environment.json"
    )
    _require_fields(
        environment,
        {
            "schema": "garnet.lane0.environment/v1",
            "credentialAndForkMainProbePerformed": False,
        },
        "00-environment.json",
        findings,
    )
    dependencies = environment.get("dependencies", {})
    if not isinstance(dependencies, dict):
        dependencies = {}
    jsonschema = dependencies.get("jsonschema", {})
    playwright = dependencies.get("playwright", {})
    if not isinstance(jsonschema, dict) or jsonschema.get("available") is not True:
        findings.append("00-environment.json must record jsonschema as available")
    if not isinstance(playwright, dict) or playwright.get("available") is not False:
        findings.append("00-environment.json must record Playwright as unavailable")
    versions = environment.get("versions", {})
    if not isinstance(versions, dict):
        versions = {}
    if not str(versions.get("rustcExactMsrv", "")).startswith("rustc 1.95.0 "):
        findings.append("00-environment.json must record exact rustc 1.95.0")
    if not str(versions.get("cargoExactMsrv", "")).startswith("cargo 1.95.0 "):
        findings.append("00-environment.json must record exact cargo 1.95.0")

    try:
        contracts_text = (evidence_dir / "01-archipelago-contracts.txt").read_text(
            encoding="utf-8"
        )
    except OSError as exc:
        findings.append(f"01-archipelago-contracts.txt is unreadable: {exc}")
        contracts_text = ""
    for required in (
        "[PASS] idea.lock.json",
        "[PASS] plan.lock.json",
        "[PASS] state.json",
        "b9f7cee2823f9791503db20f33b22c9e20af7abe",
    ):
        if required not in contracts_text:
            findings.append(f"01-archipelago-contracts.txt is missing {required!r}")

    try:
        archive_text = (evidence_dir / "02-first-parent-archive.txt").read_text(
            encoding="utf-8"
        )
    except OSError as exc:
        findings.append(f"02-first-parent-archive.txt is unreadable: {exc}")
        archive_text = ""
    archive_rows = re.findall(r"(?m)^[0-9a-f]{7} .+\(#\d+", archive_text)
    if len(archive_rows) != 34 or re.search(r"(?m)^\s*34\s*$", archive_text) is None:
        findings.append("02-first-parent-archive.txt must prove exactly 34 commits")

    try:
        successor_text = (evidence_dir / "03-successor-pin-delta.txt").read_text(
            encoding="utf-8"
        )
    except OSError as exc:
        findings.append(f"03-successor-pin-delta.txt is unreadable: {exc}")
        successor_text = ""
    if (
        "231aefa ops(mission): open parallel launch convergence (#499)" not in successor_text
        or re.search(r"(?m)^1$", successor_text) is None
    ):
        findings.append("03-successor-pin-delta.txt must prove sole successor #499")

    truth_freeze = _read_json(
        evidence_dir / "04-truth-freeze.json", findings, "04-truth-freeze.json"
    )
    _require_fields(
        truth_freeze,
        {
            "schema": "garnet.lane0_truth_freeze/v1",
            "archive_pr_count": 34,
            "checkpoint_pr_count": 35,
            "successor_archive_pr": 499,
            "successor_archive_sha": "231aefa91985e5a0520c493c7f0fc3e54d74efc8",
            "active_phase": "P7",
            "action_tasks": ["P7-T1", "P7-T2", "P7-T3", "P7-T4"],
            "adversarial_findings_resolved": False,
            "findings": [],
            "ok": True,
        },
        "04-truth-freeze.json",
        findings,
    )

    msrv = _read_json(evidence_dir / "05-msrv.json", findings, "05-msrv.json")
    _require_fields(
        msrv,
        {
            "schema": "garnet.msrv_status/v2",
            "msrv": "1.95",
            "workspace_member_count": 16,
            "workspace_members_inheriting": 16,
            "active_manifest_count": 18,
            "active_manifest_set_exact": True,
            "excluded_manifests_declaring": 2,
            "current_surfaces_aligned": True,
            "workflow_projection_valid": True,
            "stable_tracking_preserved": True,
            "exact_msrv_ci_check": True,
            "studio_exact_msrv_ci_check": True,
            "reporter_ci_wired": True,
            "rust_toolchain_file_absent": True,
            "procedural_contract_present": True,
            "findings": [],
            "ok": True,
        },
        "05-msrv.json",
        findings,
    )

    backlog = _read_json(
        evidence_dir / "06-frozen-backlog.json", findings, "06-frozen-backlog.json"
    )
    _require_fields(
        backlog,
        {
            "schema": "garnet.lane0.frozen_backlog/v1",
            "exact_base_main_sha": "231aefa91985e5a0520c493c7f0fc3e54d74efc8",
            "entry_count": 8,
            "implemented_clause_count": 9,
            "states": {"implemented": 0, "partial": 4, "planned": 4, "research": 0},
            "findings": [],
            "ok": True,
        },
        "06-frozen-backlog.json",
        findings,
    )

    watch = _read_json(
        evidence_dir / "07-quarterly-watch.json", findings, "07-quarterly-watch.json"
    )
    _require_fields(
        watch,
        {
            "schema": "garnet.quarterly_competitive_watch/v1",
            "state": "planned",
            "as_of": "2026-07-16",
            "report_count": 0,
            "next_due": "2026-09-30",
            "contract_present": True,
            "findings": [],
            "ok": True,
        },
        "07-quarterly-watch.json",
        findings,
    )

    launch = _read_json(
        evidence_dir / "08-launch-readiness.json", findings, "08-launch-readiness.json"
    )
    _require_fields(
        launch,
        {
            "schema": "garnet.launch_readiness/v1",
            "recommendation": "HOLD",
            "launch_ready": False,
        },
        "08-launch-readiness.json",
        findings,
    )
    launch_gates = launch.get("gates")
    observed_launch = (
        [
            (gate.get("id"), gate.get("state"))
            for gate in launch_gates
            if isinstance(gate, dict)
        ]
        if isinstance(launch_gates, list)
        else []
    )
    if observed_launch != list(EXPECTED_LAUNCH_GATES):
        findings.append("08-launch-readiness.json gate ids/states are not exact")
    accepted_states = {"pass", "accepted-scoped"}
    critical_ids = {identifier for identifier, _ in EXPECTED_LAUNCH_GATES[:6]}
    launch_critical = sum(
        1
        for identifier, state in observed_launch
        if identifier in critical_ids and state in accepted_states
    )
    launch_ledger = sum(
        1 for _identifier, state in observed_launch if state in accepted_states
    )
    if launch_critical != 3 or launch_ledger != 3:
        findings.append("08-launch-readiness.json must derive 3/6 and 3/8")

    mit = _read_json(
        evidence_dir / "09-mit-readiness.json", findings, "09-mit-readiness.json"
    )
    _require_fields(
        mit,
        {
            "source": "committed-truth",
            "overall_status": "active-partial",
            "completion_percent": 93.1,
        },
        "09-mit-readiness.json",
        findings,
    )
    lanes = mit.get("lanes")
    score = Decimal("0")
    lane_ids: list[str] = []
    if not isinstance(lanes, list) or len(lanes) != 70:
        findings.append("09-mit-readiness.json must contain exactly 70 committed lanes")
        lanes = []
    for index, lane in enumerate(lanes, 1):
        if not isinstance(lane, dict):
            findings.append(f"09-mit-readiness.json lane {index} is invalid")
            continue
        lane_id = lane.get("id")
        status = lane.get("status")
        if not isinstance(lane_id, str):
            findings.append(f"09-mit-readiness.json lane {index} lacks an id")
        else:
            lane_ids.append(lane_id)
        if lane.get("evidence_class") != "committed":
            findings.append(f"09-mit-readiness.json lane {lane_id} is not committed")
        if status not in MIT_STATUS_SCORES:
            findings.append(f"09-mit-readiness.json lane {lane_id} has unknown status")
        else:
            score += MIT_STATUS_SCORES[status]
    if len(set(lane_ids)) != len(lane_ids):
        findings.append("09-mit-readiness.json lane ids must be unique")
    rounded_mit = round(float(score / Decimal("70") * Decimal("100")), 1)
    if score != Decimal("65.20") or rounded_mit != 93.1:
        findings.append("09-mit-readiness.json must derive 65.2/70 = 93.1% rounded")

    try:
        truth_text = (evidence_dir / "11-truth-check.txt").read_text(encoding="utf-8")
    except OSError as exc:
        findings.append(f"11-truth-check.txt is unreadable: {exc}")
        truth_text = ""
    if "truth --check: ok (6 fields vs machine truth, 4 stamped surfaces)" not in truth_text:
        findings.append("11-truth-check.txt does not prove the truth gate passed")

    integrity = _read_json(
        evidence_dir / "12-repository-evidence-integrity.json",
        findings,
        "12-repository-evidence-integrity.json",
    )
    _require_fields(
        integrity,
        {
            "schema": "garnet.evidence_integrity/v1",
            "bundles_total": 38,
            "bundles_ok": 38,
            "bundles_failed": 0,
            "failed": [],
            "ok": True,
        },
        "12-repository-evidence-integrity.json",
        findings,
    )

    for name, identifier in (
        ("13-wv6-pending.json", "WV-6"),
        ("14-wv7-pending.json", "WV-7"),
    ):
        wv = _read_json(evidence_dir / name, findings, name)
        _require_fields(
            wv,
            {
                "schema": "garnet.wv_acceptance_status/v1",
                "wv": identifier,
                "contract_base_main_sha": (
                    "231aefa91985e5a0520c493c7f0fc3e54d74efc8"
                ),
                "state": "pending",
                "candidate_main_sha": None,
                "passed_check_count": 0,
                "required_check_count": 5,
                "artifact_count": 0,
                "ok": False,
            },
            name,
            findings,
        )
        expected_destination = {
            "WV-6": "proofs/windows/launch-verification/wv6-minimum-shelf/",
            "WV-7": "proofs/windows/launch-verification/wv7-distribution/",
        }[identifier]
        if wv.get("evidence_destination") != expected_destination:
            findings.append(f"{name}.evidence_destination is not exact")
        if wv.get("findings") != ["exact-candidate evidence manifest is pending"]:
            findings.append(f"{name}.findings must preserve the expected pending reason")

    try:
        render_text = (evidence_dir / "23-sotu-render.txt").read_text(encoding="utf-8")
    except OSError as exc:
        findings.append(f"23-sotu-render.txt is unreadable: {exc}")
        render_text = ""
    if "Rendered " not in render_text or "phases: 8, tasks: 19/23" not in render_text:
        findings.append("23-sotu-render.txt does not prove the mission SOTU rendered")
    try:
        pr_text = (evidence_dir / "24-pr-body-validation.txt").read_text(
            encoding="utf-8"
        )
    except OSError as exc:
        findings.append(f"24-pr-body-validation.txt is unreadable: {exc}")
        pr_text = ""
    pr_command = next(
        (
            entry.get("command")
            for entry in commands
            if entry.get("output") == "24-pr-body-validation.txt"
        ),
        None,
    )
    candidate_match = (
        PR_BODY_COMMAND_RE.fullmatch(pr_command)
        if isinstance(pr_command, str)
        else None
    )
    changed_path_findings, expected_path_count = (
        _verify_main_reachable_changed_path_proof(root, verify_git=verify_git)
    )
    findings.extend(changed_path_findings)
    if candidate_match is None:
        findings.append("PR-body command lacks an explicit candidate SHA")
    else:
        candidate = candidate_match.group(1)
        if candidate != EXPECTED_LANE0_REVIEW_BOUNDARY["reviewed_head"]:
            findings.append(
                "PR-body candidate SHA does not match reviewed-head provenance"
            )
    if (
        f"dogfood-pr-body: ok ({expected_path_count} changed files checked)"
        not in pr_text
    ):
        findings.append(
            "24-pr-body-validation.txt path count does not match its explicit candidate"
        )

    phases = main_state.get("phases")
    historical_tasks: list[dict] = []
    if isinstance(phases, list):
        for phase in phases:
            if (
                isinstance(phase, dict)
                and phase.get("id") in {f"P{index}" for index in range(7)}
                and isinstance(phase.get("tasks"), list)
            ):
                historical_tasks.extend(
                    task for task in phase["tasks"] if isinstance(task, dict)
                )
    s114_done = sum(1 for task in historical_tasks if task.get("status") == "done")
    if len(historical_tasks) != 19 or s114_done != 19:
        findings.append("ops/mission/state.json must derive S114 at 19/19")

    return [
        {
            "id": "s114_mission",
            "label": "S114 bounded mission",
            "numerator": s114_done,
            "denominator": len(historical_tasks),
            "percent": 100.0,
            "evidence": "ops/lane0/evidence/10-denominators.json",
        },
        {
            "id": "truth_pulse",
            "label": "Truth pulse",
            "numerator": float(score),
            "denominator": len(lanes),
            "percent": rounded_mit,
            "rounded": True,
            "evidence": "ops/lane0/evidence/09-mit-readiness.json",
        },
        {
            "id": "launch_critical",
            "label": "Launch-critical",
            "numerator": launch_critical,
            "denominator": 6,
            "percent": 50.0,
            "evidence": "ops/lane0/evidence/08-launch-readiness.json",
        },
        {
            "id": "launch_ledger",
            "label": "Whole launch ledger",
            "numerator": launch_ledger,
            "denominator": len(observed_launch),
            "percent": 37.5,
            "evidence": "ops/lane0/evidence/08-launch-readiness.json",
        },
    ]


def read_status(root: Path = ROOT, *, verify_git: bool = True) -> CloseoutStatus:
    findings: list[str] = []
    evidence_dir = root / "ops/lane0/evidence"
    manifest = evidence_dir / "MANIFEST.sha256"
    manifest_findings, manifest_entries = verify_manifest(evidence_dir, manifest)
    findings.extend(manifest_findings)
    command_findings, commands, command_start, command_end = verify_commands(
        evidence_dir / "COMMANDS.json"
    )
    findings.extend(command_findings)
    findings.extend(verify_text_transcripts(evidence_dir, commands))
    findings.extend(_scan_sensitive_evidence(evidence_dir))
    review_findings, review_approved, review_time, evidence_reviewed_head = (
        _verify_review(evidence_dir / "25-independent-review.md")
    )
    findings.extend(review_findings)
    sotu_time = _sotu_timestamp(root / "ops/mission/state-of-the-union.html")
    if sotu_time is None:
        findings.append("generated mission SOTU lacks a second-precision UTC timestamp")
    evidence_times = [
        value for value in (command_end, review_time, sotu_time) if value is not None
    ]
    evidence_floor = max(evidence_times) if evidence_times else None

    lane_state = _read_json(
        root / "ops/lane0/state.json", findings, "ops/lane0/state.json"
    )
    (
        lane_findings,
        run_id,
        band,
        verdict,
        stage,
        state_gate_evidence,
        state_loopback_at,
    ) = _verify_lane_state(lane_state)
    findings.extend(lane_findings)
    findings.extend(
        verify_ledger(
            root / "ops/lane0/ledger.jsonl",
            manifest_entries,
            run_id,
            state_gate_evidence,
            command_start,
            command_end,
            evidence_floor,
            state_loopback_at,
        )
    )
    findings.extend(_verify_audit(root / "ops/lane0/AUDIT.md"))

    main_state = _read_json(
        root / "ops/mission/state.json", findings, "ops/mission/state.json"
    )
    readiness = main_state.get("readiness", {})
    if not isinstance(readiness, dict):
        readiness = {}
    launch_status = str(readiness.get("launchStatus", ""))
    if launch_status != "HOLD":
        findings.append("main mission launch status must remain HOLD")
    derived_denominators = _verify_reporter_evidence(
        evidence_dir,
        main_state,
        commands,
        root,
        findings,
        verify_git=verify_git,
    )
    denominators = readiness.get("denominators")
    findings.extend(
        _verify_denominators(denominators, "main mission", derived_denominators)
    )

    lane_closeout = main_state.get("lane0Closeout", {})
    if not isinstance(lane_closeout, dict):
        lane_closeout = {}
    if lane_closeout.get("audit") != "ops/lane0/AUDIT.md":
        findings.append("main mission must reference the Lane 0 audit")
    if lane_closeout.get("nextActions") != list(EXPECTED_NEXT_ACTIONS):
        findings.append("main mission must preserve the exact four Lane 0 next actions")

    denom_file = _read_json(
        evidence_dir / "10-denominators.json",
        findings,
        "10-denominators.json",
    )
    if denom_file.get("schema") != "garnet.lane0.denominators/v1":
        findings.append("10-denominators.json schema is invalid")
    if denom_file.get("launchStatus") != "HOLD":
        findings.append("10-denominators.json launch status must remain HOLD")
    findings.extend(
        _verify_denominators(
            denom_file.get("denominators"), "evidence", derived_denominators
        )
    )
    lane_closeout_state = lane_state.get("closeout", {})
    if not isinstance(lane_closeout_state, dict):
        lane_closeout_state = {}
    lane_review_marker = lane_closeout_state.get("finalIntegratedReview")
    main_review_marker = lane_closeout.get("finalIntegratedReview")
    if review_approved:
        findings.extend(
            _verify_squash_durable_review_marker(
                lane_review_marker,
                evidence_reviewed_head,
                root,
                verify_git=verify_git,
                expected_boundary=EXPECTED_LANE0_REVIEW_BOUNDARY,
                boundary_label="Lane 0",
            )
        )
        if main_review_marker != lane_review_marker:
            findings.append("Lane 0 and main mission review markers diverge")
            findings.extend(
                _verify_squash_durable_review_marker(
                    main_review_marker,
                    evidence_reviewed_head,
                    root,
                    verify_git=verify_git,
                    expected_boundary=EXPECTED_LANE0_REVIEW_BOUNDARY,
                    boundary_label="Lane 0",
                )
            )
    else:
        if lane_review_marker != "pending":
            findings.append("Lane 0 state final integrated review marker is inconsistent")
        if main_review_marker != "pending":
            findings.append("main mission final integrated review marker is inconsistent")

    return CloseoutStatus(
        schema="garnet.lane0.closeout/v1",
        source=str(root),
        evidence_files=len(manifest_entries),
        ledger_entries=(
            len((root / "ops/lane0/ledger.jsonl").read_text(encoding="utf-8").splitlines())
            if (root / "ops/lane0/ledger.jsonl").is_file()
            else 0
        ),
        denominator_count=len(denominators) if isinstance(denominators, list) else 0,
        launch_status=launch_status,
        audit_band=band,
        governance_verdict=verdict,
        stage=stage,
        findings=findings,
        ok=not findings,
    )


def _utc_now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def write_denominators(root: Path, at: str | None = None) -> None:
    """Derive the only four admitted denominators from captured current evidence."""
    evidence = root / "ops/lane0/evidence"
    launch = json.loads(
        (evidence / "08-launch-readiness.json").read_text(encoding="utf-8")
    )
    mit = json.loads(
        (evidence / "09-mit-readiness.json").read_text(encoding="utf-8")
    )
    mission = json.loads(
        (root / "ops/mission/state.json").read_text(encoding="utf-8")
    )

    launch_gates = launch.get("gates")
    if not isinstance(launch_gates, list):
        raise ValueError("launch reporter must expose exactly eight ledger gates")
    observed_launch = [
        (gate.get("id"), gate.get("state"))
        for gate in launch_gates
        if isinstance(gate, dict)
    ]
    if observed_launch != list(EXPECTED_LAUNCH_GATES):
        raise ValueError("launch reporter gate ids/states are not exact")
    critical_ids = {
        "foundation_integrity",
        "native_linux",
        "s114_acceptance",
        "static_playground",
        "live_wasm_playground",
        "minimum_sealed_shelf",
    }
    accepted_states = {"pass", "accepted-scoped"}
    critical_passed = sum(
        1
        for identifier, state in observed_launch
        if identifier in critical_ids and state in accepted_states
    )
    ledger_passed = sum(
        1 for _identifier, state in observed_launch if state in accepted_states
    )
    if (
        critical_passed != 3
        or ledger_passed != 3
        or launch.get("recommendation") != "HOLD"
        or launch.get("launch_ready") is not False
    ):
        raise ValueError("launch reporter no longer derives HOLD at 3/6 and 3/8")

    lanes = mit.get("lanes")
    if not isinstance(lanes, list) or len(lanes) != 70:
        raise ValueError("committed MIT reporter must expose exactly 70 lanes")
    score = Decimal("0")
    for lane in lanes:
        if not isinstance(lane, dict) or lane.get("evidence_class") != "committed":
            raise ValueError("MIT denominator accepts committed lanes only")
        status = lane.get("status")
        if status not in MIT_STATUS_SCORES:
            raise ValueError(f"unknown MIT lane status: {status!r}")
        score += MIT_STATUS_SCORES[status]
    rounded_mit = round(float(score / Decimal("70") * Decimal("100")), 1)
    if (
        mit.get("source") != "committed-truth"
        or mit.get("completion_percent") != 93.1
        or score != Decimal("65.20")
        or rounded_mit != 93.1
    ):
        raise ValueError("committed MIT reporter no longer derives 93.1%")

    phases = mission.get("phases")
    historical_tasks = []
    if isinstance(phases, list):
        for phase in phases:
            if (
                isinstance(phase, dict)
                and phase.get("id") in {f"P{index}" for index in range(7)}
                and isinstance(phase.get("tasks"), list)
            ):
                historical_tasks.extend(phase["tasks"])
    done = sum(
        1
        for task in historical_tasks
        if isinstance(task, dict) and task.get("status") == "done"
    )
    if len(historical_tasks) != 19 or done != 19:
        raise ValueError("S114 bounded mission no longer derives 19/19")

    denominators = [
        {
            "id": "s114_mission",
            "label": "S114 bounded mission",
            "numerator": done,
            "denominator": len(historical_tasks),
            "percent": 100.0,
            "evidence": "ops/lane0/evidence/10-denominators.json",
        },
        {
            "id": "truth_pulse",
            "label": "Truth pulse",
            "numerator": float(score),
            "denominator": len(lanes),
            "percent": rounded_mit,
            "rounded": True,
            "evidence": "ops/lane0/evidence/09-mit-readiness.json",
        },
        {
            "id": "launch_critical",
            "label": "Launch-critical",
            "numerator": critical_passed,
            "denominator": 6,
            "percent": 50.0,
            "evidence": "ops/lane0/evidence/08-launch-readiness.json",
        },
        {
            "id": "launch_ledger",
            "label": "Whole launch ledger",
            "numerator": ledger_passed,
            "denominator": len(observed_launch),
            "percent": 37.5,
            "evidence": "ops/lane0/evidence/08-launch-readiness.json",
        },
    ]
    output = {
        "schema": "garnet.lane0.denominators/v1",
        "asOf": at or _utc_now(),
        "launchStatus": "HOLD",
        "denominators": denominators,
        "sources": {
            "s114": "ops/mission/state.json P0-P6 task states",
            "truthPulse": "65.2/70 committed-lane score from ops/lane0/evidence/09-mit-readiness.json",
            "launch": "ops/lane0/evidence/08-launch-readiness.json",
        },
        "discipline": "exactly four; never averaged and no fifth denominator",
    }
    (evidence / "10-denominators.json").write_text(
        json.dumps(output, indent=2) + "\n", encoding="utf-8"
    )


def _append_ledger(
    path: Path, previous: str, at: str, event: dict[str, object]
) -> str:
    body = {"at": at, "prevHash": previous, **event}
    entry_hash = _ledger_hash(body)
    body["entryHash"] = entry_hash
    with path.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(body) + "\n")
    return entry_hash


def _format_utc(value: datetime) -> str:
    return value.astimezone(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def _review_timestamp(path: Path) -> datetime | None:
    try:
        text = path.read_text(encoding="utf-8")
    except OSError:
        return None
    match = re.search(r"(?m)^Reviewed at: `([^`]+)`$", text)
    return _parse_utc(match.group(1)) if match is not None else None


def _sotu_timestamp(path: Path) -> datetime | None:
    try:
        text = path.read_text(encoding="utf-8")
    except OSError:
        return None
    match = re.search(
        r"Mission Control &middot; State of the Union &middot; generated "
        r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})Z",
        text,
    )
    if match is None:
        return None
    return _parse_utc(match.group(1).replace(" ", "T") + "Z")


def write_ledger(root: Path, run_id: str) -> None:
    """Create the namespaced ARCHIPELAGO ledger from the sealed evidence."""
    command_findings, _commands, command_start, command_end = verify_commands(
        root / "ops/lane0/evidence/COMMANDS.json"
    )
    if command_findings or command_start is None or command_end is None:
        raise ValueError(
            "cannot write ledger from invalid command capture: "
            + "; ".join(command_findings)
        )
    review_at = _review_timestamp(
        root / "ops/lane0/evidence/25-independent-review.md"
    )
    sotu_at = _sotu_timestamp(root / "ops/mission/state-of-the-union.html")
    evidence_floor = max(
        value for value in (command_end, review_at, sotu_at) if value is not None
    )
    start_at = _format_utc(command_start)
    seal_time = evidence_floor + timedelta(seconds=1)
    seal_at = _format_utc(seal_time)
    path = root / "ops/lane0/ledger.jsonl"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("", encoding="utf-8")
    previous = GENESIS_HASH
    previous = _append_ledger(
        path,
        previous,
        start_at,
        {"event": "session-start", "runId": run_id},
    )
    previous = _append_ledger(
        path,
        previous,
        start_at,
        {
            "event": "loopback",
            "runId": run_id,
            "fromGate": "G4",
            "toStage": "S2",
            "reason": (
                "Lane 2C approval lacked current deterministic evidence for three "
                "exact-candidate stress cases exceeding four minutes."
            ),
        },
    )
    manifest = root / "ops/lane0/evidence/MANIFEST.sha256"
    for rel, digest in read_manifest_entries(manifest):
        previous = _append_ledger(
            path,
            previous,
            seal_at,
            {
                "event": "evidence-sealed",
                "runId": run_id,
                "path": rel,
                "sha256": digest,
            },
        )
    manifest_digests = dict(read_manifest_entries(manifest))
    for gate, rel in EXPECTED_GATE_EVIDENCE.items():
        previous = _append_ledger(
            path,
            previous,
            seal_at,
            {
                "event": "gate-evidence",
                "runId": run_id,
                "gate": gate,
                "path": rel,
                "sha256": manifest_digests[rel],
            },
        )
    previous = _append_ledger(
        path,
        previous,
        _format_utc(seal_time + timedelta(seconds=1)),
        {"event": "audit-band", "runId": run_id, "band": 3},
    )
    previous = _append_ledger(
        path,
        previous,
        _format_utc(seal_time + timedelta(seconds=2)),
        {
            "event": "governance-verdict",
            "runId": run_id,
            "verdict": "advisory",
            "waivers": [],
        },
    )
    previous = _append_ledger(
        path,
        previous,
        _format_utc(seal_time + timedelta(seconds=3)),
        {
            "event": "stage-advance",
            "runId": run_id,
            "from": "S2",
            "to": "S6",
        },
    )
    _append_ledger(
        path,
        previous,
        _format_utc(seal_time + timedelta(seconds=4)),
        {
            "event": "session-close",
            "runId": run_id,
            "nextActions": list(EXPECTED_NEXT_ACTIONS),
        },
    )


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=ROOT)
    parser.add_argument("--format", choices=("json", "human"), default="human")
    parser.add_argument("--gate", action="store_true")
    parser.add_argument(
        "--seal",
        action="store_true",
        help="write the sorted evidence manifest and exact ARCHIPELAGO ledger",
    )
    parser.add_argument(
        "--write-denominators",
        action="store_true",
        help="derive 10-denominators.json from captured launch/MIT/mission evidence",
    )
    parser.add_argument("--run-id", help="required with --seal")
    parser.add_argument("--at", help="optional UTC timestamp used for every ledger event")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    root = args.repo_root.resolve()
    if args.write_denominators:
        try:
            write_denominators(root, args.at)
        except (OSError, ValueError, json.JSONDecodeError) as exc:
            print(f"denominator derivation failed: {exc}", file=sys.stderr)
            return 1
        if not args.gate and not args.seal:
            return 0
    if args.seal:
        if not args.run_id:
            print("--run-id is required with --seal", file=sys.stderr)
            return 2
        evidence_dir = root / "ops/lane0/evidence"
        write_manifest(evidence_dir, evidence_dir / "MANIFEST.sha256")
        try:
            write_ledger(root, args.run_id)
        except ValueError as exc:
            print(f"ledger sealing failed: {exc}", file=sys.stderr)
            return 1
    status = read_status(root)
    if args.format == "json":
        print(json.dumps(asdict(status), indent=2))
    else:
        print(
            "Lane 0 closeout:"
            f" {'PASS' if status.ok else 'FAIL'}"
            f" · evidence {status.evidence_files}/{len(EXPECTED_EVIDENCE_FILES)}"
            f" · ledger {status.ledger_entries} entries"
            f" · denominators {status.denominator_count}/4"
            f" · launch {status.launch_status or '?'}"
            f" · band {status.audit_band if status.audit_band is not None else '?'}"
            f" · S6 {status.governance_verdict or '?'}"
        )
        for finding in status.findings:
            print(f"  - {finding}")
    if args.gate and not status.ok:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
