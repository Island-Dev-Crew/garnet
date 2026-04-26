//! `garnet new --template <name> <dir>` — scaffold a new project from one
//! of the bundled templates. Phase 6B (v4.2).

use crate::new_cmd;
use std::path::PathBuf;
use std::process::ExitCode;

pub fn run(args: &[String]) -> ExitCode {
    // Accept `new --template <name> <dir>` or the shorter `new <dir>`
    // (defaults to `cli`). `-t` is accepted as a `--template` alias.
    let mut template: Option<String> = None;
    let mut dir_opt: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--template" | "-t" => {
                if i + 1 >= args.len() {
                    eprintln!("--template requires a template name");
                    return ExitCode::from(2);
                }
                template = Some(args[i + 1].clone());
                i += 2;
            }
            "--help" | "-h" => {
                println!("usage: garnet new [--template <name>] <dir>");
                println!("  templates:");
                for (key, desc) in new_cmd::template_descriptions() {
                    println!("    {key:<20} {desc}");
                }
                return ExitCode::SUCCESS;
            }
            other if !other.starts_with("--") => {
                dir_opt = Some(args[i].clone());
                i += 1;
            }
            other => {
                eprintln!("unknown `new` flag: {other}");
                return ExitCode::from(2);
            }
        }
    }
    let Some(dir) = dir_opt else {
        eprintln!("usage: garnet new [--template <name>] <dir>");
        return ExitCode::from(2);
    };
    let template_key = template.unwrap_or_else(|| "cli".to_string());
    let target = PathBuf::from(dir);

    match new_cmd::create_project(&template_key, &target) {
        Ok(report) => {
            // Colored-free output — Phase 6C adds terminal coloring via
            // `is-terminal` detection; the wordmark itself is emitted by
            // `print_help` / `print_version` and intentionally NOT repeated
            // per scaffolded project.
            println!(
                "Created `{}` from template `{}` ({} files)",
                report.root.display(),
                report.template,
                report.files_written.len(),
            );
            for f in &report.files_written {
                println!("  + {f}");
            }
            print!("{}", new_cmd::next_steps_hint(&report));
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("garnet new failed: {e}");
            ExitCode::from(1)
        }
    }
}
