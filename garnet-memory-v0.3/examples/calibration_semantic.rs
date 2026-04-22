//! R+R+I calibration probe — Semantic memory (very slow decay over days).

use garnet_memory::{MemoryKind, MemoryPolicy};

fn main() {
    let p = MemoryPolicy::default_for(MemoryKind::Semantic);
    println!("tick_days,score_high_rel,score_low_rel,retain_threshold");
    for tick_d in 0..365u64 {
        let age_s = (tick_d * 86_400) as f64;
        let high = p.score(0.95, age_s, 0.95);
        let low = p.score(0.30, age_s, 0.40);
        println!("{tick_d},{high:.6},{low:.6},{:.2}", p.retention_threshold);
    }
}
