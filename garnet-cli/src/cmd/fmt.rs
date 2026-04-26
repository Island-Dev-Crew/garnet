//! `garnet fmt <file>` — whitespace-normalize a Garnet source file.
//!
//! ## v0.4.2 scope (honest)
//!
//! This is **not** a full code formatter. The parser is trivia-dropping
//! (comments and whitespace are not preserved on the AST), so a
//! parse-then-re-emit round trip would silently delete every comment
//! in the file. Until the parser grows a CST layer, `garnet fmt` is
//! limited to safe, byte-level normalizations:
//!
//! - Strip trailing whitespace on every line.
//! - Convert CRLF / CR line endings to LF.
//! - Ensure exactly one terminal newline at end of file.
//! - Verify the source still parses after normalization (so a bad
//!   input never becomes a worse one).
//!
//! ## Modes
//!
//! - `garnet fmt <file>` — write the normalized form back to `<file>`.
//! - `garnet fmt --check <file>` — report changes that *would* be
//!   made; exit 0 if clean, 1 if changes needed. Suitable for CI.
//! - `garnet fmt --stdout <file>` — print the normalized form to
//!   stdout, leave the file untouched.
//!
//! Full AST-driven rewriting (alignment, spacing rules, import
//! sorting) is tracked as a v0.5.x roadmap item; it gates on a
//! trivia-preserving CST in the parser.

use crate::read_file;
use std::path::PathBuf;
use std::process::ExitCode;

pub fn run(args: &[String]) -> ExitCode {
    let mut check = false;
    let mut to_stdout = false;
    let mut file: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--check" => {
                check = true;
                i += 1;
            }
            "--stdout" => {
                to_stdout = true;
                i += 1;
            }
            "--help" | "-h" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            other if !other.starts_with("--") => {
                file = Some(args[i].clone());
                i += 1;
            }
            other => {
                eprintln!("unknown fmt flag: {other}");
                return ExitCode::from(2);
            }
        }
    }

    let Some(file) = file else {
        print_help();
        return ExitCode::from(2);
    };
    let path = PathBuf::from(&file);
    let original = match read_file(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };

    let normalized = normalize(&original);

    // Verify the normalized form still parses — never let a fmt pass
    // turn well-formed Garnet into something that does not parse.
    if let Err(e) = garnet_parser::parse_source(&normalized) {
        eprintln!("garnet fmt refused: normalized output failed to parse");
        let report = miette::Report::new(e).with_source_code(normalized.clone());
        eprintln!("{report:?}");
        return ExitCode::from(1);
    }

    if to_stdout {
        print!("{normalized}");
        return ExitCode::SUCCESS;
    }

    if normalized == original {
        if !check {
            // Nothing to do; mirror rustfmt's silent success.
        }
        return ExitCode::SUCCESS;
    }

    if check {
        eprintln!("garnet fmt: {} would be reformatted", path.display());
        return ExitCode::from(1);
    }

    if let Err(e) = std::fs::write(&path, &normalized) {
        eprintln!("garnet fmt: failed to write {}: {e}", path.display());
        return ExitCode::from(1);
    }
    println!("formatted {}", path.display());
    ExitCode::SUCCESS
}

/// Apply the v0.4.2 whitespace-normalization rules. Pure function — no
/// I/O, no parser; safe to test directly.
fn normalize(src: &str) -> String {
    // Step 1: normalize line endings. Treat CRLF and bare CR as LF so
    // the per-line trim below works uniformly.
    let unix = src.replace("\r\n", "\n").replace('\r', "\n");

    // Step 2: strip trailing whitespace from every line.
    let mut out = String::with_capacity(unix.len());
    let mut lines = unix.split('\n').peekable();
    while let Some(line) = lines.next() {
        out.push_str(line.trim_end());
        if lines.peek().is_some() {
            out.push('\n');
        }
    }

    // Step 3: ensure exactly one trailing newline. The split above
    // produces an empty trailing element if the input ended with '\n',
    // so a non-empty `out` that did not push the final '\n' covers the
    // "file with no trailing newline" case.
    if !out.ends_with('\n') {
        out.push('\n');
    }
    // Strip excess blank trailing lines (keep at most one final '\n').
    while out.ends_with("\n\n") {
        out.pop();
    }

    out
}

fn print_help() {
    println!("usage: garnet fmt [--check | --stdout] <file.garnet>");
    println!();
    println!("  Normalize whitespace in a Garnet source file:");
    println!("    - strip trailing whitespace per line");
    println!("    - convert CRLF / CR to LF");
    println!("    - ensure exactly one trailing newline");
    println!("    - verify the result still parses (refuses to break valid input)");
    println!();
    println!("  Modes:");
    println!("    (default)        rewrite the file in place");
    println!("    --check          report if changes are needed; exit 1 if so (CI use)");
    println!("    --stdout         print the normalized form; leave the file untouched");
    println!();
    println!("  v0.4.2 scope: whitespace only. Full AST-driven formatting (alignment,");
    println!("  import sorting, spacing rules) is a v0.5.x roadmap item — gates on a");
    println!("  trivia-preserving CST in the parser.");
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn strips_trailing_whitespace() {
        assert_eq!(
            normalize("def main()   \n  let x = 1   \n"),
            "def main()\n  let x = 1\n"
        );
    }

    #[test]
    fn normalizes_crlf_to_lf() {
        assert_eq!(normalize("a\r\nb\r\nc\r\n"), "a\nb\nc\n");
    }

    #[test]
    fn ensures_terminal_newline() {
        assert_eq!(normalize("def main() {}"), "def main() {}\n");
    }

    #[test]
    fn collapses_multiple_terminal_newlines() {
        assert_eq!(normalize("def main() {}\n\n\n\n"), "def main() {}\n");
    }

    #[test]
    fn idempotent_on_clean_input() {
        let clean = "def main() {\n  let x = 1\n}\n";
        assert_eq!(normalize(clean), clean);
    }
}
