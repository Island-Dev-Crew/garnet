//! ProvenanceStrategy — re-derivable strategy verification.
//! v3.3 Security Layer 1 (hardening #6).
//!
//! ## Historical threat & ADR 0017 retirement
//!
//! Historically (v3.2), the compiler-as-agent strategy miner (`strategies.rs`)
//! proposed `skip_check_if_unchanged_since_last_ok` when the same source_hash
//! succeeded ≥3 times. ADR 0017 retired that rule: `strategies.rs` now proposes
//! only scrutiny-adding `warn_repeated_<error_kind>` diagnostics, and
//! `check_predicate_reholds` below unconditionally quarantines any persisted
//! legacy `skip_check_if_unchanged_since_last_ok` row even if validly signed.
//!
//! Provenance verification also defends against strategy-miner poisoning on
//! active scrutiny heuristics: unverified episodes are ignored by CacheHMAC,
//! and persisted strategies must still re-satisfy their miner predicate on
//! verified local episodes.
//!
//! ## The fix
//!
//! Every strategy row carries `justifying_episode_ids` — the IDs of
//! the episodes that satisfied the miner's predicate when the rule
//! was synthesised. At **consult time**, `verify_strategy` re-reads
//! the episodes at those IDs, confirms each has a valid HMAC (so
//! post-synthesis tampering is caught), AND re-runs the miner's
//! predicate against them. If the predicate no longer holds — episodes
//! deleted, episodes tampered, fewer than the threshold — the strategy
//! is quarantined.
//!
//! Failing closed: a strategy that CAN'T be re-derived is ignored.
//! The compiler loses a learned optimization but does not act on a
//! potentially-poisoned one.

use crate::cache::{read_all_indexed_in_with_key, Episode};
use crate::strategies::{verify_strategy_hmac, Strategy};
use std::path::Path;

/// Reason a strategy was quarantined. Callers can surface these as
/// `note: quarantined strategy 'X' because ...` diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuarantineReason {
    /// Strategy's own HMAC didn't verify against the machine key.
    InvalidStrategyHmac,
    /// At least one justifying episode ID doesn't appear in the
    /// HMAC-verified episode log (deleted, tampered, or never existed).
    MissingOrTamperedJustification { missing_id: i64 },
    /// Justifications exist and verify, but the miner's predicate no
    /// longer holds on them — the strategy was valid once and is now
    /// stale, or the strategy was forged with real episode IDs that
    /// happen to verify but don't actually match the predicate.
    PredicateMismatch { heuristic: String, reason: String },
    /// The strategy has no justifying episodes at all (v3.2-era row
    /// persisted before provenance tracking). Fail closed: quarantine
    /// rather than trust a pre-provenance rule.
    NoJustification,
}

/// Verify a strategy's provenance against the episode log. Returns
/// `Ok(())` if the strategy is trustworthy; `Err(QuarantineReason)`
/// otherwise.
///
/// The `episode_dir` is the directory containing `episodes.log` (so
/// `.garnet-cache/` in production). The `key` is the machine key.
pub fn verify_strategy(
    strategy: &Strategy,
    episode_dir: &Path,
    key: &[u8; 32],
) -> Result<(), QuarantineReason> {
    // 1. Strategy's own HMAC must verify.
    if !verify_strategy_hmac(strategy, key) {
        return Err(QuarantineReason::InvalidStrategyHmac);
    }

    // 2. Strategy must declare justifications. A strategy with no
    //    provenance is from v3.2 or a forged row — fail closed.
    if strategy.justifying_episode_ids.is_empty() {
        return Err(QuarantineReason::NoJustification);
    }

    // 3. Re-read locally trusted episodes. The id we use is the
    //    0-indexed line number in episodes.log (same convention as
    //    the miner). The cache reader enforces both the machine-key MAC
    //    and the source-tree binding, so copied same-machine caches cannot
    //    re-justify a strategy in a different checkout.
    let cache_dir = resolve_cache_dir(episode_dir);

    // Build a map from id (line index in the ORIGINAL log) to episode.
    let indexed = read_all_indexed_in_with_key(&cache_dir, key);
    let index: std::collections::BTreeMap<i64, &Episode> =
        indexed.iter().map(|(id, ep)| (*id, ep)).collect();

    // 4. Every justifying id must exist in the verified-episode index.
    let mut resolved: Vec<&Episode> = Vec::new();
    for id in &strategy.justifying_episode_ids {
        match index.get(id) {
            Some(ep) => resolved.push(*ep),
            None => {
                return Err(QuarantineReason::MissingOrTamperedJustification { missing_id: *id });
            }
        }
    }

    // 5. The miner's predicate must still hold on those episodes.
    check_predicate_reholds(strategy, &resolved)?;

    Ok(())
}

fn resolve_cache_dir(dir: &Path) -> std::path::PathBuf {
    let nested = dir.join(".garnet-cache").join("episodes.log");
    if nested.exists() {
        dir.join(".garnet-cache")
    } else {
        dir.to_path_buf()
    }
}

/// Check that the miner's predicate re-holds on the resolved episodes.
/// This is tightly coupled to the synthesiser in `strategies.rs` — if
/// the heuristic-naming convention changes there, update this too.
fn check_predicate_reholds(
    strategy: &Strategy,
    resolved: &[&Episode],
) -> Result<(), QuarantineReason> {
    let heuristic = &strategy.heuristic;
    if heuristic == "skip_check_if_unchanged_since_last_ok" {
        return Err(QuarantineReason::PredicateMismatch {
            heuristic: heuristic.clone(),
            reason: "retired by ADR 0017: no track record or cache history lightens review".into(),
        });
    }
    if let Some(kind) = heuristic.strip_prefix("warn_repeated_") {
        // Predicate: ≥2 episodes, same source_hash, same error_kind == kind.
        if resolved.len() < 2 {
            return Err(QuarantineReason::PredicateMismatch {
                heuristic: heuristic.clone(),
                reason: format!("needs ≥2 justifications, got {}", resolved.len()),
            });
        }
        let first_hash = &resolved[0].source_hash;
        for ep in resolved {
            if &ep.source_hash != first_hash {
                return Err(QuarantineReason::PredicateMismatch {
                    heuristic: heuristic.clone(),
                    reason: "justifications span multiple source_hashes".into(),
                });
            }
            if ep.error_kind.as_deref() != Some(kind) {
                return Err(QuarantineReason::PredicateMismatch {
                    heuristic: heuristic.clone(),
                    reason: format!(
                        "justification error_kind {:?} != expected {:?}",
                        ep.error_kind, kind
                    ),
                });
            }
        }
        Ok(())
    } else {
        // Unknown heuristic — fail closed. New heuristics must register
        // their predicate here explicitly.
        Err(QuarantineReason::PredicateMismatch {
            heuristic: heuristic.clone(),
            reason: "no provenance predicate registered for this heuristic".into(),
        })
    }
}
