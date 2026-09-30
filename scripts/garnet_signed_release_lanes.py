#!/usr/bin/env python3
"""Signed release lanes — inventory + active-lane gate (S51).

Garnet's "signed release" posture spans three distinct lanes. This reporter makes
each one's status explicit and falsifiable, and gates the one lane that is
actually ACTIVE so it cannot silently regress:

1. **Program-manifest signing** — `garnet build --sign <key>` produces an Ed25519
   signature over the deterministic build manifest, verified to "signature valid"
   in `linux-packages.yml`. This lane is **ACTIVE** and gated here.
2. **Release-artifact signing** — the tagged release job signs `SHA256SUMS` with
   `gpg --detach-sign` and uploads `SHA256SUMS.asc`; an unsigned tagged release
   fails closed unless it is a recorded, deliberate act. **ACTIVE** since v0.8.1
   and gated here. The key lives in CI; Garnet does not bundle GPG.
3. **Supply-chain attestation** — `garnet seal [--out]` emits an in-toto predicate
   over the build + capability manifests, meant for `cosign attest --predicate`.
   **PARTIAL**: Garnet produces (and now writes) the predicate; `cosign` is
   detected, never bundled — supply-chain *signing* is external.

## Scope (do not soften)
Garnet does **not** sign its own supply chain and does **not** bundle
cosign/GPG/minisign. Lane 3 is partial by design; its status is reported, not
faked. Lanes 1 and 2 are gated: lane 1 is in-language manifest signing, which
Garnet fully owns; lane 2 is the release pipeline's own GPG signature.
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
    status: str  # active | broken | partial
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


# Every piece must be present in linux-packages.yml for release-artifact signing to
# count as wired: the detached signature, its upload, and the unsigned-release guard.
RELEASE_ARTIFACT_EVIDENCE = (
    "--detach-sign --armor SHA256SUMS",
    "files: release-dist/SHA256SUMS.asc",
    "name: Require signed SHA256SUMS (fail-closed)",
)


def release_artifact_lane(pkg: str) -> Lane:
    present = all(piece in pkg for piece in RELEASE_ARTIFACT_EVIDENCE)
    return Lane(
        id="release-artifact",
        name="Release-artifact signing (SHA256SUMS detached signature)",
        status="active" if present else "broken",
        owned_by_garnet=False,
        evidence="linux-packages.yml: gpg --detach-sign SHA256SUMS → SHA256SUMS.asc uploaded; "
        "unsigned tagged release fails closed (key held in the release environment, GPG not bundled)",
        present=present,
    )


def read_lanes() -> SignedReleaseLanes:
    pkg = _read(".github/workflows/linux-packages.yml")
    seal = _read("garnet-cli/src/cmd/seal.rs")

    lane1_present = "--sign" in pkg and "signature valid" in pkg
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
        release_artifact_lane(pkg),
        Lane(
            id="supply-chain-attestation",
            name="Supply-chain attestation (garnet seal → cosign attest)",
            status="partial",
            owned_by_garnet=False,
            evidence="cmd/seal.rs: in-toto predicate emitted (now --out writable); cosign detected, not bundled",
            present=lane3_present,
        ),
    ]
    active_lane_ok = all(l.present for l in lanes if l.id in ("program-manifest", "release-artifact"))
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
        mark = {"active": "✅ active", "broken": "✗ broken", "partial": "◐ partial"}.get(
            l.status, l.status
        )
        owned = "yes" if l.owned_by_garnet else "no (external tool)"
        lines.append(f"| {l.name} | {mark} | {owned} | {l.evidence} |")
    lines += [
        "",
        f"**Active lanes (program-manifest and release-artifact signing) wired: "
        f"{'yes' if s.active_lane_ok else 'NO'}.**",
        "",
        "Scope: Garnet does not sign its own supply chain or bundle "
        "cosign/GPG/minisign. Lane 3 (supply-chain) is partial by design — an "
        "external signing tool. Lanes 1 (program-manifest) and 2 (release-artifact, "
        "the release job's GPG signature) are gated.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero if an ACTIVE lane (program-manifest or release-artifact signing) is not wired",
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
            "(`garnet build --sign` → 'signature valid') or release-artifact signing "
            "(gpg --detach-sign SHA256SUMS → SHA256SUMS.asc, fail-closed) is no longer wired in CI",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
