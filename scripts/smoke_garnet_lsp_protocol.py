#!/usr/bin/env python3
"""Smoke-test the Garnet LSP over stdio without editor automation.

This supplements the S1 VSCode dogfood block. It intentionally stays small and
stdlib-only so clean clones can reproduce diagnostics, hover, and go-to-def
without relying on a particular GUI session.
"""

from __future__ import annotations

import argparse
import json
import os
import queue
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]


def write_message(proc: subprocess.Popen[bytes], payload: dict[str, Any]) -> None:
    assert proc.stdin is not None
    body = json.dumps(payload, separators=(",", ":")).encode("utf-8")
    header = f"Content-Length: {len(body)}\r\n\r\n".encode("ascii")
    proc.stdin.write(header + body)
    proc.stdin.flush()


def reader(proc: subprocess.Popen[bytes], out: "queue.Queue[dict[str, Any] | BaseException]") -> None:
    assert proc.stdout is not None
    try:
        while True:
            headers: dict[str, str] = {}
            while True:
                line = proc.stdout.readline()
                if line == b"":
                    return
                if line in (b"\r\n", b"\n"):
                    break
                key, _, value = line.decode("ascii").partition(":")
                headers[key.lower()] = value.strip()

            length = int(headers.get("content-length", "0"))
            body = proc.stdout.read(length)
            if not body:
                return
            out.put(json.loads(body.decode("utf-8")))
    except BaseException as exc:  # pragma: no cover - reported by caller
        out.put(exc)


def wait_for(
    messages: "queue.Queue[dict[str, Any] | BaseException]",
    predicate: Any,
    *,
    timeout: float = 5.0,
) -> dict[str, Any]:
    deadline = time.monotonic() + timeout
    seen: list[dict[str, Any]] = []
    while time.monotonic() < deadline:
        remaining = max(0.0, deadline - time.monotonic())
        try:
            message = messages.get(timeout=min(0.25, remaining))
        except queue.Empty:
            continue
        if isinstance(message, BaseException):
            raise RuntimeError("LSP reader failed") from message
        seen.append(message)
        if predicate(message):
            return message

    raise TimeoutError(f"timed out waiting for LSP response; seen={seen!r}")


def file_uri(path: Path) -> str:
    return path.resolve().as_uri()


def run_smoke(executable: Path) -> dict[str, Any]:
    if not executable.exists():
        raise FileNotFoundError(f"missing garnet-lsp executable: {executable}")

    proc = subprocess.Popen(
        [str(executable)],
        cwd=ROOT,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    messages: "queue.Queue[dict[str, Any] | BaseException]" = queue.Queue()
    thread = threading.Thread(target=reader, args=(proc, messages), daemon=True)
    thread.start()

    try:
        write_message(
            proc,
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "processId": os.getpid(),
                    "rootUri": file_uri(ROOT),
                    "capabilities": {},
                },
            },
        )
        initialize = wait_for(messages, lambda message: message.get("id") == 1)
        capabilities = initialize["result"]["capabilities"]

        write_message(
            proc,
            {"jsonrpc": "2.0", "method": "initialized", "params": {}},
        )

        invalid_source = (
            "def greet(name) {\n"
            "  name\n"
            "}\n\n"
            "@caps()\n"
            "def main() {\n"
            "  greet(\"Ada\"\n"
            "}\n"
        )
        invalid_uri = file_uri(Path(tempfile.gettempdir()) / "garnet_lsp_invalid.garnet")
        write_message(
            proc,
            {
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": invalid_uri,
                        "languageId": "garnet",
                        "version": 1,
                        "text": invalid_source,
                    }
                },
            },
        )
        invalid_diagnostics = wait_for(
            messages,
            lambda message: message.get("method") == "textDocument/publishDiagnostics"
            and message.get("params", {}).get("uri") == invalid_uri
            and len(message.get("params", {}).get("diagnostics", [])) >= 1,
        )

        clean_source = (
            "/// Friendly greeting\n"
            "def greet(name) {\n"
            "  name\n"
            "}\n\n"
            "@caps()\n"
            "def main() {\n"
            "  greet(\"Ada\")\n"
            "}\n"
        )
        clean_uri = file_uri(Path(tempfile.gettempdir()) / "garnet_lsp_clean.garnet")
        write_message(
            proc,
            {
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": clean_uri,
                        "languageId": "garnet",
                        "version": 1,
                        "text": clean_source,
                    }
                },
            },
        )
        clean_diagnostics = wait_for(
            messages,
            lambda message: message.get("method") == "textDocument/publishDiagnostics"
            and message.get("params", {}).get("uri") == clean_uri,
        )

        write_message(
            proc,
            {
                "jsonrpc": "2.0",
                "id": 2,
                "method": "textDocument/hover",
                "params": {
                    "textDocument": {"uri": clean_uri},
                    "position": {"line": 1, "character": 5},
                },
            },
        )
        hover = wait_for(messages, lambda message: message.get("id") == 2)
        hover_value = hover["result"]["contents"]["value"]
        if "def greet(name)" not in hover_value or "Friendly greeting" not in hover_value:
            raise AssertionError(f"unexpected hover payload: {hover_value!r}")

        write_message(
            proc,
            {
                "jsonrpc": "2.0",
                "id": 3,
                "method": "textDocument/definition",
                "params": {
                    "textDocument": {"uri": clean_uri},
                    "position": {"line": 7, "character": 3},
                },
            },
        )
        definition = wait_for(messages, lambda message: message.get("id") == 3)
        location = definition["result"]
        if location["range"]["start"] != {"line": 1, "character": 4}:
            raise AssertionError(f"unexpected definition location: {location!r}")

        write_message(proc, {"jsonrpc": "2.0", "id": 4, "method": "shutdown", "params": None})
        wait_for(messages, lambda message: message.get("id") == 4)
        write_message(proc, {"jsonrpc": "2.0", "method": "exit", "params": None})

        return {
            "status": "pass",
            "server": str(executable),
            "capabilities": capabilities,
            "invalid_diagnostics": invalid_diagnostics["params"]["diagnostics"],
            "clean_diagnostics": clean_diagnostics["params"]["diagnostics"],
            "hover_contains": ["def greet(name)", "Friendly greeting"],
            "definition_range": location["range"],
        }
    finally:
        try:
            proc.wait(timeout=2)
        except subprocess.TimeoutExpired:
            proc.terminate()
            proc.wait(timeout=2)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "executable",
        nargs="?",
        default=str(ROOT / "target" / "release" / "garnet-lsp"),
        help="path to the garnet-lsp executable",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        print(json.dumps(run_smoke(Path(args.executable)), indent=2))
    except Exception as exc:  # pragma: no cover - command-line report
        print(f"LSP smoke failed: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
