#!/usr/bin/env python3
"""GOV-009 exact-head/outcome evaluator and explicit-stdin live collector.

The evaluator remains injectable for adversarial tests.  The runnable collector
accepts exactly one bounded token from stdin, rejects ambient GitHub credential
variables, and renders only sanitized verdict data.  Runtime mode proves the
fresh/exact-head/outcome clauses without admin access; admin mode additionally
proves live settings equality and an empty bypass list.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
import re
import stat
import subprocess
import sys
import unicodedata
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path


def _load_sibling(name: str) -> object:
    path = Path(__file__).with_name(f"{name}.py")
    spec = importlib.util.spec_from_file_location(f"_{name}_gate", path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


transport = _load_sibling("garnet_github_governance_transport")
identity = _load_sibling("garnet_workflow_identity_policy")
required_context_contract = _load_sibling("garnet_required_context_contract")

SCHEMA = "garnet.github-governance-gate/v1"
EXPECTED_REPOSITORY = "Island-Dev-Crew/garnet"
EXPECTED_DEFAULT_BRANCH = "main"
EXPECTED_RULESET_ID = 18_936_562
EXPECTED_RULESET_NAME = "Garnet main - human-gated trust kernel"
EXPECTED_RULESET_TARGET = "branch"
EXPECTED_RULESET_ENFORCEMENT = "active"
EXPECTED_POLICY_IDENTITY_SHA256 = (
    "899944d4f0344e4b53cdd3cb37b1da26061f5eaab5d49d8482f8157b1ed51aaa"
)
EXPECTED_POLICY_SEMANTIC_SHA256 = (
    "5b5c36f13fa28ea4841aa771319633031fb1a2af4c329ffe46fed20766d55bba"
)
EXPECTED_POLICY_BINDING_SHA256 = (
    "8281eef9365a8d7469e3664ed91f37483451cdd18450512b3d963f217db88277"
)
EXPECTED_ACTIVATED_POLICY_IDENTITY_SHA256 = (
    "505abd5474941cf5f0aa460d4474418ba93cb21b3e0faed809c1e31157e866de"
)
EXPECTED_ACTIVATED_POLICY_SEMANTIC_SHA256 = (
    "ddf0076fec55e3f8dca5981cfcadc6202ad7a0470c8b8e8b2e3e0f889d431386"
)
EXPECTED_ACTIVATED_POLICY_BINDING_SHA256 = (
    "f56782c67cac3c7a84b7a346e80d925c68c39c24fe59c3715e51d06d0b69df65"
)
EXPECTED_RULESET_DOCUMENT_SHA256 = (
    "46366962f5b11a1c150a7e76e5f2fd7d4bbfa6d1ba63d445f0f04184a6d74c6f"
)
EXPECTED_ACTIVATED_RULESET_DOCUMENT_SHA256 = (
    "41974ee5a1d82e1d30044ff05117dfadbbcd1cb03a23c2a7c4d22d3f26458ba7"
)
EXPECTED_REPOSITORY_SETTINGS_DOCUMENT_SHA256 = (
    "c4f0dd0025fb9e3edbd8a12e320da49151353e09a06038737955a4a268378e3a"
)
CHECKED_RULESET_PATH = ".github/rulesets/garnet-main.json"
CHECKED_REPOSITORY_SETTINGS_PATH = ".github/rulesets/repository-settings.json"
MAX_CHECKED_DOCUMENT_BYTES = 512 * 1024
FILE_ATTRIBUTE_REPARSE_POINT = 0x400
FRESHNESS_WINDOW_SECONDS = 2 * 60 * 60
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
UTC_RE = re.compile(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$")
RULESET_KEYS = {"name", "target", "enforcement", "bypass_actors", "conditions", "rules"}
SETTINGS_KEYS = {
    "repository",
    "default_branch",
    "visibility",
    "allow_auto_merge",
    "allow_merge_commit",
    "allow_rebase_merge",
    "allow_squash_merge",
    "delete_branch_on_merge",
    "actions_default_workflow_permissions",
    "actions_can_approve_pull_request_reviews",
}
REPOSITORY_PROJECTION_KEYS = (
    "id",
    "full_name",
    "default_branch",
    "visibility",
    "allow_auto_merge",
    "allow_merge_commit",
    "allow_rebase_merge",
    "allow_squash_merge",
    "delete_branch_on_merge",
)
WORKFLOW_PROJECTION_KEYS = ("id", "name", "path", "state")
WORKFLOW_RUN_PROJECTION_KEYS = (
    "id",
    "workflow_id",
    "check_suite_id",
    "run_attempt",
    "event",
    "head_sha",
    "status",
    "conclusion",
    "created_at",
    "updated_at",
)
CHECK_RUN_PROJECTION_KEYS = (
    "id",
    "name",
    "check_suite_id",
    "head_sha",
    "status",
    "conclusion",
    "started_at",
    "completed_at",
)
CHECK_RUN_APP_PROJECTION_KEYS = ("id", "slug")
RULESET_PROJECTION_KEYS = (
    "id",
    "name",
    "target",
    "enforcement",
    "bypass_actors",
    "conditions",
    "rules",
)
ACTION_PERMISSIONS_PROJECTION_KEYS = (
    "default_workflow_permissions",
    "can_approve_pull_request_reviews",
)
ACTIONS_KEYS = {"default_workflow_permissions", "can_approve_pull_request_reviews"}
_CREDENTIAL_KEY_PARTS = (
    "authorization",
    "credential",
    "password",
    "private_key",
    "secret",
    "token",
    "cookie",
)
MAX_CANONICAL_DOCUMENT_NODES = 100_000
MAX_CANONICAL_DOCUMENT_DEPTH = 64
MAX_STDIN_TOKEN_BYTES = 1024
AMBIENT_CREDENTIAL_NAMES = (
    "GH_TOKEN",
    "GITHUB_TOKEN",
    "GARNET_ADMIN_GITHUB_TOKEN",
    "GARNET_REVIEW_GITHUB_TOKEN",
    "REVIEW_TOKEN",
)


@dataclass(frozen=True)
class GovernanceTransportEvidence:
    repository: object
    workflows: object
    workflow_runs: object
    check_runs: object
    ruleset: object
    actions_permissions: object


@dataclass(frozen=True)
class GovernanceBindingEvidence:
    context: str
    workflow_path: str
    workflow_id: int
    run_id: int
    run_attempt: int
    check_suite_id: int
    check_run_id: int
    completed_at: str


@dataclass(frozen=True)
class GovernanceGateStatus:
    schema: str
    evidence_authority: str
    repository: str
    default_branch: str
    reviewed_head: str
    observed_at: str
    workflow_count: int
    selected_run_count: int
    required_check_count: int
    ruleset_id: int | None
    transport_complete: bool
    exact_head: bool
    fresh: bool
    outcomes_verified: bool
    identity_verified: bool
    policy_equal: bool
    live_settings_no_bypass: str
    bindings: tuple[GovernanceBindingEvidence, ...]
    problems: tuple[str, ...]
    ok: bool


def _checked_no_duplicates(pairs: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in pairs:
        if key in value:
            raise ValueError(f"duplicate JSON key {key!r}")
        value[key] = item
    return value


def _is_reparse(metadata: os.stat_result) -> bool:
    return stat.S_ISLNK(metadata.st_mode) or bool(
        getattr(metadata, "st_file_attributes", 0) & FILE_ATTRIBUTE_REPARSE_POINT
    )


def _read_checked_document(
    root: Path, relative: str, label: str
) -> tuple[object | None, list[str]]:
    """Read one bounded regular checked-in JSON authority without key collapse."""
    root = root.resolve()
    path = root / relative
    try:
        current = root
        for component in Path(relative).parts:
            current /= component
            metadata = os.lstat(current)
            if _is_reparse(metadata):
                raise ValueError(f"{label} path contains a symlink/reparse point")
        leaf = os.lstat(path)
        if not stat.S_ISREG(leaf.st_mode):
            raise ValueError(f"{label} is not a regular file")
        if leaf.st_size > MAX_CHECKED_DOCUMENT_BYTES:
            raise ValueError(f"{label} exceeds size limit")
        descriptor = os.open(
            path,
            os.O_RDONLY | getattr(os, "O_BINARY", 0) | getattr(os, "O_NOFOLLOW", 0),
        )
        try:
            opened = os.fstat(descriptor)
            if not stat.S_ISREG(opened.st_mode) or (opened.st_dev, opened.st_ino) != (
                leaf.st_dev,
                leaf.st_ino,
            ):
                raise ValueError(f"{label} identity changed while opening")
            payload = bytearray()
            while len(payload) <= MAX_CHECKED_DOCUMENT_BYTES:
                chunk = os.read(
                    descriptor,
                    MAX_CHECKED_DOCUMENT_BYTES + 1 - len(payload),
                )
                if not chunk:
                    break
                payload.extend(chunk)
            after = os.fstat(descriptor)
            if (
                opened.st_dev,
                opened.st_ino,
                opened.st_size,
                opened.st_mtime_ns,
            ) != (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns):
                raise ValueError(f"{label} changed while reading")
        finally:
            os.close(descriptor)
        if len(payload) > MAX_CHECKED_DOCUMENT_BYTES:
            raise ValueError(f"{label} exceeds size limit")
        value = json.loads(
            bytes(payload).decode("utf-8"),
            object_pairs_hook=_checked_no_duplicates,
        )
        return value, []
    except (OSError, UnicodeError, json.JSONDecodeError, ValueError) as exc:
        return None, [f"cannot read {label}: {exc}"]


def load_checked_contracts(
    root: Path = Path(__file__).resolve().parents[1],
) -> tuple[object | None, object | None, list[str]]:
    ruleset, ruleset_problems = _read_checked_document(
        root, CHECKED_RULESET_PATH, "checked-in ruleset"
    )
    settings, settings_problems = _read_checked_document(
        root,
        CHECKED_REPOSITORY_SETTINGS_PATH,
        "checked-in repository settings",
    )
    return ruleset, settings, [*ruleset_problems, *settings_problems]


def _failure(
    problems: list[str],
    reviewed_head: object,
    now: object,
    *,
    evidence_authority: str = "injected-offline",
) -> GovernanceGateStatus:
    observed = (
        now.strftime("%Y-%m-%dT%H:%M:%SZ")
        if isinstance(now, datetime)
        and now.tzinfo is timezone.utc
        and now.microsecond == 0
        else ""
    )
    return GovernanceGateStatus(
        SCHEMA,
        evidence_authority,
        EXPECTED_REPOSITORY,
        EXPECTED_DEFAULT_BRANCH,
        reviewed_head if isinstance(reviewed_head, str) and SHA_RE.fullmatch(reviewed_head) else "",
        observed,
        0,
        0,
        0,
        None,
        False,
        False,
        False,
        False,
        False,
        False,
        "blocked-u17",
        (),
        tuple(dict.fromkeys(problems)) or ("governance evidence is invalid",),
        False,
    )


def _canonical_text(value: object, limit: int = 512) -> bool:
    return (
        isinstance(value, str)
        and 0 < len(value) <= limit
        and value == value.strip()
        and value.isprintable()
        and unicodedata.normalize("NFC", value) == value
        and not any(unicodedata.category(char) in {"Cc", "Cf"} for char in value)
    )


def _positive(value: object) -> bool:
    return type(value) is int and 0 < value < 2**63


def _strict_equal(left: object, right: object) -> bool:
    if type(left) is not type(right):
        return False
    if isinstance(left, dict):
        return set(left) == set(right) and all(
            _strict_equal(left[key], right[key]) for key in left
        )
    if isinstance(left, (list, tuple)):
        return len(left) == len(right) and all(
            _strict_equal(a, b) for a, b in zip(left, right, strict=True)
        )
    return left == right


def _contains_credential_field(value: object) -> bool:
    stack = [value]
    seen = 0
    while stack:
        item = stack.pop()
        seen += 1
        if seen > 100_000:
            return True
        if isinstance(item, dict):
            for key, child in item.items():
                if not isinstance(key, str):
                    return True
                normalized = key.casefold().replace("-", "_")
                if any(part in normalized for part in _CREDENTIAL_KEY_PARTS):
                    return True
                stack.append(child)
        elif isinstance(item, (list, tuple)):
            stack.extend(item)
    return False


def _object(result: object, label: str, problems: list[str]) -> dict[str, object] | None:
    if type(result) is not transport.ObjectResult:
        problems.append(f"{label} transport result type is invalid")
        return None
    if type(result.problems) is not tuple or any(
        type(item) is not transport.GitHubTransportProblem for item in result.problems
    ):
        problems.append(f"{label} object result problems tuple is invalid")
        return None
    if result.problems:
        problems.append(f"{label} transport is incomplete")
        return None
    if (
        type(result.byte_count) is not int
        or not 0 < result.byte_count <= transport.MAX_BODY_BYTES
    ):
        problems.append(f"{label} transport byte count is invalid")
        return None
    if type(result.value) is not dict:
        problems.append(f"{label} object is missing or malformed")
        return None
    return result.value


def _collection(result: object, label: str, problems: list[str]) -> tuple[object, ...] | None:
    if type(result) is not transport.CollectionResult:
        problems.append(f"{label} transport result type is invalid")
        return None
    if type(result.problems) is not tuple or any(
        type(item) is not transport.GitHubTransportProblem for item in result.problems
    ):
        problems.append(f"{label} collection problems tuple is invalid")
        return None
    if result.problems:
        problems.append(f"{label} transport is incomplete")
        return None
    if type(result.rows) is not tuple:
        problems.append(f"{label} collection rows must be an exact tuple")
        return None
    if not result.rows:
        problems.append(f"{label} collection is empty")
        return None
    if len(result.rows) > transport.MAX_COLLECTION_ROWS:
        problems.append(f"{label} collection row bound is exceeded")
        return None
    if any(type(row) is not dict for row in result.rows):
        problems.append(f"{label} collection elements must be exact objects")
        return None
    if (
        type(result.page_count) is not int
        or not 0 < result.page_count <= transport.MAX_COLLECTION_PAGES
    ):
        problems.append(f"{label} collection page bound is invalid")
        return None
    if (
        type(result.byte_count) is not int
        or not 0 < result.byte_count <= transport.MAX_COLLECTION_BYTES
    ):
        problems.append(f"{label} collection byte bound is invalid")
        return None
    return result.rows


def _timestamp(value: object, label: str, problems: list[str]) -> datetime | None:
    if not isinstance(value, str) or UTC_RE.fullmatch(value) is None:
        problems.append(f"{label} must be canonical UTC")
        return None
    try:
        return datetime.strptime(value, "%Y-%m-%dT%H:%M:%SZ").replace(
            tzinfo=timezone.utc
        )
    except ValueError:
        problems.append(f"{label} must be canonical UTC")
        return None


def _fresh_interval(
    start_value: object,
    end_value: object,
    now: datetime,
    label: str,
    problems: list[str],
) -> tuple[datetime, datetime] | None:
    start = _timestamp(start_value, f"{label} start timestamp", problems)
    end = _timestamp(end_value, f"{label} end timestamp", problems)
    if start is None or end is None:
        return None
    if start > end:
        problems.append(f"{label} timestamp order is invalid")
    for value in (start, end):
        if value > now:
            problems.append(f"{label} timestamp is in the future")
        elif (now - value).total_seconds() > FRESHNESS_WINDOW_SECONDS:
            problems.append(f"{label} timestamp is stale")
    return start, end


def _canonical_document_sha256(
    value: object, label: str, problems: list[str]
) -> str | None:
    """Hash one bounded, type-exact JSON object using a unique key order."""
    stack: list[tuple[object, int]] = [(value, 1)]
    nodes = 0
    while stack:
        item, depth = stack.pop()
        nodes += 1
        if (
            nodes > MAX_CANONICAL_DOCUMENT_NODES
            or depth > MAX_CANONICAL_DOCUMENT_DEPTH
        ):
            problems.append(f"{label} canonical JSON bounds are exceeded")
            return None
        if type(item) is dict:
            for key, child in item.items():
                if (
                    type(key) is not str
                    or not key
                    or len(key) > 512
                    or not key.isprintable()
                    or unicodedata.normalize("NFC", key) != key
                ):
                    problems.append(f"{label} canonical JSON types are invalid")
                    return None
                stack.append((child, depth + 1))
        elif type(item) is list:
            stack.extend((child, depth + 1) for child in item)
        elif type(item) is str:
            if (
                len(item) > 4096
                or (item and not item.isprintable())
                or unicodedata.normalize("NFC", item) != item
            ):
                problems.append(f"{label} canonical JSON strings are invalid")
                return None
        elif item is None or type(item) is bool:
            continue
        elif type(item) is int:
            if not -(2**63) < item < 2**63:
                problems.append(f"{label} canonical JSON integer is out of bounds")
                return None
        else:
            problems.append(f"{label} canonical JSON types are invalid")
            return None
    try:
        canonical = json.dumps(
            value,
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
            allow_nan=False,
        ).encode("utf-8")
    except (TypeError, ValueError, UnicodeError):
        problems.append(f"{label} canonical JSON encoding failed")
        return None
    return hashlib.sha256(canonical).hexdigest()


def _checked_contract_problems(
    checked_ruleset: object,
    checked_settings: object,
    expected_context_count: int,
) -> list[str]:
    problems: list[str] = []
    ruleset_sha256 = _canonical_document_sha256(
        checked_ruleset, "checked-in ruleset", problems
    )
    expected_ruleset_sha256 = {
        31: EXPECTED_RULESET_DOCUMENT_SHA256,
        32: EXPECTED_ACTIVATED_RULESET_DOCUMENT_SHA256,
    }.get(expected_context_count)
    if ruleset_sha256 != expected_ruleset_sha256:
        problems.append("checked-in ruleset canonical digest is not exact")
    if type(checked_ruleset) is not dict or set(checked_ruleset) != RULESET_KEYS:
        problems.append("checked-in ruleset keys are not exact")
    else:
        if (
            checked_ruleset.get("name") != EXPECTED_RULESET_NAME
            or checked_ruleset.get("target") != EXPECTED_RULESET_TARGET
            or checked_ruleset.get("enforcement") != EXPECTED_RULESET_ENFORCEMENT
            or checked_ruleset.get("bypass_actors") != []
        ):
            problems.append("checked-in ruleset identity is not exact")
        if type(checked_ruleset.get("conditions")) is not dict or type(
            checked_ruleset.get("rules")
        ) is not list:
            problems.append("checked-in ruleset policy is malformed")
    settings_sha256 = _canonical_document_sha256(
        checked_settings, "checked-in repository settings", problems
    )
    if settings_sha256 != EXPECTED_REPOSITORY_SETTINGS_DOCUMENT_SHA256:
        problems.append("checked-in repository settings canonical digest is not exact")
    if type(checked_settings) is not dict or set(checked_settings) != SETTINGS_KEYS:
        problems.append("checked-in repository settings keys are not exact")
    else:
        if (
            checked_settings.get("repository") != EXPECTED_REPOSITORY
            or checked_settings.get("default_branch") != EXPECTED_DEFAULT_BRANCH
        ):
            problems.append("checked-in repository identity is not exact")
    return problems


def _policy_equality_problems(
    live_repository: dict[str, object],
    live_ruleset: dict[str, object],
    live_actions: dict[str, object],
    checked_ruleset: dict[str, object],
    checked_settings: dict[str, object],
) -> list[str]:
    problems: list[str] = []
    if not _strict_equal(live_ruleset.get("id"), EXPECTED_RULESET_ID):
        problems.append("live ruleset id is not exact")
    if live_ruleset.get("bypass_actors") != []:
        problems.append("live ruleset bypass_actors must be empty")
    projection = {key: live_ruleset.get(key) for key in RULESET_KEYS}
    if not _strict_equal(projection, checked_ruleset):
        problems.append("live ruleset policy mismatch")

    if set(live_actions) != ACTIONS_KEYS:
        problems.append("live Actions settings keys are not exact")
    settings_projection: dict[str, object] = {
        "repository": live_repository.get("full_name"),
        "default_branch": live_repository.get("default_branch"),
        "actions_default_workflow_permissions": live_actions.get(
            "default_workflow_permissions"
        ),
        "actions_can_approve_pull_request_reviews": live_actions.get(
            "can_approve_pull_request_reviews"
        ),
    }
    for key in SETTINGS_KEYS - set(settings_projection):
        settings_projection[key] = live_repository.get(key)
    if not _strict_equal(settings_projection, checked_settings):
        problems.append("live repository settings mismatch")
        if (
            not _strict_equal(
                settings_projection.get("actions_default_workflow_permissions"),
                checked_settings.get("actions_default_workflow_permissions"),
            )
            or not _strict_equal(
                settings_projection.get("actions_can_approve_pull_request_reviews"),
                checked_settings.get("actions_can_approve_pull_request_reviews"),
            )
        ):
            problems.append("live Actions settings mismatch")
    return problems


def _evaluate(
    policy: object,
    evidence: GovernanceTransportEvidence,
    *,
    reviewed_head: str,
    now: datetime,
    checked_ruleset: object,
    checked_repository_settings: object,
    evidence_authority: str = "injected-offline",
    live_admin_authoritative: bool = False,
    require_live_settings: bool = False,
) -> GovernanceGateStatus:
    problems: list[str] = []
    initial_bindings = getattr(policy, "bindings", None)
    expected_context_count = (
        len(initial_bindings) if type(initial_bindings) is tuple else 0
    )
    if expected_context_count not in {31, 32}:
        problems.append("required-context policy must contain exactly 31 or 32 bindings")
    if type(reviewed_head) is not str or SHA_RE.fullmatch(reviewed_head) is None:
        problems.append("reviewed head must be one full lowercase SHA")
    if (
        not isinstance(now, datetime)
        or now.tzinfo is not timezone.utc
        or now.microsecond != 0
    ):
        problems.append("injected clock must be second-precision UTC")
    problems.extend(
        _checked_contract_problems(
            checked_ruleset,
            checked_repository_settings,
            expected_context_count,
        )
    )
    if problems:
        return _failure(
            problems,
            reviewed_head,
            now,
            evidence_authority=evidence_authority,
        )
    assert isinstance(checked_ruleset, dict)
    assert isinstance(checked_repository_settings, dict)

    if type(evidence) is not GovernanceTransportEvidence:
        return _failure(
            ["governance transport evidence type is invalid"],
            reviewed_head,
            now,
            evidence_authority=evidence_authority,
        )
    repository = _object(evidence.repository, "repository", problems)
    workflows = _collection(evidence.workflows, "workflows", problems)
    runs = _collection(evidence.workflow_runs, "workflow_runs", problems)
    checks = _collection(evidence.check_runs, "check_runs", problems)
    ruleset: dict[str, object] | None = None
    actions: dict[str, object] | None = None
    admin_presence = (
        evidence.ruleset is not None,
        evidence.actions_permissions is not None,
    )
    if any(admin_presence) and not all(admin_presence):
        problems.append("live admin evidence must contain both settings objects")
    elif all(admin_presence):
        ruleset = _object(evidence.ruleset, "ruleset", problems)
        actions = _object(
            evidence.actions_permissions, "actions_permissions", problems
        )
    if problems:
        return _failure(
            problems,
            reviewed_head,
            now,
            evidence_authority=evidence_authority,
        )
    assert repository is not None and workflows is not None and runs is not None
    assert checks is not None

    if _contains_credential_field(
        (
            repository,
            workflows,
            runs,
            checks,
            *((ruleset, actions) if ruleset is not None and actions is not None else ()),
            checked_ruleset,
            checked_repository_settings,
        )
    ):
        return _failure(
            ["governance input contains a credential-like field"],
            reviewed_head,
            now,
            evidence_authority=evidence_authority,
        )

    if not _positive(repository.get("id")):
        problems.append("repository id must be a positive integer")
    if repository.get("full_name") != EXPECTED_REPOSITORY:
        problems.append("repository full_name is not exact")
    if repository.get("default_branch") != EXPECTED_DEFAULT_BRANCH:
        problems.append("repository default branch is not exact")

    raw_bindings = getattr(policy, "bindings", None)
    policy_problems = getattr(policy, "problems", None)
    expected_bindings = raw_bindings if type(raw_bindings) is tuple else ()
    expected_contexts = tuple(
        getattr(getattr(item, "producer", None), "context", None)
        for item in expected_bindings
    )
    contexts_are_exact = all(
        type(item) is str and _canonical_text(item, 256)
        for item in expected_contexts
    )
    if (
        type(raw_bindings) is not tuple
        or type(policy_problems) is not tuple
        or policy_problems
        or len(expected_bindings) != expected_context_count
        or not contexts_are_exact
        or (
            contexts_are_exact
            and len(set(expected_contexts)) != expected_context_count
        )
    ):
        problems.append(
            "required-context policy must contain exact 31 or activated 32 bindings"
        )

    policy_identity: list[tuple[str, str, str, str, tuple[str, str] | None]] = []
    policy_semantic_identity: list[tuple[str, str]] = []
    policy_binding_identity: list[
        tuple[
            str,
            str,
            str,
            str,
            tuple[str, str] | None,
            str,
            str,
            str,
        ]
    ] = []
    expected_paths: dict[str, object] = {}
    for binding in expected_bindings:
        producer = getattr(binding, "producer", None)
        context = getattr(producer, "context", None)
        path = getattr(producer, "workflow", None)
        event = getattr(producer, "event", None)
        job = getattr(producer, "job", None)
        matrix = getattr(producer, "matrix", None)
        text_fields = ((context, 256), (path, 512), (event, 256), (job, 256))
        matrix_is_exact = matrix is None or (
            type(matrix) is tuple
            and len(matrix) == 2
            and all(type(item) is str and _canonical_text(item, 256) for item in matrix)
        )
        if not all(
            type(value) is str and _canonical_text(value, limit)
            for value, limit in text_fields
        ) or not matrix_is_exact:
            problems.append("required-context producer identity is malformed")
            continue
        assert isinstance(context, str)
        assert isinstance(path, str)
        assert isinstance(event, str)
        assert isinstance(job, str)
        policy_identity.append((context, path, event, job, matrix))
        workflow = getattr(binding, "workflow", None)
        workflow_name = getattr(getattr(workflow, "name", None), "value", None)
        if type(workflow_name) is not str or not _canonical_text(workflow_name, 512):
            problems.append("required-context workflow projection is malformed")
            continue
        declared_semantic = getattr(producer, "semantic_sha256", None)
        observed_semantic = getattr(binding, "observed_semantic_sha256", None)
        if not all(
            type(value) is str and SHA256_RE.fullmatch(value) is not None
            for value in (declared_semantic, observed_semantic)
        ):
            problems.append("required-context policy binding semantics are malformed")
            continue
        assert isinstance(declared_semantic, str)
        assert isinstance(observed_semantic, str)
        if observed_semantic != declared_semantic:
            problems.append("required-context observed semantics differ from declaration")
        policy_semantic_identity.append((context, declared_semantic))
        policy_binding_identity.append(
            (
                context,
                path,
                event,
                job,
                matrix,
                declared_semantic,
                observed_semantic,
                workflow_name,
            )
        )
        if path in expected_paths and expected_paths[path] is not workflow:
            left = getattr(expected_paths[path], "name", None)
            right = getattr(workflow, "name", None)
            if getattr(left, "value", None) != getattr(right, "value", None):
                problems.append("required-context workflow identity is inconsistent")
        expected_paths[path] = workflow

    policy_identity_sha256 = hashlib.sha256(
        json.dumps(
            policy_identity, ensure_ascii=False, separators=(",", ":")
        ).encode("utf-8")
    ).hexdigest()
    policy_semantic_sha256 = hashlib.sha256(
        json.dumps(
            policy_semantic_identity, ensure_ascii=False, separators=(",", ":")
        ).encode("utf-8")
    ).hexdigest()
    policy_binding_sha256 = hashlib.sha256(
        json.dumps(
            policy_binding_identity, ensure_ascii=False, separators=(",", ":")
        ).encode("utf-8")
    ).hexdigest()
    expected_identity_sha256 = {
        31: EXPECTED_POLICY_IDENTITY_SHA256,
        32: EXPECTED_ACTIVATED_POLICY_IDENTITY_SHA256,
    }[expected_context_count]
    expected_semantic_sha256 = {
        31: EXPECTED_POLICY_SEMANTIC_SHA256,
        32: EXPECTED_ACTIVATED_POLICY_SEMANTIC_SHA256,
    }[expected_context_count]
    expected_binding_sha256 = {
        31: EXPECTED_POLICY_BINDING_SHA256,
        32: EXPECTED_ACTIVATED_POLICY_BINDING_SHA256,
    }[expected_context_count]
    checked_contract_sha256 = getattr(
        required_context_contract,
        (
            "PREACTIVATION_PRODUCER_IDENTITY_SHA256"
            if expected_context_count == 31
            else "ACTIVATED_PRODUCER_IDENTITY_SHA256"
        ),
        None,
    )
    checked_semantic_sha256 = getattr(
        required_context_contract,
        (
            "PREACTIVATION_PRODUCER_SEMANTIC_SHA256"
            if expected_context_count == 31
            else "ACTIVATED_PRODUCER_SEMANTIC_SHA256"
        ),
        None,
    )
    if checked_contract_sha256 != expected_identity_sha256:
        problems.append("checked-in required-context contract digest is not exact")
    if checked_semantic_sha256 != expected_semantic_sha256:
        problems.append("checked-in required-context semantic digest is not exact")
    if policy_identity_sha256 != expected_identity_sha256:
        problems.append("required-context canonical policy digest is not exact")
    if policy_semantic_sha256 != expected_semantic_sha256:
        problems.append("required-context canonical semantic digest is not exact")
    if policy_binding_sha256 != expected_binding_sha256:
        problems.append("required-context canonical policy binding digest is not exact")
    if problems:
        return _failure(
            problems,
            reviewed_head,
            now,
            evidence_authority=evidence_authority,
        )

    normalized_workflows: list[dict[str, object]] = []
    workflow_ids: set[int] = set()
    workflow_paths: set[str] = set()
    by_path: dict[str, dict[str, object]] = {}
    for row in workflows:
        if not isinstance(row, dict):
            problems.append("workflow row is malformed")
            continue
        required = {"id", "name", "path", "state"}
        if not required <= set(row):
            problems.append("workflow row is missing identity fields")
            continue
        identifier, name, path, state = (
            row.get("id"),
            row.get("name"),
            row.get("path"),
            row.get("state"),
        )
        if not _positive(identifier) or not all(
            _canonical_text(value) for value in (name, path, state)
        ):
            problems.append("workflow row identity is malformed")
            continue
        if identifier in workflow_ids or path in workflow_paths:
            problems.append("duplicate workflow id/domain identity")
        workflow_ids.add(identifier)
        workflow_paths.add(path)
        projected = {"id": identifier, "name": name, "path": path, "state": state}
        normalized_workflows.append(projected)
        by_path[path] = projected

    run_ids: set[int] = set()
    suite_ids: set[int] = set()
    run_domains: set[tuple[int, str, int]] = set()
    parsed_runs: list[dict[str, object]] = []
    run_keys = {
        "id",
        "workflow_id",
        "check_suite_id",
        "run_attempt",
        "event",
        "head_sha",
        "status",
        "conclusion",
        "created_at",
        "updated_at",
    }
    for row in runs:
        if not isinstance(row, dict) or not run_keys <= set(row):
            problems.append("workflow run row is missing exact-head/outcome fields")
            continue
        identifiers = (row.get("id"), row.get("workflow_id"), row.get("check_suite_id"))
        if not all(_positive(value) for value in identifiers) or not _positive(
            row.get("run_attempt")
        ):
            problems.append("workflow run identity is malformed")
            continue
        if not all(
            _canonical_text(row.get(key))
            for key in ("event", "head_sha", "status", "conclusion", "created_at", "updated_at")
        ):
            problems.append("workflow run text is malformed")
            continue
        run_id, workflow_id, suite_id = identifiers
        domain = (workflow_id, row["head_sha"], row["run_attempt"])
        if run_id in run_ids or suite_id in suite_ids:
            problems.append("duplicate workflow run or check-suite identity")
        if domain in run_domains:
            problems.append("duplicate workflow/attempt domain identity")
        run_ids.add(run_id)
        suite_ids.add(suite_id)
        run_domains.add(domain)
        parsed_runs.append(row)

    selected_by_workflow: dict[int, dict[str, object]] = {}
    selected_runs: list[dict[str, object]] = []
    for path in expected_paths:
        workflow = by_path.get(path)
        if workflow is None:
            problems.append("expected workflow is absent from live enumeration")
            continue
        candidates = [
            row
            for row in parsed_runs
            if row["workflow_id"] == workflow["id"]
            and row["head_sha"] == reviewed_head
        ]
        if not candidates:
            problems.append("expected workflow has no run for the exact reviewed head")
            continue
        latest = max(row["run_attempt"] for row in candidates)
        selected = [row for row in candidates if row["run_attempt"] == latest]
        if len(selected) != 1:
            problems.append("latest workflow attempt is not unique")
            continue
        run = selected[0]
        selected_by_workflow[workflow["id"]] = run
        selected_runs.append(run)
        if (run.get("status"), run.get("conclusion")) != ("completed", "success"):
            problems.append("selected workflow run must be completed/success")
        _fresh_interval(
            run.get("created_at"),
            run.get("updated_at"),
            now,
            "selected workflow run",
            problems,
        )

    check_ids: set[int] = set()
    check_domains: set[tuple[int, str]] = set()
    parsed_checks: list[dict[str, object]] = []
    check_keys = {
        "id",
        "name",
        "check_suite_id",
        "head_sha",
        "status",
        "conclusion",
        "started_at",
        "completed_at",
        "app",
    }
    for row in checks:
        if not isinstance(row, dict) or not check_keys <= set(row):
            problems.append("check run row is missing exact-head/outcome fields")
            continue
        app = row.get("app")
        if not isinstance(app, dict) or not {"id", "slug"} <= set(app):
            problems.append("check run App identity is malformed")
            continue
        identifier, suite_id = row.get("id"), row.get("check_suite_id")
        if not _positive(identifier) or not _positive(suite_id) or not _positive(app.get("id")):
            problems.append("check run identity is malformed")
            continue
        if not all(
            _canonical_text(row.get(key))
            for key in (
                "name",
                "head_sha",
                "status",
                "conclusion",
                "started_at",
                "completed_at",
            )
        ) or not _canonical_text(app.get("slug")):
            problems.append("check run text is malformed")
            continue
        domain = (suite_id, row["name"])
        if identifier in check_ids or domain in check_domains:
            problems.append("duplicate check run/domain identity")
        check_ids.add(identifier)
        check_domains.add(domain)
        parsed_checks.append(row)

    selected_suite_ids = {row["check_suite_id"] for row in selected_runs}
    expected_context_set = set(expected_contexts)
    required_context_counts: dict[str, int] = {}
    for row in parsed_checks:
        name = row["name"]
        if name in expected_context_set:
            required_context_counts[name] = required_context_counts.get(name, 0) + 1
    duplicated_required_contexts = sorted(
        name for name, count in required_context_counts.items() if count > 1
    )
    if duplicated_required_contexts:
        problems.append(
            "required context names must be globally unique across all check runs: "
            + ", ".join(duplicated_required_contexts)
        )
    selected_checks = [
        row
        for row in parsed_checks
        if row["check_suite_id"] in selected_suite_ids
        and row["name"] in expected_context_set
    ]
    for check in selected_checks:
        if check.get("head_sha") != reviewed_head:
            problems.append("selected check does not bind the exact reviewed head")
        if (check.get("status"), check.get("conclusion")) != ("completed", "success"):
            problems.append("selected check run must be completed/success")
        _fresh_interval(
            check.get("started_at"),
            check.get("completed_at"),
            now,
            "selected check run",
            problems,
        )

    normalized_runs = [
        {
            "id": row["id"],
            "workflow_id": row["workflow_id"],
            "check_suite_id": row["check_suite_id"],
            "event": row["event"],
        }
        for row in selected_runs
    ]
    normalized_checks = [
        {
            "id": row["id"],
            "name": row["name"],
            "check_suite_id": row["check_suite_id"],
            "app": {"id": row["app"]["id"], "slug": row["app"]["slug"]},
        }
        for row in selected_checks
    ]
    normalized = {
        "schema": identity.SCHEMA,
        "workflows": normalized_workflows,
        "workflow_runs": normalized_runs,
        "check_runs": normalized_checks,
    }
    snapshot = identity.parse_live_identity_snapshot(
        json.dumps(normalized, ensure_ascii=False, separators=(",", ":"))
    )
    identity_result = identity.evaluate_live_identity(policy, snapshot)
    problems.extend(identity_result.problems)
    admin_policy_equal = False
    if ruleset is not None and actions is not None:
        admin_problems = _policy_equality_problems(
            repository,
            ruleset,
            actions,
            checked_ruleset,
            checked_repository_settings,
        )
        problems.extend(admin_problems)
        admin_policy_equal = not admin_problems
    live_settings_verified = live_admin_authoritative and admin_policy_equal
    if require_live_settings and not live_settings_verified:
        problems.append("blocked-u17: live admin settings evidence is unavailable")
    if problems:
        return _failure(
            problems,
            reviewed_head,
            now,
            evidence_authority=evidence_authority,
        )

    run_by_id = {row["id"]: row for row in selected_runs}
    check_by_id = {row["id"]: row for row in selected_checks}
    bindings: list[GovernanceBindingEvidence] = []
    for item in identity_result.bindings:
        run = run_by_id[item.run.id]
        check = check_by_id[item.check.id]
        bindings.append(
            GovernanceBindingEvidence(
                item.producer.context,
                item.producer.workflow,
                item.workflow.id,
                item.run.id,
                run["run_attempt"],
                item.run.check_suite_id,
                item.check.id,
                check["completed_at"],
            )
        )
    observed = now.strftime("%Y-%m-%dT%H:%M:%SZ")
    return GovernanceGateStatus(
        SCHEMA,
        evidence_authority,
        EXPECTED_REPOSITORY,
        EXPECTED_DEFAULT_BRANCH,
        reviewed_head,
        observed,
        len(expected_paths),
        len(selected_runs),
        len(bindings),
        EXPECTED_RULESET_ID,
        True,
        True,
        True,
        True,
        True,
        admin_policy_equal,
        "verified-empty" if live_settings_verified else "blocked-u17",
        tuple(bindings),
        (),
        True,
    )


def evaluate_governance_gate(
    policy: object,
    evidence: GovernanceTransportEvidence,
    *,
    reviewed_head: str,
    now: datetime,
    root: Path = Path(__file__).resolve().parents[1],
    evidence_authority: str = "injected-offline",
    live_admin_authoritative: bool = False,
    require_live_settings: bool = False,
) -> GovernanceGateStatus:
    """Load strict checked authorities, then evaluate complete injected evidence."""
    try:
        checked_ruleset, checked_repository_settings, load_problems = (
            load_checked_contracts(root)
        )
        if load_problems:
            return _failure(
                load_problems,
                reviewed_head,
                now,
                evidence_authority=evidence_authority,
            )
        return _evaluate(
            policy,
            evidence,
            reviewed_head=reviewed_head,
            now=now,
            checked_ruleset=checked_ruleset,
            checked_repository_settings=checked_repository_settings,
            evidence_authority=evidence_authority,
            live_admin_authoritative=live_admin_authoritative,
            require_live_settings=require_live_settings,
        )
    except Exception:
        return _failure(
            ["governance evaluation failed closed"],
            reviewed_head,
            now,
            evidence_authority=evidence_authority,
        )


def load_checked_policy(root: Path) -> object:
    """Build the exact checked 31- or 32-context producer policy."""
    workflow_schema = _load_sibling("garnet_workflow_schema_policy")
    inventory = required_context_contract.load_inventory(
        root / required_context_contract.INVENTORY_PATH
    )
    ledger = required_context_contract.load_required_check_ledger(
        root / required_context_contract.RULESET_PATH
    )
    projection = workflow_schema.workflow_projection(root)
    state, state_problems = required_context_contract._activation_state(
        inventory, ledger, "checked"
    )
    problems = list(state_problems)
    if state == required_context_contract.PREACTIVATION_REQUIRED_COUNT:
        problems.extend(
            required_context_contract.preactivation_ruleset_problems(
                inventory, ledger
            )
        )
    availability = required_context_contract.evaluate_producer_availability(
        inventory, projection
    )
    problems.extend(availability.problems)
    if not problems:
        contexts = tuple(
            item.producer.context for item in availability.bindings
        )
        if contexts != ledger.contexts:
            problems.append(
                "evaluated bindings do not match checked ruleset ordered contexts"
            )
    if problems:
        return required_context_contract.ProducerEvaluation(
            problems=tuple(dict.fromkeys(problems))
        )
    return availability


def _project_object_result(result: object, keys: tuple[str, ...]) -> object:
    """Project one complete object result onto the gate's explicit evidence schema."""
    if (
        type(result) is not transport.ObjectResult
        or result.problems
        or type(result.value) is not dict
    ):
        return result
    projection = {key: result.value.get(key) for key in keys}
    return transport.ObjectResult(
        value=projection,
        problems=result.problems,
        byte_count=result.byte_count,
    )


def _project_collection_result(
    result: object,
    keys: tuple[str, ...],
    *,
    nested_app: bool = False,
    nested_check_suite_id: bool = False,
) -> object:
    """Project complete collection rows without altering transport bounds/provenance."""
    if (
        type(result) is not transport.CollectionResult
        or result.problems
        or type(result.rows) is not tuple
        or any(type(row) is not dict for row in result.rows)
    ):
        return result
    rows: list[dict[str, object]] = []
    for row in result.rows:
        projection = {key: row.get(key) for key in keys}
        if nested_check_suite_id:
            check_suite = row.get("check_suite")
            suite_id = (
                check_suite.get("id") if type(check_suite) is dict else None
            )
            if not _positive(suite_id):
                return transport.CollectionResult(
                    problems=(transport.GitHubTransportProblem("collection-shape"),),
                    page_count=result.page_count,
                    byte_count=result.byte_count,
                )
            projection["check_suite_id"] = suite_id
        if nested_app:
            app = row.get("app")
            projection["app"] = (
                {key: app.get(key) for key in CHECK_RUN_APP_PROJECTION_KEYS}
                if type(app) is dict
                else app
            )
        rows.append(projection)
    return transport.CollectionResult(
        rows=tuple(rows),
        problems=result.problems,
        page_count=result.page_count,
        byte_count=result.byte_count,
    )


def collect_live_governance_status(
    policy: object,
    *,
    reviewed_head: str,
    token: str,
    now: datetime,
    include_admin: bool,
    root: Path = Path(__file__).resolve().parents[1],
    transport_factory: object = transport.GitHubGovernanceTransport,
) -> GovernanceGateStatus:
    """Collect bounded live API results, then run the same all-or-zero evaluator."""
    authority = "live-explicit-stdin"
    token_is_exact = (
        type(token) is str
        and 0 < len(token) <= MAX_STDIN_TOKEN_BYTES
        and all(33 <= ord(character) <= 126 for character in token)
    )
    if (
        type(reviewed_head) is not str
        or SHA_RE.fullmatch(reviewed_head) is None
        or not token_is_exact
        or type(include_admin) is not bool
        or not callable(transport_factory)
    ):
        return _failure(
            ["live collector configuration is invalid"],
            reviewed_head,
            now,
            evidence_authority=authority,
        )
    try:
        client = transport_factory(EXPECTED_REPOSITORY, token)
        evidence = GovernanceTransportEvidence(
            repository=_project_object_result(
                client.get_repository(), REPOSITORY_PROJECTION_KEYS
            ),
            workflows=_project_collection_result(
                client.get_collection(
                    "actions/workflows",
                    root_key="workflows",
                    require_total_count=True,
                ),
                WORKFLOW_PROJECTION_KEYS,
            ),
            workflow_runs=_project_collection_result(
                client.get_collection(
                    f"actions/runs?head_sha={reviewed_head}",
                    root_key="workflow_runs",
                    require_total_count=True,
                ),
                WORKFLOW_RUN_PROJECTION_KEYS,
            ),
            check_runs=_project_collection_result(
                client.get_collection(
                    f"commits/{reviewed_head}/check-runs",
                    root_key="check_runs",
                    require_total_count=True,
                ),
                CHECK_RUN_PROJECTION_KEYS,
                nested_app=True,
                nested_check_suite_id=True,
            ),
            ruleset=(
                _project_object_result(
                    client.get_object(f"rulesets/{EXPECTED_RULESET_ID}"),
                    RULESET_PROJECTION_KEYS,
                )
                if include_admin
                else None
            ),
            actions_permissions=(
                _project_object_result(
                    client.get_object("actions/permissions/workflow"),
                    ACTION_PERMISSIONS_PROJECTION_KEYS,
                )
                if include_admin
                else None
            ),
        )
    except Exception:
        return _failure(
            ["live governance collection failed closed"],
            reviewed_head,
            now,
            evidence_authority=authority,
        )
    return evaluate_governance_gate(
        policy,
        evidence,
        reviewed_head=reviewed_head,
        now=now,
        root=root,
        evidence_authority=authority,
        live_admin_authoritative=include_admin,
        require_live_settings=include_admin,
    )


def _read_explicit_token(stream: object) -> tuple[str | None, list[str]]:
    """Read one nonempty printable token; never return caller-controlled text in errors."""
    try:
        payload = stream.read(MAX_STDIN_TOKEN_BYTES + 2)
        if isinstance(payload, bytes):
            payload = payload.decode("ascii", errors="strict")
        if type(payload) is not str:
            raise ValueError
        if payload.endswith("\n"):
            payload = payload[:-1]
        if (
            not payload
            or "\n" in payload
            or "\r" in payload
            or len(payload.encode("ascii", errors="strict")) > MAX_STDIN_TOKEN_BYTES
            or any(not 33 <= ord(character) <= 126 for character in payload)
        ):
            raise ValueError
        return payload, []
    except (AttributeError, UnicodeError, ValueError):
        return None, ["explicit stdin credential is missing or malformed"]


def read_clean_local_head(
    root: Path, environment: dict[str, str] | os._Environ[str]
) -> tuple[str, tuple[str, ...]]:
    """Bind policy bytes to one clean, replacement-disabled local commit."""
    child_environment = {
        name: environment[name]
        for name in (
            "PATH",
            "SYSTEMROOT",
            "SystemRoot",
            "WINDIR",
            "COMSPEC",
            "ComSpec",
            "PATHEXT",
            "TMP",
            "TEMP",
            "TMPDIR",
        )
        if environment.get(name)
    }
    child_environment.update(
        {
            "GIT_ATTR_NOSYSTEM": "1",
            "GIT_CONFIG_COUNT": "0",
            "GIT_CONFIG_GLOBAL": os.devnull,
            "GIT_NO_REPLACE_OBJECTS": "1",
            "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_CONFIG_SYSTEM": os.devnull,
            "GIT_OPTIONAL_LOCKS": "0",
            "GIT_TERMINAL_PROMPT": "0",
            "GIT_CONFIG_NOSYSTEM": "1",
            "LC_ALL": "C",
            "LANG": "C",
        }
    )

    def run(arguments: list[str]) -> subprocess.CompletedProcess[bytes]:
        return subprocess.run(
            [
                "git",
                "--no-replace-objects",
                "-c",
                "core.hooksPath=/dev/null",
                "-c",
                "core.fsmonitor=false",
                "-C",
                str(root.resolve()),
                *arguments,
            ],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            check=False,
            timeout=15,
            env=child_environment,
        )

    try:
        resolved = run(["rev-parse", "--verify", "HEAD^{commit}"])
        status = run(["status", "--porcelain=v2", "--untracked-files=all"])
    except (OSError, subprocess.TimeoutExpired):
        return "", ("cannot resolve clean local HEAD",)
    if (
        resolved.returncode != 0
        or status.returncode != 0
        or len(resolved.stdout) > 256
        or len(status.stdout) > 1024 * 1024
    ):
        return "", ("cannot resolve clean local HEAD",)
    try:
        head = resolved.stdout.decode("ascii", errors="strict").strip()
    except UnicodeError:
        return "", ("cannot resolve clean local HEAD",)
    problems: list[str] = []
    if SHA_RE.fullmatch(head) is None:
        problems.append("local HEAD is not one full commit SHA")
    if status.stdout:
        problems.append("working tree is not clean")
    return head, tuple(problems)


def main(
    argv: list[str] | None = None,
    *,
    root: Path = Path(__file__).resolve().parents[1],
    stdin: object | None = None,
    stdout: object | None = None,
    environ: dict[str, str] | os._Environ[str] | None = None,
    now: datetime | None = None,
    policy_loader: object = load_checked_policy,
    transport_factory: object = transport.GitHubGovernanceTransport,
    local_head_loader: object = read_clean_local_head,
) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument(
        "--runtime-gate",
        action="store_true",
        help="prove live fresh/exact-head/outcomes without admin settings",
    )
    mode.add_argument(
        "--admin-gate",
        action="store_true",
        help="also require admin-authoritative live settings/no-bypass proof",
    )
    parser.add_argument("--reviewed-head", required=True)
    parser.add_argument(
        "--github-token-stdin",
        action="store_true",
        required=True,
        help="read the only credential from bounded stdin",
    )
    args = parser.parse_args(argv)
    input_stream = sys.stdin if stdin is None else stdin
    output_stream = sys.stdout if stdout is None else stdout
    source_environment = os.environ if environ is None else environ
    environment = dict(source_environment)
    current = (
        datetime.now(timezone.utc).replace(microsecond=0) if now is None else now
    )
    ambient = [
        name for name in AMBIENT_CREDENTIAL_NAMES if source_environment.get(name)
    ]
    for name in AMBIENT_CREDENTIAL_NAMES:
        environment.pop(name, None)
    token, token_problems = _read_explicit_token(input_stream)
    if ambient or token_problems or not callable(local_head_loader):
        problems = [
            *(["ambient GitHub credential variables are forbidden"] if ambient else []),
            *token_problems,
            *(
                ["local HEAD loader configuration is invalid"]
                if not callable(local_head_loader)
                else []
            ),
        ]
        status = _failure(
            problems,
            args.reviewed_head,
            current,
            evidence_authority="live-explicit-stdin",
        )
    else:
        try:
            local_head, local_problems = local_head_loader(root, environment)
        except Exception:
            status = _failure(
                ["clean local HEAD loading failed closed"],
                args.reviewed_head,
                current,
                evidence_authority="live-explicit-stdin",
            )
        else:
            head_problems = list(local_problems)
            if local_head != args.reviewed_head:
                head_problems.append("reviewed head differs from clean local HEAD")
            if head_problems:
                status = _failure(
                    head_problems,
                    args.reviewed_head,
                    current,
                    evidence_authority="live-explicit-stdin",
                )
            else:
                try:
                    policy = policy_loader(root)
                except Exception:
                    status = _failure(
                        ["checked producer policy loading failed closed"],
                        args.reviewed_head,
                        current,
                        evidence_authority="live-explicit-stdin",
                    )
                else:
                    assert token is not None
                    status = collect_live_governance_status(
                        policy,
                        reviewed_head=args.reviewed_head,
                        token=token,
                        now=current,
                        include_admin=bool(args.admin_gate),
                        root=root,
                        transport_factory=transport_factory,
                    )
    output_stream.write(
        json.dumps(asdict(status), ensure_ascii=False, indent=2, sort_keys=True)
        + "\n"
    )
    return 0 if status.ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
