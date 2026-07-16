#!/usr/bin/env python3
"""Validate the Lane 0 evidence-tied frozen backlog without promoting claims."""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parents[1]
BACKLOG_PATH = Path("ops/lane0/frozen-backlog.json")
SCHEMA = "garnet.lane0.frozen_backlog/v1"
EXPECTED_BASE_SHA = "231aefa91985e5a0520c493c7f0fc3e54d74efc8"
ALLOWED_STATES = {"implemented", "partial", "planned", "research"}
EXPECTED_IDS = {
    "LANE-1-ITEM-1",
    "LANE-2A",
    "LANE-2B",
    "LANE-2C",
    "WV-6",
    "WV-7",
    "U-15",
    "QWATCH",
}
ID_RE = re.compile(r"^[A-Z0-9]+(?:-[A-Z0-9]+)*$")
MAX_BYTES = 256 * 1024


@dataclass
class FrozenBacklogStatus:
    schema: str
    exact_base_main_sha: str | None
    entry_count: int
    implemented_clause_count: int
    states: dict[str, int]
    findings: list[str] = field(default_factory=list)
    ok: bool = False


def _reject_duplicates(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key {key!r}")
        result[key] = value
    return result


def load_document(root: Path = ROOT) -> dict[str, object]:
    path = root / BACKLOG_PATH
    if path.is_symlink() or not path.is_file():
        raise ValueError(f"{BACKLOG_PATH} is not a regular file")
    if path.stat().st_size <= 0 or path.stat().st_size > MAX_BYTES:
        raise ValueError(f"{BACKLOG_PATH} exceeds the bounded JSON size")
    try:
        value = json.loads(
            path.read_text(encoding="utf-8"),
            object_pairs_hook=_reject_duplicates,
        )
    except (OSError, UnicodeError, json.JSONDecodeError, ValueError) as exc:
        raise ValueError(f"{BACKLOG_PATH} is invalid strict JSON: {exc}") from exc
    if not isinstance(value, dict):
        raise ValueError(f"{BACKLOG_PATH} root must be an object")
    return value


def _relative_path(value: object, *, allow_trailing_slash: bool = False) -> str:
    if not isinstance(value, str) or not value or "\\" in value or "\n" in value:
        raise ValueError("path must be a non-empty single-line POSIX path")
    raw = value[:-1] if allow_trailing_slash and value.endswith("/") else value
    path = PurePosixPath(raw)
    if path.is_absolute() or "." in path.parts or ".." in path.parts:
        raise ValueError(f"path escapes the repository: {value!r}")
    if path.as_posix() != raw:
        raise ValueError(f"path is not canonical: {value!r}")
    return raw


def _tracked(root: Path, relative: str) -> bool:
    path = root / relative
    if path.is_file():
        args = ["git", "-C", str(root), "ls-files", "--error-unmatch", "--", relative]
    elif path.is_dir():
        args = ["git", "-C", str(root), "ls-files", "--", f"{relative}/"]
    else:
        return False
    proc = subprocess.run(args, text=True, capture_output=True, check=False)
    return proc.returncode == 0 and bool(proc.stdout.strip())


def _check_existing_path(
    value: object,
    *,
    root: Path,
    label: str,
    findings: list[str],
    verify_git: bool,
) -> None:
    try:
        relative = _relative_path(value)
    except ValueError as exc:
        findings.append(f"{label}: {exc}")
        return
    if not (root / relative).exists():
        findings.append(f"{label}: {relative} does not exist")
    elif verify_git and not _tracked(root, relative):
        findings.append(f"{label}: {relative} is not tracked in Git")


def _string_list(value: object, label: str, findings: list[str]) -> list[str]:
    if (
        not isinstance(value, list)
        or not value
        or any(not isinstance(item, str) or not item.strip() or "\n" in item for item in value)
    ):
        findings.append(f"{label} must be a non-empty array of single-line strings")
        return []
    return value


def validate_document(
    document: dict[str, object],
    root: Path = ROOT,
    *,
    verify_git: bool = True,
) -> list[str]:
    findings: list[str] = []
    if document.get("schema") != SCHEMA:
        findings.append(f"schema must be {SCHEMA}")
    if document.get("exactBaseMainSha") != EXPECTED_BASE_SHA:
        findings.append("exactBaseMainSha is not the Lane 0 pin")
    if document.get("launchStatus") != "HOLD":
        findings.append("launchStatus must remain HOLD")
    if document.get("allowedClaimStates") != sorted(ALLOWED_STATES):
        findings.append("allowedClaimStates is not the canonical state set")

    entries = document.get("entries")
    if not isinstance(entries, list):
        return findings + ["entries must be an array"]
    seen: set[str] = set()
    for index, entry in enumerate(entries):
        label = f"entries[{index}]"
        if not isinstance(entry, dict):
            findings.append(f"{label} must be an object")
            continue
        identifier = entry.get("id")
        if not isinstance(identifier, str) or ID_RE.fullmatch(identifier) is None:
            findings.append(f"{label}.id is not canonical")
            identifier = label
        elif identifier in seen:
            findings.append(f"duplicate backlog id {identifier}")
        else:
            seen.add(identifier)

        state = entry.get("claimState")
        if state not in ALLOWED_STATES:
            findings.append(f"{identifier}: unsupported claimState {state!r}")
        if entry.get("exactBaseMainSha") != EXPECTED_BASE_SHA:
            findings.append(f"{identifier}: exactBaseMainSha is not the Lane 0 pin")
        if not isinstance(entry.get("title"), str) or not entry["title"].strip():
            findings.append(f"{identifier}: title must be non-empty")
        command = entry.get("acceptanceCommand")
        if not isinstance(command, str) or not command.strip() or "\n" in command:
            findings.append(f"{identifier}: acceptanceCommand must be one exact command")
        _string_list(entry.get("jonOnlyActions"), f"{identifier}.jonOnlyActions", findings)

        authorities = entry.get("authoritySources")
        if not isinstance(authorities, list) or not authorities:
            findings.append(f"{identifier}: authoritySources must be non-empty")
        else:
            for authority_index, authority in enumerate(authorities):
                authority_label = f"{identifier}.authoritySources[{authority_index}]"
                if not isinstance(authority, dict):
                    findings.append(f"{authority_label} must be an object")
                    continue
                kind = authority.get("kind")
                if kind == "repository":
                    _check_existing_path(
                        authority.get("path"),
                        root=root,
                        label=authority_label,
                        findings=findings,
                        verify_git=verify_git,
                    )
                elif kind == "jon-directive":
                    reference = authority.get("reference")
                    if not isinstance(reference, str) or not reference.strip():
                        findings.append(f"{authority_label} requires a reference")
                else:
                    findings.append(f"{authority_label} has unsupported kind {kind!r}")

        evidence_files = entry.get("evidenceFiles")
        if not isinstance(evidence_files, list) or not evidence_files:
            findings.append(f"{identifier}: evidenceFiles must be non-empty")
        else:
            for evidence_index, evidence in enumerate(evidence_files):
                _check_existing_path(
                    evidence,
                    root=root,
                    label=f"{identifier}.evidenceFiles[{evidence_index}]",
                    findings=findings,
                    verify_git=verify_git,
                )

        implemented = entry.get("implementedClauses")
        opened = entry.get("openClauses")
        if not isinstance(implemented, list) or not isinstance(opened, list):
            findings.append(f"{identifier}: clause arrays are required")
            continue
        if state == "partial" and (not implemented or not opened):
            findings.append(
                f"{identifier}: partial entries require implementedClauses and openClauses"
            )
        if state == "implemented" and (not implemented or opened):
            findings.append(
                f"{identifier}: implemented entries require implemented clauses and no open clauses"
            )
        if state in {"planned", "research"} and implemented:
            findings.append(
                f"{identifier}: {state} entries cannot carry implementedClauses"
            )

        clause_ids: set[str] = set()
        for clause_index, clause in enumerate(implemented):
            clause_label = f"{identifier}.implementedClauses[{clause_index}]"
            if not isinstance(clause, dict):
                findings.append(f"{clause_label} must be an object")
                continue
            clause_id = clause.get("id")
            if not isinstance(clause_id, str) or not clause_id:
                findings.append(f"{clause_label}.id must be non-empty")
            elif clause_id in clause_ids:
                findings.append(f"{identifier}: duplicate clause id {clause_id}")
            else:
                clause_ids.add(clause_id)
            if not isinstance(clause.get("claim"), str) or not clause["claim"].strip():
                findings.append(f"{clause_label}.claim must be non-empty")
            for key in ("codePaths", "evidencePaths"):
                values = clause.get(key)
                if not isinstance(values, list) or not values:
                    findings.append(f"{clause_label}.{key} must be non-empty")
                    continue
                for path_index, value in enumerate(values):
                    _check_existing_path(
                        value,
                        root=root,
                        label=f"{clause_label}.{key}[{path_index}]",
                        findings=findings,
                        verify_git=verify_git,
                    )
        for clause_index, clause in enumerate(opened):
            clause_label = f"{identifier}.openClauses[{clause_index}]"
            if (
                not isinstance(clause, dict)
                or not isinstance(clause.get("id"), str)
                or not isinstance(clause.get("claim"), str)
                or not clause["claim"].strip()
            ):
                findings.append(f"{clause_label} must name an id and claim")

        future = entry.get("futureEvidence")
        if not isinstance(future, list) or not future:
            findings.append(f"{identifier}: futureEvidence must be non-empty")
        else:
            for future_index, destination in enumerate(future):
                destination_label = f"{identifier}.futureEvidence[{future_index}]"
                if not isinstance(destination, dict):
                    findings.append(f"{destination_label} must be an object")
                    continue
                if destination.get("status") != "future-not-evidence":
                    findings.append(
                        f"{destination_label}.status must be future-not-evidence"
                    )
                try:
                    _relative_path(
                        destination.get("path"),
                        allow_trailing_slash=True,
                    )
                except ValueError as exc:
                    findings.append(f"{destination_label}: {exc}")
                if (
                    not isinstance(destination.get("purpose"), str)
                    or not destination["purpose"].strip()
                ):
                    findings.append(f"{destination_label}.purpose must be non-empty")

    if seen != EXPECTED_IDS:
        findings.append(
            "backlog ids must be exactly " + ", ".join(sorted(EXPECTED_IDS))
        )
    return findings


def read_status(root: Path = ROOT, *, verify_git: bool = True) -> FrozenBacklogStatus:
    try:
        document = load_document(root)
    except ValueError as exc:
        return FrozenBacklogStatus(
            schema=SCHEMA,
            exact_base_main_sha=None,
            entry_count=0,
            implemented_clause_count=0,
            states={state: 0 for state in sorted(ALLOWED_STATES)},
            findings=[str(exc)],
            ok=False,
        )
    entries = document.get("entries") if isinstance(document.get("entries"), list) else []
    states = {state: 0 for state in sorted(ALLOWED_STATES)}
    implemented_count = 0
    for entry in entries:
        if not isinstance(entry, dict):
            continue
        state = entry.get("claimState")
        if state in states:
            states[state] += 1
        clauses = entry.get("implementedClauses")
        if isinstance(clauses, list):
            implemented_count += len(clauses)
    findings = validate_document(document, root, verify_git=verify_git)
    return FrozenBacklogStatus(
        schema=SCHEMA,
        exact_base_main_sha=document.get("exactBaseMainSha")
        if isinstance(document.get("exactBaseMainSha"), str)
        else None,
        entry_count=len(entries),
        implemented_clause_count=implemented_count,
        states=states,
        findings=findings,
        ok=not findings,
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--gate", action="store_true")
    args = parser.parse_args(argv)
    status = read_status()
    print(json.dumps(asdict(status), indent=2))
    if args.gate and not status.ok:
        print("frozen backlog gate FAILED", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
