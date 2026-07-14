#!/usr/bin/env python3
"""Immutable, typed canonical-YAML boundary for GitHub workflow policy."""
from __future__ import annotations

import importlib.util, re, sys, unicodedata
from dataclasses import dataclass
from importlib.metadata import PackageNotFoundError, distribution
from pathlib import Path


PY_YAML_VERSION = "6.0.3"
MAX_LINES, MAX_LINE_LENGTH = 2_000, 4_096
MAX_TOKENS, MAX_NODES, MAX_DEPTH = 50_000, 20_000, 64


def _load_yaml() -> tuple[object, Path]:
    if not sys.flags.isolated:
        raise RuntimeError("workflow YAML policy requires Python isolated mode (-I)")
    try:
        package = distribution("PyYAML")
    except PackageNotFoundError as exc:
        raise RuntimeError(f"PyYAML {PY_YAML_VERSION} is required") from exc
    candidates = [
        Path(package.locate_file(item)).resolve()
        for item in package.files or ()
        if str(item).replace("\\", "/") == "yaml/__init__.py"
    ]
    if package.version != PY_YAML_VERSION or len(candidates) != 1:
        raise RuntimeError(f"exact PyYAML {PY_YAML_VERSION} wheel is required")
    for name in [key for key in sys.modules if key == "yaml" or key.startswith("yaml.")]:
        del sys.modules[name]
    origin = candidates[0]
    spec = importlib.util.spec_from_file_location(
        "yaml", origin, submodule_search_locations=[str(origin.parent)]
    )
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load the pinned PyYAML distribution")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module, origin


yaml, YAML_ORIGIN = _load_yaml()
from yaml.nodes import MappingNode, ScalarNode, SequenceNode  # noqa: E402
from yaml.tokens import (  # noqa: E402
    AliasToken, AnchorToken, DirectiveToken, DocumentEndToken, DocumentStartToken, TagToken,
)
BAD_TOKENS = (AliasToken, AnchorToken, DirectiveToken, DocumentEndToken, DocumentStartToken, TagToken)


def _load_file_policy() -> object:
    path = Path(__file__).with_name("garnet_workflow_file_policy.py")
    spec = importlib.util.spec_from_file_location("_garnet_workflow_file_policy", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load workflow file policy from {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


file_policy = _load_file_policy()


class YamlPolicyError(ValueError):
    pass


@dataclass(frozen=True)
class WorkflowScalar:
    value: str
    style: str | None


@dataclass(frozen=True)
class WorkflowSequence:
    items: tuple[object, ...]


@dataclass(frozen=True)
class WorkflowMapping:
    items: tuple[tuple[str, object], ...]


@dataclass(frozen=True)
class WorkflowDocument:
    relative: str
    mode: str
    object_id: str
    root: WorkflowMapping


@dataclass(frozen=True)
class WorkflowYamlSnapshot:
    documents: tuple[WorkflowDocument, ...] = ()
    problems: tuple[str, ...] = ()


def _require(ok: bool, message: str) -> None:
    if not ok:
        raise YamlPolicyError(message)


def _canonical_key(value: str) -> bool:
    return (
        bool(value) and value == value.strip() and value.isprintable()
        and unicodedata.normalize("NFC", value) == value
        and not any(unicodedata.category(char) in {"Cc", "Cf"} for char in value)
    )


def _document(content: bytes) -> WorkflowMapping:
    _require(not content.startswith(b"\xef\xbb\xbf"), "UTF-8 BOM is not canonical")
    try:
        text = content.decode("utf-8", "strict")
    except UnicodeError as exc:
        raise YamlPolicyError("workflow is not valid UTF-8") from exc
    lines = text.split("\n")
    forbidden = any(char != "\n" and (ord(char) < 0x20 or ord(char) == 0x7F) for char in text)
    forbidden |= any(char in text for char in "\u0085\u2028\u2029")
    _require(bool(text) and "\r" not in text and not forbidden, "workflow transport is not canonical UTF-8/LF text")
    _require(len(lines) <= MAX_LINES and max(map(len, lines)) <= MAX_LINE_LENGTH, "workflow text exceeds structural limits")
    _require(max(len(line) - len(line.lstrip(" ")) for line in lines) <= MAX_DEPTH * 2, "workflow indentation exceeds structural limits")
    _require(text.count("[") + text.count("{") <= 512, "workflow flow syntax exceeds structural limits")
    try:
        for count, token in enumerate(yaml.scan(text, Loader=yaml.BaseLoader), 1):
            _require(count <= MAX_TOKENS, "workflow token count exceeds structural limits")
            _require(not isinstance(token, BAD_TOKENS), f"unsupported YAML token {type(token).__name__}")
        roots = list(yaml.compose_all(text, Loader=yaml.BaseLoader))
    except (yaml.YAMLError, RecursionError) as exc:
        raise YamlPolicyError(f"invalid YAML: {exc}") from exc
    _require(len(roots) == 1 and roots[0] is not None, "workflow must contain one implicit YAML document")
    nodes = 0

    def materialize(node: object, depth: int = 0) -> object:
        nonlocal nodes
        nodes += 1
        _require(nodes <= MAX_NODES and depth <= MAX_DEPTH, "workflow node graph exceeds structural limits")
        if isinstance(node, ScalarNode):
            return WorkflowScalar(node.value, node.style)
        if isinstance(node, SequenceNode):
            return WorkflowSequence(tuple(materialize(item, depth + 1) for item in node.value))
        _require(isinstance(node, MappingNode), "workflow contains an unsupported YAML node")
        _require(not node.flow_style or not node.value, "nonempty flow mappings are unsupported")
        result: list[tuple[str, object]] = []
        seen: set[str] = set()
        for key_node, value_node in node.value:
            _require(isinstance(key_node, ScalarNode) and key_node.style is None, "mapping keys must be plain scalars")
            key = key_node.value
            _require(_canonical_key(key), "mapping key is not canonical")
            _require(key != "<<" and key not in seen, f"duplicate or merged mapping key {key!r}")
            seen.add(key)
            result.append((key, materialize(value_node, depth + 1)))
        return WorkflowMapping(tuple(result))

    root = materialize(roots[0])
    _require(isinstance(root, WorkflowMapping), "workflow root must be a mapping")
    return root


def workflow_documents(root: Path, *, treeish: str | None = None) -> WorkflowYamlSnapshot:
    """Return one immutable typed AST per workflow, or zero ASTs on any anomaly."""
    records, boundary_problems = file_policy.workflow_snapshot(root, treeish=treeish)
    problems = list(boundary_problems)
    if not records and not problems:
        problems.append("workflow snapshot is unexpectedly empty")
    documents: list[WorkflowDocument] = []
    for record in records:
        try:
            documents.append(WorkflowDocument(
                record.relative, record.mode, record.object_id, _document(record.content)
            ))
        except YamlPolicyError as exc:
            problems.append(f"{record.relative}: {exc}")
    return WorkflowYamlSnapshot((), tuple(problems)) if problems else WorkflowYamlSnapshot(tuple(documents), ())
