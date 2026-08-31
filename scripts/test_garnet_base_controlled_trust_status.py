#!/usr/bin/env python3
"""Adversarial tests for the old-base, candidate-inert Item 7 policy gate."""
from __future__ import annotations

import copy
import importlib.util
import io
import os
import re
import sys
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock


ROOT = Path(__file__).resolve().parents[1]


def load(name: str) -> object:
    path = ROOT / "scripts" / f"{name}.py"
    spec = importlib.util.spec_from_file_location(f"_{name}_item7_test", path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


status = load("garnet_base_controlled_trust_status")
contract = status.contract


class BaseControlledTrustTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.base_inventory = contract.load_inventory(ROOT / contract.INVENTORY_PATH)
        cls.base_ledger = contract.load_required_check_ledger(
            ROOT / contract.RULESET_PATH
        )
        assert not cls.base_inventory.problems
        assert not cls.base_ledger.problems
        assert len(cls.base_inventory.producers) == 32
        assert len(cls.base_ledger.contexts) == 31

    @staticmethod
    def policy(candidate_inventory: object, candidate_ledger: object) -> object:
        active = tuple(
            SimpleNamespace(producer=producer)
            for producer in candidate_inventory.producers
            if producer.context in candidate_ledger.contexts
        )
        prepared = tuple(
            SimpleNamespace(producer=producer)
            for producer in candidate_inventory.producers
            if producer.context == contract.BASE_CONTROLLED_CONTEXT
            and producer.context not in candidate_ledger.contexts
        )
        return SimpleNamespace(
            bindings=active,
            prepared_optional=prepared,
            inactive_optional=(),
            problems=(),
        )

    @staticmethod
    def rolling(*, ok: bool = True) -> object:
        return SimpleNamespace(
            schema="garnet.trust_kernel_review/v2",
            ok=ok,
            base_commit="a" * 40,
            head_commit="b" * 40,
            reviewed_head="b" * 40,
            reviewed_tree="c" * 40,
            content_digest="sha256:" + "d" * 64,
            problems=[] if ok else ["review enumeration failed"],
        )

    def evaluate(
        self,
        *,
        base_inventory: object | None = None,
        base_ledger: object | None = None,
        candidate_inventory: object | None = None,
        candidate_ledger: object | None = None,
        candidate_policy: object | None = None,
        rolling_review: object | None = None,
        base_workflow_sha256: str = "e" * 64,
        candidate_workflow_sha256: str = "e" * 64,
    ) -> object:
        base_inventory = copy.deepcopy(base_inventory or self.base_inventory)
        base_ledger = copy.deepcopy(base_ledger or self.base_ledger)
        candidate_inventory = copy.deepcopy(candidate_inventory or base_inventory)
        candidate_ledger = copy.deepcopy(candidate_ledger or base_ledger)
        return status.evaluate_base_controlled_trust(
            base_inventory,
            base_ledger,
            candidate_inventory,
            candidate_ledger,
            candidate_policy=(
                candidate_policy
                if candidate_policy is not None
                else self.policy(candidate_inventory, candidate_ledger)
            ),
            rolling_review=rolling_review or self.rolling(),
            repository="Island-Dev-Crew/garnet",
            pull_request=507,
            base_commit="a" * 40,
            candidate_commit="b" * 40,
            base_workflow_sha256=base_workflow_sha256,
            candidate_workflow_sha256=candidate_workflow_sha256,
        )

    def assert_red(self, result: object, fragment: str) -> None:
        self.assertFalse(result.ok)
        self.assertEqual(result.credited_contexts, ())
        self.assertTrue(
            any(fragment in problem for problem in result.problems), result.problems
        )

    def test_preparation_allows_only_31_to_31_31_to_32_and_32_to_32(self) -> None:
        self.assertTrue(self.evaluate().ok)

        active = copy.deepcopy(self.base_inventory)
        active.optional_contexts.clear()
        active_ledger = contract.RequiredCheckLedger(
            (*self.base_ledger.contexts, contract.BASE_CONTROLLED_CONTEXT), ()
        )
        activated = self.evaluate(
            candidate_inventory=active,
            candidate_ledger=active_ledger,
        )
        self.assertTrue(activated.ok, activated.problems)
        self.assertEqual(activated.transition, "31-to-32")

        stable = self.evaluate(
            base_inventory=active,
            base_ledger=active_ledger,
            candidate_inventory=active,
            candidate_ledger=active_ledger,
        )
        self.assertTrue(stable.ok, stable.problems)
        self.assertEqual(stable.transition, "32-to-32")

        downgrade = self.evaluate(
            base_inventory=active,
            base_ledger=active_ledger,
            candidate_inventory=self.base_inventory,
            candidate_ledger=self.base_ledger,
        )
        self.assert_red(downgrade, "32 to 31")

    def test_candidate_policy_workflow_identity_and_rolling_review_are_all_or_zero(self) -> None:
        bad_policy = self.policy(self.base_inventory, self.base_ledger)
        bad_policy.problems = ("candidate semantic policy mismatch",)
        self.assert_red(
            self.evaluate(candidate_policy=bad_policy), "candidate semantic policy"
        )
        self.assert_red(
            self.evaluate(rolling_review=self.rolling(ok=False)),
            "rolling review",
        )
        self.assert_red(
            self.evaluate(candidate_workflow_sha256="f" * 64),
            "protected workflow bytes",
        )

    def test_git_candidate_accepts_valid_workflows_across_adapter_loads(self) -> None:
        code, raw_head = status._git(ROOT, "rev-parse", "--verify", "HEAD^{commit}")
        self.assertEqual(code, 0)
        head = raw_head.decode("ascii").strip()
        rolling = SimpleNamespace(
            schema=status.REVIEW_SCHEMA,
            ok=True,
            base_commit=head,
            head_commit=head,
            reviewed_head=head,
            reviewed_tree="c" * 40,
            content_digest="sha256:" + "d" * 64,
            problems=[],
        )
        temporary_directory = status.tempfile.TemporaryDirectory

        def rooted_temporary_directory(*args: object, **kwargs: object) -> object:
            kwargs["dir"] = ROOT
            return temporary_directory(*args, **kwargs)

        with mock.patch.object(
            status.tempfile,
            "TemporaryDirectory",
            side_effect=rooted_temporary_directory,
        ), mock.patch.object(status, "_rolling_review_adapter", return_value=rolling):
            result = status.evaluate_git_candidate(
                root=ROOT,
                candidate_repo=ROOT,
                repository=status.REPOSITORY,
                pull_request=507,
                base_commit=head,
                candidate_commit=head,
                token="test-only",
            )

        self.assertTrue(result.ok, result.problems)
        self.assertTrue(result.candidate_policy_ok, result.problems)

    def test_duplicated_yaml_identities_both_reject_invalid_roots(self) -> None:
        first = load("garnet_workflow_yaml_policy")
        second = load("garnet_workflow_yaml_policy")
        self.assertIsNot(first.WorkflowMapping, second.WorkflowMapping)
        for policy in (first, second):
            with self.subTest(module=policy.__name__), self.assertRaisesRegex(
                policy.YamlPolicyError, "workflow root must be a mapping"
            ):
                policy._document(b"- invalid\n")

    def test_review_token_is_explicit_stdin_only_and_never_rendered(self) -> None:
        secret = b"review-token-value"
        with mock.patch.dict(
            os.environ,
            {
                "GH_TOKEN": "ambient-gh",
                "GITHUB_TOKEN": "ambient-github",
                "GARNET_ADMIN_GITHUB_TOKEN": "admin-token",
            },
            clear=False,
        ):
            token, problems = status.read_explicit_review_token(
                io.BytesIO(secret + b"\n"), enabled=True
            )
        self.assertEqual(token, secret.decode())
        self.assertEqual(problems, [])
        rendered = status.render_json(self.evaluate())
        for value in (secret.decode(), "ambient-gh", "ambient-github", "admin-token"):
            self.assertNotIn(value, rendered)
        token, problems = status.read_explicit_review_token(
            io.BytesIO(b""), enabled=False
        )
        self.assertEqual(token, "")
        self.assertTrue(any("explicit" in item for item in problems))

    def test_workflow_is_exact_old_base_and_candidate_inert(self) -> None:
        path = ROOT / ".github/workflows/base-controlled-trust.yml"
        text = path.read_text(encoding="utf-8")
        self.assertRegex(text, r"(?m)^name: Base-controlled trust$")
        self.assertRegex(text, r"(?m)^  pull_request_target:$")
        self.assertNotRegex(text, r"(?m)^  pull_request:$")
        self.assertRegex(text, r"(?m)^      - main$")
        self.assertRegex(text, r"(?m)^  policy:$")
        self.assertRegex(text, r"(?m)^    name: Base-controlled trust policy$")
        self.assertIn("persist-credentials: false", text)
        self.assertIn("git init --bare", text)
        self.assertIn("REVIEW_TOKEN: ${{ github.token }}", text)
        self.assertNotIn("secrets.GARNET_REVIEW_GITHUB_TOKEN", text)
        self.assertIn("unset REVIEW_TOKEN GARNET_REVIEW_GITHUB_TOKEN", text)
        self.assertNotIn("GARNET_ADMIN_GITHUB_TOKEN", text)
        self.assertNotRegex(text, r"git\s+(?:checkout|switch|worktree)\b")
        refs = re.findall(r"uses:\s*[^\s@]+@([^\s#]+)", text)
        self.assertTrue(refs)
        self.assertTrue(all(re.fullmatch(r"[0-9a-f]{40}", ref) for ref in refs))


if __name__ == "__main__":
    unittest.main()
