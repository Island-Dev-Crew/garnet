//! S27 runtime dispatch proof: the registered `core::option` combinators execute
//! from Garnet source with first-class function arguments.
//!
//! `map`/`and_then` transform/chain the Some track and pass None through; each
//! value is extracted with `core::option::unwrap_or` so the asserted output is a
//! plain Int array — exercising the combinators end-to-end in the language.

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
fn s27_core_option_combinators_execute_from_source() {
    let result = run(r#"
        @caps()
        def double(x) { x * 2 }
        def succ_some(x) { core::option::some(x + 1) }
        def main() {
          let present = core::option::some(5)
          let absent = core::option::none()

          let mapped = core::option::map(present, double)          # Some(10)
          let mapped_none = core::option::map(absent, double)      # None
          let chained = core::option::and_then(present, succ_some) # Some(6)
          let short = core::option::and_then(absent, succ_some)    # None (short-circuit)

          [
            core::option::unwrap_or(mapped, 7),
            core::option::unwrap_or(mapped_none, 7),
            core::option::unwrap_or(chained, 7),
            core::option::unwrap_or(short, 8),
            core::option::unwrap_or(present, 7),
            core::option::unwrap_or(absent, 99)
          ]
        }
        "#);

    // mapped=Some(10); mapped_none=None→7; chained=Some(6); short=None→8;
    // unwrap_or(Some(5))=5; unwrap_or(None,99)=99.
    assert_eq!(int_array(result), vec![10, 7, 6, 8, 5, 99]);
}
