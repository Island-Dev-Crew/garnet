//! Garnet v0.3 statement and block parser (Mini-Spec §6).

use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TokenKind;

use super::{expr, types};

/// Parse a block: `{ stmt* [tail_expr] }`
pub fn parse_block(p: &mut Parser) -> Result<Block, ParseError> {
    let start = p.expect(&TokenKind::LBrace, "block")?.span;
    p.skip_separators();

    let mut stmts = Vec::new();
    let mut tail_expr = None;

    while !matches!(p.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
        let item = parse_stmt_or_expr(p)?;
        // Skip statement separators BEFORE deciding whether this is the tail
        // expression. Otherwise a trailing newline before `}` demotes the tail
        // value into a discarded Stmt::Expr, and the block returns Nil.
        p.skip_separators();
        let at_end = matches!(p.peek_kind(), TokenKind::RBrace);
        if at_end {
            match item {
                StmtOrExpr::Stmt(s) => stmts.push(s),
                StmtOrExpr::Expr(e) => tail_expr = Some(Box::new(e)),
            }
        } else {
            match item {
                StmtOrExpr::Stmt(s) => stmts.push(s),
                StmtOrExpr::Expr(e) => stmts.push(Stmt::Expr(e)),
            }
        }
    }

    p.expect(&TokenKind::RBrace, "block")?;
    let span = start.join(p.prev_span());
    Ok(Block {
        stmts,
        tail_expr,
        span,
    })
}

enum StmtOrExpr {
    Stmt(Stmt),
    Expr(Expr),
}

fn parse_stmt_or_expr(p: &mut Parser) -> Result<StmtOrExpr, ParseError> {
    match p.peek_kind() {
        TokenKind::KwLet => {
            let decl = parse_let_decl(p)?;
            Ok(StmtOrExpr::Stmt(Stmt::Let(decl)))
        }
        TokenKind::KwVar => {
            let decl = parse_var_decl(p)?;
            Ok(StmtOrExpr::Stmt(Stmt::Var(decl)))
        }
        TokenKind::KwConst => {
            let decl = parse_const_decl(p, false)?;
            Ok(StmtOrExpr::Stmt(Stmt::Const(decl)))
        }
        TokenKind::KwWhile => {
            let start = p.bump().span;
            let condition = expr::parse_expr(p)?;
            let body = parse_block(p)?;
            let span = start.join(body.span);
            Ok(StmtOrExpr::Stmt(Stmt::While {
                condition,
                body,
                span,
            }))
        }
        TokenKind::KwFor => {
            let start = p.bump().span;
            let (var, _) = p.expect_ident("for loop")?;
            p.expect(&TokenKind::KwIn, "for loop")?;
            let iter = expr::parse_expr(p)?;
            let body = parse_block(p)?;
            let span = start.join(body.span);
            Ok(StmtOrExpr::Stmt(Stmt::For {
                var,
                iter,
                body,
                span,
            }))
        }
        TokenKind::KwLoop => {
            let start = p.bump().span;
            let body = parse_block(p)?;
            let span = start.join(body.span);
            Ok(StmtOrExpr::Stmt(Stmt::Loop { body, span }))
        }
        TokenKind::KwBreak => {
            let start = p.bump().span;
            let value = if !matches!(
                p.peek_kind(),
                TokenKind::Newline | TokenKind::Semi | TokenKind::RBrace | TokenKind::Eof
            ) {
                Some(expr::parse_expr(p)?)
            } else {
                None
            };
            let span = if let Some(ref v) = value {
                start.join(v.span())
            } else {
                start
            };
            Ok(StmtOrExpr::Stmt(Stmt::Break { value, span }))
        }
        TokenKind::KwContinue => {
            let span = p.bump().span;
            Ok(StmtOrExpr::Stmt(Stmt::Continue { span }))
        }
        TokenKind::KwReturn => {
            let start = p.bump().span;
            let value = if !matches!(
                p.peek_kind(),
                TokenKind::Newline | TokenKind::Semi | TokenKind::RBrace | TokenKind::Eof
            ) {
                Some(expr::parse_expr(p)?)
            } else {
                None
            };
            let span = if let Some(ref v) = value {
                start.join(v.span())
            } else {
                start
            };
            Ok(StmtOrExpr::Stmt(Stmt::Return { value, span }))
        }
        TokenKind::KwRaise => {
            let start = p.bump().span;
            let value = expr::parse_expr(p)?;
            let span = start.join(value.span());
            Ok(StmtOrExpr::Stmt(Stmt::Raise { value, span }))
        }
        _ => {
            // Expression — then check for assignment
            let lhs = expr::parse_expr(p)?;
            let assign_op = match p.peek_kind() {
                TokenKind::Eq => Some(AssignOp::Eq),
                TokenKind::PlusEq => Some(AssignOp::PlusEq),
                TokenKind::MinusEq => Some(AssignOp::MinusEq),
                TokenKind::StarEq => Some(AssignOp::StarEq),
                TokenKind::SlashEq => Some(AssignOp::SlashEq),
                TokenKind::PercentEq => Some(AssignOp::PercentEq),
                _ => None,
            };
            if let Some(op) = assign_op {
                p.bump();
                let rhs = expr::parse_expr(p)?;
                let span = lhs.span().join(rhs.span());
                Ok(StmtOrExpr::Stmt(Stmt::Assign {
                    target: lhs,
                    op,
                    value: rhs,
                    span,
                }))
            } else {
                Ok(StmtOrExpr::Expr(lhs))
            }
        }
    }
}

/// Parse: `let [mut] name [: type] = expr`
pub fn parse_let_decl(p: &mut Parser) -> Result<LetDecl, ParseError> {
    let start = p.expect(&TokenKind::KwLet, "let declaration")?.span;
    let mutable = p.eat(&TokenKind::KwMut);
    let (name, _) = p.expect_ident("let declaration")?;
    let ty = if p.eat(&TokenKind::Colon) {
        Some(types::parse_type(p)?)
    } else {
        None
    };
    p.expect(&TokenKind::Eq, "let declaration")?;
    let value = expr::parse_expr(p)?;
    let span = start.join(value.span());
    Ok(LetDecl {
        mutable,
        name,
        ty,
        value,
        span,
    })
}

/// Parse: `var name [: type] = expr`
fn parse_var_decl(p: &mut Parser) -> Result<VarDecl, ParseError> {
    let start = p.expect(&TokenKind::KwVar, "var declaration")?.span;
    let (name, _) = p.expect_ident("var declaration")?;
    let ty = if p.eat(&TokenKind::Colon) {
        Some(types::parse_type(p)?)
    } else {
        None
    };
    p.expect(&TokenKind::Eq, "var declaration")?;
    let value = expr::parse_expr(p)?;
    let span = start.join(value.span());
    Ok(VarDecl {
        name,
        ty,
        value,
        span,
    })
}

/// Parse: `[pub] const name [: type] = expr`
pub fn parse_const_decl(p: &mut Parser, public: bool) -> Result<ConstDecl, ParseError> {
    let start = p.expect(&TokenKind::KwConst, "const declaration")?.span;
    let (name, _) = p.expect_ident("const declaration")?;
    let ty = if p.eat(&TokenKind::Colon) {
        Some(types::parse_type(p)?)
    } else {
        None
    };
    p.expect(&TokenKind::Eq, "const declaration")?;
    let value = expr::parse_expr(p)?;
    let span = start.join(value.span());
    Ok(ConstDecl {
        public,
        name,
        ty,
        value,
        span,
    })
}
