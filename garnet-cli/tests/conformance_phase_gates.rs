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
fn parser_parity_conformance_tests_are_active() {
    let source = fs::read_to_string(repo_root().join("garnet-cli/tests/conformance_skeleton.rs"))
        .expect("conformance skeleton");
    for name in [
        "parser_parity_top_level_protocol_and_dyn_trait_parse",
        "parser_parity_yield_next_dynamic_and_nonsendable_parse",
    ] {
        let idx = source.find(name).expect("parser parity test exists");
        let prefix = &source[..idx];
        let nearby = &prefix[prefix.len().saturating_sub(160)..];
        assert!(
            !nearby.contains("#[ignore"),
            "{name} must remain an active conformance test"
        );
    }
}

#[test]
fn deferred_semantic_handles_remain_explicit() {
    let source = fs::read_to_string(repo_root().join("garnet-cli/tests/conformance_skeleton.rs"))
        .expect("conformance skeleton");
    for name in [
        "deferred_arc_cycle_detection",
        "deferred_blocks_and_yield",
        "deferred_nll_lifetime_inference",
        "partial_borrow_rule_suite",
        "deferred_trait_coherence",
        "parsed_only_monomorphization",
        "deferred_dynamic_dispatch",
        "deferred_structural_protocols",
    ] {
        let idx = source.find(name).expect("deferred handle exists");
        let prefix = &source[..idx];
        let nearby = &prefix[prefix.len().saturating_sub(220)..];
        assert!(
            nearby.contains("#[ignore = \"Mini-Spec"),
            "{name} must carry an explicit Mini-Spec ignore reason"
        );
    }
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
