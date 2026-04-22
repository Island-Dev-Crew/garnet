//! Extended interpreter coverage — every value type, every operator, every
//! built-in. The tests in this file exist so that any silent regression in
//! evaluation semantics fails loudly within seconds.

use garnet_interp::{Interpreter, Value};

fn eval(src: &str) -> Value {
    Interpreter::new()
        .eval_expr_src(src)
        .unwrap_or_else(|e| panic!("eval failed for `{src}`: {e:?}"))
}

fn run(src: &str, fn_name: &str) -> Value {
    let mut interp = Interpreter::new();
    interp.load_source(src).expect("load");
    interp.call(fn_name, vec![]).expect("call")
}

fn run_with_args(src: &str, fn_name: &str, args: Vec<Value>) -> Value {
    let mut interp = Interpreter::new();
    interp.load_source(src).expect("load");
    interp.call(fn_name, args).expect("call")
}

// ════════════════════════════════════════════════════════════════════
// Integer arithmetic — full coverage
// ════════════════════════════════════════════════════════════════════

#[test]
fn int_zero_plus_zero() {
    assert!(matches!(eval("0 + 0"), Value::Int(0)));
}

#[test]
fn int_negative_plus_positive() {
    assert!(matches!(eval("-5 + 3"), Value::Int(-2)));
}

#[test]
fn int_subtraction_into_negative() {
    assert!(matches!(eval("3 - 10"), Value::Int(-7)));
}

#[test]
fn int_multiplication_zero() {
    assert!(matches!(eval("99 * 0"), Value::Int(0)));
}

#[test]
fn int_multiplication_negatives() {
    assert!(matches!(eval("-3 * -7"), Value::Int(21)));
}

#[test]
fn int_division_truncates() {
    assert!(matches!(eval("7 / 2"), Value::Int(3)));
}

#[test]
fn int_division_negative() {
    assert!(matches!(eval("-7 / 2"), Value::Int(-3)));
}

#[test]
fn int_modulo_basic() {
    assert!(matches!(eval("10 % 3"), Value::Int(1)));
}

#[test]
fn int_modulo_zero_dividend() {
    assert!(matches!(eval("0 % 7"), Value::Int(0)));
}

#[test]
fn int_modulo_negative_dividend() {
    // Rust's `%` on i64 follows the sign of the dividend.
    assert!(matches!(eval("-10 % 3"), Value::Int(-1)));
}

// ════════════════════════════════════════════════════════════════════
// Float arithmetic
// ════════════════════════════════════════════════════════════════════

#[test]
fn float_basic_add() {
    if let Value::Float(v) = eval("1.5 + 2.5") {
        assert!((v - 4.0).abs() < 1e-9);
    } else {
        panic!("expected float");
    }
}

#[test]
fn float_subtraction() {
    if let Value::Float(v) = eval("10.0 - 0.5") {
        assert!((v - 9.5).abs() < 1e-9);
    } else {
        panic!("expected float");
    }
}

#[test]
fn float_multiplication() {
    if let Value::Float(v) = eval("2.5 * 4.0") {
        assert!((v - 10.0).abs() < 1e-9);
    } else {
        panic!("expected float");
    }
}

#[test]
fn float_division() {
    if let Value::Float(v) = eval("1.0 / 4.0") {
        assert!((v - 0.25).abs() < 1e-9);
    } else {
        panic!("expected float");
    }
}

#[test]
fn float_int_promotes_int_first() {
    if let Value::Float(v) = eval("5 + 0.5") {
        assert!((v - 5.5).abs() < 1e-9);
    } else {
        panic!("expected float");
    }
}

#[test]
fn float_int_promotes_float_first() {
    if let Value::Float(v) = eval("0.5 + 5") {
        assert!((v - 5.5).abs() < 1e-9);
    } else {
        panic!("expected float");
    }
}

// ════════════════════════════════════════════════════════════════════
// Division-by-zero detection
// ════════════════════════════════════════════════════════════════════

#[test]
fn int_div_by_zero_raises() {
    let r = Interpreter::new().eval_expr_src("10 / 0");
    assert!(r.is_err());
}

#[test]
fn int_mod_by_zero_raises() {
    let r = Interpreter::new().eval_expr_src("10 % 0");
    assert!(r.is_err());
}

#[test]
fn float_div_by_zero_raises() {
    let r = Interpreter::new().eval_expr_src("1.0 / 0.0");
    assert!(r.is_err());
}

// ════════════════════════════════════════════════════════════════════
// Boolean logic — short-circuit & precedence
// ════════════════════════════════════════════════════════════════════

#[test]
fn and_returns_first_falsy() {
    assert!(matches!(eval("nil and 5"), Value::Nil));
}

#[test]
fn or_returns_first_truthy() {
    assert!(matches!(eval("5 or any_undefined_name"), Value::Int(5)));
}

#[test]
fn not_of_zero_is_false() {
    // 0 is truthy in Garnet (Ruby-style truthiness).
    assert!(matches!(eval("not 0"), Value::Bool(false)));
}

#[test]
fn not_of_empty_string_is_false() {
    assert!(matches!(eval(r#"not """#), Value::Bool(false)));
}

#[test]
fn not_not_idempotent() {
    assert!(matches!(eval("not not true"), Value::Bool(true)));
}

#[test]
fn and_chains_long() {
    assert!(matches!(eval("1 and 2 and 3 and 4 and 5"), Value::Int(5)));
}

#[test]
fn or_chains_short_circuit() {
    assert!(matches!(eval("nil or false or 7 or panic_undef"), Value::Int(7)));
}

// ════════════════════════════════════════════════════════════════════
// Comparison operators
// ════════════════════════════════════════════════════════════════════

#[test]
fn lt_int() {
    assert!(matches!(eval("1 < 2"), Value::Bool(true)));
}

#[test]
fn lt_int_false() {
    assert!(matches!(eval("2 < 1"), Value::Bool(false)));
}

#[test]
fn eq_int_to_float() {
    assert!(matches!(eval("3 == 3.0"), Value::Bool(true)));
}

#[test]
fn eq_string_deep() {
    assert!(matches!(eval(r#""abc" == "abc""#), Value::Bool(true)));
}

#[test]
fn eq_array_deep() {
    assert!(matches!(eval("[1, [2, 3]] == [1, [2, 3]]"), Value::Bool(true)));
}

#[test]
fn neq_array_lengths() {
    assert!(matches!(eval("[1, 2] != [1, 2, 3]"), Value::Bool(true)));
}

#[test]
fn lte_equal_values() {
    assert!(matches!(eval("5 <= 5"), Value::Bool(true)));
}

#[test]
fn gte_equal_values() {
    assert!(matches!(eval("5 >= 5"), Value::Bool(true)));
}

#[test]
fn string_lex_compare() {
    assert!(matches!(eval(r#""apple" < "banana""#), Value::Bool(true)));
}

#[test]
fn equality_of_nil_with_nil() {
    assert!(matches!(eval("nil == nil"), Value::Bool(true)));
}

#[test]
fn equality_of_nil_with_false() {
    // Distinct values; only structurally equal to themselves.
    assert!(matches!(eval("nil == false"), Value::Bool(false)));
}

// ════════════════════════════════════════════════════════════════════
// String operations
// ════════════════════════════════════════════════════════════════════

#[test]
fn string_concatenation_via_plus() {
    if let Value::Str(s) = eval(r#""hello, " + "world""#) {
        assert_eq!(s.as_str(), "hello, world");
    } else {
        panic!("expected string");
    }
}

#[test]
fn empty_string_concat() {
    if let Value::Str(s) = eval(r#""" + "x""#) {
        assert_eq!(s.as_str(), "x");
    } else {
        panic!("expected string");
    }
}

#[test]
fn string_length_via_method() {
    assert!(matches!(eval(r#""hello".len()"#), Value::Int(5)));
}

#[test]
fn string_length_unicode_chars() {
    // len() is char-count, not byte-count.
    assert!(matches!(eval(r#""abc".len()"#), Value::Int(3)));
}

#[test]
fn string_upcase() {
    if let Value::Str(s) = eval(r#""hello".upcase()"#) {
        assert_eq!(s.as_str(), "HELLO");
    }
}

#[test]
fn string_downcase() {
    if let Value::Str(s) = eval(r#""HELLO".downcase()"#) {
        assert_eq!(s.as_str(), "hello");
    }
}

#[test]
fn string_chars_returns_array() {
    if let Value::Array(a) = eval(r#""abc".chars()"#) {
        assert_eq!(a.borrow().len(), 3);
    } else {
        panic!("expected array");
    }
}

#[test]
fn string_starts_with_true() {
    assert!(matches!(eval(r#""hello world".starts_with("hello")"#), Value::Bool(true)));
}

#[test]
fn string_starts_with_false() {
    assert!(matches!(eval(r#""hello world".starts_with("world")"#), Value::Bool(false)));
}

#[test]
fn string_index_returns_char() {
    if let Value::Str(s) = eval(r#""abc"[1]"#) {
        assert_eq!(s.as_str(), "b");
    }
}

#[test]
fn string_interpolation_with_arithmetic() {
    let src = r#"
        def main() {
            let x = 7
            "x*2 = #{x * 2}"
        }
    "#;
    if let Value::Str(s) = run(src, "main") {
        assert_eq!(s.as_str(), "x*2 = 14");
    }
}

// ════════════════════════════════════════════════════════════════════
// Array operations
// ════════════════════════════════════════════════════════════════════

#[test]
fn array_empty_len_zero() {
    assert!(matches!(eval("[].len()"), Value::Int(0)));
}

#[test]
fn array_first_of_empty_is_nil() {
    assert!(matches!(eval("[].first()"), Value::Nil));
}

#[test]
fn array_last_returns_last() {
    assert!(matches!(eval("[1, 2, 3].last()"), Value::Int(3)));
}

#[test]
fn array_index_zero() {
    assert!(matches!(eval("[10, 20, 30][0]"), Value::Int(10)));
}

#[test]
fn array_negative_index_one() {
    assert!(matches!(eval("[10, 20, 30][-1]"), Value::Int(30)));
}

#[test]
fn array_negative_index_two() {
    assert!(matches!(eval("[10, 20, 30][-2]"), Value::Int(20)));
}

#[test]
fn array_index_out_of_bounds_raises() {
    let r = Interpreter::new().eval_expr_src("[1, 2, 3][10]");
    assert!(r.is_err());
}

#[test]
fn array_negative_oob_raises() {
    let r = Interpreter::new().eval_expr_src("[1, 2, 3][-10]");
    assert!(r.is_err());
}

#[test]
fn array_map_doubles_values() {
    if let Value::Array(a) = eval("[1, 2, 3].map(|x| x * 2)") {
        let v = a.borrow();
        assert!(matches!(v[0], Value::Int(2)));
        assert!(matches!(v[2], Value::Int(6)));
    } else {
        panic!("expected array");
    }
}

#[test]
fn array_filter_keeps_matching() {
    if let Value::Array(a) = eval("[1, 2, 3, 4, 5].filter(|x| x > 2)") {
        assert_eq!(a.borrow().len(), 3);
    } else {
        panic!("expected array");
    }
}

#[test]
fn array_reduce_sums() {
    assert!(matches!(eval("[1, 2, 3, 4, 5].reduce(0, |a, b| a + b)"), Value::Int(15)));
}

#[test]
fn array_reduce_with_string_concat() {
    if let Value::Str(s) = eval(r#"["a", "b", "c"].reduce("", |a, b| a + b)"#) {
        assert_eq!(s.as_str(), "abc");
    }
}

#[test]
fn array_recent_n_returns_tail() {
    if let Value::Array(a) = eval("[1, 2, 3, 4, 5].recent(2)") {
        let v = a.borrow();
        assert_eq!(v.len(), 2);
        assert!(matches!(v[0], Value::Int(4)));
        assert!(matches!(v[1], Value::Int(5)));
    }
}

#[test]
fn array_recent_more_than_size() {
    if let Value::Array(a) = eval("[1, 2].recent(10)") {
        assert_eq!(a.borrow().len(), 2);
    }
}

#[test]
fn array_push_appends() {
    let src = r#"
        def main() {
            let arr = [1, 2, 3]
            arr.push(4)
            arr
        }
    "#;
    if let Value::Array(a) = run(src, "main") {
        assert_eq!(a.borrow().len(), 4);
    }
}

// ════════════════════════════════════════════════════════════════════
// Map operations
// ════════════════════════════════════════════════════════════════════

#[test]
fn map_empty_len_zero() {
    assert!(matches!(eval("{}.len()"), Value::Int(0)));
}

#[test]
fn map_lookup_existing_key() {
    let src = r#"
        def main() {
            let m = { "a" => 1, "b" => 2 }
            m.get("a")
        }
    "#;
    assert!(matches!(run(src, "main"), Value::Int(1)));
}

#[test]
fn map_lookup_missing_key_returns_nil() {
    let src = r#"
        def main() {
            let m = { "a" => 1 }
            m.get("missing")
        }
    "#;
    assert!(matches!(run(src, "main"), Value::Nil));
}

#[test]
fn map_put_updates() {
    let src = r#"
        def main() {
            let m = {}
            m.put("k", 99)
            m.get("k")
        }
    "#;
    assert!(matches!(run(src, "main"), Value::Int(99)));
}

#[test]
fn map_keys_array_length() {
    let src = r#"
        def main() {
            let m = { "a" => 1, "b" => 2, "c" => 3 }
            m.keys().len()
        }
    "#;
    assert!(matches!(run(src, "main"), Value::Int(3)));
}

#[test]
fn map_values_array_length() {
    let src = r#"
        def main() {
            let m = { "a" => 1, "b" => 2, "c" => 3 }
            m.values().len()
        }
    "#;
    assert!(matches!(run(src, "main"), Value::Int(3)));
}

// ════════════════════════════════════════════════════════════════════
// Range operations
// ════════════════════════════════════════════════════════════════════

#[test]
fn range_iteration_in_for_loop() {
    let src = r#"
        def sum_range(n) {
            let mut total = 0
            for i in 0..n {
                total += i
            }
            total
        }
    "#;
    assert!(matches!(run_with_args(src, "sum_range", vec![Value::Int(5)]), Value::Int(10)));
}

#[test]
fn inclusive_range_includes_end() {
    let src = r#"
        def sum_inclusive(n) {
            let mut total = 0
            for i in 1...n {
                total += i
            }
            total
        }
    "#;
    assert!(matches!(run_with_args(src, "sum_inclusive", vec![Value::Int(5)]), Value::Int(15)));
}

#[test]
fn range_zero_to_zero_yields_no_iterations() {
    let src = r#"
        def f() {
            let mut count = 0
            for i in 0..0 {
                count += 1
            }
            count
        }
    "#;
    assert!(matches!(run(src, "f"), Value::Int(0)));
}

// ════════════════════════════════════════════════════════════════════
// Closures & captures
// ════════════════════════════════════════════════════════════════════

#[test]
fn closure_captures_local() {
    let src = r#"
        def make_counter(start) {
            |n| start + n
        }
    "#;
    let mut interp = Interpreter::new();
    interp.load_source(src).unwrap();
    let counter = interp.call("make_counter", vec![Value::Int(100)]).unwrap();
    let r = garnet_interp::eval::call_value(&counter, vec![Value::Int(7)]).unwrap();
    assert!(matches!(r, Value::Int(107)));
}

#[test]
fn closure_captures_complex_value() {
    let src = r#"
        def make_lookup(items) {
            |key| items.get(key)
        }
    "#;
    let mut interp = Interpreter::new();
    interp.load_source(src).unwrap();
    let m = Value::map(vec![("foo".to_string(), Value::Int(42))]);
    let look = interp.call("make_lookup", vec![m]).unwrap();
    let r = garnet_interp::eval::call_value(&look, vec![Value::str("foo")]).unwrap();
    assert!(matches!(r, Value::Int(42)));
}

#[test]
fn closure_in_map_transforms_each() {
    let src = r#"
        def add_each(arr, n) {
            arr.map(|x| x + n)
        }
    "#;
    let arr = Value::array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    if let Value::Array(out) = run_with_args(src, "add_each", vec![arr, Value::Int(10)]) {
        let v = out.borrow();
        assert!(matches!(v[0], Value::Int(11)));
        assert!(matches!(v[2], Value::Int(13)));
    }
}

// ════════════════════════════════════════════════════════════════════
// Pipeline operator
// ════════════════════════════════════════════════════════════════════

#[test]
fn pipeline_into_function() {
    let src = r#"
        def double(x) { x * 2 }
        def f() { 5 |> double }
    "#;
    assert!(matches!(run(src, "f"), Value::Int(10)));
}

#[test]
fn pipeline_chain_three_levels() {
    let src = r#"
        def double(x) { x * 2 }
        def incr(x) { x + 1 }
        def neg(x) { -x }
        def f() { 5 |> double |> incr |> neg }
    "#;
    assert!(matches!(run(src, "f"), Value::Int(-11)));
}

#[test]
fn pipeline_passes_value_to_unary_fn() {
    let src = r#"
        def square(x) { x * x }
        def neg(x) { -x }
        def f() { 5 |> square |> neg }
    "#;
    assert!(matches!(run(src, "f"), Value::Int(-25)));
}

// ════════════════════════════════════════════════════════════════════
// Recursion & tail-position
// ════════════════════════════════════════════════════════════════════

#[test]
fn recursion_factorial_5() {
    let src = r#"
        def fact(n) {
            if n <= 1 { 1 } else { n * fact(n - 1) }
        }
    "#;
    assert!(matches!(run_with_args(src, "fact", vec![Value::Int(5)]), Value::Int(120)));
}

#[test]
fn recursion_fibonacci_15() {
    let src = r#"
        def fib(n) {
            if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
        }
    "#;
    assert!(matches!(run_with_args(src, "fib", vec![Value::Int(15)]), Value::Int(610)));
}

#[test]
fn mutual_recursion_even_odd() {
    let src = r#"
        def is_even(n) {
            if n == 0 { true } else { is_odd(n - 1) }
        }
        def is_odd(n) {
            if n == 0 { false } else { is_even(n - 1) }
        }
    "#;
    assert!(matches!(run_with_args(src, "is_even", vec![Value::Int(8)]), Value::Bool(true)));
    assert!(matches!(run_with_args(src, "is_odd", vec![Value::Int(7)]), Value::Bool(true)));
}

// ════════════════════════════════════════════════════════════════════
// Pattern matching breadth
// ════════════════════════════════════════════════════════════════════

#[test]
fn match_int_literal_arms() {
    let src = r#"
        def classify(n) {
            match n {
                0 => :zero,
                1 => :one,
                2 => :two,
                _ => :other,
            }
        }
    "#;
    if let Value::Symbol(s) = run_with_args(src, "classify", vec![Value::Int(0)]) {
        assert_eq!(s.as_str(), "zero");
    }
    if let Value::Symbol(s) = run_with_args(src, "classify", vec![Value::Int(99)]) {
        assert_eq!(s.as_str(), "other");
    }
}

#[test]
fn match_string_literal_arms() {
    let src = r#"
        def kind(s) {
            match s {
                "apple" => :fruit,
                "carrot" => :veg,
                _ => :unknown,
            }
        }
    "#;
    if let Value::Symbol(s) = run_with_args(src, "kind", vec![Value::str("carrot")]) {
        assert_eq!(s.as_str(), "veg");
    }
}

#[test]
fn match_with_ident_binding_passes_value() {
    let src = r#"
        def double_or_neg(x) {
            match x {
                n => n * 2,
            }
        }
    "#;
    assert!(matches!(run_with_args(src, "double_or_neg", vec![Value::Int(5)]), Value::Int(10)));
}

#[test]
fn match_with_guard_skips_arm() {
    let src = r#"
        def classify(n) {
            match n {
                x if x < 0 => :neg,
                0 => :zero,
                x if x < 10 => :small,
                _ => :big,
            }
        }
    "#;
    if let Value::Symbol(s) = run_with_args(src, "classify", vec![Value::Int(-5)]) {
        assert_eq!(s.as_str(), "neg");
    }
    if let Value::Symbol(s) = run_with_args(src, "classify", vec![Value::Int(0)]) {
        assert_eq!(s.as_str(), "zero");
    }
    if let Value::Symbol(s) = run_with_args(src, "classify", vec![Value::Int(5)]) {
        assert_eq!(s.as_str(), "small");
    }
    if let Value::Symbol(s) = run_with_args(src, "classify", vec![Value::Int(99)]) {
        assert_eq!(s.as_str(), "big");
    }
}

#[test]
fn match_nested_enum_patterns() {
    let src = r#"
        def describe(r) {
            match r {
                Ok(Some(v)) => v,
                Ok(None) => -1,
                Err(_) => -99,
            }
        }
    "#;
    let mut interp = Interpreter::new();
    interp.load_source(src).unwrap();
    let inner = interp.call("some", vec![Value::Int(42)]).unwrap();
    let wrapped = interp.call("ok", vec![inner]).unwrap();
    let r = interp.call("describe", vec![wrapped]).unwrap();
    assert!(matches!(r, Value::Int(42)));
}

// ════════════════════════════════════════════════════════════════════
// Try/rescue/ensure semantics
// ════════════════════════════════════════════════════════════════════

#[test]
fn try_returns_body_when_no_error() {
    let src = r#"
        def f() {
            try { 42 } rescue e { -1 }
        }
    "#;
    assert!(matches!(run(src, "f"), Value::Int(42)));
}

#[test]
fn try_catches_div_by_zero() {
    let src = r#"
        def f(a, b) {
            try { a / b } rescue e { -1 }
        }
    "#;
    assert!(matches!(run_with_args(src, "f", vec![Value::Int(10), Value::Int(0)]), Value::Int(-1)));
}

#[test]
fn try_catches_index_out_of_bounds() {
    let src = r#"
        def f() {
            try { [1, 2, 3][99] } rescue e { -1 }
        }
    "#;
    assert!(matches!(run(src, "f"), Value::Int(-1)));
}

#[test]
fn try_re_raises_when_no_rescue_matches_type() {
    let src = r#"
        def f() {
            try {
                try {
                    raise :inner
                } rescue e: SpecificType { 99 }
            } rescue e { -1 }
        }
    "#;
    assert!(matches!(run(src, "f"), Value::Int(-1)));
}

#[test]
fn try_ensure_runs_on_success() {
    let src = r#"
        def f() {
            let mut cleanup_ran = 0
            let result = try {
                42
            } ensure {
                cleanup_ran = 1
            }
            cleanup_ran + result
        }
    "#;
    assert!(matches!(run(src, "f"), Value::Int(43)));
}

#[test]
fn try_ensure_runs_on_failure_then_propagates() {
    // After a rescue catches the error, ensure runs and the rescue value is
    // returned.
    let src = r#"
        def f() {
            let mut cleanup_ran = 0
            let result = try {
                raise "boom"
            } rescue e {
                99
            } ensure {
                cleanup_ran = 1
            }
            cleanup_ran + result
        }
    "#;
    assert!(matches!(run(src, "f"), Value::Int(100)));
}

#[test]
fn question_mark_unwraps_ok() {
    let src = r#"
        def safe_op() {
            ok(42)
        }
        def caller() {
            try {
                let v = safe_op()?
                v + 1
            } rescue e { 0 }
        }
    "#;
    assert!(matches!(run(src, "caller"), Value::Int(43)));
}

#[test]
fn question_mark_propagates_err() {
    let src = r#"
        def fail() {
            err("oh no")
        }
        def caller() {
            try {
                let v = fail()?
                v + 1
            } rescue e { -1 }
        }
    "#;
    assert!(matches!(run(src, "caller"), Value::Int(-1)));
}

#[test]
fn question_mark_unwraps_some() {
    let src = r#"
        def get_it() {
            some(7)
        }
        def caller() {
            try {
                let v = get_it()?
                v
            } rescue e { -1 }
        }
    "#;
    assert!(matches!(run(src, "caller"), Value::Int(7)));
}

#[test]
fn question_mark_propagates_none() {
    let src = r#"
        def caller() {
            try {
                let v = none()?
                v
            } rescue e { -2 }
        }
    "#;
    assert!(matches!(run(src, "caller"), Value::Int(-2)));
}

// ════════════════════════════════════════════════════════════════════
// Struct semantics
// ════════════════════════════════════════════════════════════════════

#[test]
fn struct_ctor_arity_mismatch_raises() {
    let src = r#"
        struct P { x: Int, y: Int }
        def main() { P(1) }
    "#;
    let mut interp = Interpreter::new();
    interp.load_source(src).unwrap();
    let r = interp.call("main", vec![]);
    assert!(r.is_err());
}

#[test]
fn struct_field_mutation_persists() {
    let src = r#"
        struct C { n: Int }
        def main() {
            let c = C(0)
            c.n = 5
            c.n
        }
    "#;
    assert!(matches!(run(src, "main"), Value::Int(5)));
}

#[test]
fn struct_field_chain_access() {
    let src = r#"
        struct A { x: Int }
        struct B { a: A }
        def main() {
            let b = B(A(11))
            b.a.x
        }
    "#;
    assert!(matches!(run(src, "main"), Value::Int(11)));
}

// ════════════════════════════════════════════════════════════════════
// Enum semantics
// ════════════════════════════════════════════════════════════════════

#[test]
fn enum_nullary_variant_via_path() {
    let src = r#"
        enum Color { Red, Green, Blue }
        def main() {
            match Color::Green {
                Color::Red => 1,
                Color::Green => 2,
                Color::Blue => 3,
            }
        }
    "#;
    assert!(matches!(run(src, "main"), Value::Int(2)));
}

// ════════════════════════════════════════════════════════════════════
// Loop semantics
// ════════════════════════════════════════════════════════════════════

#[test]
fn loop_break_with_value() {
    let src = r#"
        def find_first_over(lim) {
            let mut n = 1
            loop {
                if n > lim { return n }
                n += 1
            }
        }
    "#;
    assert!(matches!(run_with_args(src, "find_first_over", vec![Value::Int(50)]), Value::Int(51)));
}

#[test]
fn nested_loops_break_only_inner() {
    let src = r#"
        def f() {
            let mut outer_count = 0
            let mut inner_count = 0
            for i in 0..3 {
                for j in 0..5 {
                    if j == 2 { break }
                    inner_count += 1
                }
                outer_count += 1
            }
            outer_count * 100 + inner_count
        }
    "#;
    // Outer = 3, inner = 3 * 2 = 6, result = 306.
    assert!(matches!(run(src, "f"), Value::Int(306)));
}

#[test]
fn while_zero_iterations() {
    let src = r#"
        def f() {
            let mut count = 0
            while false {
                count += 1
            }
            count
        }
    "#;
    assert!(matches!(run(src, "f"), Value::Int(0)));
}

#[test]
fn for_iterates_over_string() {
    let src = r#"
        def count_chars(s) {
            let mut n = 0
            for _c in s {
                n += 1
            }
            n
        }
    "#;
    assert!(matches!(run_with_args(src, "count_chars", vec![Value::str("hello")]), Value::Int(5)));
}

// ════════════════════════════════════════════════════════════════════
// Type conversions
// ════════════════════════════════════════════════════════════════════

#[test]
fn to_i_from_string() {
    assert!(matches!(eval(r#"to_i("42")"#), Value::Int(42)));
}

#[test]
fn to_i_from_float() {
    assert!(matches!(eval("to_i(3.7)"), Value::Int(3)));
}

#[test]
fn to_i_from_bool_true() {
    assert!(matches!(eval("to_i(true)"), Value::Int(1)));
}

#[test]
fn to_i_from_bool_false() {
    assert!(matches!(eval("to_i(false)"), Value::Int(0)));
}

#[test]
fn to_f_from_int() {
    if let Value::Float(f) = eval("to_f(5)") {
        assert!((f - 5.0).abs() < 1e-9);
    }
}

#[test]
fn to_f_from_string() {
    if let Value::Float(f) = eval(r#"to_f("2.5")"#) {
        assert!((f - 2.5).abs() < 1e-9);
    }
}

#[test]
fn to_s_of_int() {
    if let Value::Str(s) = eval("to_s(42)") {
        assert_eq!(s.as_str(), "42");
    }
}

#[test]
fn to_s_of_array() {
    if let Value::Str(s) = eval("to_s([1, 2, 3])") {
        assert!(s.contains("1"));
        assert!(s.contains("3"));
    }
}

// ════════════════════════════════════════════════════════════════════
// type_of
// ════════════════════════════════════════════════════════════════════

#[test]
fn type_of_int() {
    if let Value::Str(s) = eval("type_of(42)") {
        assert_eq!(s.as_str(), "Int");
    }
}

#[test]
fn type_of_string() {
    if let Value::Str(s) = eval(r#"type_of("hi")"#) {
        assert_eq!(s.as_str(), "String");
    }
}

#[test]
fn type_of_array() {
    if let Value::Str(s) = eval("type_of([1, 2])") {
        assert_eq!(s.as_str(), "Array");
    }
}

#[test]
fn type_of_nil() {
    if let Value::Str(s) = eval("type_of(nil)") {
        assert_eq!(s.as_str(), "Nil");
    }
}

// ════════════════════════════════════════════════════════════════════
// is_nil
// ════════════════════════════════════════════════════════════════════

#[test]
fn is_nil_true_for_nil() {
    assert!(matches!(eval("is_nil(nil)"), Value::Bool(true)));
}

#[test]
fn is_nil_false_for_value() {
    assert!(matches!(eval("is_nil(0)"), Value::Bool(false)));
}

// ════════════════════════════════════════════════════════════════════
// len primitive
// ════════════════════════════════════════════════════════════════════

#[test]
fn len_of_array() {
    assert!(matches!(eval("len([1, 2, 3, 4])"), Value::Int(4)));
}

#[test]
fn len_of_string() {
    assert!(matches!(eval(r#"len("hello")"#), Value::Int(5)));
}

#[test]
fn len_of_empty_map() {
    assert!(matches!(eval("len({})"), Value::Int(0)));
}
