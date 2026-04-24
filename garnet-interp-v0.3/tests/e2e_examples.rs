//! End-to-end smoke tests that load and run the example programs.
//! If any of these fail, a real .garnet program in the examples/ directory
//! has stopped running — that's a regression any user would see immediately.

use garnet_interp::{Interpreter, Value};

fn load_and_run(path: &str, fn_name: &str) -> Value {
    let src =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
    let mut interp = Interpreter::new();
    interp
        .load_source(&src)
        .unwrap_or_else(|e| panic!("load {path}: {e}"));
    interp
        .call(fn_name, vec![])
        .unwrap_or_else(|e| panic!("call {fn_name}: {e}"))
}

#[test]
fn e2e_paper_vi_walkthrough_runs_to_completion() {
    let r = load_and_run("examples/paper_vi_walkthrough.garnet", "main");
    assert!(matches!(r, Value::Int(42)));
}

#[test]
fn e2e_open_questions_demo_runs_to_completion() {
    let r = load_and_run("examples/open_questions_demo.garnet", "main");
    assert!(matches!(r, Value::Int(0)));
}

#[test]
fn e2e_realistic_program_runs_to_completion() {
    let r = load_and_run("examples/realistic_program.garnet", "main");
    // entry_count (5) * 100 + nesting (4) = 504
    assert!(matches!(r, Value::Int(504)));
}

#[test]
fn e2e_hello_example_round_trip() {
    // Confirms the original v3.0 example still runs after all v3.1 changes.
    let src = include_str!("../examples/hello.garnet");
    let mut interp = Interpreter::new();
    interp.load_source(src).expect("load");
    let r = interp.call("main", vec![]).expect("main");
    // fib(0..9).sum = 0+1+1+2+3+5+8+13+21+34 = 88
    assert!(matches!(r, Value::Int(88)));
}
