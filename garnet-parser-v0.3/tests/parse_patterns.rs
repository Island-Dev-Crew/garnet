//! Pattern matching tests — all 6 pattern kinds (Mini-Spec §6.3).

use garnet_parser::ast::{Expr, Item, Pattern};
use garnet_parser::parse_source;

fn patterns_of(src: &str) -> Vec<Pattern> {
    let wrapped = format!("def main() {{ {} }}", src);
    let m = parse_source(&wrapped).unwrap();
    let tail = match &m.items[0] {
        Item::Fn(f) => f.body.tail_expr.as_ref().expect("expected tail").as_ref().clone(),
        _ => panic!("expected fn"),
    };
    match tail {
        Expr::Match { arms, .. } => arms.into_iter().map(|a| a.pattern).collect(),
        _ => panic!("expected match"),
    }
}

#[test]
fn parses_literal_int_pattern() {
    let pats = patterns_of("match x { 42 => 1, _ => 0, }");
    assert!(matches!(pats[0], Pattern::Literal(Expr::Int(42, _), _)));
}

#[test]
fn parses_literal_string_pattern() {
    let pats = patterns_of(r#"match x { "hello" => 1, _ => 0, }"#);
    assert!(matches!(pats[0], Pattern::Literal(Expr::Str(_, _), _)));
}

#[test]
fn parses_literal_symbol_pattern() {
    let pats = patterns_of("match x { :ok => 1, _ => 0, }");
    assert!(matches!(pats[0], Pattern::Literal(Expr::Symbol(_, _), _)));
}

#[test]
fn parses_literal_bool_pattern() {
    let pats = patterns_of("match x { true => 1, false => 0, }");
    assert!(matches!(pats[0], Pattern::Literal(Expr::Bool(true, _), _)));
    assert!(matches!(pats[1], Pattern::Literal(Expr::Bool(false, _), _)));
}

#[test]
fn parses_ident_pattern() {
    let pats = patterns_of("match x { v => v, }");
    assert!(matches!(pats[0], Pattern::Ident(_, _)));
}

#[test]
fn parses_wildcard_pattern() {
    let pats = patterns_of("match x { _ => 0, }");
    assert!(matches!(pats[0], Pattern::Wildcard(_)));
}

#[test]
fn parses_tuple_pattern() {
    let pats = patterns_of("match p { (a, b) => a, }");
    match &pats[0] {
        Pattern::Tuple(inner, _) => assert_eq!(inner.len(), 2),
        _ => panic!("expected tuple"),
    }
}

#[test]
fn parses_enum_pattern_with_fields() {
    let pats = patterns_of("match r { Ok(v) => v, Err(e) => 0, }");
    match &pats[0] {
        Pattern::Enum(path, inner, _) => {
            assert_eq!(path, &vec!["Ok".to_string()]);
            assert_eq!(inner.len(), 1);
        }
        _ => panic!("expected enum pattern"),
    }
}
