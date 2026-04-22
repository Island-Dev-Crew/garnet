//! Expression parser tests — exercises all 11 Pratt precedence levels.

use garnet_parser::ast::{BinOp, Expr, FnMode, Item, UnOp};
use garnet_parser::parse_source;

/// Parse a single `def main() { <expr> }` wrapper and return the tail expression.
fn expr_of(src: &str) -> Expr {
    let wrapped = format!("def main() {{ {} }}", src);
    let m = parse_source(&wrapped).unwrap();
    match &m.items[0] {
        Item::Fn(f) => {
            assert_eq!(f.mode, FnMode::Managed);
            f.body
                .tail_expr
                .as_ref()
                .expect("expected tail expression")
                .as_ref()
                .clone()
        }
        _ => panic!("expected fn item"),
    }
}

#[test]
fn parses_integer_addition() {
    match expr_of("1 + 2") {
        Expr::Binary { op, .. } => assert_eq!(op, BinOp::Add),
        _ => panic!("expected binary"),
    }
}

#[test]
fn parses_precedence_mul_over_add() {
    // 1 + 2 * 3 should parse as 1 + (2 * 3)
    match expr_of("1 + 2 * 3") {
        Expr::Binary { op: BinOp::Add, rhs, .. } => match rhs.as_ref() {
            Expr::Binary { op: BinOp::Mul, .. } => {}
            _ => panic!("expected nested multiplication"),
        },
        _ => panic!("expected binary add at top"),
    }
}

#[test]
fn parses_parentheses_override_precedence() {
    // (1 + 2) * 3 should parse as (1 + 2) * 3
    match expr_of("(1 + 2) * 3") {
        Expr::Binary { op: BinOp::Mul, lhs, .. } => match lhs.as_ref() {
            Expr::Binary { op: BinOp::Add, .. } => {}
            _ => panic!("expected nested addition"),
        },
        _ => panic!("expected binary mul at top"),
    }
}

#[test]
fn parses_pipeline_at_lowest_precedence() {
    // x + 1 |> f should parse as (x + 1) |> f
    match expr_of("x + 1 |> f") {
        Expr::Binary { op: BinOp::Pipeline, lhs, .. } => match lhs.as_ref() {
            Expr::Binary { op: BinOp::Add, .. } => {}
            _ => panic!("expected addition as LHS of pipeline"),
        },
        _ => panic!("expected pipeline at top"),
    }
}

#[test]
fn parses_logical_and_or_not() {
    match expr_of("a and b or c") {
        Expr::Binary { op: BinOp::Or, .. } => {}
        _ => panic!("expected or at top (or > and)"),
    }
}

#[test]
fn parses_comparison() {
    match expr_of("x < 10") {
        Expr::Binary { op: BinOp::Lt, .. } => {}
        _ => panic!("expected lt"),
    }
}

#[test]
fn parses_range_exclusive() {
    match expr_of("0..10") {
        Expr::Binary { op: BinOp::Range, .. } => {}
        _ => panic!("expected range"),
    }
}

#[test]
fn parses_range_inclusive() {
    match expr_of("0...10") {
        Expr::Binary { op: BinOp::RangeInclusive, .. } => {}
        _ => panic!("expected inclusive range"),
    }
}

#[test]
fn parses_unary_neg() {
    match expr_of("-x") {
        Expr::Unary { op: UnOp::Neg, .. } => {}
        _ => panic!("expected unary neg"),
    }
}

#[test]
fn parses_postfix_question() {
    // `expr?` is the error-propagation operator
    match expr_of("foo()?") {
        Expr::Unary { op: UnOp::Question, .. } => {}
        _ => panic!("expected ? postfix"),
    }
}

#[test]
fn parses_method_call_chain() {
    match expr_of("data.compress().validate()") {
        Expr::Method { method, .. } => assert_eq!(method, "validate"),
        _ => panic!("expected method call at top"),
    }
}

#[test]
fn parses_field_access() {
    match expr_of("config.host") {
        Expr::Field { field, .. } => assert_eq!(field, "host"),
        _ => panic!("expected field"),
    }
}

#[test]
fn parses_index_expression() {
    match expr_of("arr[0]") {
        Expr::Index { .. } => {}
        _ => panic!("expected index"),
    }
}

#[test]
fn parses_path_expression() {
    match expr_of("Foo::Bar::baz") {
        Expr::Path(segs, _) => assert_eq!(
            segs,
            vec!["Foo".to_string(), "Bar".to_string(), "baz".to_string()]
        ),
        _ => panic!("expected path"),
    }
}

#[test]
fn parses_array_literal() {
    match expr_of("[1, 2, 3]") {
        Expr::Array { elements, .. } => assert_eq!(elements.len(), 3),
        _ => panic!("expected array"),
    }
}

// ── Error paths ──

#[test]
fn errors_on_unclosed_paren() {
    assert!(parse_source("def main() { (1 + 2 }").is_err());
}

#[test]
fn errors_on_missing_operand() {
    assert!(parse_source("def main() { 1 + }").is_err());
}

#[test]
fn errors_on_consecutive_operators() {
    assert!(parse_source("def main() { 1 + * 2 }").is_err());
}
