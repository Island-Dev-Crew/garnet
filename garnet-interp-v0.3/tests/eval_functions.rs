//! Function definition, invocation, closure capture, recursion.

use garnet_interp::{Interpreter, Value};

fn run(src: &str, main: &str) -> Value {
    let mut interp = Interpreter::new();
    interp.load_source(src).expect("load");
    interp.call(main, vec![]).expect("call")
}

#[test]
fn simple_function_def() {
    assert!(matches!(
        run("def answer() { 42 }", "answer"),
        Value::Int(42)
    ));
}

#[test]
fn function_with_args() {
    let mut interp = Interpreter::new();
    interp.load_source("def add(x, y) { x + y }").unwrap();
    let result = interp
        .call("add", vec![Value::Int(3), Value::Int(4)])
        .unwrap();
    assert!(matches!(result, Value::Int(7)));
}

#[test]
fn function_with_local_let() {
    let src = r#"
        def compute() {
            let x = 10
            let y = 20
            x + y
        }
    "#;
    assert!(matches!(run(src, "compute"), Value::Int(30)));
}

#[test]
fn function_early_return() {
    let src = r#"
        def classify(x) {
            if x > 0 {
                return "positive"
            }
            if x < 0 {
                return "negative"
            }
            "zero"
        }
    "#;
    let mut interp = Interpreter::new();
    interp.load_source(src).unwrap();
    let pos = interp.call("classify", vec![Value::Int(5)]).unwrap();
    let neg = interp.call("classify", vec![Value::Int(-5)]).unwrap();
    let zero = interp.call("classify", vec![Value::Int(0)]).unwrap();
    match (&pos, &neg, &zero) {
        (Value::Str(a), Value::Str(b), Value::Str(c)) => {
            assert_eq!(a.as_str(), "positive");
            assert_eq!(b.as_str(), "negative");
            assert_eq!(c.as_str(), "zero");
        }
        _ => panic!("expected string results"),
    }
}

#[test]
fn recursion_fibonacci() {
    let src = r#"
        def fib(n) {
            if n < 2 {
                n
            } else {
                fib(n - 1) + fib(n - 2)
            }
        }
    "#;
    let mut interp = Interpreter::new();
    interp.load_source(src).unwrap();
    let r = interp.call("fib", vec![Value::Int(10)]).unwrap();
    assert!(matches!(r, Value::Int(55)));
}

#[test]
fn closure_captures_environment() {
    let src = r#"
        def make_adder(n) {
            |x| x + n
        }
    "#;
    let mut interp = Interpreter::new();
    interp.load_source(src).unwrap();
    let adder = interp.call("make_adder", vec![Value::Int(7)]).unwrap();
    let result = garnet_interp::eval::call_value(&adder, vec![Value::Int(3)]).unwrap();
    assert!(matches!(result, Value::Int(10)));
}

#[test]
fn pipeline_operator() {
    let src = r#"
        def double(x) { x * 2 }
        def incr(x) { x + 1 }
        def pipe() { 5 |> double |> incr }
    "#;
    assert!(matches!(run(src, "pipe"), Value::Int(11)));
}
