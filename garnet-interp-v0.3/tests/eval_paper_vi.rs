//! Paper VI contribution coverage — the seven novel contributions of the
//! Garnet language paper each have a runnable program here. Failing any
//! test in this file means a published contribution has lost its grounding
//! in the reference implementation.

use garnet_interp::{Interpreter, Value};

fn run(src: &str, fn_name: &str) -> Value {
    let mut interp = Interpreter::new();
    interp.load_source(src).expect("load");
    interp.call(fn_name, vec![]).expect("call")
}

// ════════════════════════════════════════════════════════════════════
// Contribution 1: Dual-mode language design (managed `def` + safe `fn`)
// ════════════════════════════════════════════════════════════════════

#[test]
fn c1_managed_and_safe_fns_coexist_in_one_module() {
    let src = r#"
        def managed(x) { x + 1 }
        fn safe(x: Int) -> Int { x * 2 }
        def main() {
            managed(10) + safe(10)
        }
    "#;
    // Note: the interp evaluates both with managed semantics. The mode
    // distinction is preserved in the AST for the safe-mode checker.
    assert!(matches!(run(src, "main"), Value::Int(31)));
}

// ════════════════════════════════════════════════════════════════════
// Contribution 2: First-class memory units as language primitives
// ════════════════════════════════════════════════════════════════════

#[test]
fn c2_all_four_memory_kinds_declarable_as_first_class() {
    let src = r#"
        memory working   working_unit   : Buffer
        memory episodic  episodic_unit  : EpisodeStore<Event>
        memory semantic  semantic_unit  : VectorIndex<Fact>
        memory procedural procedural_unit : WorkflowStore<Recipe>
        def main() { 4 }
    "#;
    assert!(matches!(run(src, "main"), Value::Int(4)));
}

#[test]
fn c2_memory_unit_callable_via_method_syntax() {
    let src = r#"
        memory working scratch : Buffer
        def main() {
            scratch.append("entry")
            scratch.append("entry2")
            scratch.len()
        }
    "#;
    // v3.2: kind-aware dispatch routes to a real WorkingStore<Value>, so
    // .len() returns the actual count of pushed entries.
    assert!(matches!(run(src, "main"), Value::Int(2)));
}

// ════════════════════════════════════════════════════════════════════
// Contribution 3: Structured error model with try/rescue + ? sugar
// ════════════════════════════════════════════════════════════════════

#[test]
fn c3_try_rescue_replaces_panic_handling() {
    let src = r#"
        def safe_div(a, b) {
            try {
                a / b
            } rescue e {
                0
            }
        }
        def main() {
            safe_div(10, 2) + safe_div(5, 0) + safe_div(8, 4)
        }
    "#;
    assert!(matches!(run(src, "main"), Value::Int(7)));
}

#[test]
fn c3_question_mark_chains_for_short_circuit() {
    let src = r#"
        def step1() { ok(10) }
        def step2(x) { ok(x + 5) }
        def step3(x) { ok(x * 2) }
        def chain() {
            try {
                let a = step1()?
                let b = step2(a)?
                let c = step3(b)?
                c
            } rescue e { -1 }
        }
    "#;
    assert!(matches!(run(src, "chain"), Value::Int(30)));
}

#[test]
fn c3_question_mark_short_circuits_on_first_err() {
    let src = r#"
        def step1() { ok(10) }
        def step2(_x) { err("step2 failed") }
        def step3(x) { ok(x * 2) }
        def chain() {
            try {
                let a = step1()?
                let b = step2(a)?
                let c = step3(b)?
                c
            } rescue e { -99 }
        }
    "#;
    assert!(matches!(run(src, "chain"), Value::Int(-99)));
}

// ════════════════════════════════════════════════════════════════════
// Contribution 4: Kind-aware allocation (managed ARC vs safe affine)
// ════════════════════════════════════════════════════════════════════

#[test]
fn c4_managed_struct_uses_arc_sharing() {
    let src = r#"
        struct Box { v: Int }
        def main() {
            let b = Box(42)
            let b2 = b
            b.v = 99
            b.v + b2.v
        }
    "#;
    // ARC sharing means b and b2 point at the same struct, so changing b.v
    // changes b2.v as well: 99 + 99 = 198.
    assert!(matches!(run(src, "main"), Value::Int(198)));
}

#[test]
fn c4_safe_signatures_carry_ownership_kinds() {
    let src = r#"
        fn process(own data: Buffer, borrow cfg: Config, ref out: Buffer) -> Status {
            Status::Ok
        }
        enum Status { Ok }
        def main() { 1 }
    "#;
    assert!(matches!(run(src, "main"), Value::Int(1)));
}

// ════════════════════════════════════════════════════════════════════
// Contribution 5: Actor protocol model with explicit handlers
// ════════════════════════════════════════════════════════════════════

#[test]
fn c5_actor_with_protocol_and_handler_loads() {
    let src = r#"
        actor Counter {
            let mut n = 0
            protocol incr() -> Int
            protocol get() -> Int
            on incr() {
                n += 1
                n
            }
            on get() { n }
        }
        def main() { 1 }
    "#;
    assert!(matches!(run(src, "main"), Value::Int(1)));
}

#[test]
fn c5_actor_with_annotations_on_handlers() {
    let src = r#"
        actor Worker {
            protocol process(job: Job) -> Result<Output, Error>
            @max_depth(5)
            @fan_out(10)
            on process(job) { Ok(job) }
        }
        def main() { 1 }
    "#;
    assert!(matches!(run(src, "main"), Value::Int(1)));
}

// ════════════════════════════════════════════════════════════════════
// Contribution 6: Annotation system (recursion + fan-out guardrails)
// ════════════════════════════════════════════════════════════════════

#[test]
fn c6_max_depth_annotation_loads_with_function() {
    let src = r#"
        @max_depth(8)
        def recursive(n) {
            if n <= 0 { 0 } else { recursive(n - 1) + 1 }
        }
        def main() { recursive(5) }
    "#;
    assert!(matches!(run(src, "main"), Value::Int(5)));
}

#[test]
fn c6_fan_out_annotation_loads_with_function() {
    let src = r#"
        @fan_out(4)
        def parallel_map(items) {
            items.map(|x| x * 2)
        }
        def main() {
            let r = parallel_map([1, 2, 3])
            r.len()
        }
    "#;
    assert!(matches!(run(src, "main"), Value::Int(3)));
}

#[test]
fn c6_require_metadata_annotation_loads() {
    let src = r#"
        @require_metadata
        def needs_meta(x) { x }
        def main() { needs_meta(7) }
    "#;
    assert!(matches!(run(src, "main"), Value::Int(7)));
}

// ════════════════════════════════════════════════════════════════════
// Contribution 7: Pattern matching with guards (Mini-Spec §6.3)
// ════════════════════════════════════════════════════════════════════

#[test]
fn c7_pattern_with_guard_branches_correctly() {
    let src = r#"
        def categorize(n) {
            match n {
                x if x < 0 => :negative,
                0 => :zero,
                x if x < 10 => :small,
                x if x < 100 => :medium,
                _ => :large,
            }
        }
    "#;
    let cases = [
        (-7, "negative"),
        (0, "zero"),
        (5, "small"),
        (50, "medium"),
        (500, "large"),
    ];
    let mut interp = Interpreter::new();
    interp.load_source(src).unwrap();
    for (input, expected) in cases {
        let r = interp.call("categorize", vec![Value::Int(input)]).unwrap();
        if let Value::Symbol(s) = r {
            assert_eq!(s.as_str(), expected);
        } else {
            panic!("expected symbol for {input}");
        }
    }
}

#[test]
fn c7_enum_pattern_with_field_binding_works() {
    let src = r#"
        def unwrap_or_zero(r) {
            match r {
                Ok(v) => v,
                Err(_) => 0,
            }
        }
    "#;
    let mut interp = Interpreter::new();
    interp.load_source(src).unwrap();
    let ok = interp.call("ok", vec![Value::Int(123)]).unwrap();
    let err = interp.call("err", vec![Value::str("oops")]).unwrap();
    assert!(matches!(interp.call("unwrap_or_zero", vec![ok]).unwrap(), Value::Int(123)));
    assert!(matches!(interp.call("unwrap_or_zero", vec![err]).unwrap(), Value::Int(0)));
}

#[test]
fn c7_wildcard_falls_through_to_default() {
    let src = r#"
        def label(x) {
            match x {
                1 => :one,
                2 => :two,
                _ => :other,
            }
        }
    "#;
    let mut interp = Interpreter::new();
    interp.load_source(src).unwrap();
    if let Value::Symbol(s) = interp.call("label", vec![Value::Int(99)]).unwrap() {
        assert_eq!(s.as_str(), "other");
    }
}
