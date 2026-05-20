use garnet_interp::{Interpreter, Value};
use garnet_vm::{
    compile_source, deserialize_program, run_function_with_options, serialize_program, RunOptions,
};

const MVP_01: &str = include_str!("../../examples/mvp_01_os_simulator.garnet");
const MVP_02: &str = include_str!("../../examples/mvp_02_relational_db.garnet");
const MVP_03: &str = include_str!("../../examples/mvp_03_compiler_bootstrap.garnet");
const MVP_04: &str = include_str!("../../examples/mvp_04_numerical_solver.garnet");
const MVP_05: &str = include_str!("../../examples/mvp_05_web_app.garnet");

#[test]
fn serializer_round_trips_deterministically() {
    let artifact = compile_source("def main() { 1 + 2 * 3 }").unwrap();
    let first = serialize_program(&artifact.program);
    let second = serialize_program(&artifact.program);
    assert_eq!(first, second);
    let decoded = deserialize_program(&first).unwrap();
    assert_eq!(artifact.program, decoded);
}

#[test]
fn executes_native_arithmetic_and_calls() {
    let src = r#"
        def add(a, b) { a + b }
        def main() { add(7, 8) * 3 - 5 }
    "#;
    let result = run_function_with_options(src, "main", vec![], quiet()).unwrap();
    assert_eq!("40", result.value.display());
    assert_eq!(0, result.summary.fallback_function_calls);
    assert!(result.summary.native_function_calls >= 2);
}

#[test]
fn executes_native_while_loop() {
    let result =
        run_function_with_options(MVP_01, "run_scheduler", vec![Value::Int(12)], quiet()).unwrap();
    assert_eq!("9", result.value.display());
    assert_eq!(0, result.summary.fallback_function_calls);
}

#[test]
fn executes_native_for_loop_over_arrays() {
    let rows = Value::array(vec![
        Value::Int(1),
        Value::Int(0),
        Value::Int(1),
        Value::Int(1),
        Value::Int(0),
        Value::Int(1),
    ]);
    let result = run_function_with_options(MVP_02, "query_score", vec![rows], quiet()).unwrap();
    assert_eq!("46", result.value.display());
    assert_eq!(0, result.summary.fallback_function_calls);
}

#[test]
fn mvp_01_through_05_core_functions_match_tree_walk_results() {
    let cases = [
        ("mvp_01", MVP_01, "run_scheduler", vec![Value::Int(12)]),
        (
            "mvp_02",
            MVP_02,
            "query_score",
            vec![Value::array(vec![
                Value::Int(1),
                Value::Int(0),
                Value::Int(1),
                Value::Int(1),
                Value::Int(0),
                Value::Int(1),
            ])],
        ),
        ("mvp_03", MVP_03, "eval_program", vec![]),
        (
            "mvp_04",
            MVP_04,
            "converge",
            vec![Value::Int(0), Value::Int(11), Value::Int(7)],
        ),
        (
            "mvp_05",
            MVP_05,
            "dispatch_score",
            vec![Value::array(vec![
                Value::str("/"),
                Value::str("/health"),
                Value::str("/users"),
                Value::str("/missing"),
            ])],
        ),
    ];

    for (label, src, entry, args) in cases {
        let vm = run_function_with_options(src, entry, args.clone(), quiet())
            .unwrap_or_else(|error| panic!("{label} VM failed: {error}"));
        let mut interp = Interpreter::new();
        interp.load_source(src).unwrap();
        let tree = interp
            .call(entry, args)
            .unwrap_or_else(|error| panic!("{label} interp failed: {error}"));
        assert!(
            vm.value.eq_deep(&tree),
            "{label} mismatch: VM={} tree={}",
            vm.value.display(),
            tree.display()
        );
    }
}

#[test]
fn unsupported_closure_records_fallback_boundary() {
    let src = "def main() { [1, 2, 3].map(|x| x) }";
    let artifact = compile_source(src).unwrap();
    assert_eq!(0, artifact.summary.native_functions);
    assert_eq!(1, artifact.summary.fallback_functions);
    assert!(
        artifact.summary.fallback_reasons[0].contains("main"),
        "{:?}",
        artifact.summary.fallback_reasons
    );
}

fn quiet() -> RunOptions {
    RunOptions { emit_stdout: false }
}
