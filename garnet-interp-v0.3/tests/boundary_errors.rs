//! Cross-boundary error bridging — Paper VI Contribution 5.
//!
//! These tests prove the four directions of error flow across the
//! managed (`def`) ↔ safe (`fn`) mode boundary:
//!
//! 1. **safe → managed (Err propagation):** a safe fn returning `Result::Err`
//!    is unwrapped by `?` in the managed caller and rescued.
//! 2. **managed → safe (raise capture):** a managed def that raises is
//!    intercepted at the safe-call boundary and surfaced as `Result::Err`
//!    to the safe caller.
//! 3. **double bounce (managed → safe → managed):** the middle safe layer
//!    must capture the inner managed raise as Err, then the outer managed
//!    caller catches the Err via `?`/rescue.
//! 4. **type mismatch:** passing the wrong shape across the boundary fails
//!    loudly rather than silently corrupting the call.

use garnet_interp::{Interpreter, Value};

fn run_args(src: &str, fn_name: &str, args: Vec<Value>) -> Value {
    let mut interp = Interpreter::new();
    interp.load_source(src).expect("load");
    interp.call(fn_name, args).expect("call")
}

// ── Direction 1: safe→managed Err propagation ───────────────────────

#[test]
fn safe_to_managed_err_propagates_via_question_mark() {
    let src = r#"
        # Safe fn returns Result<Int, String>.
        fn validate(n: Int) -> Result {
            if n < 0 {
                err("negative not allowed")
            } else {
                ok(n)
            }
        }

        # Managed caller uses ? to unwrap Ok or propagate Err.
        def safe_call(n) {
            try {
                let v = validate(n)?
                v + 100
            } rescue e {
                -1
            }
        }
    "#;
    assert!(matches!(run_args(src, "safe_call", vec![Value::Int(5)]), Value::Int(105)));
    assert!(matches!(run_args(src, "safe_call", vec![Value::Int(-1)]), Value::Int(-1)));
}

#[test]
fn safe_to_managed_err_can_be_pattern_matched_directly() {
    let src = r#"
        fn validate(n: Int) -> Result {
            if n < 0 { err("bad") } else { ok(n) }
        }

        def caller(n) {
            match validate(n) {
                Ok(v) => v,
                Err(_) => -99,
            }
        }
    "#;
    assert!(matches!(run_args(src, "caller", vec![Value::Int(7)]), Value::Int(7)));
    assert!(matches!(run_args(src, "caller", vec![Value::Int(-3)]), Value::Int(-99)));
}

// ── Direction 2: managed→safe raise capture ─────────────────────────

#[test]
fn managed_raise_captured_in_safe_caller_as_err() {
    // The safe caller wraps the managed call in try/rescue and converts
    // the raise to a Result::Err at the boundary. Until v0.4 wires automatic
    // bridging, the safe fn does this explicitly — which is the bridging
    // contract: safe code MUST surface raises as Err, and try/rescue is
    // the language-level escape hatch (forbidden inside true @safe modules,
    // but the v3.2 interpreter allows it as the demonstration mechanism).
    let src = r#"
        def risky(n) {
            if n < 0 { raise "oh no" }
            n * 2
        }

        fn safe_caller(n: Int) -> Result {
            try {
                ok(risky(n))
            } rescue e {
                err(e)
            }
        }

        def driver(n) {
            match safe_caller(n) {
                Ok(v) => v,
                Err(_) => -1,
            }
        }
    "#;
    assert!(matches!(run_args(src, "driver", vec![Value::Int(5)]), Value::Int(10)));
    assert!(matches!(run_args(src, "driver", vec![Value::Int(-1)]), Value::Int(-1)));
}

// ── Direction 3: double bounce ──────────────────────────────────────

#[test]
fn double_bounce_managed_safe_managed_propagates_err() {
    let src = r#"
        # Inner managed function — raises on bad input.
        def inner(n) {
            if n < 0 { raise "inner failure" }
            n + 1
        }

        # Middle safe function — converts inner's raise to Err.
        fn middle(n: Int) -> Result {
            try {
                ok(inner(n))
            } rescue e {
                err(e)
            }
        }

        # Outer managed function — uses ? on middle's Result.
        def outer(n) {
            try {
                let v = middle(n)?
                v * 100
            } rescue e {
                -42
            }
        }
    "#;
    // Happy path: outer returns (n+1)*100
    assert!(matches!(run_args(src, "outer", vec![Value::Int(2)]), Value::Int(300)));
    // Failure path: inner raises, middle wraps as Err, outer's ? raises
    // again, outer's rescue catches and returns -42.
    assert!(matches!(run_args(src, "outer", vec![Value::Int(-5)]), Value::Int(-42)));
}

// ── Direction 4: type mismatch fails loud ───────────────────────────

#[test]
fn type_mismatch_at_boundary_returns_err_loudly() {
    // A safe fn declared to return Result is passed something that isn't
    // Result-shaped (an Int). The caller's ? must surface a clear error,
    // not silently treat the int as Ok or Err.
    let src = r#"
        # Safe fn deliberately returns a raw Int (violating declared Result
        # return type). The runtime catches this when ? tries to unwrap.
        fn buggy(n: Int) -> Result {
            n + 1
        }

        def caller(n) {
            try {
                let v = buggy(n)?
                v
            } rescue _e {
                -1
            }
        }
    "#;
    // The ? operator on a non-Result/Option value raises a type error,
    // which the rescue catches as -1.
    assert!(matches!(run_args(src, "caller", vec![Value::Int(5)]), Value::Int(-1)));
}

#[test]
fn rescue_typed_filter_sees_err_payload() {
    // The Err's payload reaches the rescue handler — proves the bridge
    // doesn't lose information.
    let src = r#"
        fn step(n: Int) -> Result {
            if n == 13 { err("unlucky number") } else { ok(n) }
        }

        def caller(n) {
            try {
                let v = step(n)?
                v
            } rescue e {
                e
            }
        }
    "#;
    let r = run_args(src, "caller", vec![Value::Int(13)]);
    match r {
        Value::Variant { variant, fields, .. } if variant.as_str() == "Err" => {
            assert_eq!(fields.len(), 1);
            if let Value::Str(s) = &fields[0] {
                assert_eq!(s.as_str(), "unlucky number");
            } else {
                panic!("expected string payload, got {:?}", fields[0]);
            }
        }
        other => panic!("expected Err variant carrying the message, got {other:?}"),
    }
}
