//! S35 — the canonical capability surface.
//!
//! The `@caps(...)` annotation syntax already exists (v3.4 CapCaps); this module
//! adds the first-class, deterministic *surface* derived from it — the normalized
//! declared-capability artifact the S36 capability manifest is built on, and that
//! S37 `diff-caps` compares across revisions.
//!
//! It is purely syntactic: it reads each top-level function's declared
//! `@caps(...)` and normalizes via the canonical [`Capability::as_str`] (so
//! `NetInternal` → `"net_internal"`, `Other("x")` → `"x"`, `Wildcard` → `"*"` —
//! NOT the `Debug` rendering some call sites used). Every list is sorted and
//! deduplicated, so the surface is byte-stable across runs and machines.

use garnet_parser::ast::{Annotation, Capability, Item, Module};
use std::collections::BTreeSet;

/// A program's declared capability surface — the canonical input to the S36
/// capability manifest. Deterministic: every list is sorted and deduplicated.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapabilitySurface {
    /// Sorted, deduplicated union of every capability any function declares.
    pub aggregate: Vec<String>,
    /// Per-function declared caps: sorted by function name; each cap list sorted
    /// and deduplicated. Only functions that carry an `@caps(...)` appear.
    pub per_function: Vec<(String, Vec<String>)>,
    /// Whether any `@caps(*)` wildcard appears (debug-only; CI rejects it).
    pub has_wildcard: bool,
}

/// Derive the [`CapabilitySurface`] from a parsed module's top-level functions.
pub fn capability_surface(module: &Module) -> CapabilitySurface {
    let mut aggregate: BTreeSet<String> = BTreeSet::new();
    let mut per_function: Vec<(String, Vec<String>)> = Vec::new();
    let mut has_wildcard = false;

    for item in &module.items {
        let Item::Fn(f) = item else { continue };
        let mut declared = false;
        let mut fn_caps: BTreeSet<String> = BTreeSet::new();
        for ann in &f.annotations {
            if let Annotation::Caps(caps, _) = ann {
                declared = true;
                for c in caps {
                    if matches!(c, Capability::Wildcard) {
                        has_wildcard = true;
                    }
                    let s = c.as_str().to_string();
                    fn_caps.insert(s.clone());
                    aggregate.insert(s);
                }
            }
        }
        if declared {
            per_function.push((f.name.clone(), fn_caps.into_iter().collect()));
        }
    }
    per_function.sort_by(|a, b| a.0.cmp(&b.0));

    CapabilitySurface {
        aggregate: aggregate.into_iter().collect(),
        per_function,
        has_wildcard,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use garnet_parser::parse_source;

    fn surface(src: &str) -> CapabilitySurface {
        capability_surface(&parse_source(src).expect("parses"))
    }

    #[test]
    fn aggregate_is_sorted_and_deduped_across_functions() {
        let s = surface("@caps(net, fs)\ndef a() { 1 }\n@caps(fs)\ndef b() { 1 }\n");
        assert_eq!(s.aggregate, vec!["fs", "net"]);
    }

    #[test]
    fn per_function_sorted_by_name_with_sorted_caps() {
        let s = surface("@caps(net, fs)\ndef zebra() { 1 }\n@caps(time)\ndef alpha() { 1 }\n");
        assert_eq!(s.per_function[0].0, "alpha");
        assert_eq!(s.per_function[1].0, "zebra");
        assert_eq!(s.per_function[1].1, vec!["fs", "net"]);
    }

    #[test]
    fn empty_caps_is_a_declared_function_with_no_caps() {
        let s = surface("@caps()\ndef main() { 1 }\n");
        assert_eq!(
            s.per_function,
            vec![("main".to_string(), Vec::<String>::new())]
        );
        assert!(s.aggregate.is_empty());
    }

    #[test]
    fn uses_canonical_strings_not_debug() {
        // `net_internal`, not Debug's "netinternal" — the bug this surface fixes.
        let s = surface("@caps(net_internal)\ndef f() { 1 }\n");
        assert_eq!(s.aggregate, vec!["net_internal"]);
    }

    #[test]
    fn wildcard_is_flagged_and_canonical() {
        let s = surface("@caps(*)\ndef f() { 1 }\n");
        assert!(s.has_wildcard);
        assert_eq!(s.aggregate, vec!["*"]);
    }

    #[test]
    fn functions_without_caps_are_absent_from_per_function() {
        let s = surface("def plain() { 1 }\n@caps(fs)\ndef g() { 1 }\n");
        assert_eq!(s.per_function.len(), 1);
        assert_eq!(s.per_function[0].0, "g");
    }

    #[test]
    fn surface_is_deterministic() {
        let src = "@caps(net, fs)\ndef a() { 1 }\n@caps(time)\ndef b() { 1 }\n";
        assert_eq!(surface(src), surface(src));
    }
}
