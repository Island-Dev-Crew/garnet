//! Pattern-matching runtime: binds identifiers from a pattern against a Value.

use crate::env::Env;
use crate::error::RuntimeError;
use crate::eval::eval_expr;
use crate::value::Value;
use garnet_parser::ast::{Expr, Pattern};
use std::rc::Rc;

pub enum BindResult {
    Match,
    NoMatch,
}

/// Try to match `pattern` against `value`, binding any identifier patterns
/// into `env`. Returns `Match` / `NoMatch`; runtime errors (e.g. evaluating a
/// literal pattern that turns out non-constant) bubble up.
pub fn match_pattern(
    pattern: &Pattern,
    value: &Value,
    env: &Rc<Env>,
) -> Result<BindResult, RuntimeError> {
    match pattern {
        Pattern::Wildcard(_) => Ok(BindResult::Match),
        Pattern::Rest(_) => Ok(BindResult::Match),
        Pattern::Literal(expr, _) => {
            let lit_val = eval_expr(expr, env)?;
            Ok(if lit_val.eq_deep(value) {
                BindResult::Match
            } else {
                BindResult::NoMatch
            })
        }
        Pattern::Ident(name, _) => {
            env.define(name, value.clone());
            Ok(BindResult::Match)
        }
        Pattern::Tuple(items, _) => match value {
            Value::Tuple(tup) => {
                if tup.len() != items.len() {
                    return Ok(BindResult::NoMatch);
                }
                for (p, v) in items.iter().zip(tup.iter()) {
                    match match_pattern(p, v, env)? {
                        BindResult::Match => {}
                        BindResult::NoMatch => return Ok(BindResult::NoMatch),
                    }
                }
                Ok(BindResult::Match)
            }
            _ => Ok(BindResult::NoMatch),
        },
        Pattern::Enum(path, sub_patterns, _) => {
            let pat_variant = path.last().cloned().unwrap_or_default();
            match value {
                Value::Variant {
                    variant, fields, ..
                } => {
                    if pat_variant != **variant {
                        return Ok(BindResult::NoMatch);
                    }
                    if sub_patterns.len() != fields.len() {
                        return Ok(BindResult::NoMatch);
                    }
                    for (p, v) in sub_patterns.iter().zip(fields.iter()) {
                        match match_pattern(p, v, env)? {
                            BindResult::Match => {}
                            BindResult::NoMatch => return Ok(BindResult::NoMatch),
                        }
                    }
                    Ok(BindResult::Match)
                }
                _ => {
                    // Allow matching a bare Symbol like :not_found against
                    // enum patterns that happen to carry no payload.
                    if sub_patterns.is_empty() {
                        if let Value::Symbol(s) = value {
                            if s.as_str() == pat_variant {
                                return Ok(BindResult::Match);
                            }
                        }
                    }
                    Ok(BindResult::NoMatch)
                }
            }
        }
    }
}

/// Utility: detect if an expression is a literal pattern candidate.
pub fn is_literal_expr(e: &Expr) -> bool {
    matches!(
        e,
        Expr::Int(_, _)
            | Expr::Float(_, _)
            | Expr::Bool(_, _)
            | Expr::Nil(_)
            | Expr::Str(_, _)
            | Expr::Symbol(_, _)
    )
}
