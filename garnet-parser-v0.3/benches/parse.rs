//! Criterion benchmarks for the v0.3 parser. Run with `cargo bench`.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use garnet_parser::{lex_source, parse_source};

const HELLO: &str = include_str!("../examples/greeter_actor.garnet");

fn synthetic_program(size: usize) -> String {
    // Generate `size` chained def declarations to give the parser a real
    // workout. Each is a simple managed function with three statements.
    let mut s = String::new();
    for i in 0..size {
        s.push_str(&format!(
            "def fn_{i}(x) {{\n  let y = x + {i}\n  let z = y * 2\n  z\n}}\n\n",
        ));
    }
    s
}

fn bench_lex(c: &mut Criterion) {
    c.bench_function("lex_hello", |b| b.iter(|| lex_source(black_box(HELLO))));
    let big = synthetic_program(200);
    c.bench_function("lex_200_defs", |b| b.iter(|| lex_source(black_box(&big))));
}

fn bench_parse(c: &mut Criterion) {
    c.bench_function("parse_hello", |b| b.iter(|| parse_source(black_box(HELLO))));
    let big = synthetic_program(200);
    c.bench_function("parse_200_defs", |b| {
        b.iter(|| parse_source(black_box(&big)))
    });
}

criterion_group!(benches, bench_lex, bench_parse);
criterion_main!(benches);
