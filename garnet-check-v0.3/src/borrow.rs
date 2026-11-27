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
//!   direct-returning branch/loop-body liveness slices and scoped pattern
//!   bindings for match-arm merge. Full NLL, precise nested place borrows
//!   beyond simple fields, and lifetime containment remain deferred.

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
    move_id: usize,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum BindingKey {
    Local(usize),
    Unbound(String),
}

#[derive(Clone, Default)]
struct ScopeFrame {
    /// Current visible name -> local binding ID in this lexical scope.
    bindings: HashMap<String, usize>,
    /// Every local binding ID ever allocated while this scope frame was active
    /// (including IDs subsequently shadowed in the same scope) so `pop_scope`
    /// never leaves orphaned IDs behind.
    owned_ids: Vec<usize>,
}

#[derive(Clone)]
struct Env {
    /// Lexical scope stack mapping binding names to unique local binding IDs.
    scopes: Vec<ScopeFrame>,
    /// Moved places `(root_binding_key, subpath)` on the continuing path.
    moved: HashMap<(BindingKey, Vec<String>), MoveRecord>,
    /// Whether any function/exceptional exit (`return`, `raise`, fallible op)
    /// is reachable inside the current region.
    has_exit: bool,
    /// Moved places observed on any function/exceptional exit inside the
    /// current region.
    exit_moved: HashMap<(BindingKey, Vec<String>), MoveRecord>,
    /// Whether any `break` is reachable inside the current region.
    has_break: bool,
    /// Moved places observed on any `break` inside the current region.
    break_moved: HashMap<(BindingKey, Vec<String>), MoveRecord>,
    /// Whether any `continue` is reachable inside the current region.
    has_continue: bool,
    /// Moved places observed on any `continue` inside the current region.
    continue_moved: HashMap<(BindingKey, Vec<String>), MoveRecord>,
    /// Simple declared type names for bindings when the parser gives us one.
    types: HashMap<BindingKey, String>,
}

impl Default for Env {
    fn default() -> Self {
        Self {
            scopes: vec![ScopeFrame::default()],
            moved: HashMap::new(),
            has_exit: false,
            exit_moved: HashMap::new(),
            has_break: false,
            break_moved: HashMap::new(),
            has_continue: false,
            continue_moved: HashMap::new(),
            types: HashMap::new(),
        }
    }
}

struct BranchOutcome {
    env: Env,
    continues: bool,
}

impl Env {
    fn push_scope(&mut self) {
        self.scopes.push(ScopeFrame::default());
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() <= 1 {
            return;
        }
        let popped = self.scopes.pop().unwrap_or_default();
        let popped_ids: HashSet<usize> = popped.owned_ids.into_iter().collect();
        self.moved
            .retain(|(key, _), _| !matches!(key, BindingKey::Local(id) if popped_ids.contains(id)));
        self.exit_moved
            .retain(|(key, _), _| !matches!(key, BindingKey::Local(id) if popped_ids.contains(id)));
        self.break_moved
            .retain(|(key, _), _| !matches!(key, BindingKey::Local(id) if popped_ids.contains(id)));
        self.continue_moved
            .retain(|(key, _), _| !matches!(key, BindingKey::Local(id) if popped_ids.contains(id)));
        self.types
            .retain(|key, _| !matches!(key, BindingKey::Local(id) if popped_ids.contains(id)));
    }

    fn lookup_key(&self, binding: &str) -> BindingKey {
        for frame in self.scopes.iter().rev() {
            if let Some(&id) = frame.bindings.get(binding) {
                return BindingKey::Local(id);
            }
        }
        BindingKey::Unbound(binding.to_string())
    }

    fn declare_binding(&mut self, binding: &str, ty: Option<&TypeExpr>, next_id: &mut usize) {
        let id = *next_id;
        *next_id += 1;
        if let Some(frame) = self.scopes.last_mut() {
            frame.bindings.insert(binding.to_string(), id);
            frame.owned_ids.push(id);
        }
        let key = BindingKey::Local(id);
        match ty.and_then(simple_type_name) {
            Some(name) => {
                self.types.insert(key, name);
            }
            None => {
                self.types.remove(&key);
            }
        }
    }

    fn record_move_place(&mut self, place: &[String], callee: &str, next_id: &mut usize) {
        let Some((root, rest)) = place.split_first() else {
            return;
        };
        let key = self.lookup_key(root);
        let binding = format_place(place);
        let move_id = *next_id;
        *next_id += 1;
        self.moved.insert(
            (key, rest.to_vec()),
            MoveRecord {
                binding,
                callee: callee.to_string(),
                move_id,
            },
        );
    }

    fn rebind_place(&mut self, place: &[String]) {
        let Some((root, rest)) = place.split_first() else {
            return;
        };
        let key = self.lookup_key(root);
        self.moved.retain(|(moved_key, moved_rest), _| {
            moved_key != &key || !moved_rest.starts_with(rest)
        });
    }

    fn clear_exits(&mut self) {
        self.has_exit = false;
        self.exit_moved.clear();
        self.has_break = false;
        self.break_moved.clear();
        self.has_continue = false;
        self.continue_moved.clear();
    }

    fn record_exit_moves(&mut self) {
        self.has_exit = true;
        for (k, v) in &self.moved {
            self.exit_moved.insert(k.clone(), v.clone());
        }
    }

    fn record_break_moves(&mut self) {
        self.has_break = true;
        for (k, v) in &self.moved {
            self.break_moved.insert(k.clone(), v.clone());
        }
    }

    fn record_continue_moves(&mut self) {
        self.has_continue = true;
        for (k, v) in &self.moved {
            self.continue_moved.insert(k.clone(), v.clone());
        }
    }

    fn is_moved(&self, binding: &str) -> Option<&MoveRecord> {
        self.moved_record_for_place(&[binding.to_string()])
    }

    fn moved_record_for_place(&self, place: &[String]) -> Option<&MoveRecord> {
        let (root, rest) = place.split_first()?;
        let key = self.lookup_key(root);
        self.moved
            .iter()
            .find_map(|((moved_key, moved_rest), record)| {
                (moved_key == &key && places_overlap(moved_rest, rest)).then_some(record)
            })
    }

    fn type_of(&self, binding: &str) -> Option<&str> {
        let key = self.lookup_key(binding);
        self.types.get(&key).map(String::as_str)
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
    next_id: &mut usize,
) {
    match expr {
        Expr::Field { receiver, .. } => {
            check_place_operands(receiver, env, sigs, fn_name, diags, next_id);
        }
        Expr::Index {
            receiver, index, ..
        } => {
            check_place_operands(receiver, env, sigs, fn_name, diags, next_id);
            check_expr(index, env, sigs, fn_name, diags, next_id);
        }
        _ => {}
    }
}

fn format_place(place: &[String]) -> String {
    place.join(".")
}

fn places_overlap(left: &[String], right: &[String]) -> bool {
    left == right || left.starts_with(right) || right.starts_with(left)
}

fn check_fn_body(f: &FnDef, sigs: &SignatureTables, diags: &mut Vec<CheckError>) {
    let mut env = Env::default();
    let mut next_id = 0usize;
    // Pre-bind the function's parameters as live (not moved).
    for p in &f.params {
        env.declare_binding(&p.name, p.ty.as_ref(), &mut next_id);
    }
    let _ = check_branch_block(&f.body, &env, sigs, &f.name, diags, &mut next_id);
}

fn extend_prefer_newer(
    dst: &mut HashMap<(BindingKey, Vec<String>), MoveRecord>,
    src: HashMap<(BindingKey, Vec<String>), MoveRecord>,
) {
    for (k, v) in src {
        match dst.get(&k) {
            Some(existing) if existing.move_id >= v.move_id => {}
            _ => {
                dst.insert(k, v);
            }
        }
    }
}

fn check_branch_block(
    block: &Block,
    base: &Env,
    sigs: &SignatureTables,
    fn_name: &str,
    diags: &mut Vec<CheckError>,
    next_id: &mut usize,
) -> BranchOutcome {
    let mut branch_env = base.clone();
    branch_env.push_scope();
    let mut continues = true;
    for stmt in &block.stmts {
        let stmt_continues = check_stmt(stmt, &mut branch_env, sigs, fn_name, diags, next_id);
        match stmt {
            Stmt::Return { .. } | Stmt::Raise { .. } => {
                branch_env.record_exit_moves();
                continues = false;
                break;
            }
            Stmt::Break { .. } => {
                branch_env.record_break_moves();
                continues = false;
                break;
            }
            Stmt::Continue { .. } => {
                branch_env.record_continue_moves();
                continues = false;
                break;
            }
            _ if !stmt_continues => {
                continues = false;
                break;
            }
            _ => {}
        }
    }
    if continues {
        if let Some(tail) = &block.tail_expr {
            continues = check_expr(tail, &mut branch_env, sigs, fn_name, diags, next_id);
        }
    }
    branch_env.pop_scope();
    BranchOutcome {
        env: branch_env,
        continues,
    }
}

fn check_stmt(
    stmt: &Stmt,
    env: &mut Env,
    sigs: &SignatureTables,
    fn_name: &str,
    diags: &mut Vec<CheckError>,
    next_id: &mut usize,
) -> bool {
    match stmt {
        Stmt::Let(decl) => {
            let cont = check_expr(&decl.value, env, sigs, fn_name, diags, next_id);
            env.declare_binding(&decl.name, decl.ty.as_ref(), next_id);
            cont
        }
        Stmt::Var(decl) => {
            let cont = check_expr(&decl.value, env, sigs, fn_name, diags, next_id);
            env.declare_binding(&decl.name, decl.ty.as_ref(), next_id);
            cont
        }
        Stmt::Const(decl) => {
            let cont = check_expr(&decl.value, env, sigs, fn_name, diags, next_id);
            env.declare_binding(&decl.name, decl.ty.as_ref(), next_id);
            cont
        }
        Stmt::Assign {
            target, op, value, ..
        } => {
            let cont = check_expr(value, env, sigs, fn_name, diags, next_id);
            if *op != garnet_parser::ast::AssignOp::Eq {
                env.record_exit_moves();
            }
            if let Some(place) = place_path(target) {
                check_place_operands(target, env, sigs, fn_name, diags, next_id);
                if matches!(target, Expr::Index { .. } | Expr::Field { .. }) {
                    env.record_exit_moves();
                }
                env.rebind_place(&place);
            }
            cont
        }
        Stmt::While {
            condition, body, ..
        } => {
            check_expr(condition, env, sigs, fn_name, diags, next_id);
            let snapshot = env.clone();
            let mut body_base = snapshot.clone();
            body_base.clear_exits();
            let outcome = check_branch_block(body, &body_base, sigs, fn_name, diags, next_id);
            let mut after_moved = snapshot.moved.clone();
            if outcome.continues {
                extend_prefer_newer(&mut after_moved, outcome.env.moved);
            }
            extend_prefer_newer(&mut after_moved, outcome.env.break_moved);
            extend_prefer_newer(&mut after_moved, outcome.env.continue_moved);
            env.moved = after_moved;
            env.has_exit |= outcome.env.has_exit;
            extend_prefer_newer(&mut env.exit_moved, outcome.env.exit_moved);
            true
        }
        Stmt::For {
            iter, body, var, ..
        } => {
            check_expr(iter, env, sigs, fn_name, diags, next_id);
            env.record_exit_moves();
            let snapshot = env.clone();
            let mut body_base = snapshot.clone();
            body_base.clear_exits();
            body_base.push_scope();
            body_base.declare_binding(var, None, next_id);
            let mut outcome = check_branch_block(body, &body_base, sigs, fn_name, diags, next_id);
            outcome.env.pop_scope();
            let mut after_moved = snapshot.moved.clone();
            if outcome.continues {
                extend_prefer_newer(&mut after_moved, outcome.env.moved);
            }
            extend_prefer_newer(&mut after_moved, outcome.env.break_moved);
            extend_prefer_newer(&mut after_moved, outcome.env.continue_moved);
            env.moved = after_moved;
            env.has_exit |= outcome.env.has_exit;
            extend_prefer_newer(&mut env.exit_moved, outcome.env.exit_moved);
            true
        }
        Stmt::Loop { body, .. } => {
            let snapshot = env.clone();
            let mut body_base = snapshot.clone();
            body_base.clear_exits();
            let outcome = check_branch_block(body, &body_base, sigs, fn_name, diags, next_id);
            let mut after_moved = snapshot.moved.clone();
            if outcome.continues {
                extend_prefer_newer(&mut after_moved, outcome.env.moved);
            }
            extend_prefer_newer(&mut after_moved, outcome.env.break_moved);
            extend_prefer_newer(&mut after_moved, outcome.env.continue_moved);
            env.moved = after_moved;
            env.has_exit |= outcome.env.has_exit;
            extend_prefer_newer(&mut env.exit_moved, outcome.env.exit_moved);
            true
        }
        Stmt::Break { value, .. }
        | Stmt::Return { value, .. }
        | Stmt::Yield { value, .. }
        | Stmt::Next { value, .. } => {
            if let Some(e) = value {
                check_expr(e, env, sigs, fn_name, diags, next_id);
            }
            true
        }
        Stmt::Raise { value, .. } => {
            check_expr(value, env, sigs, fn_name, diags, next_id);
            true
        }
        Stmt::Continue { .. } => true,
        Stmt::Expr(e) => check_expr(e, env, sigs, fn_name, diags, next_id),
    }
}

fn check_expr(
    expr: &Expr,
    env: &mut Env,
    sigs: &SignatureTables,
    fn_name: &str,
    diags: &mut Vec<CheckError>,
    next_id: &mut usize,
) -> bool {
    match expr {
        Expr::Ident(name, _) => {
            if let Some(rec) = env.is_moved(name) {
                diags.push(CheckError::SafeModeViolation(format!(
                    "use-after-move: in `{fn_name}`, `{}` was moved into `{}` and cannot be used again",
                    rec.binding, rec.callee
                )));
            }
            true
        }
        Expr::Call { callee, args, .. } => {
            // First evaluate every argument expression for inner moves.
            for a in args {
                check_expr(a, env, sigs, fn_name, diags, next_id);
            }
            // Then resolve the callee and apply ownership to identifier args.
            if let Expr::Ident(callee_name, _) = callee.as_ref() {
                if let Some(kinds) = sigs.functions.get(callee_name) {
                    let pairs: Vec<_> = args.iter().zip(kinds.iter().copied()).collect();
                    apply_ownership(callee_name, &pairs, env, fn_name, diags, next_id);
                }
            } else {
                check_expr(callee, env, sigs, fn_name, diags, next_id);
            }
            env.record_exit_moves();
            true
        }
        Expr::Method {
            receiver,
            method,
            args,
            ..
        } => {
            check_expr(receiver, env, sigs, fn_name, diags, next_id);
            for a in args {
                check_expr(a, env, sigs, fn_name, diags, next_id);
            }
            if let Some(signature) = method_signature_for_receiver(receiver, method, env, sigs) {
                let mut pairs = Vec::with_capacity(args.len() + 1);
                pairs.push((receiver.as_ref(), signature.receiver));
                pairs.extend(args.iter().zip(signature.args.iter().copied()));
                apply_ownership(method, &pairs, env, fn_name, diags, next_id);
            }
            env.record_exit_moves();
            true
        }
        Expr::Field { receiver, .. } => {
            if let Some(place) = place_path(expr) {
                check_place_operands(receiver, env, sigs, fn_name, diags, next_id);
                if let Some(rec) = env.moved_record_for_place(&place) {
                    diags.push(CheckError::SafeModeViolation(format!(
                        "use-after-move: in `{fn_name}`, `{}` was moved into `{}` and cannot be used again",
                        rec.binding, rec.callee
                    )));
                }
            } else {
                check_expr(receiver, env, sigs, fn_name, diags, next_id);
            }
            env.record_exit_moves();
            true
        }
        Expr::Index {
            receiver, index, ..
        } => {
            check_expr(index, env, sigs, fn_name, diags, next_id);
            if let Some(place) = place_path(expr) {
                check_place_operands(receiver, env, sigs, fn_name, diags, next_id);
                if let Some(rec) = env.moved_record_for_place(&place) {
                    diags.push(CheckError::SafeModeViolation(format!(
                        "use-after-move: in `{fn_name}`, `{}` was moved into `{}` and cannot be used again",
                        rec.binding, rec.callee
                    )));
                }
            } else {
                check_expr(receiver, env, sigs, fn_name, diags, next_id);
            }
            env.record_exit_moves();
            true
        }
        Expr::Cast { expr, .. } => {
            check_expr(expr, env, sigs, fn_name, diags, next_id);
            env.record_exit_moves();
            true
        }
        Expr::Binary { op, lhs, rhs, .. } => {
            check_expr(lhs, env, sigs, fn_name, diags, next_id);
            check_expr(rhs, env, sigs, fn_name, diags, next_id);
            if matches!(
                op,
                garnet_parser::ast::BinOp::Add
                    | garnet_parser::ast::BinOp::Sub
                    | garnet_parser::ast::BinOp::Mul
                    | garnet_parser::ast::BinOp::Div
                    | garnet_parser::ast::BinOp::Mod
                    | garnet_parser::ast::BinOp::Pipeline
            ) {
                env.record_exit_moves();
            }
            true
        }
        Expr::Unary { op, expr, .. } => {
            check_expr(expr, env, sigs, fn_name, diags, next_id);
            if matches!(
                op,
                garnet_parser::ast::UnOp::Neg | garnet_parser::ast::UnOp::Question
            ) {
                env.record_exit_moves();
            }
            true
        }
        Expr::If {
            condition,
            then_block,
            elsif_clauses,
            else_block,
            ..
        } => check_if_expr(
            condition,
            then_block,
            elsif_clauses,
            else_block.as_ref(),
            env,
            sigs,
            fn_name,
            diags,
            next_id,
        ),
        Expr::Match { subject, arms, .. } => {
            check_match_expr(subject, arms, env, sigs, fn_name, diags, next_id);
            true
        }
        Expr::Try {
            body,
            rescues,
            ensure,
            ..
        } => check_try_expr(
            body,
            rescues,
            ensure.as_ref(),
            env,
            sigs,
            fn_name,
            diags,
            next_id,
        ),
        Expr::Closure { .. } | Expr::Spawn { .. } => true,
        Expr::Array { elements, .. } => {
            for e in elements {
                check_expr(e, env, sigs, fn_name, diags, next_id);
            }
            true
        }
        Expr::Map { entries, .. } => {
            for (k, v) in entries {
                check_expr(k, env, sigs, fn_name, diags, next_id);
                check_expr(v, env, sigs, fn_name, diags, next_id);
            }
            true
        }
        Expr::Str(lit, _) => {
            check_interp_expr(lit, env, sigs, fn_name, diags, next_id);
            true
        }
        Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::Bool(_, _)
        | Expr::Nil(_)
        | Expr::Symbol(_, _)
        | Expr::Path(_, _) => true,
    }
}

fn check_interp_expr(
    lit: &garnet_parser::ast::StringLit,
    env: &mut Env,
    sigs: &SignatureTables,
    fn_name: &str,
    diags: &mut Vec<CheckError>,
    next_id: &mut usize,
) {
    let parts: Vec<String> = lit
        .parts
        .iter()
        .filter_map(|p| match p {
            garnet_parser::token::StrPart::Interp(src) => Some(src.clone()),
            garnet_parser::token::StrPart::Lit(_) => None,
        })
        .collect();
    if parts.is_empty() {
        return;
    }
    env.record_exit_moves();
    for src in parts {
        let wrapped = format!("def __interp__() {{ {src} }}");
        if let Ok(module) = garnet_parser::parse_source(&wrapped) {
            for item in &module.items {
                if let Item::Fn(f) = item {
                    let outcome = check_branch_block(&f.body, env, sigs, fn_name, diags, next_id);
                    if outcome.continues {
                        *env = outcome.env;
                    } else {
                        env.has_exit |= outcome.env.has_exit;
                        env.exit_moved.extend(outcome.env.exit_moved);
                        env.has_break |= outcome.env.has_break;
                        env.break_moved.extend(outcome.env.break_moved);
                        env.has_continue |= outcome.env.has_continue;
                        env.continue_moved.extend(outcome.env.continue_moved);
                    }
                    env.record_exit_moves();
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(never)]
fn check_if_expr(
    condition: &Expr,
    then_block: &Block,
    elsif_clauses: &[(Expr, Block)],
    else_block: Option<&Block>,
    env: &mut Env,
    sigs: &SignatureTables,
    fn_name: &str,
    diags: &mut Vec<CheckError>,
    next_id: &mut usize,
) -> bool {
    check_expr(condition, env, sigs, fn_name, diags, next_id);
    let snapshot = Box::new(env.clone());
    // When `else` is present, execution must take one of the explicit branches,
    // so only continuing branches contribute to the merged moved map (allowing
    // unanimous re-initialization across all branches to clear an incoming move).
    // Without `else`, the implicit fallthrough preserves `snapshot.moved`.
    let has_else = else_block.is_some();
    let mut merged_moved = if has_else {
        HashMap::new()
    } else {
        snapshot.moved.clone()
    };
    let mut any_continues = !has_else;
    let mut merged_has_exit = snapshot.has_exit;
    let mut merged_exit_moved = snapshot.exit_moved.clone();
    let mut merged_has_break = snapshot.has_break;
    let mut merged_break_moved = snapshot.break_moved.clone();
    let mut merged_has_continue = snapshot.has_continue;
    let mut merged_continue_moved = snapshot.continue_moved.clone();

    let then_outcome = check_branch_block(then_block, &snapshot, sigs, fn_name, diags, next_id);
    if then_outcome.continues {
        any_continues = true;
        extend_prefer_newer(&mut merged_moved, then_outcome.env.moved);
    }
    merged_has_exit |= then_outcome.env.has_exit;
    extend_prefer_newer(&mut merged_exit_moved, then_outcome.env.exit_moved);
    merged_has_break |= then_outcome.env.has_break;
    extend_prefer_newer(&mut merged_break_moved, then_outcome.env.break_moved);
    merged_has_continue |= then_outcome.env.has_continue;
    extend_prefer_newer(&mut merged_continue_moved, then_outcome.env.continue_moved);

    for (cond, block) in elsif_clauses {
        let mut alt_env = (*snapshot).clone();
        check_expr(cond, &mut alt_env, sigs, fn_name, diags, next_id);
        let after_cond = alt_env.clone();
        let branch_outcome = check_branch_block(block, &after_cond, sigs, fn_name, diags, next_id);
        if branch_outcome.continues {
            any_continues = true;
            extend_prefer_newer(&mut merged_moved, branch_outcome.env.moved);
        } else if !has_else {
            extend_prefer_newer(&mut merged_moved, after_cond.moved);
        }
        merged_has_exit |= after_cond.has_exit | branch_outcome.env.has_exit;
        extend_prefer_newer(&mut merged_exit_moved, after_cond.exit_moved);
        extend_prefer_newer(&mut merged_exit_moved, branch_outcome.env.exit_moved);
        merged_has_break |= after_cond.has_break | branch_outcome.env.has_break;
        extend_prefer_newer(&mut merged_break_moved, after_cond.break_moved);
        extend_prefer_newer(&mut merged_break_moved, branch_outcome.env.break_moved);
        merged_has_continue |= after_cond.has_continue | branch_outcome.env.has_continue;
        extend_prefer_newer(&mut merged_continue_moved, after_cond.continue_moved);
        extend_prefer_newer(
            &mut merged_continue_moved,
            branch_outcome.env.continue_moved,
        );
    }

    if let Some(b) = else_block {
        let branch_outcome = check_branch_block(b, &snapshot, sigs, fn_name, diags, next_id);
        if branch_outcome.continues {
            any_continues = true;
            extend_prefer_newer(&mut merged_moved, branch_outcome.env.moved);
        }
        merged_has_exit |= branch_outcome.env.has_exit;
        extend_prefer_newer(&mut merged_exit_moved, branch_outcome.env.exit_moved);
        merged_has_break |= branch_outcome.env.has_break;
        extend_prefer_newer(&mut merged_break_moved, branch_outcome.env.break_moved);
        merged_has_continue |= branch_outcome.env.has_continue;
        extend_prefer_newer(
            &mut merged_continue_moved,
            branch_outcome.env.continue_moved,
        );
    }

    env.moved = if any_continues {
        merged_moved
    } else {
        snapshot.moved
    };
    env.has_exit = merged_has_exit;
    env.exit_moved = merged_exit_moved;
    env.has_break = merged_has_break;
    env.break_moved = merged_break_moved;
    env.has_continue = merged_has_continue;
    env.continue_moved = merged_continue_moved;
    any_continues
}

#[inline(never)]
fn check_match_expr(
    subject: &Expr,
    arms: &[garnet_parser::ast::MatchArm],
    env: &mut Env,
    sigs: &SignatureTables,
    fn_name: &str,
    diags: &mut Vec<CheckError>,
    next_id: &mut usize,
) {
    check_expr(subject, env, sigs, fn_name, diags, next_id);
    let snapshot = Box::new(env.clone());
    let mut merged_moved = snapshot.moved.clone();
    let mut merged_has_exit = snapshot.has_exit;
    let mut merged_exit_moved = snapshot.exit_moved.clone();
    let mut merged_has_break = snapshot.has_break;
    let mut merged_break_moved = snapshot.break_moved.clone();
    let mut merged_has_continue = snapshot.has_continue;
    let mut merged_continue_moved = snapshot.continue_moved.clone();
    for arm in arms {
        let mut arm_base = (*snapshot).clone();
        arm_base.push_scope();
        bind_pattern(&arm.pattern, &mut arm_base, next_id);
        if let Some(g) = &arm.guard {
            check_expr(g, &mut arm_base, sigs, fn_name, diags, next_id);
        }
        let mut guard_env = arm_base.clone();
        let mut arm_outcome =
            check_branch_block(&arm.body, &arm_base, sigs, fn_name, diags, next_id);
        guard_env.pop_scope();
        arm_outcome.env.pop_scope();
        if arm_outcome.continues {
            extend_prefer_newer(&mut merged_moved, arm_outcome.env.moved);
        } else if arm.guard.is_some() {
            extend_prefer_newer(&mut merged_moved, guard_env.moved);
        }
        merged_has_exit |= guard_env.has_exit | arm_outcome.env.has_exit;
        extend_prefer_newer(&mut merged_exit_moved, guard_env.exit_moved);
        extend_prefer_newer(&mut merged_exit_moved, arm_outcome.env.exit_moved);
        merged_has_break |= guard_env.has_break | arm_outcome.env.has_break;
        extend_prefer_newer(&mut merged_break_moved, guard_env.break_moved);
        extend_prefer_newer(&mut merged_break_moved, arm_outcome.env.break_moved);
        merged_has_continue |= guard_env.has_continue | arm_outcome.env.has_continue;
        extend_prefer_newer(&mut merged_continue_moved, guard_env.continue_moved);
        extend_prefer_newer(&mut merged_continue_moved, arm_outcome.env.continue_moved);
    }
    env.moved = merged_moved;
    env.has_exit = merged_has_exit;
    env.exit_moved = merged_exit_moved;
    env.has_break = merged_has_break;
    env.break_moved = merged_break_moved;
    env.has_continue = merged_has_continue;
    env.continue_moved = merged_continue_moved;
}

/// Heap-allocated working state for `check_try_expr` so deep `defer` chains
/// (up to `ParseBudget::max_depth`) keep stack frames minimal even in debug
/// builds.
struct TryFrameState {
    outer_snapshot: Env,
    continuing_moved: HashMap<(BindingKey, Vec<String>), MoveRecord>,
    local_has_exit: bool,
    local_exit: HashMap<(BindingKey, Vec<String>), MoveRecord>,
    local_has_break: bool,
    local_break: HashMap<(BindingKey, Vec<String>), MoveRecord>,
    local_has_continue: bool,
    local_continue: HashMap<(BindingKey, Vec<String>), MoveRecord>,
    ensure_visible_moved: HashMap<(BindingKey, Vec<String>), MoveRecord>,
}

#[allow(clippy::too_many_arguments)]
#[inline(never)]
fn check_try_expr(
    body: &Block,
    rescues: &[garnet_parser::ast::RescueClause],
    ensure: Option<&Block>,
    env: &mut Env,
    sigs: &SignatureTables,
    fn_name: &str,
    diags: &mut Vec<CheckError>,
    next_id: &mut usize,
) -> bool {
    let mut state = Box::new(TryFrameState {
        outer_snapshot: env.clone(),
        continuing_moved: HashMap::new(),
        local_has_exit: false,
        local_exit: HashMap::new(),
        local_has_break: false,
        local_break: HashMap::new(),
        local_has_continue: false,
        local_continue: HashMap::new(),
        ensure_visible_moved: HashMap::new(),
    });

    let mut body_base = state.outer_snapshot.clone();
    body_base.clear_exits();

    let body_outcome = check_branch_block(body, &body_base, sigs, fn_name, diags, next_id);
    drop(body_base);

    let mut any_continues = body_outcome.continues;
    if body_outcome.continues {
        state.continuing_moved = body_outcome.env.moved;
    }
    state.local_has_exit = body_outcome.env.has_exit;
    state.local_exit = body_outcome.env.exit_moved;
    state.local_has_break = body_outcome.env.has_break;
    state.local_break = body_outcome.env.break_moved;
    state.local_has_continue = body_outcome.env.has_continue;
    state.local_continue = body_outcome.env.continue_moved;

    extend_prefer_newer(
        &mut state.ensure_visible_moved,
        state.continuing_moved.clone(),
    );
    extend_prefer_newer(&mut state.ensure_visible_moved, state.local_exit.clone());
    extend_prefer_newer(&mut state.ensure_visible_moved, state.local_break.clone());
    extend_prefer_newer(
        &mut state.ensure_visible_moved,
        state.local_continue.clone(),
    );

    for r in rescues {
        let mut rescue_base = state.outer_snapshot.clone();
        extend_prefer_newer(&mut rescue_base.moved, state.local_exit.clone());
        rescue_base.clear_exits();
        rescue_base.push_scope();
        if let Some(name) = &r.name {
            rescue_base.declare_binding(name, None, next_id);
        }
        let mut rescue_outcome =
            check_branch_block(&r.body, &rescue_base, sigs, fn_name, diags, next_id);
        rescue_outcome.env.pop_scope();
        if rescue_outcome.continues {
            any_continues = true;
            extend_prefer_newer(
                &mut state.continuing_moved,
                rescue_outcome.env.moved.clone(),
            );
            extend_prefer_newer(&mut state.ensure_visible_moved, rescue_outcome.env.moved);
        }
        state.local_has_exit |= rescue_outcome.env.has_exit;
        extend_prefer_newer(&mut state.local_exit, rescue_outcome.env.exit_moved.clone());
        state.local_has_break |= rescue_outcome.env.has_break;
        extend_prefer_newer(
            &mut state.local_break,
            rescue_outcome.env.break_moved.clone(),
        );
        state.local_has_continue |= rescue_outcome.env.has_continue;
        extend_prefer_newer(
            &mut state.local_continue,
            rescue_outcome.env.continue_moved.clone(),
        );
        extend_prefer_newer(
            &mut state.ensure_visible_moved,
            rescue_outcome.env.exit_moved,
        );
        extend_prefer_newer(
            &mut state.ensure_visible_moved,
            rescue_outcome.env.break_moved,
        );
        extend_prefer_newer(
            &mut state.ensure_visible_moved,
            rescue_outcome.env.continue_moved,
        );
    }

    let mut ensure_continues = true;
    if let Some(ensure_block) = ensure {
        let mut ensure_base = state.outer_snapshot.clone();
        ensure_base.moved = std::mem::take(&mut state.ensure_visible_moved);
        ensure_base.clear_exits();
        let ensure_outcome =
            check_branch_block(ensure_block, &ensure_base, sigs, fn_name, diags, next_id);
        ensure_continues = ensure_outcome.continues;
        if ensure_outcome.continues {
            // A key is cleared if it was in `ensure_base.moved` and absent after
            // `ensure`; a key is newly moved if it was not in `ensure_base.moved`
            // OR received a fresh `move_id` (e.g. `x = [2]; take(x)`).
            let cleared: HashSet<_> = ensure_base
                .moved
                .keys()
                .filter(|k| !ensure_outcome.env.moved.contains_key(*k))
                .cloned()
                .collect();
            let newly_moved: Vec<_> = ensure_outcome
                .env
                .moved
                .iter()
                .filter(|(k, v)| {
                    ensure_base
                        .moved
                        .get(*k)
                        .is_none_or(|old| old.move_id != v.move_id)
                })
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            let apply_ensure_delta = |map: &mut HashMap<(BindingKey, Vec<String>), MoveRecord>| {
                map.retain(|k, _| !cleared.contains(k));
                for (k, v) in &newly_moved {
                    map.insert(k.clone(), v.clone());
                }
            };
            if any_continues {
                apply_ensure_delta(&mut state.continuing_moved);
            }
            if state.local_has_exit {
                apply_ensure_delta(&mut state.local_exit);
            }
            if state.local_has_break {
                apply_ensure_delta(&mut state.local_break);
            }
            if state.local_has_continue {
                apply_ensure_delta(&mut state.local_continue);
            }
        }
        state.local_has_exit |= ensure_outcome.env.has_exit;
        extend_prefer_newer(&mut state.local_exit, ensure_outcome.env.exit_moved);
        state.local_has_break |= ensure_outcome.env.has_break;
        extend_prefer_newer(&mut state.local_break, ensure_outcome.env.break_moved);
        state.local_has_continue |= ensure_outcome.env.has_continue;
        extend_prefer_newer(&mut state.local_continue, ensure_outcome.env.continue_moved);
    }

    let try_continues = any_continues && ensure_continues;
    env.moved = if try_continues {
        state.continuing_moved
    } else {
        state.outer_snapshot.moved
    };
    env.has_exit = state.outer_snapshot.has_exit | state.local_has_exit;
    env.exit_moved = state.outer_snapshot.exit_moved;
    extend_prefer_newer(&mut env.exit_moved, state.local_exit);
    env.has_break = state.outer_snapshot.has_break | state.local_has_break;
    env.break_moved = state.outer_snapshot.break_moved;
    extend_prefer_newer(&mut env.break_moved, state.local_break);
    env.has_continue = state.outer_snapshot.has_continue | state.local_has_continue;
    env.continue_moved = state.outer_snapshot.continue_moved;
    extend_prefer_newer(&mut env.continue_moved, state.local_continue);
    try_continues
}

fn bind_pattern(pattern: &Pattern, env: &mut Env, next_id: &mut usize) {
    match pattern {
        Pattern::Ident(name, _) => {
            env.declare_binding(name, None, next_id);
        }
        Pattern::Tuple(items, _) => {
            for p in items {
                bind_pattern(p, env, next_id);
            }
        }
        Pattern::Enum(_, items, _) => {
            for p in items {
                bind_pattern(p, env, next_id);
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
    next_id: &mut usize,
) {
    detect_drop_discipline_violations(callee, pairs, fn_name, diags);
    detect_aliasing_violations(callee, pairs, fn_name, diags);
    for (arg, kind) in pairs {
        if matches!(kind, Some(Ownership::Own)) {
            if let Some(place) = place_path(arg) {
                env.record_move_place(&place, callee, next_id);
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
