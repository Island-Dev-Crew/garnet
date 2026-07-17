#!/usr/bin/env python3
"""Strict declarative schema for Garnet required-context producers."""
from __future__ import annotations

import hashlib
import json
import os
import re
import stat
import unicodedata
from collections import Counter
from dataclasses import dataclass, field
from pathlib import Path


SCHEMA = "garnet.required-context-producers/v2"
INVENTORY_PATH = ".github/rulesets/required-context-producers.json"
RULESET_PATH = ".github/rulesets/garnet-main.json"
TARGET_BRANCH = "main"
MAX_INVENTORY_BYTES = 256 * 1024
ACTIONS_INTEGRATION_ID = 15368
PREACTIVATION_REQUIRED_COUNT = 31
PREACTIVATION_PRODUCER_IDENTITY_SHA256 = (
    "899944d4f0344e4b53cdd3cb37b1da26061f5eaab5d49d8482f8157b1ed51aaa"
)
PREACTIVATION_PRODUCER_SEMANTIC_SHA256 = (
    "1b5eeb6bdc983c35073726494aee26bb5bc6d72384204297f367e411090b4ee1"
)
BASE_CONTROLLED_CONTEXT = "Base-controlled trust policy"
BASE_CONTROLLED_SEMANTIC_SHA256 = (
    "618a3bf5b61a8083baf936c33e0deec0e20b0c07692b943ff42129d496acf355"
)
FILE_ATTRIBUTE_REPARSE_POINT = 0x400
ID_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_-]*$")
WORKFLOW_RE = re.compile(r"^\.github/workflows/[a-z0-9_.-]+\.(?:yml|yaml)$")
MATRIX_VALUE_RE = re.compile(r"^[A-Za-z0-9_.-]+$")
SEMANTIC_SHA256_RE = re.compile(r"^[0-9a-f]{64}$")


@dataclass(frozen=True)
class Producer:
    context: str
    workflow: str
    event: str
    job: str
    matrix: tuple[str, str] | None = None
    semantic_sha256: str = ""


BASE_CONTROLLED_PRODUCER = Producer(
    BASE_CONTROLLED_CONTEXT,
    ".github/workflows/base-controlled-trust.yml",
    "pull_request_target",
    "policy",
    None,
    BASE_CONTROLLED_SEMANTIC_SHA256,
)


@dataclass
class ProducerInventory:
    producers: list[Producer] = field(default_factory=list)
    optional_contexts: set[str] = field(default_factory=set)
    target_branch: str = ""
    problems: list[str] = field(default_factory=list)


@dataclass(frozen=True)
class RequiredCheckLedger:
    contexts: tuple[str, ...] = ()
    problems: tuple[str, ...] = ()


@dataclass(frozen=True)
class ProducerBinding:
    producer: Producer
    workflow: object
    event: object
    occurrence: object
    dependency_contexts: tuple[str, ...]
    observed_semantic_sha256: str


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


def _read_regular_utf8(path: Path, label: str = "producer inventory") -> str:
    """Read one bounded regular file without accepting path indirection."""
    absolute = path.absolute()
    for component in [*reversed(absolute.parents), absolute]:
        try:
            metadata = os.lstat(component)
        except OSError as exc:
            raise ValueError(f"cannot inspect {label} path: {exc}") from exc
        if _is_reparse(metadata):
            raise ValueError(f"{label} path contains symlink/reparse point: {component}")
    leaf = os.lstat(absolute)
    if not stat.S_ISREG(leaf.st_mode):
        raise ValueError(f"{label} is not a regular file")
    flags = os.O_RDONLY | getattr(os, "O_BINARY", 0) | getattr(os, "O_NOFOLLOW", 0)
    try:
        descriptor = os.open(absolute, flags)
    except OSError as exc:
        raise ValueError(f"cannot open {label}: {exc}") from exc
    try:
        opened = os.fstat(descriptor)
        if not stat.S_ISREG(opened.st_mode) or (leaf.st_dev, leaf.st_ino) != (
            opened.st_dev,
            opened.st_ino,
        ):
            raise ValueError(f"{label} identity changed while opening")
        if opened.st_size > MAX_INVENTORY_BYTES:
            raise ValueError(f"{label} exceeds size limit")
        payload = bytearray()
        while len(payload) <= MAX_INVENTORY_BYTES:
            chunk = os.read(descriptor, MAX_INVENTORY_BYTES + 1 - len(payload))
            if not chunk:
                break
            payload.extend(chunk)
        after = os.fstat(descriptor)
        stable = (opened.st_dev, opened.st_ino, opened.st_size, opened.st_mtime_ns)
        if stable != (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns):
            raise ValueError(f"{label} changed while reading")
        if len(payload) > MAX_INVENTORY_BYTES:
            raise ValueError(f"{label} exceeds size limit")
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


def load_required_check_ledger(path: Path) -> RequiredCheckLedger:
    """Load the strict required-check subdocument from the checked-in ruleset."""
    try:
        raw = json.loads(
            _read_regular_utf8(path, "ruleset mirror"),
            object_pairs_hook=_no_duplicates,
        )
    except (OSError, UnicodeError, json.JSONDecodeError, RecursionError, ValueError) as exc:
        return RequiredCheckLedger(problems=(f"cannot read required-check ruleset: {exc}",))
    if not isinstance(raw, dict) or not isinstance(raw.get("rules"), list):
        return RequiredCheckLedger(problems=("ruleset rules must be a list",))
    rules = raw["rules"]
    if not all(isinstance(item, dict) for item in rules):
        return RequiredCheckLedger(problems=("ruleset entries must be objects",))
    matches = [item for item in rules if item.get("type") == "required_status_checks"]
    if len(matches) != 1:
        return RequiredCheckLedger(
            problems=("ruleset must contain exactly one required_status_checks rule",)
        )
    rule = matches[0]
    problems: list[str] = []
    if set(rule) != {"type", "parameters"}:
        problems.append("required status-check rule keys are not exact")
    parameters = rule.get("parameters")
    expected_keys = {
        "do_not_enforce_on_create",
        "strict_required_status_checks_policy",
        "required_status_checks",
    }
    if not isinstance(parameters, dict):
        return RequiredCheckLedger(problems=(*problems, "required-check parameters are missing"))
    if set(parameters) != expected_keys:
        problems.append("required-check parameter keys are not exact")
    if parameters.get("strict_required_status_checks_policy") is not True:
        problems.append("required checks must run against the latest base")
    if parameters.get("do_not_enforce_on_create") is not False:
        problems.append("required checks must be enforced on creation")
    rows = parameters.get("required_status_checks")
    if not isinstance(rows, list) or not rows:
        return RequiredCheckLedger(problems=(*problems, "required-check rows must be non-empty"))
    contexts: list[str] = []
    for index, row in enumerate(rows):
        if not isinstance(row, dict) or set(row) != {"context", "integration_id"}:
            problems.append(f"required-check row keys are not exact at index {index}")
            continue
        context = row.get("context")
        integration = row.get("integration_id")
        if not isinstance(context, str) or not context or not _canonical_context(context):
            problems.append(f"required-check context is not canonical at index {index}")
        else:
            contexts.append(context)
        if type(integration) is not int or integration != ACTIONS_INTEGRATION_ID:
            problems.append(f"required-check integration is not GitHub Actions at index {index}")
    if len(set(contexts)) != len(contexts):
        problems.append("required-check ledger contains duplicate contexts")
    if problems:
        return RequiredCheckLedger(problems=tuple(problems))
    return RequiredCheckLedger(tuple(contexts), ())


def load_inventory(path: Path) -> ProducerInventory:
    """Load an exact, duplicate-key-free producer inventory."""
    try:
        raw = json.loads(
            _read_regular_utf8(path), object_pairs_hook=_no_duplicates
        )
    except (OSError, UnicodeError, json.JSONDecodeError, RecursionError, ValueError) as exc:
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
    base_keys = {"context", "workflow", "event", "job", "semantic_sha256"}
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
        semantic_sha256 = row.get("semantic_sha256")
        if (
            not isinstance(semantic_sha256, str)
            or not SEMANTIC_SHA256_RE.fullmatch(semantic_sha256)
        ):
            problems.append(f"producers[{index}] semantic_sha256 is not exact lowercase hex")
            semantic_sha256 = ""
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
        producers.append(
            Producer(context, workflow, event, job, matrix, semantic_sha256)
        )
    if optional - seen:
        problems.append("optional_contexts names contexts absent from producers")
    return ProducerInventory(producers, optional, branch, problems)


def preactivation_ruleset_problems(
    inventory: ProducerInventory, ledger: RequiredCheckLedger
) -> tuple[str, ...]:
    """Detect drift from the pinned 31-check preactivation identity."""
    problems = [*inventory.problems, *ledger.problems]
    if problems:
        return tuple(problems)
    if inventory.optional_contexts != {BASE_CONTROLLED_CONTEXT}:
        problems.append("pre-activation optional_contexts must contain only Base-controlled trust")
    base = [item for item in inventory.producers if item.context == BASE_CONTROLLED_CONTEXT]
    if base != [BASE_CONTROLLED_PRODUCER]:
        problems.append("Base-controlled producer identity is not exact")
    active = tuple(
        item.context for item in inventory.producers
        if item.context not in inventory.optional_contexts
    )
    identity = [
        (item.context, item.workflow, item.event, item.job, item.matrix)
        for item in inventory.producers
        if item.context not in inventory.optional_contexts
    ]
    identity_sha256 = hashlib.sha256(
        json.dumps(identity, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
    ).hexdigest()
    if identity_sha256 != PREACTIVATION_PRODUCER_IDENTITY_SHA256:
        problems.append("pre-activation baseline context identity is not exact")
    semantic_identity = [
        (item.context, item.semantic_sha256)
        for item in inventory.producers
        if item.context not in inventory.optional_contexts
    ]
    semantic_sha256 = hashlib.sha256(
        json.dumps(
            semantic_identity, ensure_ascii=False, separators=(",", ":")
        ).encode("utf-8")
    ).hexdigest()
    if semantic_sha256 != PREACTIVATION_PRODUCER_SEMANTIC_SHA256:
        problems.append("pre-activation producer semantic fingerprints are not exact")
    if len(active) != PREACTIVATION_REQUIRED_COUNT:
        problems.append("pre-activation inventory must contain 31 active contexts")
    if len(ledger.contexts) != PREACTIVATION_REQUIRED_COUNT:
        problems.append("pre-activation ruleset must contain 31 checks")
    if BASE_CONTROLLED_CONTEXT in ledger.contexts:
        problems.append("Base-controlled context must be absent before activation")
    if ledger.contexts != active:
        problems.append("ruleset ordered contexts do not match active inventory")
    return tuple(problems)


def _activation_state(
    inventory: ProducerInventory,
    ledger: RequiredCheckLedger,
    label: str,
) -> tuple[int | None, list[str]]:
    problems = [
        *(f"{label}: {item}" for item in inventory.problems),
        *(f"{label}: {item}" for item in ledger.problems),
    ]
    if inventory.target_branch != TARGET_BRANCH:
        problems.append(f"{label}: target branch is not {TARGET_BRANCH}")
    contexts = tuple(item.context for item in inventory.producers)
    if len(contexts) != PREACTIVATION_REQUIRED_COUNT + 1:
        problems.append(f"{label}: producer inventory must contain exactly 32 contexts")
    if not contexts or contexts[-1:] != (BASE_CONTROLLED_CONTEXT,):
        problems.append(f"{label}: Base-controlled context must be the final producer")
    if inventory.optional_contexts == {BASE_CONTROLLED_CONTEXT}:
        state = PREACTIVATION_REQUIRED_COUNT
        expected = contexts[:-1]
    elif not inventory.optional_contexts:
        state = PREACTIVATION_REQUIRED_COUNT + 1
        expected = contexts
    else:
        state = None
        expected = ()
        problems.append(
            f"{label}: optional contexts must be exactly the preactivation base context or empty"
        )
    if ledger.contexts != expected:
        problems.append(f"{label}: ruleset ordered contexts do not match activation state")
    return (None if problems else state), problems


def activation_transition_problems(
    base_inventory: ProducerInventory,
    base_ledger: RequiredCheckLedger,
    candidate_inventory: ProducerInventory,
    candidate_ledger: RequiredCheckLedger,
) -> tuple[str, ...]:
    """Allow only 31→31, 31→32, or 32→32 from trusted old-base policy."""
    base_state, problems = _activation_state(base_inventory, base_ledger, "base")
    candidate_state, candidate_problems = _activation_state(
        candidate_inventory, candidate_ledger, "candidate"
    )
    problems.extend(candidate_problems)
    if base_state == PREACTIVATION_REQUIRED_COUNT:
        problems.extend(preactivation_ruleset_problems(base_inventory, base_ledger))
    if base_inventory.producers != candidate_inventory.producers:
        problems.append("candidate producer inventory differs from trusted base inventory")
    if base_state == PREACTIVATION_REQUIRED_COUNT + 1 and candidate_state == PREACTIVATION_REQUIRED_COUNT:
        problems.append("32 to 31 governance downgrade is forbidden")
    elif (base_state, candidate_state) not in {
        (PREACTIVATION_REQUIRED_COUNT, PREACTIVATION_REQUIRED_COUNT),
        (PREACTIVATION_REQUIRED_COUNT, PREACTIVATION_REQUIRED_COUNT + 1),
        (PREACTIVATION_REQUIRED_COUNT + 1, PREACTIVATION_REQUIRED_COUNT + 1),
    }:
        problems.append("governance activation transition is not permitted")
    return tuple(dict.fromkeys(problems))


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


def _canonical_yaml_value(node: object) -> object:
    """Return style-independent canonical data from the immutable workflow AST."""
    if hasattr(node, "value") and hasattr(node, "style"):
        return {"scalar": node.value}
    items = getattr(node, "items", None)
    if not isinstance(items, tuple):
        raise ValueError("workflow semantic node is not an immutable YAML value")
    if all(
        isinstance(item, tuple)
        and len(item) == 2
        and isinstance(item[0], str)
        for item in items
    ):
        return {
            "mapping": {
                key: _canonical_yaml_value(value) for key, value in items
            }
        }
    return {"sequence": [_canonical_yaml_value(item) for item in items]}


def producer_semantic_sha256(workflow: object, occurrence: object) -> str:
    """Fingerprint one producer's global policy, transitive jobs, and matrix member."""
    root = getattr(getattr(workflow, "source", None), "root", None)
    root_items = getattr(root, "items", ())
    if not isinstance(root_items, tuple):
        raise ValueError("workflow root is unavailable for semantic fingerprint")
    global_policy = {
        key: _canonical_yaml_value(value)
        for key, value in root_items
        if key != "jobs"
    }
    problems: list[str] = []
    dependencies = _dependency_jobs(workflow, occurrence.job, problems)
    if problems:
        raise ValueError("; ".join(problems))
    binding = _matrix_binding(occurrence)
    body = {
        "schema": "garnet.required-context-producer-semantics/v1",
        "workflow": global_policy,
        "jobs": [
            {
                "job_id": job.job_id,
                "definition": _canonical_yaml_value(job.source),
            }
            for job in (*dependencies, occurrence.job)
        ],
        "matrix_binding": list(binding) if binding is not None else None,
    }
    return hashlib.sha256(
        json.dumps(
            body, ensure_ascii=False, separators=(",", ":"), sort_keys=True
        ).encode("utf-8")
    ).hexdigest()


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
        try:
            observed_semantic_sha256 = producer_semantic_sha256(
                workflow, occurrence
            )
        except (AttributeError, TypeError, ValueError) as exc:
            problems.append(
                f"{producer.workflow}:{producer.job} semantic fingerprint failed: {exc}"
            )
            observed_semantic_sha256 = ""
        if observed_semantic_sha256 != producer.semantic_sha256:
            problems.append(
                f"{producer.workflow}:{producer.job} semantic fingerprint mismatch"
            )
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
            binding = ProducerBinding(
                producer,
                workflow,
                event,
                occurrence,
                dependency_contexts,
                observed_semantic_sha256,
            )
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


def evaluate_checked_in_producer_policy(
    inventory: ProducerInventory,
    ledger: RequiredCheckLedger,
    projection: object,
) -> ProducerEvaluation:
    """Bind the immutable projection only after the checked-in 31-row policy agrees."""
    problems = preactivation_ruleset_problems(inventory, ledger)
    if problems:
        return ProducerEvaluation(problems=problems)
    result = evaluate_producer_availability(inventory, projection)
    if result.problems:
        return result
    contexts = tuple(item.producer.context for item in result.bindings)
    if contexts != ledger.contexts:
        return ProducerEvaluation(
            problems=("evaluated bindings do not match ruleset ordered contexts",)
        )
    return result
