//! Regression: impl-method capability identity in the call graph.
//!
//! Two bugs in `caps_graph` are closed here, both proven by deterministic
//! traps:
//!
//! 1. **Collision.** The graph keyed every function by its BARE name. For impl
//!    methods this collided: `impl A { @caps(fs) def go() }` and
//!    `impl B { @caps(net) def go() }` were both stored under `"go"`, so the
//!    second silently OVERWROTE the first — authority was mis-attributed (a
//!    caller of `A::go` could be charged B's `net`, or vice-versa, depending
//!    on declaration order). The graph now keys impl methods as `Owner::name`
//!    (mirroring the S114 capability surface), so `A::go` and `B::go` are
//!    DISTINCT entries.
//!
//! 2. **Method-call propagation.** `Expr::Method` was fully deferred — a
//!    `receiver.go()` call contributed ZERO caps to the transitive set. It now
//!    resolves to the UNION of declared caps of every impl method named `go`,
//!    across all types. This is a sound OVER-approximation (it never
//!    under-attributes authority); it may over-attribute when a method name
//!    collides across types. Precise type-directed dispatch is a future slice.
//!
//! These tests drive the public `caps_graph::check_caps_coverage` entry point
//! and assert on the per-function `transitive` map it returns.

use garnet_check::caps_graph::check_caps_coverage;
use garnet_parser::parse_source;

fn transitive(src: &str, key: &str) -> garnet_check::CapSet {
    let module = parse_source(src).expect("parse ok");
    let report = check_caps_coverage(&module);
    report.transitive.get(key).copied().unwrap_or_else(|| {
        panic!(
            "no transitive entry for `{key}`; have {:?}",
            report.transitive
        )
    })
}

/// `A::go` and `B::go` must be DISTINCT graph entries with their OWN caps.
///
/// Before this slice: both keyed under bare `"go"`, the second overwriting the
/// first — so exactly one of `{fs}` / `{net}` survived and the other was lost.
/// After: `A::go -> {fs}` and `B::go -> {net}`, both present.
#[test]
fn impl_method_collision_resolves_to_distinct_entries() {
    let src = r#"
        struct A {}
        struct B {}
        impl A {
            @caps(fs)
            def go(self) -> int { read_file("a") }
        }
        impl B {
            @caps(net)
            def go(self) -> int { tcp_connect("b", 80) }
        }
    "#;
    let module = parse_source(src).expect("parse ok");
    let report = check_caps_coverage(&module);

    let a_go = report
        .transitive
        .get("A::go")
        .copied()
        .unwrap_or_else(|| panic!("A::go missing; have {:?}", report.transitive));
    let b_go = report
        .transitive
        .get("B::go")
        .copied()
        .unwrap_or_else(|| panic!("B::go missing; have {:?}", report.transitive));

    // A::go reads a file → needs exactly `fs`, NOT `net`.
    assert!(a_go.contains("fs"), "A::go must carry fs, got {a_go:?}");
    assert!(
        !a_go.contains("net"),
        "A::go must NOT carry B's net (the old bare-key collision bug), got {a_go:?}"
    );

    // B::go hits the network → needs exactly `net`, NOT `fs`.
    assert!(b_go.contains("net"), "B::go must carry net, got {b_go:?}");
    assert!(
        !b_go.contains("fs"),
        "B::go must NOT carry A's fs (the old bare-key collision bug), got {b_go:?}"
    );

    // The bare key must no longer exist — keys are type-qualified now.
    assert!(
        !report.transitive.contains_key("go"),
        "impl methods must be keyed `Owner::go`, never bare `go`; have {:?}",
        report.transitive
    );
}

/// A caller of `.go()` must see the UNION `{fs, net}` of all impl methods named
/// `go`. Before this slice the method edge was deferred and contributed
/// nothing, so the caller's transitive set was EMPTY.
#[test]
fn method_call_propagates_union_of_all_same_named_methods() {
    let src = r#"
        struct A {}
        struct B {}
        impl A {
            @caps(fs)
            def go(self) -> int { read_file("a") }
        }
        impl B {
            @caps(net)
            def go(self) -> int { tcp_connect("b", 80) }
        }
        @caps(fs, net)
        def main(a) -> int {
            a.go()
        }
    "#;
    let caps = transitive(src, "main");
    // Sound over-approximation: with no receiver-type info, `.go()` resolves to
    // the union of every impl method named `go`, i.e. {fs, net}.
    assert!(
        caps.contains("fs") && caps.contains("net"),
        "main calling .go() must carry the union {{fs, net}}, got {caps:?}"
    );
}

/// Mis-attribution case. A program where ONLY `A::go` requires `fs` and a
/// caller invokes `.go()`. Under precise dispatch the caller would carry `fs`.
///
/// BEFORE (bug): with B declared after A under the same bare key `"go"`,
/// `A::go`'s `{fs}` was OVERWRITTEN by `B::go`'s `{net}`, so a caller resolving
/// `go` saw `net` instead of `fs` — authority mis-attributed.
///
/// AFTER (fix): the two methods are distinct entries; `.go()` resolves to the
/// sound union `{fs, net}`, which CONTAINS the truthful `fs` (no longer lost to
/// the overwrite) and additionally the over-approximated `net`. The key
/// assertion is that `fs` is no longer mis-dropped.
#[test]
fn previously_misattributed_authority_now_resolves() {
    let src = r#"
        struct A {}
        struct B {}
        impl A {
            @caps(fs)
            def go(self) -> int { read_file("a") }
        }
        impl B {
            @caps(net)
            def go(self) -> int { tcp_connect("b", 80) }
        }
        @caps(fs, net)
        def caller(x) -> int {
            x.go()
        }
    "#;
    let caps = transitive(src, "caller");
    assert!(
        caps.contains("fs"),
        "fs must survive — it was previously overwritten by B::go's net under \
         the shared bare key, got {caps:?}"
    );
    // The union over-approximation also includes net; documented, sound.
    assert!(caps.contains("net"), "union includes net, got {caps:?}");
}

/// A free (non-impl) function with an explicit empty `@caps()` is recognized as
/// ANNOTATED (declares_caps), so it is subject to the coverage check rather
/// than skipped as "unknown". Here the empty-declared helper calls a file
/// primitive, so it must be FLAGGED for the missing `fs`.
#[test]
fn explicit_empty_caps_is_treated_as_declared_not_unknown() {
    let src = r#"
        @caps()
        def helper() -> int {
            read_file("x")
        }
    "#;
    let module = parse_source(src).expect("parse ok");
    let report = check_caps_coverage(&module);
    assert!(
        report
            .violations
            .iter()
            .any(|v| v.fn_name == "helper" && v.missing == "fs"),
        "an explicit @caps() helper that reads a file must be flagged for fs, \
         got {:?}",
        report.violations
    );
}
