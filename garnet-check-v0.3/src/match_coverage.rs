//! Conservative safe-mode match coverage for finite domains.
//!
//! This is intentionally a narrow semantic slice: it proves exhaustiveness and
//! simple reachability for `Bool`, same-module enum subjects, imported enum
//! aliases, finite nested constructor payloads, and literal `true`/`false`
//! guards whose type is visible from parameter metadata, local annotations,
//! local boolean/enum variant initializers, or direct mutable-local
//! assignments, including conservative `if`/`elsif`/`else` branch joins,
//! nested all-path `if` assignment joins inside branch bodies, and narrowly
//! proven direct closure call-effect invalidation for local branch-selected
//! closures. It also recognizes immutable local, same-module top-level, and
//! named/glob imported top-level boolean constants, same-module and
//! named/glob imported top-level integer constants, static finite-float,
//! nil/symbol/string equality and inequality facts, including static
//! interpolated strings whose interpolation bodies resolve through this same
//! fact domain, mixed known-literal equality/inequality facts, narrow boolean
//! const aliases, and basic boolean const expressions in match guards, and
//! rejects duplicate literal arms and arms after catch-all arms in otherwise
//! open-domain matches. Boolean const
//! `and`/`or` folding honors decisive left operands without requiring the right
//! operand to resolve, and boolean const equality / inequality comparisons fold
//! over already-resolved boolean facts. Direct
//! boolean match guards use the same conservative folding. Narrow integer
//! arithmetic plus equality/inequality and relational guard comparisons, along
//! with finite-float equality/inequality, relational, and arithmetic guard
//! comparisons, static boolean/string relational guard comparisons, and
//! immutable local boolean/integer expression aliases, including path-qualified
//! constant references, fold over the same fact domain using checked or
//! runtime-aligned numeric/boolean/string rules. It does not attempt
//! full type inference, loop fixed-point inference, broader
//! mutable/escaped/general higher-order closure call-effect analysis,
//! recursive/open payload coverage, or broader non-literal guard reasoning.

use crate::CheckError;
use garnet_parser::ast::{
    AssignOp, BinOp, Block, ClosureBody, Expr, FnDef, FnMode, Item, Module, Param, Pattern, Stmt,
    StringLit, TypeExpr, UnOp, UseImports,
};
use garnet_parser::token::StrPart;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
struct EnumInfo {
    path: Vec<String>,
    variants: BTreeMap<String, VariantInfo>,
}

#[derive(Debug, Clone)]
struct VariantInfo {
    fields: Vec<TypeExpr>,
}

#[derive(Debug, Clone)]
enum FiniteDomain {
    Bool,
    Enum(EnumDomain),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum GuardCoverage {
    Unguarded,
    AlwaysTrue,
    AlwaysFalse,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
enum ConstFact {
    Bool(bool),
    Int(i64),
    Float(f64),
    Nil,
    Symbol(String),
    Str(String),
}

impl ConstFact {
    fn into_bool(self) -> Option<bool> {
        match self {
            ConstFact::Bool(value) => Some(value),
            ConstFact::Int(_)
            | ConstFact::Float(_)
            | ConstFact::Nil
            | ConstFact::Symbol(_)
            | ConstFact::Str(_) => None,
        }
    }

    fn literal_eq(self, other: Self) -> bool {
        match (self, other) {
            (ConstFact::Bool(lhs), ConstFact::Bool(rhs)) => lhs == rhs,
            (ConstFact::Int(lhs), ConstFact::Int(rhs)) => lhs == rhs,
            (ConstFact::Float(lhs), ConstFact::Float(rhs)) => lhs == rhs,
            (ConstFact::Int(lhs), ConstFact::Float(rhs))
            | (ConstFact::Float(rhs), ConstFact::Int(lhs)) => (lhs as f64) == rhs,
            (ConstFact::Nil, ConstFact::Nil) => true,
            (ConstFact::Symbol(lhs), ConstFact::Symbol(rhs)) => lhs == rhs,
            (ConstFact::Str(lhs), ConstFact::Str(rhs)) => lhs == rhs,
            _ => false,
        }
    }

    fn ordered_cmp(self, other: Self, op: BinOp) -> Option<bool> {
        let ordering = match (self, other) {
            (ConstFact::Bool(lhs), ConstFact::Bool(rhs)) => Some(lhs.cmp(&rhs)),
            (ConstFact::Int(lhs), ConstFact::Int(rhs)) => Some(lhs.cmp(&rhs)),
            (ConstFact::Float(lhs), ConstFact::Float(rhs)) => lhs.partial_cmp(&rhs),
            (ConstFact::Int(lhs), ConstFact::Float(rhs)) => (lhs as f64).partial_cmp(&rhs),
            (ConstFact::Float(lhs), ConstFact::Int(rhs)) => lhs.partial_cmp(&(rhs as f64)),
            (ConstFact::Str(lhs), ConstFact::Str(rhs)) => Some(lhs.cmp(&rhs)),
            _ => None,
        }?;

        match op {
            BinOp::Lt => Some(ordering.is_lt()),
            BinOp::Gt => Some(ordering.is_gt()),
            BinOp::LtEq => Some(ordering.is_le()),
            BinOp::GtEq => Some(ordering.is_ge()),
            _ => None,
        }
    }

    fn numeric_arith(self, other: Self, op: BinOp) -> Option<Self> {
        match (self, other) {
            (ConstFact::Int(lhs), ConstFact::Int(rhs)) => {
                let value = match op {
                    BinOp::Add => lhs.checked_add(rhs)?,
                    BinOp::Sub => lhs.checked_sub(rhs)?,
                    BinOp::Mul => lhs.checked_mul(rhs)?,
                    BinOp::Div => lhs.checked_div(rhs)?,
                    BinOp::Mod => lhs.checked_rem(rhs)?,
                    _ => return None,
                };
                Some(ConstFact::Int(value))
            }
            (ConstFact::Float(lhs), ConstFact::Float(rhs)) => float_arith(lhs, rhs, op, true),
            (ConstFact::Int(lhs), ConstFact::Float(rhs)) => float_arith(lhs as f64, rhs, op, false),
            (ConstFact::Float(lhs), ConstFact::Int(rhs)) => float_arith(lhs, rhs as f64, op, false),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct EnumDomain {
    info: EnumInfo,
    pattern_prefixes: BTreeSet<Vec<String>>,
}

#[derive(Debug, Clone, Default)]
struct Scope {
    module_path: Vec<String>,
    enum_imports: BTreeMap<String, Vec<String>>,
    glob_imports: Vec<Vec<String>>,
    module_imports: BTreeMap<String, Vec<String>>,
    guard_facts: GuardFacts,
}

#[derive(Debug, Default)]
struct Checker {
    enums: BTreeMap<Vec<String>, EnumInfo>,
    const_guard_facts: BTreeMap<Vec<String>, ConstFact>,
    scopes: BTreeMap<Vec<String>, Scope>,
    errors: Vec<CheckError>,
}

type Env = BTreeMap<String, FiniteDomain>;
type ClosureEffects = BTreeMap<String, BTreeSet<String>>;
type GuardFacts = BTreeMap<String, ConstFact>;

pub fn check_match_coverage(module: &Module) -> Vec<CheckError> {
    let mut checker = Checker::default();
    checker.collect_enums(&module.items, &[]);
    checker.collect_const_guard_facts(&module.items, &[]);
    checker.collect_scopes(&module.items, &[]);
    checker.check_items(&module.items, module.safe, &[]);
    checker.errors
}

impl Checker {
    fn collect_enums(&mut self, items: &[Item], prefix: &[String]) {
        for item in items {
            match item {
                Item::Enum(enum_def) => {
                    let mut path = prefix.to_vec();
                    path.push(enum_def.name.clone());
                    let variants = enum_def
                        .variants
                        .iter()
                        .map(|variant| {
                            (
                                variant.name.clone(),
                                VariantInfo {
                                    fields: variant.fields.clone(),
                                },
                            )
                        })
                        .collect();
                    self.enums.insert(path.clone(), EnumInfo { path, variants });
                }
                Item::Module(module) => {
                    let mut nested = prefix.to_vec();
                    nested.push(module.name.clone());
                    self.collect_enums(&module.items, &nested);
                }
                _ => {}
            }
        }
    }

    fn collect_const_guard_facts(&mut self, items: &[Item], prefix: &[String]) {
        let mut consts = Vec::new();
        collect_const_guard_decls(items, prefix, &mut consts);

        for _ in 0..consts.len() {
            let mut changed = false;
            for (path, value) in &consts {
                if self.const_guard_facts.contains_key(path) {
                    continue;
                }

                let module_path = &path[..path.len().saturating_sub(1)];
                if let Some(value) = self.const_fact_from_expr(value, module_path) {
                    self.const_guard_facts.insert(path.clone(), value);
                    changed = true;
                }
            }

            if !changed {
                break;
            }
        }
    }

    fn const_fact_from_expr(&self, expr: &Expr, module_path: &[String]) -> Option<ConstFact> {
        match expr {
            Expr::Bool(value, _) => Some(ConstFact::Bool(*value)),
            Expr::Int(value, _) => Some(ConstFact::Int(*value)),
            Expr::Float(value, _) => finite_float_fact(*value),
            Expr::Nil(_) => Some(ConstFact::Nil),
            Expr::Symbol(value, _) => Some(ConstFact::Symbol(value.clone())),
            Expr::Str(value, _) => const_string_literal(value, |src| {
                interpolation_const_fact(src, |expr| self.const_fact_from_expr(expr, module_path))
            })
            .map(ConstFact::Str),
            Expr::Ident(name, _) => {
                let mut path = module_path.to_vec();
                path.push(name.clone());
                self.const_guard_facts.get(&path).cloned()
            }
            Expr::Path(path, _) => self.resolve_const_fact_path_from_module(path, module_path),
            Expr::Unary {
                op: UnOp::Not,
                expr,
                ..
            } => self
                .const_fact_from_expr(expr, module_path)
                .and_then(ConstFact::into_bool)
                .map(|value| ConstFact::Bool(!value)),
            Expr::Unary {
                op: UnOp::Neg,
                expr,
                ..
            } => match self.const_fact_from_expr(expr, module_path)? {
                ConstFact::Int(value) => value.checked_neg().map(ConstFact::Int),
                ConstFact::Float(value) => finite_float_fact(-value),
                _ => None,
            },
            Expr::Unary { .. } => None,
            Expr::Binary {
                op: BinOp::And,
                lhs,
                rhs,
                ..
            } => {
                let lhs = self.const_fact_from_expr(lhs, module_path)?.into_bool()?;
                if !lhs {
                    Some(ConstFact::Bool(false))
                } else {
                    let rhs = self.const_fact_from_expr(rhs, module_path)?.into_bool()?;
                    Some(ConstFact::Bool(rhs))
                }
            }
            Expr::Binary {
                op: BinOp::Or,
                lhs,
                rhs,
                ..
            } => {
                let lhs = self.const_fact_from_expr(lhs, module_path)?.into_bool()?;
                if lhs {
                    Some(ConstFact::Bool(true))
                } else {
                    let rhs = self.const_fact_from_expr(rhs, module_path)?.into_bool()?;
                    Some(ConstFact::Bool(rhs))
                }
            }
            Expr::Binary {
                op: BinOp::Eq,
                lhs,
                rhs,
                ..
            } => {
                let lhs = self.const_fact_from_expr(lhs, module_path)?;
                let rhs = self.const_fact_from_expr(rhs, module_path)?;
                Some(ConstFact::Bool(lhs.literal_eq(rhs)))
            }
            Expr::Binary {
                op: BinOp::NotEq,
                lhs,
                rhs,
                ..
            } => {
                let lhs = self.const_fact_from_expr(lhs, module_path)?;
                let rhs = self.const_fact_from_expr(rhs, module_path)?;
                Some(ConstFact::Bool(!lhs.literal_eq(rhs)))
            }
            Expr::Binary {
                op: op @ (BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq),
                lhs,
                rhs,
                ..
            } => {
                let lhs = self.const_fact_from_expr(lhs, module_path)?;
                let rhs = self.const_fact_from_expr(rhs, module_path)?;
                lhs.ordered_cmp(rhs, *op).map(ConstFact::Bool)
            }
            Expr::Binary {
                op: op @ (BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod),
                lhs,
                rhs,
                ..
            } => {
                let lhs = self.const_fact_from_expr(lhs, module_path)?;
                let rhs = self.const_fact_from_expr(rhs, module_path)?;
                lhs.numeric_arith(rhs, *op)
            }
            Expr::Binary { .. } => None,
            _ => None,
        }
    }

    fn resolve_const_fact_path_from_module(
        &self,
        path: &[String],
        module_path: &[String],
    ) -> Option<ConstFact> {
        let mut candidates = BTreeMap::new();
        self.add_const_fact_candidate(&mut candidates, path);

        if !module_path.is_empty() {
            let mut relative = module_path.to_vec();
            relative.extend_from_slice(path);
            self.add_const_fact_candidate(&mut candidates, &relative);
        }

        if candidates.len() != 1 {
            return None;
        }
        candidates.into_values().next()
    }

    fn collect_scopes(&mut self, items: &[Item], module_path: &[String]) {
        let scope = self.build_scope(items, module_path);
        self.scopes.insert(module_path.to_vec(), scope);

        for item in items {
            if let Item::Module(module) = item {
                let mut nested = module_path.to_vec();
                nested.push(module.name.clone());
                self.collect_scopes(&module.items, &nested);
            }
        }
    }

    fn build_scope(&self, items: &[Item], module_path: &[String]) -> Scope {
        let mut scope = Scope {
            module_path: module_path.to_vec(),
            ..Scope::default()
        };
        scope.guard_facts = self.same_module_const_guard_facts(module_path);

        for item in items {
            let Item::Use(use_decl) = item else {
                continue;
            };

            let Some(base_path) = self.canonical_use_path(&use_decl.path, module_path) else {
                continue;
            };

            match &use_decl.imports {
                UseImports::Module => {
                    if let Some(name) = base_path.last() {
                        scope.module_imports.insert(name.clone(), base_path.clone());
                        if self.enums.contains_key(&base_path) {
                            scope.enum_imports.insert(name.clone(), base_path.clone());
                        }
                        if let Some(value) = self.const_guard_facts.get(&base_path) {
                            scope.guard_facts.insert(name.clone(), value.clone());
                        }
                    }
                }
                UseImports::Named(names) => {
                    for name in names {
                        let mut path = base_path.clone();
                        path.push(name.clone());
                        if self.enums.contains_key(&path) {
                            scope.enum_imports.insert(name.clone(), path.clone());
                        }
                        if let Some(value) = self.const_guard_facts.get(&path) {
                            scope.guard_facts.insert(name.clone(), value.clone());
                        }
                    }
                }
                UseImports::Glob => {
                    for (path, value) in self.const_guard_facts_in_module(&base_path) {
                        if let Some(name) = path.last() {
                            scope.guard_facts.insert(name.clone(), value);
                        }
                    }
                    scope.glob_imports.push(base_path);
                }
            }
        }

        scope
    }

    fn check_items(&mut self, items: &[Item], module_safe: bool, module_path: &[String]) {
        let scope = self.scope_for(module_path);
        for item in items {
            match item {
                Item::Fn(f) => self.check_fn(f, module_safe || f.mode == FnMode::Safe, &scope),
                Item::Module(module) => {
                    let mut nested = module_path.to_vec();
                    nested.push(module.name.clone());
                    self.check_items(&module.items, module_safe || module.safe, &nested);
                }
                Item::Impl(impl_block) => {
                    for method in &impl_block.methods {
                        self.check_fn(method, module_safe || method.mode == FnMode::Safe, &scope);
                    }
                }
                _ => {}
            }
        }
    }

    fn check_fn(&mut self, f: &FnDef, effective_safe: bool, scope: &Scope) {
        if !effective_safe {
            return;
        }

        let mut env = Env::new();
        for param in &f.params {
            if let Some(ty) = &param.ty {
                if let Some(domain) = self.domain_from_type(ty, scope) {
                    env.insert(param.name.clone(), domain);
                }
            }
        }

        let mut closure_effects = ClosureEffects::new();
        let mut guard_facts = scope.guard_facts.clone();
        for param in &f.params {
            guard_facts.remove(&param.name);
        }
        self.walk_block(
            &f.body,
            &f.name,
            env,
            &mut closure_effects,
            &mut guard_facts,
            scope,
        );
    }

    fn walk_block(
        &mut self,
        block: &Block,
        fn_name: &str,
        mut env: Env,
        closure_effects: &mut ClosureEffects,
        guard_facts: &mut GuardFacts,
        scope: &Scope,
    ) -> Env {
        for stmt in &block.stmts {
            self.walk_stmt(stmt, fn_name, &mut env, closure_effects, guard_facts, scope);
        }
        if let Some(tail) = &block.tail_expr {
            self.walk_expr(tail, fn_name, &mut env, closure_effects, guard_facts, scope);
        }
        env
    }

    fn walk_stmt(
        &mut self,
        stmt: &Stmt,
        fn_name: &str,
        env: &mut Env,
        closure_effects: &mut ClosureEffects,
        guard_facts: &mut GuardFacts,
        scope: &Scope,
    ) {
        match stmt {
            Stmt::Let(decl) => {
                self.walk_expr(
                    &decl.value,
                    fn_name,
                    env,
                    closure_effects,
                    guard_facts,
                    scope,
                );
                let domain = decl
                    .ty
                    .as_ref()
                    .and_then(|ty| self.domain_from_type(ty, scope))
                    .or_else(|| self.domain_from_expr(&decl.value, env, scope));
                if let Some(domain) = domain {
                    env.insert(decl.name.clone(), domain);
                }
                update_closure_effect(decl.name.clone(), &decl.value, closure_effects);
                update_guard_fact_for_let(decl, guard_facts, &|path| {
                    self.resolve_const_fact_path(path, scope)
                });
            }
            Stmt::Var(decl) => {
                self.walk_expr(
                    &decl.value,
                    fn_name,
                    env,
                    closure_effects,
                    guard_facts,
                    scope,
                );
                if let Some(domain) = decl
                    .ty
                    .as_ref()
                    .and_then(|ty| self.domain_from_type(ty, scope))
                {
                    env.insert(decl.name.clone(), domain);
                }
                update_closure_effect(decl.name.clone(), &decl.value, closure_effects);
                guard_facts.remove(&decl.name);
            }
            Stmt::Const(decl) => {
                self.walk_expr(
                    &decl.value,
                    fn_name,
                    env,
                    closure_effects,
                    guard_facts,
                    scope,
                );
                update_closure_effect(decl.name.clone(), &decl.value, closure_effects);
                update_guard_fact(decl.name.clone(), &decl.value, guard_facts, &|path| {
                    self.resolve_const_fact_path(path, scope)
                });
            }
            Stmt::Assign {
                target, op, value, ..
            } => {
                // Compound assignments are operator/type dependent, so they
                // clear finite-domain evidence instead of preserving it.
                let assigned_domain = matches!(op, AssignOp::Eq)
                    .then(|| self.domain_from_expr(value, env, scope))
                    .flatten();
                self.walk_expr(target, fn_name, env, closure_effects, guard_facts, scope);
                self.walk_expr(value, fn_name, env, closure_effects, guard_facts, scope);
                if let Expr::Ident(name, _) = target {
                    if let Some(domain) = assigned_domain {
                        env.insert(name.clone(), domain);
                    } else {
                        env.remove(name);
                    }
                    if matches!(op, AssignOp::Eq) {
                        update_closure_effect(name.clone(), value, closure_effects);
                    } else {
                        closure_effects.remove(name);
                    }
                    guard_facts.remove(name);
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.walk_expr(condition, fn_name, env, closure_effects, guard_facts, scope);
                let assigned = maybe_assigned_outer_targets_in_block(body);
                let mut body_effects = closure_effects.clone();
                let mut body_facts = guard_facts.clone();
                self.walk_block(
                    body,
                    fn_name,
                    env.clone(),
                    &mut body_effects,
                    &mut body_facts,
                    scope,
                );
                for target in assigned {
                    env.remove(&target);
                    guard_facts.remove(&target);
                }
            }
            Stmt::For {
                var, iter, body, ..
            } => {
                self.walk_expr(iter, fn_name, env, closure_effects, guard_facts, scope);
                let loop_locals = BTreeSet::from([var.clone()]);
                let assigned = maybe_assigned_outer_targets_in_block_excluding(body, &loop_locals);
                let mut body_effects = closure_effects.clone();
                let mut body_facts = guard_facts.clone();
                self.walk_block(
                    body,
                    fn_name,
                    env.clone(),
                    &mut body_effects,
                    &mut body_facts,
                    scope,
                );
                for target in assigned {
                    env.remove(&target);
                    guard_facts.remove(&target);
                }
            }
            Stmt::Loop { body, .. } => {
                let assigned = maybe_assigned_outer_targets_in_block(body);
                let mut body_effects = closure_effects.clone();
                let mut body_facts = guard_facts.clone();
                self.walk_block(
                    body,
                    fn_name,
                    env.clone(),
                    &mut body_effects,
                    &mut body_facts,
                    scope,
                );
                for target in assigned {
                    env.remove(&target);
                    guard_facts.remove(&target);
                }
            }
            Stmt::Return { value, .. }
            | Stmt::Yield { value, .. }
            | Stmt::Next { value, .. }
            | Stmt::Break { value, .. } => {
                if let Some(value) = value {
                    self.walk_expr(value, fn_name, env, closure_effects, guard_facts, scope);
                }
            }
            Stmt::Raise { value, .. } | Stmt::Expr(value) => {
                self.walk_expr(value, fn_name, env, closure_effects, guard_facts, scope);
            }
            Stmt::Continue { .. } => {}
        }
    }

    fn walk_expr(
        &mut self,
        expr: &Expr,
        fn_name: &str,
        env: &mut Env,
        closure_effects: &mut ClosureEffects,
        guard_facts: &mut GuardFacts,
        scope: &Scope,
    ) {
        match expr {
            Expr::Binary { lhs, rhs, .. } => {
                self.walk_expr(lhs, fn_name, env, closure_effects, guard_facts, scope);
                self.walk_expr(rhs, fn_name, env, closure_effects, guard_facts, scope);
            }
            Expr::Unary { expr, .. } | Expr::Cast { expr, .. } | Expr::Spawn { expr, .. } => {
                self.walk_expr(expr, fn_name, env, closure_effects, guard_facts, scope);
            }
            Expr::Call { callee, args, .. } => {
                self.walk_expr(callee, fn_name, env, closure_effects, guard_facts, scope);
                for arg in args {
                    self.walk_expr(arg, fn_name, env, closure_effects, guard_facts, scope);
                }
                let assigned =
                    closure_effect_from_value(callee, closure_effects).unwrap_or_default();
                for target in assigned {
                    env.remove(&target);
                    guard_facts.remove(&target);
                }
            }
            Expr::Method {
                receiver: callee,
                args,
                ..
            } => {
                self.walk_expr(callee, fn_name, env, closure_effects, guard_facts, scope);
                for arg in args {
                    self.walk_expr(arg, fn_name, env, closure_effects, guard_facts, scope);
                }
            }
            Expr::Field { receiver, .. } => {
                self.walk_expr(receiver, fn_name, env, closure_effects, guard_facts, scope);
            }
            Expr::Index {
                receiver, index, ..
            } => {
                self.walk_expr(receiver, fn_name, env, closure_effects, guard_facts, scope);
                self.walk_expr(index, fn_name, env, closure_effects, guard_facts, scope);
            }
            Expr::If {
                condition,
                then_block,
                elsif_clauses,
                else_block,
                ..
            } => {
                self.walk_expr(condition, fn_name, env, closure_effects, guard_facts, scope);
                let branch_base = env.clone();
                let branch_effects = closure_effects.clone();
                let branch_facts = guard_facts.clone();
                let mut then_effects = branch_effects.clone();
                let mut then_facts = branch_facts.clone();
                let then_env = self.walk_block(
                    then_block,
                    fn_name,
                    branch_base.clone(),
                    &mut then_effects,
                    &mut then_facts,
                    scope,
                );
                let mut branch_envs = vec![then_env];
                let mut branch_closure_effects = vec![then_effects];
                let mut branch_guard_facts = vec![then_facts];
                let mut branch_targets = vec![assigned_outer_targets_in_block(then_block)];
                for (condition, block) in elsif_clauses {
                    self.walk_expr(condition, fn_name, env, closure_effects, guard_facts, scope);
                    let mut elsif_effects = branch_effects.clone();
                    let mut elsif_facts = branch_facts.clone();
                    let elsif_env = self.walk_block(
                        block,
                        fn_name,
                        branch_base.clone(),
                        &mut elsif_effects,
                        &mut elsif_facts,
                        scope,
                    );
                    branch_envs.push(elsif_env);
                    branch_closure_effects.push(elsif_effects);
                    branch_guard_facts.push(elsif_facts);
                    branch_targets.push(assigned_outer_targets_in_block(block));
                }
                if let Some(block) = else_block {
                    let mut else_effects = branch_effects.clone();
                    let mut else_facts = branch_facts.clone();
                    let else_env = self.walk_block(
                        block,
                        fn_name,
                        branch_base.clone(),
                        &mut else_effects,
                        &mut else_facts,
                        scope,
                    );
                    branch_envs.push(else_env);
                    branch_closure_effects.push(else_effects);
                    branch_guard_facts.push(else_facts);
                    branch_targets.push(assigned_outer_targets_in_block(block));
                } else {
                    branch_envs.push(branch_base);
                    branch_closure_effects.push(branch_effects);
                    branch_guard_facts.push(branch_facts.clone());
                    branch_targets.push(BTreeSet::new());
                }
                *env = join_branch_envs(env, &branch_envs, &branch_targets);
                *closure_effects = join_branch_closure_effects(&branch_closure_effects);
                *guard_facts = join_branch_guard_facts(&branch_facts, &branch_guard_facts);
            }
            Expr::Match { subject, arms, .. } => {
                self.walk_expr(subject, fn_name, env, closure_effects, guard_facts, scope);
                if let Some(domain) = self.domain_from_expr(subject, env, scope) {
                    self.check_match_arms(fn_name, &domain, arms, guard_facts, scope);
                } else {
                    self.check_open_match_reachability(fn_name, arms, guard_facts, scope);
                }
                for arm in arms {
                    let mut arm_guard_facts = guard_facts.clone();
                    remove_pattern_guard_facts(&arm.pattern, &mut arm_guard_facts);
                    if let Some(guard) = &arm.guard {
                        self.walk_expr(
                            guard,
                            fn_name,
                            env,
                            closure_effects,
                            &mut arm_guard_facts,
                            scope,
                        );
                    }
                    let mut arm_env = env.clone();
                    let mut arm_effects = closure_effects.clone();
                    remove_pattern_bindings(&arm.pattern, &mut arm_env);
                    self.walk_block(
                        &arm.body,
                        fn_name,
                        arm_env,
                        &mut arm_effects,
                        &mut arm_guard_facts,
                        scope,
                    );
                }
            }
            Expr::Try {
                body,
                rescues,
                ensure,
                ..
            } => {
                let mut assigned = maybe_assigned_outer_targets_in_block(body);
                let mut body_effects = closure_effects.clone();
                let mut body_facts = guard_facts.clone();
                self.walk_block(
                    body,
                    fn_name,
                    env.clone(),
                    &mut body_effects,
                    &mut body_facts,
                    scope,
                );
                for rescue in rescues {
                    assigned.extend(maybe_assigned_outer_targets_in_block(&rescue.body));
                    let mut rescue_effects = closure_effects.clone();
                    let mut rescue_facts = guard_facts.clone();
                    self.walk_block(
                        &rescue.body,
                        fn_name,
                        env.clone(),
                        &mut rescue_effects,
                        &mut rescue_facts,
                        scope,
                    );
                }
                if let Some(block) = ensure {
                    assigned.extend(maybe_assigned_outer_targets_in_block(block));
                    let mut ensure_effects = closure_effects.clone();
                    let mut ensure_facts = guard_facts.clone();
                    self.walk_block(
                        block,
                        fn_name,
                        env.clone(),
                        &mut ensure_effects,
                        &mut ensure_facts,
                        scope,
                    );
                }
                for target in assigned {
                    env.remove(&target);
                    guard_facts.remove(&target);
                }
            }
            Expr::Closure { body, .. } => match body.as_ref() {
                ClosureBody::Block(block) => {
                    let mut closure_effects = closure_effects.clone();
                    let mut closure_facts = guard_facts.clone();
                    self.walk_block(
                        block,
                        fn_name,
                        env.clone(),
                        &mut closure_effects,
                        &mut closure_facts,
                        scope,
                    );
                }
                ClosureBody::Expr(expr) => {
                    let mut closure_env = env.clone();
                    let mut closure_effects = closure_effects.clone();
                    let mut closure_facts = guard_facts.clone();
                    self.walk_expr(
                        expr,
                        fn_name,
                        &mut closure_env,
                        &mut closure_effects,
                        &mut closure_facts,
                        scope,
                    );
                }
            },
            Expr::Array { elements, .. } => {
                for element in elements {
                    self.walk_expr(element, fn_name, env, closure_effects, guard_facts, scope);
                }
            }
            Expr::Map { entries, .. } => {
                for (key, value) in entries {
                    self.walk_expr(key, fn_name, env, closure_effects, guard_facts, scope);
                    self.walk_expr(value, fn_name, env, closure_effects, guard_facts, scope);
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

    fn check_match_arms(
        &mut self,
        fn_name: &str,
        domain: &FiniteDomain,
        arms: &[garnet_parser::ast::MatchArm],
        guard_facts: &GuardFacts,
        scope: &Scope,
    ) {
        let mut covered = BTreeSet::new();
        let mut catch_all_seen = false;

        for arm in arms {
            let pattern = describe_pattern(&arm.pattern);
            if catch_all_seen {
                self.errors.push(CheckError::SafeModeViolation(format!(
                    "unreachable match arm in safe function '{fn_name}': pattern {pattern} is covered by prior catch-all arm"
                )));
                continue;
            }

            if self.domain_is_fully_covered(domain, &covered) {
                self.errors.push(CheckError::SafeModeViolation(format!(
                    "unreachable match arm in safe function '{fn_name}': finite domain is already fully covered before pattern {pattern}"
                )));
                continue;
            }

            let keys = self.coverage_keys(domain, &arm.pattern);
            if !keys.is_empty() && keys.iter().all(|key| covered.contains(key)) {
                self.errors.push(CheckError::SafeModeViolation(format!(
                    "unreachable match arm in safe function '{fn_name}': pattern {pattern} is already covered by prior arm"
                )));
            }

            let mut arm_guard_facts = guard_facts.clone();
            remove_pattern_guard_facts(&arm.pattern, &mut arm_guard_facts);
            match self.guard_coverage(&arm.guard, &arm_guard_facts, scope) {
                GuardCoverage::AlwaysFalse => {
                    self.errors.push(CheckError::SafeModeViolation(format!(
                        "unreachable match arm in safe function '{fn_name}': pattern {pattern} has a statically false guard"
                    )));
                    continue;
                }
                GuardCoverage::Unknown => continue,
                GuardCoverage::Unguarded | GuardCoverage::AlwaysTrue => {}
            }

            if is_catch_all(&arm.pattern) {
                catch_all_seen = true;
            } else {
                covered.extend(keys);
            }
        }

        if !catch_all_seen && !self.domain_is_fully_covered(domain, &covered) {
            let missing = self
                .missing_patterns(domain, &covered)
                .into_iter()
                .collect::<Vec<_>>()
                .join(", ");
            self.errors.push(CheckError::SafeModeViolation(format!(
                "non-exhaustive match in safe function '{fn_name}': missing {missing}"
            )));
        }
    }

    fn check_open_match_reachability(
        &mut self,
        fn_name: &str,
        arms: &[garnet_parser::ast::MatchArm],
        guard_facts: &GuardFacts,
        scope: &Scope,
    ) {
        let mut covered_literals = BTreeSet::new();
        let mut catch_all_seen = false;

        for arm in arms {
            let pattern = describe_pattern(&arm.pattern);
            if catch_all_seen {
                self.errors.push(CheckError::SafeModeViolation(format!(
                    "unreachable match arm in safe function '{fn_name}': pattern {pattern} is covered by prior catch-all arm"
                )));
                continue;
            }

            let literal_key = literal_pattern_key(&arm.pattern);
            if literal_key
                .as_ref()
                .is_some_and(|key| covered_literals.contains(key))
            {
                self.errors.push(CheckError::SafeModeViolation(format!(
                    "unreachable match arm in safe function '{fn_name}': pattern {pattern} is already covered by prior literal arm"
                )));
            }

            let mut arm_guard_facts = guard_facts.clone();
            remove_pattern_guard_facts(&arm.pattern, &mut arm_guard_facts);
            match self.guard_coverage(&arm.guard, &arm_guard_facts, scope) {
                GuardCoverage::AlwaysFalse => {
                    self.errors.push(CheckError::SafeModeViolation(format!(
                        "unreachable match arm in safe function '{fn_name}': pattern {pattern} has a statically false guard"
                    )));
                    continue;
                }
                GuardCoverage::Unknown => continue,
                GuardCoverage::Unguarded | GuardCoverage::AlwaysTrue => {}
            }

            if is_catch_all(&arm.pattern) {
                catch_all_seen = true;
            } else if let Some(key) = literal_key {
                covered_literals.insert(key);
            }
        }
    }

    fn domain_is_fully_covered(&self, domain: &FiniteDomain, covered: &BTreeSet<String>) -> bool {
        self.expected_keys(domain)
            .iter()
            .all(|key| covered.contains(key))
    }

    fn missing_patterns(&self, domain: &FiniteDomain, covered: &BTreeSet<String>) -> Vec<String> {
        self.expected_keys(domain)
            .into_iter()
            .filter(|key| !covered.contains(key))
            .map(|key| format!("`{key}`"))
            .collect()
    }

    fn expected_keys(&self, domain: &FiniteDomain) -> BTreeSet<String> {
        self.expected_keys_with_seen(domain, &mut BTreeSet::new())
    }

    fn expected_keys_with_seen(
        &self,
        domain: &FiniteDomain,
        seen: &mut BTreeSet<Vec<String>>,
    ) -> BTreeSet<String> {
        match domain {
            FiniteDomain::Bool => ["true".to_string(), "false".to_string()]
                .into_iter()
                .collect(),
            FiniteDomain::Enum(domain) => {
                let info = &domain.info;
                if !seen.insert(info.path.clone()) {
                    return BTreeSet::new();
                }
                let mut keys = BTreeSet::new();
                for (variant, details) in &info.variants {
                    keys.extend(self.expected_variant_keys(domain, variant, details, seen));
                }
                seen.remove(&info.path);
                keys
            }
        }
    }

    fn expected_variant_keys(
        &self,
        domain: &EnumDomain,
        variant: &str,
        details: &VariantInfo,
        seen: &mut BTreeSet<Vec<String>>,
    ) -> BTreeSet<String> {
        if details.fields.is_empty() {
            return [variant_label(domain, variant)].into_iter().collect();
        }

        let field_scope = self.scope_for_enum(&domain.info);
        let Some(field_domains) =
            self.enumerable_field_domains(&details.fields, seen, &field_scope)
        else {
            return [variant_label(domain, variant)].into_iter().collect();
        };

        let mut field_keys = Vec::new();
        for field_domain in field_domains {
            let keys = self.expected_keys_with_seen(&field_domain, seen);
            if keys.is_empty() {
                return [variant_label(domain, variant)].into_iter().collect();
            }
            field_keys.push(keys.into_iter().collect::<Vec<_>>());
        }

        nested_variant_keys(domain, variant, &field_keys)
            .into_iter()
            .collect()
    }

    fn coverage_keys(&self, domain: &FiniteDomain, pattern: &Pattern) -> BTreeSet<String> {
        match (domain, pattern) {
            (FiniteDomain::Bool, Pattern::Literal(Expr::Bool(value, _), _)) => {
                [value.to_string()].into_iter().collect()
            }
            (FiniteDomain::Enum(domain), Pattern::Enum(path, sub_patterns, _)) => self
                .enum_pattern_coverage_keys(domain, path, sub_patterns)
                .into_iter()
                .collect(),
            _ if is_catch_all(pattern) => self.expected_keys(domain),
            _ => BTreeSet::new(),
        }
    }

    fn enum_pattern_coverage_keys(
        &self,
        domain: &EnumDomain,
        path: &[String],
        sub_patterns: &[Pattern],
    ) -> Vec<String> {
        let Some((variant, details)) = matching_variant(domain, path) else {
            return Vec::new();
        };

        if details.fields.is_empty() {
            if sub_patterns.is_empty() {
                return vec![variant_label(domain, variant)];
            }
            return Vec::new();
        }

        if sub_patterns.len() != details.fields.len() {
            return Vec::new();
        }

        let mut seen = BTreeSet::from([domain.info.path.clone()]);
        let field_scope = self.scope_for_enum(&domain.info);
        let Some(field_domains) =
            self.enumerable_field_domains(&details.fields, &mut seen, &field_scope)
        else {
            if sub_patterns.iter().all(is_catch_all) {
                return vec![variant_label(domain, variant)];
            }
            return Vec::new();
        };

        let mut field_keys = Vec::new();
        for (domain, pattern) in field_domains.iter().zip(sub_patterns) {
            let keys = self.coverage_keys(domain, pattern);
            if keys.is_empty() {
                return Vec::new();
            }
            field_keys.push(keys.into_iter().collect::<Vec<_>>());
        }

        nested_variant_keys(domain, variant, &field_keys)
    }

    fn enumerable_field_domains(
        &self,
        fields: &[TypeExpr],
        seen: &mut BTreeSet<Vec<String>>,
        scope: &Scope,
    ) -> Option<Vec<FiniteDomain>> {
        let mut domains = Vec::new();
        for field in fields {
            let domain = self.domain_from_type(field, scope)?;
            if let FiniteDomain::Enum(enum_domain) = &domain {
                if seen.contains(&enum_domain.info.path) {
                    return None;
                }
            }
            domains.push(domain);
        }
        Some(domains)
    }

    fn domain_from_expr(&self, expr: &Expr, env: &Env, scope: &Scope) -> Option<FiniteDomain> {
        match expr {
            Expr::Bool(_, _) => Some(FiniteDomain::Bool),
            Expr::Ident(name, _) => env.get(name).cloned(),
            Expr::Path(path, _) => self.domain_from_enum_variant_path(path, scope),
            Expr::Call { callee, .. } => match callee.as_ref() {
                Expr::Path(path, _) => self.domain_from_enum_variant_path(path, scope),
                _ => None,
            },
            Expr::Cast { ty, .. } => self.domain_from_type(ty, scope),
            _ => None,
        }
    }

    fn domain_from_enum_variant_path(
        &self,
        path: &[String],
        scope: &Scope,
    ) -> Option<FiniteDomain> {
        let (variant, enum_path) = path.split_last()?;
        if enum_path.is_empty() {
            return None;
        }
        let domain = self.resolve_enum(enum_path, scope)?;
        domain
            .info
            .variants
            .contains_key(variant)
            .then_some(FiniteDomain::Enum(domain))
    }

    fn domain_from_type(&self, ty: &TypeExpr, scope: &Scope) -> Option<FiniteDomain> {
        let TypeExpr::Named { path, .. } = ty else {
            return None;
        };
        if path.last().is_some_and(|name| name == "Bool") {
            return Some(FiniteDomain::Bool);
        }
        self.resolve_enum(path, scope).map(FiniteDomain::Enum)
    }

    fn resolve_enum(&self, path: &[String], scope: &Scope) -> Option<EnumDomain> {
        let mut candidates = BTreeMap::<Vec<String>, BTreeSet<Vec<String>>>::new();
        self.add_enum_candidate(&mut candidates, path, path);

        if !scope.module_path.is_empty() {
            let mut relative = scope.module_path.clone();
            relative.extend_from_slice(path);
            self.add_enum_candidate(&mut candidates, &relative, path);
        }

        if path.len() == 1 {
            if let Some(imported) = scope.enum_imports.get(&path[0]) {
                self.add_enum_candidate(&mut candidates, imported, path);
            }
            for module_path in &scope.glob_imports {
                let mut imported = module_path.clone();
                imported.push(path[0].clone());
                self.add_enum_candidate(&mut candidates, &imported, path);
            }
        }

        if path.len() > 1 {
            if let Some(module_path) = scope.module_imports.get(&path[0]) {
                let mut imported = module_path.clone();
                imported.extend_from_slice(&path[1..]);
                self.add_enum_candidate(&mut candidates, &imported, path);
            }
        }

        if candidates.len() != 1 {
            return None;
        }
        let (resolved, pattern_prefixes) = candidates.into_iter().next()?;
        let info = self.enums.get(&resolved)?.clone();
        Some(EnumDomain {
            info,
            pattern_prefixes,
        })
    }

    fn add_enum_candidate(
        &self,
        candidates: &mut BTreeMap<Vec<String>, BTreeSet<Vec<String>>>,
        resolved: &[String],
        source_prefix: &[String],
    ) {
        if self.enums.contains_key(resolved) {
            let entry = candidates.entry(resolved.to_vec()).or_default();
            entry.insert(resolved.to_vec());
            entry.insert(source_prefix.to_vec());
        }
    }

    fn scope_for(&self, module_path: &[String]) -> Scope {
        self.scopes
            .get(module_path)
            .cloned()
            .unwrap_or_else(|| Scope {
                module_path: module_path.to_vec(),
                ..Scope::default()
            })
    }

    fn scope_for_enum(&self, info: &EnumInfo) -> Scope {
        let mut module_path = info.path.clone();
        module_path.pop();
        self.scope_for(&module_path)
    }

    fn canonical_use_path(&self, path: &[String], module_path: &[String]) -> Option<Vec<String>> {
        let mut candidates = BTreeSet::new();
        if self.use_path_or_prefix_exists(path) {
            candidates.insert(path.to_vec());
        }
        if !module_path.is_empty() {
            let mut relative = module_path.to_vec();
            relative.extend_from_slice(path);
            if self.use_path_or_prefix_exists(&relative) {
                candidates.insert(relative);
            }
        }

        match candidates.len() {
            0 => Some(path.to_vec()),
            1 => candidates.into_iter().next(),
            _ => None,
        }
    }

    fn use_path_or_prefix_exists(&self, path: &[String]) -> bool {
        self.enum_path_or_prefix_exists(path) || self.const_path_or_prefix_exists(path)
    }

    fn enum_path_or_prefix_exists(&self, path: &[String]) -> bool {
        self.enums
            .keys()
            .any(|enum_path| enum_path.as_slice().starts_with(path))
    }

    fn const_path_or_prefix_exists(&self, path: &[String]) -> bool {
        self.const_guard_facts
            .keys()
            .any(|const_path| const_path.as_slice().starts_with(path))
    }

    fn guard_coverage(
        &self,
        guard: &Option<Expr>,
        guard_facts: &GuardFacts,
        scope: &Scope,
    ) -> GuardCoverage {
        match guard {
            None => GuardCoverage::Unguarded,
            Some(expr) => match self.guard_fact_from_match_guard(expr, guard_facts, scope) {
                Some(true) => GuardCoverage::AlwaysTrue,
                Some(false) => GuardCoverage::AlwaysFalse,
                None => GuardCoverage::Unknown,
            },
        }
    }

    fn guard_fact_from_match_guard(
        &self,
        expr: &Expr,
        guard_facts: &GuardFacts,
        scope: &Scope,
    ) -> Option<bool> {
        self.guard_value_from_match_guard(expr, guard_facts, scope)?
            .into_bool()
    }

    fn guard_value_from_match_guard(
        &self,
        expr: &Expr,
        guard_facts: &GuardFacts,
        scope: &Scope,
    ) -> Option<ConstFact> {
        match expr {
            Expr::Bool(value, _) => Some(ConstFact::Bool(*value)),
            Expr::Int(value, _) => Some(ConstFact::Int(*value)),
            Expr::Float(value, _) => finite_float_fact(*value),
            Expr::Nil(_) => Some(ConstFact::Nil),
            Expr::Symbol(value, _) => Some(ConstFact::Symbol(value.clone())),
            Expr::Str(value, _) => const_string_literal(value, |src| {
                interpolation_const_fact(src, |expr| {
                    self.guard_value_from_match_guard(expr, guard_facts, scope)
                })
            })
            .map(ConstFact::Str),
            Expr::Ident(name, _) => guard_facts.get(name).cloned(),
            Expr::Path(path, _) => self.resolve_const_fact_path(path, scope),
            Expr::Unary {
                op: UnOp::Not,
                expr,
                ..
            } => self
                .guard_value_from_match_guard(expr, guard_facts, scope)
                .and_then(ConstFact::into_bool)
                .map(|value| ConstFact::Bool(!value)),
            Expr::Unary {
                op: UnOp::Neg,
                expr,
                ..
            } => match self.guard_value_from_match_guard(expr, guard_facts, scope)? {
                ConstFact::Int(value) => value.checked_neg().map(ConstFact::Int),
                ConstFact::Float(value) => finite_float_fact(-value),
                _ => None,
            },
            Expr::Unary { .. } => None,
            Expr::Binary {
                op: BinOp::And,
                lhs,
                rhs,
                ..
            } => {
                let lhs = self
                    .guard_value_from_match_guard(lhs, guard_facts, scope)?
                    .into_bool()?;
                if !lhs {
                    Some(ConstFact::Bool(false))
                } else {
                    let rhs = self
                        .guard_value_from_match_guard(rhs, guard_facts, scope)?
                        .into_bool()?;
                    Some(ConstFact::Bool(rhs))
                }
            }
            Expr::Binary {
                op: BinOp::Or,
                lhs,
                rhs,
                ..
            } => {
                let lhs = self
                    .guard_value_from_match_guard(lhs, guard_facts, scope)?
                    .into_bool()?;
                if lhs {
                    Some(ConstFact::Bool(true))
                } else {
                    let rhs = self
                        .guard_value_from_match_guard(rhs, guard_facts, scope)?
                        .into_bool()?;
                    Some(ConstFact::Bool(rhs))
                }
            }
            Expr::Binary {
                op: BinOp::Eq,
                lhs,
                rhs,
                ..
            } => {
                let lhs = self.guard_value_from_match_guard(lhs, guard_facts, scope)?;
                let rhs = self.guard_value_from_match_guard(rhs, guard_facts, scope)?;
                Some(ConstFact::Bool(lhs.literal_eq(rhs)))
            }
            Expr::Binary {
                op: BinOp::NotEq,
                lhs,
                rhs,
                ..
            } => {
                let lhs = self.guard_value_from_match_guard(lhs, guard_facts, scope)?;
                let rhs = self.guard_value_from_match_guard(rhs, guard_facts, scope)?;
                Some(ConstFact::Bool(!lhs.literal_eq(rhs)))
            }
            Expr::Binary {
                op: op @ (BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq),
                lhs,
                rhs,
                ..
            } => {
                let lhs = self.guard_value_from_match_guard(lhs, guard_facts, scope)?;
                let rhs = self.guard_value_from_match_guard(rhs, guard_facts, scope)?;
                lhs.ordered_cmp(rhs, *op).map(ConstFact::Bool)
            }
            Expr::Binary {
                op: op @ (BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod),
                lhs,
                rhs,
                ..
            } => {
                let lhs = self.guard_value_from_match_guard(lhs, guard_facts, scope)?;
                let rhs = self.guard_value_from_match_guard(rhs, guard_facts, scope)?;
                lhs.numeric_arith(rhs, *op)
            }
            Expr::Binary { .. } => None,
            _ => None,
        }
    }

    fn resolve_const_fact_path(&self, path: &[String], scope: &Scope) -> Option<ConstFact> {
        let mut candidates = BTreeMap::new();
        self.add_const_fact_candidate(&mut candidates, path);

        if !scope.module_path.is_empty() {
            let mut relative = scope.module_path.clone();
            relative.extend_from_slice(path);
            self.add_const_fact_candidate(&mut candidates, &relative);
        }

        if path.len() > 1 {
            if let Some(module_path) = scope.module_imports.get(&path[0]) {
                let mut imported = module_path.clone();
                imported.extend_from_slice(&path[1..]);
                self.add_const_fact_candidate(&mut candidates, &imported);
            }
        }

        if candidates.len() != 1 {
            return None;
        }
        candidates.into_values().next()
    }

    fn add_const_fact_candidate(
        &self,
        candidates: &mut BTreeMap<Vec<String>, ConstFact>,
        path: &[String],
    ) {
        if let Some(value) = self.const_guard_facts.get(path) {
            candidates.insert(path.to_vec(), value.clone());
        }
    }

    fn same_module_const_guard_facts(&self, module_path: &[String]) -> GuardFacts {
        self.const_guard_facts_in_module(module_path)
            .into_iter()
            .filter_map(|(path, value)| path.last().map(|name| (name.clone(), value)))
            .collect()
    }

    fn const_guard_facts_in_module(&self, module_path: &[String]) -> Vec<(Vec<String>, ConstFact)> {
        self.const_guard_facts
            .iter()
            .filter_map(|(path, value)| {
                path.split_last()
                    .filter(|(_, parent)| parent == &module_path)
                    .map(|_| (path.clone(), value.clone()))
            })
            .collect()
    }
}

fn matching_variant<'a>(
    domain: &'a EnumDomain,
    path: &[String],
) -> Option<(&'a str, &'a VariantInfo)> {
    let variant = path.last()?;
    let (variant, details) = domain.info.variants.get_key_value(variant)?;
    let enum_prefix = &path[..path.len().saturating_sub(1)];
    if enum_prefix.is_empty()
        || domain
            .pattern_prefixes
            .iter()
            .any(|prefix| prefix.as_slice() == enum_prefix)
    {
        Some((variant.as_str(), details))
    } else {
        None
    }
}

fn variant_label(domain: &EnumDomain, variant: &str) -> String {
    format!("{}::{variant}", domain.info.path.join("::"))
}

fn nested_variant_keys(
    domain: &EnumDomain,
    variant: &str,
    field_keys: &[Vec<String>],
) -> Vec<String> {
    let mut combinations = vec![Vec::<String>::new()];
    for keys in field_keys {
        let mut next = Vec::new();
        for existing in &combinations {
            for key in keys {
                let mut joined = existing.clone();
                joined.push(key.clone());
                next.push(joined);
            }
        }
        combinations = next;
    }

    combinations
        .into_iter()
        .map(|parts| format!("{}({})", variant_label(domain, variant), parts.join(", ")))
        .collect()
}

fn join_branch_envs(
    branch_base: &Env,
    branch_envs: &[Env],
    branch_targets: &[BTreeSet<String>],
) -> Env {
    let Some(first) = branch_envs.first() else {
        return Env::new();
    };

    let mut eligible = branch_base.keys().cloned().collect::<BTreeSet<_>>();
    if let Some(first_targets) = branch_targets.first() {
        for target in first_targets {
            if branch_targets
                .iter()
                .skip(1)
                .all(|targets| targets.contains(target))
            {
                eligible.insert(target.clone());
            }
        }
    }

    let mut joined = Env::new();
    'bindings: for (name, domain) in first {
        if !eligible.contains(name) {
            continue;
        }
        for branch in &branch_envs[1..] {
            let Some(other) = branch.get(name) else {
                continue 'bindings;
            };
            if !domains_equivalent(domain, other) {
                continue 'bindings;
            }
        }
        joined.insert(name.clone(), domain.clone());
    }
    joined
}

fn join_branch_closure_effects(branch_effects: &[ClosureEffects]) -> ClosureEffects {
    let Some(first) = branch_effects.first() else {
        return ClosureEffects::new();
    };

    let mut joined = ClosureEffects::new();
    'bindings: for (name, targets) in first {
        let mut merged = targets.clone();
        for branch in &branch_effects[1..] {
            let Some(other) = branch.get(name) else {
                continue 'bindings;
            };
            merged.extend(other.iter().cloned());
        }
        joined.insert(name.clone(), merged);
    }
    joined
}

fn join_branch_guard_facts(branch_base: &GuardFacts, branch_facts: &[GuardFacts]) -> GuardFacts {
    let Some(first) = branch_facts.first() else {
        return GuardFacts::new();
    };

    let mut joined = GuardFacts::new();
    'bindings: for (name, value) in first {
        if !branch_base.contains_key(name) {
            continue;
        }
        for branch in &branch_facts[1..] {
            let Some(other) = branch.get(name) else {
                continue 'bindings;
            };
            if other != value {
                continue 'bindings;
            }
        }
        joined.insert(name.clone(), value.clone());
    }
    joined
}

fn assigned_outer_targets_in_block(block: &Block) -> BTreeSet<String> {
    assigned_outer_targets_in_block_excluding(block, &BTreeSet::new())
}

fn assigned_outer_targets_in_block_excluding(
    block: &Block,
    outer_local_bindings: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut local_bindings = outer_local_bindings.clone();
    let mut assigned = BTreeSet::new();
    for stmt in &block.stmts {
        assigned.extend(assigned_outer_targets_in_stmt(stmt, &local_bindings));
        match stmt {
            Stmt::Let(decl) => {
                local_bindings.insert(decl.name.clone());
            }
            Stmt::Var(decl) => {
                local_bindings.insert(decl.name.clone());
            }
            Stmt::Const(decl) => {
                local_bindings.insert(decl.name.clone());
            }
            _ => {}
        }
    }

    if let Some(tail) = &block.tail_expr {
        assigned.extend(assigned_outer_targets_in_expr(tail, &local_bindings));
    }
    assigned
}

fn assigned_outer_targets_in_stmt(
    stmt: &Stmt,
    local_bindings: &BTreeSet<String>,
) -> BTreeSet<String> {
    match stmt {
        Stmt::Assign {
            target: Expr::Ident(name, _),
            ..
        } if !local_bindings.contains(name) => BTreeSet::from([name.clone()]),
        Stmt::Let(decl) => assigned_outer_targets_in_expr(&decl.value, local_bindings),
        Stmt::Var(decl) => assigned_outer_targets_in_expr(&decl.value, local_bindings),
        Stmt::Const(decl) => assigned_outer_targets_in_expr(&decl.value, local_bindings),
        Stmt::Return { value, .. }
        | Stmt::Yield { value, .. }
        | Stmt::Next { value, .. }
        | Stmt::Break { value, .. } => value
            .as_ref()
            .map(|value| assigned_outer_targets_in_expr(value, local_bindings))
            .unwrap_or_default(),
        Stmt::Raise { value, .. } | Stmt::Expr(value) => {
            assigned_outer_targets_in_expr(value, local_bindings)
        }
        _ => BTreeSet::new(),
    }
}

fn assigned_outer_targets_in_expr(
    expr: &Expr,
    local_bindings: &BTreeSet<String>,
) -> BTreeSet<String> {
    match expr {
        Expr::If {
            then_block,
            elsif_clauses,
            else_block,
            ..
        } => {
            let Some(else_block) = else_block else {
                return BTreeSet::new();
            };

            let mut branch_sets = Vec::with_capacity(elsif_clauses.len() + 2);
            branch_sets.push(assigned_outer_targets_in_block_excluding(
                then_block,
                local_bindings,
            ));
            for (_, block) in elsif_clauses {
                branch_sets.push(assigned_outer_targets_in_block_excluding(
                    block,
                    local_bindings,
                ));
            }
            branch_sets.push(assigned_outer_targets_in_block_excluding(
                else_block,
                local_bindings,
            ));

            intersect_assigned_targets(&branch_sets)
        }
        _ => BTreeSet::new(),
    }
}

fn maybe_assigned_outer_targets_in_block(block: &Block) -> BTreeSet<String> {
    maybe_assigned_outer_targets_in_block_excluding(block, &BTreeSet::new())
}

fn maybe_assigned_outer_targets_in_block_excluding(
    block: &Block,
    outer_local_bindings: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut local_bindings = outer_local_bindings.clone();
    let mut assigned = BTreeSet::new();
    for stmt in &block.stmts {
        assigned.extend(maybe_assigned_outer_targets_in_stmt(stmt, &local_bindings));
        match stmt {
            Stmt::Let(decl) => {
                local_bindings.insert(decl.name.clone());
            }
            Stmt::Var(decl) => {
                local_bindings.insert(decl.name.clone());
            }
            Stmt::Const(decl) => {
                local_bindings.insert(decl.name.clone());
            }
            _ => {}
        }
    }

    if let Some(tail) = &block.tail_expr {
        assigned.extend(maybe_assigned_outer_targets_in_expr(tail, &local_bindings));
    }
    assigned
}

fn maybe_assigned_outer_targets_in_stmt(
    stmt: &Stmt,
    local_bindings: &BTreeSet<String>,
) -> BTreeSet<String> {
    match stmt {
        Stmt::Assign {
            target: Expr::Ident(name, _),
            ..
        } if !local_bindings.contains(name) => BTreeSet::from([name.clone()]),
        Stmt::Let(decl) => maybe_assigned_outer_targets_in_expr(&decl.value, local_bindings),
        Stmt::Var(decl) => maybe_assigned_outer_targets_in_expr(&decl.value, local_bindings),
        Stmt::Const(decl) => maybe_assigned_outer_targets_in_expr(&decl.value, local_bindings),
        Stmt::While { body, .. } | Stmt::Loop { body, .. } => {
            maybe_assigned_outer_targets_in_block_excluding(body, local_bindings)
        }
        Stmt::For { var, body, .. } => {
            let mut nested_locals = local_bindings.clone();
            nested_locals.insert(var.clone());
            maybe_assigned_outer_targets_in_block_excluding(body, &nested_locals)
        }
        Stmt::Return { value, .. }
        | Stmt::Yield { value, .. }
        | Stmt::Next { value, .. }
        | Stmt::Break { value, .. } => value
            .as_ref()
            .map(|value| maybe_assigned_outer_targets_in_expr(value, local_bindings))
            .unwrap_or_default(),
        Stmt::Raise { value, .. } | Stmt::Expr(value) => {
            maybe_assigned_outer_targets_in_expr(value, local_bindings)
        }
        _ => BTreeSet::new(),
    }
}

fn maybe_assigned_outer_targets_in_expr(
    expr: &Expr,
    local_bindings: &BTreeSet<String>,
) -> BTreeSet<String> {
    match expr {
        Expr::If {
            then_block,
            elsif_clauses,
            else_block,
            ..
        } => {
            let mut assigned =
                maybe_assigned_outer_targets_in_block_excluding(then_block, local_bindings);
            for (_, block) in elsif_clauses {
                assigned.extend(maybe_assigned_outer_targets_in_block_excluding(
                    block,
                    local_bindings,
                ));
            }
            if let Some(else_block) = else_block {
                assigned.extend(maybe_assigned_outer_targets_in_block_excluding(
                    else_block,
                    local_bindings,
                ));
            }
            assigned
        }
        Expr::Try {
            body,
            rescues,
            ensure,
            ..
        } => {
            let mut assigned =
                maybe_assigned_outer_targets_in_block_excluding(body, local_bindings);
            for rescue in rescues {
                assigned.extend(maybe_assigned_outer_targets_in_block_excluding(
                    &rescue.body,
                    local_bindings,
                ));
            }
            if let Some(ensure) = ensure {
                assigned.extend(maybe_assigned_outer_targets_in_block_excluding(
                    ensure,
                    local_bindings,
                ));
            }
            assigned
        }
        Expr::Call { callee, args, .. } => {
            let mut assigned = maybe_assigned_outer_targets_in_expr(callee, local_bindings);
            for arg in args {
                assigned.extend(maybe_assigned_outer_targets_in_expr(arg, local_bindings));
            }
            if let Expr::Closure { params, body, .. } = callee.as_ref() {
                assigned.extend(maybe_assigned_outer_targets_in_closure(
                    params,
                    body.as_ref(),
                    local_bindings,
                ));
            }
            assigned
        }
        Expr::Method { receiver, args, .. } => {
            let mut assigned = maybe_assigned_outer_targets_in_expr(receiver, local_bindings);
            for arg in args {
                assigned.extend(maybe_assigned_outer_targets_in_expr(arg, local_bindings));
            }
            assigned
        }
        Expr::Closure { .. } => BTreeSet::new(),
        _ => BTreeSet::new(),
    }
}

fn maybe_assigned_outer_targets_in_closure(
    params: &[Param],
    body: &ClosureBody,
    local_bindings: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut closure_locals = local_bindings.clone();
    for param in params {
        closure_locals.insert(param.name.clone());
    }
    match body {
        ClosureBody::Block(block) => {
            maybe_assigned_outer_targets_in_block_excluding(block, &closure_locals)
        }
        ClosureBody::Expr(expr) => maybe_assigned_outer_targets_in_expr(expr, &closure_locals),
    }
}

fn update_closure_effect(name: String, value: &Expr, closure_effects: &mut ClosureEffects) {
    let effect = closure_effect_from_value(value, closure_effects);

    if let Some(targets) = effect {
        closure_effects.insert(name, targets);
    } else {
        closure_effects.remove(&name);
    }
}

fn update_guard_fact_for_let<F>(
    decl: &garnet_parser::ast::LetDecl,
    guard_facts: &mut GuardFacts,
    resolve_path: &F,
) where
    F: Fn(&[String]) -> Option<ConstFact>,
{
    if decl.mutable {
        guard_facts.remove(&decl.name);
        return;
    }
    update_guard_fact(decl.name.clone(), &decl.value, guard_facts, resolve_path);
}

fn update_guard_fact<F>(name: String, value: &Expr, guard_facts: &mut GuardFacts, resolve_path: &F)
where
    F: Fn(&[String]) -> Option<ConstFact>,
{
    if let Some(value) = local_guard_fact_from_expr(value, guard_facts, resolve_path) {
        guard_facts.insert(name, value);
    } else {
        guard_facts.remove(&name);
    }
}

fn collect_const_guard_decls<'a>(
    items: &'a [Item],
    prefix: &[String],
    out: &mut Vec<(Vec<String>, &'a Expr)>,
) {
    for item in items {
        match item {
            Item::Const(decl) => {
                let mut path = prefix.to_vec();
                path.push(decl.name.clone());
                out.push((path, &decl.value));
            }
            Item::Module(module) => {
                let mut nested = prefix.to_vec();
                nested.push(module.name.clone());
                collect_const_guard_decls(&module.items, &nested, out);
            }
            _ => {}
        }
    }
}

fn local_guard_fact_from_expr<F>(
    expr: &Expr,
    guard_facts: &GuardFacts,
    resolve_path: &F,
) -> Option<ConstFact>
where
    F: Fn(&[String]) -> Option<ConstFact>,
{
    match expr {
        Expr::Bool(value, _) => Some(ConstFact::Bool(*value)),
        Expr::Int(value, _) => Some(ConstFact::Int(*value)),
        Expr::Float(value, _) => finite_float_fact(*value),
        Expr::Nil(_) => Some(ConstFact::Nil),
        Expr::Symbol(value, _) => Some(ConstFact::Symbol(value.clone())),
        Expr::Str(value, _) => const_string_literal(value, |src| {
            interpolation_const_fact(src, |expr| {
                local_guard_fact_from_expr(expr, guard_facts, resolve_path)
            })
        })
        .map(ConstFact::Str),
        Expr::Ident(name, _) => guard_facts.get(name).cloned(),
        Expr::Path(path, _) => resolve_path(path),
        Expr::Unary {
            op: UnOp::Not,
            expr,
            ..
        } => local_guard_fact_from_expr(expr, guard_facts, resolve_path)
            .and_then(ConstFact::into_bool)
            .map(|value| ConstFact::Bool(!value)),
        Expr::Unary {
            op: UnOp::Neg,
            expr,
            ..
        } => match local_guard_fact_from_expr(expr, guard_facts, resolve_path)? {
            ConstFact::Int(value) => value.checked_neg().map(ConstFact::Int),
            ConstFact::Float(value) => finite_float_fact(-value),
            _ => None,
        },
        Expr::Unary { .. } => None,
        Expr::Binary {
            op: BinOp::And,
            lhs,
            rhs,
            ..
        } => {
            let lhs = local_guard_fact_from_expr(lhs, guard_facts, resolve_path)?.into_bool()?;
            if !lhs {
                Some(ConstFact::Bool(false))
            } else {
                let rhs =
                    local_guard_fact_from_expr(rhs, guard_facts, resolve_path)?.into_bool()?;
                Some(ConstFact::Bool(rhs))
            }
        }
        Expr::Binary {
            op: BinOp::Or,
            lhs,
            rhs,
            ..
        } => {
            let lhs = local_guard_fact_from_expr(lhs, guard_facts, resolve_path)?.into_bool()?;
            if lhs {
                Some(ConstFact::Bool(true))
            } else {
                let rhs =
                    local_guard_fact_from_expr(rhs, guard_facts, resolve_path)?.into_bool()?;
                Some(ConstFact::Bool(rhs))
            }
        }
        Expr::Binary {
            op: BinOp::Eq,
            lhs,
            rhs,
            ..
        } => {
            let lhs = local_guard_fact_from_expr(lhs, guard_facts, resolve_path)?;
            let rhs = local_guard_fact_from_expr(rhs, guard_facts, resolve_path)?;
            Some(ConstFact::Bool(lhs.literal_eq(rhs)))
        }
        Expr::Binary {
            op: BinOp::NotEq,
            lhs,
            rhs,
            ..
        } => {
            let lhs = local_guard_fact_from_expr(lhs, guard_facts, resolve_path)?;
            let rhs = local_guard_fact_from_expr(rhs, guard_facts, resolve_path)?;
            Some(ConstFact::Bool(!lhs.literal_eq(rhs)))
        }
        Expr::Binary {
            op: op @ (BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq),
            lhs,
            rhs,
            ..
        } => {
            let lhs = local_guard_fact_from_expr(lhs, guard_facts, resolve_path)?;
            let rhs = local_guard_fact_from_expr(rhs, guard_facts, resolve_path)?;
            lhs.ordered_cmp(rhs, *op).map(ConstFact::Bool)
        }
        Expr::Binary {
            op: op @ (BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod),
            lhs,
            rhs,
            ..
        } => {
            let lhs = local_guard_fact_from_expr(lhs, guard_facts, resolve_path)?;
            let rhs = local_guard_fact_from_expr(rhs, guard_facts, resolve_path)?;
            lhs.numeric_arith(rhs, *op)
        }
        Expr::Binary { .. } => None,
        _ => None,
    }
}

fn closure_effect_from_value(
    expr: &Expr,
    closure_effects: &ClosureEffects,
) -> Option<BTreeSet<String>> {
    match expr {
        Expr::Closure { params, body, .. } => Some(maybe_assigned_outer_targets_in_closure(
            params,
            body.as_ref(),
            &BTreeSet::new(),
        )),
        Expr::Ident(source, _) => closure_effects.get(source).cloned(),
        Expr::If {
            then_block,
            elsif_clauses,
            else_block: Some(else_block),
            ..
        } => {
            let mut targets = closure_effect_from_block_tail(then_block, closure_effects)?;
            for (_, block) in elsif_clauses {
                targets.extend(closure_effect_from_block_tail(block, closure_effects)?);
            }
            targets.extend(closure_effect_from_block_tail(else_block, closure_effects)?);
            Some(targets)
        }
        _ => None,
    }
}

fn closure_effect_from_block_tail(
    block: &Block,
    closure_effects: &ClosureEffects,
) -> Option<BTreeSet<String>> {
    let mut branch_effects = closure_effects.clone();
    for stmt in &block.stmts {
        match stmt {
            Stmt::Let(decl) => {
                update_closure_effect(decl.name.clone(), &decl.value, &mut branch_effects);
            }
            Stmt::Var(decl) => {
                update_closure_effect(decl.name.clone(), &decl.value, &mut branch_effects);
            }
            Stmt::Const(decl) => {
                update_closure_effect(decl.name.clone(), &decl.value, &mut branch_effects);
            }
            Stmt::Assign {
                target: Expr::Ident(name, _),
                op: AssignOp::Eq,
                value,
                ..
            } => {
                update_closure_effect(name.clone(), value, &mut branch_effects);
            }
            Stmt::Assign {
                target: Expr::Ident(name, _),
                ..
            } => {
                branch_effects.remove(name);
            }
            _ => {}
        }
    }
    closure_effect_from_value(block.tail_expr.as_deref()?, &branch_effects)
}

fn intersect_assigned_targets(branch_sets: &[BTreeSet<String>]) -> BTreeSet<String> {
    let Some(first) = branch_sets.first() else {
        return BTreeSet::new();
    };

    first
        .iter()
        .filter(|target| {
            branch_sets
                .iter()
                .skip(1)
                .all(|targets| targets.contains(*target))
        })
        .cloned()
        .collect()
}

fn domains_equivalent(left: &FiniteDomain, right: &FiniteDomain) -> bool {
    match (left, right) {
        (FiniteDomain::Bool, FiniteDomain::Bool) => true,
        (FiniteDomain::Enum(left), FiniteDomain::Enum(right)) => {
            left.info.path == right.info.path && left.pattern_prefixes == right.pattern_prefixes
        }
        _ => false,
    }
}

fn is_catch_all(pattern: &Pattern) -> bool {
    matches!(
        pattern,
        Pattern::Ident(_, _) | Pattern::Wildcard(_) | Pattern::Rest(_)
    )
}

fn literal_pattern_key(pattern: &Pattern) -> Option<String> {
    let Pattern::Literal(expr, _) = pattern else {
        return None;
    };

    match expr {
        Expr::Bool(value, _) => Some(format!("bool:{value}")),
        Expr::Int(value, _) => Some(format!("int:{value}")),
        Expr::Float(value, _) => Some(format!("float:{value}")),
        Expr::Nil(_) => Some("nil".to_string()),
        Expr::Str(value, _) => Some(format!("str:{}", describe_string_literal(value))),
        Expr::Symbol(value, _) => Some(format!("symbol:{value}")),
        _ => None,
    }
}

fn describe_pattern(pattern: &Pattern) -> String {
    match pattern {
        Pattern::Literal(expr, _) => match expr {
            Expr::Bool(value, _) => format!("`{value}`"),
            Expr::Int(value, _) => format!("`{value}`"),
            Expr::Float(value, _) => format!("`{value}`"),
            Expr::Nil(_) => "`nil`".to_string(),
            Expr::Str(value, _) => format!("`\"{}\"`", describe_string_literal(value)),
            Expr::Symbol(value, _) => format!("`:{value}`"),
            _ => "`<literal>`".to_string(),
        },
        Pattern::Ident(name, _) => format!("`{name}`"),
        Pattern::Tuple(_, _) => "`<tuple>`".to_string(),
        Pattern::Enum(path, _, _) => format!("`{}`", path.join("::")),
        Pattern::Wildcard(_) => "`_`".to_string(),
        Pattern::Rest(_) => "`..`".to_string(),
    }
}

fn describe_string_literal(value: &StringLit) -> String {
    value
        .parts
        .iter()
        .map(|part| match part {
            StrPart::Lit(text) => text.as_str(),
            StrPart::Interp(_) => "#{...}",
        })
        .collect()
}

fn const_string_literal<F>(value: &StringLit, mut interpolation_fact: F) -> Option<String>
where
    F: FnMut(&str) -> Option<ConstFact>,
{
    let mut text = String::new();
    for part in &value.parts {
        match part {
            StrPart::Lit(part) => text.push_str(part),
            StrPart::Interp(src) => {
                text.push_str(&const_fact_interpolation_text(interpolation_fact(src)?))
            }
        }
    }
    Some(text)
}

fn interpolation_const_fact<F>(src: &str, mut expr_fact: F) -> Option<ConstFact>
where
    F: FnMut(&Expr) -> Option<ConstFact>,
{
    let wrapped = format!("def __garnet_const_interp__() {{ {src} }}");
    let module = garnet_parser::parse_source(&wrapped).ok()?;
    for item in &module.items {
        if let Item::Fn(fn_def) = item {
            if let Some(tail) = &fn_def.body.tail_expr {
                return expr_fact(tail);
            }
        }
    }
    None
}

fn const_fact_interpolation_text(value: ConstFact) -> String {
    match value {
        ConstFact::Bool(value) => value.to_string(),
        ConstFact::Int(value) => value.to_string(),
        ConstFact::Float(value) => format!("{value}"),
        ConstFact::Nil => "nil".to_string(),
        ConstFact::Symbol(value) => format!(":{value}"),
        ConstFact::Str(value) => value,
    }
}

fn finite_float_fact(value: f64) -> Option<ConstFact> {
    value.is_finite().then_some(ConstFact::Float(value))
}

fn float_arith(lhs: f64, rhs: f64, op: BinOp, allow_mod: bool) -> Option<ConstFact> {
    let value = match op {
        BinOp::Add => lhs + rhs,
        BinOp::Sub => lhs - rhs,
        BinOp::Mul => lhs * rhs,
        BinOp::Div if rhs != 0.0 => lhs / rhs,
        BinOp::Mod if allow_mod => lhs % rhs,
        _ => return None,
    };

    finite_float_fact(value)
}

fn remove_pattern_bindings(pattern: &Pattern, env: &mut Env) {
    match pattern {
        Pattern::Ident(name, _) => {
            env.remove(name);
        }
        Pattern::Tuple(items, _) | Pattern::Enum(_, items, _) => {
            for item in items {
                remove_pattern_bindings(item, env);
            }
        }
        Pattern::Literal(_, _) | Pattern::Wildcard(_) | Pattern::Rest(_) => {}
    }
}

fn remove_pattern_guard_facts(pattern: &Pattern, guard_facts: &mut GuardFacts) {
    match pattern {
        Pattern::Ident(name, _) => {
            guard_facts.remove(name);
        }
        Pattern::Tuple(items, _) | Pattern::Enum(_, items, _) => {
            for item in items {
                remove_pattern_guard_facts(item, guard_facts);
            }
        }
        Pattern::Literal(_, _) | Pattern::Wildcard(_) | Pattern::Rest(_) => {}
    }
}
