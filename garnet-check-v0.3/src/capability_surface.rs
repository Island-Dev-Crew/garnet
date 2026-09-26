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

/// The label for an impl block's owning type in per-function names: the type's
/// full written path (`R`, or `a::R` when the impl names a qualified type), so
/// two impls whose types share a last segment are not collapsed into one name.
fn type_label(ty: &TypeExpr) -> String {
    match ty {
        TypeExpr::Named { path, .. } if !path.is_empty() => path.join("::"),
        _ => "impl".to_string(),
    }
}

/// Collect every capability-bearing function in the module tree — top-level
/// functions, **impl-block methods**, and functions in nested modules — as
/// `(qualified_name, &FnDef)`. S114 closed a hole where impl-method `@caps` was
/// enforced at runtime (the interpreter installs the guard for any managed `FnDef`)
/// but invisible here, so a file-/net-reading impl method reported an empty surface
/// and slipped past `diff-caps`, the seal manifest, and the agent-loop gate.
///
/// T5a (C1-01): names carry their module path, so `module a { def f }` is `a::f`
/// and an impl method inside it is `a::Type::m`. Top-level functions and methods
/// of an impl on an unqualified type keep their names; an impl written with a
/// path (`impl a::R`) names its methods by that full path. Before this, same-named
/// functions in two modules shared one name and `diff-caps` kept only the last.
fn collect_cap_fns<'a>(items: &'a [Item], prefix: &str, out: &mut Vec<(String, &'a FnDef)>) {
    for item in items {
        match item {
            Item::Fn(f) => out.push((format!("{prefix}{}", f.name), f)),
            Item::Impl(block) => {
                let owner = type_label(&block.target);
                for m in &block.methods {
                    out.push((format!("{prefix}{owner}::{}", m.name), m));
                }
            }
            Item::Module(m) => collect_cap_fns(&m.items, &format!("{prefix}{}::", m.name), out),
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
    /// Whether any `@caps(*)` wildcard appears. The checker accepts a wildcard;
    /// `diff-caps` treats a newly introduced one as authority expansion.
    pub has_wildcard: bool,
}

/// Derive the [`CapabilitySurface`] from a parsed module's top-level functions.
pub fn capability_surface(module: &Module) -> CapabilitySurface {
    let mut aggregate: BTreeSet<String> = BTreeSet::new();
    let mut per_function: Vec<(String, Vec<String>)> = Vec::new();
    let mut has_wildcard = false;

    let mut fns: Vec<(String, &FnDef)> = Vec::new();
    collect_cap_fns(&module.items, "", &mut fns);

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

    // ── T5a C1-01: per-function names carry their module and impl path ──

    #[test]
    fn functions_in_modules_are_qualified_by_module_path() {
        let s = surface(
            "module a {\n  @caps(fs)\n  def f() -> int { 0 }\n}\nmodule b {\n  @caps(net)\n  def f() -> int { 0 }\n}\n",
        );
        assert_eq!(
            s.per_function,
            vec![
                ("a::f".to_string(), vec!["fs".to_string()]),
                ("b::f".to_string(), vec!["net".to_string()]),
            ],
            "same-named functions in two modules must not collide"
        );
    }

    #[test]
    fn nested_modules_and_impl_methods_carry_the_full_path() {
        let s = surface(
            "module a {\n  module b {\n    @caps(env)\n    def g() -> int { 0 }\n  }\n  struct R {}\n  impl R {\n    @caps(fs)\n    def m(self) -> int { 0 }\n  }\n}\n",
        );
        let names: Vec<&str> = s.per_function.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, vec!["a::R::m", "a::b::g"]);
    }

    #[test]
    fn a_top_level_impl_on_a_path_qualified_type_is_named_by_the_full_path() {
        // T5a (Codex review of #598): disclosed in the CHANGELOG. `impl a::R` and
        // `impl b::R` would both have been `R::m` and collided in diff-caps.
        let s = surface(
            "@caps(fs)\ndef main() -> int { 0 }\nimpl a::R {\n  @caps(fs)\n  def m(self) -> int { 0 }\n}\nimpl b::R {\n  @caps(net)\n  def m(self) -> int { 0 }\n}\n",
        );
        let names: Vec<&str> = s.per_function.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, vec!["a::R::m", "b::R::m", "main"]);
    }

    #[test]
    fn top_level_names_stay_bare() {
        // Top-level functions and methods of an impl on an unqualified type keep
        // their names, so these programs' capability surfaces and manifests do
        // not change. (Their seal bytes change once with C2-07.)
        let s = surface(
            "struct R {}\nimpl R {\n  @caps(fs)\n  def m(self) -> int { 0 }\n}\n@caps(net)\ndef f() -> int { 0 }\n",
        );
        let names: Vec<&str> = s.per_function.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, vec!["R::m", "f"]);
    }

    // ── A1 (element 6, surface side): surface == the enforceable-by-declaration set ──

    /// `collect_cap_fns` must descend EXACTLY the `Item` variants that can host a
    /// declarable-`@caps`, executable function — the "no enforced-but-invisible
    /// function" invariant in test form. The exhaustive match below lists all 12
    /// `Item` variants, so adding a 13th variant — or giving a currently
    /// signature-only / non-`@caps` variant an annotated executable body — fails to
    /// COMPILE here until the descender (`collect_cap_fns`) and this classification
    /// are revisited together. The behavioural assertions bind the classification to
    /// the real `capability_surface` output so the two cannot silently drift.
    #[test]
    fn collect_cap_fns_descends_exactly_the_caps_bearing_variants() {
        // Single, exhaustive source of truth for "does the surface descend this?".
        fn descended(item: &Item) -> bool {
            match item {
                // Host a declarable-`@caps`, executable function:
                Item::Fn(_) | Item::Impl(_) | Item::Module(_) => true,
                // Cannot host a declarable-`@caps` executable function:
                Item::Use(_)        // import only, no body
                | Item::Memory(_)   // store declaration, no fn
                | Item::Actor(_)    // handlers carry no `@caps` annotation (HandlerDecl)
                | Item::Struct(_)   // `@caps` is on the type; methods live in `Impl`
                | Item::Enum(_)     // type data, no body
                | Item::Trait(_)    // signatures only, no executable body
                | Item::Protocol(_) // signatures only, no executable body
                | Item::Const(_)    // initializer expression; defended by the load-time
                | Item::Let(_) => false, // entry frame + deny-by-default, NOT the surface
            }
        }
        // Behavioural binding: a real `@caps` fn in each descended position IS surfaced.
        assert_eq!(
            surface("@caps(fs)\ndef f() -> int { 1 }\n").aggregate,
            vec!["fs"]
        );
        assert_eq!(
            surface("struct R {}\nimpl R {\n  @caps(net)\n  def m(self) -> int { 0 }\n}\n")
                .aggregate,
            vec!["net"]
        );
        assert_eq!(
            surface("module m {\n  @caps(env)\n  def f() -> int { 0 }\n}\n").aggregate,
            vec!["env"]
        );
        // And the classifier agrees with the parsed shape of each descended kind.
        let parsed = parse_source(
            "@caps(fs)\ndef f() -> int { 1 }\nstruct R {}\nimpl R {\n  def m(self) -> int { 0 }\n}\nmodule n {}\n",
        )
        .expect("parses");
        let kinds: Vec<bool> = parsed.items.iter().map(descended).collect();
        assert_eq!(
            kinds,
            vec![true, false, true, true],
            "Fn, Struct, Impl, Module"
        );
    }

    /// Element-6 claim pin: top-level `let`/`const` initializers declare no
    /// `@caps`, so they are NOT in the static surface. Their host authority is gated
    /// at runtime by the load-time entry frame + deny-by-default mediation
    /// (S114-FIX-2), verified per-process in `garnet-cli/tests/s114_residual_lanes.rs`
    /// — NOT by this surface. This test exists so no future reader mistakes the
    /// 3-arm surface for total enforcement coverage.
    #[test]
    fn top_level_let_const_initializers_are_not_a_surface_concern() {
        let s = surface("let x = 1\nconst Y = 2\n@caps()\ndef main() -> int { 0 }\n");
        assert!(
            s.aggregate.is_empty(),
            "let/const declare no caps: {:?}",
            s.aggregate
        );
    }
}
