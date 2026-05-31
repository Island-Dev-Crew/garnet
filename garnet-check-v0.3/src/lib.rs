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
//! - Full non-lexical lifetime inference (NLL per Rust RFC 2094). The
//!   checker currently enforces the conservative reference-return elision
//!   subset from Mini-Spec §8.5.2.
//! - Trait coherence verification (Mini-Spec §11.5)
//! - Automatic error-model bridging code generation (§7.4)

pub mod audit;
pub mod borrow;
pub mod bounds;
pub mod capability_surface;
pub mod caps_diff;
pub mod caps_graph;
pub mod coherence;
pub mod concurrency;
pub mod explosive;
pub mod match_coverage;
pub mod overcatch;
pub mod stability;
pub mod suggest;

pub use audit::{AuditLog, BoundaryCall, BoundaryDirection};
pub use bounds::bounded_functions;
pub use capability_surface::{capability_surface, CapabilitySurface};
pub use caps_diff::{diff_caps, CapsDiff};
pub use caps_graph::{CapsReport, CapsViolation};
pub use concurrency::{concurrency_surface, ActorContract, ProtocolSig};
pub use explosive::{explosive_ops, ExplosiveKind, ExplosiveOp, FnExplosiveReport};
pub use overcatch::{overcatch_sites, OverCatchSite};

use garnet_parser::ast::{
    ActorDef, ActorItem, Annotation, Block, FnDef, FnMode, Item, Module, Ownership, Param, Stmt,
    TypeExpr,
};
use std::collections::BTreeSet;

/// Capability names the checker accepts that the parser still surfaces as
/// `Capability::Other(_)` because they postdate the parser's enum. S17
/// (v0.7) adds `env` (process-environment access for `std::env`). The
/// parser's built-in variants (fs, net, net_internal, time, proc, ffi, *)
/// are validated structurally and never reach this list.
const CHECKER_KNOWN_OTHER_CAPS: &[&str] = &["env"];

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
    /// S17 — `@stability` advisory at a primitive call site. NON-FATAL: it is
    /// deliberately excluded from [`CheckReport::ok`], so it never changes the
    /// exit code. The message carries its own severity word ("warning" for
    /// experimental/deprecated, "info" for frozen) per the Layer Policy §4
    /// enforcement table.
    #[error("{0}")]
    StabilityAdvice(String),
    /// S29 — `@stability` enforcement promoted to ERROR level. FATAL: included
    /// in [`CheckReport::ok`], so it flips the exit code. Only emitted when the
    /// opt-in `GARNET_STABILITY_ERRORS` mode is enabled (the Layer Policy §4
    /// "error-level enforcement is v0.8" line); the default stays warning-level
    /// via [`CheckError::StabilityAdvice`].
    #[error("{0}")]
    StabilityError(String),
    /// S42 — over-catch advisory. NON-FATAL (excluded from [`CheckReport::ok`]):
    /// a catch-all `rescue` (no exception type) swallows every exception; the
    /// advisory steers toward typed `Result` / typed rescues. See
    /// `C_Language_Specification/GARNET_ERROR_POLICY.md`.
    #[error("{0}")]
    OverCatch(String),
}

/// Presentation severity of a [`CheckError`] — the single source of truth shared
/// by the CLI structured diagnostics (S34) and the LSP (S44). Severity is a
/// distinct axis from *fatal-ness*: whether a finding changes the exit code is
/// decided by [`CheckReport::ok`], whereas severity governs how the finding is
/// surfaced (e.g. a red error vs. an editor hint). An advisory such as
/// [`CheckError::OverCatch`] is `Info` here yet still non-fatal in `ok`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl CheckError {
    /// The presentation severity of this diagnostic. Consumed by both the CLI
    /// (`garnet check --format json`) and the LSP, so an editor and the CLI
    /// agree on how each finding is surfaced.
    pub fn severity(&self) -> Severity {
        match self {
            CheckError::SafeModeViolation(_)
            | CheckError::AnnotationError(_)
            | CheckError::CapsCoverage { .. }
            | CheckError::StabilityError(_) => Severity::Error,
            CheckError::BoundaryNote(_) => Severity::Warning,
            CheckError::StabilityAdvice(_) | CheckError::OverCatch(_) => Severity::Info,
        }
    }

    /// The stable machine-readable code (`check.*`) for this diagnostic, shared
    /// by the CLI's structured diagnostics and the LSP `Diagnostic.code`.
    pub fn code(&self) -> &'static str {
        match self {
            CheckError::SafeModeViolation(_) => "check.safe_mode_violation",
            CheckError::BoundaryNote(_) => "check.boundary_note",
            CheckError::AnnotationError(_) => "check.annotation_error",
            CheckError::CapsCoverage { .. } => "check.caps_coverage",
            CheckError::StabilityAdvice(_) => "check.stability_advice",
            CheckError::StabilityError(_) => "check.stability_error",
            CheckError::OverCatch(_) => "check.over_catch",
        }
    }
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
                    | CheckError::StabilityError(_)
            )
        })
    }
}

/// Run all checks on a parsed module. This is the single public entry point.
pub fn check_module(module: &Module) -> CheckReport {
    let mut report = CheckReport::default();
    // S42: over-catch advisory (non-fatal) — a catch-all `rescue` swallows every
    // exception; steer toward typed Result / typed rescues. Never changes the
    // exit code (excluded from `CheckReport::ok`).
    for site in crate::overcatch::overcatch_sites(module) {
        report.errors.push(CheckError::OverCatch(format!(
            "catch-all rescue at {}..{} swallows every exception; prefer a typed Result or \
             name the rescue type (advisory)",
            site.span.start,
            site.span.start + site.span.len
        )));
    }
    let module_safe = module.safe;
    let nonsendable_types = collect_nonsendable_types(module);

    for item in &module.items {
        check_item(item, module_safe, &nonsendable_types, &mut report);
    }

    // Borrow-checker pass: layered on top of the syntactic checks. Only
    // produces diagnostics for safe-mode functions; managed-mode `def`
    // functions are skipped because ARC sharing is the contract there.
    report.errors.extend(borrow::check_borrows(module));

    // Conservative trait coherence: exact duplicate impls and orphan-rule
    // violations are rejected before richer generic overlap solving exists.
    report
        .errors
        .extend(coherence::check_trait_coherence(module));

    // Safe-mode finite-domain `match` coverage: a scoped, conservative slice
    // of Mini-Spec §6.3. This handles Bool and same-module enum subjects
    // without claiming full type inference or general exhaustiveness.
    report
        .errors
        .extend(match_coverage::check_match_coverage(module));

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

    // S17 — `@stability` advisories at primitive call sites. NON-FATAL:
    // `StabilityAdvice` is excluded from `ok()`, so these never change the
    // exit code; they surface as visible warnings/info via `garnet check`.
    report.errors.extend(stability::check_stability(module));

    report
}

fn check_item(
    item: &Item,
    module_safe: bool,
    nonsendable_types: &BTreeSet<String>,
    report: &mut CheckReport,
) {
    match item {
        Item::Fn(f) => check_fn(f, module_safe, report),
        Item::Module(m) => {
            let merged = module_safe || m.safe;
            for inner in &m.items {
                check_item(inner, merged, nonsendable_types, report);
            }
        }
        Item::Impl(impl_block) => {
            if module_safe && has_dynamic_annotation(&impl_block.annotations) {
                report.errors.push(CheckError::AnnotationError(
                    "@dynamic impl blocks are not permitted in @safe modules; use static trait dispatch"
                        .to_string(),
                ));
            }
            for method in &impl_block.methods {
                check_fn(method, module_safe, report);
            }
        }
        Item::Struct(struct_def)
            if module_safe && has_dynamic_annotation(&struct_def.annotations) =>
        {
            report.errors.push(CheckError::AnnotationError(format!(
                "@dynamic struct '{}' is not permitted in @safe modules; use trait + dyn Trait",
                struct_def.name
            )));
        }
        Item::Actor(actor) => check_actor_sendable(actor, nonsendable_types, report),
        _ => {}
    }
}

fn has_dynamic_annotation(annotations: &[Annotation]) -> bool {
    annotations
        .iter()
        .any(|ann| matches!(ann, Annotation::Dynamic(_)))
}

fn has_nonsendable_annotation(annotations: &[Annotation]) -> bool {
    annotations
        .iter()
        .any(|ann| matches!(ann, Annotation::NonSendable(_)))
}

fn contains_ref_type(ty: &TypeExpr) -> bool {
    match ty {
        TypeExpr::Ref { .. } => true,
        TypeExpr::Named { args, .. } => args.iter().any(contains_ref_type),
        TypeExpr::Fn { params, ret, .. } => {
            params.iter().any(contains_ref_type) || contains_ref_type(ret)
        }
        TypeExpr::Tuple { elements, .. } => elements.iter().any(contains_ref_type),
        TypeExpr::Dyn { trait_ty, .. } => contains_ref_type(trait_ty),
    }
}

fn contributes_input_lifetime(param: &Param) -> bool {
    matches!(
        param.ownership,
        Some(Ownership::Borrow | Ownership::Mut | Ownership::Ref)
    ) || param.ty.as_ref().is_some_and(contains_ref_type)
}

fn is_borrowed_self(param: &Param) -> bool {
    param.name == "self"
        && matches!(
            param.ownership,
            Some(Ownership::Borrow | Ownership::Mut | Ownership::Ref)
        )
}

fn collect_nonsendable_types(module: &Module) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    collect_nonsendable_types_from_items(&module.items, &mut names);
    names
}

fn collect_nonsendable_types_from_items(items: &[Item], names: &mut BTreeSet<String>) {
    for item in items {
        match item {
            Item::Struct(struct_def) if has_nonsendable_annotation(&struct_def.annotations) => {
                names.insert(struct_def.name.clone());
            }
            Item::Module(module) => collect_nonsendable_types_from_items(&module.items, names),
            _ => {}
        }
    }
}

fn check_actor_sendable(
    actor: &ActorDef,
    nonsendable_types: &BTreeSet<String>,
    report: &mut CheckReport,
) {
    if nonsendable_types.is_empty() {
        return;
    }
    for item in &actor.items {
        match item {
            ActorItem::Protocol(protocol) => check_actor_params_sendable(
                &actor.name,
                "protocol",
                &protocol.name,
                &protocol.params,
                nonsendable_types,
                report,
            ),
            ActorItem::Handler(handler) => check_actor_params_sendable(
                &actor.name,
                "handler",
                &handler.name,
                &handler.params,
                nonsendable_types,
                report,
            ),
            _ => {}
        }
    }
}

fn check_actor_params_sendable(
    actor_name: &str,
    boundary_kind: &str,
    boundary_name: &str,
    params: &[Param],
    nonsendable_types: &BTreeSet<String>,
    report: &mut CheckReport,
) {
    for param in params {
        if let Some(ty) = &param.ty {
            for name in nonsendable_type_names(ty, nonsendable_types) {
                report.errors.push(CheckError::AnnotationError(format!(
                    "@nonsendable type `{name}` cannot cross actor `{actor_name}` {boundary_kind} `{boundary_name}` via parameter `{}`",
                    param.name
                )));
            }
        }
    }
}

fn nonsendable_type_names(ty: &TypeExpr, nonsendable_types: &BTreeSet<String>) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    collect_nonsendable_type_names(ty, nonsendable_types, &mut names);
    names
}

fn collect_nonsendable_type_names(
    ty: &TypeExpr,
    nonsendable_types: &BTreeSet<String>,
    names: &mut BTreeSet<String>,
) {
    match ty {
        TypeExpr::Named { path, args, .. } => {
            if let Some(name) = path.last() {
                if nonsendable_types.contains(name) {
                    names.insert(name.clone());
                }
            }
            for arg in args {
                collect_nonsendable_type_names(arg, nonsendable_types, names);
            }
        }
        TypeExpr::Fn { params, ret, .. } => {
            for param in params {
                collect_nonsendable_type_names(param, nonsendable_types, names);
            }
            collect_nonsendable_type_names(ret, nonsendable_types, names);
        }
        TypeExpr::Tuple { elements, .. } => {
            for element in elements {
                collect_nonsendable_type_names(element, nonsendable_types, names);
            }
        }
        TypeExpr::Ref { inner, .. }
        | TypeExpr::Dyn {
            trait_ty: inner, ..
        } => {
            collect_nonsendable_type_names(inner, nonsendable_types, names);
        }
    }
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
            Annotation::Bounded(n, _) if *n <= 0 => {
                report.errors.push(CheckError::AnnotationError(format!(
                    "@bounded on '{}' must declare a positive fuel budget, got {}",
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
                        if !CHECKER_KNOWN_OTHER_CAPS.contains(&name.as_str()) {
                            report.errors.push(CheckError::AnnotationError(format!(
                                "function '{}' declares unknown capability '{}'; \
                                 known caps: fs, net, net_internal, time, proc, env, ffi, *",
                                f.name, name
                            )));
                        }
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
        check_lifetime_elision(f, report);
    }

    // Always walk the body to count call sites; only emit safe-mode
    // violations when the function is in effective-safe scope. Boundary call
    // sites are interesting in either direction so the safe/managed bridge
    // generator (Compiler Arch §5) knows where to insert adapters.
    walk_block_for_safe_violations(&f.body, &f.name, report, effective_safe);
}

fn check_lifetime_elision(f: &FnDef, report: &mut CheckReport) {
    let Some(return_ty) = &f.return_ty else {
        return;
    };
    if !contains_ref_type(return_ty) {
        return;
    }

    let borrowed_inputs = f
        .params
        .iter()
        .filter(|p| contributes_input_lifetime(p))
        .count();
    if borrowed_inputs == 0 {
        report.errors.push(CheckError::SafeModeViolation(format!(
            "missing lifetime specifier: safe function '{}' returns a reference but has no borrowed input lifetime to tie it to",
            f.name
        )));
        return;
    }

    let has_borrowed_self = f.params.iter().any(is_borrowed_self);
    if borrowed_inputs > 1 && !has_borrowed_self {
        report.errors.push(CheckError::SafeModeViolation(format!(
            "missing lifetime specifier: safe function '{}' returns a reference with multiple borrowed inputs; write an explicit lifetime once lifetime syntax is enabled",
            f.name
        )));
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
            Stmt::Yield { value, .. } | Stmt::Next { value, .. } => {
                if let Some(value) = value {
                    walk_expr_for_safe_violations(value, fn_name, report, effective_safe);
                }
            }
            Stmt::While { body, .. } | Stmt::For { body, .. } | Stmt::Loop { body, .. } => {
                walk_block_for_safe_violations(body, fn_name, report, effective_safe);
            }
            Stmt::Expr(e) => walk_expr_for_safe_violations(e, fn_name, report, effective_safe),
            _ => {}
        }
    }
}

fn walk_block_for_safe_violations(
    block: &Block,
    fn_name: &str,
    report: &mut CheckReport,
    effective_safe: bool,
) {
    walk_stmts_for_safe_violations(&block.stmts, fn_name, report, effective_safe);
    if let Some(tail) = &block.tail_expr {
        walk_expr_for_safe_violations(tail, fn_name, report, effective_safe);
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
            walk_block_for_safe_violations(then_block, fn_name, report, effective_safe);
            for (_, b) in elsif_clauses {
                walk_block_for_safe_violations(b, fn_name, report, effective_safe);
            }
            if let Some(b) = else_block {
                walk_block_for_safe_violations(b, fn_name, report, effective_safe);
            }
        }
        Match { arms, .. } => {
            for arm in arms {
                walk_block_for_safe_violations(&arm.body, fn_name, report, effective_safe);
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
    fn severity_and_code_are_canonical() {
        // S44: this mapping is the single source of truth for the CLI structured
        // diagnostics and the LSP. If a variant changes severity/code, update it
        // here once and both consumers follow.
        use CheckError::*;
        let cases: Vec<(CheckError, Severity, &str)> = vec![
            (
                SafeModeViolation("x".into()),
                Severity::Error,
                "check.safe_mode_violation",
            ),
            (
                BoundaryNote("x".into()),
                Severity::Warning,
                "check.boundary_note",
            ),
            (
                AnnotationError("x".into()),
                Severity::Error,
                "check.annotation_error",
            ),
            (
                CapsCoverage {
                    fn_name: "f".into(),
                    missing: "fs".into(),
                    via: "g".into(),
                },
                Severity::Error,
                "check.caps_coverage",
            ),
            (
                StabilityAdvice("x".into()),
                Severity::Info,
                "check.stability_advice",
            ),
            (
                StabilityError("x".into()),
                Severity::Error,
                "check.stability_error",
            ),
            (OverCatch("x".into()), Severity::Info, "check.over_catch"),
        ];
        for (err, sev, code) in cases {
            assert_eq!(err.severity(), sev, "severity for {err:?}");
            assert_eq!(err.code(), code, "code for {err:?}");
        }
    }

    #[test]
    fn error_severity_agrees_with_fatal_set() {
        // The two axes must stay aligned: an Error-severity finding is fatal
        // (flips `ok`), and the intentional advisories are non-Error + non-fatal.
        use CheckError::*;
        let fatal: Vec<CheckError> = vec![
            SafeModeViolation("x".into()),
            AnnotationError("x".into()),
            CapsCoverage {
                fn_name: "f".into(),
                missing: "m".into(),
                via: "v".into(),
            },
            StabilityError("x".into()),
        ];
        for e in fatal {
            assert_eq!(e.severity(), Severity::Error);
            let r = CheckReport {
                errors: vec![e],
                ..Default::default()
            };
            assert!(!r.ok());
        }
        let advisory: Vec<CheckError> = vec![
            BoundaryNote("x".into()),
            StabilityAdvice("x".into()),
            OverCatch("x".into()),
        ];
        for e in advisory {
            assert_ne!(e.severity(), Severity::Error);
            let r = CheckReport {
                errors: vec![e],
                ..Default::default()
            };
            assert!(r.ok(), "advisory must not be fatal");
        }
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

    // ── S17: `@caps(env)` known-capability + non-fatal stability ────────

    #[test]
    fn env_is_a_known_capability() {
        // `@caps(env)` must NOT raise an "unknown capability" AnnotationError,
        // and it must cover the `env` requirement of `std::env::get`.
        let m = parse(
            r#"
            @caps(env)
            def main() {
                std::env::get("HOME")
            }
            "#,
        );
        let r = check_module(&m);
        assert!(
            !r.errors.iter().any(
                |e| matches!(e, CheckError::AnnotationError(m) if m.contains("unknown capability"))
            ),
            "env must be a known capability, got {:?}",
            r.errors
        );
        assert!(
            !r.errors
                .iter()
                .any(|e| matches!(e, CheckError::CapsCoverage { missing, .. } if missing == "env")),
            "@caps(env) should cover std::env::get, got {:?}",
            r.errors
        );
    }

    #[test]
    fn env_call_without_declaration_is_caps_violation() {
        let m = parse(
            r#"
            @caps()
            def main() {
                std::env::get("HOME")
            }
            "#,
        );
        let r = check_module(&m);
        assert!(
            r.errors
                .iter()
                .any(|e| matches!(e, CheckError::CapsCoverage { missing, .. } if missing == "env")),
            "calling std::env::get without @caps(env) must flag a missing `env` cap, got {:?}",
            r.errors
        );
    }

    #[test]
    fn truly_unknown_capability_still_rejected() {
        let m = parse(
            r#"
            @caps(quux)
            def main() { 0 }
            "#,
        );
        let r = check_module(&m);
        assert!(
            r.errors
                .iter()
                .any(|e| matches!(e, CheckError::AnnotationError(m) if m.contains("unknown capability") && m.contains("quux"))),
            "an unknown cap other than env must still be rejected, got {:?}",
            r.errors
        );
    }

    #[test]
    fn stability_advice_is_non_fatal() {
        // Calling an experimental primitive emits a StabilityAdvice but must
        // NOT flip ok() to false (warnings, not errors). `std::json::parse`
        // stays experimental after the S76 `core::*` promotion wave.
        let m = parse(
            r#"
            @caps()
            def main() {
                std::json::parse(s)
            }
            "#,
        );
        let r = check_module(&m);
        assert!(
            r.errors
                .iter()
                .any(|e| matches!(e, CheckError::StabilityAdvice(_))),
            "expected a StabilityAdvice for an experimental prim, got {:?}",
            r.errors
        );
        assert!(
            r.ok(),
            "stability advisories must be non-fatal (ok() stays true), got {:?}",
            r.errors
        );
    }

    #[test]
    fn s29_stability_error_is_fatal_advice_is_not() {
        // ok() classifies the S29 fatal variant as an error and the S17 advisory
        // as non-fatal — the mechanism the opt-in error mode relies on.
        let mut fatal = CheckReport::default();
        fatal
            .errors
            .push(CheckError::StabilityError("stability error: x".into()));
        assert!(!fatal.ok(), "StabilityError must flip ok() to false");

        let mut advisory = CheckReport::default();
        advisory
            .errors
            .push(CheckError::StabilityAdvice("stability warning: x".into()));
        assert!(advisory.ok(), "StabilityAdvice must stay non-fatal");
    }
}
