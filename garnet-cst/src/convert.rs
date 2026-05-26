//! `cst_to_ast` — project the rowan CST onto the existing AST
//! (`garnet_parser::ast`).
//!
//! Lossy on trivia, structural on the rest: walks the CST and rebuilds the same
//! `Module` the AST parser produces. Leaf literal *values* are recovered by
//! re-lexing the leaf's source text, so `Int`/`Float`/`Str`/`Symbol` payloads
//! match the lexer exactly. Spans are derived from each node's text range;
//! validation (`tests/cst_to_ast_parity.rs`) compares span-normalized, so span
//! values are not load-bearing.
//!
//! This is the AST projection of the CST; existing AST consumers (interp,
//! check, vm) keep using `garnet_parser::parse_source` and are untouched. The
//! CST-first migration of those consumers is v0.8 work.

use crate::syntax_kind::{SyntaxKind, SyntaxNode, SyntaxToken};
use garnet_parser::ast::*;
// These AST type names collide with same-named `SyntaxKind` variants. Importing
// them explicitly shadows the `SyntaxKind::*` glob, so a bare name in a
// type/constructor position is the AST type; `SyntaxKind`-position uses of these
// names are written `SyntaxKind::Foo`.
use garnet_parser::ast::{
    ActorDef, Block, EnumDef, FnDef, FnSig, HandlerDecl, ImplBlock, MatchArm, MemoryDecl, Module,
    Param, ProtocolDecl, RescueClause, StructDef, TraitDef, UseDecl, Variant,
};
use garnet_parser::lex_source;
use garnet_parser::token::{Span, TokenKind};
use SyntaxKind::*;

/// Project a CST root node onto the AST `Module`.
#[must_use]
pub fn cst_to_ast(root: &SyntaxNode) -> Module {
    let safe = root
        .children()
        .any(|n| n.kind() == Attr && node_has_ident(&n, "safe"));
    let items = root.children().filter_map(lower_item).collect();
    Module {
        safe,
        items,
        span: span_of(root),
    }
}

// ── span / token helpers ────────────────────────────────────────────────

fn span_of(node: &SyntaxNode) -> Span {
    let r = node.text_range();
    Span::new(usize::from(r.start()), usize::from(r.len()))
}

fn child(node: &SyntaxNode, kind: SyntaxKind) -> Option<SyntaxNode> {
    node.children().find(|n| n.kind() == kind)
}

fn first_token_kind(node: &SyntaxNode, kind: SyntaxKind) -> Option<SyntaxToken> {
    node.children_with_tokens()
        .filter_map(|e| e.into_token())
        .find(|t| t.kind() == kind)
}

fn has_token(node: &SyntaxNode, kind: SyntaxKind) -> bool {
    first_token_kind(node, kind).is_some()
}

/// First non-trivia token text inside a `Name` (or similar) node.
fn name_text(node: &SyntaxNode) -> String {
    node.children_with_tokens()
        .filter_map(|e| e.into_token())
        .find(|t| !t.kind().is_trivia())
        .map(|t| t.text().to_string())
        .unwrap_or_default()
}

fn child_name(node: &SyntaxNode) -> String {
    child(node, Name).map(|n| name_text(&n)).unwrap_or_default()
}

/// The last direct `NameRef` child's text. In a field/method access the
/// receiver may itself be a `NameRef` (the *first* one), so the accessed
/// field/method name is the trailing `NameRef` (after the `.`).
fn last_name_ref(node: &SyntaxNode) -> String {
    node.children()
        .filter(|n| n.kind() == NameRef)
        .last()
        .map(|n| name_text(&n))
        .unwrap_or_default()
}

fn node_has_ident(node: &SyntaxNode, want: &str) -> bool {
    node.children_with_tokens()
        .filter_map(|e| e.into_token())
        .any(|t| t.kind() == Ident && t.text() == want)
}

fn is_expr_kind(k: SyntaxKind) -> bool {
    matches!(
        k,
        Literal
            | NameRef
            | ParenExpr
            | ArrayLit
            | MapLit
            | IfExpr
            | MatchExpr
            | TryExpr
            | SpawnExpr
            | ClosureExpr
            | BinaryExpr
            | UnaryExpr
            | CastExpr
            | RangeExpr
            | PipelineExpr
            | CallExpr
            | MethodCallExpr
            | FieldExpr
            | IndexExpr
            | PathExpr
    )
}

fn is_type_kind(k: SyntaxKind) -> bool {
    matches!(k, TypeRef | FnType | TupleType)
}

fn child_exprs(node: &SyntaxNode) -> Vec<SyntaxNode> {
    node.children().filter(|n| is_expr_kind(n.kind())).collect()
}

fn first_expr(node: &SyntaxNode) -> Option<Expr> {
    node.children()
        .find(|n| is_expr_kind(n.kind()))
        .map(|n| lower_expr(&n))
}

fn first_type(node: &SyntaxNode) -> Option<TypeExpr> {
    node.children()
        .find(|n| is_type_kind(n.kind()))
        .map(|n| lower_type(&n))
}

// ── items ─────────────────────────────────────────────────────────────

fn lower_item(node: SyntaxNode) -> Option<Item> {
    Some(match node.kind() {
        SyntaxKind::UseDecl => Item::Use(lower_use(&node)),
        SyntaxKind::Module => Item::Module(lower_module_decl(&node)),
        SyntaxKind::MemoryDecl => Item::Memory(lower_memory(&node)),
        SyntaxKind::ActorDef => Item::Actor(lower_actor(&node)),
        SyntaxKind::StructDef => Item::Struct(lower_struct(&node)),
        SyntaxKind::EnumDef => Item::Enum(lower_enum(&node)),
        SyntaxKind::TraitDef => Item::Trait(lower_trait(&node)),
        StructProtocolDef => Item::Protocol(lower_protocol(&node)),
        SyntaxKind::ImplBlock => Item::Impl(lower_impl(&node)),
        SyntaxKind::FnDef | SafeFnDef => Item::Fn(lower_fn(&node)),
        ConstDef => Item::Const(lower_const(&node)),
        LetStmt => Item::Let(lower_let(&node)),
        _ => return None,
    })
}

fn lower_annotations(node: &SyntaxNode) -> Vec<Annotation> {
    let Some(list) = child(node, AttrList) else {
        return Vec::new();
    };
    list.children()
        .filter(|n| n.kind() == Attr)
        .filter_map(|attr| lower_annotation(&attr))
        .collect()
}

fn lower_annotation(attr: &SyntaxNode) -> Option<Annotation> {
    let span = span_of(attr);
    let name = attr
        .children_with_tokens()
        .filter_map(|e| e.into_token())
        .find(|t| t.kind() == Ident)?
        .text()
        .to_string();
    let int_arg = || -> i64 {
        attr.descendants_with_tokens()
            .filter_map(|e| e.into_token())
            .find(|t| t.kind() == IntLit)
            .and_then(|t| t.text().replace('_', "").parse::<i64>().ok())
            .unwrap_or(0)
    };
    Some(match name.as_str() {
        "max_depth" => Annotation::MaxDepth(int_arg(), span),
        "fan_out" => Annotation::FanOut(int_arg(), span),
        "require_metadata" => Annotation::RequireMetadata(span),
        "safe" => Annotation::Safe(span),
        "dynamic" => Annotation::Dynamic(span),
        "mailbox" => Annotation::Mailbox(int_arg(), span),
        "nonsendable" => Annotation::NonSendable(span),
        "caps" => {
            let caps = attr
                .descendants()
                .find(|n| n.kind() == AttrArgs)
                .map(|args| {
                    args.children_with_tokens()
                        .filter_map(|e| e.into_token())
                        .filter(|t| t.kind() == Ident || t.kind() == Star)
                        .map(|t| Capability::from_ident(t.text()))
                        .collect()
                })
                .unwrap_or_default();
            Annotation::Caps(caps, span)
        }
        _ => Annotation::RequireMetadata(span),
    })
}

fn lower_use(node: &SyntaxNode) -> UseDecl {
    let mut path = Vec::new();
    let mut imports = UseImports::Module;
    if let Some(p) = child(node, UsePath) {
        // Collect ident segments before any group; detect `*` / `{...}`.
        if has_token(&p, Star) {
            imports = UseImports::Glob;
        }
        if let Some(group) = child(&p, UseGroup) {
            let names = group
                .children_with_tokens()
                .filter_map(|e| e.into_token())
                .filter(|t| t.kind() == Ident)
                .map(|t| t.text().to_string())
                .collect();
            imports = UseImports::Named(names);
        }
        // Path segments are the idents directly under UsePath (not in a group).
        path = p
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .filter(|t| t.kind() == Ident)
            .map(|t| t.text().to_string())
            .collect();
        if let UseImports::Named(ref named) = imports {
            // Remove the grouped names that leaked into the flat ident scan.
            path.retain(|s| !named.contains(s));
        }
    }
    UseDecl {
        path,
        imports,
        span: span_of(node),
    }
}

fn lower_module_decl(node: &SyntaxNode) -> ModuleDecl {
    ModuleDecl {
        safe: child(node, AttrList)
            .map(|l| node_has_ident(&l, "safe"))
            .unwrap_or(false),
        public: has_token(node, PubKw),
        name: child_name(node),
        items: node.children().filter_map(lower_item).collect(),
        span: span_of(node),
    }
}

fn lower_memory(node: &SyntaxNode) -> MemoryDecl {
    let kind = if has_token(node, WorkingKw) {
        MemoryKind::Working
    } else if has_token(node, EpisodicKw) {
        MemoryKind::Episodic
    } else if has_token(node, SemanticKw) {
        MemoryKind::Semantic
    } else {
        MemoryKind::Procedural
    };
    MemoryDecl {
        kind,
        name: child_name(node),
        store: first_type(node).unwrap_or_else(|| empty_named_type(span_of(node))),
        span: span_of(node),
    }
}

fn lower_actor(node: &SyntaxNode) -> ActorDef {
    let items = node
        .children()
        .filter_map(|n| match n.kind() {
            SyntaxKind::ProtocolDecl => Some(ActorItem::Protocol(lower_protocol_decl(&n))),
            SyntaxKind::HandlerDecl => Some(ActorItem::Handler(lower_handler(&n))),
            SyntaxKind::MemoryDecl => Some(ActorItem::Memory(lower_memory(&n))),
            LetStmt => Some(ActorItem::Let(lower_let(&n))),
            _ => None,
        })
        .collect();
    ActorDef {
        public: has_token(node, PubKw),
        name: child_name(node),
        items,
        span: span_of(node),
    }
}

fn lower_protocol_decl(node: &SyntaxNode) -> ProtocolDecl {
    ProtocolDecl {
        name: child_name(node),
        params: lower_params(node),
        return_ty: first_type(node),
        span: span_of(node),
    }
}

fn lower_handler(node: &SyntaxNode) -> HandlerDecl {
    HandlerDecl {
        name: child_name(node),
        params: lower_params(node),
        body: child(node, SyntaxKind::Block)
            .map(|b| lower_block(&b))
            .unwrap_or_else(|| empty_block(span_of(node))),
        span: span_of(node),
    }
}

fn lower_struct(node: &SyntaxNode) -> StructDef {
    let fields = node
        .children()
        .filter(|n| n.kind() == FieldDecl)
        .map(|f| FieldDef {
            public: has_token(&f, PubKw),
            name: child_name(&f),
            ty: first_type(&f).unwrap_or_else(|| empty_named_type(span_of(&f))),
            default: f
                .children()
                .find(|n| is_expr_kind(n.kind()))
                .map(|n| lower_expr(&n)),
            span: span_of(&f),
        })
        .collect();
    StructDef {
        annotations: lower_annotations(node),
        public: has_token(node, PubKw),
        name: child_name(node),
        type_params: lower_type_params(node),
        fields,
        span: span_of(node),
    }
}

fn lower_enum(node: &SyntaxNode) -> EnumDef {
    let variants = node
        .children()
        .filter(|n| n.kind() == SyntaxKind::Variant)
        .map(|v| Variant {
            name: child_name(&v),
            fields: v
                .children()
                .filter(|n| is_type_kind(n.kind()))
                .map(|t| lower_type(&t))
                .collect(),
            span: span_of(&v),
        })
        .collect();
    EnumDef {
        public: has_token(node, PubKw),
        name: child_name(node),
        type_params: lower_type_params(node),
        variants,
        span: span_of(node),
    }
}

fn lower_trait_items(node: &SyntaxNode) -> Vec<TraitItem> {
    node.children()
        .filter_map(|n| match n.kind() {
            SyntaxKind::FnSig => Some(TraitItem::FnSig(FnSig {
                mode: if has_token(&n, FnKw) {
                    FnMode::Safe
                } else {
                    FnMode::Managed
                },
                name: child_name(&n),
                params: lower_params(&n),
                return_ty: first_type(&n),
                span: span_of(&n),
            })),
            ConstDef => Some(TraitItem::Const(lower_const(&n))),
            _ => None,
        })
        .collect()
}

fn lower_trait(node: &SyntaxNode) -> TraitDef {
    TraitDef {
        public: has_token(node, PubKw),
        name: child_name(node),
        type_params: lower_type_params(node),
        items: lower_trait_items(node),
        span: span_of(node),
    }
}

fn lower_protocol(node: &SyntaxNode) -> ProtocolDef {
    ProtocolDef {
        public: has_token(node, PubKw),
        name: child_name(node),
        type_params: lower_type_params(node),
        items: lower_trait_items(node),
        span: span_of(node),
    }
}

fn lower_impl(node: &SyntaxNode) -> ImplBlock {
    let types: Vec<SyntaxNode> = node.children().filter(|n| is_type_kind(n.kind())).collect();
    let target = types
        .first()
        .map(lower_type)
        .unwrap_or_else(|| empty_named_type(span_of(node)));
    let trait_ty = if has_token(node, ForKw) {
        // `impl Trait for Type` parses target then trait; with `for` the second
        // type node is the trait. The grammar order is `impl <T> Target for Trait`.
        types.get(1).map(lower_type)
    } else {
        None
    };
    ImplBlock {
        annotations: lower_annotations(node),
        type_params: lower_type_params(node),
        target,
        trait_ty,
        methods: node
            .children()
            .filter(|n| matches!(n.kind(), SyntaxKind::FnDef | SafeFnDef))
            .map(|m| lower_fn(&m))
            .collect(),
        span: span_of(node),
    }
}

fn lower_fn(node: &SyntaxNode) -> FnDef {
    let mode = if node.kind() == SafeFnDef || has_token(node, FnKw) {
        FnMode::Safe
    } else {
        FnMode::Managed
    };
    FnDef {
        annotations: lower_annotations(node),
        public: has_token(node, PubKw),
        mode,
        name: child_name(node),
        type_params: lower_type_params(node),
        params: lower_params(node),
        return_ty: first_type(node),
        body: child(node, SyntaxKind::Block)
            .map(|b| lower_block(&b))
            .unwrap_or_else(|| empty_block(span_of(node))),
        span: span_of(node),
    }
}

fn lower_const(node: &SyntaxNode) -> ConstDecl {
    ConstDecl {
        public: has_token(node, PubKw),
        name: child_name(node),
        ty: first_type(node),
        value: first_expr(node).unwrap_or_else(|| Expr::Nil(span_of(node))),
        span: span_of(node),
    }
}

fn lower_let(node: &SyntaxNode) -> LetDecl {
    LetDecl {
        mutable: has_token(node, MutKw),
        name: child_name(node),
        ty: first_type(node),
        value: first_expr(node).unwrap_or_else(|| Expr::Nil(span_of(node))),
        span: span_of(node),
    }
}

// ── params, type params, types ──────────────────────────────────────────

fn lower_params(node: &SyntaxNode) -> Vec<Param> {
    let Some(list) = node
        .children()
        .find(|n| matches!(n.kind(), ParamList | ClosureParamList))
    else {
        return Vec::new();
    };
    list.children()
        .filter(|n| n.kind() == SyntaxKind::Param)
        .map(|p| {
            let ownership = if has_token(&p, OwnKw) {
                Some(Ownership::Own)
            } else if has_token(&p, BorrowKw) {
                Some(Ownership::Borrow)
            } else if has_token(&p, RefKw) {
                Some(Ownership::Ref)
            } else if has_token(&p, MutKw) {
                Some(Ownership::Mut)
            } else {
                None
            };
            Param {
                ownership,
                name: child_name(&p),
                ty: first_type(&p),
                span: span_of(&p),
            }
        })
        .collect()
}

fn lower_type_params(node: &SyntaxNode) -> Vec<String> {
    child(node, TypeParamList)
        .map(|l| {
            l.children()
                .filter(|n| n.kind() == TypeParam)
                .map(|tp| {
                    tp.children_with_tokens()
                        .filter_map(|e| e.into_token())
                        .find(|t| t.kind() == Ident)
                        .map(|t| t.text().to_string())
                        .unwrap_or_default()
                })
                .collect()
        })
        .unwrap_or_default()
}

fn empty_named_type(span: Span) -> TypeExpr {
    TypeExpr::Named {
        path: Vec::new(),
        args: Vec::new(),
        span,
    }
}

fn lower_type(node: &SyntaxNode) -> TypeExpr {
    let span = span_of(node);
    match node.kind() {
        FnType => TypeExpr::Fn {
            params: node
                .children()
                .filter(|n| is_type_kind(n.kind()))
                .take_while(|_| true)
                .collect::<Vec<_>>()
                .split_last()
                .map(|(_, params)| params.iter().map(lower_type).collect())
                .unwrap_or_default(),
            ret: Box::new(
                node.children()
                    .filter(|n| is_type_kind(n.kind()))
                    .last()
                    .map(|n| lower_type(&n))
                    .unwrap_or_else(|| empty_named_type(span)),
            ),
            span,
        },
        TupleType => TypeExpr::Tuple {
            elements: node
                .children()
                .filter(|n| is_type_kind(n.kind()))
                .map(|n| lower_type(&n))
                .collect(),
            span,
        },
        TypeRef => {
            if has_token(node, Amp) {
                TypeExpr::Ref {
                    mutable: has_token(node, MutKw),
                    inner: Box::new(
                        node.children()
                            .find(|n| is_type_kind(n.kind()))
                            .map(|n| lower_type(&n))
                            .unwrap_or_else(|| empty_named_type(span)),
                    ),
                    span,
                }
            } else if has_token(node, DynKw) {
                TypeExpr::Dyn {
                    trait_ty: Box::new(named_from_ref(node, span)),
                    span,
                }
            } else {
                named_from_ref(node, span)
            }
        }
        _ => empty_named_type(span),
    }
}

fn named_from_ref(node: &SyntaxNode, span: Span) -> TypeExpr {
    let path = node
        .children_with_tokens()
        .filter_map(|e| e.into_token())
        .filter(|t| t.kind() == Ident)
        .map(|t| t.text().to_string())
        .collect();
    let args = child(node, TypeArgList)
        .map(|l| {
            l.children()
                .filter(|n| is_type_kind(n.kind()))
                .map(|n| lower_type(&n))
                .collect()
        })
        .unwrap_or_default();
    TypeExpr::Named { path, args, span }
}

// ── blocks & statements ──────────────────────────────────────────────────

fn empty_block(span: Span) -> Block {
    Block {
        stmts: Vec::new(),
        tail_expr: None,
        span,
    }
}

fn lower_block(node: &SyntaxNode) -> Block {
    let stmt_nodes: Vec<SyntaxNode> = node.children().filter(|n| is_stmt_node(n.kind())).collect();
    let mut stmts = Vec::new();
    let mut tail_expr = None;
    let last = stmt_nodes.len().saturating_sub(1);
    for (i, n) in stmt_nodes.iter().enumerate() {
        if i == last && n.kind() == ExprStmt {
            tail_expr = first_expr(n).map(Box::new);
        } else if let Some(stmt) = lower_stmt(n) {
            stmts.push(stmt);
        }
    }
    Block {
        stmts,
        tail_expr,
        span: span_of(node),
    }
}

fn is_stmt_node(k: SyntaxKind) -> bool {
    matches!(
        k,
        LetStmt
            | VarStmt
            | ConstDef
            | AssignStmt
            | WhileStmt
            | ForStmt
            | LoopStmt
            | BreakStmt
            | ContinueStmt
            | ReturnStmt
            | YieldStmt
            | NextStmt
            | RaiseStmt
            | ExprStmt
    )
}

fn lower_stmt(node: &SyntaxNode) -> Option<Stmt> {
    let span = span_of(node);
    Some(match node.kind() {
        LetStmt => Stmt::Let(lower_let(node)),
        VarStmt => Stmt::Var(VarDecl {
            name: child_name(node),
            ty: first_type(node),
            value: first_expr(node).unwrap_or(Expr::Nil(span)),
            span,
        }),
        ConstDef => Stmt::Const(lower_const(node)),
        AssignStmt => {
            let exprs = child_exprs(node);
            let op = assign_op(node);
            Stmt::Assign {
                target: exprs.first().map(lower_expr).unwrap_or(Expr::Nil(span)),
                op,
                value: exprs.get(1).map(lower_expr).unwrap_or(Expr::Nil(span)),
                span,
            }
        }
        WhileStmt => Stmt::While {
            condition: first_expr(node).unwrap_or(Expr::Nil(span)),
            body: child(node, SyntaxKind::Block)
                .map(|b| lower_block(&b))
                .unwrap_or_else(|| empty_block(span)),
            span,
        },
        ForStmt => Stmt::For {
            var: child_name(node),
            iter: first_expr(node).unwrap_or(Expr::Nil(span)),
            body: child(node, SyntaxKind::Block)
                .map(|b| lower_block(&b))
                .unwrap_or_else(|| empty_block(span)),
            span,
        },
        LoopStmt => Stmt::Loop {
            body: child(node, SyntaxKind::Block)
                .map(|b| lower_block(&b))
                .unwrap_or_else(|| empty_block(span)),
            span,
        },
        BreakStmt => Stmt::Break {
            value: first_expr(node),
            span,
        },
        ContinueStmt => Stmt::Continue { span },
        ReturnStmt => Stmt::Return {
            value: first_expr(node),
            span,
        },
        YieldStmt => Stmt::Yield {
            value: first_expr(node),
            span,
        },
        NextStmt => Stmt::Next {
            value: first_expr(node),
            span,
        },
        RaiseStmt => Stmt::Raise {
            value: first_expr(node).unwrap_or(Expr::Nil(span)),
            span,
        },
        ExprStmt => Stmt::Expr(first_expr(node).unwrap_or(Expr::Nil(span))),
        _ => return None,
    })
}

fn assign_op(node: &SyntaxNode) -> AssignOp {
    for e in node.children_with_tokens() {
        if let Some(t) = e.into_token() {
            match t.kind() {
                Eq => return AssignOp::Eq,
                PlusEq => return AssignOp::PlusEq,
                MinusEq => return AssignOp::MinusEq,
                StarEq => return AssignOp::StarEq,
                SlashEq => return AssignOp::SlashEq,
                PercentEq => return AssignOp::PercentEq,
                _ => {}
            }
        }
    }
    AssignOp::Eq
}

// ── expressions ───────────────────────────────────────────────────────────

fn lower_expr(node: &SyntaxNode) -> Expr {
    let span = span_of(node);
    match node.kind() {
        Literal => lower_literal(node, span),
        NameRef => lower_name_ref(node, span),
        ParenExpr => first_expr(node).unwrap_or(Expr::Nil(span)),
        ArrayLit => Expr::Array {
            elements: child_exprs(node).iter().map(lower_expr).collect(),
            span,
        },
        MapLit => Expr::Map {
            entries: node
                .children()
                .filter(|n| n.kind() == MapEntry)
                .map(|e| {
                    let xs = child_exprs(&e);
                    (
                        xs.first().map(lower_expr).unwrap_or(Expr::Nil(span)),
                        xs.get(1).map(lower_expr).unwrap_or(Expr::Nil(span)),
                    )
                })
                .collect(),
            span,
        },
        IfExpr => lower_if(node, span),
        MatchExpr => lower_match(node, span),
        TryExpr => lower_try(node, span),
        SpawnExpr => Expr::Spawn {
            expr: Box::new(first_expr(node).unwrap_or(Expr::Nil(span))),
            span,
        },
        ClosureExpr => lower_closure(node, span),
        BinaryExpr | RangeExpr | PipelineExpr => lower_binary(node, span),
        UnaryExpr => lower_unary(node, span),
        CastExpr => Expr::Cast {
            expr: Box::new(first_expr(node).unwrap_or(Expr::Nil(span))),
            ty: first_type(node).unwrap_or_else(|| empty_named_type(span)),
            span,
        },
        CallExpr => {
            let exprs = child_exprs(node);
            let callee = exprs.first().map(lower_expr).unwrap_or(Expr::Nil(span));
            Expr::Call {
                callee: Box::new(callee),
                args: lower_args(node),
                span,
            }
        }
        MethodCallExpr => {
            let receiver = child_exprs(node)
                .first()
                .map(lower_expr)
                .unwrap_or(Expr::Nil(span));
            let method = last_name_ref(node);
            Expr::Method {
                receiver: Box::new(receiver),
                method,
                args: lower_args(node),
                span,
            }
        }
        FieldExpr => {
            let receiver = child_exprs(node)
                .first()
                .map(lower_expr)
                .unwrap_or(Expr::Nil(span));
            let field = last_name_ref(node);
            Expr::Field {
                receiver: Box::new(receiver),
                field,
                span,
            }
        }
        IndexExpr => {
            let exprs = child_exprs(node);
            Expr::Index {
                receiver: Box::new(exprs.first().map(lower_expr).unwrap_or(Expr::Nil(span))),
                index: Box::new(exprs.get(1).map(lower_expr).unwrap_or(Expr::Nil(span))),
                span,
            }
        }
        PathExpr => lower_path(node, span),
        _ => Expr::Nil(span),
    }
}

fn lower_literal(node: &SyntaxNode, span: Span) -> Expr {
    let text = node
        .children_with_tokens()
        .filter_map(|e| e.into_token())
        .find(|t| !t.kind().is_trivia())
        .map(|t| t.text().to_string())
        .unwrap_or_default();
    // Recover the lexer payload by re-lexing the leaf text.
    let tok = lex_source(&text).ok().and_then(|ts| ts.into_iter().next());
    match tok.map(|t| t.kind) {
        Some(TokenKind::Int(v)) => Expr::Int(v, span),
        Some(TokenKind::Float(v)) => Expr::Float(v, span),
        Some(TokenKind::Str(parts)) => Expr::Str(StringLit { parts }, span),
        Some(TokenKind::RawStr(s)) => Expr::Str(
            StringLit {
                parts: vec![garnet_parser::token::StrPart::Lit(s)],
            },
            span,
        ),
        Some(TokenKind::Symbol(s)) => Expr::Symbol(s, span),
        Some(TokenKind::KwTrue) => Expr::Bool(true, span),
        Some(TokenKind::KwFalse) => Expr::Bool(false, span),
        _ => Expr::Nil(span),
    }
}

fn lower_name_ref(node: &SyntaxNode, span: Span) -> Expr {
    let text = name_text(node);
    Expr::Ident(text, span)
}

fn lower_path(node: &SyntaxNode, span: Span) -> Expr {
    // Flatten `a :: b :: c` (nested PathExpr) into a single Path.
    let mut segs = Vec::new();
    collect_path(node, &mut segs);
    if segs.len() >= 2 {
        Expr::Path(segs, span)
    } else if let Some(s) = segs.into_iter().next() {
        Expr::Ident(s, span)
    } else {
        Expr::Nil(span)
    }
}

fn collect_path(node: &SyntaxNode, segs: &mut Vec<String>) {
    // Left child is either a NameRef (base) or a nested PathExpr.
    for ch in node.children() {
        match ch.kind() {
            PathExpr => collect_path(&ch, segs),
            NameRef => segs.push(name_text(&ch)),
            _ => {}
        }
    }
    // Trailing `:: segment` token is stored directly under PathExpr. S22 lets a
    // few reserved words appear only as qualified expression path segments.
    for e in node.children_with_tokens() {
        if let Some(t) = e.into_token() {
            if is_expression_path_segment_token(t.kind()) {
                segs.push(t.text().to_string());
            }
        }
    }
}

fn is_expression_path_segment_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        Ident | MemoryKw | WorkingKw | EpisodicKw | SemanticKw | ProceduralKw | SpawnKw | MatchKw
    )
}

fn lower_args(node: &SyntaxNode) -> Vec<Expr> {
    let mut args: Vec<Expr> = child(node, ArgList)
        .map(|l| child_exprs(&l).iter().map(lower_expr).collect())
        .unwrap_or_default();
    // A trailing `do…end` block becomes a final closure argument (AST behavior).
    if let Some(blk) = child(node, BlockArg) {
        args.push(lower_block_arg(&blk));
    }
    args
}

fn lower_block_arg(node: &SyntaxNode) -> Expr {
    Expr::Closure {
        params: lower_params(node),
        return_ty: None,
        body: Box::new(ClosureBody::Block(
            // The do/end body statements live directly under BlockArg.
            lower_block_like(node),
        )),
        is_do_block: true,
        span: span_of(node),
    }
}

fn lower_block_like(node: &SyntaxNode) -> Block {
    let stmt_nodes: Vec<SyntaxNode> = node.children().filter(|n| is_stmt_node(n.kind())).collect();
    let mut stmts = Vec::new();
    let mut tail_expr = None;
    let last = stmt_nodes.len().saturating_sub(1);
    for (i, n) in stmt_nodes.iter().enumerate() {
        if i == last && n.kind() == ExprStmt {
            tail_expr = first_expr(n).map(Box::new);
        } else if let Some(s) = lower_stmt(n) {
            stmts.push(s);
        }
    }
    Block {
        stmts,
        tail_expr,
        span: span_of(node),
    }
}

fn lower_closure(node: &SyntaxNode, span: Span) -> Expr {
    let body = if let Some(b) = child(node, SyntaxKind::Block) {
        ClosureBody::Block(lower_block(&b))
    } else {
        ClosureBody::Expr(first_expr(node).unwrap_or(Expr::Nil(span)))
    };
    Expr::Closure {
        params: lower_params(node),
        return_ty: node
            .children()
            .find(|n| is_type_kind(n.kind()))
            .map(|n| Box::new(lower_type(&n))),
        body: Box::new(body),
        is_do_block: false,
        span,
    }
}

fn lower_binary(node: &SyntaxNode, span: Span) -> Expr {
    let exprs = child_exprs(node);
    let lhs = exprs.first().map(lower_expr).unwrap_or(Expr::Nil(span));
    let rhs = exprs.get(1).map(lower_expr).unwrap_or(Expr::Nil(span));
    let op = bin_op(node);
    Expr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        span,
    }
}

fn bin_op(node: &SyntaxNode) -> BinOp {
    for e in node.children_with_tokens() {
        if let Some(t) = e.into_token() {
            match t.kind() {
                Plus => return BinOp::Add,
                Minus => return BinOp::Sub,
                Star => return BinOp::Mul,
                Slash => return BinOp::Div,
                Percent => return BinOp::Mod,
                EqEq => return BinOp::Eq,
                BangEq => return BinOp::NotEq,
                Lt => return BinOp::Lt,
                Gt => return BinOp::Gt,
                LtEq => return BinOp::LtEq,
                GtEq => return BinOp::GtEq,
                AndKw => return BinOp::And,
                OrKw => return BinOp::Or,
                PipeGt => return BinOp::Pipeline,
                DotDot => return BinOp::Range,
                DotDotDot => return BinOp::RangeInclusive,
                _ => {}
            }
        }
    }
    BinOp::Add
}

fn lower_unary(node: &SyntaxNode, span: Span) -> Expr {
    let op = if has_token(node, NotKw) || has_token(node, Bang) {
        UnOp::Not
    } else if has_token(node, Question) {
        UnOp::Question
    } else {
        UnOp::Neg
    };
    Expr::Unary {
        op,
        expr: Box::new(first_expr(node).unwrap_or(Expr::Nil(span))),
        span,
    }
}

fn lower_if(node: &SyntaxNode, span: Span) -> Expr {
    // Children sequence: cond-expr, then-block, [elsif: cond-expr, block]*, [else block]
    let exprs = child_exprs(node);
    let blocks: Vec<SyntaxNode> = node
        .children()
        .filter(|n| n.kind() == SyntaxKind::Block)
        .collect();
    let condition = exprs.first().map(lower_expr).unwrap_or(Expr::Nil(span));
    let then_block = blocks
        .first()
        .map(lower_block)
        .unwrap_or_else(|| empty_block(span));

    let has_else = has_token(node, ElseKw);
    let mut elsif_clauses = Vec::new();
    // exprs[1..] pair with blocks[1..], minus the trailing else block if present.
    let elsif_count = exprs.len().saturating_sub(1);
    for i in 0..elsif_count {
        let cond = lower_expr(&node_nth_expr(&exprs, i + 1));
        if let Some(b) = blocks.get(i + 1) {
            elsif_clauses.push((cond, lower_block(b)));
        }
    }
    let else_block = if has_else {
        blocks
            .last()
            .filter(|_| blocks.len() > 1 + elsif_count)
            .map(lower_block)
    } else {
        None
    };
    Expr::If {
        condition: Box::new(condition),
        then_block,
        elsif_clauses,
        else_block,
        span,
    }
}

fn node_nth_expr(exprs: &[SyntaxNode], i: usize) -> SyntaxNode {
    exprs[i].clone()
}

fn lower_match(node: &SyntaxNode, span: Span) -> Expr {
    let subject = first_expr(node).unwrap_or(Expr::Nil(span));
    let arms = node
        .children()
        .filter(|n| n.kind() == SyntaxKind::MatchArm)
        .map(|arm| {
            let pat = arm
                .children()
                .find(|n| is_pat_node(n.kind()))
                .map(|p| lower_pattern(&p))
                .unwrap_or(Pattern::Wildcard(span_of(&arm)));
            let guard = child(&arm, MatchGuard).and_then(|g| first_expr(&g));
            let body = if let Some(b) = child(&arm, SyntaxKind::Block) {
                lower_block(&b)
            } else if let Some(e) = arm.children().find(|n| is_expr_kind(n.kind())) {
                let s = span_of(&e);
                Block {
                    stmts: Vec::new(),
                    tail_expr: Some(Box::new(lower_expr(&e))),
                    span: s,
                }
            } else {
                empty_block(span_of(&arm))
            };
            MatchArm {
                pattern: pat,
                guard,
                body,
                span: span_of(&arm),
            }
        })
        .collect();
    Expr::Match {
        subject: Box::new(subject),
        arms,
        span,
    }
}

fn lower_try(node: &SyntaxNode, span: Span) -> Expr {
    let body = child(node, SyntaxKind::Block)
        .map(|b| lower_block(&b))
        .unwrap_or_else(|| empty_block(span));
    let rescues = node
        .children()
        .filter(|n| n.kind() == SyntaxKind::RescueClause)
        .map(|r| RescueClause {
            name: child(&r, Name).map(|n| name_text(&n)),
            ty: first_type(&r),
            body: child(&r, SyntaxKind::Block)
                .map(|b| lower_block(&b))
                .unwrap_or_else(|| empty_block(span_of(&r))),
            span: span_of(&r),
        })
        .collect();
    let ensure = child(node, EnsureClause)
        .and_then(|e| child(&e, SyntaxKind::Block))
        .map(|b| lower_block(&b));
    Expr::Try {
        body,
        rescues,
        ensure,
        span,
    }
}

// ── patterns ──────────────────────────────────────────────────────────────

fn is_pat_node(k: SyntaxKind) -> bool {
    matches!(
        k,
        LiteralPat | IdentPat | TuplePat | EnumPat | WildcardPat | RestPat
    )
}

fn lower_pattern(node: &SyntaxNode) -> Pattern {
    let span = span_of(node);
    match node.kind() {
        WildcardPat => Pattern::Wildcard(span),
        RestPat => Pattern::Rest(span),
        LiteralPat => Pattern::Literal(lower_literal(node, span), span),
        IdentPat => Pattern::Ident(name_token_text(node), span),
        TuplePat => Pattern::Tuple(
            node.children()
                .filter(|n| is_pat_node(n.kind()))
                .map(|p| lower_pattern(&p))
                .collect(),
            span,
        ),
        EnumPat => {
            let path: Vec<String> = node
                .children_with_tokens()
                .filter_map(|e| e.into_token())
                .filter(|t| t.kind() == Ident)
                .map(|t| t.text().to_string())
                .collect();
            let sub = node
                .children()
                .filter(|n| is_pat_node(n.kind()))
                .map(|p| lower_pattern(&p))
                .collect();
            Pattern::Enum(path, sub, span)
        }
        _ => Pattern::Wildcard(span),
    }
}

fn name_token_text(node: &SyntaxNode) -> String {
    node.children_with_tokens()
        .filter_map(|e| e.into_token())
        .find(|t| t.kind() == Ident)
        .map(|t| t.text().to_string())
        .unwrap_or_default()
}
