#!/usr/bin/env python3
"""Authenticated, bounded GitHub REST object transport; no governance verdict."""
from __future__ import annotations
import json, math, re, unicodedata
import urllib.parse, urllib.request
from collections.abc import Callable
from dataclasses import dataclass
API_ORIGIN, API_VERSION = "https://api.github.com", "2022-11-28"
USER_AGENT, TIMEOUT_SECONDS = "Garnet-Governance-Object-Transport/1", 15.0
MAX_BODY_BYTES, MAX_JSON_DEPTH = 2 * 1024 * 1024, 32
MAX_JSON_NODES, MAX_INTEGER_DIGITS = 10_000, 256
ALLOWED_PROBLEM_CODES = frozenset({
    "invalid-configuration", "invalid-path", "transport-failure", "response-shape",
    "response-too-large", "rate-limit", "http-status", "content-type",
    "json-invalid", "json-limit", "credential-in-response", "object-shape",
})
_NAME = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]{0,99}")
_MEDIA_SUBTYPE = re.compile(r"[!#$%&'*+\-.^_`|~0-9A-Za-z]+")
_HEX = frozenset("0123456789ABCDEF")
_UNRESERVED = frozenset(b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~")
_FORBIDDEN_ESCAPED = frozenset({*range(32), 37, 47, 92, 127})
@dataclass(frozen=True)
class GitHubTransportProblem:
    code: str
    def __post_init__(self) -> None:
        if type(self.code) is not str or self.code not in ALLOWED_PROBLEM_CODES:
            raise ValueError("unsupported transport problem code")
@dataclass(frozen=True)
class PageResponse:
    status: int; headers: dict[str, str]; body: bytes
@dataclass(frozen=True)
class ObjectResult:
    value: object | None = None; problems: tuple[GitHubTransportProblem, ...] = (); byte_count: int = 0
class _Failure(Exception):
    def __init__(self, code: str) -> None:
        self.problem = GitHubTransportProblem(code)
class _JsonLimit(ValueError): pass
def _fail(code: str) -> None: raise _Failure(code)
def _has_control(value: str) -> bool: return any(ord(char) < 32 or ord(char) == 127 for char in value)
def _decode_component(value: str, token: str, *, allow_empty: bool = False) -> str:
    if (not value and not allow_empty) or any(
            ord(char) <= 32 or ord(char) > 126 for char in value):
        _fail("invalid-path")
    output = bytearray()
    index = 0
    while index < len(value):
        if value[index] != "%":
            output.append(ord(value[index]))
            index += 1
            continue
        if (index + 2 >= len(value) or value[index + 1] not in _HEX
                or value[index + 2] not in _HEX):
            _fail("invalid-path")
        byte = int(value[index + 1:index + 3], 16)
        if byte in _FORBIDDEN_ESCAPED or byte in _UNRESERVED:
            _fail("invalid-path")
        output.append(byte)
        index += 3
    try:
        decoded = output.decode("utf-8", errors="strict")
    except UnicodeError:
        _fail("invalid-path")
    invalid = (token in value or token in decoded or "%" in decoded
               or "/" in decoded or "\\" in decoded or _has_control(decoded)
               or any(unicodedata.category(char) in {"Cc", "Cf"} for char in decoded))
    if invalid:
        _fail("invalid-path")
    return decoded
def _canonical_endpoint(path: str, token: str) -> str:
    if (type(path) is not str or not 0 < len(path) <= 2048
            or any(ord(char) <= 32 or ord(char) == 127 for char in path)
            or token in path):
        _fail("invalid-path")
    try:
        parsed = urllib.parse.urlsplit(path)
    except ValueError:
        _fail("invalid-path")
    if parsed.scheme or parsed.netloc or parsed.fragment or parsed.path.startswith("/"):
        _fail("invalid-path")
    segments = [_decode_component(item, token) for item in parsed.path.split("/")]
    if any(item in {".", ".."} for item in segments):
        _fail("invalid-path")
    if "?" in path and not parsed.query:
        _fail("invalid-path")
    seen: set[str] = set()
    for field in parsed.query.split("&") if parsed.query else ():
        if field.count("=") != 1:
            _fail("invalid-path")
        key, value = field.split("=", 1)
        decoded_key = _decode_component(key, token)
        _decode_component(value, token, allow_empty=True)
        if decoded_key in seen:
            _fail("invalid-path")
        seen.add(decoded_key)
    return parsed.path + (f"?{parsed.query}" if parsed.query else "")
def _json_object(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate JSON key")
        result[key] = value
    return result
def _bounded_int(value: str) -> int:
    if len(value.removeprefix("-")) > MAX_INTEGER_DIGITS:
        raise _JsonLimit()
    return int(value)
def _bounded_float(value: str) -> float:
    result = float(value)
    if not math.isfinite(result):
        raise _JsonLimit()
    return result
def _validate_tree(value: object, token: str) -> None:
    stack: list[tuple[object, int]] = [(value, 1)]
    nodes = 0
    while stack:
        item, depth = stack.pop()
        nodes += 1
        if nodes > MAX_JSON_NODES or depth > MAX_JSON_DEPTH:
            raise _JsonLimit()
        if isinstance(item, str) and token in item:
            _fail("credential-in-response")
        if isinstance(item, dict):
            nodes += len(item)
            if nodes > MAX_JSON_NODES:
                raise _JsonLimit()
            if any(token in key for key in item):
                _fail("credential-in-response")
            stack.extend((child, depth + 1) for child in item.values())
        elif isinstance(item, list):
            stack.extend((child, depth + 1) for child in item)
def _decode_json(body: bytes, token: str) -> object:
    try:
        value = json.loads(body.decode("utf-8", errors="strict"),
            object_pairs_hook=_json_object, parse_int=_bounded_int,
            parse_float=_bounded_float,
            parse_constant=lambda _: (_ for _ in ()).throw(ValueError()))
        _validate_tree(value, token)
        return value
    except _JsonLimit:
        _fail("json-limit")
    except (UnicodeError, json.JSONDecodeError, RecursionError, ValueError):
        _fail("json-invalid")
class _NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args: object, **kwargs: object) -> None: return None
_URL_OPENER = urllib.request.build_opener(_NoRedirect)
def _stdlib_open(request: object, *, timeout: float) -> PageResponse:
    with _URL_OPENER.open(request, timeout=timeout) as response:
        return PageResponse(response.status, dict(response.headers.items()),
                            response.read(MAX_BODY_BYTES + 1))
class GitHubGovernanceTransport:
    def __init__(self, repo: str, token: str,
                 opener: Callable[..., object] | None = None) -> None:
        names = repo.split("/") if isinstance(repo, str) else []
        valid_repo = len(names) == 2 and all(_NAME.fullmatch(item) for item in names)
        valid_token = (isinstance(token, str) and 0 < len(token) <= 1024
                       and all(33 <= ord(char) <= 126 for char in token))
        bound = valid_repo and valid_token and token not in repo and token not in API_ORIGIN
        self._repo = repo if bound else ""
        self._token = token if bound else ""
        self._opener = _stdlib_open if opener is None else opener
        self._configuration_valid = bound and callable(self._opener)
    def __repr__(self) -> str:
        return "GitHubGovernanceTransport(repo=<bound>, token=<redacted>)"
    def _read(self, url: str) -> tuple[object, int]:
        request = urllib.request.Request(url, headers={
            "Accept": "application/vnd.github+json",
            "X-GitHub-Api-Version": API_VERSION,
            "User-Agent": USER_AGENT,
        })
        request.add_unredirected_header("Authorization", f"Bearer {self._token}")
        try:
            page = self._opener(request, timeout=TIMEOUT_SECONDS)
        except Exception:
            _fail("transport-failure")
        try:
            if type(page) is not PageResponse:
                raise TypeError
            status, source_headers, body = page.status, page.headers, page.body
            if (type(status) is not int or type(source_headers) is not dict
                    or type(body) is not bytes):
                raise TypeError
            pairs = tuple(source_headers.items())
            if any(type(key) is not str or type(value) is not str
                   for key, value in pairs):
                raise TypeError
            headers = {key.lower(): value for key, value in pairs}
            if len(headers) != len(pairs):
                raise TypeError
            size = len(body)
        except Exception:
            _fail("response-shape")
        if size > MAX_BODY_BYTES:
            _fail("response-too-large")
        remaining = headers.get("x-ratelimit-remaining")
        if remaining is not None and (not remaining.isascii() or not remaining.isdecimal() or not remaining.strip("0")):
            _fail("rate-limit")
        if not 200 <= status < 300:
            _fail("http-status")
        media = headers.get("content-type", "").split(";", 1)[0].lower()
        subtype = media.removeprefix("application/")
        if (media != "application/json" and (not media.startswith("application/")
                or len(subtype) <= len("+json") or not subtype.endswith("+json")
                or _MEDIA_SUBTYPE.fullmatch(subtype) is None)):
            _fail("content-type")
        return _decode_json(body, self._token), size
    def get_object(self, path: str) -> ObjectResult:
        try:
            if not self._configuration_valid:
                _fail("invalid-configuration")
            endpoint = _canonical_endpoint(path, self._token)
            url = f"{API_ORIGIN}/repos/{self._repo}/{endpoint}"
            if self._token in url:
                _fail("invalid-path")
            value, size = self._read(url)
            if not isinstance(value, dict):
                _fail("object-shape")
            return ObjectResult(value, (), size)
        except _Failure as failure:
            return ObjectResult(problems=(failure.problem,))
