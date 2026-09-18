//! Red-first tests for U-117: a capability-bearing primitive reached only
//! through a call-graph cycle must be reported, whatever the functions are
//! called.
//!
//! Before the cure the propagator returned an empty set for a function that
//! was still being computed and memoised that partial answer, so the verdict
//! depended on alphabetical visit order. Every test here failed against the
//! pre-cure `transitive_caps` (the colored DFS) and passes against the SCC
//! (DeRemer–Pennello digraph) traversal that replaced it.
//!
//! This file is included from `caps_graph.rs` with `#[path]` so it stays under
//! the digest-bound `garnet-check-v0.3/src/` trust prefix.

use super::check_caps_coverage;
use garnet_parser::ast::Module;
use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

fn parse(src: &str) -> Module {
    garnet_parser::parse_source(src).expect("parse failed")
}

/// `(fn, missing, via)` triples in report order.
fn violations(src: &str) -> Vec<(String, String, String)> {
    check_caps_coverage(&parse(src))
        .violations
        .into_iter()
        .map(|v| (v.fn_name, v.missing, v.via))
        .collect()
}

fn triples(items: &[(&str, &str, &str)]) -> Vec<(String, String, String)> {
    items
        .iter()
        .map(|(f, m, v)| (f.to_string(), m.to_string(), v.to_string()))
        .collect()
}

/// The register program: `holder` declares `fs` and calls `partner` and
/// `write_file`; `partner` declares `@caps()` and calls `holder`; `main`
/// declares `@caps()` and calls `partner`.
fn register_shape(holder: &str, partner: &str) -> String {
    format!(
        r#"
        @caps(fs)
        def {holder}() {{
            {partner}()
            write_file("/tmp/never.txt", "x")
        }}
        @caps()
        def {partner}() {{
            {holder}()
        }}
        @caps()
        def main() {{
            {partner}()
        }}
        "#
    )
}

#[test]
fn u117_register_cycle_reports_b_and_main() {
    // Byte-for-byte what the acyclic control (the `a -> b` edge removed)
    // reports today.
    assert_eq!(
        violations(&register_shape("a", "b")),
        triples(&[
            ("b", "fs", "fs::write_file (via a)"),
            ("main", "fs", "fs::write_file (via b → a)"),
        ])
    );
}

#[test]
fn u117_cycle_verdict_is_independent_of_function_names() {
    // `m` < `main` < `n` covers every position of the pair relative to main.
    let pairs = [
        ("a", "b"),
        ("b", "a"),
        ("zed", "yak"),
        ("yak", "zed"),
        ("a", "zed"),
        ("zed", "a"),
        ("m", "n"),
        ("n", "m"),
    ];
    for (holder, partner) in pairs {
        let report = check_caps_coverage(&parse(&register_shape(holder, partner)));
        let got: BTreeSet<(String, String)> = report
            .violations
            .iter()
            .map(|v| (v.fn_name.clone(), v.missing.clone()))
            .collect();
        let want: BTreeSet<(String, String)> = [
            (partner.to_string(), "fs".to_string()),
            ("main".to_string(), "fs".to_string()),
        ]
        .into_iter()
        .collect();
        assert_eq!(got, want, "pair ({holder}, {partner}) gave {got:?}");
        for f in [holder, partner, "main"] {
            assert!(
                report.transitive[f].contains("fs"),
                "pair ({holder}, {partner}): transitive[{f}] lacks fs"
            );
        }
    }
}

#[test]
fn u117_three_member_cycle_reports_every_annotated_member() {
    let src = r#"
        @caps(fs)
        def a() {
            b()
            write_file("/tmp/never.txt", "x")
        }
        @caps()
        def b() {
            c()
        }
        @caps()
        def c() {
            a()
        }
        @caps()
        def main() {
            c()
        }
    "#;
    assert_eq!(
        violations(src),
        triples(&[
            ("b", "fs", "fs::write_file (via c → a)"),
            ("c", "fs", "fs::write_file (via a)"),
            ("main", "fs", "fs::write_file (via c → a)"),
        ])
    );
}

#[test]
fn u117_unannotated_cycle_still_reports_annotated_caller() {
    // Unannotated bodies are not themselves checked (U-91 boundary), but the
    // authority they reach must still flow to the annotated caller.
    let src = r#"
        def a() {
            b()
            write_file("/tmp/never.txt", "x")
        }
        def b() {
            a()
        }
        @caps()
        def main() {
            b()
        }
    "#;
    assert_eq!(
        violations(src),
        triples(&[("main", "fs", "fs::write_file (via b → a)")])
    );
}

#[test]
fn u117_method_call_cycle_reports() {
    // The cycle closes through a name-resolved method edge:
    // helper -> .go() -> A::go -> helper.
    let src = r#"
        impl A {
            @caps(fs)
            def go(self) {
                helper(1)
                write_file("/tmp/never.txt", "x")
            }
        }
        @caps()
        def helper(x) {
            x.go()
        }
        @caps()
        def main() {
            helper(1)
        }
    "#;
    assert_eq!(
        violations(src),
        triples(&[
            ("helper", "fs", "fs::write_file (via .go() → A::go)"),
            ("main", "fs", "fs::write_file (via helper → .go() → A::go)"),
        ])
    );
}

#[test]
fn u117_safe_cycle_member_gets_linear_effect_verdict() {
    // effects.rs reads `transitive[b]`; before the cure it was the memoised
    // empty set, so the safe helper `b` drew no linear/effect verdict.
    let module = parse(
        r#"
        @caps(fs)
        fn a(borrow p: String) -> String {
            b()
            fs::read_file(p)
        }
        @caps()
        fn b() -> String {
            a("x")
        }
        @caps()
        fn main() -> String {
            b()
        }
        "#,
    );
    let caps = check_caps_coverage(&module);
    let effects = crate::linear_effect_report(&module, &caps);
    assert!(
        effects
            .violations
            .iter()
            .any(|v| v.fn_name == "b" && v.required_caps.contains("fs")),
        "expected a linear/effect violation on `b` carrying fs, got {:?}",
        effects.violations
    );
    let report = crate::check_module(&module);
    assert!(
        report.errors.iter().any(|e| matches!(
            e,
            crate::CheckError::LinearEffect(msg) if msg.contains("`b`")
        )),
        "expected check.linear_effect on b, got {:?}",
        report.errors
    );
    for f in ["b", "main"] {
        assert!(
            report.errors.iter().any(|e| matches!(
                e,
                crate::CheckError::CapsCoverage { fn_name, missing, .. }
                    if fn_name == f && missing == "fs"
            )),
            "expected check.caps_coverage on {f}, got {:?}",
            report.errors
        );
    }
}

// ── Random call graphs against an independent reachability oracle ─────

struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn chance(&mut self, one_in: u64) -> bool {
        self.next().is_multiple_of(one_in)
    }
}

/// A generated program: for each function, its callees and which
/// capability-bearing primitives it calls directly.
struct RandomProgram {
    names: Vec<&'static str>,
    edges: BTreeMap<&'static str, Vec<&'static str>>,
    direct_fs: BTreeSet<&'static str>,
    direct_time: BTreeSet<&'static str>,
}

impl RandomProgram {
    fn generate(rng: &mut XorShift) -> Self {
        // A pool that straddles "main" in byte order, so visit order relative
        // to the entry point varies across programs.
        const POOL: [&str; 10] = ["ab", "b_1", "kq", "ma", "main", "mz", "n", "q9", "x", "zz"];
        let count = 2 + rng.below(7);
        let mut names: Vec<&'static str> = POOL.to_vec();
        // Fisher–Yates, then take the first `count`.
        for i in (1..names.len()).rev() {
            let j = rng.below(i + 1);
            names.swap(i, j);
        }
        names.truncate(count);
        names.sort_unstable();

        let mut edges = BTreeMap::new();
        let mut direct_fs = BTreeSet::new();
        let mut direct_time = BTreeSet::new();
        for &f in &names {
            let mut callees = Vec::new();
            for &g in &names {
                // Self-edges and cycles included.
                if rng.chance(3) {
                    callees.push(g);
                }
            }
            edges.insert(f, callees);
            if rng.chance(4) {
                direct_fs.insert(f);
            }
            if rng.chance(5) {
                direct_time.insert(f);
            }
        }
        RandomProgram {
            names,
            edges,
            direct_fs,
            direct_time,
        }
    }

    fn source(&self) -> String {
        let mut src = String::new();
        for &f in &self.names {
            src.push_str(&format!("def {f}() {{\n"));
            for g in &self.edges[f] {
                src.push_str(&format!("    {g}()\n"));
            }
            if self.direct_fs.contains(f) {
                src.push_str("    write_file(\"/tmp/never.txt\", \"x\")\n");
            }
            if self.direct_time.contains(f) {
                src.push_str("    now_ms()\n");
            }
            src.push_str("    1\n}\n");
        }
        src
    }

    /// Brute-force reachability over the generated edge list, independent of
    /// the propagator: does `from` reach (including itself) any function in
    /// `holders`?
    fn reaches(&self, from: &str, holders: &BTreeSet<&'static str>) -> bool {
        let mut seen = BTreeSet::new();
        let mut stack = vec![from];
        while let Some(f) = stack.pop() {
            if !seen.insert(f) {
                continue;
            }
            if holders.contains(f) {
                return true;
            }
            for &g in &self.edges[f] {
                stack.push(g);
            }
        }
        false
    }
}

#[test]
fn u117_transitive_equals_reachability_on_random_call_graphs() {
    let mut rng = XorShift(0x0117_0117_0117_0117);
    let mut cyclic_programs = 0usize;
    for i in 0..3_000 {
        let program = RandomProgram::generate(&mut rng);
        let src = program.source();
        let report = check_caps_coverage(&parse(&src));
        let mut has_cycle = false;
        for &f in &program.names {
            let caps = report.transitive[f];
            let want_fs = program.reaches(f, &program.direct_fs);
            let want_time = program.reaches(f, &program.direct_time);
            assert_eq!(
                caps.contains("fs"),
                want_fs,
                "program {i}, fn {f}: fs mismatch\n{src}"
            );
            assert_eq!(
                caps.contains("time"),
                want_time,
                "program {i}, fn {f}: time mismatch\n{src}"
            );
            assert!(
                caps.difference(super::CapSet::FS | super::CapSet::TIME)
                    .is_empty(),
                "program {i}, fn {f}: unexpected caps {:?}\n{src}",
                caps.names()
            );
            // A cycle exists through f when some callee reaches back to f.
            let f_set: BTreeSet<&'static str> = [f].into_iter().collect();
            if program.edges[f].iter().any(|&g| program.reaches(g, &f_set)) {
                has_cycle = true;
            }
        }
        if has_cycle {
            cyclic_programs += 1;
        }
    }
    assert!(
        cyclic_programs > 1_000,
        "the generator must exercise cycles heavily; only {cyclic_programs} of 3000 had one"
    );
}

// ── Shape and cost guards that ride the same cure ─────────────────────

#[test]
fn u117_deep_named_chain_does_not_overflow_the_stack() {
    // The recursive walk overflowed at roughly 8,400 chained functions. The
    // iterative traversal has no recursion to overflow. Run it on a thread
    // with the default test stack so the guard is not weakened by a large
    // main-thread stack.
    const DEPTH: usize = 12_000;
    let mut src = String::new();
    for i in 0..DEPTH {
        src.push_str(&format!("def f{i}() {{\n    f{}()\n}}\n", i + 1));
    }
    src.push_str(&format!(
        "def f{DEPTH}() {{\n    write_file(\"/tmp/never.txt\", \"x\")\n}}\n@caps()\ndef main() {{\n    f0()\n}}\n"
    ));
    let module = parse(&src);
    let report = std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(move || check_caps_coverage(&module))
        .expect("spawn")
        .join()
        .expect("the propagator must not abort on a deep named chain");
    assert!(report.transitive["f0"].contains("fs"));
    assert_eq!(
        report
            .violations
            .iter()
            .map(|v| (v.fn_name.as_str(), v.missing.as_str(), v.via.as_str()))
            .collect::<Vec<_>>(),
        vec![(
            "main",
            "fs",
            "fs::write_file (via f0 → f1 → f2 → … → f11998 → f11999 → f12000)"
        )]
    );
}

#[test]
fn u117_dense_cycle_finishes_in_bounded_time_and_reports_every_member() {
    // A complete digraph on N functions: N² edges, every member in one SCC.
    // The traversal must stay linear in edges (no re-traversal of the SCC
    // per root) and every annotated member must learn the one primitive.
    const N: usize = 150;
    let mut src = String::new();
    for i in 0..N {
        src.push_str(&format!("@caps()\ndef g{i}() {{\n"));
        for j in 0..N {
            src.push_str(&format!("    g{j}()\n"));
        }
        if i == N - 1 {
            src.push_str("    now_ms()\n");
        }
        src.push_str("    1\n}\n");
    }
    let module = parse(&src);
    let started = Instant::now();
    let report = check_caps_coverage(&module);
    let elapsed = started.elapsed();
    assert!(
        elapsed < Duration::from_secs(20),
        "dense cycle took {elapsed:?}; the traversal must stay near-linear"
    );
    let flagged: BTreeSet<&str> = report
        .violations
        .iter()
        .filter(|v| v.missing == "time")
        .map(|v| v.fn_name.as_str())
        .collect();
    assert_eq!(
        flagged.len(),
        N,
        "every member must be reported, got {flagged:?}"
    );
    for i in 0..N {
        assert!(report.transitive[&format!("g{i}")].contains("time"));
    }
}

// ── The `via` field names the primitive, then the path to it ─────────────

/// Follow-up to U-117 (2026-09-18): the cycle diagnostic read
/// "transitively calls `(via a)` which requires it" — it named the next hop
/// and never the primitive the function actually needs. `via` must lead with
/// the qualified primitive and follow with the named path that reaches it.
#[test]
fn via_names_the_primitive_behind_a_cycle() {
    assert_eq!(
        violations(&register_shape("a", "b")),
        triples(&[
            ("b", "fs", "fs::write_file (via a)"),
            ("main", "fs", "fs::write_file (via b → a)"),
        ])
    );
}

#[test]
fn via_names_the_primitive_behind_an_acyclic_helper() {
    let src = r#"
        @caps(fs)
        def helper() {
            read_file("a.txt")
        }
        @caps()
        def main() {
            helper()
        }
    "#;
    assert_eq!(
        violations(src),
        triples(&[("main", "fs", "fs::read_file (via helper)")])
    );
}

#[test]
fn via_qualifies_a_bare_direct_primitive() {
    let src = r#"
        @caps()
        def main() {
            read_file("a.txt")
        }
    "#;
    assert_eq!(violations(src), triples(&[("main", "fs", "fs::read_file")]));
}
