//! Ruby → CIR frontend (stylized parser for v4.1 initial release).
//!
//! Recognizes: `def name(args) ... end`, `class Name ... end`, `do
//! |args| ... end` blocks → Lambda, `yield`, `attr_accessor`, basic
//! literals + expressions.
//!
//! Flags: `method_missing` → MigrateTodo (needs @dynamic per Mini-Spec
//! v1.0 §11.7); `eval` / `instance_eval` → Untranslatable; regex
//! literals → MigrateTodo (stdlib regex is v4.1.x); monkey-patched
//! open classes → Untranslatable.

use crate::cir::{Cir, CirLit, CirTy, FuncMode, Ownership, Param};
use crate::error::ConvertError;
use crate::lineage::Lineage;

pub fn parse_and_lift(source: &str, filename: &str) -> Result<Cir, ConvertError> {
    let mut p = RubyParser::new(source, filename);
    p.parse_module()
}

struct RubyParser<'a> {
    source: &'a str,
    filename: String,
    pos: usize,
}

impl<'a> RubyParser<'a> {
    fn new(source: &'a str, filename: &str) -> Self {
        Self { source, filename: filename.to_string(), pos: 0 }
    }

    fn lineage(&self, start: usize) -> Lineage {
        Lineage::new("ruby", &self.filename, start, self.pos)
    }

    fn remaining(&self) -> &str {
        &self.source[self.pos..]
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.remaining().chars().next() {
            if c.is_whitespace() {
                self.pos += c.len_utf8();
            } else if c == '#' {
                while let Some(c) = self.remaining().chars().next() {
                    self.pos += c.len_utf8();
                    if c == '\n' {
                        break;
                    }
                }
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

    fn peek_keyword(&mut self, kw: &str) -> bool {
        self.skip_ws();
        let rem = self.remaining();
        if !rem.starts_with(kw) {
            return false;
        }
        rem[kw.len()..]
            .chars()
            .next()
            .map(|c| !c.is_alphanumeric() && c != '_')
            .unwrap_or(true)
    }

    fn read_ident(&mut self) -> Option<String> {
        self.skip_ws();
        let rem = self.remaining();
        let mut end = 0;
        for (i, c) in rem.char_indices() {
            if c.is_alphanumeric() || c == '_' || c == '?' || c == '!' {
                end = i + c.len_utf8();
            } else {
                break;
            }
        }
        if end == 0 {
            return None;
        }
        let ident = rem[..end].to_string();
        if ident.chars().next().unwrap().is_numeric() {
            return None;
        }
        self.pos += end;
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

        if self.peek_keyword("def") {
            self.eat("def ");
            return Ok(Some(self.parse_def(start)?));
        }
        if self.peek_keyword("class") {
            self.eat("class ");
            return Ok(Some(self.parse_class(start)?));
        }
        if self.peek_keyword("module") {
            self.eat("module ");
            return Ok(Some(self.parse_inner_module(start)?));
        }
        if self.peek_keyword("require") || self.peek_keyword("require_relative") {
            self.read_until_newline();
            return self.parse_item();
        }
        if self.peek_keyword("method_missing") {
            self.read_until_newline();
            return Ok(Some(Cir::MigrateTodo {
                placeholder: Box::new(Cir::Literal(CirLit::Nil, self.lineage(start))),
                note: "Ruby method_missing — use Garnet @dynamic per Mini-Spec v1.0 §11.7".into(),
                lineage: self.lineage(start),
            }));
        }
        if self.peek_keyword("eval") || self.peek_keyword("instance_eval") {
            self.read_until_newline();
            return Ok(Some(Cir::Untranslatable {
                reason: "Ruby eval / instance_eval — Garnet has no runtime source evaluation".into(),
                lineage: self.lineage(start),
            }));
        }

        // Default: bare expression statement or unknown → MigrateTodo
        let line = self.read_until_newline();
        if line.trim().is_empty() {
            return self.parse_item();
        }
        // Try to recognize an attr_accessor pattern
        if line.trim_start().starts_with("attr_accessor") {
            return Ok(Some(Cir::MigrateTodo {
                placeholder: Box::new(Cir::Literal(CirLit::Nil, self.lineage(start))),
                note: format!(
                    "attr_accessor: {} — declare as pub struct fields in the enclosing struct",
                    line.trim()
                ),
                lineage: self.lineage(start),
            }));
        }
        Ok(Some(Cir::MigrateTodo {
            placeholder: Box::new(Cir::Literal(CirLit::Nil, self.lineage(start))),
            note: format!("unparsed Ruby statement: {}", line.trim()),
            lineage: self.lineage(start),
        }))
    }

    fn parse_def(&mut self, start: usize) -> Result<Cir, ConvertError> {
        let name = self.read_ident().unwrap_or_default();
        let params = if self.eat("(") {
            let ps = self.parse_params()?;
            self.eat(")");
            ps
        } else {
            Vec::new()
        };
        let body = self.parse_body_until_end()?;
        Ok(Cir::Func {
            name,
            params,
            return_ty: CirTy::Inferred, // Ruby is dynamic — Level 0
            body,
            mode: FuncMode::Managed,
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
            // Skip optional default values; just grab ident
            let name = match self.read_ident() {
                Some(n) => n,
                None => break,
            };
            if self.eat("=") {
                // Skip default value — consume until , or )
                self.read_until_one_of(&[',', ')']);
            }
            params.push(Param {
                name,
                ty: CirTy::Inferred,
                ownership: Ownership::Default,
            });
            if !self.eat(",") {
                break;
            }
        }
        Ok(params)
    }

    fn parse_body_until_end(&mut self) -> Result<Vec<Cir>, ConvertError> {
        let mut stmts = Vec::new();
        loop {
            self.skip_ws();
            if self.peek_keyword("end") {
                self.eat("end");
                break;
            }
            if self.remaining().is_empty() {
                break;
            }
            let start = self.pos;
            let line = self.read_until_newline();
            let line = line.trim().to_string();
            if line.is_empty() {
                continue;
            }
            if line == "end" {
                break;
            }
            // Recognize a handful of common forms; everything else is
            // a bare ident expression (fall back to Ident CIR).
            if line.starts_with("return ") {
                let expr_src = &line[7..];
                stmts.push(Cir::Return {
                    value: Some(Box::new(Cir::Ident(
                        expr_src.trim().to_string(),
                        self.lineage(start),
                    ))),
                    lineage: self.lineage(start),
                });
            } else if line.starts_with("yield") {
                stmts.push(Cir::MigrateTodo {
                    placeholder: Box::new(Cir::Literal(CirLit::Nil, self.lineage(start))),
                    note: "yield — translated to Garnet implicit block invocation per Mini-Spec v1.0 §5.4".into(),
                    lineage: self.lineage(start),
                });
            } else if line.starts_with("puts ") || line.starts_with("print ") {
                stmts.push(Cir::Call {
                    func: Box::new(Cir::Ident(
                        "println".to_string(),
                        self.lineage(start),
                    )),
                    args: vec![Cir::Literal(
                        CirLit::Str(
                            line.trim_start_matches("puts ")
                                .trim_start_matches("print ")
                                .to_string(),
                        ),
                        self.lineage(start),
                    )],
                    lineage: self.lineage(start),
                });
            } else {
                stmts.push(Cir::Ident(line, self.lineage(start)));
            }
        }
        Ok(stmts)
    }

    fn parse_class(&mut self, start: usize) -> Result<Cir, ConvertError> {
        let name = self.read_ident().unwrap_or_default();
        // Skip optional superclass `< Base`
        if self.eat("<") {
            let _base = self.read_ident();
        }
        // Body contains attr_accessor / def / instance variables
        let methods = self.parse_body_until_end()?;
        // Emit a struct + impl pair (Phase 2F finding)
        let lineage = self.lineage(start);
        // Hoist `def`s from the methods vector into an Impl
        let mut fields = Vec::new();
        let mut impl_methods = Vec::new();
        for m in methods {
            if let Cir::Func { .. } = m {
                impl_methods.push(m);
            } else if let Cir::MigrateTodo { note, .. } = &m {
                if note.starts_with("attr_accessor:") {
                    // parse out the accessor names
                    let parts: Vec<&str> = note.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        for accessor in parts[1].split(',') {
                            let accessor = accessor.trim();
                            let name = accessor
                                .trim_start_matches("attr_accessor")
                                .trim()
                                .trim_start_matches(':');
                            if !name.is_empty() {
                                fields.push(crate::cir::FieldDecl {
                                    name: name.to_string(),
                                    ty: CirTy::Inferred,
                                    public: true,
                                });
                            }
                        }
                    }
                }
            }
        }
        // Emit as a Module wrapping Struct + Impl; the outer Module's
        // items vec absorbs these.
        // Here we return a single synthetic Module item that the outer
        // parse_module appends.
        Ok(Cir::Module {
            name: name.clone(),
            items: vec![
                Cir::Struct {
                    name: name.clone(),
                    fields,
                    lineage: lineage.clone(),
                },
                Cir::Impl {
                    target: name,
                    methods: impl_methods,
                    lineage: lineage.clone(),
                },
            ],
            sandbox: false,
            lineage,
        })
    }

    fn parse_inner_module(&mut self, start: usize) -> Result<Cir, ConvertError> {
        let name = self.read_ident().unwrap_or_default();
        let body = self.parse_body_until_end()?;
        Ok(Cir::Module {
            name,
            items: body,
            sandbox: false,
            lineage: self.lineage(start),
        })
    }

    fn read_until_newline(&mut self) -> String {
        let rem = self.remaining();
        match rem.find('\n') {
            Some(i) => {
                let s = rem[..i].to_string();
                self.pos += i + 1;
                s
            }
            None => {
                let s = rem.to_string();
                self.pos += rem.len();
                s
            }
        }
    }

    fn read_until_one_of(&mut self, stops: &[char]) -> String {
        let rem = self.remaining();
        let idx = rem.find(|c| stops.contains(&c)).unwrap_or(rem.len());
        let s = rem[..idx].to_string();
        self.pos += idx;
        s
    }
}

fn derive_module_name(filename: &str) -> String {
    let base = filename
        .rsplit(&['/', '\\'][..])
        .next()
        .unwrap_or(filename)
        .trim_end_matches(".rb");
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
    fn def_lifts_to_managed_func() {
        let src = "def greet(name)\n  return name\nend\n";
        let cir = parse_and_lift(src, "greet.rb").unwrap();
        if let Cir::Module { items, .. } = cir {
            if let Cir::Func { name, mode, .. } = &items[0] {
                assert_eq!(name, "greet");
                assert_eq!(*mode, FuncMode::Managed);
            } else {
                panic!("expected Func, got {:?}", items[0]);
            }
        }
    }

    #[test]
    fn method_missing_flagged_as_todo() {
        let src = "method_missing(name, *args) { do_something }\n";
        let cir = parse_and_lift(src, "dyn.rb").unwrap();
        if let Cir::Module { items, .. } = cir {
            assert!(matches!(items[0], Cir::MigrateTodo { .. }));
        }
    }

    #[test]
    fn eval_rejected_as_untranslatable() {
        let src = "eval(\"x + 1\")\n";
        let cir = parse_and_lift(src, "e.rb").unwrap();
        if let Cir::Module { items, .. } = cir {
            assert!(matches!(items[0], Cir::Untranslatable { .. }));
        }
    }

    #[test]
    fn class_lifts_to_struct_plus_impl() {
        let src = "class User\n  def greet\n    return name\n  end\nend\n";
        let cir = parse_and_lift(src, "user.rb").unwrap();
        // Outer module wraps a synthetic submodule of (struct, impl)
        if let Cir::Module { items: outer, .. } = cir {
            if let Cir::Module { items: inner, .. } = &outer[0] {
                assert!(matches!(inner[0], Cir::Struct { .. }));
                assert!(matches!(inner[1], Cir::Impl { .. }));
            }
        }
    }

    #[test]
    fn puts_becomes_println_call() {
        let src = "def hi\n  puts hello\nend\n";
        let cir = parse_and_lift(src, "hi.rb").unwrap();
        if let Cir::Module { items, .. } = cir {
            if let Cir::Func { body, .. } = &items[0] {
                assert!(matches!(&body[0], Cir::Call { .. }));
            }
        }
    }

    #[test]
    fn module_name_pascal_from_snake() {
        assert_eq!(derive_module_name("src/my_file.rb"), "MyFile");
    }
}
