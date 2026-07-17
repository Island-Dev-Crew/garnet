#!/usr/bin/env python3
"""Validate the preparation-only human 31-to-32 activation package."""
from __future__ import annotations

import argparse
import copy
import hashlib
import importlib.util
import json
import os
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
SCHEMA = "garnet.governance-activation-ceremony/v1"
CEREMONY_PATH = ".github/rulesets/governance-activation-ceremony.json"
WORKFLOW_PATH = ".github/workflows/base-controlled-trust.yml"
EVIDENCE_DESTINATION = "ops/lane1/evidence/70-item7-ceremony.md"
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
MAX_DOCUMENT_BYTES = 64 * 1024
JON_ONLY_ACTIONS = (
    "provision GARNET_ADMIN_GITHUB_TOKEN",
    "merge the Lane 1 bootstrap pull request while Base-controlled trust policy is not required",
    "confirm the base-controlled workflow is active on main and open a separate activation/terminus pull request from that base with the bootstrap squash-durable landed marker registered",
    "activate required context 31 to 32 on ruleset 18936562 while the activation/terminus pull request is open",
    "read back ruleset 18936562 and verify Base-controlled trust policy is required with bypass_actors empty",
    "rerun the authenticated governance and base-controlled gates on the exact activation/terminus head",
    "merge the activation/terminus pull request",
    "merge the bounded post-squash Lane 1 closeout pull request that registers the terminus landed marker without adding a GOV number",
)


def _load_sibling(name: str) -> object:
    path = Path(__file__).with_name(f"{name}.py")
    spec = importlib.util.spec_from_file_location(f"_{name}_ceremony", path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


transport = _load_sibling("garnet_github_governance_transport")
governance_gate = _load_sibling("garnet_github_governance_gate")
ACCEPTANCE_COMMANDS = (
    {
        "command": "python3 -I scripts/test_garnet_base_controlled_trust_status.py",
        "expected": "PASS",
    },
    {
        "command": "python3 -I scripts/test_garnet_governance_activation_ceremony.py",
        "expected": "PASS",
    },
    {
        "command": "python3 -I scripts/garnet_governance_activation_ceremony.py --gate",
        "expected": "PASS preparation only",
    },
    {
        "command": "python3 -I scripts/garnet_governance_activation_ceremony.py --activation-gate --reviewed-head $REVIEWED_HEAD --github-token-stdin",
        "expected": "RED blocked-u17 until Jon provisions GARNET_ADMIN_GITHUB_TOKEN; PASS only for authenticated exact 32-context no-bypass readback",
    },
)
TOP_KEYS = {
    "acceptance_commands",
    "activation",
    "bypass_actors",
    "evidence_destination",
    "jon_only_actions",
    "repository",
    "ruleset_id",
    "schema",
    "state",
    "target_branch",
    "token_policy",
    "transition",
    "workflow",
}


class DuplicateKeyError(ValueError):
    pass


def _object(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise DuplicateKeyError(f"duplicate JSON key {key!r}")
        result[key] = value
    return result


@dataclass(frozen=True)
class CeremonyStatus:
    schema: str
    preparation_ok: bool
    activation_ok: bool
    state: str
    ruleset_id: int | None
    workflow_sha256: str
    bypass_actors: tuple[object, ...]
    activation_problems: tuple[str, ...]
    problems: tuple[str, ...]


def _canonical_bytes(value: object) -> bytes:
    return (
        json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    ).encode("utf-8")


def _mapping(
    value: object, keys: set[str], label: str, problems: list[str]
) -> dict[str, object] | None:
    if type(value) is not dict or set(value) != keys:
        problems.append(f"{label} keys are not exact")
        return None
    return value


def read_ceremony_status(root: Path = ROOT) -> CeremonyStatus:
    problems: list[str] = []
    path = root / CEREMONY_PATH
    try:
        payload = path.read_bytes()
        if not 0 < len(payload) <= MAX_DOCUMENT_BYTES:
            raise ValueError("ceremony document size is invalid")
        document = json.loads(
            payload.decode("utf-8", errors="strict"), object_pairs_hook=_object
        )
        if payload != _canonical_bytes(document):
            problems.append("ceremony JSON is not canonical sorted UTF-8")
    except (OSError, UnicodeError, json.JSONDecodeError, DuplicateKeyError, ValueError) as exc:
        return CeremonyStatus(
            SCHEMA,
            False,
            False,
            "",
            None,
            "",
            (),
            ("blocked-u17",),
            (f"cannot load ceremony document: {exc}",),
        )
    top = _mapping(document, TOP_KEYS, "ceremony", problems)
    if top is None:
        top = {}
    if top.get("schema") != SCHEMA:
        problems.append("ceremony schema is not exact")
    if top.get("state") != "prepared-not-activated":
        problems.append("ceremony state must remain prepared-not-activated")
    if (top.get("repository"), top.get("target_branch")) != (
        "Island-Dev-Crew/garnet",
        "main",
    ):
        problems.append("ceremony repository/branch identity is not exact")
    if type(top.get("ruleset_id")) is not int or top.get("ruleset_id") != 18_936_562:
        problems.append("ceremony ruleset id is not exact")
    bypass = top.get("bypass_actors")
    if type(bypass) is not list or bypass:
        problems.append("ceremony bypass_actors must be an exact empty list")
        bypass = []

    workflow = _mapping(
        top.get("workflow"),
        {"context", "job", "name", "path", "sha256"},
        "workflow",
        problems,
    )
    workflow_sha = ""
    if workflow is not None:
        expected = {
            "context": "Base-controlled trust policy",
            "job": "policy",
            "name": "Base-controlled trust",
            "path": WORKFLOW_PATH,
        }
        if any(workflow.get(key) != value for key, value in expected.items()):
            problems.append("protected workflow identity is not exact")
        raw_sha = workflow.get("sha256")
        if type(raw_sha) is not str or SHA256_RE.fullmatch(raw_sha) is None:
            problems.append("protected workflow SHA-256 is malformed")
        else:
            workflow_sha = raw_sha
            try:
                actual_sha = hashlib.sha256((root / WORKFLOW_PATH).read_bytes()).hexdigest()
            except OSError as exc:
                problems.append(f"cannot read protected workflow: {exc}")
            else:
                if actual_sha != workflow_sha:
                    problems.append("protected workflow SHA-256 does not match exact bytes")

    transition = _mapping(
        top.get("transition"),
        {
            "appended_context",
            "from_required_context_count",
            "integration_id",
            "to_required_context_count",
        },
        "transition",
        problems,
    )
    if transition != {
        "appended_context": "Base-controlled trust policy",
        "from_required_context_count": 31,
        "integration_id": 15368,
        "to_required_context_count": 32,
    }:
        problems.append("ceremony transition is not the exact Actions-bound 31-to-32 append")

    token_policy = _mapping(
        top.get("token_policy"),
        {
            "admin_readback_token",
            "admin_token_source",
            "ambient_credentials",
            "persist",
            "print",
            "review_enumeration_scope",
            "review_enumeration_token",
        },
        "token policy",
        problems,
    )
    expected_token_policy = {
        "admin_readback_token": "GARNET_ADMIN_GITHUB_TOKEN",
        "admin_token_source": "explicit-only",
        "ambient_credentials": "forbidden",
        "persist": "forbidden",
        "print": "forbidden",
        "review_enumeration_scope": "pull-requests:read",
        "review_enumeration_token": "github.token (event-scoped)",
    }
    if token_policy != expected_token_policy:
        problems.append("ceremony token policy is not exact and separate")

    if top.get("jon_only_actions") != list(JON_ONLY_ACTIONS):
        problems.append("ceremony Jon-only actions are not exact")
    if top.get("acceptance_commands") != [dict(item) for item in ACCEPTANCE_COMMANDS]:
        problems.append("ceremony acceptance commands are not exact")
    if top.get("evidence_destination") != EVIDENCE_DESTINATION:
        problems.append("ceremony evidence destination is not exact")
    activation = _mapping(
        top.get("activation"),
        {"activated_at", "activated_by", "live_readback_evidence", "performed"},
        "activation",
        problems,
    )
    if activation != {
        "activated_at": None,
        "activated_by": None,
        "live_readback_evidence": None,
        "performed": False,
    }:
        problems.append("activation fields must remain empty and false")

    unique = tuple(dict.fromkeys(problems))
    return CeremonyStatus(
        SCHEMA,
        not unique,
        False,
        top.get("state") if type(top.get("state")) is str else "",
        top.get("ruleset_id") if type(top.get("ruleset_id")) is int else None,
        workflow_sha,
        tuple(bypass),
        ("blocked-u17",),
        unique,
    )


def _activation_blocked(
    prepared: CeremonyStatus, problems: list[str]
) -> CeremonyStatus:
    return CeremonyStatus(
        prepared.schema,
        prepared.preparation_ok,
        False,
        prepared.state,
        prepared.ruleset_id,
        prepared.workflow_sha256,
        prepared.bypass_actors,
        tuple(dict.fromkeys(problems)) or ("blocked-u17",),
        prepared.problems,
    )


def evaluate_live_activation(
    prepared: CeremonyStatus,
    live_result: object,
    *,
    root: Path = ROOT,
) -> CeremonyStatus:
    """Verify the exact 32-context ruleset and empty bypass list from live API data."""
    problems: list[str] = []
    if not prepared.preparation_ok:
        problems.append("ceremony preparation is not green")
    if type(live_result) is not transport.ObjectResult:
        problems.append("live ruleset transport result type is invalid")
        return _activation_blocked(prepared, problems)
    if type(live_result.problems) is not tuple or any(
        type(item) is not transport.GitHubTransportProblem
        for item in live_result.problems
    ):
        problems.append("live ruleset transport problems are malformed")
    elif live_result.problems:
        problems.append("live ruleset transport is incomplete")
    if (
        type(live_result.byte_count) is not int
        or not 0 < live_result.byte_count <= transport.MAX_BODY_BYTES
    ):
        problems.append("live ruleset transport byte count is invalid")
    live = live_result.value
    if type(live) is not dict:
        problems.append("live ruleset object is missing or malformed")
    if problems:
        return _activation_blocked(prepared, problems)
    assert isinstance(live, dict)
    if governance_gate._contains_credential_field(live):
        problems.append("live ruleset contains a credential-like field")
    if type(live.get("id")) is not int or live.get("id") != 18_936_562:
        problems.append("live ruleset id is not exact")
    if live.get("bypass_actors") != []:
        problems.append("live ruleset bypass_actors must be empty")

    checked, _, load_problems = governance_gate.load_checked_contracts(root)
    if load_problems or type(checked) is not dict:
        problems.append("checked ruleset authority cannot be loaded")
    else:
        expected = copy.deepcopy(checked)
        required = [
            rule
            for rule in expected.get("rules", [])
            if type(rule) is dict and rule.get("type") == "required_status_checks"
        ]
        if len(required) != 1:
            problems.append("checked ruleset required-status rule is not unique")
        else:
            parameters = required[0].get("parameters")
            contexts = (
                parameters.get("required_status_checks")
                if type(parameters) is dict
                else None
            )
            appended = {
                "context": "Base-controlled trust policy",
                "integration_id": 15368,
            }
            if type(contexts) is not list:
                problems.append("checked required contexts are malformed")
            elif len(contexts) == 31 and appended not in contexts:
                contexts.append(appended)
            elif len(contexts) != 32 or contexts[-1] != appended:
                problems.append("checked activation policy is not exact 31-to-32")
        live_rules = live.get("rules")
        live_required = [
            rule
            for rule in live_rules
            if type(rule) is dict and rule.get("type") == "required_status_checks"
        ] if type(live_rules) is list else []
        live_contexts = None
        if len(live_required) == 1 and type(live_required[0].get("parameters")) is dict:
            live_contexts = live_required[0]["parameters"].get(
                "required_status_checks"
            )
        if type(live_contexts) is not list or len(live_contexts) != 32:
            problems.append("live ruleset must require exactly 32 contexts")
        projection = {
            key: live.get(key) for key in governance_gate.RULESET_KEYS
        }
        if not governance_gate._strict_equal(projection, expected):
            problems.append("live 32-context ruleset differs from checked activation policy")
    if problems:
        return _activation_blocked(prepared, problems)
    return CeremonyStatus(
        prepared.schema,
        True,
        True,
        "live-readback-verified",
        18_936_562,
        prepared.workflow_sha256,
        (),
        (),
        prepared.problems,
    )


def main(
    argv: list[str] | None = None,
    *,
    root: Path = ROOT,
    stdin: object | None = None,
    stdout: object | None = None,
    environ: dict[str, str] | os._Environ[str] | None = None,
    transport_factory: object = transport.GitHubGovernanceTransport,
    local_head_loader: object = governance_gate.read_clean_local_head,
) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--gate", action="store_true", help="preparation-only gate")
    mode.add_argument(
        "--activation-gate",
        action="store_true",
        help="live activation/readback gate; remains RED while U-17 is open",
    )
    parser.add_argument(
        "--github-token-stdin",
        action="store_true",
        help="read the dedicated admin-authoritative credential from bounded stdin",
    )
    parser.add_argument(
        "--reviewed-head",
        help="exact clean local activation/terminus commit to verify",
    )
    args = parser.parse_args(argv)
    status = read_ceremony_status(root)
    if args.activation_gate:
        if (
            not args.github_token_stdin
            or type(args.reviewed_head) is not str
            or governance_gate.SHA_RE.fullmatch(args.reviewed_head) is None
        ):
            status = _activation_blocked(
                status,
                [
                    "blocked-u17: explicit admin stdin token and full reviewed head are required"
                ],
            )
        else:
            input_stream = sys.stdin if stdin is None else stdin
            source_environment = os.environ if environ is None else environ
            environment = dict(source_environment)
            ambient = [
                name
                for name in governance_gate.AMBIENT_CREDENTIAL_NAMES
                if source_environment.get(name)
            ]
            for name in governance_gate.AMBIENT_CREDENTIAL_NAMES:
                environment.pop(name, None)
            token, token_problems = governance_gate._read_explicit_token(input_stream)
            if ambient or token_problems:
                status = _activation_blocked(
                    status,
                    [
                        *(
                            ["ambient GitHub credential variables are forbidden"]
                            if ambient
                            else []
                        ),
                        *token_problems,
                    ],
                )
            elif not callable(transport_factory) or not callable(local_head_loader):
                status = _activation_blocked(
                    status, ["live activation collector configuration is invalid"]
                )
            else:
                try:
                    local_head, local_problems = local_head_loader(
                        root, environment
                    )
                except Exception:
                    status = _activation_blocked(
                        status, ["clean local HEAD loading failed closed"]
                    )
                else:
                    head_problems = list(local_problems)
                    if local_head != args.reviewed_head:
                        head_problems.append(
                            "reviewed head differs from clean local HEAD"
                        )
                    if head_problems:
                        status = _activation_blocked(status, head_problems)
                    else:
                        try:
                            client = transport_factory(
                                "Island-Dev-Crew/garnet", token
                            )
                            live_result = client.get_object("rulesets/18936562")
                        except Exception:
                            status = _activation_blocked(
                                status,
                                ["live activation collection failed closed"],
                            )
                        else:
                            status = evaluate_live_activation(
                                status, live_result, root=root
                            )
    elif args.github_token_stdin or args.reviewed_head is not None:
        status = CeremonyStatus(
            status.schema,
            False,
            False,
            status.state,
            status.ruleset_id,
            status.workflow_sha256,
            status.bypass_actors,
            status.activation_problems,
            (*status.problems, "preparation gate does not accept a credential"),
        )
    output_stream = sys.stdout if stdout is None else stdout
    output_stream.write(
        json.dumps(asdict(status), ensure_ascii=False, indent=2, sort_keys=True)
        + "\n"
    )
    return 0 if (status.activation_ok if args.activation_gate else status.preparation_ok) else 1


if __name__ == "__main__":
    raise SystemExit(main())
