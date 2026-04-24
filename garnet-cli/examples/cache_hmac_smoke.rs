//! Smoke test for CacheHMAC + ProvenanceStrategy — runs as a normal
//! binary (not via `cargo test`) to sidestep the local MinGW/WinLibs
//! ABI mismatch that prevents libtest-based test binaries from running
//! on this machine. Exercises the same code paths as
//! `tests/cache_hmac.rs` and `tests/provenance.rs`; exits non-zero on
//! any assertion failure.

use garnet_cli::cache::{
    read_all_in_with_key, recall_in_with_key, record_episode_in_with_key, Episode,
};
use garnet_cli::provenance::{verify_strategy, QuarantineReason};
use garnet_cli::strategies::{self, record_strategy_with_key, NewStrategy};
use std::fs;
use std::path::PathBuf;

fn fresh_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "garnet_smoke_{name}_{}_{}",
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

fn check(cond: bool, name: &str) {
    if cond {
        println!("  ✓ {name}");
    } else {
        eprintln!("  ✗ {name} FAILED");
        std::process::exit(1);
    }
}

fn main() {
    println!("== CacheHMAC smoke tests ==");

    // Sign + verify round-trip.
    let k1 = [0xAAu8; 32];
    let k2 = [0x55u8; 32];
    let mut ep = Episode::now("parse", "f.garnet", "h", "ok", None, 1, 0);
    ep.sign_with_key(&k1);
    check(
        ep.verify_with_key(&k1),
        "signed episode verifies with same key",
    );
    check(
        !ep.verify_with_key(&k2),
        "signed episode fails with foreign key",
    );

    // Tamper detection.
    ep.source_hash = "attacker".to_string();
    check(
        !ep.verify_with_key(&k1),
        "tampered source_hash fails verification",
    );

    // Unsigned always fails.
    let unsigned = Episode::now("parse", "f.garnet", "h", "ok", None, 1, 0);
    check(
        !unsigned.verify_with_key(&k1),
        "unsigned episode fails verification",
    );

    // On-disk skip of foreign records.
    let dir = fresh_dir("foreign");
    let cache_dir = dir.join(".garnet-cache");
    record_episode_in_with_key(
        &cache_dir,
        &Episode::now("parse", "a.garnet", "h1", "ok", None, 1, 0),
        &k1,
    );
    record_episode_in_with_key(
        &cache_dir,
        &Episode::now("parse", "b.garnet", "h2", "ok", None, 2, 0),
        &k2,
    );
    record_episode_in_with_key(
        &cache_dir,
        &Episode::now("parse", "c.garnet", "h1", "ok", None, 3, 0),
        &k1,
    );
    let result = read_all_in_with_key(&cache_dir, &k1);
    check(result.episodes.len() == 2, "our-key records survive");
    check(result.skipped == 1, "foreign record counted as skipped");

    // recall also filters.
    let recall = recall_in_with_key(&cache_dir, "h1", &k1);
    check(recall.episodes.len() == 2, "recall returns 2 ours for h1");
    check(recall.skipped == 1, "recall skipped 1 foreign");

    // Legacy unsigned records on disk — skipped.
    let dir2 = fresh_dir("legacy");
    let cache_dir2 = dir2.join(".garnet-cache");
    fs::create_dir_all(&cache_dir2).unwrap();
    let log = cache_dir2.join("episodes.log");
    let legacy = r#"{"ts":1,"cmd":"parse","file":"x","source_hash":"h","outcome":"ok","duration_ms":1,"parser_version":"0.3","exit_code":0}"#;
    fs::write(&log, format!("{legacy}\n")).unwrap();
    let lr = read_all_in_with_key(&cache_dir2, &k1);
    check(lr.episodes.is_empty(), "legacy unsigned records skipped");
    check(lr.skipped == 1, "legacy skipped count = 1");

    println!();
    println!("== ProvenanceStrategy smoke tests ==");

    // Legitimate strategy with full provenance — passes.
    let dir3 = fresh_dir("prov_happy");
    let cache_dir3 = dir3.join(".garnet-cache");
    let mut ids = Vec::new();
    for i in 0..3 {
        let ep = Episode::now(
            "check",
            format!("f{i}.garnet"),
            "hashA",
            "ok",
            None,
            i as u64,
            0,
        );
        record_episode_in_with_key(&cache_dir3, &ep, &k1);
        ids.push(i as i64);
    }
    let strat = NewStrategy {
        trigger_fingerprint: [0x01u8; 32],
        heuristic: "skip_check_if_unchanged_since_last_ok".to_string(),
        justifying_episode_ids: ids,
    };
    let conn = strategies::open(&dir3).unwrap();
    record_strategy_with_key(&conn, &strat, 1000, &k1).unwrap();
    let consulted =
        strategies::consult_with_audit(&conn, &strat.trigger_fingerprint, 1, &k1).unwrap();
    check(consulted.strategies.len() == 1, "legit strategy retrieved");
    let (_, persisted) = &consulted.strategies[0];
    check(
        verify_strategy(persisted, &dir3, &k1).is_ok(),
        "legit strategy passes provenance",
    );

    // Missing justification — quarantined.
    let strat2 = NewStrategy {
        trigger_fingerprint: [0x02u8; 32],
        heuristic: "skip_check_if_unchanged_since_last_ok".to_string(),
        justifying_episode_ids: vec![0, 1, 2, 999],
    };
    record_strategy_with_key(&conn, &strat2, 1001, &k1).unwrap();
    let consulted2 =
        strategies::consult_with_audit(&conn, &strat2.trigger_fingerprint, 1, &k1).unwrap();
    let (_, p2) = &consulted2.strategies[0];
    let r2 = verify_strategy(p2, &dir3, &k1).expect_err("must quarantine missing id");
    check(
        matches!(
            r2,
            QuarantineReason::MissingOrTamperedJustification { missing_id: 999 }
        ),
        "missing id 999 quarantined",
    );

    // No justification — quarantined.
    let strat3 = NewStrategy {
        trigger_fingerprint: [0x03u8; 32],
        heuristic: "skip_check_if_unchanged_since_last_ok".to_string(),
        justifying_episode_ids: vec![],
    };
    record_strategy_with_key(&conn, &strat3, 1002, &k1).unwrap();
    let consulted3 =
        strategies::consult_with_audit(&conn, &strat3.trigger_fingerprint, 1, &k1).unwrap();
    let (_, p3) = &consulted3.strategies[0];
    let r3 = verify_strategy(p3, &dir3, &k1).expect_err("must quarantine no-provenance");
    check(
        matches!(r3, QuarantineReason::NoJustification),
        "empty justification quarantined",
    );

    // Foreign-key strategy — not even retrieved by consult.
    let strat4 = NewStrategy {
        trigger_fingerprint: [0x04u8; 32],
        heuristic: "skip_check_if_unchanged_since_last_ok".to_string(),
        justifying_episode_ids: vec![0, 1, 2],
    };
    record_strategy_with_key(&conn, &strat4, 1003, &k2).unwrap();
    let consulted4 =
        strategies::consult_with_audit(&conn, &strat4.trigger_fingerprint, 10, &k1).unwrap();
    check(
        consulted4.strategies.is_empty(),
        "foreign-key strategy filtered out",
    );
    check(
        consulted4.skipped >= 1,
        "foreign-key strategy counted as skipped",
    );

    println!();
    println!("== ALL SMOKE TESTS PASSED ==");
}
