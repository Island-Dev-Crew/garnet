//! User-defined type parser tests: struct, enum, trait, impl (Mini-Spec §11.3).

use garnet_parser::ast::{FnMode, Item, TraitItem};
use garnet_parser::parse_source;

#[test]
fn parses_simple_struct() {
    let m = parse_source("struct Config { host: String, port: Int }").unwrap();
    match &m.items[0] {
        Item::Struct(s) => {
            assert_eq!(s.name, "Config");
            assert_eq!(s.fields.len(), 2);
            assert_eq!(s.fields[0].name, "host");
        }
        _ => panic!("expected struct"),
    }
}

#[test]
fn parses_struct_with_pub_fields() {
    let m = parse_source("pub struct Server { pub host: String, pub port: Int, timeout: Float }")
        .unwrap();
    match &m.items[0] {
        Item::Struct(s) => {
            assert!(s.public);
            assert!(s.fields[0].public);
            assert!(s.fields[1].public);
            assert!(!s.fields[2].public);
        }
        _ => panic!("expected struct"),
    }
}

#[test]
fn parses_struct_with_default_values() {
    let m = parse_source("struct Config { port: Int = 8080, timeout: Float = 30.0 }").unwrap();
    match &m.items[0] {
        Item::Struct(s) => {
            assert!(s.fields[0].default.is_some());
            assert!(s.fields[1].default.is_some());
        }
        _ => panic!("expected struct"),
    }
}

#[test]
fn parses_generic_struct() {
    let m = parse_source("struct Stack<T> { items: Array<T> }").unwrap();
    match &m.items[0] {
        Item::Struct(s) => {
            assert_eq!(s.type_params, vec!["T".to_string()]);
        }
        _ => panic!("expected struct"),
    }
}

#[test]
fn parses_simple_enum() {
    let src = r#"
        enum BuildResult {
            Success(Artifact),
            Failure(String),
            Timeout,
        }
    "#;
    let m = parse_source(src).unwrap();
    match &m.items[0] {
        Item::Enum(e) => {
            assert_eq!(e.variants.len(), 3);
            assert_eq!(e.variants[0].fields.len(), 1);
            assert_eq!(e.variants[2].fields.len(), 0);
        }
        _ => panic!("expected enum"),
    }
}

#[test]
fn parses_generic_enum() {
    let m = parse_source("enum Either<L, R> { Left(L), Right(R) }").unwrap();
    match &m.items[0] {
        Item::Enum(e) => {
            assert_eq!(e.type_params, vec!["L".to_string(), "R".to_string()]);
        }
        _ => panic!("expected enum"),
    }
}

#[test]
fn parses_public_enum() {
    let m = parse_source("pub enum Status { Ok, Err }").unwrap();
    match &m.items[0] {
        Item::Enum(e) => assert!(e.public),
        _ => panic!("expected enum"),
    }
}

#[test]
fn parses_simple_trait() {
    let src = r#"
        trait Serializable {
            fn serialize(borrow self: Self) -> Bytes
        }
    "#;
    let m = parse_source(src).unwrap();
    match &m.items[0] {
        Item::Trait(t) => {
            assert_eq!(t.name, "Serializable");
            assert_eq!(t.items.len(), 1);
            assert!(matches!(t.items[0], TraitItem::FnSig(_)));
        }
        _ => panic!("expected trait"),
    }
}

#[test]
fn parses_trait_with_both_fn_and_def() {
    let src = r#"
        trait Mixed {
            fn safe_method(own x: Int) -> Int
            def managed_method(y)
        }
    "#;
    let m = parse_source(src).unwrap();
    match &m.items[0] {
        Item::Trait(t) => {
            assert_eq!(t.items.len(), 2);
            if let TraitItem::FnSig(sig) = &t.items[0] {
                assert_eq!(sig.mode, FnMode::Safe);
            } else {
                panic!("expected fn sig");
            }
            if let TraitItem::FnSig(sig) = &t.items[1] {
                assert_eq!(sig.mode, FnMode::Managed);
            } else {
                panic!("expected def sig");
            }
        }
        _ => panic!("expected trait"),
    }
}

#[test]
fn parses_generic_trait() {
    let m = parse_source("trait Container<T> { def len() -> Int }").unwrap();
    match &m.items[0] {
        Item::Trait(t) => {
            assert_eq!(t.type_params, vec!["T".to_string()]);
        }
        _ => panic!("expected trait"),
    }
}

#[test]
fn parses_simple_impl() {
    let src = r#"
        impl Point {
            def origin() -> Point { Point::new(0, 0) }
        }
    "#;
    let m = parse_source(src).unwrap();
    match &m.items[0] {
        Item::Impl(i) => {
            assert_eq!(i.methods.len(), 1);
            assert!(i.trait_ty.is_none());
        }
        _ => panic!("expected impl"),
    }
}

#[test]
fn parses_trait_impl() {
    let src = r#"
        impl Display for Point {
            def to_string() -> String { "(0, 0)" }
        }
    "#;
    let m = parse_source(src).unwrap();
    match &m.items[0] {
        Item::Impl(i) => {
            assert!(i.trait_ty.is_some());
        }
        _ => panic!("expected impl"),
    }
}

#[test]
fn parses_generic_impl() {
    let src = r#"
        impl<T> Stack<T> {
            def new() -> Stack<T> { Stack::empty() }
        }
    "#;
    let m = parse_source(src).unwrap();
    match &m.items[0] {
        Item::Impl(i) => {
            assert_eq!(i.type_params, vec!["T".to_string()]);
        }
        _ => panic!("expected generic impl"),
    }
}

// ── Error paths ──

#[test]
fn errors_on_struct_missing_field_type() {
    assert!(parse_source("struct Bad { name }").is_err());
}

#[test]
fn errors_on_enum_missing_body() {
    assert!(parse_source("enum Bad").is_err());
}

#[test]
fn errors_on_impl_missing_body() {
    assert!(parse_source("impl Foo").is_err());
}

#[test]
fn errors_on_unclosed_trait() {
    assert!(parse_source("trait Bad { fn foo(x: Int) -> Int ").is_err());
}
