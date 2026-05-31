//! `garnet check <file>` — run the safe-mode checker (CapCaps + borrow + audit).

use super::{cache_file_label, record, surface_prior};
use crate::diagnostics;
use crate::read_file;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

/// Output format for `garnet check` (S34). `Human` is the default miette/Display
/// rendering; `Json` emits deterministic structured diagnostics on stdout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckFormat {
    Human,
    Json,
}

pub fn run(path: PathBuf, suggest: bool, format: CheckFormat) -> ExitCode {
    if matches!(format, CheckFormat::Json) {
        return run_json(path);
    }
    let started = Instant::now();
    let src = match read_file(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };
    let file_label = cache_file_label(&path);
    surface_prior(&src);
    let edition = match crate::edition_manifest::resolve_edition_for(&path) {
        Ok(resolved) => {
            if let Some(warning) = resolved.warning {
                eprintln!("{warning}");
            }
            resolved.edition
        }
        Err(message) => {
            eprintln!("{message}");
            record(
                "check",
                &file_label,
                &src,
                "parse_err",
                Some("bad_edition".to_string()),
                started,
                1,
            );
            return ExitCode::from(1);
        }
    };
    let module = match garnet_parser::parse_source_with_edition(&src, edition) {
        Ok(m) => m,
        Err(e) => {
            let report = miette::Report::new(e).with_source_code(src.clone());
            eprintln!("{report:?}");
            record(
                "check",
                &file_label,
                &src,
                "parse_err",
                Some("UnexpectedToken".to_string()),
                started,
                1,
            );
            return ExitCode::from(1);
        }
    };
    let report = garnet_check::check_module(&module);
    for err in &report.errors {
        println!("{err}");
    }
    println!(
        "\n{} functions checked, {} boundary call sites, {} diagnostics",
        report.mode_map.len(),
        report.boundary_call_sites,
        report.errors.len()
    );
    // Layer 2: GODEBUG-style runtime settings. `GARNET_DEBUG=diagnostics=verbose`
    // flips a CLI default without touching the program, AST, or capability set.
    let settings = crate::runtime_settings::RuntimeSettings::from_env();
    if let Some(warning) = settings.unknown_key_warning() {
        eprintln!("{warning}");
    }
    if settings.verbose_diagnostics {
        println!("\n[GARNET_DEBUG diagnostics=verbose] per-function capability sets:");
        for (name, caps) in &report.fn_caps {
            println!("  {name}: [{}]", caps.join(", "));
        }
    }
    if suggest {
        let suggestions = garnet_check::suggest::suggest_for_module(&module);
        println!(
            "\n{} advisory suggestion{}:",
            suggestions.len(),
            if suggestions.len() == 1 { "" } else { "s" },
        );
        for s in &suggestions {
            println!("- {}", garnet_check::suggest::render(s));
        }
    }
    if report.ok() {
        record("check", &file_label, &src, "ok", None, started, 0);
        ExitCode::SUCCESS
    } else {
        record(
            "check",
            &file_label,
            &src,
            "check_err",
            Some("safe_violation".to_string()),
            started,
            1,
        );
        ExitCode::from(1)
    }
}

/// `--format json`: emit structured diagnostics as deterministic JSON on stdout,
/// honoring the authoritative exit codes ([`diagnostics::EXIT_OK`] /
/// [`diagnostics::EXIT_DIAGNOSTICS`]). Prior-failure notes and edition
/// deprecation warnings go to stderr, so stdout stays pure JSON.
fn run_json(path: PathBuf) -> ExitCode {
    let started = Instant::now();
    let src = match read_file(&path) {
        Ok(s) => s,
        Err(e) => {
            let diag = diagnostics::Diagnostic {
                severity: diagnostics::Severity::Error,
                code: "io.read_error",
                message: e,
                span: None,
            };
            println!("{}", diagnostics::to_json(std::slice::from_ref(&diag)));
            return ExitCode::from(diagnostics::EXIT_DIAGNOSTICS);
        }
    };
    let file_label = cache_file_label(&path);
    let edition = match crate::edition_manifest::resolve_edition_for(&path) {
        Ok(resolved) => {
            if let Some(warning) = resolved.warning {
                eprintln!("{warning}");
            }
            resolved.edition
        }
        Err(message) => {
            let diag = diagnostics::Diagnostic {
                severity: diagnostics::Severity::Error,
                code: "manifest.bad_edition",
                message,
                span: None,
            };
            println!("{}", diagnostics::to_json(std::slice::from_ref(&diag)));
            record(
                "check",
                &file_label,
                &src,
                "parse_err",
                Some("bad_edition".to_string()),
                started,
                1,
            );
            return ExitCode::from(diagnostics::EXIT_DIAGNOSTICS);
        }
    };
    match garnet_parser::parse_source_with_edition(&src, edition) {
        Ok(module) => {
            let report = garnet_check::check_module(&module);
            let diags = diagnostics::from_check_report(&report);
            println!("{}", diagnostics::to_json(&diags));
            let ok = report.ok();
            record(
                "check",
                &file_label,
                &src,
                if ok { "ok" } else { "check_err" },
                if ok {
                    None
                } else {
                    Some("safe_violation".to_string())
                },
                started,
                if ok { 0 } else { 1 },
            );
            if ok {
                ExitCode::from(diagnostics::EXIT_OK)
            } else {
                ExitCode::from(diagnostics::EXIT_DIAGNOSTICS)
            }
        }
        Err(e) => {
            let diag = diagnostics::from_parse_error(&e);
            println!("{}", diagnostics::to_json(std::slice::from_ref(&diag)));
            record(
                "check",
                &file_label,
                &src,
                "parse_err",
                Some("parse".to_string()),
                started,
                1,
            );
            ExitCode::from(diagnostics::EXIT_DIAGNOSTICS)
        }
    }
}
