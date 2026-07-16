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
import sys
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath
from typing import Iterable


ROOT = Path(__file__).resolve().parents[1]
GENESIS_HASH = "0" * 64
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
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
        "numerator": 931,
        "denominator": 1000,
        "percent": 93.1,
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


def verify_commands(path: Path) -> list[str]:
    findings: list[str] = []
    data = _read_json(path, findings, "COMMANDS.json")
    if data.get("schema") != "garnet.lane0.commands/v1":
        findings.append("COMMANDS.json schema is invalid")
    entries = data.get("entries")
    if not isinstance(entries, list) or not entries:
        findings.append("COMMANDS.json entries must be a nonempty array")
        return findings
    exact_keys = {
        "command",
        "expectedExit",
        "actualExit",
        "startedAt",
        "endedAt",
        "output",
    }
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
        for key in ("startedAt", "endedAt"):
            if not isinstance(entry.get(key), str) or UTC_RE.fullmatch(entry[key]) is None:
                findings.append(f"COMMANDS.json entry {index} {key} is not UTC")
        output = entry.get("output")
        if output not in EXPECTED_EVIDENCE_FILES:
            findings.append(f"COMMANDS.json entry {index} output is not sealed evidence")
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
    manifest_entries: Iterable[tuple[str, str]],
    expected_run_id: str,
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
    for required in (
        "session-start",
        "loopback",
        "audit-band",
        "governance-verdict",
        "stage-advance",
        "session-close",
    ):
        if required not in events:
            findings.append(f"ledger is missing required event: {required}")

    expected_evidence = dict(manifest_entries)
    observed_evidence: dict[str, str] = {}
    for entry in entries:
        if entry.get("event") == "evidence-sealed":
            rel = entry.get("path")
            digest = entry.get("sha256")
            if isinstance(rel, str) and isinstance(digest, str):
                if rel in observed_evidence:
                    findings.append(f"ledger duplicates evidence seal: {rel}")
                observed_evidence[rel] = digest
    if observed_evidence != expected_evidence:
        findings.append("ledger evidence SHA events do not exactly match the manifest")

    observed_gates: dict[str, tuple[str, str]] = {}
    for entry in entries:
        if entry.get("event") != "gate-evidence":
            continue
        gate = entry.get("gate")
        rel = entry.get("path")
        digest = entry.get("sha256")
        if not all(isinstance(value, str) for value in (gate, rel, digest)):
            findings.append("ledger gate-evidence entry has an invalid shape")
            continue
        if gate in observed_gates:
            findings.append(f"ledger duplicates gate evidence: {gate}")
        observed_gates[gate] = (rel, digest)
    expected_gates = {
        gate: (rel, expected_evidence.get(rel, ""))
        for gate, rel in EXPECTED_GATE_EVIDENCE.items()
    }
    if observed_gates != expected_gates:
        findings.append("ledger gate/evidence SHA events do not match the closed gates")

    loopbacks = [
        entry
        for entry in entries
        if entry.get("event") == "loopback"
        and entry.get("fromGate") == "G4"
        and entry.get("toStage") == "S2"
    ]
    if len(loopbacks) != 1:
        findings.append("ledger must record exactly one G4 -> S2 loopback")
    bands = [
        entry
        for entry in entries
        if entry.get("event") == "audit-band" and entry.get("band") == 3
    ]
    if len(bands) != 1:
        findings.append("ledger must record audit band 3")
    verdicts = [
        entry
        for entry in entries
        if entry.get("event") == "governance-verdict"
        and entry.get("verdict") == "advisory"
        and entry.get("waivers") == []
    ]
    if len(verdicts) != 1:
        findings.append("ledger must record the advisory S6 verdict with no waivers")
    stages = [
        entry
        for entry in entries
        if entry.get("event") == "stage-advance" and entry.get("to") == "S6"
    ]
    if len(stages) != 1:
        findings.append("ledger must record stage S6")
    closes = [entry for entry in entries if entry.get("event") == "session-close"]
    if len(closes) != 1 or closes[0].get("nextActions") != list(EXPECTED_NEXT_ACTIONS):
        findings.append("ledger session close must preserve the exact four next actions")
    return findings


def _verify_denominators(value: object, label: str) -> list[str]:
    if not isinstance(value, list):
        return [f"{label} denominators must be an array"]
    if len(value) != 4:
        return [f"{label} must contain exactly four readiness denominators"]
    if value != list(EXPECTED_DENOMINATORS):
        return [f"{label} readiness denominators do not match the frozen values"]
    return []


def _verify_lane_state(state: dict) -> tuple[list[str], str, int | None, str, str]:
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
    return findings, run_id, band, verdict_name, stage


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


def read_status(root: Path = ROOT) -> CloseoutStatus:
    findings: list[str] = []
    evidence_dir = root / "ops/lane0/evidence"
    manifest = evidence_dir / "MANIFEST.sha256"
    manifest_findings, manifest_entries = verify_manifest(evidence_dir, manifest)
    findings.extend(manifest_findings)
    findings.extend(verify_commands(evidence_dir / "COMMANDS.json"))
    findings.extend(_scan_sensitive_evidence(evidence_dir))

    lane_state = _read_json(
        root / "ops/lane0/state.json", findings, "ops/lane0/state.json"
    )
    lane_findings, run_id, band, verdict, stage = _verify_lane_state(lane_state)
    findings.extend(lane_findings)
    findings.extend(
        verify_ledger(root / "ops/lane0/ledger.jsonl", manifest_entries, run_id)
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
    denominators = readiness.get("denominators")
    findings.extend(_verify_denominators(denominators, "main mission"))

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
        _verify_denominators(denom_file.get("denominators"), "evidence")
    )

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
    if not isinstance(launch_gates, list) or len(launch_gates) != 8:
        raise ValueError("launch reporter must expose exactly eight ledger gates")
    critical_ids = {
        "foundation_integrity",
        "native_linux",
        "s114_acceptance",
        "static_playground",
        "live_wasm_playground",
        "minimum_sealed_shelf",
    }
    accepted_states = {"pass", "accepted-scoped"}
    critical = [
        gate
        for gate in launch_gates
        if isinstance(gate, dict) and gate.get("id") in critical_ids
    ]
    critical_passed = sum(
        1 for gate in critical if gate.get("state") in accepted_states
    )
    ledger_passed = sum(
        1
        for gate in launch_gates
        if isinstance(gate, dict) and gate.get("state") in accepted_states
    )
    if (
        len(critical) != 6
        or critical_passed != 3
        or ledger_passed != 3
        or launch.get("recommendation") != "HOLD"
        or launch.get("launch_ready") is not False
    ):
        raise ValueError("launch reporter no longer derives HOLD at 3/6 and 3/8")

    if mit.get("source") != "committed-truth" or mit.get("completion_percent") != 93.1:
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

    output = {
        "schema": "garnet.lane0.denominators/v1",
        "asOf": at or _utc_now(),
        "launchStatus": "HOLD",
        "denominators": list(EXPECTED_DENOMINATORS),
        "sources": {
            "s114": "ops/mission/state.json P0-P6 task states",
            "truthPulse": "ops/lane0/evidence/09-mit-readiness.json committed-only",
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


def write_ledger(root: Path, run_id: str, at: str | None = None) -> None:
    """Create the namespaced ARCHIPELAGO ledger from the sealed evidence."""
    timestamp = at or _utc_now()
    path = root / "ops/lane0/ledger.jsonl"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("", encoding="utf-8")
    previous = GENESIS_HASH
    previous = _append_ledger(
        path,
        previous,
        timestamp,
        {"event": "session-start", "runId": run_id},
    )
    manifest = root / "ops/lane0/evidence/MANIFEST.sha256"
    for rel, digest in read_manifest_entries(manifest):
        previous = _append_ledger(
            path,
            previous,
            timestamp,
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
            timestamp,
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
        timestamp,
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
    previous = _append_ledger(
        path,
        previous,
        timestamp,
        {"event": "audit-band", "runId": run_id, "band": 3},
    )
    previous = _append_ledger(
        path,
        previous,
        timestamp,
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
        timestamp,
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
        timestamp,
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
        write_ledger(root, args.run_id, args.at)
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
