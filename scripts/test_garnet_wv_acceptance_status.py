#!/usr/bin/env python3
"""Regression tests for WV-6/WV-7 fail-closed acceptance reporting."""
from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "garnet_wv_acceptance_status.py"
SPEC = importlib.util.spec_from_file_location("garnet_wv_acceptance_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
wv = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_wv_acceptance_status"] = wv
SPEC.loader.exec_module(wv)


def _complete_evidence(
    root: Path, identifier: str = "WV-6"
) -> tuple[Path, dict[str, object]]:
    """Lay out complete, hash-verified evidence and return the manifest that
    would accept it. The manifest itself is not written: each caller decides
    which bytes reach the reporter."""
    wv.copy_contract(ROOT, root)
    contract = wv.load_contracts(root)[identifier]
    evidence = root / contract["evidenceDestination"]
    evidence.mkdir(parents=True)
    artifacts: list[dict[str, str]] = []
    checks: list[dict[str, object]] = []
    for item in contract["requiredChecks"]:
        relative = f"{item['id']}.txt"
        payload = f"{item['id']} passed\n".encode()
        (evidence / relative).write_bytes(payload)
        artifacts.append(
            {"path": relative, "sha256": hashlib.sha256(payload).hexdigest()}
        )
        checks.append(
            {
                "id": item["id"],
                "status": "passed",
                "command": f"verify {item['id']}",
                "evidence": [relative],
            }
        )
    manifest: dict[str, object] = {
        "schema": "garnet.wv_acceptance_evidence/v2",
        "wv": identifier,
        "contractBaseMainSha": wv.EXPECTED_BASE_SHA,
        "reviewedHeadSha": wv.REVIEWED_HEAD,
        "reviewedTreeSha": wv.REVIEWED_TREE,
        "productContentSha256": wv.EXPECTED_PRODUCT_CONTENT_SHA256,
        "state": "evidence_complete",
        "platform": "windows",
        "checks": checks,
        "artifacts": artifacts,
        "scopeLimitsAcknowledged": True,
        "jonOnlyActionsPerformed": [],
    }
    return evidence, manifest


def _manifest_bytes(manifest: dict[str, object]) -> bytes:
    return (json.dumps(manifest, indent=2) + "\n").encode("utf-8")


def _swap_after_lstat(target_name: str, replacement: Path, destination: Path):
    """An ``os.lstat`` stand-in reproducing the crown's deterministic
    check/use swap: right after the first metadata check of a path named
    ``target_name``, ``destination`` is atomically replaced by
    ``replacement``. A reader that re-opens by path afterwards reads the
    replacement; a reader bound to what it checked cannot."""
    real_lstat = os.lstat
    fired: list[str] = []

    def lstat_then_swap(path, *args, **kwargs):
        result = real_lstat(path, *args, **kwargs)
        if not fired and os.path.basename(os.fsdecode(path)) == target_name:
            os.replace(replacement, destination)
            fired.append(os.fsdecode(path))
        return result

    return lstat_then_swap, fired


def _swap_after_fstat(identity: tuple[int, int], replacement: Path, destination: Path):
    """An ``os.fstat`` stand-in that replaces ``destination`` on the path the
    moment a descriptor bound to ``identity`` (st_dev, st_ino) is inspected.
    Only a reader that reads from that same descriptor still sees the bytes
    it checked."""
    real_fstat = os.fstat
    fired: list[int] = []

    def fstat_then_swap(fd):
        result = real_fstat(fd)
        if not fired and (result.st_dev, result.st_ino) == identity:
            os.replace(replacement, destination)
            fired.append(fd)
        return result

    return fstat_then_swap, fired


class GarnetWvAcceptanceStatusTests(unittest.TestCase):
    def test_contracts_keep_established_meanings(self) -> None:
        contracts = wv.load_contracts(ROOT)
        self.assertEqual(
            contracts["WV-6"]["evidenceDestination"],
            "proofs/windows/launch-verification/wv6-minimum-shelf/",
        )
        self.assertIn("Core Ring Tier 1", contracts["WV-6"]["title"])
        self.assertEqual(
            contracts["WV-7"]["evidenceDestination"],
            "proofs/windows/launch-verification/wv7-distribution/",
        )
        self.assertIn("winget", contracts["WV-7"]["title"].lower())
        self.assertIn("Docker", contracts["WV-7"]["title"])

    def test_current_repository_tracks_wv6_acceptance_and_wv7_pending(self) -> None:
        expectations = {
            # Current truth since the #528 squash (0607f7fe): the accepted
            # head 8426ca76 is no longer an ancestor of main, so the reporter
            # correctly reports PARTIAL (U-58 squash-successor gap; see the
            # L1 re-acceptance redesign brief). The frozen side of each
            # finding is pinned exactly via the reporter constants; the raw
            # side is pattern-matched because every digest-domain candidate
            # (this cure included) moves it. Restoring "accepted" requires a
            # new native boundary or an adopted succession law, never an
            # expectation edit alone.
            "WV-6": {
                "schema": "garnet.wv_acceptance_status/v2",
                "state": "partial",
                "ok": False,
                "returncode": 1,
                "passed": 5,
                "required": 5,
                "artifacts": 5,
                "findings_patterns": [
                    r"product content digest mismatch \([0-9a-f]{64} != "
                    + re.escape(wv.EXPECTED_PRODUCT_CONTENT_SHA256)
                    + r"\)",
                    r"product path count mismatch \(\d+ != "
                    + re.escape(
                        str(wv.bound_shelf_reporter.EXPECTED_PRODUCT_PATH_COUNT)
                    )
                    + r"\)",
                ],
                "reviewed_head": wv.REVIEWED_HEAD,
                "reviewed_tree": wv.REVIEWED_TREE,
                "product_digest": wv.EXPECTED_PRODUCT_CONTENT_SHA256,
            },
            "WV-7": {
                "schema": "garnet.wv_acceptance_status/v2",
                "state": "pending",
                "ok": False,
                "returncode": 1,
                "passed": 0,
                "required": 5,
                "artifacts": 0,
                "findings": ["exact-candidate evidence manifest is pending"],
                "reviewed_head": None,
                "reviewed_tree": None,
                "product_digest": None,
            },
        }
        for identifier, expected in expectations.items():
            with self.subTest(identifier=identifier):
                status = wv.read_status(ROOT, identifier)
                self.assertEqual(status.schema, expected["schema"])
                self.assertEqual(status.state, expected["state"])
                self.assertEqual(status.ok, expected["ok"])
                self.assertEqual(status.passed_check_count, expected["passed"])
                self.assertEqual(status.required_check_count, expected["required"])
                self.assertEqual(status.artifact_count, expected["artifacts"])
                if "findings_patterns" in expected:
                    patterns = expected["findings_patterns"]
                    self.assertEqual(len(status.findings), len(patterns))
                    for finding, pattern in zip(status.findings, patterns):
                        self.assertRegex(finding, rf"\A{pattern}\Z")
                else:
                    self.assertEqual(status.findings, expected["findings"])
                self.assertEqual(status.reviewed_head_sha, expected["reviewed_head"])
                self.assertEqual(status.reviewed_tree_sha, expected["reviewed_tree"])
                self.assertEqual(
                    status.product_content_sha256, expected["product_digest"]
                )
                proc = subprocess.run(
                    [sys.executable, "-I", str(SCRIPT), "--wv", identifier, "--gate"],
                    cwd=ROOT,
                    text=True,
                    capture_output=True,
                    check=False,
                )
                self.assertEqual(proc.returncode, expected["returncode"])
                payload = json.loads(proc.stdout)
                self.assertEqual(payload["state"], expected["state"])
                self.assertEqual(payload["ok"], expected["ok"])
                self.assertEqual(payload["passed_check_count"], expected["passed"])
                self.assertEqual(payload["required_check_count"], expected["required"])
                self.assertEqual(payload["artifact_count"], expected["artifacts"])
                if "findings_patterns" in expected:
                    patterns = expected["findings_patterns"]
                    self.assertEqual(len(payload["findings"]), len(patterns))
                    for finding, pattern in zip(payload["findings"], patterns):
                        self.assertRegex(finding, rf"\A{pattern}\Z")
                else:
                    self.assertEqual(payload["findings"], expected["findings"])
                self.assertEqual(payload["reviewed_head_sha"], expected["reviewed_head"])
                self.assertEqual(payload["reviewed_tree_sha"], expected["reviewed_tree"])
                self.assertEqual(
                    payload["product_content_sha256"], expected["product_digest"]
                )

    def test_malformed_evidence_is_partial_not_accepted(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            wv.copy_contract(ROOT, root)
            contract = wv.load_contracts(root)["WV-6"]
            evidence = root / contract["evidenceDestination"]
            evidence.mkdir(parents=True)
            (evidence / wv.EVIDENCE_MANIFEST).write_text("{", encoding="utf-8")
            status = wv.read_status(root, "WV-6", verify_git=False)
        self.assertEqual(status.state, "partial")
        self.assertFalse(status.ok)

    def test_complete_hashed_evidence_can_satisfy_contract(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            wv.copy_contract(ROOT, root)
            contract = wv.load_contracts(root)["WV-6"]
            evidence = root / contract["evidenceDestination"]
            evidence.mkdir(parents=True)
            artifacts = []
            checks = []
            for item in contract["requiredChecks"]:
                relative = f"{item['id']}.txt"
                payload = f"{item['id']} passed\n".encode()
                (evidence / relative).write_bytes(payload)
                artifacts.append(
                    {"path": relative, "sha256": hashlib.sha256(payload).hexdigest()}
                )
                checks.append(
                    {
                        "id": item["id"],
                        "status": "passed",
                        "command": f"verify {item['id']}",
                        "evidence": [relative],
                    }
                )
            manifest = {
                "schema": "garnet.wv_acceptance_evidence/v2",
                "wv": "WV-6",
                "contractBaseMainSha": wv.EXPECTED_BASE_SHA,
                "reviewedHeadSha": wv.REVIEWED_HEAD,
                "reviewedTreeSha": wv.REVIEWED_TREE,
                "productContentSha256": wv.EXPECTED_PRODUCT_CONTENT_SHA256,
                "state": "evidence_complete",
                "platform": "windows",
                "checks": checks,
                "artifacts": artifacts,
                "scopeLimitsAcknowledged": True,
                "jonOnlyActionsPerformed": [],
            }
            (evidence / wv.EVIDENCE_MANIFEST).write_text(
                json.dumps(manifest, indent=2), encoding="utf-8"
            )
            status = wv.read_status(root, "WV-6", verify_git=False)
        self.assertEqual(status.state, "accepted")
        self.assertTrue(status.ok, status.findings)

    def test_missing_check_and_hash_mismatch_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            wv.copy_contract(ROOT, root)
            contract = wv.load_contracts(root)["WV-7"]
            evidence = root / contract["evidenceDestination"]
            evidence.mkdir(parents=True)
            artifact = evidence / "only.txt"
            artifact.write_text("not enough", encoding="utf-8")
            manifest = {
                "schema": "garnet.wv_acceptance_evidence/v2",
                "wv": "WV-7",
                "contractBaseMainSha": wv.EXPECTED_BASE_SHA,
                "reviewedHeadSha": wv.REVIEWED_HEAD,
                "reviewedTreeSha": wv.REVIEWED_TREE,
                "productContentSha256": wv.EXPECTED_PRODUCT_CONTENT_SHA256,
                "state": "evidence_complete",
                "platform": "windows",
                "checks": [],
                "artifacts": [{"path": "only.txt", "sha256": "0" * 64}],
                "scopeLimitsAcknowledged": True,
                "jonOnlyActionsPerformed": [],
            }
            (evidence / wv.EVIDENCE_MANIFEST).write_text(
                json.dumps(manifest), encoding="utf-8"
            )
            status = wv.read_status(root, "WV-7", verify_git=False)
        self.assertEqual(status.state, "partial")
        self.assertFalse(status.ok)
        self.assertTrue(any("required check" in item for item in status.findings))
        self.assertTrue(any("SHA-256" in item for item in status.findings))

    def test_invalid_review_provenance_cannot_self_promote(self) -> None:
        findings, _ = wv._verify_squash_durable_content(
            ROOT,
            reviewed_head="not-a-sha",
            reviewed_tree="also-not-a-tree",
            expected_content_digest="not-a-digest",
            verify_git=False,
        )
        self.assertTrue(any("reviewed head provenance" in item for item in findings))
        self.assertTrue(any("reviewed tree provenance" in item for item in findings))
        self.assertTrue(any("product content digest" in item for item in findings))

    # Crown ceremony, scope D, bound to beeb5e7b: the reporter accepted a CRLF
    # manifest (wv_crlf_manifest: state=accepted) and re-opened evidence by
    # path after checking it (wv_check_use_swap: accepted_source=outside).
    # Each test names the exact problem string the cured reporter must emit;
    # none may reach "accepted".

    def test_crlf_manifest_is_rejected_by_name(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            evidence, manifest = _complete_evidence(root)
            manifest_path = evidence / wv.EVIDENCE_MANIFEST
            crlf = _manifest_bytes(manifest).replace(b"\n", b"\r\n")
            offset = crlf.index(b"\r")
            manifest_path.write_bytes(crlf)
            status = wv.read_status(root, "WV-6", verify_git=False)
        self.assertEqual(status.state, "partial")
        self.assertFalse(status.ok)
        self.assertEqual(
            status.findings,
            [
                f"{manifest_path} must use LF-only line endings "
                f"(first CR byte at offset {offset})"
            ],
        )

    def test_bom_manifest_is_rejected_by_name(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            evidence, manifest = _complete_evidence(root)
            manifest_path = evidence / wv.EVIDENCE_MANIFEST
            manifest_path.write_bytes(b"\xef\xbb\xbf" + _manifest_bytes(manifest))
            status = wv.read_status(root, "WV-6", verify_git=False)
        self.assertEqual(status.state, "partial")
        self.assertFalse(status.ok)
        self.assertEqual(
            status.findings,
            [f"{manifest_path} must not begin with a UTF-8 byte-order mark"],
        )

    def test_non_utf8_manifest_is_rejected_by_name(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            evidence, manifest = _complete_evidence(root)
            manifest_path = evidence / wv.EVIDENCE_MANIFEST
            raw = _manifest_bytes(manifest).replace(b'"windows"', b'"windo\xffws"')
            self.assertIn(b"\xff", raw)
            manifest_path.write_bytes(raw)
            status = wv.read_status(root, "WV-6", verify_git=False)
        self.assertEqual(status.state, "partial")
        self.assertFalse(status.ok)
        self.assertEqual(len(status.findings), 1)
        self.assertTrue(
            status.findings[0].startswith(f"{manifest_path} is not strict UTF-8: "),
            status.findings,
        )

    def test_manifest_swapped_after_its_check_is_never_accepted(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            evidence, manifest = _complete_evidence(root)
            manifest_path = evidence / wv.EVIDENCE_MANIFEST
            # Inside: the bytes the reporter checks; they fail the contract.
            manifest_path.write_bytes(
                _manifest_bytes(dict(manifest, state="evidence_incomplete"))
            )
            # Outside: the bytes a re-open by path would find; they would accept.
            outside = root / "outside" / wv.EVIDENCE_MANIFEST
            outside.parent.mkdir()
            outside.write_bytes(_manifest_bytes(manifest))
            hook, fired = _swap_after_lstat(wv.EVIDENCE_MANIFEST, outside, manifest_path)
            with mock.patch.object(os, "lstat", hook):
                status = wv.read_status(root, "WV-6", verify_git=False)
        # The reporter now reads relative to a descriptor bound to the evidence
        # root, so the name reaching os.lstat is the relative one, not the
        # absolute path. The swap is still detected; only the identifier the
        # hook records changed shape (review v1 ancestor-swap cure).
        self.assertEqual(fired, [wv.EVIDENCE_MANIFEST])
        self.assertNotEqual(status.state, "accepted")
        self.assertFalse(status.ok)
        self.assertIn(
            f"{manifest_path} identity changed between check and open",
            status.findings,
        )

    @unittest.skipIf(os.name == "nt", "Windows cannot replace a file that is open")
    def test_manifest_bytes_come_from_the_checked_descriptor(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            evidence, manifest = _complete_evidence(root)
            manifest_path = evidence / wv.EVIDENCE_MANIFEST
            manifest_path.write_bytes(
                _manifest_bytes(dict(manifest, state="evidence_incomplete"))
            )
            outside = root / "outside" / wv.EVIDENCE_MANIFEST
            outside.parent.mkdir()
            outside.write_bytes(_manifest_bytes(manifest))
            before = os.lstat(manifest_path)
            hook, fired = _swap_after_fstat(
                (before.st_dev, before.st_ino), outside, manifest_path
            )
            with mock.patch.object(os, "fstat", hook):
                status = wv.read_status(root, "WV-6", verify_git=False)
            # The path now carries the accepting bytes...
            self.assertEqual(manifest_path.read_bytes(), _manifest_bytes(manifest))
        # ...but the reporter judged the descriptor it checked.
        self.assertEqual(len(fired), 1)
        self.assertNotEqual(status.state, "accepted")
        self.assertFalse(status.ok)
        self.assertIn(
            "evidence manifest state must be evidence_complete", status.findings
        )
        self.assertFalse(
            any("changed" in item for item in status.findings), status.findings
        )

    def test_artifact_swapped_after_its_check_is_never_accepted(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            evidence, manifest = _complete_evidence(root)
            (evidence / wv.EVIDENCE_MANIFEST).write_bytes(_manifest_bytes(manifest))
            target = str(manifest["artifacts"][0]["path"])
            inside = evidence / target
            listed = inside.read_bytes()
            inside.write_bytes(b"tampered\n")
            outside = root / "outside" / target
            outside.parent.mkdir()
            outside.write_bytes(listed)
            hook, fired = _swap_after_lstat(target, outside, inside)
            with mock.patch.object(os, "lstat", hook):
                status = wv.read_status(root, "WV-6", verify_git=False)
        # The reporter now reads relative to a descriptor bound to the evidence
        # root, so the name reaching os.lstat is the relative one, not the
        # absolute path. The swap is still detected; only the identifier the
        # hook records changed shape (review v1 ancestor-swap cure).
        self.assertEqual(fired, [target])
        self.assertNotEqual(status.state, "accepted")
        self.assertFalse(status.ok)
        self.assertIn(
            f"artifact {target} identity changed between check and open",
            status.findings,
        )

    @unittest.skipIf(os.name == "nt", "Windows cannot replace a file that is open")
    def test_artifact_hash_covers_the_checked_descriptor_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            evidence, manifest = _complete_evidence(root)
            (evidence / wv.EVIDENCE_MANIFEST).write_bytes(_manifest_bytes(manifest))
            target = str(manifest["artifacts"][0]["path"])
            inside = evidence / target
            listed = inside.read_bytes()
            inside.write_bytes(b"tampered\n")
            outside = root / "outside" / target
            outside.parent.mkdir()
            outside.write_bytes(listed)
            before = os.lstat(inside)
            hook, fired = _swap_after_fstat((before.st_dev, before.st_ino), outside, inside)
            with mock.patch.object(os, "fstat", hook):
                status = wv.read_status(root, "WV-6", verify_git=False)
            self.assertEqual(inside.read_bytes(), listed)
        self.assertEqual(len(fired), 1)
        self.assertNotEqual(status.state, "accepted")
        self.assertFalse(status.ok)
        self.assertIn(f"artifact {target} SHA-256 does not match", status.findings)
        self.assertFalse(
            any("changed" in item for item in status.findings), status.findings
        )



@unittest.skipUnless(wv.DIR_FD_SUPPORTED, "platform has no dir_fd support")
class EvidenceRootBindingTests(unittest.TestCase):
    """The evidence ROOT is bound by descriptor, so an ancestor cannot be swapped.

    O_NOFOLLOW protects only the final component. Checking the evidence
    directory by pathname and then resolving that pathname again for the
    manifest, every artifact and the inventory left an ancestor swap open: a
    deterministic swap after the directory check redirected the whole traversal
    to an outside tree and the reporter returned `accepted` (review v1 finding).
    """

    def test_absent_destination_is_reported_as_absent_not_unbindable(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            fd, reason = wv.open_evidence_root(Path(td) / "nope")
            self.assertIsNone(fd)
            self.assertEqual(reason, "absent")

    def test_symlinked_destination_is_refused(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            (root / "real").mkdir()
            link = root / "link"
            link.symlink_to(root / "real", target_is_directory=True)
            fd, reason = wv.open_evidence_root(link)
            self.assertIsNone(fd, "a symlinked evidence destination must not bind")
            self.assertEqual(reason, "symlink")

    def test_swapping_the_directory_after_binding_does_not_redirect_the_walk(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            inside = root / "evidence"
            inside.mkdir()
            (inside / "real.txt").write_bytes(b"the bytes that were checked\n")
            outside = root / "outside"
            outside.mkdir()
            (outside / "planted.txt").write_bytes(b"the bytes an ancestor swap would supply\n")

            fd, reason = wv.open_evidence_root(inside)
            self.assertEqual(reason, "ok")
            self.assertIsNotNone(fd)
            try:
                # The swap an attacker gets to perform: the checked pathname now
                # resolves somewhere else entirely.
                inside.rename(root / "moved")
                Path(inside).symlink_to(outside, target_is_directory=True)
                self.assertTrue(inside.is_symlink())

                listed = wv._inventory_from_descriptor(fd)
            finally:
                os.close(fd)

            self.assertEqual(
                listed,
                {"real.txt"},
                "the walk must follow the bound descriptor, not the swapped pathname",
            )
            self.assertNotIn("planted.txt", listed)

    def test_bound_reads_ignore_a_swapped_pathname(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            inside = root / "evidence"
            inside.mkdir()
            (inside / "f.txt").write_bytes(b"checked\n")
            outside = root / "outside"
            outside.mkdir()
            (outside / "f.txt").write_bytes(b"planted\n")

            fd, _reason = wv.open_evidence_root(inside)
            try:
                inside.rename(root / "moved")
                Path(inside).symlink_to(outside, target_is_directory=True)
                raw = wv._regular_bytes(
                    Path("f.txt"), limit=4096, minimum=0,
                    label="f.txt", bound="bounds", dir_fd=fd,
                )
            finally:
                os.close(fd)
            self.assertEqual(raw, b"checked\n")


if __name__ == "__main__":
    unittest.main()