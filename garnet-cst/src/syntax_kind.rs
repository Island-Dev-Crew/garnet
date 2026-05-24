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

use garnet_parser::token::TokenKind;

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
    AssignStmt,
    ReturnStmt,
    BreakStmt,
    ContinueStmt,
    WhileStmt,
    ForStmt,
    LoopStmt,
    YieldStmt,
    NextStmt,
    RaiseStmt,

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
    CastExpr,
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

impl SyntaxKind {
    /// Map a lexer `TokenKind` to its leaf `SyntaxKind`. Total over all token
    /// kinds; payloads are ignored (the CST stores source text on each leaf).
    #[must_use]
    pub fn from_token(kind: &TokenKind) -> SyntaxKind {
        match kind {
            TokenKind::Int(_) => SyntaxKind::IntLit,
            TokenKind::Float(_) => SyntaxKind::FloatLit,
            TokenKind::Str(_) => SyntaxKind::StrLit,
            TokenKind::RawStr(_) => SyntaxKind::RawStrLit,
            TokenKind::Symbol(_) => SyntaxKind::SymbolLit,
            TokenKind::Ident(_) => SyntaxKind::Ident,
            TokenKind::KwModule => SyntaxKind::ModuleKw,
            TokenKind::KwUse => SyntaxKind::UseKw,
            TokenKind::KwPub => SyntaxKind::PubKw,
            TokenKind::KwDo => SyntaxKind::DoKw,
            TokenKind::KwEnd => SyntaxKind::EndKw,
            TokenKind::KwDef => SyntaxKind::DefKw,
            TokenKind::KwFn => SyntaxKind::FnKw,
            TokenKind::KwLet => SyntaxKind::LetKw,
            TokenKind::KwVar => SyntaxKind::VarKw,
            TokenKind::KwConst => SyntaxKind::ConstKw,
            TokenKind::KwType => SyntaxKind::TypeKw,
            TokenKind::KwTrait => SyntaxKind::TraitKw,
            TokenKind::KwImpl => SyntaxKind::ImplKw,
            TokenKind::KwStruct => SyntaxKind::StructKw,
            TokenKind::KwEnum => SyntaxKind::EnumKw,
            TokenKind::KwMemory => SyntaxKind::MemoryKw,
            TokenKind::KwWorking => SyntaxKind::WorkingKw,
            TokenKind::KwEpisodic => SyntaxKind::EpisodicKw,
            TokenKind::KwSemantic => SyntaxKind::SemanticKw,
            TokenKind::KwProcedural => SyntaxKind::ProceduralKw,
            TokenKind::KwActor => SyntaxKind::ActorKw,
            TokenKind::KwProtocol => SyntaxKind::ProtocolKw,
            TokenKind::KwOn => SyntaxKind::OnKw,
            TokenKind::KwSpawn => SyntaxKind::SpawnKw,
            TokenKind::KwSend => SyntaxKind::SendKw,
            TokenKind::KwIf => SyntaxKind::IfKw,
            TokenKind::KwElsif => SyntaxKind::ElsifKw,
            TokenKind::KwElse => SyntaxKind::ElseKw,
            TokenKind::KwWhile => SyntaxKind::WhileKw,
            TokenKind::KwFor => SyntaxKind::ForKw,
            TokenKind::KwIn => SyntaxKind::InKw,
            TokenKind::KwLoop => SyntaxKind::LoopKw,
            TokenKind::KwBreak => SyntaxKind::BreakKw,
            TokenKind::KwContinue => SyntaxKind::ContinueKw,
            TokenKind::KwReturn => SyntaxKind::ReturnKw,
            TokenKind::KwYield => SyntaxKind::YieldKw,
            TokenKind::KwNext => SyntaxKind::NextKw,
            TokenKind::KwMatch => SyntaxKind::MatchKw,
            TokenKind::KwWhen => SyntaxKind::WhenKw,
            TokenKind::KwTry => SyntaxKind::TryKw,
            TokenKind::KwRescue => SyntaxKind::RescueKw,
            TokenKind::KwEnsure => SyntaxKind::EnsureKw,
            TokenKind::KwRaise => SyntaxKind::RaiseKw,
            TokenKind::KwOwn => SyntaxKind::OwnKw,
            TokenKind::KwBorrow => SyntaxKind::BorrowKw,
            TokenKind::KwRef => SyntaxKind::RefKw,
            TokenKind::KwMut => SyntaxKind::MutKw,
            TokenKind::KwMove => SyntaxKind::MoveKw,
            TokenKind::KwDyn => SyntaxKind::DynKw,
            TokenKind::KwAs => SyntaxKind::AsKw,
            TokenKind::KwAnd => SyntaxKind::AndKw,
            TokenKind::KwOr => SyntaxKind::OrKw,
            TokenKind::KwNot => SyntaxKind::NotKw,
            TokenKind::KwTrue => SyntaxKind::TrueKw,
            TokenKind::KwFalse => SyntaxKind::FalseKw,
            TokenKind::KwNil => SyntaxKind::NilKw,
            TokenKind::KwSelf_ => SyntaxKind::SelfKw,
            TokenKind::KwSuper => SyntaxKind::SuperKw,
            TokenKind::Plus => SyntaxKind::Plus,
            TokenKind::Minus => SyntaxKind::Minus,
            TokenKind::Star => SyntaxKind::Star,
            TokenKind::Slash => SyntaxKind::Slash,
            TokenKind::Percent => SyntaxKind::Percent,
            TokenKind::Eq => SyntaxKind::Eq,
            TokenKind::EqEq => SyntaxKind::EqEq,
            TokenKind::BangEq => SyntaxKind::BangEq,
            TokenKind::Lt => SyntaxKind::Lt,
            TokenKind::Gt => SyntaxKind::Gt,
            TokenKind::LtEq => SyntaxKind::LtEq,
            TokenKind::GtEq => SyntaxKind::GtEq,
            TokenKind::Bang => SyntaxKind::Bang,
            TokenKind::Question => SyntaxKind::Question,
            TokenKind::PipeGt => SyntaxKind::PipeGt,
            TokenKind::Pipe => SyntaxKind::Pipe,
            TokenKind::DotDot => SyntaxKind::DotDot,
            TokenKind::DotDotDot => SyntaxKind::DotDotDot,
            TokenKind::FatArrow => SyntaxKind::FatArrow,
            TokenKind::Arrow => SyntaxKind::Arrow,
            TokenKind::PlusEq => SyntaxKind::PlusEq,
            TokenKind::MinusEq => SyntaxKind::MinusEq,
            TokenKind::StarEq => SyntaxKind::StarEq,
            TokenKind::SlashEq => SyntaxKind::SlashEq,
            TokenKind::PercentEq => SyntaxKind::PercentEq,
            TokenKind::Amp => SyntaxKind::Amp,
            TokenKind::At => SyntaxKind::At,
            TokenKind::Dot => SyntaxKind::Dot,
            TokenKind::Comma => SyntaxKind::Comma,
            TokenKind::Colon => SyntaxKind::Colon,
            TokenKind::ColonCol => SyntaxKind::ColonColon,
            TokenKind::Semi => SyntaxKind::Semi,
            TokenKind::LParen => SyntaxKind::LParen,
            TokenKind::RParen => SyntaxKind::RParen,
            TokenKind::LBrace => SyntaxKind::LBrace,
            TokenKind::RBrace => SyntaxKind::RBrace,
            TokenKind::LBracket => SyntaxKind::LBracket,
            TokenKind::RBracket => SyntaxKind::RBracket,
            TokenKind::Newline => SyntaxKind::Newline,
            TokenKind::Eof => SyntaxKind::Eof,
            TokenKind::Whitespace(_) => SyntaxKind::Whitespace,
            TokenKind::Comment(_) => SyntaxKind::Comment,
        }
    }
}

/// A Garnet CST node (an interior node of the syntax tree).
pub type SyntaxNode = rowan::SyntaxNode<GarnetLanguage>;
/// A Garnet CST token (a leaf of the syntax tree, including trivia).
pub type SyntaxToken = rowan::SyntaxToken<GarnetLanguage>;
/// Either a node or a token.
pub type SyntaxElement = rowan::SyntaxElement<GarnetLanguage>;
