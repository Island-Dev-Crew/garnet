#!/usr/bin/env python3
"""Playground MVP readiness + honesty gate (S56).

The playground is a **static gallery** (real Garnet programs + their recorded
`garnet run` output), not a live editor. The Wasm build + Node execution lane is
proven; live in-browser execution still waits on the W-PLAY adapter, package,
and Playwright proof. This reporter checks the gallery is well-formed AND that
the page keeps its honest stance (it must not silently become a fake-editor
claim).

## Honest scope (do not soften)
The playground does not execute code in the browser. This gate guards the static
gallery's structure and that the page still says so; it does not claim live
execution.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PAGE = ROOT / "docs" / "playground.html"
MANIFEST = ROOT / "docs" / "playground" / "examples.json"

# The page must keep these honesty markers (calibrated-expectations anchor).
HONESTY_MARKERS = ["static", "not shipping a fake editor", "WebAssembly"]


@dataclass
class PlaygroundReadiness:
    schema: str
    page_present: bool
    manifest_present: bool
    example_count: int
    examples_well_formed: bool
    page_references_manifest: bool
    honesty_markers_present: bool
    missing_markers: list[str] = field(default_factory=list)
    ok: bool = False


def read_readiness() -> PlaygroundReadiness:
    page = PAGE.read_text(encoding="utf-8") if PAGE.is_file() else ""
    examples = []
    well_formed = False
    if MANIFEST.is_file():
        try:
            data = json.loads(MANIFEST.read_text(encoding="utf-8"))
            examples = data.get("examples", [])
            well_formed = bool(examples) and all(
                all(k in e and e[k] for k in ("name", "title", "source", "output"))
                for e in examples
            )
        except json.JSONDecodeError:
            well_formed = False

    # Normalize whitespace so a marker phrase split across HTML source lines
    # still matches.
    page_norm = re.sub(r"\s+", " ", page).lower()
    missing_markers = [m for m in HONESTY_MARKERS if m.lower() not in page_norm]
    references_manifest = "playground/examples.json" in page
    honesty_ok = not missing_markers

    return PlaygroundReadiness(
        schema="garnet.playground_readiness/v1",
        page_present=PAGE.is_file(),
        manifest_present=MANIFEST.is_file(),
        example_count=len(examples),
        examples_well_formed=well_formed,
        page_references_manifest=references_manifest,
        honesty_markers_present=honesty_ok,
        missing_markers=missing_markers,
        ok=(
            PAGE.is_file()
            and well_formed
            and references_manifest
            and honesty_ok
        ),
    )


def render_markdown(r: PlaygroundReadiness) -> str:
    return "\n".join(
        [
            "# Garnet playground (static preview) readiness",
            "",
            f"_Schema {r.schema}._",
            "",
            f"- page present: {r.page_present}",
            f"- manifest present: {r.manifest_present} ({r.example_count} examples)",
            f"- examples well-formed: {r.examples_well_formed}",
            f"- page references manifest: {r.page_references_manifest}",
            f"- honesty markers present: {r.honesty_markers_present} "
            f"(missing: {r.missing_markers or 'none'})",
            "",
            f"**Playground gallery OK: {'yes' if r.ok else 'NO'}.**",
            "",
            "Honest scope: a static gallery (recorded outputs), not a live editor; "
            "the Wasm build + Node lane is proven, while browser execution waits "
            "on the W-PLAY adapter/package/Playwright proof.",
            "",
        ]
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero if the gallery is malformed or the page loses its honesty markers",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_readiness()
    if args.format == "md":
        print(render_markdown(r))
    else:
        print(json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(
            "playground-readiness gate FAILED: "
            f"well_formed={r.examples_well_formed}, refs_manifest={r.page_references_manifest}, "
            f"missing_honesty_markers={r.missing_markers}",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
