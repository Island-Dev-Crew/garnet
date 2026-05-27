//! S28 runtime dispatch proof: the last three registered `core::iter` combinators
//! (`zip`, `collect`, `chain`) execute from Garnet source and compose with the
//! S21 higher-order `fold`.
//!
//!   collect(1..4)            -> [1,2,3]        (Range expanded)
//!   collect([10,20])         -> [10,20]        (Array materialized)
//!   chain([1,2,3],[10,20])   -> [1,2,3,10,20]  (concatenated; fold-sum = 36)
//!   zip([1,2,3],[9,8])       -> 2 pairs        (stops at the shorter sequence)

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
fn s28_core_iter_completion_executes_from_source() {
    let result = run(r#"
        @caps()
        def add(acc, x) { acc + x }
        def main() {
          let r = core::iter::collect(1..4)             # [1,2,3] from a Range
          let a = core::iter::collect([10, 20])         # [10,20] materialized
          let joined = core::iter::chain(r, a)          # [1,2,3,10,20]
          let total = core::iter::fold(joined, 0, add)  # 36 (proves chain values)
          let pairs = core::iter::zip([1, 2, 3], [9, 8]) # 2 pairs (shorter wins)
          [total, joined.len(), r.len(), a.len(), pairs.len()]
        }
        "#);

    assert_eq!(int_array(result), vec![36, 5, 3, 2, 2]);
}
