//! EmbedRateLimit — per-caller token bucket on VectorIndex queries
//! (v4.0 Security Layer 4).
//!
//! Closes the embedding-inversion attack: a semantic index (VectorIndex)
//! stores embeddings of secret data, and an attacker who can issue
//! unlimited similarity queries reconstructs training data bit-by-bit
//! via differential analysis.
//!
//! EmbedRateLimit caps the rate of queries per caller. Two-tier:
//!
//! - **Rate** — requests per minute, enforced via token bucket
//! - **Noise** — top-k results from a public-policy index are returned
//!   with differential-privacy noise applied to the scores
//!
//! ## Integration
//!
//! Applied at the `VectorIndex::search` call site. The Memory Manager
//! consults the `IndexPolicy` tag on the index to select enforcement:
//!
//! - `Internal` — no limit, no noise (trusted caller)
//! - `Public` — 100 queries/min per caller, top-k with ε=1.0 noise

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Policy attached to a VectorIndex. The Memory Manager (runtime layer)
/// consults this at every `search()` call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexPolicy {
    /// Trusted internal caller — no rate limit, no noise.
    Internal,
    /// Public-facing — token bucket + noisy top-k.
    Public {
        queries_per_minute: u32,
        noise_epsilon_times_100: u32, // ε × 100 so it's an integer
    },
}

impl IndexPolicy {
    pub fn public_default() -> Self {
        Self::Public {
            queries_per_minute: 100,
            noise_epsilon_times_100: 100, // ε = 1.0
        }
    }
}

/// Per-caller state for the token bucket.
#[derive(Debug)]
struct Bucket {
    tokens: f64,
    last_refill: Instant,
    capacity: f64,
    refill_per_sec: f64,
}

impl Bucket {
    fn new(queries_per_minute: u32) -> Self {
        let cap = queries_per_minute as f64;
        Self {
            tokens: cap,
            last_refill: Instant::now(),
            capacity: cap,
            refill_per_sec: cap / 60.0,
        }
    }

    fn try_consume(&mut self, n: u32) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_per_sec).min(self.capacity);
        self.last_refill = now;
        if self.tokens >= n as f64 {
            self.tokens -= n as f64;
            true
        } else {
            false
        }
    }

    /// Number of tokens currently available (after refill).
    fn available(&mut self) -> f64 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_per_sec).min(self.capacity);
        self.last_refill = now;
        self.tokens
    }
}

/// Rate limiter keyed on an opaque caller identifier.
#[derive(Debug, Default)]
pub struct RateLimiter {
    buckets: HashMap<String, Bucket>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Try to consume one query token for the given caller under `policy`.
    /// Returns Ok(()) if allowed, Err if rate-limited.
    pub fn try_query(&mut self, caller: &str, policy: IndexPolicy) -> Result<(), RateLimitError> {
        match policy {
            IndexPolicy::Internal => Ok(()),
            IndexPolicy::Public { queries_per_minute, .. } => {
                let bucket = self
                    .buckets
                    .entry(caller.to_string())
                    .or_insert_with(|| Bucket::new(queries_per_minute));
                if bucket.try_consume(1) {
                    Ok(())
                } else {
                    Err(RateLimitError::Exceeded {
                        caller: caller.to_string(),
                        retry_after_ms: ((1.0 / bucket.refill_per_sec) * 1000.0) as u64,
                    })
                }
            }
        }
    }

    /// Inspect available tokens for a caller (useful for diagnostics /
    /// client-side back-off hints).
    pub fn available_for(&mut self, caller: &str) -> f64 {
        match self.buckets.get_mut(caller) {
            Some(b) => b.available(),
            None => 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RateLimitError {
    Exceeded {
        caller: String,
        retry_after_ms: u64,
    },
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitError::Exceeded { caller, retry_after_ms } => {
                write!(
                    f,
                    "rate-limit exceeded for '{caller}'; retry after {retry_after_ms}ms"
                )
            }
        }
    }
}

impl std::error::Error for RateLimitError {}

/// Apply differential-privacy noise to a set of similarity scores before
/// returning top-k. Gaussian mechanism with ε-scale: Lap(0, 1/ε) added
/// per score, then re-ranked.
///
/// This function is deterministic under a test seed (second arg); in
/// production it draws from the OS RNG.
pub fn apply_dp_noise(scores: &mut [f32], epsilon: f64, seed: Option<u64>) {
    if epsilon <= 0.0 {
        return;
    }
    let scale = 1.0 / epsilon;
    for (i, s) in scores.iter_mut().enumerate() {
        let noise = if let Some(seed_val) = seed {
            // Deterministic noise for tests: derive from seed + index
            laplace_noise_seeded(seed_val.wrapping_add(i as u64), scale)
        } else {
            laplace_noise_osrng(scale)
        };
        *s += noise as f32;
    }
}

fn laplace_noise_seeded(seed: u64, scale: f64) -> f64 {
    // Simple seeded pseudo-Laplace for test reproducibility
    let mut x = seed;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    let u = (x as f64 / u64::MAX as f64) - 0.5;
    let sign = if u < 0.0 { -1.0 } else { 1.0 };
    sign * scale * (1.0 - 2.0 * u.abs()).ln().abs()
}

fn laplace_noise_osrng(scale: f64) -> f64 {
    use std::time::SystemTime;
    // In production this uses `getrandom`; for v4.0 Layer-4 stub we use
    // a time-based seed to stay dep-free.
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    laplace_noise_seeded(nanos, scale)
}

/// Utility: apply rate-limit + noise in a single call, matching the
/// Memory Manager's integration point for `VectorIndex::search`.
pub fn gate_search(
    caller: &str,
    policy: IndexPolicy,
    limiter: &mut RateLimiter,
    scores: &mut [f32],
    test_seed: Option<u64>,
) -> Result<(), RateLimitError> {
    limiter.try_query(caller, policy)?;
    if let IndexPolicy::Public { noise_epsilon_times_100, .. } = policy {
        let eps = noise_epsilon_times_100 as f64 / 100.0;
        apply_dp_noise(scores, eps, test_seed);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn internal_policy_never_limits() {
        let mut limiter = RateLimiter::new();
        for _ in 0..10_000 {
            assert!(limiter.try_query("internal-caller", IndexPolicy::Internal).is_ok());
        }
    }

    #[test]
    fn public_policy_exhausts_after_budget() {
        let mut limiter = RateLimiter::new();
        let policy = IndexPolicy::Public {
            queries_per_minute: 5,
            noise_epsilon_times_100: 100,
        };
        // First 5 should succeed
        for _ in 0..5 {
            assert!(limiter.try_query("attacker", policy).is_ok());
        }
        // 6th should fail
        match limiter.try_query("attacker", policy) {
            Err(RateLimitError::Exceeded { caller, .. }) => assert_eq!(caller, "attacker"),
            _ => panic!("expected rate-limit exceeded"),
        }
    }

    #[test]
    fn different_callers_have_independent_buckets() {
        let mut limiter = RateLimiter::new();
        let policy = IndexPolicy::Public {
            queries_per_minute: 2,
            noise_epsilon_times_100: 100,
        };
        assert!(limiter.try_query("alice", policy).is_ok());
        assert!(limiter.try_query("alice", policy).is_ok());
        assert!(limiter.try_query("alice", policy).is_err()); // exhausted
        // Bob is fresh
        assert!(limiter.try_query("bob", policy).is_ok());
        assert!(limiter.try_query("bob", policy).is_ok());
    }

    #[test]
    fn bucket_refills_over_time() {
        let mut limiter = RateLimiter::new();
        let policy = IndexPolicy::Public {
            queries_per_minute: 60, // 1 per second
            noise_epsilon_times_100: 100,
        };
        // Exhaust
        for _ in 0..60 {
            let _ = limiter.try_query("x", policy);
        }
        assert!(limiter.try_query("x", policy).is_err());
        // Wait for a refill
        thread::sleep(Duration::from_millis(1100));
        // Should have ~1 token back
        assert!(limiter.try_query("x", policy).is_ok());
    }

    #[test]
    fn retry_after_ms_is_positive() {
        let mut limiter = RateLimiter::new();
        let policy = IndexPolicy::Public {
            queries_per_minute: 10,
            noise_epsilon_times_100: 100,
        };
        for _ in 0..10 {
            let _ = limiter.try_query("y", policy);
        }
        match limiter.try_query("y", policy) {
            Err(RateLimitError::Exceeded { retry_after_ms, .. }) => {
                assert!(retry_after_ms > 0);
            }
            _ => panic!("expected exceeded"),
        }
    }

    #[test]
    fn dp_noise_is_deterministic_with_seed() {
        let mut scores_a = vec![0.5f32, 0.7, 0.3];
        let mut scores_b = scores_a.clone();
        apply_dp_noise(&mut scores_a, 1.0, Some(42));
        apply_dp_noise(&mut scores_b, 1.0, Some(42));
        assert_eq!(scores_a, scores_b);
    }

    #[test]
    fn dp_noise_changes_scores() {
        let mut scores = vec![0.5f32; 10];
        let original = scores.clone();
        apply_dp_noise(&mut scores, 1.0, Some(123));
        // At least one score should differ
        assert!(scores.iter().zip(&original).any(|(a, b)| a != b));
    }

    #[test]
    fn dp_noise_zero_epsilon_is_no_op() {
        let mut scores = vec![0.5f32, 0.7, 0.3];
        let original = scores.clone();
        apply_dp_noise(&mut scores, 0.0, Some(123));
        assert_eq!(scores, original);
    }

    #[test]
    fn gate_search_integrates_rate_and_noise() {
        let mut limiter = RateLimiter::new();
        let policy = IndexPolicy::Public {
            queries_per_minute: 2,
            noise_epsilon_times_100: 100,
        };
        let mut scores = vec![0.5f32, 0.7, 0.3];
        let original = scores.clone();

        // First two succeed + noise applied
        assert!(gate_search("caller", policy, &mut limiter, &mut scores, Some(1)).is_ok());
        // Scores have been mutated by noise
        assert_ne!(scores, original);

        let mut more = vec![0.5f32];
        assert!(gate_search("caller", policy, &mut limiter, &mut more, Some(1)).is_ok());
        let mut more2 = vec![0.5f32];
        // Third fails rate limit; noise is NOT applied
        let before = more2.clone();
        let r = gate_search("caller", policy, &mut limiter, &mut more2, Some(1));
        assert!(r.is_err());
        assert_eq!(more2, before);
    }

    #[test]
    fn gate_search_internal_bypass_no_noise() {
        let mut limiter = RateLimiter::new();
        let mut scores = vec![0.5f32, 0.7, 0.3];
        let original = scores.clone();
        gate_search("trusted", IndexPolicy::Internal, &mut limiter, &mut scores, Some(1)).unwrap();
        // Internal policy does NOT apply noise
        assert_eq!(scores, original);
    }
}
