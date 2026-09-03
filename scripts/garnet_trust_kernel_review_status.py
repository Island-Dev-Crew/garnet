#!/usr/bin/env python3
"""Fail-closed, content-bound rolling review gate for Garnet's trust kernel.

Version 2 replaces the old "a companion path exists" signal with two proofs:

* Git must completely enumerate the candidate change set.  Discovery failures,
  malformed status streams, and deletions are findings, never empty-safe diffs.
* A trust-kernel change must carry a canonical ``*.review.json`` record under
  ``F_Project_Management/W_TRUST``.  Multiple records must form a linear,
  append-only succession; only the tip-most record binds the exact changed
  paths, their bytes, the reviewed head/tree, and an independent reviewer.

The module also exports ``verify_landed_review_marker`` for post-squash
closeouts.  That verifier deliberately does not require the discarded branch
head to be an ancestor of main.  It proves the landed commit is on upstream
main's first-parent history and that its exact first-parent landing edge, tree,
and trust-kernel content match the recorded digests while retaining the earlier
reviewed-head provenance.
"""
from __future__ import annotations

import argparse
import dataclasses
import hashlib
import importlib.util
import json
import os
import re
import subprocess
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path, PurePosixPath
from typing import Any

SCHEMA = "garnet.trust_kernel_review/v2"
RECORD_SCHEMA = "garnet.trust_kernel_review_record/v2"
MARKER_SCHEMA = "garnet.trust_kernel_review_marker/v2"
ROOT = Path(__file__).resolve().parents[1]
GIT_TIMEOUT_SECONDS = 20
GIT_OID_RE = re.compile(r"^[0-9a-f]{40}$")
DIGEST_RE = re.compile(r"^sha256:[0-9a-f]{64}$")
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
IDENTITY_RE = re.compile(r"^(?:agent|email|github):[A-Za-z0-9][A-Za-z0-9._@+-]{1,127}$")
GITHUB_LOGIN_RE = re.compile(r"^[A-Za-z0-9](?:[A-Za-z0-9-]{0,37}[A-Za-z0-9])?$")
GITHUB_REPOSITORY_RE = re.compile(
    r"^[A-Za-z0-9](?:[A-Za-z0-9._-]{0,99})/[A-Za-z0-9](?:[A-Za-z0-9._-]{0,99})$"
)
MAX_GIT_OBJECTS = 100_000
MAX_TREE_DEPTH = 128
MAX_TREE_ENTRIES = 500_000

# Any changed path under these prefixes, or matching these exact files, is a
# trust-kernel change.  Keep this surface machine-readable and intentionally
# conservative: a false positive demands review; a false negative bypasses it.
TRUST_KERNEL_PREFIXES = (
    "garnet-check-v0.3/src/",
    "garnet-interp-v0.3/src/",
    "garnet-vm/src/",
    "garnet-stdlib/src/",
    "garnet-wasm/src/",
    ".github/actions/",
    ".github/rulesets/",
    ".github/workflows/",
    "scripts/garnet_",
    "scripts/test_garnet_",
    "F_Project_Management/W_TRUST/landed/",
)
TRUST_KERNEL_FILES = (
    ".github/CODEOWNERS",
    "Cargo.lock",
    "garnet-cli/Cargo.toml",
    "garnet-cli/src/bound_source.rs",
    "garnet-cli/src/cmd/add.rs",
    "garnet-cli/src/cmd/mod.rs",
    "garnet-cli/src/cmd/run.rs",
    "garnet-cli/src/cmd/test.rs",
    "garnet-cli/src/cmd/eval.rs",
    "garnet-cli/src/cmd/doctest.rs",
    "garnet-cli/src/bin/garnet.rs",
    "garnet-cli/src/lib.rs",
    "scripts/garnet_launch_readiness_status.py",
    "scripts/garnet_caps_enforcement_status.py",
    "scripts/garnet_capability_scope_status.py",
    "scripts/garnet_bounded_enforcement_status.py",
    "scripts/garnet_red_team_status.py",
    "docs/why.html",
    "C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md",
    "F_Project_Management/W_TRUST/LANDED_REVIEW_MARKERS.json",
)

REVIEW_RECORD_PREFIX = "F_Project_Management/W_TRUST/"
REVIEW_RECORD_SUFFIX = ".review.json"
LANDED_MARKER_PREFIX = "F_Project_Management/W_TRUST/landed/"
LANDED_MARKER_SUFFIX = ".landed-review.json"
LANDED_REGISTRY_PATH = "F_Project_Management/W_TRUST/LANDED_REVIEW_MARKERS.json"
LANDED_REGISTRY_SCHEMA = "garnet.trust_kernel_landed_review_registry/v1"
LEGACY_COMPANION_PREFIXES = (
    "proofs/independent/s114/",
    "F_Project_Management/W_TRUST/",
    "F_Project_Management/VALIDATION_REPORTS/",
)
LEGACY_COMPANION_FILES = (
    "F_Project_Management/LAUNCH/S114_ACCEPTANCE.json",
)

PREMERGE_KEYS = frozenset(
    {
        "author_emails",
        "author_ids",
        "base_commit",
        "blocking_findings",
        "content_digest",
        "head_repository",
        "head_repository_id",
        "pull_request_id",
        "pull_request_number",
        "repository",
        "repository_id",
        "review_scope",
        "review_state",
        "reviewed_head",
        "reviewed_tree",
        "reviewer_id",
        "reviewer_login",
        "schema",
        "state",
        "touched_paths",
        "verdict",
    }
)
LANDED_KEYS = frozenset(
    PREMERGE_KEYS
    | {
        "merged_commit",
        "merged_tree",
        "review_record_path",
        "review_record_sha256",
    }
)


@dataclass(frozen=True)
class GitResult:
    returncode: int
    stdout: bytes
    stderr: bytes
    timed_out: bool = False


@dataclass(frozen=True, order=True)
class ChangeEntry:
    path: str
    status: str
    old_mode: str
    new_mode: str
    old_oid: str
    new_oid: str


@dataclass
class DiscoveryResult:
    paths: list[str]
    base_commit: str | None
    head_commit: str | None
    problems: list[str]
    source: str = "git"
    entries: list[ChangeEntry] = field(default_factory=list)

    @property
    def ok(self) -> bool:
        return not self.problems

    @property
    def empty(self) -> bool:
        return self.ok and not self.paths


@dataclass
class TrustKernelReviewStatus:
    schema: str
    ok: bool
    discovery_ok: bool
    discovery_source: str
    base_commit: str | None
    head_commit: str | None
    trust_kernel_touched: bool
    touched_paths: list[str] = field(default_factory=list)
    review_record_present: bool = False
    review_record_path: str | None = None
    review_record_sha256: str | None = None
    reviewer: str | None = None
    reviewer_id: int | None = None
    reviewer_login: str | None = None
    reviewed_head: str | None = None
    reviewed_tree: str | None = None
    content_digest: str | None = None
    changed_count: int = 0
    problems: list[str] = field(default_factory=list)
    # Compatibility output fields are retained for downstream readers, but v2
    # only treats a valid structured record as a companion and never accepts a
    # trailer as proof.
    review_companion_present: bool = False
    companion_paths: list[str] = field(default_factory=list)
    review_trailer_present: bool = False
    trust_kernel_prefixes: list[str] = field(default_factory=lambda: list(TRUST_KERNEL_PREFIXES))
    trust_kernel_files: list[str] = field(default_factory=lambda: list(TRUST_KERNEL_FILES))


class DuplicateKeyError(ValueError):
    pass


def _norm(path: str) -> str:
    value = path.replace("\\", "/")
    while value.startswith("./"):
        value = value[2:]
    return value


def is_trust_kernel(path: str) -> bool:
    value = _norm(path)
    return value in TRUST_KERNEL_FILES or value.startswith(TRUST_KERNEL_PREFIXES)


def is_review_record(path: str) -> bool:
    value = _norm(path)
    return value.startswith(REVIEW_RECORD_PREFIX) and value.endswith(REVIEW_RECORD_SUFFIX)


def is_review_companion(path: str) -> bool:
    """Compatibility classifier for legacy prose/presence companions."""
    value = _norm(path)
    return value in LEGACY_COMPANION_FILES or value.startswith(LEGACY_COMPANION_PREFIXES)


def _scrubbed_git_environment() -> dict[str, str]:
    """Build the minimal process environment shared by every Git probe.

    Git's repository, index, object, namespace, replace, and configuration
    environment variables can redirect a command away from ``cwd`` or change
    the object graph it observes.  An allowlist is safer than trying to name
    every present and future control variable to remove.
    """
    passthrough = (
        "COMSPEC",
        "PATH",
        "PATHEXT",
        "SYSTEMROOT",
        "SystemRoot",
        "TEMP",
        "TMP",
        "TMPDIR",
        "WINDIR",
    )
    env = {
        name: value
        for name in passthrough
        if (value := os.environ.get(name)) is not None
    }
    env.update(
        {
            "GIT_ATTR_NOSYSTEM": "1",
            "GIT_CONFIG_COUNT": "0",
            "GIT_CONFIG_GLOBAL": os.devnull,
            "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_CONFIG_SYSTEM": os.devnull,
            "GIT_GRAFT_FILE": os.devnull,
            "GIT_NO_REPLACE_OBJECTS": "1",
            "GIT_OPTIONAL_LOCKS": "0",
            "GIT_TERMINAL_PROMPT": "0",
            "LC_ALL": "C",
        }
    )
    return env


def _git_bytes(root: Path, *args: str, timeout: int = GIT_TIMEOUT_SECONDS) -> GitResult:
    try:
        result = subprocess.run(
            [
                "git",
                "-c",
                "advice.graftFileDeprecated=false",
                "--no-replace-objects",
                *args,
            ],
            cwd=root,
            capture_output=True,
            check=False,
            timeout=timeout,
            env=_scrubbed_git_environment(),
        )
    except subprocess.TimeoutExpired as exc:
        return GitResult(
            124,
            exc.stdout if isinstance(exc.stdout, bytes) else b"",
            exc.stderr if isinstance(exc.stderr, bytes) else b"",
            timed_out=True,
        )
    returncode = result.returncode
    if b"graft" in result.stderr.lower():
        returncode = 125
    return GitResult(returncode, result.stdout, result.stderr)


def _one_oid(payload: bytes) -> str | None:
    try:
        text = payload.decode("ascii", errors="strict")
    except UnicodeDecodeError:
        return None
    lines = text.splitlines()
    if len(lines) != 1 or GIT_OID_RE.fullmatch(lines[0]) is None:
        return None
    return lines[0]


def _resolve_commit(ref: str, label: str, root: Path) -> tuple[str | None, list[str]]:
    result = _git_bytes(root, "rev-parse", "--verify", "--end-of-options", f"{ref}^{{commit}}")
    if result.timed_out:
        return None, [f"{label} commit resolution timed out"]
    if result.returncode != 0:
        return None, [f"{label} does not name a commit"]
    oid = _one_oid(result.stdout)
    if oid is None:
        return None, [f"{label} commit resolution returned a malformed object id"]
    return oid, []


def _resolve_tree(commit: str, label: str, root: Path) -> tuple[str | None, list[str]]:
    result = _git_bytes(root, "rev-parse", "--verify", "--end-of-options", f"{commit}^{{tree}}")
    if result.timed_out:
        return None, [f"{label} tree resolution timed out"]
    if result.returncode != 0:
        return None, [f"{label} tree could not be resolved"]
    oid = _one_oid(result.stdout)
    if oid is None:
        return None, [f"{label} tree resolution returned a malformed object id"]
    return oid, []


def _valid_repo_path(path: str) -> bool:
    pure = PurePosixPath(path)
    return bool(path) and not pure.is_absolute() and ".." not in pure.parts and "\x00" not in path


RAW_HEADER_RE = re.compile(
    r"^:([0-7]{6}) ([0-7]{6}) ([0-9a-f]{40}) ([0-9a-f]{40}) ([A-Z])$"
)


def _parse_raw_z(payload: bytes) -> tuple[list[ChangeEntry], list[str]]:
    """Parse full-index raw diff entries, including exact deletion tombstones."""
    if payload == b"":
        return [], []
    if not payload.endswith(b"\0"):
        return [], ["git diff raw status stream is not NUL terminated"]
    tokens = payload[:-1].split(b"\0")
    if len(tokens) % 2:
        return [], ["git diff raw stream has incomplete header/path pairs"]

    entries: list[ChangeEntry] = []
    problems: list[str] = []
    seen: set[str] = set()
    for offset in range(0, len(tokens), 2):
        header_bytes, path_bytes = tokens[offset], tokens[offset + 1]
        try:
            header = header_bytes.decode("ascii", errors="strict")
        except UnicodeDecodeError:
            problems.append("git diff contains a non-ASCII raw header")
            continue
        match = RAW_HEADER_RE.fullmatch(header)
        if match is None:
            problems.append("git diff contains a malformed raw status header")
            continue
        old_mode, new_mode, old_oid, new_oid, status = match.groups()
        try:
            path = _norm(path_bytes.decode("utf-8", errors="strict"))
        except UnicodeDecodeError:
            problems.append("git diff contains a path that is not valid UTF-8")
            continue
        if status not in {"A", "M", "T"}:
            if status != "D":
                problems.append(f"git diff contains unsupported status {status!r} for {path!r}")
                continue
        if not _valid_repo_path(path):
            problems.append(f"git diff contains an invalid repository path: {path!r}")
            continue
        if path in seen:
            problems.append(f"git diff contains duplicate path: {path}")
            continue
        zero = "0" * 40
        if status == "A" and not (
            old_mode == "000000" and old_oid == zero and new_mode != "000000" and new_oid != zero
        ):
            problems.append(f"addition identity is ambiguous for content proof: {path}")
            continue
        if status == "D" and not (
            old_mode != "000000" and old_oid != zero and new_mode == "000000" and new_oid == zero
        ):
            problems.append(f"ambiguous deletion identity for content proof: {path}")
            continue
        if status in {"M", "T"} and not (
            old_mode != "000000" and new_mode != "000000" and old_oid != zero and new_oid != zero
        ):
            problems.append(f"change identity is ambiguous for content proof: {path}")
            continue
        seen.add(path)
        entries.append(ChangeEntry(path, status, old_mode, new_mode, old_oid, new_oid))
    return entries, problems


def _parse_name_only_z(payload: bytes) -> tuple[list[str], list[str]]:
    if payload == b"":
        return [], []
    if not payload.endswith(b"\0"):
        return [], ["git diff name-only cross-check is not NUL terminated"]
    paths: list[str] = []
    problems: list[str] = []
    seen: set[str] = set()
    for token in payload[:-1].split(b"\0"):
        try:
            path = _norm(token.decode("utf-8", errors="strict"))
        except UnicodeDecodeError:
            problems.append("git diff name-only cross-check contains invalid UTF-8")
            continue
        if not _valid_repo_path(path):
            problems.append(f"git diff name-only cross-check contains invalid path: {path!r}")
        elif path in seen:
            problems.append(f"git diff name-only cross-check contains duplicate path: {path}")
        else:
            seen.add(path)
            paths.append(path)
    return paths, problems


def _diff_entries(root: Path, range_spec: str) -> tuple[list[ChangeEntry], list[str]]:
    result = _git_bytes(
        root,
        "diff",
        "--no-ext-diff",
        "--no-renames",
        "--raw",
        "--full-index",
        "--abbrev=40",
        "-z",
        range_spec,
    )
    if result.timed_out:
        return [], ["git diff enumeration timed out"]
    if result.returncode != 0:
        return [], ["git diff enumeration failed"]
    entries, problems = _parse_raw_z(result.stdout)
    if problems:
        return entries, problems
    name_check = _git_bytes(
        root,
        "diff",
        "--no-ext-diff",
        "--no-renames",
        "--name-only",
        "-z",
        range_spec,
    )
    if name_check.timed_out:
        return [], ["git diff name-only cross-check timed out"]
    if name_check.returncode != 0:
        return [], ["git diff name-only cross-check failed"]
    names, name_problems = _parse_name_only_z(name_check.stdout)
    if name_problems:
        return [], name_problems
    if sorted(entry.path for entry in entries) != sorted(names):
        return [], ["git diff raw enumeration is partial or disagrees with name-only cross-check"]
    return entries, []


@dataclass(frozen=True)
class RawCommit:
    tree: str
    parents: tuple[str, ...]
    author_email: str


def _raw_object(root: Path, kind: str, oid: str, label: str) -> tuple[bytes | None, list[str]]:
    """Read one object through its declared type with replacement refs disabled."""
    if GIT_OID_RE.fullmatch(oid) is None or kind not in {"commit", "tree"}:
        return None, [f"{label} has an invalid object request"]
    result = _git_bytes(root, "cat-file", kind, oid)
    if result.timed_out:
        return None, [f"{label} object read timed out"]
    if result.returncode != 0:
        return None, [f"{label} is not a readable {kind} object"]
    return result.stdout, []


_AUTHOR_LINE_RE = re.compile(
    rb"^author .* <([^<>\x00\r\n]+)> [0-9]+ [+-][0-9]{4}$"
)


def _read_raw_commit(
    root: Path,
    oid: str,
    cache: dict[str, RawCommit] | None = None,
) -> tuple[RawCommit | None, list[str]]:
    if cache is not None and oid in cache:
        return cache[oid], []
    payload, problems = _raw_object(root, "commit", oid, f"commit {oid}")
    if payload is None:
        return None, problems
    header, separator, _message = payload.partition(b"\n\n")
    if not separator:
        return None, [f"commit-object traversal found malformed commit {oid}"]
    tree_values: list[str] = []
    parents: list[str] = []
    author_values: list[str] = []
    for line in header.split(b"\n"):
        if line.startswith(b" "):
            continue
        if line.startswith(b"tree "):
            raw = line[5:]
            try:
                tree_values.append(raw.decode("ascii", errors="strict"))
            except UnicodeDecodeError:
                return None, [f"commit-object traversal found malformed tree id in {oid}"]
        elif line.startswith(b"parent "):
            raw = line[7:]
            try:
                parents.append(raw.decode("ascii", errors="strict"))
            except UnicodeDecodeError:
                return None, [f"commit-object traversal found malformed parent id in {oid}"]
        elif line.startswith(b"author "):
            match = _AUTHOR_LINE_RE.fullmatch(line)
            if match is None:
                return None, [f"commit-object traversal found malformed author in {oid}"]
            try:
                email = match.group(1).decode("utf-8", errors="strict").strip().casefold()
            except UnicodeDecodeError:
                return None, [f"commit-object traversal found malformed author in {oid}"]
            identity = "email:" + email
            if IDENTITY_RE.fullmatch(identity) is None:
                return None, [f"commit-object traversal found noncanonical author in {oid}"]
            author_values.append(identity)
    if len(tree_values) != 1 or GIT_OID_RE.fullmatch(tree_values[0]) is None:
        return None, [f"commit-object traversal found malformed tree binding in {oid}"]
    if (
        len(author_values) != 1
        or len(parents) != len(set(parents))
        or any(GIT_OID_RE.fullmatch(parent) is None for parent in parents)
    ):
        return None, [f"commit-object traversal found malformed commit headers in {oid}"]
    commit = RawCommit(tree_values[0], tuple(parents), author_values[0])
    if cache is not None:
        cache[oid] = commit
    return commit, []


def _parse_raw_tree(payload: bytes, label: str) -> tuple[list[tuple[str, str, str]], list[str]]:
    entries: list[tuple[str, str, str]] = []
    seen: set[str] = set()
    cursor = 0
    while cursor < len(payload):
        space = payload.find(b" ", cursor)
        nul = payload.find(b"\0", space + 1) if space >= 0 else -1
        if space <= cursor or nul <= space + 1 or nul + 21 > len(payload):
            return [], [f"tree-object traversal found malformed entry in {label}"]
        raw_mode = payload[cursor:space]
        raw_name = payload[space + 1:nul]
        raw_oid = payload[nul + 1:nul + 21]
        cursor = nul + 21
        try:
            mode = raw_mode.decode("ascii", errors="strict").zfill(6)
            name = raw_name.decode("utf-8", errors="strict")
        except UnicodeDecodeError:
            return [], [f"tree-object traversal found non-UTF-8 entry in {label}"]
        if mode not in {"040000", "100644", "100755", "120000", "160000"}:
            return [], [f"tree-object traversal found unsupported mode in {label}"]
        if not name or name in {".", ".."} or "/" in name or "\x00" in name:
            return [], [f"tree-object traversal found unsafe path component in {label}"]
        if name in seen:
            return [], [f"tree-object traversal found duplicate path component in {label}"]
        seen.add(name)
        entries.append((mode, name, raw_oid.hex()))
        if len(entries) > MAX_TREE_ENTRIES:
            return [], ["tree-object traversal exceeded its entry bound"]
    return entries, []


def _tree_snapshot_from_tree(
    root: Path,
    tree_oid: str,
    *,
    depth: int = 0,
    cache: dict[str, dict[str, tuple[str, str]]] | None = None,
    active: set[str] | None = None,
) -> tuple[dict[str, tuple[str, str]], list[str]]:
    if depth > MAX_TREE_DEPTH:
        return {}, ["tree-object traversal exceeded its depth bound"]
    if cache is not None and tree_oid in cache:
        return dict(cache[tree_oid]), []
    active = set() if active is None else active
    if tree_oid in active:
        return {}, ["tree-object traversal found a cyclic tree graph"]
    active.add(tree_oid)
    payload, problems = _raw_object(root, "tree", tree_oid, f"tree {tree_oid}")
    if payload is None:
        active.remove(tree_oid)
        return {}, problems
    entries, parse_problems = _parse_raw_tree(payload, tree_oid)
    if parse_problems:
        active.remove(tree_oid)
        return {}, parse_problems
    snapshot: dict[str, tuple[str, str]] = {}
    for mode, name, oid in entries:
        if mode == "040000":
            child, child_problems = _tree_snapshot_from_tree(
                root,
                oid,
                depth=depth + 1,
                cache=cache,
                active=active,
            )
            if child_problems:
                active.remove(tree_oid)
                return {}, child_problems
            for suffix, identity in child.items():
                path = f"{name}/{suffix}"
                if path in snapshot:
                    active.remove(tree_oid)
                    return {}, ["tree-object traversal found a duplicate repository path"]
                snapshot[path] = identity
        else:
            snapshot[name] = (mode, oid)
        if len(snapshot) > MAX_TREE_ENTRIES:
            active.remove(tree_oid)
            return {}, ["tree-object traversal exceeded its entry bound"]
    active.remove(tree_oid)
    if cache is not None:
        cache[tree_oid] = dict(snapshot)
    return snapshot, []


def _tree_snapshot_from_commit(
    root: Path,
    commit_oid: str,
    *,
    commit_cache: dict[str, RawCommit] | None = None,
    tree_cache: dict[str, dict[str, tuple[str, str]]] | None = None,
) -> tuple[dict[str, tuple[str, str]], list[str]]:
    commit, problems = _read_raw_commit(root, commit_oid, commit_cache)
    if commit is None:
        return {}, problems
    return _tree_snapshot_from_tree(root, commit.tree, cache=tree_cache)


def _mode_class(mode: str) -> str:
    if mode in {"100644", "100755"}:
        return "regular"
    if mode == "120000":
        return "symlink"
    if mode == "160000":
        return "gitlink"
    return mode


def _independent_tree_diff(
    root: Path,
    base_commit: str,
    head_commit: str,
) -> tuple[list[ChangeEntry], list[str]]:
    """Derive the endpoint diff by parsing commit and tree object bytes."""
    commit_cache: dict[str, RawCommit] = {}
    tree_cache: dict[str, dict[str, tuple[str, str]]] = {}
    before, before_problems = _tree_snapshot_from_commit(
        root, base_commit, commit_cache=commit_cache, tree_cache=tree_cache
    )
    if before_problems:
        return [], before_problems
    after, after_problems = _tree_snapshot_from_commit(
        root, head_commit, commit_cache=commit_cache, tree_cache=tree_cache
    )
    if after_problems:
        return [], after_problems
    zero = "0" * 40
    result: list[ChangeEntry] = []
    for path in sorted(set(before) | set(after)):
        old = before.get(path)
        new = after.get(path)
        if old == new:
            continue
        if old is None and new is not None:
            result.append(ChangeEntry(path, "A", "000000", new[0], zero, new[1]))
        elif new is None and old is not None:
            result.append(ChangeEntry(path, "D", old[0], "000000", old[1], zero))
        else:
            assert old is not None and new is not None
            status = "T" if _mode_class(old[0]) != _mode_class(new[0]) else "M"
            result.append(ChangeEntry(path, status, old[0], new[0], old[1], new[1]))
    return result, []


def _presented_commit_ids(
    base_commit: str,
    head_commit: str,
    root: Path = ROOT,
) -> tuple[list[str], list[str]]:
    commits_result = _git_bytes(root, "rev-list", "--reverse", f"{base_commit}..{head_commit}")
    if commits_result.timed_out:
        return [], ["author commit enumeration timed out"]
    if commits_result.returncode != 0:
        return [], ["author commit enumeration failed"]
    try:
        commits = commits_result.stdout.decode("ascii", errors="strict").splitlines()
    except UnicodeDecodeError:
        return [], ["author commit enumeration returned malformed output"]
    if not commits:
        return [], ["author commit enumeration returned an empty reviewed range"]
    if any(GIT_OID_RE.fullmatch(commit) is None for commit in commits):
        return [], ["author commit enumeration returned malformed or partial output"]
    if len(commits) != len(set(commits)):
        return [], ["author commit enumeration returned duplicate or partial output"]
    count_result = _git_bytes(root, "rev-list", "--count", f"{base_commit}..{head_commit}")
    if count_result.timed_out:
        return [], ["author commit count cross-check timed out"]
    if count_result.returncode != 0:
        return [], ["author commit count cross-check failed"]
    try:
        count_text = count_result.stdout.decode("ascii", errors="strict").strip()
    except UnicodeDecodeError:
        return [], ["author commit count cross-check returned malformed output"]
    if not count_text.isdigit() or int(count_text) != len(commits):
        return [], ["author commit enumeration is partial or disagrees with count cross-check"]
    return commits, []


def _independent_commit_range(
    base_commit: str,
    head_commit: str,
    root: Path,
) -> tuple[dict[str, RawCommit], list[str]]:
    cache: dict[str, RawCommit] = {}

    def walk(start: str) -> tuple[set[str], list[str]]:
        seen: set[str] = set()
        pending = [start]
        while pending:
            oid = pending.pop()
            if oid in seen:
                continue
            if len(seen) >= MAX_GIT_OBJECTS:
                return set(), ["commit-object traversal exceeded its object bound"]
            commit, problems = _read_raw_commit(root, oid, cache)
            if commit is None:
                return set(), problems
            seen.add(oid)
            pending.extend(parent for parent in commit.parents if parent not in seen)
        return seen, []

    excluded, base_problems = walk(base_commit)
    if base_problems:
        return {}, base_problems
    included: dict[str, RawCommit] = {}
    pending = [head_commit]
    while pending:
        oid = pending.pop()
        if oid in excluded or oid in included:
            continue
        if len(included) >= MAX_GIT_OBJECTS:
            return {}, ["commit-object traversal exceeded its object bound"]
        commit, problems = _read_raw_commit(root, oid, cache)
        if commit is None:
            return {}, problems
        included[oid] = commit
        pending.extend(parent for parent in commit.parents if parent not in excluded)
    return included, []


def discover_changes(
    base: str | None = None,
    head: str = "HEAD",
    root: Path = ROOT,
) -> DiscoveryResult:
    """Enumerate a complete diff or return explicit fail-closed findings."""
    root = root.resolve()
    head_commit, problems = _resolve_commit(head, "head ref", root)
    if problems:
        return DiscoveryResult([], None, None, problems)
    assert head_commit is not None

    if base is None:
        main_commit, main_problems = _resolve_commit("origin/main", "origin/main ref", root)
        if main_problems:
            return DiscoveryResult([], None, head_commit, main_problems)
        assert main_commit is not None
        merge_base = _git_bytes(root, "merge-base", head_commit, main_commit)
        if merge_base.timed_out:
            return DiscoveryResult([], None, head_commit, ["merge-base enumeration timed out"])
        if merge_base.returncode != 0:
            return DiscoveryResult([], None, head_commit, ["merge-base enumeration failed"])
        base_commit = _one_oid(merge_base.stdout)
        if base_commit is None:
            message = (
                "merge-base returned no commit"
                if not merge_base.stdout.strip()
                else "merge-base returned a malformed commit id"
            )
            return DiscoveryResult([], None, head_commit, [message])
    else:
        base_commit, base_problems = _resolve_commit(base, "base ref", root)
        if base_problems:
            return DiscoveryResult([], None, head_commit, base_problems)
        assert base_commit is not None

    entries, diff_problems = _diff_entries(root, f"{base_commit}...{head_commit}")
    if not diff_problems:
        independent, independent_problems = _independent_tree_diff(
            root, base_commit, head_commit
        )
        diff_problems.extend(independent_problems)
        if not independent_problems and sorted(entries) != sorted(independent):
            diff_problems.append(
                "git diff presentation is partial or disagrees with independent tree-object traversal"
            )
    return DiscoveryResult(
        [entry.path for entry in entries],
        base_commit,
        head_commit,
        diff_problems,
        entries=entries,
    )


def _read_blob(root: Path, ref: str, path: str) -> tuple[bytes | None, list[str]]:
    resolved = _git_bytes(root, "rev-parse", "--verify", "--end-of-options", f"{ref}:{path}")
    if resolved.timed_out:
        return None, [f"blob lookup timed out for {path}"]
    if resolved.returncode != 0:
        return None, [f"path is missing at {ref}: {path}"]
    oid = _one_oid(resolved.stdout)
    if oid is None:
        return None, [f"blob lookup returned a malformed object id for {path}"]
    return _read_blob_oid(root, oid, path)


def _read_blob_oid(root: Path, oid: str, label: str) -> tuple[bytes | None, list[str]]:
    kind = _git_bytes(root, "cat-file", "-t", oid)
    if kind.timed_out:
        return None, [f"blob type lookup timed out for {label}"]
    if kind.returncode != 0 or kind.stdout != b"blob\n":
        return None, [f"object is not a regular blob: {label}"]
    blob = _git_bytes(root, "cat-file", "blob", oid)
    if blob.timed_out:
        return None, [f"blob read timed out for {label}"]
    if blob.returncode != 0:
        return None, [f"blob read failed for {label}"]
    return blob.stdout, []


def compute_change_digest(
    entries: list[ChangeEntry] | tuple[ChangeEntry, ...],
    root: Path = ROOT,
) -> tuple[str | None, list[str]]:
    """Hash status/path, modes, OIDs, and old/new SHA-256 blob identities."""
    digest = hashlib.sha256()
    digest.update(b"garnet.trust_kernel.change/v2\0")
    problems: list[str] = []
    zero = "0" * 40
    for entry in sorted(entries):
        old_blob: bytes | None = None
        new_blob: bytes | None = None
        if entry.old_oid != zero:
            old_blob, old_problems = _read_blob_oid(root, entry.old_oid, f"old {entry.path}")
            problems.extend(old_problems)
        if entry.new_oid != zero:
            new_blob, new_problems = _read_blob_oid(root, entry.new_oid, f"new {entry.path}")
            problems.extend(new_problems)
        if problems:
            continue
        digest.update(entry.status.encode("ascii"))
        digest.update(b"\0")
        digest.update(entry.path.encode("utf-8"))
        digest.update(b"\0")
        digest.update(entry.old_mode.encode("ascii"))
        digest.update(b"\0")
        digest.update(entry.old_oid.encode("ascii"))
        digest.update(b"\0")
        digest.update(
            (hashlib.sha256(old_blob).hexdigest() if old_blob is not None else "-").encode("ascii")
        )
        digest.update(b"\0")
        digest.update(entry.new_mode.encode("ascii"))
        digest.update(b"\0")
        digest.update(entry.new_oid.encode("ascii"))
        digest.update(b"\0")
        digest.update(
            (hashlib.sha256(new_blob).hexdigest() if new_blob is not None else "-").encode("ascii")
        )
        digest.update(b"\0")
    return (None, problems) if problems else ("sha256:" + digest.hexdigest(), [])


def _no_duplicate_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise DuplicateKeyError(f"duplicate key: {key}")
        result[key] = value
    return result


def _load_canonical_record(payload: bytes) -> tuple[dict[str, Any] | None, list[str]]:
    try:
        text = payload.decode("utf-8", errors="strict")
    except UnicodeDecodeError:
        return None, ["review record is not valid UTF-8 JSON"]
    try:
        value = json.loads(text, object_pairs_hook=_no_duplicate_object)
    except DuplicateKeyError as exc:
        return None, [str(exc)]
    except json.JSONDecodeError:
        return None, ["review record is not valid JSON"]
    if not isinstance(value, dict):
        return None, ["review record root must be a JSON object"]
    canonical = (json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode()
    if payload != canonical:
        return None, ["review record must use canonical JSON (UTF-8, sorted keys, two-space indent, LF)"]
    return value, []


def derive_author_identities(
    base_commit: str,
    reviewed_head: str,
    root: Path = ROOT,
) -> tuple[list[str], list[str]]:
    """Derive author emails and cross-check Git's presentation with raw commits."""
    commits, commit_problems = _presented_commit_ids(base_commit, reviewed_head, root)
    if commit_problems:
        return [], commit_problems
    authors: set[str] = set()
    for commit in commits:
        author_result = _git_bytes(root, "show", "-s", "--format=%ae", commit)
        if author_result.timed_out:
            return [], [f"author identity lookup timed out for commit {commit}"]
        if author_result.returncode != 0:
            return [], [f"author identity lookup failed for commit {commit}"]
        try:
            lines = author_result.stdout.decode("utf-8", errors="strict").splitlines()
        except UnicodeDecodeError:
            return [], [f"author identity output is malformed for commit {commit}"]
        if len(lines) != 1 or not lines[0].strip() or "\x00" in lines[0]:
            return [], [f"author identity output is malformed or partial for commit {commit}"]
        identity = "email:" + lines[0].strip().casefold()
        if IDENTITY_RE.fullmatch(identity) is None:
            return [], [f"author identity output is not canonical for commit {commit}"]
        authors.add(identity)

    independent, independent_problems = _independent_commit_range(
        base_commit, reviewed_head, root
    )
    if independent_problems:
        return [], independent_problems
    if set(commits) != set(independent) or len(commits) != len(independent):
        return [], [
            "author commit enumeration is partial or disagrees with commit-object traversal"
        ]
    raw_authors = {commit.author_email for commit in independent.values()}
    if authors != raw_authors:
        return [], ["author identities disagree with commit-object traversal"]
    return sorted(authors, key=str.casefold), []


def _path_binding_findings(claimed: object, exact_paths: list[str]) -> list[str]:
    if not isinstance(claimed, list) or not all(isinstance(item, str) for item in claimed):
        return ["touched_paths must be a sorted list of repository paths"]
    normalized = [_norm(item) for item in claimed]
    findings: list[str] = []
    if any(item != normalized_item for item, normalized_item in zip(claimed, normalized)):
        findings.append("touched_paths must use exact canonical Git path syntax")
    if not normalized:
        findings.append("touched_paths must not be empty")
    if normalized != sorted(set(normalized)):
        findings.append("touched_paths must be sorted and unique")
    claimed_set = set(normalized)
    exact_set = set(exact_paths)
    missing = sorted(exact_set - claimed_set)
    extra = sorted(claimed_set - exact_set)
    if missing:
        findings.append("review record has missing touched path(s): " + ", ".join(missing))
    if extra:
        findings.append("review record has extra touched path(s): " + ", ".join(extra))
    return findings


def _common_record_findings(
    record: dict[str, Any],
    *,
    schema: str,
    state: str,
    allowed_keys: frozenset[str],
    exact_paths: list[str],
    expected_base: str | None,
) -> list[str]:
    findings: list[str] = []
    unknown = sorted(set(record) - allowed_keys)
    missing = sorted(allowed_keys - set(record))
    if unknown:
        findings.append("review record contains unknown key(s): " + ", ".join(unknown))
    if missing:
        findings.append("review record is missing key(s): " + ", ".join(missing))
    if record.get("schema") != schema:
        findings.append(f"review record schema must be {schema}")
    if record.get("state") != state:
        findings.append(f"review record state must be {state}")

    base_commit = record.get("base_commit")
    if not isinstance(base_commit, str) or GIT_OID_RE.fullmatch(base_commit) is None:
        findings.append("base_commit must be a full lowercase Git SHA")
    elif expected_base is not None and base_commit != expected_base:
        findings.append("base_commit does not match the discovered authoritative merge-base")

    for key in ("reviewed_head", "reviewed_tree"):
        value = record.get(key)
        if not isinstance(value, str) or GIT_OID_RE.fullmatch(value) is None:
            findings.append(f"{key} must be a full lowercase Git SHA")

    findings.extend(_path_binding_findings(record.get("touched_paths"), exact_paths))

    content_digest = record.get("content_digest")
    if not isinstance(content_digest, str) or DIGEST_RE.fullmatch(content_digest) is None:
        findings.append("content_digest must be sha256 followed by 64 lowercase hex digits")

    author_emails = record.get("author_emails")
    if not isinstance(author_emails, list) or not author_emails:
        findings.append("author_emails must be a nonempty sorted list of canonical identities")
    elif not all(
        isinstance(item, str)
        and item.startswith("email:")
        and IDENTITY_RE.fullmatch(item)
        for item in author_emails
    ):
        findings.append("author_emails contains a malformed canonical identity")
    else:
        if author_emails != sorted(set(author_emails), key=str.casefold):
            findings.append("author_emails must be sorted and unique")

    # ``author_ids`` is retained as the v2 schema key, but it binds every
    # authenticated GitHub principal attached to a commit: both the ``author``
    # and ``committer`` roles.  Treating it as author-only would let a reviewer
    # who committed another contributor's patch appear independent.
    author_ids = record.get("author_ids")
    valid_author_ids: list[int] = []
    if not isinstance(author_ids, list) or not author_ids:
        findings.append(
            "author_ids must be a nonempty sorted list of authenticated commit principal identities"
        )
    elif not all(type(item) is int and item > 0 for item in author_ids):
        findings.append("author_ids contains a malformed authenticated commit principal identity")
    else:
        valid_author_ids = author_ids
        if author_ids != sorted(set(author_ids)):
            findings.append("author_ids must be sorted and unique")

    reviewer_id = record.get("reviewer_id")
    reviewer_login = record.get("reviewer_login")
    if type(reviewer_id) is not int or reviewer_id <= 0:
        findings.append("reviewer identity is malformed")
    elif reviewer_id in valid_author_ids:
        findings.append("reviewer identity overlaps an author or committer")
    if not isinstance(reviewer_login, str) or GITHUB_LOGIN_RE.fullmatch(reviewer_login) is None:
        findings.append("reviewer login is malformed")

    for key, label in (
        ("repository", "base repository"),
        ("head_repository", "head repository"),
    ):
        repository = record.get(key)
        if not isinstance(repository, str) or GITHUB_REPOSITORY_RE.fullmatch(repository) is None:
            findings.append(f"{label} must be a canonical owner/name")
    for key, label in (
        ("repository_id", "base repository id"),
        ("head_repository_id", "head repository id"),
        ("pull_request_id", "pull request id"),
        ("pull_request_number", "pull request number"),
    ):
        value = record.get(key)
        if type(value) is not int or value <= 0:
            findings.append(f"{label} must be a positive integer")
    if record.get("review_state") != "APPROVED":
        findings.append("review_state must be APPROVED")

    if record.get("verdict") != "pass":
        findings.append("review verdict must be pass")
    blocking = record.get("blocking_findings")
    if not isinstance(blocking, list):
        findings.append("blocking_findings must be a list")
    elif blocking:
        findings.append("blocking_findings must be empty")

    review_scope = record.get("review_scope")
    if (
        not isinstance(review_scope, str)
        or "reviewed_head" not in review_scope
        or "does not extend or backdate" not in review_scope
    ):
        findings.append(
            "review_scope must state that content proof does not extend or backdate reviewed_head coverage"
        )
    return findings


def _review_record_append_only_findings(entries: list[ChangeEntry]) -> list[str]:
    findings: list[str] = []
    zero = "0" * 40
    for entry in entries:
        if not is_review_record(entry.path):
            continue
        if entry.status != "A":
            findings.append(
                f"structured review records are append-only; historical {entry.status} is RED: "
                f"{entry.path}"
            )
            continue
        if not (
            entry.old_mode == "000000"
            and entry.old_oid == zero
            and entry.new_mode == "100644"
            and entry.new_oid != zero
        ):
            findings.append(
                "new structured review record must be a regular 100644 blob: " + entry.path
            )
    return findings


def _review_record_history_findings(
    root: Path,
    base_commit: str,
    head_commit: str,
) -> list[str]:
    """Reject review-record mutation or removal on every candidate commit edge."""
    commits, graph_problems = _independent_commit_range(base_commit, head_commit, root)
    if graph_problems:
        return [
            "structured review record history enumeration failed closed: " + problem
            for problem in graph_problems
        ]
    commit_cache: dict[str, RawCommit] = {}
    tree_cache: dict[str, dict[str, tuple[str, str]]] = {}
    snapshot_cache: dict[str, dict[str, tuple[str, str]]] = {}
    findings: set[str] = set()
    introductions: dict[str, set[str]] = {}

    def snapshot(commit: str) -> dict[str, tuple[str, str]] | None:
        if commit in snapshot_cache:
            return snapshot_cache[commit]
        value, problems = _tree_snapshot_from_commit(
            root,
            commit,
            commit_cache=commit_cache,
            tree_cache=tree_cache,
        )
        if problems:
            findings.update(
                "structured review record history enumeration failed closed: " + problem
                for problem in problems
            )
            return None
        snapshot_cache[commit] = value
        return value

    for commit_oid, commit in sorted(commits.items()):
        after = snapshot(commit_oid)
        parent_snapshots: list[dict[str, tuple[str, str]]] = []
        for parent_oid in commit.parents:
            before = snapshot(parent_oid)
            if before is None or after is None:
                continue
            parent_snapshots.append(before)
            record_paths = sorted(
                path for path in set(before) | set(after) if is_review_record(path)
            )
            for path in record_paths:
                old = before.get(path)
                new = after.get(path)
                if old is None:
                    if new is not None and new[0] != "100644":
                        findings.add(
                            "structured review record history is append-only: new record "
                            "is not 100644 at " + commit_oid + ": " + path
                        )
                elif new != old:
                    findings.add(
                        "structured review record history is append-only: record changed "
                        "or was removed at " + commit_oid + ": " + path
                    )
        if after is not None and len(parent_snapshots) == len(commit.parents):
            for path in sorted(path for path in after if is_review_record(path)):
                if all(path not in parent for parent in parent_snapshots):
                    introductions.setdefault(path, set()).add(commit_oid)
    for path, commits_with_introduction in sorted(introductions.items()):
        if len(commits_with_introduction) > 1:
            findings.add(
                "structured review record history is append-only: record was introduced "
                "more than once: " + path
            )
    return sorted(findings)


def _post_review_trust_findings(
    reviewed_head: str,
    head_commit: str,
    root: Path,
) -> list[str]:
    """Reject every reviewed-lineage trust touch, including transient edits.

    A merge may carry a parent whose history is outside the reviewed lineage
    only when the merge's exact trust snapshot and its byte digest both equal
    ``reviewed_head``. This admits the already-reviewed bytes at the topology
    join without extending review coverage over the outside parent.
    """
    commits, problems = _independent_commit_range(reviewed_head, head_commit, root)
    if problems:
        return problems
    if not commits:
        return []
    allowed_parents = set(commits) | {reviewed_head}
    commit_cache: dict[str, RawCommit] = dict(commits)
    tree_cache: dict[str, dict[str, tuple[str, str]]] = {}
    snapshots: dict[str, dict[str, tuple[str, str]]] = {}
    snapshot_digests: dict[str, str] = {}

    def trust_snapshot(oid: str) -> tuple[dict[str, tuple[str, str]], list[str]]:
        if oid not in snapshots:
            snapshot, snapshot_problems = _tree_snapshot_from_commit(
                root,
                oid,
                commit_cache=commit_cache,
                tree_cache=tree_cache,
            )
            if snapshot_problems:
                return {}, snapshot_problems
            snapshots[oid] = {
                path: identity for path, identity in snapshot.items() if is_trust_kernel(path)
            }
        return snapshots[oid], []

    def trust_snapshot_digest(oid: str) -> tuple[str | None, list[str]]:
        if oid in snapshot_digests:
            return snapshot_digests[oid], []
        snapshot, snapshot_problems = trust_snapshot(oid)
        if snapshot_problems:
            return None, snapshot_problems
        digest = hashlib.sha256()
        digest.update(b"garnet.trust_kernel.snapshot/v1\0")
        for path, (mode, blob_oid) in sorted(snapshot.items()):
            blob, blob_problems = _read_blob_oid(
                root,
                blob_oid,
                f"trust snapshot {oid}: {path}",
            )
            if blob_problems:
                return None, blob_problems
            assert blob is not None
            digest.update(path.encode("utf-8"))
            digest.update(b"\0")
            digest.update(mode.encode("ascii"))
            digest.update(b"\0")
            digest.update(hashlib.sha256(blob).digest())
            digest.update(b"\0")
        value = "sha256:" + digest.hexdigest()
        snapshot_digests[oid] = value
        return value, []

    reviewed_lineage = {reviewed_head}
    unresolved = set(commits)
    while True:
        admitted = {
            oid
            for oid in unresolved
            if any(parent in reviewed_lineage for parent in commits[oid].parents)
        }
        if not admitted:
            break
        reviewed_lineage.update(admitted)
        unresolved.difference_update(admitted)

    findings: list[str] = []
    reviewed_snapshot, reviewed_snapshot_problems = trust_snapshot(reviewed_head)
    findings.extend(reviewed_snapshot_problems)
    reviewed_digest, reviewed_digest_problems = trust_snapshot_digest(reviewed_head)
    findings.extend(reviewed_digest_problems)

    accepted_outside: set[str] = set()
    accepted_merge_parents: dict[str, set[str]] = {}
    if not reviewed_snapshot_problems and not reviewed_digest_problems:
        for oid in sorted(commits):
            commit = commits[oid]
            if oid not in reviewed_lineage or len(commit.parents) < 2:
                continue
            outside_parents = {
                parent
                for parent in commit.parents
                if parent in commits and parent not in reviewed_lineage
            }
            if not outside_parents:
                continue
            current, current_problems = trust_snapshot(oid)
            findings.extend(current_problems)
            current_digest, digest_problems = trust_snapshot_digest(oid)
            findings.extend(digest_problems)
            if (
                not current_problems
                and not digest_problems
                and current == reviewed_snapshot
                and current_digest == reviewed_digest
            ):
                accepted_merge_parents[oid] = outside_parents
                pending = list(outside_parents)
                while pending:
                    outside_oid = pending.pop()
                    if outside_oid in accepted_outside or outside_oid in reviewed_lineage:
                        continue
                    accepted_outside.add(outside_oid)
                    pending.extend(
                        parent
                        for parent in commits[outside_oid].parents
                        if parent in commits
                    )

    for oid in sorted(commits):
        if oid in accepted_outside:
            continue
        commit = commits[oid]
        relevant = [parent for parent in commit.parents if parent in allowed_parents]
        if not relevant:
            findings.append(
                f"post-review commit-object traversal found no reviewed-lineage parent for {oid}"
            )
            continue
        current, current_problems = trust_snapshot(oid)
        if current_problems:
            findings.extend(current_problems)
            continue
        for parent in relevant:
            if parent in accepted_merge_parents.get(oid, set()):
                continue
            before, before_problems = trust_snapshot(parent)
            if before_problems:
                findings.extend(before_problems)
                continue
            if current != before:
                merge_label = " merge" if len(commit.parents) > 1 else ""
                findings.append(
                    f"post-review trust touch in{merge_label} commit {oid} versus parent {parent}"
                )
    return findings


def _transport_problem_codes(result: object) -> tuple[list[str], bool]:
    raw = getattr(result, "problems", None)
    if not isinstance(raw, (tuple, list)):
        return ["response-shape"], True
    codes: list[str] = []
    malformed = False
    for item in raw:
        code = getattr(item, "code", None)
        if not isinstance(code, str) or not code or len(code) > 64:
            malformed = True
        else:
            codes.append(code)
    return sorted(set(codes)), malformed


def _review_projection(value: object) -> tuple[tuple[int, int, str, str, str] | None, list[str]]:
    if not isinstance(value, dict):
        return None, ["authenticated review row is malformed"]
    user = value.get("user")
    review_id = value.get("id")
    reviewer_id = user.get("id") if isinstance(user, dict) else None
    reviewer_login = user.get("login") if isinstance(user, dict) else None
    state = value.get("state")
    commit_id = value.get("commit_id")
    if (
        type(review_id) is not int
        or review_id <= 0
        or type(reviewer_id) is not int
        or reviewer_id <= 0
        or not isinstance(reviewer_login, str)
        or GITHUB_LOGIN_RE.fullmatch(reviewer_login) is None
        or not isinstance(state, str)
        or state not in {"APPROVED", "CHANGES_REQUESTED", "COMMENTED", "DISMISSED", "PENDING"}
        or not isinstance(commit_id, str)
        or GIT_OID_RE.fullmatch(commit_id) is None
    ):
        return None, ["authenticated review row is malformed"]
    return (review_id, reviewer_id, reviewer_login, state, commit_id), []


def _transport_collection(
    transport: object,
    path: str,
    label: str,
) -> tuple[tuple[object, ...], list[str]]:
    try:
        result = transport.get_collection(path)
    except Exception:
        return (), [f"authenticated {label} enumeration failed closed: transport-failure"]
    codes, malformed = _transport_problem_codes(result)
    if malformed:
        return (), [f"authenticated {label} enumeration failed closed: response-shape"]
    if codes:
        return (), [f"authenticated {label} enumeration failed closed: {', '.join(codes)}"]
    rows = getattr(result, "rows", None)
    page_count = getattr(result, "page_count", None)
    byte_count = getattr(result, "byte_count", None)
    if not isinstance(rows, tuple) or type(page_count) is not int or page_count <= 0 or type(byte_count) is not int or byte_count < 0:
        return (), [f"authenticated {label} enumeration failed closed: response-shape"]
    return rows, []


def _transport_object(
    transport: object,
    path: str,
    label: str,
) -> tuple[dict[str, object] | None, list[str]]:
    try:
        result = transport.get_object(path)
    except Exception:
        return None, [f"authenticated {label} read failed closed: transport-failure"]
    codes, malformed = _transport_problem_codes(result)
    if malformed:
        return None, [f"authenticated {label} read failed closed: response-shape"]
    if codes:
        return None, [f"authenticated {label} read failed closed: {', '.join(codes)}"]
    value = getattr(result, "value", None)
    byte_count = getattr(result, "byte_count", None)
    if not isinstance(value, dict) or type(byte_count) is not int or byte_count < 0:
        return None, [f"authenticated {label} read failed closed: response-shape"]
    return value, []


def _authenticated_review_findings(
    record: dict[str, Any],
    *,
    transport: object | None,
    repository: str | None,
    pull_request: int | None,
    base_commit: str,
    head_commit: str,
    root: Path,
) -> list[str]:
    if transport is None:
        return ["authenticated GitHub review transport is required for a trust-kernel change"]
    findings: list[str] = []
    if not isinstance(repository, str) or GITHUB_REPOSITORY_RE.fullmatch(repository) is None:
        findings.append("an explicit canonical GitHub repository is required")
    elif record.get("repository") != repository:
        findings.append("review record repository does not match the explicit transport repository")
    if type(pull_request) is not int or pull_request <= 0:
        findings.append("an explicit positive GitHub pull request number is required")
    elif record.get("pull_request_number") != pull_request:
        findings.append("review record pull request number does not match the explicit request")
    if type(pull_request) is not int or pull_request <= 0:
        return findings

    reviews, review_problems = _transport_collection(
        transport, f"pulls/{pull_request}/reviews", "review"
    )
    findings.extend(review_problems)
    projections: dict[int, tuple[int, int, str, str, str]] = {}
    for row in reviews:
        projection, row_problems = _review_projection(row)
        findings.extend(row_problems)
        if projection is None:
            continue
        if projection[0] in projections:
            findings.append(f"authenticated review enumeration has duplicate review id {projection[0]}")
        else:
            projections[projection[0]] = projection
    reviewer_id = record.get("reviewer_id")
    reviewer_login = record.get("reviewer_login")
    decisive = sorted(
        (
            projection
            for projection in projections.values()
            if projection[1] == reviewer_id
            and projection[3] in {"APPROVED", "CHANGES_REQUESTED", "DISMISSED"}
        ),
        key=lambda projection: projection[0],
    )
    selected = decisive[-1] if decisive else None
    if not review_problems and selected is None:
        findings.append(
            "authenticated decisive review from the recorded independent reviewer is absent"
        )
    elif (
        not review_problems
        and selected is not None
        and (selected[3] != "APPROVED" or selected[4] != head_commit)
    ):
        findings.append(
            "latest decisive review from the recorded independent reviewer must be "
            "APPROVED at the exact current candidate head"
        )

    direct: dict[str, object] | None = None
    direct_problems: list[str] = []
    if selected is not None:
        direct, direct_problems = _transport_object(
            transport,
            f"pulls/{pull_request}/reviews/{selected[0]}",
            "review object",
        )
        findings.extend(direct_problems)
    direct_projection: tuple[int, int, str, str, str] | None = None
    if direct is not None:
        direct_projection, projection_problems = _review_projection(direct)
        findings.extend(projection_problems)
    if selected is not None and direct_projection is not None and selected != direct_projection:
        findings.append("authenticated review collection and direct object disagree")
    if selected is not None:
        _, authenticated_reviewer_id, authenticated_reviewer_login, state, approved_commit = selected
        if state != "APPROVED" or record.get("review_state") != "APPROVED":
            findings.append("authenticated review state must be APPROVED")
        if approved_commit != head_commit:
            findings.append("authenticated review must bind the exact current candidate head")
        if (
            authenticated_reviewer_id != record.get("reviewer_id")
            or authenticated_reviewer_login != record.get("reviewer_login")
        ):
            findings.append("authenticated reviewer identity does not match the review record")

    pr, pr_problems = _transport_object(transport, f"pulls/{pull_request}", "pull request")
    findings.extend(pr_problems)
    api_head: str | None = None
    if pr is not None:
        head = pr.get("head")
        base = pr.get("base")
        head_repo = head.get("repo") if isinstance(head, dict) else None
        base_repo = base.get("repo") if isinstance(base, dict) else None
        api_head_value = head.get("sha") if isinstance(head, dict) else None
        if not isinstance(api_head_value, str) or GIT_OID_RE.fullmatch(api_head_value) is None:
            findings.append("authenticated pull request head is malformed")
        else:
            api_head = api_head_value
            if api_head != head_commit:
                findings.append("authenticated pull request head does not match the exact candidate head")
        api_pull_request_number = pr.get("number")
        if type(api_pull_request_number) is not int or api_pull_request_number <= 0:
            findings.append("authenticated pull request number is malformed")
        elif api_pull_request_number != pull_request:
            findings.append("authenticated pull request number does not match")
        api_pull_request_id = pr.get("id")
        if type(api_pull_request_id) is not int or api_pull_request_id <= 0:
            findings.append("authenticated pull request id is malformed")
        elif api_pull_request_id != record.get("pull_request_id"):
            findings.append("authenticated pull request id does not match the review record")
        expected_repositories = (
            (
                "head",
                head_repo,
                record.get("head_repository"),
                record.get("head_repository_id"),
            ),
            ("base", base_repo, repository, record.get("repository_id")),
        )
        for label, repo_value, expected_name, expected_id in expected_repositories:
            if not isinstance(repo_value, dict):
                findings.append(f"authenticated {label} repository identity is malformed")
                continue
            api_repository_id = repo_value.get("id")
            api_repository_name = repo_value.get("full_name")
            malformed = False
            if type(api_repository_id) is not int or api_repository_id <= 0:
                findings.append(f"authenticated {label} repository id is malformed")
                malformed = True
            if (
                not isinstance(api_repository_name, str)
                or GITHUB_REPOSITORY_RE.fullmatch(api_repository_name) is None
            ):
                findings.append(f"authenticated {label} repository name is malformed")
                malformed = True
            if not malformed and (
                api_repository_name != expected_name or api_repository_id != expected_id
            ):
                findings.append(
                    f"authenticated {label} repository id/name does not match the review record"
                )

    commits, commit_problems = _transport_collection(
        transport, f"pulls/{pull_request}/commits", "commit"
    )
    findings.extend(commit_problems)
    api_commit_ids: list[str] = []
    api_principal_ids: set[int] = set()
    for row in commits:
        if not isinstance(row, dict):
            findings.append("authenticated commit enumeration contains a malformed row")
            continue
        sha = row.get("sha")
        if not isinstance(sha, str) or GIT_OID_RE.fullmatch(sha) is None:
            findings.append("authenticated commit enumeration contains a malformed row")
            continue
        api_commit_ids.append(sha)
        for role in ("author", "committer"):
            principal = row.get(role)
            principal_id = principal.get("id") if isinstance(principal, dict) else None
            principal_login = principal.get("login") if isinstance(principal, dict) else None
            if (
                type(principal_id) is not int
                or principal_id <= 0
                or not isinstance(principal_login, str)
                or GITHUB_LOGIN_RE.fullmatch(principal_login) is None
            ):
                findings.append(
                    f"authenticated commit enumeration contains a malformed {role} identity"
                )
                continue
            api_principal_ids.add(principal_id)
    if len(api_commit_ids) != len(set(api_commit_ids)):
        findings.append("authenticated commit enumeration contains a duplicate commit id")
    if api_head is not None and not commit_problems:
        presented, presented_problems = _presented_commit_ids(base_commit, api_head, root)
        findings.extend(presented_problems)
        independent, independent_problems = _independent_commit_range(base_commit, api_head, root)
        findings.extend(independent_problems)
        if (
            not presented_problems
            and not independent_problems
            and (set(presented) != set(independent) or len(presented) != len(independent))
        ):
            findings.append(
                "local commit enumeration is partial or disagrees with commit-object traversal"
            )
        if not presented_problems and api_commit_ids != presented:
            findings.append(
                "authenticated commit enumeration is partial or disagrees with the exact local range"
            )
    claimed_author_ids = record.get("author_ids")
    if isinstance(claimed_author_ids, list) and sorted(api_principal_ids) != claimed_author_ids:
        findings.append(
            "review record authors do not match authenticated GitHub author/committer identity union"
        )
    if type(record.get("reviewer_id")) is int and record.get("reviewer_id") in api_principal_ids:
        findings.append("reviewer identity overlaps an authenticated commit principal")
    return findings


def _verify_premerge_record(
    record: dict[str, Any],
    *,
    exact_entries: list[ChangeEntry],
    base_commit: str,
    head_commit: str,
    root: Path,
    github_transport: object | None,
    repository: str | None,
    pull_request: int | None,
) -> list[str]:
    exact_paths = sorted(entry.path for entry in exact_entries)
    findings = _common_record_findings(
        record,
        schema=RECORD_SCHEMA,
        state="premerge",
        allowed_keys=PREMERGE_KEYS,
        exact_paths=exact_paths,
        expected_base=base_commit,
    )
    reviewed_head = record.get("reviewed_head")
    reviewed_tree = record.get("reviewed_tree")
    claimed_digest = record.get("content_digest")
    if not isinstance(reviewed_head, str) or GIT_OID_RE.fullmatch(reviewed_head) is None:
        return findings

    resolved_head, head_findings = _resolve_commit(reviewed_head, "reviewed_head", root)
    findings.extend(head_findings)
    if resolved_head is None:
        return findings
    ancestor = _git_bytes(root, "merge-base", "--is-ancestor", resolved_head, head_commit)
    if ancestor.timed_out:
        findings.append("reviewed_head ancestry check timed out")
    elif ancestor.returncode == 1:
        findings.append("reviewed_head is not an ancestor of the current premerge head")
    elif ancestor.returncode != 0:
        findings.append("reviewed_head ancestry could not be verified")

    base_ancestor = _git_bytes(root, "merge-base", "--is-ancestor", base_commit, resolved_head)
    if base_ancestor.timed_out:
        findings.append("reviewed base ancestry check timed out")
    elif base_ancestor.returncode == 1:
        findings.append("base_commit is not an ancestor of reviewed_head")
    elif base_ancestor.returncode != 0:
        findings.append("reviewed base ancestry could not be verified")

    actual_tree, tree_findings = _resolve_tree(resolved_head, "reviewed_head", root)
    findings.extend(tree_findings)
    if actual_tree is not None and reviewed_tree != actual_tree:
        findings.append("reviewed_tree mismatch for reviewed_head")

    reviewed_entries, reviewed_diff_findings = _diff_entries(
        root, f"{base_commit}..{resolved_head}"
    )
    findings.extend(reviewed_diff_findings)
    if not reviewed_diff_findings:
        reviewed_independent, independent_findings = _independent_tree_diff(
            root, base_commit, resolved_head
        )
        findings.extend(independent_findings)
        if not independent_findings and sorted(reviewed_entries) != sorted(reviewed_independent):
            findings.append(
                "reviewed_head diff is partial or disagrees with independent tree-object traversal"
            )
    reviewed_trust = sorted(
        (entry for entry in reviewed_entries if is_trust_kernel(entry.path)),
        key=lambda entry: entry.path,
    )
    if [entry.path for entry in reviewed_trust] != exact_paths:
        findings.append("reviewed_head trust path set does not match current candidate")

    if isinstance(claimed_digest, str) and DIGEST_RE.fullmatch(claimed_digest):
        for label, entries in (("reviewed_head", reviewed_trust), ("current head", exact_entries)):
            actual_digest, digest_findings = compute_change_digest(entries, root)
            findings.extend(digest_findings)
            if actual_digest is not None and actual_digest != claimed_digest:
                findings.append(f"content digest mismatch at {label}")

    derived_authors, author_findings = derive_author_identities(base_commit, resolved_head, root)
    findings.extend(author_findings)
    if derived_authors and record.get("author_emails") != derived_authors:
        findings.append("review record authors do not match the exact base..reviewed_head commit set")
    findings.extend(_post_review_trust_findings(resolved_head, head_commit, root))
    findings.extend(
        _authenticated_review_findings(
            record,
            transport=github_transport,
            repository=repository,
            pull_request=pull_request,
            base_commit=base_commit,
            head_commit=head_commit,
            root=root,
        )
    )
    return findings


def _select_linear_record_path(
    records: dict[str, tuple[str, str]],
    strict_ancestors: set[tuple[str, str]],
) -> tuple[str | None, list[str]]:
    """Select one tip-most record when introduction and review order agree."""
    if not records:
        return None, ["structured review record is missing"]
    if len(records) == 1:
        return next(iter(records)), []

    findings: set[str] = set()
    paths = sorted(records)
    for index, left_path in enumerate(paths):
        left_intro, left_reviewed = records[left_path]
        for right_path in paths[index + 1 :]:
            right_intro, right_reviewed = records[right_path]
            intro_left_first = (left_intro, right_intro) in strict_ancestors
            intro_right_first = (right_intro, left_intro) in strict_ancestors
            reviewed_left_first = (left_reviewed, right_reviewed) in strict_ancestors
            reviewed_right_first = (right_reviewed, left_reviewed) in strict_ancestors

            if intro_left_first == intro_right_first:
                findings.add(
                    "record succession introductions must be strictly ordered by ancestry: "
                    f"{left_path} <> {right_path}"
                )
            if reviewed_left_first == reviewed_right_first:
                findings.add(
                    "record succession reviewed_heads must be strictly ordered by ancestry: "
                    f"{left_path} <> {right_path}"
                )
            elif intro_left_first != intro_right_first and (
                intro_left_first != reviewed_left_first
            ):
                findings.add(
                    "record succession tip-most record must bind the newest reviewed_head: "
                    f"{left_path} <> {right_path}"
                )

    if findings:
        return None, sorted(findings)

    terminal_paths = [
        path
        for path, (introduction, _) in records.items()
        if all(
            other_path == path
            or (other_introduction, introduction) in strict_ancestors
            for other_path, (other_introduction, _) in records.items()
        )
    ]
    if len(terminal_paths) != 1:
        return None, ["record succession requires exactly one tip-most record"]
    return terminal_paths[0], []


def _review_record_introductions(
    root: Path,
    base_commit: str,
    head_commit: str,
    record_paths: list[str],
) -> tuple[dict[str, str], list[str]]:
    """Derive each record's unique introduction commit from commit objects."""
    commits, graph_problems = _independent_commit_range(base_commit, head_commit, root)
    if graph_problems:
        return {}, [
            "record succession introduction enumeration failed closed: " + problem
            for problem in graph_problems
        ]

    commit_cache: dict[str, RawCommit] = dict(commits)
    tree_cache: dict[str, dict[str, tuple[str, str]]] = {}
    snapshot_cache: dict[str, dict[str, tuple[str, str]]] = {}
    findings: set[str] = set()
    introductions: dict[str, set[str]] = {path: set() for path in record_paths}

    def snapshot(commit: str) -> dict[str, tuple[str, str]] | None:
        if commit in snapshot_cache:
            return snapshot_cache[commit]
        value, problems = _tree_snapshot_from_commit(
            root,
            commit,
            commit_cache=commit_cache,
            tree_cache=tree_cache,
        )
        if problems:
            findings.update(
                "record succession introduction enumeration failed closed: " + problem
                for problem in problems
            )
            return None
        snapshot_cache[commit] = value
        return value

    for commit_oid, commit in sorted(commits.items()):
        after = snapshot(commit_oid)
        parent_snapshots = [snapshot(parent_oid) for parent_oid in commit.parents]
        if after is None or any(parent is None for parent in parent_snapshots):
            continue
        for path in record_paths:
            if path in after and all(
                parent is not None and path not in parent
                for parent in parent_snapshots
            ):
                introductions[path].add(commit_oid)

    resolved: dict[str, str] = {}
    for path, commit_oids in sorted(introductions.items()):
        if len(commit_oids) != 1:
            findings.add(
                "record succession requires exactly one introduction commit: " + path
            )
        else:
            resolved[path] = next(iter(commit_oids))
    return resolved, sorted(findings)


def _strict_ancestor_pairs(
    root: Path,
    commits: set[str],
) -> tuple[set[tuple[str, str]], list[str]]:
    pairs: set[tuple[str, str]] = set()
    findings: list[str] = []
    for ancestor in sorted(commits):
        for descendant in sorted(commits):
            if ancestor == descendant:
                continue
            result = _git_bytes(
                root, "merge-base", "--is-ancestor", ancestor, descendant
            )
            if result.timed_out:
                findings.append(
                    "record succession ancestry check timed out: "
                    f"{ancestor} -> {descendant}"
                )
            elif result.returncode == 0:
                pairs.add((ancestor, descendant))
            elif result.returncode != 1:
                findings.append(
                    "record succession ancestry could not be verified: "
                    f"{ancestor} -> {descendant}"
                )
    return pairs, findings


def _load_tip_review_record(
    record_paths: list[str],
    *,
    base_commit: str,
    head_commit: str,
    root: Path,
) -> tuple[str | None, dict[str, Any] | None, list[str]]:
    records: dict[str, dict[str, Any]] = {}
    reviewed_heads: dict[str, str] = {}
    findings: list[str] = []
    for path in record_paths:
        payload, payload_findings = _read_blob(root, head_commit, path)
        findings.extend(f"{path}: {problem}" for problem in payload_findings)
        if payload is None:
            continue
        record, record_findings = _load_canonical_record(payload)
        findings.extend(f"{path}: {problem}" for problem in record_findings)
        if record is None:
            continue
        records[path] = record
        reviewed_head = record.get("reviewed_head")
        if not isinstance(reviewed_head, str) or GIT_OID_RE.fullmatch(reviewed_head) is None:
            findings.append(f"{path}: reviewed_head must be a full Git object id")
            continue
        resolved, resolve_findings = _resolve_commit(reviewed_head, "reviewed_head", root)
        findings.extend(f"{path}: {problem}" for problem in resolve_findings)
        if resolved is not None:
            reviewed_heads[path] = resolved

    if findings or len(reviewed_heads) != len(record_paths):
        return None, None, findings

    introductions, introduction_findings = _review_record_introductions(
        root, base_commit, head_commit, record_paths
    )
    findings.extend(introduction_findings)
    if findings or len(introductions) != len(record_paths):
        return None, None, findings

    all_commits = set(introductions.values()) | set(reviewed_heads.values())
    strict_ancestors, ancestry_findings = _strict_ancestor_pairs(root, all_commits)
    findings.extend(ancestry_findings)
    if findings:
        return None, None, findings

    ordering = {
        path: (introductions[path], reviewed_heads[path]) for path in record_paths
    }
    selected_path, selection_findings = _select_linear_record_path(
        ordering, strict_ancestors
    )
    findings.extend(selection_findings)
    if selected_path is not None:
        for path, predecessor in sorted(records.items()):
            if path == selected_path:
                continue
            claimed_paths = predecessor.get("touched_paths")
            shape_paths = (
                sorted({_norm(item) for item in claimed_paths})
                if isinstance(claimed_paths, list)
                and all(isinstance(item, str) for item in claimed_paths)
                else []
            )
            shape_findings = _common_record_findings(
                predecessor,
                schema=RECORD_SCHEMA,
                state="premerge",
                allowed_keys=PREMERGE_KEYS,
                exact_paths=shape_paths,
                expected_base=base_commit,
            )
            findings.extend(
                f"{path}: predecessor {problem.removeprefix('review record ')}"
                for problem in shape_findings
            )
    return (
        selected_path,
        records.get(selected_path) if selected_path is not None else None,
        findings,
    )


def read_status(
    changed: list[str] | None = None,
    base: str | None = None,
    head: str = "HEAD",
    trailer: bool | None = None,
    root: Path = ROOT,
    extra_problems: list[str] | None = None,
    github_transport: object | None = None,
    repository: str | None = None,
    pull_request: int | None = None,
) -> TrustKernelReviewStatus:
    root = root.resolve()
    if changed is None:
        discovery = discover_changes(base=base, head=head, root=root)
    else:
        normalized = [_norm(path) for path in changed]
        discovery = DiscoveryResult(
            normalized,
            None,
            None,
            ["explicit changed paths cannot prove complete enumeration"],
            source="explicit-unverified",
        )

    touched = sorted({path for path in discovery.paths if is_trust_kernel(path)})
    trust_entries = sorted(
        (entry for entry in discovery.entries if is_trust_kernel(entry.path)),
        key=lambda entry: entry.path,
    )
    record_paths = sorted({path for path in discovery.paths if is_review_record(path)})
    legacy_paths = sorted({path for path in discovery.paths if is_review_companion(path)})
    problems = list(discovery.problems)
    problems.extend(_review_record_append_only_findings(discovery.entries))
    if extra_problems:
        problems.extend(extra_problems)
    if trailer:
        problems.append("trailer-only review is not accepted by rolling review v2")

    expected_digest: str | None = None
    if touched and discovery.head_commit is not None:
        expected_digest, digest_findings = compute_change_digest(trust_entries, root)
        problems.extend(digest_findings)

    record: dict[str, Any] | None = None
    selected_record_path: str | None = None
    selected_record_sha256: str | None = None
    reviewer: str | None = None
    reviewer_id: int | None = None
    reviewer_login: str | None = None
    reviewed_head: str | None = None
    reviewed_tree: str | None = None
    if (touched or record_paths) and discovery.ok:
        if not record_paths:
            if legacy_paths:
                problems.append(
                    "non-JSON or noncanonical legacy companion does not satisfy v2; "
                    "structured review record is missing"
                )
            else:
                problems.append("structured review record is missing")
        elif discovery.head_commit is not None and discovery.base_commit is not None:
            selected_record_path, record, selection_findings = _load_tip_review_record(
                record_paths,
                base_commit=discovery.base_commit,
                head_commit=discovery.head_commit,
                root=root,
            )
            problems.extend(selection_findings)
            if record is not None and selected_record_path is not None:
                raw_record, raw_record_findings = _read_blob(
                    root, discovery.head_commit, selected_record_path
                )
                problems.extend(raw_record_findings)
                if raw_record is not None:
                    selected_record_sha256 = hashlib.sha256(raw_record).hexdigest()
            if record is not None:
                problems.extend(
                    _verify_premerge_record(
                        record,
                        exact_entries=trust_entries,
                        base_commit=discovery.base_commit,
                        head_commit=discovery.head_commit,
                        root=root,
                        github_transport=github_transport,
                        repository=repository,
                        pull_request=pull_request,
                    )
                )
                reviewer_id = (
                    record.get("reviewer_id")
                    if type(record.get("reviewer_id")) is int
                    else None
                )
                reviewer_login = (
                    record.get("reviewer_login")
                    if isinstance(record.get("reviewer_login"), str)
                    else None
                )
                reviewer = f"github:{reviewer_login}" if reviewer_login is not None else None
                reviewed_head = (
                    record.get("reviewed_head")
                    if isinstance(record.get("reviewed_head"), str)
                    else None
                )
                reviewed_tree = (
                    record.get("reviewed_tree")
                    if isinstance(record.get("reviewed_tree"), str)
                    else None
                )

    if discovery.base_commit is not None and discovery.head_commit is not None:
        problems.extend(
            _review_record_history_findings(
                root, discovery.base_commit, discovery.head_commit
            )
        )
        problems.extend(
            _landed_marker_append_only_findings(
                root, discovery.base_commit, discovery.head_commit
            )
        )

    registry_ref = discovery.head_commit or head
    registry_findings = verify_repository_landed_markers(root, ref=registry_ref)
    problems.extend(f"registered landed marker: {problem}" for problem in registry_findings)

    return TrustKernelReviewStatus(
        schema=SCHEMA,
        ok=not problems,
        discovery_ok=discovery.ok,
        discovery_source=discovery.source,
        base_commit=discovery.base_commit,
        head_commit=discovery.head_commit,
        trust_kernel_touched=bool(touched),
        touched_paths=touched,
        review_record_present=bool(record_paths),
        review_record_path=selected_record_path,
        review_record_sha256=selected_record_sha256,
        reviewer=reviewer,
        reviewer_id=reviewer_id,
        reviewer_login=reviewer_login,
        reviewed_head=reviewed_head,
        reviewed_tree=reviewed_tree,
        content_digest=expected_digest,
        changed_count=len(discovery.paths),
        problems=problems,
        review_companion_present=bool(record_paths),
        companion_paths=record_paths,
        review_trailer_present=False,
    )


def verify_landed_review_marker(
    marker: object,
    *,
    root: Path = ROOT,
    main_ref: str = "refs/remotes/origin/main",
) -> list[str]:
    """Verify a squash-durable landed marker without branch-head ancestry.

    ``reviewed_head`` and ``reviewed_tree`` remain provenance fields.  They are
    format-checked but intentionally not resolved: squash merge may discard the
    premerge objects.  Landed truth comes only from ``main_ref`` first-parent,
    the recorded merged tree, and the exact first-parent-to-merge edge paths
    and bytes.  The reviewed base remains independently bound and earlier on
    main, but unrelated main advances before the squash do not broaden scope.
    """
    if not isinstance(marker, dict):
        return ["landed review marker is missing"]
    findings = _common_record_findings(
        marker,
        schema=MARKER_SCHEMA,
        state="landed",
        allowed_keys=LANDED_KEYS,
        exact_paths=(
            sorted({_norm(path) for path in marker.get("touched_paths", [])})
            if isinstance(marker.get("touched_paths"), list)
            and all(isinstance(path, str) for path in marker.get("touched_paths", []))
            else []
        ),
        expected_base=None,
    )
    merged_commit = marker.get("merged_commit")
    if merged_commit is None:
        findings.append("merged_commit is missing")
    elif not isinstance(merged_commit, str) or GIT_OID_RE.fullmatch(merged_commit) is None:
        findings.append("merged_commit must be a full lowercase Git SHA")
    merged_tree = marker.get("merged_tree")
    if not isinstance(merged_tree, str) or GIT_OID_RE.fullmatch(merged_tree) is None:
        findings.append("merged_tree must be a full lowercase Git tree SHA")
    review_record_path = marker.get("review_record_path")
    if not isinstance(review_record_path, str) or not is_review_record(review_record_path):
        findings.append("review_record_path must name a canonical W_TRUST *.review.json record")
    review_record_sha256 = marker.get("review_record_sha256")
    if not isinstance(review_record_sha256, str) or SHA256_RE.fullmatch(review_record_sha256) is None:
        findings.append("review_record_sha256 must be 64 lowercase hex digits")
    base_commit = marker.get("base_commit")
    if findings and (
        not isinstance(merged_commit, str)
        or GIT_OID_RE.fullmatch(merged_commit) is None
        or not isinstance(base_commit, str)
        or GIT_OID_RE.fullmatch(base_commit) is None
    ):
        return findings
    assert isinstance(merged_commit, str)
    assert isinstance(base_commit, str)

    main_commit, main_findings = _resolve_commit(main_ref, "authoritative upstream main ref", root)
    findings.extend(main_findings)
    landed_commit, landed_findings = _resolve_commit(merged_commit, "merged_commit", root)
    findings.extend(landed_findings)
    if main_commit is None or landed_commit is None:
        return findings

    history = _git_bytes(root, "rev-list", "--first-parent", main_commit)
    if history.timed_out:
        findings.append("upstream main first-parent history enumeration timed out")
        return findings
    if history.returncode != 0:
        findings.append("upstream main first-parent history could not be enumerated")
        return findings
    try:
        first_parent = history.stdout.decode("ascii", errors="strict").splitlines()
    except UnicodeDecodeError:
        findings.append("upstream main first-parent history is malformed")
        return findings
    if not first_parent or any(GIT_OID_RE.fullmatch(oid) is None for oid in first_parent):
        findings.append("upstream main first-parent history is malformed")
        return findings
    if landed_commit not in first_parent:
        findings.append("merged_commit is absent from upstream main first-parent history")
    if base_commit not in first_parent:
        findings.append("base_commit is absent from upstream main first-parent history")
    if landed_commit in first_parent and base_commit in first_parent:
        if first_parent.index(landed_commit) >= first_parent.index(base_commit):
            findings.append("merged_commit does not follow base_commit on main first-parent history")
    if findings:
        return findings

    landed_object, landed_object_findings = _read_raw_commit(root, landed_commit)
    findings.extend(landed_object_findings)
    if landed_object is None:
        return findings
    if not landed_object.parents:
        findings.append("merged_commit has no exact first-parent landing edge")
        return findings
    landing_parent = landed_object.parents[0]
    landed_index = first_parent.index(landed_commit)
    if (
        landed_index + 1 >= len(first_parent)
        or first_parent[landed_index + 1] != landing_parent
    ):
        findings.append("merged_commit first parent disagrees with upstream main history")
        return findings

    actual_tree, tree_findings = _resolve_tree(landed_commit, "merged_commit", root)
    findings.extend(tree_findings)
    if actual_tree is not None and actual_tree != merged_tree:
        findings.append("merged_tree mismatch for merged_commit")

    landed_entries, diff_findings = _diff_entries(
        root, f"{landing_parent}..{landed_commit}"
    )
    findings.extend(diff_findings)
    if not diff_findings:
        independent_entries, independent_findings = _independent_tree_diff(
            root, landing_parent, landed_commit
        )
        findings.extend(independent_findings)
        if not independent_findings and sorted(landed_entries) != sorted(independent_entries):
            findings.append(
                "exact first-parent landing edge is partial or disagrees with "
                "independent tree-object traversal"
            )
    landed_trust = sorted(
        (entry for entry in landed_entries if is_trust_kernel(entry.path)),
        key=lambda entry: entry.path,
    )
    exact_touched = [entry.path for entry in landed_trust]
    findings.extend(_path_binding_findings(marker.get("touched_paths"), exact_touched))
    if isinstance(review_record_path, str):
        landed_record_entries = [
            entry for entry in landed_entries if entry.path == review_record_path
        ]
        zero = "0" * 40
        if (
            len(landed_record_entries) != 1
            or landed_record_entries[0].status != "A"
            or landed_record_entries[0].old_mode != "000000"
            or landed_record_entries[0].old_oid != zero
            or landed_record_entries[0].new_mode != "100644"
            or landed_record_entries[0].new_oid == zero
        ):
            findings.append(
                "committed premerge review record must be a new regular 100644 blob "
                "on the exact first-parent landing edge"
            )

    claimed_digest = marker.get("content_digest")
    if isinstance(claimed_digest, str) and DIGEST_RE.fullmatch(claimed_digest):
        actual_digest, digest_findings = compute_change_digest(landed_trust, root)
        findings.extend(digest_findings)
        if actual_digest is not None and actual_digest != claimed_digest:
            findings.append("exact first-parent landing edge content digest mismatch")

    if isinstance(review_record_path, str) and is_review_record(review_record_path):
        record_payload, record_read_findings = _read_blob(
            root, landed_commit, review_record_path
        )
        findings.extend(record_read_findings)
        if record_payload is not None:
            actual_record_sha256 = hashlib.sha256(record_payload).hexdigest()
            if actual_record_sha256 != review_record_sha256:
                findings.append("committed premerge review record SHA-256 mismatch")
            premerge_record, record_parse_findings = _load_canonical_record(record_payload)
            findings.extend(record_parse_findings)
            if premerge_record is not None:
                findings.extend(
                    _common_record_findings(
                        premerge_record,
                        schema=RECORD_SCHEMA,
                        state="premerge",
                        allowed_keys=PREMERGE_KEYS,
                        exact_paths=exact_touched,
                        expected_base=base_commit,
                    )
                )
                if premerge_record.get("schema") != RECORD_SCHEMA or premerge_record.get("state") != "premerge":
                    findings.append("committed review record is not a premerge v2 record")
                shared_claim_keys = PREMERGE_KEYS - {"schema", "state"}
                mismatched = sorted(
                    key for key in shared_claim_keys if marker.get(key) != premerge_record.get(key)
                )
                if mismatched:
                    findings.append(
                        "landed marker claim does not match committed premerge record: "
                        + ", ".join(mismatched)
                    )
    return findings


def _registry_paths_from_snapshot(
    root: Path,
    snapshot: dict[str, tuple[str, str]],
    *,
    label: str,
) -> tuple[list[str] | None, list[str]]:
    """Read one registry snapshot without accepting path aliases or type drift."""
    entry = snapshot.get(LANDED_REGISTRY_PATH)
    discovered = sorted(
        path
        for path in snapshot
        if path.startswith(LANDED_MARKER_PREFIX) and path.endswith(LANDED_MARKER_SUFFIX)
    )
    if entry is None:
        if discovered:
            return None, [f"{label} has landed markers but no registry"]
        return None, []
    mode, oid = entry
    if mode != "100644":
        return None, [f"{label} landed marker registry must be a regular 100644 blob"]
    payload, read_problems = _read_blob_oid(root, oid, f"{label} landed marker registry")
    if payload is None:
        return None, read_problems
    registry, parse_problems = _load_canonical_record(payload)
    if registry is None:
        return None, [
            problem.replace("review record", f"{label} landed marker registry")
            for problem in parse_problems
        ]
    findings: list[str] = []
    if set(registry) != {"markers", "schema"}:
        findings.append(f"{label} landed marker registry must contain exactly markers and schema")
    if registry.get("schema") != LANDED_REGISTRY_SCHEMA:
        findings.append(
            f"{label} landed marker registry schema must be {LANDED_REGISTRY_SCHEMA}"
        )
    raw_markers = registry.get("markers")
    valid_markers: list[str] = []
    if not isinstance(raw_markers, list) or not all(
        isinstance(item, str) for item in raw_markers
    ):
        findings.append(f"{label} landed marker registry markers must be a sorted path list")
    else:
        normalized = [_norm(item) for item in raw_markers]
        if raw_markers != normalized:
            findings.append(f"{label} landed marker registry paths must use canonical Git syntax")
        valid_markers = normalized
        if valid_markers != sorted(set(valid_markers)):
            findings.append(f"{label} landed marker registry markers must be sorted and unique")
        for path in valid_markers:
            if (
                not _valid_repo_path(path)
                or not path.startswith(LANDED_MARKER_PREFIX)
                or not path.endswith(LANDED_MARKER_SUFFIX)
            ):
                findings.append(f"{label} landed marker registry has invalid path: {path}")
    if discovered != valid_markers:
        findings.append(f"{label} landed marker registry does not exactly enumerate its tree")
    for path in discovered:
        if snapshot[path][0] != "100644":
            findings.append(f"{label} landed marker must be a regular 100644 blob: {path}")
    return valid_markers, findings


def _landed_marker_append_only_findings(
    root: Path,
    base_commit: str,
    head_commit: str,
) -> list[str]:
    """Reject marker deletion, replacement, or registry rollback in any PR commit."""
    commits, graph_problems = _independent_commit_range(base_commit, head_commit, root)
    if graph_problems:
        return [
            "landed review history enumeration failed closed: " + problem
            for problem in graph_problems
        ]
    commit_cache: dict[str, RawCommit] = {}
    tree_cache: dict[str, dict[str, tuple[str, str]]] = {}
    snapshot_cache: dict[str, dict[str, tuple[str, str]]] = {}
    findings: set[str] = set()

    def snapshot(commit: str) -> dict[str, tuple[str, str]] | None:
        if commit in snapshot_cache:
            return snapshot_cache[commit]
        value, problems = _tree_snapshot_from_commit(
            root,
            commit,
            commit_cache=commit_cache,
            tree_cache=tree_cache,
        )
        if problems:
            findings.update(
                "landed review history enumeration failed closed: " + problem
                for problem in problems
            )
            return None
        snapshot_cache[commit] = value
        return value

    for commit_oid, commit in sorted(commits.items()):
        for parent_oid in commit.parents:
            before = snapshot(parent_oid)
            after = snapshot(commit_oid)
            if before is None or after is None:
                continue
            before_paths, before_problems = _registry_paths_from_snapshot(
                root, before, label=f"parent {parent_oid}"
            )
            after_paths, after_problems = _registry_paths_from_snapshot(
                root, after, label=f"commit {commit_oid}"
            )
            findings.update(before_problems)
            findings.update(after_problems)
            if before_paths is not None:
                if after_paths is None:
                    findings.add(
                        "landed review history is append-only: registry was removed at "
                        + commit_oid
                    )
                else:
                    removed = sorted(set(before_paths) - set(after_paths))
                    if removed:
                        findings.add(
                            "landed review history is append-only: registry removed marker(s) at "
                            + commit_oid
                            + ": "
                            + ", ".join(removed)
                        )
            marker_paths = sorted(
                path
                for path in set(before) | set(after)
                if path.startswith(LANDED_MARKER_PREFIX)
                and path.endswith(LANDED_MARKER_SUFFIX)
            )
            for path in marker_paths:
                old = before.get(path)
                new = after.get(path)
                if old is None:
                    if new is not None and new[0] != "100644":
                        findings.add(
                            "landed review history is append-only: new marker is not 100644: "
                            + path
                        )
                elif new != old:
                    findings.add(
                        "landed review history is append-only: marker changed or was removed at "
                        + commit_oid
                        + ": "
                        + path
                    )
    return sorted(findings)


def verify_repository_landed_markers(
    root: Path = ROOT,
    *,
    ref: str = "HEAD",
    main_ref: str = "refs/remotes/origin/main",
) -> list[str]:
    """Verify every canonical landed marker registered in repository content."""
    root = root.resolve()
    commit, commit_problems = _resolve_commit(ref, "landed marker registry ref", root)
    if commit is None:
        return commit_problems
    snapshot, snapshot_problems = _tree_snapshot_from_commit(root, commit)
    if snapshot_problems:
        return snapshot_problems
    valid_markers, findings = _registry_paths_from_snapshot(
        root, snapshot, label="repository"
    )
    if valid_markers is None and not findings:
        findings.append("landed marker registry is missing")

    if findings:
        return findings
    assert valid_markers is not None
    for marker_path in valid_markers:
        marker_payload, marker_read_problems = _read_blob(root, commit, marker_path)
        findings.extend(f"{marker_path}: {problem}" for problem in marker_read_problems)
        if marker_payload is None:
            continue
        marker, marker_parse_problems = _load_canonical_record(marker_payload)
        findings.extend(f"{marker_path}: {problem}" for problem in marker_parse_problems)
        if marker is not None:
            findings.extend(
                f"{marker_path}: {problem}"
                for problem in verify_landed_review_marker(
                    marker,
                    root=root,
                    main_ref=main_ref,
                )
            )
    return findings


def _parse_porcelain_v2_z(payload: bytes) -> tuple[list[str], list[str]]:
    if payload == b"":
        return [], []
    if not payload.endswith(b"\0"):
        return [], ["git status porcelain v2 stream is not NUL terminated"]
    tokens = payload[:-1].split(b"\0")
    paths: list[str] = []
    problems: list[str] = []
    index = 0

    def decode_path(raw: bytes) -> str | None:
        try:
            path = _norm(raw.decode("utf-8", errors="strict"))
        except UnicodeDecodeError:
            problems.append("git status porcelain v2 contains a non-UTF-8 path")
            return None
        if not _valid_repo_path(path):
            problems.append("git status porcelain v2 contains an unsafe path")
            return None
        return path

    while index < len(tokens):
        token = tokens[index]
        if token.startswith((b"? ", b"! ")):
            path = decode_path(token[2:])
            if path is not None:
                paths.append(path)
            index += 1
            continue
        if token.startswith(b"1 "):
            fields = token.split(b" ", 8)
            if len(fields) != 9:
                problems.append("git status porcelain v2 contains a malformed ordinary record")
            else:
                path = decode_path(fields[8])
                if path is not None:
                    paths.append(path)
            index += 1
            continue
        if token.startswith(b"2 "):
            fields = token.split(b" ", 9)
            if len(fields) != 10 or index + 1 >= len(tokens):
                problems.append("git status porcelain v2 contains a malformed rename record")
                index += 1
                continue
            path = decode_path(fields[9])
            original = decode_path(tokens[index + 1])
            if path is not None:
                paths.append(path)
            if original is not None:
                paths.append(original)
            index += 2
            continue
        if token.startswith(b"u "):
            fields = token.split(b" ", 10)
            if len(fields) != 11:
                problems.append("git status porcelain v2 contains a malformed unmerged record")
            else:
                path = decode_path(fields[10])
                if path is not None:
                    paths.append(path)
            index += 1
            continue
        problems.append("git status porcelain v2 contains a malformed record")
        index += 1
    return paths, problems


def check_clean_worktree(root: Path = ROOT) -> list[str]:
    result = _git_bytes(root.resolve(), "status", "--porcelain=v2", "-z", "--untracked-files=all")
    if result.timed_out:
        return ["git status enumeration timed out"]
    if result.returncode != 0:
        return ["git status enumeration failed"]
    paths, problems = _parse_porcelain_v2_z(result.stdout)
    if problems:
        return problems
    if paths:
        return ["worktree is not clean; staged, unstaged, or untracked paths are present"]
    return []


ELIGIBILITY_VERDICT_SCHEMA = "garnet.trust_kernel_review_eligibility_verdict/v1"
ELIGIBLE_RECEIPT_STATE = "approval_pending_only"
ELIGIBLE_RECEIPT_CODES = ["approval-absent"]
MAX_VERDICT_BYTES = 64 * 1024
ATTEMPT_EXHAUSTED_PROBLEM = (
    "workflow run attempt 3 or later is outside the U-59 exception; "
    "no third evaluation exists and the cure is a new record successor and venue"
)
ATTEMPT_VERDICT_PROBLEM = (
    "attempt-2 eligibility verdict does not prove an approval_pending_only "
    "attempt-1 receipt for this exact run and candidate head"
)


def _load_eligibility_verdict(path: Path) -> tuple[dict[str, Any] | None, list[str]]:
    try:
        payload = path.read_bytes()
    except OSError:
        return None, [f"{ATTEMPT_VERDICT_PROBLEM}: verdict file could not be read"]
    if len(payload) > MAX_VERDICT_BYTES:
        return None, [f"{ATTEMPT_VERDICT_PROBLEM}: verdict exceeds its size bound"]
    value, load_problems = _load_canonical_record(payload)
    if value is None:
        return None, [f"{ATTEMPT_VERDICT_PROBLEM}: {'; '.join(load_problems)}"]
    return value, []


def apply_attempt_policy(
    status: TrustKernelReviewStatus,
    *,
    run_id: int | None,
    run_attempt: int | None,
    verdict_path: Path | None,
) -> TrustKernelReviewStatus:
    """Bind the U-59 attempt law without mutating the input status.

    * attempt 1 (or no attempt binding): no additional finding;
    * attempt 2 on a record-bearing candidate: the attempt-2 eligibility
      verdict must prove the approval_pending_only receipt for this run and head;
      record-less candidates carry no receipt and gain no finding (A4);
    * attempt 3 or later: fail closed for every candidate.
    """
    problems: list[str] = []
    if (run_id is None) != (run_attempt is None):
        problems.append("--run-id and --run-attempt must be supplied together")
    elif run_attempt is not None and run_id is not None:
        if type(run_id) is not int or run_id <= 0:
            problems.append("--run-id must be a positive integer")
        if type(run_attempt) is not int or run_attempt <= 0:
            problems.append("--run-attempt must be a positive integer")
        elif run_attempt >= 3:
            problems.append(ATTEMPT_EXHAUSTED_PROBLEM)
        elif run_attempt == 2 and status.review_record_path is not None:
            if verdict_path is None:
                problems.append(f"{ATTEMPT_VERDICT_PROBLEM}: --eligibility-verdict is required")
            else:
                verdict, verdict_problems = _load_eligibility_verdict(verdict_path)
                problems.extend(verdict_problems)
                if verdict is not None and not (
                    verdict.get("schema") == ELIGIBILITY_VERDICT_SCHEMA
                    and verdict.get("ok") is True
                    and verdict.get("run_id") == run_id
                    and verdict.get("run_attempt") == 2
                    and verdict.get("candidate_head") == status.head_commit
                    and verdict.get("receipt_state") == ELIGIBLE_RECEIPT_STATE
                    and verdict.get("receipt_finding_codes") == ELIGIBLE_RECEIPT_CODES
                    and verdict.get("problems") == []
                ):
                    problems.append(ATTEMPT_VERDICT_PROBLEM)
    if not problems:
        return status
    combined = [*status.problems, *problems]
    return dataclasses.replace(status, ok=False, problems=combined)


def render_markdown(status: TrustKernelReviewStatus) -> str:
    lines = [
        "# Garnet rolling trust-kernel review status",
        "",
        f"_Schema {status.schema}._",
        "",
        f"- changed paths inspected: {status.changed_count}",
        f"- discovery: **{'ok' if status.discovery_ok else 'FAILED'}** ({status.discovery_source})",
        f"- trust-kernel touched: **{status.trust_kernel_touched}**",
        f"- structured review record present: **{status.review_record_present}**",
        f"- overall: **{'ok' if status.ok else 'REVIEW REQUIRED'}**",
    ]
    for path in status.touched_paths:
        lines.append(f"  - trust-kernel: {path}")
    for problem in status.problems:
        lines.append(f"  - PROBLEM: {problem}")
    lines.append("")
    return "\n".join(lines)


def _explicit_github_transport(
    root: Path,
    *,
    repository: str | None,
    pull_request: int | None,
    read_stdin: bool,
) -> tuple[object | None, list[str]]:
    if not read_stdin:
        return None, []
    if not isinstance(repository, str) or GITHUB_REPOSITORY_RE.fullmatch(repository) is None:
        return None, ["--github-token-stdin requires --github-repo owner/name"]
    if type(pull_request) is not int or pull_request <= 0:
        return None, ["--github-token-stdin requires a positive --github-pr"]
    raw = sys.stdin.buffer.read(1026)
    if len(raw) > 1025:
        return None, ["explicit GitHub credential exceeds its input bound"]
    if raw.endswith(b"\n"):
        raw = raw[:-1]
        if raw.endswith(b"\r"):
            raw = raw[:-1]
    try:
        token = raw.decode("ascii", errors="strict")
    except UnicodeDecodeError:
        return None, ["explicit GitHub credential is malformed"]
    if not token or len(token) > 1024 or any(ord(char) < 33 or ord(char) > 126 for char in token):
        return None, ["explicit GitHub credential is malformed"]
    module_path = root / "scripts/garnet_github_governance_transport.py"
    try:
        spec = importlib.util.spec_from_file_location("_garnet_review_transport", module_path)
        if spec is None or spec.loader is None:
            raise ImportError
        module = importlib.util.module_from_spec(spec)
        sys.modules[spec.name] = module
        spec.loader.exec_module(module)
        transport = module.GitHubGovernanceTransport(repository, token)
    except Exception:
        return None, ["explicit authenticated GitHub transport could not be constructed"]
    finally:
        token = ""
        raw = b""
    return transport, []


def main(argv: list[str] | None = None, *, root: Path = ROOT) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument("--base", default=None, help="diagnostic base override (not valid for --gate)")
    parser.add_argument("--head", default="HEAD", help="diagnostic head override (not valid for --gate)")
    parser.add_argument(
        "--changed-file",
        action="append",
        default=None,
        dest="changed_files",
        help="diagnostic explicit path; cannot satisfy --gate completeness",
    )
    parser.add_argument(
        "--assume-trailer",
        action="store_true",
        help="deprecated diagnostic: trailers never satisfy rolling review v2",
    )
    parser.add_argument("--github-repo", default=None, help="explicit owner/name transport binding")
    parser.add_argument("--github-pr", type=int, default=None, help="explicit pull request number")
    parser.add_argument(
        "--github-token-stdin",
        action="store_true",
        help="read one bounded credential from stdin; environment credentials are never inherited",
    )
    parser.add_argument("--gate", action="store_true", help="exit nonzero on any finding")
    parser.add_argument(
        "--run-id",
        type=int,
        default=None,
        help="immutable workflow-run id; binds the U-59 attempt law together with --run-attempt",
    )
    parser.add_argument(
        "--run-attempt",
        type=int,
        default=None,
        help="workflow run attempt; attempt 2 requires --eligibility-verdict for a record-bearing candidate, attempt 3+ fails closed",
    )
    parser.add_argument(
        "--eligibility-verdict",
        default=None,
        help="canonical attempt-2 verdict written by garnet_trust_kernel_review_eligibility.py verify",
    )
    parser.add_argument(
        "--status-out",
        default=None,
        help="also write exactly the printed JSON status to this path (consumed by the receipt emitter)",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    policy_problems: list[str] = []
    if args.gate and args.base is not None:
        policy_problems.append("explicit base override cannot prove the authoritative merge-base")
    if args.gate and args.head != "HEAD":
        policy_problems.append("explicit head override cannot prove the current candidate head")
    if args.gate:
        policy_problems.extend(check_clean_worktree(root))
    github_transport, transport_problems = _explicit_github_transport(
        root,
        repository=args.github_repo,
        pull_request=args.github_pr,
        read_stdin=args.github_token_stdin,
    )
    policy_problems.extend(transport_problems)
    status = read_status(
        changed=args.changed_files,
        base=args.base,
        head=args.head,
        trailer=args.assume_trailer,
        root=root,
        extra_problems=policy_problems,
        github_transport=github_transport,
        repository=args.github_repo,
        pull_request=args.github_pr,
    )
    status = apply_attempt_policy(
        status,
        run_id=args.run_id,
        run_attempt=args.run_attempt,
        verdict_path=Path(args.eligibility_verdict) if args.eligibility_verdict is not None else None,
    )
    rendered = render_markdown(status) if args.format == "md" else json.dumps(asdict(status), indent=2)
    print(rendered)
    if args.status_out is not None:
        target = Path(args.status_out)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes((rendered + "\n").encode("utf-8"))
    if args.gate and not status.ok:
        print("trust-kernel review gate: REVIEW REQUIRED", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
