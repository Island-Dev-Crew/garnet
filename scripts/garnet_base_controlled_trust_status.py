#!/usr/bin/env python3
"""Old-base Item 7 policy gate over inert candidate Git objects.

The candidate is data only: this module never checks it out, imports it, or
executes a candidate-owned file.  Policy modules are loaded from the verified
base checkout; candidate YAML/JSON is read with ``git cat-file`` and passed to
those base-owned parsers.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
import re
import subprocess
import sys
import tempfile
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import BinaryIO


ROOT = Path(__file__).resolve().parents[1]
SCHEMA = "garnet.base-controlled-trust-status/v1"
REPOSITORY = "Island-Dev-Crew/garnet"
WORKFLOW_PATH = ".github/workflows/base-controlled-trust.yml"
CONTEXT = "Base-controlled trust policy"
REVIEW_SCHEMA = "garnet.trust_kernel_review/v2"
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
DIGEST_RE = re.compile(r"^sha256:[0-9a-f]{64}$")
MAX_TOKEN_BYTES = 1024
MAX_BLOB_BYTES = 2 * 1024 * 1024
_CREDENTIAL_ENV = {
    "GH_TOKEN",
    "GITHUB_TOKEN",
    "GARNET_REVIEW_GITHUB_TOKEN",
    "GARNET_ADMIN_GITHUB_TOKEN",
}


def _load_sibling(name: str) -> object:
    path = Path(__file__).with_name(f"{name}.py")
    spec = importlib.util.spec_from_file_location(f"_{name}_base_controlled", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load base policy module {name}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


contract = _load_sibling("garnet_required_context_contract")


@dataclass(frozen=True)
class BaseControlledTrustStatus:
    schema: str
    ok: bool
    authority: str
    repository: str
    pull_request: int | None
    base_commit: str
    candidate_commit: str
    transition: str
    protected_workflow_sha256: str
    rolling_review_ok: bool
    candidate_policy_ok: bool
    credited_contexts: tuple[str, ...]
    problems: tuple[str, ...]


def _contexts(items: object, label: str, problems: list[str]) -> tuple[str, ...]:
    if type(items) is not tuple:
        problems.append(f"candidate semantic policy {label} must be an exact tuple")
        return ()
    contexts: list[str] = []
    for item in items:
        context = getattr(getattr(item, "producer", None), "context", None)
        if type(context) is not str or not context or context != context.strip():
            problems.append(f"candidate semantic policy {label} is malformed")
        else:
            contexts.append(context)
    if len(contexts) != len(set(contexts)):
        problems.append(f"candidate semantic policy {label} contains duplicates")
    return tuple(contexts)


def evaluate_base_controlled_trust(
    base_inventory: object,
    base_ledger: object,
    candidate_inventory: object,
    candidate_ledger: object,
    *,
    candidate_policy: object,
    rolling_review: object,
    repository: str,
    pull_request: int,
    base_commit: str,
    candidate_commit: str,
    base_workflow_sha256: str,
    candidate_workflow_sha256: str,
) -> BaseControlledTrustStatus:
    """Combine trusted transition, semantic, byte, and rolling-review proof."""
    problems: list[str] = []
    if repository != REPOSITORY:
        problems.append("repository identity is not exact")
    if type(pull_request) is not int or pull_request <= 0:
        problems.append("pull request number must be a positive integer")
    if type(base_commit) is not str or SHA_RE.fullmatch(base_commit) is None:
        problems.append("base commit is not one full lowercase SHA")
    if type(candidate_commit) is not str or SHA_RE.fullmatch(candidate_commit) is None:
        problems.append("candidate commit is not one full lowercase SHA")

    try:
        transition_problems = contract.activation_transition_problems(
            base_inventory, base_ledger, candidate_inventory, candidate_ledger
        )
    except Exception:
        transition_problems = ("governance activation transition evaluation failed",)
    problems.extend(transition_problems)
    base_count = len(getattr(base_ledger, "contexts", ()))
    candidate_contexts = tuple(getattr(candidate_ledger, "contexts", ()))
    candidate_count = len(candidate_contexts)
    transition = f"{base_count}-to-{candidate_count}"

    if (
        type(base_workflow_sha256) is not str
        or SHA256_RE.fullmatch(base_workflow_sha256) is None
        or type(candidate_workflow_sha256) is not str
        or SHA256_RE.fullmatch(candidate_workflow_sha256) is None
        or base_workflow_sha256 != candidate_workflow_sha256
    ):
        problems.append("protected workflow bytes differ from the trusted old base")

    policy_problems = getattr(candidate_policy, "problems", None)
    if type(policy_problems) is not tuple or policy_problems:
        problems.append("candidate semantic policy is not clean")
        if type(policy_problems) is tuple:
            problems.extend(
                f"candidate semantic policy: {item}"
                for item in policy_problems
                if type(item) is str
            )
    active = _contexts(getattr(candidate_policy, "bindings", None), "bindings", problems)
    prepared = _contexts(
        getattr(candidate_policy, "prepared_optional", None),
        "prepared bindings",
        problems,
    )
    inactive = getattr(candidate_policy, "inactive_optional", None)
    if type(inactive) is not tuple or inactive:
        problems.append("candidate semantic policy inactive evidence must be empty")
    if active != candidate_contexts:
        problems.append("candidate semantic policy bindings do not equal the candidate ledger")
    if candidate_count == 31 and prepared != (CONTEXT,):
        problems.append("31-context candidate must prepare exactly the Base-controlled context")
    if candidate_count == 32 and prepared:
        problems.append("32-context candidate cannot retain a prepared optional context")

    rolling_problems = getattr(rolling_review, "problems", None)
    rolling_ok = (
        getattr(rolling_review, "schema", None) == REVIEW_SCHEMA
        and getattr(rolling_review, "ok", None) is True
        and type(rolling_problems) is list
        and not rolling_problems
        and getattr(rolling_review, "base_commit", None) == base_commit
        and getattr(rolling_review, "head_commit", None) == candidate_commit
        and type(getattr(rolling_review, "reviewed_head", None)) is str
        and SHA_RE.fullmatch(getattr(rolling_review, "reviewed_head", "")) is not None
        and type(getattr(rolling_review, "reviewed_tree", None)) is str
        and SHA_RE.fullmatch(getattr(rolling_review, "reviewed_tree", "")) is not None
        and type(getattr(rolling_review, "content_digest", None)) is str
        and DIGEST_RE.fullmatch(getattr(rolling_review, "content_digest", "")) is not None
    )
    if not rolling_ok:
        problems.append("rolling review v2 does not bind the exact base/candidate boundary")
        if type(rolling_problems) is list:
            problems.extend(
                f"rolling review: {item}"
                for item in rolling_problems
                if type(item) is str
            )

    unique = tuple(dict.fromkeys(problems))
    return BaseControlledTrustStatus(
        SCHEMA,
        not unique,
        "trusted-old-base",
        REPOSITORY,
        pull_request if type(pull_request) is int and pull_request > 0 else None,
        base_commit if type(base_commit) is str and SHA_RE.fullmatch(base_commit) else "",
        candidate_commit
        if type(candidate_commit) is str and SHA_RE.fullmatch(candidate_commit)
        else "",
        transition,
        candidate_workflow_sha256
        if type(candidate_workflow_sha256) is str
        and SHA256_RE.fullmatch(candidate_workflow_sha256)
        else "",
        rolling_ok,
        not policy_problems,
        candidate_contexts if not unique else (),
        unique,
    )


def read_explicit_review_token(
    stream: BinaryIO, *, enabled: bool
) -> tuple[str, list[str]]:
    """Read only a caller-selected stdin credential; ambient tokens are ignored."""
    if not enabled:
        return "", ["explicit review token stdin input is required"]
    raw = stream.read(MAX_TOKEN_BYTES + 2)
    if len(raw) > MAX_TOKEN_BYTES + 1:
        return "", ["explicit review token exceeds its input bound"]
    raw = raw.removesuffix(b"\n").removesuffix(b"\r")
    try:
        token = raw.decode("ascii", errors="strict")
    except UnicodeDecodeError:
        return "", ["explicit review token is malformed"]
    if not token or len(token) > MAX_TOKEN_BYTES or any(
        ord(char) < 33 or ord(char) > 126 for char in token
    ):
        return "", ["explicit review token is malformed"]
    return token, []


def render_json(value: BaseControlledTrustStatus) -> str:
    return json.dumps(asdict(value), ensure_ascii=False, indent=2, sort_keys=True)


def _git(repo: Path, *args: str) -> tuple[int, bytes]:
    env = {key: value for key, value in os.environ.items() if key not in _CREDENTIAL_ENV}
    env.update({"GIT_NO_REPLACE_OBJECTS": "1", "GIT_TERMINAL_PROMPT": "0"})
    try:
        result = subprocess.run(
            ["git", "--no-replace-objects", *args],
            cwd=repo,
            capture_output=True,
            check=False,
            timeout=30,
            env=env,
        )
    except (OSError, subprocess.TimeoutExpired):
        return 124, b""
    return result.returncode, result.stdout


def _read_git_blob(repo: Path, commit: str, path: str) -> tuple[bytes | None, list[str]]:
    code, tree = _git(repo, "ls-tree", "-z", "--full-tree", commit, "--", path)
    if code != 0 or not tree.endswith(b"\0"):
        return None, [f"candidate Git object enumeration failed for {path}"]
    rows = tree[:-1].split(b"\0") if tree[:-1] else []
    if len(rows) != 1:
        return None, [f"candidate must contain exactly one Git object for {path}"]
    try:
        metadata, raw_path = rows[0].split(b"\t", 1)
        mode, kind, oid = metadata.decode("ascii").split(" ")
        decoded = raw_path.decode("utf-8")
    except (ValueError, UnicodeError):
        return None, [f"candidate Git object identity is malformed for {path}"]
    if (mode, kind, decoded) != ("100644", "blob", path) or SHA_RE.fullmatch(oid) is None:
        return None, [f"candidate Git object identity is not exact for {path}"]
    code, size_raw = _git(repo, "cat-file", "-s", oid)
    try:
        size = int(size_raw.strip())
    except ValueError:
        size = -1
    if code != 0 or not 0 < size <= MAX_BLOB_BYTES:
        return None, [f"candidate Git blob size is invalid for {path}"]
    code, payload = _git(repo, "cat-file", "blob", oid)
    if code != 0 or len(payload) != size:
        return None, [f"candidate Git blob read failed for {path}"]
    return payload, []


def _candidate_workflow_snapshot(
    repo: Path, commit: str, *, yaml_policy: object
) -> tuple[object | None, list[str]]:
    code, raw = _git(
        repo,
        "ls-tree",
        "-r",
        "-z",
        "--full-tree",
        commit,
        "--",
        ".github/workflows",
    )
    if code != 0 or (raw and not raw.endswith(b"\0")):
        return None, ["candidate workflow Git enumeration failed"]
    documents: list[object] = []
    problems: list[str] = []
    seen: set[str] = set()
    for row in raw[:-1].split(b"\0") if raw else ():
        try:
            metadata, raw_path = row.split(b"\t", 1)
            mode, kind, oid = metadata.decode("ascii").split(" ")
            path = raw_path.decode("utf-8")
        except (ValueError, UnicodeError):
            problems.append("candidate workflow Git identity is malformed")
            continue
        if path in seen:
            problems.append("candidate workflow Git enumeration contains a duplicate path")
            continue
        seen.add(path)
        if not re.fullmatch(r"\.github/workflows/[a-z0-9_.-]+\.(?:yml|yaml)", path):
            problems.append(f"candidate workflow path is not canonical: {path}")
            continue
        if mode != "100644" or kind != "blob" or SHA_RE.fullmatch(oid) is None:
            problems.append(f"candidate workflow object identity is not exact: {path}")
            continue
        payload, blob_problems = _read_git_blob(repo, commit, path)
        problems.extend(blob_problems)
        if payload is None:
            continue
        try:
            documents.append(
                yaml_policy.WorkflowDocument(
                    path, mode, oid, yaml_policy._document(payload)
                )
            )
        except Exception as exc:
            problems.append(f"{path}: candidate workflow parsing failed: {exc}")
    if not documents and not problems:
        problems.append("candidate workflow enumeration is empty")
    if problems:
        return None, problems
    return yaml_policy.WorkflowYamlSnapshot(tuple(documents), ()), []


def _load_candidate_contracts(
    repo: Path, commit: str
) -> tuple[object, object, bytes | None, list[str]]:
    inventory_bytes, inventory_problems = _read_git_blob(
        repo, commit, contract.INVENTORY_PATH
    )
    ruleset_bytes, ruleset_problems = _read_git_blob(repo, commit, contract.RULESET_PATH)
    workflow_bytes, workflow_problems = _read_git_blob(repo, commit, WORKFLOW_PATH)
    problems = [*inventory_problems, *ruleset_problems, *workflow_problems]
    if problems or inventory_bytes is None or ruleset_bytes is None:
        return contract.ProducerInventory(problems=problems), contract.RequiredCheckLedger(problems=tuple(problems)), workflow_bytes, problems
    with tempfile.TemporaryDirectory(prefix="garnet-candidate-policy-") as raw_directory:
        directory = Path(raw_directory)
        inventory_path = directory / "inventory.json"
        ruleset_path = directory / "ruleset.json"
        inventory_path.write_bytes(inventory_bytes)
        ruleset_path.write_bytes(ruleset_bytes)
        inventory = contract.load_inventory(inventory_path)
        ledger = contract.load_required_check_ledger(ruleset_path)
    return inventory, ledger, workflow_bytes, problems


def _rolling_review_adapter(
    repo: Path,
    *,
    base_commit: str,
    candidate_commit: str,
    repository: str,
    pull_request: int,
    token: str,
) -> object:
    try:
        review = _load_sibling("garnet_trust_kernel_review_status")
        transport_module = _load_sibling("garnet_github_governance_transport")
        transport = transport_module.GitHubGovernanceTransport(repository, token)
        return review.read_status(
            base=base_commit,
            head=candidate_commit,
            root=repo,
            github_transport=transport,
            repository=repository,
            pull_request=pull_request,
        )
    except Exception as exc:
        return type(
            "RollingDependencyFailure",
            (),
            {
                "schema": REVIEW_SCHEMA,
                "ok": False,
                "base_commit": base_commit,
                "head_commit": candidate_commit,
                "reviewed_head": None,
                "reviewed_tree": None,
                "content_digest": None,
                "problems": [f"Item 2 rolling-review adapter dependency failed: {exc}"],
            },
        )()


def evaluate_git_candidate(
    *,
    root: Path,
    candidate_repo: Path,
    repository: str,
    pull_request: int,
    base_commit: str,
    candidate_commit: str,
    token: str,
) -> BaseControlledTrustStatus:
    code, resolved_base = _git(root, "rev-parse", "--verify", "HEAD^{commit}")
    if code != 0 or resolved_base.decode("ascii", errors="ignore").strip() != base_commit:
        return evaluate_base_controlled_trust(
            contract.ProducerInventory(problems=["base checkout does not equal base SHA"]),
            contract.RequiredCheckLedger(problems=("base checkout mismatch",)),
            contract.ProducerInventory(problems=["candidate not evaluated"]),
            contract.RequiredCheckLedger(problems=("candidate not evaluated",)),
            candidate_policy=type("Policy", (), {"problems": ("candidate not evaluated",), "bindings": (), "prepared_optional": (), "inactive_optional": ()})(),
            rolling_review=type("Review", (), {"schema": REVIEW_SCHEMA, "ok": False, "problems": ["candidate not evaluated"]})(),
            repository=repository,
            pull_request=pull_request,
            base_commit=base_commit,
            candidate_commit=candidate_commit,
            base_workflow_sha256="",
            candidate_workflow_sha256="",
        )
    base_inventory = contract.load_inventory(root / contract.INVENTORY_PATH)
    base_ledger = contract.load_required_check_ledger(root / contract.RULESET_PATH)
    base_workflow = (root / WORKFLOW_PATH).read_bytes()
    candidate_inventory, candidate_ledger, candidate_workflow, object_problems = (
        _load_candidate_contracts(candidate_repo, candidate_commit)
    )
    try:
        schema = _load_sibling("garnet_workflow_schema_policy")
        snapshot, snapshot_problems = _candidate_workflow_snapshot(
            candidate_repo,
            candidate_commit,
            yaml_policy=schema.yaml_policy,
        )
        projection = schema.project_snapshot(snapshot) if snapshot is not None else None
        candidate_policy = (
            contract.evaluate_producer_availability(candidate_inventory, projection)
            if projection is not None
            else type("Policy", (), {"problems": tuple(snapshot_problems), "bindings": (), "prepared_optional": (), "inactive_optional": ()})()
        )
    except Exception as exc:
        candidate_policy = type("Policy", (), {"problems": (f"base schema adapter failed: {exc}",), "bindings": (), "prepared_optional": (), "inactive_optional": ()})()
    if object_problems:
        candidate_policy = type("Policy", (), {"problems": tuple(object_problems), "bindings": (), "prepared_optional": (), "inactive_optional": ()})()
    rolling = _rolling_review_adapter(
        candidate_repo,
        base_commit=base_commit,
        candidate_commit=candidate_commit,
        repository=repository,
        pull_request=pull_request,
        token=token,
    )
    return evaluate_base_controlled_trust(
        base_inventory,
        base_ledger,
        candidate_inventory,
        candidate_ledger,
        candidate_policy=candidate_policy,
        rolling_review=rolling,
        repository=repository,
        pull_request=pull_request,
        base_commit=base_commit,
        candidate_commit=candidate_commit,
        base_workflow_sha256=hashlib.sha256(base_workflow).hexdigest(),
        candidate_workflow_sha256=(
            hashlib.sha256(candidate_workflow).hexdigest()
            if candidate_workflow is not None
            else ""
        ),
    )


def main(argv: list[str] | None = None, *, root: Path = ROOT) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate-git-dir", type=Path, required=True)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--pull-request", type=int, required=True)
    parser.add_argument("--base-sha", required=True)
    parser.add_argument("--candidate-sha", required=True)
    parser.add_argument("--review-token-stdin", action="store_true")
    parser.add_argument("--gate", action="store_true")
    args = parser.parse_args(argv)
    token, token_problems = read_explicit_review_token(
        sys.stdin.buffer, enabled=args.review_token_stdin
    )
    if token_problems:
        failed = BaseControlledTrustStatus(
            SCHEMA,
            False,
            "trusted-old-base",
            REPOSITORY,
            args.pull_request,
            "",
            "",
            "unknown",
            "",
            False,
            False,
            (),
            tuple(token_problems),
        )
        print(render_json(failed))
        return 1 if args.gate else 0
    try:
        result = evaluate_git_candidate(
            root=root.resolve(),
            candidate_repo=args.candidate_git_dir.resolve(),
            repository=args.repository,
            pull_request=args.pull_request,
            base_commit=args.base_sha,
            candidate_commit=args.candidate_sha,
            token=token,
        )
    finally:
        token = ""
    print(render_json(result))
    return 1 if args.gate and not result.ok else 0


if __name__ == "__main__":
    raise SystemExit(main())
