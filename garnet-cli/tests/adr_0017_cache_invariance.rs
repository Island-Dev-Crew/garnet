//! ADR 0017 — No track record or cache history lightens review.
//!
//! Proves:
//! 1. Repeated "ok" episodes never synthesize `skip_check_if_unchanged_since_last_ok`.
//! 2. Any legacy persisted `skip_check_if_unchanged_since_last_ok` row in
//!    `.garnet-cache/strategies.db` is quarantined even with a valid HMAC and
//!    valid signed "ok" episodes.
//! 3. Populating `.garnet-cache` with prior passing runs never changes the exit
//!    status or diagnostics of `garnet check`, `garnet diff-caps` (including a
//!    per-function capability gain inside a module that already declares that
//!    capability), or `garnet agent-loop`.

use garnet_cli::cache::{record_episode_in_with_key, Episode};
use garnet_cli::provenance::{verify_strategy, QuarantineReason};
use garnet_cli::strategies::{
    self, record_strategy_with_key, synthesize_from_episodes_with_ids, NewStrategy,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn fresh_dir(name: &str) -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nano = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "garnet_adr0017_{name}_{}_{nano}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn garnet_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_garnet"))
}

#[test]
fn skip_check_strategy_is_never_synthesized_from_ok_history() {
    let mut episodes = Vec::new();
    for i in 0..5 {
        episodes.push(Episode::now(
            "check",
            format!("ok_{i}.garnet"),
            "same_hash",
            "ok",
            None,
            i as u64,
            0,
        ));
    }
    let strategies = synthesize_from_episodes_with_ids(
        &episodes,
        |_| Some([0x42u8; 32]),
        |ep| Some(ep.ts as i64),
    );
    assert!(
        strategies
            .iter()
            .all(|s| !s.heuristic.starts_with("skip_check")),
        "ADR 0017 forbids synthesizing skip-check strategies: {strategies:?}"
    );
    assert!(strategies.is_empty());
}

#[test]
fn legacy_persisted_skip_check_strategy_is_always_quarantined() {
    let dir = fresh_dir("legacy_skip");
    let cache_dir = dir.join(".garnet-cache");
    let key = [0x99u8; 32];

    for i in 0..4 {
        let ep = Episode::now(
            "check",
            format!("ok_{i}.garnet"),
            "hash_legacy",
            "ok",
            None,
            i as u64,
            0,
        );
        record_episode_in_with_key(&cache_dir, &ep, &key);
    }

    let strat = NewStrategy {
        trigger_fingerprint: [0x77u8; 32],
        heuristic: "skip_check_if_unchanged_since_last_ok".to_string(),
        justifying_episode_ids: vec![0, 1, 2, 3],
    };
    let conn = strategies::open(&dir).unwrap();
    record_strategy_with_key(&conn, &strat, 2000, &key).unwrap();

    let consulted =
        strategies::consult_with_audit(&conn, &strat.trigger_fingerprint, 1, &key).unwrap();
    assert_eq!(consulted.strategies.len(), 1);
    let (_, persisted) = &consulted.strategies[0];

    let err = verify_strategy(persisted, &dir, &key).expect_err(
        "ADR 0017: persisted skip_check_if_unchanged_since_last_ok must be quarantined",
    );
    match err {
        QuarantineReason::PredicateMismatch { heuristic, reason } => {
            assert_eq!(heuristic, "skip_check_if_unchanged_since_last_ok");
            assert!(
                reason.contains("ADR 0017"),
                "quarantine reason must cite ADR 0017, got: {reason}"
            );
        }
        other => panic!("expected PredicateMismatch citing ADR 0017, got {other:?}"),
    }
}

#[test]
fn cache_history_never_changes_check_diff_caps_or_agent_loop_verdicts() {
    let cold_dir = fresh_dir("cold");
    let warm_dir = fresh_dir("warm");

    let ok_src =
        "@caps(fs)\ndef reader() { fs::read_file(\"/tmp/a\") }\n@caps()\ndef main() { 1 }\n";
    let bad_check_src = "@caps()\ndef main() { fs::read_file(\"/tmp/a\") }\n";
    // Per-function gain inside a module that already declares `fs` (ADR 0017 context).
    let per_fn_gain_src =
        "@caps(fs)\ndef reader() { fs::read_file(\"/tmp/a\") }\n@caps(fs)\ndef main() { 1 }\n";
    let widened_src = "@caps(fs, net)\ndef main() { 1 }\n";

    // Warm `.garnet-cache` with 4 passing `check` runs first.
    let warm_ok = warm_dir.join("ok.garnet");
    fs::write(&warm_ok, ok_src).unwrap();
    for _ in 0..4 {
        let out = Command::new(garnet_bin())
            .current_dir(&warm_dir)
            .args(["check", warm_ok.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(out.status.success(), "warm-up check should succeed");
    }

    // 1. `garnet check` on failing source must reject identically cold vs warm.
    let cold_bad = cold_dir.join("bad.garnet");
    let warm_bad = warm_dir.join("bad.garnet");
    fs::write(&cold_bad, bad_check_src).unwrap();
    fs::write(&warm_bad, bad_check_src).unwrap();
    let cold_check = Command::new(garnet_bin())
        .current_dir(&cold_dir)
        .args(["check", cold_bad.to_str().unwrap()])
        .output()
        .unwrap();
    let warm_check = Command::new(garnet_bin())
        .current_dir(&warm_dir)
        .args(["check", warm_bad.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!cold_check.status.success());
    assert_eq!(cold_check.status.code(), warm_check.status.code());
    assert!(
        !String::from_utf8_lossy(&warm_check.stderr)
            .contains("skip_check_if_unchanged_since_last_ok"),
        "retired skip-check strategy must never surface"
    );

    // 2. `garnet diff-caps --machine` on per-function gain must report the exact
    //    same JSON surface cold vs warm.
    let cold_base = cold_dir.join("base.garnet");
    let cold_gain = cold_dir.join("gain.garnet");
    let warm_gain = warm_dir.join("gain.garnet");
    fs::write(&cold_base, ok_src).unwrap();
    fs::write(&cold_gain, per_fn_gain_src).unwrap();
    fs::write(&warm_gain, per_fn_gain_src).unwrap();
    let cold_diff = Command::new(garnet_bin())
        .current_dir(&cold_dir)
        .args([
            "diff-caps",
            "--machine",
            cold_base.to_str().unwrap(),
            cold_gain.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let warm_diff = Command::new(garnet_bin())
        .current_dir(&warm_dir)
        .args([
            "diff-caps",
            "--machine",
            warm_ok.to_str().unwrap(),
            warm_gain.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(cold_diff.status.code(), warm_diff.status.code());
    let cold_json: serde_json::Value = serde_json::from_slice(&cold_diff.stdout).unwrap();
    let warm_json: serde_json::Value = serde_json::from_slice(&warm_diff.stdout).unwrap();
    assert_eq!(
        cold_json["functions_caps_expanded"],
        warm_json["functions_caps_expanded"]
    );
    assert!(
        !warm_json["functions_caps_expanded"]
            .as_array()
            .unwrap()
            .is_empty(),
        "per-function gain on `main` must still be listed after warm cache"
    );

    // 3. `garnet agent-loop` on authority widening must reject identically cold vs warm.
    let cold_wide = cold_dir.join("wide.garnet");
    let warm_wide = warm_dir.join("wide.garnet");
    fs::write(&cold_wide, widened_src).unwrap();
    fs::write(&warm_wide, widened_src).unwrap();
    let cold_loop = Command::new(garnet_bin())
        .current_dir(&cold_dir)
        .args([
            "agent-loop",
            "--baseline",
            cold_base.to_str().unwrap(),
            "--proposal",
            cold_wide.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let warm_loop = Command::new(garnet_bin())
        .current_dir(&warm_dir)
        .args([
            "agent-loop",
            "--baseline",
            warm_ok.to_str().unwrap(),
            "--proposal",
            warm_wide.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!cold_loop.status.success());
    assert_eq!(cold_loop.status.code(), warm_loop.status.code());
}
