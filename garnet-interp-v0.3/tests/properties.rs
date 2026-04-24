//! Property-based tests for the interpreter.

use garnet_interp::{Interpreter, Value};
use proptest::prelude::*;

fn eval(src: &str) -> Result<Value, String> {
    Interpreter::new()
        .eval_expr_src(src)
        .map_err(|e| e.to_string())
}

// ── Pure integer arithmetic ─────────────────────────────────────────

proptest! {
    #[test]
    fn integer_addition_commutes(a in -10_000i64..10_000, b in -10_000i64..10_000) {
        let l = eval(&format!("{a} + {b}")).unwrap();
        let r = eval(&format!("{b} + {a}")).unwrap();
        prop_assert!(matches!((l, r), (Value::Int(x), Value::Int(y)) if x == y));
    }
}

proptest! {
    #[test]
    fn integer_addition_associates(a in -1000i64..1000, b in -1000i64..1000, c in -1000i64..1000) {
        let l = eval(&format!("({a} + {b}) + {c}")).unwrap();
        let r = eval(&format!("{a} + ({b} + {c})")).unwrap();
        prop_assert!(matches!((l, r), (Value::Int(x), Value::Int(y)) if x == y));
    }
}

proptest! {
    #[test]
    fn integer_multiplication_distributes_over_addition(
        a in -100i64..100, b in -100i64..100, c in -100i64..100
    ) {
        let l = eval(&format!("{a} * ({b} + {c})")).unwrap();
        let r = eval(&format!("{a} * {b} + {a} * {c}")).unwrap();
        prop_assert!(matches!((l, r), (Value::Int(x), Value::Int(y)) if x == y));
    }
}

proptest! {
    #[test]
    fn negate_negate_is_identity(n in -1_000_000i64..1_000_000) {
        let r = eval(&format!("-(-{n})")).unwrap();
        prop_assert!(matches!(r, Value::Int(m) if m == n));
    }
}

// ── Booleans ────────────────────────────────────────────────────────

proptest! {
    #[test]
    fn not_not_bool_is_bool(b in any::<bool>()) {
        let r = eval(&format!("not not {b}")).unwrap();
        prop_assert!(matches!(r, Value::Bool(rb) if rb == b));
    }
}

proptest! {
    #[test]
    fn and_short_circuits_on_false(rhs in 0i64..1000) {
        // false and X = false, regardless of X
        let r = eval(&format!("false and {rhs}")).unwrap();
        prop_assert!(matches!(r, Value::Bool(false)));
    }
}

// ── Collections ─────────────────────────────────────────────────────

proptest! {
    #[test]
    fn array_literal_length_matches_input(elements in proptest::collection::vec(0i64..1000, 0..30)) {
        let body = elements.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ");
        let src = format!("[{body}].len()");
        let r = eval(&src).unwrap();
        prop_assert!(matches!(r, Value::Int(n) if n == elements.len() as i64));
    }
}

proptest! {
    #[test]
    fn map_filter_length_at_most_input(elements in proptest::collection::vec(0i64..100, 0..20)) {
        let body = elements.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ");
        let src = format!("[{body}].filter(|x| x > 50).len()");
        let r = eval(&src).unwrap();
        if let Value::Int(n) = r {
            prop_assert!(n >= 0 && n as usize <= elements.len());
        } else {
            prop_assert!(false, "expected Int");
        }
    }
}

proptest! {
    #[test]
    fn reduce_sum_equals_iter_sum(elements in proptest::collection::vec(-100i64..100, 0..15)) {
        let body = elements.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ");
        let src = format!("[{body}].reduce(0, |a, b| a + b)");
        let r = eval(&src).unwrap();
        let expected: i64 = elements.iter().sum();
        prop_assert!(matches!(r, Value::Int(n) if n == expected));
    }
}

// ── No-panic: arbitrary printable expressions just error or succeed ──

proptest! {
    #[test]
    fn eval_never_panics_on_arbitrary_input(s in r"[ -~]{0,200}") {
        // The interpreter must return Ok or Err; it must never panic.
        let _ = eval(&s);
    }
}

// ── Determinism ─────────────────────────────────────────────────────

proptest! {
    #[test]
    fn eval_is_deterministic(n in 0i64..100_000) {
        let src = format!("{n} * 2 + 1");
        let r1 = eval(&src).unwrap();
        let r2 = eval(&src).unwrap();
        let r3 = eval(&src).unwrap();
        prop_assert!(matches!(&r1, Value::Int(_)));
        prop_assert_eq!(format!("{r1:?}"), format!("{r2:?}"));
        prop_assert_eq!(format!("{r2:?}"), format!("{r3:?}"));
    }
}
