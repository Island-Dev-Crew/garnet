#!/usr/bin/env python3
"""Strict preparation-only tests for the human 31-to-32 ceremony package."""
from __future__ import annotations

import copy
import contextlib
import importlib.util
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def load(name: str) -> object:
    path = ROOT / "scripts" / f"{name}.py"
    spec = importlib.util.spec_from_file_location(f"_{name}_item7_test", path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


ceremony = load("garnet_governance_activation_ceremony")


class CeremonyTests(unittest.TestCase):
    def fixture(self) -> tuple[tempfile.TemporaryDirectory[str], Path, dict[str, object]]:
        temporary = tempfile.TemporaryDirectory()
        root = Path(temporary.name)
        workflow_source = ROOT / ceremony.WORKFLOW_PATH
        workflow_target = root / ceremony.WORKFLOW_PATH
        workflow_target.parent.mkdir(parents=True, exist_ok=True)
        workflow_target.write_bytes(workflow_source.read_bytes())
        document = json.loads((ROOT / ceremony.CEREMONY_PATH).read_text(encoding="utf-8"))
        return temporary, root, document

    def write(self, root: Path, document: object) -> None:
        path = root / ceremony.CEREMONY_PATH
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(
            json.dumps(document, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

    def assert_red(self, document: object, fragment: str) -> None:
        temporary, root, _ = self.fixture()
        self.addCleanup(temporary.cleanup)
        self.write(root, document)
        result = ceremony.read_ceremony_status(root)
        self.assertFalse(result.preparation_ok)
        self.assertFalse(result.activation_ok)
        self.assertTrue(
            any(fragment in item for item in result.problems), result.problems
        )

    def test_canonical_package_is_prepared_but_activation_stays_blocked_u17(self) -> None:
        result = ceremony.read_ceremony_status(ROOT)
        self.assertTrue(result.preparation_ok, result.problems)
        self.assertFalse(result.activation_ok)
        self.assertEqual(result.state, "prepared-not-activated")
        self.assertEqual(result.activation_problems, ("blocked-u17",))
        self.assertEqual(result.bypass_actors, ())
        self.assertEqual(result.ruleset_id, 18_936_562)
        with contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(ceremony.main(["--gate"], root=ROOT), 0)
            self.assertEqual(ceremony.main(["--activation-gate"], root=ROOT), 1)

    def test_exact_jon_only_transition_token_and_acceptance_contracts(self) -> None:
        document = json.loads(
            (ROOT / ceremony.CEREMONY_PATH).read_text(encoding="utf-8")
        )
        self.assertEqual(document["jon_only_actions"], list(ceremony.JON_ONLY_ACTIONS))
        self.assertEqual(
            document["jon_only_actions"],
            [
                "provision GARNET_ADMIN_GITHUB_TOKEN",
                "merge the Lane 1 bootstrap pull request while Base-controlled trust policy is not required",
                "confirm the base-controlled workflow is active on main and open a separate activation/terminus pull request from that base",
                "activate required context 31 to 32 on ruleset 18936562 while the activation/terminus pull request is open",
                "read back ruleset 18936562 and verify Base-controlled trust policy is required with bypass_actors empty",
                "rerun the authenticated governance and base-controlled gates on the exact activation/terminus head",
                "merge the activation/terminus pull request",
            ],
        )
        self.assertEqual(
            document["acceptance_commands"],
            [dict(item) for item in ceremony.ACCEPTANCE_COMMANDS],
        )
        self.assertEqual(
            document["token_policy"],
            {
                "admin_readback_token": "GARNET_ADMIN_GITHUB_TOKEN",
                "admin_token_source": "explicit-only",
                "ambient_credentials": "forbidden",
                "persist": "forbidden",
                "print": "forbidden",
                "review_enumeration_scope": "pull-requests:read",
                "review_enumeration_token": "github.token (event-scoped)",
            },
        )
        self.assertNotEqual(
            document["token_policy"]["admin_readback_token"],
            document["token_policy"]["review_enumeration_token"],
        )
        self.assertEqual(
            document["transition"],
            {
                "appended_context": "Base-controlled trust policy",
                "from_required_context_count": 31,
                "integration_id": 15368,
                "to_required_context_count": 32,
            },
        )

    def test_coordinated_activation_or_policy_drift_is_red(self) -> None:
        temporary, _, canonical = self.fixture()
        temporary.cleanup()
        cases = []
        activated = copy.deepcopy(canonical)
        activated["state"] = "activated"
        activated["activation"]["performed"] = True
        cases.append((activated, "prepared-not-activated"))
        bypass = copy.deepcopy(canonical)
        bypass["bypass_actors"] = [{"actor_id": 1}]
        cases.append((bypass, "bypass"))
        ruleset = copy.deepcopy(canonical)
        ruleset["ruleset_id"] = 1
        cases.append((ruleset, "ruleset"))
        transition = copy.deepcopy(canonical)
        transition["transition"]["from_required_context_count"] = 32
        cases.append((transition, "transition"))
        admin = copy.deepcopy(canonical)
        admin["token_policy"]["admin_readback_token"] = "GARNET_REVIEW_GITHUB_TOKEN"
        cases.append((admin, "token policy"))
        actions = copy.deepcopy(canonical)
        actions["jon_only_actions"].pop()
        cases.append((actions, "Jon-only"))
        commands = copy.deepcopy(canonical)
        commands["acceptance_commands"][0]["command"] = "true"
        cases.append((commands, "acceptance commands"))
        for document, fragment in cases:
            with self.subTest(fragment=fragment):
                self.assert_red(document, fragment)

    def test_workflow_digest_and_evidence_destination_are_exact(self) -> None:
        temporary, _, canonical = self.fixture()
        temporary.cleanup()
        digest = copy.deepcopy(canonical)
        digest["workflow"]["sha256"] = "0" * 64
        self.assert_red(digest, "workflow SHA-256")
        destination = copy.deepcopy(canonical)
        destination["evidence_destination"] = "elsewhere.md"
        self.assert_red(destination, "evidence destination")

    def test_unknown_duplicate_and_noncanonical_json_are_red(self) -> None:
        temporary, _, canonical = self.fixture()
        temporary.cleanup()
        extra = copy.deepcopy(canonical)
        extra["unexpected"] = True
        self.assert_red(extra, "keys")

        temporary, root, _ = self.fixture()
        self.addCleanup(temporary.cleanup)
        path = root / ceremony.CEREMONY_PATH
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text('{"schema":"a","schema":"b"}\n', encoding="utf-8")
        result = ceremony.read_ceremony_status(root)
        self.assertFalse(result.preparation_ok)
        self.assertTrue(any("duplicate" in item for item in result.problems))


if __name__ == "__main__":
    unittest.main()
