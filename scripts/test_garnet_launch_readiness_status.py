#!/usr/bin/env python3
"""Regression tests for the canonical launch-readiness ledger reporter.

Truth Lock Task 3: every consumed reporter is exercised, every launch gate
has at least one failure path, and the evidence-base validator is proven
against dirty, malformed, missing, unreachable, and clean-reachable values.
"""
from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import unittest
from dataclasses import asdict, replace
from pathlib import Path
from unittest import mock

SCRIPT = Path(__file__).with_name("garnet_launch_readiness_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_launch_readiness_status", SCRIPT)
assert SPEC is not None
status_mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_launch_readiness_status"] = status_mod
SPEC.loader.exec_module(status_mod)

REPO_ROOT = Path(__file__).resolve().parents[1]


def _gates_by_id(status):
    return {gate.id: gate for gate in status.gates}


class CurrentTreeTests(unittest.TestCase):
    """Assertions against the real repository state (no mocks)."""

    def test_current_launch_state_is_hold_with_measured_foundation(self) -> None:
        # Truth Lock Task 5 re-measured machine truth on a pristine tree:
        # the evidence base is a clean reachable short SHA, so foundation
        # passes — while launch stays honestly HOLD on the remaining gates.
        status = status_mod.read_status()
        gates = _gates_by_id(status)
        self.assertEqual("pass", gates["foundation_integrity"].state)
        # S114 is accepted-scoped: Jon's decision is recorded in the external
        # acceptance artifact and read (not graded) by the reporter.
        self.assertEqual("accepted-scoped", gates["s114_acceptance"].state)
        self.assertEqual([], gates["s114_acceptance"].blockers)
        self.assertEqual("remaining", gates["live_wasm_playground"].state)
        self.assertEqual("manual-deferred", gates["minimum_sealed_shelf"].state)
        self.assertEqual("jon-only", gates["launch_fire"].state)
        self.assertEqual("measured", status.evidence_base_status)
        # Accepting S114 does NOT flip launch readiness: other gates are open.
        self.assertFalse(status.launch_ready)
        self.assertEqual("HOLD", status.recommendation)

    def test_schema_and_source_are_stamped(self) -> None:
        status = status_mod.read_status()
        self.assertEqual("garnet.launch_readiness/v1", status.schema)
        self.assertTrue(status.source.endswith("garnet_launch_readiness_status.py"))

    def test_static_playground_is_partial_never_pass(self) -> None:
        status = status_mod.read_status()
        gates = _gates_by_id(status)
        self.assertEqual("partial", gates["static_playground"].state)

    def test_shelf_gate_is_never_reporter_derived(self) -> None:
        status = status_mod.read_status()
        gates = _gates_by_id(status)
        shelf = gates["minimum_sealed_shelf"]
        self.assertEqual("manual-deferred", shelf.state)
        self.assertTrue(
            any("no reporter" in b or "manual" in b for b in shelf.blockers),
            shelf.blockers,
        )

    def test_jon_only_actions_listed(self) -> None:
        status = status_mod.read_status()
        joined = " ".join(status.jon_only).lower()
        for expected in ("tag", "launch", "s114"):
            self.assertIn(expected, joined)


class GateSubprocessTests(unittest.TestCase):
    def test_gate_fails_while_playground_and_shelf_are_remaining(self) -> None:
        proc = subprocess.run(
            [sys.executable, str(SCRIPT), "--gate", "--format", "json"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(1, proc.returncode)
        self.assertFalse(json.loads(proc.stdout)["launch_ready"])

    def test_json_and_markdown_render(self) -> None:
        for fmt in ("json", "human", "markdown"):
            proc = subprocess.run(
                [sys.executable, str(SCRIPT), "--format", fmt],
                capture_output=True,
                text=True,
            )
            self.assertEqual(0, proc.returncode, proc.stderr)
            self.assertTrue(proc.stdout.strip())
        payload = json.loads(
            subprocess.run(
                [sys.executable, str(SCRIPT), "--format", "json"],
                capture_output=True,
                text=True,
            ).stdout
        )
        self.assertEqual("garnet.launch_readiness/v1", payload["schema"])

    def test_markdown_preserves_json_gate_order(self) -> None:
        json_proc = subprocess.run(
            [sys.executable, str(SCRIPT), "--format", "json"],
            capture_output=True,
            text=True,
        )
        md_proc = subprocess.run(
            [sys.executable, str(SCRIPT), "--format", "markdown"],
            capture_output=True,
            text=True,
        )
        gate_ids = [g["id"] for g in json.loads(json_proc.stdout)["gates"]]
        md = md_proc.stdout
        positions = [md.index(f"`{gid}`") for gid in gate_ids]
        self.assertEqual(positions, sorted(positions))
        for required in ("commit", "release grade", "recommendation", "Deferred", "Jon-only"):
            self.assertIn(required.lower(), md.lower())


class LedgerPinTests(unittest.TestCase):
    def test_tracked_ledger_matches_renderer_byte_for_byte(self) -> None:
        ledger = REPO_ROOT / "F_Project_Management" / "LAUNCH" / "LAUNCH_READINESS.md"
        self.assertTrue(ledger.is_file(), "canonical ledger missing")
        proc = subprocess.run(
            [sys.executable, str(SCRIPT), "--format", "markdown"],
            capture_output=True,
            text=True,
        )
        rendered = proc.stdout.replace(str(REPO_ROOT), "<repo>")
        tracked = ledger.read_text(encoding="utf-8").replace(str(REPO_ROOT), "<repo>")
        self.assertEqual(tracked, rendered)


class EvidenceBaseValidatorTests(unittest.TestCase):
    def test_dirty_suffix_is_unmeasured(self) -> None:
        value, state = status_mod.validate_evidence_base("c4b9e28-dirty")
        self.assertEqual("c4b9e28-dirty", value)
        self.assertEqual("unmeasured", state)

    def test_malformed_value_is_unmeasured(self) -> None:
        for bad in ("", "zzzzzzz", "12345", "not a sha", "c4b9e28 "):
            _, state = status_mod.validate_evidence_base(bad)
            self.assertEqual("unmeasured", state, bad)

    def test_missing_value_is_unmeasured(self) -> None:
        _, state = status_mod.validate_evidence_base(None)
        self.assertEqual("unmeasured", state)

    def test_unreachable_commit_is_unmeasured(self) -> None:
        # Well-formed hex that does not resolve to a commit in this repo.
        _, state = status_mod.validate_evidence_base("deadbeefcafe4242deadbeefcafe4242deadbeef")
        self.assertEqual("unmeasured", state)

    def test_clean_reachable_short_sha_is_measured(self) -> None:
        head_short = subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],
            capture_output=True,
            text=True,
            cwd=REPO_ROOT,
        ).stdout.strip()
        value, state = status_mod.validate_evidence_base(head_short)
        self.assertEqual(head_short, value)
        self.assertEqual("measured", state)


class MockedDependencyTests(unittest.TestCase):
    """Prove every consumed reporter is invoked and can flip its gate."""

    def _deps(self, **overrides):
        deps = status_mod.collect_dependencies()
        return replace(deps, **overrides)

    def test_all_dependencies_are_invoked(self) -> None:
        with mock.patch.object(
            status_mod.garnet_v0_8_1_release_readiness,
            "read_readiness",
            wraps=status_mod.garnet_v0_8_1_release_readiness.read_readiness,
        ) as release, mock.patch.object(
            status_mod.garnet_red_team_status,
            "read_status",
            wraps=status_mod.garnet_red_team_status.read_status,
        ) as red_team, mock.patch.object(
            status_mod.garnet_evidence_integrity_status,
            "read_status",
            wraps=status_mod.garnet_evidence_integrity_status.read_status,
        ) as integrity, mock.patch.object(
            status_mod.garnet_seccomp_apply_status,
            "read_status",
            wraps=status_mod.garnet_seccomp_apply_status.read_status,
        ) as seccomp, mock.patch.object(
            status_mod.garnet_native_debian_cli_install_status,
            "evaluate",
            wraps=status_mod.garnet_native_debian_cli_install_status.evaluate,
        ) as native_cli, mock.patch.object(
            status_mod.garnet_native_linux_studio_status,
            "evaluate",
            wraps=status_mod.garnet_native_linux_studio_status.evaluate,
        ) as native_studio, mock.patch.object(
            status_mod.garnet_playground_readiness,
            "read_readiness",
            wraps=status_mod.garnet_playground_readiness.read_readiness,
        ) as playground, mock.patch.object(
            status_mod.garnet_wasm_readiness,
            "read_readiness",
            wraps=status_mod.garnet_wasm_readiness.read_readiness,
        ) as wasm, mock.patch.object(
            status_mod.garnet_stdlib_layer_gate,
            "read_status",
            wraps=status_mod.garnet_stdlib_layer_gate.read_status,
        ) as stdlib, mock.patch.object(
            status_mod.garnet_promo_video_status,
            "read_status",
            wraps=status_mod.garnet_promo_video_status.read_status,
        ) as promo, mock.patch.object(
            status_mod.garnet_mit_readiness_status,
            "read_status",
            wraps=status_mod.garnet_mit_readiness_status.read_status,
        ) as mit:
            status_mod.read_status()
        # Some reporters cross-call others internally (e.g. MIT consumes
        # sibling statuses), so assert invocation, not exact call counts.
        for m in (
            release,
            red_team,
            integrity,
            seccomp,
            native_cli,
            native_studio,
            playground,
            wasm,
            stdlib,
            promo,
            mit,
        ):
            m.assert_called()

    def test_failed_release_readiness_blocks_foundation(self) -> None:
        deps = self._deps()
        deps = replace(deps, release=replace(deps.release, release_ready=False))
        status = status_mod.build_status(deps)
        gate = _gates_by_id(status)["foundation_integrity"]
        self.assertEqual("blocked", gate.state)
        self.assertTrue(any("release readiness" in b for b in gate.blockers))

    def test_failed_red_team_blocks_foundation(self) -> None:
        deps = self._deps()
        deps = replace(deps, red_team=replace(deps.red_team, ok=False))
        status = status_mod.build_status(deps)
        gate = _gates_by_id(status)["foundation_integrity"]
        self.assertEqual("blocked", gate.state)
        self.assertTrue(any("red-team" in b for b in gate.blockers))

    def test_failed_evidence_integrity_blocks_foundation(self) -> None:
        deps = self._deps()
        deps = replace(deps, integrity=replace(deps.integrity, ok=False))
        status = status_mod.build_status(deps)
        gate = _gates_by_id(status)["foundation_integrity"]
        self.assertEqual("blocked", gate.state)
        self.assertTrue(any("evidence-integrity" in b for b in gate.blockers))

    def test_measured_base_with_green_inputs_passes_foundation(self) -> None:
        deps = self._deps()
        deps = replace(
            deps,
            release=replace(deps.release, release_ready=True),
            red_team=replace(deps.red_team, ok=True),
            integrity=replace(deps.integrity, ok=True),
            evidence_base="0000000",
            evidence_base_status="measured",
        )
        status = status_mod.build_status(deps)
        gate = _gates_by_id(status)["foundation_integrity"]
        self.assertEqual("pass", gate.state)

    def test_failed_native_cli_blocks_native_linux(self) -> None:
        deps = self._deps()
        deps = replace(deps, native_cli=replace(deps.native_cli, ok=False))
        status = status_mod.build_status(deps)
        gate = _gates_by_id(status)["native_linux"]
        self.assertEqual("blocked", gate.state)
        self.assertTrue(any("native Debian CLI" in b for b in gate.blockers))

    def test_failed_native_studio_blocks_native_linux(self) -> None:
        deps = self._deps()
        deps = replace(deps, native_studio=replace(deps.native_studio, ok=False))
        status = status_mod.build_status(deps)
        self.assertEqual("blocked", _gates_by_id(status)["native_linux"].state)

    def test_failed_seccomp_blocks_native_linux(self) -> None:
        deps = self._deps()
        deps = replace(deps, seccomp=replace(deps.seccomp, ok=False))
        status = status_mod.build_status(deps)
        self.assertEqual("blocked", _gates_by_id(status)["native_linux"].state)

    def test_failed_playground_degrades_static_playground(self) -> None:
        deps = self._deps()
        deps = replace(deps, playground=replace(deps.playground, ok=False))
        status = status_mod.build_status(deps)
        gate = _gates_by_id(status)["static_playground"]
        self.assertEqual("blocked", gate.state)

    def test_wasm_blockers_surface_on_live_playground(self) -> None:
        deps = self._deps()
        deps = replace(
            deps,
            wasm=replace(deps.wasm, blockers=["synthetic wasm blocker"]),
        )
        status = status_mod.build_status(deps)
        gate = _gates_by_id(status)["live_wasm_playground"]
        self.assertEqual("remaining", gate.state)
        self.assertIn("synthetic wasm blocker", gate.blockers)

    def test_stdlib_gate_failure_surfaces_as_shelf_blocker(self) -> None:
        deps = self._deps()
        deps = replace(deps, stdlib_meets_count_gate=False)
        status = status_mod.build_status(deps)
        gate = _gates_by_id(status)["minimum_sealed_shelf"]
        self.assertEqual("manual-deferred", gate.state)
        self.assertTrue(any("stdlib layer gate" in b for b in gate.blockers))

    def test_promo_stays_pending_human_even_when_reporter_completes(self) -> None:
        deps = self._deps()
        deps = replace(deps, promo=replace(deps.promo, status="complete"))
        status = status_mod.build_status(deps)
        self.assertEqual("pending-human", _gates_by_id(status)["promo_video"].state)

    def test_mit_failure_surfaces_as_foundation_evidence_regression(self) -> None:
        deps = self._deps()
        deps = replace(deps, mit_overall_status="regressed", mit_completion_percent=0.0)
        status = status_mod.build_status(deps)
        gate = _gates_by_id(status)["foundation_integrity"]
        self.assertTrue(any("MIT" in b for b in gate.blockers))

    def test_launch_ready_requires_every_critical_gate(self) -> None:
        deps = self._deps()
        status = status_mod.build_status(deps)
        self.assertFalse(status.launch_ready)
        self.assertEqual("HOLD", status.recommendation)


class S114AcceptanceGateTests(unittest.TestCase):
    """The S114 gate reflects, never grades, the external acceptance artifact."""

    def test_accepted_scoped_when_artifact_present(self) -> None:
        status = status_mod.read_status()
        gate = _gates_by_id(status)["s114_acceptance"]
        self.assertEqual("accepted-scoped", gate.state)
        self.assertEqual([], gate.blockers)
        self.assertTrue(any("accepted (scoped)" in e for e in gate.evidence))
        self.assertTrue(
            any("not an independence relabel" in e for e in gate.evidence),
            gate.evidence,
        )
        self.assertTrue(any("current scope limit (tracked)" in e for e in gate.evidence))
        self.assertTrue(any("condition-5-reopened" in e for e in gate.evidence))

    def test_external_pending_without_artifact(self) -> None:
        with mock.patch.object(status_mod, "read_s114_acceptance", return_value=None):
            status = status_mod.read_status()
        gate = _gates_by_id(status)["s114_acceptance"]
        self.assertEqual("external-pending", gate.state)
        self.assertTrue(gate.blockers)

    def test_real_committed_artifact_validates(self) -> None:
        data = status_mod.read_s114_acceptance()
        self.assertIsNotNone(data)
        self.assertEqual("accepted-scoped", data["state"])
        self.assertTrue(str(data["scope"]).strip())

    def test_reader_rejects_wrong_state_or_scopeless(self) -> None:
        import os
        import tempfile

        for payload in (
            {"schema": "garnet.s114_acceptance/v1", "state": "rejected", "scope": "x"},
            {"schema": "garnet.s114_acceptance/v1", "state": "accepted-scoped", "scope": "  "},
            {"schema": "wrong/v1", "state": "accepted-scoped", "scope": "x"},
            ["not", "a", "dict"],
        ):
            with tempfile.NamedTemporaryFile(
                "w", suffix=".json", delete=False, encoding="utf-8"
            ) as fh:
                json.dump(payload, fh)
                tmp = fh.name
            try:
                with mock.patch.object(status_mod, "S114_ACCEPTANCE_JSON", Path(tmp)):
                    self.assertIsNone(status_mod.read_s114_acceptance(), payload)
            finally:
                os.unlink(tmp)

    def test_reader_returns_none_when_file_missing(self) -> None:
        with mock.patch.object(
            status_mod, "S114_ACCEPTANCE_JSON", Path("/nonexistent/s114.json")
        ):
            self.assertIsNone(status_mod.read_s114_acceptance())

    def test_accepted_s114_is_not_a_launch_blocker(self) -> None:
        # Accepting S114 removes it from the set of open launch-critical gates;
        # launch stays HOLD only because of the remaining (playground/wasm/shelf)
        # gates, never because of S114.
        status = status_mod.read_status()
        by_id = _gates_by_id(status)
        self.assertIn(by_id["s114_acceptance"].state, ("pass", "accepted-scoped"))
        satisfying = ("pass", "accepted-scoped")
        open_critical = [
            gid
            for gid in status_mod.LAUNCH_CRITICAL_GATES
            if by_id[gid].state not in satisfying
        ]
        self.assertNotIn("s114_acceptance", open_critical)
        self.assertTrue(open_critical, "some non-S114 critical gate must still be open")
        self.assertFalse(status.launch_ready)


class PromoSnapshotTests(unittest.TestCase):
    """The promo ledger line is pinned to the committed snapshot."""

    def test_ledger_line_uses_canonical_snapshot(self) -> None:
        snap = status_mod.read_promo_snapshot()
        self.assertIsNotNone(snap)
        status = status_mod.read_status()
        gate = _gates_by_id(status)["promo_video"]
        self.assertTrue(
            any(snap[0] in e and f"{snap[1]:.1f}%" in e for e in gate.evidence),
            gate.evidence,
        )

    def test_falls_back_to_live_probe_without_snapshot(self) -> None:
        with mock.patch.object(status_mod, "read_promo_snapshot", return_value=None):
            deps = status_mod.collect_dependencies()
        self.assertIsInstance(deps.promo.completion_percent, float)

    def test_reader_rejects_malformed_snapshot(self) -> None:
        import os
        import tempfile

        for payload in (
            {"schema": "wrong/v1", "status": "x", "completion_percent": 1.0},
            {"schema": "garnet.promo_evidence_snapshot/v1", "status": "", "completion_percent": 1.0},
            {"schema": "garnet.promo_evidence_snapshot/v1", "status": "x", "completion_percent": "nope"},
            {"schema": "garnet.promo_evidence_snapshot/v1", "status": "x", "completion_percent": True},
        ):
            with tempfile.NamedTemporaryFile(
                "w", suffix=".json", delete=False, encoding="utf-8"
            ) as fh:
                json.dump(payload, fh)
                tmp = fh.name
            try:
                with mock.patch.object(status_mod, "PROMO_SNAPSHOT_JSON", Path(tmp)):
                    self.assertIsNone(status_mod.read_promo_snapshot(), payload)
            finally:
                os.unlink(tmp)


class U31CureTrapTests(unittest.TestCase):
    """Cross-family Verdict 09 (Codex GPT-5.6 Sol) trap set for the bounded
    U-31 cure: ``08.source`` moves from a host-absolute path to the
    repository-relative POSIX producer path.

    RED-before-implement note: the load-bearing RED is
    ``test_trap1_source_is_exact_repo_relative_posix_path`` — it fails against
    the un-cured absolute-path reporter and turns GREEN only with the cure.
    The trap-2/3/4 methods are *standing regression guards* committed alongside
    it; they hold in both states and fail any cure that would break real-state
    sensitivity, add collateral render semantics, or weaken the digest
    exclusion set.
    """

    AUTHORIZED_SOURCE = "scripts/garnet_launch_readiness_status.py"

    def test_trap1_source_is_exact_repo_relative_posix_path(self) -> None:
        # RED pre-cure (absolute host path); GREEN post-cure.
        status = status_mod.read_status()
        self.assertEqual(self.AUTHORIZED_SOURCE, status.source)
        self.assertFalse(status.source.startswith("/"), status.source)
        self.assertFalse(Path(status.source).is_absolute(), status.source)
        self.assertNotIn("\\", status.source)
        # The pre-existing suffix contract is preserved by the cure.
        self.assertTrue(status.source.endswith("garnet_launch_readiness_status.py"))

    def test_trap2_real_dependency_change_moves_artifact_source_constant(self) -> None:
        # A genuine readiness change (through the existing Dependencies seam)
        # must still move the serialized artifact; source never does the moving.
        deps = status_mod.collect_dependencies()
        baseline = status_mod.render_json(status_mod.build_status(deps))
        mutated = status_mod.render_json(
            status_mod.build_status(
                replace(deps, wasm=replace(deps.wasm, blockers=["u31-trap2 probe"]))
            )
        )
        self.assertNotEqual(baseline, mutated)
        self.assertEqual(json.loads(baseline)["source"], json.loads(mutated)["source"])
        # ``replace`` is non-mutating: the original state restores cleanly.
        self.assertEqual(baseline, status_mod.render_json(status_mod.build_status(deps)))

    def test_trap3_source_only_change_no_collateral_semantics(self) -> None:
        # The key is retained, schema/order are stable, and mutating ONLY the
        # source value moves the JSON by exactly one line while leaving human
        # and markdown renders byte-identical — the cure has no other reach.
        status = status_mod.read_status()
        payload = asdict(status)
        self.assertIn("source", payload)
        self.assertEqual("garnet.launch_readiness/v1", payload["schema"])
        self.assertEqual(["schema", "source"], list(payload.keys())[:2])
        other = replace(status, source="u31-trap3-sentinel")
        self.assertEqual(
            status_mod.render_human(status), status_mod.render_human(other)
        )
        self.assertEqual(
            status_mod.render_markdown(status), status_mod.render_markdown(other)
        )
        base_lines = status_mod.render_json(status).splitlines()
        other_lines = status_mod.render_json(other).splitlines()
        self.assertEqual(len(base_lines), len(other_lines))
        changed = [i for i, (x, y) in enumerate(zip(base_lines, other_lines)) if x != y]
        self.assertEqual(1, len(changed), changed)
        self.assertIn('"source"', base_lines[changed[0]])

    def test_trap4_frozen_exclusion_tuple_and_lane0_inclusion(self) -> None:
        # Digest determinism without exclusion: the frozen tuple is exactly the
        # four authorized prefixes plus the Shelf reporter self-path, and every
        # tracked ops/lane0/ path stays digest-INCLUDED.
        import garnet_content_provenance as cp

        self.assertEqual(
            (
                b"ops/lane2b/",
                b"proofs/",
                b"F_Project_Management/W_TRUST/",
                b"ops/lane1/",
            ),
            cp.FROZEN_MUTABLE_PREFIXES,
        )
        self.assertEqual(b"scripts/smoke_garnet_minimum_shelf.py", cp.REPORTER_PATH)
        raw = subprocess.run(
            ["git", "ls-files", "-z", "--", "ops/lane0/"],
            capture_output=True,
            cwd=REPO_ROOT,
        ).stdout
        lane0 = [p for p in raw.split(b"\0") if p]
        self.assertTrue(lane0, "expected tracked ops/lane0/ paths")
        self.assertEqual([], [p for p in lane0 if cp._is_mutable(p)])


if __name__ == "__main__":
    unittest.main()
