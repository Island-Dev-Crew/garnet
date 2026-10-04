//! Python → CIR frontend (stylized parser for v4.1 initial release).
//!
//! Recognizes: `def name(args):`, `class Name:`, type hints (`x: int`),
//! `return`, common literals, `if`/`elif`/`else`, `for`/`while`,
//! f-strings (translated to Garnet `#{}` interpolation).
//!
//! Flags: decorators (`@decorator`) → MigrateTodo (closure-wrap pattern);
//! `eval`/`exec` → Untranslatable; `*args`/`**kwargs` → MigrateTodo
//! (Garnet has fixed arity).
//!
//! C1-18: the unit is the logical line (see `lex`). A compound statement is its
//! header plus every deeper-indented logical line and its `elif`/`else`/`except`/
//! `finally` clauses. Only a simple statement on one physical line becomes code;
//! simple `for`/`while` loops are lowered; every other statement is kept whole as
//! one MigrateTodo. Tab indentation and constructs `lex` does not read are refused.

use super::lex::PyLex;
use crate::cir::{Cir, CirLit, CirTy, FuncMode, Ownership, Param};
use crate::error::ConvertError;
use crate::lineage::Lineage;

pub fn parse_and_lift(source: &str, filename: &str) -> Result<Cir, ConvertError> {
    let mut p = PythonParser::new(source, filename)?;
    p.parse_module()
}

struct PythonParser<'a> {
    lines: Vec<&'a str>,
    /// For the first physical line of a logical line: one past its last.
    ends: Vec<usize>,
    /// Each physical line's code: its length without a trailing comment.
    code_len: Vec<usize>,
    /// Each physical line holds a `;` outside strings and comments.
    semicolon: Vec<bool>,
    filename: String,
    line_idx: usize,
    global_byte_offset: Vec<usize>, // per-line byte offset within source
}

/// The refusal for a file the frontend does not read.
fn refuse(line: usize, why: &str) -> ConvertError {
    ConvertError::ParseError {
        source_lang: "python".into(),
        message: format!(
            "line {} uses {why}, which the converter does not lex; convert this file by hand",
            line + 1
        ),
    }
}

impl<'a> PythonParser<'a> {
    fn new(source: &'a str, filename: &str) -> Result<Self, ConvertError> {
        // Python ends a line at a bare carriage return too; the converter splits
        // lines at `\n` (and `\r\n`) only, so a bare `\r` is refused.
        if let Some(at) = source
            .bytes()
            .enumerate()
            .position(|(i, c)| c == b'\r' && source.as_bytes().get(i + 1) != Some(&b'\n'))
        {
            let line = source[..at].matches('\n').count();
            return Err(refuse(line, "a carriage return without a line feed"));
        }
        let lines: Vec<&str> = source.lines().collect();
        let mut offsets = Vec::with_capacity(lines.len());
        let mut off = 0;
        for l in &lines {
            offsets.push(off);
            off += l.len() + 1; // +1 for newline
        }
        let (ends, code_len, semicolon) = logical_lines(&lines)?;
        Ok(Self {
            lines,
            ends,
            code_len,
            semicolon,
            filename: filename.to_string(),
            line_idx: 0,
            global_byte_offset: offsets,
        })
    }

    fn lineage(&self, start_line: usize) -> Lineage {
        let start = self
            .global_byte_offset
            .get(start_line)
            .copied()
            .unwrap_or(0);
        let end = self
            .global_byte_offset
            .get(self.line_idx)
            .copied()
            .unwrap_or(start);
        Lineage::new("python", &self.filename, start, end)
    }

    /// One past the last physical line of the logical line starting at `start`.
    fn end_of(&self, start: usize) -> usize {
        self.ends.get(start).copied().unwrap_or(0).max(start + 1)
    }

    /// The code of the logical line starting at `start`: its physical lines
    /// without comments or a `\` continuation, trimmed and joined by a space.
    /// Empty for a blank or comment-only line.
    fn code_of(&self, start: usize) -> String {
        let end = self.end_of(start).min(self.lines.len());
        (start..end)
            .map(|i| {
                let code = self.lines[i][..self.code_len[i]].trim();
                match code.strip_suffix('\\') {
                    Some(joined) if i + 1 < end => joined.trim_end(),
                    _ => code,
                }
            })
            .filter(|code| !code.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// The source lines `start..end`, dedented by `indent`, for hand translation.
    fn text(&self, start: usize, indent: usize, end: usize) -> String {
        self.lines[start..end]
            .iter()
            .map(|l| {
                l.get(indent.min(leading_indent(l))..)
                    .unwrap_or("")
                    .trim_end()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn todo(&self, start: usize, note: String) -> Cir {
        Cir::MigrateTodo {
            placeholder: Box::new(Cir::Literal(CirLit::Nil, self.lineage(start))),
            note,
            lineage: self.lineage(start),
        }
    }

    fn parse_module(&mut self) -> Result<Cir, ConvertError> {
        let start = self.line_idx;
        let mut items = Vec::new();
        while self.line_idx < self.lines.len() {
            let at = self.line_idx;
            if self.code_of(at).is_empty() {
                self.line_idx = self.end_of(at);
                continue;
            }
            if leading_indent(self.lines[at]) > 0 {
                return Err(ConvertError::ParseError {
                    source_lang: "python".into(),
                    message: format!(
                        "line {} is indented where a statement starts at the margin; convert this file by hand",
                        at + 1
                    ),
                });
            }
            if let Some(item) = self.parse_item()? {
                items.push(item);
            }
        }
        Ok(Cir::Module {
            name: derive_module_name(&self.filename),
            items,
            sandbox: true,
            lineage: self.lineage(start),
        })
    }

    /// One module-level statement. Always moves past it.
    fn parse_item(&mut self) -> Result<Option<Cir>, ConvertError> {
        let start = self.line_idx;
        let code = self.code_of(start);
        if code.starts_with('@') {
            return Ok(Some(self.decorated_todo(start, 0)));
        }
        if code.starts_with("def ") && code.ends_with(':') {
            return Ok(Some(self.parse_def(start, 0)?));
        }
        if code.starts_with("class ") && code.ends_with(':') {
            return Ok(Some(self.parse_class(start, 0)?));
        }
        if let Some(keyword) = header_keyword(&code) {
            return Ok(Some(self.compound_todo(keyword, start, 0)));
        }
        let end = self.end_of(start);
        self.line_idx = end;
        // An import is skipped only when it is the whole statement: anything after
        // a `;` on its line is kept as a to-do with it.
        let statement_separator = (start..end).any(|i| self.semicolon[i]);
        if (code.starts_with("import ") || code.starts_with("from ")) && !statement_separator {
            return Ok(None);
        }
        if code.starts_with("eval(") || code.starts_with("exec(") {
            return Ok(Some(Cir::Untranslatable {
                reason: "Python eval/exec — Garnet has no runtime source evaluation".into(),
                lineage: self.lineage(start),
            }));
        }
        // Top-level statement (rare at module level except for main
        // guard). Consume as MigrateTodo.
        let text = self.text(start, 0, end);
        Ok(Some(self.todo(
            start,
            format!("Python top-level statement: {text}"),
        )))
    }

    fn parse_def(&mut self, start_line: usize, parent_indent: usize) -> Result<Cir, ConvertError> {
        // def name(params) -> ReturnType:  (the header may span lines)
        let header = self.code_of(start_line);
        let rest = &header[4..]; // after "def "
        let paren = rest.find('(').ok_or_else(|| ConvertError::ParseError {
            source_lang: "python".into(),
            message: format!("def without open paren: {header}"),
        })?;
        self.line_idx = self.end_of(start_line);
        let name = rest[..paren].trim().to_string();
        let after_paren = &rest[paren + 1..];
        let closing = after_paren.rfind("):").or_else(|| after_paren.rfind(')'));
        let params_src = match closing {
            Some(i) => &after_paren[..i],
            None => after_paren,
        };
        let params = parse_params(params_src);
        // Extract return type if present
        let return_ty = if let Some(arrow) = header.find("->") {
            let tail = &header[arrow + 2..];
            let colon = tail.find(':').unwrap_or(tail.len());
            parse_ty(tail[..colon].trim())
        } else {
            CirTy::Inferred
        };

        // Body: lines more indented than parent_indent
        let body = self.parse_indented_body(parent_indent, true);

        Ok(Cir::Func {
            name,
            params,
            return_ty,
            body,
            mode: FuncMode::Managed,
            caps: vec![],
            lineage: self.lineage(start_line),
        })
    }

    fn parse_class(
        &mut self,
        start_line: usize,
        parent_indent: usize,
    ) -> Result<Cir, ConvertError> {
        let header = self.code_of(start_line);
        self.line_idx = self.end_of(start_line);
        let rest = &header[6..]; // after "class "
        let name = rest
            .split(['(', ':'])
            .next()
            .unwrap_or("Anon")
            .trim()
            .to_string();

        // A class body is not a function body: a loop at class level is kept
        // whole and a statement stays text.
        let body = self.parse_indented_body(parent_indent, false);
        // Split body into fields (from __init__) + methods; every other
        // class-level statement is kept as a todo, not dropped.
        let mut methods = Vec::new();
        let mut fields = Vec::new();
        let mut kept = Vec::new();
        for b in body {
            match b {
                Cir::Func {
                    name: ref fname,
                    body: ref fb,
                    ..
                } if fname == "__init__" => {
                    // Try to extract self.x = y assignments as fields; the
                    // constructor itself is kept as a method, not dropped.
                    for s in fb {
                        if let Cir::Assign { lhs, .. } = s {
                            if let Cir::FieldAccess { name: fname, .. } = &**lhs {
                                fields.push(crate::cir::FieldDecl {
                                    name: fname.clone(),
                                    ty: CirTy::Inferred,
                                    public: true,
                                });
                            }
                        }
                    }
                    methods.push(b);
                }
                Cir::Func { .. } => methods.push(b),
                Cir::Ident(text, lineage) => kept.push(Cir::MigrateTodo {
                    placeholder: Box::new(Cir::Literal(CirLit::Nil, lineage.clone())),
                    note: format!("Python class-level statement: {text}"),
                    lineage,
                }),
                // A class body holds only definitions, todos and statement text.
                other => kept.push(other),
            }
        }

        let mut items = vec![
            Cir::Struct {
                name: name.clone(),
                fields,
                lineage: self.lineage(start_line),
            },
            Cir::Impl {
                target: name.clone(),
                methods,
                lineage: self.lineage(start_line),
            },
        ];
        items.extend(kept);
        Ok(Cir::Module {
            name,
            items,
            sandbox: false,
            lineage: self.lineage(start_line),
        })
    }

    /// The statements of a body indented deeper than `parent_indent`. In a
    /// function body simple `for`/`while` loops are lowered to brace form and
    /// simple statements become code; a class body keeps them as text.
    fn parse_indented_body(&mut self, parent_indent: usize, function_body: bool) -> Vec<Cir> {
        let mut body = Vec::new();
        while self.line_idx < self.lines.len() {
            let start = self.line_idx;
            let code = self.code_of(start);
            if code.is_empty() {
                self.line_idx = self.end_of(start);
                continue;
            }
            let indent = leading_indent(self.lines[start]);
            if indent <= parent_indent {
                break;
            }
            if code.starts_with('@') {
                body.push(self.decorated_todo(start, indent));
                continue;
            }
            // Recognize nested defs (methods inside a class body)
            if code.starts_with("def ") && code.ends_with(':') {
                match self.parse_def(start, indent) {
                    Ok(f) => {
                        body.push(f);
                        continue;
                    }
                    Err(_) => self.line_idx = start,
                }
            }
            // A compound statement (`for x in xs:`, `with ...:`, `if x: y` ...) is
            // handled whole. Simple `for`/`while` loops are lowered to brace form;
            // every other one is kept as one whole-statement MigrateTodo, so its
            // body is never flattened into the enclosing function.
            if let Some(keyword) = header_keyword(&code) {
                let one_line_header = self.end_of(start) == start + 1;
                if function_body && one_line_header && code.ends_with(':') {
                    let end = self.block_end(start, indent);
                    if let Some(lowered) = self.lower_loop(keyword, &code, start, indent, end) {
                        body.push(lowered);
                        continue;
                    }
                }
                body.push(self.compound_todo(keyword, start, indent));
                continue;
            }
            // A statement that spans physical lines is kept whole as one
            // MigrateTodo, so no fragment of it becomes active code.
            let end = self.end_of(start);
            if end > start + 1 {
                body.push(self.statement_todo(start, indent, end));
                self.line_idx = end;
                continue;
            }
            // A simple statement on one physical line. Its trailing comment
            // travels with the code.
            let line = self.lines[start].trim();
            if !function_body {
                body.push(Cir::Ident(line.to_string(), self.lineage(start)));
            } else if code == "return" || code.starts_with("return ") {
                let expr_src = code["return".len()..].trim();
                body.push(Cir::Return {
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
            } else if code.contains(" = ") && code.starts_with("self.") {
                // self.x = y — recognize as field write
                let parts: Vec<&str> = code.splitn(2, " = ").collect();
                let field = parts[0].trim_start_matches("self.").to_string();
                body.push(Cir::Assign {
                    lhs: Box::new(Cir::FieldAccess {
                        recv: Box::new(Cir::Ident("self".into(), self.lineage(start))),
                        name: field,
                        lineage: self.lineage(start),
                    }),
                    rhs: Box::new(Cir::Ident(parts[1].to_string(), self.lineage(start))),
                    lineage: self.lineage(start),
                });
            } else {
                body.push(Cir::Ident(line.to_string(), self.lineage(start)));
            }
            self.line_idx += 1;
        }
        body
    }

    /// One past the last line of the block whose header starts at `header`
    /// (indent `indent`): every following logical line indented deeper, blank
    /// lines between them, and any `elif`/`else`/`except`/`finally` clause at the
    /// header's indent. Trailing blank lines belong to whatever follows.
    fn block_end(&self, header: usize, indent: usize) -> usize {
        let mut idx = self.end_of(header);
        let mut end = idx;
        while idx < self.lines.len() {
            let code = self.code_of(idx);
            let next = self.end_of(idx);
            if !code.is_empty() {
                let line_indent = leading_indent(self.lines[idx]);
                let clause = line_indent == indent && is_continuation_clause(&code);
                if line_indent <= indent && !clause {
                    break;
                }
                end = next;
            }
            idx = next;
        }
        end
    }

    /// A compound statement the converter does not lower, kept whole: its
    /// header, its indented body, and its `elif`/`else`/`except`/`finally`
    /// clauses, whether or not a suite sits on the header line.
    fn compound_todo(&mut self, keyword: &str, start: usize, indent: usize) -> Cir {
        let end = self.block_end(start, indent);
        self.line_idx = end;
        self.block_todo(keyword, start, indent, end)
    }

    /// Decorators and the definition they decorate, body included, kept whole.
    fn decorated_todo(&mut self, start: usize, indent: usize) -> Cir {
        let mut def = start;
        while def < self.lines.len() && {
            let code = self.code_of(def);
            code.is_empty() || code.starts_with('@')
        } {
            def = self.end_of(def);
        }
        let end = if def >= self.lines.len() {
            def
        } else if self.code_of(def).ends_with(':') {
            self.block_end(def, indent)
        } else {
            self.end_of(def)
        };
        self.line_idx = end;
        let first = self.lines[start].trim();
        let text = self.text(start, indent, end);
        self.todo(
            start,
            format!(
                "Python decorator: {first} — convert to explicit closure-wrap; the decorated definition is kept whole for hand translation:\n{text}"
            ),
        )
    }

    /// Lower `for NAME in EXPR:` and `while COND:` (with no `else:` clause) to
    /// brace form, converting the body recursively.
    fn lower_loop(
        &mut self,
        keyword: &str,
        header: &str,
        start: usize,
        indent: usize,
        end: usize,
    ) -> Option<Cir> {
        if !matches!(keyword, "for" | "while") {
            return None;
        }
        // A loop `else:` clause has no brace form; keep the block whole.
        let mut idx = self.end_of(start);
        while idx < end {
            if leading_indent(self.lines[idx]) == indent && !self.code_of(idx).is_empty() {
                return None;
            }
            idx = self.end_of(idx);
        }
        let head = header.strip_suffix(':')?.trim_end();
        let node = if keyword == "for" {
            let rest = head.strip_prefix("for ")?;
            let (var, iter) = rest.split_once(" in ")?;
            let var = var.trim();
            if !is_identifier(var) {
                return None;
            }
            self.line_idx = self.end_of(start);
            let body = self.parse_indented_body(indent, true);
            Cir::For {
                var: var.to_string(),
                iter: Box::new(Cir::Ident(iter.trim().to_string(), self.lineage(start))),
                body,
                lineage: self.lineage(start),
            }
        } else {
            let cond = head.strip_prefix("while ")?.trim();
            self.line_idx = self.end_of(start);
            let body = self.parse_indented_body(indent, true);
            Cir::While {
                cond: Box::new(Cir::Ident(cond.to_string(), self.lineage(start))),
                body,
                lineage: self.lineage(start),
            }
        };
        self.line_idx = end;
        Some(node)
    }

    /// A statement spanning `start..end` kept whole, with its source lines
    /// (dedented to its first line) for hand translation.
    fn statement_todo(&self, start: usize, indent: usize, end: usize) -> Cir {
        let text = self.text(start, indent, end);
        self.todo(
            start,
            format!(
                "Python statement spanning {} lines kept whole for hand translation:\n{text}",
                end - start
            ),
        )
    }

    /// The whole block, header and body, as one MigrateTodo whose note carries
    /// the source lines (dedented to the header) for hand translation.
    fn block_todo(&self, keyword: &str, start: usize, indent: usize, end: usize) -> Cir {
        let text = self.text(start, indent, end);
        self.todo(
            start,
            format!("Python `{keyword}` block kept whole for hand translation:\n{text}"),
        )
    }
}

/// Split the file into logical lines with `lex::PyLex`: for each logical line's
/// first physical line, one past its last; each physical line's code length; and
/// whether each physical line holds a `;` statement separator.
/// Refuses tab or form-feed indentation, a construct the lexer does not read, and a string,
/// bracket or `\` continuation still open at the end of the file.
type LogicalLines = (Vec<usize>, Vec<usize>, Vec<bool>);

fn logical_lines(lines: &[&str]) -> Result<LogicalLines, ConvertError> {
    let mut ends = vec![0; lines.len()];
    let mut code_len = vec![0; lines.len()];
    let mut semicolon = vec![false; lines.len()];
    let mut lex = PyLex::default();
    let mut i = 0;
    while i < lines.len() {
        let start = i;
        let line = lines[i];
        let indentation = &line[..line.len() - line.trim_start().len()];
        if !line.trim().is_empty() {
            if indentation.contains('\t') {
                return Err(refuse(i, "tab indentation"));
            }
            if indentation.contains('\x0c') {
                return Err(refuse(i, "a form feed in its indentation"));
            }
        }
        let mut depth = 0i64;
        loop {
            let scanned = lex.scan(lines[i]).map_err(|why| refuse(i, why))?;
            code_len[i] = scanned.comment_at.unwrap_or(lines[i].len());
            semicolon[i] = scanned.semicolon;
            depth += scanned.delta;
            i += 1;
            if depth <= 0 && !scanned.continues && !lex.open() {
                break;
            }
            if i == lines.len() {
                let why = if lex.open() {
                    "a triple-quoted string that never closes"
                } else {
                    "a bracket or continuation that never closes"
                };
                return Err(refuse(start, why));
            }
        }
        ends[start] = i;
    }
    Ok((ends, code_len, semicolon))
}

/// The keyword of a compound statement, whether its header ends on this line
/// (`if x:`), spans lines (`if (`), or carries its body (`if x: y`). `match` and
/// `case` are soft keywords: they open a statement only when it ends in `:`.
fn header_keyword(code: &str) -> Option<&'static str> {
    // A keyword is the statement's whole leading identifier: whatever follows it
    // (a space, a tab, `(`, `{`, `*`, `:` ...) is not part of it.
    let word_end = code
        .find(|c: char| !(c.is_alphanumeric() || c == '_' || !c.is_ascii()))
        .unwrap_or(code.len());
    let word = &code[..word_end];
    [
        "for", "while", "with", "if", "elif", "else", "try", "except", "finally", "async", "class",
        "def",
    ]
    .into_iter()
    .find(|kw| *kw == word)
    .or_else(|| {
        ["match", "case"]
            .into_iter()
            .find(|kw| code.ends_with(':') && *kw == word)
    })
}

fn is_continuation_clause(code: &str) -> bool {
    matches!(
        header_keyword(code),
        Some("elif" | "else" | "except" | "finally")
    )
}

fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c == '_' || c.is_ascii_alphabetic())
        && chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

fn leading_indent(s: &str) -> usize {
    s.chars().take_while(|c| *c == ' ').count()
}

fn parse_params(s: &str) -> Vec<Param> {
    let mut out = Vec::new();
    for p in s.split(',') {
        let p = p.trim();
        if p.is_empty() || p == "self" {
            continue;
        }
        if p.starts_with("**") || p.starts_with('*') {
            // Varargs
            continue;
        }
        let (name, ty) = match p.find(':') {
            Some(i) => (p[..i].trim().to_string(), parse_ty(p[i + 1..].trim())),
            None => (p.to_string(), CirTy::Inferred),
        };
        // Strip default value if present
        let name = name.split('=').next().unwrap_or(&name).trim().to_string();
        out.push(Param {
            name,
            ty,
            ownership: Ownership::Default,
        });
    }
    out
}

fn parse_ty(s: &str) -> CirTy {
    let s = s.trim();
    if s.is_empty() {
        return CirTy::Inferred;
    }
    if s.starts_with("Optional[") && s.ends_with(']') {
        let inner = &s[9..s.len() - 1];
        return CirTy::Optional(Box::new(parse_ty(inner)));
    }
    if s.starts_with("List[") && s.ends_with(']') {
        let inner = &s[5..s.len() - 1];
        return CirTy::Array(Box::new(parse_ty(inner)));
    }
    if s.starts_with("Dict[") && s.ends_with(']') {
        let inner = &s[5..s.len() - 1];
        let parts: Vec<&str> = inner.splitn(2, ',').collect();
        if parts.len() == 2 {
            return CirTy::Map(Box::new(parse_ty(parts[0])), Box::new(parse_ty(parts[1])));
        }
    }
    match s {
        "int" => CirTy::Concrete("Int".into()),
        "float" => CirTy::Concrete("Float".into()),
        "str" => CirTy::Concrete("String".into()),
        "bool" => CirTy::Concrete("Bool".into()),
        "None" => CirTy::Concrete("Nil".into()),
        "bytes" => CirTy::Concrete("Bytes".into()),
        _ => CirTy::Concrete(s.to_string()),
    }
}

fn derive_module_name(filename: &str) -> String {
    let base = filename
        .rsplit(&['/', '\\'][..])
        .next()
        .unwrap_or(filename)
        .trim_end_matches(".py");
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
    fn simple_def_lifts() {
        let src = "def greet(name: str) -> str:\n    return name\n";
        let cir = parse_and_lift(src, "g.py").unwrap();
        if let Cir::Module { items, .. } = cir {
            if let Cir::Func {
                name,
                params,
                return_ty,
                mode,
                ..
            } = &items[0]
            {
                assert_eq!(name, "greet");
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].name, "name");
                assert!(matches!(return_ty, CirTy::Concrete(s) if s == "String"));
                assert_eq!(*mode, FuncMode::Managed);
            } else {
                panic!("expected Func");
            }
        }
    }

    #[test]
    fn decorator_flagged_as_todo() {
        let src = "@cached\ndef f():\n    return 1\n";
        let cir = parse_and_lift(src, "d.py").unwrap();
        if let Cir::Module { items, .. } = cir {
            assert!(matches!(items[0], Cir::MigrateTodo { .. }));
        }
    }

    #[test]
    fn eval_rejected() {
        let src = "eval(\"1 + 1\")\n";
        let cir = parse_and_lift(src, "e.py").unwrap();
        if let Cir::Module { items, .. } = cir {
            assert!(matches!(items[0], Cir::Untranslatable { .. }));
        }
    }

    #[test]
    fn class_produces_struct_and_impl() {
        let src = "class User:\n    def __init__(self, name):\n        self.name = name\n    def greet(self):\n        return self.name\n";
        let cir = parse_and_lift(src, "u.py").unwrap();
        if let Cir::Module { items: outer, .. } = cir {
            if let Cir::Module { items: inner, .. } = &outer[0] {
                assert!(matches!(inner[0], Cir::Struct { .. }));
                assert!(matches!(inner[1], Cir::Impl { .. }));
            }
        }
    }

    #[test]
    fn optional_type_parses() {
        let src = "def f(x: Optional[int]) -> int:\n    return 0\n";
        let cir = parse_and_lift(src, "f.py").unwrap();
        if let Cir::Module { items, .. } = cir {
            if let Cir::Func { params, .. } = &items[0] {
                assert!(matches!(&params[0].ty, CirTy::Optional(_)));
            }
        }
    }

    #[test]
    fn list_type_parses() {
        let src = "def f(xs: List[int]) -> int:\n    return 0\n";
        let cir = parse_and_lift(src, "f.py").unwrap();
        if let Cir::Module { items, .. } = cir {
            if let Cir::Func { params, .. } = &items[0] {
                assert!(matches!(&params[0].ty, CirTy::Array(_)));
            }
        }
    }

    #[test]
    fn leading_indent_counts_spaces() {
        assert_eq!(leading_indent("    x"), 4);
        assert_eq!(leading_indent("x"), 0);
    }
}
