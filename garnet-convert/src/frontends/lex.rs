//! Lexical scanning the line-based frontends share (C1-18).
//!
//! Each frontend reads a statement as a defined unit of its language, so no
//! variant of a split statement, block or definition can leave a fragment of it
//! active:
//!
//! - **Python**: a logical line. Physical lines join while a bracket is open, a
//!   line ends in `\`, or a triple-quoted string is open, as in the language
//!   reference's line structure. Every string form is lexed; an f-string whose
//!   replacement field holds the f-string's own quote (Python 3.12) is refused.
//! - **Go**: lines up to the point where Go inserts a semicolon (after an
//!   identifier, a literal, `break`, `continue`, `fallthrough`, `return`, `++`,
//!   `--`, `)`, `]` or `}` at a line end), once every bracket, block comment and
//!   raw string is closed.
//! - **Ruby**: a defined subset. Lines join while a bracket or a keyword block is
//!   open or a line ends in an operator, a comma or a modifier (`if`, `unless`,
//!   `while`, `until`, `rescue`). A keyword block is opened by
//!   `if`/`unless`/`while`/`until` where an expression starts (elsewhere they are
//!   modifiers), `case`, `begin`, `def`, `class`, `module`, `for` and `do`, and
//!   closed by `end`. A line outside the subset is refused, not guessed.
//!
//! Every scanner works on bytes: the delimiters are ASCII, and a UTF-8 multibyte
//! character never contains an ASCII byte.

// ─── Go ──────────────────────────────────────────────────────────────

/// Go lexing state carried from line to line within one statement.
#[derive(Debug, Default, Clone, Copy)]
pub struct GoLex {
    in_block_comment: bool,
    in_raw_string: bool,
    /// The last token seen is one after which Go inserts a semicolon.
    last_ends: bool,
}

/// What one line contributes to a Go statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GoLine {
    /// The bracket depth after the line.
    pub depth: i64,
    /// Where a closing bracket went below depth zero. The scan stops there: the
    /// bracket closes a construct the statement is inside.
    pub cut: Option<usize>,
}

impl GoLex {
    /// Scan one line of a statement whose brackets are `depth` deep.
    pub fn scan(&mut self, line: &str, depth: i64) -> GoLine {
        let b = line.as_bytes();
        let mut depth = depth;
        let mut i = 0;
        while i < b.len() {
            if self.in_block_comment {
                if b[i] == b'*' && b.get(i + 1) == Some(&b'/') {
                    self.in_block_comment = false;
                    i += 2;
                } else {
                    i += 1;
                }
                continue;
            }
            if self.in_raw_string {
                if b[i] == b'`' {
                    self.in_raw_string = false;
                    self.last_ends = true;
                }
                i += 1;
                continue;
            }
            match b[i] {
                b' ' | b'\t' | b'\r' => {}
                b'/' if b.get(i + 1) == Some(&b'/') => break,
                b'/' if b.get(i + 1) == Some(&b'*') => {
                    self.in_block_comment = true;
                    i += 2;
                    continue;
                }
                b'`' => self.in_raw_string = true,
                q @ (b'"' | b'\'') => {
                    // Interpreted strings and runes end on their own line in Go.
                    i += 1;
                    while i < b.len() && b[i] != q {
                        if b[i] == b'\\' {
                            i += 1;
                        }
                        i += 1;
                    }
                    self.last_ends = true;
                }
                b'(' | b'[' | b'{' => {
                    depth += 1;
                    self.last_ends = false;
                }
                b')' | b']' | b'}' => {
                    if depth == 0 {
                        return GoLine {
                            depth,
                            cut: Some(i),
                        };
                    }
                    depth -= 1;
                    self.last_ends = true;
                }
                c @ (b'+' | b'-') if b.get(i + 1) == Some(&c) => {
                    self.last_ends = true;
                    i += 2;
                    continue;
                }
                c if is_word_byte(c) => {
                    let start = i;
                    let number = c.is_ascii_digit();
                    i += 1;
                    while i < b.len() && (is_word_byte(b[i]) || (number && is_number_tail(b, i))) {
                        i += 1;
                    }
                    // Of the keywords, only break, continue, fallthrough and return
                    // end a line; `go`, `defer` and the rest take what follows.
                    self.last_ends = !GO_OPERAND_KEYWORDS.contains(&&b[start..i]);
                    continue;
                }
                _ => self.last_ends = false,
            }
            i += 1;
        }
        GoLine { depth, cut: None }
    }

    /// True while a block comment or a raw string is still open.
    pub fn open(&self) -> bool {
        self.in_block_comment || self.in_raw_string
    }

    /// Whether Go inserts a semicolon at the end of the lines scanned so far.
    pub fn ends_statement(&self) -> bool {
        self.last_ends
    }
}

/// Go keywords after which no semicolon is inserted at a line end.
const GO_OPERAND_KEYWORDS: [&[u8]; 21] = [
    b"case",
    b"chan",
    b"const",
    b"default",
    b"defer",
    b"else",
    b"for",
    b"func",
    b"go",
    b"goto",
    b"if",
    b"import",
    b"interface",
    b"map",
    b"package",
    b"range",
    b"select",
    b"struct",
    b"switch",
    b"type",
    b"var",
];

fn is_word_byte(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c >= 0x80
}

/// Inside a number literal: a `.`, or the sign of an exponent.
fn is_number_tail(b: &[u8], i: usize) -> bool {
    b[i] == b'.' || (matches!(b[i], b'+' | b'-') && matches!(b[i - 1], b'e' | b'E' | b'p' | b'P'))
}

// ─── Python ──────────────────────────────────────────────────────────

/// What one Python line contributes to its logical line.
#[derive(Debug, Default, Clone, Copy)]
pub struct PyLine {
    /// Brackets opened minus brackets closed, outside strings and comments.
    pub delta: i64,
    /// The line ends in a `\` continuation outside a string.
    pub continues: bool,
    /// Where a trailing `#` comment starts, if there is one.
    pub comment_at: Option<usize>,
}

/// Python lexing state that can span lines: a triple-quoted string.
#[derive(Debug, Default, Clone, Copy)]
pub struct PyLex {
    triple: Option<u8>,
}

impl PyLex {
    /// Scan one physical line. `Err` names a construct the frontend does not lex:
    /// an f-string whose replacement field holds the f-string's own quote (Python
    /// 3.12 syntax), or a one-line string left open at the line end.
    pub fn scan(&mut self, line: &str) -> Result<PyLine, &'static str> {
        let b = line.as_bytes();
        let mut out = PyLine::default();
        let mut i = 0;
        while i < b.len() {
            if let Some(q) = self.triple {
                match find_triple(b, i, q) {
                    Some(end) => {
                        self.triple = None;
                        i = end;
                        continue;
                    }
                    None => return Ok(out),
                }
            }
            match b[i] {
                b'#' => {
                    out.comment_at = Some(i);
                    break;
                }
                q @ (b'"' | b'\'') => {
                    let is_f = string_prefix(b, i).iter().any(|c| matches!(c, b'f' | b'F'));
                    if b.get(i + 1) == Some(&q) && b.get(i + 2) == Some(&q) {
                        match find_triple(b, i + 3, q) {
                            Some(end) => i = end,
                            None => {
                                self.triple = Some(q);
                                return Ok(out);
                            }
                        }
                        continue;
                    }
                    i = scan_py_string(b, i + 1, q, is_f)?;
                    continue;
                }
                b'(' | b'[' | b'{' => out.delta += 1,
                b')' | b']' | b'}' => out.delta -= 1,
                b'\\' if i + 1 == b.len() => out.continues = true,
                _ => {}
            }
            i += 1;
        }
        Ok(out)
    }

    /// True while a triple-quoted string is still open.
    pub fn open(&self) -> bool {
        self.triple.is_some()
    }
}

/// The string prefix letters (`r`, `b`, `f`, `u` in either case) just before
/// the quote at `quote`.
fn string_prefix(b: &[u8], quote: usize) -> &[u8] {
    let mut start = quote;
    while start > 0
        && matches!(
            b[start - 1],
            b'r' | b'R' | b'b' | b'B' | b'f' | b'F' | b'u' | b'U'
        )
    {
        start -= 1;
    }
    // A prefix is the whole word before the quote, not the tail of an identifier.
    if start > 0 && (b[start - 1].is_ascii_alphanumeric() || b[start - 1] == b'_') {
        return &[];
    }
    &b[start..quote]
}

/// The index just past the closing `qqq` at or after `from`.
fn find_triple(b: &[u8], from: usize, q: u8) -> Option<usize> {
    let mut i = from;
    while i + 3 <= b.len() {
        if b[i] == b'\\' {
            i += 2;
            continue;
        }
        if b[i] == q && b.get(i + 1) == Some(&q) && b.get(i + 2) == Some(&q) {
            return Some(i + 3);
        }
        i += 1;
    }
    None
}

/// Scan a one-line string from just after its opening quote `q`; returns the
/// index just past the closing quote. In an f-string, a replacement field is
/// scanned too: a nested string in the other quote is skipped, and the
/// f-string's own quote inside a field is refused.
fn scan_py_string(b: &[u8], mut i: usize, q: u8, is_f: bool) -> Result<usize, &'static str> {
    while i < b.len() {
        match b[i] {
            b'\\' => i += 2,
            c if c == q => return Ok(i + 1),
            b'{' if is_f && b.get(i + 1) == Some(&b'{') => i += 2,
            b'{' if is_f => {
                let mut depth = 1;
                i += 1;
                while i < b.len() && depth > 0 {
                    match b[i] {
                        b'{' => depth += 1,
                        b'}' => depth -= 1,
                        c if c == q => {
                            return Err(
                                "an f-string whose replacement field contains its own quote",
                            );
                        }
                        n @ (b'"' | b'\'') => {
                            i += 1;
                            while i < b.len() && b[i] != n {
                                i += 1;
                            }
                        }
                        _ => {}
                    }
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }
    Err("a string that is still open at the end of its line")
}

// ─── Ruby ────────────────────────────────────────────────────────────

/// What one line of Ruby contributes to its statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RubyLine<'a> {
    /// The line without its trailing comment, trimmed at the end.
    pub code: &'a str,
    /// Brackets opened minus brackets closed.
    pub brackets: i64,
    /// Keyword blocks opened minus `end`s.
    pub blocks: i64,
    /// The line ends in an operator, a comma or `\`, so the statement goes on.
    pub continues: bool,
}

/// Lex one line of the Ruby subset: comments, quoted and backtick strings (with
/// interpolation that holds no quotes), symbols, regular expressions in operand
/// position, and keyword blocks. Percent literals, heredocs, character literals,
/// `$'`-style globals, an ambiguous `/`, `=begin` comments, `__END__`, endless
/// method definitions and strings that continue on the next line are refused.
pub fn ruby_line(line: &str) -> Result<RubyLine<'_>, &'static str> {
    let b = line.as_bytes();
    let first = line.trim_start();
    if first.starts_with("=begin") {
        return Err("an =begin block comment");
    }
    if first.starts_with("__END__") {
        return Err("an __END__ data section");
    }
    let mut out = RubyLine {
        code: line,
        brackets: 0,
        blocks: 0,
        continues: false,
    };
    // The token before ends an expression: `if` after it is a modifier.
    let mut after_value = false;
    // A `while`/`until`/`for` opened on this line: a later `do` is its separator.
    let mut loop_do = false;
    // The next word is a method name (after `def`, `.` or `::`), not a keyword.
    let mut name_next = false;
    // The last token is an operator, a comma or `\`.
    let mut last_op = false;
    let mut code_end = b.len();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        let mut op = true;
        match c {
            b' ' | b'\t' | b'\r' => {
                i += 1;
                continue;
            }
            b'#' => {
                code_end = i;
                break;
            }
            b'"' | b'`' => {
                i = scan_rb_double(b, i + 1, c)?;
                after_value = true;
                last_op = false;
                continue;
            }
            b'\'' => {
                i = scan_rb_single(b, i + 1)?;
                after_value = true;
                last_op = false;
                continue;
            }
            b'%' => {
                let next = b.get(i + 1).copied();
                let delim = match next {
                    Some(b'q' | b'Q' | b'w' | b'W' | b'i' | b'I' | b'r' | b's' | b'x') => {
                        b.get(i + 2).copied()
                    }
                    other => other,
                };
                if let Some(d) = delim {
                    if !d.is_ascii_alphanumeric() && !d.is_ascii_whitespace() && d != b'=' {
                        return Err("a percent literal");
                    }
                }
                after_value = false;
            }
            b'<' if b.get(i + 1) == Some(&b'<')
                && matches!(
                    b.get(i + 2),
                    Some(b'~' | b'-' | b'"' | b'\'' | b'A'..=b'Z' | b'_')
                ) =>
            {
                return Err("a heredoc");
            }
            b'?' if operand_position(b, i)
                && b.get(i + 1).is_some_and(|c| !c.is_ascii_whitespace())
                && !b
                    .get(i + 2)
                    .is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_') =>
            {
                return Err("a character literal");
            }
            b'$' => {
                match b.get(i + 1) {
                    Some(b'\'' | b'"' | b'`') => return Err("a $' or $\" global"),
                    // `$!`, `$?`, `$~` and the other punctuation globals.
                    Some(c) if !is_word_byte(*c) && !c.is_ascii_whitespace() => i += 2,
                    _ => i = word_end(b, i + 1),
                }
                after_value = true;
                last_op = false;
                continue;
            }
            b'@' => {
                // An instance or class variable: `@end` is a name, not `end`.
                let mut j = i + 1;
                while b.get(j) == Some(&b'@') {
                    j += 1;
                }
                i = word_end(b, j);
                after_value = true;
                last_op = false;
                continue;
            }
            b'/' => {
                let spaced_after_word = i > 0
                    && b[i - 1] == b' '
                    && previous_word_is_identifier(b, i)
                    && !b.get(i + 1).is_some_and(|c| c.is_ascii_whitespace());
                if spaced_after_word {
                    return Err("an ambiguous / that may start a regular expression");
                }
                if operand_position(b, i) {
                    i += 1;
                    loop {
                        match b.get(i) {
                            None => {
                                return Err("a regular expression that continues on the next line")
                            }
                            Some(b'\\') => i += 2,
                            Some(b'/') => break,
                            Some(_) => i += 1,
                        }
                    }
                    i += 1;
                    // Regex flags.
                    while b.get(i).is_some_and(|c| c.is_ascii_alphabetic()) {
                        i += 1;
                    }
                    after_value = true;
                    last_op = false;
                    continue;
                }
                after_value = false;
            }
            b'(' | b'[' | b'{' => {
                out.brackets += 1;
                after_value = false;
            }
            b')' | b']' | b'}' => {
                out.brackets -= 1;
                after_value = true;
                op = false;
            }
            b':' if b.get(i + 1) == Some(&b':') => {
                name_next = true;
                after_value = false;
                last_op = true;
                i += 2;
                continue;
            }
            b':' if b.get(i + 1).is_some_and(|c| is_word_byte(*c)) => {
                // A symbol: `:end` is a value, not `end`.
                i = method_name_end(b, word_end(b, i + 1));
                after_value = true;
                last_op = false;
                continue;
            }
            b'.' if b.get(i + 1) == Some(&b'.') => {
                // A range operator, `..` or `...`.
                while b.get(i) == Some(&b'.') {
                    i += 1;
                }
                after_value = false;
                last_op = true;
                continue;
            }
            b'.' => {
                name_next = true;
                after_value = false;
            }
            b'&' if b.get(i + 1) == Some(&b'.') => {
                name_next = true;
                after_value = false;
                last_op = true;
                i += 2;
                continue;
            }
            c if is_word_byte(c) => {
                let start = i;
                i = method_name_end(b, word_end(b, i));
                if b.get(i) == Some(&b':') && b.get(i + 1) != Some(&b':') {
                    // A hash label such as `if:` is not a keyword.
                    i += 1;
                    after_value = false;
                    last_op = true;
                    name_next = false;
                    continue;
                }
                last_op = false;
                if name_next || c.is_ascii_digit() {
                    name_next = false;
                    after_value = true;
                    continue;
                }
                match &line[start..i] {
                    "if" | "unless" | "while" | "until" => {
                        if !after_value {
                            out.blocks += 1;
                            loop_do |= matches!(&line[start..i], "while" | "until");
                        }
                        // As a modifier at a line end it takes the next line.
                        after_value = false;
                        last_op = true;
                    }
                    "for" => {
                        out.blocks += 1;
                        loop_do = true;
                        after_value = false;
                    }
                    "case" | "begin" | "class" | "module" => {
                        out.blocks += 1;
                        after_value = false;
                    }
                    "def" => {
                        if endless_def(b, i) {
                            return Err("an endless method definition");
                        }
                        out.blocks += 1;
                        name_next = true;
                        after_value = false;
                    }
                    "do" => {
                        if loop_do {
                            loop_do = false;
                        } else {
                            out.blocks += 1;
                        }
                        after_value = false;
                    }
                    "end" => {
                        out.blocks -= 1;
                        after_value = true;
                    }
                    "and" | "or" | "not" | "rescue" => {
                        after_value = false;
                        last_op = true;
                    }
                    "then" | "else" | "elsif" | "when" | "in" | "ensure" => {
                        after_value = false;
                    }
                    _ => after_value = true,
                }
                continue;
            }
            _ => after_value = false,
        }
        last_op = op;
        i += 1;
    }
    out.code = line[..code_end].trim_end();
    out.continues = last_op;
    Ok(out)
}

/// The end of the identifier starting at `i`.
fn word_end(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && is_word_byte(b[i]) {
        i += 1;
    }
    i
}

/// A method name may end in `?` or `!` (but `!=` is an operator).
fn method_name_end(b: &[u8], i: usize) -> usize {
    match b.get(i) {
        Some(b'?' | b'!') if b.get(i + 1) != Some(&b'=') => i + 1,
        _ => i,
    }
}

/// Whether the `def` that ends just before `i` is an endless method
/// (`def name(args) = expr`), which has no `end`.
fn endless_def(b: &[u8], mut i: usize) -> bool {
    while b.get(i) == Some(&b' ') {
        i += 1;
    }
    let name = i;
    while i < b.len() && !matches!(b[i], b' ' | b'\t' | b'(') {
        i += 1;
    }
    if i == name {
        return false;
    }
    if b.get(i) == Some(&b'(') {
        let mut depth = 0;
        while i < b.len() {
            match b[i] {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        break;
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }
    while b.get(i) == Some(&b' ') {
        i += 1;
    }
    b.get(i) == Some(&b'=') && !matches!(b.get(i + 1), Some(b'=' | b'~' | b'>'))
}

/// Scan a single-quoted Ruby string from just after its quote.
fn scan_rb_single(b: &[u8], mut i: usize) -> Result<usize, &'static str> {
    loop {
        match b.get(i) {
            None => return Err("a string that continues on the next line"),
            Some(b'\\') => i += 2,
            Some(b'\'') => return Ok(i + 1),
            Some(_) => i += 1,
        }
    }
}

/// Scan a double-quoted or backtick Ruby string from just after its quote `q`.
fn scan_rb_double(b: &[u8], mut i: usize, q: u8) -> Result<usize, &'static str> {
    while i < b.len() {
        match b[i] {
            b'\\' => i += 2,
            b'#' if b.get(i + 1) == Some(&b'{') => {
                let mut depth = 1;
                i += 2;
                while i < b.len() && depth > 0 {
                    match b[i] {
                        b'{' => depth += 1,
                        b'}' => depth -= 1,
                        b'"' | b'\'' | b'`' => {
                            return Err("interpolation that contains a quoted string")
                        }
                        _ => {}
                    }
                    i += 1;
                }
            }
            c if c == q => return Ok(i + 1),
            _ => i += 1,
        }
    }
    Err("a string that continues on the next line")
}

/// Whether the byte at `i` starts an operand: nothing but an opening bracket,
/// separator or operator (or a keyword that takes an expression) precedes it.
fn operand_position(b: &[u8], i: usize) -> bool {
    let mut j = i;
    while j > 0 && b[j - 1] == b' ' {
        j -= 1;
    }
    if j == 0 {
        return true;
    }
    let prev = b[j - 1];
    if prev.is_ascii_alphanumeric() || prev == b'_' {
        let word_end = j;
        let mut start = j;
        while start > 0 && (b[start - 1].is_ascii_alphanumeric() || b[start - 1] == b'_') {
            start -= 1;
        }
        let word = &b[start..word_end];
        return matches!(
            word,
            b"if"
                | b"unless"
                | b"when"
                | b"while"
                | b"until"
                | b"and"
                | b"or"
                | b"not"
                | b"return"
                | b"in"
        );
    }
    !matches!(prev, b')' | b']' | b'}' | b'"' | b'\'' | b'`')
}

/// Whether the word just before the space at `i - 1` is an identifier (not a
/// keyword that takes an expression), so `word /x` is ambiguous in Ruby.
fn previous_word_is_identifier(b: &[u8], i: usize) -> bool {
    let end = i - 1;
    let mut start = end;
    while start > 0 && (b[start - 1].is_ascii_alphanumeric() || b[start - 1] == b'_') {
        start -= 1;
    }
    start < end && !operand_position(b, i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn go_comments_strings_and_semicolon_insertion() {
        let mut lex = GoLex::default();
        assert_eq!(1, lex.scan("if enabled { /* } */", 0).depth);
        assert!(!lex.open());
        let mut lex = GoLex::default();
        assert_eq!(0, lex.scan("x := `{", 0).depth);
        assert!(lex.open());
        assert_eq!(0, lex.scan("}`", 0).depth);
        assert!(!lex.open() && lex.ends_statement());
        assert_eq!(1, GoLex::default().scan("f( // )", 0).depth);
        assert_eq!(0, GoLex::default().scan(r#"s := "}" + '{'"#, 0).depth);
        let mut lex = GoLex::default();
        assert_eq!(Some(9), lex.scan("return 1 }", 0).cut);
        for (line, ends) in [
            ("total := 1 +", false),
            ("x++", true),
            ("f(a,", false),
            ("return", true),
            ("x := 1.5", true),
            ("y := 1e-3 // note", true),
            ("outer:", false),
            ("go", false),
            ("defer", false),
            ("break", true),
            ("x := m[key]", true),
        ] {
            let mut lex = GoLex::default();
            lex.scan(line, 0);
            assert_eq!(ends, lex.ends_statement(), "{line}");
        }
    }

    #[test]
    fn python_strings_comments_and_continuations() {
        let mut lex = PyLex::default();
        let src = "if (x == '#'):  # note";
        let line = lex.scan(src).unwrap();
        assert_eq!(0, line.delta);
        assert_eq!(Some(src.rfind('#').unwrap()), line.comment_at);
        assert_eq!(1, lex.scan("call(").unwrap().delta);
        assert!(lex.scan("x = 1 + \\").unwrap().continues);
        lex.scan("s = \"\"\"start ( #").unwrap();
        assert!(lex.open());
        lex.scan("still ) inside").unwrap();
        lex.scan("end\"\"\"").unwrap();
        assert!(!lex.open());
        assert!(PyLex::default().scan("f\"{d['k']}\"").is_ok());
        assert!(PyLex::default().scan("f\"{d[\"k\"]}\"").is_err());
    }

    fn rb(line: &str) -> RubyLine<'_> {
        ruby_line(line).unwrap()
    }

    #[test]
    fn ruby_subset_comments_strings_and_regex() {
        assert_eq!(
            "items.each do |item|",
            rb("items.each do |item| # note").code
        );
        assert_eq!("x = \"#{a}\"", rb("x = \"#{a}\" # c").code);
        assert_eq!("if line =~ /#/", rb("if line =~ /#/").code);
        assert_eq!("a = b / c", rb("a = b / c # half").code);
        assert_eq!("x.empty? ? 1 : 2", rb("x.empty? ? 1 : 2").code);
        for refused in [
            "%w[#].each do |item|",
            "text = <<~EOS",
            "c = ?#",
            "puts \"#{h[\"k\"]}\"",
            "=begin",
            "puts /#/",
            "s = 'open",
            "def one = 1",
            "def self.two() = 2",
        ] {
            assert!(ruby_line(refused).is_err(), "{refused}");
        }
        for allowed in ["def ==(other)", "def name=(value)", "def f(a, b = 1)"] {
            assert!(ruby_line(allowed).is_ok(), "{allowed}");
        }
    }

    #[test]
    fn ruby_keyword_blocks_brackets_and_continuation() {
        for (line, blocks) in [
            ("if ready", 1),
            ("x = if ready", 1),
            ("return x if ready", 0),
            ("return if ready", 0),
            ("items.each do |item|", 1),
            ("while running do", 1),
            ("if x then y end", 0),
            ("else if other", 1),
            ("end.to_h", -1),
            ("end while busy", -1),
            ("x.class", 0),
            ("h = { if: 1, end: 2 }", 0),
            ("sym = :end", 0),
            ("@end = 1", 0),
            ("class << self", 1),
            ("def end", 1),
            ("def x; 1; end", 0),
        ] {
            assert_eq!(blocks, rb(line).blocks, "{line}");
        }
        assert_eq!(1, rb("x = compute(").brackets);
        assert_eq!(1, rb("items.map { |i|").brackets);
        for (line, continues) in [
            ("total = 1 +", true),
            ("args = [a,", true),
            ("ready = a &&", true),
            ("ready = a and", true),
            ("x.empty?", false),
            ("x = y # +", false),
            ("save!", false),
            ("x = cond ?", true),
            ("raise $!", false),
            ("return 7 if", true),
            ("x = fetch rescue", true),
            ("x = 1 if ready", false),
            ("r = (1..n)", false),
        ] {
            assert_eq!(continues, rb(line).continues, "{line}");
        }
    }
}
