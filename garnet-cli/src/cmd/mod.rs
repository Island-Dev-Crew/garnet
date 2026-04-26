//! Per-subcommand command implementations.
//!
//! Each submodule exports `pub fn run(...) -> ExitCode` that the
//! `garnet` binary dispatches to from `src/bin/garnet.rs`. Splitting
//! by command keeps any one file readable and makes the dispatch
//! table in `bin/garnet.rs` a thin pattern match.
//!
//! Helpers below are `pub(crate)` and shared across multiple
//! commands (episode logging, knowledge / strategy persistence,
//! AST item summaries).

pub mod build;
pub mod check;
pub mod convert;
pub mod eval;
pub mod keygen;
pub mod new;
pub mod parse;
pub mod repl;
pub mod run;
pub mod test;
pub mod verify;

use crate::cache::{self, Episode};
use crate::{knowledge, strategies};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Helper used by every subcommand at start: surface a one-line note for any
/// prior failures of the same source hash, and consult any strategies
/// matched by AST fingerprint similarity. Silent if nothing relevant.
pub(crate) fn surface_prior(source: &str) {
    let hash = cache::source_hash(source);
    let prior = cache::recall(&hash);
    let failures = prior.iter().filter(|e| e.outcome != "ok").count();
    if failures > 0 {
        eprintln!(
            "note: this source has {failures} prior failure(s) recorded in .garnet-cache/episodes.log"
        );
    }
    // Try to surface relevant strategies — best-effort, never fail.
    if let Ok(module) = garnet_parser::parse_source(source) {
        let target = knowledge::fingerprint(&module);
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        if let Ok(conn) = strategies::open(&cwd) {
            if let Ok(matches) = strategies::consult(&conn, &target, 3) {
                for (dist, s) in matches.iter().filter(|(d, _)| *d <= 32) {
                    eprintln!(
                        "note: strategy '{}' applies (Hamming distance {dist}/256, last triggered ts={})",
                        s.heuristic, s.created_ts
                    );
                }
            }
        }
    }
}

/// Persist a knowledge-graph row + (optionally) synthesize new strategies.
/// Best-effort; failures are silent so they never break the CLI.
pub(crate) fn persist_knowledge(source: &str, source_hash: &str, outcome: &str) {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let Ok(module) = garnet_parser::parse_source(source) else {
        return;
    };
    let fp = knowledge::fingerprint(&module);
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    if let Ok(conn) = knowledge::open(&cwd) {
        let _ = knowledge::record_context(&conn, source_hash, &fp, outcome, ts);
    }
    // Synthesize strategies from the cumulative episode log.
    if let Ok(strat_conn) = strategies::open(&cwd) {
        let cache_dir = cache::cache_dir_for(&cwd);
        let indexed = cache::read_all_indexed_in(&cache_dir);
        let episodes: Vec<_> = indexed.iter().map(|(_, ep)| ep.clone()).collect();
        let id_lookup: std::collections::HashMap<*const Episode, i64> = episodes
            .iter()
            .zip(indexed.iter().map(|(idx, _)| *idx))
            .map(|(ep, idx)| (ep as *const Episode, idx))
            .collect();
        let proposed = strategies::synthesize_from_episodes_with_ids(
            &episodes,
            |hash| {
                // Only recover the fingerprint if we just observed this hash.
                if hash == source_hash {
                    Some(fp)
                } else {
                    None
                }
            },
            |ep| id_lookup.get(&(ep as *const Episode)).copied(),
        );
        for s in proposed {
            let _ = strategies::record_strategy(&strat_conn, &s, ts);
        }
    }
}

/// Helper used by every subcommand at exit: append an episode record AND
/// fold the outcome into the SQLite knowledge + strategies stores.
pub(crate) fn record(
    cmd: &str,
    file: &str,
    source: &str,
    outcome: &str,
    error_kind: Option<String>,
    started: Instant,
    exit_code: i32,
) {
    let duration_ms = started.elapsed().as_millis() as u64;
    let hash = cache::source_hash(source);
    let ep = Episode::now(
        cmd,
        file,
        hash.clone(),
        outcome,
        error_kind,
        duration_ms,
        exit_code,
    );
    let _ = cache::record_episode(&ep);
    persist_knowledge(source, &hash, outcome);
}

/// One-line description of an AST `Item` — used by `garnet parse` for
/// the structural summary and by other diagnostics that need a short
/// human-readable item label.
pub(crate) fn describe_item(item: &garnet_parser::ast::Item) -> String {
    use garnet_parser::ast::Item;
    match item {
        Item::Fn(f) => format!("fn/def {} ({} args)", f.name, f.params.len()),
        Item::Struct(s) => format!("struct {}", s.name),
        Item::Enum(e) => format!("enum {}", e.name),
        Item::Trait(t) => format!("trait {}", t.name),
        Item::Impl(_) => "impl block".to_string(),
        Item::Actor(a) => format!("actor {} ({} items)", a.name, a.items.len()),
        Item::Memory(m) => format!("memory {:?} {}", m.kind, m.name),
        Item::Module(m) => format!("module {}", m.name),
        Item::Use(u) => format!("use {}", u.path.join("::")),
        Item::Const(c) => format!("const {}", c.name),
        Item::Let(l) => format!("let {}", l.name),
    }
}
