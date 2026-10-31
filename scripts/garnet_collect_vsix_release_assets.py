#!/usr/bin/env python3
"""Collect the VS Code extension assets for one release tag (C6-06).

The build-vsix matrix uploads one VSIX per runner. A restored cache or an old
artifact can carry VSIX files from another version, so this copies only the
files named for the tag's version, requires editors/vscode/package.json to
carry that version, and requires exactly one file per matrix runner.
"""
from __future__ import annotations

import argparse
import json
import re
import shutil
import sys
from pathlib import Path

TAG_RE = re.compile(r"v(\d+\.\d+\.\d+)")


def fail(message: str) -> None:
    print(f"collect-vsix: error: {message}", file=sys.stderr)
    raise SystemExit(1)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--tag", required=True, help="release tag, vMAJOR.MINOR.PATCH")
    parser.add_argument("--from", dest="source", required=True, type=Path)
    parser.add_argument("--to", dest="dest", required=True, type=Path)
    parser.add_argument("--package-json", type=Path, default=Path("editors/vscode/package.json"))
    parser.add_argument("--expect", required=True, type=int, help="number of build-vsix runners")
    args = parser.parse_args(argv)

    match = TAG_RE.fullmatch(args.tag)
    if match is None:
        fail(f"release tag {args.tag!r} is not vMAJOR.MINOR.PATCH")
    version = match.group(1)
    declared = json.loads(args.package_json.read_text(encoding="utf-8")).get("version")
    if declared != version:
        fail(f"{args.package_json} version {declared!r} does not match release tag {args.tag}")

    asset_re = re.compile(
        rf"garnet-{re.escape(version)}-lsp-mvp-(?:darwin|linux|win32)-(?:x64|arm64)\.vsix"
    )
    found = sorted(p for p in args.source.rglob("*.vsix") if p.is_file())
    chosen = [p for p in found if asset_re.fullmatch(p.name)]
    for path in found:
        if path not in chosen:
            print(f"collect-vsix: skipping {path.name}: not a {args.tag} asset")
    names = [p.name for p in chosen]
    if len(set(names)) != len(names):
        fail(f"the same {args.tag} VSIX appears more than once: {names}")
    if len(chosen) != args.expect:
        fail(f"expected {args.expect} {args.tag} VSIX files, found {len(chosen)}: {names}")

    args.dest.mkdir(parents=True, exist_ok=True)
    for path in chosen:
        shutil.copy2(path, args.dest / path.name)
        print(f"collect-vsix: {path.name}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
