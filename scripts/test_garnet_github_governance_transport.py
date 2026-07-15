#!/usr/bin/env python3
"""No-network contracts for authenticated GitHub object and collection transport."""
from __future__ import annotations
import importlib.util
import io, json, sys, unittest, urllib.error
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
             content_type: str = "application/json", headers: object = None) -> object:
    body = value if isinstance(value, bytes) else json.dumps(value).encode()
    extras = tuple((headers or {}).items()) if isinstance(headers, dict) else (headers or ())
    return transport.PageResponse(status, (("Content-Type", content_type), *extras), body)
class FakeOpener:
    def __init__(self, outcome: object) -> None:
        self.outcome = outcome; self.requests: list[tuple[object, float]] = []
    def __call__(self, request: object, *, timeout: float) -> object:
        self.requests.append((request, timeout))
        outcome = self.outcome[len(self.requests) - 1] if type(self.outcome) is tuple else self.outcome
        if isinstance(outcome, BaseException): raise outcome
        return outcome
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
               "actions/%3Aruns", "actions/[runs]",
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
    def test_physical_header_bounds_and_credential_scan_are_closed(self) -> None:
        client, _ = self.client(response({}, headers=tuple(
            (f"X-{index}", "v") for index in range(255))))
        self.assertEqual(client.get_object("actions/runs/7").value, {})
        for headers in (tuple((f"X-{index}", "v") for index in range(256)),
                        (("X-Oversize", "v" * (64 * 1024)),)):
            client, _ = self.client(response({}, headers=headers))
            self.assert_closed(client.get_object("actions/runs/7"), "response-shape")
        for headers in (((TOKEN, "v"),), (("X-Test", TOKEN),)):
            client, _ = self.client(response({}, headers=headers))
            self.assert_closed(client.get_object("actions/runs/7"),
                               "credential-in-response")
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

class CollectionTransportTests(unittest.TestCase):
    def client(self, *outcomes: object) -> tuple[object, FakeOpener]:
        opener = FakeOpener(tuple(outcomes)); return transport.GitHubGovernanceTransport(
            REPO, TOKEN, opener=opener), opener
    def link(self, page: int, *, route: str = "repos/Island-Dev-Crew/garnet",
             query: str | None = None) -> str:
        query = query or f"branch=main&page={page}&per_page=100"
        return f"https://api.github.com/{route}/actions/runs?{query}"
    def assert_closed(self, result: object) -> None:
        self.assertEqual((result.rows, result.page_count, result.byte_count), ((), 0, 0))
        self.assertTrue(result.problems)
        self.assertTrue(all(problem.code in transport.ALLOWED_PROBLEM_CODES
                            for problem in result.problems))
        self.assertNotIn(TOKEN, repr(result))
    def test_canonical_page_one_collects_complete_named_chain_and_keeps_duplicate_ids(self) -> None:
        page_two = self.link(2, query="per_page=100&page=2&branch=main")
        first = self.link(1)
        client, opener = self.client(
            response({"total_count": 3, "workflow_runs": [{"id": 1}, {"id": 1}]}, headers=(
                ("Link", f'<{page_two}>; rel="next"'),)),
            response({"total_count": 3, "workflow_runs": [{"id": 2}]}, headers=(
                ("Link", f'<{first}>; rel="first prev"'),)))
        result = client.get_collection("actions/runs?branch=main", root_key="workflow_runs",
                                       require_total_count=True)
        self.assertEqual((result.rows, result.page_count),
                         (({"id": 1}, {"id": 1}, {"id": 2}), 2))
        self.assertEqual(tuple(item[0].full_url for item in opener.requests), (
            f"{BASE}/actions/runs?branch=main&page=1&per_page=100",
            f"{BASE}/actions/runs?branch=main&page=2&per_page=100"))
        self.assertEqual(result.byte_count, sum(len(item.body) for item in opener.outcome))
    def test_caller_pagination_cursor_and_noncanonical_query_fail_before_open(self) -> None:
        bad = ("actions/runs?page=1", "actions/runs?PER_PAGE=100",
               "actions/runs?cursor=x", "actions/runs?since=x", "actions/runs?before=x",
               "actions/runs?after=x", "actions/runs?branch=ma%69n", "actions/runs?=x",
               "actions/runs?branch=main&branch=main")
        for path in bad:
            with self.subTest(path=path):
                client, opener = self.client(response([])); self.assert_closed(
                    client.get_collection(path)); self.assertEqual(opener.requests, [])
    def test_every_relation_target_is_exact_and_relation_state_is_consistent(self) -> None:
        bad_targets = (
            self.link(3), self.link(2, query="branch=other&page=2&per_page=100"),
            self.link(2, query="branch=main&page=2&per_page=99"),
            self.link(2).replace("api.github.com", "example.invalid"),
            self.link(2).replace("/garnet/", "/other/"),
            self.link(2) + "#fragment", self.link(2, query="branch=main&page=02&per_page=100"),
            self.link(2, query="branch=%6Dain&page=2&per_page=100"),
            self.link(2, query="branch=main&branch=main&page=2&per_page=100"),
            self.link(2, query=f"branch=main&page={'9' * 100}&per_page=100"),
            self.link(2, route=f"repositories/{'9' * 100}"))
        for target in bad_targets:
            with self.subTest(target=target):
                client, _ = self.client(response([{"id": 1}], headers=(
                    ("Link", f'<{target}>; rel="next"'),)))
                self.assert_closed(client.get_collection("actions/runs?branch=main"))
        client, _ = self.client(response([{"id": 1}], headers=(
            ("Link", f'<{self.link(2)}>; rel="last"'),)))
        self.assert_closed(client.get_collection("actions/runs?branch=main"))
        for parameters in ('rel="up"', 'rel="next"; anchor="https://api.github.com/context"'):
            client, _ = self.client(response([1], headers=(("Link", f'<{self.link(2)}>; {parameters}'),)))
            self.assert_closed(client.get_collection("actions/runs?branch=main"))
    def test_numeric_link_route_requires_one_exact_named_preflight_but_requests_stay_named(self) -> None:
        numeric_one = self.link(1, route="repositories/7")
        numeric_two = self.link(2, route="repositories/7")
        client, opener = self.client(
            response([{"id": 1}], headers=(("Link", f'<{numeric_two}>; rel="next last"'),)),
            response({"id": 7, "full_name": REPO}), response([{"id": 2}], headers=(
                ("Link", f'<{numeric_one}>; rel="first prev"'),)))
        result = client.get_collection("actions/runs?branch=main")
        self.assertEqual((result.rows, result.page_count), (({"id": 1}, {"id": 2}), 2))
        self.assertEqual(tuple(item[0].full_url for item in opener.requests), (
            f"{BASE}/actions/runs?branch=main&page=1&per_page=100", BASE,
            f"{BASE}/actions/runs?branch=main&page=2&per_page=100"))
        self.assertEqual(result.byte_count, sum(len(item.body) for item in opener.outcome))
    def test_numeric_preflight_mismatch_and_late_failures_return_zero_rows(self) -> None:
        next_link = self.link(2)
        cases = (
            ((response([{"id": 1}], headers=(("Link", f'<{self.link(2, route="repositories/7")}>; rel="next"'),)),
              response({"id": 8, "full_name": REPO})), {}),
            ((response({"total_count": 2, "workflow_runs": [{"id": 1}]}, headers=(("Link", f'<{next_link}>; rel="next"'),)),
              response({"total_count": 3, "workflow_runs": [{"id": 2}]})),
             {"root_key": "workflow_runs", "require_total_count": True}),
            ((response({"total_count": 2, "workflow_runs": [{"id": 1}]}, headers=(("Link", f'<{next_link}>; rel="next"'),)),
              response({"total_count": 3, "workflow_runs": [{"id": 2}]})),
             {"root_key": "workflow_runs"}),
            ((response({"total_count": 2, "workflow_runs": [{"id": 1}]}, headers=(("Link", f'<{next_link}>; rel="next"'),)),
              response({"workflow_runs": [{"id": 2}]})), {"root_key": "workflow_runs"}),
            ((response([{"id": 1}], headers=(("Link", f'<{next_link}>; rel="next"'),)),
              response([{"id": 1}])), {}),
            ((response([{"id": 1}], headers=(("Link", f'<{next_link}>; rel="next"'),)),
              response([])), {}),
            ((response([], headers=(("Link", f'<{next_link}>; rel="next"'),)),), {}),
            ((response({"total_count": 2, "workflow_runs": [{"id": 1}]}),),
             {"root_key": "workflow_runs", "require_total_count": True}),
            ((response([{"id": 1}], headers=(("Link", f'<{next_link}>; rel="next"'),)),
              response([{"id": 2}], headers=(("Link", f'<{self.link(2)}>; rel="first prev"'),))), {})
        )
        for outcomes, options in cases:
            with self.subTest(case=len(outcomes)):
                client, _ = self.client(*outcomes); self.assert_closed(client.get_collection(
                    "actions/runs?branch=main", **options))
    def test_local_link_contradictions_precede_numeric_preflight(self) -> None:
        numeric_seven = self.link(2, route="repositories/7")
        cases = (
            (("Link", f'<{numeric_seven}>; rel="next"'),
             ("Link", f'<{self.link(1)}>; rel="prev"')),
            (("Link", f'<{numeric_seven}>; rel="next"'),
             ("Link", f'<{self.link(2, route="repositories/8")}>; rel="last"')))
        for headers in cases:
            with self.subTest(headers=headers):
                client, opener = self.client(response([{"id": 1}], headers=headers))
                self.assert_closed(client.get_collection("actions/runs?branch=main"))
                self.assertEqual(len(opener.requests), 1)
    def test_collection_bounds_and_result_configuration_are_fail_closed(self) -> None:
        next_link = self.link(2)
        with mock.patch.object(transport, "MAX_COLLECTION_BYTES", 1):
            client, _ = self.client(response([])); self.assert_closed(client.get_collection("actions/runs"))
        with mock.patch.object(transport, "MAX_COLLECTION_ROWS", 1):
            client, _ = self.client(response([1, 2])); self.assert_closed(client.get_collection("actions/runs"))
        for root_key in ("bad key", "total_count", TOKEN):
            client, opener = self.client(response([])); self.assert_closed(
                client.get_collection("actions/runs", root_key=root_key)); self.assertEqual(opener.requests, [])
        for total in (True, -1, "2"):
            for required in (False, True):
                client, _ = self.client(response({"total_count": total, "items": []})); self.assert_closed(
                    client.get_collection("issues", root_key="items", require_total_count=required))
        client, _ = self.client(response([1], headers=(("Link", f'<{next_link}>; rel="next"'),)))
        with mock.patch.object(transport, "MAX_COLLECTION_PAGES", 1): self.assert_closed(
            client.get_collection("actions/runs?branch=main"))
        client, opener = self.client(response([])); client.get_collection(
            "actions/runs?z=1&%C3%A9=1")
        self.assertEqual(opener.requests[0][0].full_url,
                         f"{BASE}/actions/runs?%C3%A9=1&z=1&page=1&per_page=100")
        last = self.link(2)
        client, opener = self.client(response({"total_count": 100, "items": [1]}, headers=(
            ("Link", f'<{last}>; rel="next last"'),)))
        self.assert_closed(client.get_collection("actions/runs?branch=main", root_key="items"))
        self.assertEqual(len(opener.requests), 1)
    def test_stdlib_http_error_is_a_closed_response_not_a_transport_failure(self) -> None:
        stream = io.BytesIO(b'{"message":"limited"}')
        error = urllib.error.HTTPError(BASE, 429, "limited", {"Content-Type": "application/json"}, stream)
        with mock.patch.object(transport._URL_OPENER, "open", side_effect=error):
            page = transport._stdlib_open(object(), timeout=1)
        self.assertEqual((page.status, page.body), (429, b'{"message":"limited"}'))
        self.assertEqual(page.headers, (("Content-Type", "application/json"),))
        self.assertTrue(stream.closed)
if __name__ == "__main__":
    unittest.main()
