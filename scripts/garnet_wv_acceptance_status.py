#!/usr/bin/env python3
"""Fail-closed acceptance reporter for the established WV-6/WV-7 contracts."""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import errno
import json
import os
import re
import shutil
import stat
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
UTF8_BOM = b"\xef\xbb\xbf"
# FILE_ATTRIBUTE_REPARSE_POINT: a junction or symlink on native Windows, where
# O_NOFOLLOW does not exist and lstat alone does not always say S_ISLNK.
WINDOWS_REPARSE_POINT = 0x400

_CONTENT_PATH = Path(__file__).with_name("garnet_content_provenance.py")
_CONTENT_SPEC = importlib.util.spec_from_file_location(
    "garnet_content_provenance", _CONTENT_PATH
)
assert _CONTENT_SPEC is not None and _CONTENT_SPEC.loader is not None
content_provenance = importlib.util.module_from_spec(_CONTENT_SPEC)
sys.modules[_CONTENT_SPEC.name] = content_provenance
_CONTENT_SPEC.loader.exec_module(content_provenance)

_SHELF_PATH = Path(__file__).with_name("smoke_garnet_minimum_shelf.py")
_SHELF_SPEC = importlib.util.spec_from_file_location(
    "lane2b_bound_shelf_reporter", _SHELF_PATH
)
assert _SHELF_SPEC is not None and _SHELF_SPEC.loader is not None
bound_shelf_reporter = importlib.util.module_from_spec(_SHELF_SPEC)
sys.modules[_SHELF_SPEC.name] = bound_shelf_reporter
_SHELF_SPEC.loader.exec_module(bound_shelf_reporter)

REVIEWED_HEAD = bound_shelf_reporter.REVIEWED_HEAD
REVIEWED_TREE = bound_shelf_reporter.REVIEWED_TREE
EXPECTED_PRODUCT_CONTENT_SHA256 = (
    bound_shelf_reporter.EXPECTED_PRODUCT_CONTENT_SHA256
)


@dataclass
class WvAcceptanceStatus:
    schema: str
    wv: str
    contract_base_main_sha: str | None
    evidence_destination: str | None
    reviewed_head_sha: str | None
    reviewed_tree_sha: str | None
    product_content_sha256: str | None
    landed_main_sha: str | None
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


class EvidenceAbsent(ValueError):
    """The evidence file does not exist, as distinct from failing a check.

    Kept separate so the gate can report `pending` for absent evidence without
    performing its own presence check first. An extra presence check ahead of
    the bound read is itself a check-then-use: it consumes the swap window, and
    the single-descriptor read that follows then sees a consistently swapped
    file and accepts it (regression caught by
    test_manifest_swapped_after_its_check_is_never_accepted).
    """


DIR_FD_SUPPORTED = (
    os.open in os.supports_dir_fd
    and os.lstat in os.supports_dir_fd
    and hasattr(os, "O_DIRECTORY")
)


def open_evidence_root(path: Path) -> tuple[int | None, str]:
    """Bind the evidence directory to one descriptor, or return None.

    ``O_NOFOLLOW`` protects only the FINAL component, so checking the evidence
    directory by pathname and then resolving that pathname again for the
    manifest, every artifact, and the inventory left an ancestor swap open: a
    deterministic swap after the directory check replaced the checked directory
    with a symlink to an outside tree, and the reporter validated the outside
    evidence and returned ``accepted`` (review v1 finding, reproduced). Every
    later read is now relative to this descriptor, so the directory identity
    cannot change underneath the traversal.

    Returns ``(descriptor, reason)``. The reason distinguishes an absent
    destination (which stays ``pending``, as before) from a destination that
    exists but cannot be bound (which is a finding), so binding does not change
    the reporter's state vocabulary. ``unsupported`` means the platform has no
    ``dir_fd`` support; the caller then falls back to the pathname checks and
    the ancestor-swap bound stands on that platform.
    """
    if not DIR_FD_SUPPORTED:
        return None, "unsupported"
    flags = (
        os.O_RDONLY
        | os.O_DIRECTORY
        | getattr(os, "O_CLOEXEC", 0)
        | getattr(os, "O_NOFOLLOW", 0)
    )
    try:
        descriptor = os.open(path, flags)
    except FileNotFoundError:
        return None, "absent"
    except OSError:
        # Do not infer the cause from errno: O_NOFOLLOW|O_DIRECTORY on a
        # symlinked directory reports ELOOP on Linux and ENOTDIR on macOS.
        # Ask the filesystem what the destination actually is instead.
        try:
            entry = os.lstat(path)
        except FileNotFoundError:
            return None, "absent"
        except OSError:
            return None, "unbindable"
        if stat.S_ISLNK(entry.st_mode):
            return None, "symlink"
        return None, "unbindable"
    try:
        opened = os.fstat(descriptor)
    except OSError:
        os.close(descriptor)
        return None, "unbindable"
    if not stat.S_ISDIR(opened.st_mode):
        os.close(descriptor)
        return None, "not-a-directory"
    return descriptor, "ok"


def _regular_bytes(
    path: Path, *, limit: int, minimum: int, label: str, bound: str,
    dir_fd: int | None = None
) -> bytes:
    """Read ``path`` once through a descriptor and return exactly the bytes
    that were checked.

    The metadata check, the open, the size bound, the read, and the post-read
    identity check all bind to one descriptor; the path is never reopened, so
    a file swapped between check and use is a finding, not a redirected read.
    Every rejection is an explicit ``ValueError`` naming the file.
    """
    target = str(path) if dir_fd is None else os.fspath(path)
    try:
        before = os.lstat(target, dir_fd=dir_fd) if dir_fd is not None else os.lstat(path)
    except FileNotFoundError as exc:
        raise EvidenceAbsent(f"{label} is not a regular file") from exc
    except OSError as exc:
        raise ValueError(f"{label} is not a regular file") from exc
    if (
        stat.S_ISLNK(before.st_mode)
        or not stat.S_ISREG(before.st_mode)
        or bool(getattr(before, "st_file_attributes", 0) & WINDOWS_REPARSE_POINT)
    ):
        raise ValueError(f"{label} is not a regular file")
    if before.st_size < minimum or before.st_size > limit:
        raise ValueError(f"{label} exceeds {bound}")
    flags = (
        os.O_RDONLY
        | getattr(os, "O_BINARY", 0)
        | getattr(os, "O_CLOEXEC", 0)
        | getattr(os, "O_NOFOLLOW", 0)
    )
    try:
        descriptor = (
            os.open(target, flags, dir_fd=dir_fd) if dir_fd is not None
            else os.open(path, flags)
        )
    except OSError as exc:
        raise ValueError(f"{label} could not be opened: {exc.strerror}") from exc
    try:
        opened = os.fstat(descriptor)
        if not stat.S_ISREG(opened.st_mode) or (opened.st_dev, opened.st_ino) != (
            before.st_dev,
            before.st_ino,
        ):
            raise ValueError(f"{label} identity changed between check and open")
        if opened.st_size < minimum or opened.st_size > limit:
            raise ValueError(f"{label} exceeds {bound}")
        payload = bytearray()
        while len(payload) <= limit:
            try:
                chunk = os.read(descriptor, limit + 1 - len(payload))
            except OSError as exc:
                raise ValueError(f"{label} could not be read: {exc.strerror}") from exc
            if not chunk:
                break
            payload.extend(chunk)
        after = os.fstat(descriptor)
        identity = (opened.st_dev, opened.st_ino, opened.st_size, opened.st_mtime_ns)
        if identity != (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns):
            raise ValueError(f"{label} changed while being read")
        if len(payload) != opened.st_size or len(payload) > limit:
            raise ValueError(f"{label} changed while being read")
        return bytes(payload)
    finally:
        os.close(descriptor)


def _strict_lf_text(raw: bytes, *, label: str) -> str:
    """Decode WV evidence JSON under the byte rule: strict UTF-8, no
    byte-order mark, LF-only line endings.

    A text-mode read would silently normalise CRLF and hide the bytes that a
    hash or a reviewer sees; the rule rejects them by name instead. The
    contract states no canonical-JSON form for the evidence manifest, so no
    canonical requirement is asserted here.
    """
    if raw.startswith(UTF8_BOM):
        raise ValueError(f"{label} must not begin with a UTF-8 byte-order mark")
    carriage_return = raw.find(b"\r")
    if carriage_return != -1:
        raise ValueError(
            f"{label} must use LF-only line endings "
            f"(first CR byte at offset {carriage_return})"
        )
    try:
        return raw.decode("utf-8", errors="strict")
    except UnicodeDecodeError as exc:
        raise ValueError(f"{label} is not strict UTF-8: {exc}") from exc


def _manifest_present(evidence_root: Path, root_fd: int | None) -> bool:
    """Is the manifest a regular file, checked through the bound descriptor?"""
    if root_fd is None:
        return evidence_root.is_dir() and (evidence_root / EVIDENCE_MANIFEST).exists()
    try:
        entry = os.lstat(EVIDENCE_MANIFEST, dir_fd=root_fd)
    except OSError:
        return False
    return stat.S_ISREG(entry.st_mode)


def _inventory_from_descriptor(root_fd: int, prefix: str = "") -> set[str]:
    """List regular files under a bound directory descriptor, never by pathname.

    Each subdirectory is opened relative to its parent descriptor with
    ``O_NOFOLLOW``, so no component of the traversal can be swapped for a
    symlink between the check and the listing.
    """
    names: set[str] = set()
    with os.scandir(root_fd) as entries:
        for entry in entries:
            relative = f"{prefix}{entry.name}"
            try:
                if entry.is_symlink():
                    continue
                if entry.is_file(follow_symlinks=False):
                    if relative != EVIDENCE_MANIFEST:
                        names.add(relative)
                    continue
                if not entry.is_dir(follow_symlinks=False):
                    continue
            except OSError:
                continue
            flags = (
                os.O_RDONLY
                | os.O_DIRECTORY
                | getattr(os, "O_CLOEXEC", 0)
                | getattr(os, "O_NOFOLLOW", 0)
            )
            try:
                child = os.open(entry.name, flags, dir_fd=root_fd)
            except OSError:
                continue
            try:
                names |= _inventory_from_descriptor(child, f"{relative}/")
            finally:
                os.close(child)
    return names


def _read_json(
    path: Path, *, limit: int, dir_fd: int | None = None, label: str | None = None
) -> dict[str, object]:
    label = label or str(path)
    raw = _regular_bytes(
        path, limit=limit, minimum=1, label=label, bound="the bounded JSON size",
        dir_fd=dir_fd,
    )
    text = _strict_lf_text(raw, label=label)
    try:
        value = json.loads(text, object_pairs_hook=_reject_duplicates)
    except (RecursionError, ValueError) as exc:
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


def _verify_squash_durable_content(
    root: Path,
    *,
    reviewed_head: str,
    reviewed_tree: str,
    expected_content_digest: str,
    expected_content_path_count: int | None = None,
    verify_git: bool,
) -> tuple[list[str], str | None]:
    return content_provenance.verify_squash_durable_content(
        root,
        reviewed_head=reviewed_head,
        reviewed_tree=reviewed_tree,
        expected_content_digest=expected_content_digest,
        expected_content_path_count=expected_content_path_count,
        verify_git=verify_git,
    )


def _validate_evidence(
    root: Path,
    contract: dict[str, object],
    evidence_root: Path,
    *,
    verify_git: bool,
    root_fd: int | None = None,
) -> tuple[list[str], str | None, str | None, str | None, str | None, int, int]:
    findings: list[str] = []
    manifest_path = evidence_root / EVIDENCE_MANIFEST
    try:
        manifest = _read_json(
            Path(EVIDENCE_MANIFEST) if root_fd is not None else manifest_path,
            limit=MAX_MANIFEST_BYTES,
            dir_fd=root_fd,
            label=str(manifest_path),
        )
    except EvidenceAbsent:
        return ["exact-candidate evidence manifest is pending"], None, None, None, None, 0, 0
    except ValueError as exc:
        return [str(exc)], None, None, None, None, 0, 0

    exact_keys = {
        "schema",
        "wv",
        "contractBaseMainSha",
        "reviewedHeadSha",
        "reviewedTreeSha",
        "productContentSha256",
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
    if manifest.get("schema") != "garnet.wv_acceptance_evidence/v2":
        findings.append("evidence manifest schema is invalid")
    if manifest.get("wv") != identifier:
        findings.append("evidence manifest WV id does not match the contract")
    if manifest.get("contractBaseMainSha") != EXPECTED_BASE_SHA:
        findings.append("evidence manifest base SHA does not match the contract")
    reviewed_head = manifest.get("reviewedHeadSha")
    if reviewed_head != REVIEWED_HEAD:
        findings.append("reviewedHeadSha does not match the authorized review boundary")
        reviewed_head_sha = None
    else:
        reviewed_head_sha = reviewed_head
    reviewed_tree = manifest.get("reviewedTreeSha")
    if reviewed_tree != REVIEWED_TREE:
        findings.append("reviewedTreeSha does not match the authorized review boundary")
        reviewed_tree_sha = None
    else:
        reviewed_tree_sha = reviewed_tree
    product_digest = manifest.get("productContentSha256")
    if product_digest != EXPECTED_PRODUCT_CONTENT_SHA256:
        findings.append("productContentSha256 does not match the reviewed product digest")
        product_content_sha256 = None
    else:
        product_content_sha256 = product_digest
    provenance_findings, landed_main_sha = _verify_squash_durable_content(
        root,
        reviewed_head=reviewed_head if isinstance(reviewed_head, str) else "",
        reviewed_tree=reviewed_tree if isinstance(reviewed_tree, str) else "",
        expected_content_digest=(
            product_digest if isinstance(product_digest, str) else ""
        ),
        expected_content_path_count=bound_shelf_reporter.EXPECTED_PRODUCT_PATH_COUNT,
        verify_git=verify_git,
    )
    findings.extend(provenance_findings)
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
        try:
            raw = _regular_bytes(
                Path(relative) if root_fd is not None else evidence_root / relative,
                limit=min(MAX_ARTIFACT_BYTES, MAX_TOTAL_BYTES - total),
                minimum=0,
                label=f"artifact {relative}",
                bound="evidence size bounds",
                dir_fd=root_fd,
            )
        except ValueError as exc:
            findings.append(str(exc))
            continue
        total += len(raw)
        if hashlib.sha256(raw).hexdigest() != digest:
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

    if root_fd is not None:
        actual_files = _inventory_from_descriptor(root_fd)
    else:
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
    return (
        findings,
        reviewed_head_sha,
        reviewed_tree_sha,
        product_content_sha256,
        landed_main_sha,
        passed,
        len(artifact_hashes),
    )


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
            schema="garnet.wv_acceptance_status/v2",
            wv=identifier,
            contract_base_main_sha=None,
            evidence_destination=None,
            reviewed_head_sha=None,
            reviewed_tree_sha=None,
            product_content_sha256=None,
            landed_main_sha=None,
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
    # Bind the evidence directory to one descriptor BEFORE any check, then do
    # every read relative to it. Checking the directory by pathname and
    # resolving that pathname again for the manifest, the artifacts and the
    # inventory left an ancestor swap open (review v1 finding, reproduced): a
    # swap after the check redirected the whole traversal to an outside tree and
    # the reporter returned `accepted`. O_NOFOLLOW here also subsumes the old
    # is_symlink() test, and O_DIRECTORY subsumes is_dir().
    root_fd, bind_reason = open_evidence_root(evidence_root)
    try:
        if bind_reason == "symlink" or (
            bind_reason == "unsupported" and evidence_root.is_symlink()
        ):
            findings.append("evidence destination must not be a symlink")
            state = "partial"
            reviewed_head = None
            reviewed_tree = None
            product_digest = None
            landed_main = None
            passed = 0
            artifact_count = 0
        elif bind_reason == "unbindable":
            # Exists, is not a symlink, and still will not bind as a directory.
            findings.append("evidence destination could not be bound as a directory")
            state = "partial"
            reviewed_head = None
            reviewed_tree = None
            product_digest = None
            landed_main = None
            passed = 0
            artifact_count = 0
        elif root_fd is None and not _manifest_present(evidence_root, root_fd):
            findings.append("exact-candidate evidence manifest is pending")
            state = "pending"
            reviewed_head = None
            reviewed_tree = None
            product_digest = None
            landed_main = None
            passed = 0
            artifact_count = 0
        else:
            (
                evidence_findings,
                reviewed_head,
                reviewed_tree,
                product_digest,
                landed_main,
                passed,
                artifact_count,
            ) = _validate_evidence(
                root, contract, evidence_root, verify_git=verify_git, root_fd=root_fd
            )
            findings.extend(evidence_findings)
            if evidence_findings == ["exact-candidate evidence manifest is pending"]:
                state = "pending"
            else:
                state = "accepted" if not findings else "partial"
    finally:
        if root_fd is not None:
            os.close(root_fd)

    return WvAcceptanceStatus(
        schema="garnet.wv_acceptance_status/v2",
        wv=identifier,
        contract_base_main_sha=str(contract["exactBaseMainSha"]),
        evidence_destination=destination,
        reviewed_head_sha=reviewed_head,
        reviewed_tree_sha=reviewed_tree,
        product_content_sha256=product_digest,
        landed_main_sha=landed_main,
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
