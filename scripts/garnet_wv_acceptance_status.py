#!/usr/bin/env python3
"""Fail-closed acceptance reporter for the established WV-6/WV-7 contracts."""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
import unicodedata
from dataclasses import asdict, dataclass, field
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parents[1]
CONTRACT_PATH = Path("F_Project_Management/LAUNCH/WV6_WV7_ACCEPTANCE_CONTRACTS.json")
EXPECTED_BASE_SHA = "231aefa91985e5a0520c493c7f0fc3e54d74efc8"
EVIDENCE_MANIFEST = "WV_ACCEPTANCE.json"
EXPECTED_DESTINATIONS = {
    "WV-6": "proofs/windows/launch-verification/wv6-minimum-shelf/",
    "WV-7": "proofs/windows/launch-verification/wv7-distribution/",
}
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
MAX_MANIFEST_BYTES = 128 * 1024
MAX_ARTIFACTS = 128
MAX_ARTIFACT_BYTES = 16 * 1024 * 1024
MAX_TOTAL_BYTES = 64 * 1024 * 1024


@dataclass
class WvAcceptanceStatus:
    schema: str
    wv: str
    contract_base_main_sha: str | None
    evidence_destination: str | None
    candidate_main_sha: str | None
    required_check_count: int
    passed_check_count: int
    artifact_count: int
    state: str
    findings: list[str] = field(default_factory=list)
    ok: bool = False


def _reject_duplicates(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key {key!r}")
        result[key] = value
    return result


def _read_json(path: Path, *, limit: int) -> dict[str, object]:
    if path.is_symlink() or not path.is_file():
        raise ValueError(f"{path} is not a regular file")
    size = path.stat().st_size
    if size <= 0 or size > limit:
        raise ValueError(f"{path} exceeds the bounded JSON size")
    try:
        value = json.loads(
            path.read_text(encoding="utf-8"),
            object_pairs_hook=_reject_duplicates,
        )
    except (OSError, UnicodeError, json.JSONDecodeError, ValueError) as exc:
        raise ValueError(f"{path} is invalid strict JSON: {exc}") from exc
    if not isinstance(value, dict):
        raise ValueError(f"{path} root must be an object")
    return value


def _contract_command(identifier: str) -> str:
    return (
        "python3 -I scripts/garnet_wv_acceptance_status.py "
        f"--wv {identifier} --gate"
    )


def load_contracts(root: Path = ROOT) -> dict[str, dict[str, object]]:
    document = _read_json(root / CONTRACT_PATH, limit=MAX_MANIFEST_BYTES)
    if document.get("schema") != "garnet.wv_acceptance_contracts/v2":
        raise ValueError("WV contract schema must be garnet.wv_acceptance_contracts/v2")
    if document.get("claimState") != "planned":
        raise ValueError("WV contract artifact must remain planned")
    if document.get("frozenAtBaseMainSha") != EXPECTED_BASE_SHA:
        raise ValueError("WV contract artifact base SHA is not the Lane 0 pin")
    rows = document.get("contracts")
    if not isinstance(rows, list):
        raise ValueError("WV contracts must be an array")

    contracts: dict[str, dict[str, object]] = {}
    for row in rows:
        if not isinstance(row, dict):
            raise ValueError("each WV contract must be an object")
        identifier = row.get("id")
        if identifier not in EXPECTED_DESTINATIONS or not isinstance(identifier, str):
            raise ValueError(f"unsupported WV contract id {identifier!r}")
        if identifier in contracts:
            raise ValueError(f"duplicate WV contract {identifier}")
        if row.get("claimState") != "planned":
            raise ValueError(f"{identifier} contract must remain planned")
        if row.get("exactBaseMainSha") != EXPECTED_BASE_SHA:
            raise ValueError(f"{identifier} exact base SHA is not the Lane 0 pin")
        if row.get("acceptanceCommand") != _contract_command(identifier):
            raise ValueError(f"{identifier} acceptance command is not canonical")
        if row.get("evidenceDestination") != EXPECTED_DESTINATIONS[identifier]:
            raise ValueError(f"{identifier} evidence destination is not canonical")
        if row.get("evidenceManifest") != EVIDENCE_MANIFEST:
            raise ValueError(f"{identifier} evidence manifest name is not canonical")
        checks = row.get("requiredChecks")
        if not isinstance(checks, list) or not checks:
            raise ValueError(f"{identifier} requiredChecks must be non-empty")
        check_ids: list[str] = []
        for check in checks:
            if not isinstance(check, dict) or set(check) != {"id", "criterion"}:
                raise ValueError(f"{identifier} required check shape is not exact")
            check_id, criterion = check.get("id"), check.get("criterion")
            if (
                not isinstance(check_id, str)
                or not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", check_id)
                or not isinstance(criterion, str)
                or not criterion.strip()
            ):
                raise ValueError(f"{identifier} required check is not canonical")
            check_ids.append(check_id)
        if len(set(check_ids)) != len(check_ids):
            raise ValueError(f"{identifier} required check ids contain duplicates")
        contracts[identifier] = row
    if set(contracts) != set(EXPECTED_DESTINATIONS):
        raise ValueError("WV contract set must be exactly WV-6 and WV-7")
    return contracts


def _artifact_relative(value: object) -> str:
    if not isinstance(value, str) or not value or "\\" in value:
        raise ValueError("artifact path must be a non-empty POSIX path")
    if value != unicodedata.normalize("NFC", value) or not value.isprintable():
        raise ValueError("artifact path is not canonical text")
    relative = PurePosixPath(value)
    if relative.is_absolute() or ".." in relative.parts or "." in relative.parts:
        raise ValueError(f"artifact path escapes evidence root: {value!r}")
    if relative.as_posix() != value or value == EVIDENCE_MANIFEST:
        raise ValueError(f"artifact path is not canonical: {value!r}")
    return value


def _git(root: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", "-C", str(root), *args],
        text=True,
        capture_output=True,
        check=False,
    )


def _verify_candidate(root: Path, candidate: str, findings: list[str]) -> None:
    if _git(root, "cat-file", "-e", f"{candidate}^{{commit}}").returncode != 0:
        findings.append("candidateMainSha is not a local commit object")
        return
    if (
        _git(root, "merge-base", "--is-ancestor", EXPECTED_BASE_SHA, candidate).returncode
        != 0
    ):
        findings.append("candidateMainSha does not descend from the Lane 0 base")
    if _git(root, "merge-base", "--is-ancestor", candidate, "HEAD").returncode != 0:
        findings.append("candidateMainSha is not reachable from current HEAD")


def _validate_evidence(
    root: Path,
    contract: dict[str, object],
    evidence_root: Path,
    *,
    verify_git: bool,
) -> tuple[list[str], str | None, int, int]:
    findings: list[str] = []
    manifest_path = evidence_root / EVIDENCE_MANIFEST
    try:
        manifest = _read_json(manifest_path, limit=MAX_MANIFEST_BYTES)
    except ValueError as exc:
        return [str(exc)], None, 0, 0

    exact_keys = {
        "schema",
        "wv",
        "contractBaseMainSha",
        "candidateMainSha",
        "state",
        "platform",
        "checks",
        "artifacts",
        "scopeLimitsAcknowledged",
        "jonOnlyActionsPerformed",
    }
    if set(manifest) != exact_keys:
        findings.append("evidence manifest keys are not exact")
    identifier = contract["id"]
    if manifest.get("schema") != "garnet.wv_acceptance_evidence/v1":
        findings.append("evidence manifest schema is invalid")
    if manifest.get("wv") != identifier:
        findings.append("evidence manifest WV id does not match the contract")
    if manifest.get("contractBaseMainSha") != EXPECTED_BASE_SHA:
        findings.append("evidence manifest base SHA does not match the contract")
    candidate = manifest.get("candidateMainSha")
    if not isinstance(candidate, str) or SHA_RE.fullmatch(candidate) is None:
        findings.append("candidateMainSha must be one full lowercase commit SHA")
        candidate_sha = None
    else:
        candidate_sha = candidate
        if candidate == EXPECTED_BASE_SHA:
            findings.append("candidateMainSha must advance beyond the Lane 0 base")
        if verify_git:
            _verify_candidate(root, candidate, findings)
    if manifest.get("state") != "evidence_complete":
        findings.append("evidence manifest state must be evidence_complete")
    if manifest.get("platform") != "windows":
        findings.append("WV acceptance evidence must be native Windows evidence")
    if manifest.get("scopeLimitsAcknowledged") is not True:
        findings.append("scope limits must be explicitly acknowledged")
    if manifest.get("jonOnlyActionsPerformed") != []:
        findings.append("the evidence gate must perform no Jon-only action")

    required = {
        item["id"] for item in contract["requiredChecks"] if isinstance(item, dict)
    }
    checks = manifest.get("checks")
    check_by_id: dict[str, dict[str, object]] = {}
    if not isinstance(checks, list) or len(checks) > 64:
        findings.append("checks must be a bounded array")
        checks = []
    for check in checks:
        if not isinstance(check, dict) or set(check) != {
            "id",
            "status",
            "command",
            "evidence",
        }:
            findings.append("evidence check shape is not exact")
            continue
        check_id = check.get("id")
        if not isinstance(check_id, str) or check_id in check_by_id:
            findings.append("evidence check id is invalid or duplicated")
            continue
        check_by_id[check_id] = check
        if check.get("status") != "passed":
            findings.append(f"required check {check_id} is not passed")
        command = check.get("command")
        if (
            not isinstance(command, str)
            or not command.strip()
            or "\n" in command
            or len(command) > 2048
        ):
            findings.append(f"required check {check_id} has invalid command evidence")
        evidence = check.get("evidence")
        if (
            not isinstance(evidence, list)
            or not evidence
            or len(evidence) > 16
            or not all(isinstance(item, str) for item in evidence)
        ):
            findings.append(f"required check {check_id} has invalid evidence paths")
    for missing in sorted(required - set(check_by_id)):
        findings.append(f"required check {missing} is missing")
    for extra in sorted(set(check_by_id) - required):
        findings.append(f"unrecognized required check {extra} is present")

    artifacts = manifest.get("artifacts")
    artifact_hashes: dict[str, str] = {}
    if not isinstance(artifacts, list) or len(artifacts) > MAX_ARTIFACTS:
        findings.append("artifacts must be a bounded array")
        artifacts = []
    total = 0
    for artifact in artifacts:
        if not isinstance(artifact, dict) or set(artifact) != {"path", "sha256"}:
            findings.append("artifact entry shape is not exact")
            continue
        try:
            relative = _artifact_relative(artifact.get("path"))
        except ValueError as exc:
            findings.append(str(exc))
            continue
        digest = artifact.get("sha256")
        if not isinstance(digest, str) or SHA256_RE.fullmatch(digest) is None:
            findings.append(f"artifact {relative} has invalid SHA-256")
            continue
        if relative in artifact_hashes:
            findings.append(f"artifact {relative} is duplicated")
            continue
        artifact_hashes[relative] = digest
        path = evidence_root / relative
        if path.is_symlink() or not path.is_file():
            findings.append(f"artifact {relative} is not a regular file")
            continue
        size = path.stat().st_size
        total += size
        if size > MAX_ARTIFACT_BYTES or total > MAX_TOTAL_BYTES:
            findings.append(f"artifact {relative} exceeds evidence size bounds")
            continue
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != digest:
            findings.append(f"artifact {relative} SHA-256 does not match")

    referenced: set[str] = set()
    for check in check_by_id.values():
        evidence = check.get("evidence")
        if isinstance(evidence, list):
            for raw in evidence:
                try:
                    referenced.add(_artifact_relative(raw))
                except ValueError as exc:
                    findings.append(str(exc))
    for missing in sorted(referenced - set(artifact_hashes)):
        findings.append(f"check evidence {missing} is absent from artifacts")

    actual_files = {
        path.relative_to(evidence_root).as_posix()
        for path in evidence_root.rglob("*")
        if path.is_file() and path.name != EVIDENCE_MANIFEST
    }
    if actual_files != set(artifact_hashes):
        findings.append("evidence directory files do not exactly match the manifest")

    passed = sum(
        1
        for check_id in required
        if check_by_id.get(check_id, {}).get("status") == "passed"
    )
    return findings, candidate_sha, passed, len(artifact_hashes)


def read_status(
    root: Path = ROOT, identifier: str = "WV-6", *, verify_git: bool = True
) -> WvAcceptanceStatus:
    findings: list[str] = []
    contract: dict[str, object] | None = None
    try:
        contract = load_contracts(root).get(identifier)
    except ValueError as exc:
        findings.append(str(exc))
    if contract is None:
        if not findings:
            findings.append(f"contract {identifier} is missing")
        return WvAcceptanceStatus(
            schema="garnet.wv_acceptance_status/v1",
            wv=identifier,
            contract_base_main_sha=None,
            evidence_destination=None,
            candidate_main_sha=None,
            required_check_count=0,
            passed_check_count=0,
            artifact_count=0,
            state="partial",
            findings=findings,
            ok=False,
        )

    destination = str(contract["evidenceDestination"])
    evidence_root = root / destination
    required_count = len(contract["requiredChecks"])
    if evidence_root.is_symlink():
        findings.append("evidence destination must not be a symlink")
        state = "partial"
        candidate = None
        passed = 0
        artifact_count = 0
    elif not evidence_root.is_dir() or not (evidence_root / EVIDENCE_MANIFEST).exists():
        findings.append("exact-candidate evidence manifest is pending")
        state = "pending"
        candidate = None
        passed = 0
        artifact_count = 0
    else:
        evidence_findings, candidate, passed, artifact_count = _validate_evidence(
            root, contract, evidence_root, verify_git=verify_git
        )
        findings.extend(evidence_findings)
        state = "accepted" if not findings else "partial"

    return WvAcceptanceStatus(
        schema="garnet.wv_acceptance_status/v1",
        wv=identifier,
        contract_base_main_sha=str(contract["exactBaseMainSha"]),
        evidence_destination=destination,
        candidate_main_sha=candidate,
        required_check_count=required_count,
        passed_check_count=passed,
        artifact_count=artifact_count,
        state=state,
        findings=findings,
        ok=state == "accepted" and not findings,
    )


def copy_contract(source: Path, destination: Path) -> None:
    target = destination / CONTRACT_PATH
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source / CONTRACT_PATH, target)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--wv", choices=sorted(EXPECTED_DESTINATIONS), required=True)
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit zero only for complete, hash-verified exact-candidate evidence",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    status = read_status(args.root.resolve(), args.wv)
    print(json.dumps(asdict(status), indent=2, sort_keys=True))
    if args.gate and not status.ok:
        print(
            f"{args.wv} acceptance gate {status.state.upper()}: "
            + "; ".join(status.findings),
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
