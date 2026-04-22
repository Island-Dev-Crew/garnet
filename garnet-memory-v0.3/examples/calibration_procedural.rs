//! R+R+I calibration probe — Procedural memory (decade-scale persistence).

use garnet_memory::{MemoryKind, MemoryPolicy};

fn main() {
    let p = MemoryPolicy::default_for(MemoryKind::Procedural);
    println!("tick_days,score_high_rel,score_low_rel,retain_threshold");
    for tick_d in 0..3650u64 {
        let age_s = (tick_d * 86_400) as f64;
        let high = p.score(0.95, age_s, 0.95);
        let low = p.score(0.30, age_s, 0.40);
        if tick_d % 30 == 0 {
            // Sample monthly for a 10-year window.
            println!("{tick_d},{high:.6},{low:.6},{:.2}", p.retention_threshold);
        }
    }
}
