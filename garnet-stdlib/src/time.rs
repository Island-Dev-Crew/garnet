//! Time primitives (cap: `time`). Mini-Spec v1.0 §11.2 + Security V2 §1.6.

use crate::StdError;
use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

static START: OnceLock<Instant> = OnceLock::new();

/// Monotonic clock in milliseconds since process start.
///
/// Monotonic means the value NEVER decreases (wall-clock can jump
/// backward on NTP sync; monotonic does not). Preferred for
/// measuring durations.
pub fn now_ms() -> i64 {
    let start = START.get_or_init(Instant::now);
    start.elapsed().as_millis() as i64
}

/// Wall clock in milliseconds since UNIX epoch. Can jump backward on
/// NTP sync — do NOT use for measuring durations.
pub fn wall_clock_ms() -> Result<i64, StdError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .map_err(|e| StdError::Arithmetic(format!("wall clock before epoch: {e}")))
}

/// Sleep the current thread for N milliseconds. Returns `Ok(())` once
/// the sleep completes. `ms < 0` returns `InvalidInput`.
pub fn sleep(ms: i64) -> Result<(), StdError> {
    if ms < 0 {
        return Err(StdError::InvalidInput(format!(
            "sleep duration must be non-negative, got {ms}"
        )));
    }
    std::thread::sleep(Duration::from_millis(ms as u64));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_ms_is_monotonic() {
        let a = now_ms();
        std::thread::sleep(Duration::from_millis(10));
        let b = now_ms();
        assert!(b > a, "expected b > a, got a={a} b={b}");
    }

    #[test]
    fn wall_clock_is_after_2025() {
        // > 2025-01-01 UTC
        let w = wall_clock_ms().unwrap();
        assert!(w > 1_735_689_600_000, "wall clock implausibly low: {w}");
    }

    #[test]
    fn sleep_zero_is_ok() {
        assert!(sleep(0).is_ok());
    }

    #[test]
    fn sleep_negative_is_rejected() {
        match sleep(-1) {
            Err(StdError::InvalidInput(_)) => {}
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    #[test]
    fn sleep_short_actually_elapses() {
        let start = now_ms();
        sleep(30).unwrap();
        let elapsed = now_ms() - start;
        assert!(elapsed >= 25, "expected >= 25ms elapsed, got {elapsed}");
    }
}
