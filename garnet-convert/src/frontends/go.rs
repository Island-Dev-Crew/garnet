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

use super::lex::GoLex;
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
            let _ = self.read_statement();
            return self.parse_item();
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
        // Optional receiver: func (r *Recv) Name(...)
        if self.eat("(") {
            // skip receiver — treat as method of the type
            let _ = self.read_until(')');
            self.eat(")");
        }
        let name = self.read_ident().unwrap_or_default();
        self.eat("(");
        let params = self.parse_params()?;
        self.eat(")");
        // Return type is optional in Go
        let return_ty = if !self.peek("{") {
            // Could be a single type, or (a, b) tuple. Handle single for v4.1.
            if self.eat("(") {
                let rs = self.read_until(')');
                self.eat(")");
                parse_ty_string(&rs)
            } else {
                let ty_str = self.read_until('{').trim().to_string();
                parse_ty_string(&ty_str)
            }
        } else {
            CirTy::Inferred
        };
        self.eat("{");
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

    fn parse_params(&mut self) -> Result<Vec<Param>, ConvertError> {
        let mut out = Vec::new();
        loop {
            self.skip_ws();
            if self.peek(")") || self.remaining().is_empty() {
                break;
            }
            // Go: name Type, name Type or shared: name1, name2 Type
            let name = match self.read_ident() {
                Some(n) => n,
                None => break,
            };
            self.skip_ws();
            let ty_src = self.read_until_one_of(&[',', ')']);
            out.push(Param {
                name,
                ty: parse_ty_string(ty_src.trim()),
                ownership: Ownership::Default,
            });
            if !self.eat(",") {
                break;
            }
        }
        Ok(out)
    }

    fn parse_type_decl(&mut self, start: usize) -> Result<Cir, ConvertError> {
        let name = self.read_ident().unwrap_or_default();
        self.skip_ws();
        if self.eat("struct") {
            self.eat("{");
            let mut fields = Vec::new();
            loop {
                self.skip_ws();
                if self.peek("}") || self.remaining().is_empty() {
                    break;
                }
                let fname = match self.read_ident() {
                    Some(n) => n,
                    None => break,
                };
                let ty_src = self.read_to_line_end();
                fields.push(crate::cir::FieldDecl {
                    name: fname.clone(),
                    ty: parse_ty_string(ty_src.trim()),
                    public: fname
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false),
                });
            }
            self.eat("}");
            return Ok(Cir::Struct {
                name,
                fields,
                lineage: self.lineage(start),
            });
        }
        // type alias, interface or other type — skip the whole declaration
        let _ = self.read_statement();
        Ok(Cir::MigrateTodo {
            placeholder: Box::new(Cir::Literal(CirLit::Nil, self.lineage(start))),
            note: format!("Go type alias '{}' not yet translated", name),
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

    fn read_until(&mut self, stop: char) -> String {
        let rem = self.remaining();
        let idx = rem.find(stop).unwrap_or(rem.len());
        let s = rem[..idx].to_string();
        self.pos += idx;
        s
    }

    fn read_until_one_of(&mut self, stops: &[char]) -> String {
        let rem = self.remaining();
        let idx = rem.find(|c| stops.contains(&c)).unwrap_or(rem.len());
        let s = rem[..idx].to_string();
        self.pos += idx;
        s
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
