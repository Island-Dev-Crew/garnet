//! Control-flow, pattern matching, and error handling.

use garnet_interp::{Interpreter, Value};

fn run(src: &str, main: &str) -> Value {
    let mut interp = Interpreter::new();
    interp.load_source(src).expect("load");
    interp.call(main, vec![]).expect("call")
}

#[test]
fn while_loop_with_mutable_var() {
    let src = r#"
        def count_to(n) {
            let mut i = 0
            while i < n {
                i += 1
            }
            i
        }
    "#;
    let mut interp = Interpreter::new();
    interp.load_source(src).unwrap();
    let r = interp.call("count_to", vec![Value::Int(5)]).unwrap();
    assert!(matches!(r, Value::Int(5)));
}

#[test]
fn for_loop_over_array() {
    let src = r#"
        def sum(nums) {
            let mut total = 0
            for n in nums {
                total += n
            }
            total
        }
    "#;
    let mut interp = Interpreter::new();
    interp.load_source(src).unwrap();
    let arr = Value::array(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]);
    let r = interp.call("sum", vec![arr]).unwrap();
    assert!(matches!(r, Value::Int(10)));
}

#[test]
fn for_loop_over_range() {
    let src = r#"
        def sum_n(n) {
            let mut total = 0
            for i in 0..n {
                total += i
            }
            total
        }
    "#;
    let mut interp = Interpreter::new();
    interp.load_source(src).unwrap();
    let r = interp.call("sum_n", vec![Value::Int(5)]).unwrap();
    assert!(matches!(r, Value::Int(10))); // 0+1+2+3+4
}

#[test]
fn break_with_value() {
    let src = r#"
        def find_positive(nums) {
            let mut result = -1
            for n in nums {
                if n > 0 {
                    result = n
                    break
                }
            }
            result
        }
    "#;
    let mut interp = Interpreter::new();
    interp.load_source(src).unwrap();
    let arr = Value::array(vec![Value::Int(-2), Value::Int(-1), Value::Int(3), Value::Int(7)]);
    let r = interp.call("find_positive", vec![arr]).unwrap();
    assert!(matches!(r, Value::Int(3)));
}

#[test]
fn continue_skips_iteration() {
    let src = r#"
        def sum_evens(nums) {
            let mut total = 0
            for n in nums {
                if n % 2 != 0 {
                    continue
                }
                total += n
            }
            total
        }
    "#;
    let mut interp = Interpreter::new();
    interp.load_source(src).unwrap();
    let arr = Value::array(
        (1..=6).map(Value::Int).collect::<Vec<_>>(),
    );
    let r = interp.call("sum_evens", vec![arr]).unwrap();
    assert!(matches!(r, Value::Int(12))); // 2+4+6
}

#[test]
fn loop_with_break_returns() {
    let src = r#"
        def find_first_square_over(limit) {
            let mut n = 1
            loop {
                if n * n > limit {
                    return n
                }
                n += 1
            }
        }
    "#;
    let mut interp = Interpreter::new();
    interp.load_source(src).unwrap();
    let r = interp.call("find_first_square_over", vec![Value::Int(100)]).unwrap();
    assert!(matches!(r, Value::Int(11))); // 11^2 = 121
}

#[test]
fn match_literal_patterns() {
    let src = r#"
        def classify(sym) {
            match sym {
                :red => 1,
                :green => 2,
                :blue => 3,
                _ => 0,
            }
        }
    "#;
    let mut interp = Interpreter::new();
    interp.load_source(src).unwrap();
    let r = interp.call("classify", vec![Value::sym("green")]).unwrap();
    assert!(matches!(r, Value::Int(2)));
    let r2 = interp.call("classify", vec![Value::sym("purple")]).unwrap();
    assert!(matches!(r2, Value::Int(0)));
}

#[test]
fn match_enum_with_binding() {
    let src = r#"
        def describe(result) {
            match result {
                Ok(v) => v,
                Err(e) => -1,
            }
        }
    "#;
    let mut interp = Interpreter::new();
    interp.load_source(src).unwrap();
    let ok = interp.call("ok", vec![Value::Int(42)]).unwrap();
    let r = interp.call("describe", vec![ok]).unwrap();
    assert!(matches!(r, Value::Int(42)));
    let err = interp.call("err", vec![Value::str("bad")]).unwrap();
    let r2 = interp.call("describe", vec![err]).unwrap();
    assert!(matches!(r2, Value::Int(-1)));
}

#[test]
fn match_with_guard() {
    let src = r#"
        def grade(score) {
            match score {
                n if n >= 90 => :a,
                n if n >= 70 => :b,
                _ => :c,
            }
        }
    "#;
    let mut interp = Interpreter::new();
    interp.load_source(src).unwrap();
    let r = interp.call("grade", vec![Value::Int(85)]).unwrap();
    match r {
        Value::Symbol(s) => assert_eq!(s.as_str(), "b"),
        other => panic!("expected :b, got {other:?}"),
    }
}

#[test]
fn try_rescue_catches_message() {
    let src = r#"
        def safe_div(a, b) {
            try {
                a / b
            } rescue e {
                -1
            }
        }
    "#;
    let mut interp = Interpreter::new();
    interp.load_source(src).unwrap();
    let ok = interp.call("safe_div", vec![Value::Int(10), Value::Int(2)]).unwrap();
    assert!(matches!(ok, Value::Int(5)));
    let err = interp.call("safe_div", vec![Value::Int(10), Value::Int(0)]).unwrap();
    assert!(matches!(err, Value::Int(-1)));
}

#[test]
fn try_ensure_always_runs() {
    let src = r#"
        def with_ensure() {
            let mut cleaned_up = 0
            let result = try {
                42
            } ensure {
                cleaned_up = 1
            }
            cleaned_up + result
        }
    "#;
    let r = run(src, "with_ensure");
    assert!(matches!(r, Value::Int(43)));
}

#[test]
fn question_mark_propagates_err() {
    let src = r#"
        def inner() {
            err("failure")
        }
        def outer() {
            try {
                let v = inner()?
                v
            } rescue e {
                :caught
            }
        }
    "#;
    let r = run(src, "outer");
    match r {
        Value::Symbol(s) => assert_eq!(s.as_str(), "caught"),
        other => panic!("expected :caught, got {other:?}"),
    }
}

#[test]
fn raise_and_rescue() {
    let src = r#"
        def handled() {
            try {
                raise "oops"
                42
            } rescue e {
                99
            }
        }
    "#;
    assert!(matches!(run(src, "handled"), Value::Int(99)));
}
