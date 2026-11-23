//! ProvenanceStrategy tests — v3.3 Security Layer 1 (hardening #6).
//!
//! Closes the *strategy-miner adversarial training* threat.
//!
//! Proves:
//!   - A legitimately-mined scrutiny strategy (from ≥2 locally-signed
//!     same-kind failure episodes) passes provenance verification.
//!   - A strategy whose justifying episodes were deleted → quarantined.
//!   - A strategy whose justifying episodes were tampered with → quarantined.
//!   - A strategy synthesised from foreign (wrong-key) episodes →
//!     quarantined because the episodes won't verify and will therefore
//!     be absent from the indexed read.
//!   - A strategy with no provenance (v3.2-era) → quarantined.
//!   - A strategy whose justifications exist+verify but don't actually
//!     satisfy the predicate → quarantined.

use garnet_cli::cache::{record_episode_in_with_key, Episode};
use garnet_cli::provenance::{verify_strategy, QuarantineReason};
use garnet_cli::strategies::{
    self, record_strategy_with_key, synthesize_from_episodes_with_ids, NewStrategy,
};
use std::fs;
use std::path::PathBuf;

fn fresh_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "garnet_prov_{name}_{}_{}",
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

// A helper to record N signed failure episodes for the same source_hash
// and error_kind and return their line indices.
fn record_n_signed_failures(
    cache_dir: &std::path::Path,
    hash: &str,
    error_kind: &str,
    n: usize,
    key: &[u8; 32],
) -> Vec<i64> {
    let mut ids = Vec::new();
    for i in 0..n {
        let ep = Episode::now(
            "check",
            format!("f{i}.garnet"),
            hash,
            "check_err",
            Some(error_kind.to_string()),
            i as u64,
            1,
        );
        record_episode_in_with_key(cache_dir, &ep, key);
        ids.push(i as i64);
    }
    ids
}

// ── Happy path ──────────────────────────────────────────────────────

#[test]
fn legitimately_mined_strategy_passes_provenance() {
    let dir = fresh_dir("happy");
    let cache_dir = dir.join(".garnet-cache");
    let key = [0xAAu8; 32];

    let ids = record_n_signed_failures(&cache_dir, "hashA", "caps_coverage", 2, &key);
    let fp = [0x01u8; 32];
    let strat = NewStrategy {
        trigger_fingerprint: fp,
        heuristic: "warn_repeated_caps_coverage".to_string(),
        justifying_episode_ids: ids,
    };

    let conn = strategies::open(&dir).unwrap();
    record_strategy_with_key(&conn, &strat, 1000, &key).unwrap();

    // Read back the persisted strategy.
    let consulted = strategies::consult_with_audit(&conn, &fp, 1, &key).unwrap();
    assert_eq!(consulted.strategies.len(), 1);
    let (_dist, persisted) = &consulted.strategies[0];

    // Verify provenance against episodes.
    let result = verify_strategy(persisted, &dir, &key);
    assert!(
        result.is_ok(),
        "legitimately mined scrutiny strategy must pass provenance: {result:?}"
    );
}

// ── Missing justification ───────────────────────────────────────────

#[test]
fn strategy_with_deleted_justification_is_quarantined() {
    let dir = fresh_dir("deleted");
    let cache_dir = dir.join(".garnet-cache");
    let key = [0xBBu8; 32];

    record_n_signed_failures(&cache_dir, "hashB", "caps_coverage", 2, &key);

    // Forge a strategy citing justification IDs that don't exist
    // (attacker-picked IDs, or IDs of episodes that were deleted).
    let strat = NewStrategy {
        trigger_fingerprint: [0x02u8; 32],
        heuristic: "warn_repeated_caps_coverage".to_string(),
        justifying_episode_ids: vec![0, 1, 999_999], // 999_999 doesn't exist
    };

    let conn = strategies::open(&dir).unwrap();
    record_strategy_with_key(&conn, &strat, 1001, &key).unwrap();
    let consulted =
        strategies::consult_with_audit(&conn, &strat.trigger_fingerprint, 1, &key).unwrap();
    let (_, persisted) = &consulted.strategies[0];
    let reason = verify_strategy(persisted, &dir, &key).expect_err("must quarantine");
    assert!(
        matches!(
            reason,
            QuarantineReason::MissingOrTamperedJustification {
                missing_id: 999_999
            }
        ),
        "expected MissingOrTamperedJustification for id=999_999, got {reason:?}"
    );
}

// ── Tampered justification ──────────────────────────────────────────

#[test]
fn strategy_with_tampered_justification_is_quarantined() {
    let dir = fresh_dir("tampered");
    let cache_dir = dir.join(".garnet-cache");
    let key = [0xCCu8; 32];

    let ids = record_n_signed_failures(&cache_dir, "hashC", "caps_coverage", 3, &key);
    let strat = NewStrategy {
        trigger_fingerprint: [0x03u8; 32],
        heuristic: "warn_repeated_caps_coverage".to_string(),
        justifying_episode_ids: ids,
    };
    let conn = strategies::open(&dir).unwrap();
    record_strategy_with_key(&conn, &strat, 1002, &key).unwrap();

    // Tamper with the middle episode's outcome.
    let log_path = cache_dir.join("episodes.log");
    let contents = fs::read_to_string(&log_path).unwrap();
    let lines: Vec<&str> = contents.lines().collect();
    let mut new_contents = Vec::with_capacity(lines.len());
    for (i, line) in lines.iter().enumerate() {
        if i == 1 {
            // Flip this record's outcome. HMAC was over old bytes so
            // this tampered line fails verification.
            new_contents
                .push(line.replace("\"outcome\":\"check_err\"", "\"outcome\":\"tampered\""));
        } else {
            new_contents.push((*line).to_string());
        }
    }
    fs::write(&log_path, new_contents.join("\n") + "\n").unwrap();

    let consulted =
        strategies::consult_with_audit(&conn, &strat.trigger_fingerprint, 1, &key).unwrap();
    let (_, persisted) = &consulted.strategies[0];
    let reason = verify_strategy(persisted, &dir, &key).expect_err("must quarantine");
    // The tampered episode will fail HMAC verification, making it
    // invisible in the indexed read — so provenance sees a missing id.
    assert!(
        matches!(
            reason,
            QuarantineReason::MissingOrTamperedJustification { missing_id: 1 }
        ),
        "expected MissingOrTamperedJustification for id=1, got {reason:?}"
    );
}

// ── Strategy with no provenance (v3.2-era) ──────────────────────────

#[test]
fn strategy_with_empty_justification_is_quarantined() {
    let dir = fresh_dir("empty");
    let cache_dir = dir.join(".garnet-cache");
    let key = [0xDDu8; 32];
    record_n_signed_failures(&cache_dir, "hashD", "caps_coverage", 2, &key);

    let strat = NewStrategy {
        trigger_fingerprint: [0x04u8; 32],
        heuristic: "warn_repeated_caps_coverage".to_string(),
        justifying_episode_ids: vec![], // no provenance
    };
    let conn = strategies::open(&dir).unwrap();
    record_strategy_with_key(&conn, &strat, 1003, &key).unwrap();
    let consulted =
        strategies::consult_with_audit(&conn, &strat.trigger_fingerprint, 1, &key).unwrap();
    let (_, persisted) = &consulted.strategies[0];
    let reason = verify_strategy(persisted, &dir, &key).expect_err("must quarantine");
    assert!(
        matches!(reason, QuarantineReason::NoJustification),
        "expected NoJustification, got {reason:?}"
    );
}

// ── Predicate mismatch — IDs exist but don't satisfy the rule ──────

#[test]
fn strategy_citing_mismatched_source_hashes_is_quarantined() {
    let dir = fresh_dir("mixed");
    let cache_dir = dir.join(".garnet-cache");
    let key = [0xEEu8; 32];

    record_episode_in_with_key(
        &cache_dir,
        &Episode::now(
            "check",
            "a.garnet",
            "X",
            "check_err",
            Some("caps_coverage".to_string()),
            1,
            1,
        ),
        &key,
    );
    record_episode_in_with_key(
        &cache_dir,
        &Episode::now(
            "check",
            "b.garnet",
            "Y",
            "check_err",
            Some("caps_coverage".to_string()),
            2,
            1,
        ),
        &key,
    );

    let strat = NewStrategy {
        trigger_fingerprint: [0x05u8; 32],
        heuristic: "warn_repeated_caps_coverage".to_string(),
        justifying_episode_ids: vec![0, 1],
    };
    let conn = strategies::open(&dir).unwrap();
    record_strategy_with_key(&conn, &strat, 1004, &key).unwrap();
    let consulted =
        strategies::consult_with_audit(&conn, &strat.trigger_fingerprint, 1, &key).unwrap();
    let (_, persisted) = &consulted.strategies[0];
    let reason = verify_strategy(persisted, &dir, &key).expect_err("must quarantine");
    match reason {
        QuarantineReason::PredicateMismatch { reason, .. } => {
            assert!(
                reason.contains("source_hash"),
                "predicate-mismatch reason must mention source_hash: {reason}"
            );
        }
        other => panic!("expected PredicateMismatch, got {other:?}"),
    }
}

// ── Cross-machine poisoning: strategy signed with foreign key ──────

#[test]
fn strategy_with_foreign_hmac_is_quarantined() {
    let dir = fresh_dir("foreign_strat");
    let cache_dir = dir.join(".garnet-cache");
    let our_key = [0xAAu8; 32];
    let foreign_key = [0x55u8; 32];

    record_n_signed_failures(
        &cache_dir,
        "attacker_hash",
        "caps_coverage",
        2,
        &foreign_key,
    );
    let strat = NewStrategy {
        trigger_fingerprint: [0x06u8; 32],
        heuristic: "warn_repeated_caps_coverage".to_string(),
        justifying_episode_ids: vec![0, 1],
    };
    let conn = strategies::open(&dir).unwrap();
    record_strategy_with_key(&conn, &strat, 1005, &foreign_key).unwrap();

    // Try to consult using OUR key. The strategy's HMAC fails before
    // it even reaches consult_with_audit's output.
    let consulted =
        strategies::consult_with_audit(&conn, &strat.trigger_fingerprint, 10, &our_key).unwrap();
    assert_eq!(
        consulted.strategies.len(),
        0,
        "foreign-key strategy must be skipped by consult"
    );
    assert_eq!(
        consulted.skipped, 1,
        "skipped count must include the foreign row"
    );
}

// ── Synthesiser output carries justifications ───────────────────────

#[test]
fn synthesiser_produces_strategies_with_provenance() {
    let dir = fresh_dir("synth");
    let cache_dir = dir.join(".garnet-cache");
    let key = [0x77u8; 32];

    let ids = record_n_signed_failures(&cache_dir, "hashS", "caps_coverage", 3, &key);

    let mut episodes = Vec::new();
    for i in 0..3 {
        let mut ep = Episode::now(
            "check",
            format!("f{i}.garnet"),
            "hashS",
            "check_err",
            Some("caps_coverage".to_string()),
            i as u64,
            1,
        );
        ep.sign_with_key(&key);
        episodes.push(ep);
    }
    let id_lookup: std::collections::HashMap<*const Episode, i64> = episodes
        .iter()
        .enumerate()
        .map(|(i, e)| (e as *const Episode, i as i64))
        .collect();

    let strategies = synthesize_from_episodes_with_ids(
        &episodes,
        |_hash| Some([0x01u8; 32]),
        |ep| id_lookup.get(&(ep as *const _)).copied(),
    );

    assert!(
        !strategies.is_empty(),
        "3 signed same-kind failures must fire the warn_repeated_caps_coverage rule"
    );
    for s in &strategies {
        assert_eq!(s.heuristic, "warn_repeated_caps_coverage");
        assert!(
            !s.justifying_episode_ids.is_empty(),
            "synthesiser must attach provenance to every strategy"
        );
    }

    let _ = ids;
}
