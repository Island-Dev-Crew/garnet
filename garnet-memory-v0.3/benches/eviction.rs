//! S6 — Memory eviction policy benchmarks.
//!
//! Each of the four Mnemos memory kinds (working / episodic / semantic /
//! procedural) gets a measured eviction strategy compared against a naive
//! FIFO baseline. The benchmark exercises the existing
//! `MemoryPolicy::score` + `should_retain` path on synthetic item streams
//! sized to the kind's `compaction_high_water` default; the comparison
//! shows the constant-factor cost of policy-driven eviction vs the
//! simplest possible "drop oldest" strategy.
//!
//! What this DOES measure:
//! - Throughput of `score()` + `should_retain()` over a fixed corpus.
//! - Wall-clock cost of policy-driven retention vs FIFO retention.
//! - Effect of `compaction_high_water` on the comparison (smaller stores
//!   amortize less per item).
//!
//! What this does NOT measure:
//! - End-to-end store throughput (EpisodeStore / VectorIndex / etc.) under
//!   eviction. Those stores already have their own bench targets in
//!   `garnet-memory-v0.3/benches/vector.rs`; cross-cutting eviction benches
//!   live here so each kind's policy contract is visible in one place.
//! - Memory footprint, allocation count, or GC overhead. Criterion times
//!   wall-clock only.
//! - Production allocator behaviour. The Memory Core roadmap's Tier 1
//!   production allocator path is separate work tracked in
//!   `C_Language_Specification/MEMORY_CORE_ROADMAP.md`.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use garnet_memory::policy::{MemoryKind, MemoryPolicy};

/// Synthetic item: relevance, age in seconds, importance. Realistic enough
/// to exercise the R+R+I scoring path without coupling the bench to a
/// specific Item struct.
#[derive(Clone, Copy)]
struct ScoredItem {
    relevance: f64,
    age_seconds: f64,
    importance: f64,
}

fn make_corpus(size: usize) -> Vec<ScoredItem> {
    (0..size)
        .map(|i| {
            // Deterministic distribution: every 7th item is "important",
            // every 11th is "stale", relevance walks across [0.1, 0.9].
            let relevance = 0.1 + 0.8 * ((i % 9) as f64 / 9.0);
            let age_seconds = if i % 11 == 0 {
                86_400.0 // a day old
            } else {
                60.0 + (i as f64).rem_euclid(3600.0)
            };
            let importance = if i % 7 == 0 { 0.9 } else { 0.4 };
            ScoredItem {
                relevance,
                age_seconds,
                importance,
            }
        })
        .collect()
}

/// Naive baseline: drop the oldest items until the corpus is at or below
/// the kind's high-water mark. Pure FIFO, no scoring.
fn evict_naive_fifo(items: &[ScoredItem], high_water: usize) -> usize {
    if items.len() <= high_water {
        return items.len();
    }
    // The "naive" baseline does no per-item work — it just clamps the
    // tail. We touch each item once so the comparison reflects the
    // same memory-traversal cost as the policy variant.
    let mut retained = 0usize;
    for (idx, _) in items.iter().enumerate() {
        if idx < high_water {
            retained += 1;
        }
    }
    retained
}

/// Policy-driven eviction: score every item, retain only those passing
/// `should_retain`. This is the path the four stores route through today
/// via `MemoryPolicy::score` + `should_retain`.
fn evict_policy_driven(items: &[ScoredItem], policy: &MemoryPolicy) -> usize {
    let mut retained = 0usize;
    for it in items {
        let s = policy.score(it.relevance, it.age_seconds, it.importance);
        if policy.should_retain(s) {
            retained += 1;
        }
    }
    retained
}

fn kind_label(kind: MemoryKind) -> &'static str {
    match kind {
        MemoryKind::Working => "working",
        MemoryKind::Episodic => "episodic",
        MemoryKind::Semantic => "semantic",
        MemoryKind::Procedural => "procedural",
    }
}

fn bench_kind(c: &mut Criterion, kind: MemoryKind, corpus_size: usize) {
    let label = kind_label(kind);
    let policy = MemoryPolicy::default_for(kind);
    let high_water = policy.compaction_high_water;
    let items = make_corpus(corpus_size);

    let mut group = c.benchmark_group(format!("eviction/{label}"));
    group.bench_with_input(
        BenchmarkId::new("naive_fifo", corpus_size),
        &items,
        |b, items| {
            b.iter(|| {
                let retained = evict_naive_fifo(black_box(items), black_box(high_water));
                black_box(retained)
            })
        },
    );
    group.bench_with_input(
        BenchmarkId::new("policy_score", corpus_size),
        &items,
        |b, items| {
            b.iter(|| {
                let retained = evict_policy_driven(black_box(items), black_box(&policy));
                black_box(retained)
            })
        },
    );
    group.finish();
}

fn bench_working(c: &mut Criterion) {
    // Working memory is bounded small (default high_water = 1024); pick a
    // corpus 4× larger to exercise the eviction decision per item.
    bench_kind(c, MemoryKind::Working, 4_096);
}

fn bench_episodic(c: &mut Criterion) {
    // Episodic memory is the long-tail store. Use 10k items — enough to
    // expose per-item cost without making the bench slow.
    bench_kind(c, MemoryKind::Episodic, 10_000);
}

fn bench_semantic(c: &mut Criterion) {
    // Semantic memory has a very high water mark; 20k items exposes the
    // policy.score cost without simulating the full default 1M.
    bench_kind(c, MemoryKind::Semantic, 20_000);
}

fn bench_procedural(c: &mut Criterion) {
    // Procedural memory is small-but-sticky; 8k items at default 10k high
    // water means the naive baseline barely evicts while the policy still
    // exercises scoring on every item.
    bench_kind(c, MemoryKind::Procedural, 8_000);
}

criterion_group!(
    benches,
    bench_working,
    bench_episodic,
    bench_semantic,
    bench_procedural,
);
criterion_main!(benches);
