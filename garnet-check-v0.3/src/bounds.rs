//! S39 — `@bounded` resource-budget extraction.
//!
//! `@bounded(N)` declares a CPU / fuel budget of N Wasmtime-fuel units;
//! enforcement lowers to Wasmtime fuel metering (the wrap). This extracts the
//! DECLARED budgets per function — the surface a fuel-metering backend consumes
//! — sorted by name. S93 adds a conservative static verifier for safe /
//! `@bounded(...)` loops; it is not runtime fuel metering.

use garnet_parser::ast::{
    Annotation, AssignOp, BinOp, Block, ClosureBody, Expr, FnDef, FnMode, Item, Module, Stmt,
};
use std::collections::BTreeMap;

type IntFacts = BTreeMap<String, i64>;

/// Functions that declare an `@bounded(N)` fuel budget, as `(name, fuel)`,
/// sorted by function name. If a function carries multiple `@bounded`, the last
/// wins (the checker rejects nonsensical values separately).
pub fn bounded_functions(module: &Module) -> Vec<(String, i64)> {
    let mut out: Vec<(String, i64)> = Vec::new();
    for item in &module.items {
        let Item::Fn(f) = item else { continue };
        let mut budget: Option<i64> = None;
        for ann in &f.annotations {
            if let Annotation::Bounded(n, _) = ann {
                budget = Some(*n);
            }
        }
        if let Some(n) = budget {
            out.push((f.name.clone(), n));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Static S93 loop proof summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedLoopReport {
    pub checked_functions: usize,
    pub skipped_functions: usize,
    pub proven_loops: usize,
    pub uncheckable_loops: Vec<UncheckableLoop>,
}

impl BoundedLoopReport {
    pub fn ok(&self) -> bool {
        self.uncheckable_loops.is_empty()
    }
}

/// One loop the conservative static verifier could not bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UncheckableLoop {
    pub function: String,
    pub kind: &'static str,
    pub span_start: usize,
    pub span_len: usize,
    pub message: String,
}

/// Verify loops in the safe subset.
///
/// S93 intentionally checks only `fn`, `@safe` scope, and functions declaring
/// `@bounded(...)`. Managed functions outside that surface are skipped so the
/// slice does not pretend to be whole-language loop enforcement.
pub fn bounded_loop_report(module: &Module) -> BoundedLoopReport {
    let mut report = BoundedLoopReport {
        checked_functions: 0,
        skipped_functions: 0,
        proven_loops: 0,
        uncheckable_loops: Vec::new(),
    };
    check_items_for_bounded_loops(&module.items, module.safe, &mut report);
    report
}

fn check_items_for_bounded_loops(
    items: &[Item],
    module_safe: bool,
    report: &mut BoundedLoopReport,
) {
    for item in items {
        match item {
            Item::Fn(function) => check_function(function, module_safe, report),
            Item::Module(module) => {
                check_items_for_bounded_loops(&module.items, module_safe || module.safe, report);
            }
            Item::Impl(impl_block) => {
                for method in &impl_block.methods {
                    check_function(method, module_safe, report);
                }
            }
            _ => {}
        }
    }
}

fn check_function(function: &FnDef, module_safe: bool, report: &mut BoundedLoopReport) {
    if module_safe || function.mode == FnMode::Safe || has_bounded_annotation(function) {
        report.checked_functions += 1;
        let mut facts = IntFacts::new();
        check_block_with_facts(&function.body, &function.name, report, &mut facts);
    } else {
        report.skipped_functions += 1;
    }
}

fn has_bounded_annotation(function: &FnDef) -> bool {
    function
        .annotations
        .iter()
        .any(|annotation| matches!(annotation, Annotation::Bounded(_, _)))
}

fn check_block(block: &Block, function_name: &str, report: &mut BoundedLoopReport) {
    let mut facts = IntFacts::new();
    check_block_with_facts(block, function_name, report, &mut facts);
}

fn check_block_with_facts(
    block: &Block,
    function_name: &str,
    report: &mut BoundedLoopReport,
    facts: &mut IntFacts,
) {
    for statement in &block.stmts {
        check_stmt(statement, function_name, report, facts);
    }
    if let Some(tail) = &block.tail_expr {
        check_expr(tail, function_name, report);
    }
}

fn check_stmt(
    statement: &Stmt,
    function_name: &str,
    report: &mut BoundedLoopReport,
    facts: &mut IntFacts,
) {
    match statement {
        Stmt::Let(decl) => {
            check_expr(&decl.value, function_name, report);
            record_binding_fact(facts, &decl.name, &decl.value);
        }
        Stmt::Var(decl) => {
            check_expr(&decl.value, function_name, report);
            record_binding_fact(facts, &decl.name, &decl.value);
        }
        Stmt::Const(decl) => {
            check_expr(&decl.value, function_name, report);
            record_binding_fact(facts, &decl.name, &decl.value);
        }
        Stmt::Assign {
            target, op, value, ..
        } => {
            check_expr(target, function_name, report);
            check_expr(value, function_name, report);
            record_assignment_fact(facts, target, *op, value);
        }
        Stmt::While {
            condition,
            body,
            span,
        } => {
            if static_while_bound(condition, body, facts).is_some() {
                report.proven_loops += 1;
            } else {
                push_uncheckable(report, function_name, "while", span.start, span.len);
            }
            check_expr(condition, function_name, report);
            check_block(body, function_name, report);
            clear_assigned_facts(facts, body);
        }
        Stmt::For {
            iter,
            body,
            span,
            var,
        } => {
            if static_iter_bound(iter).is_some() || block_exits_before_next_turn(body) {
                report.proven_loops += 1;
            } else {
                push_uncheckable(report, function_name, "for", span.start, span.len);
            }
            check_expr(iter, function_name, report);
            check_block(body, function_name, report);
            facts.remove(var);
            clear_assigned_facts(facts, body);
        }
        Stmt::Loop { body, span } => {
            if block_exits_before_next_turn(body) {
                report.proven_loops += 1;
            } else {
                push_uncheckable(report, function_name, "loop", span.start, span.len);
            }
            check_block(body, function_name, report);
            clear_assigned_facts(facts, body);
        }
        Stmt::Break { value, .. }
        | Stmt::Return { value, .. }
        | Stmt::Yield { value, .. }
        | Stmt::Next { value, .. } => {
            if let Some(value) = value {
                check_expr(value, function_name, report);
            }
        }
        Stmt::Raise { value, .. } | Stmt::Expr(value) => {
            check_expr(value, function_name, report);
        }
        Stmt::Continue { .. } => {}
    }
}

fn check_expr(expr: &Expr, function_name: &str, report: &mut BoundedLoopReport) {
    match expr {
        Expr::Binary { lhs, rhs, .. } => {
            check_expr(lhs, function_name, report);
            check_expr(rhs, function_name, report);
        }
        Expr::Unary { expr, .. } | Expr::Cast { expr, .. } | Expr::Spawn { expr, .. } => {
            check_expr(expr, function_name, report);
        }
        Expr::Call { callee, args, .. } => {
            check_expr(callee, function_name, report);
            for arg in args {
                check_expr(arg, function_name, report);
            }
        }
        Expr::Method { receiver, args, .. } => {
            check_expr(receiver, function_name, report);
            for arg in args {
                check_expr(arg, function_name, report);
            }
        }
        Expr::Field { receiver, .. } => check_expr(receiver, function_name, report),
        Expr::Index {
            receiver, index, ..
        } => {
            check_expr(receiver, function_name, report);
            check_expr(index, function_name, report);
        }
        Expr::If {
            condition,
            then_block,
            elsif_clauses,
            else_block,
            ..
        } => {
            check_expr(condition, function_name, report);
            check_block(then_block, function_name, report);
            for (condition, block) in elsif_clauses {
                check_expr(condition, function_name, report);
                check_block(block, function_name, report);
            }
            if let Some(block) = else_block {
                check_block(block, function_name, report);
            }
        }
        Expr::Match { subject, arms, .. } => {
            check_expr(subject, function_name, report);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    check_expr(guard, function_name, report);
                }
                check_block(&arm.body, function_name, report);
            }
        }
        Expr::Try {
            body,
            rescues,
            ensure,
            ..
        } => {
            check_block(body, function_name, report);
            for rescue in rescues {
                check_block(&rescue.body, function_name, report);
            }
            if let Some(ensure) = ensure {
                check_block(ensure, function_name, report);
            }
        }
        Expr::Closure { body, .. } => match body.as_ref() {
            ClosureBody::Block(block) => check_block(block, function_name, report),
            ClosureBody::Expr(expr) => check_expr(expr, function_name, report),
        },
        Expr::Array { elements, .. } => {
            for element in elements {
                check_expr(element, function_name, report);
            }
        }
        Expr::Map { entries, .. } => {
            for (key, value) in entries {
                check_expr(key, function_name, report);
                check_expr(value, function_name, report);
            }
        }
        Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::Bool(_, _)
        | Expr::Nil(_)
        | Expr::Str(_, _)
        | Expr::Symbol(_, _)
        | Expr::Ident(_, _)
        | Expr::Path(_, _) => {}
    }
}

fn static_iter_bound(expr: &Expr) -> Option<i64> {
    match expr {
        Expr::Array { elements, .. } => i64::try_from(elements.len()).ok(),
        Expr::Binary {
            op: op @ (BinOp::Range | BinOp::RangeInclusive),
            lhs,
            rhs,
            ..
        } => literal_range_bound(lhs, rhs, *op == BinOp::RangeInclusive),
        _ => None,
    }
}

fn literal_range_bound(lhs: &Expr, rhs: &Expr, inclusive: bool) -> Option<i64> {
    let Expr::Int(start, _) = lhs else {
        return None;
    };
    let Expr::Int(end, _) = rhs else {
        return None;
    };
    if start > end {
        return Some(0);
    }
    let distance = end.checked_sub(*start)?;
    if inclusive {
        distance.checked_add(1)
    } else {
        Some(distance)
    }
}

fn static_while_bound(condition: &Expr, body: &Block, facts: &IntFacts) -> Option<i64> {
    if matches!(condition, Expr::Bool(false, _)) {
        return Some(0);
    }
    if block_exits_before_next_turn(body) {
        return Some(1);
    }
    literal_counter_while_bound(condition, body, facts)
}

fn literal_counter_while_bound(condition: &Expr, body: &Block, facts: &IntFacts) -> Option<i64> {
    let required_direction = counter_condition_direction(condition, facts)?;
    if required_direction.max_iterations_hint == 0 {
        return Some(0);
    }
    let step = counter_step_for_body(body, &required_direction.var)?;
    if step.signum() == required_direction.step_sign {
        Some(required_direction.max_iterations_hint)
    } else {
        None
    }
}

struct CounterCondition {
    var: String,
    step_sign: i64,
    max_iterations_hint: i64,
}

fn counter_condition_direction(condition: &Expr, facts: &IntFacts) -> Option<CounterCondition> {
    let Expr::Binary { op, lhs, rhs, .. } = condition else {
        return None;
    };

    if let (Expr::Ident(var, _), Expr::Int(limit, _)) = (lhs.as_ref(), rhs.as_ref()) {
        let start = *facts.get(var)?;
        return counter_condition_from_parts(var.clone(), start, *limit, *op, false);
    }

    if let (Expr::Int(limit, _), Expr::Ident(var, _)) = (lhs.as_ref(), rhs.as_ref()) {
        let start = *facts.get(var)?;
        return counter_condition_from_parts(var.clone(), start, *limit, *op, true);
    }

    None
}

fn counter_condition_from_parts(
    var: String,
    start: i64,
    limit: i64,
    op: BinOp,
    reversed: bool,
) -> Option<CounterCondition> {
    let condition_initially_true = if reversed {
        compare_ints(limit, start, op)?
    } else {
        compare_ints(start, limit, op)?
    };
    if !condition_initially_true {
        return Some(CounterCondition {
            var,
            step_sign: 1,
            max_iterations_hint: 0,
        });
    }

    let step_sign = match (reversed, op) {
        (false, BinOp::Lt | BinOp::LtEq) | (true, BinOp::Gt | BinOp::GtEq) => 1,
        (false, BinOp::Gt | BinOp::GtEq) | (true, BinOp::Lt | BinOp::LtEq) => -1,
        _ => return None,
    };
    Some(CounterCondition {
        var,
        step_sign,
        max_iterations_hint: 1,
    })
}

fn compare_ints(lhs: i64, rhs: i64, op: BinOp) -> Option<bool> {
    match op {
        BinOp::Lt => Some(lhs < rhs),
        BinOp::LtEq => Some(lhs <= rhs),
        BinOp::Gt => Some(lhs > rhs),
        BinOp::GtEq => Some(lhs >= rhs),
        _ => None,
    }
}

fn counter_step_for_body(body: &Block, var: &str) -> Option<i64> {
    let mut step = 0_i64;
    for statement in &body.stmts {
        if let Some(delta) = counter_step_for_statement(statement, var)? {
            step = step.checked_add(delta)?;
        }
    }
    (step != 0).then_some(step)
}

fn counter_step_for_statement(statement: &Stmt, var: &str) -> Option<Option<i64>> {
    match statement {
        Stmt::Assign {
            target: Expr::Ident(name, _),
            op,
            value,
            ..
        } if name == var => match (op, value) {
            (AssignOp::PlusEq, Expr::Int(delta, _)) if *delta > 0 => Some(Some(*delta)),
            (AssignOp::MinusEq, Expr::Int(delta, _)) if *delta > 0 => Some(Some(-*delta)),
            _ => None,
        },
        Stmt::Assign { target, .. } if target_is_ident(target, var) => None,
        _ => Some(None),
    }
}

fn target_is_ident(expr: &Expr, expected: &str) -> bool {
    matches!(expr, Expr::Ident(name, _) if name == expected)
}

fn block_exits_before_next_turn(block: &Block) -> bool {
    for statement in &block.stmts {
        match statement {
            Stmt::Break { .. } | Stmt::Return { .. } | Stmt::Raise { .. } => return true,
            Stmt::Continue { .. } | Stmt::While { .. } | Stmt::For { .. } | Stmt::Loop { .. } => {
                return false;
            }
            _ => {}
        }
    }
    false
}

fn record_binding_fact(facts: &mut IntFacts, name: &str, value: &Expr) {
    if let Expr::Int(value, _) = value {
        facts.insert(name.to_string(), *value);
    } else {
        facts.remove(name);
    }
}

fn record_assignment_fact(facts: &mut IntFacts, target: &Expr, op: AssignOp, value: &Expr) {
    let Expr::Ident(name, _) = target else {
        return;
    };
    match (op, value) {
        (AssignOp::Eq, Expr::Int(value, _)) => {
            facts.insert(name.clone(), *value);
        }
        (AssignOp::PlusEq, Expr::Int(delta, _)) => {
            if let Some(current) = facts.get_mut(name) {
                if let Some(next) = current.checked_add(*delta) {
                    *current = next;
                    return;
                }
            }
            facts.remove(name);
        }
        (AssignOp::MinusEq, Expr::Int(delta, _)) => {
            if let Some(current) = facts.get_mut(name) {
                if let Some(next) = current.checked_sub(*delta) {
                    *current = next;
                    return;
                }
            }
            facts.remove(name);
        }
        _ => {
            facts.remove(name);
        }
    }
}

fn clear_assigned_facts(facts: &mut IntFacts, block: &Block) {
    for statement in &block.stmts {
        if let Stmt::Assign {
            target: Expr::Ident(name, _),
            ..
        } = statement
        {
            facts.remove(name);
        }
    }
}

fn push_uncheckable(
    report: &mut BoundedLoopReport,
    function_name: &str,
    kind: &'static str,
    span_start: usize,
    span_len: usize,
) {
    report.uncheckable_loops.push(UncheckableLoop {
        function: function_name.to_string(),
        kind,
        span_start,
        span_len,
        message: format!(
            "static bounded-loop verifier: function `{function_name}` contains an uncheckable {kind} loop at {span_start}..{}. No Wasmtime fuel or runtime loop enforcement is claimed; rewrite as a statically bounded `for` over a literal range/array, a literal counter `while`, a loop body that exits before continuing, or keep the loop outside safe/@bounded scope.",
            span_start + span_len
        ),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use garnet_parser::parse_source;

    fn bounds(src: &str) -> Vec<(String, i64)> {
        bounded_functions(&parse_source(src).expect("parses"))
    }

    #[test]
    fn extracts_declared_budgets_sorted_by_name() {
        let b = bounds("@bounded(2000)\ndef zebra() { 1 }\n@bounded(500)\ndef alpha() { 1 }\n");
        assert_eq!(
            b,
            vec![("alpha".to_string(), 500), ("zebra".to_string(), 2000)]
        );
    }

    #[test]
    fn functions_without_bounded_are_absent() {
        let b = bounds("def plain() { 1 }\n@bounded(10)\ndef g() { 1 }\n");
        assert_eq!(b, vec![("g".to_string(), 10)]);
    }

    #[test]
    fn positive_bound_parses_and_checks_clean() {
        let module = parse_source("@bounded(1000)\ndef f() { 1 }\n").expect("parses");
        let report = crate::check_module(&module);
        assert!(
            !report.errors.iter().any(|e| matches!(
                e,
                crate::CheckError::AnnotationError(m) if m.contains("@bounded")
            )),
            "a positive @bounded budget must not raise an annotation error"
        );
    }

    #[test]
    fn zero_bound_is_a_check_error() {
        // Zero parses (a valid Int literal) and is caught by the checker.
        let module = parse_source("@bounded(0)\ndef f() { 1 }\n").expect("parses");
        let report = crate::check_module(&module);
        assert!(
            report.errors.iter().any(|e| matches!(
                e,
                crate::CheckError::AnnotationError(m) if m.contains("@bounded")
            )),
            "@bounded(0) must raise an annotation error"
        );
    }

    #[test]
    fn negative_bound_is_rejected_at_parse() {
        // A negative literal is a leading-minus token plus an Int, not a single
        // Int — so the single-int annotation arg rejects it at parse time
        // (consistent with @mailbox / @max_depth).
        assert!(
            parse_source("@bounded(-5)\ndef f() { 1 }\n").is_err(),
            "@bounded(-5) is not a valid integer literal and must be a parse error"
        );
    }
}
