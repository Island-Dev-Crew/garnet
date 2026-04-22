//! Statement execution — variable decls, loops, break/continue/return, assignments.

use crate::env::Env;
use crate::error::RuntimeError;
use crate::eval::eval_expr;
use crate::value::Value;
use garnet_parser::ast::{AssignOp, Block, Expr, Stmt};
use std::rc::Rc;

/// Execute a single statement. Control-flow signals (Break/Continue/Return)
/// bubble up as `Err` variants; the surrounding loop or function boundary
/// converts them back to values.
pub fn exec_stmt(stmt: &Stmt, env: &Rc<Env>) -> Result<(), RuntimeError> {
    match stmt {
        Stmt::Let(decl) => {
            let v = eval_expr(&decl.value, env)?;
            env.define(&decl.name, v);
            Ok(())
        }
        Stmt::Var(decl) => {
            let v = eval_expr(&decl.value, env)?;
            env.define(&decl.name, v);
            Ok(())
        }
        Stmt::Const(decl) => {
            let v = eval_expr(&decl.value, env)?;
            env.define(&decl.name, v);
            Ok(())
        }
        Stmt::Assign { target, op, value, .. } => {
            let new_val = eval_expr(value, env)?;
            let final_val = if *op == AssignOp::Eq {
                new_val
            } else {
                // Compound assignment: read-old, combine, write-new.
                let old = eval_expr(target, env)?;
                compound_apply(*op, old, new_val)?
            };
            assign_target(target, final_val, env)
        }
        Stmt::While { condition, body, .. } => {
            while eval_expr(condition, env)?.truthy() {
                match exec_block(body, env) {
                    Ok(()) => {}
                    Err(RuntimeError::Break(_)) => break,
                    Err(RuntimeError::Continue) => continue,
                    Err(other) => return Err(other),
                }
            }
            Ok(())
        }
        Stmt::For { var, iter, body, .. } => {
            let iterable = eval_expr(iter, env)?;
            let items = materialize_iter(&iterable)?;
            for item in items {
                let loop_env = Env::new_child(env);
                loop_env.define(var, item);
                match exec_block(body, &loop_env) {
                    Ok(()) => {}
                    Err(RuntimeError::Break(_)) => break,
                    Err(RuntimeError::Continue) => continue,
                    Err(other) => return Err(other),
                }
            }
            Ok(())
        }
        Stmt::Loop { body, .. } => loop {
            match exec_block(body, env) {
                Ok(()) => {}
                Err(RuntimeError::Break(_)) => return Ok(()),
                Err(RuntimeError::Continue) => continue,
                Err(other) => return Err(other),
            }
        },
        Stmt::Break { value, .. } => {
            let v = match value {
                Some(e) => Some(eval_expr(e, env)?),
                None => None,
            };
            Err(RuntimeError::Break(v))
        }
        Stmt::Continue { .. } => Err(RuntimeError::Continue),
        Stmt::Return { value, .. } => {
            let v = match value {
                Some(e) => eval_expr(e, env)?,
                None => Value::Nil,
            };
            Err(RuntimeError::Return(v))
        }
        Stmt::Raise { value, .. } => {
            let v = eval_expr(value, env)?;
            Err(RuntimeError::Raised(v))
        }
        Stmt::Expr(e) => {
            eval_expr(e, env)?;
            Ok(())
        }
    }
}

/// Execute a block in a fresh child scope. Returns the tail-expression value
/// if present, otherwise `Nil`. Called by if/match/loops.
pub fn exec_block_value(block: &Block, env: &Rc<Env>) -> Result<Value, RuntimeError> {
    let scope = Env::new_child(env);
    for s in &block.stmts {
        exec_stmt(s, &scope)?;
    }
    if let Some(tail) = &block.tail_expr {
        eval_expr(tail, &scope)
    } else {
        Ok(Value::Nil)
    }
}

/// Execute a block ignoring the tail value. Statements that fall through
/// produce `Ok(())`; control-flow signals propagate as errors.
pub fn exec_block(block: &Block, env: &Rc<Env>) -> Result<(), RuntimeError> {
    let scope = Env::new_child(env);
    for s in &block.stmts {
        exec_stmt(s, &scope)?;
    }
    if let Some(tail) = &block.tail_expr {
        eval_expr(tail, &scope)?;
    }
    Ok(())
}

fn compound_apply(op: AssignOp, old: Value, new: Value) -> Result<Value, RuntimeError> {
    use AssignOp::*;
    use Value::*;
    let bop = match op {
        Eq => unreachable!(),
        PlusEq => garnet_parser::ast::BinOp::Add,
        MinusEq => garnet_parser::ast::BinOp::Sub,
        StarEq => garnet_parser::ast::BinOp::Mul,
        SlashEq => garnet_parser::ast::BinOp::Div,
        PercentEq => garnet_parser::ast::BinOp::Mod,
    };
    // Reuse the numeric path from eval by constructing synthetic exprs would
    // be overkill. Do the arithmetic inline for primitive types.
    use garnet_parser::ast::BinOp as B;
    match (&old, &new, bop) {
        (Int(a), Int(b), B::Add) => Ok(Int(a + b)),
        (Int(a), Int(b), B::Sub) => Ok(Int(a - b)),
        (Int(a), Int(b), B::Mul) => Ok(Int(a * b)),
        (Int(a), Int(b), B::Div) if *b != 0 => Ok(Int(a / b)),
        (Int(_), Int(0), B::Div) => Err(RuntimeError::DivByZero),
        (Int(a), Int(b), B::Mod) if *b != 0 => Ok(Int(a % b)),
        (Float(a), Float(b), B::Add) => Ok(Float(a + b)),
        (Float(a), Float(b), B::Sub) => Ok(Float(a - b)),
        (Float(a), Float(b), B::Mul) => Ok(Float(a * b)),
        (Float(a), Float(b), B::Div) => Ok(Float(a / b)),
        (Str(a), Str(b), B::Add) => Ok(Value::str(format!("{a}{b}"))),
        _ => Err(RuntimeError::msg("compound assignment on unsupported types")),
    }
}

fn assign_target(target: &Expr, value: Value, env: &Rc<Env>) -> Result<(), RuntimeError> {
    match target {
        Expr::Ident(name, _) => {
            if env.set(name, value) {
                Ok(())
            } else {
                Err(RuntimeError::msg(format!("undefined variable: {name}")))
            }
        }
        Expr::Index { receiver, index, .. } => {
            let recv = eval_expr(receiver, env)?;
            let idx = eval_expr(index, env)?;
            match (&recv, &idx) {
                (Value::Array(arr), Value::Int(i)) => {
                    let mut a = arr.borrow_mut();
                    let n = a.len() as i64;
                    let real = if *i < 0 { n + i } else { *i };
                    if real < 0 || real >= n {
                        return Err(RuntimeError::IndexOOB { idx: *i });
                    }
                    a[real as usize] = value;
                    Ok(())
                }
                (Value::Map(m), Value::Str(s)) => {
                    m.borrow_mut().insert(s.to_string(), value);
                    Ok(())
                }
                (Value::Map(m), Value::Symbol(s)) => {
                    m.borrow_mut().insert(format!(":{s}"), value);
                    Ok(())
                }
                _ => Err(RuntimeError::msg("unsupported index-assignment target")),
            }
        }
        Expr::Field { receiver, field, .. } => {
            let recv = eval_expr(receiver, env)?;
            match recv {
                Value::Struct { fields, .. } => {
                    fields.borrow_mut().insert(field.clone(), value);
                    Ok(())
                }
                Value::Map(m) => {
                    m.borrow_mut().insert(field.clone(), value);
                    Ok(())
                }
                _ => Err(RuntimeError::msg("field assignment on non-struct/map value")),
            }
        }
        _ => Err(RuntimeError::msg("invalid assignment target")),
    }
}

fn materialize_iter(v: &Value) -> Result<Vec<Value>, RuntimeError> {
    match v {
        Value::Array(arr) => Ok(arr.borrow().clone()),
        Value::Range { start, end, inclusive } => {
            let stop = if *inclusive { *end + 1 } else { *end };
            Ok((*start..stop).map(Value::Int).collect())
        }
        Value::Str(s) => Ok(s.chars().map(|c| Value::str(c.to_string())).collect()),
        Value::Map(m) => Ok(m
            .borrow()
            .iter()
            .map(|(k, v)| Value::tuple(vec![Value::str(k.clone()), v.clone()]))
            .collect()),
        _ => Err(RuntimeError::type_err("iterable", v)),
    }
}
