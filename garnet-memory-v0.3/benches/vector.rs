//! Criterion benchmarks for the memory primitives.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use garnet_memory::*;

fn bench_vector_index(c: &mut Criterion) {
    for &size in &[1_000usize, 10_000, 100_000] {
        let idx: VectorIndex<usize> = VectorIndex::new();
        for i in 0..size {
            let v = vec![(i as f32 / size as f32), (1.0 - i as f32 / size as f32)];
            idx.insert(v, i);
        }
        c.bench_function(&format!("vector_index_search_top10_at_{size}"), |b| {
            b.iter(|| {
                let r = idx.search(&black_box([0.5_f32, 0.5_f32]), 10);
                black_box(r)
            })
        });
    }
}

fn bench_episode_append(c: &mut Criterion) {
    c.bench_function("episode_append_10000", |b| {
        b.iter(|| {
            let s: EpisodeStore<i32> = EpisodeStore::new();
            for i in 0..10_000 {
                s.append_at(i, i as i32);
            }
            black_box(s.len())
        })
    });
}

fn bench_episode_recent(c: &mut Criterion) {
    let s: EpisodeStore<i32> = EpisodeStore::new();
    for i in 0..100_000 {
        s.append_at(i, i as i32);
    }
    c.bench_function("episode_recent_50_at_100k", |b| {
        b.iter(|| {
            let r = s.recent(50);
            black_box(r)
        })
    });
}

fn bench_working_push(c: &mut Criterion) {
    c.bench_function("working_push_50000", |b| {
        b.iter(|| {
            let s: WorkingStore<i32> = WorkingStore::new();
            for i in 0..50_000 {
                s.push(i);
            }
            black_box(s.len())
        })
    });
}

criterion_group!(
    benches,
    bench_vector_index,
    bench_episode_append,
    bench_episode_recent,
    bench_working_push
);
criterion_main!(benches);
