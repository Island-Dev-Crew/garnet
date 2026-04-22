//! CacheHMAC tests — v3.3 Security Layer 1 (hardening #5).
//!
//! Proves the threat model:
//!   - Episodes signed with machine key A verify with key A.
//!   - Episodes signed with key A do NOT verify with key B.
//!   - Episodes whose bytes have been tampered after signing fail verify.
//!   - Episodes without a MAC are rejected on read.
//!   - Read path reports the skip count so callers can warn loudly.

use garnet_cli::cache::{
    read_all_in_with_key, recall_in_with_key, record_episode_in_with_key, Episode,
};
use std::fs;
use std::path::PathBuf;

fn fresh_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "garnet_hmac_{name}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

// ── Sign + verify round-trips ───────────────────────────────────────

#[test]
fn signed_episode_verifies_with_same_key() {
    let key = [0xAAu8; 32];
    let mut ep = Episode::now("parse", "foo.garnet", "abc", "ok", None, 10, 0);
    ep.sign_with_key(&key);
    assert!(ep.verify_with_key(&key), "fresh sign+verify round-trip must pass");
}

#[test]
fn signed_episode_does_not_verify_with_different_key() {
    let k1 = [0xAAu8; 32];
    let k2 = [0x55u8; 32];
    let mut ep = Episode::now("parse", "foo.garnet", "abc", "ok", None, 10, 0);
    ep.sign_with_key(&k1);
    assert!(!ep.verify_with_key(&k2), "cross-machine key must NOT verify");
}

#[test]
fn unsigned_episode_never_verifies() {
    let key = [0u8; 32];
    let ep = Episode::now("parse", "foo.garnet", "abc", "ok", None, 10, 0);
    assert!(ep.hmac.is_none());
    assert!(!ep.verify_with_key(&key), "no HMAC means no trust");
}

#[test]
fn tampered_episode_fails_verification() {
    let key = [0xAAu8; 32];
    let mut ep = Episode::now("parse", "foo.garnet", "abc", "ok", None, 10, 0);
    ep.sign_with_key(&key);
    assert!(ep.verify_with_key(&key));

    // Tamper with one field AFTER signing. The stored HMAC was computed
    // over the old bytes; re-canonicalizing with the new field diverges.
    ep.source_hash = "attacker-controlled".to_string();
    assert!(
        !ep.verify_with_key(&key),
        "tampering with source_hash must break verification"
    );
}

#[test]
fn tampered_outcome_fails_verification() {
    // The outcome field is what suppresses the safety checker via
    // strategy mining — tampering with it is the highest-value attack.
    let key = [0xAAu8; 32];
    let mut ep = Episode::now("check", "foo.garnet", "abc", "check_err", None, 10, 2);
    ep.sign_with_key(&key);
    assert!(ep.verify_with_key(&key));
    ep.outcome = "ok".to_string(); // promote a failure to a success
    ep.exit_code = 0;
    assert!(!ep.verify_with_key(&key), "outcome flip must break verification");
}

// ── Round-trip through NDJSON ───────────────────────────────────────

#[test]
fn signed_episode_survives_ndjson_roundtrip() {
    let key = [0x77u8; 32];
    let mut ep = Episode::now("check", "x.garnet", "hhash", "ok", None, 5, 0);
    ep.sign_with_key(&key);
    let line = ep.to_ndjson_line();
    let parsed = Episode::from_ndjson_line(&line).expect("parse");
    assert_eq!(parsed.hmac, ep.hmac);
    assert!(parsed.verify_with_key(&key));
}

#[test]
fn signed_episode_with_error_kind_survives_roundtrip() {
    let key = [0x77u8; 32];
    let mut ep = Episode::now(
        "check",
        "bad.garnet",
        "deadbeef",
        "check_err",
        Some("safe_violation".to_string()),
        55,
        1,
    );
    ep.sign_with_key(&key);
    let line = ep.to_ndjson_line();
    let parsed = Episode::from_ndjson_line(&line).unwrap();
    assert_eq!(parsed, ep);
    assert!(parsed.verify_with_key(&key));
}

// ── On-disk read path skips foreign + tampered records ──────────────

#[test]
fn read_all_in_with_key_skips_foreign_machine_records() {
    let dir = fresh_dir("foreign");
    let cache_dir = dir.join(".garnet-cache");

    let our_key = [0x11u8; 32];
    let foreign_key = [0x99u8; 32];

    // Write 2 records with our key and 2 with the foreign key.
    let ours1 = Episode::now("parse", "a.garnet", "h1", "ok", None, 1, 0);
    let ours2 = Episode::now("check", "a.garnet", "h1", "ok", None, 2, 0);
    let theirs1 = Episode::now("parse", "b.garnet", "h2", "ok", None, 3, 0);
    let theirs2 = Episode::now("check", "b.garnet", "h2", "ok", None, 4, 0);

    record_episode_in_with_key(&cache_dir, &ours1, &our_key);
    record_episode_in_with_key(&cache_dir, &theirs1, &foreign_key);
    record_episode_in_with_key(&cache_dir, &ours2, &our_key);
    record_episode_in_with_key(&cache_dir, &theirs2, &foreign_key);

    let result = read_all_in_with_key(&cache_dir, &our_key);
    assert_eq!(
        result.episodes.len(),
        2,
        "only our-key records must survive the read filter"
    );
    assert_eq!(result.skipped, 2, "exactly 2 foreign records must be counted as skipped");
}

#[test]
fn read_all_in_with_key_skips_tampered_records() {
    let dir = fresh_dir("tamper");
    let cache_dir = dir.join(".garnet-cache");
    let key = [0x22u8; 32];

    let ep1 = Episode::now("parse", "a.garnet", "h1", "ok", None, 1, 0);
    let ep2 = Episode::now("parse", "b.garnet", "h2", "ok", None, 2, 0);
    record_episode_in_with_key(&cache_dir, &ep1, &key);
    record_episode_in_with_key(&cache_dir, &ep2, &key);

    // Simulate tampering: read the log, mutate one line's outcome
    // field from "check_err" to "ok" (or similar), write back. The
    // HMAC was computed over the old bytes so verification should fail.
    let log_path = cache_dir.join("episodes.log");
    let contents = fs::read_to_string(&log_path).unwrap();
    // Replace the first "ok" in the body with "not_ok" — this mutates
    // the outcome of the first record without touching its HMAC field,
    // so verification fails on that record only.
    let tampered = contents.replacen("\"outcome\":\"ok\"", "\"outcome\":\"not_ok\"", 1);
    fs::write(&log_path, tampered).unwrap();

    let result = read_all_in_with_key(&cache_dir, &key);
    assert_eq!(result.episodes.len(), 1);
    assert_eq!(result.skipped, 1);
}

#[test]
fn recall_in_with_key_filters_by_source_hash_only_for_verified() {
    let dir = fresh_dir("recall");
    let cache_dir = dir.join(".garnet-cache");
    let our_key = [0x33u8; 32];
    let foreign_key = [0x44u8; 32];

    // Our key: 3 records for source_hash "A"
    for i in 0..3 {
        let ep = Episode::now(
            "parse",
            format!("a{i}.garnet"),
            "A",
            "ok",
            None,
            i as u64,
            0,
        );
        record_episode_in_with_key(&cache_dir, &ep, &our_key);
    }

    // Foreign key: 5 records for source_hash "A" (attacker injecting
    // fake successes to train the strategy miner).
    for i in 0..5 {
        let ep = Episode::now("parse", format!("evil{i}.garnet"), "A", "ok", None, 0, 0);
        record_episode_in_with_key(&cache_dir, &ep, &foreign_key);
    }

    let result = recall_in_with_key(&cache_dir, "A", &our_key);
    assert_eq!(
        result.episodes.len(),
        3,
        "recall must only return OUR 3 episodes, not the attacker's 5"
    );
    assert_eq!(result.skipped, 5);
}

// ── Legacy unsigned records are ignored on read ─────────────────────

#[test]
fn pre_hmac_records_in_log_are_skipped_on_read() {
    let dir = fresh_dir("legacy");
    let cache_dir = dir.join(".garnet-cache");
    fs::create_dir_all(&cache_dir).unwrap();
    let key = [0xEEu8; 32];

    // Write an old-format record directly (no hmac field).
    let legacy = r#"{"ts":100,"cmd":"parse","file":"x.garnet","source_hash":"h1","outcome":"ok","duration_ms":1,"parser_version":"0.3.0","exit_code":0}"#;
    let log = cache_dir.join("episodes.log");
    fs::write(&log, format!("{legacy}\n")).unwrap();

    let result = read_all_in_with_key(&cache_dir, &key);
    assert_eq!(
        result.episodes.len(),
        0,
        "legacy unsigned records must be skipped on read"
    );
    assert_eq!(result.skipped, 1);
}
