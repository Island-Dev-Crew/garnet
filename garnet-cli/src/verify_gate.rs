//! S33 — `garnet verify` acceptance-gate logic: the merge-confidence band and
//! its `min`-fusion.
//!
//! Kept pure (no I/O) so the band math is unit-tested directly; the CLI wiring
//! that runs parse + safe-mode check lives in `cmd/verify_gate.rs`.
//!
//! The fuse rule is **`min` of the present signals** — a single weak signal
//! caps the result. This mirrors the `dogfood-readiness` skill's gate (a
//! confident external reviewer alone cannot lift a weak internal band), so the
//! local `garnet verify` band composes with the PR-level fusion the skill runs.

/// A 1–5 merge-confidence band. 5 == highest confidence (clean); 1 == a fatal
/// problem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Band(u8);

impl Band {
    /// Construct a band, clamping into the valid 1..=5 range.
    pub fn new(n: u8) -> Band {
        Band(n.clamp(1, 5))
    }

    /// The numeric value (1..=5).
    pub fn get(self) -> u8 {
        self.0
    }
}

/// The pluggable capability signal. S37 `diff-caps` wires this in; until then it
/// is `Pending` and never lowers the fused band.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilitySignal {
    /// Stub: `diff-caps` is not wired until S37; contributes nothing to the fuse.
    Pending,
    /// A real capability-surface band (S37+).
    Surface(Band),
}

impl CapabilitySignal {
    /// The S33 default: the slot exists but is not yet wired.
    pub fn pending_until_s37() -> Self {
        CapabilitySignal::Pending
    }

    fn band(self) -> Option<Band> {
        match self {
            CapabilitySignal::Pending => None,
            CapabilitySignal::Surface(b) => Some(b),
        }
    }
}

/// Aggregated acceptance result over the verified targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GateTally {
    pub targets: usize,
    /// Targets that failed fatally (parse failure or a non-`ok()` check report).
    pub failing: usize,
    /// Count of non-fatal advisory diagnostics across all targets.
    pub advisories: usize,
}

impl GateTally {
    /// The internal acceptance band derived from the local checks: 5 = every
    /// target clean, 4 = clean except non-fatal advisories, 1 = any fatal error.
    pub fn internal_band(self) -> Band {
        if self.failing > 0 {
            Band::new(1)
        } else if self.advisories > 0 {
            Band::new(4)
        } else {
            Band::new(5)
        }
    }

    /// Whether the gate passes (no fatal errors). Drives the process exit code.
    pub fn passes(self) -> bool {
        self.failing == 0
    }
}

/// Fuse the internal band with the optional external reviewer band and the
/// capability signal by `min` over the present signals.
pub fn fuse(internal: Band, external: Option<Band>, capability: CapabilitySignal) -> Band {
    let mut fused = internal;
    if let Some(e) = external {
        fused = fused.min(e);
    }
    if let Some(c) = capability.band() {
        fused = fused.min(c);
    }
    fused
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn band_clamps_into_range() {
        assert_eq!(Band::new(0).get(), 1);
        assert_eq!(Band::new(9).get(), 5);
        assert_eq!(Band::new(3).get(), 3);
    }

    #[test]
    fn internal_band_reflects_tally() {
        assert_eq!(
            GateTally {
                targets: 2,
                failing: 0,
                advisories: 0
            }
            .internal_band()
            .get(),
            5
        );
        assert_eq!(
            GateTally {
                targets: 2,
                failing: 0,
                advisories: 3
            }
            .internal_band()
            .get(),
            4
        );
        assert_eq!(
            GateTally {
                targets: 2,
                failing: 1,
                advisories: 0
            }
            .internal_band()
            .get(),
            1
        );
    }

    #[test]
    fn gate_passes_iff_no_failures() {
        assert!(GateTally {
            targets: 1,
            failing: 0,
            advisories: 5
        }
        .passes());
        assert!(!GateTally {
            targets: 1,
            failing: 1,
            advisories: 0
        }
        .passes());
    }

    #[test]
    fn pending_capability_never_lowers_the_fuse() {
        // A clean internal 5 with the stub capability signal stays 5.
        let fused = fuse(Band::new(5), None, CapabilitySignal::pending_until_s37());
        assert_eq!(fused.get(), 5);
    }

    #[test]
    fn external_band_caps_via_min() {
        // A confident internal band cannot exceed a weaker external reviewer.
        let fused = fuse(Band::new(5), Some(Band::new(3)), CapabilitySignal::Pending);
        assert_eq!(fused.get(), 3);
        // ...and a weak internal is not lifted by a confident external.
        let fused = fuse(Band::new(2), Some(Band::new(5)), CapabilitySignal::Pending);
        assert_eq!(fused.get(), 2);
    }

    #[test]
    fn wired_capability_signal_participates_in_min() {
        let fused = fuse(
            Band::new(5),
            Some(Band::new(5)),
            CapabilitySignal::Surface(Band::new(2)),
        );
        assert_eq!(fused.get(), 2);
    }
}
