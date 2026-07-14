#!/usr/bin/env python3
"""Immutable Git-object boundary for workflow data consumed by validators."""
from __future__ import annotations

import os
import re
import subprocess
import unicodedata
from dataclasses import dataclass
from pathlib import Path


WORKFLOW_DIRECTORY = ".github/workflows"
WORKFLOW_PREFIX = f"{WORKFLOW_DIRECTORY}/"
WORKFLOW_SUFFIXES = {".yml", ".yaml"}
REGULAR_BLOB_MODES = {"100644", "100755"}
MAX_WORKFLOW_BYTES = 1 << 20
MAX_WORKFLOW_TOTAL_BYTES = 4 << 20
MAX_WORKFLOW_FILES = 256
MAX_TREE_LIST_BYTES = 1 << 20
OID_RE = re.compile(r"[0-9a-f]{40,64}")


@dataclass(frozen=True)
class WorkflowBlob:
    """A workflow name and payload pinned to one verified Git blob."""

    relative: str
    mode: str
    object_id: str
    content: bytes


class BoundaryError(RuntimeError):
    pass


def _git(root: Path, *args: str, limit: int) -> bytes:
    """Read at most ``limit`` bytes from trusted Git plumbing."""
    environment = {
        key: value for key, value in os.environ.items() if not key.upper().startswith("GIT_")
    }
    environment.update(GIT_NO_LAZY_FETCH="1", GIT_NO_REPLACE_OBJECTS="1", GIT_TERMINAL_PROMPT="0")
    try:
        process = subprocess.Popen(
            ["git", "--no-replace-objects", "-C", str(root), *args],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            env=environment,
        )
    except OSError as exc:
        raise BoundaryError("cannot execute Git object plumbing") from exc
    assert process.stdout is not None
    output = process.stdout.read(limit + 1)
    process.stdout.close()
    if len(output) > limit:
        process.kill()
        process.wait()
        raise BoundaryError(f"Git {args[0]} output exceeds boundary")
    if process.wait() != 0:
        raise BoundaryError(f"Git {args[0]} failed")
    return output


def _raw_entries(root: Path, treeish: str | None) -> list[tuple[str, str, str, bytes]]:
    if _git(root, "rev-parse", "--show-prefix", limit=4096).strip():
        raise BoundaryError("workflow root is not the Git worktree root")
    if treeish is None:
        raw = _git(
            root,
            "ls-files",
            "--stage",
            "-z",
            "--",
            WORKFLOW_DIRECTORY,
            limit=MAX_TREE_LIST_BYTES,
        )
        entries: list[tuple[str, str, str, bytes]] = []
        for item in filter(None, raw.split(b"\0")):
            match = re.fullmatch(
                rb"([0-7]{6}) ([0-9a-f]{40,64}) ([0-3])\t(.+)", item
            )
            if match is None or match.group(3) != b"0":
                raise BoundaryError("Git index has a malformed or unmerged entry")
            entries.append(
                (
                    match.group(1).decode("ascii"),
                    "blob",
                    match.group(2).decode("ascii"),
                    match.group(4),
                )
            )
        return entries
    if OID_RE.fullmatch(treeish) is None:
        raise BoundaryError("treeish must be a full lowercase commit object ID")
    commit = _git(
        root,
        "rev-parse",
        "--verify",
        "--end-of-options",
        f"{treeish}^{{commit}}",
        limit=128,
    ).strip().decode("ascii", "strict")
    if commit != treeish:
        raise BoundaryError("Git did not resolve the exact requested commit")
    tree = _git(
        root,
        "rev-parse",
        "--verify",
        "--end-of-options",
        f"{commit}^{{tree}}",
        limit=128,
    ).strip().decode("ascii", "strict")
    if OID_RE.fullmatch(tree) is None:
        raise BoundaryError("Git did not resolve an exact commit tree")
    raw = _git(
        root,
        "ls-tree",
        "-rz",
        "--full-tree",
        tree,
        "--",
        WORKFLOW_DIRECTORY,
        limit=MAX_TREE_LIST_BYTES,
    )
    entries = []
    for item in filter(None, raw.split(b"\0")):
        match = re.fullmatch(
            rb"([0-7]{6}) (blob|tree|commit) ([0-9a-f]{40,64})\t(.+)", item
        )
        if match is None:
            raise BoundaryError("Git tree has a malformed entry")
        entries.append(
            tuple(part.decode("ascii") for part in match.groups()[:3])
            + (match.group(4),)
        )
    return entries


def workflow_snapshot(
    root: Path, *, treeish: str | None = None
) -> tuple[list[WorkflowBlob], list[str]]:
    """Return an all-or-zero snapshot from the index or an exact commit tree."""
    try:
        raw_entries = _raw_entries(root, treeish)
        selected: list[tuple[str, str, str]] = []
        collision_keys: dict[str, str] = {}
        problems: list[str] = []
        for mode, kind, object_id, raw_name in raw_entries:
            try:
                relative = raw_name.decode("utf-8", "strict")
            except UnicodeDecodeError:
                problems.append("workflow tree contains a non-UTF-8 path")
                continue
            canonical = unicodedata.normalize("NFC", relative)
            key = canonical.casefold()
            if key in collision_keys:
                problems.append(
                    f"workflow paths collide: {collision_keys[key]!r} and {relative!r}"
                )
            else:
                collision_keys[key] = relative
            if relative != canonical or not relative.startswith(WORKFLOW_PREFIX):
                problems.append(f"workflow path is not canonical: {relative!r}")
                continue
            name = relative[len(WORKFLOW_PREFIX) :]
            if not name or "/" in name or "\\" in name:
                problems.append(f"workflow path is not a direct child: {relative!r}")
                continue
            suffix = "." + name.rsplit(".", 1)[1] if "." in name else ""
            if suffix.casefold() in WORKFLOW_SUFFIXES and suffix not in WORKFLOW_SUFFIXES:
                problems.append(f"case-variant workflow suffix: {relative}")
                continue
            if suffix not in WORKFLOW_SUFFIXES:
                continue
            if kind != "blob" or mode not in REGULAR_BLOB_MODES:
                problems.append(f"workflow is not a regular Git blob: {relative}")
                continue
            selected.append((relative, mode, object_id))
        if len(selected) > MAX_WORKFLOW_FILES:
            problems.append(f"workflow count exceeds {MAX_WORKFLOW_FILES}")
        if problems:
            return [], problems
        sizes: list[int] = []
        total = 0
        for relative, _, object_id in selected:
            kind = _git(root, "cat-file", "-t", object_id, limit=16).strip()
            raw_size = _git(root, "cat-file", "-s", object_id, limit=32).strip()
            if kind != b"blob" or not raw_size.isdigit():
                raise BoundaryError(f"workflow object is not a blob: {relative}")
            size = int(raw_size)
            total += size
            if size > MAX_WORKFLOW_BYTES or total > MAX_WORKFLOW_TOTAL_BYTES:
                raise BoundaryError("workflow byte boundary exceeded")
            sizes.append(size)
        records: list[WorkflowBlob] = []
        for (relative, mode, object_id), size in zip(selected, sizes, strict=True):
            content = _git(root, "cat-file", "blob", object_id, limit=size)
            if len(content) != size:
                raise BoundaryError(f"workflow object size changed: {relative}")
            records.append(WorkflowBlob(relative, mode, object_id, content))
        return records, []
    except (BoundaryError, UnicodeError, ValueError) as exc:
        return [], [str(exc)]
