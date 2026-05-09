//! Conservative safe-mode match coverage for finite domains.
//!
//! This is intentionally a narrow semantic slice: it proves exhaustiveness and
//! simple reachability for `Bool` and same-module enum subjects whose type is
//! visible from parameter or local annotation metadata. It does not attempt
//! full type inference, nested constructor coverage, or guard reasoning.

use crate::CheckError;
use garnet_parser::ast::{
    Block, ClosureBody, Expr, FnDef, FnMode, Item, Module, Pattern, Stmt, StringLit, TypeExpr,
};
use garnet_parser::token::StrPart;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
struct EnumInfo {
    path: Vec<String>,
    variants: BTreeSet<String>,
}

#[derive(Debug, Clone)]
enum FiniteDomain {
    Bool,
    Enum(EnumInfo),
}

#[derive(Debug, Default)]
struct Checker {
    enums: BTreeMap<Vec<String>, EnumInfo>,
    errors: Vec<CheckError>,
}

type Env = BTreeMap<String, FiniteDomain>;

pub fn check_match_coverage(module: &Module) -> Vec<CheckError> {
    let mut checker = Checker::default();
    checker.collect_enums(&module.items, &[]);
    checker.check_items(&module.items, module.safe);
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
                        .map(|variant| variant.name.clone())
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

    fn check_items(&mut self, items: &[Item], module_safe: bool) {
        for item in items {
            match item {
                Item::Fn(f) => self.check_fn(f, module_safe || f.mode == FnMode::Safe),
                Item::Module(module) => {
                    self.check_items(&module.items, module_safe || module.safe);
                }
                Item::Impl(impl_block) => {
                    for method in &impl_block.methods {
                        self.check_fn(method, module_safe || method.mode == FnMode::Safe);
                    }
                }
                _ => {}
            }
        }
    }

    fn check_fn(&mut self, f: &FnDef, effective_safe: bool) {
        if !effective_safe {
            return;
        }

        let mut env = Env::new();
        for param in &f.params {
            if let Some(ty) = &param.ty {
                if let Some(domain) = self.domain_from_type(ty) {
                    env.insert(param.name.clone(), domain);
                }
            }
        }

        self.walk_block(&f.body, &f.name, &mut env);
    }

    fn walk_block(&mut self, block: &Block, fn_name: &str, env: &mut Env) {
        for stmt in &block.stmts {
            self.walk_stmt(stmt, fn_name, env);
        }
        if let Some(tail) = &block.tail_expr {
            self.walk_expr(tail, fn_name, env);
        }
    }

    fn walk_stmt(&mut self, stmt: &Stmt, fn_name: &str, env: &mut Env) {
        match stmt {
            Stmt::Let(decl) => {
                self.walk_expr(&decl.value, fn_name, env);
                if let Some(ty) = &decl.ty {
                    if let Some(domain) = self.domain_from_type(ty) {
                        env.insert(decl.name.clone(), domain);
                    }
                }
            }
            Stmt::Var(decl) => {
                self.walk_expr(&decl.value, fn_name, env);
                if let Some(ty) = &decl.ty {
                    if let Some(domain) = self.domain_from_type(ty) {
                        env.insert(decl.name.clone(), domain);
                    }
                }
            }
            Stmt::Const(decl) => {
                self.walk_expr(&decl.value, fn_name, env);
            }
            Stmt::Assign { target, value, .. } => {
                self.walk_expr(target, fn_name, env);
                self.walk_expr(value, fn_name, env);
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.walk_expr(condition, fn_name, env);
                let mut body_env = env.clone();
                self.walk_block(body, fn_name, &mut body_env);
            }
            Stmt::For { iter, body, .. } => {
                self.walk_expr(iter, fn_name, env);
                let mut body_env = env.clone();
                self.walk_block(body, fn_name, &mut body_env);
            }
            Stmt::Loop { body, .. } => {
                let mut body_env = env.clone();
                self.walk_block(body, fn_name, &mut body_env);
            }
            Stmt::Return { value, .. }
            | Stmt::Yield { value, .. }
            | Stmt::Next { value, .. }
            | Stmt::Break { value, .. } => {
                if let Some(value) = value {
                    self.walk_expr(value, fn_name, env);
                }
            }
            Stmt::Raise { value, .. } | Stmt::Expr(value) => {
                self.walk_expr(value, fn_name, env);
            }
            Stmt::Continue { .. } => {}
        }
    }

    fn walk_expr(&mut self, expr: &Expr, fn_name: &str, env: &mut Env) {
        match expr {
            Expr::Binary { lhs, rhs, .. } => {
                self.walk_expr(lhs, fn_name, env);
                self.walk_expr(rhs, fn_name, env);
            }
            Expr::Unary { expr, .. } | Expr::Cast { expr, .. } | Expr::Spawn { expr, .. } => {
                self.walk_expr(expr, fn_name, env);
            }
            Expr::Call { callee, args, .. }
            | Expr::Method {
                receiver: callee,
                args,
                ..
            } => {
                self.walk_expr(callee, fn_name, env);
                for arg in args {
                    self.walk_expr(arg, fn_name, env);
                }
            }
            Expr::Field { receiver, .. } => {
                self.walk_expr(receiver, fn_name, env);
            }
            Expr::Index {
                receiver, index, ..
            } => {
                self.walk_expr(receiver, fn_name, env);
                self.walk_expr(index, fn_name, env);
            }
            Expr::If {
                condition,
                then_block,
                elsif_clauses,
                else_block,
                ..
            } => {
                self.walk_expr(condition, fn_name, env);
                let mut then_env = env.clone();
                self.walk_block(then_block, fn_name, &mut then_env);
                for (condition, block) in elsif_clauses {
                    self.walk_expr(condition, fn_name, env);
                    let mut elsif_env = env.clone();
                    self.walk_block(block, fn_name, &mut elsif_env);
                }
                if let Some(block) = else_block {
                    let mut else_env = env.clone();
                    self.walk_block(block, fn_name, &mut else_env);
                }
            }
            Expr::Match { subject, arms, .. } => {
                self.walk_expr(subject, fn_name, env);
                if let Some(domain) = self.domain_from_expr(subject, env) {
                    self.check_match_arms(fn_name, &domain, arms);
                }
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.walk_expr(guard, fn_name, env);
                    }
                    let mut arm_env = env.clone();
                    remove_pattern_bindings(&arm.pattern, &mut arm_env);
                    self.walk_block(&arm.body, fn_name, &mut arm_env);
                }
            }
            Expr::Try {
                body,
                rescues,
                ensure,
                ..
            } => {
                let mut body_env = env.clone();
                self.walk_block(body, fn_name, &mut body_env);
                for rescue in rescues {
                    let mut rescue_env = env.clone();
                    self.walk_block(&rescue.body, fn_name, &mut rescue_env);
                }
                if let Some(block) = ensure {
                    let mut ensure_env = env.clone();
                    self.walk_block(block, fn_name, &mut ensure_env);
                }
            }
            Expr::Closure { body, .. } => match body.as_ref() {
                ClosureBody::Block(block) => {
                    let mut closure_env = env.clone();
                    self.walk_block(block, fn_name, &mut closure_env);
                }
                ClosureBody::Expr(expr) => self.walk_expr(expr, fn_name, env),
            },
            Expr::Array { elements, .. } => {
                for element in elements {
                    self.walk_expr(element, fn_name, env);
                }
            }
            Expr::Map { entries, .. } => {
                for (key, value) in entries {
                    self.walk_expr(key, fn_name, env);
                    self.walk_expr(value, fn_name, env);
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

            if domain.is_fully_covered(&covered) {
                self.errors.push(CheckError::SafeModeViolation(format!(
                    "unreachable match arm in safe function '{fn_name}': finite domain is already fully covered before pattern {pattern}"
                )));
                continue;
            }

            if let Some(key) = coverage_key(domain, &arm.pattern) {
                if covered.contains(&key) {
                    self.errors.push(CheckError::SafeModeViolation(format!(
                        "unreachable match arm in safe function '{fn_name}': pattern {pattern} is already covered by prior arm"
                    )));
                }
            }

            if arm.guard.is_some() {
                continue;
            }

            if is_catch_all(&arm.pattern) {
                catch_all_seen = true;
            } else if let Some(key) = coverage_key(domain, &arm.pattern) {
                covered.insert(key);
            }
        }

        if !catch_all_seen && !domain.is_fully_covered(&covered) {
            let missing = domain
                .missing_patterns(&covered)
                .into_iter()
                .collect::<Vec<_>>()
                .join(", ");
            self.errors.push(CheckError::SafeModeViolation(format!(
                "non-exhaustive match in safe function '{fn_name}': missing {missing}"
            )));
        }
    }

    fn domain_from_expr(&self, expr: &Expr, env: &Env) -> Option<FiniteDomain> {
        match expr {
            Expr::Bool(_, _) => Some(FiniteDomain::Bool),
            Expr::Ident(name, _) => env.get(name).cloned(),
            Expr::Cast { ty, .. } => self.domain_from_type(ty),
            _ => None,
        }
    }

    fn domain_from_type(&self, ty: &TypeExpr) -> Option<FiniteDomain> {
        let TypeExpr::Named { path, .. } = ty else {
            return None;
        };
        if path.last().is_some_and(|name| name == "Bool") {
            return Some(FiniteDomain::Bool);
        }
        self.lookup_enum(path).map(FiniteDomain::Enum)
    }

    fn lookup_enum(&self, path: &[String]) -> Option<EnumInfo> {
        if let Some(info) = self.enums.get(path) {
            return Some(info.clone());
        }
        if path.len() == 1 {
            let name = &path[0];
            let mut matches = self
                .enums
                .values()
                .filter(|info| info.path.last() == Some(name));
            let first = matches.next()?.clone();
            if matches.next().is_none() {
                return Some(first);
            }
        }
        None
    }
}

impl FiniteDomain {
    fn is_fully_covered(&self, covered: &BTreeSet<String>) -> bool {
        match self {
            FiniteDomain::Bool => covered.contains("true") && covered.contains("false"),
            FiniteDomain::Enum(info) => info
                .variants
                .iter()
                .all(|variant| covered.contains(variant)),
        }
    }

    fn missing_patterns(&self, covered: &BTreeSet<String>) -> Vec<String> {
        match self {
            FiniteDomain::Bool => ["true", "false"]
                .into_iter()
                .filter(|value| !covered.contains(*value))
                .map(|value| format!("`{value}`"))
                .collect(),
            FiniteDomain::Enum(info) => info
                .variants
                .iter()
                .filter(|variant| !covered.contains(*variant))
                .map(|variant| format!("`{}::{variant}`", info.path.join("::")))
                .collect(),
        }
    }
}

fn coverage_key(domain: &FiniteDomain, pattern: &Pattern) -> Option<String> {
    match (domain, pattern) {
        (FiniteDomain::Bool, Pattern::Literal(Expr::Bool(value, _), _)) => Some(value.to_string()),
        (FiniteDomain::Enum(info), Pattern::Enum(path, _, _)) => {
            let variant = path.last()?;
            if !info.variants.contains(variant) {
                return None;
            }
            let enum_prefix = &path[..path.len().saturating_sub(1)];
            if enum_prefix.is_empty() || enum_prefix == info.path.as_slice() {
                Some(variant.clone())
            } else {
                None
            }
        }
        _ => None,
    }
}

fn is_catch_all(pattern: &Pattern) -> bool {
    matches!(
        pattern,
        Pattern::Ident(_, _) | Pattern::Wildcard(_) | Pattern::Rest(_)
    )
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
