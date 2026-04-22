//! R+R+I calibration probe — Episodic memory (slow decay over hours).

use garnet_memory::{MemoryKind, MemoryPolicy};

fn main() {
    let p = MemoryPolicy::default_for(MemoryKind::Episodic);
    println!("tick_hours,score_high_rel,score_low_rel,retain_threshold");
    for tick_h in 0..240u64 {
        let age_s = (tick_h * 3600) as f64;
        let high = p.score(0.95, age_s, 0.95);
        let low = p.score(0.30, age_s, 0.40);
        println!("{tick_h},{high:.6},{low:.6},{:.2}", p.retention_threshold);
    }
}
