//! Garnet v0.3 Concrete Syntax Tree (CST) implementation.
//! Provides a source-faithful representation of the source code preserving
//! all comments, whitespace, and formatting details.

use crate::ast::{
    ActorItem, Block, ClosureBody, ConstDecl, Expr, FieldDef, FnDef, Item, LetDecl, Module, Stmt,
    TraitItem,
};
use crate::token::{Span, Token, TokenKind};

/// Represents the syntactic category of a CST node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CstNodeKind {
    SourceFile,
    UseDecl,
    ModuleDecl,
    MemoryDecl,
    ActorDef,
    ProtocolDecl,
    HandlerDecl,
    StructDef,
    FieldDef,
    EnumDef,
    Variant,
    TraitDef,
    ProtocolDef,
    FnSig,
    ImplBlock,
    FnDef,
    ConstDecl,
    LetDecl,
    VarDecl,
    Block,
    Stmt,
    Expr,
    MatchArm,
    RescueClause,
}

/// An element in the Concrete Syntax Tree.
#[derive(Debug, Clone, PartialEq)]
pub enum CstElement {
    Node(CstNode),
    Token(CstToken),
}

/// A node in the Concrete Syntax Tree, containing children.
#[derive(Debug, Clone, PartialEq)]
pub struct CstNode {
    pub kind: CstNodeKind,
    pub span: Span,
    pub children: Vec<CstElement>,
}

/// A leaf token in the Concrete Syntax Tree.
#[derive(Debug, Clone, PartialEq)]
pub struct CstToken {
    pub kind: TokenKind,
    pub span: Span,
}

impl CstElement {
    /// Recursively reconstructs the exact source string for this element.
    pub fn to_string(&self, src: &str) -> String {
        match self {
            CstElement::Node(node) => node.to_string(src),
            CstElement::Token(token) => token.to_string(src),
        }
    }
}

impl CstNode {
    /// Recursively reconstructs the exact source string for this node.
    pub fn to_string(&self, src: &str) -> String {
        let mut res = String::new();
        for child in &self.children {
            res.push_str(&child.to_string(src));
        }
        res
    }

    /// Construct a source-faithful Concrete Syntax Tree from an AST Module and the raw lexed Token stream.
    pub fn from_ast_and_tokens(module: &Module, tokens: Vec<Token>) -> Self {
        let mut ast_spans = Vec::new();
        collect_module(module, &mut ast_spans);

        // Map AST spans to CombinedItems
        let ast_items: Vec<_> = ast_spans
            .into_iter()
            .enumerate()
            .map(|(index, (span, kind))| CombinedItem::Node { kind, span, index })
            .collect();

        // Map raw tokens to CombinedItems
        let token_items: Vec<_> = tokens
            .into_iter()
            .map(|token| CombinedItem::Token { token })
            .collect();

        let mut all_items = ast_items;
        all_items.extend(token_items);

        // Sort items using a topological nested interval comparison
        all_items.sort_by(|a, b| {
            let sa = a.span();
            let sb = b.span();
            sa.start
                .cmp(&sb.start)
                .then_with(|| sb.end().cmp(&sa.end())) // longer spans first
                .then_with(|| match (a, b) {
                    (CombinedItem::Node { .. }, CombinedItem::Token { .. }) => {
                        std::cmp::Ordering::Less
                    }
                    (CombinedItem::Token { .. }, CombinedItem::Node { .. }) => {
                        std::cmp::Ordering::Greater
                    }
                    (
                        CombinedItem::Node { index: idx_a, .. },
                        CombinedItem::Node { index: idx_b, .. },
                    ) => idx_a.cmp(idx_b),
                    (CombinedItem::Token { .. }, CombinedItem::Token { .. }) => {
                        std::cmp::Ordering::Equal
                    }
                })
        });

        let mut stack: Vec<CstNode> = Vec::new();

        for item in all_items {
            match item {
                CombinedItem::Node { kind, span, .. } => {
                    // Pop any active nodes that finished before this node starts
                    while let Some(top) = stack.last() {
                        if top.span.start <= span.start && span.end() <= top.span.end() {
                            break;
                        }
                        let finished = stack.pop().unwrap();
                        if let Some(parent) = stack.last_mut() {
                            parent.children.push(CstElement::Node(finished));
                        } else {
                            stack.push(finished);
                            break;
                        }
                    }
                    stack.push(CstNode {
                        kind,
                        span,
                        children: Vec::new(),
                    });
                }
                CombinedItem::Token { token } => {
                    // Pop any active nodes that finished before this token starts
                    while let Some(top) = stack.last() {
                        if top.span.start <= token.span.start && token.span.end() <= top.span.end()
                        {
                            break;
                        }
                        let finished = stack.pop().unwrap();
                        if let Some(parent) = stack.last_mut() {
                            parent.children.push(CstElement::Node(finished));
                        } else {
                            stack.push(finished);
                            break;
                        }
                    }

                    if let Some(top) = stack.last_mut() {
                        top.children.push(CstElement::Token(CstToken {
                            kind: token.kind,
                            span: token.span,
                        }));
                    }
                }
            }
        }

        // Pop remaining active nodes
        while stack.len() > 1 {
            let finished = stack.pop().unwrap();
            stack
                .last_mut()
                .unwrap()
                .children
                .push(CstElement::Node(finished));
        }

        stack.pop().expect("CST must have a root SourceFile node")
    }
}

impl CstToken {
    /// Reconstructs the exact source string representing this token.
    pub fn to_string(&self, src: &str) -> String {
        src[self.span.start..self.span.end()].to_string()
    }
}

enum CombinedItem {
    Node {
        kind: CstNodeKind,
        span: Span,
        index: usize,
    },
    Token {
        token: Token,
    },
}

impl CombinedItem {
    fn span(&self) -> Span {
        match self {
            CombinedItem::Node { span, .. } => *span,
            CombinedItem::Token { token } => token.span,
        }
    }
}

// AST traversal for span collection.

fn collect_module(module: &Module, out: &mut Vec<(Span, CstNodeKind)>) {
    out.push((module.span, CstNodeKind::SourceFile));
    for item in &module.items {
        collect_item(item, out);
    }
}

fn collect_item(item: &Item, out: &mut Vec<(Span, CstNodeKind)>) {
    match item {
        Item::Use(x) => {
            out.push((x.span, CstNodeKind::UseDecl));
        }
        Item::Module(x) => {
            out.push((x.span, CstNodeKind::ModuleDecl));
            for item in &x.items {
                collect_item(item, out);
            }
        }
        Item::Memory(x) => {
            out.push((x.span, CstNodeKind::MemoryDecl));
        }
        Item::Actor(x) => {
            out.push((x.span, CstNodeKind::ActorDef));
            for item in &x.items {
                match item {
                    ActorItem::Protocol(p) => out.push((p.span, CstNodeKind::ProtocolDecl)),
                    ActorItem::Handler(h) => {
                        out.push((h.span, CstNodeKind::HandlerDecl));
                        collect_block(&h.body, out);
                    }
                    ActorItem::Memory(m) => out.push((m.span, CstNodeKind::MemoryDecl)),
                    ActorItem::Let(l) => collect_let_decl(l, out),
                }
            }
        }
        Item::Struct(x) => {
            out.push((x.span, CstNodeKind::StructDef));
            for f in &x.fields {
                collect_field_def(f, out);
            }
        }
        Item::Enum(x) => {
            out.push((x.span, CstNodeKind::EnumDef));
            for v in &x.variants {
                out.push((v.span, CstNodeKind::Variant));
            }
        }
        Item::Trait(x) => {
            out.push((x.span, CstNodeKind::TraitDef));
            for item in &x.items {
                match item {
                    TraitItem::FnSig(sig) => out.push((sig.span, CstNodeKind::FnSig)),
                    TraitItem::Const(c) => collect_const_decl(c, out),
                }
            }
        }
        Item::Protocol(x) => {
            out.push((x.span, CstNodeKind::ProtocolDef));
            for item in &x.items {
                match item {
                    TraitItem::FnSig(sig) => out.push((sig.span, CstNodeKind::FnSig)),
                    TraitItem::Const(c) => collect_const_decl(c, out),
                }
            }
        }
        Item::Impl(x) => {
            out.push((x.span, CstNodeKind::ImplBlock));
            for m in &x.methods {
                collect_fn_def(m, out);
            }
        }
        Item::Fn(x) => {
            collect_fn_def(x, out);
        }
        Item::Const(x) => {
            collect_const_decl(x, out);
        }
        Item::Let(x) => {
            collect_let_decl(x, out);
        }
    }
}

fn collect_fn_def(x: &FnDef, out: &mut Vec<(Span, CstNodeKind)>) {
    out.push((x.span, CstNodeKind::FnDef));
    collect_block(&x.body, out);
}

fn collect_const_decl(x: &ConstDecl, out: &mut Vec<(Span, CstNodeKind)>) {
    out.push((x.span, CstNodeKind::ConstDecl));
    collect_expr(&x.value, out);
}

fn collect_let_decl(x: &LetDecl, out: &mut Vec<(Span, CstNodeKind)>) {
    out.push((x.span, CstNodeKind::LetDecl));
    collect_expr(&x.value, out);
}

fn collect_field_def(x: &FieldDef, out: &mut Vec<(Span, CstNodeKind)>) {
    out.push((x.span, CstNodeKind::FieldDef));
    if let Some(d) = &x.default {
        collect_expr(d, out);
    }
}

fn collect_block(x: &Block, out: &mut Vec<(Span, CstNodeKind)>) {
    out.push((x.span, CstNodeKind::Block));
    for stmt in &x.stmts {
        collect_stmt(stmt, out);
    }
    if let Some(t) = &x.tail_expr {
        collect_expr(t, out);
    }
}

fn collect_stmt(stmt: &Stmt, out: &mut Vec<(Span, CstNodeKind)>) {
    match stmt {
        Stmt::Let(x) => collect_let_decl(x, out),
        Stmt::Var(x) => {
            out.push((x.span, CstNodeKind::VarDecl));
            collect_expr(&x.value, out);
        }
        Stmt::Const(x) => collect_const_decl(x, out),
        Stmt::Assign {
            target,
            value,
            span,
            ..
        } => {
            out.push((*span, CstNodeKind::Stmt));
            collect_expr(target, out);
            collect_expr(value, out);
        }
        Stmt::While {
            condition,
            body,
            span,
        } => {
            out.push((*span, CstNodeKind::Stmt));
            collect_expr(condition, out);
            collect_block(body, out);
        }
        Stmt::For {
            iter, body, span, ..
        } => {
            out.push((*span, CstNodeKind::Stmt));
            collect_expr(iter, out);
            collect_block(body, out);
        }
        Stmt::Loop { body, span } => {
            out.push((*span, CstNodeKind::Stmt));
            collect_block(body, out);
        }
        Stmt::Break { value, span } => {
            out.push((*span, CstNodeKind::Stmt));
            if let Some(v) = value {
                collect_expr(v, out);
            }
        }
        Stmt::Continue { span } => {
            out.push((*span, CstNodeKind::Stmt));
        }
        Stmt::Return { value, span } => {
            out.push((*span, CstNodeKind::Stmt));
            if let Some(v) = value {
                collect_expr(v, out);
            }
        }
        Stmt::Yield { value, span } => {
            out.push((*span, CstNodeKind::Stmt));
            if let Some(v) = value {
                collect_expr(v, out);
            }
        }
        Stmt::Next { value, span } => {
            out.push((*span, CstNodeKind::Stmt));
            if let Some(v) = value {
                collect_expr(v, out);
            }
        }
        Stmt::Raise { value, span } => {
            out.push((*span, CstNodeKind::Stmt));
            collect_expr(value, out);
        }
        Stmt::Expr(x) => {
            collect_expr(x, out);
        }
    }
}

fn collect_expr(expr: &Expr, out: &mut Vec<(Span, CstNodeKind)>) {
    out.push((expr.span(), CstNodeKind::Expr));
    match expr {
        Expr::Int(..)
        | Expr::Float(..)
        | Expr::Bool(..)
        | Expr::Nil(..)
        | Expr::Str(..)
        | Expr::Symbol(..)
        | Expr::Ident(..)
        | Expr::Path(..) => {}
        Expr::Binary { lhs, rhs, .. } => {
            collect_expr(lhs, out);
            collect_expr(rhs, out);
        }
        Expr::Unary { expr: inner, .. } => {
            collect_expr(inner, out);
        }
        Expr::Call { callee, args, .. } => {
            collect_expr(callee, out);
            for arg in args {
                collect_expr(arg, out);
            }
        }
        Expr::Method { receiver, args, .. } => {
            collect_expr(receiver, out);
            for arg in args {
                collect_expr(arg, out);
            }
        }
        Expr::Field { receiver, .. } => {
            collect_expr(receiver, out);
        }
        Expr::Index {
            receiver, index, ..
        } => {
            collect_expr(receiver, out);
            collect_expr(index, out);
        }
        Expr::Cast { expr: inner, .. } => {
            collect_expr(inner, out);
        }
        Expr::If {
            condition,
            then_block,
            elsif_clauses,
            else_block,
            ..
        } => {
            collect_expr(condition, out);
            collect_block(then_block, out);
            for (cond, block) in elsif_clauses {
                collect_expr(cond, out);
                collect_block(block, out);
            }
            if let Some(eb) = else_block {
                collect_block(eb, out);
            }
        }
        Expr::Match { subject, arms, .. } => {
            collect_expr(subject, out);
            for arm in arms {
                out.push((arm.span, CstNodeKind::MatchArm));
                collect_block(&arm.body, out);
            }
        }
        Expr::Try {
            body,
            rescues,
            ensure,
            ..
        } => {
            collect_block(body, out);
            for r in rescues {
                out.push((r.span, CstNodeKind::RescueClause));
                collect_block(&r.body, out);
            }
            if let Some(ens) = ensure {
                collect_block(ens, out);
            }
        }
        Expr::Closure { body, .. } => match &**body {
            ClosureBody::Block(b) => collect_block(b, out),
            ClosureBody::Expr(e) => collect_expr(e, out),
        },
        Expr::Spawn { expr: inner, .. } => {
            collect_expr(inner, out);
        }
        Expr::Array { elements, .. } => {
            for el in elements {
                collect_expr(el, out);
            }
        }
        Expr::Map { entries, .. } => {
            for (k, v) in entries {
                collect_expr(k, out);
                collect_expr(v, out);
            }
        }
    }
}
