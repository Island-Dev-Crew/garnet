#!/usr/bin/env python3
"""Shared, byte-exact Git content provenance for the bounded Lane 2B gates.

Digest reproduction starts with exactly one Git enumeration command:

    git --no-replace-objects ls-files -s -z

For each stage-0 record outside ``FROZEN_MUTABLE_PREFIXES`` and
``REPORTER_PATH``, parse ``mode SP blob-OID SP stage TAB path NUL``. Sort by
raw path bytes, then SHA-256 the concatenation ``path NUL blob-OID LF``.
Commit-tree verification uses the identical construction over
``git --no-replace-objects ls-tree -r -z <commit>``. The namespace list is
frozen by Lane 2B Review Verdict 04; adding an exclusion is a reviewed change.
"""
from __future__ import annotations

import hashlib
import re
import subprocess
from pathlib import Path


FROZEN_MUTABLE_PREFIXES = (
    b"ops/lane2b/",
    b"proofs/",
    b"F_Project_Management/W_TRUST/",
    b"ops/lane1/",
)
REPORTER_PATH = b"scripts/smoke_garnet_minimum_shelf.py"
GIT_OID_RE = re.compile(rb"^[0-9a-f]{40}$")
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
GIT_SHA_RE = re.compile(r"^[0-9a-f]{40}$")


def _git_bytes(root: Path, *args: str) -> bytes:
    result = subprocess.run(
        ["git", "--no-replace-objects", *args],
        cwd=root,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", errors="replace").strip()
        raise ValueError(f"git {' '.join(args)} failed: {detail}")
    return result.stdout


def _is_mutable(path: bytes) -> bool:
    return path == REPORTER_PATH or any(
        path.startswith(prefix) for prefix in FROZEN_MUTABLE_PREFIXES
    )


def _index_entries(raw: bytes) -> list[tuple[bytes, bytes]]:
    entries: list[tuple[bytes, bytes]] = []
    for record in raw.split(b"\0"):
        if not record:
            continue
        try:
            metadata, path = record.split(b"\t", 1)
            mode, oid, stage = metadata.split(b" ")
        except ValueError as exc:
            raise ValueError("Git index record is not canonical") from exc
        if not mode or stage != b"0" or GIT_OID_RE.fullmatch(oid) is None:
            raise ValueError("Git index contains a non-stage-0 or invalid blob record")
        if not path or b"\0" in path:
            raise ValueError("Git index path is not canonical")
        if not _is_mutable(path):
            entries.append((path, oid))
    return entries


def _tree_entries(raw: bytes) -> list[tuple[bytes, bytes]]:
    entries: list[tuple[bytes, bytes]] = []
    for record in raw.split(b"\0"):
        if not record:
            continue
        try:
            metadata, path = record.split(b"\t", 1)
            mode, kind, oid = metadata.split(b" ")
        except ValueError as exc:
            raise ValueError("Git tree record is not canonical") from exc
        if (
            not mode
            or kind != b"blob"
            or GIT_OID_RE.fullmatch(oid) is None
            or not path
        ):
            raise ValueError("Git tree contains a non-blob or invalid record")
        if not _is_mutable(path):
            entries.append((path, oid))
    return entries


def _digest(entries: list[tuple[bytes, bytes]]) -> tuple[str, int]:
    ordered = sorted(entries, key=lambda item: (item[0], item[1]))
    if len({path for path, _ in ordered}) != len(ordered):
        raise ValueError("tracked content contains duplicate paths")
    digest = hashlib.sha256()
    for path, oid in ordered:
        digest.update(path)
        digest.update(b"\0")
        digest.update(oid)
        digest.update(b"\n")
    return digest.hexdigest(), len(ordered)


def tracked_content_digest(
    root: Path, revision: str | None = None
) -> tuple[str, int]:
    """Return the frozen-namespace-filtered (path, blob-OID) content digest."""
    if revision is None:
        return _digest(_index_entries(_git_bytes(root, "ls-files", "-s", "-z")))
    return _digest(
        _tree_entries(_git_bytes(root, "ls-tree", "-r", "-z", revision))
    )


def verify_product_content(root: Path, expected_digest: str) -> list[str]:
    findings: list[str] = []
    if SHA256_RE.fullmatch(expected_digest) is None:
        return ["expected product content digest is invalid"]
    try:
        actual, _ = tracked_content_digest(root)
    except ValueError as exc:
        return [str(exc)]
    if actual != expected_digest:
        findings.append(
            f"product content digest mismatch ({actual} != {expected_digest})"
        )
    return findings


def verify_squash_durable_content(
    root: Path,
    *,
    reviewed_head: str,
    reviewed_tree: str,
    expected_content_digest: str,
    verify_git: bool,
) -> tuple[list[str], str | None]:
    """Mirror U-19: branch SHAs are provenance; landed main is first-parent.

    A topic branch passes from its current content proof. When HEAD is present
    on authoritative ``origin/main`` first-parent history, its committed tree
    must reproduce the same product digest and is reported as the landed main
    commit. No reviewed branch object is resolved or required.
    """
    findings: list[str] = []
    if GIT_SHA_RE.fullmatch(reviewed_head) is None:
        findings.append("reviewed head provenance must be a full lowercase Git SHA")
    if GIT_SHA_RE.fullmatch(reviewed_tree) is None:
        findings.append("reviewed tree provenance must be a full lowercase Git SHA")
    if SHA256_RE.fullmatch(expected_content_digest) is None:
        findings.append("product content digest must be lowercase SHA-256")
    if findings or not verify_git:
        return findings, None

    findings.extend(verify_product_content(root, expected_content_digest))
    try:
        _git_bytes(root, "show-ref", "--verify", "refs/remotes/origin/main")
        first_parent = _git_bytes(
            root, "rev-list", "--first-parent", "refs/remotes/origin/main"
        ).decode("ascii").splitlines()
        head = _git_bytes(root, "rev-parse", "HEAD").decode("ascii").strip()
    except (UnicodeError, ValueError) as exc:
        findings.append(str(exc))
        return findings, None

    landed: str | None = None
    if head in first_parent:
        try:
            committed_digest, _ = tracked_content_digest(root, head)
        except ValueError as exc:
            findings.append(str(exc))
        else:
            if committed_digest != expected_content_digest:
                findings.append(
                    "landed first-parent main product content digest mismatch"
                )
            else:
                landed = head
    return findings, landed
