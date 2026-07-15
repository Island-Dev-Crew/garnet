#!/usr/bin/env python3
"""Adversarial tests for the offline required-context identity graph."""
from __future__ import annotations
import copy
import importlib.util
import json
import sys
import unittest
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
def load(name: str) -> object:
    path = ROOT / "scripts" / f"{name}.py"
    spec = importlib.util.spec_from_file_location(f"_{name}_test", path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module
contract = load("garnet_required_context_contract")
schema = load("garnet_workflow_schema_policy")
identity = load("garnet_workflow_identity_policy")
class LiveIdentityPolicyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        inventory = contract.load_inventory(
            ROOT / ".github/rulesets/required-context-producers.json"
        )
        ledger = contract.load_required_check_ledger(
            ROOT / ".github/rulesets/garnet-main.json"
        )
        projection = schema.workflow_projection(ROOT)
        cls.policy = contract.evaluate_checked_in_producer_policy(
            inventory, ledger, projection
        )
        assert not cls.policy.problems

    def payload(self) -> dict[str, object]:
        workflows: list[dict[str, object]] = []
        runs: list[dict[str, object]] = []
        checks: list[dict[str, object]] = []
        by_path: dict[str, tuple[int, int]] = {}
        for binding in self.policy.bindings:
            producer = binding.producer
            if producer.workflow not in by_path:
                number = len(by_path) + 1
                workflow_id, suite_id = 100 + number, 1000 + number
                by_path[producer.workflow] = (workflow_id, suite_id)
                workflows.append({
                    "id": workflow_id,
                    "name": binding.workflow.name.value,
                    "path": producer.workflow,
                    "state": "active",
                })
                runs.append({
                    "id": 10000 + number,
                    "workflow_id": workflow_id,
                    "check_suite_id": suite_id,
                    "event": producer.event,
                })
            _, suite_id = by_path[producer.workflow]
            checks.append({
                "id": 20000 + len(checks),
                "name": producer.context,
                "check_suite_id": suite_id,
                "app": {"id": 15368, "slug": "github-actions"},
            })
        workflows.append({
            "id": 999,
            "name": "pages-build-deployment",
            "path": "dynamic/pages/pages-build-deployment",
            "state": "active",
        })
        runs.append({
            "id": 19999, "workflow_id": 999, "check_suite_id": 1999,
            "event": "push",
        })
        checks.append({
            "id": 29999, "name": "CodeQL", "check_suite_id": 2999,
            "app": {"id": 57789, "slug": "github-advanced-security"},
        })
        return {
            "schema": "garnet.github-required-context-identity/v1",
            "workflows": workflows,
            "workflow_runs": runs,
            "check_runs": checks,
        }
    def evaluate(self, payload: dict[str, object]) -> object:
        snapshot = identity.parse_live_identity_snapshot(json.dumps(payload))
        return identity.evaluate_live_identity(self.policy, snapshot)
    def assertRed(self, payload: dict[str, object], fragment: str) -> None:  # noqa: N802
        result = self.evaluate(payload)
        self.assertEqual(result.bindings, ())
        self.assertTrue(any(fragment in item for item in result.problems), result.problems)
    def test_current_31_form_exact_ordered_identity_chains(self) -> None:
        result = self.evaluate(self.payload())
        self.assertEqual(result.problems, ())
        self.assertEqual(len(result.bindings), 31)
        self.assertEqual(
            tuple(item.producer.context for item in result.bindings),
            tuple(item.producer.context for item in self.policy.bindings),
        )
        self.assertNotIn(
            "Base-controlled trust policy",
            {item.producer.context for item in result.bindings},
        )
    def test_required_name_must_be_globally_unique_before_app_filtering(self) -> None:
        missing = self.payload()
        missing["check_runs"].pop(0)
        self.assertRed(missing, "exactly one live check")
        duplicate = self.payload()
        row = copy.deepcopy(duplicate["check_runs"][0])
        row["id"], row["app"]["id"] = 30000, 57789
        duplicate["check_runs"].append(row)
        self.assertRed(duplicate, "exactly one live check")
    def test_app_and_relational_identity_are_exact(self) -> None:
        cases = []
        for field, value in (("id", True), ("id", 57789), ("slug", "actions")):
            payload = self.payload()
            payload["check_runs"][0]["app"][field] = value
            cases.append((payload, "GitHub Actions App"))
        orphan = self.payload()
        orphan["check_runs"][0]["check_suite_id"] = 7777
        cases.append((orphan, "exactly one workflow run"))
        rewired = self.payload()
        rewired["check_runs"][0]["check_suite_id"] = rewired["workflow_runs"][1]["check_suite_id"]
        cases.append((rewired, "declared workflow"))
        crossed = self.payload()
        crossed["workflow_runs"][0]["workflow_id"] = crossed["workflows"][1]["id"]
        cases.append((crossed, "declared workflow"))
        event = self.payload()
        event["workflow_runs"][0]["event"] = "push"
        cases.append((event, "producer event"))
        spliced = self.payload()
        second = {**spliced["workflow_runs"][0], "id": 55555, "check_suite_id": 5555}
        spliced["workflow_runs"].append(second)
        spliced["check_runs"][0]["check_suite_id"] = 5555
        cases.append((spliced, "exactly one selected workflow run"))
        for payload, fragment in cases:
            with self.subTest(fragment=fragment):
                self.assertRed(payload, fragment)
    def test_expected_workflow_identity_is_unique_active_and_named(self) -> None:
        for mutate, fragment in (
            (lambda p: p["workflows"].pop(0), "exactly one live workflow"),
            (lambda p: p["workflows"].append({**p["workflows"][0], "id": 888}),
             "exactly one live workflow"),
            (lambda p: p["workflows"][0].__setitem__("name", "Spoof"), "name"),
            (lambda p: p["workflows"][0].__setitem__("state", "disabled_manually"),
             "active"),
        ):
            payload = self.payload()
            mutate(payload)
            with self.subTest(fragment=fragment):
                self.assertRed(payload, fragment)
    def test_parser_rejects_ambiguous_or_noncanonical_input_all_or_zero(self) -> None:
        texts = [
            '{"schema":"x","schema":"y","workflows":[],"workflow_runs":[],"check_runs":[]}',
            json.dumps({**self.payload(), "unknown": []}),
            "[" * 2000 + "0" + "]" * 2000,
        ]
        bad_id = self.payload()
        bad_id["workflow_runs"][0]["id"] = True
        texts.append(json.dumps(bad_id))
        bad_path = self.payload()
        bad_path["workflows"][-1]["path"] = "dynamic/../pages"
        texts.append(json.dumps(bad_path))
        bad_text = self.payload()
        bad_text["check_runs"][0]["name"] += "\u200b"
        texts.append(json.dumps(bad_text))
        for text in texts:
            with self.subTest(text=text[:40]):
                snapshot = identity.parse_live_identity_snapshot(text)
                self.assertTrue(snapshot.problems)
                self.assertEqual((snapshot.workflows, snapshot.workflow_runs,
                                  snapshot.check_runs), ((), (), ()))
                self.assertEqual(identity.evaluate_live_identity(
                    self.policy, snapshot
                ).bindings, ())
    def test_upstream_failure_and_optional_base_never_leak_evidence(self) -> None:
        snapshot = identity.parse_live_identity_snapshot(json.dumps(self.payload()))
        broken = contract.ProducerEvaluation(problems=("upstream red",))
        result = identity.evaluate_live_identity(broken, snapshot)
        self.assertEqual(result.bindings, ())
        self.assertIn("upstream red", result.problems)
        payload = self.payload()
        payload["check_runs"].append({
            "id": 40000, "name": "Base-controlled trust policy",
            "check_suite_id": 1999,
            "app": {"id": 15368, "slug": "github-actions"},
        })
        result = self.evaluate(payload)
        self.assertEqual(len(result.bindings), 31)
if __name__ == "__main__":
    unittest.main()
