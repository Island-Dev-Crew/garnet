#!/usr/bin/env python3
"""Adversarial tests for exact required-context producer availability."""
from __future__ import annotations
import copy, importlib.util, sys, unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).parents[1]

def load(name: str) -> object:
    path = Path(__file__).with_name(name + ".py")
    spec = importlib.util.spec_from_file_location("_test_" + name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module

contract = load("garnet_required_context_contract")
schema = load("garnet_workflow_schema_policy")

WORKFLOW = """name: Checks
on:
  pull_request:
    branches: [main]
  workflow_dispatch: {}
jobs:
  static:
    name: Static check
    runs-on: ubuntu-latest
    steps:
      - run: echo static
  matrix:
    name: Matrix (${{ matrix.os }})
    needs: static
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    steps:
      - run: echo matrix
  release:
    if: startsWith(github.ref, 'refs/tags/v')
    runs-on: ubuntu-latest
    steps:
      - run: echo release
"""

def snapshot(*items: tuple[str, str]) -> object:
    documents = tuple(schema.yaml_policy.WorkflowDocument(
        f".github/workflows/{name}.yml", "100644", f"{index + 1:040x}",
        schema.yaml_policy._document(content.encode()),
    ) for index, (name, content) in enumerate(items))
    return schema.yaml_policy.WorkflowYamlSnapshot(documents, ())

def projection(workflow: str = WORKFLOW, *extra: tuple[str, str]) -> object:
    return schema.project_snapshot(snapshot(("checks", workflow), *extra))

def inventory(projected: object | None = None) -> object:
    projected = projected or projection()
    semantic = {
        item.context: contract.producer_semantic_sha256(workflow, item)
        for workflow in projected.workflows
        for item in workflow.contexts
    }
    rows = [
        contract.Producer("Static check", ".github/workflows/checks.yml", "pull_request", "static", None, semantic["Static check"]),
        contract.Producer("Matrix (ubuntu-latest)", ".github/workflows/checks.yml", "pull_request", "matrix", ("os", "ubuntu-latest"), semantic["Matrix (ubuntu-latest)"]),
        contract.Producer("Matrix (macos-latest)", ".github/workflows/checks.yml", "pull_request", "matrix", ("os", "macos-latest"), semantic["Matrix (macos-latest)"]),
        contract.Producer("Base-controlled trust policy", ".github/workflows/base-controlled-trust.yml", "pull_request_target", "policy", None, "0" * 64),
    ]
    return contract.ProducerInventory(rows, {"Base-controlled trust policy"}, "main", [])

class RequiredContextEvaluatorTests(unittest.TestCase):
    def evaluate(self, projected: object | None = None, declared: object | None = None) -> object:
        return contract.evaluate_producer_availability(
            declared or inventory(), projected or projection()
        )

    def assertProblem(self, result: object, fragment: str) -> None:  # noqa: N802
        self.assertEqual((result.bindings, result.prepared_optional, result.inactive_optional), ((), (), ()))
        self.assertTrue(any(fragment in item for item in result.problems), result.problems)

    def checked_policy(self, declared: object, ledger: object, projected: object) -> object:
        self.assertTrue(hasattr(contract, "evaluate_checked_in_producer_policy"))
        return contract.evaluate_checked_in_producer_policy(declared, ledger, projected)

    def current_policy_inputs(self) -> tuple[object, object, object]:
        self.assertTrue(hasattr(contract, "load_required_check_ledger"))
        declared = contract.load_inventory(ROOT / contract.INVENTORY_PATH)
        ledger = contract.load_required_check_ledger(
            ROOT / ".github/rulesets/garnet-main.json"
        )
        return declared, ledger, schema.workflow_projection(ROOT)

    def test_current_index_binds_exact_31_and_classifies_optional(self) -> None:
        declared, ledger, projected = self.current_policy_inputs()
        result = self.checked_policy(declared, ledger, projected)
        self.assertEqual(result.problems, ())
        self.assertEqual(len(result.bindings), 31)
        self.assertEqual(
            [item.producer.context for item in result.prepared_optional],
            ["Base-controlled trust policy"],
        )
        self.assertEqual(result.inactive_optional, ())
        compare = next(item for item in result.bindings
                       if item.producer.context == "Cross-OS determinism comparison")
        self.assertEqual(compare.dependency_contexts,
                         ("Generate single signing key for cross-OS build",
                          "Deterministic build on ubuntu-latest",
                          "Deterministic build on macos-latest"))

    def test_preactivation_rejects_coordinated_activation_and_shrinkage(self) -> None:
        declared, ledger, projected = self.current_policy_inputs()
        declared.optional_contexts.clear()
        activated = contract.RequiredCheckLedger(
            (*ledger.contexts, "Base-controlled trust policy"), ()
        )
        self.assertProblem(
            self.checked_policy(declared, activated, projected), "optional_contexts"
        )

        declared, ledger, projected = self.current_policy_inputs()
        removed = declared.producers.pop(0).context
        shrunk = contract.RequiredCheckLedger(
            tuple(item for item in ledger.contexts if item != removed), ()
        )
        self.assertProblem(self.checked_policy(declared, shrunk, projected), "31 active")

    def test_activation_transition_allows_only_monotonic_31_to_32(self) -> None:
        base_inventory, base_ledger, _ = self.current_policy_inputs()
        candidate_inventory = copy.deepcopy(base_inventory)
        candidate_ledger = contract.RequiredCheckLedger(base_ledger.contexts, ())
        self.assertEqual(
            contract.activation_transition_problems(
                base_inventory,
                base_ledger,
                candidate_inventory,
                candidate_ledger,
            ),
            (),
        )

        candidate_inventory.optional_contexts.clear()
        candidate_ledger = contract.RequiredCheckLedger(
            (*base_ledger.contexts, contract.BASE_CONTROLLED_CONTEXT), ()
        )
        self.assertEqual(
            contract.activation_transition_problems(
                base_inventory,
                base_ledger,
                candidate_inventory,
                candidate_ledger,
            ),
            (),
        )
        self.assertEqual(
            contract.activation_transition_problems(
                candidate_inventory,
                candidate_ledger,
                copy.deepcopy(candidate_inventory),
                candidate_ledger,
            ),
            (),
        )

        downgrade = copy.deepcopy(base_inventory)
        problems = contract.activation_transition_problems(
            candidate_inventory,
            candidate_ledger,
            downgrade,
            base_ledger,
        )
        self.assertTrue(any("32 to 31" in item for item in problems), problems)

        drift = copy.deepcopy(candidate_inventory)
        drift.producers[0] = replace(drift.producers[0], semantic_sha256="0" * 64)
        problems = contract.activation_transition_problems(
            base_inventory, base_ledger, drift, candidate_ledger
        )
        self.assertTrue(any("producer inventory" in item for item in problems), problems)

    def test_preactivation_pins_base_identity_and_ordered_ledger(self) -> None:
        declared, ledger, projected = self.current_policy_inputs()
        index = next(
            i for i, item in enumerate(declared.producers)
            if item.context == "Base-controlled trust policy"
        )
        declared.producers[index] = replace(declared.producers[index], job="other")
        self.assertProblem(
            self.checked_policy(declared, ledger, projected), "Base-controlled producer"
        )

        for contexts, fragment in (
            (("Base-controlled trust policy", *ledger.contexts), "must be absent"),
            (tuple(reversed(ledger.contexts)), "ordered contexts"),
            (("Rogue", *ledger.contexts[1:]), "ordered contexts"),
        ):
            with self.subTest(fragment=fragment):
                declared, _, projected = self.current_policy_inputs()
                candidate = contract.RequiredCheckLedger(tuple(contexts), ())
                self.assertProblem(
                    self.checked_policy(declared, candidate, projected), fragment
                )

    def test_preactivation_pins_historic_context_identity(self) -> None:
        declared, ledger, _ = self.current_policy_inputs()
        declared.producers[0] = replace(
            declared.producers[0], context="Replacement no-op"
        )
        candidate = contract.RequiredCheckLedger(
            ("Replacement no-op", *ledger.contexts[1:]), ()
        )
        problems = contract.preactivation_ruleset_problems(declared, candidate)
        self.assertTrue(
            any("baseline context identity" in item for item in problems), problems
        )

        declared, ledger, _ = self.current_policy_inputs()
        declared.producers[0] = replace(
            declared.producers[0], semantic_sha256="0" * 64
        )
        problems = contract.preactivation_ruleset_problems(declared, ledger)
        self.assertTrue(
            any("semantic fingerprints" in item for item in problems), problems
        )

    def test_evaluation_preserves_order_and_never_reopens_paths(self) -> None:
        frozen = projection()
        with patch("builtins.open", side_effect=AssertionError("path reopened")), \
             patch.object(Path, "read_text", side_effect=AssertionError("path reopened")):
            result = self.evaluate(frozen)
        self.assertEqual(result.problems, ())
        self.assertEqual([item.producer.context for item in result.bindings],
                         ["Static check", "Matrix (ubuntu-latest)", "Matrix (macos-latest)"])
        self.assertEqual(result.prepared_optional, ())
        self.assertEqual(result.bindings[-1].occurrence.binding[1].value, "macos-latest")

    def test_global_duplicate_context_fails_all_or_zero(self) -> None:
        rogue = """name: Rogue
on:
  workflow_dispatch: {}
jobs:
  copy:
    name: Static check
    if: always()
    runs-on: ubuntu-latest
    steps:
      - run: echo rogue
"""
        self.assertProblem(self.evaluate(projection(WORKFLOW, ("rogue", rogue))),
                           "duplicate projected context")

    def test_identity_and_selected_job_expansion_are_exact(self) -> None:
        for field, value in (("workflow", ".github/workflows/other.yml"),
                             ("job", "other"), ("event", "pull_request_target"),
                             ("matrix", ("os", "windows-latest"))):
            with self.subTest(field=field):
                declared = inventory()
                declared.producers[1] = replace(declared.producers[1], **{field: value})
                self.assertProblem(self.evaluate(declared=declared), "identity")
        expanded = WORKFLOW.replace("ubuntu-latest, macos-latest",
                                    "ubuntu-latest, macos-latest, windows-latest")
        self.assertProblem(self.evaluate(projection(expanded)), "job expansion")

    def test_only_unfiltered_or_exact_main_pr_events_are_available(self) -> None:
        exact = WORKFLOW.replace("  pull_request:\n    branches: [main]", "  pull_request: {}")
        projected = projection(exact)
        self.assertEqual(self.evaluate(projected, inventory(projected)).problems, ())
        unsafe = (
            "    paths: ['src/**']", "    paths-ignore: [docs/**]",
            "    branches-ignore: [develop]", "    types: [closed]",
            "    branches: [main, develop]", "    branches: ['m*']",
        )
        for trigger in unsafe:
            with self.subTest(trigger=trigger):
                candidate = WORKFLOW.replace("    branches: [main]", trigger)
                self.assertProblem(self.evaluate(projection(candidate)), "PR event")
        both = WORKFLOW.replace("  workflow_dispatch: {}",
                                "  pull_request_target: {}\n  workflow_dispatch: {}")
        self.assertProblem(self.evaluate(projection(both)), "exactly one PR-class event")

    def test_conditions_and_transitive_needs_must_remain_active(self) -> None:
        conditioned = WORKFLOW.replace("    name: Static check",
                                       "    name: Static check\n    if: always()")
        self.assertProblem(self.evaluate(projection(conditioned)), "job-level if")
        helper = "  helper:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo helper\n"
        undeclared = WORKFLOW.replace("  matrix:\n", helper + "  matrix:\n", 1).replace(
            "    needs: static", "    needs: helper")
        self.assertProblem(self.evaluate(projection(undeclared)), "dependency")

    def test_optional_producer_may_be_absent_or_safely_prepared(self) -> None:
        future = """name: Base policy
on:
  pull_request_target:
    branches: [main]
jobs:
  policy:
    name: Base-controlled trust policy
    runs-on: ubuntu-latest
    steps:
      - run: echo policy
"""
        projected = projection(WORKFLOW, ("base-controlled-trust", future))
        declared = inventory()
        occurrence = next(
            item
            for workflow in projected.workflows
            for item in workflow.contexts
            if item.context == "Base-controlled trust policy"
        )
        workflow = next(
            item for item in projected.workflows if occurrence in item.contexts
        )
        index = next(
            i for i, item in enumerate(declared.producers)
            if item.context == "Base-controlled trust policy"
        )
        declared.producers[index] = replace(
            declared.producers[index],
            semantic_sha256=contract.producer_semantic_sha256(workflow, occurrence),
        )
        result = self.evaluate(projected, declared)
        self.assertEqual(result.problems, ())
        self.assertEqual(len(result.bindings), 3)
        self.assertEqual(result.prepared_optional[0].producer.context,
                         "Base-controlled trust policy")
        self.assertEqual(result.inactive_optional, ())
        declared = inventory()
        declared.producers.append(contract.Producer(
            "Optional helper", ".github/workflows/checks.yml", "pull_request", "helper"))
        declared.optional_contexts.add("Optional helper")
        helper = "  helper:\n    name: Optional helper\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo helper\n"
        optional_need = WORKFLOW.replace("  matrix:\n", helper + "  matrix:\n", 1).replace(
            "    needs: static", "    needs: helper")
        self.assertProblem(self.evaluate(projection(optional_need), declared),
                           "optional dependency")

    def test_upstream_problems_short_circuit_all_evidence(self) -> None:
        declared = inventory()
        declared.problems.append("inventory red")
        self.assertProblem(self.evaluate(declared=declared), "inventory red")
        broken = schema.WorkflowProjection(problems=("projection red",))
        self.assertProblem(self.evaluate(broken), "projection red")

    def test_semantic_fingerprint_changes_with_meaningful_job_behavior(self) -> None:
        original = projection().workflows[0]
        changed_projection = projection(WORKFLOW.replace("echo static", "echo changed", 1))
        changed = changed_projection.workflows[0]
        self.assertNotEqual(
            contract.producer_semantic_sha256(original, original.contexts[0]),
            contract.producer_semantic_sha256(changed, changed.contexts[0]),
        )
        self.assertProblem(
            self.evaluate(changed_projection), "semantic fingerprint mismatch"
        )

if __name__ == "__main__":
    unittest.main()
