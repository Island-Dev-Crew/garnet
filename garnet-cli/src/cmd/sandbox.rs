//! `garnet sandbox <file>` — generate sandbox policy from declared `@caps` (S46).
//!
//! Derives the file's capability surface (S35) and prints the seccomp / WASI /
//! egress policy it implies. **Generation only** — see `crate::sandbox` and
//! `C_Language_Specification/GARNET_SANDBOX_POLICY.md` for the honest scope
//! (nothing is enforced at runtime here).

use crate::cap_manifest::surface_for_path;
use crate::sandbox::sandbox_policy;
use std::path::PathBuf;
use std::process::ExitCode;

enum Format {
    Human,
    Json,
}

pub fn run(args: &[String]) -> ExitCode {
    let mut format = Format::Human;
    let mut file: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--format" => {
                match args.get(i + 1).map(String::as_str) {
                    Some("human") => format = Format::Human,
                    Some("json") => format = Format::Json,
                    other => {
                        eprintln!("--format expects human|json, got {other:?}");
                        return ExitCode::from(2);
                    }
                }
                i += 2;
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
                eprintln!("unknown sandbox flag: {other}");
                return ExitCode::from(2);
            }
        }
    }

    let Some(file) = file else {
        print_help();
        return ExitCode::from(2);
    };

    let surface = match surface_for_path(&PathBuf::from(&file)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("garnet sandbox: {e}");
            return ExitCode::from(1);
        }
    };
    let policy = sandbox_policy(&surface.aggregate);

    match format {
        Format::Human => print!("{}", policy.to_human()),
        Format::Json => println!("{}", policy.to_json()),
    }
    ExitCode::SUCCESS
}

fn print_help() {
    println!("usage: garnet sandbox [--format human|json] <file.garnet>");
    println!();
    println!("  Generate the seccomp / WASI / egress sandbox policy implied by a");
    println!("  file's declared `@caps(...)`. Generation only — nothing is enforced");
    println!("  at runtime (see GARNET_SANDBOX_POLICY.md).");
}
