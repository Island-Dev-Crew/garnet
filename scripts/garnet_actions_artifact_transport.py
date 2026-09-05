#!/usr/bin/env python3
"""Bounded authenticated download of one GitHub Actions artifact archive.

This is the attempt-2 archive transport for the U-59 eligibility receipt
(L1 act 2).  It is deliberately a sibling of, not a member of, the Jon-only
``scripts/garnet_github_*`` family: it carries no policy and returns no
verdict.  It performs exactly one authenticated request against
``api.github.com`` and follows exactly one redirect hop to the signed blob
host, stripping ``Authorization`` on that hop.  It accepts only
``application/zip`` or ``application/octet-stream`` bodies of at most
``MAX_ARCHIVE_BYTES`` and returns the final status, the final host, the raw
archive bytes, and their SHA-256 so the caller can bind the archive by
endpoint, status, artifact id, raw-body digest, and archive digest.

The module never reads an ambient credential: the token is a constructor
argument supplied by the caller, which reads it from stdin.
"""
from __future__ import annotations

import hashlib
import re
import urllib.error
import urllib.parse
import urllib.request
from collections.abc import Callable
from dataclasses import dataclass

API_ORIGIN = "https://api.github.com"
API_HOST = "api.github.com"
API_VERSION = "2022-11-28"
USER_AGENT = "Garnet-Actions-Artifact-Transport/1"
TIMEOUT_SECONDS = 30.0
MAX_ARCHIVE_BYTES = 8 * 1024 * 1024
MAX_ARTIFACT_ID_DIGITS = 20
MAX_RESPONSE_HEADERS = 256
MAX_RESPONSE_HEADER_CHARS = 64 * 1024
ACCEPTED_MEDIA_TYPES = frozenset({"application/zip", "application/octet-stream"})
ALLOWED_PROBLEM_CODES = frozenset(
    {
        "invalid-configuration",
        "invalid-artifact-id",
        "transport-failure",
        "response-shape",
        "credential-in-response",
        "http-status",
        "redirect-missing",
        "redirect-target",
        "redirect-excess",
        "content-type",
        "content-encoding",
        "content-length",
        "archive-too-large",
    }
)
_NAME = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]{0,99}")
_HOST = re.compile(r"^[A-Za-z0-9](?:[A-Za-z0-9-]{0,62}[A-Za-z0-9])?(?:\.[A-Za-z0-9](?:[A-Za-z0-9-]{0,62}[A-Za-z0-9])?)+$")
# The only destinations an artifact-archive redirect may name: the signed-URL
# hosts GitHub serves archives from, as a subdomain of either suffix, on the
# default port, never an IP literal (review v1, F2: any DNS-shaped https host
# was accepted, loopback and arbitrary ports included).
ARCHIVE_HOST_SUFFIXES = (".blob.core.windows.net", ".actions.githubusercontent.com")


def _archive_host_admissible(host: str, port: int | None) -> bool:
    lowered = host.lower()
    if port not in (None, 443):
        return False
    if all(label.isdigit() for label in lowered.split(".")):
        return False
    return any(
        lowered.endswith(suffix) and len(lowered) > len(suffix) for suffix in ARCHIVE_HOST_SUFFIXES
    )


@dataclass(frozen=True)
class ArchiveResponse:
    """One raw HTTP exchange as observed by the opener; no interpretation."""

    status: int
    headers: tuple[tuple[str, str], ...]
    body: bytes


@dataclass(frozen=True)
class ArchiveDownload:
    """The bound archive: endpoint, final status, final host, raw bytes, digest."""

    endpoint: str
    status: int | None
    final_url_host: str | None
    raw_bytes: bytes
    sha256: str | None
    problems: tuple[str, ...] = ()

    @property
    def ok(self) -> bool:
        return not self.problems and self.status == 200 and self.sha256 is not None


class _Failure(Exception):
    def __init__(self, code: str) -> None:
        if code not in ALLOWED_PROBLEM_CODES:
            raise ValueError("unsupported transport problem code")
        super().__init__(code)
        self.code = code


def _fail(code: str) -> None:
    raise _Failure(code)


class _NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args: object, **kwargs: object) -> None:
        return None


_URL_OPENER = urllib.request.build_opener(_NoRedirect)


def _stdlib_open(request: urllib.request.Request, *, timeout: float) -> ArchiveResponse:
    try:
        response = _URL_OPENER.open(request, timeout=timeout)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        return ArchiveResponse(
            response.status,
            tuple(response.headers.items()),
            response.read(MAX_ARCHIVE_BYTES + 1),
        )


def _valid_token(token: object) -> bool:
    return (
        isinstance(token, str)
        and 0 < len(token) <= 1024
        and all(33 <= ord(char) <= 126 for char in token)
    )


def _valid_repository(repository: object) -> bool:
    names = repository.split("/") if isinstance(repository, str) else []
    return len(names) == 2 and all(_NAME.fullmatch(item) for item in names)


class ActionsArtifactTransport:
    """One-hop authenticated artifact archive download; no verdict, no policy."""

    def __init__(
        self,
        repository: str,
        token: str,
        opener: Callable[..., object] | None = None,
    ) -> None:
        bound = (
            _valid_repository(repository)
            and _valid_token(token)
            and token not in repository
            and token not in API_ORIGIN
        )
        self._repository = repository if bound else ""
        self._token = token if bound else ""
        self._opener = _stdlib_open if opener is None else opener
        self._configuration_valid = bound and callable(self._opener)

    def __repr__(self) -> str:
        return "ActionsArtifactTransport(repository=<bound>, token=<redacted>)"

    def _exchange(self, request: urllib.request.Request) -> ArchiveResponse:
        try:
            page = self._opener(request, timeout=TIMEOUT_SECONDS)
        except Exception:
            _fail("transport-failure")
        if type(page) is not ArchiveResponse:
            _fail("response-shape")
        assert isinstance(page, ArchiveResponse)
        headers = page.headers
        if (
            type(page.status) is not int
            or type(page.body) is not bytes
            or type(headers) is not tuple
            or len(headers) > MAX_RESPONSE_HEADERS
            or any(
                type(field) is not tuple
                or len(field) != 2
                or type(field[0]) is not str
                or type(field[1]) is not str
                for field in headers
            )
        ):
            _fail("response-shape")
        if sum(len(key) + len(value) for key, value in headers) > MAX_RESPONSE_HEADER_CHARS:
            _fail("response-shape")
        if any(self._token in key or self._token in value for key, value in headers):
            _fail("credential-in-response")
        return page

    @staticmethod
    def _header(page: ArchiveResponse, name: str) -> str | None:
        values = [value for key, value in page.headers if key.lower() == name]
        if not values:
            return None
        if len(values) != 1:
            _fail("response-shape")
        return values[0]

    def _redirect_target(self, page: ArchiveResponse) -> tuple[str, str]:
        location = self._header(page, "location")
        if location is None or not location:
            _fail("redirect-missing")
        assert isinstance(location, str)
        if self._token in location or any(ord(char) <= 32 or ord(char) == 127 for char in location):
            _fail("redirect-target")
        try:
            parsed = urllib.parse.urlsplit(location)
        except ValueError:
            _fail("redirect-target")
        host = parsed.hostname
        try:
            port = parsed.port
        except ValueError:
            _fail("redirect-target")
            raise AssertionError("unreachable")
        if (
            parsed.scheme != "https"
            or host is None
            or parsed.username is not None
            or parsed.password is not None
            or parsed.fragment
            or _HOST.fullmatch(host) is None
            or host.lower() == API_HOST
            or host.lower().endswith("." + API_HOST)
            or not _archive_host_admissible(host, port)
        ):
            _fail("redirect-target")
        return location, host.lower()

    def download_archive(self, artifact_id: int) -> ArchiveDownload:
        endpoint = f"actions/artifacts/{artifact_id}/zip"
        try:
            if not self._configuration_valid:
                _fail("invalid-configuration")
            if (
                type(artifact_id) is not int
                or artifact_id <= 0
                or len(str(artifact_id)) > MAX_ARTIFACT_ID_DIGITS
            ):
                _fail("invalid-artifact-id")
            first = urllib.request.Request(
                f"{API_ORIGIN}/repos/{self._repository}/{endpoint}",
                headers={
                    "Accept": "application/vnd.github+json",
                    "X-GitHub-Api-Version": API_VERSION,
                    "User-Agent": USER_AGENT,
                },
            )
            first.add_unredirected_header("Authorization", f"Bearer {self._token}")
            page = self._exchange(first)
            if page.status != 302:
                _fail("http-status")
            location, host = self._redirect_target(page)
            second = urllib.request.Request(
                location,
                headers={"Accept": "application/zip", "User-Agent": USER_AGENT},
            )
            archive = self._exchange(second)
            if 300 <= archive.status < 400:
                _fail("redirect-excess")
            if archive.status != 200:
                _fail("http-status")
            media = (self._header(archive, "content-type") or "").split(";", 1)[0].strip().lower()
            if media not in ACCEPTED_MEDIA_TYPES:
                _fail("content-type")
            encoding = self._header(archive, "content-encoding")
            if encoding is not None and encoding.strip().lower() not in {"", "identity"}:
                _fail("content-encoding")
            declared_length = self._header(archive, "content-length")
            if declared_length is not None and (
                not declared_length.isascii()
                or not declared_length.isdecimal()
                or int(declared_length) != len(archive.body)
            ):
                _fail("content-length")
            if len(archive.body) > MAX_ARCHIVE_BYTES:
                _fail("archive-too-large")
            if not archive.body:
                _fail("response-shape")
            return ArchiveDownload(
                endpoint,
                archive.status,
                host,
                archive.body,
                hashlib.sha256(archive.body).hexdigest(),
            )
        except _Failure as failure:
            return ArchiveDownload(endpoint, None, None, b"", None, (failure.code,))
