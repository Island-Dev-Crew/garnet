#!/usr/bin/env python3
"""Build the static playground example gallery manifest (S56).

Generates `docs/playground/examples.json` from a curated example list by reading
each program's source and recording its real `garnet run` output. The static
playground page (`docs/playground.html`) renders this manifest — a browsable
gallery of real Garnet programs and their recorded outputs.

This is a build step, run locally (it needs the garnet binary), not in CI. The
committed `examples.json` is the artifact; `scripts/garnet_playground_readiness.py`
validates it. Regenerate with: `python3 scripts/garnet_playground_build.py`.
"""
from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs" / "playground" / "examples.json"

# Curated gallery: (example file stem, title, one-line description).
GALLERY = [
    ("hello", "Hello, Garnet", "The canonical hello-world (`@caps()`, pure compute + stdout)."),
    ("documented_math", "Docs-as-tests", "Documented functions whose `///` examples `garnet doctest` runs (S43)."),
    ("mvp_05_web_app", "Web route dispatch", "Route-dispatch scoring — one of the 12 proof-matrix domains (S48)."),
    (
        "undeclared_memory_tier",
        "Undeclared memory tier",
        "`@caps()` reaches `memory::working`; `check` reports the missing `mem` and `run` traps before the store exists (D-04).",
    ),
]

# `garnet run` prints its own diagnostics to stderr beside cache notes
# (`note: this source has N prior failure(s) ...`) that depend on the
# builder's local `.garnet-cache`. Only the diagnostics are part of the
# recorded output.
STDERR_NOISE_PREFIX = "note:"


def recorded_output(proc: subprocess.CompletedProcess[str]) -> str:
    """The gallery shows what actually happened, on either exit path.

    A program that completes records its stdout (with the `=> <value>`
    trailer). A program the runtime stops records its partial stdout, the
    runtime diagnostic lines, and the exit code, so a preset whose point is
    the trap shows the trap rather than an empty string.
    """
    stdout = proc.stdout.strip()
    if proc.returncode == 0:
        return stdout
    diagnostics = [
        line
        for line in proc.stderr.splitlines()
        if line.strip() and not line.startswith(STDERR_NOISE_PREFIX)
    ]
    lines = ([stdout] if stdout else []) + diagnostics + [f"exit {proc.returncode}"]
    return "\n".join(lines)


def resolve_garnet() -> list[str]:
    env = os.environ.get("GARNET_CLI")
    if env:
        return [env]
    exe = "garnet.exe" if os.name == "nt" else "garnet"
    cands = [ROOT / "target" / p / exe for p in ("release", "debug")]
    cands = [c for c in cands if c.exists()]
    if cands:
        return [str(max(cands, key=lambda p: p.stat().st_mtime))]
    found = shutil.which("garnet")
    if found:
        return [found]
    raise FileNotFoundError("garnet CLI not found; build with `cargo build -p garnet-cli`")


def main() -> int:
    garnet = resolve_garnet()
    entries = []
    for stem, title, desc in GALLERY:
        src_path = ROOT / "examples" / f"{stem}.garnet"
        source = src_path.read_text(encoding="utf-8")
        proc = subprocess.run(
            [*garnet, "run", str(src_path)], cwd=ROOT, capture_output=True, text=True
        )
        output = recorded_output(proc)
        entries.append(
            {"name": stem, "title": title, "description": desc, "source": source, "output": output}
        )
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps({"schema": "garnet.playground/v1", "examples": entries}, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {OUT} ({len(entries)} examples)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
