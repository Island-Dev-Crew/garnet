#!/usr/bin/env python3
"""Report the broader Garnet MIT/productization objective status.

This reporter intentionally differs from `garnet_readiness_status.py`: the
tracked implementation plan can be complete while the larger public-readiness
goal still has distribution, mobile, video, proof, and converter-assist gates.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOMAIN_MATRIX_ROOT_ENV = "GARNET_STUDIO_DOMAIN_MATRIX_ROOT"
DOMAIN_MATRIX_SCHEMA = "garnet.studio.domain_matrix.v1"
DOMAIN_MATRIX_MISMATCH_MARKER = "BLAKE3 fingerprint mismatch"
ULTRAPUNCH_REPRO_SCHEMA = "garnet.ultrapunch.repro.v1"
ULTRAPUNCH_ACCEPT_ARTIFACTS = [
    "capability_manifest.json",
    "diff_caps.txt",
    "seal.json",
    "transparency_log.jsonl",
    "decision.md",
]
ULTRAPUNCH_REPRO_FIXTURES = {
    "baseline.garnet": "garnet-cli/tests/fixtures/ultrapunch/baseline.garnet",
    "accept_proposal.garnet": "garnet-cli/tests/fixtures/ultrapunch/accept_proposal.garnet",
    "reject_widen.garnet": "garnet-cli/tests/fixtures/ultrapunch/reject_widen.garnet",
    "reject_overdepth.garnet": "garnet-cli/tests/fixtures/ultrapunch/reject_overdepth.garnet",
}
DOMAIN_MATRIX_CASES = {
    "mvp_01_os_simulator": "examples/mvp_01_os_simulator.garnet",
    "mvp_02_relational_db": "examples/mvp_02_relational_db.garnet",
    "mvp_03_compiler_bootstrap": "examples/mvp_03_compiler_bootstrap.garnet",
    "mvp_04_numerical_solver": "examples/mvp_04_numerical_solver.garnet",
    "mvp_05_web_app": "examples/mvp_05_web_app.garnet",
    "mvp_06_multi_agent": "examples/mvp_06_multi_agent.garnet",
    "mvp_07_game_server": "examples/mvp_07_game_server.garnet",
    "mvp_08_distributed_kv": "examples/mvp_08_distributed_kv.garnet",
    "mvp_09_graph_db": "examples/mvp_09_graph_db.garnet",
    "mvp_10_terminal_ui": "examples/mvp_10_terminal_ui.garnet",
    "mvp_11_signed_hotreload": "examples/mvp_11_signed_hotreload.garnet",
    "mvp_11_signed_hotreload_mismatch": "examples/mvp_11_signed_hotreload_mismatch.garnet",
    "agent_toolbelt_01_triage_router": "examples/agent_toolbelt_01_triage_router.garnet",
    "agent_toolbelt_02_capability_budget": "examples/agent_toolbelt_02_capability_budget.garnet",
    "agent_toolbelt_03_memory_recall": "examples/agent_toolbelt_03_memory_recall.garnet",
    "agent_toolbelt_04_release_gate": "examples/agent_toolbelt_04_release_gate.garnet",
    "agent_toolbelt_05_repair_planner": "examples/agent_toolbelt_05_repair_planner.garnet",
    "multi_agent_builder": "examples/multi_agent_builder.garnet",
    "agentic_log_analyzer": "examples/agentic_log_analyzer.garnet",
    "safe_io_layer": "examples/safe_io_layer.garnet",
}
sys.path.insert(0, str(ROOT / "scripts"))

from garnet_reporter_io import configure_utf8_stdout  # noqa: E402

configure_utf8_stdout()

import garnet_converter_status  # noqa: E402
import garnet_cap_manifest_standard_status  # noqa: E402
import garnet_linear_effect_status  # noqa: E402
import garnet_paper_vi_exp1_status  # noqa: E402
import garnet_paper_vi_exp3_5k_status  # noqa: E402
import garnet_proof_benchmark_status  # noqa: E402
import garnet_provenance_seal_chain_status  # noqa: E402
import garnet_promo_video_status  # noqa: E402
import garnet_readiness_status  # noqa: E402
import garnet_stdlib_layer_gate  # noqa: E402
import garnet_windows_cross_os_enforcement_proof  # noqa: E402
import garnet_windows_linux_studio_status  # noqa: E402
import smoke_garnet_studio_linux_wsl_deb  # noqa: E402
import smoke_garnet_studio_linux_wsl_deb_install  # noqa: E402
import smoke_garnet_studio_windows_wsl  # noqa: E402


def _promo_probe_skipped_status(exc: BaseException) -> garnet_promo_video_status.PromoVideoStatus:
    reason = f"local promo probe skipped: {exc.__class__.__name__}: {exc}"
    return garnet_promo_video_status.PromoVideoStatus(
        source=str(ROOT),
        status="planned-contract",
        completion_percent=25.0,
        target_duration_seconds=30,
        rendered_video_present=False,
        visual_qa_present=False,
        website_export_present=False,
        public_site_embed_present=False,
        composition_source_present=False,
        visual_identity_locked=False,
        source_surfaces_locked=False,
        current_truth=[
            reason,
            "No local promo artifact completion is claimed from this degraded probe.",
        ],
        required_gates=[
            "visual identity lock",
            "30-second storyboard and shot list",
            "HyperFrames or Remotion composition",
            "rendered MP4 or WebM artifact",
            "visual QA verdict",
            "website-ready export",
            "Desktop dogfood evidence bundle",
            "repo/site copy check for overclaims",
            "human/aesthetic acceptance",
        ],
        completed_gates=[],
        open_gates=[
            "local promo evidence probe",
            "rendered MP4 or WebM artifact",
            "visual QA verdict",
            "website-ready export",
        ],
        locked_assets=[],
        source_surfaces=[],
        composition_source={
            "path": "",
            "design_contract_path": "",
            "tool": "probe-skipped",
            "exists": False,
            "design_contract_exists": False,
            "composition_id": "promo-probe-skipped",
            "duration_seconds": 30,
        },
        storyboard_beats=[],
        production_rules=[
            "Do not claim local promo readiness when the local evidence probe is unavailable."
        ],
        forbidden_claims=[
            "Do not claim a rendered promo video exists from a skipped local probe.",
            "Do not claim full MIT/productization completion.",
        ],
        next_steps=[
            "Run the promo readiness probe again on a machine with writable local temp fixtures."
        ],
    )


def _read_promo_status() -> tuple[garnet_promo_video_status.PromoVideoStatus, str]:
    try:
        return garnet_promo_video_status.read_status(), ""
    except OSError as exc:
        skipped = _promo_probe_skipped_status(exc)
        return skipped, skipped.current_truth[0]


@dataclass(frozen=True)
class ObjectiveLane:
    id: str
    label: str
    status: str
    completion_percent: float
    evidence: str
    blocked_by: list[str]
    deferred: list[str]
    # "committed" lanes are scored from committed repo evidence and are byte-identical
    # on every machine — they feed the headline % and the no-regression gate.
    # "local" lanes derive from machine-specific live probes (~/Desktop bundles, local
    # render/build artifacts, env/certs); they are reported as evidence but NEVER feed
    # the headline % or the gate. (S31-PR2 deterministic-reporter split.)
    evidence_class: str = "committed"


@dataclass(frozen=True)
class MitReadinessStatus:
    source: str
    overall_status: str
    completion_percent: float
    current_truth: list[str]
    lanes: list[ObjectiveLane]


@dataclass(frozen=True)
class DomainMatrixEvidence:
    source_present: bool
    verified: bool
    bundle_json: Path | None
    reason: str
    committed: bool = False


@dataclass(frozen=True)
class UltrapunchReproEvidence:
    verified: bool
    windows_bundle_json: Path | None
    wsl_bundle_json: Path | None
    reason: str


def _lane_score(lane: ObjectiveLane) -> float:
    if lane.status == "verified":
        return 1.0
    if lane.status == "active-partial":
        return 0.5
    if lane.status == "rendered-artifact-ready":
        return 0.65
    if lane.status == "visual-qa-ready":
        return 0.8
    if lane.status == "website-export-ready":
        return 0.9
    if lane.status == "public-site-embedded":
        return 0.95
    if lane.status == "composition-ready":
        return 0.5
    if lane.status == "source-locked":
        return 0.35
    if lane.status == "planned-contract":
        return 0.25
    if lane.status == "source-present":
        return 0.6
    if lane.status == "local-registry-source-ready":
        return 0.85
    if lane.status == "feature-gated-source-ready":
        return 0.85
    if lane.status == "provider-gated-harness":
        return 1.0
    if lane.status == "provider-gated-5k-harness":
        return 1.0
    return 0.0


def _dedupe(items: list[str]) -> list[str]:
    seen: set[str] = set()
    deduped: list[str] = []
    for item in items:
        if item not in seen:
            deduped.append(item)
            seen.add(item)
    return deduped


def _lsp_source_present() -> bool:
    required = [
        ROOT / "garnet-lsp" / "Cargo.toml",
        ROOT / "garnet-lsp" / "src" / "lib.rs",
        ROOT / "garnet-lsp" / "src" / "main.rs",
        ROOT / "editors" / "vscode" / "package.json",
        ROOT / "editors" / "vscode" / "src" / "extension.ts",
    ]
    return all(path.exists() for path in required)


def _cst_layer_present() -> bool:
    required = [
        ROOT / "garnet-parser-v0.3" / "src" / "cst.rs",
        ROOT / "garnet-parser-v0.3" / "tests" / "cst_round_trip.rs",
    ]
    return all(path.exists() for path in required)


def _rowan_cst_present() -> bool:
    required = [
        ROOT / "garnet-cst" / "src" / "builder.rs",
        ROOT / "garnet-cst" / "src" / "convert.rs",
        ROOT / "garnet-cst" / "src" / "nodes.rs",
        ROOT / "garnet-cst" / "tests" / "examples_roundtrip.rs",
        ROOT / "garnet-cst" / "tests" / "cst_to_ast_parity.rs",
        ROOT / "garnet-cst" / "benches" / "parse_cst_vs_ast.rs",
    ]
    return all(path.exists() for path in required)


def _lsp_v02_present() -> bool:
    required = [
        ROOT / "garnet-lsp" / "src" / "lib.rs",
        ROOT / "editors" / "vscode" / "package.json",
        ROOT / "scripts" / "smoke_garnet_lsp_protocol.py",
    ]
    return _cst_layer_present() and all(path.exists() for path in required)


def _lsp_precision_present() -> bool:
    lib = ROOT / "garnet-lsp" / "src" / "lib.rs"
    package = ROOT / "editors" / "vscode" / "package.json"
    smoke = ROOT / "scripts" / "smoke_garnet_lsp_precision.py"
    if not (lib.exists() and package.exists() and smoke.exists()):
        return False
    lib_text = lib.read_text(encoding="utf-8")
    package_text = package.read_text(encoding="utf-8")
    smoke_text = smoke.read_text(encoding="utf-8")
    return (
        "garnet_cst" in lib_text
        and "identifier_spans" in lib_text
        and "capability" in lib_text
        and "garnet-0.7.0-lsp-precision.vsix" in package_text
        and "Refactor long parameter list" in smoke_text
        and "Add return type `Int`" in smoke_text
    )


def _vm_scaffold_present(proof: garnet_proof_benchmark_status.ProofBenchmarkStatus) -> bool:
    return any(
        bench.id == "vm_parse_compile_execute"
        and bench.bench_file_exists
        and bench.cargo_entry_present
        for bench in proof.benchmarks
    )


def _compiler_agent_llm_tier_present() -> bool:
    required = [
        ROOT / "garnet-suggest-llm" / "Cargo.toml",
        ROOT / "garnet-suggest-llm" / "src" / "lib.rs",
        ROOT / "garnet-suggest-llm" / "src" / "llm.rs",
        ROOT / "scripts" / "check_determinism_no_llm.py",
        ROOT / "scripts" / "test_check_determinism_no_llm.py",
        ROOT
        / "benchmarks"
        / "paper_vi_exp3_compiler_as_agent"
        / "run_stateless.sh",
        ROOT
        / "benchmarks"
        / "paper_vi_exp3_compiler_as_agent"
        / "run_history_aware.sh",
        ROOT
        / "benchmarks"
        / "paper_vi_exp3_compiler_as_agent"
        / "aggregate.py",
        ROOT
        / "benchmarks"
        / "paper_vi_exp3_compiler_as_agent"
        / "analyze.py",
    ]
    snapshots = [
        ROOT
        / "benchmarks"
        / "paper_vi_exp3_compiler_as_agent"
        / "codebase_versions"
        / f"v{idx:02d}"
        / "main.garnet"
        for idx in range(1, 11)
    ]
    return all(path.exists() for path in required + snapshots)


def _official_packages_seed_present() -> bool:
    package_names = ["http-client", "llm", "cli", "test-property", "log"]
    registry = ROOT / "examples" / "garnet_lang_registry_seed"
    index_path = registry / "index.json"
    required = [
        ROOT / "tools" / "garnet-lang-template" / "README.md",
        ROOT / "tools" / "garnet-lang-template" / "Garnet.toml",
        ROOT / "tools" / "garnet-lang-template" / "garnet" / "lib.garnet",
        ROOT / "tools" / "garnet-lang-template" / "tests" / "smoke.garnet",
        ROOT / "examples" / "mvp_18_all_official_packages" / "Garnet.toml",
        ROOT / "examples" / "mvp_18_all_official_packages" / "src" / "main.garnet",
        ROOT / "scripts" / "smoke_garnet_lang_packages_seed.py",
        index_path,
    ]
    for package in package_names:
        package_root = registry / package / "0.1.0"
        required.extend(
            [
                package_root / "README.md",
                package_root / "CHANGELOG.md",
                package_root / "Garnet.toml",
                package_root / "lib.garnet",
                package_root / "tests" / "smoke.garnet",
            ]
        )
    if not all(path.exists() for path in required):
        return False
    try:
        index = json.loads(index_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return False
    packages = index.get("packages", {})
    return all("0.1.0" in packages.get(package, {}).get("versions", {}) for package in package_names)


def _domain_matrix_root() -> Path:
    return Path(
        os.environ.get(
            DOMAIN_MATRIX_ROOT_ENV,
            str(Path.home() / "Desktop" / "dogfood" / "garnet-studio-domain-matrix"),
        )
    )


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _verify_manifest(bundle_dir: Path) -> set[str] | None:
    manifest = bundle_dir / "MANIFEST.sha256"
    if not manifest.is_file():
        return None

    try:
        lines = [line.strip() for line in manifest.read_text(encoding="utf-8").splitlines() if line.strip()]
    except OSError:
        return None

    recorded: set[str] = set()
    for line in lines:
        match = re.fullmatch(r"([0-9a-f]{64})  ([^\r\n]+)", line)
        if not match:
            return None
        digest, relative = match.groups()
        relative_path = Path(relative)
        if relative_path.is_absolute() or ".." in relative_path.parts:
            return None
        target = bundle_dir / relative_path
        if not target.is_file():
            return None
        try:
            if _sha256(target) != digest:
                return None
        except OSError:
            return None
        recorded.add(relative.replace("\\", "/"))
    return recorded


def _source_digest_matches(case_id: str, case: dict) -> bool:
    expected_relative = DOMAIN_MATRIX_CASES[case_id]
    source = ROOT / expected_relative
    return (
        case.get("repo_relative_file") == expected_relative
        and source.is_file()
        and case.get("source_sha256") == _sha256(source)
    )


def _manifested_text(bundle_dir: Path, manifest_entries: set[str], relative: str) -> str | None:
    if relative not in manifest_entries:
        return None
    try:
        return (bundle_dir / Path(relative)).read_text(encoding="utf-8")
    except OSError:
        return None


def _domain_matrix_summary_verified(path: Path) -> bool:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return False

    bundle_dir = path.parent
    manifest_entries = _verify_manifest(bundle_dir)
    if manifest_entries is None:
        return False
    required_bundle_files = {
        "garnet-studio-domain-matrix.json",
        "garnet-studio-domain-matrix.md",
    }
    if not required_bundle_files.issubset(manifest_entries):
        return False
    cases = data.get("cases", [])
    if not isinstance(cases, list):
        return False
    by_id = {case.get("id"): case for case in cases if isinstance(case, dict)}
    if set(by_id) != set(DOMAIN_MATRIX_CASES):
        return False

    for case_id, case in by_id.items():
        if case.get("status") != "passed" or not _source_digest_matches(case_id, case):
            return False
        commands = case.get("commands", [])
        if not isinstance(commands, list):
            return False
        by_step = {command.get("step"): command for command in commands if isinstance(command, dict)}
        if set(by_step) != {"parse", "check", "run"}:
            return False
        for step, command in by_step.items():
            stdout_file = command.get("stdout_file")
            stderr_file = command.get("stderr_file")
            if not isinstance(stdout_file, str) or not isinstance(stderr_file, str):
                return False
            if stdout_file not in manifest_entries or stderr_file not in manifest_entries:
                return False
            if command.get("status") != "passed":
                return False
            if step in {"parse", "check"} and command.get("exit_code") != 0:
                return False
            if (
                step == "run"
                and case_id != "mvp_11_signed_hotreload_mismatch"
                and (
                    command.get("exit_code") != 0
                    or command.get("expected_failure") is not False
                )
            ):
                return False

    mismatch_run = next(
        command
        for command in by_id["mvp_11_signed_hotreload_mismatch"]["commands"]
        if command.get("step") == "run"
    )
    mismatch_stdout = _manifested_text(
        bundle_dir,
        manifest_entries,
        mismatch_run.get("stdout_file", ""),
    )
    mismatch_stderr = _manifested_text(
        bundle_dir,
        manifest_entries,
        mismatch_run.get("stderr_file", ""),
    )
    if mismatch_stdout is None or mismatch_stderr is None:
        return False
    mismatch_output = f"{mismatch_stdout}\n{mismatch_stderr}"

    return (
        data.get("schema") == DOMAIN_MATRIX_SCHEMA
        and data.get("suite") == "all"
        and data.get("status") == "passed"
        and data.get("case_count") == len(DOMAIN_MATRIX_CASES)
        and data.get("passed_cases") == len(DOMAIN_MATRIX_CASES)
        and data.get("failed_cases") == 0
        and data.get("command_count") == len(DOMAIN_MATRIX_CASES) * 3
        and data.get("passed_commands") == len(DOMAIN_MATRIX_CASES) * 3
        and data.get("failed_commands") == 0
        and data.get("source_included") is False
        and data.get("provider_api_called") is False
        and mismatch_run.get("status") == "passed"
        and mismatch_run.get("expected_failure") is True
        and mismatch_run.get("exit_code") != 0
        and DOMAIN_MATRIX_MISMATCH_MARKER in mismatch_output
    )


def _verified_domain_matrix_under(root: Path) -> Path | None:
    if root.exists():
        candidates = sorted(
            root.rglob("garnet-studio-domain-matrix.json"),
            key=lambda path: path.stat().st_mtime,
            reverse=True,
        )
        for candidate in candidates:
            if _domain_matrix_summary_verified(candidate):
                return candidate
    return None


def _repo_relative_display(path: Path) -> str:
    try:
        return path.relative_to(ROOT).as_posix()
    except ValueError:
        return path.as_posix()


def _committed_domain_matrix_evidence() -> DomainMatrixEvidence | None:
    windows = _verified_domain_matrix_under(ROOT / "proofs" / "windows" / "domains")
    wsl = _verified_domain_matrix_under(ROOT / "proofs" / "linux" / "execution" / "domains")
    if windows is None or wsl is None:
        return None
    return DomainMatrixEvidence(
        True,
        True,
        windows,
        (
            f"Committed Windows bundle: `{_repo_relative_display(windows)}`. "
            f"Committed WSL portability bundle: `{_repo_relative_display(wsl)}`. "
            "The WSL row is execution/portability evidence only, not Linux seccomp "
            "or OS-sandbox enforcement."
        ),
        committed=True,
    )


def _domain_matrix_evidence() -> DomainMatrixEvidence:
    source_present = (ROOT / "scripts" / "smoke_garnet_studio_domain_matrix.py").is_file()
    if not source_present:
        return DomainMatrixEvidence(False, False, None, "No repo-owned domain matrix script exists yet.")

    committed = _committed_domain_matrix_evidence()
    if committed is not None:
        return committed

    candidate = _verified_domain_matrix_under(_domain_matrix_root())
    if candidate is not None:
        return DomainMatrixEvidence(
            True,
            True,
            candidate,
            f"Verified bundle: `{candidate}`.",
        )

    return DomainMatrixEvidence(
        True,
        False,
        None,
        (
            "Script present, but no verified `--suite all` bundle was found in the "
            f"configured domain-matrix evidence root (`{DOMAIN_MATRIX_ROOT_ENV}` or "
            "`~/Desktop/dogfood/garnet-studio-domain-matrix`). Run "
            "`scripts/smoke_garnet_studio_domain_matrix.py --suite all` and keep the "
            "JSON/Markdown plus `MANIFEST.sha256` evidence bundle."
        ),
    )


def _ultrapunch_source_digests_match(source_files: object) -> bool:
    if not isinstance(source_files, list):
        return False
    by_path = {
        item.get("path"): item.get("sha256")
        for item in source_files
        if isinstance(item, dict)
    }
    for relative in ULTRAPUNCH_REPRO_FIXTURES.values():
        source = ROOT / relative
        if not source.is_file() or by_path.get(relative) != _sha256(source):
            return False
    return True


def _ultrapunch_repro_summary_verified(path: Path) -> bool:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return False

    bundle_dir = path.parent
    manifest_entries = _verify_manifest(bundle_dir)
    if manifest_entries is None:
        return False
    required_files = {
        "garnet-ultrapunch-repro.json",
        "garnet-ultrapunch-repro.md",
        *{f"accept/{artifact}" for artifact in ULTRAPUNCH_ACCEPT_ARTIFACTS},
    }
    if not required_files.issubset(manifest_entries):
        return False

    commands = data.get("commands", [])
    if not isinstance(commands, list) or len(commands) != 4:
        return False
    command_ids = {command.get("id") for command in commands if isinstance(command, dict)}
    if command_ids != {
        "accept-agent-loop",
        "accept-caps-log-verify",
        "reject-widen-agent-loop",
        "reject-overdepth-agent-loop",
    }:
        return False
    for command in commands:
        if not isinstance(command, dict) or command.get("status") != "passed":
            return False
        stdout_file = command.get("stdout_file")
        stderr_file = command.get("stderr_file")
        if not isinstance(stdout_file, str) or not isinstance(stderr_file, str):
            return False
        if stdout_file not in manifest_entries or stderr_file not in manifest_entries:
            return False

    accept = data.get("accept", {})
    reject_widen = data.get("reject_widen", {})
    reject_overdepth = data.get("reject_overdepth", {})
    honest_scope = " ".join(data.get("honest_scope", []))
    markdown = _manifested_text(bundle_dir, manifest_entries, "garnet-ultrapunch-repro.md")
    return (
        data.get("schema") == ULTRAPUNCH_REPRO_SCHEMA
        and data.get("status") == "passed"
        and data.get("source_included") is False
        and data.get("provider_api_called") is False
        and data.get("command_count") == 4
        and data.get("passed_commands") == 4
        and data.get("failed_commands") == 0
        and _ultrapunch_source_digests_match(data.get("source_files"))
        and sorted(accept.get("artifacts", [])) == sorted(ULTRAPUNCH_ACCEPT_ARTIFACTS)
        and accept.get("chain_verified") is True
        and accept.get("sealed") is True
        and reject_widen.get("refused") is True
        and reject_widen.get("sealed") is False
        and reject_widen.get("expected_stage") == "diff-caps"
        and reject_overdepth.get("refused") is True
        and reject_overdepth.get("sealed") is False
        and reject_overdepth.get("expected_stage") == "enforced-kernel"
        and "not seccomp proof" in honest_scope
        and "not OS-sandbox proof" in honest_scope
        and markdown is not None
    )


def _verified_ultrapunch_repro_under(root: Path) -> Path | None:
    if root.exists():
        candidates = sorted(
            root.rglob("garnet-ultrapunch-repro.json"),
            key=lambda path: path.stat().st_mtime,
            reverse=True,
        )
        for candidate in candidates:
            if _ultrapunch_repro_summary_verified(candidate):
                return candidate
    return None


def _committed_ultrapunch_repro_evidence() -> UltrapunchReproEvidence | None:
    windows = _verified_ultrapunch_repro_under(ROOT / "proofs" / "windows" / "ultrapunch")
    wsl = _verified_ultrapunch_repro_under(ROOT / "proofs" / "linux" / "repro")
    if windows is None or wsl is None:
        return None
    return UltrapunchReproEvidence(
        True,
        windows,
        wsl,
        (
            f"Committed Windows ultrapunch bundle: `{_repo_relative_display(windows)}`. "
            f"Committed WSL portability-repro bundle: `{_repo_relative_display(wsl)}`. "
            "The WSL row replays S104 accept/reject decisions as execution/portability "
            "evidence only, not Linux seccomp, OS-sandbox enforcement, or Wasmtime fuel proof."
        ),
    )


def read_status() -> MitReadinessStatus:
    plan = garnet_readiness_status.read_status(
        ROOT / "F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md"
    )
    converter = garnet_converter_status.read_status()
    contract = converter.intelligent_assist_contract
    promo, promo_probe_note = _read_promo_status()
    paper_vi_exp1 = garnet_paper_vi_exp1_status.read_status()
    paper_vi_exp3_5k = garnet_paper_vi_exp3_5k_status.read_status()
    linear_effect = garnet_linear_effect_status.read_status()
    provenance_chain = garnet_provenance_seal_chain_status.read_status()
    cap_manifest_standard = garnet_cap_manifest_standard_status.read_status()
    windows_cross_os = garnet_windows_cross_os_enforcement_proof.read_status()
    proof = garnet_proof_benchmark_status.read_status()
    vm_scaffold_present = _vm_scaffold_present(proof)
    wls = garnet_windows_linux_studio_status.read_status()
    studio_smoke = smoke_garnet_studio_windows_wsl.read_committed_evidence(ROOT)
    linux_deb_package = smoke_garnet_studio_linux_wsl_deb.read_committed_evidence(ROOT)
    linux_deb_install = smoke_garnet_studio_linux_wsl_deb_install.read_committed_evidence(ROOT)
    stdlib = garnet_stdlib_layer_gate.read_status()
    novel_compositions_present = all(
        (ROOT / p).exists()
        for p in (
            "examples/novel_01_capability_budgeted_memory_agent.garnet",
            "examples/novel_02_signed_provenance_pipeline.garnet",
            "examples/novel_03_release_gate_quorum.garnet",
            "scripts/smoke_garnet_novel_compositions.py",
            "scripts/test_garnet_novel_compositions.py",
            "C_Language_Specification/GARNET_NOVEL_COMPOSITIONS.md",
        )
    )
    bridge_src = ROOT / "garnet-interp-v0.3" / "src" / "stdlib_bridge.rs"
    bridge_text = bridge_src.read_text(encoding="utf-8") if bridge_src.exists() else ""
    interp_dispatch_present = (
        (ROOT / "examples/novel_04_dispatched_stdlib_pipeline.garnet").exists()
        and (ROOT / "garnet-interp-v0.3/tests/mnemos_stdlib_combination.rs").exists()
        and bridge_src.exists()
        and "core::math::sqrt" in bridge_text
    )
    parser_path = ROOT / "garnet-parser-v0.3" / "src" / "parser.rs"
    parser_path_text = parser_path.read_text(encoding="utf-8") if parser_path.exists() else ""
    stdlib_memory_dispatch_present = (
        (ROOT / "examples/novel_05_s22_stdlib_memory_pipeline.garnet").exists()
        and (ROOT / "garnet-interp-v0.3/tests/stdlib_s22_dispatch.rs").exists()
        and "std::json::parse" in bridge_text
        and "std::regex::match" in bridge_text
        and "std::uuid::new_v5" in bridge_text
        and "std::env::set" in bridge_text
        and "std::process::spawn" in bridge_text
        and "std::log::info" in bridge_text
        and "memory::working" in bridge_text
        and "expect_path_segment" in parser_path_text
    )
    process_runtime_present = (
        (ROOT / "garnet-interp-v0.3/tests/stdlib_s23_dispatch.rs").exists()
        and "std::process::spawn_args" in bridge_text
        and "std::process::output" in bridge_text
    )
    log_file_sink_present = (
        (ROOT / "garnet-interp-v0.3/tests/stdlib_s24_dispatch.rs").exists()
        and "std::log::to_file" in bridge_text
    )
    host_effect_composition_present = (
        (ROOT / "examples/novel_06_observability_provenance_pipeline.garnet").exists()
        and (ROOT / "garnet-interp-v0.3/tests/host_effect_composition.rs").exists()
    )
    result_dispatch_present = (
        (ROOT / "garnet-interp-v0.3/tests/core_result_dispatch.rs").exists()
        and "core::result::and_then" in bridge_text
    )
    option_dispatch_present = (
        (ROOT / "garnet-interp-v0.3/tests/core_option_dispatch.rs").exists()
        and "core::option::and_then" in bridge_text
    )
    iter_completion_present = (
        (ROOT / "garnet-interp-v0.3/tests/core_iter_completion_dispatch.rs").exists()
        and "core::iter::zip" in bridge_text
        and "core::iter::collect" in bridge_text
        and "core::iter::chain" in bridge_text
    )
    stability_src = ROOT / "garnet-check-v0.3" / "src" / "stability.rs"
    stability_text = (
        stability_src.read_text(encoding="utf-8") if stability_src.exists() else ""
    )
    stability_errors_present = (
        "GARNET_STABILITY_ERRORS" in stability_text and "StabilityError" in stability_text
    )
    functional_core_present = (
        (ROOT / "examples/novel_07_functional_core_pipeline.garnet").exists()
        and (ROOT / "garnet-interp-v0.3/tests/functional_core_composition.rs").exists()
    )
    wls_clean_vm_verified = any(
        gate.id == "windows_unsigned_nsis" and gate.status == "clean-vm-proof-verified"
        for gate in wls.packaging_gates
    )
    domain_matrix = _domain_matrix_evidence()
    ultrapunch_repro = _committed_ultrapunch_repro_evidence()
    lsp_precision_present = _lsp_precision_present()
    if wls_clean_vm_verified:
        wls_completion_percent = (
            76.0
            if domain_matrix.verified and linux_deb_install.verified
            else 75.0
            if domain_matrix.verified and linux_deb_package.verified
            else 70.0
            if domain_matrix.verified
            else 72.0
            if domain_matrix.source_present and linux_deb_package.verified
            else 67.0
            if domain_matrix.source_present
            else 68.0
            if linux_deb_package.verified
            else 65.0
        )
    else:
        wls_completion_percent = (
            64.0
            if domain_matrix.verified and linux_deb_install.verified
            else 63.0
            if domain_matrix.verified and linux_deb_package.verified
            else 60.0
            if domain_matrix.verified
            else 61.0
            if domain_matrix.source_present and linux_deb_package.verified
            else 58.0
            if domain_matrix.source_present
            else 58.0
            if linux_deb_package.verified
            else 55.0
        )
    domain_matrix_tail = (
        "a repo-owned Domain Proof Matrix for the canonical MVP plus agentic examples, "
        if domain_matrix.source_present
        else ""
    )
    wls_evidence_tail = (
        f"readiness reporter parity actions, {domain_matrix_tail}a verified x64 clean-VM installer "
        "proof, and open Linux plus signing, winget, and Windows ARM64 package gates."
        if wls_clean_vm_verified
        else (
            f"readiness reporter parity actions, {domain_matrix_tail}a Windows clean-VM installer "
            "proof contract, and open Linux plus clean-machine package gates."
        )
    )
    if promo.public_site_embed_present:
        promo_evidence_tail = "records local rendered MP4/WebM evidence, automated visual QA evidence, a website export package, and public-site embedding while keeping human/aesthetic acceptance open."
    elif promo.website_export_present:
        promo_evidence_tail = "records local rendered MP4/WebM evidence, automated visual QA evidence, and a website export package while keeping public-site embedding open."
    elif promo.visual_qa_present:
        promo_evidence_tail = "records local rendered MP4/WebM evidence plus automated visual QA evidence, and keeps website export open."
    elif promo.rendered_video_present:
        promo_evidence_tail = "records local rendered MP4/WebM evidence, and keeps visual QA plus website export open."
    else:
        promo_evidence_tail = "preserves that no verified rendered artifact exists."
    if promo_probe_note:
        promo_evidence_tail = (
            f"{promo_probe_note}; no local promo artifact completion is claimed "
            "from this run."
        )
    promo_blockers = ["website-ready export"]
    if promo.public_site_embed_present:
        promo_blockers = ["human/aesthetic acceptance review"]
    elif promo.website_export_present:
        promo_blockers = ["public-site embedding and review"]
    if not promo.visual_qa_present:
        promo_blockers.insert(0, "visual QA verdict")
    if not promo.rendered_video_present:
        promo_blockers.insert(0, "rendered artifact")

    lanes = [
        ObjectiveLane(
            id="tracked_implementation_plan",
            label="Tracked implementation plan",
            status="verified" if plan.completion_percent == 100.0 else "active-partial",
            completion_percent=plan.completion_percent,
            evidence=(
                f"`scripts/garnet_readiness_status.py` reports "
                f"{plan.completed_slices}/{plan.total_slices} slices."
            ),
            blocked_by=[],
            deferred=[] if plan.completion_percent == 100.0 else [item.title for item in plan.open_slices],
        ),
        ObjectiveLane(
            id="agentic_dogfood_matrix",
            label="Agentic dogfood matrix",
            status="verified",
            completion_percent=100.0,
            evidence=(
                "`scripts/run_agentic_dogfood_matrix.py --copy-to-desktop --strict` "
                "covers the current advanced source-checkout domains with Desktop evidence."
            ),
            blocked_by=[],
            deferred=["Use future slices to add domains when product surfaces expand."],
        ),
        ObjectiveLane(
            id="converter_truth",
            label="Converter adoption truth",
            status="verified",
            completion_percent=100.0,
            evidence=(
                "`scripts/garnet_converter_status.py` separates Rust/Ruby/Python/Go active "
                "lanes from planned language and LLM-assist lanes."
            ),
            blocked_by=[],
            deferred=[
                "New deterministic frontends require separate tested slices.",
                "LLM assist requires a secure advisory implementation before activation.",
            ],
        ),
        ObjectiveLane(
            id="macos_studio_dmg",
            label="macOS Studio DMG",
            status="active-partial",
            completion_percent=75.0,
            evidence=(
                "Garnet Studio DMG build, mounted-copy smoke, packaged PWA smoke, and "
                "agentic matrix evidence are active in Desktop dogfood bundles."
            ),
            blocked_by=[
                "No valid local Developer ID Application identity",
                "No notarization profile",
            ],
            deferred=["Clean-machine Gatekeeper install evidence"],
        ),
        ObjectiveLane(
            id="developer_id_notarization",
            label="Developer ID notarization",
            status="blocked",
            completion_percent=0.0,
            evidence=(
                "`scripts/preflight_garnet_studio_notarization.sh --copy-to-desktop` "
                "records the current blockers without claiming notarization; "
                "`scripts/garnet_studio_notarization_status.py` summarizes that "
                "bundle for agents and PR/site evidence."
            ),
            blocked_by=[
                "APPLE_DEV_ID_APP",
                "APPLE_NOTARY_PROFILE",
                "valid Developer ID Application certificate",
                "stapled DMG ticket",
            ],
            deferred=["Signed + notarized macOS distribution"],
        ),
        ObjectiveLane(
            id="windows_linux_distribution",
            evidence_class="local",
            label="Windows/Linux distribution",
            status="active-partial",
            completion_percent=wls_completion_percent,
            evidence=(
                "`scripts/garnet_windows_linux_studio_status.py` now reports the "
                "Tauri v2 shell scaffold in `apps/garnet-studio`, minimal webview "
                "permissions, Windows local release build/smoke evidence, "
                f"{'committed Windows/WSL Studio smoke evidence, ' if studio_smoke.verified else ''}"
                f"{'committed WSL Linux `.deb` package-build/command-smoke evidence, ' if linux_deb_package.verified else ''}"
                f"{'committed WSL Linux `.deb` extract/command-smoke evidence, ' if linux_deb_install.verified else ''}"
                f"v0.5 {wls_evidence_tail}"
            ),
            blocked_by=list(wls.user_assistance_needed),
            deferred=list(wls.next_slices),
        ),
        ObjectiveLane(
            id="windows_wsl_studio_smoke",
            evidence_class="committed",
            label="Windows/WSL Studio smoke proof (S117 increment)",
            status="verified" if studio_smoke.verified else "planned",
            completion_percent=100.0 if studio_smoke.verified else 0.0,
            evidence=(
                "`scripts/smoke_garnet_studio_windows_wsl.py --record` records "
                "manifest-backed Windows Tauri `--studio-smoke` evidence plus a "
                f"WSL command-contract replay. {studio_smoke.reason}"
                if studio_smoke.verified
                else studio_smoke.reason
            ),
            blocked_by=[] if studio_smoke.verified else ["committed Windows + WSL Studio smoke evidence"],
            deferred=[
                "WSL is execution/portability only, not Linux seccomp or OS-sandbox enforcement",
                "Linux desktop GUI launch and native Linux package proof remain open",
                "Signed MSI, winget, Windows ARM64, production, and v1.0 remain unclaimed",
            ],
        ),
        ObjectiveLane(
            id="linux_wsl_studio_deb_package",
            evidence_class="committed",
            label="Linux WSL Studio DEB package proof (S117 increment)",
            status="verified" if linux_deb_package.verified else "planned",
            completion_percent=100.0 if linux_deb_package.verified else 0.0,
            evidence=(
                "`scripts/smoke_garnet_studio_linux_wsl_deb.py --record` records "
                "a WSL-driven Tauri Linux `.deb` build, `dpkg-deb` inspection, and "
                f"non-GUI `--studio-smoke` command evidence. {linux_deb_package.reason}"
                if linux_deb_package.verified
                else linux_deb_package.reason
            ),
            blocked_by=[] if linux_deb_package.verified else ["committed WSL Linux `.deb` proof bundle"],
            deferred=[
                "not Linux desktop GUI launch proof",
                "not Linux seccomp or OS-sandbox enforcement",
                "not clean Linux install proof",
                "not signed, production, or v1.0 readiness",
            ],
        ),
        ObjectiveLane(
            id="linux_wsl_studio_deb_install",
            evidence_class="committed",
            label="Linux WSL Studio DEB install/extract proof (S117 increment)",
            status="verified" if linux_deb_install.verified else "planned",
            completion_percent=100.0 if linux_deb_install.verified else 0.0,
            evidence=(
                "`scripts/smoke_garnet_studio_linux_wsl_deb_install.py --record` records "
                "a WSL-driven Tauri Linux `.deb` build, `dpkg-deb --extract`, and "
                f"extracted-binary non-GUI `--studio-smoke` command evidence. {linux_deb_install.reason}"
                if linux_deb_install.verified
                else linux_deb_install.reason
            ),
            blocked_by=[] if linux_deb_install.verified else ["committed WSL Linux `.deb` install/extract proof bundle"],
            deferred=[
                "not Linux desktop GUI launch proof",
                "not Linux seccomp or OS-sandbox enforcement",
                "not clean Linux install proof",
                "not privileged system package install proof",
                "not signed, production, or v1.0 readiness",
            ],
        ),
        ObjectiveLane(
            id="windows_linux_domain_proof_matrix",
            evidence_class="committed" if domain_matrix.committed else "local",
            label="Windows/Linux domain proof matrix",
            status=(
                "verified"
                if domain_matrix.verified
                else "source-present" if domain_matrix.source_present else "planned"
            ),
            completion_percent=100.0 if domain_matrix.verified else 60.0 if domain_matrix.source_present else 0.0,
            evidence=(
                "`scripts/smoke_garnet_studio_domain_matrix.py --suite all` "
                "has a verified manifest-backed parse/check/run bundle for 20 current examples: "
                "10 canonical MVP domains, signed hot-reload success, expected BLAKE3 "
                "mismatch rejection, five agent toolbelt programs, and three agentic design programs. "
                f"{domain_matrix.reason}"
                if domain_matrix.verified
                else domain_matrix.reason
            ),
            blocked_by=[] if domain_matrix.verified else ["verified Windows/Linux domain matrix evidence bundle"],
            deferred=[
                "Studio screenshot evidence from Windows and WSL/Linux shells",
                "Future native backend and package-format permutations",
            ],
        ),
        ObjectiveLane(
            id="windows_wsl_ultrapunch_repro",
            evidence_class="committed",
            label="Windows/WSL ultrapunch reproduction (S110)",
            status="verified" if ultrapunch_repro else "planned",
            completion_percent=100.0 if ultrapunch_repro else 0.0,
            evidence=(
                "`scripts/smoke_garnet_ultrapunch_repro.py` records the S104 "
                "accept/reject loop as committed evidence: ACCEPT keeps the four "
                "trust artifacts and verifies the transparency-log chain; REJECT "
                "covers both diff-caps widening refusal and an over-depth enforced-kernel "
                f"trap. {ultrapunch_repro.reason}"
                if ultrapunch_repro
                else (
                    "No committed Windows + WSL S110 ultrapunch reproduction bundles "
                    "exist yet. Expected paths: `proofs/windows/ultrapunch/` and "
                    "`proofs/linux/repro/`."
                )
            ),
            blocked_by=[] if ultrapunch_repro else ["committed Windows + WSL S110 repro bundles"],
            deferred=[
                "WSL is portability-repro only, not Linux seccomp or OS-sandbox enforcement",
                "No Wasmtime fuel, production, or v1.0 claim",
                "Cross-OS consolidation waits for S107-S109/S112 evidence",
            ],
        ),
        ObjectiveLane(
            id="web_pwa",
            label="Web/PWA productization",
            status="verified",
            completion_percent=100.0,
            evidence=(
                "Local service-worker, local PWA, browser offline, live Pages, and CI "
                "web/PWA smoke gates are active."
            ),
            blocked_by=[],
            deferred=["Full browser IDE/workbench remains future product scope."],
        ),
        ObjectiveLane(
            id="editor_lsp_adoption",
            label="Editor/LSP adoption",
            status=(
                "verified"
                if _lsp_v02_present()
                else "source-present" if _lsp_source_present() else "planned"
            ),
            completion_percent=(
                100.0 if _lsp_v02_present() else 60.0 if _lsp_source_present() else 0.0
            ),
            evidence=(
                "`garnet-lsp/` and `editors/vscode/` provide the verified S16 CST-precise LSP surface. "
                "`scripts/smoke_garnet_lsp_protocol.py` proves document/workspace symbols, CST-precise rename, "
                "rules-based quick-fix actions, and semantic tokens on top of the CST over stdio with 100% validation success. "
                "Desktop evidence records Cursor/VS Code diagnostics and release-backed VSIX execution on the `v0.5.0` tag."
            )
            if _lsp_v02_present()
            else (
                "`garnet-lsp/` and `editors/vscode/` provide the S1 source "
                "surface for diagnostics, hover, and basic go-to-definition. "
                "`scripts/smoke_garnet_lsp_protocol.py` proves those paths over stdio."
            )
            if _lsp_source_present()
            else "No committed LSP or VSCode extension source is present yet.",
            blocked_by=[] if _lsp_source_present() else ["S1 LSP implementation"],
            deferred=[]
            if _lsp_v02_present()
            else [
                "manual VSCode hover/go-to-def screenshot confirmation",
                "Marketplace/OpenVSX publication",
                "safe-mode hover",
                "workspace symbols",
                "rename",
                "CST-grade incremental precision",
            ]
            if _lsp_source_present()
            else [],
        ),
        ObjectiveLane(
            id="editor_lsp_precision",
            label="Editor/LSP precision (S16)",
            status="verified" if lsp_precision_present else "source-present",
            completion_percent=100.0 if lsp_precision_present else 60.0,
            evidence=(
                "`garnet-lsp/` now consumes the canonical rowan `garnet-cst` token/span surface for "
                "rename and semantic tokens while preserving parser/check diagnostics. "
                "`scripts/smoke_garnet_lsp_precision.py` proves document symbols, workspace symbols, "
                "cross-file function rename, scoped parameter rename, three code actions, and the S16 "
                "semantic-token categories (`capability`, `attribute`, `parameter`) over stdio. "
                "`editors/vscode` exposes the three Garnet quick-fix commands and packages "
                "`garnet-0.7.0-lsp-precision.vsix`."
            )
            if lsp_precision_present
            else (
                "LSP precision source is present but the rowan-backed S16 smoke, VS Code command "
                "surface, or package evidence is incomplete."
            ),
            blocked_by=[]
            if lsp_precision_present
            else ["rowan-backed precision smoke", "VS Code command/package evidence"],
            deferred=[
                "safe-mode precision",
                "cross-package rename",
                "per-project semantic-token themes",
                "Marketplace/OpenVSX publication",
            ],
        ),
        ObjectiveLane(
            id="mobile_distribution",
            label="Mobile distribution",
            status="planned",
            completion_percent=0.0,
            evidence="No iOS, Android, Expo, TestFlight, or app-store lane is implemented yet.",
            blocked_by=["product surface decision", "mobile build/test pipeline"],
            deferred=["iOS", "Android", "Expo", "TestFlight", "Play Store"],
        ),
        ObjectiveLane(
            id="promo_video",
            evidence_class="local",
            label="Promo video",
            status=promo.status,
            completion_percent=promo.completion_percent,
            evidence=(
                "`scripts/garnet_promo_video_status.py` defines the 30-second "
                "promo contract, locks visual identity/source surfaces to repo evidence, "
                f"adds a HyperFrames-compatible composition source, and {promo_evidence_tail}"
            ),
            blocked_by=promo_blockers,
            deferred=["human/aesthetic acceptance review"],
        ),
        ObjectiveLane(
            id="llm_assist",
            label="LLM assist",
            status="active-partial",
            completion_percent=40.0,
            evidence=(
                "Garnet-aware assist contract, deterministic local context pack, "
                "planned-language assist-plan reporter, converter LLM feasibility "
                "gate, advisory-only provider option registry, Studio provider-options "
                "evidence action, provider-neutral advisory bundle, advisory review "
                "gate, and provider-neutral handoff packet are active, but no "
                "provider-backed assist lane or LLM conversion is active."
            ),
            blocked_by=["secure advisory implementation", "provider/runtime boundary", "dogfood gate"],
            deferred=contract.analysis_targets + contract.required_gates,
        ),
        ObjectiveLane(
            id="official_packages_seed",
            label="Official Layer-2 package seed (S18)",
            status=(
                "local-registry-source-ready"
                if _official_packages_seed_present()
                else "planned"
            ),
            completion_percent=85.0 if _official_packages_seed_present() else 0.0,
            evidence=(
                "`tools/garnet-lang-template/` provides the reusable Layer-2 package scaffold; "
                "`examples/garnet_lang_registry_seed/` contains local filesystem-registry "
                "v0.1.0 seeds for `http-client`, `llm`, `cli`, `test-property`, and `log`; "
                "`examples/mvp_18_all_official_packages/` plus "
                "`scripts/smoke_garnet_lang_packages_seed.py` vendors all five through the "
                "S13 registry stub and runs one primitive from each. This is local source proof, "
                "not external GitHub publication."
            )
            if _official_packages_seed_present()
            else "No committed S18 local package template, registry seed, or all-packages smoke is present yet.",
            blocked_by=[]
            if _official_packages_seed_present()
            else ["S18 local package seed"],
            deferred=[
                "`github.com/garnet-lang/` org creation or authority is a Jon/manual step",
                "Five external `github.com/garnet-lang/*` repos still need publication and CI",
                "Source-level `@stability(...)` on package functions waits on the parser annotation handoff",
                "HTTP/LLM live transport remains out of the local-registry seed proof",
            ]
            if _official_packages_seed_present()
            else [],
        ),
        ObjectiveLane(
            id="compiler_agent_llm_tier",
            label="Compiler-as-agent LLM tier (S19)",
            status=(
                "feature-gated-source-ready"
                if _compiler_agent_llm_tier_present()
                else "planned"
            ),
            completion_percent=85.0 if _compiler_agent_llm_tier_present() else 0.0,
            evidence=(
                "`garnet-suggest-llm/` provides the feature-gated S19 source tier: "
                "deterministic suggestions run first, provider-compatible Anthropic/OpenAI/Ollama "
                "clients use an explicit `LlmTransport` boundary, LLM findings are tagged "
                "`@stability(non-deterministic)`, and `.garnet-cache/llm-suggest-log.jsonl` "
                "records prompt hashes, model identity, raw response text, emitted suggestions, "
                "timestamp, and warnings. `scripts/check_determinism_no_llm.py` guards the "
                "determinism workflow from `--llm`; `benchmarks/paper_vi_exp3_compiler_as_agent/` "
                "ships the harness-only Paper VI Exp 3 scaffold with ten snapshots. This is not "
                "a shipped end-to-end CLI claim until the `garnet-cli` handoff wires "
                "`garnet check --suggest --llm`."
            )
            if _compiler_agent_llm_tier_present()
            else "No committed S19 LLM suggestion crate, determinism guard, or Exp 3 harness is present yet.",
            blocked_by=[]
            if _compiler_agent_llm_tier_present()
            else ["S19 feature-gated source tier"],
            deferred=[
                "`garnet-cli` integration for `garnet check --suggest --llm` is a read-only-crate handoff",
                "Shared `garnet-lang/llm` package trait waits on S18 after S17 merges",
                "Streaming, tool/function calling, vision, and provider-specific edge features are v0.8+",
                "Running Paper VI Exp 3 to produce h3a/h3b/h3c results is v0.7.1",
            ]
            if _compiler_agent_llm_tier_present()
            else [],
        ),
        ObjectiveLane(
            id="paper_vi_exp1_provider_gated_harness",
            label="Paper VI Exp 1 LLM pass@1 harness (S94)",
            status="provider-gated-harness" if paper_vi_exp1.ok else "planned",
            completion_percent=100.0 if paper_vi_exp1.ok else 0.0,
            evidence=(
                "`benchmarks/paper_vi_exp1_llm_pass_at_1/` now contains the "
                "S94 provider-gated harness for the registered pass@1 task "
                "shape. `scripts/garnet_paper_vi_exp1_status.py --gate` proves "
                f"{paper_vi_exp1.seed_task_count} seed tasks, provider-free "
                f"`{paper_vi_exp1.provider_free_status}` rows, and fixture-only "
                f"scoring ({paper_vi_exp1.fixture_pass_rows}/"
                f"{paper_vi_exp1.fixture_measured_rows} pass rows) without a "
                "network call. Real providers remain behind the explicit "
                "`--provider` / `--execute-provider` gate."
            ),
            blocked_by=[],
            deferred=[
                "Provider-backed pass@1 measurement requires credentials and reviewed execution",
                "Full 500-task corpus remains pending infrastructure",
                "Hidden-test scorer and statistical significance run remain future work",
                "Fine-tuned model comparison remains unclaimed",
            ],
        ),
        ObjectiveLane(
            id="paper_vi_exp3_5k_rerun_harness",
            label="Paper VI Exp 3 5K-LOC rerun harness (S95)",
            status="provider-gated-5k-harness" if paper_vi_exp3_5k.ok else "planned",
            completion_percent=100.0 if paper_vi_exp3_5k.ok else 0.0,
            evidence=(
                "`benchmarks/paper_vi_exp3_compiler_as_agent/` now contains the "
                "S95 5K-LOC rerun harness. "
                "`scripts/garnet_paper_vi_exp3_5k_status.py --gate` generates "
                f"{paper_vi_exp3_5k.snapshot_count} deterministic snapshots "
                f"(minimum {paper_vi_exp3_5k.min_snapshot_loc} LOC, total "
                f"{paper_vi_exp3_5k.total_generated_loc} LOC), writes "
                f"{paper_vi_exp3_5k.stateless_rows} stateless and "
                f"{paper_vi_exp3_5k.history_aware_rows} history-aware provider-free "
                "rows, aggregates/analyzes them, and keeps h3a at "
                f"`{paper_vi_exp3_5k.h3a_status}` with no new 5K measurement claim."
            ),
            blocked_by=[],
            deferred=[
                "Provider-backed 5K runtime rows require credentials and reviewed execution",
                "The recorded v4.0 h3a 6.5% partial stands until that rerun exists",
                "h3b/h3c revalidation at 5K scale remains future measured-study work",
                "Provider cost/budget approval remains out of scope for the committed gate",
            ],
        ),
        ObjectiveLane(
            id="linear_effect_safe_mode_seed",
            label="Linear/effect safe-mode seed (S96)",
            status="verified" if linear_effect.ok else "planned",
            completion_percent=100.0 if linear_effect.ok else 0.0,
            evidence=(
                "`garnet-check-v0.3/src/effects.rs` adds the S96 static "
                "linear/effect safe-mode seed. It reuses the CapCaps transitive "
                "surface and rejects non-entry safe helper functions that perform "
                "authority effects without any explicit ownership-qualified "
                "parameter boundary (`own`, `borrow`, `ref`, or `mut`). "
                "`scripts/garnet_linear_effect_status.py --gate` runs the focused "
                "checker proof."
            ),
            blocked_by=[],
            deferred=[
                "First static increment only: not whole-language linear typing",
                "No VM/runtime capability enforcement is claimed",
                "No OS sandbox enforcement is claimed",
                "Method-call effect resolution remains limited by the current cap graph",
            ],
        ),
        ObjectiveLane(
            id="provenance_seal_chain",
            label="Provenance seal chain (S97)",
            status="verified" if provenance_chain.ok else "planned",
            completion_percent=100.0 if provenance_chain.ok else 0.0,
            evidence=(
                "`garnet seal --provenance-chain` validates the conventional "
                "`agent`, `model`, and `prompt_sha256` attestation keys, then "
                "emits a deterministic `provenance_chain` block bound to the "
                "current seal's source and subject digests. "
                "`scripts/garnet_provenance_seal_chain_status.py --gate` runs "
                "the focused proof."
            ),
            blocked_by=[],
            deferred=[
                "Self-declared provenance only: no independent model-run proof",
                "No claim that the named agent actually produced the artifact",
                "No proof that the declared tool list is complete",
                "Supply-chain signing still depends on external cosign/Sigstore",
            ],
        ),
        ObjectiveLane(
            id="capability_manifest_standard_seed",
            label="Capability-manifest standard seed (S98)",
            status="verified" if cap_manifest_standard.ok else "planned",
            completion_percent=100.0 if cap_manifest_standard.ok else 0.0,
            evidence=(
                "`garnet caps --standard-profile` emits the S98 "
                "`capability-manifest/v1` draft/reference profile over the "
                "same declared capability surface used by S36-S38. "
                "`scripts/garnet_cap_manifest_standard_status.py --gate` "
                "checks the schema doc, RFC alignment, test vectors, and "
                "focused CLI proof."
            ),
            blocked_by=[],
            deferred=[
                "Draft/reference seed only: no OWASP/LF adoption is claimed",
                "No multi-language ecosystem or conformance suite is claimed",
                "Declared surface only: no proof of undeclared-authority absence",
                "No VM/runtime capability enforcement is claimed",
            ],
        ),
        ObjectiveLane(
            id="windows_cross_os_enforcement_phase1",
            label="Windows cross-OS enforcement proof (S106 Phase 1)",
            status="verified" if windows_cross_os.ok else "planned",
            completion_percent=100.0 if windows_cross_os.ok else 0.0,
            evidence=(
                "`scripts/garnet_windows_cross_os_enforcement_proof.py --gate` "
                "checks committed proof records for the S101 Stage V trap gate on "
                "Windows and a WSL execution/portability rerun. The Windows row "
                "records `@max_depth`, `@caps(env)`, `@caps(proc)`, `@caps(fs)`, "
                "`@caps(net)`, and the S92 program-entry `@caps(proc)` trap; the "
                "WSL row is labeled `execution/portability, not enforcement`."
            ),
            blocked_by=[] if windows_cross_os.ok else ["S106 Windows/WSL proof gate"],
            deferred=[
                "WSL is not Linux seccomp enforcement",
                "WSL is not OS-sandbox enforcement",
                "S103 ultrapunch accept/reject reproduction is Phase 2",
                "S105 domain execution is Phase 2",
                "Wasmtime fuel / @bounded runtime enforcement remains out of scope",
                "memory/time/@mailbox runtime ceilings remain out of scope",
            ],
        ),
        ObjectiveLane(
            id="broad_converter_frontends",
            label="Broad converter frontends",
            status="planned",
            completion_percent=0.0,
            evidence="Only Rust/Ruby/Python/Go deterministic frontends are active today.",
            blocked_by=["frontend implementation slices", "corpus fixtures", "lineage/sandbox/check gates"],
            deferred=[language.label for language in converter.planned_languages],
        ),
        ObjectiveLane(
            id="proof_empirics",
            label="Proof and empirical validation",
            status="active-partial",
            completion_percent=45.0 if vm_scaffold_present else 40.0,
            evidence=(
                "`scripts/garnet_proof_benchmark_status.py` inventories the current "
                f"{len(proof.benchmarks)} Criterion benchmark harnesses, "
                f"{len(proof.fuzz_harnesses)} fuzz harness"
                f"{'' if len(proof.fuzz_harnesses) == 1 else 'es'}, plus proof/study "
                "protocols"
                + (
                    ", including the S2 VM parse/compile/execute harness,"
                    if vm_scaffold_present
                    else ""
                )
                + " while reporting measurements, mechanized proof, and empirical "
                "study execution as unclaimed and long-running fuzz hours as pending."
            ),
            blocked_by=proof.blocked_by,
            deferred=proof.deferred,
        ),
        ObjectiveLane(
            id="determinism_ci_cross_machine",
            label="Determinism CI cross-machine",
            status="verified"
            if (ROOT / ".github/workflows/determinism.yml").exists()
            and (ROOT / "examples/det_fixture_01.garnet").exists()
            else "planned",
            completion_percent=100.0
            if (ROOT / ".github/workflows/determinism.yml").exists()
            and (ROOT / "examples/det_fixture_01.garnet").exists()
            else 0.0,
            evidence=(
                "`.github/workflows/determinism.yml` builds "
                "`examples/det_fixture_01.garnet` with `garnet build --deterministic "
                "--sign <key>` on a matrix of ubuntu-latest and macos-latest runners; "
                "a single shared signing key is generated once and downloaded by each "
                "OS, so signatures are byte-identical when the source is. The "
                "compare job fails with `::error::` if the per-OS manifest SHA-256 "
                "diverges. Closes Paper VI Contribution 6 verification gap."
            ),
            blocked_by=[],
            deferred=[
                "Windows runner in the cross-OS matrix",
                "Linux aarch64 in the cross-OS matrix",
                "Multi-key rotation testing",
                "Byte-for-byte binary determinism once a native backend exists",
            ],
        ),
        ObjectiveLane(
            id="parser_fuzz_harness",
            label="Parser fuzz harness (nightly)",
            status="verified"
            if (ROOT / ".github/workflows/fuzz-nightly.yml").exists()
            and (ROOT / "garnet-parser-v0.3/fuzz/Cargo.toml").exists()
            and (ROOT / "garnet-parser-v0.3/fuzz/fuzz_targets/parse_input.rs").exists()
            else "planned",
            completion_percent=100.0
            if (ROOT / ".github/workflows/fuzz-nightly.yml").exists()
            and (ROOT / "garnet-parser-v0.3/fuzz/Cargo.toml").exists()
            and (ROOT / "garnet-parser-v0.3/fuzz/fuzz_targets/parse_input.rs").exists()
            else 0.0,
            evidence=(
                "`.github/workflows/fuzz-nightly.yml` runs `cargo +nightly fuzz run parse_input` "
                "for a 60-second PR smoke and ≥ 1 hour on a nightly schedule against the `garnet-parser-v0.3/fuzz/` "
                "cargo-fuzz sub-workspace. The `parse_input` target wraps every call in a "
                "strict `ParseBudget` (4096-byte source cap, 1024-token cap, 32-depth cap, "
                "512-byte literal cap), so neither CPU nor memory can be unbounded. Seed "
                "corpus is populated from canonical `examples/*.garnet` files; crashes "
                "upload as artifacts for triage. Closes the S5 surface gap."
            ),
            blocked_by=[],
            deferred=[
                "Interpreter and checker fuzz targets",
                "Differential fuzzing against the archived v0.2 parser",
                "OSS-Fuzz upstream integration",
                "Coverage-guided corpus minimization (currently raw seed only)",
            ],
        ),
        ObjectiveLane(
            id="compiler_advisory_rules_based",
            label="Compiler advisory mode (rules-based)",
            status="verified"
            if (ROOT / "garnet-check-v0.3/src/suggest.rs").exists()
            and (ROOT / "garnet-check-v0.3/tests/suggest_corpus.rs").exists()
            else "planned",
            completion_percent=100.0
            if (ROOT / "garnet-check-v0.3/src/suggest.rs").exists()
            and (ROOT / "garnet-check-v0.3/tests/suggest_corpus.rs").exists()
            else 0.0,
            evidence=(
                "`garnet-check-v0.3/src/suggest.rs` ships a deterministic, "
                "rules-based suggestion engine with three patterns today "
                "(managed-fn-missing-caps, long-parameter-list, "
                "empty-function-body). `garnet-cli/src/cmd/check.rs` exposes "
                "`garnet check --suggest <file.garnet>`; output is prefixed "
                "with the literal `compiler suggested:` so downstream tooling "
                "can grep for advisories. "
                "`garnet-check-v0.3/tests/suggest_corpus.rs` proves at least "
                "three distinct rules fire across the committed corpus. "
                "Closes Paper VI Contribution 7 surface for the rules-based "
                "tier; the LLM tier remains pending-infra (separate provider "
                "boundary and budget)."
            ),
            blocked_by=[],
            deferred=[
                "LLM-derived suggestions (pending Paper VI Exp 1 infrastructure)",
                "Auto-apply / quick-fix wiring into LSP code-actions (meshes with S1)",
                "Cross-module suggestions (current rules are intra-module only)",
                "Configurable rule severity per project",
            ],
        ),
        ObjectiveLane(
            id="signed_hot_reload_demo",
            label="Signed hot-reload BLAKE3 demo",
            status="verified"
            if (ROOT / "examples/mvp_11_signed_hotreload.garnet").exists()
            and (ROOT / "examples/mvp_11_signed_hotreload_mismatch.garnet").exists()
            else "planned",
            completion_percent=100.0
            if (ROOT / "examples/mvp_11_signed_hotreload.garnet").exists()
            and (ROOT / "examples/mvp_11_signed_hotreload_mismatch.garnet").exists()
            else 0.0,
            evidence=(
                "`examples/mvp_11_signed_hotreload.garnet` and "
                "`examples/mvp_11_signed_hotreload_mismatch.garnet` are runnable "
                "managed-mode demonstrations of the BLAKE3 fingerprint check that "
                "drives the Rust-runtime `actor.reload_signed` path. Both call "
                "`crypto::blake3` on a payload and compare against an embedded "
                "expected hash; the success example prints `reloaded successfully` "
                "and exits 0, the mismatch example raises with the literal text "
                "`BLAKE3 fingerprint mismatch` and exits 1. Closes Paper VI "
                "Contribution 5 surface gap."
            ),
            blocked_by=[],
            deferred=[
                "Managed-mode `actor.reload_signed` syntax (the Rust runtime "
                "API is tested separately in `garnet-actor-runtime/tests/reload.rs`; "
                "exposing it to managed mode is a separate slice)",
                "Real signed-bytes payload (the demo uses a text-shaped marker "
                "for readability; production payloads are arbitrary bytes)",
                "Ed25519 signature verification of the payload (BLAKE3 fingerprint "
                "check is one of two layers; the cryptographic signature layer is "
                "validated only in the Rust runtime today)",
            ],
        ),
        ObjectiveLane(
            id="actor_trust_report_bridge",
            label="Actor OS-thread bridge (`trust-report`)",
            status="verified"
            if (ROOT / "garnet-cli/src/cmd/trust_report.rs").exists()
            and (ROOT / "examples/agent_orchestrator_3thread.garnet").exists()
            and (ROOT / "garnet-cli/tests/trust_report.rs").exists()
            else "planned",
            completion_percent=100.0
            if (ROOT / "garnet-cli/src/cmd/trust_report.rs").exists()
            and (ROOT / "examples/agent_orchestrator_3thread.garnet").exists()
            and (ROOT / "garnet-cli/tests/trust_report.rs").exists()
            else 0.0,
            evidence=(
                "`garnet-cli/src/cmd/trust_report.rs` implements "
                "`garnet trust-report <file.garnet>` — a structural report that "
                "counts actor declarations and surfaces the per-function caps "
                "set, printing the literal line `actors: N / threads: N` per "
                "the S7 contract dogfood. `examples/agent_orchestrator_3thread.garnet` "
                "is the three-actor fixture; `garnet-cli/tests/trust_report.rs` "
                "asserts the dogfood block on every `cargo test --workspace`. "
                "The actor runtime in `garnet-actor-runtime/src/runtime.rs` "
                "already spawns one OS thread per actor (see runtime.rs header); "
                "S7 lands the CLI bridge that surfaces what the runtime does."
            ),
            blocked_by=[],
            deferred=[
                "Live-runtime instrumentation (current report is STRUCTURAL, "
                "counting actor declarations; it does not spawn the runtime "
                "or measure live thread counts)",
                "Mailbox-size + Sendable-boundary audit beyond what "
                "`garnet check` already enforces",
                "Transitive caps aggregation from `use` imports (resolves to "
                "stdlib today, not to vendored deps from S3)",
                "Cross-actor message-graph visualization",
            ],
        ),
        ObjectiveLane(
            id="garnet_add_manifest",
            label="Garnet manifest + vendored deps (`garnet add`)",
            status="verified"
            if (ROOT / "garnet-cli/src/cmd/add.rs").exists()
            and (ROOT / "C_Language_Specification/GARNET_MANIFEST_v0_1.md").exists()
            else "planned",
            completion_percent=100.0
            if (ROOT / "garnet-cli/src/cmd/add.rs").exists()
            and (ROOT / "C_Language_Specification/GARNET_MANIFEST_v0_1.md").exists()
            else 0.0,
            evidence=(
                "`garnet-cli/src/cmd/add.rs` implements `garnet add [--name <id>] "
                "<path>` to vendor a local Garnet directory into "
                "`.garnet/vendor/<name>/`, update `Garnet.toml`'s `[dependencies]` "
                "table, and write `Garnet.lock` with BLAKE3-per-file hashes. "
                "Lockfile output is deterministic (alpha-sorted deps, lex-sorted "
                "files, lowercase hex). Six inline unit tests cover the round-trip. "
                "Format documented in "
                "`C_Language_Specification/GARNET_MANIFEST_v0_1.md`."
            ),
            blocked_by=[],
            deferred=[
                "Remote sources (https://, git+ssh://, @scope/name registries)",
                "Transitive dependency vendoring",
                "SemVer matching (caret/tilde/equality beyond string compare)",
                "Workspace mode (multi-crate projects)",
                "`garnet verify-deps` lockfile-drift detector",
            ],
        ),
        ObjectiveLane(
            id="pkg_resolver_v0_2",
            label="Package-manager resolver (S12)",
            status="verified"
            if (ROOT / "garnet-cli/tests/run_resolver.rs").exists()
            and (ROOT / "garnet-cli/src/cmd/run.rs").exists()
            else "planned",
            completion_percent=100.0
            if (ROOT / "garnet-cli/tests/run_resolver.rs").exists()
            and (ROOT / "garnet-cli/src/cmd/run.rs").exists()
            else 0.0,
            evidence=(
                "`garnet-cli/src/cmd/run.rs::preload_dependencies` reads "
                "`Garnet.toml`'s `[dependencies]` table via "
                "`garnet-cli/src/cmd/add.rs::read_dependency_table`, walks each "
                "declared vendor directory, and pre-loads every `.garnet` source "
                "into the interpreter's global environment before the user "
                "source is loaded. `Item::Use(_)` in the interpreter stays a "
                "no-op; the vendored symbols are already in scope by the time "
                "`use <dep>::*` is reached. `garnet-cli/tests/run_resolver.rs` "
                "covers the round trip end-to-end against a temp project with "
                "the same on-disk layout `garnet add` produces. Four inline "
                "unit tests in `cmd::run::tests` cover the `strip_top_level_main` "
                "guard that prevents a vendored dep's own `main` from shadowing "
                "the user's entry point. Closes the S3 deferred line on resolver."
            ),
            blocked_by=[],
            deferred=[
                "Qualified-path resolution (`local_lib::hello()` with the "
                "prefix in the call site)",
                "Remote sources (https://, git+ssh://, @scope/name registries)",
                "Transitive dependency vendoring",
                "SemVer matching (caret/tilde/equality beyond string compare)",
                "Workspace mode (multi-crate projects)",
                "VM path pre-load (S14 will harmonize the `--vm` resolver)",
                "Lockfile BLAKE3 verification at run time "
                "(`garnet verify-deps` slice)",
                "Name-collision handling between deps (last-loaded wins today)",
                "Module-scoped `use local_lib::Foo::bar` paths "
                "(only top-level items are pre-loaded today)",
            ],
        ),
        ObjectiveLane(
            id="vm_function_call_lowering",
            label="Bytecode VM v0.2 function-call lowering (S14)",
            status="verified"
            if (ROOT / "garnet-vm/tests/function_call.rs").exists()
            and (ROOT / "C_Language_Specification/GARNET_BYTECODE_v0_2.md").exists()
            else "planned",
            completion_percent=100.0
            if (ROOT / "garnet-vm/tests/function_call.rs").exists()
            and (ROOT / "C_Language_Specification/GARNET_BYTECODE_v0_2.md").exists()
            else 0.0,
            evidence=(
                "`garnet-vm/src/vm.rs` executes native function calls on an "
                "explicit, heap-allocated call-frame stack (`Frame` + "
                "`run_frames`) instead of recursing in the host (Rust) "
                "language, so deep Garnet recursion no longer overflows the "
                "Rust stack. `garnet-vm/tests/function_call.rs` proves "
                "`countdown(200000)` and mutual recursion to depth 500 run to "
                "completion on the VM, plus VM/interpreter parity for "
                "mixed-arity and nested calls. The codec is version-bumped to "
                "`GARNVM02` with an explicit per-function arity field that the "
                "deserializer cross-checks. `garnet run --vm --dump-lowering` "
                "reports the native/fallback ratio (`lowered: N%`); "
                "`examples/mvp_function_call_demo.garnet` reports `lowered: "
                "100%`. Documented in "
                "`C_Language_Specification/GARNET_BYTECODE_v0_2.md`."
            ),
            blocked_by=[],
            deferred=[
                "Tail-call optimization (each call still costs one heap frame)",
                "Closures, captured environments, dynamic-receiver method "
                "dispatch (still fall back)",
                "Pattern matching, try/rescue/ensure, struct/enum constructors "
                "(still fall back)",
                "`and` / `or` short-circuit native lowering (Ruby-style "
                "operand-returning semantics need value-preserving "
                "conditional-jump + Dup opcodes)",
                "VM-path vendored-dependency pre-load (the S12 resolver is "
                "`--interp` only)",
                "Stable cross-version bytecode ABI (GARNVM02 is tightened, "
                "not frozen)",
                "Production native-compiler proof",
            ],
        ),
        ObjectiveLane(
            id="registry_stub_v0_1",
            label="Registry stub v0.1 (S13)",
            status="verified"
            if (ROOT / "garnet-registry-stub/src/lib.rs").exists()
            and (ROOT / "C_Language_Specification/GARNET_REGISTRY_v0_1.md").exists()
            else "planned",
            completion_percent=100.0
            if (ROOT / "garnet-registry-stub/src/lib.rs").exists()
            and (ROOT / "C_Language_Specification/GARNET_REGISTRY_v0_1.md").exists()
            else 0.0,
            evidence=(
                "`garnet-registry-stub/` is a filesystem-backed registry: an "
                "`index.json` (serde) maps `name -> version -> { path, "
                "BLAKE3-per-file }` over `<name>/<version>/` package "
                "directories. `garnet-registry-stub build|verify` generates and "
                "checks the index. `garnet add --registry <location> "
                "<name>@<version>` (in `garnet-cli/src/cmd/add.rs`) loads the "
                "index, resolves the version, verifies every file's BLAKE3 "
                "(refusing path-traversal outside the registry root), and "
                "vendors the package into `.garnet/vendor/<name>/`. Because the "
                "S12 resolver loads vendored deps at `garnet run` time, a "
                "registry-resolved `use <name>::*` resolves end-to-end "
                "(`examples/registry_stub_fixture/` + "
                "`garnet-cli/tests/registry_add.rs`, 3 integration tests + 6 "
                "stub-crate unit tests). Documented in "
                "`C_Language_Specification/GARNET_REGISTRY_v0_1.md`."
            ),
            blocked_by=[],
            deferred=[
                "HTTP(S) transport (filesystem / file:// only today)",
                "Tarball packaging (packages are directories)",
                "Authentication, accounts, publish/upload flow",
                "Signature verification (the index `signature` field is "
                "reserved but unread; meshes with the notarization slice)",
                "SemVer ranges (exact <name>@<version> only)",
                "Multi-registry resolution (one registry per invocation)",
                "Transitive dependency resolution from the registry",
            ],
        ),
        ObjectiveLane(
            id="memory_eviction_benchmarks",
            label="Memory eviction policy benchmarks",
            status="verified"
            if (ROOT / "garnet-memory-v0.3/benches/eviction.rs").exists()
            and (ROOT / "scripts/garnet_memory_eviction_status.py").exists()
            else "planned",
            completion_percent=100.0
            if (ROOT / "garnet-memory-v0.3/benches/eviction.rs").exists()
            and (ROOT / "scripts/garnet_memory_eviction_status.py").exists()
            else 0.0,
            evidence=(
                "`garnet-memory-v0.3/benches/eviction.rs` is a Criterion bench "
                "that exercises `MemoryPolicy::score` + `should_retain` per "
                "Mnemos kind (working/episodic/semantic/procedural) against a "
                "naive FIFO baseline; `scripts/garnet_memory_eviction_status.py` "
                "inventories per-kind coverage. Closes the S6 contract surface "
                "and half of Paper VI Contribution 3's production-allocator gap "
                "(this is policy-cost measurement, not production allocator)."
            ),
            blocked_by=[],
            deferred=[
                "Fresh Criterion measurement run (`cargo bench -p garnet-memory "
                "--bench eviction`) — captured as Desktop evidence on a maintainer "
                "machine, not embedded in this lane",
                "End-to-end store throughput benches under eviction (EpisodeStore, "
                "VectorIndex, etc.) — separate slice",
                "Production allocator path — remains Tier 1 work in MEMORY_CORE_ROADMAP.md",
            ],
        ),
        ObjectiveLane(
            id="formatter_idempotent_baseline",
            label="Formatter idempotent baseline",
            status="verified"
            if (ROOT / "garnet-cli/src/cmd/fmt.rs").exists()
            and (ROOT / "garnet-cli/tests/fmt_idempotency.rs").exists()
            else "planned",
            completion_percent=100.0
            if (ROOT / "garnet-cli/src/cmd/fmt.rs").exists()
            and (ROOT / "garnet-cli/tests/fmt_idempotency.rs").exists()
            else 0.0,
            evidence=(
                "`garnet-cli/src/cmd/fmt.rs` normalizes trailing whitespace, "
                "CRLF/CR line endings, and final newline to a single LF, then "
                "re-parses to refuse any change that would break the source. "
                "`garnet-cli/tests/fmt_idempotency.rs` proves the canonical "
                "`examples/{mvp_,det_}*.garnet` corpus is byte-identical after "
                "two passes of `garnet fmt --stdout` and that three runs on the "
                "same input produce identical bytes. Satisfies the S4 contract "
                "goal (deterministic, idempotent source formatter)."
            ),
            blocked_by=[],
            deferred=[
                "AST-driven semantic formatting (alignment, spacing rules, "
                "import sorting) — gates on a trivia-preserving CST in the parser",
                "Comment-preserving round-trip (today the parser drops trivia)",
                "Pretty-printer for malformed input recovery",
                "Workspace-level `garnet fmt --workspace`",
            ],
        ),
        ObjectiveLane(
            id="parser_cst_layer",
            label="Trivia-preserving CST (S15)",
            status="verified" if _cst_layer_present() else "planned",
            completion_percent=100.0 if _cst_layer_present() else 0.0,
            evidence=(
                "Trivia-preserving Concrete Syntax Tree (CST) helper in `cst.rs` is fully integrated with the parser. "
                "CST round-trip integration tests in `tests/cst_round_trip.rs` assert 100% byte-identical "
                "reconstruction of all parser-crate and workspace examples under the examples directories."
            ) if _cst_layer_present() else "No committed CST or cst_round_trip test source is present yet.",
            blocked_by=[] if _cst_layer_present() else ["S15 CST implementation"],
            deferred=[
                "incremental syntax parsing",
                "error-recovery parsing",
            ] if _cst_layer_present() else [],
        ),
        ObjectiveLane(
            id="parser_cst_migration",
            label="Rowan CST migration (S15, build-both-then-compare)",
            status="verified" if _rowan_cst_present() else "planned",
            completion_percent=100.0 if _rowan_cst_present() else 0.0,
            evidence=(
                "`garnet-cst/` is a rowan-backed, trivia-preserving CST built cold "
                "(direct recursive-descent over the token stream) for the v0.7 "
                "build-both-then-compare A/B. S15-Compare chose rowan as the "
                "canonical CST; #221's parser CST is retained temporarily as a "
                "legacy migration oracle. `parse_cst` round-trips byte-identically "
                "across the canonical examples corpus and a `proptest` over arbitrary "
                "UTF-8 (`tests/examples_roundtrip.rs`, `tests/roundtrip.rs`). `cst_to_ast` "
                "projects onto `garnet_parser::ast::Module` with span-normalized "
                "structural parity vs `parse_source` across the corpus "
                "(`tests/cst_to_ast_parity.rs`). The `parse_cst_vs_ast` Criterion bench "
                "measures the CST path at ~0.99x the AST path (well under the 1.5x gate). "
                "`garnet parse --mode cst` routes to this rowan parser, and `tokens.rs` "
                "preserves the #221 token/span ergonomics for LSP migration. "
                "Reproduce via the S15 dogfood block in `GARNET_v0_7_SLICE_DOGFOOD.md`."
            )
            if _rowan_cst_present()
            else "No rowan `garnet-cst` builder/converter/bench source present yet.",
            blocked_by=[] if _rowan_cst_present() else ["S15 PR-2 rowan CST"],
            deferred=[
                "Error-recovery parsing is best-effort (round-trip always holds; structure "
                "may flatten on malformed input)",
                "Incremental re-parsing",
                "Rowan-backed LSP migration; #221's parser CST remains as a temporary "
                "legacy oracle until that migration is green",
                "CST-first migration of interp/check/vm (v0.8; they stay on the AST path "
                "via `parse_source`, untouched by S15)",
            ]
            if _rowan_cst_present()
            else [],
        ),
        ObjectiveLane(
            id="stdlib_layer_policy",
            label="Stdlib layer policy + `@stability` (S17)",
            status="verified" if stdlib.ok else "active-partial",
            completion_percent=100.0
            if stdlib.ok
            else round(stdlib.explicit_stability_percent, 1),
            evidence=(
                "`C_Language_Specification/GARNET_STDLIB_LAYER_POLICY.md` codifies the "
                "five-layer model + `@stability` semantics. `garnet-stdlib/src/registry.rs` "
                f"tags every primitive with a Layer + Stability tier: {stdlib.total} "
                f"primitives ({stdlib.by_layer.get('Core', 0)} Layer-0 `core`, "
                f"{stdlib.by_layer.get('Std', 0)} Layer-1 `std`), "
                f"{stdlib.explicit_stability_percent:.1f}% with an explicit tier. "
                "`garnet-check-v0.3/src/stability.rs` warns non-fatally at call sites into "
                "experimental/deprecated primitives (info for frozen); `@caps(env)` is a new "
                "known capability. `scripts/garnet_stdlib_layer_gate.py` enforces >= 50 "
                f"primitives and >= 95% explicit `@stability` (gate: "
                f"{'PASS' if stdlib.ok else 'FAIL'}). Reproduce via the S17 dogfood block in "
                "`F_Project_Management/GARNET_v0_7_SLICE_DOGFOOD.md`."
            ),
            blocked_by=[],
            deferred=[
                "Source-level `@stability(...)` on user-defined functions + "
                "`@uses(experimental)` opt-in + `@migration(...)` hints — pending the "
                "win-opus → mac-opus parser annotation handoff (primitive-stability "
                "enforcement ships now)",
                "Error-level `@stability` enforcement (v0.7 is warning-level for "
                "backwards compat; error-level is v0.8)",
                "Garnet-source execution of the new Layer-0/1 primitives via the "
                "interpreter (registry surface + Rust host impls + unit tests ship now; "
                "interpreter dispatch is v0.8, as garnet-interp is outside S17's ownership)",
                "Layer-2 `@garnet-lang/*` packages (S18) and the `@caps(fs)` file-sink "
                "path of `std::log`",
            ],
        ),
        ObjectiveLane(
            id="novel_composition_dogfood",
            label="Novel-composition dogfood (S20)",
            status="verified" if novel_compositions_present else "planned",
            completion_percent=100.0 if novel_compositions_present else 0.0,
            evidence=(
                "S20 fuses Paper-VI contributions into runnable novel programs: "
                "`novel_01` (capability-budget + memory-recall + agent pipeline → "
                "governance 16), `novel_02` (BLAKE3 signed provenance + pipeline + "
                "determinism → verified content-addressed lineage), `novel_03` "
                "(release-gate + capability-budget + provenance + memory quorum → "
                "APPROVED quorum 4). The same harness now also carries `novel_04` "
                "(S21 dispatched stdlib) and `novel_05` (S22 stdlib + Mnemos handles). "
                "`scripts/smoke_garnet_novel_compositions.py` (+ "
                "`test_garnet_novel_compositions.py`) proves all five `garnet "
                "check` / `garnet run` cases with deterministic asserted output; "
                "the composition story is in "
                "`C_Language_Specification/GARNET_NOVEL_COMPOSITIONS.md`. Complements "
                "the single-concern domain proof matrix (#232) without duplicating it."
            )
            if novel_compositions_present
            else "No novel-composition programs/harness/story present yet.",
            blocked_by=[],
            deferred=[
                "Compositions are modeled deterministically in managed mode; live "
                "runtime integration (actor mailboxes, Mnemos stores, Ed25519 signing) "
                "is tracked separately",
                "These programs use the proven runnable subset + `crypto::blake3`; "
                "S21 (`interp_stdlib_dispatch`) makes the new Layer-0/1 prims runnable",
            ]
            if novel_compositions_present
            else [],
        ),
        ObjectiveLane(
            id="interp_stdlib_dispatch",
            label="Interpreter dispatch for S17 stdlib (S21)",
            status="verified" if interp_dispatch_present else "planned",
            completion_percent=100.0 if interp_dispatch_present else 0.0,
            evidence=(
                "S21 closes the S17 deferred line — the Layer-0/1 stdlib primitives now "
                "EXECUTE from Garnet source. `garnet-interp-v0.3/src/eval.rs` resolves "
                "fully-qualified names first (backward-compatible), so prims bind under "
                "`core::math::sqrt` etc. without colliding with bare prelude builtins "
                "(`map`/`ok`). `stdlib_bridge.rs` dispatches core::math (6), core::cmp (4), "
                "core::iter map/filter/fold/take/drop/enumerate (6 — higher-order via "
                "call_value, first-class-function combinators from managed Garnet), and "
                "std::base64 (2). Verified live and via "
                "`examples/novel_04_dispatched_stdlib_pipeline.garnet` (deterministic run). "
                "`garnet-interp-v0.3/tests/mnemos_stdlib_combination.rs` composes the four "
                "Mnemos memory kinds with the stdlib (blake3 provenance + base64)."
            )
            if interp_dispatch_present
            else "No qualified stdlib dispatch present yet.",
            blocked_by=[],
            deferred=[
                "S22 closes the remaining `std::json` / regex / uuid / env / process / log "
                "and `memory::` dispatch line in the separate `stdlib_memory_runtime_dispatch` lane",
                "core::cmp / core::iter are Value-level bridges; the `garnet_stdlib` "
                "generics remain the tested Rust reference (dynamic Values can't be the "
                "stdlib's monomorphic `T`)",
            ]
            if interp_dispatch_present
            else [],
        ),
        ObjectiveLane(
            id="stdlib_memory_runtime_dispatch",
            label="Stdlib + memory runtime dispatch completion (S22)",
            status="verified" if stdlib_memory_dispatch_present else "planned",
            completion_percent=100.0 if stdlib_memory_dispatch_present else 0.0,
            evidence=(
                "S22 closes the S21-deferred runtime surface. `garnet-parser-v0.3` now accepts "
                "selected keywords as qualified path segments only, so official APIs such as "
                "`std::regex::match`, `std::process::spawn`, and `memory::working` are callable "
                "without making those words legal bare identifiers. `stdlib_bridge.rs` dispatches "
                "`std::json` (parse/get/set/stringify), `std::regex` (compile/match/find_all/replace), "
                "`std::uuid` (v4/v5/v7), `std::env` (get/set/vars), `std::process` "
                "(spawn/wait/exit_code with a managed Process handle), `std::log` "
                "(formatting), and `memory::working|episodic|semantic|procedural` constructors "
                "that return live Mnemos `MemoryStore` handles. `garnet-interp-v0.3/tests/"
                "stdlib_s22_dispatch.rs` proves JSON/regex/uuid/log/env/process/memory from "
                "Garnet source; `examples/novel_05_s22_stdlib_memory_pipeline.garnet` plus "
                "`scripts/smoke_garnet_novel_compositions.py` prove deterministic stdlib + "
                "Mnemos composition through the CLI."
            )
            if stdlib_memory_dispatch_present
            else "No S22 stdlib/memory runtime dispatch proof present yet.",
            blocked_by=[],
            deferred=[
                "`std::env` and `std::process` are proven in Rust integration tests, not the "
                "deterministic novel example, because they mutate or launch host state",
                "`std::process::spawn` still uses the v0.7 whitespace-delimited command-line "
                "contract from `garnet_stdlib`; richer argv handling is v0.8+",
                "`std::log` remains formatting-only; file sinks that require `@caps(fs)` are v0.8+",
            ]
            if stdlib_memory_dispatch_present
            else [],
        ),
        ObjectiveLane(
            id="process_runtime_completion",
            label="std::process structured argv + output capture (S23)",
            status="verified" if process_runtime_present else "planned",
            completion_percent=100.0 if process_runtime_present else 0.0,
            evidence=(
                "S23 closes the S22-deferred `std::process` argv line. "
                "`garnet-stdlib/src/process.rs` adds `spawn_args(program, [args])` (explicit "
                "argv — the program and each argument are passed to the OS literally, so an "
                "argument containing spaces is not re-split) and `output(program, [args])` "
                "(run-to-completion capturing stdout/stderr/exit-code as an `Output`), keeping "
                "`spawn`/`wait`/`exit_code` backward-compatible. `stdlib_bridge.rs` dispatches "
                "both under their qualified names; `output` returns a `{code, stdout, stderr}` "
                "map. `garnet-interp-v0.3/tests/stdlib_s23_dispatch.rs` proves stdout capture, "
                "exit-code reporting, and explicit-argv spawn from Garnet source; the stdlib "
                "unit tests prove (on POSIX) that a spaced argument survives as one argv element "
                "via the `printf \"%s\"` discriminator."
            )
            if process_runtime_present
            else "No S23 std::process argv/output proof present yet.",
            blocked_by=[],
            deferred=[
                "Process stdout/stderr are host-dependent (line endings, locale); the "
                "deterministic proof asserts substring + exit-code, not byte-exact full output",
                "Still synchronous managed-mode execution; no async/OS-thread or "
                "streaming-stdout claim",
                "`std::log` file sinks requiring `@caps(fs)` remain the next deferred line (S24)",
            ]
            if process_runtime_present
            else [],
        ),
        ObjectiveLane(
            id="log_file_sink_runtime",
            label="std::log file sink with @caps(fs) (S24)",
            status="verified" if log_file_sink_present else "planned",
            completion_percent=100.0 if log_file_sink_present else 0.0,
            evidence=(
                "S24 closes the S22/S23-deferred `std::log` file-sink line. "
                "`garnet-stdlib/src/log.rs` adds `to_file(path, level, message)`, which formats "
                "the same `[LEVEL] message` line and appends it (create-if-missing) to a file, so "
                "it requires `@caps(fs)`; the formatting helpers are unchanged. `registry.rs` tags "
                "`std::log::to_file` Layer 1 / cap `fs` / experimental, and `stdlib_bridge.rs` "
                "dispatches it. `garnet-interp-v0.3/tests/stdlib_s24_dispatch.rs` writes two lines "
                "from a `@caps(fs)` Garnet `main` and reads them back via `read_file`, asserting "
                "ordered contents; the stdlib unit tests prove append-not-truncate and the IO "
                "error path."
            )
            if log_file_sink_present
            else "No S24 std::log file-sink proof present yet.",
            blocked_by=[],
            deferred=[
                "Line-append text sink only (create-if-missing); no log rotation, "
                "structured/JSON sinks, or async writers",
                "Capability enforcement remains the checker's job; the registry tags "
                "`std::log::to_file` `@caps(fs)`",
            ]
            if log_file_sink_present
            else [],
        ),
        ObjectiveLane(
            id="host_effect_composition",
            label="Host-effect composition capstone (S25)",
            status="verified" if host_effect_composition_present else "planned",
            completion_percent=100.0 if host_effect_composition_present else 0.0,
            evidence=(
                "S25 threads the S22-S24 runtime surfaces into one capability-checked "
                "pipeline. `garnet-interp-v0.3/tests/host_effect_composition.rs` runs a "
                "`@caps(proc, fs)` Garnet program that captures a host command's stdout "
                "(`std::process::output`, S23), appends a leveled line to a file "
                "(`std::log::to_file`, S24), keeps an episodic Mnemos trace of it "
                "(`memory::episodic`, S22), reads the sink back (`read_file`), and binds "
                "`crypto::blake3` provenance — asserting the composed token, recall count, "
                "file contents, exit code, and fingerprint. The deterministic, cross-platform "
                "`examples/novel_06_observability_provenance_pipeline.garnet` composes the "
                "side-effect-free subset (json + file-sink + episodic memory + blake3) and is "
                "in the novel-composition harness (now 6/6). Story: "
                "`C_Language_Specification/GARNET_NOVEL_COMPOSITIONS.md`."
            )
            if host_effect_composition_present
            else "No S25 host-effect composition proof present yet.",
            blocked_by=[],
            deferred=[
                "Process step in the deterministic novel example is omitted (host command + "
                "output are platform-variable); it is proven instead in the cfg-guarded "
                "integration test",
                "Still synchronous managed-mode; no async actor/OS-thread runtime claim",
            ]
            if host_effect_composition_present
            else [],
        ),
        ObjectiveLane(
            id="core_result_dispatch",
            label="core::result combinator dispatch (S26)",
            status="verified" if result_dispatch_present else "planned",
            completion_percent=100.0 if result_dispatch_present else 0.0,
            evidence=(
                "S26 makes the registered `core::result` combinators runnable from Garnet "
                "source, continuing the S21/S22 registry-to-runtime arc. `stdlib_bridge.rs` "
                "dispatches `core::result::{ok,err,map,and_then,or_else,unwrap_or}` at the "
                "Value layer over `Result` Variants (Ok/Err) identical to the prelude builders; "
                "`map`/`and_then`/`or_else` are higher-order via `call_value`. Bound under their "
                "qualified names so `core::result::map` does not collide with the bare `map` "
                "(Map constructor) on the last-segment fallback. "
                "`garnet-interp-v0.3/tests/core_result_dispatch.rs` proves a railway-oriented "
                "pipeline from source (map over Ok, Err pass-through, and_then chain + "
                "short-circuit, or_else recovery, unwrap_or default → [10,7,6,8,0,5,99]); bridge "
                "unit tests cover each combinator plus the non-Result type error."
            )
            if result_dispatch_present
            else "No S26 core::result dispatch proof present yet.",
            blocked_by=[],
            deferred=[
                "`and_then`/`or_else` trust the callee to return a Result (dynamic typing); "
                "no static Result-shape check",
                "Ergonomic method syntax (`result.map(..)`) is a later follow-on; S26 ships the "
                "qualified-function form",
            ]
            if result_dispatch_present
            else [],
        ),
        ObjectiveLane(
            id="core_option_dispatch",
            label="core::option combinator dispatch (S27)",
            status="verified" if option_dispatch_present else "planned",
            completion_percent=100.0 if option_dispatch_present else 0.0,
            evidence=(
                "S27 makes the registered `core::option` combinators runnable from Garnet "
                "source (sibling of the S26 `core::result` work). `stdlib_bridge.rs` dispatches "
                "`core::option::{some,none,map,and_then,unwrap_or}` at the Value layer over "
                "`Option` Variants (Some/None) identical to the prelude builders; `map`/`and_then` "
                "are higher-order via `call_value`, bound qualified to avoid the bare-`map` "
                "collision. `garnet-interp-v0.3/tests/core_option_dispatch.rs` proves it from "
                "source (map over Some, None pass-through, and_then chain + short-circuit, "
                "unwrap_or default → [10,7,6,8,5,99]); bridge unit tests cover each combinator, "
                "the None constructor, and the non-Option type error."
            )
            if option_dispatch_present
            else "No S27 core::option dispatch proof present yet.",
            blocked_by=[],
            deferred=[
                "`and_then` trusts the callee to return an Option (dynamic typing); no static "
                "Option-shape check",
                "Ergonomic method syntax (`option.map(..)`) is a later follow-on",
            ]
            if option_dispatch_present
            else [],
        ),
        ObjectiveLane(
            id="core_iter_completion",
            label="core::iter completion: zip/collect/chain (S28)",
            status="verified" if iter_completion_present else "planned",
            completion_percent=100.0 if iter_completion_present else 0.0,
            evidence=(
                "S28 dispatches the last three registered `core::iter` combinators, so all 9 are "
                "now runnable. `stdlib_bridge.rs` adds `core::iter::zip` (pairs two arrays, "
                "stopping at the shorter), `core::iter::chain` (concatenates), and "
                "`core::iter::collect` (materializes a sequence — a `Range` is expanded to its "
                "integers, an Array passes through). `garnet-interp-v0.3/tests/"
                "core_iter_completion_dispatch.rs` proves them from source composed with the S21 "
                "higher-order `fold` (collect(1..4)+collect([10,20]) chained → fold-sum 36; "
                "zip stops at the shorter → [36,5,3,2,2]); bridge unit tests cover each plus the "
                "non-sequence type error."
            )
            if iter_completion_present
            else "No S28 core::iter completion proof present yet.",
            blocked_by=[],
            deferred=[
                "`collect` materializes a `Range` or passes an Array through — there is no other "
                "lazy sequence in managed mode to collect (eager map/filter already return arrays)",
            ]
            if iter_completion_present
            else [],
        ),
        ObjectiveLane(
            id="stability_error_enforcement",
            label="@stability error-level enforcement, opt-in (S29)",
            status="verified" if stability_errors_present else "planned",
            completion_percent=100.0 if stability_errors_present else 0.0,
            evidence=(
                "S29 ships the Layer Policy §4 'error-level enforcement is v0.8' line as an "
                "opt-in. `garnet-check-v0.3` adds a FATAL `CheckError::StabilityError` variant "
                "(listed in `CheckReport::ok()`), and `stability.rs` promotes experimental/"
                "deprecated call sites from non-fatal advisories to that fatal error when "
                "`GARNET_STABILITY_ERRORS=1|true` is set (frozen stays informational; `stable` "
                "silent). Default is unchanged warning-level, so existing programs/CI stay green. "
                "Proven end-to-end through the CLI with no garnet-cli change (it already exits on "
                "`report.ok()`): `garnet check examples/novel_04_*.garnet` warns + exits 0, while "
                "`GARNET_STABILITY_ERRORS=1 garnet check …` prints 'stability error:' and exits 1. "
                "Unit tests cover the policy (experimental/deprecated→error, frozen→info, "
                "default→warning) and the `ok()` fatal classification."
            )
            if stability_errors_present
            else "No S29 @stability error-level enforcement present yet.",
            blocked_by=[],
            deferred=[
                "Error mode is process-global via env var; per-call-site or per-source "
                "`@uses(experimental)` opt-out still needs the parser annotation variants "
                "(win-opus → mac-opus handoff), unchanged from S17",
            ]
            if stability_errors_present
            else [],
        ),
        ObjectiveLane(
            id="functional_core_composition",
            label="Functional-core composition capstone (S30)",
            status="verified" if functional_core_present else "planned",
            completion_percent=100.0 if functional_core_present else 0.0,
            evidence=(
                "S30 caps the S26-S28 arc: with the full functional `core::` surface now "
                "interpreter-dispatched, result/option/iter compose into railway-oriented "
                "pipelines from Garnet source. `garnet-interp-v0.3/tests/"
                "functional_core_composition.rs` exercises BOTH tracks — `core::iter` "
                "collect/map/fold → 20; `core::result` Ok-railway → 40 and Err-railway recovered "
                "via `or_else` → 0; `core::option` Some → 80 and None default → 7 ([20,40,0,80,7]). "
                "The deterministic, cross-platform `examples/novel_07_functional_core_pipeline.garnet` "
                "composes the happy path (iter → result → option → 80) and joins the "
                "novel-composition harness (now 7/7). Story: "
                "`C_Language_Specification/GARNET_NOVEL_COMPOSITIONS.md`."
            )
            if functional_core_present
            else "No S30 functional-core composition proof present yet.",
            blocked_by=[],
            deferred=[
                "Composition is pure managed-mode compute (no host effects); the host-effect "
                "composition is the separate S25 capstone",
            ]
            if functional_core_present
            else [],
        ),
        ObjectiveLane(
            id="reporter_determinism",
            evidence_class="committed",
            label="Readiness reporter determinism",
            status="verified",
            completion_percent=100.0,
            evidence=(
                "S31-PR2 split the readiness reporter into committed-truth lanes "
                "(scored from committed repo evidence; byte-identical on every machine) "
                "and local-evidence lanes (machine-specific live probes: Windows/Linux "
                "distribution build gates, the ~/Desktop domain-proof bundle, and promo "
                "render). The headline % and `--check-no-regression` now read committed-truth "
                "lanes only; local-evidence lanes are reported but never scored or gated."
            ),
            blocked_by=[],
            deferred=[
                "Per-lane committed-vs-live decomposition inside the wls/promo sub-reporters "
                "(S31-PR2 splits at the aggregation layer only)",
            ],
        ),
        ObjectiveLane(
            id="edition_compatibility",
            evidence_class="committed",
            label="Edition / compatibility model (S32)",
            status="verified",
            completion_percent=100.0,
            evidence=(
                "S32 installs the two-layer compatibility mechanism. Layer 1 (editions, "
                "parse-time): a `garnet_parser::Edition` registry (`v1.0` default + a "
                "registered `v2.0` that exists only to prove the mechanism), read from "
                "`[project].edition` in Garnet.toml (legacy `[package]` / `garnet-0.3` "
                "accepted as a deprecated alias with a one-line warning; an unknown edition "
                "is a hard error). The single edition-gated surface difference is the "
                "reserved word `async` (a free identifier under v1.0, rejected at lex time "
                "under v2.0), confined to the lexer so the grammar and AST are untouched. "
                "The one-canonical-IR invariant is proven: source valid in both editions "
                "parses to a byte-identical AST and an identical capability manifest "
                "(`Manifest::build` ast_hash). Layer 2 (runtime settings, GODEBUG-style): "
                "`GARNET_DEBUG=k=v` flips a CLI default (`diagnostics=verbose`) without "
                "changing the manifest; unknown keys warn, never error. Wired into "
                "`garnet check` and `garnet run --interp`; proven end-to-end (a v1.0 program "
                "passes, the identical v2.0 program fails on the reserved word) and by 22 "
                "unit/integration tests across garnet-parser and garnet-cli."
            ),
            blocked_by=[],
            deferred=[
                "Mechanism + invariant only: no per-edition syntax-migration catalog (future)",
                "`garnet run --vm` uses the default edition; the VM has a separate load path "
                "(harmonized in a later slice, per the S12/S14 split)",
                "No manifest `[runtime]` table yet — the GODEBUG layer is the env var only; a "
                "`[runtime]` table would be a spec change deferred to a future Handoff",
            ],
        ),
        ObjectiveLane(
            id="garnet_verify_gate",
            evidence_class="committed",
            label="One-command `garnet verify` acceptance gate (S33)",
            status="verified",
            completion_percent=100.0,
            evidence=(
                "S33 adds `garnet verify <path>` — a single acceptance gate (distinct from the "
                "2-arg `garnet verify <file> <manifest.json>` manifest verify; routed by "
                "positional-arg count). It runs edition-aware parse + safe-mode check over a file "
                "or every .garnet under a directory, and emits a fused merge-confidence band: the "
                "internal local band (5 clean / 4 advisory / 1 fatal) fused by `min` with an "
                "optional external-reviewer band (`--external-band`, Greptile at PR time) and a "
                "pluggable capability signal. Exits 0 on a clean tree, non-zero on a planted "
                "regression. Proven by 6 unit tests (band clamp, internal-band mapping, min-fusion "
                "incl. a weak signal capping a confident one, pending-capability no-op) + 4 "
                "integration tests + end-to-end CLI smoke (clean->0/band5, planted->1, "
                "--external-band caps via min, directory walk, 2-arg manifest verify preserved)."
            ),
            blocked_by=[],
            deferred=[
                "Capability-signal slot is a STUB until S37 diff-caps wires it in (it never "
                "lowers the fuse while pending)",
                "The gate's internal band is the LOCAL acceptance signal; the full PR "
                "falsification ledger + Greptile fusion is the dogfood-readiness skill's job, "
                "which this gate feeds — `garnet verify` does not itself run cargo/CI",
                "Test execution (`garnet test`) is not folded into the gate in S33; parse + "
                "safe-mode check is the acceptance signal",
            ],
        ),
        ObjectiveLane(
            id="capability_diff_caps",
            evidence_class="committed",
            label="diff-caps — capability-surface diff gate (S37)",
            status="verified",
            completion_percent=100.0,
            evidence=(
                "S37 ships the headline novelty. `garnet-check::diff_caps` diffs two S35 "
                "`CapabilitySurface`s into a `CapsDiff` (aggregate added/removed, functions "
                "added/removed/expanded, wildcard introduced) and answers `authority_expanded()` "
                "(a NEW aggregate capability or an introduced `@caps(*)` — a function re-declaring "
                "a cap already in the aggregate is NOT new program authority). `garnet diff-caps "
                "<old> <new>` exits non-zero iff authority expanded; `garnet verify "
                "--caps-baseline <old>` wires the diff into the S33 fused band via "
                "`capability_band` (5 if no expansion, 2 if expanded), completing the "
                "previously-stubbed capability-signal slot (`min` governs). Proven by 6 `caps_diff` "
                "unit tests + 4 integration tests (binary: expansion->exit1, reduction->exit0, "
                "identical->no-changes, verify --caps-baseline caps the fused band at 2/5) + the "
                "shared `surface_for_path` consolidation across caps/diff-caps/verify."
            ),
            blocked_by=[],
            deferred=[
                "diff-caps reads the DECLARED capability surface; it does not prove the absence "
                "of undeclared authority (that is the sandbox-policy job, S46)",
                "`garnet verify --caps-baseline` flags authority expansion via a low fused band "
                "(review signal); it does not hard-fail the gate on expansion — `garnet diff-caps` "
                "is the hard gate",
                "\"Two revisions\" are two source paths the caller supplies; S37 does not itself "
                "drive git",
            ],
        ),
        ObjectiveLane(
            id="seal_attestation",
            evidence_class="committed",
            label="seal — in-toto attestation predicate (S38)",
            status="verified",
            completion_percent=100.0,
            evidence=(
                "S38 ships `garnet seal <file>` (wrap-don't-rebuild): it emits a deterministic "
                "in-toto Statement (v1) whose subject is the program's BLAKE3 AST digest and "
                "whose predicate embeds the deterministic build manifest (`manifest.rs`) and the "
                "S36 capability manifest (the native SBOM-equivalent extension). The predicate is "
                "fully produced and validated as JSON; `garnet seal` detects `cosign` and prints "
                "the `cosign attest` command to sign it. Proven by 3 `seal` unit tests + 2 "
                "integration tests (binary emits a valid in-toto Statement with both embedded "
                "manifests; the cosign-availability note is present)."
            ),
            blocked_by=[],
            deferred=[
                "Garnet does NOT implement its own supply-chain signing (contract anchor): "
                "`cosign attest` signs the predicate. `cosign` is ABSENT in this environment, so "
                "the predicate is emitted UNSIGNED and the wrapper prints the sign command — it "
                "does not auto-sign",
                "External SBOM tools (syft/cyclonedx) are absent; the capability manifest is the "
                "native SBOM-equivalent until they are wired",
                "Per-file seal (per `garnet build`); per-package seal is a follow-up",
            ],
        ),
    ]

    # Committed-truth headline: only machine-independent lanes feed the score, so the
    # number is byte-identical on every machine (S31-PR2). Local-evidence lanes are
    # reported separately and never scored.
    committed = [lane for lane in lanes if lane.evidence_class == "committed"]
    percent = round(sum(_lane_score(lane) for lane in committed) / len(committed) * 100.0, 1)
    return MitReadinessStatus(
        source=str(ROOT),
        overall_status="verified" if percent == 100.0 else "active-partial",
        completion_percent=percent,
        current_truth=[
            "tracked implementation plan is complete",
            "goal remains active",
            "100% tracked slices is not full MIT/productization completion",
            "S1 LSP source is tracked separately from full editor dogfood completion",
        ],
        lanes=lanes,
    )


def render_markdown(status: MitReadinessStatus) -> str:
    lines = [
        "# Garnet MIT Readiness Objective Status",
        "",
        f"Source: `{status.source}`",
        "",
        f"Overall status: **{status.overall_status}**",
        f"Objective completion: **{status.completion_percent:.1f}%**",
        "",
        (
            "Current truth: the tracked implementation plan is complete, but that is "
            "not full MIT/productization completion."
        ),
        "",
        "## Committed truth (scored, gated — byte-identical on every machine)",
        "",
        "| Lane | Status | Percent | Evidence | Blocked / deferred |",
        "|---|---|---:|---|---|",
    ]

    def _row(lane: ObjectiveLane) -> str:
        blockers = _dedupe(lane.blocked_by + lane.deferred)
        blocked_text = "<br>".join(blockers) if blockers else "None"
        return (
            f"| {lane.label} | `{lane.status}` | {lane.completion_percent:.1f}% | "
            f"{lane.evidence} | {blocked_text} |"
        )

    committed = [lane for lane in status.lanes if lane.evidence_class == "committed"]
    local = [lane for lane in status.lanes if lane.evidence_class != "committed"]
    for lane in committed:
        lines.append(_row(lane))
    if local:
        lines += [
            "",
            "## Local evidence (machine-specific; NOT scored, NOT gated)",
            "",
            (
                "These lanes derive from live machine probes (Windows/Linux build "
                "artifacts, the `~/Desktop` domain-proof bundle, local promo render). "
                "Their percentages vary by machine and are excluded from the headline "
                "% and the `--check-no-regression` gate."
            ),
            "",
            "| Lane | Status | Percent (local) | Evidence | Blocked / deferred |",
            "|---|---|---:|---|---|",
        ]
        for lane in local:
            lines.append(_row(lane))
    return "\n".join(lines) + "\n"


def committed_only_status(status: MitReadinessStatus) -> MitReadinessStatus:
    """Return the machine-independent readiness surface.

    Full MIT readiness includes local evidence lanes such as Windows installer
    bundles and Desktop promo artifacts. This view is intentionally smaller:
    only committed evidence remains, and the absolute checkout path is replaced
    so JSON/Markdown comparisons are meaningful across machines.
    """
    return MitReadinessStatus(
        source="committed-truth",
        overall_status=status.overall_status,
        completion_percent=status.completion_percent,
        current_truth=[
            *status.current_truth,
            "committed-only surface excludes local machine evidence",
        ],
        lanes=[lane for lane in status.lanes if lane.evidence_class == "committed"],
    )


DEFAULT_BASELINE = (
    ROOT / "F_Project_Management" / "GARNET_v0_5_READINESS_BASELINE.json"
)


def _baseline_lanes(baseline: dict) -> dict[str, float]:
    """Extract `id -> completion_percent` from a baseline JSON snapshot.

    The baseline is the output of this same script's --format json invocation,
    captured at a known-good point and committed to the repo. Lanes not present
    in the baseline cannot regress (the gate is one-directional).
    """
    return {lane["id"]: float(lane["completion_percent"]) for lane in baseline.get("lanes", [])}


def check_no_regression(
    status: MitReadinessStatus, baseline_path: Path
) -> tuple[list[str], list[str]]:
    """Return (regressions, missing_lanes).

    A regression is a lane whose live percent dropped below the baseline.
    A missing_lane is a lane present in baseline but absent from live output
    (a slice was deleted or renamed without updating the baseline).

    Only committed-truth lanes are gated. Local-evidence lanes (machine-specific
    live probes) are excluded so the gate is byte-identical on every machine
    (S31-PR2): a Mac without Windows build artifacts must not be flagged as a
    regression against a baseline captured on a Windows-capable machine.
    """
    if not baseline_path.exists():
        return (
            [],
            [f"baseline missing at {baseline_path}; run with --format json > {baseline_path} to seed."],
        )
    baseline = json.loads(baseline_path.read_text(encoding="utf-8"))
    baseline_pct = _baseline_lanes(baseline)
    live_committed_ids = {lane.id for lane in status.lanes if lane.evidence_class == "committed"}
    live_local_ids = {lane.id for lane in status.lanes if lane.evidence_class != "committed"}
    live_pct = {lane.id: lane.completion_percent for lane in status.lanes if lane.id in live_committed_ids}
    regressions: list[str] = []
    missing: list[str] = []
    for lane_id, baseline_value in baseline_pct.items():
        if lane_id in live_local_ids:
            # Present in live but classified local-evidence (or reclassified from
            # committed): machine-variable, so never gated.
            continue
        if lane_id not in live_committed_ids:
            # Absent from live entirely (and not a live local lane): a slice was
            # deleted or renamed without updating the baseline.
            missing.append(lane_id)
            continue
        if live_pct[lane_id] + 1e-9 < baseline_value:
            regressions.append(
                f"{lane_id}: live {live_pct[lane_id]:.1f}% < baseline {baseline_value:.1f}%"
            )
    return regressions, missing


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--format",
        choices=("markdown", "json"),
        default="markdown",
        help="output format",
    )
    parser.add_argument(
        "--committed-only",
        action="store_true",
        help=(
            "Emit only committed-truth lanes and normalize the source field. "
            "Use this for byte-comparable cross-machine readiness snapshots."
        ),
    )
    parser.add_argument(
        "--check-no-regression",
        action="store_true",
        help=(
            "Exit 1 if any lane in the committed baseline has regressed in the "
            "current run. Used by CI to prevent silent readiness drift across "
            "PRs. Baseline path defaults to "
            f"{DEFAULT_BASELINE.relative_to(ROOT)} and can be overridden with "
            "--baseline."
        ),
    )
    parser.add_argument(
        "--baseline",
        type=Path,
        default=DEFAULT_BASELINE,
        help="Path to the readiness baseline JSON (for --check-no-regression).",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    status = read_status()
    if args.committed_only:
        status = committed_only_status(status)
    if args.format == "json":
        print(json.dumps(asdict(status), indent=2))
    else:
        print(render_markdown(status), end="")
    if args.check_no_regression:
        regressions, missing = check_no_regression(status, args.baseline)
        if regressions or missing:
            print("", file=sys.stderr)
            print("Readiness regression detected vs baseline:", file=sys.stderr)
            for r in regressions:
                print(f"  - regressed: {r}", file=sys.stderr)
            for m in missing:
                print(f"  - missing lane (baseline-only): {m}", file=sys.stderr)
            return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
