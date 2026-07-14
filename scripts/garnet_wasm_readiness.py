#!/usr/bin/env python3
"""Evidence-backed Wasm and W-PLAY readiness reporter.

The committed WV-5 proof is the authority for the already-completed Wasm build
and Node-execution lane. Local tool presence is reported only as reproduction
convenience; it can never erase committed product evidence. Browser execution
remains open until W-PLAY lands the adapter, package, and Playwright proof.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
HELLO = ROOT / "examples" / "hello.garnet"
TARGET_DOC = ROOT / "F_Project_Management" / "GARNET_WASM_TARGET.md"
WASM_CRATE = ROOT / "garnet-wasm" / "src" / "lib.rs"
WV5_DIR = (
    ROOT
    / "proofs"
    / "windows"
    / "launch-verification"
    / "wv5-wasm-lane-20260712-0915"
)
WV5_PROOF = WV5_DIR / "wv5-wasm-lane-proof.json"
WV5_MANIFEST = WV5_DIR / "MANIFEST.sha256"
WV5_NODE_LOG = WV5_DIR / "commands" / "node-smoke-stdout.txt"
LIVE_ADAPTER = ROOT / "docs" / "playground" / "live.js"
BROWSER_PACKAGE = ROOT / "docs" / "playground" / "pkg" / "garnet_wasm_bg.wasm"
BROWSER_PROOF = (
    ROOT / "F_Project_Management" / "LAUNCH" / "W_PLAY_BROWSER_PROOF.json"
)

REQUIRED_WV5_COMMANDS = {
    "wasm-native-tests",
    "output-capture-tests",
    "wasm32-build",
    "wasm-pack-web",
    "wasm-pack-nodejs",
    "node-smoke",
}


@dataclass
class WasmReadiness:
    schema: str
    hello_example_present: bool
    target_doc_present: bool
    wasm_crate_present: bool
    windows_proof_present: bool
    windows_proof_valid: bool
    windows_proof_commit: str
    wasm_build_passed: bool
    node_execution_passed: bool
    check_source_export_present: bool
    caps_surface_export_present: bool
    browser_adapter_present: bool
    browser_package_present: bool
    browser_proof_present: bool
    wasm32_target_installed: bool
    wasm_pack_present: bool
    node_present: bool
    wasmtime_present: bool
    miette_fancy_detected: bool
    blockers: list[str] = field(default_factory=list)
    owned_bits_ready: bool = False


def _has_wasm32_target() -> bool:
    rustup = shutil.which("rustup")
    if rustup is None:
        return False
    proc = subprocess.run(
        [rustup, "target", "list", "--installed"],
        capture_output=True,
        text=True,
        check=False,
    )
    return proc.returncode == 0 and "wasm32-unknown-unknown" in proc.stdout


def _read_wv5_proof() -> tuple[dict | None, dict[str, dict]]:
    try:
        payload = json.loads(WV5_PROOF.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None, {}
    if not isinstance(payload, dict):
        return None, {}
    commands: dict[str, dict] = {}
    for entry in payload.get("commands", []):
        if isinstance(entry, dict) and isinstance(entry.get("name"), str):
            commands[entry["name"]] = entry
    return payload, commands


def _command_passed(commands: dict[str, dict], name: str) -> bool:
    entry = commands.get(name)
    return bool(
        isinstance(entry, dict)
        and entry.get("ok") is True
        and entry.get("exit_code") == 0
    )


def _manifest_valid() -> bool:
    """Verify every file named by the committed WV-5 SHA-256 manifest."""
    try:
        lines = WV5_MANIFEST.read_text(encoding="utf-8").splitlines()
    except OSError:
        return False
    if not lines:
        return False
    root = WV5_DIR.resolve()
    for line in lines:
        parts = line.split(maxsplit=1)
        if len(parts) != 2 or re.fullmatch(r"[0-9a-f]{64}", parts[0]) is None:
            return False
        candidate = (WV5_DIR / parts[1].strip()).resolve()
        try:
            candidate.relative_to(root)
            actual = hashlib.sha256(candidate.read_bytes()).hexdigest()
        except (OSError, ValueError):
            return False
        if actual != parts[0]:
            return False
    return True


def _node_semantics_valid() -> bool:
    try:
        text = WV5_NODE_LOG.read_text(encoding="utf-8")
    except OSError:
        return False
    return all(
        marker in text
        for marker in (
            "NODE_SMOKE hello:",
            '"stdout":"Hello from Garnet!\\n"',
            "NODE_SMOKE authority-fail-closed: runtime_error",
            "NODE_SMOKE: PASS",
        )
    )


def read_readiness() -> WasmReadiness:
    wasm_source = WASM_CRATE.read_text(encoding="utf-8") if WASM_CRATE.is_file() else ""
    interp_cargo = ROOT / "garnet-interp-v0.3" / "Cargo.toml"
    cargo_text = interp_cargo.read_text(encoding="utf-8") if interp_cargo.is_file() else ""
    proof, commands = _read_wv5_proof()

    proof_valid = bool(
        proof
        and proof.get("schema") == "garnet.windows_launch_verification_proof/v1"
        and proof.get("item") == "WV-5"
        and proof.get("verdict") == "pass"
        and proof.get("workdir_porcelain_dirty_at_start") is False
        and re.fullmatch(r"[0-9a-f]{40}", str(proof.get("git_head", ""))) is not None
        and REQUIRED_WV5_COMMANDS.issubset(commands)
        and all(_command_passed(commands, name) for name in REQUIRED_WV5_COMMANDS)
        and _manifest_valid()
        and _node_semantics_valid()
    )
    wasm_build_passed = proof_valid and all(
        _command_passed(commands, name)
        for name in ("wasm32-build", "wasm-pack-web", "wasm-pack-nodejs")
    )
    node_execution_passed = proof_valid and _command_passed(commands, "node-smoke")

    check_source_present = "pub fn check_source" in wasm_source
    caps_surface_present = (
        "pub fn caps_surface" in wasm_source or "pub fn diff_caps" in wasm_source
    )
    browser_adapter_present = LIVE_ADAPTER.is_file()
    browser_package_present = BROWSER_PACKAGE.is_file()
    browser_proof_present = BROWSER_PROOF.is_file()

    blockers: list[str] = []
    if not wasm_build_passed:
        blockers.append("committed WV-5 wasm32 + wasm-pack build proof is missing or invalid")
    if not node_execution_passed:
        blockers.append("committed WV-5 real Node execution proof is missing or invalid")
    if not check_source_present:
        blockers.append("W-PLAY check_source Wasm export is not implemented")
    if not caps_surface_present:
        blockers.append("W-PLAY capability-surface/diff Wasm export is not implemented")
    if not browser_adapter_present:
        blockers.append("docs/playground/live.js browser adapter is not implemented")
    if not browser_package_present:
        blockers.append("browser Wasm package is not present under docs/playground/pkg")
    if not browser_proof_present:
        blockers.append("W-PLAY Playwright browser proof is not recorded")

    owned_bits_ready = bool(
        HELLO.is_file()
        and TARGET_DOC.is_file()
        and WASM_CRATE.is_file()
        and wasm_build_passed
        and node_execution_passed
    )
    return WasmReadiness(
        schema="garnet.wasm_readiness/v2",
        hello_example_present=HELLO.is_file(),
        target_doc_present=TARGET_DOC.is_file(),
        wasm_crate_present=WASM_CRATE.is_file(),
        windows_proof_present=WV5_PROOF.is_file(),
        windows_proof_valid=proof_valid,
        windows_proof_commit=str(proof.get("git_head", "")) if proof else "",
        wasm_build_passed=wasm_build_passed,
        node_execution_passed=node_execution_passed,
        check_source_export_present=check_source_present,
        caps_surface_export_present=caps_surface_present,
        browser_adapter_present=browser_adapter_present,
        browser_package_present=browser_package_present,
        browser_proof_present=browser_proof_present,
        wasm32_target_installed=_has_wasm32_target(),
        wasm_pack_present=shutil.which("wasm-pack") is not None,
        node_present=shutil.which("node") is not None,
        wasmtime_present=shutil.which("wasmtime") is not None,
        miette_fancy_detected="miette" in cargo_text and '"fancy"' in cargo_text,
        blockers=blockers,
        owned_bits_ready=owned_bits_ready,
    )


def render_markdown(r: WasmReadiness) -> str:
    lines = [
        "# Garnet Wasm / W-PLAY readiness",
        "",
        f"_Schema {r.schema}._",
        "",
        "## Recorded product evidence",
        "",
        f"- `garnet-wasm` crate present: {r.wasm_crate_present}",
        f"- clean-Windows WV-5 proof valid: {r.windows_proof_valid}",
        f"- proof commit: `{r.windows_proof_commit or '<missing>'}`",
        f"- wasm32 + wasm-pack builds passed: {r.wasm_build_passed}",
        f"- real Node execution passed: {r.node_execution_passed}",
        f"- build/execution owned bits ready: **{'yes' if r.owned_bits_ready else 'NO'}**",
        "",
        "## Remaining browser product surface",
        "",
        f"- `check_source` export: {r.check_source_export_present}",
        f"- capability-surface/diff export: {r.caps_surface_export_present}",
        f"- browser adapter: {r.browser_adapter_present}",
        f"- browser package: {r.browser_package_present}",
        f"- Playwright proof: {r.browser_proof_present}",
    ]
    lines.extend(f"- BLOCKER: {item}" for item in r.blockers)
    lines += [
        "",
        "## Local reproduction convenience (not product truth)",
        "",
        f"- wasm32 target installed here: {r.wasm32_target_installed}",
        f"- wasm-pack present here: {r.wasm_pack_present}",
        f"- Node present here: {r.node_present}",
        f"- wasmtime present here (optional): {r.wasmtime_present}",
        f"- miette `fancy` detected: {r.miette_fancy_detected} (recorded WV-5 build proves it is not a blocker)",
        "",
        "Honest scope: WV-5 proves a real interpreter-to-Wasm build and Node "
        "execution. It does not prove live browser-page execution; that claim "
        "waits for the W-PLAY Playwright artifact.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless committed Wasm build + Node execution evidence is valid",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    result = read_readiness()
    print(render_markdown(result) if args.format == "md" else json.dumps(asdict(result), indent=2))
    if args.gate and not result.owned_bits_ready:
        print(
            "wasm-readiness gate FAILED: committed build/execution evidence is incomplete "
            f"(crate={result.wasm_crate_present}, proof={result.windows_proof_valid}, "
            f"build={result.wasm_build_passed}, node={result.node_execution_passed})",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
