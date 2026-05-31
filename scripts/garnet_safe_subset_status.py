#!/usr/bin/env python3
"""Safe-subset spec status (S74).

Garnet is dual-mode: managed `def` (FnMode::Managed) and safe `fn`
(FnMode::Safe). The **safe subset** is the safe mode. S74 specifies it:
`C_Language_Specification/GARNET_SAFE_SUBSET.md` documents (1) the safe subset
*as implemented today* — the typed, ownership-disciplined `fn` mode plus the
fn↔def boundary audit that closes the "hidden safe→managed escalation" threat
class — and (2) a *proposed* optional linear/effect-typed rigor mode (Austral
linear capabilities / Koka effects) for high-assurance components.

This reporter is a static anti-overclaim gate: it verifies the spec exists, that
the spec's "implemented today" claims are grounded in real source (`FnMode::Safe`
in the AST; the boundary audit in the checker), and that the proposed
linear/effect mode is honestly marked NOT IMPLEMENTED.

## Honest scope (do not soften)
This slice SPECIFIES; it does not implement a linear type system, effect rows, or
any soundness proof. §2 of the spec is a proposal, not shipped behavior.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = ROOT / "C_Language_Specification" / "GARNET_SAFE_SUBSET.md"
AST = ROOT / "garnet-parser-v0.3" / "src" / "ast.rs"
AUDIT = ROOT / "garnet-check-v0.3" / "src" / "audit.rs"

# Spec must state both the implemented baseline and the honest "not implemented".
SPEC_ANCHORS = [
    "safe subset today",
    "hidden safe→managed escalation",
    "NOT IMPLEMENTED",
    "linear capabilities",
]


@dataclass
class SafeSubsetStatus:
    schema: str
    spec_present: bool
    missing_spec_anchors: list[str]
    fnmode_safe_in_ast: bool
    boundary_audit_in_checker: bool
    ok: bool = False
    groundings: list[str] = field(default_factory=list)


def _read(p: Path) -> str:
    return p.read_text(encoding="utf-8") if p.is_file() else ""


def read_status() -> SafeSubsetStatus:
    spec = _read(SPEC)
    ast = _read(AST)
    audit = _read(AUDIT)
    missing = [a for a in SPEC_ANCHORS if a not in spec]
    # Ground the spec's "implemented today" claims in real source.
    fnmode_safe = "Safe," in ast and "FnMode" in ast
    boundary_audit = 'safe→managed escalation' in audit and "ModeAuditLog" in audit
    spec_present = bool(spec) and not missing
    ok = spec_present and fnmode_safe and boundary_audit
    return SafeSubsetStatus(
        schema="garnet.safe_subset_status/v1",
        spec_present=spec_present,
        missing_spec_anchors=missing,
        fnmode_safe_in_ast=fnmode_safe,
        boundary_audit_in_checker=boundary_audit,
        ok=ok,
        groundings=[
            f"FnMode::Safe in {AST.relative_to(ROOT)}: {fnmode_safe}",
            f"ModeAuditLog boundary audit in {AUDIT.relative_to(ROOT)}: {boundary_audit}",
        ],
    )


def render_markdown(r: SafeSubsetStatus) -> str:
    lines = [
        "# Garnet safe-subset spec status (S74)",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- spec `GARNET_SAFE_SUBSET.md` present + anchored: {'yes' if r.spec_present else 'NO'}"
        + (f" (missing: {r.missing_spec_anchors})" if r.missing_spec_anchors else ""),
        f"- grounded: `FnMode::Safe` in AST: {'yes' if r.fnmode_safe_in_ast else 'NO'}",
        f"- grounded: fn↔def boundary audit in checker: {'yes' if r.boundary_audit_in_checker else 'NO'}",
        "",
        "The safe subset today = the typed, ownership-disciplined `fn` mode + the "
        "fn↔def boundary audit (closes the hidden safe→managed escalation threat "
        "class). The optional linear/effect-typed rigor mode is a PROPOSAL — NOT "
        "IMPLEMENTED; this slice specifies, it builds no type system.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero if the spec is missing/unanchored or its 'implemented "
        "today' claims are not grounded in source. The proposed linear/effect mode "
        "is NOT gated (it is explicitly not implemented).",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status()
    if args.format == "md":
        print(render_markdown(r))
    else:
        print(json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(
            "safe-subset gate FAILED: "
            f"spec_present={r.spec_present} missing={r.missing_spec_anchors} "
            f"fnmode_safe={r.fnmode_safe_in_ast} boundary_audit={r.boundary_audit_in_checker}",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
