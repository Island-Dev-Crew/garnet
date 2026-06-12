//! RB-1 differential suite — `CapSet` propagator vs the old set-based one.
//!
//! Proptest generates random Garnet modules (random `@caps(...)` subsets —
//! including unknown/user-defined names and `@caps(*)` — over random call
//! graphs of user fns and stdlib primitives, with cycles), parses them, and
//! runs BOTH `caps_graph` (the new `CapSet` bitset propagator) and
//! `caps_graph_set` (the old `BTreeSet<String>` implementation, kept
//! verbatim for this slice). Propagation results, coverage violations
//! (content AND order), and per-fn transitive sets must be identical.
//!
//! This module and `caps_graph_set.rs` are deleted together in the same PR
//! once green, per `W_REBUILD_SPEC.md` §3 RB-1. The permanent property
//! coverage that outlives them: the `CapSet`-vs-`BTreeSet` model suite in
//! `capset.rs` and the diff-caps reference suite in `caps_diff.rs`.

use crate::{caps_graph, caps_graph_set};
use proptest::prelude::*;

/// Primitive call snippets with their registry-required capability —
/// bare-name, qualified, and pure calls all represented.
const PRIM_CALLS: &[&str] = &[
    "read_file(\"x\")",           // fs (bare)
    "write_file(\"x\", \"y\")",   // fs (bare)
    "fs::read_file(\"q\")",       // fs (qualified)
    "now_ms()",                   // time
    "tcp_connect(\"h\")",         // net
    "std::env::get(\"k\")",       // env (qualified)
    "std::process::spawn(\"c\")", // proc (qualified)
    "trim(\"s\")",                // pure — no caps
];

/// Capability names a generated `@caps(...)` may declare: every canonical
/// name plus unknown/user-defined ones (which the old impl carried verbatim
/// and the new impl folds into the `OTHER` presence bit).
const CAP_NAMES: &[&str] = &[
    "fs",
    "net",
    "net_internal",
    "time",
    "proc",
    "ffi",
    "env",
    "custom_cap",
    "zz_unknown",
];

#[derive(Debug, Clone)]
enum AnnSpec {
    /// No `@caps` annotation at all.
    None,
    /// `@caps(*)`.
    Wildcard,
    /// `@caps(<names>)` — possibly empty, indices into [`CAP_NAMES`].
    Caps(Vec<usize>),
}

#[derive(Debug, Clone)]
enum CallSpec {
    /// A stdlib primitive call, index into [`PRIM_CALLS`].
    Prim(usize),
    /// A call to user fn `f{i % n_fns}` (self-calls and cycles included).
    User(usize),
}

#[derive(Debug, Clone)]
struct FnSpec {
    ann: AnnSpec,
    calls: Vec<CallSpec>,
}

fn ann_strategy() -> impl Strategy<Value = AnnSpec> {
    prop_oneof![
        1 => Just(AnnSpec::None),
        1 => Just(AnnSpec::Wildcard),
        4 => proptest::collection::vec(0..CAP_NAMES.len(), 0..4).prop_map(AnnSpec::Caps),
    ]
}

fn call_strategy() -> impl Strategy<Value = CallSpec> {
    prop_oneof![
        2 => (0..PRIM_CALLS.len()).prop_map(CallSpec::Prim),
        2 => (0..8usize).prop_map(CallSpec::User),
    ]
}

fn module_strategy() -> impl Strategy<Value = (Vec<FnSpec>, bool)> {
    (
        proptest::collection::vec(
            (
                ann_strategy(),
                proptest::collection::vec(call_strategy(), 0..5),
            )
                .prop_map(|(ann, calls)| FnSpec { ann, calls }),
            1..6,
        ),
        proptest::bool::ANY,
    )
}

fn fn_name(i: usize, first_is_main: bool) -> String {
    if first_is_main && i == 0 {
        "main".to_string()
    } else {
        format!("f{i}")
    }
}

/// Render the specs as Garnet source. Deterministic; every generated
/// module parses (asserted in the tests).
fn render(specs: &[FnSpec], first_is_main: bool) -> String {
    let n = specs.len();
    let mut src = String::new();
    for (i, spec) in specs.iter().enumerate() {
        match &spec.ann {
            AnnSpec::None => {}
            AnnSpec::Wildcard => src.push_str("@caps(*)\n"),
            AnnSpec::Caps(idxs) => {
                let mut names: Vec<&str> = idxs.iter().map(|&i| CAP_NAMES[i]).collect();
                names.sort_unstable();
                names.dedup();
                src.push_str(&format!("@caps({})\n", names.join(", ")));
            }
        }
        src.push_str(&format!("def {}() {{\n", fn_name(i, first_is_main)));
        for call in &spec.calls {
            match call {
                CallSpec::Prim(p) => src.push_str(&format!("    {}\n", PRIM_CALLS[*p])),
                CallSpec::User(u) => {
                    src.push_str(&format!("    {}()\n", fn_name(*u % n, first_is_main)))
                }
            }
        }
        src.push_str("    1\n}\n");
    }
    src
}

/// Flatten a new-impl report into comparable (name, missing/caps, via) rows.
fn new_rows(
    r: &caps_graph::CapsReport,
) -> (Vec<(String, String, String)>, Vec<(String, Vec<String>)>) {
    let violations = r
        .violations
        .iter()
        .map(|v| (v.fn_name.clone(), v.missing.clone(), v.via.clone()))
        .collect();
    let transitive = r
        .transitive
        .iter()
        .map(|(k, v)| (k.clone(), v.names().iter().map(|s| s.to_string()).collect()))
        .collect();
    (violations, transitive)
}

/// Flatten an old-impl report into the same comparable rows.
fn old_rows(
    r: &caps_graph_set::CapsReport,
) -> (Vec<(String, String, String)>, Vec<(String, Vec<String>)>) {
    let violations = r
        .violations
        .iter()
        .map(|v| (v.fn_name.clone(), v.missing.clone(), v.via.clone()))
        .collect();
    let transitive = r
        .transitive
        .iter()
        .map(|(k, v)| (k.clone(), v.iter().cloned().collect()))
        .collect();
    (violations, transitive)
}

fn assert_identical(src: &str) {
    let module = garnet_parser::parse_source(src)
        .unwrap_or_else(|e| panic!("generated source must parse: {e:?}\n---\n{src}"));
    let new_report = caps_graph::check_caps_coverage(&module);
    let old_report = caps_graph_set::check_caps_coverage(&module);
    let (new_v, new_t) = new_rows(&new_report);
    let (old_v, old_t) = old_rows(&old_report);
    assert_eq!(
        new_v, old_v,
        "coverage violations diverged (content or order)\n---\n{src}"
    );
    assert_eq!(new_t, old_t, "transitive caps diverged\n---\n{src}");
}

proptest! {
    /// The headline differential: random cap-sets over random call graphs →
    /// identical violations (content + order) and transitive sets.
    #[test]
    fn propagation_and_violations_identical((specs, first_is_main) in module_strategy()) {
        let src = render(&specs, first_is_main);
        let module = garnet_parser::parse_source(&src)
            .expect("generated source parses");
        let new_report = caps_graph::check_caps_coverage(&module);
        let old_report = caps_graph_set::check_caps_coverage(&module);
        let (new_v, new_t) = new_rows(&new_report);
        let (old_v, old_t) = old_rows(&old_report);
        prop_assert_eq!(new_v, old_v, "violations diverged\n---\n{}", src);
        prop_assert_eq!(new_t, old_t, "transitive diverged\n---\n{}", src);
    }
}

/// Fixed corpus pinning the edge cases the random generator may visit only
/// occasionally. Each is a behavior the bitset must reproduce exactly.
#[test]
fn corpus_unknown_only_annotation_still_gates_coverage() {
    // The fn's ONLY declared cap is unknown — the old impl treated the
    // non-empty set as "annotated" and still ran the coverage check. The
    // OTHER presence bit must preserve that.
    assert_identical(
        r#"
        @caps(custom_cap)
        def main() {
            read_file("a")
            1
        }
        "#,
    );
}

#[test]
fn corpus_wildcard_skips_coverage() {
    assert_identical(
        r#"
        @caps(*)
        def main() {
            read_file("a")
            1
        }
        "#,
    );
}

#[test]
fn corpus_mutual_recursion_converges() {
    assert_identical(
        r#"
        def ping(n) {
            read_file("ping")
            pong(n)
        }
        def pong(n) {
            tcp_connect("h")
            ping(n)
        }
        @caps(fs)
        def main() {
            ping(0)
        }
        "#,
    );
}

#[test]
fn corpus_qualified_and_bare_prims_resolve_alike() {
    assert_identical(
        r#"
        @caps()
        def main() {
            fs::read_file("q")
            std::env::get("k")
            std::process::spawn("c")
            now_ms()
            trim("s")
            1
        }
        "#,
    );
}

#[test]
fn corpus_env_is_known_via_checker_whitelist() {
    assert_identical(
        r#"
        @caps(env)
        def main() {
            std::env::get("k")
            1
        }
        "#,
    );
}

#[test]
fn corpus_unannotated_helper_transits_caps_to_main() {
    assert_identical(
        r#"
        def helper(p) {
            read_file(p)
        }
        @caps()
        def main() {
            helper("x")
        }
        "#,
    );
}

/// Machine-local timing harness for the RB-1 perf note. NOT a benchmark
/// claim: run explicitly (`cargo test -p garnet-check --lib
/// timing_old_vs_new -- --ignored --nocapture`) and record the output as
/// machine-local evidence. Deleted with the old impl.
#[test]
#[ignore = "manual timing harness for the RB-1 perf note"]
fn timing_old_vs_new_propagation() {
    // 120-fn call chain, each annotated and calling two prims + the next fn.
    let mut src = String::new();
    for i in 0..120 {
        src.push_str("@caps(fs, time)\n");
        src.push_str(&format!("def f{i}() {{\n"));
        src.push_str("    read_file(\"x\")\n    now_ms()\n");
        if i + 1 < 120 {
            src.push_str(&format!("    f{}()\n", i + 1));
        }
        src.push_str("    1\n}\n");
    }
    let module = garnet_parser::parse_source(&src).expect("chain parses");

    const RUNS: u32 = 200;
    let t_old = std::time::Instant::now();
    for _ in 0..RUNS {
        std::hint::black_box(caps_graph_set::check_caps_coverage(&module));
    }
    let old_elapsed = t_old.elapsed();
    let t_new = std::time::Instant::now();
    for _ in 0..RUNS {
        std::hint::black_box(caps_graph::check_caps_coverage(&module));
    }
    let new_elapsed = t_new.elapsed();
    println!(
        "RB-1 timing (machine-local, 120-fn chain x {RUNS} runs): \
         old set-based = {old_elapsed:?}, new CapSet = {new_elapsed:?}"
    );
}
