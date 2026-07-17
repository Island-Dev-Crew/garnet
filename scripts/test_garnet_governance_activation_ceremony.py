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
REVIEWED_HEAD = "a" * 40


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
                "confirm the base-controlled workflow is active on main and open a separate activation/terminus pull request from that base with the bootstrap squash-durable landed marker registered",
                "activate required context 31 to 32 on ruleset 18936562 while the activation/terminus pull request is open",
                "read back ruleset 18936562 and verify Base-controlled trust policy is required with bypass_actors empty",
                "rerun the authenticated governance and base-controlled gates on the exact activation/terminus head",
                "merge the activation/terminus pull request",
                "merge the bounded post-squash Lane 1 closeout pull request that registers the terminus landed marker without adding a GOV number",
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

    def live_ruleset(self) -> dict[str, object]:
        document = json.loads(
            (ROOT / ".github/rulesets/garnet-main.json").read_text(encoding="utf-8")
        )
        required = next(
            rule
            for rule in document["rules"]
            if rule.get("type") == "required_status_checks"
        )
        required["parameters"]["required_status_checks"].append(
            {
                "context": "Base-controlled trust policy",
                "integration_id": 15368,
            }
        )
        return {"id": 18_936_562, **document}

    def live_result(self, value: object) -> object:
        return ceremony.transport.ObjectResult(value=value, byte_count=100)

    def test_authenticated_live_readback_can_close_activation_exactly(self) -> None:
        prepared = ceremony.read_ceremony_status(ROOT)
        result = ceremony.evaluate_live_activation(
            prepared,
            self.live_result(self.live_ruleset()),
            root=ROOT,
        )
        self.assertTrue(result.preparation_ok, result.problems)
        self.assertTrue(result.activation_ok, result.activation_problems)
        self.assertEqual(result.activation_problems, ())
        self.assertEqual(result.bypass_actors, ())

    def test_live_readback_bypass_missing_context_and_boolean_id_are_red(self) -> None:
        cases: list[tuple[dict[str, object], str]] = []
        bypass = self.live_ruleset()
        bypass["bypass_actors"] = [{"actor_id": 1}]
        cases.append((bypass, "bypass"))
        missing = self.live_ruleset()
        required = next(
            rule
            for rule in missing["rules"]
            if rule.get("type") == "required_status_checks"
        )
        required["parameters"]["required_status_checks"].pop()
        cases.append((missing, "32"))
        boolean = self.live_ruleset()
        boolean["id"] = True
        cases.append((boolean, "ruleset id"))
        for live, fragment in cases:
            with self.subTest(fragment=fragment):
                result = ceremony.evaluate_live_activation(
                    ceremony.read_ceremony_status(ROOT),
                    self.live_result(live),
                    root=ROOT,
                )
                self.assertFalse(result.activation_ok)
                self.assertTrue(
                    any(fragment in item for item in result.activation_problems),
                    result.activation_problems,
                )

    def test_activation_cli_reads_only_explicit_stdin_and_can_turn_green(self) -> None:
        secret = "explicit-admin-token"
        calls: list[tuple[str, str]] = []
        live = self.live_ruleset()

        class Client:
            def get_object(self, path: str) -> object:
                calls.append(("object", path))
                return ceremony.transport.ObjectResult(value=live, byte_count=100)

        seen: list[tuple[str, str]] = []

        def factory(repository: str, token: str) -> object:
            seen.append((repository, token))
            return Client()

        output = io.StringIO()
        result = ceremony.main(
            [
                "--activation-gate",
                "--reviewed-head",
                REVIEWED_HEAD,
                "--github-token-stdin",
            ],
            root=ROOT,
            stdin=io.StringIO(secret + "\n"),
            stdout=output,
            environ={},
            transport_factory=factory,
            local_head_loader=lambda _root, _environment: (REVIEWED_HEAD, ()),
        )
        self.assertEqual(result, 0)
        rendered = output.getvalue()
        self.assertNotIn(secret, rendered)
        self.assertTrue(json.loads(rendered)["activation_ok"])
        self.assertEqual(
            seen, [("Island-Dev-Crew/garnet", secret)]
        )
        self.assertEqual(calls, [("object", "rulesets/18936562")])

        output = io.StringIO()
        result = ceremony.main(
            [
                "--activation-gate",
                "--reviewed-head",
                REVIEWED_HEAD,
                "--github-token-stdin",
            ],
            root=ROOT,
            stdin=io.StringIO(secret + "\n"),
            stdout=output,
            environ={"GARNET_ADMIN_GITHUB_TOKEN": "ambient-secret"},
            transport_factory=factory,
            local_head_loader=lambda _root, _environment: (REVIEWED_HEAD, ()),
        )
        self.assertEqual(result, 1)
        self.assertNotIn("ambient-secret", output.getvalue())
        self.assertNotIn(secret, output.getvalue())

    def test_activation_cli_rejects_stale_or_dirty_head_before_transport(self) -> None:
        calls: list[tuple[str, str]] = []

        def factory(repository: str, token: str) -> object:
            calls.append((repository, token))
            raise AssertionError("transport must not open")

        for local_result, fragment in (
            (("b" * 40, ()), "differs from clean local HEAD"),
            ((REVIEWED_HEAD, ("working tree is not clean",)), "working tree"),
        ):
            with self.subTest(fragment=fragment):
                output = io.StringIO()
                result = ceremony.main(
                    [
                        "--activation-gate",
                        "--reviewed-head",
                        REVIEWED_HEAD,
                        "--github-token-stdin",
                    ],
                    root=ROOT,
                    stdin=io.StringIO("explicit-admin-token\n"),
                    stdout=output,
                    environ={},
                    transport_factory=factory,
                    local_head_loader=lambda _root, _environment, value=local_result: value,
                )
                self.assertEqual(result, 1)
                self.assertIn(fragment, output.getvalue())
        self.assertEqual(calls, [])

    def test_activation_cli_rejects_ambient_without_mutating_caller_environment(self) -> None:
        environment = {
            "GARNET_ADMIN_GITHUB_TOKEN": "ambient-secret",
            "PATH": "/usr/bin",
        }
        original = dict(environment)
        output = io.StringIO()
        result = ceremony.main(
            [
                "--activation-gate",
                "--reviewed-head",
                REVIEWED_HEAD,
                "--github-token-stdin",
            ],
            root=ROOT,
            stdin=io.StringIO("explicit-admin-token\n"),
            stdout=output,
            environ=environment,
            local_head_loader=lambda _root, _environment: (REVIEWED_HEAD, ()),
        )
        self.assertEqual(result, 1)
        self.assertEqual(environment, original)
        self.assertNotIn("ambient-secret", output.getvalue())


if __name__ == "__main__":
    unittest.main()
