#!/usr/bin/env python3
"""Run the Garnet Studio cross-platform domain proof matrix.

The matrix is intentionally evidence-first: it runs the current Garnet CLI over
the executable example corpus, records stdout/stderr for each parse/check/run
step, and writes a manifest-backed bundle. It does not include source by
default and it does not call provider APIs.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import shlex
import shutil
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Iterable, Sequence

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_DOGFOOD_ROOT = Path.home() / "Desktop" / "dogfood" / "garnet-studio-domain-matrix"
SCHEMA = "garnet.studio.domain_matrix.v1"
TRUST_MISMATCH_MARKER = "BLAKE3 fingerprint mismatch"


@dataclass(frozen=True)
class DomainCase:
    id: str
    label: str
    group: str
    file: str
    run: bool = True
    expected_run_failure: bool = False
    expected_stderr_contains: str | None = None


@dataclass(frozen=True)
class CommandEvidence:
    step: str
    argv: list[str]
    exit_code: int
    status: str
    stdout_file: str
    stderr_file: str
    expected_failure: bool = False
    expectation: str = "exit 0"


@dataclass(frozen=True)
class CaseEvidence:
    id: str
    label: str
    group: str
    file: str
    repo_relative_file: str
    source_sha256: str | None
    status: str
    commands: list[CommandEvidence]


CORE_12_CASES: tuple[DomainCase, ...] = (
    DomainCase("mvp_01_os_simulator", "Cooperative scheduler simulation", "core-mvp", "examples/mvp_01_os_simulator.garnet"),
    DomainCase("mvp_02_relational_db", "In-memory relational query score", "core-mvp", "examples/mvp_02_relational_db.garnet"),
    DomainCase("mvp_03_compiler_bootstrap", "Mini expression evaluator", "core-mvp", "examples/mvp_03_compiler_bootstrap.garnet"),
    DomainCase("mvp_04_numerical_solver", "Iterative convergence solver", "core-mvp", "examples/mvp_04_numerical_solver.garnet"),
    DomainCase("mvp_05_web_app", "Route dispatch logic", "core-mvp", "examples/mvp_05_web_app.garnet"),
    DomainCase("mvp_06_multi_agent", "Deterministic researcher pipeline", "core-mvp", "examples/mvp_06_multi_agent.garnet"),
    DomainCase("mvp_07_game_server", "Game tick simulation", "core-mvp", "examples/mvp_07_game_server.garnet"),
    DomainCase("mvp_08_distributed_kv", "Vector-clock merge scoring", "core-mvp", "examples/mvp_08_distributed_kv.garnet"),
    DomainCase("mvp_09_graph_db", "Graph traversal score", "core-mvp", "examples/mvp_09_graph_db.garnet"),
    DomainCase("mvp_10_terminal_ui", "Terminal widget layout score", "core-mvp", "examples/mvp_10_terminal_ui.garnet"),
    DomainCase("mvp_11_signed_hotreload", "Signed hot-reload success path", "trust-boundary", "examples/mvp_11_signed_hotreload.garnet"),
    DomainCase(
        "mvp_11_signed_hotreload_mismatch",
        "Signed hot-reload mismatch rejection",
        "trust-boundary",
        "examples/mvp_11_signed_hotreload_mismatch.garnet",
        expected_run_failure=True,
        expected_stderr_contains=TRUST_MISMATCH_MARKER,
    ),
)

AGENTIC_CASES: tuple[DomainCase, ...] = (
    DomainCase("agent_toolbelt_01_triage_router", "Agent triage routing", "agent-toolbelt", "examples/agent_toolbelt_01_triage_router.garnet"),
    DomainCase("agent_toolbelt_02_capability_budget", "Capability budget scoring", "agent-toolbelt", "examples/agent_toolbelt_02_capability_budget.garnet"),
    DomainCase("agent_toolbelt_03_memory_recall", "Memory recall ranking", "agent-toolbelt", "examples/agent_toolbelt_03_memory_recall.garnet"),
    DomainCase("agent_toolbelt_04_release_gate", "Release evidence gate", "agent-toolbelt", "examples/agent_toolbelt_04_release_gate.garnet"),
    DomainCase("agent_toolbelt_05_repair_planner", "Repair planner prioritization", "agent-toolbelt", "examples/agent_toolbelt_05_repair_planner.garnet"),
    DomainCase("multi_agent_builder", "Multi-agent build pipeline", "agentic-design", "examples/multi_agent_builder.garnet"),
    DomainCase("agentic_log_analyzer", "Agentic log analyzer", "agentic-design", "examples/agentic_log_analyzer.garnet"),
    DomainCase("safe_io_layer", "Safe IO boundary layer", "agentic-design", "examples/safe_io_layer.garnet"),
)


def timestamp_slug(now: datetime | None = None) -> str:
    current = now or datetime.now(timezone.utc)
    return current.strftime("%Y%m%d-%H%M%S")


def default_output_dir(now: datetime | None = None) -> Path:
    return DEFAULT_DOGFOOD_ROOT / f"garnet-studio-domain-matrix-{timestamp_slug(now)}"


def select_cases(suite: str) -> list[DomainCase]:
    if suite == "core12":
        return list(CORE_12_CASES)
    if suite == "agentic":
        return list(AGENTIC_CASES)
    if suite == "all":
        return list(CORE_12_CASES) + list(AGENTIC_CASES)
    raise ValueError(f"unknown suite: {suite}")


def locate_garnet(configured: str | None = None) -> list[str]:
    if configured:
        return [configured]

    env_value = os.environ.get("GARNET_CLI")
    if env_value:
        return split_cli_command(env_value)

    executable = "garnet.exe" if os.name == "nt" else "garnet"
    for profile in ("release", "debug"):
        candidate = ROOT / "target" / profile / executable
        if candidate.exists():
            return [str(candidate)]

    installed = shutil.which("garnet")
    if installed:
        return [installed]

    raise FileNotFoundError(
        "Could not find Garnet CLI. Build it with `cargo build -p garnet-cli` "
        "or pass --garnet /path/to/garnet."
    )


def split_cli_command(value: str) -> list[str]:
    stripped = value.strip()
    if not stripped:
        return []

    if Path(stripped).exists():
        return [stripped]

    if len(stripped) >= 2 and stripped[0] == stripped[-1] and stripped[0] in "\"'":
        unquoted = stripped[1:-1]
        if Path(unquoted).exists():
            return [unquoted]

    return shlex.split(stripped, posix=True)


def build_cli() -> None:
    subprocess.run(["cargo", "build", "-p", "garnet-cli"], cwd=ROOT, check=True)


def _case_path(case: DomainCase, repo_root: Path = ROOT) -> Path:
    path = Path(case.file)
    return path if path.is_absolute() else repo_root / path


def _step_plan(case: DomainCase, source: Path) -> list[tuple[str, list[str]]]:
    steps = [
        ("parse", ["parse", str(source)]),
        ("check", ["check", str(source)]),
    ]
    if case.run:
        steps.append(("run", ["run", str(source)]))
    return steps


def _write_command_files(output_dir: Path, case_id: str, step: str, stdout: str, stderr: str) -> tuple[str, str]:
    commands_dir = output_dir / "commands"
    commands_dir.mkdir(parents=True, exist_ok=True)
    stdout_path = commands_dir / f"{case_id}-{step}-stdout.txt"
    stderr_path = commands_dir / f"{case_id}-{step}-stderr.txt"
    stdout_path.write_text(stdout, encoding="utf-8")
    stderr_path.write_text(stderr, encoding="utf-8")
    return (
        stdout_path.relative_to(output_dir).as_posix(),
        stderr_path.relative_to(output_dir).as_posix(),
    )


def _command_status(case: DomainCase, step: str, exit_code: int, stdout: str, stderr: str) -> tuple[str, str, bool]:
    if step == "run" and case.expected_run_failure:
        marker = case.expected_stderr_contains or ""
        marker_found = marker in stderr or marker in stdout
        if exit_code != 0 and marker_found:
            return "passed", f"nonzero exit with `{marker}`", True
        return "failed", f"nonzero exit with `{marker}`", True
    if exit_code == 0:
        return "passed", "exit 0", False
    return "failed", "exit 0", False


def run_case(
    case: DomainCase,
    *,
    garnet_argv: Sequence[str],
    output_dir: Path,
    repo_root: Path = ROOT,
) -> CaseEvidence:
    source = _case_path(case, repo_root)
    commands: list[CommandEvidence] = []
    try:
        repo_relative_file = source.relative_to(repo_root).as_posix()
    except ValueError:
        repo_relative_file = case.file

    if not source.exists():
        stdout_file, stderr_file = _write_command_files(output_dir, case.id, "source", "", f"missing source: {source}\n")
        commands.append(
            CommandEvidence(
                step="source",
                argv=[],
                exit_code=1,
                status="failed",
                stdout_file=stdout_file,
                stderr_file=stderr_file,
                expectation="source file exists",
            )
        )
        return CaseEvidence(
            case.id,
            case.label,
            case.group,
            str(source),
            repo_relative_file,
            None,
            "failed",
            commands,
        )

    source_sha256 = hashlib.sha256(source.read_bytes()).hexdigest()

    for step, step_args in _step_plan(case, source):
        argv = list(garnet_argv) + step_args
        completed = subprocess.run(
            argv,
            cwd=repo_root,
            text=True,
            capture_output=True,
            check=False,
        )
        stdout_file, stderr_file = _write_command_files(
            output_dir,
            case.id,
            step,
            completed.stdout,
            completed.stderr,
        )
        status, expectation, expected_failure = _command_status(
            case,
            step,
            completed.returncode,
            completed.stdout,
            completed.stderr,
        )
        commands.append(
            CommandEvidence(
                step=step,
                argv=argv,
                exit_code=completed.returncode,
                status=status,
                stdout_file=stdout_file,
                stderr_file=stderr_file,
                expected_failure=expected_failure,
                expectation=expectation,
            )
        )

    case_status = "passed" if all(command.status == "passed" for command in commands) else "failed"
    return CaseEvidence(
        case.id,
        case.label,
        case.group,
        str(source),
        repo_relative_file,
        source_sha256,
        case_status,
        commands,
    )


def run_matrix(
    suite: str,
    *,
    output_dir: Path,
    garnet_argv: Sequence[str],
    cases: Sequence[DomainCase] | None = None,
    now: datetime | None = None,
    repo_root: Path = ROOT,
) -> dict:
    output_dir.mkdir(parents=True, exist_ok=True)
    selected = list(cases) if cases is not None else select_cases(suite)
    case_results = [
        run_case(case, garnet_argv=garnet_argv, output_dir=output_dir, repo_root=repo_root)
        for case in selected
    ]
    passed_cases = [case for case in case_results if case.status == "passed"]
    failed_cases = [case for case in case_results if case.status != "passed"]
    command_count = sum(len(case.commands) for case in case_results)
    passed_commands = sum(
        1 for case in case_results for command in case.commands if command.status == "passed"
    )
    summary = {
        "schema": SCHEMA,
        "created_at": (now or datetime.now(timezone.utc)).isoformat(),
        "source": str(repo_root),
        "suite": suite,
        "status": "passed" if not failed_cases else "failed",
        "platform": platform.system().lower(),
        "arch": platform.machine(),
        "garnet_command": list(garnet_argv),
        "case_count": len(case_results),
        "passed_cases": len(passed_cases),
        "failed_cases": len(failed_cases),
        "command_count": command_count,
        "passed_commands": passed_commands,
        "failed_commands": command_count - passed_commands,
        "source_included": False,
        "provider_api_called": False,
        "core12_case_count": len(CORE_12_CASES),
        "agentic_case_count": len(AGENTIC_CASES),
        "cases": [_case_to_json(case) for case in case_results],
    }
    (output_dir / "garnet-studio-domain-matrix.json").write_text(
        json.dumps(summary, indent=2) + "\n",
        encoding="utf-8",
    )
    (output_dir / "garnet-studio-domain-matrix.md").write_text(
        render_markdown(summary),
        encoding="utf-8",
    )
    _write_manifest(output_dir)
    return summary


def _case_to_json(case: CaseEvidence) -> dict:
    return {
        "id": case.id,
        "label": case.label,
        "group": case.group,
        "file": case.file,
        "repo_relative_file": case.repo_relative_file,
        "source_sha256": case.source_sha256,
        "status": case.status,
        "commands": [
            {
                "step": command.step,
                "argv": command.argv,
                "exit_code": command.exit_code,
                "status": command.status,
                "stdout_file": command.stdout_file,
                "stderr_file": command.stderr_file,
                "expected_failure": command.expected_failure,
                "expectation": command.expectation,
            }
            for command in case.commands
        ],
    }


def _write_manifest(directory: Path) -> None:
    lines = []
    for path in sorted(directory.rglob("*")):
        if not path.is_file() or path.name == "MANIFEST.sha256":
            continue
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        lines.append(f"{digest}  {path.relative_to(directory).as_posix()}")
    (directory / "MANIFEST.sha256").write_text("\n".join(lines) + "\n", encoding="utf-8")


def render_markdown(summary: dict) -> str:
    lines = [
        "# Garnet Studio Domain Proof Matrix",
        "",
        f"Status: `{summary['status']}`",
        f"Suite: `{summary['suite']}`",
        f"Platform: `{summary['platform']} {summary['arch']}`",
        f"Cases: `{summary['passed_cases']}/{summary['case_count']}`",
        f"Commands: `{summary['passed_commands']}/{summary['command_count']}`",
        f"Source included: `{str(summary['source_included']).lower()}`",
        f"Provider API called: `{str(summary['provider_api_called']).lower()}`",
        "",
        "## Cases",
        "",
        "| Case | Group | Status | Parse | Check | Run |",
        "| --- | --- | --- | --- | --- | --- |",
    ]
    for case in summary["cases"]:
        steps = {command["step"]: command for command in case["commands"]}
        parse_status = steps.get("parse", steps.get("source", {})).get("status", "n/a")
        check_status = steps.get("check", {}).get("status", "n/a")
        run_status = steps.get("run", {}).get("status", "n/a")
        lines.append(
            f"| `{case['id']}` | {case['group']} | `{case['status']}` | "
            f"`{parse_status}` | `{check_status}` | `{run_status}` |"
        )
    lines.extend(
        [
            "",
            "## Honesty Notes",
            "",
            "- A passed expected-failure case means Garnet rejected the unsafe path with the expected diagnostic.",
            "- This matrix proves current CLI parse/check/run behavior for the selected examples only.",
            "- It does not claim Windows signing, winget, Linux package completion, Windows ARM64, or provider-backed conversion.",
        ]
    )
    return "\n".join(lines) + "\n"


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", choices=("core12", "agentic", "all"), default="all")
    parser.add_argument("--garnet", help="path to the Garnet CLI executable")
    parser.add_argument("--build-cli", action="store_true", help="run `cargo build -p garnet-cli` before the matrix")
    parser.add_argument("--output-dir", type=Path, help="directory for JSON/Markdown/log evidence")
    parser.add_argument("--format", choices=("markdown", "json"), default="markdown")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    if args.build_cli:
        build_cli()
    garnet_argv = locate_garnet(args.garnet)
    output_dir = args.output_dir or default_output_dir()
    summary = run_matrix(args.suite, output_dir=output_dir, garnet_argv=garnet_argv)
    if args.format == "json":
        print(json.dumps(summary, indent=2))
    else:
        print(render_markdown(summary), end="")
        print(f"\nEvidence: {output_dir}", file=sys.stderr)
    return 0 if summary["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
