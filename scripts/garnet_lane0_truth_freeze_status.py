#!/usr/bin/env python3
"""Validate the Lane 0 first-parent archive and U-18 resume contract."""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ARCHIVE_BASE_EXCLUSIVE = "d0d4f7cc988db4d793c5c7555a4043aef9b27180"
ARCHIVE_HEAD_INCLUSIVE = "1fe74892c588f912e103742afc9d11e845e8d4e6"
SUCCESSOR_ARCHIVE_PR = 499
SUCCESSOR_ARCHIVE_SHA = "231aefa91985e5a0520c493c7f0fc3e54d74efc8"
EXPECTED_ACTION_TASKS = ["P7-T1", "P7-T2", "P7-T3", "P7-T4"]
EXPECTED_P0_COMMANDS = [
    "python3 -I scripts/test_garnet_lane0_truth_freeze_status.py",
    "python3 -I scripts/garnet_lane0_truth_freeze_status.py --gate",
]
PR_RE = re.compile(r"\(#(?P<number>[0-9]+)")


def _path(root: Path, value: str) -> Path:
    path = Path(value)
    return path if path.is_absolute() else root / path


def _git_first_parent_prs(
    repo_root: Path, base_exclusive: str, head_inclusive: str
) -> tuple[list[tuple[int, str]], list[str]]:
    proc = subprocess.run(
        [
            "git",
            "-C",
            str(repo_root),
            "log",
            "--first-parent",
            "--reverse",
            "--format=%H%x09%s",
            f"{base_exclusive}..{head_inclusive}",
        ],
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        return [], [f"Git first-parent probe failed: {proc.stderr.strip()}"]

    records: list[tuple[int, str]] = []
    findings: list[str] = []
    for line in proc.stdout.splitlines():
        try:
            sha, subject = line.split("\t", 1)
        except ValueError:
            findings.append(f"malformed Git first-parent row: {line!r}")
            continue
        match = PR_RE.search(subject)
        if match is None:
            findings.append(f"first-parent commit {sha} has no '(#PR)' marker")
            continue
        records.append((int(match.group("number")), sha))
    return records, findings


def _read_json(path: Path, label: str, findings: list[str]) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        findings.append(f"{label} is unreadable or invalid JSON: {exc}")
        return {}
    if not isinstance(value, dict):
        findings.append(f"{label} root must be an object")
        return {}
    return value


def read_status(repo_root: Path, state_path: Path, plan_path: Path) -> dict[str, object]:
    findings: list[str] = []
    state = _read_json(state_path, "mission state", findings)
    plan = _read_json(plan_path, "plan lock", findings)

    actual_archive, git_findings = _git_first_parent_prs(
        repo_root, ARCHIVE_BASE_EXCLUSIVE, ARCHIVE_HEAD_INCLUSIVE
    )
    findings.extend(git_findings)
    actual_order = [pr for pr, _sha in actual_archive]
    actual_mapping = {str(pr): sha for pr, sha in actual_archive}

    checkpoint = state.get("mainlineCheckpoint", {})
    if not isinstance(checkpoint, dict):
        findings.append("mainlineCheckpoint must be an object")
        checkpoint = {}
    archived = checkpoint.get("archivedFirstParentRange", {})
    if not isinstance(archived, dict):
        findings.append("archivedFirstParentRange must be an object")
        archived = {}

    if archived.get("baseExclusiveMainSha") != ARCHIVE_BASE_EXCLUSIVE:
        findings.append("archived range baseExclusiveMainSha does not match the frozen claim")
    if archived.get("headInclusiveMainSha") != ARCHIVE_HEAD_INCLUSIVE:
        findings.append("archived range headInclusiveMainSha does not match the frozen claim")
    if archived.get("firstParentPullRequestCommitCount") != len(actual_archive):
        findings.append("archived first-parent PR count does not match Git-derived history")
    if archived.get("firstParentPullRequestMergeOrder") != actual_order:
        findings.append("archived first-parent PR merge order does not match Git-derived history")
    if archived.get("pullRequestToMainSha") != actual_mapping:
        findings.append("archived PR-to-main SHA mapping does not match Git-derived SHA mapping")

    successor = checkpoint.get("successorArchiveMerge", {})
    if not isinstance(successor, dict):
        findings.append("successorArchiveMerge must be an object")
        successor = {}
    if successor.get("pullRequest") != SUCCESSOR_ARCHIVE_PR:
        findings.append("successor archive PR must be #499")
    if successor.get("mainSha") != SUCCESSOR_ARCHIVE_SHA:
        findings.append("successor archive main SHA must be 231aefa")
    if successor.get("immediatelyFollowsArchivedRange") is not True:
        findings.append("successor archive must declare that it immediately follows the range")

    actual_successor, successor_findings = _git_first_parent_prs(
        repo_root, ARCHIVE_HEAD_INCLUSIVE, SUCCESSOR_ARCHIVE_SHA
    )
    findings.extend(successor_findings)
    expected_successor = [(SUCCESSOR_ARCHIVE_PR, SUCCESSOR_ARCHIVE_SHA)]
    if actual_successor != expected_successor:
        findings.append("Git history does not contain #499 as the sole successor archive merge")

    full_order = actual_order + [SUCCESSOR_ARCHIVE_PR]
    if checkpoint.get("checkpointPullRequestCommitCount") != len(full_order):
        findings.append("full checkpoint PR count does not include range plus successor")
    if checkpoint.get("checkpointFirstParentPullRequestMergeOrder") != full_order:
        findings.append("full checkpoint merge order does not match Git range plus successor")

    phases = state.get("phases", [])
    if not isinstance(phases, list):
        phases = []
        findings.append("phases must be an array")
    phase_by_id = {
        phase.get("id"): phase
        for phase in phases
        if isinstance(phase, dict) and isinstance(phase.get("id"), str)
    }
    resume = state.get("resume", {})
    if not isinstance(resume, dict):
        resume = {}
        findings.append("resume must be an object")
    active_phase = resume.get("activePhase")
    if active_phase != "P7":
        findings.append("resume.activePhase must be exactly P7")
    if active_phase not in phase_by_id:
        findings.append(f"resume.activePhase {active_phase!r} is not materialized in phases[]")

    p7 = phase_by_id.get("P7", {})
    if not isinstance(p7, dict):
        p7 = {}
    tasks = p7.get("tasks", [])
    if not isinstance(tasks, list):
        tasks = []
        findings.append("P7 tasks must be an array")
    task_by_id = {
        task.get("id"): task
        for task in tasks
        if isinstance(task, dict) and isinstance(task.get("id"), str)
    }
    for task_id in EXPECTED_ACTION_TASKS:
        if task_id not in task_by_id:
            findings.append(f"materialized P7 is missing required task {task_id}")

    actions = resume.get("nextActions", [])
    if not isinstance(actions, list):
        actions = []
        findings.append("resume.nextActions must be an array")
    action_tasks: list[str] = []
    for action in actions:
        match = re.match(r"^(P7-T[1-4]):", action) if isinstance(action, str) else None
        if match is not None:
            action_tasks.append(match.group(1))
    if action_tasks != EXPECTED_ACTION_TASKS or len(actions) != len(EXPECTED_ACTION_TASKS):
        findings.append("resume.nextActions must map exactly once to P7-T1 through P7-T4")
    for task_id in action_tasks:
        if task_id not in task_by_id:
            findings.append(f"resume action references missing task {task_id}")

    if re.search(r"\bP(?:8|9|10)(?:\b|-)", json.dumps(resume, sort_keys=True)):
        findings.append("resume contains a stale P8/P9/P10 reference")
    for phase in phases:
        if isinstance(phase, dict) and phase.get("status") == "in-progress":
            findings.append(f"phase {phase.get('id')} uses unsupported in-progress status")
        if isinstance(phase, dict):
            for task in phase.get("tasks", []):
                if isinstance(task, dict) and task.get("status") == "in-progress":
                    findings.append(f"task {task.get('id')} uses unsupported in-progress status")

    verdict = plan.get("verdict", {})
    if not isinstance(verdict, dict):
        verdict = {}
        findings.append("plan verdict must be an object")
    if verdict.get("nbcli") != "warning":
        findings.append("plan verdict nbcli must remain warning")
    if verdict.get("adversarialFindingsResolved") is not False:
        findings.append("plan adversarialFindingsResolved must remain false while High findings are open")

    plan_phases = plan.get("phases", [])
    if not isinstance(plan_phases, list):
        plan_phases = []
    p0 = next(
        (
            phase
            for phase in plan_phases
            if isinstance(phase, dict) and phase.get("id") == "P0"
        ),
        {},
    )
    gates = p0.get("gates", []) if isinstance(p0, dict) else []
    commands = [
        gate.get("command")
        for gate in gates
        if isinstance(gate, dict) and isinstance(gate.get("command"), str)
    ]
    if commands != EXPECTED_P0_COMMANDS:
        findings.append("frozen P0 gates must run the durable checker tests and gate")

    return {
        "schema": "garnet.lane0_truth_freeze/v1",
        "archive_base_exclusive": ARCHIVE_BASE_EXCLUSIVE,
        "archive_head_inclusive": ARCHIVE_HEAD_INCLUSIVE,
        "archive_pr_count": len(actual_archive),
        "successor_archive_pr": SUCCESSOR_ARCHIVE_PR,
        "successor_archive_sha": SUCCESSOR_ARCHIVE_SHA,
        "checkpoint_pr_count": len(full_order),
        "active_phase": active_phase,
        "action_tasks": action_tasks,
        "adversarial_findings_resolved": verdict.get("adversarialFindingsResolved"),
        "findings": findings,
        "ok": not findings,
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", default=str(ROOT))
    parser.add_argument("--state", default="ops/mission/state.json")
    parser.add_argument("--plan", default="ops/lane0/plan.lock.json")
    parser.add_argument("--gate", action="store_true")
    args = parser.parse_args(list(argv) if argv is not None else None)

    repo_root = Path(args.repo_root).resolve()
    status = read_status(
        repo_root,
        _path(repo_root, args.state),
        _path(repo_root, args.plan),
    )
    print(json.dumps(status, indent=2))
    if args.gate and not status["ok"]:
        print(
            f"Lane 0 truth-freeze gate FAILED: {len(status['findings'])} finding(s)",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
