#!/usr/bin/env python3
"""Strict bounded RFC 8288 Link and physical response-header parser."""
from __future__ import annotations
import re, urllib.parse
from dataclasses import dataclass
MAX_LINK_HEADER_CHARS = 32_768
_TCHAR = frozenset("!#$%&'*+-.^_`|~0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz")
_URI_CHAR = frozenset("!#$%&'()*+,-./0123456789:;=?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]_abcdefghijklmnopqrstuvwxyz~")
_HEX = frozenset("0123456789ABCDEFabcdef")
_PCHAR = frozenset("!$&'()*+,-.0123456789:;=@ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz~")
_QUERY_FRAGMENT_CHAR = _PCHAR | frozenset("/?")
_RESTRICTED_NAME = re.compile(r"[A-Za-z0-9][A-Za-z0-9!#$&^_.+\-]{0,126}")
_REG_REL = re.compile(r"[a-z][a-z0-9.\-]*")
_SCHEME = re.compile(r"[A-Za-z][A-Za-z0-9+\-.]*")
class HeaderSyntaxError(ValueError): pass
def _bad() -> None: raise HeaderSyntaxError("invalid response headers")
@dataclass(frozen=True)
class LinkValue:
    target: str
    relations: tuple[str, ...]
    parameters: tuple[tuple[str, str | None], ...]
    def parameter(self, name: str) -> str | None:
        key = name.lower() if type(name) is str else ""
        return next((value for candidate, value in self.parameters if candidate == key), None)
@dataclass(frozen=True)
class HeaderBlock:
    fields: tuple[tuple[str, str], ...]
    links: tuple[LinkValue, ...]
    def get_all(self, name: str) -> tuple[str, ...]:
        key = name.lower() if type(name) is str else ""
        return tuple(value for candidate, value in self.fields if candidate == key)
    def get_singleton(self, name: str) -> str | None:
        values = self.get_all(name)
        if len(values) > 1: _bad()
        return values[0] if values else None
def _component(value: str, allowed: frozenset[str]) -> None:
    index = 0
    while index < len(value):
        if value[index] == "%":
            index += 3; continue
        if value[index] not in allowed: _bad()
        index += 1
def _uri_reference(value: str, *, absolute: bool = False) -> urllib.parse.SplitResult:
    if not value:
        _bad()
    index = 0
    while index < len(value):
        char = value[index]
        if char == "%":
            if index + 2 >= len(value) or value[index + 1] not in _HEX or value[index + 2] not in _HEX:
                _bad()
            index += 3; continue
        if char not in _URI_CHAR:
            _bad()
        index += 1
    try:
        parsed = urllib.parse.urlsplit(value)
        if parsed.scheme and _SCHEME.fullmatch(parsed.scheme) is None: _bad()
        if value.startswith("//") and not parsed.netloc: _bad()
        if parsed.netloc:
            if parsed.netloc.count("@") > 1: _bad()
            if parsed.hostname is None: _bad()
            _ = parsed.port
    except (ValueError, UnicodeError):
        _bad()
    if absolute and not parsed.scheme:
        _bad()
    _component(parsed.path, _PCHAR | frozenset("/"))
    _component(parsed.query, _QUERY_FRAGMENT_CHAR)
    _component(parsed.fragment, _QUERY_FRAGMENT_CHAR)
    if value.count("#") > 1: _bad()
    return parsed
def _quoted(value: str, index: int) -> tuple[str, int]:
    index += 1; output: list[str] = []
    while index < len(value) and value[index] != '"':
        char = value[index]
        if char == "\\":
            index += 1
            if index == len(value): _bad()
            char = value[index]
            if not (char == "\t" or 32 <= ord(char) <= 126 or 128 <= ord(char) <= 255): _bad()
        elif not (char == "\t" or char == " " or char == "!"
                  or 35 <= ord(char) <= 91 or 93 <= ord(char) <= 126
                  or 128 <= ord(char) <= 255):
            _bad()
        output.append(char); index += 1
    if index == len(value): _bad()
    return "".join(output), index + 1
def _relation_types(value: str) -> tuple[str, ...]:
    if not value or value[0] == " " or value[-1] == " " or "\t" in value:
        _bad()
    relations = tuple(re.split(" +", value))
    normalized: list[str] = []
    for relation in relations:
        if _REG_REL.fullmatch(relation) is not None:
            normalized.append(relation)
        else:
            parsed = _uri_reference(relation, absolute=True)
            if parsed.fragment: _bad()
            normalized.append(relation.lower())
    if len(set(normalized)) != len(normalized):
        _bad()
    return tuple(normalized)
def _parse_link_field(value: str) -> tuple[LinkValue, ...]:
    if not value or any((ord(char) < 32 and char != "\t") or ord(char) > 255 for char in value):
        _bad()
    index, links = 0, []
    while index < len(value):
        while index < len(value) and value[index] in " \t": index += 1
        if index == len(value) or value[index] != "<": _bad()
        close = value.find(">", index + 1)
        if close < 0: _bad()
        target = value[index + 1:close]; _uri_reference(target)
        index = close + 1; parameters: list[tuple[str, str | None]] = []
        names: set[str] = set()
        while True:
            while index < len(value) and value[index] in " \t": index += 1
            if index == len(value) or value[index] == ",": break
            if value[index] != ";": _bad()
            index += 1
            while index < len(value) and value[index] in " \t": index += 1
            start = index
            while index < len(value) and value[index] in _TCHAR: index += 1
            if start == index: _bad()
            name = value[start:index].lower()
            if name in names: _bad()
            names.add(name)
            while index < len(value) and value[index] in " \t": index += 1
            parameter: str | None = None; quoted = False
            if index < len(value) and value[index] == "=":
                index += 1
                while index < len(value) and value[index] in " \t": index += 1
                if index < len(value) and value[index] == '"':
                    quoted = True; parameter, index = _quoted(value, index)
                else:
                    start = index
                    while index < len(value) and value[index] in _TCHAR: index += 1
                    if start == index: _bad()
                    parameter = value[start:index]
            if name == "type":
                if not quoted or parameter is None or parameter.count("/") != 1: _bad()
                major, minor = parameter.split("/", 1)
                if (_RESTRICTED_NAME.fullmatch(major) is None
                        or _RESTRICTED_NAME.fullmatch(minor) is None): _bad()
            if name == "anchor":
                if parameter is None: _bad()
                _uri_reference(parameter)
            parameters.append((name, parameter))
        rel = next((item for name, item in parameters if name == "rel"), None)
        if rel is None: _bad()
        links.append(LinkValue(target, _relation_types(rel), tuple(parameters)))
        if index < len(value):
            index += 1
            if index == len(value): _bad()
    return tuple(links)
def parse_header_fields(fields: object) -> HeaderBlock:
    if type(fields) is not tuple:
        _bad()
    physical: list[tuple[str, str]] = []
    physical_links: list[str] = []
    for field in fields:
        if type(field) is not tuple or len(field) != 2: _bad()
        name, value = field
        if (type(name) is not str or type(value) is not str or not name
                or any(char not in _TCHAR for char in name)
                or any((ord(char) < 32 and char != "\t") or ord(char) == 127
                       or ord(char) > 255 for char in value)):
            _bad()
        lower = name.lower()
        physical.append((lower, value))
        if lower == "link": physical_links.append(value)
    aggregate = sum(len(value) for value in physical_links) + max(0, 2 * (len(physical_links) - 1))
    if aggregate > MAX_LINK_HEADER_CHARS: _bad()
    links: list[LinkValue] = []; relations: set[str] = set()
    for value in physical_links:
        for link in _parse_link_field(value):
            if any(relation in relations for relation in link.relations): _bad()
            relations.update(link.relations); links.append(link)
    return HeaderBlock(tuple(physical), tuple(links))
