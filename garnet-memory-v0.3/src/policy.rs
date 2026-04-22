//! Per-kind retention policy with R+R+I decay defaults from
//! `GARNET_Memory_Manager_Architecture.md §3`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryKind {
    Working,
    Episodic,
    Semantic,
    Procedural,
}

/// Per-kind retention parameters. All callers may override; these are the
/// principled defaults.
#[derive(Debug, Clone)]
pub struct MemoryPolicy {
    /// Exponential decay rate λ, expressed as "per second". Defaults derive
    /// from the per-kind table in the Memory Manager Architecture doc.
    pub decay_lambda_per_sec: f64,
    /// Retention threshold: items with score < threshold are eligible for
    /// eviction.
    pub retention_threshold: f64,
    /// Maximum in-memory items before compaction (approximate).
    pub compaction_high_water: usize,
}

impl MemoryPolicy {
    pub fn default_for(kind: MemoryKind) -> Self {
        match kind {
            MemoryKind::Working => Self {
                decay_lambda_per_sec: 0.5 / 60.0,
                retention_threshold: 0.1,
                compaction_high_water: 1024,
            },
            MemoryKind::Episodic => Self {
                decay_lambda_per_sec: 0.01 / 86_400.0,
                retention_threshold: 0.3,
                compaction_high_water: 100_000,
            },
            MemoryKind::Semantic => Self {
                decay_lambda_per_sec: 0.001 / 86_400.0,
                retention_threshold: 0.5,
                compaction_high_water: 1_000_000,
            },
            MemoryKind::Procedural => Self {
                decay_lambda_per_sec: 0.0005 / 86_400.0,
                retention_threshold: 0.4,
                compaction_high_water: 10_000,
            },
        }
    }

    /// Core R+R+I scoring function. Callers supply the three components.
    pub fn score(
        &self,
        relevance: f64,
        age_seconds: f64,
        importance: f64,
    ) -> f64 {
        let recency = (-self.decay_lambda_per_sec * age_seconds).exp();
        (relevance.clamp(0.0, 1.0)) * recency * (importance.clamp(0.0, 1.0))
    }

    /// Whether an item with the given score should be retained.
    pub fn should_retain(&self, score: f64) -> bool {
        score >= self.retention_threshold
    }
}
