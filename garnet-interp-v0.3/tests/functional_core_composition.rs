//! S30 capstone: the functional `core::` surface (result + option + iter, made
//! runnable across S26-S28) composes end-to-end from Garnet source, exercising
//! BOTH the success and failure tracks of the railway.
//!
//!   iter:   collect(1..5) -> map double -> fold(+) = 20
//!   result: gate(20)=Ok -> map double -> unwrap_or = 40        (Ok track)
//!           gate(3)=Err -> or_else recover -> unwrap_or = 0     (Err track + recovery)
//!   option: some(40) -> map double -> unwrap_or = 80            (Some track)
//!           none() -> unwrap_or = 7                             (None track)
//!   => [20, 40, 0, 80, 7]

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
fn s30_functional_core_railway_composes_success_and_failure_tracks() {
    let result = run(r#"
        @caps()
        def double(x) { x * 2 }
        def add(acc, x) { acc + x }
        def gate(x) { if x > 5 { core::result::ok(x) } else { core::result::err(x) } }
        def recover(e) { core::result::ok(0) }
        def main() {
          # core::iter: build + transform + reduce
          let total = core::iter::fold(
            core::iter::map(core::iter::collect(1..5), double), 0, add)   # 20

          # core::result Ok track: gate(20)=Ok -> map double -> unwrap_or
          let big = core::result::unwrap_or(core::result::map(gate(total), double), 0)  # 40
          # core::result Err track: gate(3)=Err -> or_else recover -> unwrap_or
          let small = core::result::unwrap_or(core::result::or_else(gate(3), recover), 99) # 0

          # core::option Some track + None track
          let present = core::option::unwrap_or(
            core::option::map(core::option::some(big), double), 0)        # 80
          let absent = core::option::unwrap_or(core::option::none(), 7)   # 7

          [total, big, small, present, absent]
        }
        "#);

    assert_eq!(int_array(result), vec![20, 40, 0, 80, 7]);
}
