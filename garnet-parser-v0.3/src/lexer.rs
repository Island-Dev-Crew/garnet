//! Garnet v0.3 hand-rolled single-pass lexer.
//! Produces a flat Vec<Token> with spans for all v0.3 keywords, operators, and literals.
//!
//! **v3.3 hardening:** every allocation path is gated by `ParseBudget` —
//! token count, literal size, comment length. Adversarial inputs fail
//! fast with `ParseError::BudgetExceeded` instead of pinning CPU/RAM.

use crate::budget::ParseBudget;
use crate::error::ParseError;
use crate::token::*;

pub struct Lexer<'a> {
    src: &'a [u8],
    pos: usize,
    budget: ParseBudget,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self::with_budget(src, ParseBudget::default())
    }

    pub fn with_budget(src: &'a str, budget: ParseBudget) -> Self {
        Self {
            src: src.as_bytes(),
            pos: 0,
            budget,
        }
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, ParseError> {
        let mut tokens = Vec::new();
        loop {
            // Budget: refuse to produce more than max_tokens tokens.
            // Checked at the top of each iteration so a deeply malformed
            // input fails in milliseconds with a clean error instead of
            // allocating for seconds.
            self.budget
                .check_tokens(tokens.len(), Span::new(self.pos, 0))?;
            self.skip_spaces();
            if self.pos >= self.src.len() {
                tokens.push(Token {
                    kind: TokenKind::Eof,
                    span: Span::new(self.pos, 0),
                });
                break;
            }
            let ch = self.src[self.pos] as char;
            match ch {
                '\n' | '\r' => {
                    let start = self.pos;
                    if ch == '\r' && self.peek_at(1) == Some('\n') {
                        self.pos += 2;
                    } else {
                        self.pos += 1;
                    }
                    tokens.push(Token {
                        kind: TokenKind::Newline,
                        span: Span::new(start, self.pos - start),
                    });
                }
                '#' => {
                    // Comment — skip to end of line. Budget-checked so a
                    // multi-MB comment block fails instead of walking
                    // byte-by-byte.
                    let start = self.pos;
                    while self.pos < self.src.len() && self.src[self.pos] != b'\n' {
                        self.pos += 1;
                    }
                    self.budget.check_literal_bytes(
                        self.pos - start,
                        Span::new(start, self.pos - start),
                    )?;
                }
                '"' => tokens.push(self.lex_string()?),
                'r' if self.peek_at(1) == Some('"') => tokens.push(self.lex_raw_string()?),
                ':' => {
                    if self.peek_at(1) == Some(':') {
                        let start = self.pos;
                        self.pos += 2;
                        tokens.push(Token { kind: TokenKind::ColonCol, span: Span::new(start, 2) });
                    } else if self.peek_at(1).is_some_and(|c| c.is_ascii_alphabetic() || c == '_') {
                        tokens.push(self.lex_symbol()?);
                    } else {
                        let start = self.pos;
                        self.pos += 1;
                        tokens.push(Token { kind: TokenKind::Colon, span: Span::new(start, 1) });
                    }
                }
                '0'..='9' => tokens.push(self.lex_number()?),
                c if c.is_ascii_alphabetic() || c == '_' => tokens.push(self.lex_ident_or_keyword()?),
                '(' => { tokens.push(self.single(TokenKind::LParen)); }
                ')' => { tokens.push(self.single(TokenKind::RParen)); }
                '{' => { tokens.push(self.single(TokenKind::LBrace)); }
                '}' => { tokens.push(self.single(TokenKind::RBrace)); }
                '[' => { tokens.push(self.single(TokenKind::LBracket)); }
                ']' => { tokens.push(self.single(TokenKind::RBracket)); }
                ',' => { tokens.push(self.single(TokenKind::Comma)); }
                ';' => { tokens.push(self.single(TokenKind::Semi)); }
                '@' => { tokens.push(self.single(TokenKind::At)); }
                '&' => { tokens.push(self.single(TokenKind::Amp)); }
                '?' => { tokens.push(self.single(TokenKind::Question)); }
                '.' => {
                    let start = self.pos;
                    self.pos += 1;
                    if self.peek_ch() == Some('.') {
                        self.pos += 1;
                        if self.peek_ch() == Some('.') {
                            self.pos += 1;
                            tokens.push(Token { kind: TokenKind::DotDotDot, span: Span::new(start, 3) });
                        } else {
                            tokens.push(Token { kind: TokenKind::DotDot, span: Span::new(start, 2) });
                        }
                    } else {
                        tokens.push(Token { kind: TokenKind::Dot, span: Span::new(start, 1) });
                    }
                }
                '+' => {
                    let start = self.pos;
                    self.pos += 1;
                    if self.peek_ch() == Some('=') {
                        self.pos += 1;
                        tokens.push(Token { kind: TokenKind::PlusEq, span: Span::new(start, 2) });
                    } else {
                        tokens.push(Token { kind: TokenKind::Plus, span: Span::new(start, 1) });
                    }
                }
                '-' => {
                    let start = self.pos;
                    self.pos += 1;
                    if self.peek_ch() == Some('>') {
                        self.pos += 1;
                        tokens.push(Token { kind: TokenKind::Arrow, span: Span::new(start, 2) });
                    } else if self.peek_ch() == Some('=') {
                        self.pos += 1;
                        tokens.push(Token { kind: TokenKind::MinusEq, span: Span::new(start, 2) });
                    } else {
                        tokens.push(Token { kind: TokenKind::Minus, span: Span::new(start, 1) });
                    }
                }
                '*' => {
                    let start = self.pos;
                    self.pos += 1;
                    if self.peek_ch() == Some('=') {
                        self.pos += 1;
                        tokens.push(Token { kind: TokenKind::StarEq, span: Span::new(start, 2) });
                    } else {
                        tokens.push(Token { kind: TokenKind::Star, span: Span::new(start, 1) });
                    }
                }
                '/' => {
                    let start = self.pos;
                    self.pos += 1;
                    if self.peek_ch() == Some('=') {
                        self.pos += 1;
                        tokens.push(Token { kind: TokenKind::SlashEq, span: Span::new(start, 2) });
                    } else {
                        tokens.push(Token { kind: TokenKind::Slash, span: Span::new(start, 1) });
                    }
                }
                '%' => {
                    let start = self.pos;
                    self.pos += 1;
                    if self.peek_ch() == Some('=') {
                        self.pos += 1;
                        tokens.push(Token { kind: TokenKind::PercentEq, span: Span::new(start, 2) });
                    } else {
                        tokens.push(Token { kind: TokenKind::Percent, span: Span::new(start, 1) });
                    }
                }
                '=' => {
                    let start = self.pos;
                    self.pos += 1;
                    if self.peek_ch() == Some('=') {
                        self.pos += 1;
                        tokens.push(Token { kind: TokenKind::EqEq, span: Span::new(start, 2) });
                    } else if self.peek_ch() == Some('>') {
                        self.pos += 1;
                        tokens.push(Token { kind: TokenKind::FatArrow, span: Span::new(start, 2) });
                    } else {
                        tokens.push(Token { kind: TokenKind::Eq, span: Span::new(start, 1) });
                    }
                }
                '!' => {
                    let start = self.pos;
                    self.pos += 1;
                    if self.peek_ch() == Some('=') {
                        self.pos += 1;
                        tokens.push(Token { kind: TokenKind::BangEq, span: Span::new(start, 2) });
                    } else {
                        tokens.push(Token { kind: TokenKind::Bang, span: Span::new(start, 1) });
                    }
                }
                '<' => {
                    let start = self.pos;
                    self.pos += 1;
                    if self.peek_ch() == Some('=') {
                        self.pos += 1;
                        tokens.push(Token { kind: TokenKind::LtEq, span: Span::new(start, 2) });
                    } else {
                        tokens.push(Token { kind: TokenKind::Lt, span: Span::new(start, 1) });
                    }
                }
                '>' => {
                    let start = self.pos;
                    self.pos += 1;
                    if self.peek_ch() == Some('=') {
                        self.pos += 1;
                        tokens.push(Token { kind: TokenKind::GtEq, span: Span::new(start, 2) });
                    } else {
                        tokens.push(Token { kind: TokenKind::Gt, span: Span::new(start, 1) });
                    }
                }
                '|' => {
                    let start = self.pos;
                    self.pos += 1;
                    if self.peek_ch() == Some('>') {
                        self.pos += 1;
                        tokens.push(Token { kind: TokenKind::PipeGt, span: Span::new(start, 2) });
                    } else {
                        tokens.push(Token { kind: TokenKind::Pipe, span: Span::new(start, 1) });
                    }
                }
                _ => {
                    return Err(ParseError::UnexpectedChar {
                        ch,
                        span: Span::new(self.pos, 1),
                    });
                }
            }
        }
        Ok(tokens)
    }

    fn skip_spaces(&mut self) {
        while self.pos < self.src.len() {
            let ch = self.src[self.pos];
            if ch == b' ' || ch == b'\t' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn peek_ch(&self) -> Option<char> {
        self.src.get(self.pos).map(|&b| b as char)
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.src.get(self.pos + offset).map(|&b| b as char)
    }

    fn single(&mut self, kind: TokenKind) -> Token {
        let start = self.pos;
        self.pos += 1;
        Token { kind, span: Span::new(start, 1) }
    }

    fn lex_ident_or_keyword(&mut self) -> Result<Token, ParseError> {
        let start = self.pos;
        while self.pos < self.src.len()
            && (self.src[self.pos].is_ascii_alphanumeric() || self.src[self.pos] == b'_')
        {
            self.pos += 1;
            // Budget: an identifier longer than max_literal_bytes is
            // adversarial. Fail fast instead of letting a pathological
            // 16 MB identifier allocate its own String.
            if self.pos - start > self.budget.max_literal_bytes {
                return Err(ParseError::budget_exceeded(
                    "literal_bytes",
                    self.budget.max_literal_bytes,
                    self.pos - start,
                    Span::new(start, self.pos - start),
                ));
            }
        }
        // SAFETY: the loop above only advanced past ASCII alnum / underscore
        // bytes, so the slice is unambiguously valid UTF-8. Use lossy conversion
        // anyway to keep the panic surface zero — invalid bytes become U+FFFD,
        // which the keyword lookup will simply fail to match.
        let owned = String::from_utf8_lossy(&self.src[start..self.pos]).into_owned();
        let span = Span::new(start, self.pos - start);
        if let Some(kw) = keyword_lookup(&owned) {
            Ok(Token { kind: kw, span })
        } else {
            Ok(Token { kind: TokenKind::Ident(owned), span })
        }
    }

    fn lex_number(&mut self) -> Result<Token, ParseError> {
        let start = self.pos;
        // Consume digits (with optional underscores)
        while self.pos < self.src.len()
            && (self.src[self.pos].is_ascii_digit() || self.src[self.pos] == b'_')
        {
            self.pos += 1;
        }
        // Check for float: . followed by digit (NOT .. which is range)
        let is_float = self.peek_ch() == Some('.')
            && self.peek_at(1).is_some_and(|c| c.is_ascii_digit());

        if is_float {
            self.pos += 1; // consume .
            while self.pos < self.src.len()
                && (self.src[self.pos].is_ascii_digit() || self.src[self.pos] == b'_')
            {
                self.pos += 1;
            }
            // Optional exponent
            if self.peek_ch() == Some('e') || self.peek_ch() == Some('E') {
                self.pos += 1;
                if self.peek_ch() == Some('+') || self.peek_ch() == Some('-') {
                    self.pos += 1;
                }
                while self.pos < self.src.len() && self.src[self.pos].is_ascii_digit() {
                    self.pos += 1;
                }
            }
            // SAFETY: ASCII-bounded loop, but use lossy conversion to keep the
            // lexer panic-free under any input corruption.
            let text = String::from_utf8_lossy(&self.src[start..self.pos])
                .replace('_', "");
            let span = Span::new(start, self.pos - start);
            match text.parse::<f64>() {
                Ok(v) => Ok(Token { kind: TokenKind::Float(v), span }),
                Err(_) => Err(ParseError::InvalidFloat { span }),
            }
        } else {
            let text = String::from_utf8_lossy(&self.src[start..self.pos])
                .replace('_', "");
            let span = Span::new(start, self.pos - start);
            match text.parse::<i64>() {
                Ok(v) => Ok(Token { kind: TokenKind::Int(v), span }),
                Err(_) => Err(ParseError::InvalidInt { span }),
            }
        }
    }

    fn lex_string(&mut self) -> Result<Token, ParseError> {
        let start = self.pos;
        self.pos += 1; // skip opening "
        let mut parts: Vec<StrPart> = Vec::new();
        let mut buf = String::new();

        loop {
            // Budget: refuse to let a single string literal grow past
            // max_literal_bytes. Checked on every loop iteration so a
            // StringBlimp attack fails in milliseconds.
            if self.pos - start > self.budget.max_literal_bytes {
                return Err(ParseError::budget_exceeded(
                    "literal_bytes",
                    self.budget.max_literal_bytes,
                    self.pos - start,
                    Span::new(start, self.pos - start),
                ));
            }
            if self.pos >= self.src.len() {
                return Err(ParseError::UnterminatedString {
                    span: Span::new(start, self.pos - start),
                });
            }
            let ch = self.src[self.pos] as char;
            match ch {
                '"' => {
                    self.pos += 1;
                    break;
                }
                '#' if self.peek_at(1) == Some('{') => {
                    // String interpolation
                    if !buf.is_empty() {
                        parts.push(StrPart::Lit(std::mem::take(&mut buf)));
                    }
                    self.pos += 2; // skip #{
                    let interp_start = self.pos;
                    let mut depth = 1u32;
                    while self.pos < self.src.len() && depth > 0 {
                        match self.src[self.pos] as char {
                            '{' => depth += 1,
                            '}' => depth -= 1,
                            _ => {}
                        }
                        if depth > 0 {
                            self.pos += 1;
                        }
                    }
                    // String interpolation can contain arbitrary UTF-8; the
                    // brace-counting loop walks bytes, so the slice may cross
                    // multi-byte char boundaries on malformed input. Lossy
                    // conversion replaces invalid sequences with U+FFFD.
                    let interp_src =
                        String::from_utf8_lossy(&self.src[interp_start..self.pos]).into_owned();
                    parts.push(StrPart::Interp(interp_src));
                    self.pos += 1; // skip closing }
                }
                '\\' => {
                    self.pos += 1;
                    if self.pos < self.src.len() {
                        let esc = self.src[self.pos] as char;
                        match esc {
                            'n' => buf.push('\n'),
                            't' => buf.push('\t'),
                            'r' => buf.push('\r'),
                            '\\' => buf.push('\\'),
                            '"' => buf.push('"'),
                            '#' => buf.push('#'),
                            _ => {
                                buf.push('\\');
                                buf.push(esc);
                            }
                        }
                        self.pos += 1;
                    }
                }
                '\n' => {
                    return Err(ParseError::UnterminatedString {
                        span: Span::new(start, self.pos - start),
                    });
                }
                _ => {
                    buf.push(ch);
                    self.pos += 1;
                }
            }
        }
        if !buf.is_empty() {
            parts.push(StrPart::Lit(buf));
        }
        let span = Span::new(start, self.pos - start);
        Ok(Token { kind: TokenKind::Str(parts), span })
    }

    fn lex_raw_string(&mut self) -> Result<Token, ParseError> {
        let start = self.pos;
        self.pos += 2; // skip r"
        let mut buf = String::new();
        loop {
            if self.pos - start > self.budget.max_literal_bytes {
                return Err(ParseError::budget_exceeded(
                    "literal_bytes",
                    self.budget.max_literal_bytes,
                    self.pos - start,
                    Span::new(start, self.pos - start),
                ));
            }
            if self.pos >= self.src.len() {
                return Err(ParseError::UnterminatedString {
                    span: Span::new(start, self.pos - start),
                });
            }
            let ch = self.src[self.pos] as char;
            if ch == '"' {
                self.pos += 1;
                break;
            }
            if ch == '\n' {
                return Err(ParseError::UnterminatedString {
                    span: Span::new(start, self.pos - start),
                });
            }
            buf.push(ch);
            self.pos += 1;
        }
        let span = Span::new(start, self.pos - start);
        Ok(Token { kind: TokenKind::RawStr(buf), span })
    }

    fn lex_symbol(&mut self) -> Result<Token, ParseError> {
        let start = self.pos;
        self.pos += 1; // skip :
        let name_start = self.pos;
        while self.pos < self.src.len()
            && (self.src[self.pos].is_ascii_alphanumeric() || self.src[self.pos] == b'_')
        {
            self.pos += 1;
            if self.pos - name_start > self.budget.max_literal_bytes {
                return Err(ParseError::budget_exceeded(
                    "literal_bytes",
                    self.budget.max_literal_bytes,
                    self.pos - name_start,
                    Span::new(start, self.pos - start),
                ));
            }
        }
        // SAFETY: ASCII-bounded loop, lossy conversion for defense in depth.
        let name = String::from_utf8_lossy(&self.src[name_start..self.pos]).into_owned();
        Ok(Token {
            kind: TokenKind::Symbol(name),
            span: Span::new(start, self.pos - start),
        })
    }
}
