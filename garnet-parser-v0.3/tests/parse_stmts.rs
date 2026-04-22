//! Statement parser tests — let/var/const, while/for/loop, break/continue/return/raise, assignments.

use garnet_parser::ast::{AssignOp, Item, Stmt};
use garnet_parser::parse_source;

fn stmts_of(src: &str) -> Vec<Stmt> {
    let wrapped = format!("def main() {{ {} }}", src);
    let m = parse_source(&wrapped).unwrap();
    match &m.items[0] {
        Item::Fn(f) => f.body.stmts.clone(),
        _ => panic!("expected fn item"),
    }
}

#[test]
fn parses_let_binding() {
    let stmts = stmts_of("let x = 42\nx");
    match &stmts[0] {
        Stmt::Let(ld) => {
            assert!(!ld.mutable);
            assert_eq!(ld.name, "x");
        }
        _ => panic!("expected let"),
    }
}

#[test]
fn parses_let_mut_binding() {
    let stmts = stmts_of("let mut counter = 0\ncounter");
    match &stmts[0] {
        Stmt::Let(ld) => {
            assert!(ld.mutable);
            assert_eq!(ld.name, "counter");
        }
        _ => panic!("expected let mut"),
    }
}

#[test]
fn parses_let_with_type_annotation() {
    let stmts = stmts_of("let x: Int = 42\nx");
    match &stmts[0] {
        Stmt::Let(ld) => assert!(ld.ty.is_some()),
        _ => panic!("expected let with type"),
    }
}

#[test]
fn parses_var_binding() {
    let stmts = stmts_of("var counter = 0\ncounter");
    match &stmts[0] {
        Stmt::Var(vd) => assert_eq!(vd.name, "counter"),
        _ => panic!("expected var"),
    }
}

#[test]
fn parses_assignment() {
    let stmts = stmts_of("let mut x = 0\nx = 5\nx");
    match &stmts[1] {
        Stmt::Assign { op, .. } => assert_eq!(*op, AssignOp::Eq),
        _ => panic!("expected assignment"),
    }
}

#[test]
fn parses_compound_assignment() {
    let stmts = stmts_of("let mut x = 0\nx += 1\nx");
    match &stmts[1] {
        Stmt::Assign { op, .. } => assert_eq!(*op, AssignOp::PlusEq),
        _ => panic!("expected compound assignment"),
    }
}

#[test]
fn parses_while_loop() {
    let stmts = stmts_of("let mut i = 0\nwhile i < 10 { i += 1 }\ni");
    match &stmts[1] {
        Stmt::While { .. } => {}
        _ => panic!("expected while"),
    }
}

#[test]
fn parses_for_loop() {
    let stmts = stmts_of("let mut sum = 0\nfor n in nums { sum += n }\nsum");
    match &stmts[1] {
        Stmt::For { var, .. } => assert_eq!(var, "n"),
        _ => panic!("expected for"),
    }
}

#[test]
fn parses_loop_infinite() {
    let stmts = stmts_of("loop { break }\n0");
    match &stmts[0] {
        Stmt::Loop { .. } => {}
        _ => panic!("expected loop"),
    }
}

#[test]
fn parses_break_with_value() {
    let stmts = stmts_of("loop { break 42 }\n0");
    // The break statement is inside the loop body, which is a Block.
    match &stmts[0] {
        Stmt::Loop { body, .. } => match &body.stmts[0] {
            Stmt::Break { value: Some(_), .. } => {}
            _ => panic!("expected break with value"),
        },
        _ => panic!("expected loop"),
    }
}

#[test]
fn parses_return_with_value() {
    let stmts = stmts_of("return 42\n0");
    match &stmts[0] {
        Stmt::Return { value: Some(_), .. } => {}
        _ => panic!("expected return with value"),
    }
}

#[test]
fn parses_continue() {
    let stmts =
        stmts_of("let mut i = 0\nwhile i < 10 { i += 1\ncontinue }\ni");
    match &stmts[1] {
        Stmt::While { body, .. } => {
            assert!(body.stmts.iter().any(|s| matches!(s, Stmt::Continue { .. })));
        }
        _ => panic!("expected while containing continue"),
    }
}

#[test]
fn parses_raise() {
    let stmts = stmts_of("raise MyError::new()\n0");
    match &stmts[0] {
        Stmt::Raise { .. } => {}
        _ => panic!("expected raise"),
    }
}
