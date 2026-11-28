//! Garnet v0.3 statement and block parser (Mini-Spec §6).

use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::{Span, TokenKind};

use super::{expr, types};

/// Parse a block: `{ stmt* [tail_expr] }`
pub fn parse_block(p: &mut Parser) -> Result<Block, ParseError> {
    let start = p.expect(&TokenKind::LBrace, "block")?.span;
    p.skip_separators();

    let mut items = Vec::new();
    while !matches!(p.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
        items.push(parse_block_item(p)?);
        // Skip statement separators BEFORE deciding whether the last item is
        // the tail expression. Otherwise a trailing newline before `}` demotes
        // the tail value into a discarded Stmt::Expr, and the block returns Nil.
        p.skip_separators();
    }

    p.expect(&TokenKind::RBrace, "block")?;
    let span = start.join(p.prev_span());
    Ok(fold_block_items(items, span))
}

/// Parse the body of a Ruby-style trailing block after `do` has been consumed:
/// `do [|params|] stmt* [tail_expr] end`.
pub fn parse_do_block_body(p: &mut Parser, start: Span) -> Result<Block, ParseError> {
    p.skip_separators();

    let mut items = Vec::new();
    while !matches!(p.peek_kind(), TokenKind::KwEnd | TokenKind::Eof) {
        items.push(parse_block_item(p)?);
        p.skip_separators();
    }

    p.expect(&TokenKind::KwEnd, "do/end block")?;
    let span = start.join(p.prev_span());
    Ok(fold_block_items(items, span))
}

enum BlockItem {
    Stmt(Stmt),
    Expr(Expr),
    Defer { span: Span, cleanup: Block },
}

fn block_item_span(item: &BlockItem) -> Span {
    match item {
        BlockItem::Stmt(s) => s.span(),
        BlockItem::Expr(e) => e.span(),
        BlockItem::Defer { span, .. } => *span,
    }
}

/// Fold a block's parsed items into an AST `Block`, lexically desugaring any
/// `defer` statements into nested rescue-free `Expr::Try { ensure: Some(..) }`
/// in LIFO order so the last registered `defer` runs first and the first runs
/// last. Iterative right-to-left so arbitrarily many sequential `defer`s never
/// grow recursion depth.
fn fold_block_items(items: Vec<BlockItem>, block_span: Span) -> Block {
    if !items
        .iter()
        .any(|item| matches!(item, BlockItem::Defer { .. }))
    {
        let mut stmts = Vec::with_capacity(items.len());
        let mut tail_expr = None;
        let last = items.len().saturating_sub(1);
        for (i, item) in items.into_iter().enumerate() {
            match item {
                BlockItem::Stmt(s) => stmts.push(s),
                BlockItem::Expr(e) => {
                    if i == last {
                        tail_expr = Some(Box::new(e));
                    } else {
                        stmts.push(Stmt::Expr(e));
                    }
                }
                BlockItem::Defer { .. } => unreachable!(),
            }
        }
        return Block {
            stmts,
            tail_expr,
            span: block_span,
        };
    }

    let mut deferred_segments: Vec<(Vec<BlockItem>, Span, Block)> = Vec::new();
    let mut current: Vec<BlockItem> = Vec::new();
    for item in items {
        match item {
            BlockItem::Defer { span, cleanup } => {
                deferred_segments.push((std::mem::take(&mut current), span, cleanup));
            }
            other => current.push(other),
        }
    }

    let Some((mut seg_items, defer_span, cleanup)) = deferred_segments.pop() else {
        unreachable!("checked that at least one Defer is present");
    };

    let rest_span = match (current.first(), current.last()) {
        (Some(first), Some(last)) => block_item_span(first).join(block_item_span(last)),
        _ => defer_span,
    };
    let mut rest_stmts = Vec::with_capacity(current.len());
    let mut rest_tail = None;
    let last_idx = current.len().saturating_sub(1);
    for (i, item) in current.into_iter().enumerate() {
        match item {
            BlockItem::Stmt(s) => rest_stmts.push(s),
            BlockItem::Expr(e) => {
                if i == last_idx {
                    rest_tail = Some(Box::new(e));
                } else {
                    rest_stmts.push(Stmt::Expr(e));
                }
            }
            BlockItem::Defer { .. } => unreachable!(),
        }
    }
    let rest_block = Block {
        stmts: rest_stmts,
        tail_expr: rest_tail,
        span: rest_span,
    };
    let try_span = defer_span.join(rest_block.span);
    let mut wrapped = Expr::Try {
        body: rest_block,
        rescues: Vec::new(),
        ensure: Some(cleanup),
        span: try_span,
    };

    while let Some((prev_seg, prev_defer_span, prev_cleanup)) = deferred_segments.pop() {
        let seg_span = match seg_items.first() {
            Some(first) => block_item_span(first).join(wrapped.span()),
            None => wrapped.span(),
        };
        let stmts: Vec<Stmt> = seg_items
            .into_iter()
            .map(|item| match item {
                BlockItem::Stmt(s) => s,
                BlockItem::Expr(e) => Stmt::Expr(e),
                BlockItem::Defer { .. } => unreachable!(),
            })
            .collect();
        let body = Block {
            stmts,
            tail_expr: Some(Box::new(wrapped)),
            span: seg_span,
        };
        let span = prev_defer_span.join(body.span);
        wrapped = Expr::Try {
            body,
            rescues: Vec::new(),
            ensure: Some(prev_cleanup),
            span,
        };
        seg_items = prev_seg;
    }

    let stmts: Vec<Stmt> = seg_items
        .into_iter()
        .map(|item| match item {
            BlockItem::Stmt(s) => s,
            BlockItem::Expr(e) => Stmt::Expr(e),
            BlockItem::Defer { .. } => unreachable!(),
        })
        .collect();
    Block {
        stmts,
        tail_expr: Some(Box::new(wrapped)),
        span: block_span,
    }
}

fn parse_block_item(p: &mut Parser) -> Result<BlockItem, ParseError> {
    match p.peek_kind() {
        TokenKind::KwLet => {
            let decl = parse_let_decl(p)?;
            Ok(BlockItem::Stmt(Stmt::Let(decl)))
        }
        TokenKind::KwVar => {
            let decl = parse_var_decl(p)?;
            Ok(BlockItem::Stmt(Stmt::Var(decl)))
        }
        TokenKind::KwConst => {
            let decl = parse_const_decl(p, false)?;
            Ok(BlockItem::Stmt(Stmt::Const(decl)))
        }
        TokenKind::KwWhile => {
            let start = p.bump().span;
            let condition = expr::parse_expr(p)?;
            let body = parse_block(p)?;
            let span = start.join(body.span);
            Ok(BlockItem::Stmt(Stmt::While {
                condition,
                body,
                span,
            }))
        }
        TokenKind::KwUntil => {
            let start = p.bump().span;
            let raw_cond = expr::parse_expr(p)?;
            let cond_span = raw_cond.span();
            let condition = Expr::Unary {
                op: UnOp::Not,
                expr: Box::new(raw_cond),
                span: cond_span,
            };
            let body = parse_block(p)?;
            let span = start.join(body.span);
            Ok(BlockItem::Stmt(Stmt::While {
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
            Ok(BlockItem::Stmt(Stmt::For {
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
            Ok(BlockItem::Stmt(Stmt::Loop { body, span }))
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
            Ok(BlockItem::Stmt(Stmt::Break { value, span }))
        }
        TokenKind::KwContinue => {
            let span = p.bump().span;
            Ok(BlockItem::Stmt(Stmt::Continue { span }))
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
            Ok(BlockItem::Stmt(Stmt::Return { value, span }))
        }
        TokenKind::KwYield => {
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
            Ok(BlockItem::Stmt(Stmt::Yield { value, span }))
        }
        TokenKind::KwNext => {
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
            Ok(BlockItem::Stmt(Stmt::Next { value, span }))
        }
        TokenKind::KwRaise => {
            let start = p.bump().span;
            let value = expr::parse_expr(p)?;
            let span = start.join(value.span());
            Ok(BlockItem::Stmt(Stmt::Raise { value, span }))
        }
        TokenKind::KwDefer => {
            let start = p.bump().span;
            let cleanup = if matches!(p.peek_kind(), TokenKind::LBrace) {
                parse_block(p)?
            } else {
                let expr = expr::parse_expr(p)?;
                let expr_span = expr.span();
                Block {
                    stmts: Vec::new(),
                    tail_expr: Some(Box::new(expr)),
                    span: expr_span,
                }
            };
            let span = start.join(cleanup.span);
            Ok(BlockItem::Defer { span, cleanup })
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
                Ok(BlockItem::Stmt(Stmt::Assign {
                    target: lhs,
                    op,
                    value: rhs,
                    span,
                }))
            } else {
                Ok(BlockItem::Expr(lhs))
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
