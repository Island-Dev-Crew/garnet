//! Criterion bench: CST path (`garnet_cst::parse_cst`) vs AST path
//! (`garnet_parser::parse_source`) over the canonical `mvp_*` examples.
//!
//! S15 target: CST path ≤ 1.5× the AST path. If slower, ship anyway and record
//! the ratio in CHANGELOG (per PRD A §7 — perf optimization is v0.8 work).

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use std::path::Path;

fn mvp_examples() -> Vec<String> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("examples");
    let mut srcs = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        let mut paths: Vec<_> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("mvp_") && n.ends_with(".garnet"))
                    .unwrap_or(false)
            })
            .collect();
        paths.sort();
        for p in paths {
            if let Ok(s) = fs::read_to_string(&p) {
                srcs.push(s);
            }
        }
    }
    srcs
}

fn bench(c: &mut Criterion) {
    let examples = mvp_examples();
    assert!(!examples.is_empty(), "mvp_* examples should exist");

    let mut group = c.benchmark_group("parse_cst_vs_ast");
    group.bench_function("ast_parse_source", |b| {
        b.iter(|| {
            for src in &examples {
                let _ = garnet_parser::parse_source(black_box(src));
            }
        });
    });
    group.bench_function("cst_parse_cst", |b| {
        b.iter(|| {
            for src in &examples {
                let _ = garnet_cst::parse_cst(black_box(src));
            }
        });
    });
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
