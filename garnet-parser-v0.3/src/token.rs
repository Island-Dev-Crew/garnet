//! Garnet v0.3 token definitions.
//! Every token carries a Span for diagnostic source-location tracking.

/// A byte-offset range in the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub len: usize,
}

impl Span {
    pub fn new(start: usize, len: usize) -> Self {
        Self { start, len }
    }

    pub fn end(&self) -> usize {
        self.start + self.len
    }

    /// Join two spans into one covering both.
    pub fn join(self, other: Span) -> Span {
        let start = self.start.min(other.start);
        let end = self.end().max(other.end());
        Span {
            start,
            len: end - start,
        }
    }
}

impl From<Span> for miette::SourceSpan {
    fn from(s: Span) -> miette::SourceSpan {
        (s.start, s.len).into()
    }
}

/// A single token produced by the lexer.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

/// Part of an interpolated string.
#[derive(Debug, Clone, PartialEq)]
pub enum StrPart {
    Lit(String),
    Interp(String), // source text inside #{...}, to be re-lexed/re-parsed
}

/// All token kinds in the Garnet v0.3 lexical grammar.
#[derive(Debug, Clone)]
pub enum TokenKind {
    // ── Literals ──
    Int(i64),
    Float(f64),
    Str(Vec<StrPart>),
    RawStr(String),
    Symbol(String), // :name
    Ident(String),

    // ── Mode & structure keywords ──
    KwModule,
    KwUse,
    KwPub,
    KwEnd,

    // ── Declaration keywords ──
    KwDef,
    KwFn,
    KwLet,
    KwVar,
    KwConst,
    KwType,
    KwTrait,
    KwImpl,
    KwStruct,
    KwEnum,

    // ── Memory & actor keywords ──
    KwMemory,
    KwWorking,
    KwEpisodic,
    KwSemantic,
    KwProcedural,
    KwActor,
    KwProtocol,
    KwOn,
    KwSpawn,
    KwSend,

    // ── Control flow keywords ──
    KwIf,
    KwElsif,
    KwElse,
    KwWhile,
    KwFor,
    KwIn,
    KwLoop,
    KwBreak,
    KwContinue,
    KwReturn,
    KwMatch,
    KwWhen,

    // ── Error handling keywords ──
    KwTry,
    KwRescue,
    KwEnsure,
    KwRaise,

    // ── Ownership keywords (safe mode) ──
    KwOwn,
    KwBorrow,
    KwRef,
    KwMut,
    KwMove,

    // ── Logical keywords ──
    KwAnd,
    KwOr,
    KwNot,

    // ── Literal keywords ──
    KwTrue,
    KwFalse,
    KwNil,
    KwSelf_,
    KwSuper,

    // ── Operators ──
    Plus,      // +
    Minus,     // -
    Star,      // *
    Slash,     // /
    Percent,   // %
    Eq,        // =
    EqEq,      // ==
    BangEq,    // !=
    Lt,        // <
    Gt,        // >
    LtEq,      // <=
    GtEq,      // >=
    Bang,      // !
    Question,  // ?
    PipeGt,    // |>
    Pipe,      // |
    DotDot,    // ..
    DotDotDot, // ...
    FatArrow,  // =>
    Arrow,     // ->
    PlusEq,    // +=
    MinusEq,   // -=
    StarEq,    // *=
    SlashEq,   // /=
    PercentEq, // %=
    Amp,       // &
    At,        // @

    // ── Punctuation ──
    Dot,      // .
    Comma,    // ,
    Colon,    // :
    ColonCol, // ::
    Semi,     // ;
    LParen,   // (
    RParen,   // )
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]

    // ── Structural ──
    Newline,
    Eof,
}

impl TokenKind {
    /// Check if this token kind matches another for expect/eat operations.
    /// Ignores payload values — only compares discriminants.
    pub fn matches(&self, other: &TokenKind) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

/// Map a keyword string to its TokenKind, or None if not a keyword.
pub fn keyword_lookup(s: &str) -> Option<TokenKind> {
    match s {
        "module" => Some(TokenKind::KwModule),
        "use" => Some(TokenKind::KwUse),
        "pub" => Some(TokenKind::KwPub),
        "end" => Some(TokenKind::KwEnd),
        "def" => Some(TokenKind::KwDef),
        "fn" => Some(TokenKind::KwFn),
        "let" => Some(TokenKind::KwLet),
        "var" => Some(TokenKind::KwVar),
        "const" => Some(TokenKind::KwConst),
        "type" => Some(TokenKind::KwType),
        "trait" => Some(TokenKind::KwTrait),
        "impl" => Some(TokenKind::KwImpl),
        "struct" => Some(TokenKind::KwStruct),
        "enum" => Some(TokenKind::KwEnum),
        "memory" => Some(TokenKind::KwMemory),
        "working" => Some(TokenKind::KwWorking),
        "episodic" => Some(TokenKind::KwEpisodic),
        "semantic" => Some(TokenKind::KwSemantic),
        "procedural" => Some(TokenKind::KwProcedural),
        "actor" => Some(TokenKind::KwActor),
        "protocol" => Some(TokenKind::KwProtocol),
        "on" => Some(TokenKind::KwOn),
        "spawn" => Some(TokenKind::KwSpawn),
        "send" => Some(TokenKind::KwSend),
        "if" => Some(TokenKind::KwIf),
        "elsif" => Some(TokenKind::KwElsif),
        "else" => Some(TokenKind::KwElse),
        "while" => Some(TokenKind::KwWhile),
        "for" => Some(TokenKind::KwFor),
        "in" => Some(TokenKind::KwIn),
        "loop" => Some(TokenKind::KwLoop),
        "break" => Some(TokenKind::KwBreak),
        "continue" => Some(TokenKind::KwContinue),
        "return" => Some(TokenKind::KwReturn),
        "match" => Some(TokenKind::KwMatch),
        "when" => Some(TokenKind::KwWhen),
        "try" => Some(TokenKind::KwTry),
        "rescue" => Some(TokenKind::KwRescue),
        "ensure" => Some(TokenKind::KwEnsure),
        "raise" => Some(TokenKind::KwRaise),
        "own" => Some(TokenKind::KwOwn),
        "borrow" => Some(TokenKind::KwBorrow),
        "ref" => Some(TokenKind::KwRef),
        "mut" => Some(TokenKind::KwMut),
        "move" => Some(TokenKind::KwMove),
        "and" => Some(TokenKind::KwAnd),
        "or" => Some(TokenKind::KwOr),
        "not" => Some(TokenKind::KwNot),
        "true" => Some(TokenKind::KwTrue),
        "false" => Some(TokenKind::KwFalse),
        "nil" => Some(TokenKind::KwNil),
        "self" => Some(TokenKind::KwSelf_),
        "super" => Some(TokenKind::KwSuper),
        _ => None,
    }
}

/// Human-readable name for a token kind, used in error messages.
pub fn describe_kind(kind: &TokenKind) -> &'static str {
    match kind {
        TokenKind::Int(_) => "integer",
        TokenKind::Float(_) => "float",
        TokenKind::Str(_) => "string",
        TokenKind::RawStr(_) => "raw string",
        TokenKind::Symbol(_) => "symbol",
        TokenKind::Ident(_) => "identifier",
        TokenKind::KwModule => "'module'",
        TokenKind::KwUse => "'use'",
        TokenKind::KwPub => "'pub'",
        TokenKind::KwEnd => "'end'",
        TokenKind::KwDef => "'def'",
        TokenKind::KwFn => "'fn'",
        TokenKind::KwLet => "'let'",
        TokenKind::KwVar => "'var'",
        TokenKind::KwConst => "'const'",
        TokenKind::KwType => "'type'",
        TokenKind::KwTrait => "'trait'",
        TokenKind::KwImpl => "'impl'",
        TokenKind::KwStruct => "'struct'",
        TokenKind::KwEnum => "'enum'",
        TokenKind::KwMemory => "'memory'",
        TokenKind::KwWorking => "'working'",
        TokenKind::KwEpisodic => "'episodic'",
        TokenKind::KwSemantic => "'semantic'",
        TokenKind::KwProcedural => "'procedural'",
        TokenKind::KwActor => "'actor'",
        TokenKind::KwProtocol => "'protocol'",
        TokenKind::KwOn => "'on'",
        TokenKind::KwSpawn => "'spawn'",
        TokenKind::KwSend => "'send'",
        TokenKind::KwIf => "'if'",
        TokenKind::KwElsif => "'elsif'",
        TokenKind::KwElse => "'else'",
        TokenKind::KwWhile => "'while'",
        TokenKind::KwFor => "'for'",
        TokenKind::KwIn => "'in'",
        TokenKind::KwLoop => "'loop'",
        TokenKind::KwBreak => "'break'",
        TokenKind::KwContinue => "'continue'",
        TokenKind::KwReturn => "'return'",
        TokenKind::KwMatch => "'match'",
        TokenKind::KwWhen => "'when'",
        TokenKind::KwTry => "'try'",
        TokenKind::KwRescue => "'rescue'",
        TokenKind::KwEnsure => "'ensure'",
        TokenKind::KwRaise => "'raise'",
        TokenKind::KwOwn => "'own'",
        TokenKind::KwBorrow => "'borrow'",
        TokenKind::KwRef => "'ref'",
        TokenKind::KwMut => "'mut'",
        TokenKind::KwMove => "'move'",
        TokenKind::KwAnd => "'and'",
        TokenKind::KwOr => "'or'",
        TokenKind::KwNot => "'not'",
        TokenKind::KwTrue => "'true'",
        TokenKind::KwFalse => "'false'",
        TokenKind::KwNil => "'nil'",
        TokenKind::KwSelf_ => "'self'",
        TokenKind::KwSuper => "'super'",
        TokenKind::Plus => "'+'",
        TokenKind::Minus => "'-'",
        TokenKind::Star => "'*'",
        TokenKind::Slash => "'/'",
        TokenKind::Percent => "'%'",
        TokenKind::Eq => "'='",
        TokenKind::EqEq => "'=='",
        TokenKind::BangEq => "'!='",
        TokenKind::Lt => "'<'",
        TokenKind::Gt => "'>'",
        TokenKind::LtEq => "'<='",
        TokenKind::GtEq => "'>='",
        TokenKind::Bang => "'!'",
        TokenKind::Question => "'?'",
        TokenKind::PipeGt => "'|>'",
        TokenKind::Pipe => "'|'",
        TokenKind::DotDot => "'..'",
        TokenKind::DotDotDot => "'...'",
        TokenKind::FatArrow => "'=>'",
        TokenKind::Arrow => "'->'",
        TokenKind::PlusEq => "'+='",
        TokenKind::MinusEq => "'-='",
        TokenKind::StarEq => "'*='",
        TokenKind::SlashEq => "'/='",
        TokenKind::PercentEq => "'%='",
        TokenKind::Amp => "'&'",
        TokenKind::At => "'@'",
        TokenKind::Dot => "'.'",
        TokenKind::Comma => "','",
        TokenKind::Colon => "':'",
        TokenKind::ColonCol => "'::'",
        TokenKind::Semi => "';'",
        TokenKind::LParen => "'('",
        TokenKind::RParen => "')'",
        TokenKind::LBrace => "'{'",
        TokenKind::RBrace => "'}'",
        TokenKind::LBracket => "'['",
        TokenKind::RBracket => "']'",
        TokenKind::Newline => "newline",
        TokenKind::Eof => "end of file",
    }
}
