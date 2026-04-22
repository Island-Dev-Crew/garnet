//! Garnet v0.3 Abstract Syntax Tree — all node types for 90 EBNF productions.
//! Every node carries a Span for diagnostic source-location tracking.

use crate::token::{Span, StrPart};

// ════════════════════════════════════════════════════════════════════
// Top-level
// ════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct Module {
    pub safe: bool,
    pub items: Vec<Item>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Item {
    Use(UseDecl),
    Module(ModuleDecl),
    Memory(MemoryDecl),
    Actor(ActorDef),
    Struct(StructDef),
    Enum(EnumDef),
    Trait(TraitDef),
    Impl(ImplBlock),
    Fn(FnDef),
    Const(ConstDecl),
    Let(LetDecl),
}

// ════════════════════════════════════════════════════════════════════
// Modules & imports (Mini-Spec §3)
// ════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct ModuleDecl {
    pub safe: bool,
    pub public: bool,
    pub name: String,
    pub items: Vec<Item>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct UseDecl {
    pub path: Vec<String>,
    pub imports: UseImports,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum UseImports {
    Module,                  // use Foo::Bar
    Named(Vec<String>),      // use Foo::Bar::{A, B}
    Glob,                    // use Foo::Bar::*
}

// ════════════════════════════════════════════════════════════════════
// Memory units (Mini-Spec §4)
// ════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct MemoryDecl {
    pub kind: MemoryKind,
    pub name: String,
    pub store: TypeExpr,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryKind {
    Working,
    Episodic,
    Semantic,
    Procedural,
}

// ════════════════════════════════════════════════════════════════════
// Type expressions (Mini-Spec §11)
// ════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub enum TypeExpr {
    /// Simple or generic type: `Int`, `Array<String>`, `Foo::Bar<T>`
    Named {
        path: Vec<String>,
        args: Vec<TypeExpr>,
        span: Span,
    },
    /// Function type: `(Int, String) -> Bool`
    Fn {
        params: Vec<TypeExpr>,
        ret: Box<TypeExpr>,
        span: Span,
    },
    /// Tuple type: `(Int, String)`
    Tuple {
        elements: Vec<TypeExpr>,
        span: Span,
    },
    /// Reference type: `&T` or `&mut T`
    Ref {
        mutable: bool,
        inner: Box<TypeExpr>,
        span: Span,
    },
}

impl TypeExpr {
    pub fn span(&self) -> Span {
        match self {
            TypeExpr::Named { span, .. }
            | TypeExpr::Fn { span, .. }
            | TypeExpr::Tuple { span, .. }
            | TypeExpr::Ref { span, .. } => *span,
        }
    }
}

// ════════════════════════════════════════════════════════════════════
// Functions (Mini-Spec §5)
// ════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FnMode {
    Managed, // def
    Safe,    // fn
}

#[derive(Debug, Clone)]
pub struct FnDef {
    pub annotations: Vec<Annotation>,
    pub public: bool,
    pub mode: FnMode,
    pub name: String,
    pub type_params: Vec<String>,
    pub params: Vec<Param>,
    pub return_ty: Option<TypeExpr>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub ownership: Option<Ownership>,
    pub name: String,
    pub ty: Option<TypeExpr>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ownership {
    Own,
    Borrow,
    Ref,
    Mut,
}

#[derive(Debug, Clone)]
pub enum Annotation {
    MaxDepth(i64, Span),
    FanOut(i64, Span),
    RequireMetadata(Span),
    Safe(Span),
    Dynamic(Span),
    /// Capability annotation (v3.4 CapCaps / Security Layer 2).
    /// `@caps(fs, net)` declares the OS authority a function needs.
    /// Empty list `@caps()` declares "no caps required" — purely
    /// computational. Wildcard `@caps(*)` is debug-only; CI rejects.
    Caps(Vec<Capability>, Span),
    /// Mailbox capacity for an actor (v3.4 BoundedMail).
    /// `@mailbox(N)` overrides the default 1024-message cap on send.
    Mailbox(i64, Span),
    /// Mark a type as not transferable across actor boundaries
    /// (v3.4 Sendable / Mini-Spec v1.0 §9.4.3).
    NonSendable(Span),
}

/// A capability tag carried by `@caps(...)` annotations.
///
/// The standard set (v3.4 Security Layer 2 spec §1.6) is enumerated
/// below. Unknown identifiers parse successfully and surface as
/// `Other(name)` so user-defined caps remain extensible — the checker
/// is responsible for validating the canonical set is used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Capability {
    /// File-system read + write
    Fs,
    /// TCP/UDP outbound + listen against public addresses
    Net,
    /// TCP/UDP against RFC1918/loopback (NetDefaults bypass)
    NetInternal,
    /// Wall clock + sleep
    Time,
    /// Process spawn, signals
    Proc,
    /// `extern "C"` calls
    Ffi,
    /// Wildcard — DEBUG ONLY; CI must reject in release
    Wildcard,
    /// User-defined or unrecognised capability name
    Other(String),
}

impl Capability {
    pub fn from_ident(s: &str) -> Self {
        match s {
            "fs" => Capability::Fs,
            "net" => Capability::Net,
            "net_internal" => Capability::NetInternal,
            "time" => Capability::Time,
            "proc" => Capability::Proc,
            "ffi" => Capability::Ffi,
            "*" => Capability::Wildcard,
            other => Capability::Other(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Capability::Fs => "fs",
            Capability::Net => "net",
            Capability::NetInternal => "net_internal",
            Capability::Time => "time",
            Capability::Proc => "proc",
            Capability::Ffi => "ffi",
            Capability::Wildcard => "*",
            Capability::Other(s) => s,
        }
    }
}

// ════════════════════════════════════════════════════════════════════
// Actors (Mini-Spec §9)
// ════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct ActorDef {
    pub public: bool,
    pub name: String,
    pub items: Vec<ActorItem>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ActorItem {
    Protocol(ProtocolDecl),
    Handler(HandlerDecl),
    Memory(MemoryDecl),
    Let(LetDecl),
}

#[derive(Debug, Clone)]
pub struct ProtocolDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_ty: Option<TypeExpr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HandlerDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Block,
    pub span: Span,
}

// ════════════════════════════════════════════════════════════════════
// User-defined types (Mini-Spec §11.3)
// ════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct StructDef {
    pub public: bool,
    pub name: String,
    pub type_params: Vec<String>,
    pub fields: Vec<FieldDef>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub public: bool,
    pub name: String,
    pub ty: TypeExpr,
    pub default: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumDef {
    pub public: bool,
    pub name: String,
    pub type_params: Vec<String>,
    pub variants: Vec<Variant>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub name: String,
    pub fields: Vec<TypeExpr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TraitDef {
    pub public: bool,
    pub name: String,
    pub type_params: Vec<String>,
    pub items: Vec<TraitItem>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TraitItem {
    FnSig(FnSig),
    Const(ConstDecl),
}

#[derive(Debug, Clone)]
pub struct FnSig {
    pub mode: FnMode,
    pub name: String,
    pub params: Vec<Param>,
    pub return_ty: Option<TypeExpr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub type_params: Vec<String>,
    pub target: TypeExpr,
    pub trait_ty: Option<TypeExpr>,
    pub methods: Vec<FnDef>,
    pub span: Span,
}

// ════════════════════════════════════════════════════════════════════
// Statements (Mini-Spec §6)
// ════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub tail_expr: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(LetDecl),
    Var(VarDecl),
    Const(ConstDecl),
    Assign {
        target: Expr,
        op: AssignOp,
        value: Expr,
        span: Span,
    },
    While {
        condition: Expr,
        body: Block,
        span: Span,
    },
    For {
        var: String,
        iter: Expr,
        body: Block,
        span: Span,
    },
    Loop {
        body: Block,
        span: Span,
    },
    Break {
        value: Option<Expr>,
        span: Span,
    },
    Continue {
        span: Span,
    },
    Return {
        value: Option<Expr>,
        span: Span,
    },
    Raise {
        value: Expr,
        span: Span,
    },
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub struct LetDecl {
    pub mutable: bool,
    pub name: String,
    pub ty: Option<TypeExpr>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: String,
    pub ty: Option<TypeExpr>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ConstDecl {
    pub public: bool,
    pub name: String,
    pub ty: Option<TypeExpr>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Eq,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
}

// ════════════════════════════════════════════════════════════════════
// Expressions (Mini-Spec §§5–7)
// ════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64, Span),
    Float(f64, Span),
    Bool(bool, Span),
    Nil(Span),
    Str(StringLit, Span),
    Symbol(String, Span),
    Ident(String, Span),
    Path(Vec<String>, Span),

    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        span: Span,
    },
    Unary {
        op: UnOp,
        expr: Box<Expr>,
        span: Span,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    Method {
        receiver: Box<Expr>,
        method: String,
        args: Vec<Expr>,
        span: Span,
    },
    Field {
        receiver: Box<Expr>,
        field: String,
        span: Span,
    },
    Index {
        receiver: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },

    If {
        condition: Box<Expr>,
        then_block: Block,
        elsif_clauses: Vec<(Expr, Block)>,
        else_block: Option<Block>,
        span: Span,
    },
    Match {
        subject: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    Try {
        body: Block,
        rescues: Vec<RescueClause>,
        ensure: Option<Block>,
        span: Span,
    },

    Closure {
        params: Vec<Param>,
        return_ty: Option<Box<TypeExpr>>,
        body: Box<ClosureBody>,
        span: Span,
    },
    Spawn {
        expr: Box<Expr>,
        span: Span,
    },
    Array {
        elements: Vec<Expr>,
        span: Span,
    },
    Map {
        entries: Vec<(Expr, Expr)>,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub enum ClosureBody {
    Block(Block),
    Expr(Expr),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Int(_, s) | Expr::Float(_, s) | Expr::Bool(_, s)
            | Expr::Nil(s) | Expr::Str(_, s) | Expr::Symbol(_, s)
            | Expr::Ident(_, s) | Expr::Path(_, s) => *s,
            Expr::Binary { span, .. } | Expr::Unary { span, .. }
            | Expr::Call { span, .. } | Expr::Method { span, .. }
            | Expr::Field { span, .. } | Expr::Index { span, .. }
            | Expr::If { span, .. } | Expr::Match { span, .. }
            | Expr::Try { span, .. } | Expr::Closure { span, .. }
            | Expr::Spawn { span, .. } | Expr::Array { span, .. }
            | Expr::Map { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
    Pipeline,
    Range,
    RangeInclusive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
    Question, // postfix ?
}

#[derive(Debug, Clone)]
pub struct StringLit {
    pub parts: Vec<StrPart>,
}

// ════════════════════════════════════════════════════════════════════
// Pattern matching (Mini-Spec §6.3)
// ════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(Expr, Span),          // 42, "hello", :ok, true, nil
    Ident(String, Span),          // x (binds the value)
    Tuple(Vec<Pattern>, Span),    // (a, b, c)
    Enum(Vec<String>, Vec<Pattern>, Span),  // Ok(value), Err(e)
    Wildcard(Span),               // _
    Rest(Span),                   // ..
}

impl Pattern {
    pub fn span(&self) -> Span {
        match self {
            Pattern::Literal(_, s) | Pattern::Ident(_, s)
            | Pattern::Tuple(_, s) | Pattern::Enum(_, _, s)
            | Pattern::Wildcard(s) | Pattern::Rest(s) => *s,
        }
    }
}

// ════════════════════════════════════════════════════════════════════
// Error handling (Mini-Spec §7)
// ════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct RescueClause {
    pub name: Option<String>,
    pub ty: Option<TypeExpr>,
    pub body: Block,
    pub span: Span,
}
