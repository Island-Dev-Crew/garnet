#!/usr/bin/env python3
"""Record and verify a consolidated Linux/Tauri gate replay proof.

This S117 consolidation increment replays the existing WSL/WSLg package,
runtime, display, domain-shell, and release-shell gates from one repo-owned
command. It deliberately remains an execution/portability proof: WSL and WSLg
do not prove Linux seccomp, OS-sandbox enforcement, or clean/non-WSL Linux
desktop behavior.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Callable, Sequence

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = "garnet.studio.linux_gate_replay.v1"
SUMMARY_NAME = "garnet-studio-linux-gate-replay.json"
MARKDOWN_NAME = "garnet-studio-linux-gate-replay.md"
MANIFEST_NAME = "MANIFEST.sha256"
DEFAULT_PROOF_ROOT = ROOT / "proofs" / "linux" / "execution" / "studio-gate-replay"
PLATFORM_TIER = "WSL/WSLg execution-portability only, not enforcement"
REQUIRED_HONEST_SCOPE = [
    "WSL/WSLg execution/portability only",
    "not clean/non-WSL Linux desktop proof",
    "not Linux seccomp or OS-sandbox enforcement",
    "not signed, production, or v1.0 readiness",
]
FORBIDDEN_CLAIMS = [
    "Linux seccomp enforced",
    "Linux seccomp verified",
    "OS-sandbox enforcement verified",
    "clean Linux desktop proof verified",
    "non-WSL Linux desktop proof verified",
    "signed release verified",
    "production readiness verified",
    "v1.0 readiness verified",
]


@dataclass(frozen=True)
class GateSpec:
    id: str
    label: str
    script: str


@dataclass(frozen=True)
class LinuxGateReplayEvidence:
    status: str
    verified: bool
    reason: str
    bundle: str | None
    deferred: list[str]


GATE_SPECS = [
    GateSpec("linux-wsl-deb-package", "WSL .deb package build and command smoke", "smoke_garnet_studio_linux_wsl_deb.py"),
    GateSpec("linux-wsl-deb-install", "WSL .deb extract and installed-tree command smoke", "smoke_garnet_studio_linux_wsl_deb_install.py"),
    GateSpec("linux-wsl-rpm-package", "WSL .rpm extract and command smoke", "smoke_garnet_studio_linux_wsl_rpm.py"),
    GateSpec("linux-wsl-xvfb-runtime", "WSL Xvfb runtime-start proof", "smoke_garnet_studio_linux_wsl_xvfb.py"),
    GateSpec("linux-wsl-xvfb-window", "WSL Xvfb virtual-display window capture", "smoke_garnet_studio_linux_wsl_xvfb_window.py"),
    GateSpec("linux-wslg-system-install", "WSLg system install and launch observation", "smoke_garnet_studio_linux_wslg_install_launch.py"),
    GateSpec("windows-wsl-domain-shell", "Windows/WSL Studio domain-shell proof", "smoke_garnet_studio_domain_shell.py"),
    GateSpec("windows-wsl-release-readiness-shell", "Windows/WSL Studio release/readiness shell proof", "smoke_garnet_studio_release_readiness_shell.py"),
]


Runner = Callable[..., subprocess.CompletedProcess[str]]


def timestamp_slug(now: datetime | None = None) -> str:
    return (now or datetime.now(UTC)).strftime("%Y%m%d-%H%M%S")


def default_output_dir(now: datetime | None = None) -> Path:
    return DEFAULT_PROOF_ROOT / f"linux-gate-replay-{timestamp_slug(now)}"


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8", newline="\n")


def _write_manifest(bundle: Path) -> None:
    lines = []
    for path in sorted(p for p in bundle.rglob("*") if p.is_file()):
        if path.name == MANIFEST_NAME:
            continue
        lines.append(f"{_sha256(path)}  {path.relative_to(bundle).as_posix()}")
    _write_text(bundle / MANIFEST_NAME, "\n".join(lines) + "\n")


def _manifest_entries(bundle: Path) -> dict[str, str] | None:
    manifest = bundle / MANIFEST_NAME
    try:
        rows = manifest.read_text(encoding="utf-8").splitlines()
    except OSError:
        return None
    entries: dict[str, str] = {}
    for row in rows:
        if not row.strip():
            continue
        parts = row.split("  ", 1)
        if len(parts) != 2:
            return None
        digest, rel = parts
        target = bundle / rel
        if not target.is_file() or _sha256(target) != digest:
            return None
        entries[rel] = digest
    return entries


def _child_gate_verified(stdout_text: str) -> bool:
    try:
        data = json.loads(stdout_text)
    except json.JSONDecodeError:
        return False
    if not isinstance(data, dict):
        return False
    if data.get("ok") is False or data.get("verified") is False:
        return False
    if data.get("ok") is True or data.get("verified") is True:
        return True
    return data.get("status") in {"passed", "verified"}


def _run_gate(spec: GateSpec, bundle: Path, runner: Runner) -> dict[str, object]:
    script = ROOT / "scripts" / spec.script
    args = [sys.executable, str(script), "--gate", "--format", "json"]
    result = runner(args, cwd=str(ROOT), text=True, capture_output=True)
    stdout_file = f"commands/{spec.id}-stdout.json"
    stderr_file = f"commands/{spec.id}-stderr.txt"
    _write_text(bundle / stdout_file, result.stdout)
    _write_text(bundle / stderr_file, result.stderr)
    verified = result.returncode == 0 and _child_gate_verified(result.stdout)
    return {
        "id": spec.id,
        "label": spec.label,
        "script": spec.script,
        "display_args": args,
        "exit_code": result.returncode,
        "status": "passed" if verified else "failed",
        "verified": verified,
        "stdout_file": stdout_file,
        "stderr_file": stderr_file,
    }


def record_replay(output_dir: Path | None = None, *, runner: Runner = subprocess.run) -> Path:
    bundle = output_dir or default_output_dir()
    bundle.mkdir(parents=True, exist_ok=True)
    gates = [_run_gate(spec, bundle, runner) for spec in GATE_SPECS]
    all_passed = all(gate.get("verified") is True and gate.get("status") == "passed" for gate in gates)
    data = {
        "schema": SCHEMA,
        "generated_at": datetime.now(UTC).isoformat(),
        "status": "passed" if all_passed else "failed",
        "platform_tier": PLATFORM_TIER,
        "source_included": False,
        "provider_api_called": False,
        "all_current_linux_gates_replayed": all_passed,
        "wsl_is_enforcement": False,
        "linux_enforcement_proven": False,
        "linux_seccomp_proven": False,
        "os_sandbox_enforcement_proven": False,
        "clean_linux_desktop_proven": False,
        "non_wsl_linux_desktop_proven": False,
        "signed_release_proven": False,
        "production_readiness_claimed": False,
        "v1_readiness_claimed": False,
        "gate_count": len(gates),
        "passed_gate_count": sum(1 for gate in gates if gate.get("verified") is True),
        "gates": gates,
        "honest_scope": list(REQUIRED_HONEST_SCOPE),
    }
    summary = bundle / SUMMARY_NAME
    _write_text(summary, json.dumps(data, indent=2) + "\n")
    _write_text(bundle / MARKDOWN_NAME, render_markdown(data))
    _write_manifest(bundle)
    return summary


def verify_bundle(summary_path: Path) -> bool:
    try:
        data = json.loads(summary_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return False
    bundle = summary_path.parent
    manifest = _manifest_entries(bundle)
    if manifest is None:
        return False
    if SUMMARY_NAME not in manifest or MARKDOWN_NAME not in manifest:
        return False
    if data.get("schema") != SCHEMA or data.get("status") != "passed":
        return False
    if data.get("platform_tier") != PLATFORM_TIER:
        return False
    if data.get("all_current_linux_gates_replayed") is not True:
        return False
    for key in [
        "source_included",
        "provider_api_called",
        "wsl_is_enforcement",
        "linux_enforcement_proven",
        "linux_seccomp_proven",
        "os_sandbox_enforcement_proven",
        "clean_linux_desktop_proven",
        "non_wsl_linux_desktop_proven",
        "signed_release_proven",
        "production_readiness_claimed",
        "v1_readiness_claimed",
    ]:
        if data.get(key) is not False:
            return False
    scope = " ".join(str(item) for item in data.get("honest_scope", []))
    if not all(anchor in scope for anchor in REQUIRED_HONEST_SCOPE):
        return False
    all_text = json.dumps(data, sort_keys=True).lower()
    if any(claim.lower() in all_text for claim in FORBIDDEN_CLAIMS):
        return False
    gates = data.get("gates")
    if not isinstance(gates, list):
        return False
    if [gate.get("id") for gate in gates if isinstance(gate, dict)] != [spec.id for spec in GATE_SPECS]:
        return False
    by_id = {spec.id: spec for spec in GATE_SPECS}
    for gate in gates:
        if not isinstance(gate, dict):
            return False
        spec = by_id.get(str(gate.get("id")))
        if spec is None or gate.get("script") != spec.script:
            return False
        if gate.get("status") != "passed" or gate.get("verified") is not True:
            return False
        if int(gate.get("exit_code", 1)) != 0:
            return False
        stdout_file = gate.get("stdout_file")
        stderr_file = gate.get("stderr_file")
        if not isinstance(stdout_file, str) or not isinstance(stderr_file, str):
            return False
        if stdout_file not in manifest or stderr_file not in manifest:
            return False
        try:
            stdout_text = (bundle / stdout_file).read_text(encoding="utf-8")
        except OSError:
            return False
        if not _child_gate_verified(stdout_text):
            return False
    return True


def read_committed_evidence(root: Path = ROOT) -> LinuxGateReplayEvidence:
    if (root / SUMMARY_NAME).is_file():
        summaries = [root / SUMMARY_NAME]
    else:
        summaries = sorted((root / "proofs" / "linux" / "execution" / "studio-gate-replay").glob(f"*/{SUMMARY_NAME}"))
        if not summaries:
            summaries = sorted(root.glob(f"**/{SUMMARY_NAME}"))
    for summary in reversed(summaries):
        if verify_bundle(summary):
            return LinuxGateReplayEvidence(
                status="verified",
                verified=True,
                reason=(
                    "Consolidated replay of all current Linux/Tauri gates verified at "
                    f"`{summary.as_posix()}` ({len(GATE_SPECS)} gates)."
                ),
                bundle=summary.parent.as_posix(),
                deferred=[
                    "WSL/WSLg is execution/portability only",
                    "not clean/non-WSL Linux desktop proof",
                    "not Linux seccomp or OS-sandbox enforcement",
                    "not signed, production, or v1.0 readiness",
                ],
            )
    return LinuxGateReplayEvidence(
        status="missing",
        verified=False,
        reason="No committed consolidated Linux/Tauri gate replay bundle verified.",
        bundle=None,
        deferred=[
            "record with `scripts/smoke_garnet_studio_linux_gate_replay.py --record`",
            "WSL/WSLg remains portability only, not Linux enforcement",
        ],
    )


def render_markdown(data: dict[str, object]) -> str:
    gates = data.get("gates", [])
    lines = [
        "# Garnet Studio Linux/Tauri Gate Replay Proof",
        "",
        f"- status: `{data.get('status')}`",
        f"- platform tier: `{data.get('platform_tier')}`",
        f"- all current Linux gates replayed: `{str(data.get('all_current_linux_gates_replayed')).lower()}`",
        "",
        "## Gates",
        "",
        "| Gate | Status | Script |",
        "|---|---|---|",
    ]
    if isinstance(gates, list):
        for gate in gates:
            if isinstance(gate, dict):
                lines.append(f"| `{gate.get('id')}` | `{gate.get('status')}` | `{gate.get('script')}` |")
    lines.extend(
        [
            "",
            "## Honest Scope",
            "",
            *[f"- {item}" for item in data.get("honest_scope", []) if isinstance(item, str)],
            "",
        ]
    )
    return "\n".join(lines)


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--record", action="store_true", help="Replay the current Linux/Tauri gates and write a proof bundle")
    parser.add_argument("--gate", action="store_true", help="Exit non-zero unless committed replay evidence verifies")
    parser.add_argument("--output-dir", type=Path, default=None, help="Proof output directory for --record")
    parser.add_argument("--format", choices=["json", "md"], default="md")
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv)
    summary: Path | None = None
    if args.record:
        summary = record_replay(args.output_dir)
    evidence = read_committed_evidence(ROOT)
    ok = evidence.verified
    if args.format == "json":
        payload = {
            "ok": ok,
            "verified": ok,
            "status": evidence.status,
            "recorded": str(summary) if summary else "",
            "bundle": evidence.bundle or "",
            "reason": evidence.reason,
            "deferred": evidence.deferred,
        }
        print(json.dumps(payload, indent=2))
    else:
        print(("# Garnet Studio Linux/Tauri Gate Replay Status\n\n"))
        print(f"- status: `{evidence.status}`")
        print(f"- verified: `{str(evidence.verified).lower()}`")
        print(f"- reason: {evidence.reason}")
        print("\n## Deferred\n")
        for item in evidence.deferred:
            print(f"- {item}")
    if args.gate and not ok:
        return 1
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
