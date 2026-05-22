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

        write_message(
            proc,
            {
                "jsonrpc": "2.0",
                "id": 5,
                "method": "textDocument/documentSymbol",
                "params": {
                    "textDocument": {"uri": clean_uri},
                },
            },
        )
        doc_sym = wait_for(messages, lambda message: message.get("id") == 5)
        symbols = doc_sym["result"]
        names = [s["name"] for s in symbols]
        if "greet" not in names or "main" not in names:
            raise AssertionError(f"missing expected document symbols: {names!r}")

        write_message(
            proc,
            {
                "jsonrpc": "2.0",
                "id": 6,
                "method": "workspace/symbol",
                "params": {
                    "query": "greet",
                },
            },
        )
        work_sym = wait_for(messages, lambda message: message.get("id") == 6)
        w_symbols = work_sym["result"]
        w_names = [s["name"] for s in w_symbols]
        if "greet" not in w_names:
            raise AssertionError(f"missing expected workspace symbol: {w_names!r}")

        write_message(
            proc,
            {
                "jsonrpc": "2.0",
                "id": 7,
                "method": "textDocument/rename",
                "params": {
                    "textDocument": {"uri": clean_uri},
                    "position": {"line": 1, "character": 5},
                    "newName": "greet_all",
                },
            },
        )
        rename_res = wait_for(messages, lambda message: message.get("id") == 7)
        changes = rename_res["result"]["changes"]
        if clean_uri not in changes:
            raise AssertionError(f"expected rename changes on {clean_uri}")
        edits = changes[clean_uri]
        if len(edits) != 2:
            raise AssertionError(f"expected 2 rename edits, got {len(edits)}: {edits!r}")

        advisory_source = (
            "def main() {\n"
            "}\n"
        )
        advisory_uri = file_uri(Path(tempfile.gettempdir()) / "garnet_lsp_advisory.garnet")
        write_message(
            proc,
            {
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": advisory_uri,
                        "languageId": "garnet",
                        "version": 1,
                        "text": advisory_source,
                    }
                },
            },
        )
        advisory_diagnostics = wait_for(
            messages,
            lambda message: message.get("method") == "textDocument/publishDiagnostics"
            and message.get("params", {}).get("uri") == advisory_uri,
        )
        diagnostics = advisory_diagnostics["params"]["diagnostics"]
        advisory_diag = next((d for d in diagnostics if d.get("code") in ("ManagedFnMissingCaps", "managed-fn-missing-caps")), None)
        if not advisory_diag:
            raise AssertionError(f"expected managed-fn-missing-caps advisory diagnostic, got: {diagnostics!r}")

        write_message(
            proc,
            {
                "jsonrpc": "2.0",
                "id": 8,
                "method": "textDocument/codeAction",
                "params": {
                    "textDocument": {"uri": advisory_uri},
                    "range": advisory_diag["range"],
                    "context": {
                        "diagnostics": [advisory_diag],
                    },
                },
            },
        )
        code_action_res = wait_for(messages, lambda message: message.get("id") == 8)
        actions = code_action_res["result"]
        quick_fix_actions = [
            action for action in actions if action.get("kind") in ("quickfix", "quickfix.preferred")
        ]
        quick_fixes = [action.get("title", "") for action in quick_fix_actions]
        caps_fix = next(
            (action for action in quick_fix_actions if "Add `@caps()`" in action.get("title", "")),
            None,
        )
        if caps_fix is None:
            raise AssertionError(f"expected Add @caps() quick fix code action, got: {actions!r}")
        caps_edit = caps_fix["edit"]["changes"][advisory_uri][0]
        if caps_edit["newText"] != "@caps()\n":
            raise AssertionError(f"unexpected @caps quick-fix text: {caps_edit!r}")
        if caps_edit["range"]["start"] != caps_edit["range"]["end"]:
            raise AssertionError(f"@caps quick fix must be a zero-width insertion: {caps_edit!r}")

        write_message(
            proc,
            {
                "jsonrpc": "2.0",
                "id": 9,
                "method": "textDocument/semanticTokens/full",
                "params": {
                    "textDocument": {"uri": clean_uri},
                },
            },
        )
        sem_tokens = wait_for(messages, lambda message: message.get("id") == 9)
        tokens_data = sem_tokens["result"]["data"]
        if not tokens_data:
            raise AssertionError("expected semantic tokens data to not be empty")

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
            "document_symbols": names,
            "workspace_symbols": w_names,
            "rename_edits_count": len(edits),
            "quick_fixes": quick_fixes,
            "semantic_tokens_count": len(tokens_data),
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
