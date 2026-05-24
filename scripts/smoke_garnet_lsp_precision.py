#!/usr/bin/env python3
"""S16 precision smoke for Garnet LSP over stdio.

This is the v0.7 S16 dogfood lane: it proves the server advertises and returns
workspace/document symbols, rowan-token rename edits across open workspace
documents, scoped parameter rename, three code actions, and semantic token data
with the S16 capability/attribute categories.
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
from typing import Any, Callable


ROOT = Path(__file__).resolve().parents[1]


def default_server() -> Path:
    name = "garnet-lsp.exe" if os.name == "nt" else "garnet-lsp"
    return ROOT / "target" / "release" / name


def write_message(proc: subprocess.Popen[bytes], payload: dict[str, Any]) -> None:
    assert proc.stdin is not None
    body = json.dumps(payload, separators=(",", ":")).encode("utf-8")
    proc.stdin.write(f"Content-Length: {len(body)}\r\n\r\n".encode("ascii") + body)
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
            body = proc.stdout.read(int(headers["content-length"]))
            out.put(json.loads(body.decode("utf-8")))
    except BaseException as exc:  # pragma: no cover
        out.put(exc)


def wait_for(
    messages: "queue.Queue[dict[str, Any] | BaseException]",
    predicate: Callable[[dict[str, Any]], bool],
    *,
    timeout: float = 8.0,
) -> dict[str, Any]:
    deadline = time.monotonic() + timeout
    seen: list[dict[str, Any]] = []
    while time.monotonic() < deadline:
        try:
            message = messages.get(timeout=0.25)
        except queue.Empty:
            continue
        if isinstance(message, BaseException):
            raise RuntimeError("LSP reader failed") from message
        seen.append(message)
        if predicate(message):
            return message
    raise TimeoutError(f"timed out waiting for response; seen={seen!r}")


def file_uri(name: str) -> str:
    return (Path(tempfile.gettempdir()) / name).resolve().as_uri()


def request(proc: subprocess.Popen[bytes], messages: "queue.Queue[dict[str, Any] | BaseException]", msg_id: int, method: str, params: Any) -> Any:
    write_message(proc, {"jsonrpc": "2.0", "id": msg_id, "method": method, "params": params})
    response = wait_for(messages, lambda message: message.get("id") == msg_id)
    if "error" in response:
        raise AssertionError(f"{method} returned error: {response['error']!r}")
    return response["result"]


def did_open(proc: subprocess.Popen[bytes], uri: str, text: str, version: int) -> None:
    write_message(
        proc,
        {
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": "garnet",
                    "version": version,
                    "text": text,
                }
            },
        },
    )


def titles(actions: list[dict[str, Any]]) -> list[str]:
    return [action.get("title", "") for action in actions]


def observed_semantic_token_types(data: list[int], legend: list[str]) -> list[str]:
    if len(data) % 5 != 0:
        raise AssertionError(f"semantic token payload length must be a multiple of 5: {len(data)}")
    observed: set[str] = set()
    for index in range(0, len(data), 5):
        token_type_index = data[index + 3]
        try:
            observed.add(legend[token_type_index])
        except IndexError as exc:
            raise AssertionError(
                f"semantic token type index {token_type_index} outside legend {legend!r}"
            ) from exc
    return sorted(observed)


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
        initialize = request(
            proc,
            messages,
            1,
            "initialize",
            {"processId": os.getpid(), "rootUri": ROOT.as_uri(), "capabilities": {}},
        )
        capabilities = initialize["capabilities"]
        semantic_legend = capabilities["semanticTokensProvider"]["legend"]["tokenTypes"]
        for required in ("capability", "attribute", "parameter"):
            if required not in semantic_legend:
                raise AssertionError(f"semantic legend missing {required!r}: {semantic_legend!r}")
        for required in (
            "documentSymbolProvider",
            "workspaceSymbolProvider",
            "renameProvider",
            "codeActionProvider",
            "semanticTokensProvider",
        ):
            if required not in capabilities:
                raise AssertionError(f"initialize capabilities missing {required}")

        write_message(proc, {"jsonrpc": "2.0", "method": "initialized", "params": {}})

        lib_source = "/// Friendly greeting\ndef greet(name) {\n  name\n}\n"
        main_source = "@caps(fs)\ndef main(name) {\n  greet(name)\n}\n"
        lib_uri = file_uri("garnet_lsp_precision_lib.garnet")
        main_uri = file_uri("garnet_lsp_precision_main.garnet")
        did_open(proc, lib_uri, lib_source, 1)
        did_open(proc, main_uri, main_source, 1)
        wait_for(messages, lambda message: message.get("method") == "textDocument/publishDiagnostics" and message.get("params", {}).get("uri") == lib_uri)
        wait_for(messages, lambda message: message.get("method") == "textDocument/publishDiagnostics" and message.get("params", {}).get("uri") == main_uri)

        doc_symbols = request(
            proc,
            messages,
            2,
            "textDocument/documentSymbol",
            {"textDocument": {"uri": lib_uri}},
        )
        doc_names = [symbol["name"] for symbol in doc_symbols]
        if "greet" not in doc_names:
            raise AssertionError(f"missing document symbol greet: {doc_names!r}")

        workspace_symbols = request(proc, messages, 3, "workspace/symbol", {"query": "greet"})
        workspace_names = [symbol["name"] for symbol in workspace_symbols]
        if "greet" not in workspace_names:
            raise AssertionError(f"missing workspace symbol greet: {workspace_names!r}")

        rename = request(
            proc,
            messages,
            4,
            "textDocument/rename",
            {
                "textDocument": {"uri": lib_uri},
                "position": {"line": 1, "character": 5},
                "newName": "welcome",
            },
        )
        rename_changes = rename["changes"]
        if lib_uri not in rename_changes or main_uri not in rename_changes:
            raise AssertionError(f"expected cross-file rename edits, got {rename_changes!r}")

        parameter_rename = request(
            proc,
            messages,
            5,
            "textDocument/rename",
            {
                "textDocument": {"uri": lib_uri},
                "position": {"line": 1, "character": 10},
                "newName": "person",
            },
        )
        parameter_changes = parameter_rename["changes"]
        if list(parameter_changes) != [lib_uri]:
            raise AssertionError(f"parameter rename should stay in declaring file: {parameter_changes!r}")
        if len(parameter_changes[lib_uri]) != 2:
            raise AssertionError(f"expected parameter declaration + use edits: {parameter_changes!r}")

        actions_source = "def build(a, b, c, d) {\n  a\n}\n\n@caps()\ndef answer() {\n  42\n}\n"
        actions_uri = file_uri("garnet_lsp_precision_actions.garnet")
        did_open(proc, actions_uri, actions_source, 2)
        wait_for(messages, lambda message: message.get("method") == "textDocument/publishDiagnostics" and message.get("params", {}).get("uri") == actions_uri)
        actions = request(
            proc,
            messages,
            6,
            "textDocument/codeAction",
            {
                "textDocument": {"uri": actions_uri},
                "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 0}},
                "context": {"diagnostics": []},
            },
        )
        action_titles = titles(actions)
        for expected in ("Add `@caps()`", "Refactor long parameter list", "Add return type `Int`"):
            if not any(expected in title for title in action_titles):
                raise AssertionError(f"missing code action {expected!r}: {action_titles!r}")

        semantic = request(
            proc,
            messages,
            7,
            "textDocument/semanticTokens/full",
            {"textDocument": {"uri": main_uri}},
        )
        if not semantic["data"]:
            raise AssertionError("semantic token response was empty")
        observed_token_types = observed_semantic_token_types(semantic["data"], semantic_legend)
        for required in ("capability", "attribute", "parameter"):
            if required not in observed_token_types:
                raise AssertionError(
                    f"semantic tokens did not emit {required!r}: {observed_token_types!r}"
                )

        write_message(proc, {"jsonrpc": "2.0", "id": 8, "method": "shutdown"})
        wait_for(messages, lambda message: message.get("id") == 8)
        write_message(proc, {"jsonrpc": "2.0", "method": "exit", "params": None})

        return {
            "status": "pass",
            "server": str(executable),
            "document_symbols": doc_names,
            "workspace_symbols": workspace_names,
            "rename_files": sorted(rename_changes),
            "parameter_rename_edits": len(parameter_changes[lib_uri]),
            "code_actions": action_titles,
            "semantic_token_types": semantic_legend,
            "semantic_token_observed_types": observed_token_types,
            "semantic_token_u32_count": len(semantic["data"]),
        }
    finally:
        try:
            proc.wait(timeout=2)
        except subprocess.TimeoutExpired:
            proc.terminate()
            proc.wait(timeout=2)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("executable", nargs="?", default=str(default_server()))
    args = parser.parse_args(sys.argv[1:] if argv is None else argv)
    try:
        print(json.dumps(run_smoke(Path(args.executable)), indent=2))
    except Exception as exc:  # pragma: no cover
        print(f"S16 LSP precision smoke failed: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
