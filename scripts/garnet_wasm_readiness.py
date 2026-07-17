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
BROWSER_PACKAGE_DIR = ROOT / "docs" / "playground" / "pkg"
BROWSER_PACKAGE = BROWSER_PACKAGE_DIR / "garnet_wasm_bg.wasm"
BROWSER_PROVENANCE = BROWSER_PACKAGE_DIR / "provenance.json"
BROWSER_PROOF = (
    ROOT / "F_Project_Management" / "LAUNCH" / "W_PLAY_BROWSER_PROOF.json"
)
PACKAGE_FILES = {"garnet_wasm.js", "garnet_wasm_bg.wasm", "provenance.json"}
STUDIO_PACKAGE = ROOT / "apps" / "garnet-studio" / "package.json"
STUDIO_LOCK = ROOT / "apps" / "garnet-studio" / "package-lock.json"
BROWSER_RUNTIME_INPUTS = (
    "apps/garnet-studio/package-lock.json",
    "apps/garnet-studio/package.json",
    "docs/icons/garnet-192.png",
    "docs/playground.html",
    "docs/playground/examples.json",
    "docs/playground/live.js",
    "docs/playground/pkg/garnet_wasm.js",
    "docs/playground/pkg/garnet_wasm_bg.wasm",
    "docs/playground/pkg/provenance.json",
    "scripts/smoke_garnet_playground_browser.mjs",
)
DIFF_SCOPE = (
    "declared-surface-only; does not prove absence of undeclared authority; "
    "bound annotations are not part of this surface"
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
    browser_package_valid: bool
    browser_proof_present: bool
    browser_proof_valid: bool
    browser_ready: bool
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


def _sha256(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def _canonical_source(raw: bytes) -> bytes:
    return raw.replace(b"\r\n", b"\n")


def _safe_repo_file(relative: str) -> Path | None:
    if not relative or "\\" in relative:
        return None
    candidate = (ROOT / relative).resolve()
    try:
        candidate.relative_to(ROOT.resolve())
    except ValueError:
        return None
    if candidate.is_symlink() or not candidate.is_file():
        return None
    return candidate


def _read_json_object(path: Path) -> dict | None:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError):
        return None
    return value if isinstance(value, dict) else None


def read_browser_package_provenance() -> dict | None:
    return _read_json_object(BROWSER_PROVENANCE)


def browser_package_valid(provenance: dict | None = None) -> bool:
    if provenance is None:
        provenance = read_browser_package_provenance()
    if provenance is None or provenance.get("schema") != "garnet.playground.wasm-package/1":
        return False
    try:
        if BROWSER_PACKAGE_DIR.is_symlink() or {
            path.name for path in BROWSER_PACKAGE_DIR.iterdir()
        } != PACKAGE_FILES:
            return False
        artifacts = provenance["artifacts"]
        if set(artifacts) != {"garnet_wasm.js", "garnet_wasm_bg.wasm"}:
            return False
        for name, metadata in artifacts.items():
            path = BROWSER_PACKAGE_DIR / name
            if path.is_symlink() or not path.is_file():
                return False
            raw = path.read_bytes()
            if metadata != {"bytes": len(raw), "sha256": _sha256(raw)}:
                return False
        source = provenance["source"]
        if "build_parent_commit_observed" in source:
            return False
        inputs = source["inputs"]
        if not isinstance(inputs, list) or inputs != sorted(set(inputs)):
            return False
        digest = hashlib.sha256()
        for relative in inputs:
            if not isinstance(relative, str):
                return False
            path = _safe_repo_file(relative)
            if path is None:
                return False
            digest.update(relative.encode() + b"\0" + _canonical_source(path.read_bytes()) + b"\0")
        if digest.hexdigest() != source["source_tree_sha256"]:
            return False
        cargo_lock = _safe_repo_file("Cargo.lock")
        studio_lock = _safe_repo_file("apps/garnet-studio/package-lock.json")
        if cargo_lock is None or studio_lock is None:
            return False
        if _sha256(_canonical_source(cargo_lock.read_bytes())) != source["cargo_lock_sha256"]:
            return False
        if _sha256(_canonical_source(studio_lock.read_bytes())) != source["studio_package_lock_sha256"]:
            return False
        tools = provenance["tools"]
        for name in ("cargo", "esbuild", "node", "rustc", "wasm_pack"):
            if not isinstance(tools.get(name), dict) or not tools[name].get("version"):
                return False
    except (KeyError, OSError, TypeError, ValueError):
        return False
    return True


def read_browser_proof() -> dict | None:
    return _read_json_object(BROWSER_PROOF)


def _proof_package_matches(proof: dict, provenance: dict) -> bool:
    try:
        return proof["package"] == {
            "schema": provenance["schema"],
            "source_tree_sha256": provenance["source"]["source_tree_sha256"],
            "artifacts": provenance["artifacts"],
        }
    except (KeyError, TypeError):
        return False


def current_browser_runtime_inputs() -> dict | None:
    aggregate = hashlib.sha256()
    files: dict[str, dict[str, int | str]] = {}
    try:
        for relative in BROWSER_RUNTIME_INPUTS:
            path = _safe_repo_file(relative)
            if path is None:
                return None
            raw = path.read_bytes()
            digest = _sha256(raw)
            files[relative] = {"bytes": len(raw), "sha256": digest}
            aggregate.update(relative.encode("utf-8"))
            aggregate.update(b"\0")
            aggregate.update(bytes.fromhex(digest))
            aggregate.update(b"\0")
    except OSError:
        return None
    return {
        "schema": "garnet.w-play.runtime-inputs/1",
        "sha256": aggregate.hexdigest(),
        "files": files,
    }


def expected_playwright_identity() -> dict | None:
    try:
        package_raw = STUDIO_PACKAGE.read_bytes()
        lock_raw = STUDIO_LOCK.read_bytes()
        package = json.loads(package_raw)
        lock = json.loads(lock_raw)
        declared = package["devDependencies"]["@playwright/test"]
        locked = lock["packages"]["node_modules/@playwright/test"]
        identity_fields = (declared, locked["version"], locked["integrity"])
        if not all(isinstance(value, str) and value for value in identity_fields):
            return None
        return {
            "package": "@playwright/test",
            "declared": declared,
            "version": locked["version"],
            "integrity": locked["integrity"],
            "package_json_sha256": _sha256(package_raw),
            "package_lock_sha256": _sha256(lock_raw),
            "install_command": "npm ci --ignore-scripts",
        }
    except (KeyError, OSError, TypeError, ValueError, json.JSONDecodeError):
        return None


def _proof_runtime_inputs_match(proof: dict) -> bool:
    current = current_browser_runtime_inputs()
    return current is not None and proof.get("runtime_inputs") == current


def _proof_playwright_matches(proof: dict) -> bool:
    expected = expected_playwright_identity()
    observed = proof.get("toolchain", {}).get("playwright")
    return expected is not None and observed == expected


def _proof_screenshot_valid(proof: dict) -> bool:
    try:
        visual = proof["visual"]
        relative = visual["screenshot"]
        if not isinstance(relative, str):
            return False
        screenshot = _safe_repo_file(relative)
        return bool(
            screenshot
            and _sha256(screenshot.read_bytes()) == visual["screenshot_sha256"]
            and visual["desktop"]["horizontal_overflow"] is False
            and visual["desktop"]["runtime_state"] == "ready"
            and visual["mobile"]["horizontal_overflow"] is False
        )
    except (KeyError, OSError, TypeError):
        return False


def browser_proof_valid(proof: dict | None = None) -> bool:
    if proof is None:
        proof = read_browser_proof()
    provenance = read_browser_package_provenance()
    if proof is None or provenance is None or not browser_package_valid(provenance):
        return False
    try:
        execution = proof["execution"]
        git = proof["git"]
        network = proof["network"]
        journeys = proof["journeys"]
        run = journeys["run"]
        check = journeys["check"]
        diff = journeys["diff"]
        adapter = diff["adapter_result"]
        machine = diff["machine_verdict"]
        denial = journeys["denial"]
        denial_run = denial["run"]
        requested = network["requested_committed_files"]
        if (
            not isinstance(requested, list)
            or not 1 <= len(requested) <= 32
            or len(requested) != len(set(requested))
            or not all(isinstance(item, str) and _safe_repo_file(item) for item in requested)
        ):
            return False
        screenshot_relative = proof["visual"]["screenshot"]
        if not isinstance(screenshot_relative, str):
            return False
        try:
            proof_relative = BROWSER_PROOF.resolve().relative_to(ROOT.resolve()).as_posix()
        except ValueError:
            return False
        required_requests = {
            "docs/playground.html",
            "docs/playground/live.js",
            "docs/playground/pkg/garnet_wasm.js",
            "docs/playground/pkg/garnet_wasm_bg.wasm",
        }
        tracked_paths = [
            *requested,
            *BROWSER_RUNTIME_INPUTS,
            screenshot_relative,
            proof_relative,
        ]
        tracked = subprocess.run(
            ["git", "ls-files", "--error-unmatch", "--", *tracked_paths],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        return bool(
            proof.get("schema") == "garnet.w-play.browser-proof/1"
            and proof.get("verdict") == "pass"
            and isinstance(proof.get("duration_ms"), int)
            and 0 < proof["duration_ms"] < 30_000
            and execution.get("engine") == "playwright-browser-page"
            and execution.get("node_global_present") is False
            and execution.get("runtime_ready") is True
            and execution.get("service_workers") == "blocked"
            and git.get("runtime_inputs_clean") is True
            and re.fullmatch(r"[0-9a-f]{40}", str(git.get("tested_commit", "")))
            and re.fullmatch(r"[0-9a-f]{40}", str(git.get("tested_tree", "")))
            and _proof_package_matches(proof, provenance)
            and _proof_runtime_inputs_match(proof)
            and _proof_playwright_matches(proof)
            and network.get("external_requests") == []
            and network.get("untracked_requests") == []
            and required_requests.issubset(requested)
            and tracked.returncode == 0
            and run == {
                "schema": "garnet.wasm.run/1",
                "exit_class": "ok",
                "stdout": "Hello from Garnet!\n",
                "diagnostic": None,
            }
            and check == {
                "schema": "garnet.wasm.check/1",
                "ok": True,
                "diagnostics": [],
            }
            and adapter.get("schema") == "garnet.wasm.diff-caps/1"
            and adapter.get("ok") is True
            and adapter.get("authority_expanded") is True
            and adapter.get("aggregate_added") == ["fs"]
            and adapter.get("aggregate_removed") == []
            and adapter.get("wildcard_introduced") is False
            and adapter.get("scope") == DIFF_SCOPE
            and diff.get("human_verdict") == "Authority expanded"
            and machine == {
                "schema": "garnet.playground.diff-caps-verdict/1",
                "verdict": "expanded",
                "authority_expanded": True,
                "aggregate_added": ["fs"],
                "aggregate_removed": [],
                "wildcard_introduced": False,
                "scope": DIFF_SCOPE,
            }
            and denial.get("ui_state") == "Denied"
            and denial_run.get("schema") == "garnet.wasm.run/1"
            and denial_run.get("exit_class") == "runtime_error"
            and denial_run.get("stdout") == ""
            and "proc" in str(denial_run.get("diagnostic", "")).lower()
            and proof["diagnostics"].get("console_errors") == []
            and proof["diagnostics"].get("page_errors") == []
            and _proof_screenshot_valid(proof)
        )
    except (KeyError, TypeError, ValueError):
        return False


def read_readiness() -> WasmReadiness:
    wasm_source = WASM_CRATE.read_text(encoding="utf-8") if WASM_CRATE.is_file() else ""
    interp_cargo = ROOT / "garnet-interp-v0.3" / "Cargo.toml"
    cargo_text = interp_cargo.read_text(encoding="utf-8") if interp_cargo.is_file() else ""
    proof, commands = _read_wv5_proof()

    wv5_proof_valid = bool(
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
    wasm_build_passed = wv5_proof_valid and all(
        _command_passed(commands, name)
        for name in ("wasm32-build", "wasm-pack-web", "wasm-pack-nodejs")
    )
    node_execution_passed = wv5_proof_valid and _command_passed(commands, "node-smoke")

    check_source_present = "pub fn check_source" in wasm_source
    caps_surface_present = (
        "pub fn caps_surface" in wasm_source or "pub fn diff_caps" in wasm_source
    )
    browser_adapter_present = LIVE_ADAPTER.is_file()
    browser_package_present = BROWSER_PACKAGE.is_file()
    package_valid = browser_package_valid()
    browser_proof_present = BROWSER_PROOF.is_file()
    browser_proof_is_valid = browser_proof_valid()
    browser_ready = browser_adapter_present and package_valid and browser_proof_is_valid

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
    if not package_valid:
        blockers.append(
            "browser Wasm package is missing, invalid, or does not match current committed inputs"
        )
    if not browser_proof_is_valid:
        blockers.append("W-PLAY Playwright browser proof is missing or invalid")

    owned_bits_ready = bool(
        HELLO.is_file()
        and TARGET_DOC.is_file()
        and WASM_CRATE.is_file()
        and wasm_build_passed
        and node_execution_passed
    )
    return WasmReadiness(
        schema="garnet.wasm_readiness/v3",
        hello_example_present=HELLO.is_file(),
        target_doc_present=TARGET_DOC.is_file(),
        wasm_crate_present=WASM_CRATE.is_file(),
        windows_proof_present=WV5_PROOF.is_file(),
        windows_proof_valid=wv5_proof_valid,
        windows_proof_commit=str(proof.get("git_head", "")) if proof else "",
        wasm_build_passed=wasm_build_passed,
        node_execution_passed=node_execution_passed,
        check_source_export_present=check_source_present,
        caps_surface_export_present=caps_surface_present,
        browser_adapter_present=browser_adapter_present,
        browser_package_present=browser_package_present,
        browser_package_valid=package_valid,
        browser_proof_present=browser_proof_present,
        browser_proof_valid=browser_proof_is_valid,
        browser_ready=browser_ready,
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
        f"- browser package present: {r.browser_package_present}",
        f"- browser package valid: {r.browser_package_valid}",
        f"- Playwright proof present: {r.browser_proof_present}",
        f"- Playwright proof valid: {r.browser_proof_valid}",
        f"- browser ready: **{'yes' if r.browser_ready else 'NO'}**",
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
        "Honest scope: WV-5 alone proves a real interpreter-to-Wasm build and "
        "Node execution. Browser readiness is promoted separately only when the "
        "committed package and strict Playwright proof both validate.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless committed Wasm, Node, package, and browser evidence is valid",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    result = read_readiness()
    print(render_markdown(result) if args.format == "md" else json.dumps(asdict(result), indent=2))
    if args.gate and not (result.owned_bits_ready and result.browser_ready):
        print(
            "wasm-readiness gate FAILED: committed build/execution/browser evidence is incomplete "
            f"(crate={result.wasm_crate_present}, proof={result.windows_proof_valid}, "
            f"build={result.wasm_build_passed}, node={result.node_execution_passed}, "
            f"package={result.browser_package_valid}, browser={result.browser_proof_valid})",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
