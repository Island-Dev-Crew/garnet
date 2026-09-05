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
import threading
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


class ContractTests(unittest.TestCase):
    """No evidence I/O: these run on every platform."""

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


@unittest.skipUnless(wv.DIR_FD_SUPPORTED, "evidence acceptance requires dir_fd (POSIX)")
class GarnetWvAcceptanceStatusTests(unittest.TestCase):
    """Every test here reads evidence, which the reporter does only through a
    bound directory descriptor; a platform without ``dir_fd`` never accepts
    (see UnsupportedPlatformTests) and skips these."""

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
        # ...and the reporter, holding the descriptor it checked, sees the
        # rename-over itself: unlinking the held inode moves its ctime, which
        # is part of the post-read identity since review finding 5. The swap
        # is a finding in its own right, not merely survived.
        self.assertEqual(len(fired), 1)
        self.assertNotEqual(status.state, "accepted")
        self.assertFalse(status.ok)
        self.assertEqual(
            status.findings, [f"{manifest_path} changed while being read"]
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
        # The rename-over is detected outright (ctime moves when the held
        # inode is unlinked), so the tampered bytes are never even hashed.
        self.assertEqual(len(fired), 1)
        self.assertNotEqual(status.state, "accepted")
        self.assertFalse(status.ok)
        self.assertIn(f"artifact {target} changed while being read", status.findings)
        self.assertFalse(
            any("SHA-256" in item for item in status.findings), status.findings
        )


    # Review v1 of this change (Codex): seven findings, each reproduced below
    # from the reviewer's own probe. None may reach "accepted".

    def test_nested_artifact_parent_swap_is_never_accepted(self) -> None:
        """Finding 1: O_NOFOLLOW protects only the LAST component. With
        `nested/` swapped for a symlink to an outside directory during the
        leaf's check and open, then restored before the inventory, outside
        bytes with the listed hash were accepted."""
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            evidence, manifest = _complete_evidence(root)
            good = b"the bytes the manifest lists\n"
            nested = evidence / "nested"
            nested.mkdir()
            (nested / "proof.bin").write_bytes(b"wrong inside bytes\n")
            manifest["artifacts"].append(
                {"path": "nested/proof.bin", "sha256": hashlib.sha256(good).hexdigest()}
            )
            (evidence / wv.EVIDENCE_MANIFEST).write_bytes(_manifest_bytes(manifest))
            outside = root / "outside"
            outside.mkdir()
            (outside / "proof.bin").write_bytes(good)
            moved = root / "moved"
            nested.rename(moved)
            nested.symlink_to(outside, target_is_directory=True)
            outside_identity = os.lstat(outside / "proof.bin")
            real_fstat = os.fstat
            restored: list[int] = []

            def fstat_then_restore_parent(fd):
                result = real_fstat(fd)
                if not restored and (result.st_dev, result.st_ino) == (
                    outside_identity.st_dev,
                    outside_identity.st_ino,
                ):
                    nested.unlink()
                    moved.rename(nested)
                    restored.append(fd)
                return result

            with mock.patch.object(os, "fstat", fstat_then_restore_parent):
                status = wv.read_status(root, "WV-6", verify_git=False)
            if nested.is_symlink():
                nested.unlink()
                moved.rename(nested)
        self.assertNotEqual(status.state, "accepted")
        self.assertFalse(status.ok)
        self.assertTrue(
            any(
                item.startswith(
                    "artifact nested/proof.bin parent 'nested' is not a bound directory"
                )
                for item in status.findings
            ),
            status.findings,
        )

    def test_unreadable_evidence_subtree_is_a_finding_not_empty(self) -> None:
        """Finding 3: an inventory error was swallowed, so an unreadable
        subtree counted as empty and an unlisted file stopped mattering."""
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            evidence, manifest = _complete_evidence(root)
            (evidence / wv.EVIDENCE_MANIFEST).write_bytes(_manifest_bytes(manifest))
            (evidence / "unlisted").mkdir()
            (evidence / "unlisted" / "extra.txt").write_bytes(b"not in the manifest\n")
            control = wv.read_status(root, "WV-6", verify_git=False)
            self.assertIn(
                "evidence directory files do not exactly match the manifest",
                control.findings,
            )
            real_open = os.open

            def open_denying_unlisted(path, flags, *args, **kwargs):
                if os.fsdecode(path) == "unlisted" and kwargs.get("dir_fd") is not None:
                    raise PermissionError(13, "Permission denied")
                return real_open(path, flags, *args, **kwargs)

            with mock.patch.object(os, "open", open_denying_unlisted):
                status = wv.read_status(root, "WV-6", verify_git=False)
        self.assertNotEqual(status.state, "accepted")
        self.assertFalse(status.ok)
        self.assertIn(
            "evidence inventory could not read unlisted: Permission denied",
            status.findings,
        )

    def test_descriptor_inspection_failure_is_a_named_finding(self) -> None:
        """Finding 3, second half: an fstat failure escaped as a raw OSError."""
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            evidence, manifest = _complete_evidence(root)
            manifest_path = evidence / wv.EVIDENCE_MANIFEST
            manifest_path.write_bytes(_manifest_bytes(manifest))
            before = os.lstat(manifest_path)
            real_fstat = os.fstat

            def failing_fstat(fd):
                result = real_fstat(fd)
                if (result.st_dev, result.st_ino) == (before.st_dev, before.st_ino):
                    raise OSError(5, "Input/output error")
                return result

            with mock.patch.object(os, "fstat", failing_fstat):
                status = wv.read_status(root, "WV-6", verify_git=False)
        self.assertNotEqual(status.state, "accepted")
        self.assertIn(
            f"{manifest_path} could not be inspected: Input/output error",
            status.findings,
        )

    @unittest.skipUnless(hasattr(os, "mkfifo"), "no FIFOs on this platform")
    def test_fifo_swapped_after_the_check_is_rejected_without_blocking(self) -> None:
        """Finding 4: a blocking open of a substituted FIFO waited for a
        writer forever and never reached the descriptor type check."""
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            evidence, manifest = _complete_evidence(root)
            manifest_path = evidence / wv.EVIDENCE_MANIFEST
            manifest_path.write_bytes(_manifest_bytes(manifest))
            fifo = root / "fifo"
            os.mkfifo(fifo)
            hook, fired = _swap_after_lstat(wv.EVIDENCE_MANIFEST, fifo, manifest_path)
            result: list[object] = []

            def run() -> None:
                with mock.patch.object(os, "lstat", hook):
                    result.append(wv.read_status(root, "WV-6", verify_git=False))

            worker = threading.Thread(target=run, daemon=True)
            worker.start()
            worker.join(10)
            self.assertFalse(
                worker.is_alive(), "the open of a substituted FIFO must not block"
            )
        status = result[0]
        self.assertEqual(fired, [wv.EVIDENCE_MANIFEST])
        self.assertNotEqual(status.state, "accepted")
        self.assertIn(
            f"{manifest_path} identity changed between check and open",
            status.findings,
        )

    @unittest.skipIf(os.name == "nt", "hard links and utime differ on Windows")
    def test_same_length_rewrite_through_an_alias_is_detected(self) -> None:
        """Finding 5: dev/ino/size/mtime equality was sold as byte
        immutability. A same-length rewrite through a hard link with mtime
        restored passed every one of those and changed the judged bytes."""
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            evidence, manifest = _complete_evidence(root)
            manifest_path = evidence / wv.EVIDENCE_MANIFEST
            accepting = _manifest_bytes(manifest)
            rejecting = accepting.replace(b'"evidence_complete"', b'"evidence_COMPLETE"')
            self.assertEqual(len(accepting), len(rejecting))
            manifest_path.write_bytes(rejecting)
            alias = root / "alias"
            os.link(manifest_path, alias)
            before = os.lstat(manifest_path)
            real_fstat = os.fstat
            fired: list[int] = []

            def fstat_then_rewrite(fd):
                result = real_fstat(fd)
                if not fired and (result.st_dev, result.st_ino) == (
                    before.st_dev,
                    before.st_ino,
                ):
                    with open(alias, "r+b") as handle:
                        handle.write(accepting)
                    os.utime(alias, ns=(before.st_atime_ns, before.st_mtime_ns))
                    fired.append(fd)
                return result

            with mock.patch.object(os, "fstat", fstat_then_rewrite):
                status = wv.read_status(root, "WV-6", verify_git=False)
        self.assertEqual(len(fired), 1)
        self.assertNotEqual(status.state, "accepted")
        self.assertIn(f"{manifest_path} changed while being read", status.findings)

    @unittest.skipUnless(hasattr(os, "mkfifo"), "no FIFOs on this platform")
    def test_non_regular_entries_in_the_evidence_directory_are_findings(self) -> None:
        """A symlink or a FIFO beside the evidence was silently left out of the
        inventory; an entry the manifest cannot describe is a finding."""
        for kind in ("symlink", "fifo"):
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as td:
                root = Path(td)
                evidence, manifest = _complete_evidence(root)
                (evidence / wv.EVIDENCE_MANIFEST).write_bytes(_manifest_bytes(manifest))
                if kind == "symlink":
                    (evidence / "stray").symlink_to(evidence / "nothing.txt")
                else:
                    os.mkfifo(evidence / "stray")
                status = wv.read_status(root, "WV-6", verify_git=False)
                self.assertNotEqual(status.state, "accepted")
                self.assertIn(
                    "evidence directory contains a non-regular entry stray",
                    status.findings,
                )


class UnsupportedPlatformTests(unittest.TestCase):
    """Finding 2: the no-dir_fd fallback re-checked and re-read by pathname,
    and a root swap after the symlink check was accepted. There is no
    fallback now: a platform that cannot bind the evidence directory says so
    and never accepts."""

    def test_platform_without_dir_fd_never_accepts(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            evidence, manifest = _complete_evidence(root)
            (evidence / wv.EVIDENCE_MANIFEST).write_bytes(_manifest_bytes(manifest))
            with mock.patch.object(wv, "DIR_FD_SUPPORTED", False):
                status = wv.read_status(root, "WV-6", verify_git=False)
        self.assertEqual(status.state, "partial")
        self.assertFalse(status.ok)
        self.assertEqual(
            status.findings,
            [
                "evidence identity cannot be bound on this platform "
                "(no dir_fd support); acceptance is not available here"
            ],
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