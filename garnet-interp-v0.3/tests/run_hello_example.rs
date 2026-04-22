//! End-to-end test: parse, load, and run examples/hello.garnet.

use garnet_interp::{Interpreter, Value};

#[test]
fn hello_example_runs_to_completion() {
    let src = include_str!("../examples/hello.garnet");
    let mut interp = Interpreter::new();
    interp.load_source(src).expect("load hello.garnet");
    // fib(0..=9).sum() = 0+1+1+2+3+5+8+13+21+34 = 88
    let result = interp.call("main", vec![]).expect("main");
    assert!(matches!(result, Value::Int(88)));
}
