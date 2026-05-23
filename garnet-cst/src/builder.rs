//! Direct recursive-descent CST builder over the trivia-preserving token stream.
//!
//! Built cold from Mini-Spec v1.0 §2–§11 for the v0.7 build-both-then-compare
//! A/B (S15). This is architecturally distinct from #221's in-parser CST, which
//! projects an already-built AST + tokens into a tree; here we parse the token
//! stream directly into a rowan green tree.
//!
//! ## Round-trip invariant (guaranteed by construction)
//!
//! Every lexer token's source slice (`&src[tok.span]`) is emitted into the tree
//! exactly once, in order, and a safety-net flush at the end emits anything the
//! grammar did not consume. Because the lexer's spans tile the source
//! contiguously, the concatenation of all emitted leaves equals the input — so
//! `cst_to_source(parse_cst(s).syntax()) == s` for any `s` that lexes, **even
//! when `s` is not grammatically valid** (error recovery still emits every
//! token).
//!
//! ## Token view
//!
//! Lookahead skips `Whitespace`/`Comment` (matching the AST parser's filtered
//! stream) but treats `Newline`/`Semi` as significant separators, consumed at
//! the same points the AST parser calls `skip_separators`.

use crate::syntax_kind::{GarnetLanguage, SyntaxKind, SyntaxNode};
use crate::{Parse, SyntaxError};
use garnet_parser::lex_source;
use garnet_parser::token::{Span, Token, TokenKind};
use rowan::{Checkpoint, GreenNodeBuilder, Language};

use SyntaxKind::*;

/// Parse a source string into a CST. Entry point behind `crate::parse_cst`.
pub(crate) fn parse(src: &str) -> Parse<SyntaxNode> {
    match lex_source(src) {
        Ok(tokens) => {
            let mut b = Builder::new(src, tokens);
            b.parse_root();
            b.finish()
        }
        Err(err) => {
            // Lexing failed: we cannot tile the source from tokens, so emit the
            // whole input as a single Error leaf (still round-trips) and record
            // the error. Recovery from un-lexable input is best-effort.
            let mut gb = GreenNodeBuilder::new();
            gb.start_node(raw(Root));
            if !src.is_empty() {
                gb.token(raw(Error), src);
            }
            gb.finish_node();
            Parse {
                root: SyntaxNode::new_root(gb.finish()),
                errors: vec![SyntaxError {
                    message: format!("lex error: {err}"),
                    offset: 0,
                }],
            }
        }
    }
}

fn raw(kind: SyntaxKind) -> rowan::SyntaxKind {
    GarnetLanguage::kind_to_raw(kind)
}

struct Builder<'a> {
    src: &'a str,
    tokens: Vec<Token>,
    pos: usize,
    gb: GreenNodeBuilder<'static>,
    errors: Vec<SyntaxError>,
}

impl<'a> Builder<'a> {
    fn new(src: &'a str, tokens: Vec<Token>) -> Self {
        Self {
            src,
            tokens,
            pos: 0,
            gb: GreenNodeBuilder::new(),
            errors: Vec::new(),
        }
    }

    fn finish(self) -> Parse<SyntaxNode> {
        Parse {
            root: SyntaxNode::new_root(self.gb.finish()),
            errors: self.errors,
        }
    }

    // ── low-level token access ──────────────────────────────────────────

    /// Kind of the n-th *significant* token from `pos` (skips Whitespace and
    /// Comment; Newline/Semi are significant). Returns `Eof` past the end.
    fn nth(&self, n: usize) -> &TokenKind {
        let mut count = 0;
        for tok in &self.tokens[self.pos.min(self.tokens.len())..] {
            if is_trivia(&tok.kind) {
                continue;
            }
            if count == n {
                return &tok.kind;
            }
            count += 1;
        }
        static EOF: TokenKind = TokenKind::Eof;
        &EOF
    }

    fn at_eof(&self) -> bool {
        matches!(self.nth(0), TokenKind::Eof)
    }

    /// Emit the token at `pos` as a leaf (skipping the zero-width Eof), advance.
    fn emit_one(&mut self) {
        if self.pos >= self.tokens.len() {
            return;
        }
        let kind = SyntaxKind::from_token(&self.tokens[self.pos].kind);
        let sp: Span = self.tokens[self.pos].span;
        if sp.len > 0 {
            let text = &self.src[sp.start..sp.end()];
            self.gb.token(raw(kind), text);
        }
        self.pos += 1;
    }

    /// Emit leading Whitespace/Comment trivia into the current node.
    fn eat_trivia(&mut self) {
        while self.pos < self.tokens.len() && is_trivia(&self.tokens[self.pos].kind) {
            self.emit_one();
        }
    }

    /// Consume runs of Whitespace/Comment/Newline/Semi (statement separators).
    fn skip_separators(&mut self) {
        while self.pos < self.tokens.len()
            && matches!(
                self.tokens[self.pos].kind,
                TokenKind::Whitespace(_)
                    | TokenKind::Comment(_)
                    | TokenKind::Newline
                    | TokenKind::Semi
            )
        {
            self.emit_one();
        }
    }

    /// Consume the next significant token (flushing leading trivia first).
    fn bump(&mut self) {
        self.eat_trivia();
        self.emit_one();
    }

    /// If the next significant token satisfies `pred`, bump it and return true.
    fn eat(&mut self, pred: fn(&TokenKind) -> bool) -> bool {
        if pred(self.nth(0)) {
            self.bump();
            true
        } else {
            false
        }
    }

    /// Expect the next significant token to satisfy `pred`; bump it or record an
    /// error (without consuming). Best-effort recovery — the trailing flush
    /// guarantees round-trip regardless.
    fn expect(&mut self, pred: fn(&TokenKind) -> bool, what: &str) {
        if pred(self.nth(0)) {
            self.bump();
        } else {
            self.error(what);
        }
    }

    fn error(&mut self, what: &str) {
        let offset = self
            .tokens
            .get(self.pos)
            .map(|t| t.span.start)
            .unwrap_or_else(|| self.src.len());
        self.errors.push(SyntaxError {
            message: format!("expected {what}"),
            offset,
        });
    }

    fn checkpoint(&self) -> Checkpoint {
        self.gb.checkpoint()
    }

    fn start(&mut self, kind: SyntaxKind) {
        self.gb.start_node(raw(kind));
    }

    fn start_at(&mut self, cp: Checkpoint, kind: SyntaxKind) {
        self.gb.start_node_at(cp, raw(kind));
    }

    fn wrap(&mut self) {
        self.gb.finish_node();
    }

    /// Emit any remaining tokens into the current (Root) node. Guarantees the
    /// round-trip invariant even if the grammar stopped early on malformed input.
    fn flush_rest(&mut self) {
        while self.pos < self.tokens.len() {
            self.emit_one();
        }
    }

    // ── grammar: top level ──────────────────────────────────────────────

    fn parse_root(&mut self) {
        self.start(Root);
        self.skip_separators();

        // File-level `@safe` marker: `@` ident("safe") at the very start.
        if matches!(self.nth(0), TokenKind::At) && is_ident_named(self.nth(1), "safe") {
            self.start(Attr);
            self.bump(); // @
            self.bump(); // safe
            self.wrap();
            self.skip_separators();
        }

        loop {
            self.skip_separators();
            if self.at_eof() {
                break;
            }
            let before = self.pos;
            self.parse_item();
            // Guard against a stuck position on an unexpected token.
            if self.pos == before {
                self.start(Error);
                self.bump();
                self.wrap();
            }
        }
        self.flush_rest();
        self.wrap(); // Root
    }

    fn parse_item(&mut self) {
        let cp = self.checkpoint();
        self.parse_attr_list();
        let _ = self.eat(is_pub);
        match self.nth(0) {
            TokenKind::KwUse => self.finish_item(cp, UseDecl, |b| b.parse_use_rest()),
            TokenKind::KwModule => self.finish_item(cp, Module, |b| b.parse_module_rest()),
            TokenKind::KwMemory => self.finish_item(cp, MemoryDecl, |b| b.parse_memory_rest()),
            TokenKind::KwActor => self.finish_item(cp, ActorDef, |b| b.parse_actor_rest()),
            TokenKind::KwStruct => self.finish_item(cp, StructDef, |b| b.parse_struct_rest()),
            TokenKind::KwEnum => self.finish_item(cp, EnumDef, |b| b.parse_enum_rest()),
            TokenKind::KwTrait => self.finish_item(cp, TraitDef, |b| b.parse_trait_like_rest()),
            TokenKind::KwProtocol => {
                self.finish_item(cp, StructProtocolDef, |b| b.parse_trait_like_rest())
            }
            TokenKind::KwImpl => self.finish_item(cp, ImplBlock, |b| b.parse_impl_rest()),
            TokenKind::KwDef => self.finish_item(cp, FnDef, |b| b.parse_fn_rest(false)),
            TokenKind::KwFn => self.finish_item(cp, SafeFnDef, |b| b.parse_fn_rest(true)),
            TokenKind::KwConst => self.finish_item(cp, ConstDef, |b| b.parse_const_rest()),
            TokenKind::KwLet => self.finish_item(cp, LetStmt, |b| b.parse_let_rest()),
            _ => {
                // Unknown item start: record + the parse_root loop will recover.
                self.error("a top-level item");
            }
        }
    }

    fn finish_item(&mut self, cp: Checkpoint, kind: SyntaxKind, f: impl FnOnce(&mut Self)) {
        self.start_at(cp, kind);
        f(self);
        self.wrap();
    }

    // ── attributes ──────────────────────────────────────────────────────

    fn parse_attr_list(&mut self) {
        if !matches!(self.nth(0), TokenKind::At) {
            return;
        }
        self.start(AttrList);
        while matches!(self.nth(0), TokenKind::At) {
            self.start(Attr);
            self.bump(); // @
            self.expect(is_ident, "annotation name");
            if matches!(self.nth(0), TokenKind::LParen) {
                self.start(AttrArgs);
                self.bump(); // (
                while !matches!(self.nth(0), TokenKind::RParen | TokenKind::Eof) {
                    self.bump();
                }
                self.expect(is_rparen, "')'");
                self.wrap();
            }
            self.wrap(); // Attr
            self.skip_separators();
        }
        self.wrap(); // AttrList
    }

    // ── items ───────────────────────────────────────────────────────────

    fn parse_use_rest(&mut self) {
        self.bump(); // use
        self.parse_path();
        // `::{a, b}` or `::*` already folded into path parsing below.
    }

    fn parse_path(&mut self) {
        self.start(UsePath);
        self.expect(is_ident, "path");
        while matches!(self.nth(0), TokenKind::ColonCol) {
            self.bump(); // ::
            if matches!(self.nth(0), TokenKind::Star) {
                self.bump();
                break;
            }
            if matches!(self.nth(0), TokenKind::LBrace) {
                self.start(UseGroup);
                self.bump(); // {
                while !matches!(self.nth(0), TokenKind::RBrace | TokenKind::Eof) {
                    self.expect(is_ident, "import name");
                    if !self.eat(is_comma) {
                        break;
                    }
                }
                self.expect(is_rbrace, "'}'");
                self.wrap();
                break;
            }
            self.expect(is_ident, "path segment");
        }
        self.wrap();
    }

    fn parse_module_rest(&mut self) {
        self.bump(); // module
        self.parse_name();
        self.expect(is_lbrace, "'{'");
        self.skip_separators();
        while !matches!(self.nth(0), TokenKind::RBrace | TokenKind::Eof) {
            let before = self.pos;
            self.parse_item();
            if self.pos == before {
                self.start(Error);
                self.bump();
                self.wrap();
            }
            self.skip_separators();
        }
        self.expect(is_rbrace, "'}'");
    }

    fn parse_memory_rest(&mut self) {
        self.bump(); // memory
        self.bump(); // kind keyword (working/episodic/semantic/procedural)
        self.parse_name();
        self.expect(is_colon, "':'");
        self.parse_type();
    }

    fn parse_actor_rest(&mut self) {
        self.bump(); // actor
        self.parse_name();
        self.expect(is_lbrace, "'{'");
        self.skip_separators();
        while !matches!(self.nth(0), TokenKind::RBrace | TokenKind::Eof) {
            self.parse_attr_list();
            self.skip_separators();
            match self.nth(0) {
                TokenKind::KwProtocol => {
                    self.start(ProtocolDecl);
                    self.bump(); // protocol
                    self.parse_name();
                    self.parse_param_list();
                    if self.eat(is_arrow) {
                        self.parse_type();
                    }
                    self.wrap();
                }
                TokenKind::KwOn => {
                    self.start(HandlerDecl);
                    self.bump(); // on
                    self.parse_name();
                    self.parse_param_list();
                    self.parse_block();
                    self.wrap();
                }
                TokenKind::KwMemory => {
                    self.start(MemoryDecl);
                    self.parse_memory_rest();
                    self.wrap();
                }
                TokenKind::KwLet => {
                    self.start(LetStmt);
                    self.parse_let_rest();
                    self.wrap();
                }
                TokenKind::Eof => break,
                _ => {
                    self.start(Error);
                    self.bump();
                    self.wrap();
                }
            }
            self.skip_separators();
        }
        self.expect(is_rbrace, "'}'");
    }

    fn parse_struct_rest(&mut self) {
        self.bump(); // struct
        self.parse_name();
        self.parse_type_params();
        self.expect(is_lbrace, "'{'");
        self.skip_separators();
        while !matches!(self.nth(0), TokenKind::RBrace | TokenKind::Eof) {
            let before = self.pos;
            self.start(FieldDecl);
            let _ = self.eat(is_pub);
            self.parse_name();
            self.expect(is_colon, "':'");
            self.parse_type();
            if self.eat(is_eq) {
                self.parse_expr();
            }
            self.wrap();
            let _ = self.eat(is_comma);
            self.skip_separators();
            if self.pos == before {
                self.start(Error);
                self.bump();
                self.wrap();
            }
        }
        self.expect(is_rbrace, "'}'");
    }

    fn parse_enum_rest(&mut self) {
        self.bump(); // enum
        self.parse_name();
        self.parse_type_params();
        self.expect(is_lbrace, "'{'");
        self.skip_separators();
        while !matches!(self.nth(0), TokenKind::RBrace | TokenKind::Eof) {
            let before = self.pos;
            self.start(Variant);
            self.parse_name();
            if matches!(self.nth(0), TokenKind::LParen) {
                self.bump(); // (
                while !matches!(self.nth(0), TokenKind::RParen | TokenKind::Eof) {
                    self.parse_type();
                    if !self.eat(is_comma) {
                        break;
                    }
                }
                self.expect(is_rparen, "')'");
            }
            self.wrap();
            let _ = self.eat(is_comma);
            self.skip_separators();
            if self.pos == before {
                self.start(Error);
                self.bump();
                self.wrap();
            }
        }
        self.expect(is_rbrace, "'}'");
    }

    /// trait / protocol bodies share the same item grammar.
    fn parse_trait_like_rest(&mut self) {
        self.bump(); // trait | protocol
        self.parse_name();
        self.parse_type_params();
        self.expect(is_lbrace, "'{'");
        self.skip_separators();
        while !matches!(self.nth(0), TokenKind::RBrace | TokenKind::Eof) {
            match self.nth(0) {
                TokenKind::KwFn | TokenKind::KwDef => {
                    self.start(FnSig);
                    self.bump(); // fn | def
                    self.parse_name();
                    self.parse_param_list();
                    if self.eat(is_arrow) {
                        self.parse_type();
                    }
                    self.wrap();
                }
                TokenKind::KwConst => {
                    self.start(ConstDef);
                    self.parse_const_rest();
                    self.wrap();
                }
                TokenKind::Eof => break,
                _ => {
                    self.start(Error);
                    self.bump();
                    self.wrap();
                }
            }
            self.skip_separators();
        }
        self.expect(is_rbrace, "'}'");
    }

    fn parse_impl_rest(&mut self) {
        self.bump(); // impl
        self.parse_type_params();
        self.parse_type(); // target
        if self.eat(is_for) {
            self.parse_type(); // trait
        }
        self.expect(is_lbrace, "'{'");
        self.skip_separators();
        while !matches!(self.nth(0), TokenKind::RBrace | TokenKind::Eof) {
            let cp = self.checkpoint();
            self.parse_attr_list();
            let _ = self.eat(is_pub);
            match self.nth(0) {
                TokenKind::KwDef => self.finish_item(cp, FnDef, |b| b.parse_fn_rest(false)),
                TokenKind::KwFn => self.finish_item(cp, SafeFnDef, |b| b.parse_fn_rest(true)),
                TokenKind::Eof => break,
                _ => {
                    self.start(Error);
                    self.bump();
                    self.wrap();
                }
            }
            self.skip_separators();
        }
        self.expect(is_rbrace, "'}'");
    }

    fn parse_fn_rest(&mut self, safe: bool) {
        self.bump(); // def | fn
        self.parse_name();
        self.parse_type_params();
        self.parse_param_list();
        if safe {
            self.expect(is_arrow, "'->'");
            self.parse_type();
        } else if self.eat(is_arrow) {
            self.parse_type();
        }
        self.parse_block();
    }

    fn parse_const_rest(&mut self) {
        self.bump(); // const
        self.parse_name();
        if self.eat(is_colon) {
            self.parse_type();
        }
        self.expect(is_eq, "'='");
        self.parse_expr();
    }

    fn parse_let_rest(&mut self) {
        self.bump(); // let
        let _ = self.eat(is_mut);
        self.parse_name();
        if self.eat(is_colon) {
            self.parse_type();
        }
        self.expect(is_eq, "'='");
        self.parse_expr();
    }

    // ── params, names, types ────────────────────────────────────────────

    fn parse_name(&mut self) {
        self.start(Name);
        // Accept ident or `self` (for method receivers).
        if matches!(self.nth(0), TokenKind::Ident(_) | TokenKind::KwSelf_) {
            self.bump();
        } else {
            self.error("a name");
        }
        self.wrap();
    }

    fn parse_param_list(&mut self) {
        self.expect(is_lparen, "'('");
        self.start(ParamList);
        while !matches!(self.nth(0), TokenKind::RParen | TokenKind::Eof) {
            self.start(Param);
            // ownership keyword
            if matches!(
                self.nth(0),
                TokenKind::KwOwn | TokenKind::KwBorrow | TokenKind::KwRef | TokenKind::KwMut
            ) {
                self.bump();
            }
            self.parse_name();
            if self.eat(is_colon) {
                self.parse_type();
            }
            self.wrap();
            if !self.eat(is_comma) {
                break;
            }
        }
        self.wrap(); // ParamList
        self.expect(is_rparen, "')'");
    }

    fn parse_type_params(&mut self) {
        if !matches!(self.nth(0), TokenKind::Lt) {
            return;
        }
        self.start(TypeParamList);
        self.bump(); // <
        while !matches!(self.nth(0), TokenKind::Gt | TokenKind::Eof) {
            self.start(TypeParam);
            self.expect(is_ident, "type parameter");
            self.wrap();
            if !self.eat(is_comma) {
                break;
            }
        }
        self.expect(is_gt, "'>'");
        self.wrap();
    }

    fn parse_type_args(&mut self) {
        if !matches!(self.nth(0), TokenKind::Lt) {
            return;
        }
        self.start(TypeArgList);
        self.bump(); // <
        while !matches!(self.nth(0), TokenKind::Gt | TokenKind::Eof) {
            self.parse_type();
            if !self.eat(is_comma) {
                break;
            }
        }
        self.expect(is_gt, "'>'");
        self.wrap();
    }

    fn parse_type(&mut self) {
        match self.nth(0) {
            TokenKind::KwDyn => {
                self.start(TypeRef);
                self.bump(); // dyn
                self.parse_named_type_inner();
                self.wrap();
            }
            TokenKind::Amp => {
                self.start(TypeRef);
                self.bump(); // &
                let _ = self.eat(is_mut);
                self.parse_type();
                self.wrap();
            }
            TokenKind::LParen => {
                let cp = self.checkpoint();
                self.bump(); // (
                let mut count = 0;
                while !matches!(self.nth(0), TokenKind::RParen | TokenKind::Eof) {
                    self.parse_type();
                    count += 1;
                    if !self.eat(is_comma) {
                        break;
                    }
                }
                self.expect(is_rparen, "')'");
                if self.eat(is_arrow) {
                    self.start_at(cp, FnType);
                    self.parse_type();
                    self.wrap();
                } else if count != 1 {
                    self.start_at(cp, TupleType);
                    self.wrap();
                }
                // count == 1 with no arrow: parenthesized type, leave as-is.
            }
            _ => {
                self.start(TypeRef);
                self.parse_named_type_inner();
                self.wrap();
            }
        }
    }

    fn parse_named_type_inner(&mut self) {
        self.expect(is_ident, "type name");
        while matches!(self.nth(0), TokenKind::ColonCol) {
            self.bump(); // ::
            self.expect(is_ident, "type path segment");
        }
        self.parse_type_args();
    }

    // ── blocks & statements ─────────────────────────────────────────────

    fn parse_block(&mut self) {
        self.expect(is_lbrace, "'{'");
        self.start(Block);
        self.skip_separators();
        while !matches!(self.nth(0), TokenKind::RBrace | TokenKind::Eof) {
            let before = self.pos;
            self.parse_stmt();
            if self.pos == before {
                self.start(Error);
                self.bump();
                self.wrap();
            }
            self.skip_separators();
        }
        self.wrap(); // Block
        self.expect(is_rbrace, "'}'");
    }

    fn parse_stmt(&mut self) {
        match self.nth(0) {
            TokenKind::KwLet => {
                self.start(LetStmt);
                self.parse_let_rest();
                self.wrap();
            }
            TokenKind::KwVar => {
                self.start(VarStmt);
                self.bump(); // var
                self.parse_name();
                if self.eat(is_colon) {
                    self.parse_type();
                }
                self.expect(is_eq, "'='");
                self.parse_expr();
                self.wrap();
            }
            TokenKind::KwConst => {
                self.start(ConstDef);
                self.parse_const_rest();
                self.wrap();
            }
            TokenKind::KwWhile => {
                self.start(WhileStmt);
                self.bump();
                self.parse_expr();
                self.parse_block();
                self.wrap();
            }
            TokenKind::KwFor => {
                self.start(ForStmt);
                self.bump();
                self.parse_name();
                self.expect(is_in, "'in'");
                self.parse_expr();
                self.parse_block();
                self.wrap();
            }
            TokenKind::KwLoop => {
                self.start(LoopStmt);
                self.bump();
                self.parse_block();
                self.wrap();
            }
            TokenKind::KwBreak => self.parse_value_stmt(BreakStmt),
            TokenKind::KwReturn => self.parse_value_stmt(ReturnStmt),
            TokenKind::KwYield => self.parse_value_stmt(YieldStmt),
            TokenKind::KwNext => self.parse_value_stmt(NextStmt),
            TokenKind::KwContinue => {
                self.start(ContinueStmt);
                self.bump();
                self.wrap();
            }
            TokenKind::KwRaise => {
                self.start(RaiseStmt);
                self.bump();
                self.parse_expr();
                self.wrap();
            }
            _ => {
                // expression, optionally an assignment statement
                let cp = self.checkpoint();
                self.parse_expr();
                if matches!(
                    self.nth(0),
                    TokenKind::Eq
                        | TokenKind::PlusEq
                        | TokenKind::MinusEq
                        | TokenKind::StarEq
                        | TokenKind::SlashEq
                        | TokenKind::PercentEq
                ) {
                    self.start_at(cp, AssignStmt);
                    self.bump(); // assign op
                    self.parse_expr();
                    self.wrap();
                } else {
                    self.start_at(cp, ExprStmt);
                    self.wrap();
                }
            }
        }
    }

    fn parse_value_stmt(&mut self, kind: SyntaxKind) {
        self.start(kind);
        self.bump(); // keyword
        if !matches!(
            self.nth(0),
            TokenKind::Newline | TokenKind::Semi | TokenKind::RBrace | TokenKind::Eof
        ) {
            self.parse_expr();
        }
        self.wrap();
    }

    // ── expressions: 11-level Pratt tower ───────────────────────────────

    fn parse_expr(&mut self) {
        self.parse_pipeline();
    }

    fn parse_pipeline(&mut self) {
        let cp = self.checkpoint();
        self.parse_or();
        // Allow `|>` to begin a continuation line (peek past newlines).
        loop {
            let mut look = 0;
            while matches!(self.nth(look), TokenKind::Newline) {
                look += 1;
            }
            if !matches!(self.nth(look), TokenKind::PipeGt) {
                break;
            }
            self.start_at(cp, PipelineExpr);
            self.skip_separators(); // consume the peeked newlines + ws
            self.bump(); // |>
            self.parse_or();
            self.wrap();
        }
    }

    fn parse_or(&mut self) {
        let cp = self.checkpoint();
        self.parse_and();
        while matches!(self.nth(0), TokenKind::KwOr) {
            self.start_at(cp, BinaryExpr);
            self.bump();
            self.parse_and();
            self.wrap();
        }
    }

    fn parse_and(&mut self) {
        let cp = self.checkpoint();
        self.parse_not();
        while matches!(self.nth(0), TokenKind::KwAnd) {
            self.start_at(cp, BinaryExpr);
            self.bump();
            self.parse_not();
            self.wrap();
        }
    }

    fn parse_not(&mut self) {
        if matches!(self.nth(0), TokenKind::KwNot) {
            self.start(UnaryExpr);
            self.bump();
            self.parse_not();
            self.wrap();
        } else {
            self.parse_comparison();
        }
    }

    fn parse_comparison(&mut self) {
        let cp = self.checkpoint();
        self.parse_range();
        if matches!(
            self.nth(0),
            TokenKind::EqEq
                | TokenKind::BangEq
                | TokenKind::Lt
                | TokenKind::Gt
                | TokenKind::LtEq
                | TokenKind::GtEq
        ) {
            self.start_at(cp, BinaryExpr);
            self.bump();
            self.parse_range();
            self.wrap();
        }
    }

    fn parse_range(&mut self) {
        let cp = self.checkpoint();
        self.parse_addition();
        if matches!(self.nth(0), TokenKind::DotDot | TokenKind::DotDotDot) {
            self.start_at(cp, RangeExpr);
            self.bump();
            self.parse_addition();
            self.wrap();
        }
    }

    fn parse_addition(&mut self) {
        let cp = self.checkpoint();
        self.parse_multiplication();
        while matches!(self.nth(0), TokenKind::Plus | TokenKind::Minus) {
            self.start_at(cp, BinaryExpr);
            self.bump();
            self.parse_multiplication();
            self.wrap();
        }
    }

    fn parse_multiplication(&mut self) {
        let cp = self.checkpoint();
        self.parse_unary();
        while matches!(
            self.nth(0),
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent
        ) {
            self.start_at(cp, BinaryExpr);
            self.bump();
            self.parse_unary();
            self.wrap();
        }
    }

    fn parse_unary(&mut self) {
        if matches!(self.nth(0), TokenKind::Minus | TokenKind::Bang) {
            self.start(UnaryExpr);
            self.bump();
            self.parse_unary();
            self.wrap();
        } else {
            self.parse_postfix();
        }
    }

    fn parse_postfix(&mut self) {
        let cp = self.checkpoint();
        self.parse_primary();
        loop {
            match self.nth(0) {
                TokenKind::Dot => {
                    self.bump(); // .
                    self.parse_member_name();
                    if matches!(self.nth(0), TokenKind::LParen) {
                        self.start_at(cp, MethodCallExpr);
                        self.parse_arg_list();
                        self.parse_trailing_do_block();
                        self.wrap();
                    } else {
                        self.start_at(cp, FieldExpr);
                        self.wrap();
                    }
                }
                TokenKind::LParen => {
                    self.start_at(cp, CallExpr);
                    self.parse_arg_list();
                    self.parse_trailing_do_block();
                    self.wrap();
                }
                TokenKind::LBracket => {
                    self.start_at(cp, IndexExpr);
                    self.bump(); // [
                    self.parse_expr();
                    self.expect(is_rbracket, "']'");
                    self.wrap();
                }
                TokenKind::ColonCol => {
                    self.start_at(cp, PathExpr);
                    self.bump(); // ::
                    self.expect(is_ident, "path segment");
                    self.wrap();
                }
                TokenKind::Question => {
                    self.start_at(cp, UnaryExpr);
                    self.bump();
                    self.wrap();
                }
                TokenKind::KwAs => {
                    self.start_at(cp, CastExpr);
                    self.bump();
                    self.parse_type();
                    self.wrap();
                }
                _ => break,
            }
        }
    }

    fn parse_member_name(&mut self) {
        // `.spawn` is accepted as a method name; otherwise an ident.
        if matches!(self.nth(0), TokenKind::Ident(_) | TokenKind::KwSpawn) {
            self.start(NameRef);
            self.bump();
            self.wrap();
        } else {
            self.error("field or method name");
        }
    }

    /// `(...)` argument list. Assumes the next significant token is `(`.
    fn parse_arg_list(&mut self) {
        self.start(ArgList);
        self.bump(); // (
        self.skip_separators();
        while !matches!(
            self.nth(0),
            TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace | TokenKind::Eof
        ) {
            self.parse_expr();
            self.skip_separators();
            if !self.eat(is_comma) {
                break;
            }
            self.skip_separators();
        }
        self.skip_separators();
        self.expect(is_rparen, "')'");
        self.wrap();
    }

    fn parse_trailing_do_block(&mut self) {
        if !matches!(self.nth(0), TokenKind::KwDo) {
            return;
        }
        self.start(BlockArg);
        self.bump(); // do
                     // `do |a, b|` params; `do ||` is the empty list. `eat` is side-effecting,
                     // so the `&&` short-circuit drives the same token consumption as the
                     // nested form.
        if self.eat(is_pipe) && !self.eat(is_pipe) {
            self.parse_closure_params_until_pipe();
            self.expect(is_pipe, "'|'");
        }
        self.skip_separators();
        while !matches!(self.nth(0), TokenKind::KwEnd | TokenKind::Eof) {
            let before = self.pos;
            self.parse_stmt();
            if self.pos == before {
                self.start(Error);
                self.bump();
                self.wrap();
            }
            self.skip_separators();
        }
        self.expect(is_end, "'end'");
        self.wrap();
    }

    fn parse_closure_params_until_pipe(&mut self) {
        self.start(ClosureParamList);
        while !matches!(self.nth(0), TokenKind::Pipe | TokenKind::Eof) {
            self.start(Param);
            self.parse_name();
            if self.eat(is_colon) {
                self.parse_type();
            }
            self.wrap();
            if !self.eat(is_comma) {
                break;
            }
        }
        self.wrap();
    }

    fn parse_primary(&mut self) {
        match self.nth(0) {
            TokenKind::Int(_)
            | TokenKind::Float(_)
            | TokenKind::Str(_)
            | TokenKind::RawStr(_)
            | TokenKind::Symbol(_)
            | TokenKind::KwTrue
            | TokenKind::KwFalse
            | TokenKind::KwNil => {
                self.start(Literal);
                self.bump();
                self.wrap();
            }
            TokenKind::Ident(_) | TokenKind::KwSelf_ | TokenKind::KwSuper => {
                self.start(NameRef);
                self.bump();
                self.wrap();
            }
            TokenKind::LParen => {
                self.start(ParenExpr);
                self.bump(); // (
                if !matches!(self.nth(0), TokenKind::RParen) {
                    self.parse_expr();
                }
                self.expect(is_rparen, "')'");
                self.wrap();
            }
            TokenKind::LBracket => {
                self.start(ArrayLit);
                self.bump(); // [
                self.skip_separators();
                while !matches!(self.nth(0), TokenKind::RBracket | TokenKind::Eof) {
                    self.parse_expr();
                    self.skip_separators();
                    if !self.eat(is_comma) {
                        break;
                    }
                    self.skip_separators();
                }
                self.expect(is_rbracket, "']'");
                self.wrap();
            }
            TokenKind::LBrace => self.parse_map_literal(),
            TokenKind::KwIf => self.parse_if(),
            TokenKind::KwMatch => self.parse_match(),
            TokenKind::KwTry => self.parse_try(),
            TokenKind::KwSpawn => {
                self.start(SpawnExpr);
                self.bump();
                self.parse_expr();
                self.wrap();
            }
            TokenKind::Pipe => self.parse_closure(),
            _ => self.error("expression"),
        }
    }

    fn parse_map_literal(&mut self) {
        self.start(MapLit);
        self.bump(); // {
        self.skip_separators();
        while !matches!(self.nth(0), TokenKind::RBrace | TokenKind::Eof) {
            self.start(MapEntry);
            self.parse_expr();
            self.expect(is_fat_arrow, "'=>'");
            self.parse_expr();
            self.wrap();
            self.skip_separators();
            if !self.eat(is_comma) {
                break;
            }
            self.skip_separators();
        }
        self.expect(is_rbrace, "'}'");
        self.wrap();
    }

    fn parse_closure(&mut self) {
        self.start(ClosureExpr);
        self.bump(); // |
        if !matches!(self.nth(0), TokenKind::Pipe) {
            self.parse_closure_params_until_pipe();
        }
        self.expect(is_pipe, "'|'");
        if self.eat(is_arrow) {
            self.parse_type();
        }
        if matches!(self.nth(0), TokenKind::LBrace) {
            self.parse_block();
        } else {
            self.parse_expr();
        }
        self.wrap();
    }

    fn parse_if(&mut self) {
        self.start(IfExpr);
        self.bump(); // if
        self.parse_expr();
        self.parse_block();
        while matches!(self.nth(0), TokenKind::KwElsif) {
            self.bump();
            self.parse_expr();
            self.parse_block();
        }
        if self.eat(is_else) {
            self.parse_block();
        }
        self.wrap();
    }

    fn parse_match(&mut self) {
        self.start(MatchExpr);
        self.bump(); // match
        self.parse_expr();
        self.expect(is_lbrace, "'{'");
        self.skip_separators();
        while !matches!(self.nth(0), TokenKind::RBrace | TokenKind::Eof) {
            let before = self.pos;
            self.start(MatchArm);
            self.parse_pattern();
            if self.eat(is_if) {
                self.start(MatchGuard);
                self.parse_expr();
                self.wrap();
            }
            self.expect(is_fat_arrow, "'=>'");
            if matches!(self.nth(0), TokenKind::LBrace) {
                self.parse_block();
            } else {
                self.parse_expr();
            }
            self.wrap();
            let _ = self.eat(is_comma);
            self.skip_separators();
            if self.pos == before {
                self.start(Error);
                self.bump();
                self.wrap();
            }
        }
        self.expect(is_rbrace, "'}'");
        self.wrap();
    }

    fn parse_try(&mut self) {
        self.start(TryExpr);
        self.bump(); // try
        self.parse_block();
        while matches!(self.nth(0), TokenKind::KwRescue) {
            self.start(RescueClause);
            self.bump(); // rescue
            if matches!(self.nth(0), TokenKind::Ident(_)) {
                self.parse_name();
                if self.eat(is_colon) {
                    self.parse_type();
                }
            }
            self.parse_block();
            self.wrap();
        }
        if matches!(self.nth(0), TokenKind::KwEnsure) {
            self.start(EnsureClause);
            self.bump();
            self.parse_block();
            self.wrap();
        }
        self.wrap();
    }

    fn parse_pattern(&mut self) {
        match self.nth(0) {
            TokenKind::DotDot => {
                self.start(RestPat);
                self.bump();
                self.wrap();
            }
            TokenKind::LParen => {
                self.start(TuplePat);
                self.bump(); // (
                while !matches!(self.nth(0), TokenKind::RParen | TokenKind::Eof) {
                    self.parse_pattern();
                    if !self.eat(is_comma) {
                        break;
                    }
                }
                self.expect(is_rparen, "')'");
                self.wrap();
            }
            TokenKind::Int(_)
            | TokenKind::Float(_)
            | TokenKind::Str(_)
            | TokenKind::Symbol(_)
            | TokenKind::KwTrue
            | TokenKind::KwFalse
            | TokenKind::KwNil => {
                self.start(LiteralPat);
                self.bump();
                self.wrap();
            }
            TokenKind::Ident(name) if name == "_" => {
                self.start(WildcardPat);
                self.bump();
                self.wrap();
            }
            TokenKind::Ident(_) => {
                let cp = self.checkpoint();
                self.bump(); // first ident
                let mut is_enum = false;
                while matches!(self.nth(0), TokenKind::ColonCol) {
                    is_enum = true;
                    self.bump();
                    self.expect(is_ident, "pattern path");
                }
                if matches!(self.nth(0), TokenKind::LParen) {
                    self.start_at(cp, EnumPat);
                    self.bump(); // (
                    while !matches!(self.nth(0), TokenKind::RParen | TokenKind::Eof) {
                        self.parse_pattern();
                        if !self.eat(is_comma) {
                            break;
                        }
                    }
                    self.expect(is_rparen, "')'");
                    self.wrap();
                } else if is_enum {
                    self.start_at(cp, EnumPat);
                    self.wrap();
                } else {
                    self.start_at(cp, IdentPat);
                    self.wrap();
                }
            }
            _ => self.error("pattern"),
        }
    }
}

// ── token classification helpers ────────────────────────────────────────

fn is_trivia(k: &TokenKind) -> bool {
    matches!(k, TokenKind::Whitespace(_) | TokenKind::Comment(_))
}

fn is_ident(k: &TokenKind) -> bool {
    matches!(k, TokenKind::Ident(_))
}

fn is_ident_named(k: &TokenKind, name: &str) -> bool {
    matches!(k, TokenKind::Ident(s) if s == name)
}

fn is_pub(k: &TokenKind) -> bool {
    matches!(k, TokenKind::KwPub)
}
fn is_mut(k: &TokenKind) -> bool {
    matches!(k, TokenKind::KwMut)
}
fn is_comma(k: &TokenKind) -> bool {
    matches!(k, TokenKind::Comma)
}
fn is_colon(k: &TokenKind) -> bool {
    matches!(k, TokenKind::Colon)
}
fn is_eq(k: &TokenKind) -> bool {
    matches!(k, TokenKind::Eq)
}
fn is_arrow(k: &TokenKind) -> bool {
    matches!(k, TokenKind::Arrow)
}
fn is_for(k: &TokenKind) -> bool {
    matches!(k, TokenKind::KwFor)
}
fn is_in(k: &TokenKind) -> bool {
    matches!(k, TokenKind::KwIn)
}
fn is_if(k: &TokenKind) -> bool {
    matches!(k, TokenKind::KwIf)
}
fn is_else(k: &TokenKind) -> bool {
    matches!(k, TokenKind::KwElse)
}
fn is_pipe(k: &TokenKind) -> bool {
    matches!(k, TokenKind::Pipe)
}
fn is_end(k: &TokenKind) -> bool {
    matches!(k, TokenKind::KwEnd)
}
fn is_fat_arrow(k: &TokenKind) -> bool {
    matches!(k, TokenKind::FatArrow)
}
fn is_lparen(k: &TokenKind) -> bool {
    matches!(k, TokenKind::LParen)
}
fn is_rparen(k: &TokenKind) -> bool {
    matches!(k, TokenKind::RParen)
}
fn is_lbrace(k: &TokenKind) -> bool {
    matches!(k, TokenKind::LBrace)
}
fn is_rbrace(k: &TokenKind) -> bool {
    matches!(k, TokenKind::RBrace)
}
fn is_rbracket(k: &TokenKind) -> bool {
    matches!(k, TokenKind::RBracket)
}
fn is_gt(k: &TokenKind) -> bool {
    matches!(k, TokenKind::Gt)
}
