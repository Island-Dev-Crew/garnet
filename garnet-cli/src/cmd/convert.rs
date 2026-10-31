//! `garnet convert <lang> <file>` — run the migration assistant. Writes
//! `<stem>.<lang>.garnet` + `.lineage.json` + `.migrate_todo.md` +
//! `.metrics.json` beside the source, or under `--out <dir>` if supplied.
//! Output is a scaffolded port that parses but is unreviewed (an `@sandbox`
//! comment, an empty `@caps()`, and `MigrateTodo`/`Untranslatable`
//! placeholders) — see `garnet-convert` crate docs for the scope caveats.

use crate::convert_cmd::{self, ConvertArgs};
use std::path::PathBuf;
use std::process::ExitCode;

pub fn run(args: &[String]) -> ExitCode {
    // `new` subcommand pattern — accept flags in any order.
    let mut source_lang: Option<String> = None;
    let mut source_path: Option<String> = None;
    let mut out_dir: Option<String> = None;
    let mut strict = false;
    let mut fail_on_todo = false;
    let mut fail_on_untranslatable = false;
    let mut quiet = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--lang" => {
                if i + 1 >= args.len() {
                    eprintln!("--lang requires an argument (rust/ruby/python/go)");
                    return ExitCode::from(2);
                }
                source_lang = Some(args[i + 1].clone());
                i += 2;
            }
            "--out" => {
                if i + 1 >= args.len() {
                    eprintln!("--out requires a directory argument");
                    return ExitCode::from(2);
                }
                out_dir = Some(args[i + 1].clone());
                i += 2;
            }
            "--strict" => {
                strict = true;
                i += 1;
            }
            "--fail-on-todo" => {
                fail_on_todo = true;
                i += 1;
            }
            "--fail-on-untranslatable" => {
                fail_on_untranslatable = true;
                i += 1;
            }
            "--quiet" => {
                quiet = true;
                i += 1;
            }
            "--help" | "-h" => {
                println!("usage: garnet convert [--lang <lang>] [--out <dir>] [--strict] [--fail-on-todo] [--fail-on-untranslatable] [--quiet] <lang> <file>");
                println!("  langs: rust, ruby, python, go (also inferred from file extension)");
                return ExitCode::SUCCESS;
            }
            other if !other.starts_with("--") => {
                // Positional: first is lang (unless --lang flag already set), second is file.
                if source_lang.is_none() && source_path.is_none() {
                    source_lang = Some(args[i].clone());
                } else if source_path.is_none() {
                    source_path = Some(args[i].clone());
                } else {
                    eprintln!("unexpected extra positional argument: {other}");
                    return ExitCode::from(2);
                }
                i += 1;
            }
            other => {
                eprintln!("unknown convert flag: {other}");
                return ExitCode::from(2);
            }
        }
    }

    let Some(file) = source_path else {
        eprintln!("usage: garnet convert <lang> <file>");
        return ExitCode::from(2);
    };
    let lang = source_lang.unwrap_or_default();

    let convert_args = ConvertArgs {
        source_lang: lang,
        source_path: PathBuf::from(&file),
        strict,
        fail_on_todo,
        fail_on_untranslatable,
        out_dir: out_dir.map(PathBuf::from),
        quiet,
    };

    // `convert_cmd::run` prints the one summary; this wrapper only sets the exit code.
    match convert_cmd::run(convert_args) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("garnet convert failed: {e}");
            ExitCode::from(1)
        }
    }
}
