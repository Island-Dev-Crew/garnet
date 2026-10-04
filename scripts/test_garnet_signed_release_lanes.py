#!/usr/bin/env python3
"""Regression tests for the signed release lanes reporter (S51)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_signed_release_lanes.py")
SPEC = importlib.util.spec_from_file_location("garnet_signed_release_lanes", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
lanes = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_signed_release_lanes"] = lanes
SPEC.loader.exec_module(lanes)


class SignedReleaseLanesTests(unittest.TestCase):
    def test_three_lanes(self) -> None:
        s = lanes.read_lanes()
        ids = {l.id for l in s.lanes}
        self.assertEqual(
            ids, {"program-manifest", "release-artifact", "supply-chain-attestation"}
        )

    def test_active_lane_is_program_manifest_and_wired(self) -> None:
        s = lanes.read_lanes()
        program = next(l for l in s.lanes if l.id == "program-manifest")
        self.assertEqual(program.status, "active")
        self.assertTrue(program.owned_by_garnet)
        self.assertTrue(s.active_lane_ok, "program-manifest signing must be wired in CI")

    def test_release_artifact_lane_is_active_and_partial_lane_stays_partial(self) -> None:
        s = lanes.read_lanes()
        by_id = {l.id: l for l in s.lanes}
        # SHA256SUMS.asc has shipped since v0.8.1: the lane is active, not deferred.
        self.assertEqual(by_id["release-artifact"].status, "active")
        self.assertTrue(by_id["release-artifact"].present)
        self.assertEqual(by_id["supply-chain-attestation"].status, "partial")
        # External-tool lanes are explicitly NOT claimed as Garnet-owned.
        self.assertFalse(by_id["release-artifact"].owned_by_garnet)
        self.assertFalse(by_id["supply-chain-attestation"].owned_by_garnet)

    def test_release_artifact_lane_keys_on_the_signing_step_not_the_old_todo(self) -> None:
        workflow = lanes._read(".github/workflows/linux-packages.yml")
        self.assertNotIn("TODO(release-security)", workflow)
        stale = "# TODO(release-security): SHA256SUMS is computed but not signed.\nsha256sum * > SHA256SUMS\n"
        self.assertEqual(lanes.release_artifact_lane(stale).status, "broken")
        for piece in lanes.RELEASE_ARTIFACT_EVIDENCE:
            with self.subTest(missing=piece):
                self.assertIn(piece, workflow)
                self.assertEqual(lanes.release_artifact_lane(workflow.replace(piece, "")).status, "broken")

    def test_gate_fails_when_release_artifact_signing_is_unwired(self) -> None:
        original = lanes._read

        def without_asc_upload(rel: str) -> str:
            text = original(rel)
            return text.replace("files: release-dist/SHA256SUMS.asc", "") if rel.endswith("linux-packages.yml") else text

        lanes._read = without_asc_upload
        try:
            self.assertEqual(lanes.main(["--gate", "--format", "json"]), 1)
        finally:
            lanes._read = original

    def test_release_artifact_lane_breaks_when_the_signing_steps_change_but_keep_their_names(self) -> None:
        # Codex lane A on #607: the reporter only looked for the refusal step's name, so a
        # refusal that no longer refused stayed "active".
        workflow = lanes._read(".github/workflows/linux-packages.yml")
        refusal = "      - name: Require signed SHA256SUMS (fail-closed)\n"
        publish = "      - name: Publish release\n"
        self.assertEqual(1, workflow.count(refusal))
        self.assertEqual(1, workflow.count(publish))
        refusal_at = workflow.index(refusal)
        refusal_end = workflow.index(publish)
        refusal_block = workflow[refusal_at:refusal_end]
        cases = {
            "the refusal exits 0": workflow.replace(
                refusal_block, refusal_block.replace("            exit 1\n", "            exit 0\n")
            ),
            "the refusal's condition never matches": workflow.replace(
                refusal_block, refusal_block.replace("if: env.HAS_GPG != 'true'", "if: false")
            ),
            "the signature is attached unconditionally": workflow.replace(
                "      - name: Attach SHA256SUMS signature (only when signed)\n        if: env.HAS_GPG == 'true'\n",
                "      - name: Attach SHA256SUMS signature (only when signed)\n        if: always()\n",
            ),
            "HAS_GPG no longer follows the key": workflow.replace(
                "      HAS_GPG: ${{ secrets.GPG_SIGNING_KEY != '' }}\n", "      HAS_GPG: 'false'\n"
            ),
        }
        # The reordering case moves the refusal block after Publish release.
        moved = workflow[:refusal_at] + workflow[refusal_end:]
        publish_end = moved.index("      - name: Attach SHA256SUMS signature (only when signed)\n")
        cases["the refusal runs after the release is published"] = moved[:publish_end] + refusal_block + moved[publish_end:]
        for label, text in cases.items():
            with self.subTest(label):
                self.assertNotEqual(workflow, text, "the mutation must change the workflow")
                self.assertEqual("broken", lanes.release_artifact_lane(text).status)
        self.assertEqual("active", lanes.release_artifact_lane(workflow).status)

    def test_release_artifact_lane_breaks_when_publication_can_run_past_the_refusal(self) -> None:
        # Codex delta review on #607: `if: always()` on Publish release kept every
        # signing step pinned and in order, yet published after the refusal failed.
        workflow = lanes._read(".github/workflows/linux-packages.yml")
        publish = "      - name: Publish release\n"
        attach = "      - name: Attach SHA256SUMS signature (only when signed)\n"
        sign = "      - name: Sign SHA256SUMS (activates when GPG_SIGNING_KEY secret is set)\n"
        release_job = "\n  release:\n"
        self.assertEqual(1, workflow.count(publish))
        cases = {
            "publish runs always": workflow.replace(publish, publish + "        if: always()\n"),
            "a step after the refusal publishes regardless": workflow.replace(
                publish,
                "      - name: Publish anyway\n        if: always()\n        run: gh release create \"$GITHUB_REF_NAME\" release-dist/*\n"
                + publish,
            ),
            "the job continues on error": workflow.replace(
                release_job + "    if: startsWith(github.ref, 'refs/tags/v')\n",
                release_job + "    if: startsWith(github.ref, 'refs/tags/v')\n    continue-on-error: true\n",
            ),
            "a step before signing publishes": workflow.replace(
                sign, "      - name: Early publish\n        run: gh release create \"$GITHUB_REF_NAME\" release-dist/*\n\n" + sign
            ),
            "publish continues on error": workflow.replace(
                publish + "        uses:", publish + "        continue-on-error: true\n        uses:"
            ),
        }
        self.assertIn(attach, workflow)
        for label, text in cases.items():
            with self.subTest(label):
                self.assertNotEqual(workflow, text, "the mutation must change the workflow")
                self.assertEqual("broken", lanes.release_artifact_lane(text).status)
        self.assertEqual("active", lanes.release_artifact_lane(workflow).status)

    def test_release_artifact_lane_breaks_on_any_edit_to_the_release_job_or_a_writer_elsewhere(self) -> None:
        # Codex delta round 3 on #607: a named step running `gh --repo ... release create`
        # before signing, and an unnamed release-action step with `if: always()` after the
        # refusal, both kept the lane active. A list of publishing commands is never
        # complete, so the reviewed release job is pinned whole and no other job may hold
        # a token that can write a release.
        workflow = lanes._read(".github/workflows/linux-packages.yml")
        sign = "      - name: Sign SHA256SUMS (activates when GPG_SIGNING_KEY secret is set)\n"
        attach = "      - name: Attach SHA256SUMS signature (only when signed)\n"
        release_job = "\n  release:\n"
        self.assertEqual(1, workflow.count(sign))
        self.assertEqual(1, workflow.count(release_job))
        early_job = (
            "\n  early-publish:\n    runs-on: ubuntu-24.04\n"
            "    permissions:\n      contents: write\n"
            "    steps:\n      - run: gh release create \"$GITHUB_REF_NAME\"\n"
        )
        cases = {
            "a named step publishes with gh --repo before signing": workflow.replace(
                sign,
                "      - name: Publish assets\n        env:\n          GH_TOKEN: ${{ github.token }}\n"
                "        run: gh --repo \"$GITHUB_REPOSITORY\" release create \"$GITHUB_REF_NAME\" release-dist/*\n\n"
                + sign,
            ),
            "an unnamed release-action step runs always after the refusal": workflow.rstrip("\n")
            + "\n      - uses: softprops/action-gh-release@efb35369e0ad2afab669f228072c1b0d510eae64 # v3.0.3\n"
            "        if: always()\n        with:\n          files: release-dist/SHA256SUMS\n",
            "a step calls the releases API with curl": workflow.replace(
                sign,
                "      - name: Prepare\n        run: curl -X POST -H \"Authorization: Bearer $T\" "
                "\"https://api.github.com/repos/$GITHUB_REPOSITORY/releases\"\n\n" + sign,
            ),
            "the attach step gains a line": workflow.replace(
                attach, attach + "        continue-on-error: false\n"
            ),
            "another job may write contents": workflow.replace(release_job, early_job + release_job),
            "another job may write everything": workflow.replace(
                release_job,
                early_job.replace("    permissions:\n      contents: write\n", "    permissions: write-all\n")
                + release_job,
            ),
            "another job reads a secret": workflow.replace(
                release_job,
                early_job.replace(
                    "    permissions:\n      contents: write\n",
                    "    env:\n      GH_TOKEN: ${{ secrets.RELEASE_PAT }}\n",
                )
                + release_job,
            ),
            # The YAML is read decoded, so another spelling of the same key or
            # value counts: a quoted key, an escape in a double-quoted value, a
            # differently cased context name, or permissions for the whole workflow.
            "another job quotes its permission key": workflow.replace(
                release_job,
                early_job.replace("      contents: write\n", '      "contents": write\n') + release_job,
            ),
            "another job reads an escaped secret": workflow.replace(
                release_job,
                early_job.replace(
                    "    permissions:\n      contents: write\n",
                    '    env:\n      GH_TOKEN: "${{ s\\x65crets.RELEASE_PAT }}"\n',
                )
                + release_job,
            ),
            "another job reads SECRETS": workflow.replace(
                release_job,
                early_job.replace(
                    "    permissions:\n      contents: write\n",
                    "    env:\n      GH_TOKEN: ${{ SECRETS.RELEASE_PAT }}\n",
                )
                + release_job,
            ),
            "the workflow grants write to every job": workflow.replace(
                "\njobs:\n", "\npermissions:\n  contents: write\n\njobs:\n", 1
            ),
        }
        for label, text in cases.items():
            with self.subTest(label):
                self.assertNotEqual(workflow, text, "the mutation must change the workflow")
                self.assertEqual("broken", lanes.release_artifact_lane(text).status)
        self.assertEqual("active", lanes.release_artifact_lane(workflow).status)

    def test_supply_chain_lane_notes_out_flag(self) -> None:
        s = lanes.read_lanes()
        sc = next(l for l in s.lanes if l.id == "supply-chain-attestation")
        self.assertTrue(sc.present, "seal --out + cosign detection must be present")

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(lanes.main(["--gate", "--format", "json"]), 0)

    def test_markdown_states_honest_scope(self) -> None:
        md = lanes.render_markdown(lanes.read_lanes())
        self.assertIn("does not sign its own supply chain", md)
        self.assertIn("active", md)


if __name__ == "__main__":
    unittest.main()
