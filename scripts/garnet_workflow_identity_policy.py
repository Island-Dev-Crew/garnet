#!/usr/bin/env python3
"""Pure supplied-data consistency check; live authority belongs to transport."""
from __future__ import annotations
import json
import unicodedata
from dataclasses import dataclass
SCHEMA = "garnet.github-required-context-identity/v1"
ACTIONS_APP_ID = 15368
ACTIONS_APP_SLUG = "github-actions"
MAX_BYTES = 256 * 1024
MAX_ROWS = 512
@dataclass(frozen=True)
class LiveWorkflow:
    id: int; name: str; path: str; state: str
@dataclass(frozen=True)
class LiveWorkflowRun:
    id: int; workflow_id: int; check_suite_id: int; event: str
@dataclass(frozen=True)
class LiveCheckRun:
    id: int; name: str; check_suite_id: int; app_id: int; app_slug: str
@dataclass(frozen=True)
class LiveIdentitySnapshot:
    workflows: tuple[LiveWorkflow, ...] = ()
    workflow_runs: tuple[LiveWorkflowRun, ...] = ()
    check_runs: tuple[LiveCheckRun, ...] = ()
    problems: tuple[str, ...] = ()
@dataclass(frozen=True)
class LiveIdentityBinding:
    producer: object; workflow: LiveWorkflow; run: LiveWorkflowRun; check: LiveCheckRun
@dataclass(frozen=True)
class LiveIdentityEvaluation:
    bindings: tuple[LiveIdentityBinding, ...] = ()
    problems: tuple[str, ...] = ()
def _no_duplicates(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key {key!r}")
        result[key] = value
    return result
def _record(value: object, keys: set[str], label: str,
            problems: list[str]) -> dict[str, object] | None:
    if not isinstance(value, dict) or set(value) != keys:
        problems.append(f"{label} keys are not exact")
        return None
    return value
def _positive(value: object, label: str, problems: list[str]) -> int | None:
    if type(value) is not int or not 0 < value < 2**63:
        problems.append(f"{label} must be a positive integer")
        return None
    return value
def _text(value: object, label: str, problems: list[str], limit: int = 256) -> str | None:
    valid = (
        isinstance(value, str) and 0 < len(value) <= limit
        and value == value.strip() and value.isprintable()
        and unicodedata.normalize("NFC", value) == value
        and not any(unicodedata.category(char) in {"Cc", "Cf"} for char in value)
    )
    if not valid:
        problems.append(f"{label} is not canonical text")
        return None
    return value
def _path(value: object, label: str, problems: list[str]) -> str | None:
    result = _text(value, label, problems, 512)
    if result is not None and (
        "\\" in result or any(part in {"", ".", ".."} for part in result.split("/"))
    ):
        problems.append(f"{label} is not a canonical POSIX path")
        return None
    return result
def parse_live_identity_snapshot(text: str) -> LiveIdentitySnapshot:
    """Parse one bounded, exact, duplicate-key-free normalized JSON snapshot."""
    try:
        if type(text) is not str or len(text.encode("utf-8")) > MAX_BYTES:
            raise ValueError("snapshot is not bounded UTF-8 text")
        raw = json.loads(
            text, object_pairs_hook=_no_duplicates,
            parse_constant=lambda value: (_ for _ in ()).throw(
                ValueError(f"invalid JSON constant {value}")
            ),
        )
    except (UnicodeError, json.JSONDecodeError, RecursionError, ValueError) as exc:
        return LiveIdentitySnapshot(problems=(f"cannot parse identity snapshot: {exc}",))
    problems: list[str] = []
    root = _record(raw, {"schema", "workflows", "workflow_runs", "check_runs"},
                   "snapshot", problems)
    if root is None:
        return LiveIdentitySnapshot(problems=tuple(problems))
    if root.get("schema") != SCHEMA:
        problems.append(f"snapshot schema must be {SCHEMA!r}")
    collections = [root.get(key) for key in ("workflows", "workflow_runs", "check_runs")]
    if any(not isinstance(rows, list) or len(rows) > MAX_ROWS for rows in collections):
        problems.append(f"snapshot row collections must be lists of at most {MAX_ROWS}")
        return LiveIdentitySnapshot(problems=tuple(problems))
    workflows: list[LiveWorkflow] = []
    for index, value in enumerate(collections[0]):
        row = _record(value, {"id", "name", "path", "state"},
                      f"workflows[{index}]", problems)
        if row is None:
            continue
        fields = (_positive(row["id"], "workflow id", problems),
                  _text(row["name"], "workflow name", problems),
                  _path(row["path"], "workflow path", problems),
                  _text(row["state"], "workflow state", problems))
        if all(item is not None for item in fields):
            workflows.append(LiveWorkflow(*fields))
    runs: list[LiveWorkflowRun] = []
    for index, value in enumerate(collections[1]):
        row = _record(value, {"id", "workflow_id", "check_suite_id", "event"},
                      f"workflow_runs[{index}]", problems)
        if row is None:
            continue
        fields = (_positive(row["id"], "workflow-run id", problems),
                  _positive(row["workflow_id"], "workflow-run workflow id", problems),
                  _positive(row["check_suite_id"], "workflow-run suite id", problems),
                  _text(row["event"], "workflow-run event", problems))
        if all(item is not None for item in fields):
            runs.append(LiveWorkflowRun(*fields))
    checks: list[LiveCheckRun] = []
    for index, value in enumerate(collections[2]):
        row = _record(value, {"id", "name", "check_suite_id", "app"},
                      f"check_runs[{index}]", problems)
        app = _record(row.get("app") if row else None, {"id", "slug"},
                      f"check_runs[{index}].app", problems) if row else None
        if row is None or app is None:
            continue
        fields = (_positive(row["id"], "check-run id", problems),
                  _text(row["name"], "check-run name", problems),
                  _positive(row["check_suite_id"], "check-run suite id", problems),
                  _positive(app["id"], "GitHub Actions App id", problems),
                  _text(app["slug"], "check-run App slug", problems))
        if all(item is not None for item in fields):
            checks.append(LiveCheckRun(*fields))
    unique = (
        ("workflow id", [item.id for item in workflows]),
        ("workflow path must identify exactly one live workflow", [item.path for item in workflows]),
        ("workflow-run id", [item.id for item in runs]),
        ("check suite must join exactly one workflow run", [item.check_suite_id for item in runs]),
        ("check-run id", [item.id for item in checks]),
    )
    for label, values in unique:
        if len(values) != len(set(values)):
            problems.append(f"duplicate {label}")
    return (LiveIdentitySnapshot(problems=tuple(problems)) if problems else
            LiveIdentitySnapshot(tuple(workflows), tuple(runs), tuple(checks), ()))
def evaluate_live_identity(policy: object,
                           snapshot: LiveIdentitySnapshot) -> LiveIdentityEvaluation:
    """Join the supplied observations to the ordered GOV-007 bindings, all-or-zero."""
    problems = [*getattr(policy, "problems", ()), *snapshot.problems]
    expected = tuple(getattr(policy, "bindings", ()))
    contexts = tuple(getattr(item.producer, "context", None) for item in expected)
    count = len(expected)
    valid_state = (
        count in {31, 32}
        and len(set(contexts)) == count
        and (
            (count == 31 and "Base-controlled trust policy" not in contexts)
            or (count == 32 and contexts[-1:] == ("Base-controlled trust policy",))
        )
    )
    if not valid_state:
        problems.append("live identity policy must contain exact 31 or activated 32 bindings")
    if problems:
        return LiveIdentityEvaluation(problems=tuple(problems))
    by_path = {item.path: item for item in snapshot.workflows}
    by_suite: dict[int, list[LiveWorkflowRun]] = {}
    by_workflow: dict[int, list[LiveWorkflowRun]] = {}
    by_name: dict[str, list[LiveCheckRun]] = {}
    for item in snapshot.workflow_runs:
        by_suite.setdefault(item.check_suite_id, []).append(item)
        by_workflow.setdefault(item.workflow_id, []).append(item)
    for item in snapshot.check_runs:
        by_name.setdefault(item.name, []).append(item)
    evidence: list[LiveIdentityBinding] = []
    for binding in expected:
        producer = binding.producer
        workflow = by_path.get(producer.workflow)
        checks = by_name.get(producer.context, [])
        if workflow is None:
            problems.append(f"{producer.workflow!r} must identify exactly one live workflow")
        elif workflow.name != binding.workflow.name.value:
            problems.append(f"{producer.workflow!r} live workflow name differs from checked-in YAML")
        elif workflow.state != "active":
            problems.append(f"{producer.workflow!r} live workflow must be active")
        selected = by_workflow.get(workflow.id, []) if workflow else []
        if workflow and len(selected) != 1:
            problems.append(f"{producer.workflow!r} must have exactly one selected workflow run")
        if len(checks) != 1:
            problems.append(f"{producer.context!r} must have exactly one live check")
        if workflow is None or len(checks) != 1:
            continue
        check = checks[0]
        joined = by_suite.get(check.check_suite_id, [])
        if len(joined) != 1:
            problems.append(f"{producer.context!r} suite must join exactly one workflow run")
            continue
        run = joined[0]
        if run.workflow_id != workflow.id:
            problems.append(f"{producer.context!r} did not run under its declared workflow")
        if len(selected) == 1 and run != selected[0]:
            problems.append(f"{producer.context!r} is outside the selected workflow run")
        if run.event != producer.event:
            problems.append(f"{producer.context!r} live run differs from its producer event")
        if (check.app_id, check.app_slug) != (ACTIONS_APP_ID, ACTIONS_APP_SLUG):
            problems.append(f"{producer.context!r} is not from the GitHub Actions App")
        evidence.append(LiveIdentityBinding(producer, workflow, run, check))
    return (LiveIdentityEvaluation(problems=tuple(problems)) if problems else
            LiveIdentityEvaluation(tuple(evidence), ()))
