//! Handler-block interior — minimal expression-statement grammar.
//!
//! **v0.2 underspec note.** Mini-Spec v0.2 §4.1 defines `handler-decl :=
//! "on" ident "(" param-list ")" block` but never defines `block`. The
//! parser cannot consume handler bodies without *some* interpretation, so
//! this module provides the smallest one that lets useful programs parse:
//!
//! ```text
//! block := "{" stmt* "}"
//! stmt  := "let" ident "=" expr
//!        | "return" expr?
//!        | expr
//! ```
//!
//! Statements are separated by newlines or `;`. The expression grammar is
//! Pratt-style: equality < comparison < add < mul < unary < postfix < primary.
//! Postfix supports `.field`, `.method(args)`, and `expr(args)` direct calls.
//! Primary covers literals, identifiers, `Path::segments`, parenthesized
//! expressions, and string literals (including `#{...}` interpolation, which
//! re-lexes the inner source through this same grammar).
//!
//! When v0.3 specifies `block`, this module is the file to revise.

use crate::ast::{BinOp, Block, Expr, Stmt, StringLit, StringPart, UnOp};
use crate::error::ParseError;
use crate::lexer::Lexer;
use crate::parser::{describe_kind, Parser};
use crate::token::{StrPart, TokenKind};

pub fn parse_block(p: &mut Parser) -> Result<Block, ParseError> {
    let open = p.expect(&TokenKind::LBrace, "`{` to open block")?;
    let mut stmts = Vec::new();
    p.skip_separators();
    while !matches!(p.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
        let s = parse_stmt(p)?;
        stmts.push(s);
        p.skip_separators();
    }
    let close = p.expect(&TokenKind::RBrace, "`}` to close block")?;
    Ok(Block {
        stmts,
        span: open.span.join(close.span),
    })
}

fn parse_stmt(p: &mut Parser) -> Result<Stmt, ParseError> {
    match p.peek_kind() {
        TokenKind::KwLet => parse_let(p),
        TokenKind::KwReturn => parse_return(p),
        _ => {
            let e = parse_expr(p)?;
            Ok(Stmt::Expr(e))
        }
    }
}

fn parse_let(p: &mut Parser) -> Result<Stmt, ParseError> {
    let let_tok = p.expect(&TokenKind::KwLet, "`let`")?;
    let (name, _) = p.expect_ident("variable name")?;
    p.expect(&TokenKind::Eq, "`=` in let-binding")?;
    let value = parse_expr(p)?;
    let span = let_tok.span.join(value.span());
    Ok(Stmt::Let { name, value, span })
}

fn parse_return(p: &mut Parser) -> Result<Stmt, ParseError> {
    let ret_tok = p.expect(&TokenKind::KwReturn, "`return`")?;
    if matches!(
        p.peek_kind(),
        TokenKind::Newline | TokenKind::Semi | TokenKind::RBrace | TokenKind::Eof
    ) {
        return Ok(Stmt::Return {
            value: None,
            span: ret_tok.span,
        });
    }
    let e = parse_expr(p)?;
    let span = ret_tok.span.join(e.span());
    Ok(Stmt::Return {
        value: Some(e),
        span,
    })
}

pub fn parse_expr(p: &mut Parser) -> Result<Expr, ParseError> {
    parse_equality(p)
}

fn parse_equality(p: &mut Parser) -> Result<Expr, ParseError> {
    let mut lhs = parse_comparison(p)?;
    loop {
        let op = match p.peek_kind() {
            TokenKind::EqEq => BinOp::Eq,
            TokenKind::BangEq => BinOp::NotEq,
            _ => break,
        };
        p.bump();
        let rhs = parse_comparison(p)?;
        let span = lhs.span().join(rhs.span());
        lhs = Expr::Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            span,
        };
    }
    Ok(lhs)
}

fn parse_comparison(p: &mut Parser) -> Result<Expr, ParseError> {
    let mut lhs = parse_add(p)?;
    loop {
        let op = match p.peek_kind() {
            TokenKind::Lt => BinOp::Lt,
            TokenKind::Gt => BinOp::Gt,
            TokenKind::LtEq => BinOp::LtEq,
            TokenKind::GtEq => BinOp::GtEq,
            _ => break,
        };
        p.bump();
        let rhs = parse_add(p)?;
        let span = lhs.span().join(rhs.span());
        lhs = Expr::Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            span,
        };
    }
    Ok(lhs)
}

fn parse_add(p: &mut Parser) -> Result<Expr, ParseError> {
    let mut lhs = parse_mul(p)?;
    loop {
        let op = match p.peek_kind() {
            TokenKind::Plus => BinOp::Add,
            TokenKind::Minus => BinOp::Sub,
            _ => break,
        };
        p.bump();
        let rhs = parse_mul(p)?;
        let span = lhs.span().join(rhs.span());
        lhs = Expr::Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            span,
        };
    }
    Ok(lhs)
}

fn parse_mul(p: &mut Parser) -> Result<Expr, ParseError> {
    let mut lhs = parse_unary(p)?;
    loop {
        let op = match p.peek_kind() {
            TokenKind::Star => BinOp::Mul,
            TokenKind::Slash => BinOp::Div,
            _ => break,
        };
        p.bump();
        let rhs = parse_unary(p)?;
        let span = lhs.span().join(rhs.span());
        lhs = Expr::Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            span,
        };
    }
    Ok(lhs)
}

fn parse_unary(p: &mut Parser) -> Result<Expr, ParseError> {
    if matches!(p.peek_kind(), TokenKind::Minus) {
        let neg = p.bump();
        let e = parse_unary(p)?;
        let span = neg.span.join(e.span());
        return Ok(Expr::Unary {
            op: UnOp::Neg,
            expr: Box::new(e),
            span,
        });
    }
    parse_postfix(p)
}

fn parse_postfix(p: &mut Parser) -> Result<Expr, ParseError> {
    let mut e = parse_primary(p)?;
    loop {
        match p.peek_kind() {
            TokenKind::Dot => {
                p.bump();
                let (field, fspan) = p.expect_ident("field or method name")?;
                if matches!(p.peek_kind(), TokenKind::LParen) {
                    let args = parse_arg_list(p)?;
                    let last_span = p.tokens[p.pos.saturating_sub(1)].span;
                    let span = e.span().join(last_span);
                    e = Expr::Method {
                        recv: Box::new(e),
                        method: field,
                        args,
                        span,
                    };
                } else {
                    let span = e.span().join(fspan);
                    e = Expr::Field {
                        recv: Box::new(e),
                        field,
                        span,
                    };
                }
            }
            TokenKind::LParen => {
                let args = parse_arg_list(p)?;
                let last_span = p.tokens[p.pos.saturating_sub(1)].span;
                let span = e.span().join(last_span);
                e = Expr::Call {
                    callee: Box::new(e),
                    args,
                    span,
                };
            }
            _ => break,
        }
    }
    Ok(e)
}

fn parse_arg_list(p: &mut Parser) -> Result<Vec<Expr>, ParseError> {
    p.expect(&TokenKind::LParen, "`(`")?;
    let mut args = Vec::new();
    if matches!(p.peek_kind(), TokenKind::RParen) {
        p.bump();
        return Ok(args);
    }
    loop {
        let arg = parse_expr(p)?;
        args.push(arg);
        if !p.eat(&TokenKind::Comma) {
            break;
        }
    }
    p.expect(&TokenKind::RParen, "`)`")?;
    Ok(args)
}

fn parse_primary(p: &mut Parser) -> Result<Expr, ParseError> {
    let tok = p.peek().clone();
    match tok.kind.clone() {
        TokenKind::Int(n) => {
            p.bump();
            Ok(Expr::Int(n, tok.span))
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
            p.bump();
            let mut out = Vec::new();
            for part in parts {
                match part {
                    StrPart::Lit(s) => out.push(StringPart::Lit(s)),
                    StrPart::Interp(src) => {
                        let inner_tokens = Lexer::new(&src).lex()?;
                        let mut inner = Parser::new(inner_tokens);
                        let e = parse_expr(&mut inner)?;
                        out.push(StringPart::Interp(e));
                    }
                }
            }
            Ok(Expr::Str(StringLit { parts: out }, tok.span))
        }
        TokenKind::Ident(name) => {
            p.bump();
            if matches!(p.peek_kind(), TokenKind::ColonCol) {
                let mut segments = vec![name];
                let mut span = tok.span;
                while matches!(p.peek_kind(), TokenKind::ColonCol) {
                    p.bump();
                    let (next, nspan) = p.expect_ident("path segment")?;
                    segments.push(next);
                    span = span.join(nspan);
                }
                Ok(Expr::Path(segments, span))
            } else {
                Ok(Expr::Ident(name, tok.span))
            }
        }
        TokenKind::LParen => {
            p.bump();
            let e = parse_expr(p)?;
            p.expect(&TokenKind::RParen, "`)`")?;
            Ok(e)
        }
        other => Err(ParseError::unexpected_token(
            "expression",
            describe_kind(&other),
            tok.span,
        )),
    }
}
