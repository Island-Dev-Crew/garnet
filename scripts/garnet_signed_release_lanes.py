#!/usr/bin/env python3
"""Signed release lanes — inventory + active-lane gate (S51).

Garnet's "signed release" posture spans three distinct lanes. This reporter makes
each one's status explicit and falsifiable, and gates the one lane that is
actually ACTIVE so it cannot silently regress:

1. **Program-manifest signing** — `garnet build --sign <key>` produces an Ed25519
   signature over the deterministic build manifest, verified to "signature valid"
   in `linux-packages.yml`. This lane is **ACTIVE** and gated here.
2. **Release-artifact signing** — the `SHA256SUMS` over release assets is computed
   but NOT signed (a documented `TODO(release-security)` in `linux-packages.yml`).
   **DEFERRED**: needs a GPG/minisign key in CI.
3. **Supply-chain attestation** — `garnet seal [--out]` emits an in-toto predicate
   over the build + capability manifests, meant for `cosign attest --predicate`.
   **PARTIAL**: Garnet produces (and now writes) the predicate; `cosign` is
   detected, never bundled — supply-chain *signing* is external.

## Honest scope (do not soften)
Garnet does **not** sign its own supply chain and does **not** bundle
cosign/GPG/minisign. Lanes 2 and 3 are deferred/partial by design; their status
is reported, not faked. Only lane 1 (in-language manifest signing, which Garnet
fully owns) is gated.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


@dataclass
class Lane:
    id: str
    name: str
    status: str  # active | deferred | partial
    owned_by_garnet: bool
    evidence: str
    present: bool


@dataclass
class SignedReleaseLanes:
    schema: str
    lanes: list[Lane]
    active_lane_ok: bool


def _read(rel: str) -> str:
    p = ROOT / rel
    return p.read_text(encoding="utf-8") if p.is_file() else ""


def read_lanes() -> SignedReleaseLanes:
    pkg = _read(".github/workflows/linux-packages.yml")
    seal = _read("garnet-cli/src/cmd/seal.rs")

    lane1_present = "--sign" in pkg and "signature valid" in pkg
    lane2_deferred_ack = "TODO(release-security)" in pkg and "SHA256SUMS" in pkg
    lane3_present = "garnet seal" in seal and "--out" in seal and "cosign" in seal

    lanes = [
        Lane(
            id="program-manifest",
            name="Program-manifest signing (garnet build --sign)",
            status="active" if lane1_present else "broken",
            owned_by_garnet=True,
            evidence="linux-packages.yml: --sign round-trip → 'signature valid'",
            present=lane1_present,
        ),
        Lane(
            id="release-artifact",
            name="Release-artifact signing (SHA256SUMS detached signature)",
            status="deferred",
            owned_by_garnet=False,
            evidence="linux-packages.yml: TODO(release-security) — computed, not signed (needs GPG/minisign key)",
            present=lane2_deferred_ack,
        ),
        Lane(
            id="supply-chain-attestation",
            name="Supply-chain attestation (garnet seal → cosign attest)",
            status="partial",
            owned_by_garnet=False,
            evidence="cmd/seal.rs: in-toto predicate emitted (now --out writable); cosign detected, not bundled",
            present=lane3_present,
        ),
    ]
    active_lane_ok = next(l.present for l in lanes if l.id == "program-manifest")
    return SignedReleaseLanes(
        schema="garnet.signed_release_lanes/v1",
        lanes=lanes,
        active_lane_ok=active_lane_ok,
    )


def render_markdown(s: SignedReleaseLanes) -> str:
    lines = [
        "# Garnet signed release lanes",
        "",
        f"_Schema {s.schema}._",
        "",
        "| lane | status | Garnet-owned | evidence |",
        "|---|---|---|---|",
    ]
    for l in s.lanes:
        mark = {"active": "✅ active", "deferred": "⏸ deferred", "partial": "◐ partial"}.get(
            l.status, l.status
        )
        owned = "yes" if l.owned_by_garnet else "no (external tool)"
        lines.append(f"| {l.name} | {mark} | {owned} | {l.evidence} |")
    lines += [
        "",
        f"**Active lane (program-manifest signing) wired: "
        f"{'yes' if s.active_lane_ok else 'NO'}.**",
        "",
        "Honest scope: Garnet does not sign its own supply chain or bundle "
        "cosign/GPG/minisign. Lanes 2 (release-artifact) and 3 (supply-chain) are "
        "deferred/partial by design — external signing tools. Only lane 1, which "
        "Garnet fully owns, is gated.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero if the ACTIVE lane (program-manifest signing) is not wired",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    status = read_lanes()
    if args.format == "md":
        print(render_markdown(status))
    else:
        print(json.dumps(asdict(status), indent=2))

    if args.gate and not status.active_lane_ok:
        print(
            "signed-release-lanes gate FAILED: program-manifest signing "
            "(`garnet build --sign` → 'signature valid') is no longer wired in CI",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
