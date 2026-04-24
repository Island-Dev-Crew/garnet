//! SQLite knowledge.db + strategies.db tests — Paper VI Contribution 3, layer 2-3.
//!
//! Verifies:
//! - Records persist in `compilation_contexts`.
//! - `similar_contexts` returns top-k by Hamming distance.
//! - `synthesize_from_episodes` proposes the expected heuristics for the
//!   sample patterns (3+ ok = skip_check; 2+ same-error = warn).
//! - Reading the DBs from disk after the CLI ran works.

use garnet_cli::cache::Episode;
use garnet_cli::knowledge;
use garnet_cli::strategies::{self, NewStrategy};
use std::path::PathBuf;
use std::process::Command;

fn fresh_dir(name: &str) -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nano = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("garnet_kb_{name}_{}_{nano}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn garnet_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_garnet"))
}

#[test]
fn records_inserted_into_compilation_contexts() {
    let dir = fresh_dir("inserts");
    let conn = knowledge::open(&dir).unwrap();
    let fp = [42u8; 32];
    knowledge::record_context(&conn, "abc", &fp, "ok", 1).unwrap();
    knowledge::record_context(&conn, "def", &fp, "parse_err", 2).unwrap();
    assert_eq!(knowledge::count_contexts(&conn).unwrap(), 2);
}

#[test]
fn similar_contexts_returns_top_k_by_hamming() {
    let dir = fresh_dir("similar");
    let conn = knowledge::open(&dir).unwrap();

    // Insert three fingerprints: identical, 8 bits off, 64 bits off.
    let mut a = [0u8; 32];
    a[0] = 0x00;
    let mut b = [0u8; 32];
    b[0] = 0xFF; // 8 bits off from a
    let mut c = [0u8; 32];
    c[0] = 0xFF;
    c[1] = 0xFF;
    c[2] = 0xFF;
    c[3] = 0xFF;
    c[4] = 0xFF;
    c[5] = 0xFF;
    c[6] = 0xFF;
    c[7] = 0xFF; // 64 bits off

    knowledge::record_context(&conn, "a", &a, "ok", 1).unwrap();
    knowledge::record_context(&conn, "b", &b, "ok", 2).unwrap();
    knowledge::record_context(&conn, "c", &c, "ok", 3).unwrap();

    let target = a;
    let top3 = knowledge::similar_contexts(&conn, &target, 3).unwrap();
    assert_eq!(top3.len(), 3);
    assert_eq!(top3[0].0, 0); // a is identical
    assert_eq!(top3[1].0, 8); // b is 8 bits off
    assert_eq!(top3[2].0, 64); // c is 64 bits off
}

#[test]
fn synthesize_proposes_skip_check_after_three_ok() {
    let mut episodes = Vec::new();
    for _ in 0..3 {
        episodes.push(Episode::now(
            "parse", "x.garnet", "h_abc", "ok", None, 10, 0,
        ));
    }
    let fp = [7u8; 32];
    let proposed =
        strategies::synthesize_from_episodes(
            &episodes,
            |h| {
                if h == "h_abc" {
                    Some(fp)
                } else {
                    None
                }
            },
        );
    assert!(proposed
        .iter()
        .any(|s| s.heuristic.starts_with("skip_check")));
}

#[test]
fn synthesize_proposes_warn_after_two_same_failures() {
    let episodes = vec![
        Episode::now(
            "parse",
            "y.garnet",
            "h_xyz",
            "parse_err",
            Some("UnexpectedToken".to_string()),
            5,
            1,
        ),
        Episode::now(
            "parse",
            "y.garnet",
            "h_xyz",
            "parse_err",
            Some("UnexpectedToken".to_string()),
            6,
            1,
        ),
    ];
    let fp = [11u8; 32];
    let proposed =
        strategies::synthesize_from_episodes(
            &episodes,
            |h| {
                if h == "h_xyz" {
                    Some(fp)
                } else {
                    None
                }
            },
        );
    assert!(proposed
        .iter()
        .any(|s| s.heuristic == "warn_repeated_UnexpectedToken"));
}

#[test]
fn cli_populates_knowledge_db_after_parse() {
    let dir = fresh_dir("cli_kb");
    let f = dir.join("hi.garnet");
    std::fs::write(&f, "def main() { 1 + 2 }").unwrap();
    for _ in 0..2 {
        let _ = Command::new(garnet_bin())
            .current_dir(&dir)
            .args(["parse", f.to_str().unwrap()])
            .output()
            .unwrap();
    }
    let conn = knowledge::open(&dir).unwrap();
    let count = knowledge::count_contexts(&conn).unwrap();
    assert!(count >= 2, "expected ≥2 contexts, got {count}");
}

#[test]
fn cli_populates_strategies_db_after_three_ok_parses() {
    let dir = fresh_dir("cli_strat");
    let f = dir.join("ok.garnet");
    std::fs::write(&f, "def main() { 42 }").unwrap();

    for _ in 0..3 {
        let out = Command::new(garnet_bin())
            .current_dir(&dir)
            .args(["parse", f.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(out.status.success());
    }

    let conn = strategies::open(&dir).unwrap();
    let count = strategies::count_strategies(&conn).unwrap();
    assert!(
        count >= 1,
        "expected ≥1 strategy after 3 ok parses, got {count}"
    );
}

#[test]
fn record_strategy_is_idempotent() {
    let dir = fresh_dir("strat_idem");
    let conn = strategies::open(&dir).unwrap();
    let fp = [3u8; 32];
    let s = NewStrategy {
        trigger_fingerprint: fp,
        heuristic: "skip_check_if_unchanged_since_last_ok".to_string(),
        justifying_episode_ids: vec![0, 1, 2],
    };
    strategies::record_strategy(&conn, &s, 1).unwrap();
    strategies::record_strategy(&conn, &s, 2).unwrap(); // same trigger+heuristic
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM strategies", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1, "duplicate strategies must be deduped");
}
