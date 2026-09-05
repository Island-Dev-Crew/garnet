#!/usr/bin/env python3
"""Adversarial tests for the attempt-1 eligibility receipt and attempt-2 verifier.

Contract: ``C_Language_Specification/GARNET_WV_ACCEPTANCE_SUCCESSION_CONTRACT.md``
(schema ``garnet.trust_kernel_review_eligibility/v1``) and the transcribed R2 block
in ``C_Language_Specification/GARNET_TRUST_KERNEL_ROLLING_REVIEW.md``.  The tests
exercise the canonical byte contract, the exact 21-key set, artifact-name
derivation, the sole eligible tuple, the strict approval-absent normalization,
predecessor-base producer-inventory binding, the in-run attempt-2 verifier,
the archive transport, and the act-4 job-multiset/census callables.
"""
from __future__ import annotations

import contextlib
import hashlib
import importlib.util
import io
import json
import os
import struct
import subprocess
import sys
import tempfile
import unittest
import zipfile
import zlib
from dataclasses import dataclass, field
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]


def _load(name: str) -> object:
    path = Path(__file__).with_name(f"{name}.py")
    spec = importlib.util.spec_from_file_location(f"_test_{name}", path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


elig = _load("garnet_trust_kernel_review_eligibility")
archive_transport = _load("garnet_actions_artifact_transport")

HEAD = "a" * 40
TREE = "b" * 40
BASE = "c" * 40
WORKFLOW_SHA = "d" * 40
RECORD_PATH = "F_Project_Management/W_TRUST/LANE1_ACT2.review.json"
RECORD_SHA = "1" * 64
INVENTORY_SHA = "2" * 64
RUN_ID = 17_000_000_001
WORKFLOW_REF = "Island-Dev-Crew/garnet/.github/workflows/ci.yml@refs/pull/547/merge"
REPO = "Island-Dev-Crew/garnet"
TOKEN = "ghs_explicit_token_value_1234567890"
MEMBER = "eligibility.json"
CI_PULL_REQUEST_JOBS = [
    "agent documentation contracts",
    "canonical MVP examples",
    "cargo doc",
    "cargo test (macos-latest)",
    "cargo test (ubuntu-latest)",
    "cargo test (windows-latest)",
    "clippy (-D warnings)",
    "machine-truth drift guard",
    "rustfmt",
]


def canonical(value: object) -> bytes:
    return (json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode()


def receipt(**overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "artifact_name": f"r2-approval-pending-{RUN_ID}-attempt-1",
        "base_ref": "main",
        "base_sha": BASE,
        "candidate_head": HEAD,
        "candidate_tree": TREE,
        "event": "pull_request",
        "finding_codes": ["approval-absent"],
        "producer_inventory_sha256": INVENTORY_SHA,
        "pull_request_id": 4395439253,
        "pull_request_number": 547,
        "repository_id": 1218183013,
        "review_record_path": RECORD_PATH,
        "review_record_sha256": RECORD_SHA,
        "run_attempt": 1,
        "run_id": RUN_ID,
        "run_number": 901,
        "schema": "garnet.trust_kernel_review_eligibility/v1",
        "state": "approval_pending_only",
        "workflow_id": 12345678,
        "workflow_ref": WORKFLOW_REF,
        "workflow_sha": WORKFLOW_SHA,
    }
    value.update(overrides)
    return value


def build_zip(
    members: list[tuple[bytes, bytes]],
    *,
    method: int = 8,
    data_descriptor: bool = False,
    flags_extra: int = 0,
    external_attr: int = (0o100644 << 16),
    made_by: int = 3 << 8,
    local_name: bytes | None = None,
    local_crc_override: int | None = None,
    comment: bytes = b"",
    prefix: bytes = b"",
    suffix: bytes = b"",
    extra: bytes = b"",
    descriptor_signature: bool = True,
) -> bytes:
    """Build a small ZIP by hand so every header field is under test control."""
    out = bytearray(prefix)
    central = bytearray()
    for index, (name, payload) in enumerate(members):
        crc = zlib.crc32(payload) & 0xFFFFFFFF
        if method == 8:
            compressor = zlib.compressobj(9, zlib.DEFLATED, -15)
            data = compressor.compress(payload) + compressor.flush()
        else:
            data = payload
        flags = (0x0008 if data_descriptor else 0) | flags_extra
        local_offset = len(out)
        header_name = local_name if (local_name is not None and index == 0) else name
        local_crc = 0 if data_descriptor else crc
        if local_crc_override is not None and index == 0:
            local_crc = local_crc_override
        out += struct.pack(
            "<IHHHHHIIIHH",
            0x04034B50,
            20,
            flags,
            method,
            0,
            0,
            local_crc,
            0 if data_descriptor else len(data),
            0 if data_descriptor else len(payload),
            len(header_name),
            len(extra),
        )
        out += header_name + extra + data
        if data_descriptor:
            if descriptor_signature:
                out += struct.pack("<I", 0x08074B50)
            out += struct.pack("<III", crc, len(data), len(payload))
        central += struct.pack(
            "<IHHHHHHIIIHHHHHII",
            0x02014B50,
            made_by | 20,
            20,
            flags,
            method,
            0,
            0,
            crc,
            len(data),
            len(payload),
            len(name),
            len(extra),
            0,
            0,
            0,
            external_attr,
            local_offset,
        )
        central += name + extra
    cd_offset = len(out)
    out += central
    out += struct.pack(
        "<IHHHHIIH",
        0x06054B50,
        0,
        0,
        len(members),
        len(members),
        len(central),
        cd_offset,
        len(comment),
    )
    out += comment + suffix
    return bytes(out)


@dataclass
class FakeProblem:
    code: str


@dataclass
class FakeObject:
    value: object = None
    problems: tuple[FakeProblem, ...] = ()
    byte_count: int = 10


@dataclass
class FakeCollection:
    rows: tuple[object, ...] = ()
    problems: tuple[FakeProblem, ...] = ()
    page_count: int = 1
    byte_count: int = 10


class FakeJsonTransport:
    """Records every endpoint and returns fixture objects/collections."""

    def __init__(self, objects: dict[str, object], collections: dict[str, list[object]]) -> None:
        self.objects = objects
        self.collections = collections
        self.calls: list[tuple[str, str]] = []

    def get_object(self, path: str) -> FakeObject:
        self.calls.append(("object", path))
        if path not in self.objects:
            return FakeObject(problems=(FakeProblem("http-status"),))
        return FakeObject(value=self.objects[path])

    def get_collection(self, path: str, *, root_key: str | None = None, require_total_count: bool = False) -> FakeCollection:
        self.calls.append(("collection", path))
        if path not in self.collections:
            return FakeCollection(problems=(FakeProblem("http-status"),))
        return FakeCollection(rows=tuple(self.collections[path]))


class FakeArchiveTransport:
    def __init__(self, archive: bytes, *, status: int = 200, host: str = "productionresultssa0.blob.core.windows.net", problems: tuple[str, ...] = ()) -> None:
        self.archive = archive
        self.status = status
        self.host = host
        self.problems = problems
        self.calls: list[int] = []

    def download_archive(self, artifact_id: int) -> object:
        self.calls.append(artifact_id)
        return archive_transport.ArchiveDownload(
            endpoint=f"actions/artifacts/{artifact_id}/zip",
            status=None if self.problems else self.status,
            final_url_host=None if self.problems else self.host,
            raw_bytes=b"" if self.problems else self.archive,
            sha256=None if self.problems else hashlib.sha256(self.archive).hexdigest(),
            problems=self.problems,
        )


class ReceiptContractTests(unittest.TestCase):
    def test_canonical_bytes_match_agents_md_byte_contract(self) -> None:
        value = {"z": 1, "a": "é", "m": [1, 2]}
        self.assertEqual(elig.canonical_bytes(value), canonical(value))
        self.assertTrue(elig.canonical_bytes(value).endswith(b"\n"))
        self.assertNotIn(b"\\u00e9", elig.canonical_bytes(value))

    def test_exact_21_keys_are_the_contract_set(self) -> None:
        self.assertEqual(len(elig.RECEIPT_KEYS), 21)
        self.assertEqual(sorted(elig.RECEIPT_KEYS), list(elig.RECEIPT_KEYS))
        self.assertEqual(set(elig.RECEIPT_KEYS), set(receipt()))
        self.assertEqual(elig.validate_receipt(receipt()), [])

    def test_missing_extra_and_duplicate_keys_are_red(self) -> None:
        missing = receipt()
        del missing["workflow_id"]
        self.assertTrue(any("exact key set" in item for item in elig.validate_receipt(missing)))
        extra = receipt(artifact_id=99)
        self.assertTrue(any("exact key set" in item for item in elig.validate_receipt(extra)))
        duplicate = canonical(receipt()).decode().replace('"event": "pull_request",', '"event": "pull_request",\n  "event": "pull_request",', 1)
        value, problems = elig.load_receipt(duplicate.encode())
        self.assertIsNone(value)
        self.assertTrue(any("duplicate" in item for item in problems))

    def test_noncanonical_bytes_are_red(self) -> None:
        good = canonical(receipt())
        self.assertEqual(elig.load_receipt(good)[1], [])
        for label, payload in (
            ("crlf", good.replace(b"\n", b"\r\n")),
            ("ascii-escaped", (json.dumps(receipt(base_ref="release/é"), ensure_ascii=True, indent=2, sort_keys=True) + "\n").encode()),
            ("trailing-bytes", good + b"\n"),
            ("missing-final-lf", good[:-1]),
            ("unsorted", (json.dumps(dict(reversed(list(receipt().items()))), ensure_ascii=False, indent=2, sort_keys=False) + "\n").encode()),
            ("four-space", (json.dumps(receipt(), ensure_ascii=False, indent=4, sort_keys=True) + "\n").encode()),
            ("not-utf8", good + b"\xff"),
            ("array-root", b"[]\n"),
        ):
            with self.subTest(label=label):
                value, problems = elig.load_receipt(payload)
                self.assertIsNone(value, label)
                self.assertTrue(problems, label)

    def test_artifact_name_is_derived_from_decimal_run_id(self) -> None:
        self.assertEqual(elig.artifact_name(RUN_ID), f"r2-approval-pending-{RUN_ID}-attempt-1")
        for bad in (0, -1, True, "17", 1.0):
            with self.subTest(bad=bad):
                with self.assertRaises(ValueError):
                    elig.artifact_name(bad)
        problems = elig.validate_receipt(receipt(artifact_name=f"r2-approval-pending-{RUN_ID + 1}-attempt-1"))
        self.assertTrue(any("artifact_name" in item for item in problems))
        problems = elig.validate_receipt(receipt(artifact_name=f"r2-approval-pending-{RUN_ID}-attempt-2"))
        self.assertTrue(any("artifact_name" in item for item in problems))

    def test_type_and_constant_invariants(self) -> None:
        cases = {
            "event": "push",
            "schema": "garnet.trust_kernel_review_eligibility/v2",
            "run_attempt": 2,
            "run_attempt_bool": True,
            "run_id": 0,
            "pull_request_id": -1,
            "pull_request_number": "547",
            "repository_id": 1.0,
            "run_number": False,
            "workflow_id": None,
            "base_sha": BASE.upper(),
            "candidate_head": HEAD[:39],
            "candidate_tree": "",
            "workflow_sha": WORKFLOW_SHA + "0",
            "producer_inventory_sha256": "sha256:" + INVENTORY_SHA,
            "review_record_sha256": RECORD_SHA[:63],
            "review_record_path": "F_Project_Management/W_TRUST/landed/X.landed-review.json",
            "review_record_path_traversal": "F_Project_Management/W_TRUST/../X.review.json",
            "review_record_path_backslash": "F_Project_Management\\W_TRUST\\X.review.json",
            "workflow_ref": ".github/workflows/ci.yml@refs/pull/1/merge",
            "workflow_ref_other_file": "Island-Dev-Crew/garnet/.github/workflows/security.yml@refs/pull/1/merge",
            "base_ref": "",
            "base_ref_control": "main\n",
            "state": "eligible",
            "finding_codes": "approval-absent",
            "finding_codes_unsorted": ["b-code", "a-code"],
            "finding_codes_duplicate": ["a-code", "a-code"],
            "finding_codes_shape": ["Approval Absent"],
        }
        for label, value in cases.items():
            key = label.split("_bool")[0].split("_traversal")[0].split("_backslash")[0].split("_other_file")[0].split("_control")[0].split("_unsorted")[0].split("_duplicate")[0].split("_shape")[0]
            with self.subTest(label=label):
                bad = receipt(**{key: value})
                if key == "finding_codes" and isinstance(value, list) and value != ["approval-absent"]:
                    bad["state"] = "ineligible"
                self.assertTrue(elig.validate_receipt(bad), label)

    def test_sole_eligible_tuple(self) -> None:
        self.assertTrue(elig.is_eligible_tuple("approval_pending_only", ["approval-absent"]))
        for state, codes in (
            ("ineligible", ["approval-absent"]),
            ("approval_pending_only", []),
            ("approval_pending_only", ["approval-absent", "review-record-missing"]),
            ("approval_pending_only", ["approval-not-at-head"]),
            ("ineligible", []),
            ("APPROVAL_PENDING_ONLY", ["approval-absent"]),
        ):
            with self.subTest(state=state, codes=codes):
                self.assertFalse(elig.is_eligible_tuple(state, codes))
        # state/code consistency is enforced inside the receipt itself
        self.assertTrue(elig.validate_receipt(receipt(state="ineligible", finding_codes=["approval-absent"])))
        self.assertTrue(elig.validate_receipt(receipt(state="approval_pending_only", finding_codes=[])))
        self.assertEqual(elig.validate_receipt(receipt(state="ineligible", finding_codes=[])), [])
        self.assertEqual(elig.validate_receipt(receipt(state="ineligible", finding_codes=["approval-not-at-head"])), [])

    def test_strict_approval_absent_normalization_table(self) -> None:
        exact = "authenticated decisive review from the recorded independent reviewer is absent"
        self.assertEqual(elig.classify_problems([exact]), ["approval-absent"])
        self.assertEqual(elig.receipt_state(["approval-absent"]), "approval_pending_only")
        not_at_head = (
            "latest decisive review from the recorded independent reviewer must be "
            "APPROVED at the exact current candidate head"
        )
        self.assertEqual(elig.classify_problems([not_at_head]), ["approval-not-at-head"])
        for variant in (
            exact + " ",
            " " + exact,
            exact.upper(),
            exact.replace("absent", "missing"),
            exact + "; also stale",
            "structured review record is missing",
            "reviewer identity overlaps an authenticated commit principal",
            "authenticated review enumeration failed closed: rate-limit",
            "some future reporter finding",
        ):
            with self.subTest(variant=variant):
                codes = elig.classify_problems([variant])
                self.assertNotIn("approval-absent", codes)
                self.assertEqual(elig.receipt_state(codes), "ineligible")
        self.assertEqual(elig.classify_problems(["some future reporter finding"]), ["unclassified-finding"])
        self.assertEqual(elig.classify_problems([]), [])
        self.assertEqual(elig.receipt_state([]), "ineligible")
        mixed = elig.classify_problems([exact, "structured review record is missing", exact])
        self.assertEqual(mixed, sorted(set(mixed)))
        self.assertIn("approval-absent", mixed)
        self.assertEqual(elig.receipt_state(mixed), "ineligible")
        for code in elig.classify_problems(["structured review record is missing", not_at_head, "x"]):
            self.assertRegex(code, r"^[a-z0-9]+(?:-[a-z0-9]+)*$")

    def test_every_known_reporter_finding_has_a_non_approval_code(self) -> None:
        reporter = (ROOT / "scripts/garnet_trust_kernel_review_status.py").read_text(encoding="utf-8")
        exact = "authenticated decisive review from the recorded independent reviewer is absent"
        self.assertIn(exact, reporter)
        self.assertEqual(elig.APPROVAL_ABSENT_PROBLEM, exact)
        for entry in elig.FINDING_CODE_TABLE:
            self.assertEqual(len(entry), 3)
            kind, pattern, code = entry
            self.assertIn(kind, {"exact", "prefix"})
            self.assertRegex(code, r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
            if code == "approval-absent":
                self.assertEqual((kind, pattern), ("exact", exact))


class ProducerInventoryTests(unittest.TestCase):
    def test_digest_is_over_raw_bytes_and_requires_schema_v2(self) -> None:
        raw = (ROOT / ".github/rulesets/required-context-producers.json").read_bytes()
        digest, problems = elig.producer_inventory_digest(raw)
        self.assertEqual(problems, [])
        self.assertEqual(digest, hashlib.sha256(raw).hexdigest())
        altered = json.loads(raw)
        altered["schema"] = "garnet.required-context-producers/v1"
        digest, problems = elig.producer_inventory_digest(json.dumps(altered).encode())
        self.assertIsNone(digest)
        self.assertTrue(any("schema" in item for item in problems))
        digest, problems = elig.producer_inventory_digest(b'{"schema": 1, "schema": 2}')
        self.assertIsNone(digest)
        self.assertTrue(problems)

    def test_expected_job_multiset_is_the_nine_ci_pull_request_rows(self) -> None:
        raw = (ROOT / ".github/rulesets/required-context-producers.json").read_bytes()
        names = elig.expected_job_multiset(raw, ".github/workflows/ci.yml", "pull_request")
        self.assertEqual(names, CI_PULL_REQUEST_JOBS)
        self.assertEqual(elig.expected_job_multiset(raw, ".github/workflows/ci.yml", "push"), [])
        self.assertEqual(
            elig.expected_job_multiset(raw, ".github/workflows/base-controlled-trust.yml", "pull_request_target"),
            ["Base-controlled trust policy"],
        )
        with self.assertRaises(ValueError):
            elig.expected_job_multiset(b'{"schema": "garnet.required-context-producers/v1", "producers": []}', ".github/workflows/ci.yml", "pull_request")

    def test_inventory_digest_binds_the_predecessor_base_not_the_candidate(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            repo = Path(temp).resolve()
            git = _GitRepo(repo)
            base_inventory = canonical({"schema": "garnet.required-context-producers/v2", "target_branch": "main", "optional_contexts": [], "producers": []})
            git.commit_file(".github/rulesets/required-context-producers.json", base_inventory, "base")
            base = git.rev("HEAD")
            git.run("update-ref", "refs/remotes/origin/main", base)
            candidate_inventory = canonical({"schema": "garnet.required-context-producers/v2", "target_branch": "main", "optional_contexts": [], "producers": [], "note": "candidate"})
            git.commit_file(".github/rulesets/required-context-producers.json", candidate_inventory, "candidate")
            head = git.rev("HEAD")
            digest, problems = elig.predecessor_inventory_digest(repo, head)
            self.assertEqual(problems, [])
            self.assertEqual(digest, hashlib.sha256(base_inventory).hexdigest())
            self.assertNotEqual(digest, hashlib.sha256(candidate_inventory).hexdigest())


class _GitRepo:
    def __init__(self, root: Path) -> None:
        self.root = root
        self.run("init", "-q")
        self.run("config", "core.autocrlf", "false")
        self.run("config", "user.email", "author@example.invalid")
        self.run("config", "user.name", "Author")
        self.run("commit", "-q", "--allow-empty", "-m", "genesis")
        self.run("branch", "-M", "main")

    def run(self, *args: str) -> str:
        result = subprocess.run(["git", *args], cwd=self.root, capture_output=True, text=True, check=False)
        if result.returncode != 0:
            raise AssertionError(f"git {' '.join(args)} failed: {result.stderr or result.stdout}")
        return result.stdout.strip()

    def rev(self, ref: str) -> str:
        return self.run("rev-parse", "--verify", ref)

    def commit_file(self, relative: str, content: bytes, message: str) -> str:
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
        self.run("add", relative)
        self.run("commit", "-q", "-m", message)
        return self.rev("HEAD")


class ZipMemberTests(unittest.TestCase):
    PAYLOAD = canonical(receipt())

    def test_single_deflated_and_stored_member_are_accepted(self) -> None:
        for method in (8, 0):
            for descriptor in (False, True):
                for signature in (True, False):
                    with self.subTest(method=method, descriptor=descriptor, signature=signature):
                        archive = build_zip([(b"eligibility.json", self.PAYLOAD)], method=method, data_descriptor=descriptor, descriptor_signature=signature)
                        member, problems = elig.parse_single_member_zip(archive)
                        self.assertEqual(problems, [])
                        self.assertEqual(member, self.PAYLOAD)

    def test_hostile_archives_are_red(self) -> None:
        good = build_zip([(b"eligibility.json", self.PAYLOAD)])
        cases = {
            "extra-member": build_zip([(b"eligibility.json", self.PAYLOAD), (b"other.json", b"{}\n")]),
            "duplicate-member": build_zip([(b"eligibility.json", self.PAYLOAD), (b"eligibility.json", self.PAYLOAD)]),
            "directory-entry": build_zip([(b"eligibility.json/", b"")], external_attr=(0o40755 << 16) | 0x10),
            "traversal": build_zip([(b"../eligibility.json", self.PAYLOAD)]),
            "backslash": build_zip([(b"r2\\eligibility.json", self.PAYLOAD)]),
            "absolute": build_zip([(b"/eligibility.json", self.PAYLOAD)]),
            "dot-component": build_zip([(b"./eligibility.json", self.PAYLOAD)]),
            "empty-component": build_zip([(b"r2//eligibility.json", self.PAYLOAD)]),
            "wrong-name": build_zip([(b"eligibility.JSON", self.PAYLOAD)]),
            "nested-name": build_zip([(b"r2/eligibility.json", self.PAYLOAD)]),
            "encrypted": build_zip([(b"eligibility.json", self.PAYLOAD)], flags_extra=0x0001),
            "strong-encrypted": build_zip([(b"eligibility.json", self.PAYLOAD)], flags_extra=0x0040),
            "symlink": build_zip([(b"eligibility.json", b"target")], external_attr=(0o120777 << 16)),
            "device": build_zip([(b"eligibility.json", self.PAYLOAD)], external_attr=(0o20644 << 16)),
            "header-name-disagreement": build_zip([(b"eligibility.json", self.PAYLOAD)], local_name=b"eligibility.jsom"),
            "header-crc-disagreement": build_zip([(b"eligibility.json", self.PAYLOAD)], local_crc_override=0x12345678),
            "zip64-extra": build_zip([(b"eligibility.json", self.PAYLOAD)], extra=struct.pack("<HH", 0x0001, 0)),
            "prefix-bytes": build_zip([(b"eligibility.json", self.PAYLOAD)], prefix=b"GARBAGE"),
            "trailing-bytes": good + b"\x00",
            "comment": build_zip([(b"eligibility.json", self.PAYLOAD)], comment=b"c"),
            "empty-archive": build_zip([]),
            "not-a-zip": b"PK\x03\x04garbage",
            "empty": b"",
            "unsupported-method": build_zip([(b"eligibility.json", self.PAYLOAD)], method=12),
        }
        for label, archive in cases.items():
            with self.subTest(label=label):
                member, problems = elig.parse_single_member_zip(archive)
                self.assertIsNone(member, label)
                self.assertTrue(problems, label)

    def test_real_writer_archives_interoperate_and_non_regular_types_stay_red(self) -> None:
        """A member whose Unix mode omits the type bits is a regular file.

        Python's own zipfile writer emits external_attr without S_IFREG, and so
        do several other producers. Rejecting that shape would fail closed in a
        way that permanently disables U-59, so it must be accepted while every
        positively declared non-regular type stays RED.
        """
        payload = canonical(receipt())
        for method, label in ((zipfile.ZIP_DEFLATED, "deflate"), (zipfile.ZIP_STORED, "stored")):
            with self.subTest(writer=label):
                buffer = io.BytesIO()
                with zipfile.ZipFile(buffer, "w", method) as archive:
                    archive.writestr(MEMBER, payload)
                member, problems = elig.parse_single_member_zip(buffer.getvalue())
                self.assertEqual(problems, [], label)
                self.assertEqual(member, payload, label)
        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, "w") as archive:
            info = zipfile.ZipInfo(MEMBER)
            info.external_attr = 0
            archive.writestr(info, payload)
        member, problems = elig.parse_single_member_zip(buffer.getvalue())
        self.assertEqual(problems, [])
        self.assertEqual(member, payload)
        for mode, label in ((0o120777, "symlink"), (0o040755, "directory-mode"), (0o020644, "device")):
            with self.subTest(mode=label):
                buffer = io.BytesIO()
                with zipfile.ZipFile(buffer, "w") as archive:
                    info = zipfile.ZipInfo(MEMBER)
                    info.external_attr = mode << 16
                    archive.writestr(info, payload)
                member, problems = elig.parse_single_member_zip(buffer.getvalue())
                self.assertIsNone(member, label)
                self.assertTrue(problems, label)

    def test_corrupt_deflate_stream_and_size_lies_are_red(self) -> None:
        archive = bytearray(build_zip([(b"eligibility.json", self.PAYLOAD)]))
        # flip a byte inside the compressed payload (after the 30-byte local header + name)
        index = 30 + len(b"eligibility.json") + 5
        archive[index] ^= 0xFF
        member, problems = elig.parse_single_member_zip(bytes(archive))
        self.assertIsNone(member)
        self.assertTrue(problems)


class ExpectedJobsAndCensusTests(unittest.TestCase):
    INVENTORY = (ROOT / ".github/rulesets/required-context-producers.json").read_bytes()
    CI_WORKFLOW_ID = 5001

    def _jobs(self, attempt: int, names: list[str], *, first_id: int) -> list[dict[str, object]]:
        return [
            {
                "id": first_id + index,
                "run_id": RUN_ID,
                "run_attempt": attempt,
                "head_sha": HEAD,
                "name": name,
                "status": "completed",
                "conclusion": "success",
                "started_at": "2026-09-02T00:00:00Z",
                "completed_at": "2026-09-02T00:05:00Z",
            }
            for index, name in enumerate(names)
        ]

    def _workflow_ids(self) -> dict[str, int]:
        rows = json.loads(self.INVENTORY)["producers"]
        paths = sorted({row["workflow"] for row in rows})
        return {path: self.CI_WORKFLOW_ID + index for index, path in enumerate(paths, start=1)} | {".github/workflows/ci.yml": self.CI_WORKFLOW_ID}

    def _runs(self, workflow_ids: dict[str, int], *, ci_attempt: int = 2, other_attempt: int = 1) -> list[dict[str, object]]:
        runs = []
        for index, (path, workflow_id) in enumerate(sorted(workflow_ids.items())):
            if path == ".github/workflows/base-controlled-trust.yml":
                continue
            is_ci = path == ".github/workflows/ci.yml"
            runs.append({
                "id": RUN_ID if is_ci else RUN_ID + 100 + index,
                "workflow_id": workflow_id,
                "run_attempt": ci_attempt if is_ci else other_attempt,
                "event": "pull_request",
                "head_sha": HEAD,
            })
        return runs

    def _verify(self, **overrides: object) -> list[str]:
        workflow_ids = self._workflow_ids()
        kwargs: dict[str, object] = {
            "inventory_bytes": self.INVENTORY,
            "workflow_path": ".github/workflows/ci.yml",
            "event": "pull_request",
            "run_id": RUN_ID,
            "head_sha": HEAD,
            "workflow_id": self.CI_WORKFLOW_ID,
            "attempt1_jobs": self._jobs(1, CI_PULL_REQUEST_JOBS, first_id=100),
            "attempt2_jobs": self._jobs(2, CI_PULL_REQUEST_JOBS, first_id=200),
            "head_runs": self._runs(workflow_ids),
            "producer_workflow_ids": workflow_ids,
        }
        kwargs.update(overrides)
        return elig.verify_jobs_and_census(**kwargs)

    def test_complete_fresh_attempt2_is_green(self) -> None:
        self.assertEqual(self._verify(), [])

    def test_seven_row_first_attempt_with_an_unexpanded_matrix_is_accepted(self) -> None:
        """Review v1 (F3): the deliberate first-attempt RED skips the
        downstream matrix before expansion, so attempt 1 has seven rows
        including the placeholder; the callable demanded nine."""
        names = [name for name in CI_PULL_REQUEST_JOBS if not name.startswith("cargo test (")]
        names += ["cargo test (${{ matrix.os }})"]
        self.assertEqual(len(names), 7)
        self.assertEqual(self._verify(attempt1_jobs=self._jobs(1, names, first_id=100)), [])

    def test_an_empty_first_attempt_census_is_red(self) -> None:
        problems = self._verify(attempt1_jobs=[])
        self.assertTrue(any("attempt-1 job census is empty" in item for item in problems), problems)

    def test_matrix_placeholder_does_not_satisfy_the_expanded_multiset(self) -> None:
        names = [name for name in CI_PULL_REQUEST_JOBS if not name.startswith("cargo test (")]
        names += ["cargo test (${{ matrix.os }})"] * 3
        problems = self._verify(attempt2_jobs=self._jobs(2, names, first_id=200))
        self.assertTrue(any("multiset" in item for item in problems), problems)

    def test_missing_duplicate_reused_and_unsuccessful_jobs_are_red(self) -> None:
        good = self._jobs(2, CI_PULL_REQUEST_JOBS, first_id=200)
        missing = good[:-1]
        duplicate = good + [dict(good[0], id=999)]
        reused = [dict(row, id=100 + index) for index, row in enumerate(good)]
        failed = [dict(row, conclusion="failure") if index == 0 else row for index, row in enumerate(good)]
        skipped = [dict(row, conclusion="skipped") if index == 0 else row for index, row in enumerate(good)]
        in_progress = [dict(row, status="in_progress") if index == 0 else row for index, row in enumerate(good)]
        wrong_attempt = [dict(row, run_attempt=1) if index == 0 else row for index, row in enumerate(good)]
        wrong_run = [dict(row, run_id=RUN_ID + 1) if index == 0 else row for index, row in enumerate(good)]
        wrong_head = [dict(row, head_sha="f" * 40) if index == 0 else row for index, row in enumerate(good)]
        no_timestamps = [dict(row, started_at="") if index == 0 else row for index, row in enumerate(good)]
        extra = good + [dict(good[0], id=998, name="unexpected job")]
        for label, rows in (
            ("missing", missing), ("duplicate", duplicate), ("reused", reused), ("failed", failed),
            ("skipped", skipped), ("in-progress", in_progress), ("wrong-attempt", wrong_attempt),
            ("wrong-run", wrong_run), ("wrong-head", wrong_head), ("no-timestamps", no_timestamps), ("extra", extra),
        ):
            with self.subTest(label=label):
                self.assertTrue(self._verify(attempt2_jobs=rows), label)

    def test_cross_workflow_census_requires_other_producers_at_attempt_one(self) -> None:
        workflow_ids = self._workflow_ids()
        self.assertTrue(self._verify(head_runs=self._runs(workflow_ids, other_attempt=2)))
        self.assertTrue(self._verify(head_runs=self._runs(workflow_ids, ci_attempt=1)))
        self.assertTrue(self._verify(head_runs=self._runs(workflow_ids, ci_attempt=3)))
        runs = self._runs(workflow_ids)
        self.assertTrue(self._verify(head_runs=[row for row in runs if row["id"] != RUN_ID]))
        self.assertTrue(self._verify(head_runs=runs + [dict(runs[0], id=RUN_ID + 5000)]))
        self.assertTrue(self._verify(head_runs=[row for row in runs if row["workflow_id"] != workflow_ids[".github/workflows/security.yml"]]))
        self.assertTrue(self._verify(head_runs=[dict(row, head_sha="f" * 40) for row in runs]))
        self.assertTrue(self._verify(producer_workflow_ids={k: v for k, v in workflow_ids.items() if k != ".github/workflows/security.yml"}))


class AttemptTwoVerifierTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.repo = Path(self.temp.name).resolve()
        git = _GitRepo(self.repo)
        self.inventory = (ROOT / ".github/rulesets/required-context-producers.json").read_bytes()
        git.commit_file(".github/rulesets/required-context-producers.json", self.inventory, "base")
        self.base = git.rev("HEAD")
        git.run("update-ref", "refs/remotes/origin/main", self.base)
        self.record_bytes = canonical({"author_ids": [101], "reviewer_id": 202, "schema": "garnet.trust_kernel_review_record/v2", "state": "premerge"})
        git.commit_file(RECORD_PATH, self.record_bytes, "record")
        self.head = git.rev("HEAD")
        self.tree = git.rev("HEAD^{tree}")
        self.git = git
        self.constants = elig.RunConstants(
            repository=REPO,
            repository_id=1218183013,
            pull_request_id=4395439253,
            pull_request_number=547,
            base_ref="main",
            base_sha=self.base,
            head_sha=self.head,
            run_id=RUN_ID,
            run_number=901,
            run_attempt=2,
            workflow_ref=WORKFLOW_REF,
            workflow_sha=WORKFLOW_SHA,
        )

    def tearDown(self) -> None:
        self.temp.cleanup()

    def _receipt(self, **overrides: object) -> dict[str, object]:
        value = receipt(
            base_sha=self.base,
            candidate_head=self.head,
            candidate_tree=self.tree,
            review_record_sha256=hashlib.sha256(self.record_bytes).hexdigest(),
            producer_inventory_sha256=hashlib.sha256(self.inventory).hexdigest(),
        )
        value.update(overrides)
        return value

    def _artifact(self, **overrides: object) -> dict[str, object]:
        value: dict[str, object] = {
            "id": 777,
            "name": elig.artifact_name(RUN_ID),
            "size_in_bytes": 1234,
            "expired": False,
            "created_at": "2026-09-02T00:00:00Z",
            "workflow_run": {"id": RUN_ID, "repository_id": 1218183013, "head_sha": self.head},
        }
        value.update(overrides)
        return value

    def _transport(self, *, artifacts: list[dict[str, object]] | None = None, run: dict[str, object] | None = None, pull: dict[str, object] | None = None) -> FakeJsonTransport:
        run_object = {
            "id": RUN_ID,
            "run_number": 901,
            "run_attempt": 2,
            "event": "pull_request",
            "head_sha": self.head,
            "workflow_id": 12345678,
            "path": ".github/workflows/ci.yml",
            "repository": {"id": 1218183013, "full_name": REPO},
            # The attempt-2 carrier: the principal that re-ran the workflow,
            # as the API reports it. Distinct from the author (101) and the
            # reviewer (202) by construction here; F1 tests vary it.
            "triggering_actor": {"id": 303, "login": "carrier"},
        }
        if run:
            run_object.update(run)
        pull_object = {
            "id": 4395439253,
            "number": 547,
            "state": "open",
            "draft": False,
            "head": {"sha": self.head},
            "base": {"ref": "main", "sha": self.base, "repo": {"id": 1218183013, "full_name": REPO}},
        }
        if pull:
            pull_object.update(pull)
        return FakeJsonTransport(
            {f"actions/runs/{RUN_ID}": run_object, "pulls/547": pull_object},
            {f"actions/runs/{RUN_ID}/artifacts": [self._artifact()] if artifacts is None else artifacts},
        )

    def _verify(self, *, receipt_value: dict[str, object] | None = None, archive: bytes | None = None, transport: FakeJsonTransport | None = None, archive_transport_override: FakeArchiveTransport | None = None, constants: object | None = None) -> object:
        value = self._receipt() if receipt_value is None else receipt_value
        payload = archive if archive is not None else build_zip([(b"eligibility.json", canonical(value))])
        return elig.verify_attempt2(
            transport=self._transport() if transport is None else transport,
            archive_transport=FakeArchiveTransport(payload) if archive_transport_override is None else archive_transport_override,
            root=self.repo,
            constants=self.constants if constants is None else constants,
        )

    def test_exact_attempt2_receipt_is_green_and_binds_transport_evidence(self) -> None:
        verdict = self._verify()
        self.assertEqual(verdict.problems, (), verdict.problems)
        self.assertTrue(verdict.ok)
        self.assertEqual(verdict.artifact_id, 777)
        self.assertEqual(verdict.artifact_name, elig.artifact_name(RUN_ID))
        self.assertEqual(verdict.archive_endpoint, "actions/artifacts/777/zip")
        self.assertEqual(verdict.archive_status, 200)
        self.assertEqual(verdict.raw_body_sha256, verdict.archive_sha256)
        self.assertEqual(verdict.artifact_created_at, "2026-09-02T00:00:00Z")
        self.assertEqual(verdict.receipt_state, "approval_pending_only")
        self.assertEqual(verdict.receipt_finding_codes, ["approval-absent"])
        self.assertEqual(verdict.candidate_head, self.head)
        document = json.loads(elig.render_verdict(verdict))
        self.assertEqual(document["schema"], elig.VERDICT_SCHEMA)
        self.assertEqual(elig.render_verdict(verdict), canonical(document).decode())

    def test_green_verdict_carries_the_carrier_identity(self) -> None:
        verdict = self._verify()
        self.assertTrue(verdict.ok, verdict.problems)
        self.assertEqual(verdict.carrier_id, 303)
        self.assertEqual(json.loads(elig.render_verdict(verdict))["carrier_id"], 303)

    def test_attempt2_carrier_must_exist_and_be_neither_author_nor_reviewer(self) -> None:
        """Review v1 (F1): the verifier accepted a run with no triggering
        actor, a carrier equal to the reviewer, and a carrier in the author
        set — r2_role_separation_v1 was not enforced anywhere."""
        cases = {
            "missing": ({"triggering_actor": None}, "no authenticated triggering_actor"),
            "not-an-object": ({"triggering_actor": "carrier"}, "no authenticated triggering_actor"),
            "no-id": ({"triggering_actor": {"login": "carrier"}}, "no authenticated triggering_actor"),
            "reviewer": ({"triggering_actor": {"id": 202, "login": "independent-reviewer"}}, "is the reviewer"),
            "author": ({"triggering_actor": {"id": 101, "login": "author"}}, "is in the candidate's author set"),
        }
        for label, (run, fragment) in cases.items():
            with self.subTest(label=label):
                verdict = self._verify(transport=self._transport(run=run))
                self.assertFalse(verdict.ok, label)
                self.assertTrue(any(fragment in item for item in verdict.problems), (label, verdict.problems))
                self.assertIsNone(verdict.carrier_id if label != "reviewer" and label != "author" else None)

    def test_a_record_without_principals_cannot_separate_the_carrier(self) -> None:
        bare = canonical({"schema": "garnet.trust_kernel_review_record/v2", "state": "premerge"})
        self.assertEqual(
            ["review record does not name its author set and reviewer, so the carrier cannot be separated"],
            elig._carrier_separation_problems(bare, 303),
        )
        self.assertEqual(["review record could not be read for carrier separation"], elig._carrier_separation_problems(b"{", 303))
        self.assertEqual([], elig._carrier_separation_problems(self.record_bytes, 303))

    def test_zero_two_expired_wrong_name_and_wrong_run_artifacts_are_red(self) -> None:
        cases = {
            "zero": [],
            "two": [self._artifact(), self._artifact(id=778)],
            "two-one-expired": [self._artifact(), self._artifact(id=778, expired=True)],
            "expired": [self._artifact(expired=True)],
            "wrong-name": [self._artifact(name=elig.artifact_name(RUN_ID + 1))],
            "attempt-2-name": [self._artifact(name=f"r2-approval-pending-{RUN_ID}-attempt-2")],
            "wrong-run": [self._artifact(workflow_run={"id": RUN_ID + 1, "repository_id": 1218183013, "head_sha": self.head})],
            "wrong-repository": [self._artifact(workflow_run={"id": RUN_ID, "repository_id": 1, "head_sha": self.head})],
            "non-positive-id": [self._artifact(id=0)],
            "string-id": [self._artifact(id="777")],
        }
        for label, artifacts in cases.items():
            with self.subTest(label=label):
                verdict = self._verify(transport=self._transport(artifacts=artifacts))
                self.assertFalse(verdict.ok, label)
                self.assertTrue(verdict.problems, label)

    def test_pagination_failure_is_red(self) -> None:
        transport = self._transport()
        transport.collections = {}
        verdict = self._verify(transport=transport)
        self.assertFalse(verdict.ok)
        self.assertTrue(any("artifact enumeration" in item for item in verdict.problems), verdict.problems)

    def test_api_digest_and_archive_sha256_disagreement_is_red(self) -> None:
        good_archive = build_zip([(b"eligibility.json", canonical(self._receipt()))])
        matching = self._transport(artifacts=[self._artifact(digest="sha256:" + hashlib.sha256(good_archive).hexdigest())])
        self.assertTrue(self._verify(archive=good_archive, transport=matching).ok)
        wrong = self._transport(artifacts=[self._artifact(digest="sha256:" + "0" * 64)])
        verdict = self._verify(archive=good_archive, transport=wrong)
        self.assertFalse(verdict.ok)
        self.assertTrue(any("digest" in item for item in verdict.problems), verdict.problems)
        malformed = self._transport(artifacts=[self._artifact(digest="md5:abc")])
        self.assertFalse(self._verify(archive=good_archive, transport=malformed).ok)

    def test_archive_transport_failure_and_non_200_are_red(self) -> None:
        archive = build_zip([(b"eligibility.json", canonical(self._receipt()))])
        failed = FakeArchiveTransport(archive, problems=("transport-failure",))
        self.assertFalse(self._verify(archive_transport_override=failed).ok)
        redirected = FakeArchiveTransport(archive, status=302)
        self.assertFalse(self._verify(archive_transport_override=redirected).ok)

    def test_hostile_zip_members_are_red(self) -> None:
        payload = canonical(self._receipt())
        for label, archive in (
            ("extra-member", build_zip([(b"eligibility.json", payload), (b"x", b"")])),
            ("wrong-name", build_zip([(b"receipt.json", payload)])),
            ("encrypted", build_zip([(b"eligibility.json", payload)], flags_extra=1)),
            ("noncanonical", build_zip([(b"eligibility.json", payload.replace(b"\n", b"\r\n"))])),
            ("empty-member", build_zip([(b"eligibility.json", b"")])),
        ):
            with self.subTest(label=label):
                self.assertFalse(self._verify(archive=archive).ok, label)

    def test_run_constant_mismatches_are_red(self) -> None:
        cases = {
            "repository_id": {"repository_id": 1},
            "pull_request_id": {"pull_request_id": 2},
            "pull_request_number": {"pull_request_number": 548},
            "base_ref": {"base_ref": "release"},
            "base_sha": {"base_sha": "e" * 40},
            "candidate_head": {"candidate_head": "e" * 40},
            "candidate_tree": {"candidate_tree": "e" * 40},
            "review_record_path": {"review_record_path": "F_Project_Management/W_TRUST/OTHER.review.json"},
            "review_record_sha256": {"review_record_sha256": "3" * 64},
            "producer_inventory_sha256": {"producer_inventory_sha256": "4" * 64},
            "run_id": {"run_id": RUN_ID + 1, "artifact_name": elig.artifact_name(RUN_ID + 1)},
            "run_number": {"run_number": 902},
            "workflow_id": {"workflow_id": 1},
            "workflow_ref": {"workflow_ref": WORKFLOW_REF.replace("547", "548")},
            "workflow_sha": {"workflow_sha": "e" * 40},
            "event": {"event": "push"},
            "run_attempt": {"run_attempt": 2},
            "ineligible-state": {"state": "ineligible", "finding_codes": ["review-record-missing"]},
            "ineligible-codes": {"state": "ineligible", "finding_codes": ["approval-absent"]},
        }
        for label, overrides in cases.items():
            with self.subTest(label=label):
                verdict = self._verify(receipt_value=self._receipt(**overrides))
                self.assertFalse(verdict.ok, label)

    def test_live_pull_request_and_run_divergence_are_red(self) -> None:
        for label, transport in (
            ("closed", self._transport(pull={"state": "closed"})),
            ("draft", self._transport(pull={"draft": True})),
            ("head-moved", self._transport(pull={"head": {"sha": "e" * 40}})),
            ("base-moved", self._transport(pull={"base": {"ref": "main", "sha": "e" * 40, "repo": {"id": 1218183013, "full_name": REPO}}})),
            ("retargeted", self._transport(pull={"base": {"ref": "release", "sha": self.base, "repo": {"id": 1218183013, "full_name": REPO}}})),
            ("run-attempt-1", self._transport(run={"run_attempt": 1})),
            ("run-attempt-3", self._transport(run={"run_attempt": 3})),
            ("run-event", self._transport(run={"event": "workflow_dispatch"})),
            ("run-head", self._transport(run={"head_sha": "e" * 40})),
            ("run-workflow", self._transport(run={"workflow_id": 1})),
            ("run-path", self._transport(run={"path": ".github/workflows/security.yml"})),
            ("run-number", self._transport(run={"run_number": 1})),
        ):
            with self.subTest(label=label):
                verdict = self._verify(transport=transport)
                self.assertFalse(verdict.ok, label)
        missing = self._transport()
        missing.objects = {}
        self.assertFalse(self._verify(transport=missing).ok)

    def test_attempt_three_and_attempt_one_are_red_before_any_transport(self) -> None:
        for attempt in (1, 3, 4):
            with self.subTest(attempt=attempt):
                transport = self._transport()
                constants = elig.RunConstants(**{**self.constants.__dict__, "run_attempt": attempt})
                verdict = self._verify(transport=transport, constants=constants)
                self.assertFalse(verdict.ok)
                self.assertEqual(transport.calls, [])
                self.assertTrue(any("attempt" in item for item in verdict.problems))

    def test_verify_cli_writes_verdict_and_reads_token_only_from_stdin(self) -> None:
        archive = build_zip([(b"eligibility.json", canonical(self._receipt()))])
        transport = self._transport()
        seen: list[tuple[str, str]] = []

        def json_factory(repository: str, token: str) -> FakeJsonTransport:
            seen.append((repository, token))
            return transport

        def archive_factory(repository: str, token: str) -> FakeArchiveTransport:
            seen.append((repository, token))
            return FakeArchiveTransport(archive)

        out = self.repo / "verdict.json"
        argv = [
            "verify", "--root", str(self.repo), "--verdict-out", str(out), "--github-repo", REPO, "--github-token-stdin",
            "--repository-id", "1218183013", "--pull-request-id", "4395439253", "--pull-request-number", "547",
            "--base-ref", "main", "--base-sha", self.base, "--head-sha", self.head, "--run-id", str(RUN_ID),
            "--run-number", "901", "--run-attempt", "2", "--workflow-ref", WORKFLOW_REF, "--workflow-sha", WORKFLOW_SHA,
        ]
        with mock.patch.dict(os.environ, {"GITHUB_TOKEN": "ambient", "GH_TOKEN": "ambient"}):
            with contextlib.redirect_stdout(io.StringIO()) as stdout:
                code = elig.main(argv, stdin=io.BytesIO(TOKEN.encode() + b"\n"), transport_factory=json_factory, archive_transport_factory=archive_factory)
        self.assertEqual(code, 0, stdout.getvalue())
        self.assertEqual(seen, [(REPO, TOKEN), (REPO, TOKEN)])
        self.assertNotIn("ambient", stdout.getvalue())
        self.assertNotIn(TOKEN, stdout.getvalue())
        document = json.loads(out.read_bytes())
        self.assertTrue(document["ok"])
        self.assertEqual(out.read_bytes(), canonical(document))
        with contextlib.redirect_stdout(io.StringIO()):
            gated = elig.main([*argv, "--gate"], stdin=io.BytesIO(TOKEN.encode()), transport_factory=json_factory, archive_transport_factory=archive_factory)
        self.assertEqual(gated, 0)
        transport.collections = {}
        with contextlib.redirect_stdout(io.StringIO()):
            red = elig.main([*argv, "--gate"], stdin=io.BytesIO(TOKEN.encode()), transport_factory=json_factory, archive_transport_factory=archive_factory)
        self.assertEqual(red, 1)
        self.assertFalse(json.loads(out.read_bytes())["ok"])

    def test_verify_cli_without_token_is_red(self) -> None:
        out = self.repo / "verdict.json"
        argv = [
            "verify", "--root", str(self.repo), "--verdict-out", str(out), "--github-repo", REPO,
            "--repository-id", "1", "--pull-request-id", "1", "--pull-request-number", "1", "--base-ref", "main",
            "--base-sha", self.base, "--head-sha", self.head, "--run-id", str(RUN_ID), "--run-number", "1",
            "--run-attempt", "2", "--workflow-ref", WORKFLOW_REF, "--workflow-sha", WORKFLOW_SHA,
        ]
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            code = elig.main(argv, stdin=io.BytesIO(b""), transport_factory=lambda r, t: None, archive_transport_factory=lambda r, t: None)
        self.assertNotEqual(code, 0)
        self.assertFalse(out.exists())


class EmitTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.repo = Path(self.temp.name).resolve()
        git = _GitRepo(self.repo)
        self.inventory = (ROOT / ".github/rulesets/required-context-producers.json").read_bytes()
        git.commit_file(".github/rulesets/required-context-producers.json", self.inventory, "base")
        self.base = git.rev("HEAD")
        git.run("update-ref", "refs/remotes/origin/main", self.base)
        self.record_bytes = canonical({"author_ids": [101], "reviewer_id": 202, "schema": "garnet.trust_kernel_review_record/v2", "state": "premerge"})
        git.commit_file(RECORD_PATH, self.record_bytes, "record")
        self.head = git.rev("HEAD")
        self.tree = git.rev("HEAD^{tree}")
        self.git = git

    def tearDown(self) -> None:
        self.temp.cleanup()

    def _status(self, problems: list[str], *, record: bool = True) -> Path:
        value = {
            "schema": "garnet.trust_kernel_review/v2",
            "ok": not problems,
            "discovery_ok": True,
            "discovery_source": "git",
            "base_commit": self.base,
            "head_commit": self.head,
            "trust_kernel_touched": True,
            "touched_paths": ["scripts/garnet_alpha.py"],
            "review_record_present": record,
            "review_record_path": RECORD_PATH if record else None,
            "review_record_sha256": hashlib.sha256(self.record_bytes).hexdigest() if record else None,
            "problems": problems,
        }
        path = self.repo / "status.json"
        path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
        return path

    def _argv(self, status: Path, output: Path, *, attempt: str = "1") -> list[str]:
        return [
            "emit", "--root", str(self.repo), "--status", str(status), "--output", str(output), "--github-repo", REPO, "--github-token-stdin",
            "--repository-id", "1218183013", "--pull-request-id", "4395439253", "--pull-request-number", "547",
            "--base-ref", "main", "--base-sha", self.base, "--head-sha", self.head, "--run-id", str(RUN_ID),
            "--run-number", "901", "--run-attempt", attempt, "--workflow-ref", WORKFLOW_REF, "--workflow-sha", WORKFLOW_SHA,
        ]

    def _transport(self, **run: object) -> FakeJsonTransport:
        run_object = {
            "id": RUN_ID, "run_number": 901, "run_attempt": 1, "event": "pull_request", "head_sha": self.head,
            "workflow_id": 12345678, "path": ".github/workflows/ci.yml", "repository": {"id": 1218183013, "full_name": REPO},
        }
        run_object.update(run)
        return FakeJsonTransport({f"actions/runs/{RUN_ID}": run_object}, {})

    def _emit(self, problems: list[str], *, record: bool = True, transport: FakeJsonTransport | None = None, attempt: str = "1") -> tuple[int, Path, str]:
        status = self._status(problems, record=record)
        output = self.repo / "r2" / "eligibility.json"
        factory = lambda repository, token: (self._transport() if transport is None else transport)  # noqa: E731
        with contextlib.redirect_stdout(io.StringIO()) as stdout, contextlib.redirect_stderr(io.StringIO()):
            code = elig.main(self._argv(status, output, attempt=attempt), stdin=io.BytesIO(TOKEN.encode()), transport_factory=factory, archive_transport_factory=lambda r, t: None)
        return code, output, stdout.getvalue()

    def test_approval_absent_only_emits_the_sole_eligible_receipt(self) -> None:
        code, output, text = self._emit([elig.APPROVAL_ABSENT_PROBLEM])
        self.assertEqual(code, 0, text)
        payload = output.read_bytes()
        value, problems = elig.load_receipt(payload)
        self.assertEqual(problems, [])
        self.assertEqual(value["state"], "approval_pending_only")
        self.assertEqual(value["finding_codes"], ["approval-absent"])
        self.assertEqual(value["candidate_head"], self.head)
        self.assertEqual(value["candidate_tree"], self.tree)
        self.assertEqual(value["base_sha"], self.base)
        self.assertEqual(value["review_record_path"], RECORD_PATH)
        self.assertEqual(value["review_record_sha256"], hashlib.sha256(self.record_bytes).hexdigest())
        self.assertEqual(value["producer_inventory_sha256"], hashlib.sha256(self.inventory).hexdigest())
        self.assertEqual(value["workflow_id"], 12345678)
        self.assertEqual(value["artifact_name"], elig.artifact_name(RUN_ID))
        self.assertEqual(value["run_attempt"], 1)
        self.assertEqual(payload, canonical(value))
        self.assertNotIn(TOKEN, text)

    def test_other_findings_and_clean_status_emit_ineligible_receipts(self) -> None:
        for label, problems in (
            ("not-at-head", [elig.APPROVAL_NOT_AT_HEAD_PROBLEM]),
            ("absent-plus-other", [elig.APPROVAL_ABSENT_PROBLEM, "structured review record is missing"]),
            ("unknown", ["some new finding"]),
            ("clean", []),
        ):
            with self.subTest(label=label):
                code, output, text = self._emit(problems)
                self.assertEqual(code, 0, text)
                value, load_problems = elig.load_receipt(output.read_bytes())
                self.assertEqual(load_problems, [])
                self.assertEqual(value["state"], "ineligible")
                self.assertNotEqual(value["finding_codes"], ["approval-absent"])
                output.unlink()

    def test_record_less_candidate_emits_nothing_and_exits_zero(self) -> None:
        code, output, text = self._emit([], record=False)
        self.assertEqual(code, 0)
        self.assertFalse(output.exists())
        self.assertIn("record-less", text)
        self.assertEqual(len(text.strip().splitlines()), 1)

    def test_missing_status_file_emits_nothing_and_exits_zero(self) -> None:
        output = self.repo / "r2" / "eligibility.json"
        argv = self._argv(self.repo / "absent.json", output)
        with contextlib.redirect_stdout(io.StringIO()) as stdout, contextlib.redirect_stderr(io.StringIO()):
            code = elig.main(argv, stdin=io.BytesIO(TOKEN.encode()), transport_factory=lambda r, t: self._transport(), archive_transport_factory=lambda r, t: None)
        self.assertEqual(code, 0)
        self.assertFalse(output.exists())
        self.assertEqual(len(stdout.getvalue().strip().splitlines()), 1)

    def test_emit_refuses_non_first_attempt_and_inconsistent_status(self) -> None:
        code, output, _ = self._emit([elig.APPROVAL_ABSENT_PROBLEM], attempt="2")
        self.assertNotEqual(code, 0)
        self.assertFalse(output.exists())
        status = self._status([elig.APPROVAL_ABSENT_PROBLEM])
        value = json.loads(status.read_text(encoding="utf-8"))
        value["head_commit"] = "e" * 40
        status.write_text(json.dumps(value), encoding="utf-8")
        out = self.repo / "r2" / "eligibility.json"
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            code = elig.main(self._argv(status, out), stdin=io.BytesIO(TOKEN.encode()), transport_factory=lambda r, t: self._transport(), archive_transport_factory=lambda r, t: None)
        self.assertNotEqual(code, 0)
        self.assertFalse(out.exists())

    def test_emit_fails_closed_when_the_run_object_disagrees(self) -> None:
        for label, transport in (
            ("wrong-run-id", self._transport(id=RUN_ID + 1)),
            ("wrong-attempt", self._transport(run_attempt=2)),
            ("wrong-event", self._transport(event="push")),
            ("wrong-head", self._transport(head_sha="e" * 40)),
            ("wrong-number", self._transport(run_number=1)),
            ("wrong-path", self._transport(path=".github/workflows/security.yml")),
            ("no-workflow-id", self._transport(workflow_id="x")),
            ("transport-failure", FakeJsonTransport({}, {})),
        ):
            with self.subTest(label=label):
                code, output, _ = self._emit([elig.APPROVAL_ABSENT_PROBLEM], transport=transport)
                self.assertNotEqual(code, 0, label)
                self.assertFalse(output.exists(), label)


class ArchiveTransportTests(unittest.TestCase):
    ARCHIVE = build_zip([(b"eligibility.json", canonical(receipt()))])
    BLOB = "https://productionresultssa0.blob.core.windows.net/actions-results/x.zip?sig=abc"

    def _opener(self, script: list[tuple[int, list[tuple[str, str]], bytes]]) -> tuple[object, list[object]]:
        requests: list[object] = []
        responses = list(script)

        def opener(request: object, *, timeout: float) -> object:
            requests.append(request)
            status, headers, body = responses.pop(0)
            return archive_transport.ArchiveResponse(status, tuple(headers), body)

        return opener, requests

    def _client(self, script: list[tuple[int, list[tuple[str, str]], bytes]]) -> tuple[object, list[object]]:
        opener, requests = self._opener(script)
        return archive_transport.ActionsArtifactTransport(REPO, TOKEN, opener=opener), requests

    def test_exactly_one_hop_strips_authorization_and_binds_the_archive(self) -> None:
        client, requests = self._client([
            (302, [("Location", self.BLOB), ("Content-Type", "text/html")], b""),
            (200, [("Content-Type", "application/zip"), ("Content-Length", str(len(self.ARCHIVE)))], self.ARCHIVE),
        ])
        with mock.patch.dict(os.environ, {"GITHUB_TOKEN": "ambient", "GH_TOKEN": "ambient"}):
            result = client.download_archive(777)
        self.assertEqual(result.problems, ())
        self.assertEqual(result.status, 200)
        self.assertEqual(result.final_url_host, "productionresultssa0.blob.core.windows.net")
        self.assertEqual(result.raw_bytes, self.ARCHIVE)
        self.assertEqual(result.sha256, hashlib.sha256(self.ARCHIVE).hexdigest())
        self.assertEqual(result.endpoint, "actions/artifacts/777/zip")
        self.assertEqual(len(requests), 2)
        first, second = requests
        self.assertEqual(first.full_url, f"https://api.github.com/repos/{REPO}/actions/artifacts/777/zip")
        self.assertEqual(first.get_header("Authorization"), f"Bearer {TOKEN}")
        self.assertEqual(second.full_url, self.BLOB)
        self.assertFalse(second.has_header("Authorization"))
        self.assertNotIn("ambient", str(first.header_items()) + str(second.header_items()))

    def test_both_signed_archive_hosts_are_accepted_on_the_default_port(self) -> None:
        for location in (
            "https://productionresultssa0.blob.core.windows.net/x.zip?sig=abc",
            "https://pipelinesghubeus2.actions.githubusercontent.com/x/y.zip",
            "https://productionresultssa0.blob.core.windows.net:443/x.zip",
        ):
            with self.subTest(location=location):
                client, _ = self._client([
                    (302, [("Location", location)], b""),
                    (200, [("Content-Type", "application/zip")], self.ARCHIVE),
                ])
                self.assertEqual(client.download_archive(777).problems, ())

    def test_octet_stream_is_accepted(self) -> None:
        client, _ = self._client([
            (302, [("Location", self.BLOB)], b""),
            (200, [("Content-Type", "application/octet-stream")], self.ARCHIVE),
        ])
        self.assertEqual(client.download_archive(777).problems, ())

    def test_hostile_hops_are_red(self) -> None:
        cases = {
            "no-redirect-200-json": [(200, [("Content-Type", "application/json")], b"{}")],
            "no-location": [(302, [], b"")],
            "redirect-to-api": [(302, [("Location", f"https://api.github.com/repos/{REPO}/actions/artifacts/777/zip")], b""), (200, [("Content-Type", "application/zip")], self.ARCHIVE)],
            "redirect-http": [(302, [("Location", "http://productionresultssa0.blob.core.windows.net/x.zip")], b""), (200, [("Content-Type", "application/zip")], self.ARCHIVE)],
            "redirect-userinfo": [(302, [("Location", "https://user:pw@blob.example/x.zip")], b""), (200, [("Content-Type", "application/zip")], self.ARCHIVE)],
            "second-redirect": [(302, [("Location", self.BLOB)], b""), (302, [("Location", self.BLOB)], b"")],
            "second-status-404": [(302, [("Location", self.BLOB)], b""), (404, [("Content-Type", "application/zip")], self.ARCHIVE)],
            "wrong-media": [(302, [("Location", self.BLOB)], b""), (200, [("Content-Type", "text/plain")], self.ARCHIVE)],
            "content-encoding": [(302, [("Location", self.BLOB)], b""), (200, [("Content-Type", "application/zip"), ("Content-Encoding", "gzip")], self.ARCHIVE)],
            "length-lie": [(302, [("Location", self.BLOB)], b""), (200, [("Content-Type", "application/zip"), ("Content-Length", "1")], self.ARCHIVE)],
            "too-large": [(302, [("Location", self.BLOB)], b""), (200, [("Content-Type", "application/zip")], b"x" * (archive_transport.MAX_ARCHIVE_BYTES + 1))],
            "first-status-301": [(301, [("Location", self.BLOB)], b""), (200, [("Content-Type", "application/zip")], self.ARCHIVE)],
            "first-status-401": [(401, [("Content-Type", "application/json")], b"{}")],
            "token-in-headers": [(302, [("Location", self.BLOB), ("X-Echo", TOKEN)], b""), (200, [("Content-Type", "application/zip")], self.ARCHIVE)],
            "token-in-location": [(302, [("Location", self.BLOB + "&t=" + TOKEN)], b""), (200, [("Content-Type", "application/zip")], self.ARCHIVE)],
            # Review v1 (F2): the hop accepted any DNS-shaped https host.
            "redirect-other-host": [(302, [("Location", "https://example.org/not-a-blob.zip")], b""), (200, [("Content-Type", "application/zip")], self.ARCHIVE)],
            "redirect-loopback": [(302, [("Location", "https://127.0.0.1:8443/not-a-blob.zip")], b""), (200, [("Content-Type", "application/zip")], self.ARCHIVE)],
            "redirect-ipv6": [(302, [("Location", "https://[::1]/x.zip")], b""), (200, [("Content-Type", "application/zip")], self.ARCHIVE)],
            "redirect-port": [(302, [("Location", "https://productionresultssa0.blob.core.windows.net:8443/x.zip")], b""), (200, [("Content-Type", "application/zip")], self.ARCHIVE)],
            "redirect-deceptive-suffix": [(302, [("Location", "https://productionresultssa0.blob.core.windows.net.evil.example/x.zip")], b""), (200, [("Content-Type", "application/zip")], self.ARCHIVE)],
            "redirect-suffix-as-prefix": [(302, [("Location", "https://evil.example/blob.core.windows.net/x.zip")], b""), (200, [("Content-Type", "application/zip")], self.ARCHIVE)],
            "redirect-bare-suffix-domain": [(302, [("Location", "https://blob.core.windows.net/x.zip")], b""), (200, [("Content-Type", "application/zip")], self.ARCHIVE)],
        }
        for label, script in cases.items():
            with self.subTest(label=label):
                client, _ = self._client(script)
                result = client.download_archive(777)
                self.assertTrue(result.problems, label)
                self.assertEqual(result.raw_bytes, b"", label)
                self.assertIsNone(result.sha256, label)
                for code in result.problems:
                    self.assertIn(code, archive_transport.ALLOWED_PROBLEM_CODES)

    def test_invalid_configuration_and_artifact_ids_are_red_without_transport(self) -> None:
        opener, requests = self._opener([])
        for repository, token in (("garnet", TOKEN), (REPO, ""), (REPO, "bad token"), (REPO, "x" * 1025)):
            with self.subTest(repository=repository, token=token):
                client = archive_transport.ActionsArtifactTransport(repository, token, opener=opener)
                self.assertEqual(client.download_archive(777).problems, ("invalid-configuration",))
        client = archive_transport.ActionsArtifactTransport(REPO, TOKEN, opener=opener)
        for artifact_id in (0, -1, True, "777", 10**21):
            with self.subTest(artifact_id=artifact_id):
                self.assertEqual(client.download_archive(artifact_id).problems, ("invalid-artifact-id",))
        self.assertEqual(requests, [])
        self.assertNotIn(TOKEN, repr(client))

    def test_opener_exception_is_transport_failure(self) -> None:
        def opener(request: object, *, timeout: float) -> object:
            raise OSError("network down")

        client = archive_transport.ActionsArtifactTransport(REPO, TOKEN, opener=opener)
        self.assertEqual(client.download_archive(777).problems, ("transport-failure",))

    def test_module_never_reads_ambient_credentials(self) -> None:
        for name in ("garnet_actions_artifact_transport", "garnet_trust_kernel_review_eligibility"):
            source = (ROOT / "scripts" / f"{name}.py").read_text(encoding="utf-8")
            self.assertNotIn("getenv", source, name)
            self.assertNotIn("GITHUB_TOKEN", source, name)
            self.assertNotIn("GH_TOKEN", source, name)
            self.assertNotIn("environb", source, name)
        transport_source = (ROOT / "scripts/garnet_actions_artifact_transport.py").read_text(encoding="utf-8")
        self.assertNotIn("import os", transport_source)
        self.assertNotIn("os.environ", transport_source)
        # The eligibility module may touch os.environ only inside the scrubbed
        # Git allowlist helper, and that helper may not pass a token through.
        elig_source = (ROOT / "scripts/garnet_trust_kernel_review_eligibility.py").read_text(encoding="utf-8")
        helper = elig_source.split("def _scrubbed_git_environment", 1)[1].split("\ndef ", 1)[0]
        self.assertEqual(elig_source.count("os.environ"), helper.count("os.environ"))
        self.assertGreaterEqual(helper.count("os.environ"), 1)
        self.assertNotIn("TOKEN", helper)
        self.assertNotIn("SECRET", helper)


class RepositoryWiringTests(unittest.TestCase):
    def test_ci_wires_attempt1_receipt_and_attempt2_verification(self) -> None:
        workflow = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
        self.assertIn("permissions:\n  actions: read\n  contents: read\n  pull-requests: read\n", workflow)
        self.assertNotIn("actions: write", workflow)
        self.assertIn("name: emit r2 eligibility receipt", workflow)
        self.assertIn("name: upload r2 eligibility receipt", workflow)
        self.assertEqual(workflow.count("if: always() && github.event_name == 'pull_request' && github.run_attempt == '1'"), 2)
        self.assertIn("uses: actions/upload-artifact@b7c566a772e6b6bfb58ed0dc250532a479d7789f", workflow)
        self.assertIn("name: r2-approval-pending-${{ github.run_id }}-attempt-1", workflow)
        self.assertIn("path: ${{ runner.temp }}/r2/eligibility.json", workflow)
        self.assertIn("if-no-files-found: ignore", workflow)
        self.assertIn("retention-days: 90", workflow)
        self.assertNotIn("overwrite:", workflow)
        self.assertIn("garnet_trust_kernel_review_eligibility.py emit", workflow)
        self.assertIn("garnet_trust_kernel_review_eligibility.py verify", workflow)
        self.assertIn('--run-id "$REVIEW_RUN_ID" --run-attempt "$REVIEW_RUN_ATTEMPT" --status-out "$RUNNER_TEMP/trust-kernel-review.json"', workflow)
        self.assertIn("REVIEW_RUN_ID: ${{ github.run_id }}", workflow)
        self.assertIn("REVIEW_RUN_ATTEMPT: ${{ github.run_attempt }}", workflow)
        self.assertIn("REVIEW_REPOSITORY_ID: ${{ github.repository_id }}", workflow)
        self.assertIn("REVIEW_PR_ID: ${{ github.event.pull_request.id }}", workflow)
        self.assertIn("REVIEW_BASE_REF: ${{ github.event.pull_request.base.ref }}", workflow)
        self.assertIn("REVIEW_BASE_SHA: ${{ github.event.pull_request.base.sha }}", workflow)
        self.assertIn("REVIEW_HEAD_SHA: ${{ github.event.pull_request.head.sha }}", workflow)
        self.assertIn("REVIEW_RUN_NUMBER: ${{ github.run_number }}", workflow)
        self.assertIn("REVIEW_WORKFLOW_REF: ${{ github.workflow_ref }}", workflow)
        self.assertIn("REVIEW_WORKFLOW_SHA: ${{ github.workflow_sha }}", workflow)
        self.assertIn('if [[ "$REVIEW_RUN_ATTEMPT" == "2" ]]; then', workflow)
        self.assertIn('--eligibility-verdict "$RUNNER_TEMP/r2/verdict.json"', workflow)
        self.assertIn("--github-token-stdin", workflow.split("name: emit r2 eligibility receipt", 1)[1])
        emit_step = workflow.split("name: emit r2 eligibility receipt", 1)[1].split("name: upload r2 eligibility receipt", 1)[0]
        self.assertIn("unset REVIEW_TOKEN GH_TOKEN GITHUB_TOKEN GARNET_REVIEW_GITHUB_TOKEN GARNET_ADMIN_GITHUB_TOKEN", emit_step)
        self.assertIn('--output "$RUNNER_TEMP/r2/eligibility.json"', emit_step)
        self.assertIn('--status "$RUNNER_TEMP/trust-kernel-review.json"', emit_step)

    def test_base_controlled_workflow_is_untouched_by_this_act(self) -> None:
        digest = hashlib.sha256((ROOT / ".github/workflows/base-controlled-trust.yml").read_bytes()).hexdigest()
        self.assertEqual(digest, "33bb45ac7400ed4572f5777c1ccd277ad73316244dca413e9c890fe2da8bf4c7")

    def test_agents_md_names_the_new_commands(self) -> None:
        agents = (ROOT / "AGENTS.md").read_text(encoding="utf-8")
        self.assertIn("python3 -I scripts/test_garnet_trust_kernel_review_eligibility.py", agents)
        self.assertIn("scripts/garnet_trust_kernel_review_eligibility.py", agents)
        self.assertIn("scripts/garnet_actions_artifact_transport.py", agents)


if __name__ == "__main__":
    unittest.main()
