//! Array and Map operations.

use garnet_interp::{Interpreter, Value};

fn run(src: &str, main: &str) -> Value {
    let mut interp = Interpreter::new();
    interp.load_source(src).expect("load");
    interp.call(main, vec![]).expect("call")
}

#[test]
fn array_literal() {
    match run("def main() { [1, 2, 3] }", "main") {
        Value::Array(a) => {
            let v = a.borrow();
            assert_eq!(v.len(), 3);
        }
        other => panic!("expected array, got {other:?}"),
    }
}

#[test]
fn array_index() {
    assert!(matches!(run("def main() { [10, 20, 30][1] }", "main"), Value::Int(20)));
}

#[test]
fn array_negative_index() {
    assert!(matches!(run("def main() { [10, 20, 30][-1] }", "main"), Value::Int(30)));
}

#[test]
fn array_len_method() {
    assert!(matches!(run("def main() { [1, 2, 3, 4].len() }", "main"), Value::Int(4)));
}

#[test]
fn array_map_method() {
    let src = r#"
        def main() {
            [1, 2, 3].map(|x| x * 10)
        }
    "#;
    match run(src, "main") {
        Value::Array(a) => {
            let v = a.borrow();
            assert_eq!(v.len(), 3);
            assert!(matches!(v[0], Value::Int(10)));
            assert!(matches!(v[1], Value::Int(20)));
            assert!(matches!(v[2], Value::Int(30)));
        }
        other => panic!("expected array, got {other:?}"),
    }
}

#[test]
fn array_filter_method() {
    let src = r#"
        def main() {
            [1, 2, 3, 4, 5].filter(|x| x > 2)
        }
    "#;
    match run(src, "main") {
        Value::Array(a) => assert_eq!(a.borrow().len(), 3),
        other => panic!("expected array, got {other:?}"),
    }
}

#[test]
fn array_reduce_method() {
    let src = r#"
        def main() {
            [1, 2, 3, 4].reduce(0, |acc, x| acc + x)
        }
    "#;
    assert!(matches!(run(src, "main"), Value::Int(10)));
}

#[test]
fn map_literal_and_lookup() {
    let src = r#"
        def main() {
            let m = { "a" => 1, "b" => 2 }
            m["a"]
        }
    "#;
    assert!(matches!(run(src, "main"), Value::Int(1)));
}

#[test]
fn map_keys_method() {
    let src = r#"
        def main() {
            let m = { "x" => 10, "y" => 20 }
            m.keys().len()
        }
    "#;
    assert!(matches!(run(src, "main"), Value::Int(2)));
}

#[test]
fn string_methods() {
    let src = r#"def main() { "hello".upcase() }"#;
    match run(src, "main") {
        Value::Str(s) => assert_eq!(s.as_str(), "HELLO"),
        other => panic!("expected string, got {other:?}"),
    }
}
