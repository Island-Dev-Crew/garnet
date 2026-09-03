#!/usr/bin/env python3
"""Attempt-1 eligibility receipt and attempt-2 verification for U-59 (L1 act 2).

Contract: ``C_Language_Specification/GARNET_WV_ACCEPTANCE_SUCCESSION_CONTRACT.md``
(schema ``garnet.trust_kernel_review_eligibility/v1``) and the transcribed R2
block of ``C_Language_Specification/GARNET_TRUST_KERNEL_ROLLING_REVIEW.md``.

``emit`` runs at CI attempt 1 after the rolling reporter.  It classifies the
reporter's problem list through a fixed table, binds the immutable run
constants, the candidate tree, the loaded review record's raw digest, and the
predecessor-base producer inventory digest, and writes the canonical receipt
that ``actions/upload-artifact`` carries as the sole member ``eligibility.json``
of the artifact ``r2-approval-pending-<run_id>-attempt-1``.  Only the tuple
``state=approval_pending_only`` / ``finding_codes=["approval-absent"]`` grants
one same-run re-evaluation; every other tuple is ``ineligible``.  A record-less
candidate emits nothing.

``verify`` runs at CI attempt 2 before the reporter.  It completely paginates
the run's artifacts, requires exactly one non-expired artifact of the exact
name and run, downloads the archive through the bounded one-hop transport,
parses the ZIP without extraction, requires exactly one unencrypted regular
member literally named ``eligibility.json`` whose bytes equal the recomputed
canonical receipt, requires live equality of PR, base, head, tree, record,
workflow, run, event, and producer inventory, requires ``run_attempt == 2``,
and rejects attempt 3 or later.  It writes a canonical verdict that the
reporter consumes through ``--eligibility-verdict``.

``expected_job_multiset`` and ``verify_jobs_and_census`` are the post-run
all-jobs proof and cross-workflow census callables for the act-4 readback
instrument; act 2 implements and tests them but does not wire them into CI.

The module never reads an ambient credential: the token arrives on stdin
through ``--github-token-stdin`` and is handed to the transports by the caller.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
import re
import struct
import subprocess
import sys
import zlib
from collections import Counter
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any

SCHEMA = "garnet.trust_kernel_review_eligibility/v1"
VERDICT_SCHEMA = "garnet.trust_kernel_review_eligibility_verdict/v1"
REPORTER_SCHEMA = "garnet.trust_kernel_review/v2"
PRODUCER_INVENTORY_PATH = ".github/rulesets/required-context-producers.json"
PRODUCER_INVENTORY_SCHEMA = "garnet.required-context-producers/v2"
CI_WORKFLOW_PATH = ".github/workflows/ci.yml"
MEMBER_NAME = "eligibility.json"
ARTIFACT_NAME_RE = re.compile(r"^r2-approval-pending-([1-9][0-9]{0,19})-attempt-1$")
ELIGIBLE_STATE = "approval_pending_only"
INELIGIBLE_STATE = "ineligible"
APPROVAL_ABSENT_CODE = "approval-absent"
ELIGIBLE_FINDING_CODES = (APPROVAL_ABSENT_CODE,)
APPROVAL_ABSENT_PROBLEM = (
    "authenticated decisive review from the recorded independent reviewer is absent"
)
APPROVAL_NOT_AT_HEAD_PROBLEM = (
    "latest decisive review from the recorded independent reviewer must be "
    "APPROVED at the exact current candidate head"
)
UNCLASSIFIED_CODE = "unclassified-finding"
ROOT = Path(__file__).resolve().parents[1]
GIT_TIMEOUT_SECONDS = 20
GIT_OID_RE = re.compile(r"^[0-9a-f]{40}$")
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
CODE_RE = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
REPOSITORY_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,99}/[A-Za-z0-9][A-Za-z0-9._-]{0,99}$")
BASE_REF_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._/-]{0,255}$")
RECORD_PATH_RE = re.compile(r"^F_Project_Management/W_TRUST/[A-Za-z0-9_-][A-Za-z0-9._-]{0,199}\.review\.json$")
WORKFLOW_REF_RE = re.compile(
    r"^(?P<repository>[A-Za-z0-9][A-Za-z0-9._-]{0,99}/[A-Za-z0-9][A-Za-z0-9._-]{0,99})/"
    r"(?P<path>\.github/workflows/[A-Za-z0-9_.-]{1,100}\.(?:yml|yaml))@(?P<ref>refs/[A-Za-z0-9._/-]{1,255})$"
)
TIMESTAMP_RE = re.compile(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{1,9})?(?:Z|[+-]\d{2}:\d{2})$")
MAX_JSON_BYTES = 1024 * 1024
MAX_MEMBER_BYTES = 1024 * 1024
MAX_CODE_LENGTH = 64
MAX_ID_DIGITS = 20
RECEIPT_KEYS = (
    "artifact_name",
    "base_ref",
    "base_sha",
    "candidate_head",
    "candidate_tree",
    "event",
    "finding_codes",
    "producer_inventory_sha256",
    "pull_request_id",
    "pull_request_number",
    "repository_id",
    "review_record_path",
    "review_record_sha256",
    "run_attempt",
    "run_id",
    "run_number",
    "schema",
    "state",
    "workflow_id",
    "workflow_ref",
    "workflow_sha",
)
# A2 (STRICT): only the exact reporter string maps to `approval-absent`.  Every
# other reporter finding maps to its own normalized code through this ordered
# table (exact matches first, then prefixes); anything unknown is
# `unclassified-finding`.  Any code other than the sole eligible tuple makes
# the receipt `ineligible`.
FINDING_CODE_TABLE: tuple[tuple[str, str, str], ...] = (
    ("exact", APPROVAL_ABSENT_PROBLEM, APPROVAL_ABSENT_CODE),
    ("exact", APPROVAL_NOT_AT_HEAD_PROBLEM, "approval-not-at-head"),
    ("exact", "structured review record is missing", "review-record-missing"),
    (
        "exact",
        "non-JSON or noncanonical legacy companion does not satisfy v2; "
        "structured review record is missing",
        "review-record-missing",
    ),
    ("exact", "trailer-only review is not accepted by rolling review v2", "trailer-only-review"),
    (
        "exact",
        "authenticated GitHub review transport is required for a trust-kernel change",
        "review-transport-required",
    ),
    ("exact", "authenticated review collection and direct object disagree", "review-object-disagreement"),
    ("exact", "authenticated review state must be APPROVED", "review-state-not-approved"),
    ("exact", "authenticated review must bind the exact current candidate head", "review-head-mismatch"),
    ("exact", "authenticated reviewer identity does not match the review record", "reviewer-identity-mismatch"),
    ("exact", "reviewer identity overlaps an authenticated commit principal", "reviewer-overlaps-principal"),
    ("exact", "reviewer identity overlaps an author or committer", "reviewer-overlaps-principal"),
    ("exact", "an explicit canonical GitHub repository is required", "transport-configuration"),
    ("exact", "an explicit positive GitHub pull request number is required", "transport-configuration"),
    ("exact", "local commit enumeration is partial or disagrees with commit-object traversal", "commit-enumeration-mismatch"),
    ("exact", "explicit changed paths cannot prove complete enumeration", "diagnostic-override"),
    ("exact", "explicit base override cannot prove the authoritative merge-base", "diagnostic-override"),
    ("exact", "explicit head override cannot prove the current candidate head", "diagnostic-override"),
    ("exact", "worktree is not clean; staged, unstaged, or untracked paths are present", "worktree-not-clean"),
    ("exact", "landed marker registry is missing", "landed-marker-registry"),
    ("prefix", "authenticated review enumeration failed closed", "transport-failed"),
    ("prefix", "authenticated review row is malformed", "review-enumeration-malformed"),
    ("prefix", "authenticated review enumeration has duplicate review id", "review-enumeration-malformed"),
    ("prefix", "authenticated review object read failed closed", "transport-failed"),
    ("prefix", "authenticated pull request", "pull-request-identity-mismatch"),
    ("prefix", "authenticated commit enumeration", "commit-enumeration-mismatch"),
    ("prefix", "authenticated ", "repository-identity-mismatch"),
    ("prefix", "review record authors do not match", "record-authors-mismatch"),
    ("prefix", "review record repository", "record-shape"),
    ("prefix", "review record pull request number", "record-shape"),
    ("prefix", "review record schema", "record-shape"),
    ("prefix", "review record state", "record-shape"),
    ("prefix", "review_state must be", "record-shape"),
    ("prefix", "review verdict must be", "record-shape"),
    ("prefix", "review_scope must", "record-shape"),
    ("prefix", "blocking_findings", "record-shape"),
    ("prefix", "author_emails", "record-shape"),
    ("prefix", "author_ids", "record-shape"),
    ("prefix", "reviewer identity is malformed", "record-shape"),
    ("prefix", "reviewer login is malformed", "record-shape"),
    ("prefix", "touched_paths", "record-touched-paths"),
    ("prefix", "content_digest must be", "record-shape"),
    ("prefix", "content digest mismatch", "content-digest-mismatch"),
    ("prefix", "base_commit does not match", "record-base-mismatch"),
    ("prefix", "base_commit must be", "record-shape"),
    ("prefix", "base_commit is not an ancestor", "reviewed-head-binding"),
    ("prefix", "reviewed_head", "reviewed-head-binding"),
    ("prefix", "reviewed base", "reviewed-head-binding"),
    ("prefix", "reviewed_tree", "reviewed-head-binding"),
    ("prefix", "post-review", "post-review-trust-touch"),
    ("prefix", "structured review records are append-only", "record-append-only"),
    ("prefix", "record succession", "record-succession"),
    ("prefix", "F_Project_Management/W_TRUST/", "record-succession"),
    ("prefix", "registered landed marker", "landed-marker-registry"),
    ("prefix", "git diff", "change-enumeration"),
    ("prefix", "addition identity", "change-enumeration"),
    ("prefix", "ambiguous deletion identity", "change-enumeration"),
    ("prefix", "change identity", "change-enumeration"),
    ("prefix", "merge-base", "discovery-failed"),
    ("prefix", "git status", "worktree-not-clean"),
    ("prefix", "--github-token-stdin requires", "transport-configuration"),
    ("prefix", "explicit GitHub credential", "transport-configuration"),
    ("prefix", "explicit authenticated GitHub transport", "transport-configuration"),
    ("prefix", "workflow run attempt", "attempt-exhausted"),
    ("prefix", "attempt-2 eligibility verdict", "eligibility-verdict"),
    ("prefix", "--run-id", "attempt-binding"),
    ("prefix", "--run-attempt", "attempt-binding"),
)


def canonical_bytes(value: object) -> bytes:
    """The AGENTS.md byte contract: UTF-8, sorted keys, two-space indent, one LF."""
    return (json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode("utf-8")


def _positive_int(value: object) -> bool:
    return type(value) is int and value > 0 and len(str(value)) <= MAX_ID_DIGITS


def artifact_name(run_id: int) -> str:
    if not _positive_int(run_id):
        raise ValueError("run_id must be a positive integer")
    return f"r2-approval-pending-{run_id}-attempt-1"


def classify_problems(problems: list[str] | tuple[str, ...]) -> list[str]:
    """Normalize reporter problem strings to sorted, duplicate-free codes (A2 STRICT)."""
    codes: set[str] = set()
    for problem in problems:
        if not isinstance(problem, str):
            codes.add(UNCLASSIFIED_CODE)
            continue
        matched = UNCLASSIFIED_CODE
        for kind, pattern, code in FINDING_CODE_TABLE:
            if (kind == "exact" and problem == pattern) or (kind == "prefix" and problem.startswith(pattern)):
                matched = code
                break
        codes.add(matched)
    return sorted(codes)


def is_eligible_tuple(state: object, finding_codes: object) -> bool:
    return state == ELIGIBLE_STATE and isinstance(finding_codes, list) and finding_codes == list(ELIGIBLE_FINDING_CODES)


def receipt_state(finding_codes: list[str]) -> str:
    return ELIGIBLE_STATE if list(finding_codes) == list(ELIGIBLE_FINDING_CODES) else INELIGIBLE_STATE


def _canonical_text(value: object, limit: int) -> bool:
    return (
        isinstance(value, str)
        and 0 < len(value) <= limit
        and value == value.strip()
        and value.isprintable()
    )


def valid_base_ref(value: object) -> bool:
    return (
        isinstance(value, str)
        and BASE_REF_RE.fullmatch(value) is not None
        and ".." not in value
        and "//" not in value
        and not value.endswith(("/", ".lock"))
        and "/." not in value
    )


def workflow_path_from_ref(workflow_ref: object, repository: str | None = None) -> tuple[str | None, list[str]]:
    if not isinstance(workflow_ref, str):
        return None, ["workflow_ref must be a string"]
    match = WORKFLOW_REF_RE.fullmatch(workflow_ref)
    if match is None:
        return None, ["workflow_ref is not owner/name/.github/workflows/<file>@refs/..."]
    if repository is not None and match.group("repository") != repository:
        return None, ["workflow_ref repository does not match the bound repository"]
    return match.group("path"), []


def validate_receipt(value: object) -> list[str]:
    """Exact key set, JSON types, constants, and derivations of one receipt."""
    if not isinstance(value, dict):
        return ["receipt root must be a JSON object"]
    problems: list[str] = []
    if tuple(sorted(value)) != RECEIPT_KEYS:
        problems.append("receipt must contain the exact key set of the contract")
        return problems
    if value["schema"] != SCHEMA:
        problems.append(f"schema must be {SCHEMA}")
    if value["event"] != "pull_request":
        problems.append("event must be exactly pull_request")
    if type(value["run_attempt"]) is not int or value["run_attempt"] != 1:
        problems.append("run_attempt must be exactly the integer 1")
    for key in ("pull_request_id", "pull_request_number", "repository_id", "run_id", "run_number", "workflow_id"):
        if not _positive_int(value[key]):
            problems.append(f"{key} must be a positive integer")
    for key in ("base_sha", "candidate_head", "candidate_tree", "workflow_sha"):
        if not isinstance(value[key], str) or GIT_OID_RE.fullmatch(value[key]) is None:
            problems.append(f"{key} must be a full lowercase Git object id")
    for key in ("producer_inventory_sha256", "review_record_sha256"):
        if not isinstance(value[key], str) or SHA256_RE.fullmatch(value[key]) is None:
            problems.append(f"{key} must be 64 lowercase hex digits")
    if not valid_base_ref(value["base_ref"]):
        problems.append("base_ref must be a canonical branch name")
    if not isinstance(value["review_record_path"], str) or RECORD_PATH_RE.fullmatch(value["review_record_path"]) is None:
        problems.append("review_record_path must name a canonical W_TRUST *.review.json record")
    path, ref_problems = workflow_path_from_ref(value["workflow_ref"])
    problems.extend(ref_problems)
    if path is not None and path != CI_WORKFLOW_PATH:
        problems.append(f"workflow_ref must name {CI_WORKFLOW_PATH}")
    if _positive_int(value["run_id"]):
        if value["artifact_name"] != artifact_name(value["run_id"]):
            problems.append("artifact_name must be r2-approval-pending-<run_id>-attempt-1 for this run_id")
    elif not isinstance(value["artifact_name"], str) or ARTIFACT_NAME_RE.fullmatch(value["artifact_name"]) is None:
        problems.append("artifact_name is malformed")
    state = value["state"]
    codes = value["finding_codes"]
    if state not in {ELIGIBLE_STATE, INELIGIBLE_STATE}:
        problems.append("state must be approval_pending_only or ineligible")
    if (
        not isinstance(codes, list)
        or any(not isinstance(code, str) or len(code) > MAX_CODE_LENGTH or CODE_RE.fullmatch(code) is None for code in codes)
        or codes != sorted(set(codes))
    ):
        problems.append("finding_codes must be sorted, duplicate-free normalized machine codes")
    elif state in {ELIGIBLE_STATE, INELIGIBLE_STATE} and receipt_state(codes) != state:
        problems.append("state does not agree with finding_codes; only [approval-absent] is approval_pending_only")
    return problems


class DuplicateKeyError(ValueError):
    pass


def _no_duplicate_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise DuplicateKeyError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def load_strict_json(payload: bytes, label: str) -> tuple[Any, list[str]]:
    if not isinstance(payload, bytes) or len(payload) > MAX_JSON_BYTES:
        return None, [f"{label} exceeds its size bound or is not bytes"]
    try:
        text = payload.decode("utf-8", errors="strict")
    except UnicodeDecodeError:
        return None, [f"{label} is not valid UTF-8"]
    try:
        value = json.loads(text, object_pairs_hook=_no_duplicate_object)
    except DuplicateKeyError as exc:
        return None, [f"{label}: {exc}"]
    except (json.JSONDecodeError, RecursionError, ValueError):
        return None, [f"{label} is not valid JSON"]
    return value, []


def load_receipt(payload: bytes) -> tuple[dict[str, Any] | None, list[str]]:
    """Strict canonical-byte load plus full receipt validation."""
    value, problems = load_strict_json(payload, "receipt")
    if problems:
        return None, problems
    if not isinstance(value, dict):
        return None, ["receipt root must be a JSON object"]
    if payload != canonical_bytes(value):
        return None, ["receipt bytes are not canonical (UTF-8, sorted keys, two-space indent, one trailing LF)"]
    problems = validate_receipt(value)
    return (value, []) if not problems else (None, problems)


# ── Git probes (scrubbed allowlist environment; no credential passthrough) ──


def _scrubbed_git_environment() -> dict[str, str]:
    passthrough = ("COMSPEC", "PATH", "PATHEXT", "SYSTEMROOT", "SystemRoot", "TEMP", "TMP", "TMPDIR", "WINDIR")
    env = {name: value for name in passthrough if (value := os.environ.get(name)) is not None}
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


def _git(root: Path, *args: str) -> tuple[int, bytes]:
    try:
        result = subprocess.run(
            ["git", "--no-replace-objects", *args],
            cwd=root,
            capture_output=True,
            check=False,
            timeout=GIT_TIMEOUT_SECONDS,
            env=_scrubbed_git_environment(),
        )
    except (subprocess.TimeoutExpired, OSError):
        return 124, b""
    return result.returncode, result.stdout


def _one_oid(payload: bytes) -> str | None:
    text = payload.decode("ascii", errors="replace").strip()
    return text if GIT_OID_RE.fullmatch(text) else None


def resolve_commit(root: Path, ref: str) -> tuple[str | None, list[str]]:
    code, out = _git(root, "rev-parse", "--verify", "--end-of-options", f"{ref}^{{commit}}")
    oid = _one_oid(out) if code == 0 else None
    return (oid, []) if oid else (None, [f"commit could not be resolved: {ref}"])


def resolve_tree(root: Path, commit: str) -> tuple[str | None, list[str]]:
    code, out = _git(root, "rev-parse", "--verify", "--end-of-options", f"{commit}^{{tree}}")
    oid = _one_oid(out) if code == 0 else None
    return (oid, []) if oid else (None, [f"tree could not be resolved for {commit}"])


def read_blob(root: Path, commit: str, path: str) -> tuple[bytes | None, list[str]]:
    code, out = _git(root, "rev-parse", "--verify", "--end-of-options", f"{commit}:{path}")
    oid = _one_oid(out) if code == 0 else None
    if oid is None:
        return None, [f"path is missing at {commit}: {path}"]
    code, kind = _git(root, "cat-file", "-t", oid)
    if code != 0 or kind != b"blob\n":
        return None, [f"object is not a regular blob: {path}"]
    code, blob = _git(root, "cat-file", "blob", oid)
    if code != 0:
        return None, [f"blob read failed for {path}"]
    return blob, []


def predecessor_base(root: Path, head: str) -> tuple[str | None, list[str]]:
    """The reporter's authoritative merge-base of the head and origin/main."""
    main_commit, problems = resolve_commit(root, "refs/remotes/origin/main")
    if problems:
        return None, problems
    assert main_commit is not None
    code, out = _git(root, "merge-base", "--all", head, main_commit)
    if code != 0:
        return None, ["merge-base enumeration failed"]
    bases = [line for line in out.decode("ascii", errors="replace").split() if line]
    if len(bases) != 1 or GIT_OID_RE.fullmatch(bases[0]) is None:
        return None, ["merge-base is not unique or is malformed"]
    return bases[0], []


def producer_inventory_digest(payload: bytes) -> tuple[str | None, list[str]]:
    """SHA-256 of the raw inventory bytes after asserting schema v2 strictly."""
    value, problems = load_strict_json(payload, "producer inventory")
    if problems:
        return None, problems
    if not isinstance(value, dict) or value.get("schema") != PRODUCER_INVENTORY_SCHEMA:
        return None, [f"producer inventory schema must be {PRODUCER_INVENTORY_SCHEMA}"]
    if not isinstance(value.get("producers"), list):
        return None, ["producer inventory producers must be a list"]
    return hashlib.sha256(payload).hexdigest(), []


def predecessor_inventory_digest(root: Path, head: str) -> tuple[str | None, list[str]]:
    base, problems = predecessor_base(root, head)
    if problems or base is None:
        return None, problems
    payload, blob_problems = read_blob(root, base, PRODUCER_INVENTORY_PATH)
    if blob_problems or payload is None:
        return None, blob_problems
    return producer_inventory_digest(payload)


def expected_job_multiset(inventory_bytes: bytes, workflow_path: str, event: str) -> list[str]:
    """Sorted expanded job names of the inventory rows for one workflow/event."""
    value, problems = load_strict_json(inventory_bytes, "producer inventory")
    if problems or not isinstance(value, dict) or value.get("schema") != PRODUCER_INVENTORY_SCHEMA:
        raise ValueError("producer inventory is not a strict schema v2 document")
    rows = value.get("producers")
    if not isinstance(rows, list):
        raise ValueError("producer inventory producers must be a list")
    names: list[str] = []
    for row in rows:
        if not isinstance(row, dict) or not all(isinstance(row.get(key), str) for key in ("context", "workflow", "event", "job")):
            raise ValueError("producer inventory row is malformed")
        if row["workflow"] == workflow_path and row["event"] == event:
            if "${{" in row["context"]:
                raise ValueError("producer inventory context is not expanded")
            names.append(row["context"])
    return sorted(names)


# ── ZIP parsing without extraction ──

_LOCAL_SIGNATURE = 0x04034B50
_CENTRAL_SIGNATURE = 0x02014B50
_EOCD_SIGNATURE = 0x06054B50
_DESCRIPTOR_SIGNATURE = 0x08074B50
_ALLOWED_FLAG_BITS = 0x0002 | 0x0004 | 0x0008 | 0x0800
_ZIP64_EXTRA_ID = 0x0001


def _member_name_problems(name: bytes, expected: str) -> list[str]:
    problems: list[str] = []
    try:
        text = name.decode("ascii", errors="strict")
    except UnicodeDecodeError:
        return ["member name is not ASCII"]
    if "\\" in text:
        problems.append("member name contains a backslash")
    if text.startswith("/"):
        problems.append("member name is an absolute path")
    if text.endswith("/"):
        problems.append("member is a directory entry")
    components = text.split("/")
    if any(component == "" for component in components):
        problems.append("member name has an empty component")
    if any(component in {".", ".."} for component in components):
        problems.append("member name has a . or .. component")
    if text != expected:
        problems.append(f"member name must be literally {expected}")
    return problems


def _extra_has_zip64(extra: bytes) -> bool:
    index = 0
    while index + 4 <= len(extra):
        header_id, size = struct.unpack_from("<HH", extra, index)
        if header_id == _ZIP64_EXTRA_ID:
            return True
        index += 4 + size
    return index != len(extra)


def parse_single_member_zip(archive: bytes, member_name: str = MEMBER_NAME) -> tuple[bytes | None, list[str]]:
    """Parse a ZIP with exactly one unencrypted regular member and return its bytes."""
    if not isinstance(archive, bytes) or len(archive) < 22 + 46 + 30:
        return None, ["archive is too small to be a one-member ZIP"]
    eocd = archive[-22:]
    (signature, disk, cd_disk, entries_disk, entries_total, cd_size, cd_offset, comment_length) = struct.unpack("<IHHHHIIH", eocd)
    if signature != _EOCD_SIGNATURE or comment_length != 0:
        return None, ["archive end-of-central-directory is absent, commented, or not at the end"]
    if disk != 0 or cd_disk != 0:
        return None, ["multi-disk archives are rejected"]
    if entries_disk != 1 or entries_total != 1:
        return None, [f"archive must contain exactly one entry; central directory declares {entries_total}"]
    if cd_offset + cd_size != len(archive) - 22:
        return None, ["central directory does not end at the end-of-central-directory record"]
    if cd_offset + 46 > len(archive):
        return None, ["central directory header is truncated"]
    central = struct.unpack_from("<IHHHHHHIIIHHHHHII", archive, cd_offset)
    (c_signature, made_by, _c_version, c_flags, c_method, _c_time, _c_date, c_crc, c_compressed, c_uncompressed, c_name_length, c_extra_length, c_comment_length, c_disk_start, _c_internal, c_external, local_offset) = central
    if c_signature != _CENTRAL_SIGNATURE:
        return None, ["central directory header signature is invalid"]
    if 46 + c_name_length + c_extra_length + c_comment_length != cd_size:
        return None, ["central directory size does not describe exactly one entry"]
    name_start = cd_offset + 46
    c_name = archive[name_start : name_start + c_name_length]
    c_extra = archive[name_start + c_name_length : name_start + c_name_length + c_extra_length]
    problems = _member_name_problems(c_name, member_name)
    if c_flags & 0x0001 or c_flags & 0x0040:
        problems.append("member is encrypted")
    if c_flags & ~(_ALLOWED_FLAG_BITS):
        problems.append("member uses unsupported general-purpose flags")
    if c_method not in {0, 8}:
        problems.append("member compression method must be stored or deflate")
    if c_disk_start != 0:
        problems.append("member starts on another disk")
    if _extra_has_zip64(c_extra) or 0xFFFFFFFF in {c_compressed, c_uncompressed, local_offset}:
        problems.append("ZIP64 members are rejected")
    unix_mode = (c_external >> 16) & 0xFFFF
    file_type = unix_mode & 0o170000
    if c_external & 0x10:
        problems.append("member carries the directory attribute")
    # A zero file-type field means "unspecified", which ordinary writers emit for
    # a regular file (Python's zipfile among them). Only a positively declared
    # non-regular type is rejected, so a symlink, directory, device, fifo, or
    # socket entry stays RED while an interoperable regular file is accepted.
    if file_type and file_type != 0o100000:
        problems.append("member is not a regular file (symlink, directory, device, fifo, or socket)")
    if c_uncompressed > MAX_MEMBER_BYTES or c_compressed > MAX_MEMBER_BYTES:
        problems.append("member exceeds the receipt size bound")
    if local_offset != 0:
        problems.append("the sole member must begin at archive offset 0")
    if problems:
        return None, problems
    if 30 > len(archive):
        return None, ["local header is truncated"]
    local = struct.unpack_from("<IHHHHHIIIHH", archive, 0)
    (l_signature, _l_version, l_flags, l_method, _l_time, _l_date, l_crc, l_compressed, l_uncompressed, l_name_length, l_extra_length) = local
    if l_signature != _LOCAL_SIGNATURE:
        return None, ["local header signature is invalid"]
    l_name = archive[30 : 30 + l_name_length]
    l_extra = archive[30 + l_name_length : 30 + l_name_length + l_extra_length]
    if l_name != c_name or l_flags != c_flags or l_method != c_method:
        return None, ["local header disagrees with the central directory (name, flags, or method)"]
    if _extra_has_zip64(l_extra):
        return None, ["ZIP64 members are rejected"]
    data_start = 30 + l_name_length + l_extra_length
    data_end = data_start + c_compressed
    if data_end > cd_offset:
        return None, ["member data overruns the central directory"]
    if c_flags & 0x0008:
        if (l_crc, l_compressed, l_uncompressed) != (0, 0, 0):
            return None, ["local header must be zero when a data descriptor is declared"]
        descriptor = archive[data_end:cd_offset]
        if len(descriptor) == 16 and struct.unpack_from("<I", descriptor)[0] == _DESCRIPTOR_SIGNATURE:
            descriptor = descriptor[4:]
        if len(descriptor) != 12 or struct.unpack("<III", descriptor) != (c_crc, c_compressed, c_uncompressed):
            return None, ["data descriptor disagrees with the central directory"]
    else:
        if (l_crc, l_compressed, l_uncompressed) != (c_crc, c_compressed, c_uncompressed):
            return None, ["local header disagrees with the central directory (crc or sizes)"]
        if data_end != cd_offset:
            return None, ["unexpected bytes between member data and the central directory"]
    data = archive[data_start:data_end]
    if c_method == 0:
        member = data
    else:
        inflater = zlib.decompressobj(-15)
        try:
            member = inflater.decompress(data, MAX_MEMBER_BYTES + 1)
        except zlib.error:
            return None, ["deflate stream is corrupt"]
        if not inflater.eof or inflater.unconsumed_tail or inflater.unused_data:
            return None, ["deflate stream did not end exactly with the member data"]
    if len(member) != c_uncompressed or (zlib.crc32(member) & 0xFFFFFFFF) != c_crc:
        return None, ["member size or CRC-32 disagrees with the central directory"]
    if not member:
        return None, ["member is empty"]
    return member, []


# ── Attempt-2 verification ──


@dataclass(frozen=True)
class RunConstants:
    repository: str
    repository_id: int
    pull_request_id: int
    pull_request_number: int
    base_ref: str
    base_sha: str
    head_sha: str
    run_id: int
    run_number: int
    run_attempt: int
    workflow_ref: str
    workflow_sha: str

    def problems(self) -> list[str]:
        problems: list[str] = []
        if not isinstance(self.repository, str) or REPOSITORY_RE.fullmatch(self.repository) is None:
            problems.append("repository must be owner/name")
        for key in ("repository_id", "pull_request_id", "pull_request_number", "run_id", "run_number", "run_attempt"):
            if not _positive_int(getattr(self, key)):
                problems.append(f"{key} must be a positive integer")
        for key in ("base_sha", "head_sha", "workflow_sha"):
            value = getattr(self, key)
            if not isinstance(value, str) or GIT_OID_RE.fullmatch(value) is None:
                problems.append(f"{key} must be a full lowercase Git object id")
        if not valid_base_ref(self.base_ref):
            problems.append("base_ref must be a canonical branch name")
        path, ref_problems = workflow_path_from_ref(self.workflow_ref, self.repository if isinstance(self.repository, str) else None)
        problems.extend(ref_problems)
        if path is not None and path != CI_WORKFLOW_PATH:
            problems.append(f"workflow_ref must name {CI_WORKFLOW_PATH}")
        return problems


@dataclass(frozen=True)
class Verdict:
    schema: str
    ok: bool
    problems: tuple[str, ...]
    run_id: int | None
    run_attempt: int | None
    candidate_head: str | None
    candidate_tree: str | None = None
    workflow_id: int | None = None
    artifact_id: int | None = None
    artifact_name: str | None = None
    artifact_created_at: str | None = None
    archive_endpoint: str | None = None
    archive_status: int | None = None
    archive_final_host: str | None = None
    archive_sha256: str | None = None
    raw_body_sha256: str | None = None
    api_digest: str | None = None
    receipt_sha256: str | None = None
    receipt_state: str | None = None
    receipt_finding_codes: list[str] = field(default_factory=list)


def render_verdict(verdict: Verdict) -> str:
    return canonical_bytes(asdict(verdict)).decode("utf-8")


def _transport_codes(result: object) -> list[str]:
    raw = getattr(result, "problems", None)
    if not isinstance(raw, (tuple, list)):
        return ["response-shape"]
    codes: list[str] = []
    for item in raw:
        code = getattr(item, "code", None)
        codes.append(code if isinstance(code, str) and code else "response-shape")
    return sorted(set(codes))


def _get_object(transport: object, path: str, label: str) -> tuple[dict[str, Any] | None, list[str]]:
    try:
        result = transport.get_object(path)
    except Exception:
        return None, [f"authenticated {label} read failed closed: transport-failure"]
    codes = _transport_codes(result)
    if codes:
        return None, [f"authenticated {label} read failed closed: {', '.join(codes)}"]
    value = getattr(result, "value", None)
    if not isinstance(value, dict):
        return None, [f"authenticated {label} read failed closed: response-shape"]
    return value, []


def _get_collection(transport: object, path: str, label: str, *, root_key: str) -> tuple[tuple[object, ...], list[str]]:
    try:
        result = transport.get_collection(path, root_key=root_key, require_total_count=True)
    except Exception:
        return (), [f"authenticated {label} enumeration failed closed: transport-failure"]
    codes = _transport_codes(result)
    if codes:
        return (), [f"authenticated {label} enumeration failed closed: {', '.join(codes)}"]
    rows = getattr(result, "rows", None)
    if not isinstance(rows, tuple):
        return (), [f"authenticated {label} enumeration failed closed: response-shape"]
    return rows, []


def _run_object_problems(run: dict[str, Any], constants: RunConstants, *, expected_attempt: int, workflow_path: str) -> list[str]:
    problems: list[str] = []
    if run.get("id") != constants.run_id:
        problems.append("live workflow run id does not match")
    if type(run.get("run_attempt")) is not int or run.get("run_attempt") != expected_attempt:
        problems.append(f"live workflow run attempt must be exactly {expected_attempt}")
    if run.get("event") != "pull_request":
        problems.append("live workflow run event must be pull_request")
    if run.get("head_sha") != constants.head_sha:
        problems.append("live workflow run head does not match the candidate head")
    if run.get("run_number") != constants.run_number:
        problems.append("live workflow run number does not match")
    if run.get("path") != workflow_path:
        problems.append(f"live workflow run path must be {workflow_path}")
    if not _positive_int(run.get("workflow_id")):
        problems.append("live workflow run workflow_id is malformed")
    repository = run.get("repository")
    if not isinstance(repository, dict) or repository.get("id") != constants.repository_id or repository.get("full_name") != constants.repository:
        problems.append("live workflow run repository does not match")
    return problems


def _pull_request_problems(pull: dict[str, Any], constants: RunConstants) -> list[str]:
    problems: list[str] = []
    if pull.get("id") != constants.pull_request_id:
        problems.append("live pull request id does not match")
    if pull.get("number") != constants.pull_request_number:
        problems.append("live pull request number does not match")
    if pull.get("state") != "open":
        problems.append("live pull request must be open")
    if pull.get("draft") is not False:
        problems.append("live pull request must not be a draft")
    head = pull.get("head")
    if not isinstance(head, dict) or head.get("sha") != constants.head_sha:
        problems.append("live pull request head does not match the candidate head")
    base = pull.get("base")
    base_repo = base.get("repo") if isinstance(base, dict) else None
    if (
        not isinstance(base, dict)
        or base.get("ref") != constants.base_ref
        or base.get("sha") != constants.base_sha
        or not isinstance(base_repo, dict)
        or base_repo.get("id") != constants.repository_id
        or base_repo.get("full_name") != constants.repository
    ):
        problems.append("live pull request base ref/sha/repository does not equal attempt 1")
    return problems


def _select_artifact(rows: tuple[object, ...], constants: RunConstants) -> tuple[dict[str, Any] | None, list[str]]:
    expected_name = artifact_name(constants.run_id)
    named = [row for row in rows if isinstance(row, dict) and row.get("name") == expected_name]
    if len(named) != 1:
        return None, [f"exactly one artifact named {expected_name} is required; found {len(named)}"]
    artifact = named[0]
    problems: list[str] = []
    if not _positive_int(artifact.get("id")):
        problems.append("artifact id must be a positive integer")
    if artifact.get("expired") is not False:
        problems.append("artifact must be non-expired")
    run = artifact.get("workflow_run")
    if not isinstance(run, dict) or run.get("id") != constants.run_id:
        problems.append("artifact workflow_run id does not match this run")
    elif run.get("repository_id") != constants.repository_id:
        problems.append("artifact workflow_run repository does not match")
    digest = artifact.get("digest")
    if digest is not None and (not isinstance(digest, str) or not digest.startswith("sha256:") or SHA256_RE.fullmatch(digest[7:]) is None):
        problems.append("artifact digest field is malformed")
    created = artifact.get("created_at")
    if created is not None and not _canonical_text(created, 64):
        problems.append("artifact created_at is malformed")
    return (artifact, []) if not problems else (None, problems)


def verify_attempt2(*, transport: object, archive_transport: object, root: Path, constants: RunConstants) -> Verdict:
    """The in-run attempt-2 half: receipt enumeration, download, ZIP/receipt equality, run constants."""
    problems = constants.problems()
    run_id = constants.run_id if _positive_int(constants.run_id) else None
    attempt = constants.run_attempt if type(constants.run_attempt) is int else None
    base = Verdict(VERDICT_SCHEMA, False, (), run_id, attempt, constants.head_sha if isinstance(constants.head_sha, str) else None)
    if problems:
        return _finish(base, problems)
    if constants.run_attempt >= 3:
        return _finish(base, [f"workflow run attempt {constants.run_attempt} is outside the U-59 exception; no third attempt exists"])
    if constants.run_attempt != 2:
        return _finish(base, ["attempt-2 verification requires run_attempt exactly 2"])
    workflow_path, _ = workflow_path_from_ref(constants.workflow_ref, constants.repository)
    assert workflow_path is not None
    root = root.resolve()
    head, head_problems = resolve_commit(root, constants.head_sha)
    problems.extend(head_problems)
    if head != constants.head_sha:
        problems.append("candidate head is not resolvable in the checkout")
        return _finish(base, problems)
    tree, tree_problems = resolve_tree(root, head)
    problems.extend(tree_problems)
    inventory_sha256, inventory_problems = predecessor_inventory_digest(root, head)
    problems.extend(inventory_problems)
    run, run_problems = _get_object(transport, f"actions/runs/{constants.run_id}", "workflow run")
    problems.extend(run_problems)
    workflow_id: int | None = None
    if run is not None:
        problems.extend(_run_object_problems(run, constants, expected_attempt=2, workflow_path=workflow_path))
        workflow_id = run.get("workflow_id") if _positive_int(run.get("workflow_id")) else None
    pull, pull_problems = _get_object(transport, f"pulls/{constants.pull_request_number}", "pull request")
    problems.extend(pull_problems)
    if pull is not None:
        problems.extend(_pull_request_problems(pull, constants))
    rows, artifact_problems = _get_collection(transport, f"actions/runs/{constants.run_id}/artifacts", "artifact", root_key="artifacts")
    problems.extend(artifact_problems)
    artifact, selection_problems = _select_artifact(rows, constants) if not artifact_problems else (None, [])
    problems.extend(selection_problems)
    state = Verdict(
        VERDICT_SCHEMA, False, (), constants.run_id, constants.run_attempt, head, tree, workflow_id,
        artifact.get("id") if artifact else None,
        artifact_name(constants.run_id),
        artifact.get("created_at") if artifact else None,
        api_digest=artifact.get("digest") if artifact else None,
    )
    if artifact is None:
        return _finish(state, problems)
    download = archive_transport.download_archive(artifact["id"])
    download_problems = tuple(getattr(download, "problems", ("response-shape",)))
    status = getattr(download, "status", None)
    raw = getattr(download, "raw_bytes", b"")
    sha256 = getattr(download, "sha256", None)
    endpoint = getattr(download, "endpoint", None)
    host = getattr(download, "final_url_host", None)
    state = _replace(state, archive_endpoint=endpoint, archive_status=status, archive_final_host=host)
    if download_problems:
        problems.append(f"artifact archive download failed closed: {', '.join(str(item) for item in download_problems)}")
        return _finish(state, problems)
    if status != 200 or not isinstance(raw, bytes) or not raw or not isinstance(sha256, str) or sha256 != hashlib.sha256(raw).hexdigest():
        problems.append("artifact archive transport did not bind a 200 body with its SHA-256")
        return _finish(state, problems)
    if endpoint != f"actions/artifacts/{artifact['id']}/zip":
        problems.append("artifact archive endpoint does not bind the selected artifact id")
    state = _replace(state, raw_body_sha256=sha256, archive_sha256=hashlib.sha256(raw).hexdigest())
    if state.api_digest is not None and state.api_digest != f"sha256:{sha256}":
        problems.append("artifact API digest disagrees with the downloaded archive SHA-256")
    member, zip_problems = parse_single_member_zip(raw)
    problems.extend(zip_problems)
    if member is None:
        return _finish(state, problems)
    state = _replace(state, receipt_sha256=hashlib.sha256(member).hexdigest())
    receipt, receipt_problems = load_receipt(member)
    problems.extend(receipt_problems)
    if receipt is None:
        return _finish(state, problems)
    state = _replace(state, receipt_state=receipt["state"], receipt_finding_codes=list(receipt["finding_codes"]))
    record_path = receipt["review_record_path"]
    record, record_problems = read_blob(root, head, record_path)
    problems.extend(record_problems)
    expected = {
        "artifact_name": artifact_name(constants.run_id),
        "base_ref": constants.base_ref,
        "base_sha": constants.base_sha,
        "candidate_head": head,
        "candidate_tree": tree,
        "event": "pull_request",
        "finding_codes": list(receipt["finding_codes"]),
        "producer_inventory_sha256": inventory_sha256,
        "pull_request_id": constants.pull_request_id,
        "pull_request_number": constants.pull_request_number,
        "repository_id": constants.repository_id,
        "review_record_path": record_path,
        "review_record_sha256": hashlib.sha256(record).hexdigest() if record is not None else None,
        "run_attempt": 1,
        "run_id": constants.run_id,
        "run_number": constants.run_number,
        "schema": SCHEMA,
        "state": receipt["state"],
        "workflow_id": workflow_id,
        "workflow_ref": constants.workflow_ref,
        "workflow_sha": constants.workflow_sha,
    }
    for key in RECEIPT_KEYS:
        if receipt[key] != expected[key]:
            problems.append(f"receipt {key} does not equal the live attempt-2 value")
    if canonical_bytes(expected) != member:
        problems.append("member bytes do not equal the recomputed canonical receipt")
    if not is_eligible_tuple(receipt["state"], receipt["finding_codes"]):
        problems.append(
            f"attempt-1 receipt is ineligible: state={receipt['state']} finding_codes={receipt['finding_codes']}"
        )
    return _finish(state, problems)


def _replace(verdict: Verdict, **changes: object) -> Verdict:
    values = asdict(verdict)
    values.update(changes)
    return Verdict(**values)


def _finish(verdict: Verdict, problems: list[str]) -> Verdict:
    unique = tuple(sorted(dict.fromkeys(problems)))
    return _replace(verdict, ok=not unique, problems=unique)


# ── Act-4 callables: all-jobs proof and cross-workflow census ──

_JOB_KEYS = ("id", "run_id", "run_attempt", "head_sha", "name", "status", "conclusion", "started_at", "completed_at")


def _job_rows(rows: object, *, attempt: int, run_id: int, head_sha: str, label: str) -> tuple[list[dict[str, Any]], set[int], list[str]]:
    problems: list[str] = []
    parsed: list[dict[str, Any]] = []
    ids: set[int] = set()
    if not isinstance(rows, (list, tuple)):
        return [], set(), [f"{label} jobs must be a list"]
    for row in rows:
        if not isinstance(row, dict) or not all(key in row for key in _JOB_KEYS):
            problems.append(f"{label} job row is missing required fields")
            continue
        if not _positive_int(row["id"]) or row["id"] in ids:
            problems.append(f"{label} job id is malformed or duplicated")
            continue
        ids.add(row["id"])
        if row["run_id"] != run_id or row["run_attempt"] != attempt or row["head_sha"] != head_sha:
            problems.append(f"{label} job {row['id']} does not bind this run, attempt, and head")
        if not _canonical_text(row["name"], 256):
            problems.append(f"{label} job {row['id']} name is malformed")
        parsed.append(row)
    return parsed, ids, problems


def verify_jobs_and_census(
    *,
    inventory_bytes: bytes,
    workflow_path: str,
    event: str,
    run_id: int,
    head_sha: str,
    workflow_id: int,
    attempt1_jobs: object,
    attempt2_jobs: object,
    head_runs: object,
    producer_workflow_ids: dict[str, int],
) -> list[str]:
    """Post-run all-jobs proof plus the head-scoped cross-workflow census (act 4 invokes)."""
    problems: list[str] = []
    try:
        expected = Counter(expected_job_multiset(inventory_bytes, workflow_path, event))
    except ValueError as exc:
        return [f"expected job multiset could not be derived: {exc}"]
    if not expected:
        return [f"predecessor inventory declares no {event} producers for {workflow_path}"]
    first, first_ids, first_problems = _job_rows(attempt1_jobs, attempt=1, run_id=run_id, head_sha=head_sha, label="attempt-1")
    second, second_ids, second_problems = _job_rows(attempt2_jobs, attempt=2, run_id=run_id, head_sha=head_sha, label="attempt-2")
    problems.extend(first_problems)
    problems.extend(second_problems)
    actual = Counter(row["name"] for row in second)
    if actual != expected:
        missing = sorted((expected - actual).elements())
        extra = sorted((actual - expected).elements())
        problems.append(
            "attempt-2 job-name multiset does not equal the expanded predecessor inventory"
            f" (missing={missing}, extra={extra})"
        )
    for row in second:
        if row["status"] != "completed" or row["conclusion"] != "success":
            problems.append(f"attempt-2 job {row['id']} ({row['name']}) did not conclude completed/success")
        if not _canonical_text(row["started_at"], 64) or not _canonical_text(row["completed_at"], 64):
            problems.append(f"attempt-2 job {row['id']} lacks start/completion timestamps")
    reused = sorted(first_ids & second_ids)
    if reused:
        problems.append(f"attempt-2 reused attempt-1 job identities: {reused}")
    if len(first) != len(expected):
        problems.append("attempt-1 job census does not cover the expected multiset")
    if producer_workflow_ids.get(workflow_path) != workflow_id:
        problems.append("workflow id mapping does not bind the re-evaluated workflow path")
    if not isinstance(head_runs, (list, tuple)):
        problems.append("head-scoped run census must be a list")
        return sorted(dict.fromkeys(problems))
    by_workflow: dict[int, list[dict[str, Any]]] = {}
    for row in head_runs:
        if (
            not isinstance(row, dict)
            or not _positive_int(row.get("id"))
            or not _positive_int(row.get("workflow_id"))
            or not _positive_int(row.get("run_attempt"))
        ):
            problems.append("head-scoped run census row is malformed")
            continue
        if row.get("head_sha") != head_sha:
            problems.append(f"run {row['id']} in the head-scoped census is outside the candidate head")
            continue
        by_workflow.setdefault(row["workflow_id"], []).append(row)
    ci_runs = by_workflow.get(workflow_id, [])
    if len(ci_runs) != 1 or ci_runs[0]["id"] != run_id:
        problems.append("the re-evaluated workflow must have exactly this one run at the candidate head")
    elif ci_runs[0]["run_attempt"] != 2 or ci_runs[0].get("event") != event:
        problems.append("the re-evaluated workflow run must be at attempt 2 for the pull_request event")
    try:
        inventory, _ = load_strict_json(inventory_bytes, "producer inventory")
        other_paths = sorted({row["workflow"] for row in inventory["producers"] if row["event"] == event and row["workflow"] != workflow_path})
    except (TypeError, KeyError):
        other_paths = []
    for path in other_paths:
        other_id = producer_workflow_ids.get(path)
        if not _positive_int(other_id):
            problems.append(f"producer workflow {path} has no live workflow id in the census")
            continue
        runs = by_workflow.get(other_id, [])
        if len(runs) != 1:
            problems.append(
                f"producer workflow {path} must have exactly one run at the candidate head; found {len(runs)}"
            )
        for row in runs:
            if row["run_attempt"] != 1 or row.get("event") != event:
                problems.append(f"producer workflow {path} run {row['id']} is not at attempt 1 for {event}")
    return sorted(dict.fromkeys(problems))


# ── Emit ──


def _load_reporter_status(path: Path) -> tuple[dict[str, Any] | None, list[str]]:
    try:
        payload = path.read_bytes()
    except OSError:
        return None, ["reporter status could not be read"]
    value, problems = load_strict_json(payload, "reporter status")
    if problems:
        return None, problems
    if not isinstance(value, dict) or value.get("schema") != REPORTER_SCHEMA:
        return None, [f"reporter status schema must be {REPORTER_SCHEMA}"]
    problems_field = value.get("problems")
    if not isinstance(problems_field, list) or any(not isinstance(item, str) for item in problems_field):
        return None, ["reporter status problems must be a list of strings"]
    return value, []


def emit_receipt(*, transport: object, root: Path, status_path: Path, output: Path, constants: RunConstants) -> tuple[int, str]:
    """Return (exit code, one-line message); writes the receipt only on success."""
    problems = constants.problems()
    if problems:
        return 1, "; ".join(problems)
    if constants.run_attempt != 1:
        return 1, f"receipt emission is attempt-1 only; attempt {constants.run_attempt} observed"
    if not status_path.is_file():
        return 0, "no receipt: reporter status absent (an earlier step failed before the rolling review ran)"
    status, status_problems = _load_reporter_status(status_path)
    if status is None:
        return 1, "; ".join(status_problems)
    if status.get("review_record_path") is None:
        return 0, "no receipt: record-less candidate (the rolling review loaded no premerge record)"
    root = root.resolve()
    head, head_problems = resolve_commit(root, constants.head_sha)
    problems.extend(head_problems)
    if head is None or status.get("head_commit") != head:
        problems.append("reporter status head_commit does not equal the candidate head")
        return 1, "; ".join(problems)
    record_path = status["review_record_path"]
    if not isinstance(record_path, str) or RECORD_PATH_RE.fullmatch(record_path) is None:
        return 1, "reporter status review_record_path is not a canonical W_TRUST record"
    record, record_problems = read_blob(root, head, record_path)
    problems.extend(record_problems)
    if record is None:
        return 1, "; ".join(problems)
    record_sha256 = hashlib.sha256(record).hexdigest()
    if status.get("review_record_sha256") != record_sha256:
        return 1, "reporter status review_record_sha256 does not equal the loaded record's raw digest"
    merge_base, base_problems = predecessor_base(root, head)
    problems.extend(base_problems)
    if merge_base is None or status.get("base_commit") != merge_base:
        problems.append("reporter status base_commit does not equal the authoritative merge-base")
        return 1, "; ".join(problems)
    tree, tree_problems = resolve_tree(root, head)
    problems.extend(tree_problems)
    inventory_sha256, inventory_problems = predecessor_inventory_digest(root, head)
    problems.extend(inventory_problems)
    workflow_path, _ = workflow_path_from_ref(constants.workflow_ref, constants.repository)
    run, run_problems = _get_object(transport, f"actions/runs/{constants.run_id}", "workflow run")
    problems.extend(run_problems)
    if run is not None:
        problems.extend(_run_object_problems(run, constants, expected_attempt=1, workflow_path=workflow_path or ""))
    if problems:
        return 1, "; ".join(problems)
    assert run is not None
    codes = classify_problems(status["problems"])
    receipt = {
        "artifact_name": artifact_name(constants.run_id),
        "base_ref": constants.base_ref,
        "base_sha": constants.base_sha,
        "candidate_head": head,
        "candidate_tree": tree,
        "event": "pull_request",
        "finding_codes": codes,
        "producer_inventory_sha256": inventory_sha256,
        "pull_request_id": constants.pull_request_id,
        "pull_request_number": constants.pull_request_number,
        "repository_id": constants.repository_id,
        "review_record_path": record_path,
        "review_record_sha256": record_sha256,
        "run_attempt": 1,
        "run_id": constants.run_id,
        "run_number": constants.run_number,
        "schema": SCHEMA,
        "state": receipt_state(codes),
        "workflow_id": run["workflow_id"],
        "workflow_ref": constants.workflow_ref,
        "workflow_sha": constants.workflow_sha,
    }
    receipt_problems = validate_receipt(receipt)
    if receipt_problems:
        return 1, "; ".join(receipt_problems)
    payload = canonical_bytes(receipt)
    try:
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_bytes(payload)
    except OSError:
        return 1, "receipt could not be written"
    summary = {
        "artifact_name": receipt["artifact_name"],
        "finding_codes": codes,
        "receipt_sha256": hashlib.sha256(payload).hexdigest(),
        "state": receipt["state"],
    }
    return 0, json.dumps(summary, sort_keys=True)


# ── CLI ──


def _read_token(stdin: object) -> tuple[str | None, list[str]]:
    buffer = getattr(stdin, "buffer", stdin)
    raw = buffer.read(1026)
    if not isinstance(raw, bytes) or len(raw) > 1025:
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
    return token, []


def _load_sibling(name: str) -> object:
    path = Path(__file__).with_name(f"{name}.py")
    spec = importlib.util.spec_from_file_location(f"_garnet_eligibility_{name}", path)
    if spec is None or spec.loader is None:
        raise ImportError(name)
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def _default_json_transport(repository: str, token: str) -> object:
    return _load_sibling("garnet_github_governance_transport").GitHubGovernanceTransport(repository, token)


def _default_archive_transport(repository: str, token: str) -> object:
    return _load_sibling("garnet_actions_artifact_transport").ActionsArtifactTransport(repository, token)


def _add_constant_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--root", default=str(ROOT), help="repository checkout root")
    parser.add_argument("--github-repo", required=True, help="explicit owner/name transport binding")
    parser.add_argument("--github-token-stdin", action="store_true", help="read one bounded credential from stdin; no environment credential is ever read")
    parser.add_argument("--repository-id", type=int, required=True)
    parser.add_argument("--pull-request-id", type=int, required=True)
    parser.add_argument("--pull-request-number", type=int, required=True)
    parser.add_argument("--base-ref", required=True)
    parser.add_argument("--base-sha", required=True)
    parser.add_argument("--head-sha", required=True)
    parser.add_argument("--run-id", type=int, required=True)
    parser.add_argument("--run-number", type=int, required=True)
    parser.add_argument("--run-attempt", type=int, required=True)
    parser.add_argument("--workflow-ref", required=True)
    parser.add_argument("--workflow-sha", required=True)


def _constants(args: argparse.Namespace) -> RunConstants:
    return RunConstants(
        repository=args.github_repo,
        repository_id=args.repository_id,
        pull_request_id=args.pull_request_id,
        pull_request_number=args.pull_request_number,
        base_ref=args.base_ref,
        base_sha=args.base_sha,
        head_sha=args.head_sha,
        run_id=args.run_id,
        run_number=args.run_number,
        run_attempt=args.run_attempt,
        workflow_ref=args.workflow_ref,
        workflow_sha=args.workflow_sha,
    )


def main(
    argv: list[str] | None = None,
    *,
    stdin: object | None = None,
    transport_factory: object | None = None,
    archive_transport_factory: object | None = None,
) -> int:
    parser = argparse.ArgumentParser(description="U-59 attempt-1 eligibility receipt and attempt-2 verification")
    commands = parser.add_subparsers(dest="command", required=True)
    emit_parser = commands.add_parser("emit", help="write the canonical attempt-1 receipt")
    _add_constant_arguments(emit_parser)
    emit_parser.add_argument("--status", required=True, help="reporter JSON written by --status-out")
    emit_parser.add_argument("--output", required=True, help="receipt path (uploaded as eligibility.json)")
    verify_parser = commands.add_parser("verify", help="verify the attempt-1 receipt at attempt 2")
    _add_constant_arguments(verify_parser)
    verify_parser.add_argument("--verdict-out", required=True, help="canonical verdict path consumed by the reporter")
    verify_parser.add_argument("--gate", action="store_true", help="exit nonzero when the verdict is not ok")
    args = parser.parse_args(list(argv) if argv is not None else None)

    if not args.github_token_stdin:
        print("--github-token-stdin is required; no environment credential is ever read", file=sys.stderr)
        return 2
    token, token_problems = _read_token(sys.stdin if stdin is None else stdin)
    if token is None:
        print("; ".join(token_problems), file=sys.stderr)
        return 2
    if REPOSITORY_RE.fullmatch(args.github_repo) is None:
        print("--github-repo must be owner/name", file=sys.stderr)
        return 2
    json_factory = _default_json_transport if transport_factory is None else transport_factory
    archive_factory = _default_archive_transport if archive_transport_factory is None else archive_transport_factory
    try:
        transport = json_factory(args.github_repo, token)
        archive_transport = archive_factory(args.github_repo, token) if args.command == "verify" else None
    except Exception:
        print("authenticated transport could not be constructed", file=sys.stderr)
        return 2
    finally:
        token = ""
    constants = _constants(args)
    root = Path(args.root)
    if args.command == "emit":
        code, message = emit_receipt(
            transport=transport, root=root, status_path=Path(args.status), output=Path(args.output), constants=constants
        )
        print(message)
        return code
    verdict = verify_attempt2(transport=transport, archive_transport=archive_transport, root=root, constants=constants)
    rendered = render_verdict(verdict)
    out = Path(args.verdict_out)
    try:
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_bytes(rendered.encode("utf-8"))
    except OSError:
        print("verdict could not be written", file=sys.stderr)
        return 2
    print(rendered, end="")
    if args.gate and not verdict.ok:
        print("attempt-2 eligibility verification: RED", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
