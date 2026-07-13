//! Criterion benchmark for the S2 bytecode VM scaffold.
//!
//! Measures parse+compile+execute for the canonical MVP core functions and
//! compares them against parse+load+execute through the tree-walk interpreter.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use garnet_interp::{Interpreter, Value};
use garnet_vm::{compile_source, PreparedVm, RunOptions};
use std::time::Duration;

struct Case {
    label: &'static str,
    source: &'static str,
    entry: &'static str,
    args: fn() -> Vec<Value>,
}

fn cases() -> Vec<Case> {
    vec![
        Case {
            label: "mvp_01_os_simulator",
            source: include_str!("../../examples/mvp_01_os_simulator.garnet"),
            entry: "run_scheduler",
            args: || vec![Value::Int(12)],
        },
        Case {
            label: "mvp_02_relational_db",
            source: include_str!("../../examples/mvp_02_relational_db.garnet"),
            entry: "query_score",
            args: || {
                vec![Value::array(vec![
                    Value::Int(1),
                    Value::Int(0),
                    Value::Int(1),
                    Value::Int(1),
                    Value::Int(0),
                    Value::Int(1),
                ])]
            },
        },
        Case {
            label: "mvp_03_compiler_bootstrap",
            source: include_str!("../../examples/mvp_03_compiler_bootstrap.garnet"),
            entry: "eval_program",
            args: Vec::new,
        },
        Case {
            label: "mvp_04_numerical_solver",
            source: include_str!("../../examples/mvp_04_numerical_solver.garnet"),
            entry: "converge",
            args: || vec![Value::Int(0), Value::Int(11), Value::Int(7)],
        },
        Case {
            label: "mvp_05_web_app",
            source: include_str!("../../examples/mvp_05_web_app.garnet"),
            entry: "dispatch_score",
            args: || {
                vec![Value::array(vec![
                    Value::str("/"),
                    Value::str("/health"),
                    Value::str("/users"),
                    Value::str("/missing"),
                ])]
            },
        },
        // S14: a function-call-heavy, fully native-lowered program that
        // exercises recursive + mutually-recursive calls through the explicit
        // call-frame stack.
        Case {
            label: "mvp_function_call_demo",
            source: include_str!("../../examples/mvp_function_call_demo.garnet"),
            entry: "main",
            args: Vec::new,
        },
    ]
}

fn bench_parse_compile_execute(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_compile_execute");
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(250));
    group.measurement_time(Duration::from_secs(1));
    for case in cases() {
        let artifact = compile_source(case.source).unwrap();
        let mut vm = PreparedVm::new(&artifact, RunOptions { emit_stdout: false }).unwrap();
        // Trusted timing harness loading example programs unframed: permissive
        // constructor (the opt-out from strict-by-default) so example top-level
        // host authority does not trap here.
        let mut interp = Interpreter::new_permissive();
        interp.load_source(case.source).unwrap();

        group.bench_function(BenchmarkId::new("vm", case.label), |b| {
            b.iter(|| {
                let value = vm
                    .call_function(black_box(case.entry), (case.args)())
                    .unwrap();
                black_box(value)
            })
        });
        group.bench_function(BenchmarkId::new("interp", case.label), |b| {
            b.iter(|| {
                let value = interp.call(black_box(case.entry), (case.args)()).unwrap();
                black_box(value)
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench_parse_compile_execute);
criterion_main!(benches);
