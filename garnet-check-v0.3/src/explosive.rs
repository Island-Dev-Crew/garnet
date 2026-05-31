//! S40 — explosive-operation / default-ceiling analysis (closes the v0.8
//! foundation band).
//!
//! Static identification of **unbounded / explosive operations** plus a
//! default-ceiling **policy**. Two unambiguous constructs are flagged:
//!   * `Stmt::Loop` — an *unconditional loop*. Static termination is undecidable,
//!     so every `loop` is flagged regardless of an internal `break`; declare
//!     `@bounded` to govern it.
//!   * `Expr::Spawn` — actor *fan-out*; declare `@fan_out` to govern it.
//!
//! The AST visitor is **compiler-exhaustive** (every `Stmt`/`Expr` variant is
//! matched and recursed) so no nested site is silently missed.
//!
//! Honest scope: this is static IDENTIFICATION + a default-ceiling POLICY.
//! Runtime ENFORCEMENT lowers to the S39 `@bounded` / Wasmtime-fuel path and is
//! deferred (wasmtime absent); no ceiling is faked here.

use garnet_parser::ast::{Annotation, Block, ClosureBody, Expr, Item, Module, Stmt};
use garnet_parser::token::Span;

/// Default iteration ceiling applied to an unconditional loop that declares no
/// `@bounded` budget (a policy constant — enforcement is the deferred wrap).
pub const DEFAULT_LOOP_CEILING: u64 = 10_000_000;
/// Default fan-out ceiling applied to a `spawn` that declares no `@fan_out`.
pub const DEFAULT_SPAWN_FANOUT: u64 = 1_024;

/// The kind of explosive operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplosiveKind {
    /// An unconditional `loop` (unbounded iteration).
    UnconditionalLoop,
    /// A `spawn` (unbounded actor fan-out).
    Spawn,
}

/// One explosive-operation site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExplosiveOp {
    pub kind: ExplosiveKind,
    pub span: Span,
}

/// Per-function explosive-operation report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnExplosiveReport {
    pub fn_name: String,
    pub ops: Vec<ExplosiveOp>,
    /// Function declares `@bounded(N)` — governs unconditional loops (fuel).
    pub has_bounded: bool,
    /// Function declares `@fan_out(K)` — governs spawn fan-out.
    pub has_fan_out: bool,
}

/// Identify explosive operations per top-level function, sorted by name.
/// Functions with no explosive operation are omitted.
pub fn explosive_ops(module: &Module) -> Vec<FnExplosiveReport> {
    let mut out: Vec<FnExplosiveReport> = Vec::new();
    for item in &module.items {
        let Item::Fn(f) = item else { continue };
        let mut ops = Vec::new();
        walk_block(&f.body, &mut ops);
        if ops.is_empty() {
            continue;
        }
        out.push(FnExplosiveReport {
            fn_name: f.name.clone(),
            ops,
            has_bounded: f
                .annotations
                .iter()
                .any(|a| matches!(a, Annotation::Bounded(..))),
            has_fan_out: f
                .annotations
                .iter()
                .any(|a| matches!(a, Annotation::FanOut(..))),
        });
    }
    out.sort_by(|a, b| a.fn_name.cmp(&b.fn_name));
    out
}

fn walk_block(b: &Block, out: &mut Vec<ExplosiveOp>) {
    for stmt in &b.stmts {
        walk_stmt(stmt, out);
    }
    if let Some(tail) = &b.tail_expr {
        walk_expr(tail, out);
    }
}

fn walk_stmt(s: &Stmt, out: &mut Vec<ExplosiveOp>) {
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
        Stmt::Loop { body, span } => {
            out.push(ExplosiveOp {
                kind: ExplosiveKind::UnconditionalLoop,
                span: *span,
            });
            walk_block(body, out);
        }
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

fn walk_expr(e: &Expr, out: &mut Vec<ExplosiveOp>) {
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
        Expr::Unary { expr, .. } | Expr::Cast { expr, .. } => walk_expr(expr, out),
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
        Expr::Spawn { expr, span } => {
            out.push(ExplosiveOp {
                kind: ExplosiveKind::Spawn,
                span: *span,
            });
            walk_expr(expr, out);
        }
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

    fn report(src: &str) -> Vec<FnExplosiveReport> {
        explosive_ops(&parse_source(src).expect("parses"))
    }

    #[test]
    fn flags_unconditional_loop() {
        let r = report("def f() { loop { break } }\n");
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].fn_name, "f");
        assert_eq!(r[0].ops.len(), 1);
        assert_eq!(r[0].ops[0].kind, ExplosiveKind::UnconditionalLoop);
    }

    #[test]
    fn flags_spawn() {
        let r = report("def f() { spawn g() }\n");
        assert_eq!(r[0].ops[0].kind, ExplosiveKind::Spawn);
    }

    #[test]
    fn detects_nested_sites_via_exhaustive_walk() {
        // A spawn nested in a call arg + a loop nested in an if — both found.
        let r = report("def f(c) { if c { loop { h(spawn g()) } } }\n");
        let kinds: Vec<_> = r[0].ops.iter().map(|o| o.kind).collect();
        assert!(kinds.contains(&ExplosiveKind::UnconditionalLoop));
        assert!(kinds.contains(&ExplosiveKind::Spawn));
    }

    #[test]
    fn governing_annotations_are_reported() {
        let r = report("@bounded(1000)\ndef f() { loop { break } }\n");
        assert!(r[0].has_bounded);
        assert!(!r[0].has_fan_out);
        let r = report("@fan_out(8)\ndef f() { spawn g() }\n");
        assert!(r[0].has_fan_out);
    }

    #[test]
    fn clean_function_has_no_report() {
        assert!(report("def f() { 1 + 2 }\n").is_empty());
    }
}
