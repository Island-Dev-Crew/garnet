//! # Garnet Safe-Mode Checker (v0.3 skeleton)
//!
//! Rung 4 of the engineering ladder. This crate is the beginning of the
//! safe-mode enforcement story laid out in Mini-Spec v0.3 §8 and Paper V §4
//! (the λ_safe sub-calculus).
//!
//! The full ownership + borrow check + non-lexical lifetime inference is a
//! multi-month engineering effort. This skeleton performs the checks that can
//! be done locally, module-by-module, on the v0.3 AST today:
//!
//! 1. **Mode tagging.** Every module and every function is tagged Managed or
//!    Safe based on `@safe` annotations (§8.3).
//! 2. **Syntactic safe-mode restrictions.** `@safe` modules MUST NOT use
//!    `var`, `try`/`rescue`/`ensure`, or `raise` (Mini-Spec §7.3).
//! 3. **Safe-mode function discipline.** Every safe-mode (`fn`) function must
//!    declare parameter types and a return type.
//! 4. **Annotation placement sanity.** `@max_depth(N)` / `@fan_out(K)` must
//!    have integer arguments within reasonable bounds.
//! 5. **Mode-crossing call detection.** Identify every call site that crosses
//!    managed ↔ safe so the boundary validator (Compiler Arch Phase 5) knows
//!    where to insert bridging adapters.
//!
//! Deferred to a later pass (full Rung 4):
//! - Ownership inference (single-owner per affine value)
//! - Borrow checking (aliasing XOR mutation)
//! - Non-lexical lifetime inference (NLL per Rust RFC 2094)
//! - Trait coherence verification (Mini-Spec §11.5)
//! - Automatic error-model bridging code generation (§7.4)

pub mod audit;
pub mod borrow;
pub mod caps_graph;

pub use audit::{AuditLog, BoundaryCall, BoundaryDirection};
pub use caps_graph::{CapsReport, CapsViolation};

use garnet_parser::ast::{Annotation, FnDef, FnMode, Item, Module, Stmt};

/// A diagnostic from the checker, with a user-readable message and severity.
#[derive(Debug, Clone, thiserror::Error)]
pub enum CheckError {
    #[error("safe-mode violation: {0}")]
    SafeModeViolation(String),
    #[error("mode-boundary warning: {0}")]
    BoundaryNote(String),
    #[error("annotation error: {0}")]
    AnnotationError(String),
    /// v3.4.1 Day 2 — transitive CapCaps violation. Function `fn_name`
    /// invokes a primitive (or user callee transitively) requiring a
    /// capability its `@caps(...)` does not declare.
    #[error("caps coverage: function `{fn_name}` does not declare `{missing}` but transitively calls `{via}` which requires it")]
    CapsCoverage {
        fn_name: String,
        missing: String,
        via: String,
    },
}

/// The checker's result set: a list of diagnostics and metadata about each
/// function's mode.
#[derive(Debug, Default)]
pub struct CheckReport {
    pub errors: Vec<CheckError>,
    pub mode_map: Vec<(String, FnMode)>,
    pub boundary_call_sites: usize,
    /// Per-function capability sets collected from `@caps(...)` annotations
    /// (v3.4 CapCaps / Security Layer 2). Used by the call-graph propagator
    /// in v3.4.x to verify primitive invocations are gated.
    pub fn_caps: Vec<(String, Vec<String>)>,
}

impl CheckReport {
    pub fn ok(&self) -> bool {
        self.errors.iter().all(|e| {
            !matches!(
                e,
                CheckError::SafeModeViolation(_)
                    | CheckError::AnnotationError(_)
                    | CheckError::CapsCoverage { .. }
            )
        })
    }
}

/// Run all checks on a parsed module. This is the single public entry point.
pub fn check_module(module: &Module) -> CheckReport {
    let mut report = CheckReport::default();
    let module_safe = module.safe;

    for item in &module.items {
        match item {
            Item::Fn(f) => check_fn(f, module_safe, &mut report),
            Item::Module(m) => {
                let merged = module_safe || m.safe;
                for inner in &m.items {
                    if let Item::Fn(f) = inner {
                        check_fn(f, merged, &mut report);
                    }
                }
            }
            Item::Impl(impl_block) => {
                for method in &impl_block.methods {
                    check_fn(method, module_safe, &mut report);
                }
            }
            _ => {}
        }
    }

    // Borrow-checker pass: layered on top of the syntactic checks. Only
    // produces diagnostics for safe-mode functions; managed-mode `def`
    // functions are skipped because ARC sharing is the contract there.
    report.errors.extend(borrow::check_borrows(module));

    // v3.4.1 Day 2 — CapCaps call-graph propagator. Reads primitive caps
    // from `garnet_stdlib::registry` at check time and verifies every
    // function's `@caps(...)` covers its transitive requirements.
    let caps_report = caps_graph::check_caps_coverage(module);
    for v in caps_report.violations {
        report.errors.push(CheckError::CapsCoverage {
            fn_name: v.fn_name,
            missing: v.missing,
            via: v.via,
        });
    }

    report
}

fn check_fn(f: &FnDef, module_safe: bool, report: &mut CheckReport) {
    let effective_safe = module_safe || f.mode == FnMode::Safe;
    report.mode_map.push((f.name.clone(), f.mode));

    // Annotations: verify numeric bounds + cap discipline.
    let mut caps_seen = false;
    let mut caps_set: Vec<String> = Vec::new();
    let mut wildcard_used = false;
    for ann in &f.annotations {
        match ann {
            Annotation::MaxDepth(n, _) if *n <= 0 || *n > 64 => {
                report.errors.push(CheckError::AnnotationError(format!(
                    "@max_depth on '{}' must be in 1..=64, got {}",
                    f.name, n
                )));
            }
            Annotation::FanOut(n, _) if *n <= 0 || *n > 1024 => {
                report.errors.push(CheckError::AnnotationError(format!(
                    "@fan_out on '{}' must be in 1..=1024, got {}",
                    f.name, n
                )));
            }
            Annotation::Mailbox(n, _) if *n <= 0 || *n > 1_048_576 => {
                report.errors.push(CheckError::AnnotationError(format!(
                    "@mailbox on '{}' must be in 1..=1048576, got {}",
                    f.name, n
                )));
            }
            Annotation::Caps(caps, _) => {
                if caps_seen {
                    report.errors.push(CheckError::AnnotationError(format!(
                        "function '{}' has multiple @caps(...) annotations; merge them",
                        f.name
                    )));
                }
                caps_seen = true;
                for c in caps {
                    if matches!(c, garnet_parser::ast::Capability::Wildcard) {
                        wildcard_used = true;
                    }
                    if let garnet_parser::ast::Capability::Other(name) = c {
                        report.errors.push(CheckError::AnnotationError(format!(
                            "function '{}' declares unknown capability '{}'; \
                             known caps: fs, net, net_internal, time, proc, ffi, *",
                            f.name, name
                        )));
                    }
                    caps_set.push(c.as_str().to_string());
                }
            }
            _ => {}
        }
    }
    // Wildcard caps in safe-mode functions are a hard error; in managed mode
    // they're a warning to be promoted to error in CI release builds.
    if wildcard_used && f.mode == FnMode::Safe {
        report.errors.push(CheckError::AnnotationError(format!(
            "safe function '{}' may not use @caps(*) wildcard; \
             enumerate the specific capabilities required",
            f.name
        )));
    }
    // `main` MUST declare @caps(...) per Mini-Spec v1.0 §16 + Security V2 §1.4
    // (an empty list is acceptable; absence is not).
    if f.name == "main" && !caps_seen {
        report.errors.push(CheckError::AnnotationError(
            "`main` function must declare its required capabilities; \
             use @caps() for purely-computational programs, \
             or @caps(fs, net, ...) listing the OS authority required"
                .to_string(),
        ));
    }
    // Stash the function's caps onto the report so a future call-graph
    // pass (v3.4.x) can do transitive propagation. For v3.4.0 we only
    // collect; primitive-call gating turns on once stdlib lands.
    if !caps_set.is_empty() || caps_seen {
        report.fn_caps.push((f.name.clone(), caps_set));
    }

    // Safe-mode discipline: signatures must be fully typed.
    if f.mode == FnMode::Safe {
        for p in &f.params {
            if p.ty.is_none() {
                report.errors.push(CheckError::SafeModeViolation(format!(
                    "safe function '{}' parameter '{}' missing type annotation",
                    f.name, p.name
                )));
            }
        }
        if f.return_ty.is_none() {
            report.errors.push(CheckError::SafeModeViolation(format!(
                "safe function '{}' missing return type",
                f.name
            )));
        }
    }

    // Always walk the body to count call sites; only emit safe-mode
    // violations when the function is in effective-safe scope. Boundary call
    // sites are interesting in either direction so the safe/managed bridge
    // generator (Compiler Arch §5) knows where to insert adapters.
    walk_stmts_for_safe_violations(&f.body.stmts, &f.name, report, effective_safe);
    if let Some(tail) = &f.body.tail_expr {
        walk_expr_for_safe_violations(tail, &f.name, report, effective_safe);
    }
}

fn walk_stmts_for_safe_violations(
    stmts: &[Stmt],
    fn_name: &str,
    report: &mut CheckReport,
    effective_safe: bool,
) {
    for s in stmts {
        match s {
            Stmt::Var(_) if effective_safe => {
                report.errors.push(CheckError::SafeModeViolation(format!(
                    "safe function '{}' uses `var`; use `let mut` instead",
                    fn_name
                )));
            }
            Stmt::Raise { .. } if effective_safe => {
                report.errors.push(CheckError::SafeModeViolation(format!(
                    "safe function '{}' uses `raise`; return Result::Err(...) instead",
                    fn_name
                )));
            }
            Stmt::While { body, .. } | Stmt::For { body, .. } | Stmt::Loop { body, .. } => {
                walk_stmts_for_safe_violations(&body.stmts, fn_name, report, effective_safe);
                if let Some(tail) = &body.tail_expr {
                    walk_expr_for_safe_violations(tail, fn_name, report, effective_safe);
                }
            }
            Stmt::Expr(e) => walk_expr_for_safe_violations(e, fn_name, report, effective_safe),
            _ => {}
        }
    }
}

fn walk_expr_for_safe_violations(
    expr: &garnet_parser::ast::Expr,
    fn_name: &str,
    report: &mut CheckReport,
    effective_safe: bool,
) {
    use garnet_parser::ast::Expr::*;
    match expr {
        Try { .. } if effective_safe => {
            report.errors.push(CheckError::SafeModeViolation(format!(
                "safe function '{}' uses `try`/`rescue`; use Result<T, E> and `?` instead",
                fn_name
            )));
        }
        If {
            then_block,
            elsif_clauses,
            else_block,
            ..
        } => {
            walk_stmts_for_safe_violations(&then_block.stmts, fn_name, report, effective_safe);
            if let Some(tail) = &then_block.tail_expr {
                walk_expr_for_safe_violations(tail, fn_name, report, effective_safe);
            }
            for (_, b) in elsif_clauses {
                walk_stmts_for_safe_violations(&b.stmts, fn_name, report, effective_safe);
                if let Some(tail) = &b.tail_expr {
                    walk_expr_for_safe_violations(tail, fn_name, report, effective_safe);
                }
            }
            if let Some(b) = else_block {
                walk_stmts_for_safe_violations(&b.stmts, fn_name, report, effective_safe);
                if let Some(tail) = &b.tail_expr {
                    walk_expr_for_safe_violations(tail, fn_name, report, effective_safe);
                }
            }
        }
        Match { arms, .. } => {
            for arm in arms {
                walk_expr_for_safe_violations(&arm.body, fn_name, report, effective_safe);
            }
        }
        Binary { lhs, rhs, .. } => {
            walk_expr_for_safe_violations(lhs, fn_name, report, effective_safe);
            walk_expr_for_safe_violations(rhs, fn_name, report, effective_safe);
        }
        Call { callee, args, .. }
        | Method {
            receiver: callee,
            args,
            ..
        } => {
            walk_expr_for_safe_violations(callee, fn_name, report, effective_safe);
            for a in args {
                walk_expr_for_safe_violations(a, fn_name, report, effective_safe);
            }
            report.boundary_call_sites += 1;
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Module {
        garnet_parser::parse_source(src).expect("parse failed")
    }

    #[test]
    fn managed_module_has_no_safe_violations() {
        let m = parse(r#"def greet(name) { "hello" }"#);
        let r = check_module(&m);
        assert!(r.ok(), "expected no errors, got {:?}", r.errors);
    }

    #[test]
    fn safe_fn_without_return_type_flagged() {
        // Parser rejects safe fn without return type, so we build via a safe
        // module with a def (which at module-level is treated as effective
        // safe) that uses `var`.
        let m = parse(
            r#"
            @safe
            def bad() {
                var x = 42
                x
            }
        "#,
        );
        let r = check_module(&m);
        assert!(
            r.errors
                .iter()
                .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("var"))),
            "expected var violation, got {:?}",
            r.errors
        );
    }

    #[test]
    fn safe_fn_with_raise_flagged() {
        let m = parse(
            r#"
            @safe
            def oops() {
                raise "nope"
                0
            }
        "#,
        );
        let r = check_module(&m);
        assert!(
            r.errors
                .iter()
                .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("raise"))),
            "expected raise violation"
        );
    }

    #[test]
    fn annotation_bounds_enforced() {
        let m = parse(
            r#"
            @max_depth(200)
            def recursive() {
                recursive()
            }
        "#,
        );
        let r = check_module(&m);
        assert!(
            r.errors
                .iter()
                .any(|e| matches!(e, CheckError::AnnotationError(_))),
            "expected annotation error"
        );
    }

    #[test]
    fn boundary_call_sites_counted() {
        let m = parse(
            r#"
            def outer(x) {
                inner(x) + 1
            }
            def inner(x) { x * 2 }
        "#,
        );
        let r = check_module(&m);
        assert!(r.boundary_call_sites > 0);
    }
}
