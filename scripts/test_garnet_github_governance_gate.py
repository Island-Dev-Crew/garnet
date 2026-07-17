#!/usr/bin/env python3
"""Adversarial offline tests for the GOV-009 exact-head/outcome gate."""
from __future__ import annotations

import copy
import contextlib
import importlib.util
import io
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from dataclasses import asdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
from types import SimpleNamespace
from unittest import mock


ROOT = Path(__file__).resolve().parents[1]
REVIEWED_HEAD = "a" * 40
NOW = datetime(2026, 7, 17, 6, 0, 0, tzinfo=timezone.utc)
WORKFLOW_NAMES = {
    ".github/workflows/agentic-dogfood-matrix.yml": "Agentic dogfood matrix",
    ".github/workflows/ci.yml": "CI",
    ".github/workflows/codeql.yml": "CodeQL",
    ".github/workflows/determinism.yml": "Determinism",
    ".github/workflows/dogfood-readiness.yml": "Dogfood readiness",
    ".github/workflows/fuzz-nightly.yml": "Parser fuzz nightly",
    ".github/workflows/linux-packages.yml": "Linux packages",
    ".github/workflows/macos-studio.yml": "Studio Trust",
    ".github/workflows/security.yml": "Security",
    ".github/workflows/vscode-extension.yml": "VS Code extension",
    ".github/workflows/web-pwa-readiness.yml": "Web PWA Readiness",
}


def load(name: str) -> object:
    path = ROOT / "scripts" / f"{name}.py"
    spec = importlib.util.spec_from_file_location(f"_{name}_test", path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


gate = load("garnet_github_governance_gate")


class GovernanceGateTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        inventory = json.loads(
            (ROOT / ".github/rulesets/required-context-producers.json").read_text(
                encoding="utf-8"
            )
        )
        active = [
            row
            for row in inventory["producers"]
            if row["context"] not in set(inventory["optional_contexts"])
        ]
        by_path: dict[str, object] = {}
        bindings: list[object] = []
        for row in active:
            workflow = by_path.setdefault(
                row["workflow"],
                SimpleNamespace(
                    name=SimpleNamespace(value=WORKFLOW_NAMES[row["workflow"]])
                ),
            )
            bindings.append(
                SimpleNamespace(
                    producer=SimpleNamespace(
                        context=row["context"],
                        workflow=row["workflow"],
                        event=row["event"],
                        job=row["job"],
                        matrix=(
                            tuple(next(iter(row["matrix"].items())))
                            if "matrix" in row
                            else None
                        ),
                        semantic_sha256=row["semantic_sha256"],
                    ),
                    workflow=workflow,
                    observed_semantic_sha256=row["semantic_sha256"],
                )
            )
        cls.policy = SimpleNamespace(bindings=tuple(bindings), problems=())
        assert len(cls.policy.bindings) == 31
        base_row = next(
            row
            for row in inventory["producers"]
            if row["context"] == "Base-controlled trust policy"
        )
        base_workflow = SimpleNamespace(
            name=SimpleNamespace(value="Base-controlled trust")
        )
        base_binding = SimpleNamespace(
            producer=SimpleNamespace(
                context=base_row["context"],
                workflow=base_row["workflow"],
                event=base_row["event"],
                job=base_row["job"],
                matrix=None,
                semantic_sha256=base_row["semantic_sha256"],
            ),
            workflow=base_workflow,
            observed_semantic_sha256=base_row["semantic_sha256"],
        )
        cls.policy32 = SimpleNamespace(
            bindings=(*cls.policy.bindings, base_binding), problems=()
        )
        cls.checked_ruleset = json.loads(
            (ROOT / ".github/rulesets/garnet-main.json").read_text(encoding="utf-8")
        )
        cls.checked_settings = json.loads(
            (ROOT / ".github/rulesets/repository-settings.json").read_text(
                encoding="utf-8"
            )
        )
        cls.checked_ruleset32 = copy.deepcopy(cls.checked_ruleset)
        required = next(
            row
            for row in cls.checked_ruleset32["rules"]
            if row.get("type") == "required_status_checks"
        )
        required["parameters"]["required_status_checks"].append(
            {
                "context": "Base-controlled trust policy",
                "integration_id": 15368,
            }
        )

    @staticmethod
    def ts(value: datetime) -> str:
        return value.strftime("%Y-%m-%dT%H:%M:%SZ")

    @staticmethod
    def obj(value: object, *, problem: str | None = None) -> object:
        problems = (
            (gate.transport.GitHubTransportProblem(problem),) if problem else ()
        )
        return gate.transport.ObjectResult(
            value=None if problem else value, problems=problems, byte_count=0 if problem else 100
        )

    @staticmethod
    def collection(
        rows: list[object], *, problem: str | None = None, pages: int = 1
    ) -> object:
        problems = (
            (gate.transport.GitHubTransportProblem(problem),) if problem else ()
        )
        return gate.transport.CollectionResult(
            rows=() if problem else tuple(rows),
            problems=problems,
            page_count=0 if problem else pages,
            byte_count=0 if problem else 100,
        )

    def payload(self) -> dict[str, object]:
        workflows: list[dict[str, object]] = []
        runs: list[dict[str, object]] = []
        checks: list[dict[str, object]] = []
        by_path: dict[str, tuple[int, int, str]] = {}
        for binding in self.policy.bindings:
            producer = binding.producer
            if producer.workflow not in by_path:
                number = len(by_path) + 1
                workflow_id, suite_id = 100 + number, 1000 + number
                by_path[producer.workflow] = (workflow_id, suite_id, producer.event)
                workflows.append(
                    {
                        "id": workflow_id,
                        "name": binding.workflow.name.value,
                        "path": producer.workflow,
                        "state": "active",
                    }
                )
                runs.append(
                    {
                        "id": 10_000 + number,
                        "workflow_id": workflow_id,
                        "check_suite_id": suite_id,
                        "run_attempt": 1,
                        "event": producer.event,
                        "head_sha": REVIEWED_HEAD,
                        "status": "completed",
                        "conclusion": "success",
                        "created_at": self.ts(NOW - timedelta(minutes=5)),
                        "updated_at": self.ts(NOW - timedelta(minutes=1)),
                    }
                )
            _, suite_id, _ = by_path[producer.workflow]
            checks.append(
                {
                    "id": 20_000 + len(checks),
                    "name": producer.context,
                    "check_suite_id": suite_id,
                    "head_sha": REVIEWED_HEAD,
                    "status": "completed",
                    "conclusion": "success",
                    "started_at": self.ts(NOW - timedelta(minutes=4)),
                    "completed_at": self.ts(NOW - timedelta(seconds=30)),
                    "app": {"id": 15368, "slug": "github-actions"},
                }
            )

        repository = {
            "id": 123456,
            "full_name": "Island-Dev-Crew/garnet",
            "default_branch": "main",
            **{
                key: value
                for key, value in self.checked_settings.items()
                if key
                not in {
                    "repository",
                    "default_branch",
                    "actions_default_workflow_permissions",
                    "actions_can_approve_pull_request_reviews",
                }
            },
        }
        actions = {
            "default_workflow_permissions": self.checked_settings[
                "actions_default_workflow_permissions"
            ],
            "can_approve_pull_request_reviews": self.checked_settings[
                "actions_can_approve_pull_request_reviews"
            ],
        }
        ruleset = {"id": 18_936_562, **copy.deepcopy(self.checked_ruleset)}
        return {
            "repository": repository,
            "workflows": workflows,
            "runs": runs,
            "checks": checks,
            "ruleset": ruleset,
            "actions": actions,
        }

    def evidence(self, payload: dict[str, object]) -> object:
        return gate.GovernanceTransportEvidence(
            repository=self.obj(payload["repository"]),
            workflows=self.collection(payload["workflows"]),
            workflow_runs=self.collection(payload["runs"]),
            check_runs=self.collection(payload["checks"]),
            ruleset=self.obj(payload["ruleset"]),
            actions_permissions=self.obj(payload["actions"]),
        )

    def payload32(self) -> dict[str, object]:
        payload = self.payload()
        workflow_id = 901
        suite_id = 1901
        payload["workflows"].append(
            {
                "id": workflow_id,
                "name": "Base-controlled trust",
                "path": ".github/workflows/base-controlled-trust.yml",
                "state": "active",
            }
        )
        payload["runs"].append(
            {
                "id": 19_001,
                "workflow_id": workflow_id,
                "check_suite_id": suite_id,
                "run_attempt": 1,
                "event": "pull_request_target",
                "head_sha": REVIEWED_HEAD,
                "status": "completed",
                "conclusion": "success",
                "created_at": self.ts(NOW - timedelta(minutes=5)),
                "updated_at": self.ts(NOW - timedelta(minutes=1)),
            }
        )
        payload["checks"].append(
            {
                "id": 29_001,
                "name": "Base-controlled trust policy",
                "check_suite_id": suite_id,
                "head_sha": REVIEWED_HEAD,
                "status": "completed",
                "conclusion": "success",
                "started_at": self.ts(NOW - timedelta(minutes=4)),
                "completed_at": self.ts(NOW - timedelta(seconds=30)),
                "app": {"id": 15368, "slug": "github-actions"},
            }
        )
        payload["ruleset"] = {"id": 18_936_562, **copy.deepcopy(self.checked_ruleset32)}
        return payload

    def evaluate(
        self,
        payload: dict[str, object] | None = None,
        *,
        evidence: object | None = None,
        reviewed_head: str = REVIEWED_HEAD,
        now: datetime = NOW,
        checked_ruleset: object | None = None,
        checked_settings: object | None = None,
        policy: object | None = None,
        root: Path = ROOT,
    ) -> object:
        payload = self.payload() if payload is None else payload
        arguments = (
            self.policy if policy is None else policy,
            self.evidence(payload) if evidence is None else evidence,
        )
        if checked_ruleset is None and checked_settings is None:
            return gate.evaluate_governance_gate(
                *arguments,
                reviewed_head=reviewed_head,
                now=now,
                root=root,
            )
        return gate._evaluate(
            *arguments,
            reviewed_head=reviewed_head,
            now=now,
            checked_ruleset=(
                self.checked_ruleset if checked_ruleset is None else checked_ruleset
            ),
            checked_repository_settings=(
                self.checked_settings if checked_settings is None else checked_settings
            ),
        )

    def assert_red(self, result: object, fragment: str) -> None:
        self.assertFalse(result.ok)
        self.assertEqual(result.bindings, ())
        self.assertTrue(
            any(fragment in problem for problem in result.problems), result.problems
        )

    def test_complete_exact_head_success_is_sanitized_and_selects_latest_attempt(self) -> None:
        payload = self.payload()
        first = payload["runs"][0]
        old_suite = 9001
        payload["runs"].append(
            {
                **first,
                "id": 90_001,
                "check_suite_id": old_suite,
                "run_attempt": 1,
                "status": "completed",
                "conclusion": "failure",
                "created_at": self.ts(NOW - timedelta(hours=1)),
                "updated_at": self.ts(NOW - timedelta(minutes=40)),
            }
        )
        first["id"] = 90_002
        first["check_suite_id"] = 9002
        first["run_attempt"] = 2
        for check in payload["checks"]:
            if check["check_suite_id"] == 1001:
                check["check_suite_id"] = 9002

        result = self.evaluate(payload)
        self.assertTrue(result.ok, result.problems)
        self.assertEqual(len(result.bindings), 31)
        self.assertEqual(result.evidence_authority, "injected-offline")
        self.assertEqual(result.live_settings_no_bypass, "blocked-u17")
        selected = [item for item in result.bindings if item.workflow_id == 101]
        self.assertTrue(selected)
        self.assertEqual({item.run_attempt for item in selected}, {2})
        rendered = json.dumps(asdict(result), sort_keys=True)
        self.assertNotIn("Authorization", rendered)
        self.assertNotIn("token", rendered.casefold())
        self.assertNotIn("raw", rendered.casefold())

    def test_transport_failures_empty_or_incomplete_results_are_all_or_zero(self) -> None:
        base = self.evidence(self.payload())
        cases = [
            (
                gate.GovernanceTransportEvidence(
                    **{**base.__dict__, "repository": self.obj({}, problem="http-status")}
                ),
                "repository transport",
            ),
            (
                gate.GovernanceTransportEvidence(
                    **{
                        **base.__dict__,
                        "workflow_runs": self.collection([], problem="pagination"),
                    }
                ),
                "workflow_runs transport",
            ),
            (
                gate.GovernanceTransportEvidence(
                    **{**base.__dict__, "check_runs": self.collection([])}
                ),
                "check_runs collection is empty",
            ),
            (
                gate.GovernanceTransportEvidence(
                    **{**base.__dict__, "workflows": self.collection([], pages=0)}
                ),
                "workflows collection is empty",
            ),
        ]
        for evidence, fragment in cases:
            with self.subTest(fragment=fragment):
                self.assert_red(self.evaluate(evidence=evidence), fragment)

    def test_collection_result_internals_types_and_bounds_are_exact(self) -> None:
        base = self.evidence(self.payload())
        valid = base.workflow_runs
        bad_results = [
            gate.transport.CollectionResult(
                rows=valid.rows, problems=[], page_count=1, byte_count=100
            ),
            gate.transport.CollectionResult(
                rows=valid.rows, problems=(object(),), page_count=1, byte_count=100
            ),
            gate.transport.CollectionResult(
                rows=list(valid.rows), problems=(), page_count=1, byte_count=100
            ),
            gate.transport.CollectionResult(
                rows=([],), problems=(), page_count=1, byte_count=100
            ),
            gate.transport.CollectionResult(
                rows=valid.rows, problems=(), page_count=True, byte_count=100
            ),
            gate.transport.CollectionResult(
                rows=valid.rows,
                problems=(),
                page_count=gate.transport.MAX_COLLECTION_PAGES + 1,
                byte_count=100,
            ),
            gate.transport.CollectionResult(
                rows=valid.rows, problems=(), page_count=1, byte_count=True
            ),
            gate.transport.CollectionResult(
                rows=valid.rows,
                problems=(),
                page_count=1,
                byte_count=gate.transport.MAX_COLLECTION_BYTES + 1,
            ),
            gate.transport.CollectionResult(
                rows=tuple({"id": index} for index in range(gate.transport.MAX_COLLECTION_ROWS + 1)),
                problems=(),
                page_count=1,
                byte_count=100,
            ),
        ]
        for index, result in enumerate(bad_results):
            evidence = gate.GovernanceTransportEvidence(
                **{**base.__dict__, "workflow_runs": result}
            )
            with self.subTest(case=index):
                self.assert_red(
                    self.evaluate(evidence=evidence), "workflow_runs collection"
                )

    def test_repository_default_branch_reviewed_head_and_run_head_are_exact(self) -> None:
        mutations = [
            (lambda p: p["repository"].__setitem__("full_name", "Navigata1/garnet"), "repository full_name"),
            (lambda p: p["repository"].__setitem__("default_branch", "trunk"), "default branch"),
            (lambda p: p["runs"][0].__setitem__("head_sha", "b" * 40), "exact reviewed head"),
        ]
        for mutate, fragment in mutations:
            payload = self.payload()
            mutate(payload)
            with self.subTest(fragment=fragment):
                self.assert_red(self.evaluate(payload), fragment)
        for value in ("a" * 39, "A" * 40, True):
            with self.subTest(reviewed_head=value):
                self.assert_red(
                    self.evaluate(reviewed_head=value), "reviewed head must be"
                )

    def test_unique_latest_attempt_is_selected_without_split_or_duplicate_domains(self) -> None:
        duplicate = self.payload()
        duplicate["runs"].append({**duplicate["runs"][0], "id": 88_888})
        self.assert_red(self.evaluate(duplicate), "workflow/attempt domain")

        split = self.payload()
        old = {**split["runs"][0], "id": 88_889, "check_suite_id": 8889}
        split["runs"].append(old)
        split["runs"][0]["run_attempt"] = 2
        target = next(
            row for row in split["checks"] if row["check_suite_id"] == 1001
        )
        target["check_suite_id"] = 8889
        self.assert_red(self.evaluate(split), "exactly one live check")

        cases = []
        for collection, field in (("runs", "id"), ("checks", "id")):
            payload = self.payload()
            payload[collection][1][field] = payload[collection][0][field]
            cases.append(payload)
        duplicate_workflow = self.payload()
        duplicate_workflow["workflows"][1]["path"] = duplicate_workflow["workflows"][0]["path"]
        cases.append(duplicate_workflow)
        for payload in cases:
            with self.subTest():
                self.assert_red(self.evaluate(payload), "duplicate")

    def test_freshness_uses_one_frozen_nonfuture_canonical_utc_clock(self) -> None:
        mutations = [
            (
                lambda p: p["runs"][0].__setitem__(
                    "created_at",
                    self.ts(NOW - timedelta(seconds=gate.FRESHNESS_WINDOW_SECONDS + 1)),
                ),
                "stale",
            ),
            (
                lambda p: p["runs"][0].__setitem__(
                    "updated_at", self.ts(NOW + timedelta(seconds=1))
                ),
                "future",
            ),
            (
                lambda p: p["checks"][0].__setitem__(
                    "completed_at", "2026-07-17T05:59:30.000Z"
                ),
                "canonical UTC",
            ),
            (
                lambda p: p["checks"][0].__setitem__(
                    "started_at", "2026-07-17T06:00:00Z"
                ),
                "timestamp order",
            ),
        ]
        for mutate, fragment in mutations:
            payload = self.payload()
            mutate(payload)
            with self.subTest(fragment=fragment):
                self.assert_red(self.evaluate(payload), fragment)
        self.assert_red(
            self.evaluate(now=NOW.replace(microsecond=1)), "injected clock"
        )

    def test_terminal_success_is_required_for_selected_runs_and_checks(self) -> None:
        cases = []
        for collection, field, value in (
            ("runs", "status", "in_progress"),
            ("runs", "conclusion", "failure"),
            ("checks", "status", "queued"),
            ("checks", "conclusion", "neutral"),
        ):
            payload = self.payload()
            payload[collection][0][field] = value
            cases.append((payload, "completed/success"))
        for payload, fragment in cases:
            with self.subTest():
                self.assert_red(self.evaluate(payload), fragment)

    def test_workflow_event_context_suite_and_github_actions_app_identity_are_exact(self) -> None:
        cases = []
        for collection, field, value, fragment in (
            ("workflows", "name", "Spoof", "name differs"),
            ("workflows", "state", "disabled_manually", "active"),
            ("runs", "event", "push", "producer event"),
            ("checks", "name", "Spoof", "exactly one live check"),
            ("checks", "head_sha", "b" * 40, "exact reviewed head"),
        ):
            payload = self.payload()
            payload[collection][0][field] = value
            cases.append((payload, fragment))
        app = self.payload()
        app["checks"][0]["app"] = {"id": 57789, "slug": "github-actions"}
        cases.append((app, "GitHub Actions App"))
        suite = self.payload()
        suite["checks"][0]["check_suite_id"] = 7777
        cases.append((suite, "exactly one live check"))
        for payload, fragment in cases:
            with self.subTest(fragment=fragment):
                self.assert_red(self.evaluate(payload), fragment)

    def test_required_context_names_are_globally_unique_across_all_check_runs(self) -> None:
        payload = self.payload()
        duplicate = copy.deepcopy(payload["checks"][0])
        duplicate["id"] = 99_999
        duplicate["check_suite_id"] = 99_998
        payload["checks"].append(duplicate)
        self.assert_red(self.evaluate(payload), "globally unique")

    def test_coordinated_fabricated_policy_cannot_replace_checked_in_identity(self) -> None:
        payload = self.payload()
        fabricated = copy.deepcopy(self.policy)
        original = fabricated.bindings[0].producer.context
        replacement = "Fabricated coordinated context"
        fabricated.bindings[0].producer.context = replacement
        next(row for row in payload["checks"] if row["name"] == original)["name"] = replacement
        self.assert_red(
            self.evaluate(
                payload,
                policy=fabricated,
            ),
            "canonical policy digest",
        )

    def test_canonical_policy_digest_does_not_bypass_binding_shape_validation(self) -> None:
        malformed = copy.deepcopy(self.policy)
        malformed.bindings[0].workflow = None
        self.assert_red(
            self.evaluate(policy=malformed), "workflow projection is malformed"
        )

    def test_policy_binding_digest_includes_declared_and_observed_semantics(self) -> None:
        fabricated = copy.deepcopy(self.policy)
        replacement = "f" * 64
        fabricated.bindings[0].producer.semantic_sha256 = replacement
        fabricated.bindings[0].observed_semantic_sha256 = replacement
        self.assert_red(
            self.evaluate(policy=fabricated), "canonical policy binding digest"
        )

        malformed = copy.deepcopy(self.policy)
        malformed.bindings[0].observed_semantic_sha256 = True
        self.assert_red(
            self.evaluate(policy=malformed), "policy binding semantics are malformed"
        )

    def test_policy_binding_digest_includes_immutable_workflow_yaml_name(self) -> None:
        payload = self.payload()
        fabricated = copy.deepcopy(self.policy)
        producer = fabricated.bindings[0].producer
        fabricated.bindings[0].workflow.name.value = "Fabricated workflow name"
        next(
            row for row in payload["workflows"] if row["path"] == producer.workflow
        )["name"] = "Fabricated workflow name"
        self.assert_red(
            self.evaluate(payload, policy=fabricated),
            "canonical policy binding digest",
        )

    def test_checked_document_digests_reject_coordinated_live_drift(self) -> None:
        cases = []

        payload = self.payload()
        checked = copy.deepcopy(self.checked_ruleset)
        for document in (payload["ruleset"], checked):
            document["conditions"]["ref_name"]["include"] = ["refs/heads/release"]
        cases.append((payload, checked, None, "checked-in ruleset canonical digest"))

        payload = self.payload()
        checked = copy.deepcopy(self.checked_ruleset)
        for document in (payload["ruleset"], checked):
            pull_request = next(
                row for row in document["rules"] if row.get("type") == "pull_request"
            )
            pull_request["parameters"]["allowed_merge_methods"].append("merge")
        cases.append((payload, checked, None, "checked-in ruleset canonical digest"))

        payload = self.payload()
        checked_settings = copy.deepcopy(self.checked_settings)
        payload["repository"]["allow_auto_merge"] = True
        checked_settings["allow_auto_merge"] = True
        cases.append(
            (
                payload,
                None,
                checked_settings,
                "checked-in repository settings canonical digest",
            )
        )

        for payload, checked, settings, fragment in cases:
            with self.subTest(fragment=fragment):
                self.assert_red(
                    self.evaluate(
                        payload,
                        checked_ruleset=checked,
                        checked_settings=settings,
                    ),
                    fragment,
                )

    def test_checked_document_digest_rejects_noncanonical_json_types(self) -> None:
        payload = self.payload()
        checked = copy.deepcopy(self.checked_ruleset)
        for document in (payload["ruleset"], checked):
            document["conditions"]["ref_name"]["include"] = ("~DEFAULT_BRANCH",)
        self.assert_red(
            self.evaluate(payload, checked_ruleset=checked),
            "checked-in ruleset canonical JSON types",
        )

    def test_checked_authority_loader_rejects_duplicate_json_keys_before_collapse(self) -> None:
        cases = (
            (
                "repository-settings.json",
                '  "allow_auto_merge": false,',
                '  "allow_auto_merge": true,\n',
            ),
            (
                "garnet-main.json",
                '  "name": "Garnet main - human-gated trust kernel",',
                '  "name": "fabricated",\n',
            ),
        )
        for filename, needle, duplicate in cases:
            with self.subTest(filename=filename), tempfile.TemporaryDirectory() as raw_temp:
                root = Path(raw_temp)
                rulesets = root / ".github/rulesets"
                rulesets.mkdir(parents=True)
                for name in ("garnet-main.json", "repository-settings.json"):
                    shutil.copyfile(ROOT / ".github/rulesets" / name, rulesets / name)
                path = rulesets / filename
                raw = path.read_text(encoding="utf-8")
                self.assertIn(needle, raw)
                path.write_text(
                    raw.replace(needle, duplicate + needle, 1), encoding="utf-8"
                )
                self.assert_red(self.evaluate(root=root), "duplicate JSON key")

    def test_complete_live_policy_projection_must_equal_checked_in_contracts(self) -> None:
        mutations = [
            (lambda p: p["ruleset"].__setitem__("id", 1), "ruleset id"),
            (lambda p: p["ruleset"].__setitem__("id", 18_936_562.0), "ruleset id"),
            (lambda p: p["ruleset"].__setitem__("name", "Other"), "ruleset policy mismatch"),
            (lambda p: p["ruleset"].__setitem__("target", "tag"), "ruleset policy mismatch"),
            (lambda p: p["ruleset"].__setitem__("enforcement", "evaluate"), "ruleset policy mismatch"),
            (lambda p: p["ruleset"].__setitem__("bypass_actors", [{"actor_id": 1}]), "bypass_actors"),
            (lambda p: p["ruleset"]["rules"].pop(), "ruleset policy mismatch"),
            (lambda p: p["ruleset"]["rules"].append({"type": "required_signatures"}), "ruleset policy mismatch"),
            (lambda p: p["repository"].__setitem__("allow_auto_merge", True), "repository settings mismatch"),
            (lambda p: p["repository"].__setitem__("allow_auto_merge", 0), "repository settings mismatch"),
            (lambda p: p["actions"].__setitem__("default_workflow_permissions", "write"), "Actions settings mismatch"),
            (lambda p: p["actions"].__setitem__("can_approve_pull_request_reviews", 0), "Actions settings mismatch"),
            (lambda p: p["actions"].__setitem__("extra", False), "Actions settings keys"),
        ]
        typed = self.payload()
        required = next(
            row
            for row in typed["ruleset"]["rules"]
            if row.get("type") == "required_status_checks"
        )
        required["parameters"]["required_status_checks"][0]["integration_id"] = 15368.0
        mutations.append((lambda p, source=typed: p.update(source), "ruleset policy mismatch"))
        for mutate, fragment in mutations:
            payload = self.payload()
            mutate(payload)
            with self.subTest(fragment=fragment):
                self.assert_red(self.evaluate(payload), fragment)

        base = self.evidence(self.payload())
        missing = gate.GovernanceTransportEvidence(
            **{**base.__dict__, "ruleset": self.obj(None)}
        )
        self.assert_red(self.evaluate(evidence=missing), "ruleset object")

    def test_credential_like_fields_and_invalid_checked_contracts_are_red(self) -> None:
        payload = self.payload()
        payload["repository"]["access_token"] = "never-store-this"
        result = self.evaluate(payload)
        self.assert_red(result, "credential-like")
        self.assertNotIn("never-store-this", json.dumps(asdict(result), sort_keys=True))

        checked = copy.deepcopy(self.checked_ruleset)
        checked["unexpected"] = True
        self.assert_red(
            self.evaluate(checked_ruleset=checked), "checked-in ruleset keys"
        )
        settings = copy.deepcopy(self.checked_settings)
        settings.pop("allow_auto_merge")
        self.assert_red(
            self.evaluate(checked_settings=settings),
            "checked-in repository settings keys",
        )

    def live_client(self, payload: dict[str, object]) -> tuple[object, list[object]]:
        calls: list[object] = []
        outer = self

        class Client:
            def get_repository(self) -> object:
                calls.append(("repository",))
                return outer.obj(payload["repository"])

            def get_collection(
                self,
                path: str,
                *,
                root_key: str | None = None,
                require_total_count: bool = False,
            ) -> object:
                calls.append(("collection", path, root_key, require_total_count))
                mapping = {
                    "actions/workflows": payload["workflows"],
                    f"actions/runs?head_sha={REVIEWED_HEAD}": payload["runs"],
                    f"commits/{REVIEWED_HEAD}/check-runs": payload["checks"],
                }
                return outer.collection(mapping[path])

            def get_object(self, path: str) -> object:
                calls.append(("object", path))
                mapping = {
                    "rulesets/18936562": payload["ruleset"],
                    "actions/permissions/workflow": payload["actions"],
                }
                return outer.obj(mapping[path])

        return Client(), calls

    def test_live_runtime_collector_uses_explicit_scope_and_keeps_u17_blocked(self) -> None:
        payload = self.payload()
        client, calls = self.live_client(payload)
        secret = "explicit-review-token"
        seen: list[tuple[str, str]] = []

        def factory(repository: str, token: str) -> object:
            seen.append((repository, token))
            return client

        result = gate.collect_live_governance_status(
            self.policy,
            reviewed_head=REVIEWED_HEAD,
            token=secret,
            now=NOW,
            include_admin=False,
            transport_factory=factory,
            root=ROOT,
        )
        self.assertTrue(result.ok, result.problems)
        self.assertTrue(result.transport_complete)
        self.assertTrue(result.exact_head)
        self.assertTrue(result.fresh)
        self.assertTrue(result.outcomes_verified)
        self.assertFalse(result.policy_equal)
        self.assertEqual(result.evidence_authority, "live-explicit-stdin")
        self.assertEqual(result.live_settings_no_bypass, "blocked-u17")
        self.assertEqual(seen, [("Island-Dev-Crew/garnet", secret)])
        self.assertNotIn(secret, json.dumps(asdict(result), sort_keys=True))
        self.assertNotIn(("object", "rulesets/18936562"), calls)
        self.assertNotIn(("object", "actions/permissions/workflow"), calls)

    def test_live_collector_projects_realistic_repository_before_credential_scan(self) -> None:
        payload = self.payload()
        payload["repository"].update(
            {
                "temp_clone_token": None,
                "security_and_analysis": {
                    "secret_scanning": {"status": "enabled"}
                },
                "owner": {"login": "Island-Dev-Crew"},
            }
        )
        client, _ = self.live_client(payload)
        result = gate.collect_live_governance_status(
            self.policy,
            reviewed_head=REVIEWED_HEAD,
            token="explicit-review-token",
            now=NOW,
            include_admin=False,
            transport_factory=lambda _repository, _token: client,
            root=ROOT,
        )
        self.assertTrue(result.ok, result.problems)
        rendered = json.dumps(asdict(result), sort_keys=True)
        self.assertNotIn("temp_clone_token", rendered)
        self.assertNotIn("secret_scanning", rendered)

    def test_live_collector_projects_realistic_nested_api_rows_before_scan(self) -> None:
        payload = self.payload()
        for key in ("workflows", "runs", "checks"):
            payload[key][0]["temp_clone_token"] = None
            payload[key][0]["security_and_analysis"] = {
                "secret_scanning": {"status": "enabled"}
            }
        payload["ruleset"]["temp_clone_token"] = None
        payload["actions"]["secret_scanning"] = "irrelevant-api-expansion"
        client, _ = self.live_client(payload)
        result = gate.collect_live_governance_status(
            self.policy,
            reviewed_head=REVIEWED_HEAD,
            token="explicit-admin-token",
            now=NOW,
            include_admin=True,
            transport_factory=lambda _repository, _token: client,
            root=ROOT,
        )
        self.assertTrue(result.ok, result.problems)
        rendered = json.dumps(asdict(result), sort_keys=True)
        self.assertNotIn("temp_clone_token", rendered)
        self.assertNotIn("secret_scanning", rendered)

    def test_live_admin_collector_can_close_no_bypass_without_changing_runtime_scope(self) -> None:
        payload = self.payload()
        client, calls = self.live_client(payload)
        result = gate.collect_live_governance_status(
            self.policy,
            reviewed_head=REVIEWED_HEAD,
            token="explicit-admin-token",
            now=NOW,
            include_admin=True,
            transport_factory=lambda _repository, _token: client,
            root=ROOT,
        )
        self.assertTrue(result.ok, result.problems)
        self.assertTrue(result.policy_equal)
        self.assertEqual(result.live_settings_no_bypass, "verified-empty")
        self.assertIn(("object", "rulesets/18936562"), calls)
        self.assertIn(("object", "actions/permissions/workflow"), calls)

    def test_activated_32_policy_and_live_collector_are_exactly_supported(self) -> None:
        payload = self.payload32()
        result = gate._evaluate(
            self.policy32,
            self.evidence(payload),
            reviewed_head=REVIEWED_HEAD,
            now=NOW,
            checked_ruleset=self.checked_ruleset32,
            checked_repository_settings=self.checked_settings,
        )
        self.assertTrue(result.ok, result.problems)
        self.assertEqual(len(result.bindings), 32)

        with tempfile.TemporaryDirectory() as raw_temp:
            root = Path(raw_temp).resolve()
            rulesets = root / ".github/rulesets"
            rulesets.mkdir(parents=True)
            (rulesets / "garnet-main.json").write_text(
                json.dumps(self.checked_ruleset32, indent=2) + "\n",
                encoding="utf-8",
            )
            shutil.copyfile(
                ROOT / ".github/rulesets/repository-settings.json",
                rulesets / "repository-settings.json",
            )
            client, _ = self.live_client(payload)
            live = gate.collect_live_governance_status(
                self.policy32,
                reviewed_head=REVIEWED_HEAD,
                token="explicit-admin-token",
                now=NOW,
                include_admin=True,
                transport_factory=lambda _repository, _token: client,
                root=root,
            )
        self.assertTrue(live.ok, live.problems)
        self.assertEqual(len(live.bindings), 32)
        self.assertEqual(live.live_settings_no_bypass, "verified-empty")

    def test_checked_policy_loader_accepts_exact_activated_32_root(self) -> None:
        inventory = json.loads(
            (ROOT / ".github/rulesets/required-context-producers.json").read_text(
                encoding="utf-8"
            )
        )
        inventory["optional_contexts"] = []
        with tempfile.TemporaryDirectory() as raw_temp:
            root = Path(raw_temp).resolve()
            rulesets = root / ".github/rulesets"
            rulesets.mkdir(parents=True)
            (rulesets / "required-context-producers.json").write_text(
                json.dumps(inventory), encoding="utf-8"
            )
            (rulesets / "garnet-main.json").write_text(
                json.dumps(self.checked_ruleset32), encoding="utf-8"
            )
            availability = gate.required_context_contract.ProducerEvaluation(
                bindings=tuple(self.policy32.bindings), problems=()
            )
            schema = SimpleNamespace(
                workflow_projection=lambda _root: SimpleNamespace(problems=())
            )
            with mock.patch.object(gate, "_load_sibling", return_value=schema), mock.patch.object(
                gate.required_context_contract,
                "evaluate_producer_availability",
                return_value=availability,
            ):
                policy = gate.load_checked_policy(root)
        self.assertEqual(policy.problems, ())
        self.assertEqual(len(policy.bindings), 32)
        self.assertEqual(
            policy.bindings[-1].producer.context,
            "Base-controlled trust policy",
        )

    def test_live_collector_rejects_invalid_head_or_token_before_transport(self) -> None:
        calls: list[tuple[str, str]] = []

        def factory(repository: str, token: str) -> object:
            calls.append((repository, token))
            raise AssertionError("transport must not be constructed")

        for reviewed_head, token in (
            ("main", "explicit-token"),
            (REVIEWED_HEAD, "bad\ntoken"),
            (REVIEWED_HEAD, ""),
        ):
            with self.subTest(reviewed_head=reviewed_head, token=bool(token)):
                result = gate.collect_live_governance_status(
                    self.policy,
                    reviewed_head=reviewed_head,
                    token=token,
                    now=NOW,
                    include_admin=False,
                    transport_factory=factory,
                    root=ROOT,
                )
                self.assertFalse(result.ok)
        self.assertEqual(calls, [])

    def test_cli_requires_one_explicit_stdin_token_and_rejects_ambient_credentials(self) -> None:
        payload = self.payload()
        secret = "explicit-review-token"
        client, _ = self.live_client(payload)
        output = io.StringIO()
        result = gate.main(
            [
                "--runtime-gate",
                "--reviewed-head",
                REVIEWED_HEAD,
                "--github-token-stdin",
            ],
            root=ROOT,
            stdin=io.StringIO(secret + "\n"),
            stdout=output,
            environ={},
            now=NOW,
            policy_loader=lambda _root: self.policy,
            transport_factory=lambda _repository, _token: client,
            local_head_loader=lambda _root, _environment: (REVIEWED_HEAD, ()),
        )
        self.assertEqual(result, 0)
        self.assertNotIn(secret, output.getvalue())
        self.assertTrue(json.loads(output.getvalue())["ok"])

        for token_text in ("", "one\ntwo\n", "bad token\n"):
            with self.subTest(token_text=token_text):
                output = io.StringIO()
                result = gate.main(
                    [
                        "--runtime-gate",
                        "--reviewed-head",
                        REVIEWED_HEAD,
                        "--github-token-stdin",
                    ],
                    root=ROOT,
                    stdin=io.StringIO(token_text),
                    stdout=output,
                    environ={},
                    now=NOW,
                    policy_loader=lambda _root: self.policy,
                    transport_factory=lambda _repository, _token: client,
                    local_head_loader=lambda _root, _environment: (REVIEWED_HEAD, ()),
                )
                self.assertEqual(result, 1)

        output = io.StringIO()
        result = gate.main(
            [
                "--runtime-gate",
                "--reviewed-head",
                REVIEWED_HEAD,
                "--github-token-stdin",
            ],
            root=ROOT,
            stdin=io.StringIO(secret + "\n"),
            stdout=output,
            environ={"GH_TOKEN": "ambient-secret"},
            now=NOW,
            policy_loader=lambda _root: self.policy,
            transport_factory=lambda _repository, _token: client,
            local_head_loader=lambda _root, _environment: (REVIEWED_HEAD, ()),
        )
        self.assertEqual(result, 1)
        self.assertNotIn("ambient-secret", output.getvalue())
        self.assertNotIn(secret, output.getvalue())

    def test_cli_rejects_ambient_credentials_without_mutating_caller_environment(self) -> None:
        environment = {"GH_TOKEN": "ambient-secret", "PATH": "/usr/bin"}
        original = dict(environment)
        output = io.StringIO()
        result = gate.main(
            [
                "--runtime-gate",
                "--reviewed-head",
                REVIEWED_HEAD,
                "--github-token-stdin",
            ],
            root=ROOT,
            stdin=io.StringIO("explicit-review-token\n"),
            stdout=output,
            environ=environment,
            now=NOW,
            local_head_loader=lambda _root, _environment: (REVIEWED_HEAD, ()),
        )
        self.assertEqual(result, 1)
        self.assertEqual(environment, original)
        self.assertNotIn("ambient-secret", output.getvalue())

    def test_cli_binds_reviewed_head_to_one_clean_local_commit_before_transport(self) -> None:
        payload = self.payload()
        client, _ = self.live_client(payload)
        transport_calls: list[tuple[str, str]] = []

        def factory(repository: str, token: str) -> object:
            transport_calls.append((repository, token))
            return client

        cases = (
            (("b" * 40, ()), "differs from clean local HEAD"),
            ((REVIEWED_HEAD, ("working tree is not clean",)), "working tree"),
        )
        for local_result, fragment in cases:
            with self.subTest(fragment=fragment):
                output = io.StringIO()
                result = gate.main(
                    [
                        "--runtime-gate",
                        "--reviewed-head",
                        REVIEWED_HEAD,
                        "--github-token-stdin",
                    ],
                    root=ROOT,
                    stdin=io.StringIO("explicit-review-token\n"),
                    stdout=output,
                    environ={},
                    now=NOW,
                    policy_loader=lambda _root: self.policy,
                    transport_factory=factory,
                    local_head_loader=lambda _root, _environment, value=local_result: value,
                )
                self.assertEqual(result, 1)
                rendered = output.getvalue()
                self.assertIn(fragment, rendered)
                self.assertNotIn("explicit-review-token", rendered)
        self.assertEqual(transport_calls, [])

    def test_clean_head_loader_ignores_ambient_git_redirection(self) -> None:
        with tempfile.TemporaryDirectory() as raw_temp:
            parent = Path(raw_temp)
            roots = (parent / "real", parent / "alternate")
            heads: list[str] = []
            for index, root in enumerate(roots):
                root.mkdir()
                subprocess.run(
                    ["git", "init", "-q", str(root)], check=True
                )
                subprocess.run(
                    ["git", "-C", str(root), "config", "user.name", "Test"],
                    check=True,
                )
                subprocess.run(
                    [
                        "git",
                        "-C",
                        str(root),
                        "config",
                        "user.email",
                        "test@example.invalid",
                    ],
                    check=True,
                )
                (root / "tracked.txt").write_text(
                    f"version {index}\n", encoding="utf-8"
                )
                subprocess.run(
                    ["git", "-C", str(root), "add", "tracked.txt"], check=True
                )
                subprocess.run(
                    ["git", "-C", str(root), "commit", "-qm", "initial"],
                    check=True,
                )
                heads.append(
                    subprocess.check_output(
                        ["git", "-C", str(root), "rev-parse", "HEAD"], text=True
                    ).strip()
                )
            (roots[0] / "tracked.txt").write_text("dirty\n", encoding="utf-8")
            environment = {
                "PATH": os.environ.get("PATH", ""),
                "GIT_DIR": str(roots[1] / ".git"),
                "GIT_WORK_TREE": str(roots[1]),
                "GIT_INDEX_FILE": str(roots[1] / ".git/index"),
                "GIT_CONFIG_COUNT": "1",
                "GIT_CONFIG_KEY_0": "core.worktree",
                "GIT_CONFIG_VALUE_0": str(roots[1]),
            }
            head, problems = gate.read_clean_local_head(roots[0], environment)
        self.assertEqual(head, heads[0])
        self.assertNotEqual(head, heads[1])
        self.assertIn("working tree is not clean", problems)


if __name__ == "__main__":
    unittest.main()
