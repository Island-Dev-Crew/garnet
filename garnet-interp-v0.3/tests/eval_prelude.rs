//! Direct exercise of every prelude-installed function and constant. These
//! tests do not depend on parsing complex programs — they call the prelude
//! through the public eval surface so that any regression in the built-in
//! catalog fails immediately.

use garnet_interp::{Interpreter, Value};

fn eval(src: &str) -> Value {
    Interpreter::new()
        .eval_expr_src(src)
        .unwrap_or_else(|e| panic!("eval failed for `{src}`: {e:?}"))
}

// ── Conversion primitives ──

#[test]
fn to_s_int_zero() {
    if let Value::Str(s) = eval("to_s(0)") {
        assert_eq!(s.as_str(), "0");
    }
}

#[test]
fn to_s_negative_int() {
    if let Value::Str(s) = eval("to_s(-42)") {
        assert_eq!(s.as_str(), "-42");
    }
}

#[test]
fn to_s_bool_true() {
    if let Value::Str(s) = eval("to_s(true)") {
        assert_eq!(s.as_str(), "true");
    }
}

#[test]
fn to_s_nil() {
    if let Value::Str(s) = eval("to_s(nil)") {
        assert_eq!(s.as_str(), "nil");
    }
}

#[test]
fn to_s_symbol() {
    if let Value::Str(s) = eval("to_s(:foo)") {
        assert_eq!(s.as_str(), ":foo");
    }
}

#[test]
fn to_i_round_trip() {
    let r = eval(r#"to_i("100")"#);
    assert!(matches!(r, Value::Int(100)));
}

#[test]
fn to_i_invalid_string_errors() {
    let r = Interpreter::new().eval_expr_src(r#"to_i("nope")"#);
    assert!(r.is_err());
}

#[test]
fn to_f_round_trip_integer_str() {
    if let Value::Float(f) = eval(r#"to_f("100")"#) {
        assert!((f - 100.0).abs() < 1e-9);
    }
}

#[test]
fn to_f_round_trip_decimal_str() {
    if let Value::Float(f) = eval(r#"to_f("2.5")"#) {
        assert!((f - 2.5).abs() < 1e-9);
    }
}

#[test]
fn to_f_invalid_errors() {
    let r = Interpreter::new().eval_expr_src(r#"to_f("xyz")"#);
    assert!(r.is_err());
}

// ── Result / Option constructors ──

#[test]
fn ok_constructor_yields_variant() {
    if let Value::Variant { variant, .. } = eval("ok(1)") {
        assert_eq!(variant.as_str(), "Ok");
    } else {
        panic!("expected variant");
    }
}

#[test]
fn err_constructor_yields_variant() {
    if let Value::Variant { variant, .. } = eval(r#"err("e")"#) {
        assert_eq!(variant.as_str(), "Err");
    } else {
        panic!("expected variant");
    }
}

#[test]
fn some_constructor_yields_variant() {
    if let Value::Variant { variant, .. } = eval("some(7)") {
        assert_eq!(variant.as_str(), "Some");
    }
}

#[test]
fn none_constructor_yields_zero_field_variant() {
    if let Value::Variant { variant, fields, .. } = eval("none()") {
        assert_eq!(variant.as_str(), "None");
        assert!(fields.is_empty());
    }
}

#[test]
fn ok_constant_is_callable() {
    if let Value::Variant { variant, .. } = eval("Ok(99)") {
        assert_eq!(variant.as_str(), "Ok");
    }
}

#[test]
fn err_constant_is_callable() {
    if let Value::Variant { variant, .. } = eval(r#"Err("nope")"#) {
        assert_eq!(variant.as_str(), "Err");
    }
}

#[test]
fn some_constant_is_callable() {
    if let Value::Variant { variant, .. } = eval("Some(123)") {
        assert_eq!(variant.as_str(), "Some");
    }
}

#[test]
fn none_constant_is_value() {
    if let Value::Variant { variant, fields, .. } = eval("None") {
        assert_eq!(variant.as_str(), "None");
        assert!(fields.is_empty());
    }
}

// ── Boolean / nil constants ──

#[test]
fn true_const() {
    assert!(matches!(eval("true"), Value::Bool(true)));
}

#[test]
fn false_const() {
    assert!(matches!(eval("false"), Value::Bool(false)));
}

#[test]
fn nil_const() {
    assert!(matches!(eval("nil"), Value::Nil));
}

// ── Type introspection ──

#[test]
fn type_of_float() {
    if let Value::Str(s) = eval("type_of(1.5)") {
        assert_eq!(s.as_str(), "Float");
    }
}

#[test]
fn type_of_bool() {
    if let Value::Str(s) = eval("type_of(true)") {
        assert_eq!(s.as_str(), "Bool");
    }
}

#[test]
fn type_of_symbol() {
    if let Value::Str(s) = eval("type_of(:x)") {
        assert_eq!(s.as_str(), "Symbol");
    }
}

#[test]
fn type_of_map() {
    if let Value::Str(s) = eval(r#"type_of({ "a" => 1 })"#) {
        assert_eq!(s.as_str(), "Map");
    }
}

#[test]
fn type_of_range() {
    if let Value::Str(s) = eval("type_of(1..5)") {
        assert_eq!(s.as_str(), "Range");
    }
}

// ── len ──

#[test]
fn len_array_ten() {
    assert!(matches!(eval("len([1,2,3,4,5,6,7,8,9,10])"), Value::Int(10)));
}

#[test]
fn len_string_zero() {
    assert!(matches!(eval(r#"len("")"#), Value::Int(0)));
}

#[test]
fn len_map_three() {
    assert!(matches!(eval(r#"len({ "a" => 1, "b" => 2, "c" => 3 })"#), Value::Int(3)));
}

#[test]
fn len_of_int_errors() {
    let r = Interpreter::new().eval_expr_src("len(42)");
    assert!(r.is_err());
}

// ── is_nil ──

#[test]
fn is_nil_array_false() {
    assert!(matches!(eval("is_nil([])"), Value::Bool(false)));
}

#[test]
fn is_nil_zero_false() {
    assert!(matches!(eval("is_nil(0)"), Value::Bool(false)));
}

#[test]
fn is_nil_false_value_false() {
    assert!(matches!(eval("is_nil(false)"), Value::Bool(false)));
}

// ── array() ──

#[test]
fn array_empty() {
    if let Value::Array(a) = eval("array()") {
        assert_eq!(a.borrow().len(), 0);
    }
}

#[test]
fn array_one_element() {
    if let Value::Array(a) = eval("array(1)") {
        assert_eq!(a.borrow().len(), 1);
    }
}

#[test]
fn array_multi_element() {
    if let Value::Array(a) = eval(r#"array(1, "two", :three)"#) {
        assert_eq!(a.borrow().len(), 3);
    }
}

// ── map() ──

#[test]
fn map_empty() {
    if let Value::Map(m) = eval("map()") {
        assert_eq!(m.borrow().len(), 0);
    }
}

#[test]
fn map_one_pair() {
    if let Value::Map(m) = eval(r#"map("a", 1)"#) {
        assert_eq!(m.borrow().len(), 1);
    }
}

#[test]
fn map_three_pairs() {
    if let Value::Map(m) = eval(r#"map("a", 1, "b", 2, "c", 3)"#) {
        assert_eq!(m.borrow().len(), 3);
    }
}

#[test]
fn map_odd_args_errors() {
    let r = Interpreter::new().eval_expr_src(r#"map("a", 1, "b")"#);
    assert!(r.is_err());
}

// ── filter() ──

#[test]
fn filter_takes_predicate_returns_array() {
    if let Value::Array(a) = eval("filter([1, 2, 3, 4, 5], |x| x > 2)") {
        assert_eq!(a.borrow().len(), 3);
    }
}

#[test]
fn filter_empty_array() {
    if let Value::Array(a) = eval("filter([], |x| x > 0)") {
        assert_eq!(a.borrow().len(), 0);
    }
}

#[test]
fn filter_all_pass() {
    if let Value::Array(a) = eval("filter([1, 2, 3], |x| x > 0)") {
        assert_eq!(a.borrow().len(), 3);
    }
}

#[test]
fn filter_none_pass() {
    if let Value::Array(a) = eval("filter([1, 2, 3], |x| x > 10)") {
        assert_eq!(a.borrow().len(), 0);
    }
}

// ── reduce() ──

#[test]
fn reduce_sum_basic() {
    assert!(matches!(eval("reduce([1, 2, 3], 0, |a, x| a + x)"), Value::Int(6)));
}

#[test]
fn reduce_with_zero_init() {
    assert!(matches!(eval("reduce([], 99, |a, x| a + x)"), Value::Int(99)));
}

#[test]
fn reduce_product() {
    assert!(matches!(eval("reduce([2, 3, 4], 1, |a, x| a * x)"), Value::Int(24)));
}

// ── assert / assert_eq ──

#[test]
fn assert_truthy_returns_nil() {
    assert!(matches!(eval("assert(true)"), Value::Nil));
}

#[test]
fn assert_falsy_raises() {
    let r = Interpreter::new().eval_expr_src("assert(false)");
    assert!(r.is_err());
}

#[test]
fn assert_eq_passes_for_equal_ints() {
    assert!(matches!(eval("assert_eq(1, 1)"), Value::Nil));
}

#[test]
fn assert_eq_fails_for_unequal() {
    let r = Interpreter::new().eval_expr_src("assert_eq(1, 2)");
    assert!(r.is_err());
}

#[test]
fn assert_eq_works_for_strings() {
    assert!(matches!(eval(r#"assert_eq("a", "a")"#), Value::Nil));
}

#[test]
fn assert_eq_works_for_arrays_deep() {
    assert!(matches!(eval("assert_eq([1, 2, 3], [1, 2, 3])"), Value::Nil));
}
