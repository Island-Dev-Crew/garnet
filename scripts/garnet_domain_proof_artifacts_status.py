#!/usr/bin/env python3
"""Use-case-domain proof-artifacts gate (S116).

The six demonstrator domains (S105) are rendered as proof artifacts in
`F_Project_Management/GARNET_DOMAIN_PROOF_ARTIFACTS.md`. This gate keeps that doc
HONEST and CONSISTENT with the recorded Mac-native execution floor
(`proofs/mac/domains/*/garnet-mac-domain-proofs.json`) — it does not re-run the
domains (the floor already did, 6/6 passed); it asserts the doc faithfully reflects
the floor and never over-claims.

It checks, sourcing truth from the JSON, that:
  - the floor itself passed (status=passed, passed==count==6, no live provider);
  - every recorded domain label appears in the doc;
  - EXACTLY the domain whose recorded artifacts include `seal.json` +
    `transparency_log.jsonl` (the accept path) is presented as sealed, and the doc
    says "no seal" for the refusal/report domains;
  - the MCP domain is marked enforced=false in BOTH the JSON and the doc;
  - the honest fences (not seccomp / not OS-sandbox / not Wasmtime fuel / not
    production-v1.0 / simulated agent) are present in the doc.

## Honest scope (do not soften)
This is the macOS-native row only — not Windows/Linux completion (S109), not seccomp,
not OS-sandbox on macOS, not Wasmtime fuel, not an MCP-host runtime budget, not a
live-LLM agent, not production/v1.0. The doc must keep those fences.
"""
from __future__ import annotations

import argparse
import glob
import json
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "F_Project_Management" / "GARNET_DOMAIN_PROOF_ARTIFACTS.md"
PROOF_GLOB = "proofs/mac/domains/*/garnet-mac-domain-proofs.json"

SEAL_ARTIFACTS = {"seal.json", "transparency_log.jsonl"}
# Plain-substring fences (lowercased) that must appear in the doc.
FENCE_SUBSTRINGS = (
    "not seccomp",
    "os-sandbox",
    "wasmtime fuel",
    "simulated",
    "no production / 1.0",
)


@dataclass
class DomainStatus:
    schema: str
    doc_present: bool
    proof_present: bool
    floor_passed: bool
    domain_count: int
    labels_in_doc: int
    sealed_domain_correct: bool
    refusals_unsealed_in_doc: bool
    mcp_enforced_false: bool
    fences_present: bool
    ok: bool = False


def _read(p: Path) -> str:
    return p.read_text(encoding="utf-8") if p.is_file() else ""


def _norm(text: str) -> str:
    """Lowercase, drop markdown emphasis, and collapse whitespace — so a substring
    check is robust to `**bold**` and to prose that line-wraps between words."""
    return re.sub(r"\s+", " ", text.replace("*", "").lower())


def _proof_path() -> Path | None:
    hits = sorted(glob.glob(str(ROOT / PROOF_GLOB)))
    return Path(hits[-1]) if hits else None


def read_status() -> DomainStatus:
    doc = _read(DOC)
    low = _norm(doc)
    pp = _proof_path()
    proof = json.loads(pp.read_text(encoding="utf-8")) if pp else {}

    domains = proof.get("domains", [])
    count = len(domains)
    floor_passed = (
        proof.get("status") == "passed"
        and proof.get("passed_domains") == count
        and proof.get("failed_domains") == 0
        and proof.get("provider_api_called") is False
        and count == 6
    )

    # Every recorded label must appear in the doc.
    labels = [d.get("label", "") for d in domains]
    labels_in_doc = sum(1 for lab in labels if lab and lab.lower() in low)

    # The sealed domain is the one whose recorded artifacts include the seal pair.
    sealed_ids = [
        d.get("id")
        for d in domains
        if SEAL_ARTIFACTS.issubset(set(d.get("artifacts", [])))
    ]
    sealed_correct = (
        len(sealed_ids) == 1
        and sealed_ids[0] == "accept_provenance_dossier"
        # the doc must present exactly that domain as the sealed/accept one
        and "the only sealed domain" in low
        and "seal.json" in doc
    )

    # The doc must say "no seal" for the refusal/report domains (>=3 occurrences).
    refusals_unsealed = low.count("no seal") >= 3

    # MCP domain: enforced=false in the JSON AND surfaced honestly in the doc.
    mcp = next((d for d in domains if d.get("id") == "mcp_tool_authority_creep"), {})
    mcp_false = mcp.get("enforced") is False and "enforced=false" in low

    fences = all(f in low for f in FENCE_SUBSTRINGS)

    ok = (
        bool(doc)
        and pp is not None
        and floor_passed
        and labels_in_doc == count == 6
        and sealed_correct
        and refusals_unsealed
        and mcp_false
        and fences
    )
    return DomainStatus(
        schema="garnet.domain_proof_artifacts/v1",
        doc_present=bool(doc),
        proof_present=pp is not None,
        floor_passed=floor_passed,
        domain_count=count,
        labels_in_doc=labels_in_doc,
        sealed_domain_correct=sealed_correct,
        refusals_unsealed_in_doc=refusals_unsealed,
        mcp_enforced_false=mcp_false,
        fences_present=fences,
        ok=ok,
    )


def render_markdown(r: DomainStatus) -> str:
    return "\n".join(
        [
            "# Garnet domain proof-artifacts status (S116)",
            "",
            f"_Schema {r.schema}._",
            "",
            f"- doc present: {'yes' if r.doc_present else 'NO'}",
            f"- Mac proof floor present: {'yes' if r.proof_present else 'NO'}",
            f"- floor passed (6/6, no live provider): "
            f"{'yes' if r.floor_passed else 'NO'}",
            f"- domain labels in doc: {r.labels_in_doc}/{r.domain_count}",
            f"- sealed domain correct (only accept_provenance_dossier seals): "
            f"{'yes' if r.sealed_domain_correct else 'NO'}",
            f"- refusal/report domains shown unsealed: "
            f"{'yes' if r.refusals_unsealed_in_doc else 'NO'}",
            f"- MCP domain enforced=false (JSON + doc): "
            f"{'yes' if r.mcp_enforced_false else 'NO'}",
            f"- honest fences present: {'yes' if r.fences_present else 'NO'}",
            "",
            "Six domains rendered as Mac-native proof artifacts: accept seals (4 trust "
            "artifacts), the four refusal/report domains do not, the MCP lens is a "
            "static report (enforced=false). macOS-native row only — not Windows/Linux "
            "completion, not seccomp/OS-sandbox/Wasmtime fuel, not production / 1.0.",
            "",
        ]
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless the doc faithfully reflects the recorded Mac "
        "domain floor (all 6 labels, only the accept domain sealed, refusals "
        "unsealed, MCP enforced=false, honest fences).",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status()
    print(render_markdown(r) if args.format == "md" else json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(f"domain proof-artifacts gate FAILED: {asdict(r)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
