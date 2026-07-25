#!/usr/bin/env python3
"""Fail-closed inventory of immutable external GitHub Action references."""
from __future__ import annotations

import argparse
import importlib.util
import json
import os
import re
import stat
import sys
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MANIFEST_PATH = Path(".github/rulesets/external-action-pins.json")
MANIFEST_SCHEMA = "garnet.github_action_pins/v1"
STATUS_SCHEMA = "garnet.workflow_action_integrity/v1"
MAX_MANIFEST_BYTES = 128 * 1024
ACTION_RE = re.compile(
    r"(?P<action>[A-Za-z0-9_.-]+(?:/[A-Za-z0-9_.-]+)+)@(?P<commit>[0-9a-f]{40})"
)
ACTION_NAME_RE = re.compile(r"[A-Za-z0-9_.-]+(?:/[A-Za-z0-9_.-]+)+")
SHA_RE = re.compile(r"[0-9a-f]{40}")
REF_RE = re.compile(r"[A-Za-z0-9][A-Za-z0-9._/-]{0,127}")
ENTRY_KEYS = {"action", "commit", "resolved_at", "source_kind", "source_ref"}


@dataclass(frozen=True)
class ActionPin:
    action: str
    commit: str
    resolved_at: str
    source_kind: str
    source_ref: str


@dataclass
class ActionIntegrityStatus:
    schema: str = STATUS_SCHEMA
    manifest: str = MANIFEST_PATH.as_posix()
    workflow_count: int = 0
    occurrence_count: int = 0
    distinct_action_count: int = 0
    manifest_entry_count: int = 0
    mutable_count: int = 0
    credited_occurrences: int = 0
    findings: list[str] = field(default_factory=list)
    ok: bool = False


def _load_schema_policy() -> object:
    path = Path(__file__).with_name("garnet_workflow_schema_policy.py")
    spec = importlib.util.spec_from_file_location(
        "_garnet_workflow_schema_policy_action_integrity", path
    )
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load workflow schema policy")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def _no_duplicates(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key {key!r}")
        result[key] = value
    return result


def _regular_bytes(path: Path, root: Path) -> bytes:
    absolute_root = root.resolve(strict=True)
    lexical = absolute_root / path
    for component in (absolute_root / ".github", lexical.parent, lexical):
        metadata = os.lstat(component)
        if stat.S_ISLNK(metadata.st_mode) or bool(
            getattr(metadata, "st_file_attributes", 0) & 0x400
        ):
            raise ValueError(
                f"manifest path contains symlink/reparse point: {component}"
            )
    before = os.lstat(lexical)
    if not stat.S_ISREG(before.st_mode) or before.st_size > MAX_MANIFEST_BYTES:
        raise ValueError("action-pin manifest must be a bounded regular file")
    flags = os.O_RDONLY | getattr(os, "O_BINARY", 0) | getattr(os, "O_NOFOLLOW", 0)
    descriptor = os.open(lexical, flags)
    try:
        opened = os.fstat(descriptor)
        if not stat.S_ISREG(opened.st_mode) or (opened.st_dev, opened.st_ino) != (
            before.st_dev,
            before.st_ino,
        ):
            raise ValueError("action-pin manifest identity changed while opening")
        payload = bytearray()
        while len(payload) <= MAX_MANIFEST_BYTES:
            chunk = os.read(descriptor, MAX_MANIFEST_BYTES + 1 - len(payload))
            if not chunk:
                break
            payload.extend(chunk)
        after = os.fstat(descriptor)
        identity = (opened.st_dev, opened.st_ino, opened.st_size, opened.st_mtime_ns)
        if identity != (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns):
            raise ValueError("action-pin manifest changed while being read")
        if len(payload) > MAX_MANIFEST_BYTES or len(payload) != opened.st_size:
            raise ValueError("action-pin manifest exceeds its byte boundary")
        return bytes(payload)
    finally:
        os.close(descriptor)


def _canonical_json(value: object) -> bytes:
    return (
        json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    ).encode("utf-8")


def _timestamp(value: object) -> bool:
    if not isinstance(value, str) or not re.fullmatch(
        r"20[0-9]{2}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z", value
    ):
        return False
    try:
        observed = datetime.strptime(value, "%Y-%m-%dT%H:%M:%SZ").replace(
            tzinfo=timezone.utc
        )
    except ValueError:
        return False
    return observed <= datetime.now(timezone.utc)


def _manifest(root: Path) -> tuple[dict[tuple[str, str], ActionPin], list[str]]:
    findings: list[str] = []
    try:
        raw = _regular_bytes(MANIFEST_PATH, root)
        value = json.loads(
            raw.decode("utf-8", errors="strict"), object_pairs_hook=_no_duplicates
        )
        if raw != _canonical_json(value):
            raise ValueError("action-pin manifest must be canonical JSON")
    except (OSError, UnicodeError, json.JSONDecodeError, RecursionError, ValueError) as exc:
        return {}, [f"cannot read action-pin manifest: {exc}"]
    if not isinstance(value, dict) or set(value) != {"schema", "entries"}:
        return {}, ["action-pin manifest top-level keys are not exact"]
    if value.get("schema") != MANIFEST_SCHEMA:
        findings.append(f"action-pin manifest schema must be {MANIFEST_SCHEMA}")
    rows = value.get("entries")
    if not isinstance(rows, list) or not rows:
        return {}, [*findings, "action-pin manifest entries must be non-empty"]
    pins: dict[tuple[str, str], ActionPin] = {}
    sources: set[tuple[str, str]] = set()
    for index, row in enumerate(rows):
        if not isinstance(row, dict) or set(row) != ENTRY_KEYS:
            findings.append(f"manifest entry keys are not exact at index {index}")
            continue
        action = row.get("action")
        commit = row.get("commit")
        resolved_at = row.get("resolved_at")
        source_kind = row.get("source_kind")
        source_ref = row.get("source_ref")
        if not isinstance(action, str) or ACTION_NAME_RE.fullmatch(action) is None:
            findings.append(f"manifest action is not canonical at index {index}")
            continue
        if not isinstance(commit, str) or SHA_RE.fullmatch(commit) is None:
            findings.append(f"manifest commit is not a full lowercase SHA at index {index}")
            continue
        if not _timestamp(resolved_at):
            findings.append(f"manifest resolution timestamp is invalid at index {index}")
            continue
        if source_kind not in {"tag", "branch"}:
            findings.append(f"manifest source_kind is invalid at index {index}")
            continue
        if not isinstance(source_ref, str) or REF_RE.fullmatch(source_ref) is None:
            findings.append(f"manifest source_ref is invalid at index {index}")
            continue
        key = (action, commit)
        source_key = (action, source_ref)
        if key in pins or source_key in sources:
            findings.append(f"duplicate manifest pin {action!r} at index {index}")
            continue
        pins[key] = ActionPin(action, commit, resolved_at, source_kind, source_ref)
        sources.add(source_key)
    return ({} if findings else pins), findings


def _uses(
    projection: object,
) -> tuple[list[tuple[str, str, dict[str, str]]], list[str]]:
    problems = list(getattr(projection, "problems", ()))
    workflows = tuple(getattr(projection, "workflows", ()))
    if problems:
        return [], problems
    if not workflows:
        return [], ["workflow projection is unexpectedly empty"]
    uses: list[tuple[str, str, dict[str, str]]] = []
    for workflow in workflows:
        relative = workflow.source.relative
        for job in workflow.jobs:
            for index, step in enumerate(job.steps):
                values = dict(step.items)
                raw = values.get("uses")
                if raw is None:
                    continue
                value = getattr(raw, "value", None)
                if not isinstance(value, str):
                    problems.append(
                        f"{relative}:{job.job_id}:steps[{index}] uses is not text"
                    )
                    continue
                inputs: dict[str, str] = {}
                raw_inputs = values.get("with")
                if raw_inputs is not None:
                    for key, raw_value in getattr(raw_inputs, "items", ()):
                        input_value = getattr(raw_value, "value", None)
                        if isinstance(key, str) and isinstance(input_value, str):
                            inputs[key] = input_value
                        else:
                            problems.append(
                                f"{relative}:{job.job_id}:steps[{index}] with is malformed"
                            )
                uses.append(
                    (f"{relative}:{job.job_id}:steps[{index}]", value, inputs)
                )
    if not uses and not problems:
        problems.append("workflow projection contains no external action references")
    return ([] if problems else uses), problems


def read_status(root: Path = ROOT) -> ActionIntegrityStatus:
    result = ActionIntegrityStatus()
    pins, manifest_findings = _manifest(root)
    result.manifest_entry_count = len(pins)
    try:
        projection = _load_schema_policy().workflow_projection(root)
        workflows = tuple(getattr(projection, "workflows", ()))
        result.workflow_count = len(workflows)
        occurrences, projection_findings = _uses(projection)
    except Exception as exc:  # fail closed when the dependency/parser cannot load
        occurrences, projection_findings = [], [f"workflow projection failed: {exc}"]
    result.occurrence_count = len(occurrences)
    actions: set[str] = set()
    used_pins: set[tuple[str, str]] = set()
    findings = [*manifest_findings, *projection_findings]
    mutable = 0
    for label, value, inputs in occurrences:
        match = ACTION_RE.fullmatch(value)
        if match is None:
            mutable += 1
            findings.append(
                f"{label} must pin an external action to a full 40-character lowercase SHA"
            )
            continue
        action, commit = match.group("action"), match.group("commit")
        actions.add(action)
        if action == "dtolnay/rust-toolchain":
            toolchain = inputs.get("toolchain")
            if (
                not isinstance(toolchain, str)
                or not toolchain
                or len(toolchain) > 128
                or "${{" in toolchain
                or "}}" in toolchain
            ):
                findings.append(
                    f"{label} SHA-pinned rust-toolchain requires an explicit toolchain input"
                )
        action_pins = {
            pin_commit for pin_action, pin_commit in pins if pin_action == action
        }
        if not action_pins:
            findings.append(f"{label} action {action!r} is absent from the reviewed manifest")
        elif commit not in action_pins:
            findings.append(
                f"{label} action {action!r} does not match the reviewed manifest"
            )
        else:
            used_pins.add((action, commit))
    result.distinct_action_count = len(actions)
    result.mutable_count = mutable
    for action, commit in sorted(set(pins) - used_pins):
        findings.append(f"unused manifest entry {action!r}@{commit}")
    result.findings = findings
    result.ok = not findings and bool(occurrences)
    result.credited_occurrences = len(occurrences) if result.ok else 0
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--gate", action="store_true")
    parser.add_argument("--format", choices=("json",), default="json")
    args = parser.parse_args()
    result = read_status()
    print(json.dumps(asdict(result), indent=2, sort_keys=True))
    return 1 if args.gate and not result.ok else 0


if __name__ == "__main__":
    raise SystemExit(main())
