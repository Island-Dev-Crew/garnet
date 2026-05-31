//! `garnet trust-report <file>` — structural trust report for a Garnet source.
//!
//! ## v0.5.1 scope (honest, S7)
//!
//! Counts the actor declarations + capability surface in a parsed Garnet
//! source and prints a one-screen report including the literal line
//! `actors: N / threads: N`. The `threads` count equals the `actors`
//! count because `garnet-actor-runtime/src/runtime.rs` spawns one OS
//! thread per actor (see its header: "Spawn-and-mailbox runtime: each
//! actor gets one OS thread plus a mpsc"). The bridge from managed-mode
//! `actor` declarations to that runtime is the existing path; this
//! command surfaces the count so external CI / dogfood blocks can
//! grep for it.
//!
//! What this command DOES today:
//! - Parses + checks the source, then walks the AST.
//! - Counts `Item::Actor`, `Item::Fn`, and per-function `@caps(...)`
//!   declarations.
//! - Prints the literal line `actors: <N> / threads: <N>` per the S7
//!   contract dogfood block.
//! - Prints the per-actor and per-function caps surface so reviewers see
//!   what OS authority the program asks for.
//!
//! What this command does NOT do (honest partial):
//! - Spawn the runtime or measure actual thread counts. The report is
//!   structural, derived from the source AST.
//! - Verify mailbox sizes or message-type Sendable boundaries beyond
//!   what `garnet check` already enforces.
//! - Aggregate transitive caps from `use` imports (which today resolve to
//!   stdlib, not to vendored deps).

use crate::read_file;
use std::path::PathBuf;
use std::process::ExitCode;

use garnet_parser::ast::{Item, Module};

#[derive(Debug, Default)]
struct TrustCounts {
    actors: Vec<String>,
    fn_count: usize,
    fns_with_caps: usize,
    fns_without_caps: usize,
    caps_seen: std::collections::BTreeSet<String>,
}

fn collect(module: &Module) -> TrustCounts {
    // S35: the canonical, deduped, sorted capability surface — replaces the
    // prior per-call-site `format!("{c:?}").to_lowercase()`, which mislabeled
    // `net_internal` / `Other(_)` / wildcard caps. The checker and trust-report
    // now share one normalization.
    let surface = garnet_check::capability_surface(module);
    let mut out = TrustCounts {
        caps_seen: surface.aggregate.iter().cloned().collect(),
        fns_with_caps: surface.per_function.len(),
        ..TrustCounts::default()
    };
    for item in &module.items {
        match item {
            Item::Actor(a) => out.actors.push(a.name.clone()),
            Item::Fn(_) => out.fn_count += 1,
            _ => {}
        }
    }
    out.fns_without_caps = out.fn_count - out.fns_with_caps;
    out
}

pub fn run(path: PathBuf) -> ExitCode {
    let src = match read_file(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("garnet trust-report: {e}");
            return ExitCode::from(1);
        }
    };
    let module = match garnet_parser::parse_source(&src) {
        Ok(m) => m,
        Err(e) => {
            let report = miette::Report::new(e).with_source_code(src.clone());
            eprintln!("{report:?}");
            return ExitCode::from(1);
        }
    };
    let counts = collect(&module);

    // The header carries the literal `actors: N / threads: N` line because
    // every actor in `garnet-actor-runtime/src/runtime.rs` gets one OS
    // thread plus an mpsc mailbox by construction. This is structural,
    // not a live runtime measurement.
    let n_actors = counts.actors.len();
    println!("garnet trust-report for {}", path.display());
    println!("actors: {n_actors} / threads: {n_actors}");
    println!("  (one OS thread + mpsc mailbox per actor per actor-runtime/src/runtime.rs)");
    println!();
    println!("Actors ({n_actors}):");
    if counts.actors.is_empty() {
        println!("  (none)");
    } else {
        for name in &counts.actors {
            println!("  - {name}");
        }
    }
    println!();
    println!("Top-level functions: {}", counts.fn_count);
    println!("  with @caps(...): {}", counts.fns_with_caps);
    println!("  without @caps(...): {}", counts.fns_without_caps);
    if !counts.caps_seen.is_empty() {
        let joined: Vec<String> = counts.caps_seen.iter().cloned().collect();
        println!("Capabilities surfaced: {}", joined.join(", "));
    } else {
        println!(
            "Capabilities surfaced: (none — purely computational, or no @caps annotations present)"
        );
    }
    println!();
    println!("Honest scope: this report is structural (AST-derived); it does not");
    println!("spawn the runtime, measure live thread counts, or audit mailbox sizes.");

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn module(src: &str) -> Module {
        garnet_parser::parse_source(src).expect("parse")
    }

    #[test]
    fn counts_three_actors_and_one_fn() {
        let src = r#"
@caps()
def main() { 3 }
actor A { on a() { 1 } }
actor B { on b() { 1 } }
actor C { on c() { 1 } }
"#;
        let counts = collect(&module(src));
        assert_eq!(3, counts.actors.len());
        assert_eq!(vec!["A", "B", "C"], counts.actors);
        assert_eq!(1, counts.fn_count);
        assert_eq!(1, counts.fns_with_caps);
        assert_eq!(0, counts.fns_without_caps);
    }

    #[test]
    fn counts_caps_surface_dedup_sorted() {
        let src = r#"
@caps(fs)
def reader() { 1 }
@caps(fs, net)
def writer() { 1 }
@caps()
def main() { 0 }
"#;
        let counts = collect(&module(src));
        let caps: Vec<String> = counts.caps_seen.into_iter().collect();
        // The Capability enum's Debug forms include the variant tokens we
        // lowercased — Fs -> "fs", Net -> "net".
        assert!(caps.contains(&"fs".to_string()), "got {caps:?}");
        assert!(caps.contains(&"net".to_string()), "got {caps:?}");
        assert_eq!(3, counts.fn_count);
        assert_eq!(3, counts.fns_with_caps);
    }

    #[test]
    fn no_actors_means_zero_threads() {
        let src = r#"@caps() def main() { 0 }"#;
        let counts = collect(&module(src));
        assert_eq!(0, counts.actors.len());
        assert_eq!(0, counts.fns_without_caps);
    }
}
