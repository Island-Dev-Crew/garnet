//! Struct instantiation, field access, enum construction, pattern matching.

use garnet_interp::{Interpreter, Value};

fn run(src: &str, main: &str) -> Value {
    let mut interp = Interpreter::new();
    interp.load_source(src).expect("load");
    interp.call(main, vec![]).expect("call")
}

#[test]
fn struct_instantiation_and_field_access() {
    let src = r#"
        struct Point { x: Int, y: Int }
        def main() {
            let p = Point(3, 4)
            p.x + p.y
        }
    "#;
    assert!(matches!(run(src, "main"), Value::Int(7)));
}

#[test]
fn struct_field_mutation_via_assign() {
    // Managed mode allows mutating struct fields via the ARC-style semantics
    let src = r#"
        struct Counter { n: Int }
        def main() {
            let c = Counter(0)
            c.n = c.n + 1
            c.n = c.n + 10
            c.n
        }
    "#;
    assert!(matches!(run(src, "main"), Value::Int(11)));
}

#[test]
fn enum_variant_construction_via_path() {
    let src = r#"
        enum Color { Red, Green, Blue }
        def main() {
            let c = Color::Green
            match c {
                Color::Red => 1,
                Color::Green => 2,
                Color::Blue => 3,
            }
        }
    "#;
    assert!(matches!(run(src, "main"), Value::Int(2)));
}

#[test]
fn enum_with_payload() {
    let src = r#"
        enum Shape {
            Circle(Int),
            Square(Int),
        }
        def area(shape) {
            match shape {
                Circle(r) => 3 * r * r,
                Square(s) => s * s,
            }
        }
    "#;
    let mut interp = Interpreter::new();
    interp.load_source(src).unwrap();
    // Build a Variant directly (the path form needs Variant or Call dispatch).
    let circle = Value::Variant {
        path: std::rc::Rc::new(vec!["Shape".to_string()]),
        variant: std::rc::Rc::new("Circle".to_string()),
        fields: std::rc::Rc::new(vec![Value::Int(10)]),
    };
    let r = interp.call("area", vec![circle]).unwrap();
    assert!(matches!(r, Value::Int(300)));
}

#[test]
fn nested_struct_field_access() {
    let src = r#"
        struct Address { city: String }
        struct Person { name: String, addr: Address }
        def main() {
            let addr = Address("Huntsville")
            let p = Person("Jon", addr)
            p.addr.city
        }
    "#;
    match run(src, "main") {
        Value::Str(s) => assert_eq!(s.as_str(), "Huntsville"),
        other => panic!("expected string, got {other:?}"),
    }
}
