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
//!
//! C1-18: a statement is read as a unit of the lexed subset (see `lex`): lines
//! join while a bracket or a keyword block is open, a line ends in an operator
//! or a comma, or the next line starts with `.`. Only a statement on one line
//! becomes code; `EXPR.each do |x| ... end` is lowered; every other statement is
//! kept whole as one MigrateTodo. A file outside the subset is refused.

use super::lex::{ruby_line, ruby_line_after};
use crate::cir::{Cir, CirLit, CirTy, FuncMode, Ownership, Param};
use crate::error::ConvertError;
use crate::lineage::Lineage;

pub fn parse_and_lift(source: &str, filename: &str) -> Result<Cir, ConvertError> {
    refuse_unlexed(source)?;
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
        Self {
            source,
            filename: filename.to_string(),
            pos: 0,
        }
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

    fn todo(&self, start: usize, note: String) -> Cir {
        Cir::MigrateTodo {
            placeholder: Box::new(Cir::Literal(CirLit::Nil, self.lineage(start))),
            note,
            lineage: self.lineage(start),
        }
    }

    fn parse_item(&mut self) -> Result<Option<Cir>, ConvertError> {
        self.skip_ws();
        if self.remaining().is_empty() {
            return Ok(None);
        }
        let start = self.pos;

        if self.peek_keyword("def") {
            return Ok(Some(self.parse_def(start)?));
        }
        if self.peek_keyword("class") {
            return Ok(Some(self.parse_class(start)?));
        }
        if self.peek_keyword("module") {
            return Ok(Some(self.parse_inner_module(start)?));
        }
        let end = self.statement_end(start);
        let text = self.source[start..end].trim_end().to_string();
        self.pos = end;
        let first = first_word(&text);
        // A require is skipped only in its exact form, `require "x"` or
        // `require("x")`; anything else on its line is kept as a to-do with it.
        if text.lines().count() == 1 && is_plain_require(ruby_code(&text)) {
            return self.parse_item();
        }
        if first == "method_missing" {
            return Ok(Some(self.todo(
                start,
                format!(
                    "Ruby method_missing — use Garnet @dynamic per Mini-Spec v1.0 §11.7: {text}"
                ),
            )));
        }
        // An `eval` stands for its statement only as one call with one string
        // literal; anything else in the statement is kept as a to-do.
        if text.lines().count() == 1
            && single_string_call(ruby_code(&text), &["eval", "instance_eval"])
        {
            return Ok(Some(Cir::Untranslatable {
                reason: format!(
                    "Ruby eval / instance_eval — Garnet has no runtime source evaluation: {text}"
                ),
                lineage: self.lineage(start),
            }));
        }
        // Try to recognize an attr_accessor pattern
        if text.starts_with("attr_accessor") {
            return Ok(Some(self.todo(
                start,
                format!(
                    "attr_accessor: {text} — declare as pub struct fields in the enclosing struct"
                ),
            )));
        }
        Ok(Some(
            self.todo(start, format!("unparsed Ruby statement: {text}")),
        ))
    }

    /// The whole statement starting at `start` (a definition the frontend does
    /// not read, or a block it does not lower) as one MigrateTodo carrying its
    /// source lines, dedented, for hand translation.
    fn whole_todo(&mut self, start: usize, kind: &str) -> Cir {
        let end = self.statement_end(start);
        let line_begin = self.source[..start].rfind('\n').map_or(0, |i| i + 1);
        let indent = start - line_begin;
        let text = self.source[line_begin..end]
            .lines()
            .map(|l| {
                l.get(indent.min(l.len() - l.trim_start().len())..)
                    .unwrap_or("")
                    .trim_end()
            })
            .collect::<Vec<_>>()
            .join("\n");
        self.pos = end;
        self.todo(
            start,
            format!(
                "Ruby `{kind}` block kept whole for hand translation:\n{}",
                text.trim_end()
            ),
        )
    }

    /// Whether the rest of the current line holds no code.
    fn rest_of_line_is_blank(&self) -> bool {
        let rest = self.remaining();
        let line = &rest[..rest.find('\n').unwrap_or(rest.len())];
        ruby_code(line).is_empty()
    }

    /// Skip spaces and tabs, but not a line end.
    fn skip_blanks(&mut self) {
        let rest = self.remaining();
        self.pos += rest.len() - rest.trim_start_matches([' ', '\t']).len();
    }

    fn parse_def(&mut self, start: usize) -> Result<Cir, ConvertError> {
        self.pos += "def".len();
        self.skip_blanks();
        let name = self.read_ident().unwrap_or_default();
        self.skip_blanks();
        let params = if self.remaining().starts_with('(') {
            self.pos += 1;
            let ps = self.parse_params()?;
            self.eat(")");
            ps
        } else {
            Vec::new()
        };
        // The header must end here. A receiver (`def self.x`), an operator or
        // setter name, parameters without parentheses, or a body on the header
        // line are not read: the definition is kept whole.
        if name.is_empty() || !self.rest_of_line_is_blank() {
            self.pos = start;
            return Ok(self.whole_todo(start, "def"));
        }
        // The closing `end` must end its line too: `end if cond` or `end; more`
        // makes the definition conditional or shares its line, so it is kept whole.
        let body = match self.parse_body_until_end(true)? {
            Some(body) if self.rest_of_line_is_blank() => body,
            _ => {
                self.pos = start;
                return Ok(self.whole_todo(start, "def"));
            }
        };
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

    /// The statements of a body up to its closing `end`. In a function body
    /// `EXPR.each do |x| ... end` is lowered and `return`/`yield`/`puts` are
    /// recognized; a class or module body keeps every statement as text. `None`
    /// when a clause keyword (`rescue`, `ensure`, `else`, ...) opens a line of
    /// the body: the body is not a plain sequence of statements.
    fn parse_body_until_end(
        &mut self,
        function_body: bool,
    ) -> Result<Option<Vec<Cir>>, ConvertError> {
        let mut stmts = Vec::new();
        loop {
            self.skip_ws();
            if self.peek_keyword("end") {
                self.pos += "end".len();
                break;
            }
            if self.remaining().is_empty() {
                break;
            }
            if ["rescue", "ensure", "else", "elsif", "when", "in", "then"]
                .iter()
                .any(|kw| self.peek_keyword(kw))
            {
                return Ok(None);
            }
            let start = self.pos;
            // A method inside a class body (or a nested def) is parsed as a real
            // function, so its own `end` does not close the enclosing body.
            if self.peek_keyword("def") {
                stmts.push(self.parse_def(start)?);
                continue;
            }
            let end = self.statement_end(start);
            let text = self.source[start..end].trim_end();
            let lines: Vec<&str> = text.lines().collect();
            let header = ruby_code(lines[0]);
            // C1-18: a statement that spans lines (a `do ... end` or keyword
            // block, a brace block, a call left open, a line ending in an
            // operator, a chain continued with `.`) is handled whole.
            // `EXPR.each do |x| ... end` is lowered to `for x in EXPR { ... }`;
            // every other one is kept as one whole-statement MigrateTodo, so
            // neither its body nor its `end` spills into the enclosing def.
            if lines.len() > 1 || !complete(lines[0]) {
                let last = lines.last().map_or("", |l| ruby_code(l));
                if let (true, Some((iter, var))) =
                    (function_body && last == "end", each_header(header))
                {
                    let body_start = start + lines[0].len() + 1;
                    self.pos = body_start.min(end);
                    if let Some(body) = self.parse_body_until_end(true)? {
                        let rest_is_blank = self.pos <= end
                            && self.source[self.pos..end]
                                .lines()
                                .all(|l| ruby_code(l).is_empty());
                        if rest_is_blank {
                            self.pos = end;
                            stmts.push(Cir::For {
                                var,
                                iter: Box::new(Cir::Ident(iter, self.lineage(start))),
                                body,
                                lineage: self.lineage(start),
                            });
                            continue;
                        }
                    }
                }
                self.pos = start;
                let kind = block_kind(header);
                stmts.push(self.whole_todo(start, kind));
                continue;
            }
            self.pos = end;
            let line = lines[0].trim().to_string();
            if !function_body {
                stmts.push(Cir::Ident(line, self.lineage(start)));
            } else if let Some(expr_src) = line.strip_prefix("return ") {
                // Recognize a handful of common forms; everything else is
                // a bare ident expression (fall back to Ident CIR).
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
                    func: Box::new(Cir::Ident("println".to_string(), self.lineage(start))),
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
        Ok(Some(stmts))
    }

    /// One past the end of the statement starting at `from`: lines join while a
    /// bracket or a keyword block is open, a line ends in an operator or a comma,
    /// or the next code line starts with `.` or `&.`. The file was checked
    /// against the lexed subset before parsing.
    fn statement_end(&self, from: usize) -> usize {
        let mut brackets = 0;
        let mut blocks = 0;
        let mut continues = false;
        // A method name is due at the start of the next code line (`obj.`).
        let mut name_pending = false;
        // The operands of `alias` or `undef` continue on the next code line.
        let mut names_pending = false;
        let mut pos = from;
        for line in self.source[from..].split_inclusive('\n') {
            pos += line.len();
            match ruby_line_after(
                line.trim_end_matches(['\n', '\r']),
                name_pending,
                names_pending,
            ) {
                // A blank or comment line neither opens nor closes anything, and
                // does not end a statement that is still open.
                Ok(l) if l.code.trim().is_empty() => {
                    if brackets > 0 || blocks > 0 || continues {
                        continue;
                    }
                }
                Ok(l) => {
                    brackets += l.brackets;
                    blocks += l.blocks;
                    continues = l.continues;
                    name_pending = l.name_pending;
                    names_pending = l.names_pending;
                    if brackets > 0 || blocks > 0 || continues {
                        continue;
                    }
                }
                Err(_) => {}
            }
            let chained = self.source[pos..]
                .lines()
                .map(ruby_code)
                .find(|code| !code.is_empty())
                .is_some_and(|code| {
                    (code.starts_with('.') && !code.starts_with("..")) || code.starts_with("&.")
                });
            if !chained {
                break;
            }
        }
        pos
    }

    fn parse_class(&mut self, start: usize) -> Result<Cir, ConvertError> {
        self.pos += "class".len();
        self.skip_blanks();
        let name = self.read_ident().unwrap_or_default();
        // Skip optional superclass `< Base` (a constant path such as `A::B`)
        self.skip_blanks();
        if self.remaining().starts_with('<') && !self.remaining().starts_with("<<") {
            self.pos += 1;
            self.skip_blanks();
            let rest = self.remaining();
            self.pos += rest.len()
                - rest
                    .trim_start_matches(|c: char| c.is_alphanumeric() || c == '_' || c == ':')
                    .len();
        }
        // `class << self`, a superclass expression or code after the header is
        // not read: the class is kept whole.
        if name.is_empty() || !self.rest_of_line_is_blank() {
            self.pos = start;
            return Ok(self.whole_todo(start, "class"));
        }
        // Body contains attr_accessor / def / instance variables
        let body = match self.parse_body_until_end(false)? {
            Some(body) if self.rest_of_line_is_blank() => body,
            _ => {
                self.pos = start;
                return Ok(self.whole_todo(start, "class"));
            }
        };
        // Emit a struct + impl pair (Phase 2F finding)
        let lineage = self.lineage(start);
        // Hoist `def`s into an Impl and `attr_accessor` names into fields; every
        // other class-level statement is kept as a todo, not dropped.
        let mut fields = Vec::new();
        let mut impl_methods = Vec::new();
        let mut kept = Vec::new();
        for m in body {
            match m {
                Cir::Func { .. } => impl_methods.push(m),
                Cir::Ident(text, _) if text.starts_with("attr_accessor") => {
                    for accessor in ruby_code(&text).split(',') {
                        let name = accessor
                            .trim()
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
                Cir::Ident(text, lin) => kept.push(Cir::MigrateTodo {
                    placeholder: Box::new(Cir::Literal(CirLit::Nil, lin.clone())),
                    note: format!("Ruby class-level statement: {text}"),
                    lineage: lin,
                }),
                // A class body holds only definitions, todos and statement text.
                other => kept.push(other),
            }
        }
        // Emit as a Module wrapping Struct + Impl; the outer Module's
        // items vec absorbs these.
        let mut items = vec![
            Cir::Struct {
                name: name.clone(),
                fields,
                lineage: lineage.clone(),
            },
            Cir::Impl {
                target: name.clone(),
                methods: impl_methods,
                lineage: lineage.clone(),
            },
        ];
        items.extend(kept);
        Ok(Cir::Module {
            name,
            items,
            sandbox: false,
            lineage,
        })
    }

    fn parse_inner_module(&mut self, start: usize) -> Result<Cir, ConvertError> {
        self.pos += "module".len();
        self.skip_blanks();
        let name = self.read_ident().unwrap_or_default();
        if name.is_empty() || !self.rest_of_line_is_blank() {
            self.pos = start;
            return Ok(self.whole_todo(start, "module"));
        }
        let body = match self.parse_body_until_end(false)? {
            Some(body) if self.rest_of_line_is_blank() => body,
            _ => {
                self.pos = start;
                return Ok(self.whole_todo(start, "module"));
            }
        };
        let items = body
            .into_iter()
            .map(|item| match item {
                Cir::Ident(text, lin) => Cir::MigrateTodo {
                    placeholder: Box::new(Cir::Literal(CirLit::Nil, lin.clone())),
                    note: format!("Ruby module-level statement: {text}"),
                    lineage: lin,
                },
                other => other,
            })
            .collect();
        Ok(Cir::Module {
            name,
            items,
            sandbox: false,
            lineage: self.lineage(start),
        })
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

/// The code of a Ruby line without its trailing `# comment`, trimmed. The file
/// was checked by `refuse_unlexed`, so every line is inside the lexed subset.
fn ruby_code(line: &str) -> &str {
    ruby_line(line).map_or_else(|_| line.trim(), |l| l.code.trim())
}

/// A one-line statement that opens nothing it does not close.
fn complete(line: &str) -> bool {
    ruby_line(line).is_ok_and(|l| l.brackets == 0 && l.blocks == 0 && !l.continues)
}

/// `require "path"`, `require_relative 'path'` or the same with parentheses,
/// and nothing else.
fn is_plain_require(code: &str) -> bool {
    single_string_call(code, &["require_relative", "require"])
}

/// `name "literal"` or `name("literal")` for one of `names`, and nothing else:
/// the literal holds no quote, `#` or backslash.
fn single_string_call(code: &str, names: &[&str]) -> bool {
    let rest = names.iter().find_map(|name| {
        code.strip_prefix(name)
            .filter(|rest| !rest.starts_with(|c: char| c.is_alphanumeric() || c == '_'))
    });
    let Some(rest) = rest else {
        return false;
    };
    let arg = match rest.strip_prefix('(') {
        Some(inner) => match inner.trim_end().strip_suffix(')') {
            Some(arg) => arg.trim(),
            None => return false,
        },
        None if rest.starts_with([' ', '\t']) => rest.trim(),
        None => return false,
    };
    let b = arg.as_bytes();
    b.len() >= 2
        && matches!(b[0], b'"' | b'\'')
        && b[b.len() - 1] == b[0]
        && !arg[1..arg.len() - 1].contains(['"', '\'', '#', '\\'])
}

/// The leading identifier of a statement (`require` in `require "x"`).
fn first_word(text: &str) -> &str {
    let end = text
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(text.len());
    &text[..end]
}

/// Refuse a file the Ruby frontend cannot lex: a construct outside the subset
/// `lex` reads could hide a block or a statement boundary, and the converter
/// would then leave part of a statement active.
fn refuse_unlexed(source: &str) -> Result<(), ConvertError> {
    for (n, line) in source.lines().enumerate() {
        if let Err(why) = ruby_line(line) {
            return Err(ConvertError::ParseError {
                source_lang: "ruby".into(),
                message: format!(
                    "line {} uses {why}, which the converter does not lex; convert this file by hand",
                    n + 1
                ),
            });
        }
    }
    Ok(())
}

/// What kind of statement a header line starts, for the todo note: its leading
/// keyword, `do` for a `do` block, or `statement`.
fn block_kind(code: &str) -> &'static str {
    let first = code.split_whitespace().next().unwrap_or("");
    [
        "if", "unless", "while", "until", "case", "begin", "for", "def", "class", "module",
    ]
    .into_iter()
    .find(|k| *k == first)
    .unwrap_or(if code.split_whitespace().any(|w| w == "do") {
        "do"
    } else {
        "statement"
    })
}

/// `EXPR.each do |NAME|` → `(EXPR, NAME)`.
fn each_header(line: &str) -> Option<(String, String)> {
    let (head, params) = line.strip_suffix('|')?.split_once(" do |")?;
    let iter = head.strip_suffix(".each")?.trim();
    let var = params.trim();
    let valid = !iter.is_empty()
        && var
            .chars()
            .next()
            .is_some_and(|c| c == '_' || c.is_ascii_alphabetic())
        && var.chars().all(|c| c == '_' || c.is_ascii_alphanumeric());
    valid.then(|| (iter.to_string(), var.to_string()))
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
