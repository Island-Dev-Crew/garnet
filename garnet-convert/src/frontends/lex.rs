//! Lexical scanning the line-based frontends share (C1-18).
//!
//! Each frontend reads a statement as a defined unit of its language, so no
//! variant of a split statement, block or definition can leave a fragment of it
//! active:
//!
//! - **Python**: a logical line. Physical lines join while a bracket is open, a
//!   line ends in `\`, or a triple-quoted string is open, as in the language
//!   reference's line structure. Every string form is lexed, f-strings and
//!   t-strings with their replacement fields; a field holding the string's own
//!   quote, a comment or a backslash outside a nested string (Python 3.12) is
//!   refused.
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
    /// A `;` appeared outside strings and comments.
    semicolon: bool,
}

/// What one line contributes to a Go statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GoLine {
    /// The bracket depth after the line.
    pub depth: i64,
    /// Where a closing bracket went below depth zero. The scan stops there: the
    /// bracket closes a construct the statement is inside.
    pub cut: Option<usize>,
    /// Where a block comment opens that does not close on this line, when the
    /// statement could end there (outside brackets, after a token that ends
    /// one): a comment holding a newline acts as one, so the statement ends.
    pub comment_break: Option<usize>,
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
                    let closes_here = b[i + 2..].windows(2).any(|w| w == b"*/");
                    if !closes_here && depth == 0 && self.last_ends {
                        return GoLine {
                            depth,
                            cut: None,
                            comment_break: Some(i),
                        };
                    }
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
                b';' => {
                    self.semicolon = true;
                    self.last_ends = false;
                }
                b')' | b']' | b'}' => {
                    if depth == 0 {
                        return GoLine {
                            depth,
                            cut: Some(i),
                            comment_break: None,
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
        GoLine {
            depth,
            cut: None,
            comment_break: None,
        }
    }

    /// True while a block comment or a raw string is still open.
    pub fn open(&self) -> bool {
        self.in_block_comment || self.in_raw_string
    }

    /// Whether Go inserts a semicolon at the end of the lines scanned so far.
    pub fn ends_statement(&self) -> bool {
        self.last_ends
    }

    /// Whether a `;` appeared, outside strings and comments, in the lines
    /// scanned so far.
    pub fn semicolon(&self) -> bool {
        self.semicolon
    }
}

/// Whether `word` is a Go keyword after which no semicolon is inserted at a
/// line end (it takes what follows).
pub fn go_keyword_takes_operand(word: &[u8]) -> bool {
    GO_OPERAND_KEYWORDS.contains(&word)
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

/// A byte of a Ruby identifier in the subset: ASCII only (see `NON_ASCII`).
fn is_rb_word_byte(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
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
    /// A `;` statement separator appears outside strings and comments.
    pub semicolon: bool,
}

/// A replacement field of an f-string or t-string being scanned.
#[derive(Debug, Clone, Copy)]
struct Field {
    /// Brackets open in the field's expression.
    brackets: u32,
    /// Past the format spec's `:`.
    spec: bool,
}

/// A string being scanned, which a triple-quoted one keeps open across lines.
#[derive(Debug, Clone)]
struct OpenString {
    quote: u8,
    triple: bool,
    /// A raw string: `\N{...}` is not a named character.
    raw: bool,
    /// An f-string or t-string: `{` opens a replacement field.
    fields_allowed: bool,
    fields: Vec<Field>,
}

const FIELD_QUOTE: &str = "an f-string or t-string whose replacement field contains its own quote";
const FIELD_COMMENT: &str = "an f-string or t-string whose replacement field contains a comment";
const FIELD_BACKSLASH: &str =
    "an f-string or t-string whose replacement field contains a backslash";
const FIELD_TRIPLE: &str =
    "an f-string or t-string whose replacement field holds a triple-quoted string";
const FIELD_FSTRING: &str =
    "an f-string or t-string whose replacement field holds another f-string or t-string";
const OPEN_STRING: &str = "a string that is still open at the end of its line";

/// Python lexing state that can span lines: a triple-quoted string.
#[derive(Debug, Default, Clone)]
pub struct PyLex {
    open: Option<OpenString>,
}

impl PyLex {
    /// Scan one physical line. `Err` names a construct the frontend does not lex:
    /// an f-string or t-string whose replacement field holds its own quote, a
    /// comment or a backslash outside a nested string (Python 3.12 syntax), or a
    /// one-line string left open at the line end.
    pub fn scan(&mut self, line: &str) -> Result<PyLine, &'static str> {
        let b = line.as_bytes();
        let mut out = PyLine::default();
        let mut i = 0;
        if let Some(open) = self.open.as_mut() {
            match scan_string(b, 0, open)? {
                Some(end) => {
                    self.open = None;
                    i = end;
                }
                None => return Ok(out),
            }
        }
        while i < b.len() {
            match b[i] {
                b'#' => {
                    out.comment_at = Some(i);
                    break;
                }
                quote @ (b'"' | b'\'') => {
                    let prefix = string_prefix(b, i);
                    let triple = b.get(i + 1) == Some(&quote) && b.get(i + 2) == Some(&quote);
                    let mut open = OpenString {
                        quote,
                        triple,
                        raw: prefix.iter().any(|c| matches!(c, b'r' | b'R')),
                        fields_allowed: prefix
                            .iter()
                            .any(|c| matches!(c, b'f' | b'F' | b't' | b'T')),
                        fields: Vec::new(),
                    };
                    let from = if triple { i + 3 } else { i + 1 };
                    match scan_string(b, from, &mut open)? {
                        Some(end) => {
                            i = end;
                            continue;
                        }
                        None => {
                            self.open = Some(open);
                            return Ok(out);
                        }
                    }
                }
                b'(' | b'[' | b'{' => out.delta += 1,
                b')' | b']' | b'}' => out.delta -= 1,
                b';' => out.semicolon = true,
                b'\\' if i + 1 == b.len() => out.continues = true,
                _ => {}
            }
            i += 1;
        }
        Ok(out)
    }

    /// True while a triple-quoted string is still open.
    pub fn open(&self) -> bool {
        self.open.is_some()
    }
}

/// The string prefix letters (`r`, `b`, `f`, `t`, `u` in either case) just
/// before the quote at `quote`.
fn string_prefix(b: &[u8], quote: usize) -> &[u8] {
    let mut start = quote;
    while start > 0
        && matches!(
            b[start - 1],
            b'r' | b'R' | b'b' | b'B' | b'f' | b'F' | b't' | b'T' | b'u' | b'U'
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

/// Scan the string `open` from `i`. Returns the index just past its closing
/// quote, or `None` when the line ends inside a triple-quoted string. In an
/// f-string or t-string each replacement field is scanned: a nested string in
/// the other quote is skipped, and the string's own quote, a comment or a
/// backslash in the field's expression is refused.
fn scan_string(
    b: &[u8],
    mut i: usize,
    open: &mut OpenString,
) -> Result<Option<usize>, &'static str> {
    while i < b.len() {
        let c = b[i];
        if let Some(field) = open.fields.last_mut() {
            if field.spec {
                match c {
                    b'{' => open.fields.push(Field {
                        brackets: 0,
                        spec: false,
                    }),
                    b'}' => {
                        open.fields.pop();
                    }
                    c if c == open.quote => return Err(FIELD_QUOTE),
                    _ => {}
                }
                i += 1;
                continue;
            }
            match c {
                c if c == open.quote => return Err(FIELD_QUOTE),
                b'#' => return Err(FIELD_COMMENT),
                b'\\' => return Err(FIELD_BACKSLASH),
                nested @ (b'"' | b'\'') => {
                    if b.get(i + 1) == Some(&nested) && b.get(i + 2) == Some(&nested) {
                        return Err(FIELD_TRIPLE);
                    }
                    if string_prefix(b, i)
                        .iter()
                        .any(|c| matches!(c, b'f' | b'F' | b't' | b'T'))
                    {
                        return Err(FIELD_FSTRING);
                    }
                    i += 1;
                    while i < b.len() && b[i] != nested {
                        if b[i] == b'\\' {
                            i += 1;
                        }
                        i += 1;
                    }
                    if i >= b.len() {
                        return Err(OPEN_STRING);
                    }
                }
                b'(' | b'[' | b'{' => field.brackets += 1,
                b')' | b']' => field.brackets = field.brackets.saturating_sub(1),
                b'}' if field.brackets > 0 => field.brackets -= 1,
                b'}' => {
                    open.fields.pop();
                }
                b':' if field.brackets == 0 => field.spec = true,
                _ => {}
            }
            i += 1;
            continue;
        }
        match c {
            b'\\' => {
                // An escape never hides a replacement field: `\{` leaves the
                // `{` to open one, and only in a non-raw string is `\N{...}` a
                // named character.
                if open.fields_allowed && b.get(i + 1) == Some(&b'{') {
                    i += 1;
                    continue;
                }
                if open.fields_allowed
                    && !open.raw
                    && b.get(i + 1) == Some(&b'N')
                    && b.get(i + 2) == Some(&b'{')
                {
                    i += 3
                        + b[i + 3..]
                            .iter()
                            .position(|&c| c == b'}')
                            .ok_or(OPEN_STRING)?
                        + 1;
                    continue;
                }
                i += 2;
                continue;
            }
            b'{' if open.fields_allowed && b.get(i + 1) == Some(&b'{') => {
                i += 2;
                continue;
            }
            b'{' if open.fields_allowed => open.fields.push(Field {
                brackets: 0,
                spec: false,
            }),
            c if c == open.quote => {
                if !open.triple {
                    return Ok(Some(i + 1));
                }
                if b.get(i + 1) == Some(&c) && b.get(i + 2) == Some(&c) {
                    return Ok(Some(i + 3));
                }
            }
            _ => {}
        }
        i += 1;
    }
    if open.triple {
        Ok(None)
    } else {
        Err(OPEN_STRING)
    }
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
    /// Keyword blocks opened on the line, whether or not they close on it.
    pub openers: u32,
    /// The line ends in an operator, a comma or `\`, so the statement goes on.
    pub continues: bool,
    /// A `;` statement separator appears outside strings and comments.
    pub semicolon: bool,
    /// The line ends where a method name is due (after `.`, `&.`, `::` or
    /// `def`), so the next line's first word is a name, not a keyword.
    pub name_pending: bool,
    /// The lowest the running keyword-block count reaches on the line: below 0,
    /// an `end` closed a block opened before the line.
    pub min_blocks: i64,
    /// The lowest running keyword-block count at which a clause keyword
    /// (`then`, `else`, `elsif`, `when`, `ensure`, or `rescue`/`in` where an
    /// expression starts) appears: at 0 it continues a block opened before the
    /// line.
    pub clause_at: Option<i64>,
}

/// Lex one line of the Ruby subset that starts a statement or follows a
/// complete one. See `ruby_line_after`.
pub fn ruby_line(line: &str) -> Result<RubyLine<'_>, &'static str> {
    ruby_line_after(line, false)
}

/// Lex one line of the Ruby subset: comments, quoted and backtick strings (with
/// interpolation that holds no quotes), symbols, regular expressions in operand
/// position, and keyword blocks. Code is ASCII: a non-ASCII character outside a
/// string, regular expression or comment is refused, so every check below reads
/// identifiers the same way. Percent literals, heredocs, character literals,
/// `$'`-style globals, an ambiguous `/`, `=begin` comments, `__END__`, endless
/// method definitions and strings that continue on the next line are refused.
///
/// `name_first`: the line before ended where a method name is due, so this
/// line's first word is a name (`obj.` then `end`).
pub fn ruby_line_after(line: &str, name_first: bool) -> Result<RubyLine<'_>, &'static str> {
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
        openers: 0,
        continues: false,
        semicolon: false,
        name_pending: false,
        min_blocks: 0,
        clause_at: None,
    };
    // The token before ends an expression: `if` after it is a modifier.
    let mut after_value = false;
    // A `while`/`until`/`for` opened on this line: a later `do` is its separator.
    let mut loop_do = false;
    // The next word is a method name (after `def`, `.` or `::`), not a keyword.
    let mut name_next = name_first;
    // The rest of the statement holds method names (the operands of `alias`
    // and `undef`), not keywords; they must be complete on this line.
    let mut names_rest = false;
    let mut operands_needed = 0;
    let mut operands = 0;
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
                // Where an operand starts, `%` begins a percent literal whatever
                // follows it; after a spaced identifier with no space after it,
                // Ruby reads one too. Elsewhere it is the modulo operator.
                if operand_position(b, i) {
                    return Err("a percent literal");
                }
                let spaced_after_word = spaced_after_identifier(b, i)
                    && !b
                        .get(i + 1)
                        .is_some_and(|c| c.is_ascii_whitespace() || *c == b'=');
                if spaced_after_word {
                    return Err("an ambiguous % that may start a percent literal");
                }
                after_value = false;
            }
            b'<' if b.get(i + 1) == Some(&b'<')
                && b.get(i + 2).is_some_and(|c| {
                    matches!(c, b'~' | b'-' | b'"' | b'\'' | b'`' | b'_') || c.is_ascii_alphabetic()
                }) =>
            {
                return Err("a heredoc, or a << written without a space after it");
            }
            b';' => {
                if names_rest && (operands < operands_needed || last_op) {
                    return Err(ALIAS_OPERANDS);
                }
                out.semicolon = true;
                after_value = false;
                names_rest = false;
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
                if names_rest {
                    operands += 1;
                }
                match b.get(i + 1) {
                    Some(b'\'' | b'"' | b'`') => return Err("a $' or $\" global"),
                    // `$!`, `$?`, `$~` and the other punctuation globals.
                    Some(c) if !is_rb_word_byte(*c) && !c.is_ascii_whitespace() => i += 2,
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
                let spaced_after_word = spaced_after_identifier(b, i)
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
                            Some(b'#') if b.get(i + 1) == Some(&b'{') => {
                                return Err("a regular expression with interpolation")
                            }
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
            b':' if b.get(i + 1).is_some_and(|c| is_rb_word_byte(*c)) => {
                // A symbol: `:end` is a value, not `end`.
                if names_rest {
                    operands += 1;
                }
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
            c if c >= 0x80 => return Err(NON_ASCII),
            c if is_rb_word_byte(c) => {
                let start = i;
                // A number is not a method name: in `1?` the `?` is an operator.
                i = word_end(b, i);
                if !c.is_ascii_digit() {
                    i = method_name_end(b, i);
                }
                if b.get(i) == Some(&b':') && b.get(i + 1) != Some(&b':') {
                    // A hash label such as `if:` is not a keyword.
                    i += 1;
                    after_value = false;
                    last_op = true;
                    name_next = false;
                    continue;
                }
                last_op = false;
                if names_rest {
                    operands += 1;
                }
                if name_next || names_rest || c.is_ascii_digit() {
                    name_next = false;
                    after_value = true;
                    continue;
                }
                match &line[start..i] {
                    "if" | "unless" | "while" | "until" => {
                        if !after_value {
                            out.blocks += 1;
                            out.openers += 1;
                            loop_do |= matches!(&line[start..i], "while" | "until");
                        }
                        // As a modifier at a line end it takes the next line.
                        after_value = false;
                        last_op = true;
                    }
                    "for" => {
                        out.blocks += 1;
                        out.openers += 1;
                        loop_do = true;
                        after_value = false;
                    }
                    "case" | "begin" | "class" | "module" => {
                        out.blocks += 1;
                        out.openers += 1;
                        after_value = false;
                    }
                    "def" => {
                        if endless_def(b, i) {
                            return Err("an endless method definition");
                        }
                        out.blocks += 1;
                        out.openers += 1;
                        name_next = true;
                        after_value = false;
                    }
                    "do" => {
                        if loop_do {
                            loop_do = false;
                        } else {
                            out.blocks += 1;
                            out.openers += 1;
                        }
                        after_value = false;
                    }
                    "end" => {
                        out.blocks -= 1;
                        out.min_blocks = out.min_blocks.min(out.blocks);
                        after_value = true;
                    }
                    "alias" | "undef" => {
                        names_rest = true;
                        operands_needed = if &line[start..i] == "alias" { 2 } else { 1 };
                        operands = 0;
                        after_value = false;
                    }
                    "and" | "or" | "not" | "rescue" | "in" => {
                        // Where an expression starts, `rescue` and `in` are
                        // clauses; after a value, a modifier and an operator.
                        if !after_value && matches!(&line[start..i], "rescue" | "in") {
                            out.clause_at =
                                Some(out.clause_at.map_or(out.blocks, |d| d.min(out.blocks)));
                        }
                        after_value = false;
                        last_op = !matches!(&line[start..i], "in");
                    }
                    "then" | "else" | "elsif" | "when" | "ensure" => {
                        out.clause_at =
                            Some(out.clause_at.map_or(out.blocks, |d| d.min(out.blocks)));
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
    if names_rest && (operands < operands_needed || last_op) {
        return Err(ALIAS_OPERANDS);
    }
    out.name_pending = name_next;
    Ok(out)
}

/// The end of the identifier starting at `i`.
fn word_end(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && is_rb_word_byte(b[i]) {
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
    while matches!(b.get(i), Some(b' ' | b'\t')) {
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
    while matches!(b.get(i), Some(b' ' | b'\t')) {
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

const ALIAS_OPERANDS: &str = "an alias or undef whose operands are not complete on its line";

/// Ruby code (outside strings, regular expressions and comments) is read as
/// ASCII.
const NON_ASCII: &str = "a non-ASCII character outside a string or comment";

const INTERPOLATION: &str =
    "interpolation beyond plain expressions (a string, regex, percent or character literal, `/`, `?`, a comment or a heredoc)";

/// Scan a double-quoted or backtick Ruby string from just after its quote `q`.
fn scan_rb_double(b: &[u8], mut i: usize, q: u8) -> Result<usize, &'static str> {
    while i < b.len() {
        match b[i] {
            b'\\' => i += 2,
            b'#' if b.get(i + 1) == Some(&b'{') => {
                // Interpolation holds plain expressions only (names, numbers,
                // calls, indexing, arithmetic): a string, regex, percent or
                // character literal, comment or heredoc inside it could hide
                // where it ends.
                let mut depth = 1;
                i += 2;
                while i < b.len() && depth > 0 {
                    match b[i] {
                        b'{' => depth += 1,
                        b'}' => depth -= 1,
                        b'<' if b.get(i + 1) == Some(&b'<') => return Err(INTERPOLATION),
                        // A predicate method name such as `empty?`.
                        b'?' if i > 0 && (b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_') => {
                        }
                        c if c >= 0x80 => return Err(NON_ASCII),
                        c if c.is_ascii_alphanumeric()
                            || matches!(
                                c,
                                b' ' | b'\t'
                                    | b'_'
                                    | b'.'
                                    | b'@'
                                    | b'$'
                                    | b':'
                                    | b'('
                                    | b')'
                                    | b'['
                                    | b']'
                                    | b','
                                    | b'+'
                                    | b'-'
                                    | b'*'
                                    | b'='
                                    | b'!'
                                    | b'<'
                                    | b'>'
                                    | b'&'
                                    | b'|'
                            ) => {}
                        _ => return Err(INTERPOLATION),
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
    while j > 0 && matches!(b[j - 1], b' ' | b'\t') {
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

/// Whether spaces or tabs separate the byte at `i` from an identifier before
/// them (not a keyword that takes an expression), so `word /x` or `word %x` is
/// ambiguous in Ruby.
fn spaced_after_identifier(b: &[u8], i: usize) -> bool {
    let mut end = i;
    while end > 0 && matches!(b[end - 1], b' ' | b'\t') {
        end -= 1;
    }
    if end == i {
        return false;
    }
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
            "def one\t= 1",
            "def one()\t=\t1",
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
