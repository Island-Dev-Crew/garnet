#!/usr/bin/env python3
"""WASM hello-world readiness reporter (S55).

The browser story (run Garnet in-browser via WebAssembly) is a major adoption
driver and the enabler for the S56 playground. This reporter inventories that
path honestly: what is owned and ready vs. what is blocked, naming the concrete
blockers rather than claiming a wasm build that does not exist.

## Environment reality → honest-partial (do not soften)
- The `wasm32` Rust target is **not installed**; `wasm-pack`/`wasmtime` are
  **absent** — no wasm artifact can be built or run here.
- `garnet-interp` depends on `miette` with the **`fancy`** feature (terminal /
  backtrace machinery), a concrete wasm-portability blocker that must be
  feature-gated off for a wasm build.
- Garnet has **no wasm backend**; the interpreter (compiled to wasm) is the
  in-browser execution path, not a Garnet→wasm compiler.

This reporter does **not** build wasm or claim a browser run. It checks the owned
bits (the hello-world example + the target doc) and reports the blockers.
"""
from __future__ import annotations

import argparse
import json
import shutil
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
HELLO = ROOT / "examples" / "hello.garnet"
TARGET_DOC = ROOT / "F_Project_Management" / "GARNET_WASM_TARGET.md"
INTERP_CARGO = ROOT / "garnet-interp-v0.3" / "Cargo.toml"


@dataclass
class WasmReadiness:
    schema: str
    hello_example_present: bool
    target_doc_present: bool
    wasm32_target_installed: bool
    wasm_pack_present: bool
    wasmtime_present: bool
    miette_fancy_blocker: bool
    blockers: list[str] = field(default_factory=list)
    owned_bits_ready: bool = False


def _has_wasm32_target() -> bool:
    rustup = shutil.which("rustup")
    if rustup is None:
        return False
    import subprocess

    proc = subprocess.run(
        [rustup, "target", "list", "--installed"], capture_output=True, text=True
    )
    return "wasm32" in proc.stdout


def read_readiness() -> WasmReadiness:
    interp_cargo = INTERP_CARGO.read_text(encoding="utf-8") if INTERP_CARGO.is_file() else ""
    miette_fancy = 'miette' in interp_cargo and '"fancy"' in interp_cargo

    blockers: list[str] = []
    if not _has_wasm32_target():
        blockers.append("rustup wasm32 target not installed (`rustup target add wasm32-unknown-unknown`)")
    if shutil.which("wasm-pack") is None:
        blockers.append("wasm-pack absent (needed to bundle the interp for the browser)")
    if shutil.which("wasmtime") is None:
        blockers.append("wasmtime absent (needed for a non-browser wasm run)")
    if miette_fancy:
        blockers.append("garnet-interp pulls miette `fancy` (terminal/backtrace) — feature-gate it off for wasm")

    return WasmReadiness(
        schema="garnet.wasm_readiness/v1",
        hello_example_present=HELLO.is_file(),
        target_doc_present=TARGET_DOC.is_file(),
        wasm32_target_installed=_has_wasm32_target(),
        wasm_pack_present=shutil.which("wasm-pack") is not None,
        wasmtime_present=shutil.which("wasmtime") is not None,
        miette_fancy_blocker=miette_fancy,
        blockers=blockers,
        owned_bits_ready=HELLO.is_file() and TARGET_DOC.is_file(),
    )


def render_markdown(r: WasmReadiness) -> str:
    lines = [
        "# Garnet WASM hello-world readiness",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- hello-world example present: {r.hello_example_present}",
        f"- wasm target doc present: {r.target_doc_present}",
        f"- wasm32 target installed: {r.wasm32_target_installed}",
        f"- wasm-pack present: {r.wasm_pack_present}",
        f"- wasmtime present: {r.wasmtime_present}",
        f"- miette `fancy` portability blocker: {r.miette_fancy_blocker}",
        "",
        f"**Owned bits ready (example + doc): {'yes' if r.owned_bits_ready else 'NO'}.**",
        "",
        "## Blockers (why wasm is not built/run here)",
    ]
    for b in r.blockers:
        lines.append(f"- {b}")
    lines += [
        "",
        "Honest scope: no wasm is built and no browser run is claimed. Garnet has "
        "no wasm backend; the interpreter compiled to wasm is the in-browser path, "
        "and that build is DEFERRED until the blockers above are resolved.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero only if the OWNED bits (hello example + target doc) are missing",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_readiness()
    if args.format == "md":
        print(render_markdown(r))
    else:
        print(json.dumps(asdict(r), indent=2))

    if args.gate and not r.owned_bits_ready:
        print(
            "wasm-readiness gate FAILED: the owned bits are missing "
            f"(hello={r.hello_example_present}, doc={r.target_doc_present}). "
            "(The absent wasm toolchain is NOT gated — it is an honest deferral.)",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
