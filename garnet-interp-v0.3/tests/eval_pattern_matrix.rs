//! Pattern × value matrix — exhaustively covers every supported (pattern,
//! value) combination so any change to `pattern.rs` or `eval.rs` that
//! breaks matching semantics fails immediately.

use garnet_interp::{Interpreter, Value};

fn run(src: &str, fn_name: &str, args: Vec<Value>) -> Value {
    let mut interp = Interpreter::new();
    interp.load_source(src).expect("load");
    interp.call(fn_name, args).expect("call")
}

// ════════════════════════════════════════════════════════════════════
// Wildcard pattern — accepts any value
// ════════════════════════════════════════════════════════════════════

const WILDCARD: &str = r#"def m(x) { match x { _ => :ok } }"#;

#[test]
fn wildcard_matches_int() {
    if let Value::Symbol(s) = run(WILDCARD, "m", vec![Value::Int(42)]) {
        assert_eq!(s.as_str(), "ok");
    }
}

#[test]
fn wildcard_matches_float() {
    if let Value::Symbol(s) = run(WILDCARD, "m", vec![Value::Float(1.5)]) {
        assert_eq!(s.as_str(), "ok");
    }
}

#[test]
fn wildcard_matches_string() {
    if let Value::Symbol(s) = run(WILDCARD, "m", vec![Value::str("x")]) {
        assert_eq!(s.as_str(), "ok");
    }
}

#[test]
fn wildcard_matches_bool() {
    if let Value::Symbol(s) = run(WILDCARD, "m", vec![Value::Bool(true)]) {
        assert_eq!(s.as_str(), "ok");
    }
}

#[test]
fn wildcard_matches_nil() {
    if let Value::Symbol(s) = run(WILDCARD, "m", vec![Value::Nil]) {
        assert_eq!(s.as_str(), "ok");
    }
}

#[test]
fn wildcard_matches_symbol() {
    if let Value::Symbol(s) = run(WILDCARD, "m", vec![Value::sym("anything")]) {
        assert_eq!(s.as_str(), "ok");
    }
}

#[test]
fn wildcard_matches_array() {
    if let Value::Symbol(s) = run(WILDCARD, "m", vec![Value::array(vec![])]) {
        assert_eq!(s.as_str(), "ok");
    }
}

#[test]
fn wildcard_matches_map() {
    if let Value::Symbol(s) = run(WILDCARD, "m", vec![Value::map(vec![])]) {
        assert_eq!(s.as_str(), "ok");
    }
}

// ════════════════════════════════════════════════════════════════════
// Identifier pattern — binds and matches any value
// ════════════════════════════════════════════════════════════════════

const IDENT_BIND: &str = r#"def m(x) { match x { y => y } }"#;

#[test]
fn ident_binds_int() {
    assert!(matches!(
        run(IDENT_BIND, "m", vec![Value::Int(7)]),
        Value::Int(7)
    ));
}

#[test]
fn ident_binds_string() {
    if let Value::Str(s) = run(IDENT_BIND, "m", vec![Value::str("hi")]) {
        assert_eq!(s.as_str(), "hi");
    }
}

#[test]
fn ident_binds_array() {
    if let Value::Array(a) = run(
        IDENT_BIND,
        "m",
        vec![Value::array(vec![Value::Int(1), Value::Int(2)])],
    ) {
        assert_eq!(a.borrow().len(), 2);
    }
}

#[test]
fn ident_binds_nil() {
    assert!(matches!(run(IDENT_BIND, "m", vec![Value::Nil]), Value::Nil));
}

// ════════════════════════════════════════════════════════════════════
// Literal int patterns
// ════════════════════════════════════════════════════════════════════

const INT_LIT: &str =
    r#"def m(x) { match x { 0 => :zero, 1 => :one, 42 => :fortytwo, _ => :other } }"#;

#[test]
fn int_lit_matches_zero() {
    if let Value::Symbol(s) = run(INT_LIT, "m", vec![Value::Int(0)]) {
        assert_eq!(s.as_str(), "zero");
    }
}

#[test]
fn int_lit_matches_one() {
    if let Value::Symbol(s) = run(INT_LIT, "m", vec![Value::Int(1)]) {
        assert_eq!(s.as_str(), "one");
    }
}

#[test]
fn int_lit_matches_42() {
    if let Value::Symbol(s) = run(INT_LIT, "m", vec![Value::Int(42)]) {
        assert_eq!(s.as_str(), "fortytwo");
    }
}

#[test]
fn int_lit_falls_through_to_wildcard() {
    if let Value::Symbol(s) = run(INT_LIT, "m", vec![Value::Int(99)]) {
        assert_eq!(s.as_str(), "other");
    }
}

#[test]
fn int_lit_does_not_match_string() {
    if let Value::Symbol(s) = run(INT_LIT, "m", vec![Value::str("0")]) {
        assert_eq!(s.as_str(), "other");
    }
}

// ════════════════════════════════════════════════════════════════════
// Literal string patterns
// ════════════════════════════════════════════════════════════════════

const STR_LIT: &str = r#"def m(x) { match x { "a" => 1, "b" => 2, _ => 0 } }"#;

#[test]
fn str_lit_matches_a() {
    assert!(matches!(
        run(STR_LIT, "m", vec![Value::str("a")]),
        Value::Int(1)
    ));
}

#[test]
fn str_lit_matches_b() {
    assert!(matches!(
        run(STR_LIT, "m", vec![Value::str("b")]),
        Value::Int(2)
    ));
}

#[test]
fn str_lit_falls_through() {
    assert!(matches!(
        run(STR_LIT, "m", vec![Value::str("c")]),
        Value::Int(0)
    ));
}

#[test]
fn str_lit_empty_falls_through() {
    assert!(matches!(
        run(STR_LIT, "m", vec![Value::str("")]),
        Value::Int(0)
    ));
}

// ════════════════════════════════════════════════════════════════════
// Literal symbol patterns
// ════════════════════════════════════════════════════════════════════

const SYM_LIT: &str = r#"def m(x) { match x { :red => 1, :green => 2, :blue => 3, _ => 0 } }"#;

#[test]
fn sym_lit_red() {
    assert!(matches!(
        run(SYM_LIT, "m", vec![Value::sym("red")]),
        Value::Int(1)
    ));
}

#[test]
fn sym_lit_green() {
    assert!(matches!(
        run(SYM_LIT, "m", vec![Value::sym("green")]),
        Value::Int(2)
    ));
}

#[test]
fn sym_lit_blue() {
    assert!(matches!(
        run(SYM_LIT, "m", vec![Value::sym("blue")]),
        Value::Int(3)
    ));
}

#[test]
fn sym_lit_unknown_falls_through() {
    assert!(matches!(
        run(SYM_LIT, "m", vec![Value::sym("yellow")]),
        Value::Int(0)
    ));
}

// ════════════════════════════════════════════════════════════════════
// Literal bool patterns
// ════════════════════════════════════════════════════════════════════

const BOOL_LIT: &str = r#"def m(x) { match x { true => 1, false => 0, _ => -1 } }"#;

#[test]
fn bool_lit_true() {
    assert!(matches!(
        run(BOOL_LIT, "m", vec![Value::Bool(true)]),
        Value::Int(1)
    ));
}

#[test]
fn bool_lit_false() {
    assert!(matches!(
        run(BOOL_LIT, "m", vec![Value::Bool(false)]),
        Value::Int(0)
    ));
}

// ════════════════════════════════════════════════════════════════════
// Literal nil pattern
// ════════════════════════════════════════════════════════════════════

const NIL_LIT: &str = r#"def m(x) { match x { nil => 1, _ => 0 } }"#;

#[test]
fn nil_lit_matches_nil() {
    assert!(matches!(run(NIL_LIT, "m", vec![Value::Nil]), Value::Int(1)));
}

#[test]
fn nil_lit_does_not_match_zero() {
    assert!(matches!(
        run(NIL_LIT, "m", vec![Value::Int(0)]),
        Value::Int(0)
    ));
}

#[test]
fn nil_lit_does_not_match_false() {
    assert!(matches!(
        run(NIL_LIT, "m", vec![Value::Bool(false)]),
        Value::Int(0)
    ));
}

// ════════════════════════════════════════════════════════════════════
// Enum pattern with bound field
// ════════════════════════════════════════════════════════════════════

const ENUM_BIND: &str = r#"def m(x) { match x { Ok(v) => v, Err(_) => -1 } }"#;

#[test]
fn enum_pattern_ok_binds_value() {
    let mut interp = Interpreter::new();
    interp.load_source(ENUM_BIND).unwrap();
    let ok = interp.call("ok", vec![Value::Int(99)]).unwrap();
    let r = interp.call("m", vec![ok]).unwrap();
    assert!(matches!(r, Value::Int(99)));
}

#[test]
fn enum_pattern_err_uses_wildcard_for_payload() {
    let mut interp = Interpreter::new();
    interp.load_source(ENUM_BIND).unwrap();
    let err = interp.call("err", vec![Value::str("boom")]).unwrap();
    let r = interp.call("m", vec![err]).unwrap();
    assert!(matches!(r, Value::Int(-1)));
}

#[test]
fn nested_enum_pattern_binds_inner() {
    let src = r#"
        def m(x) {
            match x {
                Ok(Some(v)) => v,
                Ok(None) => -1,
                Err(_) => -2,
            }
        }
    "#;
    let mut interp = Interpreter::new();
    interp.load_source(src).unwrap();

    let inner = interp.call("some", vec![Value::Int(7)]).unwrap();
    let wrapped = interp.call("ok", vec![inner]).unwrap();
    assert!(matches!(
        interp.call("m", vec![wrapped]).unwrap(),
        Value::Int(7)
    ));

    let none = interp.call("none", vec![]).unwrap();
    let wrapped = interp.call("ok", vec![none]).unwrap();
    assert!(matches!(
        interp.call("m", vec![wrapped]).unwrap(),
        Value::Int(-1)
    ));
}

// ════════════════════════════════════════════════════════════════════
// Guards alter behavior even when literal pattern matches
// ════════════════════════════════════════════════════════════════════

#[test]
fn guard_blocks_arm_when_false() {
    let src = r#"
        def m(x) {
            match x {
                n if n > 10 => :big,
                n => :small,
            }
        }
    "#;
    if let Value::Symbol(s) = run(src, "m", vec![Value::Int(5)]) {
        assert_eq!(s.as_str(), "small");
    }
}

#[test]
fn guard_allows_arm_when_true() {
    let src = r#"
        def m(x) {
            match x {
                n if n > 10 => :big,
                n => :small,
            }
        }
    "#;
    if let Value::Symbol(s) = run(src, "m", vec![Value::Int(50)]) {
        assert_eq!(s.as_str(), "big");
    }
}

#[test]
fn guard_with_compound_predicate() {
    let src = r#"
        def m(x) {
            match x {
                n if n > 0 and n < 10 => :single_digit,
                n if n >= 10 and n < 100 => :double_digit,
                _ => :bigger,
            }
        }
    "#;
    if let Value::Symbol(s) = run(src, "m", vec![Value::Int(5)]) {
        assert_eq!(s.as_str(), "single_digit");
    }
    if let Value::Symbol(s) = run(src, "m", vec![Value::Int(50)]) {
        assert_eq!(s.as_str(), "double_digit");
    }
    if let Value::Symbol(s) = run(src, "m", vec![Value::Int(500)]) {
        assert_eq!(s.as_str(), "bigger");
    }
}
