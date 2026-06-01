#!/usr/bin/env python3
"""Windows release-tooling status reporter (S88).

S88 inventories the external release tools that were absent during the Windows
audit: `cosign`, `syft`, `cyclonedx`, and `wasmtime`. The reporter is deliberately
honest:

- absent tools are reported as **pending-infra**, not as failures and never as
  evidence;
- present tools must run a concrete probe command successfully before they are
  marked verified;
- a present-but-failing tool blocks the gate because the local machine cannot
  honestly claim that lane is runnable.

## Honest scope (do not soften)
This reporter proves local tool availability/runnability only. It does not sign a
Garnet release artifact, publish an SBOM, or integrate Wasmtime fuel/epoch
metering into Garnet runtime. It never stamps signed/SBOM/fuel without the tool.
"""
from __future__ import annotations

import argparse
import os
import json
import shutil
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Callable

ABSENT_SUMMARY = "tools absent — not verified here"


@dataclass
class ProbeResult:
    returncode: int
    stdout: str
    stderr: str


@dataclass
class ToolStatus:
    name: str
    purpose: str
    executable: str | None
    probe: list[str]
    state: str  # absent | verified | failed
    evidence: str


@dataclass
class ReleaseToolingStatus:
    schema: str
    tools: list[ToolStatus]
    summary: str
    ok: bool


RunFn = Callable[..., ProbeResult]
WhichFn = Callable[[str], str | None]
CandidateFn = Callable[[str], list[str]]


def _run(argv: list[str], **kwargs) -> ProbeResult:
    proc = subprocess.run(
        argv,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        timeout=kwargs.get("timeout", 30),
        env=kwargs.get("env"),
    )
    return ProbeResult(proc.returncode, proc.stdout, proc.stderr)


def _tool_plan() -> list[tuple[str, str, list[str]]]:
    return [
        (
            "cosign",
            "Sigstore verify-blob lane for external supply-chain attestation",
            ["cosign", "verify-blob", "--help"],
        ),
        (
            "syft",
            "SBOM generation lane",
            ["syft", "--version"],
        ),
        (
            "cyclonedx",
            "CycloneDX SBOM compatibility lane",
            ["cyclonedx", "--version"],
        ),
        (
            "wasmtime",
            "WASI/fuel/epoch host lane for bounded-execution follow-up",
            ["wasmtime", "--help"],
        ),
    ]


def _default_candidates(name: str) -> list[str]:
    paths: list[Path] = []
    local = Path.home() / "AppData" / "Local" / "Microsoft" / "WinGet" / "Packages"
    program_files = Path("C:/Program Files")
    aliases = {
        "cosign": ["cosign.exe", "cosign-windows-amd64.exe"],
        "syft": ["syft.exe"],
        "cyclonedx": ["cyclonedx.exe"],
        "wasmtime": ["wasmtime.exe"],
    }.get(name, [f"{name}.exe"])

    if local.is_dir():
        for alias in aliases:
            paths.extend(local.glob(f"**/{alias}"))
    if name == "wasmtime":
        paths.append(program_files / "Wasmtime" / "bin" / "wasmtime.exe")

    return [str(p) for p in paths if p.is_file()]


def _resolve_tool(name: str, which: WhichFn, candidates: CandidateFn) -> tuple[str | None, bool]:
    found = which(name)
    if found is not None:
        return found, False
    for candidate in candidates(name):
        if Path(candidate).is_file() or "/" in candidate or "\\" in candidate:
            return candidate, True
    return None, False


def _failure(name: str, exe: str, command: list[str], result: ProbeResult) -> ToolStatus:
    stderr = (result.stderr or result.stdout or "probe failed").strip()
    return ToolStatus(
        name=name,
        purpose=dict((n, p) for n, p, _ in _tool_plan()).get(name, ""),
        executable=exe,
        probe=command,
        state="failed",
        evidence=f"{' '.join(command)} exited {result.returncode}: {stderr}",
    )


def _cosign_probe(name: str, purpose: str, exe: str, run: RunFn) -> ToolStatus:
    with TemporaryDirectory(prefix="garnet-s88-cosign-") as td:
        root = Path(td)
        blob = root / "blob.txt"
        config = root / "signing-config.json"
        prefix = root / "cosign"
        bundle = root / "blob.bundle.json"
        blob.write_text("garnet-s88-release-tooling-proof\n", encoding="utf-8")
        env = os.environ.copy()
        env["COSIGN_PASSWORD"] = ""
        commands = [
            [
                exe,
                "signing-config",
                "create",
                "--no-default-fulcio",
                "--no-default-oidc",
                "--no-default-rekor",
                "--no-default-tsa",
                "--out",
                str(config),
            ],
            [exe, "generate-key-pair", "--output-key-prefix", str(prefix)],
            [
                exe,
                "sign-blob",
                "--key",
                str(prefix.with_suffix(".key")),
                "--signing-config",
                str(config),
                "--bundle",
                str(bundle),
                str(blob),
                "--yes",
            ],
            [
                exe,
                "verify-blob",
                "--key",
                str(prefix.with_suffix(".pub")),
                "--bundle",
                str(bundle),
                "--insecure-ignore-tlog",
                str(blob),
            ],
        ]
        for command in commands:
            result = run(command, env=env)
            if result.returncode != 0:
                return _failure(name, exe, command, result)
        return ToolStatus(
            name=name,
            purpose=purpose,
            executable=exe,
            probe=commands[-1],
            state="verified",
            evidence="local cosign sign-blob + verify-blob proof exited 0 (tlog intentionally disabled for offline local key proof)",
        )


def _syft_probe(name: str, purpose: str, exe: str, run: RunFn) -> ToolStatus:
    with TemporaryDirectory(prefix="garnet-s88-syft-") as td:
        root = Path(td)
        (root / "demo.txt").write_text("garnet-s88-sbom-proof\n", encoding="utf-8")
        command = [exe, "scan", f"dir:{root}", "-o", "cyclonedx-json"]
        result = run(command)
        if result.returncode != 0:
            return _failure(name, exe, command, result)
        return ToolStatus(
            name=name,
            purpose=purpose,
            executable=exe,
            probe=command,
            state="verified",
            evidence="syft scan of temp directory emitted CycloneDX JSON",
        )


def _cyclonedx_probe(name: str, purpose: str, exe: str, run: RunFn) -> ToolStatus:
    with TemporaryDirectory(prefix="garnet-s88-cyclonedx-") as td:
        bom = Path(td) / "bom.json"
        bom.write_text(
            '{"bomFormat":"CycloneDX","specVersion":"1.6","version":1,"components":[]}\n',
            encoding="utf-8",
        )
        command = [
            exe,
            "validate",
            "--input-file",
            str(bom),
            "--input-format",
            "json",
            "--fail-on-errors",
        ]
        result = run(command)
        if result.returncode != 0:
            return _failure(name, exe, command, result)
        return ToolStatus(
            name=name,
            purpose=purpose,
            executable=exe,
            probe=command,
            state="verified",
            evidence="cyclonedx validate accepted a minimal CycloneDX 1.6 BOM",
        )


def _wasmtime_probe(name: str, purpose: str, exe: str, run: RunFn) -> ToolStatus:
    with TemporaryDirectory(prefix="garnet-s88-wasmtime-") as td:
        module = Path(td) / "tiny.wat"
        module.write_text('(module (func (export "run") (result i32) i32.const 7))\n', encoding="utf-8")
        commands = [
            [exe, "-W", "fuel=1000", "--invoke", "run", str(module)],
            [exe, "-W", "epoch-interruption=y", "-W", "timeout=1s", "--invoke", "run", str(module)],
        ]
        for command in commands:
            result = run(command)
            if result.returncode != 0:
                return _failure(name, exe, command, result)
        return ToolStatus(
            name=name,
            purpose=purpose,
            executable=exe,
            probe=commands[-1],
            state="verified",
            evidence="wasmtime ran a tiny WAT module with fuel and epoch/timeout settings",
        )


def _verified_probe(name: str, purpose: str, exe: str, run: RunFn) -> ToolStatus:
    if name == "cosign":
        return _cosign_probe(name, purpose, exe, run)
    if name == "syft":
        return _syft_probe(name, purpose, exe, run)
    if name == "cyclonedx":
        return _cyclonedx_probe(name, purpose, exe, run)
    if name == "wasmtime":
        return _wasmtime_probe(name, purpose, exe, run)
    command = [exe, "--version"]
    result = run(command)
    if result.returncode != 0:
        return _failure(name, exe, command, result)
    return ToolStatus(
        name=name,
        purpose=purpose,
        executable=exe,
        probe=command,
        state="verified",
        evidence=f"{' '.join(command)} exited 0",
    )


def _one_tool(
    name: str,
    purpose: str,
    probe: list[str],
    *,
    which: WhichFn,
    run: RunFn,
    candidates: CandidateFn,
) -> ToolStatus:
    exe, from_candidate = _resolve_tool(name, which, candidates)
    if exe is None:
        return ToolStatus(
            name=name,
            purpose=purpose,
            executable=None,
            probe=probe,
            state="absent",
            evidence=f"{name} absent; {ABSENT_SUMMARY}",
        )

    executable = exe if from_candidate else name
    return _verified_probe(name, purpose, executable, run)


def read_status(
    *,
    which: WhichFn = shutil.which,
    run: RunFn = _run,
    candidates: CandidateFn = _default_candidates,
) -> ReleaseToolingStatus:
    tools = [
        _one_tool(name, purpose, probe, which=which, run=run, candidates=candidates)
        for name, purpose, probe in _tool_plan()
    ]
    failed = [t.name for t in tools if t.state == "failed"]
    verified = [t.name for t in tools if t.state == "verified"]
    absent = [t.name for t in tools if t.state == "absent"]

    if failed:
        summary = f"tool probes failed: {', '.join(failed)}"
        ok = False
    elif not verified:
        summary = ABSENT_SUMMARY
        ok = True
    elif absent:
        summary = (
            f"verified here: {', '.join(verified)}; absent/pending-infra: "
            f"{', '.join(absent)}"
        )
        ok = True
    else:
        summary = "all external release tools verified here"
        ok = True

    return ReleaseToolingStatus(
        schema="garnet.release_tooling_status/v1",
        tools=tools,
        summary=summary,
        ok=ok,
    )


def render_markdown(status: ReleaseToolingStatus) -> str:
    lines = [
        "# Garnet release tooling status (S88)",
        "",
        f"_Schema {status.schema}._",
        "",
        f"Summary: **{status.summary}**.",
        "",
        "| tool | state | purpose | evidence |",
        "|---|---|---|---|",
    ]
    for tool in status.tools:
        lines.append(
            f"| `{tool.name}` | {tool.state} | {tool.purpose} | {tool.evidence} |"
        )
    lines += [
        "",
        "Honest scope: this reporter proves local tool availability/runnability only; "
        "it does not sign a Garnet release artifact, publish an SBOM, or integrate "
        "Wasmtime fuel/epoch metering into Garnet runtime. It never stamps "
        "signed/SBOM/fuel without the tool.",
        "",
    ]
    return "\n".join(lines)


def main(
    argv: list[str] | None = None,
    *,
    which: WhichFn = shutil.which,
    run: RunFn = _run,
    candidates: CandidateFn = _default_candidates,
) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero only when a present tool's probe fails; absent tools are honest pending-infra",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    status = read_status(which=which, run=run, candidates=candidates)
    print(render_markdown(status) if args.format == "md" else json.dumps(asdict(status), indent=2))

    if args.gate and not status.ok:
        print(f"release-tooling gate FAILED: {status.summary}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
