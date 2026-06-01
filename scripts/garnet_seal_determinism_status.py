#!/usr/bin/env python3
"""Seal source-hash determinism status (S82 / WIN-S38-001).

The seal predicate's `source_blake3` hashed raw source bytes, so an LF (Mac/Linux)
checkout and a CRLF (Windows `core.autocrlf`) checkout of the same logical source
produced different predicates. S82 fixes this in two layers and this gate enforces
that they stay in place:

  1. `.gitattributes` pins `*.garnet text eol=lf` (checkouts are LF);
  2. `garnet-cli/src/manifest.rs::Manifest::build` hashes `normalize_source_eol`
     of the source (LF-normalized; idempotent on LF — existing seals unchanged);
  3. the canonicalization contract is documented in `GARNET_ATTESTATION.md`.

## Honest scope (do not soften)
Only line endings are canonicalized; other whitespace still changes the source
hash by design. Mac-authored + Mac-unit-tested (LF↔CRLF same `source_hash`); the
end-to-end Windows proof (fresh Windows checkout → matching `source_blake3`) is
recorded in `WINDOWS_AUDIT_S1_S80.md` as Windows-proof-pending.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GITATTRIBUTES = ROOT / ".gitattributes"
MANIFEST = ROOT / "garnet-cli" / "src" / "manifest.rs"
DOC = ROOT / "C_Language_Specification" / "GARNET_ATTESTATION.md"


@dataclass
class SealDeterminismStatus:
    schema: str
    gitattributes_pins_garnet_lf: bool
    manifest_normalizes_source_eol: bool
    contract_documented: bool
    ok: bool = False


def _read(p: Path) -> str:
    return p.read_text(encoding="utf-8") if p.is_file() else ""


def read_status() -> SealDeterminismStatus:
    ga = _read(GITATTRIBUTES)
    manifest = _read(MANIFEST)
    doc = _read(DOC)
    pin = "*.garnet text eol=lf" in ga
    normalizes = (
        "normalize_source_eol" in manifest
        and 'replace("\\r\\n", "\\n")' in manifest
        and "hash_str(&normalize_source_eol(source))" in manifest
    )
    documented = "canonicalization contract" in doc and "source_blake3" in doc
    ok = pin and normalizes and documented
    return SealDeterminismStatus(
        schema="garnet.seal_determinism/v1",
        gitattributes_pins_garnet_lf=pin,
        manifest_normalizes_source_eol=normalizes,
        contract_documented=documented,
        ok=ok,
    )


def render_markdown(r: SealDeterminismStatus) -> str:
    return "\n".join([
        "# Garnet seal source-hash determinism status (S82)",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- `.gitattributes` pins `*.garnet text eol=lf`: "
        f"{'yes' if r.gitattributes_pins_garnet_lf else 'NO'}",
        f"- `Manifest::build` hashes LF-normalized source: "
        f"{'yes' if r.manifest_normalizes_source_eol else 'NO'}",
        f"- canonicalization contract documented: "
        f"{'yes' if r.contract_documented else 'NO'}",
        "",
        "Two-layer fix for WIN-S38-001: the seal `source_blake3` is LF/CRLF-stable "
        "(in-code normalization, idempotent on LF) + `.gitattributes` pin. "
        "End-to-end Windows proof is Windows-proof-pending.",
        "",
    ])


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless the .gitattributes pin + the in-code EOL "
        "normalization + the documented contract are all present.",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status()
    print(render_markdown(r) if args.format == "md" else json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(f"seal-determinism gate FAILED: {asdict(r)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
