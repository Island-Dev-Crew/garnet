//! Control flow parser tests — if/elsif/else, match, try/rescue/ensure.

use garnet_parser::ast::{Expr, Item};
use garnet_parser::parse_source;

fn expr_of(src: &str) -> Expr {
    let wrapped = format!("def main() {{ {} }}", src);
    let m = parse_source(&wrapped).unwrap();
    match &m.items[0] {
        Item::Fn(f) => f
            .body
            .tail_expr
            .as_ref()
            .expect("expected tail expression")
            .as_ref()
            .clone(),
        _ => panic!("expected fn"),
    }
}

#[test]
fn parses_simple_if_expr() {
    match expr_of("if x { 1 } else { 2 }") {
        Expr::If {
            else_block: Some(_),
            ..
        } => {}
        _ => panic!("expected if-else"),
    }
}

#[test]
fn parses_if_without_else() {
    match expr_of("if x { 1 }") {
        Expr::If {
            else_block: None, ..
        } => {}
        _ => panic!("expected if without else"),
    }
}

#[test]
fn parses_if_elsif_else() {
    match expr_of("if a { 1 } elsif b { 2 } elsif c { 3 } else { 4 }") {
        Expr::If {
            elsif_clauses,
            else_block: Some(_),
            ..
        } => assert_eq!(elsif_clauses.len(), 2),
        _ => panic!("expected if with elsif"),
    }
}

#[test]
fn parses_match_with_literal_patterns() {
    match expr_of("match x { 1 => :one, 2 => :two, _ => :other, }") {
        Expr::Match { arms, .. } => assert_eq!(arms.len(), 3),
        _ => panic!("expected match"),
    }
}

#[test]
fn parses_match_with_enum_patterns() {
    match expr_of("match result { Ok(v) => v, Err(e) => 0, }") {
        Expr::Match { arms, .. } => assert_eq!(arms.len(), 2),
        _ => panic!("expected match"),
    }
}

#[test]
fn parses_match_with_path_enum_pattern() {
    match expr_of("match r { Result::Ok(v) => v, Result::Err(e) => 0, }") {
        Expr::Match { arms, .. } => assert_eq!(arms.len(), 2),
        _ => panic!("expected match"),
    }
}

#[test]
fn parses_match_with_tuple_pattern() {
    match expr_of("match pt { (0, 0) => :origin, (x, y) => :other, }") {
        Expr::Match { arms, .. } => assert_eq!(arms.len(), 2),
        _ => panic!("expected match"),
    }
}

#[test]
fn parses_match_with_guard() {
    match expr_of("match r { Err(e) if e.retryable? => 1, Err(e) => 0, Ok(v) => v, }") {
        Expr::Match { arms, .. } => {
            assert!(arms[0].guard.is_some());
            assert!(arms[1].guard.is_none());
        }
        _ => panic!("expected match"),
    }
}

#[test]
fn parses_match_with_symbol_pattern() {
    match expr_of("match s { :ok => 1, :err => 0, _ => -1, }") {
        Expr::Match { arms, .. } => assert_eq!(arms.len(), 3),
        _ => panic!("expected match"),
    }
}

#[test]
fn parses_match_with_wildcard() {
    match expr_of("match v { _ => 0, }") {
        Expr::Match { arms, .. } => assert_eq!(arms.len(), 1),
        _ => panic!("expected match"),
    }
}

#[test]
fn parses_try_with_rescue() {
    match expr_of("try { foo() } rescue e { 0 }") {
        Expr::Try { rescues, .. } => assert_eq!(rescues.len(), 1),
        _ => panic!("expected try"),
    }
}

#[test]
fn parses_try_with_typed_rescue() {
    match expr_of("try { foo() } rescue e: NetworkError { 0 }") {
        Expr::Try { rescues, .. } => {
            assert_eq!(rescues.len(), 1);
            assert!(rescues[0].ty.is_some());
        }
        _ => panic!("expected try"),
    }
}

#[test]
fn parses_try_with_multiple_rescues_and_ensure() {
    match expr_of(
        "try { foo() } rescue e: NetworkError { 1 } rescue e: ParseError { 2 } ensure { cleanup() }",
    ) {
        Expr::Try { rescues, ensure: Some(_), .. } => assert_eq!(rescues.len(), 2),
        _ => panic!("expected try with ensure"),
    }
}

// ── Error paths ──

#[test]
fn errors_on_if_without_block() {
    assert!(parse_source("def main() { if x 1 else 2 }").is_err());
}

#[test]
fn errors_on_match_without_fat_arrow() {
    assert!(parse_source("def main() { match x { 1 => 1, 2 2, } }").is_err());
}

#[test]
fn errors_on_try_without_block() {
    assert!(parse_source("def main() { try foo() rescue e { 0 } }").is_err());
}
