//! Reusable REPL core — used by both the interp crate's example binary and the
//! top-level `garnet` CLI. Uses `std::io` (no rustyline dependency) so the
//! interp crate stays minimal; the CLI crate can layer line editing on top.

use crate::{Interpreter, Value};
use std::io::{self, BufRead, Write};

/// The wordmark shown at REPL startup. Kept literal here (rather than
/// imported from garnet-cli's `GARNET_WORDMARK`) because the interp crate
/// is an upstream dependency of cli — the reverse import would create a
/// cycle. Duplication is 7 lines; the next refactor can lift both to a
/// shared `garnet-ui` crate if it becomes a pattern.
const REPL_WORDMARK: &str = concat!(
    "                                                  \n",
    "   ####   ###  ####  #   # ####### ##### ######   \n",
    "  #    # #   # #   # ##  # #         #     #      \n",
    "  #      ##### ####  # # # #####     #     #      \n",
    "  #  ### #   # #  #  #  ## #         #     #      \n",
    "  #    # #   # #   # #   # #         #     #      \n",
    "   ####  #   # #   # #   # #######   #     #      ",
);

pub struct Repl {
    interp: Interpreter,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            interp: Interpreter::new(),
        }
    }

    /// Load source code (e.g. a file) into the REPL before taking user input.
    pub fn preload(&mut self, src: &str) -> Result<(), String> {
        self.interp
            .load_source(src)
            .map_err(|e| format!("{e}"))
    }

    /// Run the interactive loop on stdin/stdout.
    pub fn run_stdio(&mut self) -> io::Result<()> {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut out = stdout.lock();
        // Phase 6C — REPL prompt banner. Mirrors the wordmark shown by
        // `garnet --version` / `garnet --help`. Kept ASCII-plain here (no
        // ANSI coloring) because the interp crate has no dep on is-terminal
        // — the CLI crate's wrapper can prepend a colored banner if it
        // wants. A future refactor can pull `colored_wordmark` up to a
        // shared util once the CLI and interp agree on a common "ui" crate.
        writeln!(out, "{}", REPL_WORDMARK)?;
        writeln!(out, "  Rust Rigor. Ruby Velocity. One Coherent Language.")?;
        writeln!(out)?;
        writeln!(out, "Garnet REPL — Rung 3. Type :quit to exit.")?;
        writeln!(out, r#"  (a plain expression prints its value; "def name(...) {{ ... }}" registers a function)"#)?;
        loop {
            write!(out, "garnet> ")?;
            out.flush()?;
            let mut line = String::new();
            let n = stdin.lock().read_line(&mut line)?;
            if n == 0 {
                break; // EOF
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed == ":quit" || trimmed == ":q" {
                break;
            }
            // Try to parse as a top-level item first (def, struct, etc.).
            if looks_like_item(trimmed) {
                match self.interp.load_source(trimmed) {
                    Ok(()) => writeln!(out, "ok")?,
                    Err(e) => writeln!(out, "error: {e}")?,
                }
            } else {
                match self.interp.eval_expr_src(trimmed) {
                    Ok(v) => writeln!(out, "=> {}", v.display())?,
                    Err(e) => writeln!(out, "error: {e}")?,
                }
            }
        }
        Ok(())
    }

    /// Evaluate a single line and return the result (used by testing and
    /// non-interactive integrations).
    pub fn eval_line(&mut self, line: &str) -> Result<Value, String> {
        if looks_like_item(line) {
            self.interp.load_source(line).map_err(|e| format!("{e}"))?;
            Ok(Value::Nil)
        } else {
            self.interp.eval_expr_src(line).map_err(|e| format!("{e}"))
        }
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}

fn looks_like_item(s: &str) -> bool {
    let lead = s
        .split_whitespace()
        .next()
        .unwrap_or("");
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
