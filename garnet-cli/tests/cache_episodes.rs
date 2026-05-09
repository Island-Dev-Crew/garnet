//! Compiler-as-Agent episode-log tests — Paper VI Contribution 3, layer 1.
//!
//! Drives the CLI against a temp dir, asserts:
//! - `parse`/`check`/`run`/`eval` each append one NDJSON record to
//!   `.garnet-cache/episodes.log` under the cwd.
//! - `recall` (called via the cache module directly) returns those records
//!   filtered by source hash.
//! - The same-hash second invocation surfaces a "prior failures" note when
//!   the prior outcome was an error.

use garnet_cli::{
    cache::{self, Episode},
    knowledge, machine_key,
    strategies::{self, NewStrategy},
};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

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

fn key_path(dir: &Path) -> PathBuf {
    dir.join("machine.key")
}

fn garnet_cmd(dir: &Path) -> Command {
    garnet_cmd_with_key(dir, &key_path(dir))
}

fn garnet_cmd_with_key(dir: &Path, key_path: &Path) -> Command {
    let mut cmd = Command::new(garnet_bin());
    cmd.current_dir(dir)
        .env("GARNET_MACHINE_KEY_PATH", key_path);
    cmd
}

fn cache_key(dir: &Path) -> [u8; 32] {
    machine_key::load_or_generate_key(&key_path(dir)).unwrap()
}

fn cache_key_at(path: &Path) -> [u8; 32] {
    machine_key::load_or_generate_key(path).unwrap()
}

fn read_episodes(dir: &Path) -> Vec<Episode> {
    let cache_dir = dir.join(".garnet-cache");
    cache::read_all_in_with_key(&cache_dir, &cache_key(dir)).episodes
}

fn read_episodes_with_key(dir: &Path, key_path: &Path) -> cache::ReadResult {
    let cache_dir = dir.join(".garnet-cache");
    cache::read_all_in_with_key(&cache_dir, &cache_key_at(key_path))
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
        let out = garnet_cmd(&dir)
            .args(["parse", file.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(out.status.success(), "parse should succeed");
    }

    let episodes = read_episodes(&dir);
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

    let out = garnet_cmd(&dir)
        .args(["parse", file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success(), "should fail");

    let episodes = read_episodes(&dir);
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
        let _ = garnet_cmd(&dir)
            .args(["parse", f.to_str().unwrap()])
            .output()
            .unwrap();
    }

    let cache_dir = dir.join(".garnet-cache");
    let key = cache_key(&dir);
    let hash_a = cache::source_hash("def main() { 1 }");
    let hash_b = cache::source_hash("def main() { 2 }");
    let recalled_a = cache::recall_in_with_key(&cache_dir, &hash_a, &key).episodes;
    let recalled_b = cache::recall_in_with_key(&cache_dir, &hash_b, &key).episodes;
    assert_eq!(recalled_a.len(), 2);
    assert_eq!(recalled_b.len(), 1);
}

#[test]
fn second_run_after_failure_surfaces_prior_failure_note() {
    let dir = fresh_temp_dir("prior_note");
    let file = dir.join("buggy.garnet");
    std::fs::write(&file, "def main() { 99/0 }").unwrap();

    // First run: errors via runtime div-by-zero.
    let _ = garnet_cmd(&dir)
        .args(["run", file.to_str().unwrap()])
        .output()
        .unwrap();
    // Second run: stderr should contain the prior-failures hint.
    let out2 = garnet_cmd(&dir)
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
    let _ = garnet_cmd(&dir)
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

#[test]
fn absolute_project_paths_are_logged_as_relative_cache_labels() {
    let dir = fresh_temp_dir("relative_label");
    let nested = dir.join("nested");
    std::fs::create_dir_all(&nested).unwrap();
    let file = nested.join("hello.garnet");
    std::fs::write(&file, "def main() { 42 }").unwrap();

    let out = garnet_cmd(&dir)
        .args(["parse", file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "parse should succeed");

    let episodes = read_episodes(&dir);
    assert_eq!(episodes.len(), 1);
    assert_eq!(episodes[0].file, "nested/hello.garnet");
    assert!(
        !episodes[0].file.contains(dir.to_string_lossy().as_ref()),
        "cache episode leaked absolute project path: {}",
        episodes[0].file
    );
}

#[test]
fn external_absolute_paths_are_redacted_in_cache_labels() {
    let project = fresh_temp_dir("external_project");
    let external = fresh_temp_dir("external_source");
    let file = external.join("secret_agent.garnet");
    std::fs::write(&file, "def main() { 7 }").unwrap();

    let out = garnet_cmd(&project)
        .args(["parse", file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "parse should succeed");

    let episodes = read_episodes(&project);
    assert_eq!(episodes.len(), 1);
    assert_eq!(episodes[0].file, "<external>/secret_agent.garnet");
    assert!(
        !episodes[0]
            .file
            .contains(external.to_string_lossy().as_ref()),
        "cache episode leaked external absolute path: {}",
        episodes[0].file
    );
}

#[test]
fn foreign_machine_episode_in_same_cache_is_ignored_and_warned() {
    let dir = fresh_temp_dir("foreign_same_cache");
    let file = dir.join("buggy.garnet");
    std::fs::write(&file, "def main() { 99/0 }").unwrap();
    let key_a = dir.join("machine-a.key");
    let key_b = dir.join("machine-b.key");

    let out1 = garnet_cmd_with_key(&dir, &key_a)
        .args(["run", file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out1.status.success(), "first run should fail");

    let out2 = garnet_cmd_with_key(&dir, &key_b)
        .args(["run", file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out2.status.success(), "second run should fail");
    let stderr = String::from_utf8_lossy(&out2.stderr);
    assert!(
        stderr.contains("ignored 1 untrusted cache record"),
        "expected untrusted-cache warning, got stderr: {stderr}"
    );
    assert!(
        !stderr.contains("prior failure"),
        "foreign prior failure must not influence diagnostics: {stderr}"
    );

    let result = read_episodes_with_key(&dir, &key_b);
    assert_eq!(result.episodes.len(), 1);
    assert_eq!(result.skipped, 1);
}

#[test]
fn copied_foreign_cache_replay_is_ignored_and_warned() {
    let attacker = fresh_temp_dir("attacker_cache");
    let victim = fresh_temp_dir("victim_cache");
    let source = "def main() { 99/0 }";
    let attacker_file = attacker.join("buggy.garnet");
    let victim_file = victim.join("buggy.garnet");
    std::fs::write(&attacker_file, source).unwrap();
    std::fs::write(&victim_file, source).unwrap();

    let attacker_run = garnet_cmd(&attacker)
        .args(["run", attacker_file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!attacker_run.status.success(), "attacker run should fail");

    let victim_cache = victim.join(".garnet-cache");
    std::fs::create_dir_all(&victim_cache).unwrap();
    std::fs::copy(
        attacker.join(".garnet-cache").join("episodes.log"),
        victim_cache.join("episodes.log"),
    )
    .unwrap();

    let victim_run = garnet_cmd(&victim)
        .args(["run", victim_file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!victim_run.status.success(), "victim run should fail");
    let stderr = String::from_utf8_lossy(&victim_run.stderr);
    assert!(
        stderr.contains("ignored 1 untrusted cache record"),
        "expected untrusted-cache warning, got stderr: {stderr}"
    );
    assert!(
        !stderr.contains("prior failure"),
        "copied foreign failure must not influence diagnostics: {stderr}"
    );

    let result = read_episodes(&victim);
    assert_eq!(result.len(), 1);
}

#[test]
fn copied_same_machine_cache_from_other_source_tree_is_ignored_and_warned() {
    let attacker = fresh_temp_dir("attacker_same_machine_cache");
    let victim = fresh_temp_dir("victim_same_machine_cache");
    let shared_key_path = victim.join("shared-machine.key");
    let source = "def main() { 99/0 }";
    let attacker_file = attacker.join("buggy.garnet");
    let victim_file = victim.join("buggy.garnet");
    std::fs::write(&attacker_file, source).unwrap();
    std::fs::write(&victim_file, source).unwrap();

    let attacker_run = garnet_cmd_with_key(&attacker, &shared_key_path)
        .args(["run", attacker_file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!attacker_run.status.success(), "attacker run should fail");

    let victim_cache = victim.join(".garnet-cache");
    std::fs::create_dir_all(&victim_cache).unwrap();
    std::fs::copy(
        attacker.join(".garnet-cache").join("episodes.log"),
        victim_cache.join("episodes.log"),
    )
    .unwrap();

    let victim_run = garnet_cmd_with_key(&victim, &shared_key_path)
        .args(["run", victim_file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!victim_run.status.success(), "victim run should fail");
    let stderr = String::from_utf8_lossy(&victim_run.stderr);
    assert!(
        stderr.contains("ignored 1 untrusted cache record"),
        "expected copied source-tree warning, got stderr: {stderr}"
    );
    assert!(
        !stderr.contains("prior failure"),
        "copied same-machine failure must not influence diagnostics: {stderr}"
    );

    let result = read_episodes_with_key(&victim, &shared_key_path);
    assert_eq!(result.episodes.len(), 1);
    assert_eq!(result.skipped, 1);
}

#[test]
fn copied_strategy_db_without_local_justifications_is_quarantined_and_warned() {
    let attacker = fresh_temp_dir("attacker_strategy");
    let victim = fresh_temp_dir("victim_strategy");
    let shared_key_path = victim.join("shared-machine.key");
    let key = cache_key_at(&shared_key_path);
    let source = "def main() { 42 }";
    let victim_file = victim.join("safe.garnet");
    std::fs::write(&victim_file, source).unwrap();

    let module = garnet_parser::parse_source(source).unwrap();
    let fp = knowledge::fingerprint(&module);
    let conn = strategies::open(&attacker).unwrap();
    let replayed = NewStrategy {
        trigger_fingerprint: fp,
        heuristic: "skip_check_if_unchanged_since_last_ok".to_string(),
        justifying_episode_ids: vec![0, 1, 2],
    };
    strategies::record_strategy_with_key(&conn, &replayed, 4242, &key).unwrap();
    drop(conn);

    let victim_cache = victim.join(".garnet-cache");
    std::fs::create_dir_all(&victim_cache).unwrap();
    std::fs::copy(
        attacker.join(".garnet-cache").join("strategies.db"),
        victim_cache.join("strategies.db"),
    )
    .unwrap();

    let out = garnet_cmd_with_key(&victim, &shared_key_path)
        .args(["parse", victim_file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "victim parse should still succeed");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("quarantined 1 untrusted strategy record"),
        "expected strategy quarantine warning, got stderr: {stderr}"
    );
    assert!(
        !stderr.contains("strategy 'skip_check_if_unchanged_since_last_ok' applies"),
        "copied strategy must not influence diagnostics: {stderr}"
    );
}

#[test]
fn copied_same_machine_strategy_with_replayed_episodes_is_quarantined() {
    let attacker = fresh_temp_dir("attacker_same_machine_strategy");
    let victim = fresh_temp_dir("victim_same_machine_strategy");
    let shared_key_path = victim.join("shared-machine.key");
    let source = "def main() { 42 }";
    let attacker_file = attacker.join("safe.garnet");
    let victim_file = victim.join("safe.garnet");
    std::fs::write(&attacker_file, source).unwrap();
    std::fs::write(&victim_file, source).unwrap();

    for _ in 0..3 {
        let out = garnet_cmd_with_key(&attacker, &shared_key_path)
            .args(["parse", attacker_file.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(out.status.success(), "attacker parse should succeed");
    }

    let victim_cache = victim.join(".garnet-cache");
    std::fs::create_dir_all(&victim_cache).unwrap();
    for name in ["episodes.log", "strategies.db"] {
        std::fs::copy(
            attacker.join(".garnet-cache").join(name),
            victim_cache.join(name),
        )
        .unwrap();
    }

    let out = garnet_cmd_with_key(&victim, &shared_key_path)
        .args(["parse", victim_file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "victim parse should still succeed");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("ignored 3 untrusted cache record"),
        "expected copied source-tree cache warning, got stderr: {stderr}"
    );
    assert!(
        stderr.contains("quarantined 1 untrusted strategy record"),
        "expected copied source-tree strategy quarantine, got stderr: {stderr}"
    );
    assert!(
        !stderr.contains("strategy 'skip_check_if_unchanged_since_last_ok' applies"),
        "copied same-machine strategy must not influence diagnostics: {stderr}"
    );
}

#[test]
fn concurrent_episode_appends_preserve_all_verified_records() {
    let dir = fresh_temp_dir("concurrent_appends");
    let cache_dir = Arc::new(dir.join(".garnet-cache"));
    let key = Arc::new(cache_key(&dir));
    let writers = 8;
    let records_per_writer = 40;
    let mut handles = Vec::new();

    for writer in 0..writers {
        let cache_dir = Arc::clone(&cache_dir);
        let key = Arc::clone(&key);
        handles.push(std::thread::spawn(move || {
            for seq in 0..records_per_writer {
                let ep = Episode::now(
                    "check",
                    format!("writer_{writer}.garnet"),
                    format!("hash_{writer}_{seq}"),
                    "ok",
                    None,
                    seq as u64,
                    0,
                );
                assert!(cache::record_episode_in_with_key(&cache_dir, &ep, &key));
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let result = cache::read_all_in_with_key(&cache_dir, &key);
    assert_eq!(result.skipped, 0);
    assert_eq!(result.episodes.len(), writers * records_per_writer);
}

#[test]
fn episode_append_soak_preserves_ndjson_and_all_bound_records() {
    let dir = fresh_temp_dir("append_soak");
    let cache_dir = Arc::new(dir.join(".garnet-cache"));
    let key = Arc::new(cache_key(&dir));
    let writers = 16;
    let records_per_writer = 120;
    let mut handles = Vec::new();

    for writer in 0..writers {
        let cache_dir = Arc::clone(&cache_dir);
        let key = Arc::clone(&key);
        handles.push(std::thread::spawn(move || {
            for seq in 0..records_per_writer {
                let ep = Episode::now(
                    "check",
                    format!("soak_{writer}.garnet"),
                    format!("hash_{writer}_{seq}"),
                    "ok",
                    None,
                    seq as u64,
                    0,
                );
                assert!(cache::record_episode_in_with_key(&cache_dir, &ep, &key));
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let log_path = cache_dir.join("episodes.log");
    let raw = std::fs::read_to_string(&log_path).unwrap();
    assert_eq!(raw.lines().count(), writers * records_per_writer);
    for line in raw.lines() {
        Episode::from_ndjson_line(line).unwrap_or_else(|| panic!("malformed NDJSON line: {line}"));
        assert!(
            line.contains("\"source_tree\""),
            "soak-written episodes must carry a source-tree binding"
        );
    }

    let result = cache::read_all_in_with_key(&cache_dir, &key);
    assert_eq!(result.skipped, 0);
    assert_eq!(result.episodes.len(), writers * records_per_writer);
}
