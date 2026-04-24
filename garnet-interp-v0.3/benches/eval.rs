//! Criterion benchmarks for the v0.3 tree-walk interpreter.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use garnet_interp::{Interpreter, Value};

fn fib_program(target: i64) -> String {
    format!(
        r#"
        def fib(n) {{
            if n < 2 {{ n }} else {{ fib(n - 1) + fib(n - 2) }}
        }}
        def main() {{ fib({target}) }}
    "#
    )
}

fn array_program(size: usize) -> String {
    let body: String = (0..size)
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    format!("def main() {{ [{body}].map(|x| x * 2).reduce(0, |a, b| a + b) }}")
}

fn bench_fib_15(c: &mut Criterion) {
    let src = fib_program(15);
    let mut interp = Interpreter::new();
    interp.load_source(&src).unwrap();
    c.bench_function("eval_fib_15", |b| {
        b.iter(|| {
            let v = interp.call("main", vec![]).unwrap();
            black_box(v)
        })
    });
}

fn bench_array_1000(c: &mut Criterion) {
    let src = array_program(1000);
    let mut interp = Interpreter::new();
    interp.load_source(&src).unwrap();
    c.bench_function("eval_array_1000_map_reduce", |b| {
        b.iter(|| {
            let v = interp.call("main", vec![]).unwrap();
            black_box(v)
        })
    });
}

fn bench_eval_expr(c: &mut Criterion) {
    let interp = Interpreter::new();
    c.bench_function("eval_expr_arithmetic", |b| {
        b.iter(|| {
            let r: Value = interp
                .eval_expr_src(black_box("1 + 2 * 3 - 4 / 2 % 3"))
                .unwrap();
            black_box(r)
        })
    });
}

criterion_group!(benches, bench_fib_15, bench_array_1000, bench_eval_expr);
criterion_main!(benches);
