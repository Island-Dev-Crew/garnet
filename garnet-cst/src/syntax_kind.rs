//! Syntax kinds for the Garnet CST and the rowan `Language` binding.
//!
//! Built cold from Mini-Spec v1.0 §2–§11 for the v0.7 build-both-then-compare
//! A/B (S15). Token kinds map 1:1 from `garnet_parser::token::TokenKind`;
//! composite node kinds follow the Mini-Spec grammar productions.
//!
//! The `SyntaxKind` enum is the stable, full language definition published in
//! PR-1 so S16 (LSP precision) never has to chase enum churn. PR-2's builder
//! populates these node kinds; PR-2 may add node kinds additively without
//! breaking the trait surface.

/// Define `SyntaxKind` together with safe `u16` <-> enum conversions.
///
/// We avoid the usual `mem::transmute` round-trip (the workspace forbids new
/// ambient `unsafe`): because the enum is `#[repr(u16)]` with implicit
/// discriminants `0..N` in declaration order, the `ALL` slice is indexed by
/// discriminant, so `ALL[raw]` recovers the variant whose discriminant is
/// `raw`.
macro_rules! syntax_kinds {
    ($($kind:ident,)+) => {
        /// Every token and composite-node kind in the Garnet CST.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(u16)]
        pub enum SyntaxKind {
            $($kind,)+
        }

        impl SyntaxKind {
            /// All kinds in declaration order; index equals the `u16` discriminant.
            const ALL: &'static [SyntaxKind] = &[$(SyntaxKind::$kind,)+];

            /// Total number of distinct kinds.
            pub const COUNT: u16 = SyntaxKind::ALL.len() as u16;

            /// Recover a kind from its raw `u16` discriminant, or `None` if out of range.
            #[must_use]
            pub fn from_u16(raw: u16) -> Option<SyntaxKind> {
                SyntaxKind::ALL.get(raw as usize).copied()
            }

            /// The raw `u16` discriminant for this kind.
            #[must_use]
            pub fn to_u16(self) -> u16 {
                self as u16
            }

            /// True if this kind is a trivia token (whitespace or comment).
            #[must_use]
            pub fn is_trivia(self) -> bool {
                matches!(self, SyntaxKind::Whitespace | SyntaxKind::Comment)
            }
        }
    };
}

syntax_kinds! {
    // ── Trivia & structural tokens ──
    Whitespace,
    Comment,
    Newline,

    // ── Literal tokens ──
    IntLit,
    FloatLit,
    StrLit,
    RawStrLit,
    SymbolLit,
    Ident,

    // ── Keyword tokens (mode & structure) ──
    ModuleKw,
    UseKw,
    PubKw,
    DoKw,
    EndKw,

    // ── Keyword tokens (declarations) ──
    DefKw,
    FnKw,
    LetKw,
    VarKw,
    ConstKw,
    TypeKw,
    TraitKw,
    ImplKw,
    StructKw,
    EnumKw,

    // ── Keyword tokens (memory & actors) ──
    MemoryKw,
    WorkingKw,
    EpisodicKw,
    SemanticKw,
    ProceduralKw,
    ActorKw,
    ProtocolKw,
    OnKw,
    SpawnKw,
    SendKw,

    // ── Keyword tokens (control flow) ──
    IfKw,
    ElsifKw,
    ElseKw,
    WhileKw,
    ForKw,
    InKw,
    LoopKw,
    BreakKw,
    ContinueKw,
    ReturnKw,
    YieldKw,
    NextKw,
    MatchKw,
    WhenKw,

    // ── Keyword tokens (error handling) ──
    TryKw,
    RescueKw,
    EnsureKw,
    RaiseKw,

    // ── Keyword tokens (ownership) ──
    OwnKw,
    BorrowKw,
    RefKw,
    MutKw,
    MoveKw,
    DynKw,
    AsKw,

    // ── Keyword tokens (logical) ──
    AndKw,
    OrKw,
    NotKw,

    // ── Keyword tokens (literal values) ──
    TrueKw,
    FalseKw,
    NilKw,
    SelfKw,
    SuperKw,

    // ── Operator & punctuation tokens ──
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    EqEq,
    BangEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    Bang,
    Question,
    PipeGt,
    Pipe,
    DotDot,
    DotDotDot,
    FatArrow,
    Arrow,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    Amp,
    At,
    Dot,
    Comma,
    Colon,
    ColonColon,
    Semi,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Eof,

    // ── Composite nodes: root & top level ──
    Root,
    Module,
    UseDecl,
    UsePath,
    UseGroup,

    // ── Composite nodes: items ──
    FnDef,
    SafeFnDef,
    ParamList,
    Param,
    StructDef,
    FieldList,
    FieldDecl,
    EnumDef,
    VariantList,
    Variant,
    TraitDef,
    TraitItemList,
    FnSig,
    ImplBlock,
    TypeAliasDef,
    ConstDef,
    MemoryDecl,
    ActorDef,
    ActorItemList,
    ProtocolDecl,
    HandlerDecl,
    StructProtocolDef,

    // ── Composite nodes: attributes, generics, visibility ──
    AttrList,
    Attr,
    AttrArgs,
    TypeParamList,
    TypeParam,
    Visibility,

    // ── Composite nodes: types ──
    TypeRef,
    TypeArgList,
    FnType,
    TupleType,

    // ── Composite nodes: blocks & statements ──
    Block,
    LetStmt,
    VarStmt,
    ExprStmt,
    ReturnStmt,
    BreakStmt,
    ContinueStmt,
    WhileStmt,
    ForStmt,
    LoopStmt,

    // ── Composite nodes: expressions ──
    IfExpr,
    MatchExpr,
    MatchArmList,
    MatchArm,
    MatchGuard,
    TryExpr,
    RescueClause,
    EnsureClause,
    BinaryExpr,
    UnaryExpr,
    AssignExpr,
    RangeExpr,
    PipelineExpr,
    CallExpr,
    MethodCallExpr,
    FieldExpr,
    IndexExpr,
    PathExpr,
    ParenExpr,
    SpawnExpr,
    SendExpr,
    YieldExpr,
    NextExpr,
    RaiseExpr,
    ClosureExpr,
    ClosureParamList,
    ArgList,
    Arg,
    BlockArg,
    ArrayLit,
    MapLit,
    MapEntry,
    TupleLit,
    Literal,
    NameRef,
    Name,

    // ── Composite nodes: patterns ──
    LiteralPat,
    IdentPat,
    TuplePat,
    EnumPat,
    WildcardPat,
    RestPat,
    PatList,

    // ── Catch-all ──
    Error,
}

/// The rowan `Language` binding for Garnet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GarnetLanguage {}

impl rowan::Language for GarnetLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> SyntaxKind {
        // `raw` always originates from `kind_to_raw`, so it is in range; the
        // fallback keeps this total without `unsafe`.
        SyntaxKind::from_u16(raw.0).unwrap_or(SyntaxKind::Error)
    }

    fn kind_to_raw(kind: SyntaxKind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind.to_u16())
    }
}

/// A Garnet CST node (an interior node of the syntax tree).
pub type SyntaxNode = rowan::SyntaxNode<GarnetLanguage>;
/// A Garnet CST token (a leaf of the syntax tree, including trivia).
pub type SyntaxToken = rowan::SyntaxToken<GarnetLanguage>;
/// Either a node or a token.
pub type SyntaxElement = rowan::SyntaxElement<GarnetLanguage>;
