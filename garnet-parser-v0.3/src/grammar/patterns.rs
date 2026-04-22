//! Garnet v0.3 pattern parser (Mini-Spec §6.3).

use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TokenKind;

/// Parse a pattern for match arms.
pub fn parse_pattern(p: &mut Parser) -> Result<Pattern, ParseError> {
    let tok = p.peek().clone();
    match &tok.kind {
        // Wildcard: _
        TokenKind::Ident(name) if name == "_" => {
            p.bump();
            Ok(Pattern::Wildcard(tok.span))
        }
        // Rest: ..
        TokenKind::DotDot => {
            p.bump();
            Ok(Pattern::Rest(tok.span))
        }
        // Tuple pattern: (a, b, c)
        TokenKind::LParen => {
            let start = p.bump().span;
            let mut patterns = Vec::new();
            if !matches!(p.peek_kind(), TokenKind::RParen) {
                loop {
                    patterns.push(parse_pattern(p)?);
                    if !p.eat(&TokenKind::Comma) {
                        break;
                    }
                }
            }
            p.expect(&TokenKind::RParen, "tuple pattern")?;
            let span = start.join(p.prev_span());
            Ok(Pattern::Tuple(patterns, span))
        }
        // Literal patterns
        TokenKind::Int(v) => {
            let v = *v;
            p.bump();
            Ok(Pattern::Literal(Expr::Int(v, tok.span), tok.span))
        }
        TokenKind::Float(v) => {
            let v = *v;
            p.bump();
            Ok(Pattern::Literal(Expr::Float(v, tok.span), tok.span))
        }
        TokenKind::Str(parts) => {
            let parts = parts.clone();
            p.bump();
            Ok(Pattern::Literal(
                Expr::Str(
                    StringLit {
                        parts: parts
                            .into_iter()
                            .map(|sp| match sp {
                                crate::token::StrPart::Lit(s) => crate::token::StrPart::Lit(s),
                                crate::token::StrPart::Interp(s) => crate::token::StrPart::Interp(s),
                            })
                            .collect(),
                    },
                    tok.span,
                ),
                tok.span,
            ))
        }
        TokenKind::Symbol(name) => {
            let name = name.clone();
            p.bump();
            Ok(Pattern::Literal(Expr::Symbol(name, tok.span), tok.span))
        }
        TokenKind::KwTrue => {
            p.bump();
            Ok(Pattern::Literal(Expr::Bool(true, tok.span), tok.span))
        }
        TokenKind::KwFalse => {
            p.bump();
            Ok(Pattern::Literal(Expr::Bool(false, tok.span), tok.span))
        }
        TokenKind::KwNil => {
            p.bump();
            Ok(Pattern::Literal(Expr::Nil(tok.span), tok.span))
        }
        // Ident or Enum pattern: `name` or `Name(args)` or `Path::Name(args)`
        TokenKind::Ident(_) => {
            let (name, start_span) = p.expect_ident("pattern")?;
            let mut path = vec![name.clone()];

            // Check for :: path extension
            while p.eat(&TokenKind::ColonCol) {
                let (seg, _) = p.expect_ident("pattern path")?;
                path.push(seg);
            }

            // Check for ( args ) — enum pattern
            if p.eat(&TokenKind::LParen) {
                let mut sub_patterns = Vec::new();
                if !matches!(p.peek_kind(), TokenKind::RParen) {
                    loop {
                        sub_patterns.push(parse_pattern(p)?);
                        if !p.eat(&TokenKind::Comma) {
                            break;
                        }
                    }
                }
                p.expect(&TokenKind::RParen, "enum pattern")?;
                let span = start_span.join(p.prev_span());
                Ok(Pattern::Enum(path, sub_patterns, span))
            } else if path.len() > 1 {
                // Multi-segment path without parens — still an enum pattern with no fields
                let span = start_span.join(p.prev_span());
                Ok(Pattern::Enum(path, Vec::new(), span))
            } else {
                // Simple identifier pattern (binding)
                Ok(Pattern::Ident(name, start_span))
            }
        }
        _ => Err(ParseError::unexpected_token(
            "pattern",
            &format!("{:?}", tok.kind),
            tok.span,
        )),
    }
}
