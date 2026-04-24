//! Basic interpreter smoke tests — arithmetic, booleans, strings, control flow.

use garnet_interp::{Interpreter, Value};

fn eval(src: &str) -> Value {
    let interp = Interpreter::new();
    interp.eval_expr_src(src).expect("eval failed")
}

#[test]
fn arithmetic_add() {
    assert!(matches!(eval("1 + 2"), Value::Int(3)));
}

#[test]
fn arithmetic_precedence() {
    assert!(matches!(eval("1 + 2 * 3"), Value::Int(7)));
}

#[test]
fn arithmetic_parens() {
    assert!(matches!(eval("(1 + 2) * 3"), Value::Int(9)));
}

#[test]
fn arithmetic_division() {
    assert!(matches!(eval("10 / 3"), Value::Int(3)));
}

#[test]
fn arithmetic_modulo() {
    assert!(matches!(eval("10 % 3"), Value::Int(1)));
}

#[test]
fn float_math() {
    match eval("3.5 + 2.0") {
        Value::Float(v) => assert!((v - 5.5).abs() < 1e-9),
        other => panic!("expected float, got {other:?}"),
    }
}

#[test]
fn mixed_int_float_promotes() {
    match eval("1 + 2.5") {
        Value::Float(v) => assert!((v - 3.5).abs() < 1e-9),
        other => panic!("expected float, got {other:?}"),
    }
}

#[test]
fn boolean_and_short_circuit() {
    assert!(matches!(eval("true and true"), Value::Bool(true)));
    assert!(matches!(eval("true and false"), Value::Bool(false)));
    assert!(matches!(
        eval("false and any_undefined_name"),
        Value::Bool(false)
    ));
}

#[test]
fn boolean_or_short_circuit() {
    assert!(matches!(
        eval("true or any_undefined_name"),
        Value::Bool(true)
    ));
    assert!(matches!(eval("false or true"), Value::Bool(true)));
}

#[test]
fn boolean_not() {
    assert!(matches!(eval("not false"), Value::Bool(true)));
    assert!(matches!(eval("not nil"), Value::Bool(true)));
    assert!(matches!(eval("not 0"), Value::Bool(false)));
}

#[test]
fn comparison_ops() {
    assert!(matches!(eval("1 < 2"), Value::Bool(true)));
    assert!(matches!(eval("2 >= 2"), Value::Bool(true)));
    assert!(matches!(eval("\"a\" < \"b\""), Value::Bool(true)));
}

#[test]
fn equality_deep() {
    assert!(matches!(eval("[1, 2, 3] == [1, 2, 3]"), Value::Bool(true)));
    assert!(matches!(eval("[1, 2] == [1, 2, 3]"), Value::Bool(false)));
}

#[test]
fn if_else_expression() {
    assert!(matches!(eval("if true { 42 } else { 0 }"), Value::Int(42)));
    assert!(matches!(eval("if false { 42 } else { 0 }"), Value::Int(0)));
}

#[test]
fn if_elsif_else() {
    let src = r#"if 5 > 10 { "big" } elsif 5 > 3 { "mid" } else { "small" }"#;
    match eval(src) {
        Value::Str(s) => assert_eq!(s.as_str(), "mid"),
        other => panic!("expected string, got {other:?}"),
    }
}

#[test]
fn string_interpolation() {
    let mut interp = Interpreter::new();
    interp
        .load_source("def main() { let name = \"Jon\"\n\"hello, #{name}\" }")
        .unwrap();
    match interp.call("main", vec![]).unwrap() {
        Value::Str(s) => assert_eq!(s.as_str(), "hello, Jon"),
        other => panic!("expected string, got {other:?}"),
    }
}

#[test]
fn range_materializes_to_int_list() {
    match eval("0..3") {
        Value::Range {
            start,
            end,
            inclusive,
        } => {
            assert_eq!(start, 0);
            assert_eq!(end, 3);
            assert!(!inclusive);
        }
        other => panic!("expected range, got {other:?}"),
    }
}

#[test]
fn range_inclusive() {
    match eval("1...5") {
        Value::Range { inclusive, .. } => assert!(inclusive),
        other => panic!("expected range, got {other:?}"),
    }
}

#[test]
fn unary_minus() {
    assert!(matches!(eval("-42"), Value::Int(-42)));
}
