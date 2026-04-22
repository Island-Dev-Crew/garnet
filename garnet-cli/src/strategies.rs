//! Compiler-as-Agent strategy synthesis — Paper VI Contribution 3 (layer 3).
//!
//! `.garnet-cache/strategies.db` is a SQLite store that records optimization
//! and validation heuristics the compiler discovers from past episodes. The
//! synthesizer is intentionally simple in v3.2: it observes patterns in
//! `(source_hash, outcome)` tuples and proposes `skip_check_if_unchanged`
//! and `warn_pattern_X` rules. The CLI consults strategies before each
//! compile and surfaces matching ones as `note:` lines.
//!
//! **v3.3 hardening (CacheHMAC + ProvenanceStrategy).**
//!
//! Every strategy row now carries:
//! - `hmac BLOB` — BLAKE3-keyed MAC over the strategy's canonical bytes,
//!   computed with the per-machine key. Rows with invalid MAC are filtered
//!   out at `consult()` time. Closes the `strategies.db` poisoning threat
//!   (writing to a shared `.garnet-cache/` dir or committing the cache).
//! - `justifying_episode_ids TEXT` — JSON array of episode IDs the strategy
//!   was derived from. At consult time, `provenance::verify_strategy` can
//!   re-check that each justification exists AND has a valid HMAC AND
//!   independently re-satisfies the miner's predicate — quarantining
//!   strategies whose justifications have been deleted or tampered with.
//!
//! These two changes together close the Garnet-specific novel threat of
//! *strategy-miner adversarial training* (attacker pre-seeds episodes so
//! the miner synthesises a check-suppressing rule on the attacker's own
//! source_hash).

use crate::cache::{cache_dir_for, Episode};
use crate::machine_key;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::path::Path;

const STRATEGIES_DB: &str = "strategies.db";

#[derive(Debug, Clone)]
pub struct Strategy {
    pub id: i64,
    pub trigger_fingerprint: [u8; 32],
    pub heuristic: String,
    pub success_count: i64,
    pub failure_count: i64,
    pub created_ts: i64,
    /// HMAC over canonical bytes. Empty `Vec` until signed.
    pub hmac: Vec<u8>,
    /// IDs of episodes that justified this strategy. Parsed from the
    /// JSON-array `justifying_episode_ids` column. Used by
    /// `provenance::verify_strategy` to re-derive the rule.
    pub justifying_episode_ids: Vec<i64>,
}

pub fn open(base: &Path) -> rusqlite::Result<Connection> {
    let dir = cache_dir_for(base);
    std::fs::create_dir_all(&dir).ok();
    let conn = Connection::open(dir.join(STRATEGIES_DB))?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS strategies (
            id INTEGER PRIMARY KEY,
            trigger_fingerprint BLOB NOT NULL,
            heuristic TEXT NOT NULL,
            success_count INTEGER NOT NULL DEFAULT 0,
            failure_count INTEGER NOT NULL DEFAULT 0,
            created_ts INTEGER NOT NULL,
            hmac BLOB NOT NULL DEFAULT (X''),
            justifying_episode_ids TEXT NOT NULL DEFAULT '[]',
            UNIQUE(trigger_fingerprint, heuristic)
        );
        CREATE INDEX IF NOT EXISTS idx_heuristic ON strategies(heuristic);",
    )?;
    // Best-effort column backfill for existing v3.2 databases: ignore
    // the error if columns already exist. This lets an upgraded CLI
    // open a v3.2-era database without forcing users to wipe it.
    let _ = conn.execute_batch(
        "ALTER TABLE strategies ADD COLUMN hmac BLOB NOT NULL DEFAULT (X'');
         ALTER TABLE strategies ADD COLUMN justifying_episode_ids TEXT NOT NULL DEFAULT '[]';",
    );
    Ok(conn)
}

pub fn count_strategies(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM strategies", [], |r| r.get(0))
}

/// Newly proposed strategy emitted by `synthesize_from_episodes`. The caller
/// decides whether to persist them via `record_strategy`.
///
/// v3.3: carries `justifying_episode_ids` so every persisted strategy
/// records *which* episodes supported the predicate. `provenance::verify_strategy`
/// re-checks the justification at consult time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewStrategy {
    pub trigger_fingerprint: [u8; 32],
    pub heuristic: String,
    /// Episode IDs (rowids in the on-disk ndjson or sqlite, per caller's
    /// convention) that satisfied the miner's predicate for this rule.
    /// Empty only for legacy synthesisers that don't track provenance.
    pub justifying_episode_ids: Vec<i64>,
}

/// Walk an episode log and propose strategies. Currently fires on two
/// patterns:
/// - **skip_check_if_unchanged**: the same source_hash succeeded ≥3 times.
/// - **warn_repeated_failure**: the same source_hash failed ≥2 times with
///   the same error_kind.
///
/// v3.3: the caller can pass an `id_of` closure mapping each episode
/// back to its rowid so the returned strategies carry provenance. If
/// the closure returns `None`, the strategy's `justifying_episode_ids`
/// stays empty (but provenance verification will then quarantine it on
/// the next load — explicitly failing closed rather than silently open).
pub fn synthesize_from_episodes_with_ids(
    episodes: &[Episode],
    fingerprint_of: impl Fn(&str) -> Option<[u8; 32]>,
    id_of: impl Fn(&Episode) -> Option<i64>,
) -> Vec<NewStrategy> {
    let mut by_hash_ok: HashMap<&str, Vec<i64>> = HashMap::new();
    let mut by_hash_err: HashMap<(&str, &str), Vec<i64>> = HashMap::new();
    for ep in episodes {
        let id = id_of(ep);
        if ep.outcome == "ok" {
            if let Some(i) = id {
                by_hash_ok.entry(&ep.source_hash).or_default().push(i);
            } else {
                by_hash_ok.entry(&ep.source_hash).or_default();
            }
        } else if let Some(ek) = ep.error_kind.as_deref() {
            if let Some(i) = id {
                by_hash_err.entry((&ep.source_hash, ek)).or_default().push(i);
            }
        }
    }
    let mut out = Vec::new();
    for (hash, ids) in by_hash_ok {
        if ids.len() >= 3 {
            if let Some(fp) = fingerprint_of(hash) {
                out.push(NewStrategy {
                    trigger_fingerprint: fp,
                    heuristic: "skip_check_if_unchanged_since_last_ok".to_string(),
                    justifying_episode_ids: ids,
                });
            }
        }
    }
    for ((hash, kind), ids) in by_hash_err {
        if ids.len() >= 2 {
            if let Some(fp) = fingerprint_of(hash) {
                out.push(NewStrategy {
                    trigger_fingerprint: fp,
                    heuristic: format!("warn_repeated_{kind}"),
                    justifying_episode_ids: ids,
                });
            }
        }
    }
    out.sort_by(|a, b| a.heuristic.cmp(&b.heuristic));
    out
}

/// Back-compat shim: v3.2 callers that don't track episode ids. The
/// strategies returned here will be quarantined on the next load
/// because they have no justification — by design. New code should
/// use `synthesize_from_episodes_with_ids`.
pub fn synthesize_from_episodes(
    episodes: &[Episode],
    fingerprint_of: impl Fn(&str) -> Option<[u8; 32]>,
) -> Vec<NewStrategy> {
    synthesize_from_episodes_with_ids(episodes, fingerprint_of, |_| None)
}

/// Canonical byte serialization of a strategy for HMAC coverage.
/// Length-prefixed fields — two distinct strategies cannot collide to
/// the same canonical bytes.
pub fn canonical_strategy_bytes(
    fingerprint: &[u8; 32],
    heuristic: &str,
    created_ts: i64,
    justifying_episode_ids: &[i64],
) -> Vec<u8> {
    let mut out = Vec::new();
    push_field(&mut out, fingerprint);
    push_field(&mut out, heuristic.as_bytes());
    push_field(&mut out, &created_ts.to_le_bytes());
    out.extend_from_slice(&(justifying_episode_ids.len() as u32).to_le_bytes());
    for id in justifying_episode_ids {
        out.extend_from_slice(&id.to_le_bytes());
    }
    out
}

fn push_field(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(bytes);
}

/// Serialize `[i64]` as a compact JSON array for SQLite TEXT storage.
fn ids_to_json(ids: &[i64]) -> String {
    let parts: Vec<String> = ids.iter().map(|i| i.to_string()).collect();
    format!("[{}]", parts.join(","))
}

fn ids_from_json(s: &str) -> Vec<i64> {
    let s = s.trim();
    let inner = s.trim_start_matches('[').trim_end_matches(']');
    if inner.is_empty() {
        return Vec::new();
    }
    inner
        .split(',')
        .filter_map(|p| p.trim().parse::<i64>().ok())
        .collect()
}

pub fn record_strategy(
    conn: &Connection,
    new_strat: &NewStrategy,
    ts: i64,
) -> rusqlite::Result<i64> {
    record_strategy_with_key(conn, new_strat, ts, machine_key::machine_key())
}

/// Sign + insert. The HMAC covers trigger_fingerprint + heuristic +
/// created_ts + justifying_episode_ids, so tampering with any of those
/// fields (or injecting a forged strategy with the wrong key) breaks
/// verification at `consult` time.
pub fn record_strategy_with_key(
    conn: &Connection,
    new_strat: &NewStrategy,
    ts: i64,
    key: &[u8; 32],
) -> rusqlite::Result<i64> {
    let canonical = canonical_strategy_bytes(
        &new_strat.trigger_fingerprint,
        &new_strat.heuristic,
        ts,
        &new_strat.justifying_episode_ids,
    );
    let mac = machine_key::mac_with_key(key, &canonical);
    let justifying_json = ids_to_json(&new_strat.justifying_episode_ids);
    conn.execute(
        "INSERT OR IGNORE INTO strategies
            (trigger_fingerprint, heuristic, success_count, failure_count, created_ts, hmac, justifying_episode_ids)
         VALUES (?1, ?2, 0, 0, ?3, ?4, ?5)",
        params![
            &new_strat.trigger_fingerprint[..],
            new_strat.heuristic,
            ts,
            &mac[..],
            justifying_json,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Top-k strategies whose trigger fingerprint is closest (Hamming) to
/// `target`. **v3.3 behavior:** strategies whose HMAC does not verify
/// are silently skipped. Use `consult_with_audit` to get the skip count.
pub fn consult(
    conn: &Connection,
    target: &[u8; 32],
    k: usize,
) -> rusqlite::Result<Vec<(u32, Strategy)>> {
    Ok(consult_with_audit(conn, target, k, machine_key::machine_key())?.strategies)
}

/// Result of a verified consult: only strategies whose HMAC verified
/// with the given key are returned; `skipped` counts rejected rows so
/// callers can warn about cache tampering.
pub struct ConsultResult {
    pub strategies: Vec<(u32, Strategy)>,
    pub skipped: usize,
}

pub fn consult_with_audit(
    conn: &Connection,
    target: &[u8; 32],
    k: usize,
    key: &[u8; 32],
) -> rusqlite::Result<ConsultResult> {
    let mut stmt = conn.prepare(
        "SELECT id, trigger_fingerprint, heuristic, success_count, failure_count, created_ts, hmac, justifying_episode_ids
         FROM strategies",
    )?;
    let rows = stmt.query_map([], |r| {
        let fp_bytes: Vec<u8> = r.get(1)?;
        let mut fp = [0u8; 32];
        let len = fp_bytes.len().min(32);
        fp[..len].copy_from_slice(&fp_bytes[..len]);
        let hmac: Vec<u8> = r.get(6)?;
        let justifying_json: String = r.get(7)?;
        Ok(Strategy {
            id: r.get(0)?,
            trigger_fingerprint: fp,
            heuristic: r.get(2)?,
            success_count: r.get(3)?,
            failure_count: r.get(4)?,
            created_ts: r.get(5)?,
            hmac,
            justifying_episode_ids: ids_from_json(&justifying_json),
        })
    })?;
    let mut scored: Vec<(u32, Strategy)> = Vec::new();
    let mut skipped = 0;
    for row in rows {
        let row = row?;
        if !verify_strategy_hmac(&row, key) {
            skipped += 1;
            continue;
        }
        let dist = hamming(&row.trigger_fingerprint, target);
        scored.push((dist, row));
    }
    scored.sort_by_key(|(d, _)| *d);
    scored.truncate(k);
    Ok(ConsultResult {
        strategies: scored,
        skipped,
    })
}

/// Verify a strategy row's HMAC. Constant-time byte comparison to
/// avoid leaking MAC prefix timing info.
pub fn verify_strategy_hmac(s: &Strategy, key: &[u8; 32]) -> bool {
    if s.hmac.len() != 32 {
        return false;
    }
    let canonical = canonical_strategy_bytes(
        &s.trigger_fingerprint,
        &s.heuristic,
        s.created_ts,
        &s.justifying_episode_ids,
    );
    let fresh = machine_key::mac_with_key(key, &canonical);
    let mut diff = 0u8;
    for (recorded, expected) in s.hmac.iter().zip(fresh.iter()) {
        diff |= recorded ^ expected;
    }
    diff == 0
}

fn hamming(a: &[u8; 32], b: &[u8; 32]) -> u32 {
    let mut d = 0u32;
    for i in 0..32 {
        d += (a[i] ^ b[i]).count_ones();
    }
    d
}
