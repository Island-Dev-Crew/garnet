#!/usr/bin/env python3
"""Deterministic, read-only status reporter for the bounded Minimum Shelf."""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = "garnet.minimum_shelf_status/v1"
PROOF_PATH = Path("proofs/minimum-shelf/lane2b/PROOF.json")
INPUT_HEX_PATH = Path("proofs/minimum-shelf/lane2b/mcp-session.input.hex")
OUTPUT_HEX_PATH = Path("proofs/minimum-shelf/lane2b/mcp-session.output.hex")
WV6_ROOT = Path("proofs/windows/launch-verification/wv6-minimum-shelf")
CROSS_CHECKOUT_EVIDENCE = Path("ops/lane2b/evidence/12-reporter-cross-checkout.txt")
RUNTIME_COMMIT = "a6f0da2b81a9b181dafb83e15a17f8f313406e49"
RUNTIME_TREE = "fb4efe6ddc0280a942b4d0ac60b9c6017a72ca10"
MAX_JSON_BYTES = 64 * 1024
MAX_HEX_BYTES = 4 * 1024 * 1024
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
HEX_RE = re.compile(r"^[0-9a-f]+$")

EXPECTED_FILE_SHA256 = {
    ".gitattributes": "cb75c76ea88a8a18b990a1ce323ed26c3c03849c40fe08c847886e865fc42bc8",
    "examples/minimum-shelf-flagship/SHELF_PACKAGE.json": "dc12370de087c8beeb8885b4d51d4e37b8b02b8d7af90a3957bcb7c1e111b618",
    "examples/minimum-shelf-flagship/tool.garnet": "25ebd3dc02c8ab7d17e343c867e6becda9918c84a567f56f5b14ba4aba08a967",
    "examples/minimum-shelf-flagship/tool.seal.json": "526ac0f63f8ac487f6c38fd947defe1b80e1c3c14d80e3cfb38f5a66355b9cbd",
    "garnet-interp-v0.3/src/prelude.rs": "784864f2ccf16eb5494a39e42c5c4bf15b6ac0e09b7f729ca8ecc0e23ad81c62",
    "ops/lane2b/evidence/10-f1-canonical-reseal-green.txt": "02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7",
}
EXPECTED_INPUT_SHA256 = "2328d55497368e0a351cfbd0e5421ab46c3826abf4f313c1a497e95a9fbfd769"
EXPECTED_OUTPUT_SHA256 = "dac0eea7138d0f58865eecefc8db0c64490605068b956f757f420d6b284ba15f"
EXPECTED_SEAL_BLAKE3 = "6913c59afbb93e737c5e87d507d7f66b786a71668c0f85288bf636bbc3433696"
EXPECTED_PRELUDE_BLAKE3 = "df4f1648cf79ea77d0842fd1cb8725aba82be1b2631d5a906952640f9a25cc6d"
EXPECTED_SCOPE = [
    "one Garnet-owned local package",
    "Core Ring Tier 1 only",
    "raw-byte stdio only",
    "no hosted registry or network transport",
    "reviewed local content, not signer identity",
]
EXPECTED_COMMANDS = {
    "core-ring-tier1": "cargo test -p garnet-cli minimum_shelf --no-fail-fast",
    "mcp-raw-byte-stdio": "cargo test -p garnet-cli --test mcp_stdio --no-fail-fast",
    "sealed-baseline": "cargo test -p garnet-cli --test minimum_shelf_package sealed --no-fail-fast",
    "reject-without-seal": "cargo test -p garnet-cli --test minimum_shelf_package rejects --no-fail-fast",
}


@dataclass
class MinimumShelfStatus:
    schema: str = SCHEMA
    state: str = "partial"
    platform: str = "windows"
    runtime_candidate_commit: str = RUNTIME_COMMIT
    runtime_candidate_tree: str = RUNTIME_TREE
    current_commit: str | None = None
    current_tree: str | None = None
    implementer: str = "Codex GPT-5.6 Sol"
    reviewer: str = "Claude Code Fable 5"
    package: str = "examples/minimum-shelf-flagship"
    tool: str = "garnet.core.double"
    ring: str = "core"
    tier: int = 1
    unsigned_predicate: bool = True
    request_frame_count: int = 0
    response_frame_count: int = 0
    checks: dict[str, bool] = field(default_factory=dict)
    artifact_sha256: dict[str, str] = field(default_factory=dict)
    scope_limits: list[str] = field(default_factory=lambda: list(EXPECTED_SCOPE))
    findings: list[str] = field(default_factory=list)
    ok: bool = False


def _reject_duplicates(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key {key!r}")
        result[key] = value
    return result


def _read_bytes(relative: str | Path, limit: int) -> bytes:
    path = ROOT / relative
    if path.is_symlink() or not path.is_file():
        raise ValueError(f"{Path(relative).as_posix()} is not a regular file")
    size = path.stat().st_size
    if size <= 0 or size > limit:
        raise ValueError(f"{Path(relative).as_posix()} exceeds its byte bound")
    return path.read_bytes()


def _read_json(relative: str | Path) -> dict[str, object]:
    raw = _read_bytes(relative, MAX_JSON_BYTES)
    try:
        value = json.loads(
            raw.decode("utf-8"), object_pairs_hook=_reject_duplicates
        )
    except (UnicodeError, json.JSONDecodeError, ValueError) as exc:
        raise ValueError(f"{Path(relative).as_posix()} is not strict JSON: {exc}") from exc
    if not isinstance(value, dict):
        raise ValueError(f"{Path(relative).as_posix()} root is not an object")
    return value


def _sha256(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def _canonical_json(value: object) -> bytes:
    return (
        json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
        + "\n"
    ).encode("utf-8")


def _git(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", "-C", str(ROOT), *args],
        text=True,
        capture_output=True,
        check=False,
        encoding="utf-8",
    )


def _git_value(findings: list[str], *args: str) -> str | None:
    proc = _git(*args)
    value = proc.stdout.strip()
    if proc.returncode != 0 or SHA_RE.fullmatch(value) is None:
        findings.append(f"git {' '.join(args)} did not return one commit/tree id")
        return None
    return value


def _decode_hex(relative: Path) -> bytes:
    raw = _read_bytes(relative, MAX_HEX_BYTES)
    try:
        text = raw.decode("ascii")
    except UnicodeError as exc:
        raise ValueError(f"{relative.as_posix()} is not ASCII hex") from exc
    if not text.endswith("\n") or text.count("\n") != 1 or "\r" in text:
        raise ValueError(f"{relative.as_posix()} must be one canonical LF-terminated hex line")
    payload = text[:-1]
    if len(payload) % 2 or HEX_RE.fullmatch(payload) is None:
        raise ValueError(f"{relative.as_posix()} is not lowercase byte-exact hex")
    return bytes.fromhex(payload)


def _parse_frames(raw: bytes) -> list[object]:
    messages: list[object] = []
    offset = 0
    while offset < len(raw):
        end = raw.find(b"\r\n\r\n", offset)
        if end < 0:
            raise ValueError("transcript has an incomplete CRLF header")
        header = raw[offset:end]
        match = re.fullmatch(rb"Content-Length: (0|[1-9][0-9]*)", header)
        if match is None:
            raise ValueError("transcript has a noncanonical Content-Length header")
        length = int(match.group(1))
        body_start = end + 4
        body_end = body_start + length
        if body_end > len(raw):
            raise ValueError("transcript body is truncated")
        try:
            message = json.loads(
                raw[body_start:body_end].decode("utf-8"),
                object_pairs_hook=_reject_duplicates,
            )
        except (UnicodeError, json.JSONDecodeError, ValueError) as exc:
            raise ValueError(f"transcript body is not strict JSON: {exc}") from exc
        messages.append(message)
        offset = body_end
    return messages


def _expected_requests() -> list[dict[str, object]]:
    return [
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {"name": "garnet-committed-trap", "version": "1"},
            },
        },
        {"jsonrpc": "2.0", "method": "notifications/initialized"},
        {"jsonrpc": "2.0", "id": 2, "method": "tools/list"},
        {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {"name": "garnet.core.double", "arguments": {"value": 21}},
        },
        {"jsonrpc": "2.0", "id": 4, "method": "not/a/method"},
    ]


def _responses_are_exact(messages: list[object]) -> bool:
    if len(messages) != 4 or not all(isinstance(item, dict) for item in messages):
        return False
    first, listed, called, failed = messages
    tools = listed.get("result", {}).get("tools", [])
    return (
        first
        == {
            "id": 1,
            "jsonrpc": "2.0",
            "result": {
                "capabilities": {"tools": {"listChanged": False}},
                "protocolVersion": "2025-11-25",
                "serverInfo": {"name": "garnet-minimum-shelf", "version": "0.8.1"},
            },
        }
        and isinstance(tools, list)
        and len(tools) == 1
        and tools[0].get("name") == "garnet.core.double"
        and tools[0].get("inputSchema", {}).get("additionalProperties") is False
        and tools[0].get("outputSchema", {}).get("additionalProperties") is False
        and called
        == {
            "id": 3,
            "jsonrpc": "2.0",
            "result": {
                "content": [{"text": "42", "type": "text"}],
                "isError": False,
                "structuredContent": {"value": 42},
            },
        }
        and failed
        == {
            "error": {"code": -32601, "message": "Method not found"},
            "id": 4,
            "jsonrpc": "2.0",
        }
    )


def _validate_proof(proof: dict[str, object], findings: list[str]) -> None:
    exact_keys = {
        "schema",
        "runtimeCandidateCommit",
        "runtimeCandidateTree",
        "platform",
        "implementer",
        "reviewer",
        "authenticatedCarrier",
        "unsignedPredicate",
        "checks",
        "transcript",
        "scopeLimits",
        "jonOnlyActionsPerformed",
    }
    if set(proof) != exact_keys:
        findings.append("proof keys are not exact")
    expected_scalars = {
        "schema": "garnet.minimum-shelf-proof/v1",
        "runtimeCandidateCommit": RUNTIME_COMMIT,
        "runtimeCandidateTree": RUNTIME_TREE,
        "platform": "windows",
        "implementer": "Codex GPT-5.6 Sol",
        "reviewer": "Claude Code Fable 5",
        "authenticatedCarrier": "Jon",
        "unsignedPredicate": True,
    }
    for key, expected in expected_scalars.items():
        if proof.get(key) != expected:
            findings.append(f"proof {key} is not canonical")
    if proof.get("scopeLimits") != EXPECTED_SCOPE:
        findings.append("proof scope limits are not exact")
    if proof.get("jonOnlyActionsPerformed") != []:
        findings.append("proof records a forbidden Jon-only action")
    checks = proof.get("checks")
    found: dict[str, dict[str, object]] = {}
    if not isinstance(checks, list) or len(checks) != len(EXPECTED_COMMANDS):
        findings.append("proof check set is not exact")
        checks = []
    for item in checks:
        if not isinstance(item, dict) or set(item) != {"id", "status", "command"}:
            findings.append("proof check shape is not exact")
            continue
        identifier = item.get("id")
        if not isinstance(identifier, str) or identifier in found:
            findings.append("proof check id is invalid or duplicated")
            continue
        found[identifier] = item
    if set(found) != set(EXPECTED_COMMANDS):
        findings.append("proof check ids are not exact")
    for identifier, command in EXPECTED_COMMANDS.items():
        item = found.get(identifier, {})
        if item.get("status") != "passed" or item.get("command") != command:
            findings.append(f"proof check {identifier} is not canonical passed evidence")
    transcript = proof.get("transcript")
    expected_transcript = {
        "input": INPUT_HEX_PATH.as_posix(),
        "inputSha256": EXPECTED_INPUT_SHA256,
        "output": OUTPUT_HEX_PATH.as_posix(),
        "outputSha256": EXPECTED_OUTPUT_SHA256,
        "processExit": 0,
        "stderrBytes": 0,
    }
    if transcript != expected_transcript:
        findings.append("proof transcript binding is not exact")


def read_status(root: Path = ROOT) -> MinimumShelfStatus:
    if root != ROOT:
        raise ValueError("alternate roots are not supported by this locked reporter")
    status = MinimumShelfStatus()
    findings = status.findings

    status.current_commit = _git_value(findings, "rev-parse", "HEAD")
    status.current_tree = _git_value(findings, "rev-parse", "HEAD^{tree}")
    runtime_tree = _git_value(findings, "rev-parse", f"{RUNTIME_COMMIT}^{{tree}}")
    if runtime_tree != RUNTIME_TREE:
        findings.append("runtime candidate tree does not match its committed binding")
    if _git("merge-base", "--is-ancestor", RUNTIME_COMMIT, "HEAD").returncode != 0:
        findings.append("runtime candidate is not reachable from current HEAD")
    product_diff = _git(
        "diff",
        "--quiet",
        f"{RUNTIME_COMMIT}..HEAD",
        "--",
        ".",
        ":(exclude)ops/lane2b/**",
        ":(exclude)proofs/**",
        ":(exclude)F_Project_Management/W_TRUST/**",
        ":(exclude)scripts/smoke_garnet_minimum_shelf.py",
    )
    if product_diff.returncode != 0:
        findings.append("product bytes changed after the recorded runtime candidate")

    for relative, expected in EXPECTED_FILE_SHA256.items():
        try:
            raw = _read_bytes(relative, MAX_HEX_BYTES)
        except ValueError as exc:
            findings.append(str(exc))
            continue
        actual = _sha256(raw)
        status.artifact_sha256[relative] = actual
        if actual != expected:
            findings.append(f"{relative} SHA-256 does not match the reviewed artifact")
        if _git("ls-files", "--error-unmatch", "--", relative).returncode != 0:
            findings.append(f"{relative} is not tracked")

    try:
        proof = _read_json(PROOF_PATH)
        _validate_proof(proof, findings)
    except ValueError as exc:
        findings.append(str(exc))
    for relative in [
        PROOF_PATH,
        INPUT_HEX_PATH,
        OUTPUT_HEX_PATH,
        Path("scripts/smoke_garnet_minimum_shelf.py"),
    ]:
        if _git("ls-files", "--error-unmatch", "--", relative.as_posix()).returncode != 0:
            findings.append(f"{relative.as_posix()} is not tracked")

    try:
        input_bytes = _decode_hex(INPUT_HEX_PATH)
        output_bytes = _decode_hex(OUTPUT_HEX_PATH)
        status.artifact_sha256[INPUT_HEX_PATH.as_posix()] = _sha256(input_bytes)
        status.artifact_sha256[OUTPUT_HEX_PATH.as_posix()] = _sha256(output_bytes)
        if _sha256(input_bytes) != EXPECTED_INPUT_SHA256:
            findings.append("raw MCP input transcript hash does not match")
        if _sha256(output_bytes) != EXPECTED_OUTPUT_SHA256:
            findings.append("raw MCP output transcript hash does not match")
        if b"\n\n" in input_bytes or b"\n\n" in output_bytes:
            findings.append("raw MCP transcript contains text-mode LF framing")
        requests = _parse_frames(input_bytes)
        responses = _parse_frames(output_bytes)
        status.request_frame_count = len(requests)
        status.response_frame_count = len(responses)
        if requests != _expected_requests():
            findings.append("raw MCP request transcript is not the frozen journey")
        if not _responses_are_exact(responses):
            findings.append("raw MCP response transcript is not the frozen result")
    except ValueError as exc:
        findings.append(str(exc))

    try:
        attributes = _read_bytes(".gitattributes", MAX_JSON_BYTES).decode("utf-8")
        for line in [
            "garnet-interp-v0.3/src/prelude.rs text eol=lf",
            "examples/minimum-shelf-flagship/SHELF_PACKAGE.json text eol=lf",
            "examples/minimum-shelf-flagship/tool.seal.json text eol=lf",
            "proofs/** -text",
        ]:
            if line not in attributes.splitlines():
                findings.append(f"missing canonical attribute rule: {line}")
        prelude = _read_bytes("garnet-interp-v0.3/src/prelude.rs", MAX_HEX_BYTES)
        if b"\r" in prelude:
            findings.append("byte-hashed prelude contains CR bytes")
        package = _read_json("examples/minimum-shelf-flagship/SHELF_PACKAGE.json")
        if (
            package.get("schema") != "garnet.minimum-shelf-package/v1"
            or package.get("ring") != "core"
            or package.get("tier") != 1
            or package.get("tool") != "garnet.core.double"
            or package.get("sealKind") != "in-toto-predicate-unsigned"
            or package.get("sealBlake3") != EXPECTED_SEAL_BLAKE3
        ):
            findings.append("flagship package contract is not exact")
        seal = _read_json("examples/minimum-shelf-flagship/tool.seal.json")
        if seal.get("predicate", {}).get("build_manifest", {}).get("prelude_hash") != EXPECTED_PRELUDE_BLAKE3:
            findings.append("seal does not bind the canonical prelude")
        cosign = seal.get("predicate", {}).get("tooling", {}).get("cosign")
        if cosign != "not installed — predicate emitted UNSIGNED; install cosign to attest":
            findings.append("UNSIGNED predicate honesty language changed")
    except (UnicodeError, ValueError, AttributeError) as exc:
        findings.append(str(exc))

    status.checks = {
        identifier: not any(identifier in finding for finding in findings)
        for identifier in EXPECTED_COMMANDS
    }
    status.checks["deterministic-shelf-reporter"] = not findings
    status.ok = not findings
    status.state = "accepted" if status.ok else "partial"
    return status


def _emit_wv6(status: MinimumShelfStatus) -> None:
    if not status.ok or status.current_commit is None:
        raise ValueError("refusing to emit WV-6 evidence from a non-accepted status")
    destination = ROOT / WV6_ROOT
    if destination.exists() or destination.is_symlink():
        raise ValueError(f"{WV6_ROOT.as_posix()} already exists; overwrite refused")

    sources = {
        "mcp-session.input.hex": INPUT_HEX_PATH,
        "mcp-session.output.hex": OUTPUT_HEX_PATH,
        "f1-canonical-reseal.txt": Path(
            "ops/lane2b/evidence/10-f1-canonical-reseal-green.txt"
        ),
        "reporter-cross-checkout.txt": CROSS_CHECKOUT_EVIDENCE,
    }
    payloads = {
        "minimum-shelf-status.json": _canonical_json(asdict(status)),
    }
    for output_name, source in sources.items():
        payloads[output_name] = _read_bytes(source, MAX_HEX_BYTES)

    destination.mkdir(parents=True)
    for name, raw in sorted(payloads.items()):
        (destination / name).write_bytes(raw)

    artifact_rows = [
        {"path": name, "sha256": _sha256(raw)}
        for name, raw in sorted(payloads.items())
    ]
    common = ["minimum-shelf-status.json"]
    checks = [
        {
            "id": "core-ring-tier1",
            "status": "passed",
            "command": EXPECTED_COMMANDS["core-ring-tier1"],
            "evidence": common,
        },
        {
            "id": "mcp-raw-byte-stdio",
            "status": "passed",
            "command": EXPECTED_COMMANDS["mcp-raw-byte-stdio"],
            "evidence": [
                "minimum-shelf-status.json",
                "mcp-session.input.hex",
                "mcp-session.output.hex",
            ],
        },
        {
            "id": "sealed-baseline",
            "status": "passed",
            "command": EXPECTED_COMMANDS["sealed-baseline"],
            "evidence": ["minimum-shelf-status.json", "f1-canonical-reseal.txt"],
        },
        {
            "id": "reject-without-seal",
            "status": "passed",
            "command": EXPECTED_COMMANDS["reject-without-seal"],
            "evidence": ["minimum-shelf-status.json", "f1-canonical-reseal.txt"],
        },
        {
            "id": "deterministic-shelf-reporter",
            "status": "passed",
            "command": "python3 -I scripts/smoke_garnet_minimum_shelf.py --gate",
            "evidence": [
                "minimum-shelf-status.json",
                "reporter-cross-checkout.txt",
            ],
        },
    ]
    manifest = {
        "schema": "garnet.wv_acceptance_evidence/v1",
        "wv": "WV-6",
        "contractBaseMainSha": "231aefa91985e5a0520c493c7f0fc3e54d74efc8",
        "candidateMainSha": status.current_commit,
        "state": "evidence_complete",
        "platform": "windows",
        "checks": checks,
        "artifacts": artifact_rows,
        "scopeLimitsAcknowledged": True,
        "jonOnlyActionsPerformed": [],
    }
    (destination / "WV_ACCEPTANCE.json").write_bytes(_canonical_json(manifest))


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--gate", action="store_true")
    parser.add_argument("--emit-wv6", action="store_true")
    args = parser.parse_args(argv)
    status = read_status()
    print(
        json.dumps(
            asdict(status),
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
        )
    )
    if args.emit_wv6:
        try:
            _emit_wv6(status)
        except (OSError, ValueError) as exc:
            print(f"WV-6 evidence emission FAILED: {exc}", file=sys.stderr)
            return 1
    if args.gate and not status.ok:
        print(
            "Minimum Shelf gate FAILED: " + "; ".join(status.findings),
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
