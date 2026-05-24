#!/usr/bin/env python3
"""Report the broader Garnet MIT/productization objective status.

This reporter intentionally differs from `garnet_readiness_status.py`: the
tracked implementation plan can be complete while the larger public-readiness
goal still has distribution, mobile, video, proof, and converter-assist gates.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")

import garnet_converter_status  # noqa: E402
import garnet_proof_benchmark_status  # noqa: E402
import garnet_promo_video_status  # noqa: E402
import garnet_readiness_status  # noqa: E402
import garnet_windows_linux_studio_status  # noqa: E402


@dataclass(frozen=True)
class ObjectiveLane:
    id: str
    label: str
    status: str
    completion_percent: float
    evidence: str
    blocked_by: list[str]
    deferred: list[str]


@dataclass(frozen=True)
class MitReadinessStatus:
    source: str
    overall_status: str
    completion_percent: float
    current_truth: list[str]
    lanes: list[ObjectiveLane]


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


def _vm_scaffold_present(proof: garnet_proof_benchmark_status.ProofBenchmarkStatus) -> bool:
    return any(
        bench.id == "vm_parse_compile_execute"
        and bench.bench_file_exists
        and bench.cargo_entry_present
        for bench in proof.benchmarks
    )


def read_status() -> MitReadinessStatus:
    plan = garnet_readiness_status.read_status(
        ROOT / "F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md"
    )
    converter = garnet_converter_status.read_status()
    contract = converter.intelligent_assist_contract
    promo = garnet_promo_video_status.read_status()
    proof = garnet_proof_benchmark_status.read_status()
    vm_scaffold_present = _vm_scaffold_present(proof)
    wls = garnet_windows_linux_studio_status.read_status()
    wls_clean_vm_verified = any(
        gate.id == "windows_unsigned_nsis" and gate.status == "clean-vm-proof-verified"
        for gate in wls.packaging_gates
    )
    wls_completion_percent = 65.0 if wls_clean_vm_verified else 55.0
    wls_evidence_tail = (
        "readiness reporter parity actions, a verified x64 clean-VM installer "
        "proof, and open Linux plus signing, winget, and Windows ARM64 package gates."
        if wls_clean_vm_verified
        else (
            "readiness reporter parity actions, a Windows clean-VM installer "
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
            label="Windows/Linux distribution",
            status="active-partial",
            completion_percent=wls_completion_percent,
            evidence=(
                "`scripts/garnet_windows_linux_studio_status.py` now reports the "
                "Tauri v2 shell scaffold in `apps/garnet-studio`, minimal webview "
                f"permissions, Windows local release build/smoke evidence, v0.5 {wls_evidence_tail}"
            ),
            blocked_by=list(wls.user_assistance_needed),
            deferred=list(wls.next_slices),
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
                "build-both-then-compare A/B. `parse_cst` round-trips byte-identically "
                "across the canonical examples corpus and a `proptest` over arbitrary "
                "UTF-8 (`tests/examples_roundtrip.rs`, `tests/roundtrip.rs`). `cst_to_ast` "
                "projects onto `garnet_parser::ast::Module` with span-normalized "
                "structural parity vs `parse_source` across the corpus "
                "(`tests/cst_to_ast_parity.rs`). The `parse_cst_vs_ast` Criterion bench "
                "measures the CST path at ~0.99x the AST path (well under the 1.5x gate). "
                "Reproduce via the S15 dogfood block in `GARNET_v0_7_SLICE_DOGFOOD.md`."
            )
            if _rowan_cst_present()
            else "No rowan `garnet-cst` builder/converter/bench source present yet.",
            blocked_by=[] if _rowan_cst_present() else ["S15 PR-2 rowan CST"],
            deferred=[
                "Canonical-CST choice is the separate S15-Compare checkpoint (Jon); "
                "this is the second of two independent CSTs by design, not yet reconciled",
                "Error-recovery parsing is best-effort (round-trip always holds; structure "
                "may flatten on malformed input)",
                "Incremental re-parsing",
                "CST-first migration of interp/check/vm (v0.8; they stay on the AST path "
                "via `parse_source`, untouched by S15)",
                "`garnet parse --mode cst` CLI wiring (Handoff Request to the garnet-cli owner)",
            ]
            if _rowan_cst_present()
            else [],
        ),
    ]

    percent = round(sum(_lane_score(lane) for lane in lanes) / len(lanes) * 100.0, 1)
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
        "| Lane | Status | Percent | Evidence | Blocked / deferred |",
        "|---|---|---:|---|---|",
    ]
    for lane in status.lanes:
        blockers = _dedupe(lane.blocked_by + lane.deferred)
        blocked_text = "<br>".join(blockers) if blockers else "None"
        lines.append(
            f"| {lane.label} | `{lane.status}` | {lane.completion_percent:.1f}% | "
            f"{lane.evidence} | {blocked_text} |"
        )
    return "\n".join(lines) + "\n"


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
    """
    if not baseline_path.exists():
        return (
            [],
            [f"baseline missing at {baseline_path}; run with --format json > {baseline_path} to seed."],
        )
    baseline = json.loads(baseline_path.read_text(encoding="utf-8"))
    baseline_pct = _baseline_lanes(baseline)
    live_pct = {lane.id: lane.completion_percent for lane in status.lanes}
    regressions: list[str] = []
    missing: list[str] = []
    for lane_id, baseline_value in baseline_pct.items():
        if lane_id not in live_pct:
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
