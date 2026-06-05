//! S35 — the canonical capability surface.
//!
//! The `@caps(...)` annotation syntax already exists (v3.4 CapCaps); this module
//! adds the first-class, deterministic *surface* derived from it — the normalized
//! declared-capability artifact the S36 capability manifest is built on, and that
//! S37 `diff-caps` compares across revisions.
//!
//! It is purely syntactic: it reads each function's declared `@caps(...)` —
//! top-level functions, **impl-block methods**, and functions in nested modules
//! (S114 closed a hole where impl-method caps were enforced but invisible here) —
//! and normalizes via the canonical [`Capability::as_str`] (so
//! `NetInternal` → `"net_internal"`, `Other("x")` → `"x"`, `Wildcard` → `"*"` —
//! NOT the `Debug` rendering some call sites used). Every list is sorted and
//! deduplicated, so the surface is byte-stable across runs and machines.

use garnet_parser::ast::{Annotation, Capability, FnDef, Item, Module, TypeExpr};
use std::collections::BTreeSet;

/// A short label for an impl block's owning type, for per-function names.
fn type_label(ty: &TypeExpr) -> String {
    match ty {
        TypeExpr::Named { path, .. } => path.last().cloned().unwrap_or_else(|| "impl".to_string()),
        _ => "impl".to_string(),
    }
}

/// Collect every capability-bearing function in the module tree — top-level
/// functions, **impl-block methods**, and functions in nested modules — as
/// `(display_name, &FnDef)`. S114 closed a hole where impl-method `@caps` was
/// enforced at runtime (the interpreter installs the guard for any managed `FnDef`)
/// but invisible here, so a file-/net-reading impl method reported an empty surface
/// and slipped past `diff-caps`, the seal manifest, and the agent-loop gate.
fn collect_cap_fns<'a>(items: &'a [Item], out: &mut Vec<(String, &'a FnDef)>) {
    for item in items {
        match item {
            Item::Fn(f) => out.push((f.name.clone(), f)),
            Item::Impl(block) => {
                let owner = type_label(&block.target);
                for m in &block.methods {
                    out.push((format!("{owner}::{}", m.name), m));
                }
            }
            Item::Module(m) => collect_cap_fns(&m.items, out),
            _ => {}
        }
    }
}

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

    let mut fns: Vec<(String, &FnDef)> = Vec::new();
    collect_cap_fns(&module.items, &mut fns);

    for (name, f) in fns {
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
            per_function.push((name, fn_caps.into_iter().collect()));
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
    fn impl_method_caps_are_in_the_surface() {
        // S114 red-team hole: `@caps` on an impl method is enforced at runtime but
        // was invisible to the surface, so an impl-method file-read slipped past
        // diff-caps / the seal manifest / the agent-loop. The surface must now
        // include impl-method (and nested-module) capabilities.
        let s = surface(
            "struct Reader {}\nimpl Reader {\n  @caps(fs)\n  def read(self) -> int { 0 }\n}\n@caps()\ndef main() -> int { 0 }\n",
        );
        assert_eq!(
            s.aggregate,
            vec!["fs"],
            "impl-method @caps(fs) must be in the aggregate"
        );
        assert!(
            s.per_function
                .iter()
                .any(|(n, c)| n == "Reader::read" && c == &["fs"]),
            "impl method must appear in per_function: {:?}",
            s.per_function
        );
    }

    #[test]
    fn nested_module_fn_caps_are_in_the_surface() {
        let s = surface("module m {\n  @caps(net)\n  def f() -> int { 0 }\n}\n");
        assert_eq!(s.aggregate, vec!["net"]);
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
