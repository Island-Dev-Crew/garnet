//! Integration test for S10 advisory mode.
//!
//! Walks the corpus under `tests/suggest_corpus_fixtures/` and confirms that
//! every fixture file triggers at least one suggestion under the matching
//! rule. The contract requires at least 3 distinct patterns produce
//! suggestions on the corpus.

use garnet_check::suggest::{suggest_for_module, Rule};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/suggest_corpus_fixtures")
}

fn suggestions_for(path: &Path) -> Vec<garnet_check::suggest::Suggestion> {
    let src = fs::read_to_string(path).expect("fixture readable");
    let module = garnet_parser::parse_source(&src).expect("fixture parses");
    suggest_for_module(&module)
}

#[test]
fn corpus_exists_and_is_non_empty() {
    let dir = fixtures_dir();
    let entries: Vec<_> = fs::read_dir(&dir)
        .expect("corpus dir readable")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("garnet"))
        .collect();
    assert!(
        !entries.is_empty(),
        "no .garnet fixture files in {}",
        dir.display()
    );
}

#[test]
fn every_fixture_triggers_at_least_one_suggestion() {
    let dir = fixtures_dir();
    let mut paths: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("corpus dir readable")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("garnet"))
        .collect();
    paths.sort();
    for path in &paths {
        let suggestions = suggestions_for(path);
        assert!(
            !suggestions.is_empty(),
            "fixture produced no suggestions: {} — every fixture must \
            demonstrate at least one rule. Add a rule or remove the fixture.",
            path.display()
        );
    }
}

#[test]
fn at_least_three_distinct_rules_fire_across_corpus() {
    let dir = fixtures_dir();
    let mut rules_fired: HashSet<Rule> = HashSet::new();
    for entry in fs::read_dir(&dir).expect("corpus dir readable").flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("garnet") {
            continue;
        }
        for s in suggestions_for(&path) {
            rules_fired.insert(s.rule);
        }
    }
    assert!(
        rules_fired.len() >= 3,
        "S10 contract requires at least 3 distinct rules to fire on the \
        corpus; got {} ({rules_fired:?})",
        rules_fired.len()
    );
}

#[test]
fn each_named_fixture_triggers_its_target_rule() {
    let cases = [
        ("managed_fn_missing_caps.garnet", Rule::ManagedFnMissingCaps),
        ("long_parameter_list.garnet", Rule::LongParameterList),
        ("empty_function_body.garnet", Rule::EmptyFunctionBody),
    ];
    let dir = fixtures_dir();
    for (filename, expected_rule) in cases {
        let path = dir.join(filename);
        let suggestions = suggestions_for(&path);
        assert!(
            suggestions.iter().any(|s| s.rule == expected_rule),
            "{filename} did not trigger {expected_rule:?}; got {suggestions:?}"
        );
    }
}
