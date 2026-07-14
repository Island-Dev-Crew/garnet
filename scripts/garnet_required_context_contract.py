#!/usr/bin/env python3
"""Strict declarative schema for Garnet required-context producers."""
from __future__ import annotations

import json
import os
import re
import stat
import unicodedata
from collections import Counter
from dataclasses import dataclass, field
from pathlib import Path


SCHEMA = "garnet.required-context-producers/v1"
INVENTORY_PATH = ".github/rulesets/required-context-producers.json"
TARGET_BRANCH = "main"
MAX_INVENTORY_BYTES = 256 * 1024
FILE_ATTRIBUTE_REPARSE_POINT = 0x400
ID_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_-]*$")
WORKFLOW_RE = re.compile(r"^\.github/workflows/[a-z0-9_.-]+\.(?:yml|yaml)$")
MATRIX_VALUE_RE = re.compile(r"^[A-Za-z0-9_.-]+$")


@dataclass(frozen=True)
class Producer:
    context: str
    workflow: str
    event: str
    job: str
    matrix: tuple[str, str] | None = None


@dataclass
class ProducerInventory:
    producers: list[Producer] = field(default_factory=list)
    optional_contexts: set[str] = field(default_factory=set)
    target_branch: str = ""
    problems: list[str] = field(default_factory=list)


@dataclass(frozen=True)
class ProducerBinding:
    producer: Producer
    workflow: object
    event: object
    occurrence: object
    dependency_contexts: tuple[str, ...]


@dataclass(frozen=True)
class ProducerEvaluation:
    bindings: tuple[ProducerBinding, ...] = ()
    prepared_optional: tuple[ProducerBinding, ...] = ()
    inactive_optional: tuple[Producer, ...] = ()
    problems: tuple[str, ...] = ()


def _no_duplicates(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key {key!r}")
        result[key] = value
    return result


def _is_reparse(metadata: os.stat_result) -> bool:
    return stat.S_ISLNK(metadata.st_mode) or bool(
        getattr(metadata, "st_file_attributes", 0) & FILE_ATTRIBUTE_REPARSE_POINT
    )


def _read_regular_utf8(path: Path) -> str:
    """Read one bounded regular file without accepting path indirection."""
    absolute = path.absolute()
    for component in [*reversed(absolute.parents), absolute]:
        try:
            metadata = os.lstat(component)
        except OSError as exc:
            raise ValueError(f"cannot inspect inventory path: {exc}") from exc
        if _is_reparse(metadata):
            raise ValueError(f"inventory path contains symlink/reparse point: {component}")
    leaf = os.lstat(absolute)
    if not stat.S_ISREG(leaf.st_mode):
        raise ValueError("producer inventory is not a regular file")
    flags = os.O_RDONLY | getattr(os, "O_BINARY", 0) | getattr(os, "O_NOFOLLOW", 0)
    try:
        descriptor = os.open(absolute, flags)
    except OSError as exc:
        raise ValueError(f"cannot open producer inventory: {exc}") from exc
    try:
        opened = os.fstat(descriptor)
        if not stat.S_ISREG(opened.st_mode) or (leaf.st_dev, leaf.st_ino) != (
            opened.st_dev,
            opened.st_ino,
        ):
            raise ValueError("producer inventory identity changed while opening")
        if opened.st_size > MAX_INVENTORY_BYTES:
            raise ValueError("producer inventory exceeds size limit")
        payload = bytearray()
        while len(payload) <= MAX_INVENTORY_BYTES:
            chunk = os.read(descriptor, MAX_INVENTORY_BYTES + 1 - len(payload))
            if not chunk:
                break
            payload.extend(chunk)
        after = os.fstat(descriptor)
        stable = (opened.st_dev, opened.st_ino, opened.st_size, opened.st_mtime_ns)
        if stable != (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns):
            raise ValueError("producer inventory changed while reading")
        if len(payload) > MAX_INVENTORY_BYTES:
            raise ValueError("producer inventory exceeds size limit")
        return bytes(payload).decode("utf-8")
    finally:
        os.close(descriptor)


def _canonical_context(value: str) -> bool:
    return (
        value == value.strip()
        and value.isprintable()
        and unicodedata.normalize("NFC", value) == value
        and not any(unicodedata.category(char) in {"Cc", "Cf"} for char in value)
    )


def load_inventory(path: Path) -> ProducerInventory:
    """Load an exact, duplicate-key-free producer inventory."""
    try:
        raw = json.loads(
            _read_regular_utf8(path), object_pairs_hook=_no_duplicates
        )
    except (OSError, UnicodeError, json.JSONDecodeError, ValueError) as exc:
        return ProducerInventory(problems=[f"cannot read producer inventory: {exc}"])
    if not isinstance(raw, dict) or set(raw) != {
        "schema",
        "target_branch",
        "optional_contexts",
        "producers",
    }:
        return ProducerInventory(
            problems=["producer inventory top-level keys are not exact"]
        )
    problems: list[str] = []
    if raw.get("schema") != SCHEMA:
        problems.append(f"producer inventory schema must be {SCHEMA!r}")
    branch = raw.get("target_branch")
    if branch != TARGET_BRANCH:
        problems.append(f"producer inventory target_branch must be {TARGET_BRANCH!r}")
        branch = ""
    optional_raw = raw.get("optional_contexts")
    if (
        not isinstance(optional_raw, list)
        or not all(isinstance(item, str) and item for item in optional_raw)
        or len(set(optional_raw)) != len(optional_raw)
    ):
        problems.append("optional_contexts must be a unique non-empty string list")
        optional: set[str] = set()
    else:
        optional = set(optional_raw)
    rows = raw.get("producers")
    if not isinstance(rows, list) or not rows:
        return ProducerInventory(
            optional_contexts=optional,
            target_branch=branch,
            problems=[*problems, "producers must be a non-empty list"],
        )
    producers: list[Producer] = []
    seen: set[str] = set()
    base_keys = {"context", "workflow", "event", "job"}
    for index, row in enumerate(rows):
        if not isinstance(row, dict) or set(row) not in (base_keys, base_keys | {"matrix"}):
            problems.append(f"producers[{index}] keys are not exact")
            continue
        identity = tuple(row.get(key) for key in ("context", "workflow", "event", "job"))
        if not all(isinstance(item, str) and item for item in identity):
            problems.append(
                f"producers[{index}] identity fields must be non-empty strings"
            )
            continue
        context, workflow, event, job = identity
        if not _canonical_context(context):
            problems.append(f"producers[{index}] context is not canonical text")
        if context in seen:
            problems.append(f"duplicate inventory context {context!r}")
        seen.add(context)
        if not WORKFLOW_RE.fullmatch(workflow):
            problems.append(
                f"producers[{index}] workflow path is not canonical lowercase YAML"
            )
        if event not in {"pull_request", "pull_request_target"}:
            problems.append(
                f"producers[{index}] event is not an approved pull-request event"
            )
        if not ID_RE.fullmatch(job):
            problems.append(f"producers[{index}] job id is not canonical")
        matrix: tuple[str, str] | None = None
        if "matrix" in row:
            value = row["matrix"]
            if not isinstance(value, dict) or len(value) != 1:
                problems.append(
                    f"producers[{index}] matrix must have exactly one binding"
                )
            else:
                axis, member = next(iter(value.items()))
                if (
                    not isinstance(axis, str)
                    or not ID_RE.fullmatch(axis)
                    or not isinstance(member, str)
                    or not MATRIX_VALUE_RE.fullmatch(member)
                ):
                    problems.append(f"producers[{index}] matrix binding is invalid")
                else:
                    matrix = (axis, member)
        producers.append(Producer(context, workflow, event, job, matrix))
    if optional - seen:
        problems.append("optional_contexts names contexts absent from producers")
    return ProducerInventory(producers, optional, branch, problems)


def _matrix_binding(occurrence: object) -> tuple[str, str] | None:
    binding = occurrence.binding
    return None if binding is None else (binding[0], binding[1].value)


def _producer_event(workflow: object, producer: Producer, problems: list[str]) -> object | None:
    label = f"{producer.workflow}:{producer.job}"
    events = [item for item in workflow.events if item.name in {"pull_request", "pull_request_target"}]
    if len(events) != 1:
        problems.append(f"{label} must have exactly one PR-class event")
        return None
    event = events[0]
    if event.name != producer.event:
        problems.append(f"{label} producer identity event mismatch")
    filters = tuple((key, tuple(value.value for value in values)) for key, values in event.filters)
    safe = not filters or filters == (("branches", (TARGET_BRANCH,)),)
    if not safe:
        problems.append(f"{label} PR event must be unfiltered or exact branches [{TARGET_BRANCH}]")
    return event


def _dependency_jobs(workflow: object, job: object, problems: list[str]) -> tuple[object, ...]:
    by_id = {item.job_id: item for item in workflow.jobs}
    ordered: list[object] = []
    seen: set[str] = set()
    visiting: set[str] = set()

    def visit(job_id: str) -> None:
        if job_id in seen:
            return
        if job_id in visiting or job_id not in by_id:
            problems.append(f"{workflow.source.relative}:{job.job_id} has an invalid dependency {job_id!r}")
            return
        visiting.add(job_id)
        dependency = by_id[job_id]
        for raw in dependency.needs:
            visit(raw.value)
        visiting.remove(job_id)
        seen.add(job_id)
        ordered.append(dependency)

    for raw in job.needs:
        visit(raw.value)
    return tuple(ordered)


def evaluate_producer_availability(
    inventory: ProducerInventory, projection: object
) -> ProducerEvaluation:
    """Bind declared producers to immutable projected occurrences, all-or-zero."""
    problems = [*inventory.problems, *getattr(projection, "problems", ())]
    if inventory.target_branch != TARGET_BRANCH:
        problems.append(f"producer inventory target_branch must be {TARGET_BRANCH!r}")
    if not inventory.producers:
        problems.append("producer inventory is unexpectedly empty")
    workflows = tuple(getattr(projection, "workflows", ()))
    if not workflows and not problems:
        problems.append("workflow projection is unexpectedly empty")
    if problems:
        return ProducerEvaluation(problems=tuple(problems))

    occurrences: list[tuple[object, object]] = [
        (workflow, occurrence)
        for workflow in workflows
        for occurrence in workflow.contexts
    ]
    by_context: dict[str, list[tuple[object, object]]] = {}
    for item in occurrences:
        by_context.setdefault(item[1].context, []).append(item)
    for context, matches in by_context.items():
        if len(matches) != 1:
            problems.append(f"duplicate projected context {context!r}: {len(matches)} occurrences")

    declared = {item.context: item for item in inventory.producers}
    bindings: list[ProducerBinding] = []
    prepared: list[ProducerBinding] = []
    inactive: list[Producer] = []
    matched: list[tuple[Producer, object, object]] = []
    for producer in inventory.producers:
        optional = producer.context in inventory.optional_contexts
        matches = by_context.get(producer.context, [])
        if not matches:
            if optional:
                inactive.append(producer)
            else:
                problems.append(f"producer {producer.context!r} must occur exactly once; found 0")
            continue
        if len(matches) != 1:
            continue
        workflow, occurrence = matches[0]
        actual = (workflow.source.relative, occurrence.job.job_id, _matrix_binding(occurrence))
        expected = (producer.workflow, producer.job, producer.matrix)
        if actual != expected:
            problems.append(f"producer {producer.context!r} identity mismatch: {actual!r} != {expected!r}")
        event = _producer_event(workflow, producer, problems)
        job = occurrence.job
        if job.condition is not None:
            problems.append(f"{producer.workflow}:{producer.job} has job-level if")
        if job.continue_on_error is not None and job.continue_on_error.value != "false":
            problems.append(f"{producer.workflow}:{producer.job} enables soft failure")
        dependencies = _dependency_jobs(workflow, job, problems)
        dependency_contexts = tuple(
            item.context for dependency in dependencies
            for item in workflow.contexts if item.job is dependency
        )
        for dependency in dependencies:
            if dependency.condition is not None:
                problems.append(f"{producer.workflow}:{producer.job} dependency {dependency.job_id!r} has job-level if")
            if dependency.continue_on_error is not None and dependency.continue_on_error.value != "false":
                problems.append(f"{producer.workflow}:{producer.job} dependency {dependency.job_id!r} enables soft failure")
        for context in dependency_contexts:
            dependency = declared.get(context)
            if dependency is None:
                problems.append(f"{producer.workflow}:{producer.job} dependency context {context!r} is undeclared")
            elif not optional and context in inventory.optional_contexts:
                problems.append(f"{producer.workflow}:{producer.job} has optional dependency {context!r}")
        matched.append((producer, workflow, occurrence))
        if event is not None:
            binding = ProducerBinding(producer, workflow, event, occurrence, dependency_contexts)
            (prepared if optional else bindings).append(binding)

    checked_jobs: set[tuple[str, str]] = set()
    for producer, workflow, occurrence in matched:
        key = (workflow.source.relative, occurrence.job.job_id)
        if key in checked_jobs:
            continue
        checked_jobs.add(key)
        actual = Counter(
            (item.context, _matrix_binding(item))
            for item in workflow.contexts if item.job is occurrence.job
        )
        expected = Counter(
            (item.context, item.matrix) for item in inventory.producers
            if (item.workflow, item.job) == key and by_context.get(item.context)
        )
        if actual != expected:
            problems.append(f"{key[0]}:{key[1]} job expansion does not match its declared occurrences")

    if problems:
        return ProducerEvaluation(problems=tuple(problems))
    return ProducerEvaluation(tuple(bindings), tuple(prepared), tuple(inactive), ())
