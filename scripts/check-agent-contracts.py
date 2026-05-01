#!/usr/bin/env python3
"""Validate Garnet's AGENTS.md documentation-runtime contract map."""
from __future__ import annotations

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
REQUIRED = [
    "AGENTS.md",
    "C_Language_Specification/AGENTS.md",
    "F_Project_Management/AGENTS.md",
    "garnet-parser-v0.3/AGENTS.md",
    "garnet-interp-v0.3/AGENTS.md",
    "garnet-check-v0.3/AGENTS.md",
    "garnet-memory-v0.3/AGENTS.md",
    "garnet-actor-runtime/AGENTS.md",
    "garnet-stdlib/AGENTS.md",
    "garnet-cli/AGENTS.md",
    "garnet-cli/templates/AGENTS.md",
    "garnet-convert/AGENTS.md",
    "examples/AGENTS.md",
    "xtask/AGENTS.md",
]


def fail(msg: str) -> None:
    print(f"agent-contracts: {msg}", file=sys.stderr)
    raise SystemExit(1)


def main() -> int:
    root_doc = ROOT / "AGENTS.md"
    if not root_doc.exists():
        fail("missing root AGENTS.md")
    root_text = root_doc.read_text(encoding="utf-8")

    seen = set()
    for rel in REQUIRED:
        path = ROOT / rel
        if not path.exists():
            fail(f"missing required contract {rel}")
        text = path.read_text(encoding="utf-8")
        if not text.strip().startswith("# AGENTS.md"):
            fail(f"{rel} must start with an AGENTS.md H1")
        if "## Scope" not in text and rel != "AGENTS.md":
            fail(f"{rel} must include a Scope section")
        indexed = f"/{rel}" if rel != "AGENTS.md" else "/AGENTS.md"
        if indexed not in root_text:
            fail(f"root AGENTS.md index omits {indexed}")
        if rel in seen:
            fail(f"duplicate required contract {rel}")
        seen.add(rel)

    actual = sorted(
        str(path.relative_to(ROOT))
        for path in ROOT.rglob("AGENTS.md")
        if ".git" not in path.parts and "target" not in path.parts
    )
    extra = [p for p in actual if p not in REQUIRED]
    if extra:
        fail("new AGENTS.md files must be added to REQUIRED and root index: " + ", ".join(extra))

    # Catch malformed absolute index links like //foo or missing leading slash.
    index_lines = [line.strip() for line in root_text.splitlines() if line.strip().startswith("- `/")]
    indexed_paths = []
    for line in index_lines:
        match = re.search(r"`/([^`]+)`", line)
        if match:
            indexed_paths.append(match.group(1))
    missing_from_required = sorted(set(indexed_paths) - set(REQUIRED))
    if missing_from_required:
        fail("root index contains paths not in REQUIRED: " + ", ".join(missing_from_required))

    print(f"agent-contracts: ok ({len(REQUIRED)} contracts)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
