//! R+R+I calibration probe — Working memory.
//!
//! Drives a synthetic 1000-tick workload representative of working-memory
//! access patterns (high churn, narrow time horizon, every item needed
//! within seconds of insertion). Prints CSV: tick,score_high_relevance,
//! score_low_relevance. Inspect to confirm published defaults track the
//! intuition: high-relevance items stay above the retention threshold;
//! low-relevance items fall below within a handful of minutes.

use garnet_memory::{MemoryKind, MemoryPolicy};

fn main() {
    let p = MemoryPolicy::default_for(MemoryKind::Working);
    println!("tick,score_high_rel,score_low_rel,retain_threshold");
    for tick in 0..1000u64 {
        let age_s = tick as f64;
        let high = p.score(0.95, age_s, 0.95);
        let low = p.score(0.30, age_s, 0.40);
        println!("{tick},{high:.6},{low:.6},{:.2}", p.retention_threshold);
    }
}
