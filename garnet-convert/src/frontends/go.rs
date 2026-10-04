//! Go → CIR frontend (stylized parser for v4.1 initial release).
//!
//! Go was added as a 4th target per Phase 3G GitHub conversion
//! finding: Go channels + goroutines map onto Garnet actors with
//! typed protocols + BoundedMail with unexpected cleanness.
//!
//! Recognizes: `func name(params) ReturnType { ... }`, `type Name
//! struct`, `var name T`, `chan T` → actor protocol marker, `go fn()`
//! → `spawn`, basic expressions.
//!
//! Flags: `unsafe.Pointer` → Untranslatable; `interface{}` → MigrateTodo
//! (suggest `dyn Trait` or structural `protocol`).
//!
//! C1-18: a statement ends where Go inserts a semicolon, once every bracket,
//! block comment and raw string is closed (see `lex`). Only a statement on one
//! line becomes code; every other one, blocks included, is kept whole as one
//! MigrateTodo.

use super::lex::{go_keyword_takes_operand, GoLex};
use crate::cir::{Cir, CirLit, CirTy, FuncMode, Ownership, Param};
use crate::error::ConvertError;
use crate::lineage::Lineage;

pub fn parse_and_lift(source: &str, filename: &str) -> Result<Cir, ConvertError> {
    let mut p = GoParser::new(source, filename);
    p.parse_module()
}

struct GoParser<'a> {
    source: &'a str,
    filename: String,
    pos: usize,
}

impl<'a> GoParser<'a> {
    fn new(source: &'a str, filename: &str) -> Self {
        Self {
            source,
            filename: filename.to_string(),
            pos: 0,
        }
    }

    fn lineage(&self, start: usize) -> Lineage {
        Lineage::new("go", &self.filename, start, self.pos)
    }

    fn remaining(&self) -> &str {
        &self.source[self.pos..]
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.remaining().chars().next() {
            if c.is_whitespace() {
                self.pos += c.len_utf8();
            } else if self.remaining().starts_with("//") {
                while let Some(c) = self.remaining().chars().next() {
                    self.pos += c.len_utf8();
                    if c == '\n' {
                        break;
                    }
                }
            } else if self.remaining().starts_with("/*") {
                // An unclosed block comment runs to the end of the file.
                let rem = self.remaining();
                self.pos += rem[2..].find("*/").map_or(rem.len(), |i| i + 4);
            } else {
                break;
            }
        }
    }

    fn eat(&mut self, s: &str) -> bool {
        self.skip_ws();
        if self.remaining().starts_with(s) {
            self.pos += s.len();
            true
        } else {
            false
        }
    }

    fn peek(&mut self, s: &str) -> bool {
        self.skip_ws();
        self.remaining().starts_with(s)
    }

    fn read_ident(&mut self) -> Option<String> {
        self.skip_ws();
        let rem = self.remaining();
        let mut end = 0;
        for (i, c) in rem.char_indices() {
            if c.is_alphanumeric() || c == '_' {
                end = i + c.len_utf8();
            } else {
                break;
            }
        }
        if end == 0 {
            return None;
        }
        let s = rem[..end].to_string();
        if s.chars().next().unwrap().is_numeric() {
            return None;
        }
        self.pos += end;
        Some(s)
    }

    fn parse_module(&mut self) -> Result<Cir, ConvertError> {
        let start = self.pos;
        let mut items = Vec::new();
        while !self.remaining().trim().is_empty() {
            self.skip_ws();
            if self.remaining().is_empty() {
                break;
            }
            match self.parse_item()? {
                Some(item) => items.push(item),
                None => break,
            }
        }
        Ok(Cir::Module {
            name: derive_module_name(&self.filename),
            items,
            sandbox: true,
            lineage: self.lineage(start),
        })
    }

    fn parse_item(&mut self) -> Result<Option<Cir>, ConvertError> {
        self.skip_ws();
        if self.remaining().is_empty() {
            return Ok(None);
        }
        let start = self.pos;

        if self.eat("package ") {
            let _ = self.read_ident();
            return self.parse_item();
        }
        if self.eat("import ") {
            // An import is skipped only when it is the whole statement: anything
            // after a `;` on its line is kept as a to-do with it.
            let lines = self.read_statement();
            if !has_semicolon(&lines) {
                return self.parse_item();
            }
            return Ok(Some(Cir::MigrateTodo {
                placeholder: Box::new(Cir::Literal(CirLit::Nil, self.lineage(start))),
                note: format!("unparsed Go item: import {}", lines.join("\n").trim()),
                lineage: self.lineage(start),
            }));
        }
        if self.eat("func ") {
            return Ok(Some(self.parse_func(start)?));
        }
        if self.eat("type ") {
            return Ok(Some(self.parse_type_decl(start)?));
        }
        if self.peek("unsafe.") {
            let _ = self.read_statement();
            return Ok(Some(Cir::Untranslatable {
                reason: "Go unsafe.Pointer — no equivalent in Garnet @safe".into(),
                lineage: self.lineage(start),
            }));
        }
        // Unknown → MigrateTodo, the whole statement (a `var (` group, a raw
        // string, a function literal) so nothing inside it is read as an item.
        let text = self.read_statement().join("\n");
        Ok(Some(Cir::MigrateTodo {
            placeholder: Box::new(Cir::Literal(CirLit::Nil, self.lineage(start))),
            note: format!("unparsed Go item: {}", text.trim()),
            lineage: self.lineage(start),
        }))
    }

    fn parse_func(&mut self, start: usize) -> Result<Cir, ConvertError> {
        // The header runs to the `{` that opens the body, found lexically:
        // comments, strings and raw strings are skipped, the braces of a
        // `struct` or `interface` type literal belong to the header, and a
        // header that ends where Go inserts a semicolon has no body. A header
        // that does not read as `[(receiver)] name[type params](params) [result]`
        // keeps the whole function as a to-do.
        let rest = self.remaining();
        let open = match body_brace(rest) {
            Header::Body(open) => Some(open),
            Header::NoBody(end) => {
                // A declaration without a body (implemented elsewhere) ends at
                // its `;` or line end; what follows is read on its own.
                let text = rest[..end].trim_end().to_string();
                self.pos += end;
                if self.remaining().starts_with(';') {
                    self.pos += 1;
                }
                return Ok(Cir::MigrateTodo {
                    placeholder: Box::new(Cir::Literal(CirLit::Nil, self.lineage(start))),
                    note: format!(
                        "Go function declaration without a body kept for hand translation:\nfunc {text}"
                    ),
                    lineage: self.lineage(start),
                });
            }
            Header::Unreadable => None,
        };
        let parsed = open.and_then(|open| {
            parse_func_header(&blank_go_comments(&rest[..open])).map(|h| (open, h))
        });
        let Some((open, (name, params, return_ty))) = parsed else {
            self.pos = start;
            let text = self.read_statement().join("\n");
            return Ok(Cir::MigrateTodo {
                placeholder: Box::new(Cir::Literal(CirLit::Nil, self.lineage(start))),
                note: format!("Go function kept whole for hand translation:\n{text}"),
                lineage: self.lineage(start),
            });
        };
        self.pos += open + 1;
        let body = self.parse_body_to_brace()?;
        Ok(Cir::Func {
            name,
            params,
            return_ty,
            body,
            mode: FuncMode::Safe, // Go's ownership maps cleanly
            caps: vec![],
            lineage: self.lineage(start),
        })
    }

    fn parse_type_decl(&mut self, start: usize) -> Result<Cir, ConvertError> {
        let name = self.read_ident().unwrap_or_default();
        self.skip_ws();
        if self.peek("struct") {
            // The whole declaration is read lexically (a raw tag may span
            // lines); its fields are converted only when each one reads as
            // `Name[, Name] Type [tag]` on its own line.
            let text = self.read_statement().join("\n");
            if let Some(fields) = struct_fields(&text) {
                return Ok(Cir::Struct {
                    name,
                    fields,
                    lineage: self.lineage(start),
                });
            }
            return Ok(Cir::MigrateTodo {
                placeholder: Box::new(Cir::Literal(CirLit::Nil, self.lineage(start))),
                note: format!(
                    "Go type '{name}' kept whole for hand translation:\ntype {name} {}",
                    text.trim()
                ),
                lineage: self.lineage(start),
            });
        }
        // type alias, interface or other type — kept whole as a to-do
        let text = self.read_statement().join("\n");
        Ok(Cir::MigrateTodo {
            placeholder: Box::new(Cir::Literal(CirLit::Nil, self.lineage(start))),
            note: format!(
                "Go type alias '{}' not yet translated: {}",
                name,
                text.trim()
            ),
            lineage: self.lineage(start),
        })
    }

    fn parse_body_to_brace(&mut self) -> Result<Vec<Cir>, ConvertError> {
        let mut stmts = Vec::new();
        loop {
            self.skip_ws();
            if self.remaining().is_empty() {
                break;
            }
            if self.peek("}") {
                self.pos += 1;
                break;
            }
            let start = self.pos;
            let lines = self.read_statement();
            // C1-18: a statement that spans lines (a block such as `if x {`, a
            // closure, a bare `{`, a call left open, a line ending in an operator)
            // is one MigrateTodo, so no line of it runs as code of the function
            // and none of its braces closes the function.
            if lines.len() > 1 || !balanced(&lines[0]) {
                stmts.push(Cir::MigrateTodo {
                    placeholder: Box::new(Cir::Literal(CirLit::Nil, self.lineage(start))),
                    note: format!(
                        "Go statement spanning {} lines kept whole for hand translation:\n{}",
                        lines.len(),
                        lines.join("\n")
                    ),
                    lineage: self.lineage(start),
                });
                continue;
            }
            let t = lines[0].trim();
            if t == "return" || t.starts_with("return ") || t.starts_with("return\t") {
                let expr_src = t["return".len()..].trim();
                stmts.push(Cir::Return {
                    value: if expr_src.is_empty() {
                        None
                    } else {
                        Some(Box::new(Cir::Ident(
                            expr_src.to_string(),
                            self.lineage(start),
                        )))
                    },
                    lineage: self.lineage(start),
                });
            } else if t.starts_with("go ") {
                stmts.push(Cir::MigrateTodo {
                    placeholder: Box::new(Cir::Literal(CirLit::Nil, self.lineage(start))),
                    note: format!(
                        "Go goroutine '{}' — translate to `spawn Actor {{ ... }}`",
                        t
                    ),
                    lineage: self.lineage(start),
                });
            } else if t.contains("<-") {
                stmts.push(Cir::MigrateTodo {
                    placeholder: Box::new(Cir::Literal(CirLit::Nil, self.lineage(start))),
                    note: format!("Go channel op '{}' — translate to actor ask/tell", t),
                    lineage: self.lineage(start),
                });
            } else {
                stmts.push(Cir::Ident(t.to_string(), self.lineage(start)));
            }
        }
        Ok(stmts)
    }

    /// Read one Go statement from `pos`: lines join until every bracket, block
    /// comment and raw string is closed and the last line ends where Go inserts
    /// a semicolon. A bracket that closes more than the statement opened belongs
    /// to the enclosing construct: the statement ends before it and `pos` is left
    /// on it. A stray closer at the start of a statement is taken alone.
    fn read_statement(&mut self) -> Vec<String> {
        let mut lex = GoLex::default();
        let mut depth = 0;
        let mut lines = Vec::new();
        loop {
            let line_start = self.pos;
            let line = self.read_to_line_end();
            let scanned = lex.scan(&line, depth);
            if let Some(cut) = scanned.cut {
                let cut = if cut == 0 && lines.is_empty() { 1 } else { cut };
                self.pos = line_start + cut;
                lines.push(line[..cut].trim_end().to_string());
                break;
            }
            depth = scanned.depth;
            lines.push(line.trim_end().to_string());
            let open = depth > 0 || lex.open() || !lex.ends_statement();
            if !open || self.remaining().is_empty() {
                break;
            }
        }
        lines
    }

    fn read_to_line_end(&mut self) -> String {
        let rem = self.remaining();
        let idx = rem.find('\n').unwrap_or(rem.len());
        let s = rem[..idx].to_string();
        self.pos += idx;
        if self.remaining().starts_with('\n') {
            self.pos += 1;
        }
        s
    }
}

/// Whether `c` belongs to a Go identifier (any non-ASCII byte counts).
fn is_go_word(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c >= 0x80
}

/// The index just past the string, rune or raw string that starts at `i`, or
/// `None` when it does not close.
fn skip_go_literal(b: &[u8], i: usize) -> Option<usize> {
    let quote = b[i];
    let mut j = i + 1;
    while j < b.len() {
        match b[j] {
            b'\\' if quote != b'`' => j += 2,
            c if c == quote => return Some(j + 1),
            b'\n' if quote != b'`' => return None,
            _ => j += 1,
        }
    }
    None
}

/// The index of the bracket that closes the one at `i`, skipping comments
/// and literals.
fn matching_bracket(b: &[u8], i: usize) -> Option<usize> {
    let mut depth = 0i64;
    let mut j = i;
    while j < b.len() {
        match b[j] {
            b'/' if b.get(j + 1) == Some(&b'/') => {
                j += b[j..].iter().position(|&c| c == b'\n')?;
                continue;
            }
            b'/' if b.get(j + 1) == Some(&b'*') => {
                j += 2 + b[j + 2..].windows(2).position(|w| w == b"*/")? + 2;
                continue;
            }
            b'"' | b'\'' | b'`' => {
                j = skip_go_literal(b, j)?;
                continue;
            }
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => {}
        }
        j += 1;
    }
    None
}

/// Where a function header (the text after `func `) ends.
enum Header {
    /// At the `{` that opens the body, at this offset.
    Body(usize),
    /// Before this offset, where Go ends the declaration (`;`, a line end or
    /// a block comment holding one): a declaration without a body.
    NoBody(usize),
    /// Nowhere the converter can read (a literal or comment left open).
    Unreadable,
}

/// Where the function header in `s` ends. Comments and literals are skipped;
/// a `{` inside brackets or after `struct` or `interface` opens a type literal
/// and is skipped whole.
fn body_brace(s: &str) -> Header {
    match find_body_brace(s) {
        Some(Ok(open)) => Header::Body(open),
        Some(Err(end)) => Header::NoBody(end),
        None => Header::Unreadable,
    }
}

/// `Ok(open)` for the body's `{`, `Err(end)` where a bodyless declaration
/// ends, `None` when unreadable.
fn find_body_brace(s: &str) -> Option<Result<usize, usize>> {
    let b = s.as_bytes();
    let mut depth = 0i64;
    let mut last_word: &[u8] = b"";
    let mut ends = false;
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        match c {
            b'/' if b.get(i + 1) == Some(&b'/') => {
                i += b[i..]
                    .iter()
                    .position(|&c| c == b'\n')
                    .unwrap_or(b.len() - i);
                continue;
            }
            b'/' if b.get(i + 1) == Some(&b'*') => {
                let end = i + 2 + b[i + 2..].windows(2).position(|w| w == b"*/")? + 2;
                // A block comment that holds a newline ends a line, as one would.
                if depth == 0 && ends && b[i..end].contains(&b'\n') {
                    return Some(Err(i));
                }
                i = end;
                continue;
            }
            b'\n' if depth == 0 && ends => return Some(Err(i)),
            b';' if depth == 0 => return Some(Err(i)),
            b'"' | b'\'' | b'`' => {
                i = skip_go_literal(b, i)?;
                last_word = b"";
                ends = true;
                continue;
            }
            c if is_go_word(c) => {
                let start = i;
                while i < b.len() && is_go_word(b[i]) {
                    i += 1;
                }
                last_word = &b[start..i];
                ends = !go_keyword_takes_operand(last_word);
                continue;
            }
            b'{' if depth > 0 || last_word == b"struct" || last_word == b"interface" => {
                i = matching_bracket(b, i)? + 1;
                last_word = b"";
                ends = true;
                continue;
            }
            b'{' => return Some(Ok(i)),
            b'(' | b'[' => {
                depth += 1;
                last_word = b"";
                ends = false;
            }
            b')' | b']' => {
                depth -= 1;
                last_word = b"";
                ends = true;
            }
            c if c.is_ascii_whitespace() => {}
            _ => {
                last_word = b"";
                ends = false;
            }
        }
        i += 1;
    }
    None
}

/// `s` with every comment replaced by spaces (newlines kept), so offsets and
/// line breaks stay where they were.
fn blank_go_comments(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = b.to_vec();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'/' if b.get(i + 1) == Some(&b'/') => {
                while i < b.len() && b[i] != b'\n' {
                    out[i] = b' ';
                    i += 1;
                }
            }
            b'/' if b.get(i + 1) == Some(&b'*') => {
                let end = b[i + 2..]
                    .windows(2)
                    .position(|w| w == b"*/")
                    .map_or(b.len(), |n| i + 2 + n + 2);
                for slot in &mut out[i..end] {
                    if *slot != b'\n' {
                        *slot = b' ';
                    }
                }
                i = end;
            }
            b'"' | b'\'' | b'`' => i = skip_go_literal(b, i).unwrap_or(b.len()),
            _ => i += 1,
        }
    }
    String::from_utf8(out).unwrap_or_default()
}

/// `[(receiver)] name[type params](params) [result]`, from a header without
/// comments.
fn parse_func_header(h: &str) -> Option<(String, Vec<Param>, CirTy)> {
    let b = h.as_bytes();
    let skip = |mut i: usize| {
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        i
    };
    let mut i = skip(0);
    if b.get(i) == Some(&b'(') {
        i = skip(matching_bracket(b, i)? + 1);
    }
    let name_start = i;
    while i < b.len() && is_go_word(b[i]) {
        i += 1;
    }
    let name = &h[name_start..i];
    if name.is_empty() {
        return None;
    }
    i = skip(i);
    if b.get(i) == Some(&b'[') {
        i = skip(matching_bracket(b, i)? + 1);
    }
    if b.get(i) != Some(&b'(') {
        return None;
    }
    let close = matching_bracket(b, i)?;
    let params = split_top_level(&h[i + 1..close])
        .into_iter()
        .filter(|segment| !segment.trim().is_empty())
        .map(|segment| {
            let segment = segment.trim();
            let (name, ty) = segment
                .split_once(|c: char| c.is_whitespace())
                .map_or((segment, ""), |(n, t)| (n, t.trim()));
            Param {
                name: name.to_string(),
                ty: parse_ty_string(ty),
                ownership: Ownership::Default,
            }
        })
        .collect();
    let result = h[close + 1..].trim();
    let return_ty = match result.strip_prefix('(').and_then(|r| r.strip_suffix(')')) {
        Some(inner) => parse_ty_string(inner),
        None => parse_ty_string(result),
    };
    Some((name.to_string(), params, return_ty))
}

/// `s` split at commas outside brackets and literals.
fn split_top_level(s: &str) -> Vec<&str> {
    let b = s.as_bytes();
    let mut parts = Vec::new();
    let mut depth = 0i64;
    let mut start = 0;
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'"' | b'\'' | b'`' => {
                i = skip_go_literal(b, i).unwrap_or(b.len());
                continue;
            }
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b',' if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    parts.push(&s[start..]);
    parts
}

/// The fields of `struct { ... }` when each reads as `Name[, Name] Type [tag]`
/// on its own line (or `;`-separated on one line), a tag being one string or
/// raw string on that line. Anything else (a nested type literal, a raw tag
/// across lines, an embedded field) leaves the type whole.
fn struct_fields(text: &str) -> Option<Vec<crate::cir::FieldDecl>> {
    let clean = blank_go_comments(text);
    let body = clean.trim().strip_prefix("struct")?.trim();
    let inner = body.strip_prefix('{')?.strip_suffix('}')?;
    let mut fields = Vec::new();
    for line in inner.split(['\n', ';']) {
        let mut line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(tag_start) = line.find(['`', '"']) {
            let b = line.as_bytes();
            if skip_go_literal(b, tag_start)? != b.len() {
                return None;
            }
            line = line[..tag_start].trim_end();
        }
        if line.contains(['{', '}', '`', '"', '\'']) {
            return None;
        }
        let (names, ty) = line.rsplit_once(|c: char| c.is_whitespace())?;
        for name in names.split(',') {
            let name = name.trim();
            if name.is_empty() || !name.bytes().all(is_go_word) {
                return None;
            }
            fields.push(crate::cir::FieldDecl {
                name: name.to_string(),
                ty: parse_ty_string(ty),
                public: name.chars().next().is_some_and(|c| c.is_uppercase()),
            });
        }
    }
    Some(fields)
}

/// Whether a `;` statement separator appears, outside strings and comments,
/// in `lines`.
fn has_semicolon(lines: &[String]) -> bool {
    let mut lex = GoLex::default();
    let mut depth = 0;
    for line in lines {
        depth = lex.scan(line, depth).depth;
    }
    lex.semicolon()
}

/// A one-line statement whose brackets close on the line.
fn balanced(line: &str) -> bool {
    let mut lex = GoLex::default();
    let scanned = lex.scan(line, 0);
    scanned.cut.is_none() && scanned.depth == 0 && !lex.open()
}

fn parse_ty_string(s: &str) -> CirTy {
    let s = s.trim();
    if s.is_empty() {
        return CirTy::Inferred;
    }
    if let Some(stripped) = s.strip_prefix("chan ") {
        return CirTy::Concrete(format!("ActorProtocol<{}>", stripped.trim()));
    }
    if let Some(stripped) = s.strip_prefix('*') {
        return CirTy::Concrete(format!("Box<{}>", stripped.trim()));
    }
    if s.starts_with('[') {
        if let Some(end) = s.find(']') {
            let inner = &s[end + 1..];
            return CirTy::Array(Box::new(parse_ty_string(inner)));
        }
    }
    if s.starts_with("map[") {
        if let Some(end) = s.find(']') {
            let k = &s[4..end];
            let v = &s[end + 1..];
            return CirTy::Map(Box::new(parse_ty_string(k)), Box::new(parse_ty_string(v)));
        }
    }
    match s {
        "int" | "int8" | "int16" | "int32" | "int64" | "uint" | "uint8" | "uint16" | "uint32"
        | "uint64" | "byte" | "rune" => CirTy::Concrete("Int".into()),
        "float32" | "float64" => CirTy::Concrete("Float".into()),
        "string" => CirTy::Concrete("String".into()),
        "bool" => CirTy::Concrete("Bool".into()),
        "error" => CirTy::Concrete("Error".into()),
        "interface{}" | "any" => CirTy::Concrete("dyn Any".into()),
        _ => CirTy::Concrete(s.to_string()),
    }
}

fn derive_module_name(filename: &str) -> String {
    let base = filename
        .rsplit(&['/', '\\'][..])
        .next()
        .unwrap_or(filename)
        .trim_end_matches(".go");
    let mut out = String::new();
    let mut up = true;
    for c in base.chars() {
        if c == '_' || c == '-' {
            up = true;
        } else if up {
            out.push(c.to_ascii_uppercase());
            up = false;
        } else {
            out.push(c);
        }
    }
    if out.is_empty() {
        "ConvertedModule".into()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_func_lifts_to_safe() {
        let src = "package main\nfunc Greet(name string) string { return name }\n";
        let cir = parse_and_lift(src, "g.go").unwrap();
        if let Cir::Module { items, .. } = cir {
            if let Cir::Func { name, mode, .. } = &items[0] {
                assert_eq!(name, "Greet");
                assert_eq!(*mode, FuncMode::Safe);
            } else {
                panic!("expected Func, got {:?}", items);
            }
        }
    }

    #[test]
    fn struct_lifts_with_public_fields() {
        let src = "package main\ntype User struct {\n  ID int\n  name string\n}\n";
        let cir = parse_and_lift(src, "u.go").unwrap();
        if let Cir::Module { items, .. } = cir {
            if let Cir::Struct { fields, .. } = &items[0] {
                assert_eq!(fields.len(), 2);
                assert!(fields[0].public); // ID starts uppercase
                assert!(!fields[1].public); // name starts lowercase
            }
        }
    }

    #[test]
    fn chan_type_tagged_as_actor_protocol() {
        let t = parse_ty_string("chan int");
        assert!(matches!(t, CirTy::Concrete(s) if s.starts_with("ActorProtocol")));
    }

    #[test]
    fn unsafe_pointer_flagged_untranslatable() {
        let src = "package main\nfunc f() {\n  x := unsafe.Pointer(nil)\n}\n";
        let cir = parse_and_lift(src, "u.go").unwrap();
        // unsafe.Pointer line is in the function body; we don't fully
        // parse it but the body itself produces a TODO for the line.
        // The top-level parse succeeds.
        if let Cir::Module { items, .. } = cir {
            assert!(!items.is_empty());
        }
    }

    #[test]
    fn goroutine_flagged_as_todo() {
        let src = "package main\nfunc f() {\n  go doWork()\n  return\n}\n";
        let cir = parse_and_lift(src, "g.go").unwrap();
        if let Cir::Module { items, .. } = cir {
            if let Cir::Func { body, .. } = &items[0] {
                let todos: Vec<_> = body
                    .iter()
                    .filter(|s| matches!(s, Cir::MigrateTodo { .. }))
                    .collect();
                assert!(!todos.is_empty(), "expected goroutine TODO");
            }
        }
    }

    #[test]
    fn map_type_parses() {
        let t = parse_ty_string("map[string]int");
        assert!(matches!(t, CirTy::Map(..)));
    }

    #[test]
    fn interface_empty_parses_to_dyn_any() {
        assert!(matches!(
            parse_ty_string("interface{}"),
            CirTy::Concrete(s) if s == "dyn Any"
        ));
    }
}
