//! S42 — over-catch analysis.
//!
//! A **catch-all `rescue`** — a `rescue` clause with *no exception type* — swallows
//! every exception, including unanticipated ones ("agents over-catch exceptions").
//! This collects those sites so `garnet check` can emit a non-fatal advisory
//! steering toward typed `Result` / typed rescues. See
//! `C_Language_Specification/GARNET_ERROR_POLICY.md`.
//!
//! The AST visitor is compiler-exhaustive (every `Stmt`/`Expr` variant matched +
//! recursed), so a `try`/`rescue` nested anywhere is found.

use garnet_parser::ast::{Block, ClosureBody, Expr, Item, Module, Stmt};
use garnet_parser::token::Span;

/// One catch-all `rescue` site (a `rescue` clause with no exception type).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverCatchSite {
    pub span: Span,
}

/// Collect catch-all `rescue` sites across all top-level functions, in source order.
pub fn overcatch_sites(module: &Module) -> Vec<OverCatchSite> {
    let mut out = Vec::new();
    for item in &module.items {
        if let Item::Fn(f) = item {
            walk_block(&f.body, &mut out);
        }
    }
    out
}

fn walk_block(b: &Block, out: &mut Vec<OverCatchSite>) {
    for stmt in &b.stmts {
        walk_stmt(stmt, out);
    }
    if let Some(tail) = &b.tail_expr {
        walk_expr(tail, out);
    }
}

fn walk_stmt(s: &Stmt, out: &mut Vec<OverCatchSite>) {
    match s {
        Stmt::Let(d) => walk_expr(&d.value, out),
        Stmt::Var(d) => walk_expr(&d.value, out),
        Stmt::Const(d) => walk_expr(&d.value, out),
        Stmt::Assign { target, value, .. } => {
            walk_expr(target, out);
            walk_expr(value, out);
        }
        Stmt::While {
            condition, body, ..
        } => {
            walk_expr(condition, out);
            walk_block(body, out);
        }
        Stmt::For { iter, body, .. } => {
            walk_expr(iter, out);
            walk_block(body, out);
        }
        Stmt::Loop { body, .. } => walk_block(body, out),
        Stmt::Break { value, .. }
        | Stmt::Return { value, .. }
        | Stmt::Yield { value, .. }
        | Stmt::Next { value, .. } => {
            if let Some(v) = value {
                walk_expr(v, out);
            }
        }
        Stmt::Continue { .. } => {}
        Stmt::Raise { value, .. } => walk_expr(value, out),
        Stmt::Expr(e) => walk_expr(e, out),
    }
}

fn walk_expr(e: &Expr, out: &mut Vec<OverCatchSite>) {
    match e {
        Expr::Int(..)
        | Expr::Float(..)
        | Expr::Bool(..)
        | Expr::Nil(..)
        | Expr::Str(..)
        | Expr::Symbol(..)
        | Expr::Ident(..)
        | Expr::Path(..) => {}
        Expr::Binary { lhs, rhs, .. } => {
            walk_expr(lhs, out);
            walk_expr(rhs, out);
        }
        Expr::Unary { expr, .. } | Expr::Cast { expr, .. } | Expr::Spawn { expr, .. } => {
            walk_expr(expr, out)
        }
        Expr::Call { callee, args, .. } => {
            walk_expr(callee, out);
            args.iter().for_each(|a| walk_expr(a, out));
        }
        Expr::Method { receiver, args, .. } => {
            walk_expr(receiver, out);
            args.iter().for_each(|a| walk_expr(a, out));
        }
        Expr::Field { receiver, .. } => walk_expr(receiver, out),
        Expr::Index {
            receiver, index, ..
        } => {
            walk_expr(receiver, out);
            walk_expr(index, out);
        }
        Expr::If {
            condition,
            then_block,
            elsif_clauses,
            else_block,
            ..
        } => {
            walk_expr(condition, out);
            walk_block(then_block, out);
            for (cond, block) in elsif_clauses {
                walk_expr(cond, out);
                walk_block(block, out);
            }
            if let Some(block) = else_block {
                walk_block(block, out);
            }
        }
        Expr::Match { subject, arms, .. } => {
            walk_expr(subject, out);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    walk_expr(guard, out);
                }
                walk_block(&arm.body, out);
            }
        }
        Expr::Try {
            body,
            rescues,
            ensure,
            ..
        } => {
            walk_block(body, out);
            for rescue in rescues {
                // A rescue with no exception TYPE is a catch-all (over-catch).
                if rescue.ty.is_none() {
                    out.push(OverCatchSite { span: rescue.span });
                }
                walk_block(&rescue.body, out);
            }
            if let Some(block) = ensure {
                walk_block(block, out);
            }
        }
        Expr::Closure { body, .. } => match body.as_ref() {
            ClosureBody::Block(block) => walk_block(block, out),
            ClosureBody::Expr(expr) => walk_expr(expr, out),
        },
        Expr::Array { elements, .. } => elements.iter().for_each(|el| walk_expr(el, out)),
        Expr::Map { entries, .. } => entries.iter().for_each(|(k, v)| {
            walk_expr(k, out);
            walk_expr(v, out);
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use garnet_parser::parse_source;

    fn sites(src: &str) -> Vec<OverCatchSite> {
        overcatch_sites(&parse_source(src).expect("parses"))
    }

    #[test]
    fn flags_catch_all_rescue() {
        // `rescue e { ... }` with no type is a catch-all.
        let s = sites("def f() { try { g() } rescue e { 0 } }\n");
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn typed_rescue_is_not_flagged() {
        let s = sites("def f() { try { g() } rescue e: IoError { 0 } }\n");
        assert!(s.is_empty(), "a typed rescue is not an over-catch");
    }

    #[test]
    fn no_try_no_sites() {
        assert!(sites("def f() { 1 + 2 }\n").is_empty());
    }
}
