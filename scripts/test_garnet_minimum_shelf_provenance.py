#!/usr/bin/env python3
"""Adversarial tests for Lane 2B squash-durable content provenance."""
from __future__ import annotations

import importlib.util
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


def _load(name: str, filename: str):
    script = Path(__file__).with_name(filename)
    spec = importlib.util.spec_from_file_location(name, script)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


shelf = _load("smoke_garnet_minimum_shelf", "smoke_garnet_minimum_shelf.py")
wv = _load("garnet_wv_acceptance_status", "garnet_wv_acceptance_status.py")
cp = _load("garnet_content_provenance", "garnet_content_provenance.py")


class SquashDurableContentProvenanceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self._git("init")
        self._git("config", "user.email", "lane2b@example.invalid")
        self._git("config", "user.name", "Lane 2B Test")
        self._write("product.txt", "reviewed product\n")
        self._write("ops/lane2b/note.txt", "mutable evidence\n")
        self._write("proofs/lane2b/proof.txt", "mutable proof\n")
        self._write(
            "F_Project_Management/W_TRUST/review.md", "mutable companion\n"
        )
        self._write(
            "scripts/smoke_garnet_minimum_shelf.py", "self excluded\n"
        )
        self._write("ops/wv6-reaccept/journal.md", "frozen record\n")
        self._write(
            "ops/wv6-reaccept/review/01-request.md", "frozen request\n"
        )
        self._git("add", ".")
        self._git("commit", "-m", "synthetic squash result")
        self._git("branch", "-M", "main")
        self.landed = self._git("rev-parse", "HEAD")
        self._git("update-ref", "refs/remotes/origin/main", self.landed)

    def tearDown(self) -> None:
        self.temp.cleanup()

    def _write(self, relative: str, text: str) -> None:
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")

    def _git(self, *args: str, check: bool = True) -> str:
        result = subprocess.run(
            ["git", "--no-replace-objects", *args],
            cwd=self.root,
            capture_output=True,
            text=True,
            check=False,
        )
        if check and result.returncode != 0:
            self.fail(f"git {' '.join(args)} failed: {result.stderr or result.stdout}")
        return result.stdout.strip()

    def test_any_product_blob_change_is_red(self) -> None:
        expected, _ = shelf._tracked_content_digest(self.root)
        self._write("product.txt", "tampered product\n")
        self._git("add", "product.txt")
        findings = shelf._verify_product_content(self.root, expected)
        self.assertTrue(
            any("product content digest" in item for item in findings), findings
        )

    def test_absent_branch_commits_are_green_on_fresh_main_content(self) -> None:
        for discarded in (
            "a6f0da2b81a9b181dafb83e15a17f8f313406e49",
            "e2820ce54e9c1fee030d50e9fba31be4bcdc8891",
        ):
            self.assertNotEqual(
                0,
                subprocess.run(
                    ["git", "cat-file", "-e", f"{discarded}^{{commit}}"],
                    cwd=self.root,
                    capture_output=True,
                    check=False,
                ).returncode,
            )
        expected, _ = shelf._tracked_content_digest(self.root)
        findings, landed = wv._verify_squash_durable_content(
            self.root,
            reviewed_head="a" * 40,
            reviewed_tree="b" * 40,
            expected_content_digest=expected,
            verify_git=True,
        )
        self.assertEqual([], findings)
        self.assertEqual(self.landed, landed)

    def test_mismatched_evidence_content_digest_is_red(self) -> None:
        findings, _ = wv._verify_squash_durable_content(
            self.root,
            reviewed_head="a" * 40,
            reviewed_tree="b" * 40,
            expected_content_digest="0" * 64,
            verify_git=True,
        )
        self.assertTrue(
            any("product content digest" in item for item in findings), findings
        )

    # ------------------------------------------------------------------ U-35
    # The frozen construction must treat Lane 1's own review/operational
    # artifacts as non-product (like ops/lane2b/), so a review round cannot move
    # the product digest and invalidate the WV pin at merge.

    def test_frozen_mutable_prefixes_are_exactly_the_authorized_set(self) -> None:
        # Trap (d): the digest definition remains exactly the three historical
        # prefixes plus the Lane 1 namespace — never a general predicate and
        # never the separately tolerated WV-6 post-acceptance record class.
        self.assertEqual(
            cp.FROZEN_MUTABLE_PREFIXES,
            (
                b"ops/lane2b/",
                b"proofs/",
                b"F_Project_Management/W_TRUST/",
                b"ops/lane1/",
            ),
        )
        self.assertEqual(cp.REPORTER_PATH, b"scripts/smoke_garnet_minimum_shelf.py")
        # Lane 1 is excluded; a sibling lane namespace is NOT (a general
        # ops/<lane>/ predicate would wrongly exclude ops/lane3/ and fail here).
        self.assertTrue(cp._is_mutable(b"ops/lane1/review/07-request.md"))
        self.assertFalse(cp._is_mutable(b"ops/lane3/note.txt"))

    def test_lane1_review_artifacts_do_not_move_the_digest(self) -> None:
        # Trap (b): adding and modifying only ops/lane1/ artifacts leaves the
        # digest and count byte-identical.
        before, count_before = cp.tracked_content_digest(self.root)
        self._write("ops/lane1/review/99-later-request.md", "later review artifact\n")
        self._write("ops/lane1/journal.md", "heartbeat line\n")
        self._write("ops/lane1/evidence/zz.txt", "evidence\n")
        self._git("add", ".")
        after, count_after = cp.tracked_content_digest(self.root)
        self.assertEqual(before, after)
        self.assertEqual(count_before, count_after)

    def test_wv6_reaccept_records_remain_in_the_frozen_digest_definition(self) -> None:
        before, count_before = cp.tracked_content_digest(self.root)
        self._write("ops/wv6-reaccept/review/03-verdict.md", "later verdict\n")
        self._write("ops/wv6-reaccept/journal.md", "record heartbeat\n")
        self._write("ops/wv6-reaccept/evidence/topology.txt", "topology evidence\n")
        self._git("add", ".")
        after, count_after = cp.tracked_content_digest(self.root)
        self.assertNotEqual(before, after)
        self.assertEqual(count_before + 2, count_after)

    def test_record_only_tip_drift_preserves_the_frozen_pair(self) -> None:
        frozen_head = self._git("rev-parse", "HEAD")
        frozen_tree = self._git("rev-parse", "HEAD^{tree}")
        expected, expected_count = cp.tracked_content_digest(self.root, frozen_head)
        self._write("ops/wv6-reaccept/journal.md", "post-acceptance heartbeat\n")
        self._write("ops/wv6-reaccept/review/01-verdict.md", "verdict record\n")
        self._write("proofs/wv6/capture.txt", "proof record\n")
        self._write(
            "F_Project_Management/W_TRUST/wv6.review.json", "review record\n"
        )
        self._write("scripts/smoke_garnet_minimum_shelf.py", "reporter rebind\n")
        self._git("add", ".")
        self._git("commit", "-m", "post-acceptance records")

        findings, landed = wv._verify_squash_durable_content(
            self.root,
            reviewed_head=frozen_head,
            reviewed_tree=frozen_tree,
            expected_content_digest=expected,
            verify_git=True,
        )
        self.assertEqual([], findings)
        self.assertIsNone(landed)
        self.assertEqual(
            (expected, expected_count),
            cp.tracked_content_digest(self.root, frozen_head),
        )
        self.assertNotEqual(
            (expected, expected_count), cp.tracked_content_digest(self.root)
        )

    def test_one_non_record_byte_in_tip_drift_is_red(self) -> None:
        frozen_head = self._git("rev-parse", "HEAD")
        frozen_tree = self._git("rev-parse", "HEAD^{tree}")
        expected, _ = cp.tracked_content_digest(self.root, frozen_head)
        self._write("ops/wv6-reaccept/journal.md", "allowed record drift\n")
        self._write("product.txt", "reviewed producU\n")
        self._git("add", ".")
        self._git("commit", "-m", "one product byte plus records")

        findings, _ = wv._verify_squash_durable_content(
            self.root,
            reviewed_head=frozen_head,
            reviewed_tree=frozen_tree,
            expected_content_digest=expected,
            verify_git=True,
        )
        self.assertIn(
            "post-acceptance drift contains non-record path: product.txt", findings
        )

    def test_included_product_change_moves_while_lane1_does_not(self) -> None:
        # Trap (a): the crux pair. A product blob change moves the digest AND
        # trips the content verifier; an ops/lane1/-only change does neither.
        baseline, _ = cp.tracked_content_digest(self.root)
        self._write("ops/lane1/evidence/only-lane1.txt", "lane1 only\n")
        self._git("add", ".")
        lane1_digest, _ = cp.tracked_content_digest(self.root)
        self.assertEqual(baseline, lane1_digest)
        self.assertEqual([], shelf._verify_product_content(self.root, baseline))
        self._write("product.txt", "tampered product\n")
        self._git("add", ".")
        product_digest, _ = cp.tracked_content_digest(self.root)
        self.assertNotEqual(baseline, product_digest)
        self.assertTrue(
            any(
                "product content digest" in item
                for item in shelf._verify_product_content(self.root, baseline)
            )
        )


if __name__ == "__main__":
    unittest.main()
