//! Control-flow expression evaluators: if/elsif/else, match, try/rescue/ensure.

use crate::env::Env;
use crate::error::RuntimeError;
use crate::eval::eval_expr;
use crate::pattern::{match_pattern, BindResult};
use crate::stmt::exec_block_value;
use crate::value::Value;
use garnet_parser::ast::{Block, Expr, MatchArm, RescueClause};
use std::rc::Rc;

pub fn eval_if(
    condition: &Expr,
    then_block: &Block,
    elsif_clauses: &[(Expr, Block)],
    else_block: Option<&Block>,
    env: &Rc<Env>,
) -> Result<Value, RuntimeError> {
    if eval_expr(condition, env)?.truthy() {
        return exec_block_value(then_block, env);
    }
    for (cond, block) in elsif_clauses {
        if eval_expr(cond, env)?.truthy() {
            return exec_block_value(block, env);
        }
    }
    if let Some(block) = else_block {
        return exec_block_value(block, env);
    }
    Ok(Value::Nil)
}

pub fn eval_match(
    subject: &Expr,
    arms: &[MatchArm],
    env: &Rc<Env>,
) -> Result<Value, RuntimeError> {
    let subj = eval_expr(subject, env)?;
    for arm in arms {
        let match_env = Env::new_child(env);
        match match_pattern(&arm.pattern, &subj, &match_env)? {
            BindResult::Match => {
                // Check guard clause if present.
                if let Some(guard) = &arm.guard {
                    if !eval_expr(guard, &match_env)?.truthy() {
                        continue;
                    }
                }
                return eval_expr(&arm.body, &match_env);
            }
            BindResult::NoMatch => continue,
        }
    }
    Err(RuntimeError::NoMatch {
        value: subj.display(),
    })
}

pub fn eval_try(
    body: &Block,
    rescues: &[RescueClause],
    ensure: Option<&Block>,
    env: &Rc<Env>,
) -> Result<Value, RuntimeError> {
    let result = exec_block_value(body, env);
    let outcome = match result {
        Ok(v) => Ok(v),
        Err(RuntimeError::Raised(ex_value)) => try_rescue(ex_value, rescues, env),
        Err(RuntimeError::Message(msg)) => {
            // Message-style errors can also be caught via untyped rescue.
            try_rescue(Value::str(msg), rescues, env)
        }
        Err(RuntimeError::Type { expected, got }) => {
            try_rescue(
                Value::str(format!("type error: expected {expected}, got {got}")),
                rescues,
                env,
            )
        }
        Err(RuntimeError::DivByZero) => {
            try_rescue(Value::str("division by zero"), rescues, env)
        }
        Err(RuntimeError::IndexOOB { idx }) => {
            try_rescue(Value::str(format!("index out of bounds: {idx}")), rescues, env)
        }
        Err(other) => Err(other),
    };
    // Ensure always runs.
    if let Some(ensure_block) = ensure {
        exec_block_value(ensure_block, env)?;
    }
    outcome
}

fn try_rescue(
    exception: Value,
    rescues: &[RescueClause],
    env: &Rc<Env>,
) -> Result<Value, RuntimeError> {
    for clause in rescues {
        // If a type annotation is present, we only match on variant-name.
        let type_ok = match &clause.ty {
            None => true,
            Some(t) => type_matches_exception(t, &exception),
        };
        if !type_ok {
            continue;
        }
        let scope = Env::new_child(env);
        if let Some(name) = &clause.name {
            scope.define(name, exception.clone());
        }
        return exec_block_value(&clause.body, &scope);
    }
    // No matching rescue — re-raise.
    Err(RuntimeError::Raised(exception))
}

fn type_matches_exception(ty: &garnet_parser::ast::TypeExpr, ex: &Value) -> bool {
    use garnet_parser::ast::TypeExpr;
    match ty {
        TypeExpr::Named { path, .. } => {
            let target = path.last().cloned().unwrap_or_default();
            match ex {
                Value::Variant { variant, path: p, .. } => {
                    let ex_name = p.last().cloned().unwrap_or_default();
                    target == **variant || target == ex_name
                }
                Value::Struct { name, .. } => target == **name,
                _ => target.eq_ignore_ascii_case("string") && matches!(ex, Value::Str(_)),
            }
        }
        _ => true,
    }
}
