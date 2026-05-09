//! Borrow-checker semantic pass (Rung 4 completion, simple form).
//!
//! Tracks linear-resource discipline for safe-mode functions:
//!
//! 1. **Move tracking.** When a binding is passed to a callee parameter
//!    annotated `own`, or a receiver is passed to an unambiguous method whose
//!    `self` parameter is `own`, the binding is recorded as moved. Any
//!    subsequent use of that binding within the same scope produces a
//!    `use-after-move` diagnostic.
//! 2. **Aliasing-XOR-mutation.** Within a single expression, the same
//!    binding cannot appear as both a `mut` (exclusive) argument and any
//!    other argument.
//! 3. **Re-assign rebinds.** A `let mut name = expr` re-introduces `name`
//!    as a fresh, owned binding (overwriting any prior moved state).
//!
//! Limitations of this first cut (a later production checker will lift
//! them):
//! - Direct calls to top-level `fn` items are tracked. Method calls are tracked
//!   when the receiver has a simple declared type, with an unambiguous
//!   same-module method-name fallback for still-untyped receivers. Full
//!   impl-block dispatch remains deferred.
//! - Simple field projections are tracked as places for move and alias checks.
//!   Index expressions are tracked conservatively as wildcard sub-places.
//! - Branch joins are conservative and coarse-grained, with first
//!   direct-returning branch and loop-body liveness slices. Full NLL, precise
//!   nested place borrows beyond simple fields, and lifetime containment remain
//!   deferred.

use garnet_parser::ast::{
    Block, Expr, FnDef, FnMode, Item, Module, Ownership, Pattern, Stmt, TypeExpr,
};
use std::collections::{HashMap, HashSet};

use crate::CheckError;

const INDEX_PLACE_SEGMENT: &str = "[*]";

/// One recorded move site, used by the diagnostic that names both halves.
#[derive(Debug, Clone)]
pub struct MoveRecord {
    pub binding: String,
    pub callee: String,
}

/// Map from function name -> ordered ownership kinds for its parameters.
type FunctionSignatureTable = HashMap<String, Vec<Option<Ownership>>>;

/// Receiver + argument ownership contract for a method call.
#[derive(Debug, Clone, PartialEq, Eq)]
struct MethodSignature {
    receiver: Option<Ownership>,
    args: Vec<Option<Ownership>>,
}

#[derive(Default)]
struct SignatureTables {
    functions: FunctionSignatureTable,
    typed_methods: HashMap<(String, String), MethodSignature>,
    ambiguous_typed_methods: HashSet<(String, String)>,
    methods: HashMap<String, MethodSignature>,
    ambiguous_methods: HashSet<String>,
}

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

fn collect_signatures(module: &Module) -> SignatureTables {
    let mut tables = SignatureTables::default();
    for item in &module.items {
        match item {
            Item::Fn(f) => {
                let kinds: Vec<Option<Ownership>> = f.params.iter().map(|p| p.ownership).collect();
                tables.functions.insert(f.name.clone(), kinds);
            }
            Item::Impl(impl_block) => {
                for method in &impl_block.methods {
                    let target_type = simple_type_name(&impl_block.target);
                    register_method_signature(&mut tables, target_type.as_deref(), method);
                }
            }
            _ => {}
        }
    }
    tables
}

fn register_method_signature(
    tables: &mut SignatureTables,
    target_type: Option<&str>,
    method: &FnDef,
) {
    let signature = method_signature(method);

    if let Some(target_type) = target_type {
        let key = (target_type.to_string(), method.name.clone());
        if !tables.ambiguous_typed_methods.contains(&key) {
            match tables.typed_methods.get(&key) {
                Some(existing) if existing == &signature => {}
                Some(_) => {
                    tables.typed_methods.remove(&key);
                    tables.ambiguous_typed_methods.insert(key);
                }
                None => {
                    tables.typed_methods.insert(key, signature.clone());
                }
            }
        }
    }

    if tables.ambiguous_methods.contains(&method.name) {
        return;
    }

    match tables.methods.get(&method.name) {
        Some(existing) if existing == &signature => {}
        Some(_) => {
            tables.methods.remove(&method.name);
            tables.ambiguous_methods.insert(method.name.clone());
        }
        None => {
            tables.methods.insert(method.name.clone(), signature);
        }
    }
}

fn simple_type_name(ty: &TypeExpr) -> Option<String> {
    match ty {
        TypeExpr::Named { path, args, .. } if args.is_empty() => Some(path.join("::")),
        _ => None,
    }
}

fn method_signature(method: &FnDef) -> MethodSignature {
    match method.params.first() {
        Some(first) if first.name == "self" => MethodSignature {
            receiver: first.ownership,
            args: method.params.iter().skip(1).map(|p| p.ownership).collect(),
        },
        _ => MethodSignature {
            receiver: None,
            args: method.params.iter().map(|p| p.ownership).collect(),
        },
    }
}

#[derive(Default, Clone)]
struct Env {
    /// Set of bindings that have been moved out of and may not be used.
    moved: HashMap<String, MoveRecord>,
    /// Simple declared type names for bindings when the parser gives us one.
    types: HashMap<String, String>,
}

struct BranchOutcome {
    env: Env,
    continues: bool,
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

    fn record_move_place(&mut self, place: &[String], callee: &str) {
        let binding = format_place(place);
        self.record_move(&binding, callee);
    }

    fn rebind(&mut self, binding: &str) {
        self.rebind_place(&[binding.to_string()]);
    }

    fn rebind_place(&mut self, place: &[String]) {
        let binding = format_place(place);
        let prefix = format!("{binding}.");
        self.moved
            .retain(|moved, _| moved != &binding && !moved.starts_with(&prefix));
    }

    fn rebind_with_type(&mut self, binding: &str, ty: Option<&TypeExpr>) {
        self.rebind(binding);
        match ty.and_then(simple_type_name) {
            Some(name) => {
                self.types.insert(binding.to_string(), name);
            }
            None => {
                self.types.remove(binding);
            }
        }
    }

    fn forget_type(&mut self, binding: &str) {
        self.types.remove(binding);
    }

    fn restore_binding_from(&mut self, binding: &str, snapshot: &Env) {
        let prefix = format!("{binding}.");
        self.moved
            .retain(|moved, _| moved != binding && !moved.starts_with(&prefix));
        for (moved, record) in &snapshot.moved {
            if moved == binding || moved.starts_with(&prefix) {
                self.moved.insert(moved.clone(), record.clone());
            }
        }
        match snapshot.types.get(binding) {
            Some(ty) => {
                self.types.insert(binding.to_string(), ty.clone());
            }
            None => {
                self.types.remove(binding);
            }
        }
    }

    fn is_moved(&self, binding: &str) -> Option<&MoveRecord> {
        self.moved_record_for_place(&[binding.to_string()])
    }

    fn moved_record_for_place(&self, place: &[String]) -> Option<&MoveRecord> {
        self.moved.iter().find_map(|(moved, record)| {
            let moved_place = split_place(moved);
            places_overlap(&moved_place, place).then_some(record)
        })
    }

    fn type_of(&self, binding: &str) -> Option<&str> {
        self.types.get(binding).map(String::as_str)
    }
}

fn place_path(expr: &Expr) -> Option<Vec<String>> {
    match expr {
        Expr::Ident(name, _) => Some(vec![name.clone()]),
        Expr::Field {
            receiver, field, ..
        } => {
            let mut path = place_path(receiver)?;
            path.push(field.clone());
            Some(path)
        }
        Expr::Index { receiver, .. } => {
            let mut path = place_path(receiver)?;
            path.push(INDEX_PLACE_SEGMENT.to_string());
            Some(path)
        }
        _ => None,
    }
}

fn check_place_operands(
    expr: &Expr,
    env: &mut Env,
    sigs: &SignatureTables,
    fn_name: &str,
    diags: &mut Vec<CheckError>,
) {
    match expr {
        Expr::Field { receiver, .. } => check_place_operands(receiver, env, sigs, fn_name, diags),
        Expr::Index {
            receiver, index, ..
        } => {
            check_place_operands(receiver, env, sigs, fn_name, diags);
            check_expr(index, env, sigs, fn_name, diags);
        }
        _ => {}
    }
}

fn format_place(place: &[String]) -> String {
    place.join(".")
}

fn split_place(place: &str) -> Vec<String> {
    place.split('.').map(ToOwned::to_owned).collect()
}

fn places_overlap(left: &[String], right: &[String]) -> bool {
    left == right || left.starts_with(right) || right.starts_with(left)
}

fn check_fn_body(f: &FnDef, sigs: &SignatureTables, diags: &mut Vec<CheckError>) {
    let mut env = Env::default();
    // Pre-bind the function's parameters as live (not moved).
    for p in &f.params {
        env.rebind_with_type(&p.name, p.ty.as_ref());
    }
    let _ = check_branch_block(&f.body, &env, sigs, &f.name, diags);
}

fn check_branch_block(
    block: &Block,
    base: &Env,
    sigs: &SignatureTables,
    fn_name: &str,
    diags: &mut Vec<CheckError>,
) -> BranchOutcome {
    let mut branch_env = base.clone();
    for stmt in &block.stmts {
        check_stmt(stmt, &mut branch_env, sigs, fn_name, diags);
        if matches!(stmt, Stmt::Return { .. }) {
            return BranchOutcome {
                env: branch_env,
                continues: false,
            };
        }
    }
    if let Some(tail) = &block.tail_expr {
        check_expr(tail, &mut branch_env, sigs, fn_name, diags);
    }
    BranchOutcome {
        env: branch_env,
        continues: true,
    }
}

fn check_stmt(
    stmt: &Stmt,
    env: &mut Env,
    sigs: &SignatureTables,
    fn_name: &str,
    diags: &mut Vec<CheckError>,
) {
    match stmt {
        Stmt::Let(decl) => {
            check_expr(&decl.value, env, sigs, fn_name, diags);
            env.rebind_with_type(&decl.name, decl.ty.as_ref());
        }
        Stmt::Var(decl) => {
            check_expr(&decl.value, env, sigs, fn_name, diags);
            env.rebind_with_type(&decl.name, decl.ty.as_ref());
        }
        Stmt::Const(decl) => {
            check_expr(&decl.value, env, sigs, fn_name, diags);
            env.rebind_with_type(&decl.name, decl.ty.as_ref());
        }
        Stmt::Assign { target, value, .. } => {
            check_expr(value, env, sigs, fn_name, diags);
            if let Some(place) = place_path(target) {
                check_place_operands(target, env, sigs, fn_name, diags);
                env.rebind_place(&place);
            }
        }
        Stmt::While {
            condition, body, ..
        } => {
            check_expr(condition, env, sigs, fn_name, diags);
            let snapshot = env.clone();
            let outcome = check_branch_block(body, &snapshot, sigs, fn_name, diags);
            if outcome.continues {
                *env = outcome.env;
            } else {
                *env = snapshot;
            }
        }
        Stmt::For {
            iter, body, var, ..
        } => {
            check_expr(iter, env, sigs, fn_name, diags);
            let snapshot = env.clone();
            let mut body_base = snapshot.clone();
            body_base.rebind(var);
            body_base.forget_type(var);
            let outcome = check_branch_block(body, &body_base, sigs, fn_name, diags);
            if outcome.continues {
                *env = outcome.env;
                env.restore_binding_from(var, &snapshot);
            } else {
                *env = snapshot;
            }
        }
        Stmt::Loop { body, .. } => {
            let snapshot = env.clone();
            let outcome = check_branch_block(body, &snapshot, sigs, fn_name, diags);
            if outcome.continues {
                *env = outcome.env;
            } else {
                *env = snapshot;
            }
        }
        Stmt::Break { value, .. }
        | Stmt::Return { value, .. }
        | Stmt::Yield { value, .. }
        | Stmt::Next { value, .. } => {
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
    sigs: &SignatureTables,
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
                if let Some(kinds) = sigs.functions.get(callee_name) {
                    let pairs: Vec<_> = args.iter().zip(kinds.iter().copied()).collect();
                    apply_ownership(callee_name, &pairs, env, fn_name, diags);
                }
            } else {
                check_expr(callee, env, sigs, fn_name, diags);
            }
        }
        Expr::Method {
            receiver,
            method,
            args,
            ..
        } => {
            check_expr(receiver, env, sigs, fn_name, diags);
            for a in args {
                check_expr(a, env, sigs, fn_name, diags);
            }
            if let Some(signature) = method_signature_for_receiver(receiver, method, env, sigs) {
                let mut pairs = Vec::with_capacity(args.len() + 1);
                pairs.push((receiver.as_ref(), signature.receiver));
                pairs.extend(args.iter().zip(signature.args.iter().copied()));
                apply_ownership(method, &pairs, env, fn_name, diags);
            }
        }
        Expr::Field { receiver, .. } => {
            if let Some(place) = place_path(expr) {
                check_place_operands(receiver, env, sigs, fn_name, diags);
                if let Some(rec) = env.moved_record_for_place(&place) {
                    diags.push(CheckError::SafeModeViolation(format!(
                        "use-after-move: in `{fn_name}`, `{}` was moved into `{}` and cannot be used again",
                        rec.binding, rec.callee
                    )));
                }
            } else {
                check_expr(receiver, env, sigs, fn_name, diags);
            }
        }
        Expr::Index {
            receiver, index, ..
        } => {
            check_expr(index, env, sigs, fn_name, diags);
            if let Some(place) = place_path(expr) {
                check_place_operands(receiver, env, sigs, fn_name, diags);
                if let Some(rec) = env.moved_record_for_place(&place) {
                    diags.push(CheckError::SafeModeViolation(format!(
                        "use-after-move: in `{fn_name}`, `{}` was moved into `{}` and cannot be used again",
                        rec.binding, rec.callee
                    )));
                }
            } else {
                check_expr(receiver, env, sigs, fn_name, diags);
            }
        }
        Expr::Cast { expr, .. } => check_expr(expr, env, sigs, fn_name, diags),
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
            let snapshot = env.clone();
            let mut merged_moved = snapshot.moved.clone();

            // Each branch is checked independently against a snapshot of
            // env. Conservative: moves from branches that may continue are
            // merged after the if. A branch that returns from the function
            // cannot poison later code on paths that still continue.
            let then_outcome = check_branch_block(then_block, &snapshot, sigs, fn_name, diags);
            if then_outcome.continues {
                merged_moved.extend(then_outcome.env.moved);
            }

            for (cond, block) in elsif_clauses {
                let mut alt_env = snapshot.clone();
                check_expr(cond, &mut alt_env, sigs, fn_name, diags);
                let after_cond = alt_env.clone();
                let branch_outcome = check_branch_block(block, &after_cond, sigs, fn_name, diags);
                if branch_outcome.continues {
                    merged_moved.extend(branch_outcome.env.moved);
                } else {
                    merged_moved.extend(after_cond.moved);
                }
            }

            if let Some(b) = else_block {
                let branch_outcome = check_branch_block(b, &snapshot, sigs, fn_name, diags);
                if branch_outcome.continues {
                    merged_moved.extend(branch_outcome.env.moved);
                }
            }

            env.moved = merged_moved;
        }
        Expr::Match { subject, arms, .. } => {
            check_expr(subject, env, sigs, fn_name, diags);
            let snapshot = env.clone();
            for arm in arms {
                let mut arm_env = snapshot.clone();
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
        Pattern::Ident(name, _) => {
            env.rebind(name);
            env.forget_type(name);
        }
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

fn method_signature_for_receiver<'a>(
    receiver: &Expr,
    method: &str,
    env: &Env,
    sigs: &'a SignatureTables,
) -> Option<&'a MethodSignature> {
    if let Expr::Ident(name, _) = receiver {
        if let Some(receiver_type) = env.type_of(name) {
            let key = (receiver_type.to_string(), method.to_string());
            return sigs.typed_methods.get(&key);
        }
    }
    sigs.methods.get(method)
}

fn apply_ownership(
    callee: &str,
    pairs: &[(&Expr, Option<Ownership>)],
    env: &mut Env,
    fn_name: &str,
    diags: &mut Vec<CheckError>,
) {
    detect_drop_discipline_violations(callee, pairs, fn_name, diags);
    detect_aliasing_violations(callee, pairs, fn_name, diags);
    for (arg, kind) in pairs {
        if matches!(kind, Some(Ownership::Own)) {
            if let Some(place) = place_path(arg) {
                env.record_move_place(&place, callee);
            }
        }
    }
}

fn detect_drop_discipline_violations(
    callee: &str,
    pairs: &[(&Expr, Option<Ownership>)],
    fn_name: &str,
    diags: &mut Vec<CheckError>,
) {
    let mut owned_places: Vec<Vec<String>> = Vec::new();
    for (arg, kind) in pairs {
        if !matches!(kind, Some(Ownership::Own)) {
            continue;
        }
        let Some(place) = place_path(arg) else {
            continue;
        };
        if let Some(existing) = owned_places
            .iter()
            .find(|existing| places_overlap(existing, &place))
        {
            diags.push(CheckError::SafeModeViolation(format!(
                "drop discipline violation: in `{fn_name}`, `{}` and `{}` are both passed as `own` to `{callee}`, which would drop the same place twice",
                format_place(existing),
                format_place(&place)
            )));
            return;
        }
        owned_places.push(place);
    }
}

fn detect_aliasing_violations(
    callee: &str,
    pairs: &[(&Expr, Option<Ownership>)],
    fn_name: &str,
    diags: &mut Vec<CheckError>,
) {
    // Find any binding that appears as a `mut` argument and at least once
    // somewhere else in the same call. That's the basic
    // aliasing-XOR-mutation rule: an exclusive borrow may not coexist with
    // any other reference to the same binding.
    let mut mut_places: Vec<Vec<String>> = Vec::new();
    let mut other_places: Vec<Vec<String>> = Vec::new();
    for (arg, kind) in pairs {
        if let Some(place) = place_path(arg) {
            if matches!(kind, Some(Ownership::Mut)) {
                mut_places.push(place);
            } else {
                other_places.push(place);
            }
        }
    }
    for (idx, mut_place) in mut_places.iter().enumerate() {
        let overlaps_other_mut = mut_places
            .iter()
            .enumerate()
            .any(|(other_idx, other)| other_idx != idx && places_overlap(mut_place, other));
        let overlaps_other = other_places
            .iter()
            .any(|other| places_overlap(mut_place, other));
        if overlaps_other_mut || overlaps_other {
            diags.push(CheckError::SafeModeViolation(format!(
                "aliasing violation: in `{fn_name}`, `{}` is passed as `mut` to `{callee}` while another reference to the same binding is in flight",
                format_place(mut_place)
            )));
        }
    }
}
