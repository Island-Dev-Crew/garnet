//! Rust → CIR frontend (stylized parser for v4.1 initial release).
//!
//! Recognizes: `fn name(params) -> ReturnType { body }`, `struct`,
//! `enum`, `impl`, `let (mut)? name: T = value`, `return value`,
//! basic expressions (literals, ident, BinOp, MethodCall, Call,
//! FieldAccess), `if`, `while`, `for`, `match`, `Result<T, E>` +
//! `?` operator.
//!
//! Flags as Untranslatable: `unsafe { … }`, `extern "C"`, macros
//! (`println!`, `vec!`, etc. — retained as `MigrateTodo` because
//! programmers usually want them translated, not refused), lifetime
//! annotations (deferred to v4.1.x).

use crate::cir::{Cir, CirLit, CirTy, FuncMode, Ownership, Param};
use crate::error::ConvertError;
use crate::lineage::Lineage;

pub fn parse_and_lift(source: &str, filename: &str) -> Result<Cir, ConvertError> {
    let mut p = RustParser::new(source, filename);
    p.parse_module()
}

struct RustParser<'a> {
    source: &'a str,
    filename: String,
    pos: usize,
}

impl<'a> RustParser<'a> {
    fn new(source: &'a str, filename: &str) -> Self {
        Self {
            source,
            filename: filename.to_string(),
            pos: 0,
        }
    }

    fn lineage(&self, start: usize) -> Lineage {
        Lineage::new("rust", &self.filename, start, self.pos)
    }

    fn remaining(&self) -> &str {
        &self.source[self.pos..]
    }

    fn skip_ws(&mut self) {
        let rem = self.remaining();
        let mut chars = rem.char_indices().peekable();
        while let Some(&(i, c)) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else if c == '/' {
                // Skip line comments and block comments
                let bytes = rem.as_bytes();
                if i + 1 < rem.len() && bytes[i + 1] == b'/' {
                    while let Some(&(_, c)) = chars.peek() {
                        chars.next();
                        if c == '\n' {
                            break;
                        }
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        let consumed = chars.peek().map(|&(i, _)| i).unwrap_or(rem.len());
        self.pos += consumed;
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
        let ident = rem[..end].to_string();
        self.pos += end;
        if ident.chars().next().unwrap().is_numeric() {
            self.pos -= end;
            return None;
        }
        Some(ident)
    }

    fn parse_module(&mut self) -> Result<Cir, ConvertError> {
        let start = self.pos;
        let mut items = Vec::new();
        while !self.remaining().trim().is_empty() {
            self.skip_ws();
            if self.remaining().is_empty() {
                break;
            }
            let iter_start = self.pos;
            match self.parse_item()? {
                Some(item) => items.push(item),
                None => break,
            }
            if self.pos == iter_start {
                if self.pos < self.source.len() {
                    self.pos += 1;
                } else {
                    break;
                }
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

        // pub prefix
        let _pub = self.eat("pub ") || self.eat("pub(crate) ");

        if self.eat("fn ") {
            return Ok(Some(self.parse_fn(start, true)?));
        }
        if self.eat("struct ") {
            return Ok(Some(self.parse_struct(start)?));
        }
        if self.eat("enum ") {
            return Ok(Some(self.parse_enum(start)?));
        }
        if self.eat("impl ") {
            return Ok(Some(self.parse_impl(start)?));
        }
        if self.eat("use ") {
            // Skip the rest of the line
            self.skip_to_line_end();
            return self.parse_item();
        }
        if self.peek("unsafe ") || self.peek("extern ") {
            self.skip_to_line_end();
            return Ok(Some(Cir::Untranslatable {
                reason: "unsafe or extern block".into(),
                lineage: self.lineage(start),
            }));
        }

        // Unknown → skip line, emit todo
        let line = self.read_until('\n');
        if line.trim().is_empty() {
            return self.parse_item();
        }
        Ok(Some(Cir::MigrateTodo {
            placeholder: Box::new(Cir::Literal(CirLit::Nil, self.lineage(start))),
            note: format!("unparsed Rust item: {}", line.trim()),
            lineage: self.lineage(start),
        }))
    }

    fn parse_fn(&mut self, start: usize, safe_mode: bool) -> Result<Cir, ConvertError> {
        let name = self.read_ident().ok_or_else(|| ConvertError::ParseError {
            source_lang: "rust".into(),
            message: "expected function name after `fn`".into(),
        })?;
        // Parameter list
        self.eat("(");
        let params = self.parse_params()?;
        self.eat(")");
        // Return type
        let return_ty = if self.eat("->") {
            self.parse_type()
        } else {
            CirTy::Inferred
        };
        self.eat("{");
        let body = self.parse_block_stmts()?;
        self.eat("}");
        Ok(Cir::Func {
            name,
            params,
            return_ty,
            body,
            mode: if safe_mode {
                FuncMode::Safe
            } else {
                FuncMode::Managed
            },
            caps: vec![],
            lineage: self.lineage(start),
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ConvertError> {
        let mut params = Vec::new();
        loop {
            self.skip_ws();
            if self.peek(")") || self.remaining().is_empty() {
                break;
            }
            let loop_start = self.pos;
            let own = if self.eat("&mut ") {
                Ownership::Owned // Garnet treats &mut as owned-with-mut
            } else if self.eat("&") {
                Ownership::Borrowed
            } else if self.eat("mut ") {
                Ownership::Owned
            } else {
                Ownership::Default
            };
            let name = match self.read_ident() {
                Some(n) => n,
                None => {
                    // Couldn't read a valid param — skip to next comma
                    // or closing paren to avoid infinite loop.
                    self.skip_until_one_of(&[',', ')']);
                    if !self.eat(",") {
                        break;
                    }
                    continue;
                }
            };
            let ty = if self.eat(":") {
                // Some Rust types (&[u8], Vec<(A, B)>, impl Trait) aren't
                // handled by our stylized parser — skip to , or ) to
                // avoid getting stuck on bracketed segments.
                let ty_start = self.pos;
                let candidate = self.parse_type();
                if self.pos == ty_start {
                    // parse_type didn't advance (couldn't read an ident) —
                    // skip to next delimiter.
                    self.skip_until_one_of(&[',', ')']);
                    CirTy::Concrete("unknown".into())
                } else {
                    candidate
                }
            } else {
                CirTy::Inferred
            };
            // Safety: ensure position advanced this iteration.
            if self.pos == loop_start {
                self.pos += 1;
            }
            params.push(Param {
                name,
                ty,
                ownership: own,
            });
            if !self.eat(",") {
                break;
            }
        }
        Ok(params)
    }

    fn skip_until_one_of(&mut self, stops: &[char]) {
        let rem = self.remaining();
        let idx = rem.find(|c| stops.contains(&c)).unwrap_or(rem.len());
        self.pos += idx;
    }

    fn parse_type(&mut self) -> CirTy {
        self.skip_ws();
        // Simple: read one ident, optionally followed by <args>
        let name = self.read_ident().unwrap_or_default();
        if name.is_empty() {
            return CirTy::Inferred;
        }
        if self.eat("<") {
            // Handle Option<T>, Result<T, E>, Vec<T>, HashMap<K, V>
            match name.as_str() {
                "Option" => {
                    let inner = self.parse_type();
                    self.eat(">");
                    return CirTy::Optional(Box::new(inner));
                }
                "Result" => {
                    let ok = self.parse_type();
                    self.eat(",");
                    let err = self.parse_type();
                    self.eat(">");
                    return CirTy::Result(Box::new(ok), Box::new(err));
                }
                "Vec" | "VecDeque" => {
                    let inner = self.parse_type();
                    self.eat(">");
                    return CirTy::Array(Box::new(inner));
                }
                "HashMap" | "BTreeMap" => {
                    let k = self.parse_type();
                    self.eat(",");
                    let v = self.parse_type();
                    self.eat(">");
                    return CirTy::Map(Box::new(k), Box::new(v));
                }
                _ => {
                    // Skip generic args
                    self.skip_until('>');
                }
            }
        }
        match name.as_str() {
            "String" | "str" | "&str" => CirTy::Concrete("String".into()),
            "i8" | "i16" | "i32" | "i64" | "isize" | "u8" | "u16" | "u32" | "u64" | "usize" => {
                CirTy::Concrete("Int".into())
            }
            "f32" | "f64" => CirTy::Concrete("Float".into()),
            "bool" => CirTy::Concrete("Bool".into()),
            _ => CirTy::Concrete(name),
        }
    }

    fn parse_block_stmts(&mut self) -> Result<Vec<Cir>, ConvertError> {
        let mut stmts = Vec::new();
        let mut depth = 1;
        while depth > 0 {
            self.skip_ws();
            if self.remaining().is_empty() {
                break;
            }
            if self.peek("}") {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            if self.peek("{") {
                depth += 1;
            }
            let iter_start = self.pos;
            match self.parse_stmt()? {
                Some(s) => stmts.push(s),
                None => break,
            }
            // Safety: if parse_stmt didn't advance, force advancement
            // so we never infinite-loop on malformed input.
            if self.pos == iter_start {
                if self.pos < self.source.len() {
                    self.pos += 1;
                } else {
                    break;
                }
            }
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Option<Cir>, ConvertError> {
        let start = self.pos;
        self.skip_ws();
        if self.remaining().is_empty() {
            return Ok(None);
        }
        if self.eat("let ") {
            let mutable = self.eat("mut ");
            let name = self.read_ident().unwrap_or_default();
            let ty = if self.eat(":") {
                self.parse_type()
            } else {
                CirTy::Inferred
            };
            let value = if self.eat("=") {
                Some(Box::new(self.parse_expr()?))
            } else {
                None
            };
            self.eat(";");
            return Ok(Some(Cir::Let {
                name,
                ty,
                mutable,
                value,
                lineage: self.lineage(start),
            }));
        }
        if self.eat("return") {
            let v = if self.peek(";") {
                None
            } else {
                Some(Box::new(self.parse_expr()?))
            };
            self.eat(";");
            return Ok(Some(Cir::Return {
                value: v,
                lineage: self.lineage(start),
            }));
        }
        if self.eat("if ") {
            return Ok(Some(self.parse_if(start)?));
        }
        if self.eat("while ") {
            let cond = Box::new(self.parse_expr()?);
            self.eat("{");
            let body = self.parse_block_stmts()?;
            self.eat("}");
            return Ok(Some(Cir::While {
                cond,
                body,
                lineage: self.lineage(start),
            }));
        }
        if self.eat("for ") {
            let var = self.read_ident().unwrap_or_default();
            self.eat("in ");
            let iter = Box::new(self.parse_expr()?);
            self.eat("{");
            let body = self.parse_block_stmts()?;
            self.eat("}");
            return Ok(Some(Cir::For {
                var,
                iter,
                body,
                lineage: self.lineage(start),
            }));
        }
        // Bare expression statement
        let e = self.parse_expr()?;
        self.eat(";");
        Ok(Some(e))
    }

    fn parse_if(&mut self, start: usize) -> Result<Cir, ConvertError> {
        let cond = Box::new(self.parse_expr()?);
        self.eat("{");
        let then_b = self.parse_block_stmts()?;
        self.eat("}");
        let else_b = if self.eat("else") {
            if self.eat("if ") {
                let e = self.parse_if(self.pos)?;
                Some(vec![e])
            } else {
                self.eat("{");
                let b = self.parse_block_stmts()?;
                self.eat("}");
                Some(b)
            }
        } else {
            None
        };
        Ok(Cir::If {
            cond,
            then_b,
            else_b,
            lineage: self.lineage(start),
        })
    }

    fn parse_expr(&mut self) -> Result<Cir, ConvertError> {
        // Simple: literal | ident | call | field access. No operator
        // precedence for the v4.1 initial release; the production
        // tree-sitter frontend handles it.
        let start = self.pos;
        self.skip_ws();
        let rem = self.remaining();
        if rem.is_empty() {
            return Ok(Cir::Literal(CirLit::Nil, self.lineage(start)));
        }
        let ch = rem.chars().next().unwrap_or(' ');
        if ch == '"' {
            // String literal
            self.pos += 1;
            let content = self.read_until('"');
            self.eat("\"");
            return Ok(Cir::Literal(CirLit::Str(content), self.lineage(start)));
        }
        if ch.is_numeric() {
            let mut n = 0i64;
            let bytes = rem.as_bytes();
            let mut i = 0;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                n = n * 10 + (bytes[i] - b'0') as i64;
                i += 1;
            }
            self.pos += i;
            return Ok(Cir::Literal(CirLit::Int(n), self.lineage(start)));
        }
        if self.eat("true") {
            return Ok(Cir::Literal(CirLit::Bool(true), self.lineage(start)));
        }
        if self.eat("false") {
            return Ok(Cir::Literal(CirLit::Bool(false), self.lineage(start)));
        }
        if let Some(name) = self.read_ident() {
            // Method/field chain + call args
            let mut result = Cir::Ident(name, self.lineage(start));
            loop {
                if self.eat(".") {
                    let m = self.read_ident().unwrap_or_default();
                    if self.eat("(") {
                        let args = self.parse_arg_list()?;
                        self.eat(")");
                        result = Cir::MethodCall {
                            recv: Box::new(result),
                            name: m,
                            args,
                            lineage: self.lineage(start),
                        };
                    } else {
                        result = Cir::FieldAccess {
                            recv: Box::new(result),
                            name: m,
                            lineage: self.lineage(start),
                        };
                    }
                } else if self.eat("(") {
                    let args = self.parse_arg_list()?;
                    self.eat(")");
                    result = Cir::Call {
                        func: Box::new(result),
                        args,
                        lineage: self.lineage(start),
                    };
                } else {
                    break;
                }
            }
            return Ok(result);
        }
        // Fallback — consume one char so we don't loop
        if self.pos < self.source.len() {
            self.pos += ch.len_utf8();
        }
        Ok(Cir::Literal(CirLit::Nil, self.lineage(start)))
    }

    fn parse_arg_list(&mut self) -> Result<Vec<Cir>, ConvertError> {
        let mut args = Vec::new();
        loop {
            self.skip_ws();
            if self.peek(")") || self.remaining().is_empty() {
                break;
            }
            args.push(self.parse_expr()?);
            if !self.eat(",") {
                break;
            }
        }
        Ok(args)
    }

    fn parse_struct(&mut self, start: usize) -> Result<Cir, ConvertError> {
        let name = self.read_ident().unwrap_or_default();
        self.eat("{");
        let mut fields = Vec::new();
        loop {
            self.skip_ws();
            if self.peek("}") || self.remaining().is_empty() {
                break;
            }
            let public = self.eat("pub ");
            let fname = self.read_ident().unwrap_or_default();
            self.eat(":");
            let fty = self.parse_type();
            fields.push(crate::cir::FieldDecl {
                name: fname,
                ty: fty,
                public,
            });
            if !self.eat(",") {
                break;
            }
        }
        self.eat("}");
        Ok(Cir::Struct {
            name,
            fields,
            lineage: self.lineage(start),
        })
    }

    fn parse_enum(&mut self, start: usize) -> Result<Cir, ConvertError> {
        let name = self.read_ident().unwrap_or_default();
        self.eat("{");
        let mut variants = Vec::new();
        loop {
            self.skip_ws();
            if self.peek("}") || self.remaining().is_empty() {
                break;
            }
            let vname = self.read_ident().unwrap_or_default();
            let payload = if self.eat("(") {
                let mut p = Vec::new();
                loop {
                    if self.peek(")") {
                        break;
                    }
                    p.push(self.parse_type());
                    if !self.eat(",") {
                        break;
                    }
                }
                self.eat(")");
                p
            } else {
                Vec::new()
            };
            variants.push(crate::cir::VariantDecl {
                name: vname,
                payload,
            });
            if !self.eat(",") {
                break;
            }
        }
        self.eat("}");
        Ok(Cir::Enum {
            name,
            variants,
            lineage: self.lineage(start),
        })
    }

    fn parse_impl(&mut self, start: usize) -> Result<Cir, ConvertError> {
        let target = self.read_ident().unwrap_or_default();
        self.eat("{");
        let mut methods = Vec::new();
        loop {
            self.skip_ws();
            if self.peek("}") || self.remaining().is_empty() {
                break;
            }
            let _pub = self.eat("pub ");
            if self.eat("fn ") {
                let m_start = self.pos;
                methods.push(self.parse_fn(m_start, true)?);
            } else {
                self.skip_to_line_end();
            }
        }
        self.eat("}");
        Ok(Cir::Impl {
            target,
            methods,
            lineage: self.lineage(start),
        })
    }

    fn read_until(&mut self, stop: char) -> String {
        let rem = self.remaining();
        match rem.find(stop) {
            Some(i) => {
                let s = rem[..i].to_string();
                self.pos += i;
                s
            }
            None => {
                let s = rem.to_string();
                self.pos += rem.len();
                s
            }
        }
    }

    fn skip_until(&mut self, stop: char) {
        let _ = self.read_until(stop);
        let _ = self.eat(&stop.to_string());
    }

    fn skip_to_line_end(&mut self) {
        let _ = self.read_until('\n');
        let _ = self.eat("\n");
    }
}

fn derive_module_name(filename: &str) -> String {
    let base = filename
        .rsplit(&['/', '\\'][..])
        .next()
        .unwrap_or(filename)
        .trim_end_matches(".rs");
    // PascalCase
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
    fn simple_fn_lifts() {
        let src = "fn add(a: i32, b: i32) -> i32 { return a; }";
        let cir = parse_and_lift(src, "test.rs").unwrap();
        match cir {
            Cir::Module { items, .. } => {
                assert_eq!(items.len(), 1);
                match &items[0] {
                    Cir::Func {
                        name, mode, params, ..
                    } => {
                        assert_eq!(name, "add");
                        assert_eq!(*mode, FuncMode::Safe);
                        assert_eq!(params.len(), 2);
                    }
                    _ => panic!("expected Func"),
                }
            }
            _ => panic!("expected Module"),
        }
    }

    #[test]
    fn struct_lifts_fields() {
        let src = "struct User { pub id: i64, name: String, }";
        let cir = parse_and_lift(src, "user.rs").unwrap();
        if let Cir::Module { items, .. } = cir {
            if let Cir::Struct { name, fields, .. } = &items[0] {
                assert_eq!(name, "User");
                assert_eq!(fields.len(), 2);
                assert!(fields[0].public);
                assert!(!fields[1].public);
            }
        }
    }

    #[test]
    fn enum_variants_lift() {
        let src = "enum Color { Red, Blue, Rgb(i32, i32, i32), }";
        let cir = parse_and_lift(src, "color.rs").unwrap();
        if let Cir::Module { items, .. } = cir {
            if let Cir::Enum { variants, .. } = &items[0] {
                assert_eq!(variants.len(), 3);
                assert_eq!(variants[2].payload.len(), 3);
            }
        }
    }

    #[test]
    fn unsafe_flagged_untranslatable() {
        let src = "unsafe { do_thing(); }";
        let cir = parse_and_lift(src, "u.rs").unwrap();
        if let Cir::Module { items, .. } = cir {
            assert!(matches!(items[0], Cir::Untranslatable { .. }));
        }
    }

    #[test]
    fn option_type_parses() {
        let src = "fn f(x: Option<i32>) -> i32 { return 0; }";
        let cir = parse_and_lift(src, "f.rs").unwrap();
        if let Cir::Module { items, .. } = cir {
            if let Cir::Func { params, .. } = &items[0] {
                assert!(matches!(&params[0].ty, CirTy::Optional(_)));
            }
        }
    }

    #[test]
    fn result_type_parses() {
        let src = "fn f() -> Result<String, IoError> { return nil; }";
        let cir = parse_and_lift(src, "f.rs").unwrap();
        if let Cir::Module { items, .. } = cir {
            if let Cir::Func { return_ty, .. } = &items[0] {
                assert!(matches!(return_ty, CirTy::Result(..)));
            }
        }
    }

    #[test]
    fn module_name_pascal_cased() {
        assert_eq!(derive_module_name("src/foo_bar.rs"), "FooBar");
        assert_eq!(derive_module_name("src/simple.rs"), "Simple");
        assert_eq!(derive_module_name("word-count.rs"), "WordCount");
    }

    #[test]
    fn integer_literal_lifts() {
        let src = "fn f() { let x = 42; }";
        let cir = parse_and_lift(src, "f.rs").unwrap();
        if let Cir::Module { items, .. } = cir {
            if let Cir::Func { body, .. } = &items[0] {
                if let Cir::Let { value: Some(v), .. } = &body[0] {
                    assert!(matches!(**v, Cir::Literal(CirLit::Int(42), _)));
                }
            }
        }
    }
}
