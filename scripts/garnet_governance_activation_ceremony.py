#!/usr/bin/env python3
"""Validate the preparation-only human 31-to-32 activation package."""
from __future__ import annotations

import argparse
import hashlib
import json
import re
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
    "confirm the base-controlled workflow is active on main and open a separate activation/terminus pull request from that base",
    "activate required context 31 to 32 on ruleset 18936562 while the activation/terminus pull request is open",
    "read back ruleset 18936562 and verify Base-controlled trust policy is required with bypass_actors empty",
    "rerun the authenticated governance and base-controlled gates on the exact activation/terminus head",
    "merge the activation/terminus pull request",
)
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
        "command": "python3 -I scripts/garnet_governance_activation_ceremony.py --activation-gate",
        "expected": "RED blocked-u17 until Jon provisions GARNET_ADMIN_GITHUB_TOKEN and records live readback",
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


def main(argv: list[str] | None = None, *, root: Path = ROOT) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--gate", action="store_true", help="preparation-only gate")
    mode.add_argument(
        "--activation-gate",
        action="store_true",
        help="live activation/readback gate; remains RED while U-17 is open",
    )
    args = parser.parse_args(argv)
    status = read_ceremony_status(root)
    print(json.dumps(asdict(status), ensure_ascii=False, indent=2, sort_keys=True))
    if args.activation_gate:
        return 1
    return 0 if status.preparation_ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
