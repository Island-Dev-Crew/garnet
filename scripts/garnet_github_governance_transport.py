#!/usr/bin/env python3
"""Authenticated GitHub REST object and bounded page-number transport; no verdict."""
from __future__ import annotations
import hashlib, importlib.util, json, math, re, sys, unicodedata
import urllib.error, urllib.parse, urllib.request
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path
_LINK_SPEC = importlib.util.spec_from_file_location(
    "_garnet_github_link_headers_transport", Path(__file__).with_name("garnet_github_link_headers.py"))
assert _LINK_SPEC and _LINK_SPEC.loader
_link_headers = importlib.util.module_from_spec(_LINK_SPEC)
sys.modules[_LINK_SPEC.name] = _link_headers; _LINK_SPEC.loader.exec_module(_link_headers)
HeaderBlock, parse_header_fields = _link_headers.HeaderBlock, _link_headers.parse_header_fields
API_ORIGIN, API_VERSION = "https://api.github.com", "2022-11-28"
USER_AGENT, TIMEOUT_SECONDS = "Garnet-Governance-Object-Transport/1", 15.0
MAX_BODY_BYTES, MAX_JSON_DEPTH = 2 * 1024 * 1024, 32
MAX_JSON_NODES, MAX_INTEGER_DIGITS = 10_000, 256
MAX_RESPONSE_HEADERS, MAX_RESPONSE_HEADER_CHARS = 256, 64 * 1024
MAX_COLLECTION_BYTES, MAX_COLLECTION_PAGES = 16 * 1024 * 1024, 32
MAX_COLLECTION_ROWS, COLLECTION_PAGE_SIZE = 3_200, 100
ALLOWED_PROBLEM_CODES = frozenset({
    "invalid-configuration", "invalid-path", "transport-failure", "response-shape",
    "response-too-large", "rate-limit", "http-status", "content-type",
    "json-invalid", "json-limit", "credential-in-response", "object-shape",
    "collection-shape", "collection-limit", "pagination",
})
_NAME = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]{0,99}")
_MEDIA_SUBTYPE = re.compile(r"[!#$%&'*+\-.^_`|~0-9A-Za-z]+")
_HEX = frozenset("0123456789ABCDEF")
_UNRESERVED = frozenset(b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~")
_FORBIDDEN_ESCAPED = frozenset({*range(32), 37, 47, 92, 127})
_PAGINATION_KEYS = frozenset({"page", "per_page", "cursor", "since", "before", "after"})
_PAGE_RELATIONS = frozenset({"first", "prev", "next", "last"})
@dataclass(frozen=True)
class GitHubTransportProblem:
    code: str
    def __post_init__(self) -> None:
        if type(self.code) is not str or self.code not in ALLOWED_PROBLEM_CODES:
            raise ValueError("unsupported transport problem code")
@dataclass(frozen=True)
class PageResponse:
    status: int; headers: tuple[tuple[str, str], ...]; body: bytes
@dataclass(frozen=True)
class ObjectResult:
    value: object | None = None; problems: tuple[GitHubTransportProblem, ...] = (); byte_count: int = 0
@dataclass(frozen=True)
class CollectionResult:
    rows: tuple[object, ...] = (); problems: tuple[GitHubTransportProblem, ...] = ()
    page_count: int = 0; byte_count: int = 0
@dataclass(frozen=True)
class _DecodedResponse:
    value: object; headers: HeaderBlock; byte_count: int; body_hash: bytes
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
    raw_segments = parsed.path.split("/")
    segments = [_decode_component(item, token) for item in raw_segments]
    if (any(item in {".", ".."} for item in segments)
            or any(raw != urllib.parse.quote(decoded, safe="!$&'()*+,-.:;=@_~")
                   for raw, decoded in zip(raw_segments, segments))):
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
def _query_pairs(query: str, token: str) -> tuple[tuple[str, str], ...]:
    result: list[tuple[str, str]] = []; seen: set[str] = set()
    for field in query.split("&") if query else ():
        if field.count("=") != 1: _fail("invalid-path")
        raw_key, raw_value = field.split("=", 1)
        key = _decode_component(raw_key, token); value = _decode_component(
            raw_value, token, allow_empty=True)
        if (raw_key != urllib.parse.quote(key, safe="-._~")
                or raw_value != urllib.parse.quote(value, safe="-._~") or key in seen):
            _fail("invalid-path")
        seen.add(key); result.append((key, value))
    return tuple(result)
def _collection_endpoint(path: str, token: str) -> tuple[str, tuple[tuple[str, str], ...]]:
    endpoint = _canonical_endpoint(path, token); parsed = urllib.parse.urlsplit(endpoint)
    fixed = _query_pairs(parsed.query, token)
    if any(key.lower() in _PAGINATION_KEYS for key, _ in fixed): _fail("invalid-path")
    return parsed.path, tuple(sorted(fixed, key=lambda pair: tuple(
        urllib.parse.quote(item, safe="-._~") for item in pair)))
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
    try: response = _URL_OPENER.open(request, timeout=timeout)
    except urllib.error.HTTPError as error: response = error
    with response:
        return PageResponse(response.status, tuple(response.headers.items()),
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
    def _read(self, url: str) -> _DecodedResponse:
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
            if type(status) is not int or type(body) is not bytes:
                raise TypeError
            if type(source_headers) is not tuple: raise TypeError
            pairs = source_headers
            if (len(pairs) > MAX_RESPONSE_HEADERS or any(
                    type(field) is not tuple or len(field) != 2
                    or type(field[0]) is not str or type(field[1]) is not str
                    for field in pairs)):
                raise TypeError
            if sum(len(key) + len(value) for key, value in pairs) > MAX_RESPONSE_HEADER_CHARS:
                raise TypeError
            if any(self._token in key or self._token in value for key, value in pairs):
                _fail("credential-in-response")
            raw_remaining = tuple(value for key, value in pairs if type(key) is str
                                  and key.lower() == "x-ratelimit-remaining")
            if len(raw_remaining) == 1 and type(raw_remaining[0]) is str and (
                    not raw_remaining[0].isascii() or not raw_remaining[0].isdecimal()
                    or not raw_remaining[0].strip("0")): _fail("rate-limit")
            headers = parse_header_fields(pairs); size = len(body)
            remaining = headers.get_singleton("x-ratelimit-remaining")
            media_header = headers.get_singleton("content-type") or ""
        except _Failure:
            raise
        except Exception:
            _fail("response-shape")
        if size > MAX_BODY_BYTES:
            _fail("response-too-large")
        if remaining is not None and (not remaining.isascii() or not remaining.isdecimal() or not remaining.strip("0")):
            _fail("rate-limit")
        if not 200 <= status < 300:
            _fail("http-status")
        media = media_header.split(";", 1)[0].lower()
        subtype = media.removeprefix("application/")
        if (media != "application/json" and (not media.startswith("application/")
                or len(subtype) <= len("+json") or not subtype.endswith("+json")
                or _MEDIA_SUBTYPE.fullmatch(subtype) is None)):
            _fail("content-type")
        return _DecodedResponse(_decode_json(body, self._token), headers, size,
                                hashlib.sha256(body).digest())
    def _link_target(self, target: str, endpoint: str,
                     fixed: tuple[tuple[str, str], ...]) -> tuple[int, int | None]:
        try: parsed = urllib.parse.urlsplit(target)
        except ValueError: _fail("pagination")
        if (parsed.scheme != "https" or parsed.netloc != "api.github.com"
                or parsed.fragment): _fail("pagination")
        named = f"/repos/{self._repo}/{endpoint}"; repository_id: int | None = None
        if parsed.path != named:
            prefix = "/repositories/"
            if not parsed.path.startswith(prefix): _fail("pagination")
            identifier, separator, tail = parsed.path[len(prefix):].partition("/")
            if (not separator or tail != endpoint or not identifier.isascii()
                    or not identifier.isdecimal() or identifier.startswith("0")
                    or len(identifier) > 20):
                _fail("pagination")
            repository_id = int(identifier)
        pairs = _query_pairs(parsed.query, self._token); values = dict(pairs)
        page, per_page = values.pop("page", None), values.pop("per_page", None)
        if (page is None or per_page != str(COLLECTION_PAGE_SIZE)
                or not page.isascii() or not page.isdecimal() or page.startswith("0")
                or values != dict(fixed)):
            _fail("pagination")
        if len(page) > len(str(MAX_COLLECTION_PAGES)): _fail("collection-limit")
        page_number = int(page)
        if page_number > MAX_COLLECTION_PAGES: _fail("collection-limit")
        return page_number, repository_id
    def _repository_identity(self) -> tuple[int, int]:
        response = self._read(f"{API_ORIGIN}/repos/{self._repo}"); value = response.value
        if (type(value) is not dict or type(value.get("id")) is not int
                or value["id"] <= 0 or value.get("full_name") != self._repo):
            _fail("pagination")
        return value["id"], response.byte_count
    def get_collection(self, path: str, *, root_key: str | None = None,
                       require_total_count: bool = False) -> CollectionResult:
        try:
            if not self._configuration_valid: _fail("invalid-configuration")
            if (type(require_total_count) is not bool or (root_key is not None and
                    (type(root_key) is not str or _NAME.fullmatch(root_key) is None
                     or root_key == "total_count" or self._token in root_key))
                    or require_total_count and root_key is None): _fail("invalid-path")
            endpoint, fixed = _collection_endpoint(path, self._token)
            encoded = "&".join(f"{urllib.parse.quote(key, safe='-._~')}="
                               f"{urllib.parse.quote(value, safe='-._~')}" for key, value in fixed)
            rows: list[object] = []; seen_bodies: set[bytes] = set()
            page = 1; pages = 0; byte_count = 0
            repository_id: int | None = None; frozen_last: int | None = None
            frozen_total: int | None = None; total_presence: bool | None = None
            while True:
                if page > MAX_COLLECTION_PAGES: _fail("collection-limit")
                query = f"{encoded}&" if encoded else ""
                response = self._read(f"{API_ORIGIN}/repos/{self._repo}/{endpoint}?"
                                      f"{query}page={page}&per_page={COLLECTION_PAGE_SIZE}")
                pages += 1; byte_count += response.byte_count
                if byte_count > MAX_COLLECTION_BYTES: _fail("collection-limit")
                if response.body_hash in seen_bodies: _fail("pagination")
                seen_bodies.add(response.body_hash); value = response.value
                if root_key is None:
                    if type(value) is not list: _fail("collection-shape")
                    current, total = tuple(value), None
                else:
                    if type(value) is not dict or type(value.get(root_key)) is not list:
                        _fail("collection-shape")
                    current = tuple(value[root_key]); present = "total_count" in value
                    total = value.get("total_count")
                    if (require_total_count and not present or present and
                            (type(total) is not int or total < 0)): _fail("collection-shape")
                    if total_presence is None: total_presence = present
                    elif total_presence != present: _fail("pagination")
                    if present and frozen_total is None: frozen_total = total
                    elif present and total != frozen_total: _fail("pagination")
                    if present and total > MAX_COLLECTION_ROWS: _fail("collection-limit")
                if len(current) > COLLECTION_PAGE_SIZE: _fail("collection-limit")
                relations: list[tuple[str, int]] = []; target_ids: set[int] = set()
                for link in response.headers.links:
                    target_page, target_id = self._link_target(link.target, endpoint, fixed)
                    if link.parameter("anchor") is not None: _fail("pagination")
                    if target_id is not None: target_ids.add(target_id)
                    for relation in link.relations:
                        if relation not in _PAGE_RELATIONS: _fail("pagination")
                        relations.append((relation, target_page))
                frozen_relations = tuple(relations); relation_pages = dict(frozen_relations)
                first, previous = relation_pages.get("first"), relation_pages.get("prev")
                following, last = relation_pages.get("next"), relation_pages.get("last")
                if (first is not None and first != 1
                        or previous is not None and (page == 1 or previous != page - 1)
                        or following is not None and following != page + 1): _fail("pagination")
                if last is not None:
                    if last < page or frozen_last is not None and last != frozen_last:
                        _fail("pagination")
                    frozen_last = last
                if (total is not None and frozen_last is not None
                        and frozen_last != max(1, math.ceil(total / COLLECTION_PAGE_SIZE))): _fail("pagination")
                if (following is None and last is not None and last > page
                        or frozen_last is not None and (following is not None and following > frozen_last
                                                       or following is None and page != frozen_last)):
                    _fail("pagination")
                prospective = len(rows) + len(current)
                if prospective > MAX_COLLECTION_ROWS: _fail("collection-limit")
                if total is not None and (total < prospective
                        or (following is not None) != (prospective < total)): _fail("pagination")
                if not current and (pages > 1 or following is not None): _fail("pagination")
                if (
                    len(current) == COLLECTION_PAGE_SIZE
                    and following is None
                    and total is None
                ):
                    _fail("pagination")
                if target_ids:
                    if len(target_ids) != 1: _fail("pagination")
                    target_id = next(iter(target_ids))
                    if repository_id is None:
                        repository_id, preflight_bytes = self._repository_identity()
                        byte_count += preflight_bytes
                        if byte_count > MAX_COLLECTION_BYTES: _fail("collection-limit")
                    if target_id != repository_id: _fail("pagination")
                rows.extend(current)
                if following is None:
                    return CollectionResult(tuple(rows), (), pages, byte_count)
                page = following
        except _Failure as failure:
            return CollectionResult(problems=(failure.problem,))
    def get_object(self, path: str) -> ObjectResult:
        try:
            if not self._configuration_valid:
                _fail("invalid-configuration")
            endpoint = _canonical_endpoint(path, self._token)
            url = f"{API_ORIGIN}/repos/{self._repo}/{endpoint}"
            if self._token in url:
                _fail("invalid-path")
            response = self._read(url)
            if not isinstance(response.value, dict):
                _fail("object-shape")
            return ObjectResult(response.value, (), response.byte_count)
        except _Failure as failure:
            return ObjectResult(problems=(failure.problem,))
