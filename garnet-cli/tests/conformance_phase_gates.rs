//! Repo-level phase-gate checks for the language completion plan.
//!
//! These tests make the roadmap auditable: parser-parity progress, deferred
//! semantic handles, and dogfood readiness links must stay visible in tracked
//! files instead of living only in handoff prose.

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn implemented_conformance_tests_are_active() {
    let source = fs::read_to_string(repo_root().join("garnet-cli/tests/conformance_skeleton.rs"))
        .expect("conformance skeleton");
    for name in [
        "parser_parity_top_level_protocol_and_dyn_trait_parse",
        "parser_parity_yield_next_dynamic_and_nonsendable_parse",
        "deferred_blocks_and_yield",
        "deferred_dynamic_dispatch",
        "static_impl_dispatch_and_method_missing",
        "deferred_structural_protocols",
        "deferred_nll_lifetime_inference",
        "partial_borrow_rule_suite",
        "deferred_full_borrow_rule_suite",
        "deferred_trait_coherence",
        "generic_instantiation_runs_without_monomorphization_claims",
        "deferred_arc_cycle_detection",
    ] {
        let idx = source.find(name).expect("active conformance test exists");
        let prefix = &source[..idx];
        let nearby = &prefix[prefix.len().saturating_sub(160)..];
        assert!(
            !nearby.contains("#[ignore"),
            "{name} must remain an active conformance test"
        );
    }
}

#[test]
fn full_borrow_handle_documents_active_partial_scope() {
    let source = fs::read_to_string(repo_root().join("garnet-cli/tests/conformance_skeleton.rs"))
        .expect("conformance skeleton");
    let name = "deferred_full_borrow_rule_suite";
    let idx = source.find(name).expect("full borrow handle exists");
    let prefix = &source[..idx];
    let nearby = &prefix[prefix.len().saturating_sub(220)..];
    assert!(
        !nearby.contains("#[ignore"),
        "{name} must remain an active partial conformance test"
    );
    let body = &source[idx..source.len().min(idx + 4_800)];
    assert!(
        body.contains("safe-mode B5") && body.contains("drop discipline"),
        "{name} must document the active B5 drop-discipline subset"
    );
    assert!(
        body.contains("returning_branch_liveness"),
        "{name} must document the active branch-return liveness subset"
    );
    assert!(
        body.contains("returning_loop_liveness"),
        "{name} must document the active direct-return loop-body liveness subset"
    );
    assert!(
        body.contains("returning_for_liveness"),
        "{name} must document the active direct-return for-body liveness subset"
    );
    assert!(
        body.contains("for_loop_shadow_preserves_outer_move"),
        "{name} must document the active for-loop variable scoping subset"
    );
}

#[test]
fn roadmap_links_conformance_and_dogfood_gates() {
    let roadmap = fs::read_to_string(
        repo_root()
            .join("F_Project_Management/ROADMAPS/GARNET_v0_5_LANGUAGE_COMPLETION_ROADMAP.md"),
    )
    .expect("language completion roadmap");
    for needle in [
        "parser_parity_top_level_protocol_and_dyn_trait_parse",
        "canonical_mvp_examples_emit_stable_results",
        "dogfood readiness",
        "Phase 7",
    ] {
        assert!(roadmap.contains(needle), "roadmap missing {needle}");
    }
}

#[test]
fn completion_plan_preserves_current_vs_deferred_truth() {
    let plan = fs::read_to_string(
        repo_root().join("F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md"),
    )
    .expect("language completion implementation plan");
    for needle in [
        "10 MVP app corpus",
        "Blocks, `yield`, `next` runtime semantics",
        "Dynamic method dispatch tables",
        "Structural protocol satisfaction and runtime casts",
        "Actor protocol enforcement and `Sendable`",
        "Rust-grade NLL and borrow rules",
        "Trait coherence",
        "Monomorphization",
        "Formal RustBelt/Iris/Coq proof",
        "Native compiler",
        "Empirical PLDI-grade validation",
        "Done Means Executable",
    ] {
        assert!(plan.contains(needle), "completion plan missing {needle}");
    }
}

#[test]
fn security_dogfood_rubric_is_indexed_and_actionable() {
    let index = fs::read_to_string(repo_root().join("F_Project_Management/v0_5_ROADMAP_INDEX.md"))
        .expect("roadmap index");
    assert!(
        index.contains("GARNET_SECURITY_DOGFOOD_RUBRIC.md"),
        "roadmap index must include the security dogfood rubric"
    );

    let rubric = fs::read_to_string(
        repo_root().join("F_Project_Management/DOGFOOD/GARNET_SECURITY_DOGFOOD_RUBRIC.md"),
    )
    .expect("security dogfood rubric");
    for needle in [
        "Frontend/XSS",
        "Backend/API",
        "Database",
        "Command execution",
        "Filesystem authority",
        "Network authority",
        "Supply chain",
        "Release integrity",
        "cargo audit",
        "cargo deny --all-features check",
        "security_coverage_gaps",
        "unreviewed_high_risk_trust_boundaries",
    ] {
        assert!(rubric.contains(needle), "security rubric missing {needle}");
    }
}
