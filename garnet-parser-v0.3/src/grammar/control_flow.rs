//! Garnet v0.3 control flow expression parser (Mini-Spec §6.2, §6.3, §7).

use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TokenKind;

use super::{expr, patterns, stmts, types};

/// Parse: `if expr block { elsif expr block } [else block]`
pub fn parse_if_expr(p: &mut Parser) -> Result<Expr, ParseError> {
    let start = p.expect(&TokenKind::KwIf, "if expression")?.span;
    let condition = expr::parse_expr(p)?;
    let then_block = stmts::parse_block(p)?;

    let mut elsif_clauses = Vec::new();
    while p.eat(&TokenKind::KwElsif) {
        let cond = expr::parse_expr(p)?;
        let block = stmts::parse_block(p)?;
        elsif_clauses.push((cond, block));
    }

    let else_block = if p.eat(&TokenKind::KwElse) {
        Some(stmts::parse_block(p)?)
    } else {
        None
    };

    let end_span = else_block
        .as_ref()
        .map(|b| b.span)
        .or_else(|| elsif_clauses.last().map(|(_, b)| b.span))
        .unwrap_or(then_block.span);
    let span = start.join(end_span);

    Ok(Expr::If {
        condition: Box::new(condition),
        then_block,
        elsif_clauses,
        else_block,
        span,
    })
}

/// Parse: `match expr { arm, arm, ... }`
pub fn parse_match_expr(p: &mut Parser) -> Result<Expr, ParseError> {
    let start = p.expect(&TokenKind::KwMatch, "match expression")?.span;
    let subject = expr::parse_expr(p)?;
    p.expect(&TokenKind::LBrace, "match body")?;
    p.skip_separators();

    let mut arms = Vec::new();
    while !matches!(p.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
        let arm = parse_match_arm(p)?;
        arms.push(arm);
        // Arms separated by comma or newline
        p.eat(&TokenKind::Comma);
        p.skip_separators();
    }

    p.expect(&TokenKind::RBrace, "match body")?;
    let span = start.join(p.prev_span());
    Ok(Expr::Match {
        subject: Box::new(subject),
        arms,
        span,
    })
}

fn parse_match_arm(p: &mut Parser) -> Result<MatchArm, ParseError> {
    let pattern = patterns::parse_pattern(p)?;
    let guard = if p.eat(&TokenKind::KwIf) {
        Some(expr::parse_expr(p)?)
    } else {
        None
    };
    p.expect(&TokenKind::FatArrow, "match arm")?;
    let body = if matches!(p.peek_kind(), TokenKind::LBrace) {
        let block = stmts::parse_block(p)?;
        // Wrap block in an expression — use the tail_expr if present, otherwise Nil
        if let Some(tail) = block.tail_expr {
            *tail
        } else {
            Expr::Nil(block.span)
        }
    } else {
        expr::parse_expr(p)?
    };
    let span = pattern.span().join(body.span());
    Ok(MatchArm { pattern, guard, body, span })
}

/// Parse: `try block { rescue [name [: type]] block } [ensure block]`
pub fn parse_try_expr(p: &mut Parser) -> Result<Expr, ParseError> {
    let start = p.expect(&TokenKind::KwTry, "try expression")?.span;
    let body = stmts::parse_block(p)?;

    let mut rescues = Vec::new();
    while p.eat(&TokenKind::KwRescue) {
        let rescue_start = p.prev_span();
        let (name, ty) = if matches!(p.peek_kind(), TokenKind::Ident(_)) {
            let (name, _) = p.expect_ident("rescue clause")?;
            let ty = if p.eat(&TokenKind::Colon) {
                Some(types::parse_type(p)?)
            } else {
                None
            };
            (Some(name), ty)
        } else {
            (None, None)
        };
        let rescue_body = stmts::parse_block(p)?;
        let span = rescue_start.join(rescue_body.span);
        rescues.push(RescueClause { name, ty, body: rescue_body, span });
    }

    let ensure = if p.eat(&TokenKind::KwEnsure) {
        Some(stmts::parse_block(p)?)
    } else {
        None
    };

    let end_span = ensure
        .as_ref()
        .map(|b| b.span)
        .or_else(|| rescues.last().map(|r| r.span))
        .unwrap_or(body.span);
    let span = start.join(end_span);

    Ok(Expr::Try { body, rescues, ensure, span })
}
