#!/usr/bin/env python3
"""No-network contract tests for the authenticated GitHub object transport."""
from __future__ import annotations
import importlib.util
import json, sys, unittest
from pathlib import Path
from unittest import mock
SCRIPT = Path(__file__).with_name("garnet_github_governance_transport.py")
SPEC = importlib.util.spec_from_file_location("_github_governance_transport_test", SCRIPT)
assert SPEC and SPEC.loader
transport = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = transport; SPEC.loader.exec_module(transport)
REPO, TOKEN = "Island-Dev-Crew/garnet", "test-token-not-a-credential"
BASE = "https://api.github.com/repos/Island-Dev-Crew/garnet"
def response(value: object = None, *, status: int = 200,
             content_type: str = "application/json",
             headers: dict[str, str] | None = None) -> object:
    body = value if isinstance(value, bytes) else json.dumps(value).encode()
    return transport.PageResponse(status, {"Content-Type": content_type, **(headers or {})}, body)
class FakeOpener:
    def __init__(self, outcome: object) -> None:
        self.outcome = outcome; self.requests: list[tuple[object, float]] = []
    def __call__(self, request: object, *, timeout: float) -> object:
        self.requests.append((request, timeout))
        if isinstance(self.outcome, BaseException):
            raise self.outcome
        return self.outcome
class ObjectTransportTests(unittest.TestCase):
    def client(self, outcome: object) -> tuple[object, FakeOpener]:
        opener = FakeOpener(outcome); return transport.GitHubGovernanceTransport(REPO, TOKEN, opener=opener), opener
    def assert_closed(self, result: object, code: str) -> None:
        self.assertEqual((result.value, result.byte_count), (None, 0))
        self.assertEqual(tuple(item.code for item in result.problems), (code,))
        allowed = getattr(transport, "ALLOWED_PROBLEM_CODES", ()); self.assertTrue(
            allowed and all(item.code in allowed for item in result.problems))
        self.assertNotIn(TOKEN, repr(result))
        self.assertNotIn(TOKEN, repr(result.problems))
    def test_authenticated_object_request_is_exact_and_secret_safe(self) -> None:
        client, opener = self.client(response({"id": 7}))
        result = client.get_object("actions/runs/7?ref=feature%20one")
        self.assertEqual((result.value, result.problems), ({"id": 7}, ()))
        request, timeout = opener.requests[0]
        self.assertEqual(request.full_url, f"{BASE}/actions/runs/7?ref=feature%20one")
        headers = {key.lower(): value for key, value in request.header_items()}
        self.assertEqual(headers["authorization"], f"Bearer {TOKEN}")
        self.assertEqual(headers["accept"], "application/vnd.github+json")
        self.assertEqual(headers["x-github-api-version"], "2022-11-28")
        self.assertTrue(headers["user-agent"].strip())
        self.assertGreater(timeout, 0)
        for public_value in (request.full_url, repr(request), repr(result), repr(client)):
            self.assertNotIn(TOKEN, public_value)
    def test_token_in_caller_path_or_query_is_rejected_before_open(self) -> None:
        for path in (f"actions/runs/{TOKEN}", f"actions/runs/7?ref={TOKEN}"):
            with self.subTest(path=path):
                client, opener = self.client(response({"id": 7}))
                self.assert_closed(client.get_object(path), "invalid-path")
                self.assertEqual(opener.requests, [])
    def test_invalid_repository_or_token_is_rejected_before_open(self) -> None:
        for repo, token in (("owner", TOKEN), (REPO, "bad\ntoken")):
            with self.subTest(repo=repo):
                opener = FakeOpener(response({"id": 7}))
                client = transport.GitHubGovernanceTransport(repo, token, opener=opener)
                self.assert_closed(client.get_object("actions/runs/7"),
                                   "invalid-configuration")
                self.assertEqual(opener.requests, [])
    def test_problem_codes_are_closed_and_validated(self) -> None:
        for invalid in (TOKEN, None, []):
            with self.subTest(invalid=invalid):
                try:
                    transport.GitHubTransportProblem(invalid)
                except ValueError as caught:
                    self.assertEqual(str(caught), "unsupported transport problem code")
                except Exception as caught:
                    self.fail(f"unexpected problem-code exception {type(caught).__name__}")
                else:
                    self.fail("invalid problem code accepted")
        allowed = getattr(transport, "ALLOWED_PROBLEM_CODES", ())
        self.assertTrue(allowed)
        for code in allowed:
            self.assertEqual(transport.GitHubTransportProblem(code).code, code)
    def test_opener_exceptions_are_constant_and_sanitized(self) -> None:
        failures = (OSError(f"socket failed with {TOKEN}"),
                    transport._Failure("invalid-path"))
        for failure in failures:
            with self.subTest(kind=type(failure).__name__):
                client, _ = self.client(failure)
                self.assert_closed(client.get_object("actions/runs/7"), "transport-failure")
    def test_noncanonical_percent_and_query_forms_never_open(self) -> None:
        bad = ("../actions", "/actions", "https://example.invalid/actions",
               " actions/runs/7", "  actions/runs/7", "actions#fragment",
               "actions\\runs", "actions/\nruns",
               "actions/%", "actions/%GG", "actions/%2F/runs", "actions/%2f/runs",
               "actions/%5C/runs", "actions/%0A/runs", "actions/%25/runs",
               "actions/%2E%2E/runs", "actions/%252E%252E/runs", "actions/%41/runs",
               "actions/runs?ref=%", "actions/runs?ref=%2F", "actions/runs?flag",
               "actions/runs?ref=a&ref=b")
        for path in bad:
            with self.subTest(path=path):
                client, opener = self.client(response({"id": 7}))
                self.assert_closed(client.get_object(path), "invalid-path")
                self.assertEqual(opener.requests, [])
    def test_http_content_rate_shape_and_byte_failures_are_closed(self) -> None:
        cases = ((response({}, status=500), "http-status"),
                 (response({}, content_type="text/plain"), "content-type"),
                 (response({}, content_type="text/vnd.example+json"), "content-type"),
                 (response({}, headers={"X-RateLimit-Remaining": "0"}), "rate-limit"), (response({}, headers={"X-RateLimit-Remaining": "unknown"}), "rate-limit"),
                 (response({}, headers={"X-RateLimit-Remaining": "\u0660"}), "rate-limit"), (response({}, headers={"X-RateLimit-Remaining": "\u0661"}), "rate-limit"),
                 (object(), "response-shape"))
        for outcome, code in cases:
            with self.subTest(code=code):
                client, _ = self.client(outcome)
                self.assert_closed(client.get_object("actions/runs/7"), code)
        with mock.patch.object(transport, "MAX_BODY_BYTES", 1, create=True):
            client, _ = self.client(response(b"{}"))
            self.assert_closed(client.get_object("actions/runs/7"), "response-too-large")
    def test_hostile_response_fields_are_all_or_zero_and_secret_safe(self) -> None:
        class HostileStr(str):
            def lower(self) -> str: raise RuntimeError(TOKEN)
            def strip(self, *args: object) -> str: raise RuntimeError(TOKEN)
        class HostileBytes(bytes):
            def __len__(self) -> int: raise RuntimeError(TOKEN)
        class ThrowingDict(dict):
            def items(self) -> object: raise RuntimeError(TOKEN)
        class ThrowingPage(transport.PageResponse):
            @property
            def body(self) -> bytes: raise RuntimeError(TOKEN)
        poison = object.__new__(ThrowingPage)
        cases = (transport.PageResponse(True, {"Content-Type": "application/json"}, b"{}"),
                 transport.PageResponse(200, {HostileStr("Content-Type"): "application/json"}, b"{}"),
                 transport.PageResponse(200, {"Content-Type": HostileStr("application/json")}, b"{}"),
                 transport.PageResponse(200, ThrowingDict(), b"{}"),
                 transport.PageResponse(200, {"Content-Type": "application/json"}, HostileBytes(b"{}")), poison)
        for outcome in cases:
            with self.subTest(kind=type(outcome).__name__):
                client, _ = self.client(outcome)
                try: result = client.get_object("actions/runs/7")
                except Exception: result = None
                self.assertIsNotNone(result, "response extraction escaped sanitization")
                self.assert_closed(result, "response-shape")
    def test_json_content_type_requires_a_valid_nonempty_subtype(self) -> None:
        for media in ("application/json", "application/vnd.github+json"):
            client, _ = self.client(response({}, content_type=media)); self.assertEqual(
                client.get_object("actions/runs/7").value, {})
        for media in ("application/+json", " application/json", "application /json",
                      "application/", "application/vnd github+json", "application/@bad+json"):
            with self.subTest(media=media):
                client, _ = self.client(response({}, content_type=media)); self.assert_closed(
                    client.get_object("actions/runs/7"), "content-type")
    def test_json_syntax_utf8_duplicates_and_constants_are_closed(self) -> None:
        bodies = (b"\xff", b"{", b'{"id":1,"id":2}', b'{"id":NaN}', b'{"id":Infinity}')
        for body in bodies:
            with self.subTest(body=body):
                client, _ = self.client(response(body))
                self.assert_closed(client.get_object("actions/runs/7"), "json-invalid")
    def test_json_exponent_depth_node_and_integer_limits_are_closed(self) -> None:
        depth = getattr(transport, "MAX_JSON_DEPTH", 32) + 1
        digits = getattr(transport, "MAX_INTEGER_DIGITS", 256) + 1
        bodies = (b'{"value":1e400}',
                  b'{"value":' + b"[" * depth + b"0" + b"]" * depth + b"}",
                  b'{"value":' + b"9" * digits + b"}")
        for body in bodies:
            with self.subTest(size=len(body)):
                client, _ = self.client(response(body))
                self.assert_closed(client.get_object("actions/runs/7"), "json-limit")
        with mock.patch.object(transport, "MAX_JSON_NODES", 4, create=True):
            client, _ = self.client(response({"a": 1, "b": 2}))
            self.assert_closed(client.get_object("actions/runs/7"), "json-limit")
    def test_object_shape_and_echoed_credential_are_closed(self) -> None:
        for value, code in (([], "object-shape"), ({"echo": TOKEN}, "credential-in-response")):
            with self.subTest(code=code):
                client, _ = self.client(response(value))
                self.assert_closed(client.get_object("actions/runs/7"), code)
    def test_default_opener_refuses_redirects(self) -> None: self.assertIsNone(
        transport._NoRedirect().redirect_request())
if __name__ == "__main__":
    unittest.main()
