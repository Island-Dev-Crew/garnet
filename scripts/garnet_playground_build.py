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
]


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
        # stdout is the program's output (incl. the `=> <value>` trailer);
        # stderr carries CLI record/strategy notes we don't want in the gallery.
        output = proc.stdout.strip()
        entries.append(
            {"name": stem, "title": title, "description": desc, "source": source, "output": output}
        )
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps({"schema": "garnet.playground/v1", "examples": entries}, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {OUT} ({len(entries)} examples)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
