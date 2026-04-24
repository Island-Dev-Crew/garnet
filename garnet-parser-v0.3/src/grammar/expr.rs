//! Garnet v0.3 expression parser — 11-level Pratt precedence tower.
//! Entry point: parse_expr -> parse_pipeline (lowest precedence).

use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::{StrPart, TokenKind};

use super::{control_flow, functions};

/// Parse an expression (entry point).
///
/// Enters one level of the depth budget for every recursive parse_expr
/// call. Since all nested expression parsing — parenthesized sub-exprs,
/// call args, array/map literals, match scrutinees, closure bodies —
/// ultimately routes through this entry point, this single guard covers
/// the parser's ParensBomb-class attack surface.
pub fn parse_expr(p: &mut Parser) -> Result<Expr, ParseError> {
    let _guard = p.enter_depth()?;
    parse_pipeline(p)
}

// ── Level 1: Pipeline |> ──
fn parse_pipeline(p: &mut Parser) -> Result<Expr, ParseError> {
    let mut lhs = parse_or(p)?;
    loop {
        // Allow the pipeline operator to begin a continuation line:
        //   items
        //     |> filter(...)
        //     |> map(...)
        // Peek past newline tokens; if the next significant token is `|>`,
        // commit by consuming the newlines and continuing.
        let mut look = 0;
        while matches!(p.peek_nth(look).kind, TokenKind::Newline) {
            look += 1;
        }
        if !matches!(p.peek_nth(look).kind, TokenKind::PipeGt) {
            break;
        }
        for _ in 0..look {
            p.bump();
        }
        p.bump(); // consume |>
        let rhs = parse_or(p)?;
        let span = lhs.span().join(rhs.span());
        lhs = Expr::Binary {
            op: BinOp::Pipeline,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            span,
        };
    }
    Ok(lhs)
}

// ── Level 2: Logical OR ──
fn parse_or(p: &mut Parser) -> Result<Expr, ParseError> {
    let mut lhs = parse_and(p)?;
    while p.eat(&TokenKind::KwOr) {
        let rhs = parse_and(p)?;
        let span = lhs.span().join(rhs.span());
        lhs = Expr::Binary {
            op: BinOp::Or,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            span,
        };
    }
    Ok(lhs)
}

// ── Level 3: Logical AND ──
fn parse_and(p: &mut Parser) -> Result<Expr, ParseError> {
    let mut lhs = parse_not(p)?;
    while p.eat(&TokenKind::KwAnd) {
        let rhs = parse_not(p)?;
        let span = lhs.span().join(rhs.span());
        lhs = Expr::Binary {
            op: BinOp::And,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            span,
        };
    }
    Ok(lhs)
}

// ── Level 4: Logical NOT (prefix) ──
fn parse_not(p: &mut Parser) -> Result<Expr, ParseError> {
    if p.eat(&TokenKind::KwNot) {
        let start = p.prev_span();
        let expr = parse_not(p)?; // right-recursive for chaining
        let span = start.join(expr.span());
        Ok(Expr::Unary {
            op: UnOp::Not,
            expr: Box::new(expr),
            span,
        })
    } else {
        parse_comparison(p)
    }
}

// ── Level 5: Comparison (non-associative) ──
fn parse_comparison(p: &mut Parser) -> Result<Expr, ParseError> {
    let lhs = parse_range(p)?;
    let op = match p.peek_kind() {
        TokenKind::EqEq => Some(BinOp::Eq),
        TokenKind::BangEq => Some(BinOp::NotEq),
        TokenKind::Lt => Some(BinOp::Lt),
        TokenKind::Gt => Some(BinOp::Gt),
        TokenKind::LtEq => Some(BinOp::LtEq),
        TokenKind::GtEq => Some(BinOp::GtEq),
        _ => None,
    };
    if let Some(op) = op {
        p.bump();
        let rhs = parse_range(p)?;
        let span = lhs.span().join(rhs.span());
        Ok(Expr::Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            span,
        })
    } else {
        Ok(lhs)
    }
}

// ── Level 6: Range (non-associative) ──
fn parse_range(p: &mut Parser) -> Result<Expr, ParseError> {
    let lhs = parse_addition(p)?;
    match p.peek_kind() {
        TokenKind::DotDot => {
            p.bump();
            let rhs = parse_addition(p)?;
            let span = lhs.span().join(rhs.span());
            Ok(Expr::Binary {
                op: BinOp::Range,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                span,
            })
        }
        TokenKind::DotDotDot => {
            p.bump();
            let rhs = parse_addition(p)?;
            let span = lhs.span().join(rhs.span());
            Ok(Expr::Binary {
                op: BinOp::RangeInclusive,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                span,
            })
        }
        _ => Ok(lhs),
    }
}

// ── Level 7: Addition / Subtraction ──
fn parse_addition(p: &mut Parser) -> Result<Expr, ParseError> {
    let mut lhs = parse_multiplication(p)?;
    loop {
        let op = match p.peek_kind() {
            TokenKind::Plus => Some(BinOp::Add),
            TokenKind::Minus => Some(BinOp::Sub),
            _ => None,
        };
        if let Some(op) = op {
            p.bump();
            let rhs = parse_multiplication(p)?;
            let span = lhs.span().join(rhs.span());
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                span,
            };
        } else {
            break;
        }
    }
    Ok(lhs)
}

// ── Level 8: Multiplication / Division / Modulo ──
fn parse_multiplication(p: &mut Parser) -> Result<Expr, ParseError> {
    let mut lhs = parse_unary(p)?;
    loop {
        let op = match p.peek_kind() {
            TokenKind::Star => Some(BinOp::Mul),
            TokenKind::Slash => Some(BinOp::Div),
            TokenKind::Percent => Some(BinOp::Mod),
            _ => None,
        };
        if let Some(op) = op {
            p.bump();
            let rhs = parse_unary(p)?;
            let span = lhs.span().join(rhs.span());
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                span,
            };
        } else {
            break;
        }
    }
    Ok(lhs)
}

// ── Level 9: Unary (prefix) ──
fn parse_unary(p: &mut Parser) -> Result<Expr, ParseError> {
    match p.peek_kind() {
        TokenKind::Minus => {
            let start = p.bump().span;
            let expr = parse_unary(p)?;
            let span = start.join(expr.span());
            Ok(Expr::Unary {
                op: UnOp::Neg,
                expr: Box::new(expr),
                span,
            })
        }
        TokenKind::Bang => {
            let start = p.bump().span;
            let expr = parse_unary(p)?;
            let span = start.join(expr.span());
            Ok(Expr::Unary {
                op: UnOp::Not,
                expr: Box::new(expr),
                span,
            })
        }
        _ => parse_postfix(p),
    }
}

// ── Level 10: Postfix (.field, .method(), call(), [index], ::path, ?) ──
fn parse_postfix(p: &mut Parser) -> Result<Expr, ParseError> {
    let mut expr = parse_primary(p)?;
    loop {
        match p.peek_kind() {
            TokenKind::Dot => {
                p.bump();
                let (name, name_span) = p.expect_ident("field or method name")?;
                if p.eat(&TokenKind::LParen) {
                    let args = parse_arg_list(p)?;
                    p.expect(&TokenKind::RParen, "method call")?;
                    let span = expr.span().join(p.prev_span());
                    expr = Expr::Method {
                        receiver: Box::new(expr),
                        method: name,
                        args,
                        span,
                    };
                } else {
                    let span = expr.span().join(name_span);
                    expr = Expr::Field {
                        receiver: Box::new(expr),
                        field: name,
                        span,
                    };
                }
            }
            TokenKind::LParen => {
                p.bump();
                let args = parse_arg_list(p)?;
                p.expect(&TokenKind::RParen, "function call")?;
                let span = expr.span().join(p.prev_span());
                expr = Expr::Call {
                    callee: Box::new(expr),
                    args,
                    span,
                };
            }
            TokenKind::LBracket => {
                p.bump();
                let index = parse_expr(p)?;
                p.expect(&TokenKind::RBracket, "index expression")?;
                let span = expr.span().join(p.prev_span());
                expr = Expr::Index {
                    receiver: Box::new(expr),
                    index: Box::new(index),
                    span,
                };
            }
            TokenKind::ColonCol => {
                p.bump();
                let (seg, seg_span) = p.expect_ident("path segment")?;
                // Extend path or create one
                match expr {
                    Expr::Ident(ref name, ref span) => {
                        let new_span = span.join(seg_span);
                        expr = Expr::Path(vec![name.clone(), seg], new_span);
                    }
                    Expr::Path(ref mut segs, ref mut span) => {
                        segs.push(seg);
                        *span = span.join(seg_span);
                    }
                    _ => {
                        let span = expr.span().join(seg_span);
                        expr = Expr::Field {
                            receiver: Box::new(expr),
                            field: seg,
                            span,
                        };
                    }
                }
            }
            TokenKind::Question => {
                let q_span = p.bump().span;
                let span = expr.span().join(q_span);
                expr = Expr::Unary {
                    op: UnOp::Question,
                    expr: Box::new(expr),
                    span,
                };
            }
            _ => break,
        }
    }
    Ok(expr)
}

// ── Level 11: Primary ──
fn parse_primary(p: &mut Parser) -> Result<Expr, ParseError> {
    let tok = p.peek().clone();
    match &tok.kind {
        TokenKind::Int(v) => {
            let v = *v;
            p.bump();
            Ok(Expr::Int(v, tok.span))
        }
        TokenKind::Float(v) => {
            let v = *v;
            p.bump();
            Ok(Expr::Float(v, tok.span))
        }
        TokenKind::KwTrue => {
            p.bump();
            Ok(Expr::Bool(true, tok.span))
        }
        TokenKind::KwFalse => {
            p.bump();
            Ok(Expr::Bool(false, tok.span))
        }
        TokenKind::KwNil => {
            p.bump();
            Ok(Expr::Nil(tok.span))
        }
        TokenKind::Str(parts) => {
            let parts = parts.clone();
            p.bump();
            Ok(Expr::Str(
                StringLit {
                    parts: parts
                        .into_iter()
                        .map(|sp| match sp {
                            crate::token::StrPart::Lit(s) => StrPart::Lit(s),
                            crate::token::StrPart::Interp(s) => StrPart::Interp(s),
                        })
                        .collect(),
                },
                tok.span,
            ))
        }
        TokenKind::RawStr(s) => {
            let s = s.clone();
            p.bump();
            Ok(Expr::Str(
                StringLit {
                    parts: vec![StrPart::Lit(s)],
                },
                tok.span,
            ))
        }
        TokenKind::Symbol(name) => {
            let name = name.clone();
            p.bump();
            Ok(Expr::Symbol(name, tok.span))
        }
        TokenKind::Ident(_) => {
            let (name, span) = p.expect_ident("expression")?;
            Ok(Expr::Ident(name, span))
        }
        TokenKind::LParen => {
            if let Some(expr) = parse_directly_nested_atom(p)? {
                return Ok(expr);
            }
            p.bump();
            let expr = parse_expr(p)?;
            p.expect(&TokenKind::RParen, "parenthesized expression")?;
            Ok(expr)
        }
        TokenKind::LBracket => {
            let start = p.bump().span;
            let elements = if matches!(p.peek_kind(), TokenKind::RBracket) {
                Vec::new()
            } else {
                parse_arg_list(p)?
            };
            p.expect(&TokenKind::RBracket, "array literal")?;
            let span = start.join(p.prev_span());
            Ok(Expr::Array { elements, span })
        }
        TokenKind::LBrace => parse_map_literal(p),
        TokenKind::KwIf => control_flow::parse_if_expr(p),
        TokenKind::KwMatch => control_flow::parse_match_expr(p),
        TokenKind::KwTry => control_flow::parse_try_expr(p),
        TokenKind::KwSpawn => {
            let start = p.bump().span;
            let expr = parse_expr(p)?;
            let span = start.join(expr.span());
            Ok(Expr::Spawn {
                expr: Box::new(expr),
                span,
            })
        }
        TokenKind::Pipe => functions::parse_closure(p),
        _ => Err(ParseError::unexpected_token(
            "expression",
            &format!("{:?}", tok.kind),
            tok.span,
        )),
    }
}

fn parse_directly_nested_atom(p: &mut Parser) -> Result<Option<Expr>, ParseError> {
    let mut open_count = 0;
    while matches!(p.peek_nth(open_count).kind, TokenKind::LParen) {
        open_count += 1;
    }
    if open_count < 2 {
        return Ok(None);
    }

    let atom = p.peek_nth(open_count).clone();
    if !is_collapsible_atom(&atom.kind) {
        return Ok(None);
    }
    for i in 0..open_count {
        if !matches!(p.peek_nth(open_count + 1 + i).kind, TokenKind::RParen) {
            return Ok(None);
        }
    }

    let mut depth_guards = Vec::with_capacity(open_count);
    for _ in 0..open_count {
        depth_guards.push(p.enter_depth()?);
        p.bump();
    }
    p.bump();
    for _ in 0..open_count {
        p.expect(&TokenKind::RParen, "parenthesized expression")?;
    }

    Ok(Some(expr_from_collapsible_atom(atom)))
}

fn is_collapsible_atom(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Int(_)
            | TokenKind::Float(_)
            | TokenKind::KwTrue
            | TokenKind::KwFalse
            | TokenKind::KwNil
            | TokenKind::Str(_)
            | TokenKind::RawStr(_)
            | TokenKind::Symbol(_)
            | TokenKind::Ident(_)
    )
}

fn expr_from_collapsible_atom(tok: crate::token::Token) -> Expr {
    match tok.kind {
        TokenKind::Int(v) => Expr::Int(v, tok.span),
        TokenKind::Float(v) => Expr::Float(v, tok.span),
        TokenKind::KwTrue => Expr::Bool(true, tok.span),
        TokenKind::KwFalse => Expr::Bool(false, tok.span),
        TokenKind::KwNil => Expr::Nil(tok.span),
        TokenKind::Str(parts) => Expr::Str(
            StringLit {
                parts: parts
                    .into_iter()
                    .map(|sp| match sp {
                        crate::token::StrPart::Lit(s) => StrPart::Lit(s),
                        crate::token::StrPart::Interp(s) => StrPart::Interp(s),
                    })
                    .collect(),
            },
            tok.span,
        ),
        TokenKind::RawStr(s) => Expr::Str(
            StringLit {
                parts: vec![StrPart::Lit(s)],
            },
            tok.span,
        ),
        TokenKind::Symbol(name) => Expr::Symbol(name, tok.span),
        TokenKind::Ident(name) => Expr::Ident(name, tok.span),
        _ => unreachable!("caller checked collapsible atom"),
    }
}

/// Parse a map literal: `{ k => v, k2 => v2, ... }` or empty `{}`. Tolerates
/// trailing comma + newlines around the commas so multi-line maps parse.
fn parse_map_literal(p: &mut Parser) -> Result<Expr, ParseError> {
    let start = p.expect(&TokenKind::LBrace, "map literal")?.span;
    p.skip_separators();
    let mut entries = Vec::new();
    if !matches!(p.peek_kind(), TokenKind::RBrace) {
        loop {
            let key = parse_expr(p)?;
            p.expect(&TokenKind::FatArrow, "map literal entry separator '=>'")?;
            let value = parse_expr(p)?;
            entries.push((key, value));
            p.skip_separators();
            if !p.eat(&TokenKind::Comma) {
                break;
            }
            p.skip_separators();
            // Tolerate trailing comma before the closing brace.
            if matches!(p.peek_kind(), TokenKind::RBrace) {
                break;
            }
        }
    }
    p.skip_separators();
    p.expect(&TokenKind::RBrace, "map literal")?;
    let span = start.join(p.prev_span());
    Ok(Expr::Map { entries, span })
}

/// Parse a comma-separated list of expressions. Tolerates newlines around
/// the commas and at either end, plus trailing commas, so that multi-line
/// array and call argument lists parse cleanly.
pub fn parse_arg_list(p: &mut Parser) -> Result<Vec<Expr>, ParseError> {
    let mut args = Vec::new();
    p.skip_separators();
    if matches!(
        p.peek_kind(),
        TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace
    ) {
        return Ok(args);
    }
    loop {
        args.push(parse_expr(p)?);
        p.skip_separators();
        if !p.eat(&TokenKind::Comma) {
            break;
        }
        p.skip_separators();
        // Allow trailing comma before the closing delimiter.
        if matches!(
            p.peek_kind(),
            TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace
        ) {
            break;
        }
    }
    Ok(args)
}
