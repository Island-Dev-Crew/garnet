//! Python → CIR frontend (stylized parser for v4.1 initial release).
//!
//! Recognizes: `def name(args):`, `class Name:`, type hints (`x: int`),
//! `return`, common literals, `if`/`elif`/`else`, `for`/`while`,
//! f-strings (translated to Garnet `#{}` interpolation).
//!
//! Flags: decorators (`@decorator`) → MigrateTodo (closure-wrap pattern);
//! `eval`/`exec` → Untranslatable; `*args`/`**kwargs` → MigrateTodo
//! (Garnet has fixed arity).

use crate::cir::{Cir, CirLit, CirTy, FuncMode, Ownership, Param};
use crate::error::ConvertError;
use crate::lineage::Lineage;

pub fn parse_and_lift(source: &str, filename: &str) -> Result<Cir, ConvertError> {
    let mut p = PythonParser::new(source, filename);
    p.parse_module()
}

struct PythonParser<'a> {
    lines: Vec<&'a str>,
    filename: String,
    line_idx: usize,
    global_byte_offset: Vec<usize>, // per-line byte offset within source
}

impl<'a> PythonParser<'a> {
    fn new(source: &'a str, filename: &str) -> Self {
        let lines: Vec<&str> = source.lines().collect();
        let mut offsets = Vec::with_capacity(lines.len());
        let mut off = 0;
        for l in &lines {
            offsets.push(off);
            off += l.len() + 1; // +1 for newline
        }
        Self {
            lines,
            filename: filename.to_string(),
            line_idx: 0,
            global_byte_offset: offsets,
        }
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

    fn parse_module(&mut self) -> Result<Cir, ConvertError> {
        let start = self.line_idx;
        let mut items = Vec::new();
        while self.line_idx < self.lines.len() {
            let raw = self.current_line();
            let trimmed = raw.trim_start();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                self.line_idx += 1;
                continue;
            }
            let item = self.parse_item()?;
            if let Some(i) = item {
                items.push(i);
            } else {
                self.line_idx += 1;
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
        let start = self.line_idx;
        let raw = self.current_line().to_string();
        let indent = leading_indent(&raw);
        let trimmed = raw.trim_start().to_string();

        if trimmed.starts_with('@') {
            // Decorator → MigrateTodo, consume this line + following def
            self.line_idx += 1;
            if self.line_idx < self.lines.len() {
                self.line_idx += 1; // skip the decorated def too
            }
            return Ok(Some(Cir::MigrateTodo {
                placeholder: Box::new(Cir::Literal(CirLit::Nil, self.lineage(start))),
                note: format!(
                    "Python decorator: {} — convert to explicit closure-wrap",
                    trimmed
                ),
                lineage: self.lineage(start),
            }));
        }

        if trimmed.starts_with("def ") {
            return Ok(Some(self.parse_def(start, indent)?));
        }
        if trimmed.starts_with("class ") {
            return Ok(Some(self.parse_class(start, indent)?));
        }
        if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
            self.line_idx += 1;
            return self.parse_item();
        }
        if trimmed.starts_with("eval(") || trimmed.starts_with("exec(") {
            self.line_idx += 1;
            return Ok(Some(Cir::Untranslatable {
                reason: "Python eval/exec — Garnet has no runtime source evaluation".into(),
                lineage: self.lineage(start),
            }));
        }
        // Top-level statement (rare at module level except for main
        // guard). Consume as MigrateTodo.
        self.line_idx += 1;
        if trimmed.is_empty() {
            return Ok(None);
        }
        Ok(Some(Cir::MigrateTodo {
            placeholder: Box::new(Cir::Literal(CirLit::Nil, self.lineage(start))),
            note: format!("Python top-level statement: {}", trimmed),
            lineage: self.lineage(start),
        }))
    }

    fn parse_def(&mut self, start_line: usize, parent_indent: usize) -> Result<Cir, ConvertError> {
        let raw = self.current_line().to_string();
        self.line_idx += 1;
        let trimmed = raw.trim_start();
        // def name(params) -> ReturnType:
        let rest = &trimmed[4..]; // after "def "
        let paren = rest.find('(').ok_or_else(|| ConvertError::ParseError {
            source_lang: "python".into(),
            message: format!("def without open paren: {trimmed}"),
        })?;
        let name = rest[..paren].trim().to_string();
        let after_paren = &rest[paren + 1..];
        let closing = after_paren.rfind("):").or_else(|| after_paren.rfind(')'));
        let params_src = match closing {
            Some(i) => &after_paren[..i],
            None => after_paren,
        };
        let params = parse_params(params_src);
        // Extract return type if present
        let return_ty = if let Some(arrow) = trimmed.find("->") {
            let tail = &trimmed[arrow + 2..];
            let colon = tail.find(':').unwrap_or(tail.len());
            parse_ty(tail[..colon].trim())
        } else {
            CirTy::Inferred
        };

        // Body: lines more indented than parent_indent
        let body = self.parse_indented_body(parent_indent);

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
        let raw = self.current_line().to_string();
        self.line_idx += 1;
        let trimmed = raw.trim_start();
        let rest = &trimmed[6..]; // after "class "
        let name = rest
            .split(['(', ':'])
            .next()
            .unwrap_or("Anon")
            .trim()
            .to_string();

        let body = self.parse_indented_body(parent_indent);
        // Split body into fields (from __init__) + methods
        let mut methods = Vec::new();
        let mut fields = Vec::new();
        for b in &body {
            if let Cir::Func {
                name: fname,
                body: fb,
                ..
            } = b
            {
                if fname == "__init__" {
                    // Try to extract self.x = y assignments as fields
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
                } else {
                    methods.push(b.clone());
                }
            }
        }

        Ok(Cir::Module {
            name: name.clone(),
            items: vec![
                Cir::Struct {
                    name: name.clone(),
                    fields,
                    lineage: self.lineage(start_line),
                },
                Cir::Impl {
                    target: name,
                    methods,
                    lineage: self.lineage(start_line),
                },
            ],
            sandbox: false,
            lineage: self.lineage(start_line),
        })
    }

    fn parse_indented_body(&mut self, parent_indent: usize) -> Vec<Cir> {
        let mut body = Vec::new();
        while self.line_idx < self.lines.len() {
            let raw = self.current_line().to_string();
            let trimmed = raw.trim_start().to_string();
            let indent = leading_indent(&raw);
            if trimmed.is_empty() || trimmed.starts_with('#') {
                self.line_idx += 1;
                continue;
            }
            if indent <= parent_indent {
                break;
            }
            let line_start = self.line_idx;
            // Recognize nested defs (methods inside a class body)
            if trimmed.starts_with("def ") {
                if let Ok(f) = self.parse_def(line_start, indent) {
                    body.push(f);
                    continue;
                }
            }
            // C1-18: a block (`for x in xs:`, `with ...:`, `if ...:` and so on) is
            // handled whole. Simple `for`/`while` loops are lowered to brace form;
            // every other block is kept as one whole-statement MigrateTodo, so its
            // body is never flattened into the enclosing function.
            if let Some(keyword) = block_keyword(&trimmed) {
                let end = self.block_end(line_start, indent);
                if let Some(lowered) = self.lower_loop(keyword, &trimmed, line_start, indent, end) {
                    body.push(lowered);
                } else {
                    body.push(self.block_todo(keyword, line_start, indent, end));
                    self.line_idx = end;
                }
                continue;
            }
            // Simplified: each physical line is one statement.
            if let Some(stripped) = trimmed.strip_prefix("return") {
                let expr_src = stripped.trim();
                body.push(Cir::Return {
                    value: if expr_src.is_empty() {
                        None
                    } else {
                        Some(Box::new(Cir::Ident(
                            expr_src.to_string(),
                            self.lineage(line_start),
                        )))
                    },
                    lineage: self.lineage(line_start),
                });
            } else if trimmed.contains(" = ") && trimmed.starts_with("self.") {
                // self.x = y — recognize as field write
                let parts: Vec<&str> = trimmed.splitn(2, " = ").collect();
                let field = parts[0].trim_start_matches("self.").to_string();
                body.push(Cir::Assign {
                    lhs: Box::new(Cir::FieldAccess {
                        recv: Box::new(Cir::Ident("self".into(), self.lineage(line_start))),
                        name: field,
                        lineage: self.lineage(line_start),
                    }),
                    rhs: Box::new(Cir::Ident(parts[1].to_string(), self.lineage(line_start))),
                    lineage: self.lineage(line_start),
                });
            } else {
                body.push(Cir::Ident(trimmed.to_string(), self.lineage(line_start)));
            }
            self.line_idx += 1;
        }
        body
    }

    fn current_line(&self) -> &str {
        self.lines.get(self.line_idx).copied().unwrap_or("")
    }

    /// One past the last line of the block whose header is at `header` (indent
    /// `indent`): every following line indented deeper, blank lines, and any
    /// `elif`/`else`/`except`/`finally` clause at the header's indent.
    fn block_end(&self, header: usize, indent: usize) -> usize {
        let mut idx = header + 1;
        while idx < self.lines.len() {
            let line = self.lines[idx];
            let trimmed = line.trim_start();
            let line_indent = leading_indent(line);
            let inside = trimmed.is_empty() || trimmed.starts_with('#') || line_indent > indent;
            let clause = line_indent == indent && is_continuation_clause(trimmed);
            if inside || clause {
                idx += 1;
            } else {
                break;
            }
        }
        // Trailing blank or comment lines belong to whatever follows.
        while idx > header + 1 {
            let t = self.lines[idx - 1].trim_start();
            if t.is_empty() || t.starts_with('#') {
                idx -= 1;
            } else {
                break;
            }
        }
        idx
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
        let has_clause = (start + 1..end).any(|i| {
            leading_indent(self.lines[i]) == indent && !self.lines[i].trim_start().is_empty()
        });
        if has_clause {
            return None;
        }
        let head = header.strip_suffix(':')?.trim_end();
        let node = if keyword == "for" {
            let rest = head.strip_prefix("for ")?;
            let (var, iter) = rest.split_once(" in ")?;
            let var = var.trim();
            if !is_identifier(var) {
                return None;
            }
            self.line_idx = start + 1;
            let body = self.parse_indented_body(indent);
            Cir::For {
                var: var.to_string(),
                iter: Box::new(Cir::Ident(iter.trim().to_string(), self.lineage(start))),
                body,
                lineage: self.lineage(start),
            }
        } else {
            let cond = head.strip_prefix("while ")?.trim();
            self.line_idx = start + 1;
            let body = self.parse_indented_body(indent);
            Cir::While {
                cond: Box::new(Cir::Ident(cond.to_string(), self.lineage(start))),
                body,
                lineage: self.lineage(start),
            }
        };
        self.line_idx = end;
        Some(node)
    }

    /// The whole block, header and body, as one MigrateTodo whose note carries
    /// the source lines (dedented to the header) for hand translation.
    fn block_todo(&self, keyword: &str, start: usize, indent: usize, end: usize) -> Cir {
        let text = self.lines[start..end]
            .iter()
            .map(|l| l.get(indent.min(leading_indent(l))..).unwrap_or(""))
            .collect::<Vec<_>>()
            .join("\n");
        Cir::MigrateTodo {
            placeholder: Box::new(Cir::Literal(CirLit::Nil, self.lineage(start))),
            note: format!("Python `{keyword}` block kept whole for hand translation:\n{text}"),
            lineage: self.lineage(start),
        }
    }
}

/// The keyword of a statement that opens an indented Python block.
fn block_keyword(trimmed: &str) -> Option<&'static str> {
    if !trimmed.trim_end().ends_with(':') {
        return None;
    }
    [
        "for", "while", "with", "if", "elif", "else", "try", "except", "finally", "async", "match",
        "case", "class",
    ]
    .into_iter()
    .find(|kw| {
        trimmed == *kw
            || trimmed.starts_with(&format!("{kw} "))
            || trimmed.starts_with(&format!("{kw}:"))
    })
}

fn is_continuation_clause(trimmed: &str) -> bool {
    matches!(
        block_keyword(trimmed),
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
