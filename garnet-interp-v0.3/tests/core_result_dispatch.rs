//! S26 runtime dispatch proof: the registered `core::result` combinators execute
//! from Garnet source with first-class function arguments.
//!
//! Railway-oriented: `map`/`and_then` transform/chain the Ok track and pass Err
//! through untouched; `or_else` recovers an Err; `unwrap_or` extracts. Each value
//! is extracted with `core::result::unwrap_or` so the asserted output is a plain
//! Int array — exercising the combinators end-to-end without leaving the language.

use garnet_interp::{Interpreter, Value};

fn run(src: &str) -> Value {
    let mut interp = Interpreter::new();
    interp.load_source(src).expect("load source");
    interp.call("main", vec![]).expect("call main")
}

fn int_array(value: Value) -> Vec<i64> {
    match value {
        Value::Array(items) => items
            .borrow()
            .iter()
            .map(|v| match v {
                Value::Int(i) => *i,
                other => panic!("expected Int element, got {other:?}"),
            })
            .collect(),
        other => panic!("expected Array, got {other:?}"),
    }
}

#[test]
fn s26_core_result_railway_executes_from_source() {
    let result = run(r#"
        @caps()
        def double(x) { x * 2 }
        def succ_ok(x) { core::result::ok(x + 1) }
        def recover(e) { core::result::ok(0) }
        def main() {
          let good = core::result::ok(5)
          let bad = core::result::err("boom")

          let mapped = core::result::map(good, double)          # Ok(10)
          let mapped_err = core::result::map(bad, double)       # Err("boom")
          let chained = core::result::and_then(good, succ_ok)   # Ok(6)
          let short = core::result::and_then(bad, succ_ok)      # Err (short-circuit)
          let recovered = core::result::or_else(bad, recover)   # Ok(0)

          [
            core::result::unwrap_or(mapped, 7),
            core::result::unwrap_or(mapped_err, 7),
            core::result::unwrap_or(chained, 7),
            core::result::unwrap_or(short, 8),
            core::result::unwrap_or(recovered, 7),
            core::result::unwrap_or(good, 7),
            core::result::unwrap_or(bad, 99)
          ]
        }
        "#);

    // mapped=Ok(10); mapped_err=Err→7; chained=Ok(6); short=Err→8;
    // recovered=Ok(0); unwrap_or(Ok(5))=5; unwrap_or(Err,99)=99.
    assert_eq!(int_array(result), vec![10, 7, 6, 8, 0, 5, 99]);
}
