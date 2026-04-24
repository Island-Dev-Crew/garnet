//! Borrow-checker semantic pass (Rung 4 completion, simple form).
//!
//! Tracks linear-resource discipline for safe-mode functions:
//!
//! 1. **Move tracking.** When a binding is passed to a callee parameter
//!    annotated `own`, the binding is recorded as moved. Any subsequent use
//!    of that binding within the same scope produces a `use-after-move`
//!    diagnostic.
//! 2. **Aliasing-XOR-mutation.** Within a single expression, the same
//!    binding cannot appear as both a `mut` (exclusive) argument and any
//!    other argument.
//! 3. **Re-assign rebinds.** A `let mut name = expr` re-introduces `name`
//!    as a fresh, owned binding (overwriting any prior moved state).
//!
//! Limitations of this first cut (the v0.4 production checker will lift
//! them):
//! - Only direct calls to top-level `fn` items in the same module are
//!   tracked. Closures, method calls, and impl-block dispatch are skipped.
//! - Only simple identifier arguments are tracked. Field projections and
//!   index expressions are conservatively treated as non-moves.
//! - No flow analysis across `if`/`match` branches; each branch is checked
//!   independently against the entry environment.

use garnet_parser::ast::{Expr, FnDef, FnMode, Item, Module, Ownership, Pattern, Stmt};
use std::collections::HashMap;

use crate::CheckError;

/// One recorded move site, used by the diagnostic that names both halves.
#[derive(Debug, Clone)]
pub struct MoveRecord {
    pub binding: String,
    pub callee: String,
}

/// Map from function name → ordered ownership kinds for its parameters.
type SignatureTable = HashMap<String, Vec<Option<Ownership>>>;

/// Run the borrow checker on a parsed module. Returns any move/aliasing
/// diagnostics found in safe-mode (`fn`) bodies.
pub fn check_borrows(module: &Module) -> Vec<CheckError> {
    let signatures = collect_signatures(module);
    let mut diags = Vec::new();
    for item in &module.items {
        if let Item::Fn(f) = item {
            if effective_safe(module.safe, f) {
                check_fn_body(f, &signatures, &mut diags);
            }
        }
    }
    diags
}

fn effective_safe(module_safe: bool, f: &FnDef) -> bool {
    module_safe || f.mode == FnMode::Safe
}

fn collect_signatures(module: &Module) -> SignatureTable {
    let mut table = SignatureTable::new();
    for item in &module.items {
        if let Item::Fn(f) = item {
            let kinds: Vec<Option<Ownership>> = f.params.iter().map(|p| p.ownership).collect();
            table.insert(f.name.clone(), kinds);
        }
    }
    table
}

#[derive(Default)]
struct Env {
    /// Set of bindings that have been moved out of and may not be used.
    moved: HashMap<String, MoveRecord>,
}

impl Env {
    fn record_move(&mut self, binding: &str, callee: &str) {
        self.moved.insert(
            binding.to_string(),
            MoveRecord {
                binding: binding.to_string(),
                callee: callee.to_string(),
            },
        );
    }

    fn rebind(&mut self, binding: &str) {
        self.moved.remove(binding);
    }

    fn is_moved(&self, binding: &str) -> Option<&MoveRecord> {
        self.moved.get(binding)
    }
}

fn check_fn_body(f: &FnDef, sigs: &SignatureTable, diags: &mut Vec<CheckError>) {
    let mut env = Env::default();
    // Pre-bind the function's parameters as live (not moved).
    for p in &f.params {
        env.rebind(&p.name);
    }
    for stmt in &f.body.stmts {
        check_stmt(stmt, &mut env, sigs, &f.name, diags);
    }
    if let Some(tail) = &f.body.tail_expr {
        check_expr(tail, &mut env, sigs, &f.name, diags);
    }
}

fn check_stmt(
    stmt: &Stmt,
    env: &mut Env,
    sigs: &SignatureTable,
    fn_name: &str,
    diags: &mut Vec<CheckError>,
) {
    match stmt {
        Stmt::Let(decl) => {
            check_expr(&decl.value, env, sigs, fn_name, diags);
            env.rebind(&decl.name);
        }
        Stmt::Var(decl) => {
            check_expr(&decl.value, env, sigs, fn_name, diags);
            env.rebind(&decl.name);
        }
        Stmt::Const(decl) => {
            check_expr(&decl.value, env, sigs, fn_name, diags);
            env.rebind(&decl.name);
        }
        Stmt::Assign { target, value, .. } => {
            check_expr(value, env, sigs, fn_name, diags);
            if let Expr::Ident(name, _) = target {
                env.rebind(name);
            }
        }
        Stmt::While {
            condition, body, ..
        } => {
            check_expr(condition, env, sigs, fn_name, diags);
            for s in &body.stmts {
                check_stmt(s, env, sigs, fn_name, diags);
            }
            if let Some(tail) = &body.tail_expr {
                check_expr(tail, env, sigs, fn_name, diags);
            }
        }
        Stmt::For {
            iter, body, var, ..
        } => {
            check_expr(iter, env, sigs, fn_name, diags);
            env.rebind(var);
            for s in &body.stmts {
                check_stmt(s, env, sigs, fn_name, diags);
            }
            if let Some(tail) = &body.tail_expr {
                check_expr(tail, env, sigs, fn_name, diags);
            }
        }
        Stmt::Loop { body, .. } => {
            for s in &body.stmts {
                check_stmt(s, env, sigs, fn_name, diags);
            }
            if let Some(tail) = &body.tail_expr {
                check_expr(tail, env, sigs, fn_name, diags);
            }
        }
        Stmt::Break { value, .. } | Stmt::Return { value, .. } => {
            if let Some(e) = value {
                check_expr(e, env, sigs, fn_name, diags);
            }
        }
        Stmt::Raise { value, .. } => check_expr(value, env, sigs, fn_name, diags),
        Stmt::Continue { .. } => {}
        Stmt::Expr(e) => check_expr(e, env, sigs, fn_name, diags),
    }
}

fn check_expr(
    expr: &Expr,
    env: &mut Env,
    sigs: &SignatureTable,
    fn_name: &str,
    diags: &mut Vec<CheckError>,
) {
    match expr {
        Expr::Ident(name, _) => {
            if let Some(rec) = env.is_moved(name) {
                diags.push(CheckError::SafeModeViolation(format!(
                    "use-after-move: in `{fn_name}`, `{}` was moved into `{}` and cannot be used again",
                    rec.binding, rec.callee
                )));
            }
        }
        Expr::Call { callee, args, .. } => {
            // First evaluate every argument expression for inner moves.
            for a in args {
                check_expr(a, env, sigs, fn_name, diags);
            }
            // Then resolve the callee and apply ownership to identifier args.
            if let Expr::Ident(callee_name, _) = callee.as_ref() {
                if let Some(kinds) = sigs.get(callee_name) {
                    detect_aliasing_violations(callee_name, args, kinds, fn_name, diags);
                    for (arg, kind) in args.iter().zip(kinds.iter()) {
                        if matches!(kind, Some(Ownership::Own)) {
                            if let Expr::Ident(name, _) = arg {
                                env.record_move(name, callee_name);
                            }
                        }
                    }
                }
            } else {
                check_expr(callee, env, sigs, fn_name, diags);
            }
        }
        Expr::Method { receiver, args, .. } => {
            check_expr(receiver, env, sigs, fn_name, diags);
            for a in args {
                check_expr(a, env, sigs, fn_name, diags);
            }
        }
        Expr::Field { receiver, .. } => {
            check_expr(receiver, env, sigs, fn_name, diags);
        }
        Expr::Index {
            receiver, index, ..
        } => {
            check_expr(receiver, env, sigs, fn_name, diags);
            check_expr(index, env, sigs, fn_name, diags);
        }
        Expr::Binary { lhs, rhs, .. } => {
            check_expr(lhs, env, sigs, fn_name, diags);
            check_expr(rhs, env, sigs, fn_name, diags);
        }
        Expr::Unary { expr, .. } => check_expr(expr, env, sigs, fn_name, diags),
        Expr::If {
            condition,
            then_block,
            elsif_clauses,
            else_block,
            ..
        } => {
            check_expr(condition, env, sigs, fn_name, diags);
            // Each branch is checked independently against a snapshot of
            // env. Conservative: if any branch moves a binding, the binding
            // is considered moved after the if.
            let snapshot = env.moved.clone();
            for s in &then_block.stmts {
                check_stmt(s, env, sigs, fn_name, diags);
            }
            if let Some(tail) = &then_block.tail_expr {
                check_expr(tail, env, sigs, fn_name, diags);
            }
            for (cond, block) in elsif_clauses {
                let mut alt_env = Env {
                    moved: snapshot.clone(),
                };
                check_expr(cond, &mut alt_env, sigs, fn_name, diags);
                for s in &block.stmts {
                    check_stmt(s, &mut alt_env, sigs, fn_name, diags);
                }
                if let Some(tail) = &block.tail_expr {
                    check_expr(tail, &mut alt_env, sigs, fn_name, diags);
                }
                env.moved.extend(alt_env.moved);
            }
            if let Some(b) = else_block {
                let mut alt_env = Env { moved: snapshot };
                for s in &b.stmts {
                    check_stmt(s, &mut alt_env, sigs, fn_name, diags);
                }
                if let Some(tail) = &b.tail_expr {
                    check_expr(tail, &mut alt_env, sigs, fn_name, diags);
                }
                env.moved.extend(alt_env.moved);
            }
        }
        Expr::Match { subject, arms, .. } => {
            check_expr(subject, env, sigs, fn_name, diags);
            let snapshot = env.moved.clone();
            for arm in arms {
                let mut arm_env = Env {
                    moved: snapshot.clone(),
                };
                bind_pattern(&arm.pattern, &mut arm_env);
                if let Some(g) = &arm.guard {
                    check_expr(g, &mut arm_env, sigs, fn_name, diags);
                }
                check_expr(&arm.body, &mut arm_env, sigs, fn_name, diags);
                env.moved.extend(arm_env.moved);
            }
        }
        Expr::Try {
            body,
            rescues,
            ensure,
            ..
        } => {
            for s in &body.stmts {
                check_stmt(s, env, sigs, fn_name, diags);
            }
            if let Some(tail) = &body.tail_expr {
                check_expr(tail, env, sigs, fn_name, diags);
            }
            for r in rescues {
                if let Some(name) = &r.name {
                    env.rebind(name);
                }
                for s in &r.body.stmts {
                    check_stmt(s, env, sigs, fn_name, diags);
                }
                if let Some(tail) = &r.body.tail_expr {
                    check_expr(tail, env, sigs, fn_name, diags);
                }
            }
            if let Some(e) = ensure {
                for s in &e.stmts {
                    check_stmt(s, env, sigs, fn_name, diags);
                }
                if let Some(tail) = &e.tail_expr {
                    check_expr(tail, env, sigs, fn_name, diags);
                }
            }
        }
        Expr::Closure { .. } | Expr::Spawn { .. } => {
            // Closure / spawn bodies are deferred to v0.4: they capture the
            // surrounding environment and require a richer flow analysis.
        }
        Expr::Array { elements, .. } => {
            for e in elements {
                check_expr(e, env, sigs, fn_name, diags);
            }
        }
        Expr::Map { entries, .. } => {
            for (k, v) in entries {
                check_expr(k, env, sigs, fn_name, diags);
                check_expr(v, env, sigs, fn_name, diags);
            }
        }
        Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::Bool(_, _)
        | Expr::Nil(_)
        | Expr::Str(_, _)
        | Expr::Symbol(_, _)
        | Expr::Path(_, _) => {}
    }
}

fn bind_pattern(pattern: &Pattern, env: &mut Env) {
    match pattern {
        Pattern::Ident(name, _) => env.rebind(name),
        Pattern::Tuple(items, _) => {
            for p in items {
                bind_pattern(p, env);
            }
        }
        Pattern::Enum(_, items, _) => {
            for p in items {
                bind_pattern(p, env);
            }
        }
        Pattern::Literal(_, _) | Pattern::Wildcard(_) | Pattern::Rest(_) => {}
    }
}

fn detect_aliasing_violations(
    callee: &str,
    args: &[Expr],
    kinds: &[Option<Ownership>],
    fn_name: &str,
    diags: &mut Vec<CheckError>,
) {
    // Find any binding that appears as a `mut` argument and at least once
    // somewhere else in the same call. That's the basic
    // aliasing-XOR-mutation rule: an exclusive borrow may not coexist with
    // any other reference to the same binding.
    let mut mut_names: Vec<&str> = Vec::new();
    let mut other_names: Vec<&str> = Vec::new();
    for (arg, kind) in args.iter().zip(kinds.iter()) {
        if let Expr::Ident(name, _) = arg {
            if matches!(kind, Some(Ownership::Mut)) {
                mut_names.push(name.as_str());
            } else {
                other_names.push(name.as_str());
            }
        }
    }
    for m in &mut_names {
        if other_names.contains(m) || mut_names.iter().filter(|n| n == &m).count() > 1 {
            diags.push(CheckError::SafeModeViolation(format!(
                "aliasing violation: in `{fn_name}`, `{}` is passed as `mut` to `{callee}` while another reference to the same binding is in flight",
                m
            )));
        }
    }
}
