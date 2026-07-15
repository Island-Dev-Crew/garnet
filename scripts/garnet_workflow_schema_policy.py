#!/usr/bin/env python3
"""Fail-closed producer projection over Garnet's immutable workflow YAML AST."""
from __future__ import annotations
import importlib.util, re, sys, unicodedata
from dataclasses import dataclass
from pathlib import Path
ID_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_-]*$")
PLAIN_TYPED_RE = re.compile(r"(?i:true|false|yes|no|on|off|null|~|[-+]?(?:(?:[0-9][0-9_]*)(?:\.[0-9_]*)?(?:e[-+]?[0-9]+)?|0x[0-9a-f_]+|0o[0-7_]+|\.[0-9_]+|\.inf|\.nan))")
MATRIX_EXPR_RE = re.compile(r"\$\{\{\s*matrix\.([A-Za-z_][A-Za-z0-9_-]*)\s*\}\}")
TOP_KEYS = {"name", "on", "permissions", "env", "concurrency", "jobs"}
EVENTS = {"push", "pull_request", "pull_request_target", "schedule", "workflow_dispatch"}
PR_FILTERS = {"branches", "branches-ignore", "paths", "paths-ignore", "types"}
PUSH_FILTERS = (PR_FILTERS - {"types"}) | {"tags", "tags-ignore"}
JOB_KEYS = {"name", "permissions", "needs", "if", "runs-on", "env", "timeout-minutes",
            "continue-on-error", "container", "strategy", "steps", "uses"}
STEP_KEYS = {"name", "if", "uses", "run", "shell", "with", "env", "continue-on-error",
             "timeout-minutes", "working-directory"}
PERMISSION_SCOPES = {"actions", "artifact-metadata", "attestations", "checks", "code-quality", "contents",
                     "deployments", "discussions", "id-token", "issues", "models", "packages", "pages",
                     "pull-requests", "security-events", "statuses", "vulnerability-alerts"}
READ_ONLY_SCOPES = {"models", "vulnerability-alerts"}
WRITE_ONLY_SCOPES = {"id-token"}
def _load_yaml_policy() -> object:
    path = Path(__file__).with_name("garnet_workflow_yaml_policy.py")
    spec = importlib.util.spec_from_file_location("_garnet_workflow_yaml_policy", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load workflow YAML policy from {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module
yaml_policy = _load_yaml_policy()
class SchemaPolicyError(ValueError):
    pass
@dataclass(frozen=True)
class EventProfile:
    name: str; source: object
    filters: tuple[tuple[str, tuple[object, ...]], ...] = ()
@dataclass(frozen=True)
class MatrixProfile:
    axis: str; values: tuple[object, ...]
@dataclass(frozen=True)
class JobProfile:
    job_id: str; source: object; name: object | None; runs_on: object
    condition: object | None; continue_on_error: object | None
    needs: tuple[object, ...]; matrix: MatrixProfile | None; steps: tuple[object, ...]
@dataclass(frozen=True)
class ContextOccurrence:
    context: str; source: object; job: JobProfile; template: object | None
    binding: tuple[str, object] | None = None
@dataclass(frozen=True)
class WorkflowProfile:
    source: object; name: object; events: tuple[EventProfile, ...]; jobs: tuple[JobProfile, ...]
    contexts: tuple[ContextOccurrence, ...]
@dataclass(frozen=True)
class WorkflowProjection:
    workflows: tuple[WorkflowProfile, ...] = ()
    problems: tuple[str, ...] = ()
def _require(ok: bool, message: str) -> None:
    if not ok:
        raise SchemaPolicyError(message)
def _mapping(node: object, label: str) -> dict[str, object]:
    _require(isinstance(node, yaml_policy.WorkflowMapping), f"{label} must be a mapping")
    return dict(node.items)
def _scalar(node: object, label: str) -> object:
    _require(isinstance(node, yaml_policy.WorkflowScalar), f"{label} must be a scalar")
    return node
def _value(node: object, label: str) -> object:
    scalar = _scalar(node, label)
    remainder = re.sub(r"\$\{\{.*?\}\}", "", scalar.value, flags=re.DOTALL)
    _require("${{" not in remainder and "}}" not in remainder, f"{label} has an incomplete expression")
    return scalar
def _text(node: object, label: str) -> object:
    scalar = _value(node, label)
    value, style = scalar.value, scalar.style
    _require(style in {None, "'", '"'}, f"{label} must be a plain or quoted string")
    _require(bool(value) and value == value.strip() and "\n" not in value, f"{label} is not canonical text")
    _require(unicodedata.normalize("NFC", value) == value and value.isprintable(), f"{label} is not canonical text")
    _require(not (style is None and PLAIN_TYPED_RE.fullmatch(value)), f"{label} has a non-string plain scalar")
    return scalar
def _boolean(node: object, label: str) -> object:
    scalar = _scalar(node, label)
    _require(scalar.style is None and scalar.value in {"true", "false"}, f"{label} must be an unquoted boolean")
    return scalar
def _integer(node: object, label: str, maximum: int = 360) -> object:
    scalar = _scalar(node, label)
    valid = scalar.style is None and scalar.value.isdigit() and 0 < int(scalar.value) <= maximum
    _require(valid, f"{label} must be an unquoted positive integer at most {maximum}")
    return scalar
def _scalar_mapping(node: object, label: str) -> dict[str, object]:
    values = _mapping(node, label)
    for key, value in values.items():
        _value(value, f"{label}.{key}")
    return values
def _command(node: object, label: str) -> object:
    scalar = _value(node, label)
    _require(scalar.style in {None, "'", '"', "|", ">"} and bool(scalar.value.strip()), f"{label} must be command text")
    _require(not (scalar.style is None and PLAIN_TYPED_RE.fullmatch(scalar.value)), f"{label} has a non-string plain scalar")
    return scalar
def _permissions(node: object, label: str) -> None:
    if isinstance(node, yaml_policy.WorkflowScalar):
        _require(_text(node, label).value == "read-all", f"{label} scalar must be read-all")
        return
    values = _mapping(node, label)
    _require(not (set(values) - PERMISSION_SCOPES), f"{label} contains an unknown scope")
    for scope, raw in values.items():
        value = _text(raw, f"{label}.{scope}").value
        allowed = {"none", "read"} if scope in READ_ONLY_SCOPES else (
            {"none", "write"} if scope in WRITE_ONLY_SCOPES else {"none", "read", "write"}
        )
        _require(value in allowed, f"{label}.{scope} has an invalid level")
        _require(not (scope in {"checks", "statuses"} and value == "write"),
                 f"{label}.{scope} can forge required results")
def _text_sequence(node: object, label: str) -> tuple[object, ...]:
    _require(isinstance(node, yaml_policy.WorkflowSequence), f"{label} must be a sequence")
    values = tuple(_text(item, label) for item in node.items)
    _require(0 < len(values) <= 64, f"{label} must be a bounded non-empty sequence")
    _require(len({item.value for item in values}) == len(values), f"{label} contains duplicates")
    return values
def _filtered_event(name: str, node: object) -> EventProfile:
    values = _mapping(node, name)
    allowed = PUSH_FILTERS if name == "push" else PR_FILTERS
    _require(not (set(values) - allowed), f"{name} has unsupported keys")
    for left, right in (("branches", "branches-ignore"), ("paths", "paths-ignore"), ("tags", "tags-ignore")):
        _require(not ({left, right} <= set(values)), f"{name} cannot combine {left} and {right}")
    filters = tuple((key, _text_sequence(value, f"{name}.{key}")) for key, value in values.items())
    return EventProfile(name, node, filters)
def _event(name: str, node: object) -> EventProfile:
    _require(name in EVENTS, f"unsupported workflow event {name!r}")
    if name in {"push", "pull_request", "pull_request_target"}:
        return _filtered_event(name, node)
    if name == "schedule":
        _require(isinstance(node, yaml_policy.WorkflowSequence) and node.items,
                 "schedule must be a non-empty sequence")
        for index, item in enumerate(node.items):
            row = _mapping(item, f"schedule[{index}]")
            _require(set(row) == {"cron"}, f"schedule[{index}] keys are not exact")
            cron = _text(row["cron"], f"schedule[{index}].cron").value
            _require(bool(re.fullmatch(r"(?:[0-5]?\d) (?:[01]?\d|2[0-3]) \* \* (?:\*|[0-6])", cron)), f"schedule[{index}].cron is outside the fixed-time profile")
        return EventProfile(name, node)
    dispatch = _mapping(node, name)
    _require(not (set(dispatch) - {"inputs"}), "workflow_dispatch has unsupported keys")
    if "inputs" in dispatch:
        inputs = _mapping(dispatch["inputs"], "workflow_dispatch.inputs")
        _require(len(inputs) <= 10, "workflow_dispatch has too many inputs")
        for input_id, raw in inputs.items():
            _require(bool(ID_RE.fullmatch(input_id)), f"workflow_dispatch input id {input_id!r} is invalid")
            label, row = f"workflow_dispatch.inputs.{input_id}", _mapping(raw, f"workflow_dispatch.inputs.{input_id}")
            _require(not (set(row) - {"description", "required", "default"}),
                     f"{label} has unsupported keys")
            if "description" in row:
                _text(row["description"], f"{label}.description")
            if "required" in row:
                _boolean(row["required"], f"{label}.required")
            if "default" in row:
                _value(row["default"], f"{label}.default")
    return EventProfile(name, node)
def _needs(node: object, label: str) -> tuple[object, ...]:
    values = _text_sequence(node, label) if isinstance(node, yaml_policy.WorkflowSequence) else (_text(node, label),)
    _require(all(ID_RE.fullmatch(item.value) for item in values), f"{label} contains an invalid job id")
    return values
def _matrix(node: object, label: str) -> MatrixProfile | None:
    if node is None:
        return None
    strategy = _mapping(node, label)
    _require("matrix" in strategy and not (set(strategy) - {"matrix", "fail-fast", "max-parallel"}),
             f"{label} keys are not exact")
    if "fail-fast" in strategy:
        _boolean(strategy["fail-fast"], f"{label}.fail-fast")
    if "max-parallel" in strategy:
        _integer(strategy["max-parallel"], f"{label}.max-parallel", 256)
    axes = _mapping(strategy["matrix"], f"{label}.matrix")
    _require(len(axes) == 1, f"{label}.matrix must have exactly one static axis")
    axis, raw_values = next(iter(axes.items()))
    _require(axis not in {"include", "exclude"} and bool(ID_RE.fullmatch(axis)), f"{label}.matrix axis is invalid")
    values = _text_sequence(raw_values, f"{label}.matrix.{axis}")
    _require(all("${{" not in value.value for value in values), f"{label}.matrix values must be static")
    return MatrixProfile(axis, values)
def _step(node: object, label: str) -> object:
    values = _mapping(node, label)
    _require(not (set(values) - STEP_KEYS), f"{label} has unsupported keys")
    _require(("run" in values) != ("uses" in values) and not ("run" in values and "with" in values) and not ("uses" in values and ({"shell", "working-directory"} & set(values))), f"{label} has an invalid run/uses field combination")
    for key in ("name", "if", "shell", "working-directory"):
        if key in values:
            _text(values[key], f"{label}.{key}")
    if "run" in values:
        _command(values["run"], f"{label}.run")
    if "uses" in values:
        use = _text(values["uses"], f"{label}.uses")
        _require("${{" not in use.value and bool(re.fullmatch(r"[A-Za-z0-9_.-]+(?:/[A-Za-z0-9_.-]+)+@[A-Za-z0-9_.-]+", use.value)), f"{label}.uses must be a static external action reference")
    for key in ("with", "env"):
        if key in values:
            _scalar_mapping(values[key], f"{label}.{key}")
    if "continue-on-error" in values:
        soft = _boolean(values["continue-on-error"], f"{label}.continue-on-error")
        _require(soft.value == "false", f"{label}.continue-on-error cannot enable soft failure")
    if "timeout-minutes" in values:
        _integer(values["timeout-minutes"], f"{label}.timeout-minutes")
    return node
def _job(job_id: str, node: object) -> JobProfile:
    _require(bool(ID_RE.fullmatch(job_id)), f"job id {job_id!r} is invalid")
    values = _mapping(node, f"job {job_id}")
    _require(not (set(values) - JOB_KEYS), f"job {job_id} has unsupported keys")
    _require("uses" not in values, f"job {job_id} reusable workflow calls are unsupported")
    _require("runs-on" in values and "steps" in values, f"job {job_id} must contain runs-on and steps")
    if "permissions" in values:
        _permissions(values["permissions"], f"job {job_id}.permissions")
    if "env" in values:
        _scalar_mapping(values["env"], f"job {job_id}.env")
    if "timeout-minutes" in values:
        _integer(values["timeout-minutes"], f"job {job_id}.timeout-minutes")
    if "container" in values:
        _text(values["container"], f"job {job_id}.container")
    raw_steps = values["steps"]
    _require(isinstance(raw_steps, yaml_policy.WorkflowSequence) and raw_steps.items,
             f"job {job_id}.steps must be a non-empty sequence")
    steps = tuple(_step(step, f"job {job_id}.steps[{index}]") for index, step in enumerate(raw_steps.items))
    name = _text(values["name"], f"job {job_id}.name") if "name" in values else None
    condition = _text(values["if"], f"job {job_id}.if") if "if" in values else None
    continued = _boolean(values["continue-on-error"], f"job {job_id}.continue-on-error") if "continue-on-error" in values else None
    _require(continued is None or continued.value == "false", f"job {job_id}.continue-on-error cannot enable soft failure")
    needs = _needs(values["needs"], f"job {job_id}.needs") if "needs" in values else ()
    matrix = _matrix(values.get("strategy"), f"job {job_id}.strategy")
    runner = _text(values["runs-on"], f"job {job_id}.runs-on")
    if "${{" in runner.value:
        expected = f"${{{{ matrix.{matrix.axis} }}}}" if matrix else ""
        _require(runner.value == expected, f"job {job_id}.runs-on has an unsupported expression")
    return JobProfile(job_id, node, name, runner, condition, continued, needs, matrix, steps)
def _contexts(source: object, job: JobProfile) -> tuple[ContextOccurrence, ...]:
    template = job.name
    if job.matrix is None:
        _require(template is None or "${{" not in template.value, f"job {job.job_id} has a dynamic context name")
        context = template.value if template else job.job_id
        _require(len(context) <= 256, f"job {job.job_id} context exceeds the policy limit")
        return (ContextOccurrence(context, source, job, template),)
    if template:
        matches = list(MATRIX_EXPR_RE.finditer(template.value))
        valid = len(matches) == 1 and template.value.count("${{") == 1 and matches[0].group(1) == job.matrix.axis
        _require(valid, f"job {job.job_id} matrix name must contain one supported placeholder")
    result: list[ContextOccurrence] = []
    for member in job.matrix.values:
        context = (MATRIX_EXPR_RE.sub(lambda _: member.value, template.value, count=1)
                   if template else f"{job.job_id} ({member.value})")
        _require("${{" not in context and "}}" not in context, f"job {job.job_id} context remains dynamic")
        _require(len(context) <= 256, f"job {job.job_id} context exceeds the policy limit")
        result.append(ContextOccurrence(context, source, job, template, (job.matrix.axis, member)))
    return tuple(result)
def project_snapshot(snapshot: object) -> WorkflowProjection:
    problems, workflows = list(snapshot.problems), []
    documents = snapshot.documents
    if not documents and not problems:
        problems.append("workflow YAML snapshot is unexpectedly empty")
    for document in documents:
        try:
            root = _mapping(document.root, "workflow root")
            _require({"name", "on", "jobs"} <= set(root) and not (set(root) - TOP_KEYS),
                     "workflow top-level keys are not accepted")
            name = _text(root["name"], "workflow name")
            if "permissions" in root:
                _permissions(root["permissions"], "workflow permissions")
            if "env" in root:
                _scalar_mapping(root["env"], "workflow env")
            if "concurrency" in root:
                concurrency = _mapping(root["concurrency"], "workflow concurrency")
                _require("group" in concurrency and not (set(concurrency) - {"group", "cancel-in-progress"}),
                         "workflow concurrency keys are not exact")
                _text(concurrency["group"], "workflow concurrency.group")
                if "cancel-in-progress" in concurrency:
                    _boolean(concurrency["cancel-in-progress"], "workflow concurrency.cancel-in-progress")
            event_map = _mapping(root["on"], "workflow on")
            _require(bool(event_map), "workflow on must not be empty")
            events = tuple(_event(name, value) for name, value in event_map.items())
            job_map = _mapping(root["jobs"], "workflow jobs")
            _require(bool(job_map), "workflow jobs must not be empty")
            jobs = tuple(_job(job_id, value) for job_id, value in job_map.items())
            known = {job.job_id for job in jobs}
            for job in jobs:
                refs = {item.value for item in job.needs}
                _require(job.job_id not in refs and refs <= known, f"job {job.job_id} has an invalid needs dependency")
            pending = {job.job_id: {item.value for item in job.needs} for job in jobs}
            while pending:
                ready = {job_id for job_id, dependencies in pending.items() if not dependencies}
                _require(bool(ready), "job needs graph contains a cycle")
                pending = {job_id: deps - ready for job_id, deps in pending.items() if job_id not in ready}
            contexts = tuple(item for job in jobs for item in _contexts(document, job))
            workflows.append(WorkflowProfile(document, name, events, jobs, contexts))
        except SchemaPolicyError as exc:
            problems.append(f"{document.relative}: {exc}")
    return WorkflowProjection((), tuple(problems)) if problems else WorkflowProjection(tuple(workflows), ())
def workflow_projection(root: Path, *, treeish: str | None = None) -> WorkflowProjection:
    """Project producer-relevant schema without reopening immutable workflow blobs."""
    return project_snapshot(yaml_policy.workflow_documents(root, treeish=treeish))
