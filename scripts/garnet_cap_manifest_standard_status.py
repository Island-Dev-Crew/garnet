#!/usr/bin/env python3
"""S98 capability-manifest standard seed status.

This gate verifies a draft/reference seed only. It must not imply OWASP/LF
adoption, a multi-language ecosystem, or proof of undeclared-authority absence.
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
STANDARD_DOC = ROOT / "C_Language_Specification" / "GARNET_CAPABILITY_MANIFEST_STANDARD.md"
RFC_0001 = ROOT / "rfcs" / "0001-capability-manifest-standard.md"
CAP_MANIFEST_RS = ROOT / "garnet-cli" / "src" / "cap_manifest.rs"
CAPS_BIN = ROOT / "garnet-cli" / "src" / "bin" / "garnet.rs"
CLI_TEST = ROOT / "garnet-cli" / "tests" / "cap_manifest_standard.rs"
VECTORS = ROOT / "test-vectors" / "capability-manifest-v1"


@dataclass
class CapManifestStandardStatus:
    schema: str
    standard_doc_present: bool
    rfc_references_standard_doc: bool
    reference_impl_present: bool
    test_vectors_present: bool
    cli_gate_present: bool
    focused_gate_ok: bool | None
    ok: bool
    scope_summary: str


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8") if path.is_file() else ""


def _run_focused_gate() -> bool:
    result = subprocess.run(
        ["cargo", "test", "-p", "garnet-cli", "--test", "cap_manifest_standard", "--no-fail-fast"],
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    return result.returncode == 0


def read_status(run_gate: bool = False) -> CapManifestStandardStatus:
    doc = _read(STANDARD_DOC)
    rfc = _read(RFC_0001)
    impl = _read(CAP_MANIFEST_RS)
    cli = _read(CAPS_BIN)
    vectors = sorted(VECTORS.glob("*.json")) if VECTORS.is_dir() else []
    vector_text = "\n".join(_read(path) for path in vectors)

    standard_doc_present = (
        "capability-manifest/v1" in doc
        and "draft/reference seed" in doc
        and "No OWASP" in doc
    )
    rfc_references_standard_doc = (
        "GARNET_CAPABILITY_MANIFEST_STANDARD.md" in rfc
        and "no external body has adopted it" in rfc
    )
    reference_impl_present = (
        'STANDARD_SCHEMA: &str = "capability-manifest/v1"' in impl
        and "to_standard_profile_json" in impl
    )
    test_vectors_present = (
        len(vectors) >= 2
        and '"schema":"capability-manifest/v1"' in vector_text
        and '"status":"draft-reference-seed"' in vector_text
        and "no OWASP/LF adoption claimed" in vector_text
    )
    cli_gate_present = (
        "--standard-profile" in cli
        and CLI_TEST.is_file()
        and "caps_standard_profile_emits_language_neutral_schema" in _read(CLI_TEST)
    )
    focused_gate_ok = _run_focused_gate() if run_gate else None
    static_ok = (
        standard_doc_present
        and rfc_references_standard_doc
        and reference_impl_present
        and test_vectors_present
        and cli_gate_present
    )
    ok = static_ok and (focused_gate_ok is not False)
    return CapManifestStandardStatus(
        schema="garnet.cap_manifest_standard/v1",
        standard_doc_present=standard_doc_present,
        rfc_references_standard_doc=rfc_references_standard_doc,
        reference_impl_present=reference_impl_present,
        test_vectors_present=test_vectors_present,
        cli_gate_present=cli_gate_present,
        focused_gate_ok=focused_gate_ok,
        ok=ok,
        scope_summary=(
            "S98 is a draft/reference seed for a capability-manifest profile, "
            "not adopted by any standards body and not a proof of undeclared-authority absence."
        ),
    )


def render_markdown(status: CapManifestStandardStatus) -> str:
    lines = [
        "# Garnet capability-manifest standard status (S98)",
        "",
        f"_Schema {status.schema}._",
        "",
        f"- Standard doc present: {'yes' if status.standard_doc_present else 'NO'}",
        f"- RFC-0001 references standard doc: {'yes' if status.rfc_references_standard_doc else 'NO'}",
        f"- Reference implementation present: {'yes' if status.reference_impl_present else 'NO'}",
        f"- Test vectors present: {'yes' if status.test_vectors_present else 'NO'}",
        f"- CLI proof present: {'yes' if status.cli_gate_present else 'NO'}",
        "",
        "Honest scope: draft/reference seed over the declared capability surface. "
        "No OWASP/LF adoption is claimed, and the profile does not prove absence "
        "of undeclared authority.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument("--gate", action="store_true")
    args = parser.parse_args(list(argv) if argv is not None else None)

    status = read_status(run_gate=args.gate)
    if args.format == "md":
        print(render_markdown(status))
    else:
        print(json.dumps(asdict(status), indent=2))
    if args.gate and not status.ok:
        print(f"cap-manifest standard gate FAILED: {asdict(status)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
