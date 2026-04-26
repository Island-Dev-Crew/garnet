//! Compiler-as-Agent episode-log tests — Paper VI Contribution 3, layer 1.
//!
//! Drives the CLI against a temp dir, asserts:
//! - `parse`/`check`/`run`/`eval` each append one NDJSON record to
//!   `.garnet-cache/episodes.log` under the cwd.
//! - `recall` (called via the cache module directly) returns those records
//!   filtered by source hash.
//! - The same-hash second invocation surfaces a "prior failures" note when
//!   the prior outcome was an error.

use garnet_cli::cache::{self, Episode};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn garnet_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_garnet"))
}

fn fresh_temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "garnet_cache_{}_{}_{}",
        name,
        std::process::id(),
        rand_suffix()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn rand_suffix() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let seq = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{nanos}_{seq}")
}

#[test]
fn three_parse_invocations_append_three_episodes() {
    let dir = fresh_temp_dir("three_parse");
    let file = dir.join("hello.garnet");
    std::fs::write(&file, "def main() { 42 }").unwrap();

    for _ in 0..3 {
        let out = Command::new(garnet_bin())
            .current_dir(&dir)
            .args(["parse", file.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(out.status.success(), "parse should succeed");
    }

    let cache_dir = dir.join(".garnet-cache");
    let episodes = cache::read_all_in(&cache_dir);
    assert_eq!(
        episodes.len(),
        3,
        "expected 3 episodes, got {}: {:?}",
        episodes.len(),
        episodes
    );
    for ep in &episodes {
        assert_eq!(ep.cmd, "parse");
        assert_eq!(ep.outcome, "ok");
        assert_eq!(ep.exit_code, 0);
    }
}

#[test]
fn parse_failure_records_parse_err_outcome() {
    let dir = fresh_temp_dir("parse_failure");
    let file = dir.join("bad.garnet");
    std::fs::write(&file, "def @!@ syntax error here").unwrap();

    let out = Command::new(garnet_bin())
        .current_dir(&dir)
        .args(["parse", file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success(), "should fail");

    let cache_dir = dir.join(".garnet-cache");
    let episodes = cache::read_all_in(&cache_dir);
    assert_eq!(episodes.len(), 1);
    assert_eq!(episodes[0].outcome, "parse_err");
    assert_eq!(episodes[0].exit_code, 1);
    assert!(episodes[0].error_kind.is_some());
}

#[test]
fn recall_filters_by_source_hash() {
    let dir = fresh_temp_dir("recall_filter");
    let file_a = dir.join("a.garnet");
    let file_b = dir.join("b.garnet");
    std::fs::write(&file_a, "def main() { 1 }").unwrap();
    std::fs::write(&file_b, "def main() { 2 }").unwrap();

    for f in [&file_a, &file_b, &file_a] {
        let _ = Command::new(garnet_bin())
            .current_dir(&dir)
            .args(["parse", f.to_str().unwrap()])
            .output()
            .unwrap();
    }

    let cache_dir = dir.join(".garnet-cache");
    let hash_a = cache::source_hash("def main() { 1 }");
    let hash_b = cache::source_hash("def main() { 2 }");
    let recalled_a = cache::recall_in(&cache_dir, &hash_a);
    let recalled_b = cache::recall_in(&cache_dir, &hash_b);
    assert_eq!(recalled_a.len(), 2);
    assert_eq!(recalled_b.len(), 1);
}

#[test]
fn second_run_after_failure_surfaces_prior_failure_note() {
    let dir = fresh_temp_dir("prior_note");
    let file = dir.join("buggy.garnet");
    std::fs::write(&file, "def main() { 99/0 }").unwrap();

    // First run: errors via runtime div-by-zero.
    let _ = Command::new(garnet_bin())
        .current_dir(&dir)
        .args(["run", file.to_str().unwrap()])
        .output()
        .unwrap();
    // Second run: stderr should contain the prior-failures hint.
    let out2 = Command::new(garnet_bin())
        .current_dir(&dir)
        .args(["run", file.to_str().unwrap()])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out2.stderr);
    assert!(
        stderr.contains("prior failure"),
        "expected prior-failure note, got stderr: {stderr}"
    );
}

#[test]
fn episode_ndjson_is_valid_json_per_line() {
    let dir = fresh_temp_dir("ndjson");
    let file = dir.join("clean.garnet");
    std::fs::write(&file, "def main() { 1 + 1 }").unwrap();
    let _ = Command::new(garnet_bin())
        .current_dir(&dir)
        .args(["parse", file.to_str().unwrap()])
        .output()
        .unwrap();

    let log = dir.join(".garnet-cache").join("episodes.log");
    let raw = std::fs::read_to_string(&log).unwrap();
    for line in raw.lines() {
        // Round-trip via Episode parser to validate format.
        let ep = Episode::from_ndjson_line(line)
            .unwrap_or_else(|| panic!("malformed NDJSON line: {line}"));
        assert!(!ep.source_hash.is_empty());
        assert!(!ep.cmd.is_empty());
    }
}
