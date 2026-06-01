#!/usr/bin/env python3
"""Status gate for the S97 provenance seal-chain seed."""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SEAL_RS = ROOT / "garnet-cli" / "src" / "seal.rs"
SEAL_CMD_RS = ROOT / "garnet-cli" / "src" / "cmd" / "seal.rs"
FOCUSED_TESTS = ROOT / "garnet-cli" / "tests" / "provenance_seal_chain.rs"
ATTESTATION_DOC = ROOT / "C_Language_Specification" / "GARNET_ATTESTATION.md"
READINESS = ROOT / "scripts" / "garnet_mit_readiness_status.py"


@dataclass(frozen=True)
class ProvenanceSealChainStatus:
    schema: str
    rust_test_present: bool
    cli_flag_present: bool
    chain_builder_present: bool
    attestation_doc_present: bool
    readiness_lane_present: bool
    focused_gate_ok: bool | None
    scope_summary: str
    ok: bool


def _text(path: Path) -> str:
    if not path.is_file():
        return ""
    return path.read_text(encoding="utf-8")


def _run_focused_gate() -> bool:
    completed = subprocess.run(
        ["cargo", "test", "-p", "garnet-cli", "--test", "provenance_seal_chain", "--no-fail-fast"],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    return completed.returncode == 0


def read_status(run_gate: bool = False) -> ProvenanceSealChainStatus:
    seal_text = _text(SEAL_RS)
    cmd_text = _text(SEAL_CMD_RS)
    tests_text = _text(FOCUSED_TESTS)
    doc_text = _text(ATTESTATION_DOC)
    readiness_text = _text(READINESS)

    rust_test_present = (
        "provenance_chain_binds_declared_agent_model_prompt_to_current_seal" in tests_text
        and "provenance_chain_output_is_deterministic_across_attestation_order" in tests_text
    )
    cli_flag_present = (
        "--provenance-chain" in cmd_text
        and "build_provenance_chain" in cmd_text
        and "provenance-chain:" in cmd_text
    )
    chain_builder_present = (
        "garnet-provenance-chain-v1" in seal_text
        and "build_provenance_chain" in seal_text
        and "binding_verified" in seal_text
        and "independent_origin_verified" in seal_text
    )
    attestation_doc_present = (
        "Provenance seal chain (S97)" in doc_text
        and "verification of **binding**" in doc_text
        and "does not prove that a model" in doc_text
    )
    readiness_lane_present = "provenance_seal_chain" in readiness_text
    focused_gate_ok = _run_focused_gate() if run_gate else None

    inventory_ok = all(
        [
            rust_test_present,
            cli_flag_present,
            chain_builder_present,
            attestation_doc_present,
            readiness_lane_present,
        ]
    )
    ok = inventory_ok and (focused_gate_ok is not False)
    return ProvenanceSealChainStatus(
        schema="garnet.provenance_seal_chain/v1",
        rust_test_present=rust_test_present,
        cli_flag_present=cli_flag_present,
        chain_builder_present=chain_builder_present,
        attestation_doc_present=attestation_doc_present,
        readiness_lane_present=readiness_lane_present,
        focused_gate_ok=focused_gate_ok,
        scope_summary=(
            "S97 binds self-declared agent/model/prompt metadata to the sealed "
            "artifact and verifies that deterministic binding. The declared "
            "origin remains not independently verified."
        ),
        ok=ok,
    )


def render_markdown(status: ProvenanceSealChainStatus) -> str:
    return "\n".join(
        [
            "# Garnet S97 provenance seal-chain status",
            "",
            f"_Schema {status.schema}._",
            "",
            f"- focused Rust tests: {'yes' if status.rust_test_present else 'NO'}",
            f"- CLI flag wired: {'yes' if status.cli_flag_present else 'NO'}",
            f"- chain builder present: {'yes' if status.chain_builder_present else 'NO'}",
            f"- attestation contract documented: {'yes' if status.attestation_doc_present else 'NO'}",
            f"- readiness lane: {'yes' if status.readiness_lane_present else 'NO'}",
            f"- focused gate: {status.focused_gate_ok}",
            "",
            status.scope_summary,
            "It does not prove the model executed the prompt or that the declared tool list is complete.",
            "",
        ]
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument("--gate", action="store_true")
    args = parser.parse_args(list(argv) if argv is not None else None)

    status = read_status(run_gate=args.gate)
    if args.format == "md":
        print(render_markdown(status))
    else:
        print(json.dumps(asdict(status), indent=2, sort_keys=True))

    if args.gate and not status.ok:
        print(f"S97 provenance seal-chain gate FAILED: {asdict(status)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
