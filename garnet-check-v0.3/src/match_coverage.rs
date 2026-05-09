//! Conservative safe-mode match coverage for finite domains.
//!
//! This is intentionally a narrow semantic slice: it proves exhaustiveness and
//! simple reachability for `Bool`, same-module enum subjects, imported enum
//! aliases, and finite nested constructor payloads whose type is visible from
//! parameter or local annotation metadata. It does not attempt full type
//! inference, recursive/open payload coverage, or guard reasoning.

use crate::CheckError;
use garnet_parser::ast::{
    Block, ClosureBody, Expr, FnDef, FnMode, Item, Module, Pattern, Stmt, StringLit, TypeExpr,
    UseImports,
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
}

#[derive(Debug, Default)]
struct Checker {
    enums: BTreeMap<Vec<String>, EnumInfo>,
    scopes: BTreeMap<Vec<String>, Scope>,
    errors: Vec<CheckError>,
}

type Env = BTreeMap<String, FiniteDomain>;

pub fn check_match_coverage(module: &Module) -> Vec<CheckError> {
    let mut checker = Checker::default();
    checker.collect_enums(&module.items, &[]);
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
                    }
                }
                UseImports::Named(names) => {
                    for name in names {
                        let mut path = base_path.clone();
                        path.push(name.clone());
                        if self.enums.contains_key(&path) {
                            scope.enum_imports.insert(name.clone(), path);
                        }
                    }
                }
                UseImports::Glob => {
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

        self.walk_block(&f.body, &f.name, env, scope);
    }

    fn walk_block(&mut self, block: &Block, fn_name: &str, mut env: Env, scope: &Scope) -> Env {
        for stmt in &block.stmts {
            self.walk_stmt(stmt, fn_name, &mut env, scope);
        }
        if let Some(tail) = &block.tail_expr {
            self.walk_expr(tail, fn_name, &mut env, scope);
        }
        env
    }

    fn walk_stmt(&mut self, stmt: &Stmt, fn_name: &str, env: &mut Env, scope: &Scope) {
        match stmt {
            Stmt::Let(decl) => {
                self.walk_expr(&decl.value, fn_name, env, scope);
                if let Some(ty) = &decl.ty {
                    if let Some(domain) = self.domain_from_type(ty, scope) {
                        env.insert(decl.name.clone(), domain);
                    }
                }
            }
            Stmt::Var(decl) => {
                self.walk_expr(&decl.value, fn_name, env, scope);
                if let Some(ty) = &decl.ty {
                    if let Some(domain) = self.domain_from_type(ty, scope) {
                        env.insert(decl.name.clone(), domain);
                    }
                }
            }
            Stmt::Const(decl) => {
                self.walk_expr(&decl.value, fn_name, env, scope);
            }
            Stmt::Assign { target, value, .. } => {
                self.walk_expr(target, fn_name, env, scope);
                self.walk_expr(value, fn_name, env, scope);
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.walk_expr(condition, fn_name, env, scope);
                self.walk_block(body, fn_name, env.clone(), scope);
            }
            Stmt::For { iter, body, .. } => {
                self.walk_expr(iter, fn_name, env, scope);
                self.walk_block(body, fn_name, env.clone(), scope);
            }
            Stmt::Loop { body, .. } => {
                self.walk_block(body, fn_name, env.clone(), scope);
            }
            Stmt::Return { value, .. }
            | Stmt::Yield { value, .. }
            | Stmt::Next { value, .. }
            | Stmt::Break { value, .. } => {
                if let Some(value) = value {
                    self.walk_expr(value, fn_name, env, scope);
                }
            }
            Stmt::Raise { value, .. } | Stmt::Expr(value) => {
                self.walk_expr(value, fn_name, env, scope);
            }
            Stmt::Continue { .. } => {}
        }
    }

    fn walk_expr(&mut self, expr: &Expr, fn_name: &str, env: &mut Env, scope: &Scope) {
        match expr {
            Expr::Binary { lhs, rhs, .. } => {
                self.walk_expr(lhs, fn_name, env, scope);
                self.walk_expr(rhs, fn_name, env, scope);
            }
            Expr::Unary { expr, .. } | Expr::Cast { expr, .. } | Expr::Spawn { expr, .. } => {
                self.walk_expr(expr, fn_name, env, scope);
            }
            Expr::Call { callee, args, .. }
            | Expr::Method {
                receiver: callee,
                args,
                ..
            } => {
                self.walk_expr(callee, fn_name, env, scope);
                for arg in args {
                    self.walk_expr(arg, fn_name, env, scope);
                }
            }
            Expr::Field { receiver, .. } => {
                self.walk_expr(receiver, fn_name, env, scope);
            }
            Expr::Index {
                receiver, index, ..
            } => {
                self.walk_expr(receiver, fn_name, env, scope);
                self.walk_expr(index, fn_name, env, scope);
            }
            Expr::If {
                condition,
                then_block,
                elsif_clauses,
                else_block,
                ..
            } => {
                self.walk_expr(condition, fn_name, env, scope);
                self.walk_block(then_block, fn_name, env.clone(), scope);
                for (condition, block) in elsif_clauses {
                    self.walk_expr(condition, fn_name, env, scope);
                    self.walk_block(block, fn_name, env.clone(), scope);
                }
                if let Some(block) = else_block {
                    self.walk_block(block, fn_name, env.clone(), scope);
                }
            }
            Expr::Match { subject, arms, .. } => {
                self.walk_expr(subject, fn_name, env, scope);
                if let Some(domain) = self.domain_from_expr(subject, env, scope) {
                    self.check_match_arms(fn_name, &domain, arms);
                }
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.walk_expr(guard, fn_name, env, scope);
                    }
                    let mut arm_env = env.clone();
                    remove_pattern_bindings(&arm.pattern, &mut arm_env);
                    self.walk_block(&arm.body, fn_name, arm_env, scope);
                }
            }
            Expr::Try {
                body,
                rescues,
                ensure,
                ..
            } => {
                self.walk_block(body, fn_name, env.clone(), scope);
                for rescue in rescues {
                    self.walk_block(&rescue.body, fn_name, env.clone(), scope);
                }
                if let Some(block) = ensure {
                    self.walk_block(block, fn_name, env.clone(), scope);
                }
            }
            Expr::Closure { body, .. } => match body.as_ref() {
                ClosureBody::Block(block) => {
                    self.walk_block(block, fn_name, env.clone(), scope);
                }
                ClosureBody::Expr(expr) => self.walk_expr(expr, fn_name, env, scope),
            },
            Expr::Array { elements, .. } => {
                for element in elements {
                    self.walk_expr(element, fn_name, env, scope);
                }
            }
            Expr::Map { entries, .. } => {
                for (key, value) in entries {
                    self.walk_expr(key, fn_name, env, scope);
                    self.walk_expr(value, fn_name, env, scope);
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

            if arm.guard.is_some() {
                continue;
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
            Expr::Cast { ty, .. } => self.domain_from_type(ty, scope),
            _ => None,
        }
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
        if self.enum_path_or_prefix_exists(path) {
            candidates.insert(path.to_vec());
        }
        if !module_path.is_empty() {
            let mut relative = module_path.to_vec();
            relative.extend_from_slice(path);
            if self.enum_path_or_prefix_exists(&relative) {
                candidates.insert(relative);
            }
        }

        match candidates.len() {
            0 => Some(path.to_vec()),
            1 => candidates.into_iter().next(),
            _ => None,
        }
    }

    fn enum_path_or_prefix_exists(&self, path: &[String]) -> bool {
        self.enums
            .keys()
            .any(|enum_path| enum_path.as_slice().starts_with(path))
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
