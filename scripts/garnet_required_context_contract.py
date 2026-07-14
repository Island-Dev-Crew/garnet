#!/usr/bin/env python3
"""Strict declarative schema for Garnet required-context producers."""
from __future__ import annotations

import json
import os
import re
import stat
import unicodedata
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
