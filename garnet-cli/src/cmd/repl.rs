//! `garnet repl [file]` — the RB-7 "joy" REPL.
//!
//! Reedline line editing (history, multiline, tab-completion) plus REPL
//! ergonomics: `?doc <name>` (a primitive's doc / caps / arity), `:caps` (the
//! session's declared + available authority surface), and pretty-printed
//! values.
//!
//! ## Where the joy lives, and why
//!
//! Reedline and every rich feature live HERE, in the `garnet-cli` crate — NOT
//! in `garnet-interp`. The interpreter crate stays terminal-dependency-free so
//! it keeps compiling to `wasm32-wasip1` (proven in the RB-6 spike); a line
//! editor would pull `crossterm` and break that portability. The CLI is not a
//! wasm target, so it is the right home.
//!
//! ## Testable core + thin glue
//!
//! Command dispatch is a pure function (`dispatch`) with no reedline or I/O in
//! it, so the behaviour is unit-tested and also drives a **plain non-TTY
//! fallback** (`run_plain`) used for pipes and CI. Reedline is a thin input
//! layer (`run_interactive`) over the same dispatch.

use crate::read_file;
use garnet_interp::{Interpreter, Value};
use garnet_parser::ast::{Annotation, Capability};
use garnet_stdlib::registry::all_prims;
use std::collections::BTreeMap;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::process::ExitCode;

// ════════════════════════════════════════════════════════════════════
// Entry point
// ════════════════════════════════════════════════════════════════════

pub fn run(preload: Option<PathBuf>) -> ExitCode {
    let mut interp = Interpreter::new();
    if let Some(p) = preload {
        match read_file(&p) {
            // Firewalled: the preload file's top-level `let`/`const` initializers
            // are evaluated during `load_source`, so a load-time panic (e.g.
            // `const X = i64::MIN.abs()`) degrades to a controlled exit-1 rather
            // than aborting before the session ever starts.
            Ok(src) => match crate::panic_firewall::firewalled(|| interp.load_source(&src)) {
                Ok(Ok(())) => println!("preloaded {}", p.display()),
                Ok(Err(e)) => {
                    eprintln!("preload error: {e}");
                    return ExitCode::from(1);
                }
                Err(panic_msg) => {
                    eprintln!("preload error: runtime panic: {panic_msg}");
                    return ExitCode::from(1);
                }
            },
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::from(1);
            }
        }
    }

    // Reedline needs a real terminal. When stdin is piped (CI, demos, `echo |
    // garnet repl`), fall back to a plain line reader that exercises the exact
    // same dispatch — so scripted transcripts are faithful evidence.
    let result = if io::stdin().is_terminal() {
        run_interactive(&mut interp)
    } else {
        run_plain(&mut interp, io::stdin().lock(), io::stdout().lock())
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("REPL IO error: {e}");
            ExitCode::from(1)
        }
    }
}

// ════════════════════════════════════════════════════════════════════
// Pure command dispatch (unit-tested; no reedline, no I/O)
// ════════════════════════════════════════════════════════════════════

/// What the REPL should do after handling one complete input.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// Print this text (no trailing newline; the caller adds one).
    Print(String),
    /// Handled, nothing to print.
    Silent,
    /// Leave the REPL.
    Quit,
    /// A top-level item was loaded — completion candidates should refresh.
    Loaded(String),
}

/// Handle one complete input line/block against `interp`.
pub(crate) fn dispatch(interp: &mut Interpreter, input: &str) -> Outcome {
    let line = input.trim();
    if line.is_empty() {
        return Outcome::Silent;
    }
    match line {
        ":quit" | ":q" => return Outcome::Quit,
        ":help" | ":h" | ":?" => return Outcome::Print(help_text()),
        ":caps" => return Outcome::Print(caps_overview(interp)),
        _ => {}
    }
    // `?doc name` or the `?name` shorthand.
    if let Some(rest) = line
        .strip_prefix("?doc")
        .filter(|r| r.is_empty() || r.starts_with(' '))
    {
        return Outcome::Print(doc_text(interp, rest.trim()));
    }
    if let Some(rest) = line.strip_prefix('?') {
        return Outcome::Print(doc_text(interp, rest.trim()));
    }
    // Firewalled: a panic in `load_source`/`eval_expr_src` (e.g. the
    // `i64::MIN.abs()` overflow) becomes a printed error so the REPL session
    // survives — one bad expression must not kill the whole session.
    if looks_like_item(line) {
        return match crate::panic_firewall::firewalled(|| interp.load_source(line)) {
            Ok(Ok(())) => Outcome::Loaded("ok".into()),
            Ok(Err(e)) => Outcome::Print(format!("error: {e}")),
            Err(panic_msg) => Outcome::Print(format!("error: runtime panic: {panic_msg}")),
        };
    }
    match crate::panic_firewall::firewalled(|| interp.eval_expr_src(line)) {
        Ok(Ok(v)) => Outcome::Print(pretty(&v)),
        Ok(Err(e)) => Outcome::Print(format!("error: {e}")),
        Err(panic_msg) => Outcome::Print(format!("error: runtime panic: {panic_msg}")),
    }
}

/// Top-level item keywords (mirrors the interp REPL's classifier).
pub(crate) fn looks_like_item(s: &str) -> bool {
    let lead = s.split_whitespace().next().unwrap_or("");
    matches!(
        lead,
        "def"
            | "fn"
            | "struct"
            | "enum"
            | "trait"
            | "impl"
            | "actor"
            | "memory"
            | "module"
            | "use"
            | "pub"
            | "@safe"
            | "const"
    ) || s.starts_with('@')
}

/// Pretty-print an evaluated value. Scalars print bare; composite values carry
/// a `: Type` tag so a `<fn add>` or `[1, 2]` is self-describing.
pub(crate) fn pretty(v: &Value) -> String {
    let body = v.display();
    match v {
        Value::Nil
        | Value::Bool(_)
        | Value::Int(_)
        | Value::Float(_)
        | Value::Str(_)
        | Value::Symbol(_) => format!("=> {body}"),
        _ => format!("=> {body}  : {}", v.type_name()),
    }
}

fn help_text() -> String {
    [
        "REPL commands:",
        "  <expr>            evaluate and print a value",
        "  def name(..) {..} register a top-level item (def/struct/enum/impl/...)",
        "  ?doc <name>       show a primitive's doc, arity, and required caps",
        "  ?<name>           shorthand for ?doc <name>",
        "  :caps             show the session's declared + available authority",
        "  :help  :h         this help",
        "  :quit  :q         leave the REPL  (Ctrl-D also exits)",
        "",
        "Editing: history (Up/Down), multiline (open `{`/`(`/`[` continues the",
        "input), and Tab completion over commands, primitives, and live bindings.",
    ]
    .join("\n")
}

// ── ?doc ────────────────────────────────────────────────────────────

/// Render documentation for `name` — a stdlib primitive (by bare or qualified
/// name) or a user function loaded this session.
pub(crate) fn doc_text(interp: &Interpreter, name: &str) -> String {
    if name.is_empty() {
        return "usage: ?doc <name>   (e.g. `?doc read_file`, `?doc fs::read_file`)".into();
    }
    let prims = all_prims();
    // Exact qualified key, else unique bare last-segment match.
    let hit = prims.get(name).map(|m| (name.to_string(), m)).or_else(|| {
        let matches: Vec<_> = prims
            .iter()
            .filter(|(k, _)| k.rsplit("::").next() == Some(name))
            .collect();
        match matches.as_slice() {
            [(k, m)] => Some(((*k).clone(), *m)),
            _ => None,
        }
    });
    if let Some((key, m)) = hit {
        let caps = if m.required_caps.0.is_empty() {
            "none".to_string()
        } else {
            m.required_caps.0.join(", ")
        };
        return format!(
            "{key}  ({}-arg primitive)\n  caps: {caps}\n  layer: {:?} · stability: {:?}\n  {}",
            m.arity, m.layer, m.stability, m.doc
        );
    }
    // A user-defined function bound this session?
    if let Some(Value::Fn(f)) = interp.lookup_binding(name) {
        let caps = declared_caps(&f.def.annotations);
        let caps_str = if caps.is_empty() {
            "none declared".to_string()
        } else {
            caps.join(", ")
        };
        return format!(
            "{name}  ({}-arg user function)\n  declared caps: {caps_str}\n  (defined in this session)",
            f.def.params.len()
        );
    }
    let mut suggestions: Vec<&str> = prims
        .keys()
        .map(String::as_str)
        .filter(|k| k.contains(name) || k.rsplit("::").next().is_some_and(|s| s.contains(name)))
        .take(5)
        .collect();
    suggestions.sort_unstable();
    if suggestions.is_empty() {
        format!("no doc for `{name}` (not a known primitive or session binding)")
    } else {
        format!(
            "no exact match for `{name}`; did you mean: {}",
            suggestions.join(", ")
        )
    }
}

// ── :caps ───────────────────────────────────────────────────────────

/// The session's authority overview. Two honest sections:
///   1. what the loaded user functions *declare* they need (`@caps`), and
///   2. the *available* primitives grouped by the capability each requires.
///
/// This is a DECLARED / AVAILABLE surface, NOT an enforced runtime budget — a
/// bare call at the prompt holds no capability frame; `@caps` is enforced per
/// function at entry (interp S90). The header says so, so no reader mistakes it
/// for a live grant.
pub(crate) fn caps_overview(interp: &Interpreter) -> String {
    let mut out = String::from(
        "capability surface (declared + available — NOT an enforced budget;\n\
         @caps is enforced per-function at entry, S90):\n",
    );

    // 1. Session-declared authority (union over loaded user functions).
    let mut declared: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for name in interp.live_binding_names() {
        if let Some(Value::Fn(f)) = interp.lookup_binding(&name) {
            for cap in declared_caps(&f.def.annotations) {
                declared.entry(cap).or_default().push(name.clone());
            }
        }
    }
    out.push_str("\n  session-declared (by loaded @caps functions):\n");
    if declared.is_empty() {
        out.push_str("    (none — no loaded function declares @caps)\n");
    } else {
        for (cap, mut fns) in declared {
            fns.sort_unstable();
            fns.dedup();
            out.push_str(&format!("    {cap}: {}\n", fns.join(", ")));
        }
    }

    // 2. Available primitive authority, grouped by required capability.
    let mut by_cap: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for (key, meta) in all_prims() {
        for cap in &meta.required_caps.0 {
            by_cap.entry(cap).or_default().push(key.clone());
        }
    }
    out.push_str("\n  available primitives by capability:\n");
    if by_cap.is_empty() {
        out.push_str("    (none)\n");
    } else {
        for (cap, mut prims) in by_cap {
            prims.sort_unstable();
            out.push_str(&format!("    {cap}: {}\n", prims.join(", ")));
        }
    }
    out.trim_end().to_string()
}

/// Capability names declared by a function's `@caps(...)` annotation(s).
fn declared_caps(annotations: &[Annotation]) -> Vec<String> {
    let mut caps = Vec::new();
    for a in annotations {
        if let Annotation::Caps(cs, _) = a {
            for c in cs {
                caps.push(cap_name(c));
            }
        }
    }
    caps
}

fn cap_name(c: &Capability) -> String {
    c.as_str().to_string()
}

// ── completion (pure candidate computation) ─────────────────────────

/// A completion candidate plus a short kind label for the menu description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Candidate {
    pub value: String,
    pub kind: &'static str,
}

/// The full static + live candidate set: REPL commands, every primitive (bare
/// and qualified), and the live session bindings.
pub(crate) fn all_candidates(interp: &Interpreter) -> Vec<Candidate> {
    let mut out = Vec::new();
    for cmd in [":help", ":caps", ":quit", "?doc"] {
        out.push(Candidate {
            value: cmd.into(),
            kind: "command",
        });
    }
    for key in all_prims().keys() {
        out.push(Candidate {
            value: key.clone(),
            kind: "primitive",
        });
        if let Some(bare) = key.rsplit("::").next() {
            if bare != key {
                out.push(Candidate {
                    value: bare.to_string(),
                    kind: "primitive",
                });
            }
        }
    }
    for name in interp.live_binding_names() {
        out.push(Candidate {
            value: name,
            kind: "binding",
        });
    }
    out.sort_by(|a, b| a.value.cmp(&b.value));
    out.dedup_by(|a, b| a.value == b.value);
    out
}

/// The byte range `[start, pos)` of the identifier word ending at `pos`. An
/// identifier here is `[A-Za-z0-9_:?]` so `fs::read` and `?doc` complete whole.
pub(crate) fn current_word(line: &str, pos: usize) -> (usize, &str) {
    let mut pos = pos.min(line.len());
    // Clamp to a char boundary so the helper is total even if a caller passes a
    // byte offset mid-codepoint (reedline keeps the cursor on a boundary, so
    // this is defensive — but it makes `current_word` panic-free for any input).
    while pos > 0 && !line.is_char_boundary(pos) {
        pos -= 1;
    }
    let upto = &line[..pos];
    let start = upto
        .char_indices()
        .rev()
        .take_while(|(_, c)| c.is_alphanumeric() || *c == '_' || *c == ':' || *c == '?')
        .last()
        .map_or(pos, |(i, _)| i);
    (start, &line[start..pos])
}

/// Candidates whose value starts with the current word. Returns `(start,
/// matches)` so the caller can build replacement spans.
pub(crate) fn completion_candidates<'a>(
    candidates: &'a [Candidate],
    line: &str,
    pos: usize,
) -> (usize, Vec<&'a Candidate>) {
    let (start, word) = current_word(line, pos);
    if word.is_empty() {
        return (start, Vec::new());
    }
    let mut hits: Vec<&Candidate> = candidates
        .iter()
        .filter(|c| c.value.starts_with(word) && c.value != word)
        .collect();
    hits.sort_by(|a, b| a.value.cmp(&b.value));
    (start, hits)
}

// ── multiline detection (pure) ──────────────────────────────────────

/// Is `buf` a complete input, or should the REPL keep reading more lines?
/// Incomplete when a bracket is still open (string/comment aware), the buffer
/// ends with an explicit `\` continuation, or a leading annotation (`@caps`,
/// `@max_depth`, …) is still waiting for the item it decorates.
pub(crate) fn is_input_complete(buf: &str) -> bool {
    let t = buf.trim();
    if t.is_empty() {
        return true;
    }
    if t.ends_with('\\') {
        return false;
    }
    let scan = scan_delimiters(buf);
    // A Garnet string can't span lines; if the last content line ends mid-string
    // it is a (complete) lex error — dispatch it so the parser reports the
    // unterminated string rather than the prompt hanging forever.
    if scan.last_line_open_string {
        return true;
    }
    if scan.depth > 0 {
        return false;
    }
    // A dangling annotation (`@caps(net)` on its own) is balanced but not yet a
    // complete item — keep reading until the def/struct/... it prefixes lands,
    // so the annotation and its item are dispatched as ONE block.
    if t.starts_with('@') && !has_item_keyword(t) {
        return false;
    }
    true
}

/// Does `s` contain a top-level item keyword as a whole word? Used to tell a
/// dangling annotation apart from a complete annotated item.
fn has_item_keyword(s: &str) -> bool {
    s.split(|c: char| c.is_whitespace() || c == '(' || c == '{')
        .any(|w| {
            matches!(
                w,
                "def"
                    | "fn"
                    | "struct"
                    | "enum"
                    | "trait"
                    | "impl"
                    | "actor"
                    | "memory"
                    | "module"
                    | "use"
                    | "const"
            )
        })
}

struct DelimScan {
    /// Net open `(`/`[`/`{` across the whole buffer.
    depth: i32,
    /// Did the last non-blank line end inside an unterminated `"…` string?
    last_line_open_string: bool,
}

/// Scan bracket depth line-by-line. Garnet strings are **single-line** and only
/// `"` opens one (a bare `'` is a lexer error, not a char literal), so a string
/// is confined to its line and an unterminated `"` never holds the prompt open
/// across newlines — it surfaces as the last line's `last_line_open_string`.
/// `#` and `//` start line comments.
fn scan_delimiters(buf: &str) -> DelimScan {
    let mut depth: i32 = 0;
    let mut last_line_open_string = false;
    for line in buf.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let mut in_str = false;
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            if in_str {
                if c == '\\' {
                    chars.next(); // skip the escaped char
                } else if c == '"' {
                    in_str = false;
                }
                continue;
            }
            match c {
                '"' => in_str = true,
                '#' => break, // rest of line is a comment
                '/' if chars.peek() == Some(&'/') => break,
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                _ => {}
            }
        }
        last_line_open_string = in_str;
    }
    DelimScan {
        depth,
        last_line_open_string,
    }
}

// ════════════════════════════════════════════════════════════════════
// Plain (non-TTY) loop — pipes, CI, demo transcripts
// ════════════════════════════════════════════════════════════════════

const BANNER: &str = "Garnet REPL — RB-7. :help for commands, :quit to exit.";

/// Line-buffered loop over any reader/writer. Accumulates continuation lines
/// (via [`is_input_complete`]) so piped multiline blocks work without a TTY.
pub(crate) fn run_plain<R: io::BufRead, W: Write>(
    interp: &mut Interpreter,
    mut input: R,
    mut out: W,
) -> io::Result<()> {
    writeln!(out, "{BANNER}")?;
    let mut buf = String::new();
    loop {
        write!(
            out,
            "{}",
            if buf.is_empty() {
                "garnet> "
            } else {
                "   ...> "
            }
        )?;
        out.flush()?;
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            break; // EOF
        }
        buf.push_str(&line);
        if !is_input_complete(&buf) {
            continue;
        }
        let block = std::mem::take(&mut buf);
        match dispatch(interp, &block) {
            Outcome::Quit => break,
            Outcome::Silent => {}
            Outcome::Print(s) | Outcome::Loaded(s) => writeln!(out, "{s}")?,
        }
    }
    // EOF with a non-empty buffer = an unterminated trailing block (e.g. an
    // unclosed brace or string). Dispatch it once so its parse error SURFACES
    // instead of being silently dropped — this path is the CI/demo evidence
    // path, so a silent swallow would be a faithfulness bug.
    if !buf.trim().is_empty() {
        match dispatch(interp, &buf) {
            Outcome::Quit | Outcome::Silent => {}
            Outcome::Print(s) | Outcome::Loaded(s) => writeln!(out, "{s}")?,
        }
    }
    Ok(())
}

// ════════════════════════════════════════════════════════════════════
// Interactive (reedline) loop — TTY
// ════════════════════════════════════════════════════════════════════

fn run_interactive(interp: &mut Interpreter) -> io::Result<()> {
    use reedline::{
        default_emacs_keybindings, ColumnarMenu, Emacs, KeyCode, KeyModifiers, MenuBuilder,
        Reedline, ReedlineEvent, ReedlineMenu, Signal,
    };
    use std::sync::{Arc, Mutex};

    println!("{BANNER}");

    // Shared, live-updated candidate set (Send-safe for reedline's Completer).
    let shared = Arc::new(Mutex::new(all_candidates(interp)));
    let completer = Box::new(GarnetCompleter {
        shared: Arc::clone(&shared),
    });

    let menu = Box::new(ColumnarMenu::default().with_name("completion_menu"));
    let mut keybindings = default_emacs_keybindings();
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".to_string()),
            ReedlineEvent::MenuNext,
        ]),
    );

    let mut editor = Reedline::create()
        .with_completer(completer)
        .with_validator(Box::new(GarnetValidator))
        .with_menu(ReedlineMenu::EngineCompleter(menu))
        .with_edit_mode(Box::new(Emacs::new(keybindings)));
    let prompt = GarnetPrompt;

    loop {
        match editor.read_line(&prompt) {
            Ok(Signal::Success(line)) => match dispatch(interp, &line) {
                Outcome::Quit => break,
                Outcome::Silent => {}
                Outcome::Print(s) => println!("{s}"),
                Outcome::Loaded(s) => {
                    println!("{s}");
                    // A new binding/item may have appeared — refresh completion.
                    if let Ok(mut c) = shared.lock() {
                        *c = all_candidates(interp);
                    }
                }
            },
            Ok(Signal::CtrlC) => continue, // abandon the current line
            Ok(Signal::CtrlD) => break,    // EOF — leave
            Ok(_) => continue,             // other signals (e.g. host payloads) — ignore
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

struct GarnetCompleter {
    shared: std::sync::Arc<std::sync::Mutex<Vec<Candidate>>>,
}

impl reedline::Completer for GarnetCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<reedline::Suggestion> {
        let candidates = match self.shared.lock() {
            Ok(c) => c.clone(),
            Err(_) => return Vec::new(),
        };
        let (start, hits) = completion_candidates(&candidates, line, pos);
        hits.into_iter()
            .map(|c| reedline::Suggestion {
                value: c.value.clone(),
                description: Some(c.kind.to_string()),
                style: None,
                extra: None,
                span: reedline::Span::new(start, pos),
                append_whitespace: false,
                display_override: None,
                match_indices: None,
            })
            .collect()
    }
}

struct GarnetValidator;

impl reedline::Validator for GarnetValidator {
    fn validate(&self, line: &str) -> reedline::ValidationResult {
        if is_input_complete(line) {
            reedline::ValidationResult::Complete
        } else {
            reedline::ValidationResult::Incomplete
        }
    }
}

struct GarnetPrompt;

impl reedline::Prompt for GarnetPrompt {
    fn render_prompt_left(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed("garnet")
    }
    fn render_prompt_right(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed("")
    }
    fn render_prompt_indicator(
        &self,
        _mode: reedline::PromptEditMode,
    ) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed("> ")
    }
    fn render_prompt_multiline_indicator(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed("   ...> ")
    }
    fn render_prompt_history_search_indicator(
        &self,
        history_search: reedline::PromptHistorySearch,
    ) -> std::borrow::Cow<'_, str> {
        let prefix = match history_search.status {
            reedline::PromptHistorySearchStatus::Passing => "",
            reedline::PromptHistorySearchStatus::Failing => "failing ",
        };
        std::borrow::Cow::Owned(format!(
            "({prefix}reverse-search: {}) ",
            history_search.term
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn interp() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn evaluates_an_expression_and_pretty_prints() {
        let mut i = interp();
        assert_eq!(dispatch(&mut i, "1 + 2 * 3"), Outcome::Print("=> 7".into()));
    }

    #[test]
    fn a_panicking_expression_is_reported_and_the_session_survives() {
        let mut i = interp();
        // i64::MIN.abs() overflows → a Rust panic in the interpreter. Before the
        // firewall this aborted the whole REPL process; now `dispatch` returns a
        // printed error (no panic) and the SAME interpreter keeps working.
        match dispatch(&mut i, "(0 - 9223372036854775807 - 1).abs()") {
            Outcome::Print(s) => assert!(
                s.contains("runtime panic") && s.contains("overflow"),
                "expected a runtime-panic report, got: {s}"
            ),
            other => panic!("expected Print (session survives), got {other:?}"),
        }
        // The session is still usable after the panic.
        assert_eq!(dispatch(&mut i, "40 + 2"), Outcome::Print("=> 42".into()));
    }

    #[test]
    fn composite_values_carry_a_type_tag() {
        let mut i = interp();
        match dispatch(&mut i, "[1, 2, 3]") {
            Outcome::Print(s) => {
                assert!(
                    s.starts_with("=> [1, 2, 3]") && s.contains(": Array"),
                    "got: {s}"
                )
            }
            other => panic!("expected Print, got {other:?}"),
        }
    }

    #[test]
    fn quit_and_help_and_blank() {
        let mut i = interp();
        assert_eq!(dispatch(&mut i, ":quit"), Outcome::Quit);
        assert_eq!(dispatch(&mut i, ":q"), Outcome::Quit);
        assert_eq!(dispatch(&mut i, "   "), Outcome::Silent);
        assert!(matches!(dispatch(&mut i, ":help"), Outcome::Print(s) if s.contains("?doc")));
    }

    #[test]
    fn defining_a_function_loads_then_is_callable() {
        let mut i = interp();
        assert!(matches!(
            dispatch(&mut i, "def add(a, b) { a + b }"),
            Outcome::Loaded(_)
        ));
        assert_eq!(dispatch(&mut i, "add(2, 3)"), Outcome::Print("=> 5".into()));
    }

    #[test]
    fn doc_of_a_primitive_shows_caps_and_arity() {
        let i = interp();
        let d = doc_text(&i, "read_file");
        assert!(d.contains("read_file"), "got: {d}");
        assert!(
            d.contains("caps: fs"),
            "fs-requiring primitive should show its cap: {d}"
        );
    }

    #[test]
    fn doc_shorthand_and_unknown() {
        let mut i = interp();
        assert!(
            matches!(dispatch(&mut i, "?read_file"), Outcome::Print(s) if s.contains("caps: fs"))
        );
        assert!(matches!(
            dispatch(&mut i, "?doc nonsuch_xyz"),
            Outcome::Print(s) if s.contains("no doc") || s.contains("did you mean")
        ));
    }

    #[test]
    fn doc_of_a_user_function_reads_declared_caps() {
        let mut i = interp();
        let _ = dispatch(&mut i, "@caps(fs)\ndef reader() { 1 }");
        let d = doc_text(&i, "reader");
        assert!(d.contains("user function"), "got: {d}");
        assert!(d.contains("fs"), "declared caps should surface: {d}");
    }

    #[test]
    fn caps_overview_is_labeled_not_an_enforced_budget() {
        let i = interp();
        let c = caps_overview(&i);
        assert!(
            c.contains("NOT an enforced budget"),
            "honesty label required: {c}"
        );
        assert!(
            c.contains("fs:"),
            "fs primitives should be grouped under fs: {c}"
        );
    }

    #[test]
    fn caps_overview_reflects_session_declarations() {
        let mut i = interp();
        let _ = dispatch(&mut i, "@caps(net)\ndef fetch() { 1 }");
        let c = caps_overview(&i);
        assert!(c.contains("session-declared"), "got: {c}");
        assert!(
            c.contains("fetch"),
            "a loaded @caps(net) fn should appear: {c}"
        );
    }

    #[test]
    fn multiline_holds_open_brace_then_completes() {
        assert!(!is_input_complete("def f() {"));
        assert!(!is_input_complete("def f() {\n  1 +"));
        assert!(is_input_complete("def f() {\n  1\n}"));
        assert!(is_input_complete("1 + 2"));
        assert!(is_input_complete(":caps"));
    }

    #[test]
    fn multiline_ignores_braces_inside_strings_and_comments() {
        assert!(is_input_complete(r#"print("{ not a real brace }")"#));
        assert!(is_input_complete("1 + 1 # trailing { comment"));
        assert!(!is_input_complete("f(\"(\"")); // a genuinely open paren
    }

    #[test]
    fn explicit_backslash_continues() {
        assert!(!is_input_complete("1 + \\"));
    }

    #[test]
    fn single_line_unterminated_string_does_not_hang() {
        // Garnet strings are single-line; an unterminated `"` is a complete lex
        // error, NOT a reason to keep reading. (Adversarial-review fix: it used
        // to hold the prompt open forever.)
        assert!(is_input_complete("print(\"hi"));
        assert!(is_input_complete("\"oops"));
        // A bare `'` is not a string delimiter (no char literals in Garnet), so
        // it does not hold input open by itself.
        assert!(is_input_complete("x = 'a"));
    }

    #[test]
    fn eof_surfaces_an_unterminated_trailing_block() {
        // Regression: run_plain must NOT silently drop a non-empty buffer at EOF
        // — the parse error has to surface (this is the CI/demo evidence path).
        let mut i = interp();
        let script = "1 + 1\ndef broken( {\n"; // never closes; no :quit
        let mut out: Vec<u8> = Vec::new();
        run_plain(&mut i, io::Cursor::new(script), &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(
            text.contains("=> 2"),
            "earlier complete line still ran: {text}"
        );
        assert!(
            text.contains("error"),
            "the unterminated trailing block must surface an error, not vanish: {text}"
        );
    }

    #[test]
    fn dangling_annotation_waits_for_its_item() {
        // A bare `@caps(net)` is delimiter-balanced but NOT a complete item —
        // dispatching it alone would drop the annotation from its def.
        assert!(!is_input_complete("@caps(net)"));
        assert!(!is_input_complete("@caps(fs)\n@max_depth(10)"));
        // Once the item arrives (and braces close) it is complete.
        assert!(is_input_complete("@caps(net)\ndef fetch() { 1 }"));
        assert!(!is_input_complete("@caps(net)\ndef fetch() {"));
    }

    #[test]
    fn plain_loop_keeps_annotation_with_its_def() {
        // Regression: `@caps(net)` and `def fetch` on separate lines must load
        // as one block so `:caps` sees the declared authority.
        let mut i = interp();
        let script = "@caps(net)\ndef fetch() { 1 }\n:caps\n:quit\n";
        let mut out: Vec<u8> = Vec::new();
        run_plain(&mut i, io::Cursor::new(script), &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(
            !text.contains("parse error"),
            "annotation split from def: {text}"
        );
        assert!(
            text.contains("net: fetch"),
            "session-declared net authority should list fetch: {text}"
        );
    }

    #[test]
    fn current_word_finds_the_identifier_under_the_cursor() {
        assert_eq!(current_word("read_fi", 7), (0, "read_fi"));
        assert_eq!(current_word("a + read_fi", 11), (4, "read_fi"));
        assert_eq!(current_word("fs::read", 8), (0, "fs::read"));
        assert_eq!(current_word("", 0), (0, ""));
    }

    #[test]
    fn completion_matches_commands_primitives_and_bindings() {
        let mut i = interp();
        let _ = dispatch(&mut i, "def myhelper() { 1 }");
        let cands = all_candidates(&i);

        let (start, hits) = completion_candidates(&cands, ":ca", 3);
        assert_eq!(start, 0);
        assert!(
            hits.iter().any(|c| c.value == ":caps"),
            "command completion"
        );

        let (_, hits) = completion_candidates(&cands, "read_f", 6);
        assert!(
            hits.iter().any(|c| c.value == "read_file"),
            "primitive completion"
        );

        let (_, hits) = completion_candidates(&cands, "myhel", 5);
        assert!(
            hits.iter().any(|c| c.value == "myhelper"),
            "live-binding completion"
        );
    }

    #[test]
    fn plain_loop_runs_a_scripted_session() {
        let mut i = interp();
        let script = "1 + 1\ndef sq(x) { x * x }\nsq(9)\n:quit\n";
        let mut out: Vec<u8> = Vec::new();
        run_plain(&mut i, io::Cursor::new(script), &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("=> 2"), "got: {text}");
        assert!(text.contains("=> 81"), "got: {text}");
    }
}
