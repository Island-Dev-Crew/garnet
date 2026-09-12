//! `garnet ceilings <file.garnet>` — explosive-op / default-ceiling report (S40).
//!
//! Reports each function's explosive operations (unconditional `loop`, `spawn`)
//! and whether each is governed by a declared bound (`@bounded` / `@fan_out`) or
//! falls back to the default ceiling. Static identification + a default-ceiling
//! POLICY; runtime enforcement lowers to the S39 `@bounded` / Wasmtime-fuel path
//! and is deferred (wasmtime absent) — no ceiling is faked.

use crate::{edition_manifest, read_file};
use garnet_check::explosive::{DEFAULT_LOOP_CEILING, DEFAULT_SPAWN_FANOUT};
use garnet_check::{explosive_ops, ExplosiveKind};
use std::path::PathBuf;
use std::process::ExitCode;

pub fn run(path: PathBuf) -> ExitCode {
    let src = match read_file(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("garnet ceilings: {e}");
            return ExitCode::from(1);
        }
    };
    let edition = match edition_manifest::resolve_edition_for(&path) {
        Ok(resolved) => {
            if let Some(warning) = resolved.warning {
                eprintln!("{warning}");
            }
            resolved.edition
        }
        Err(message) => {
            eprintln!("garnet ceilings: {message}");
            return ExitCode::from(1);
        }
    };
    let module = match garnet_parser::parse_source_with_edition(&src, edition) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("garnet ceilings: parse error: {e}");
            return ExitCode::from(1);
        }
    };

    let reports = explosive_ops(&module);
    println!("garnet ceilings for {}", path.display());
    if reports.is_empty() {
        println!("  no explosive operations (unconditional loops / spawn) found.");
    } else {
        for r in &reports {
            println!("  fn {}: {} explosive operation(s)", r.fn_name, r.ops.len());
            for op in &r.ops {
                let at = format!("{}..{}", op.span.start, op.span.start + op.span.len);
                match op.kind {
                    ExplosiveKind::UnconditionalLoop if r.has_bounded => {
                        println!(
                            "    - unconditional loop @ {at}: governed by @bounded fuel budget"
                        );
                    }
                    ExplosiveKind::UnconditionalLoop => {
                        println!(
                            "    - unconditional loop @ {at}: no @bounded(N); the default ceiling of {DEFAULT_LOOP_CEILING} iterations is reported policy, not enforced"
                        );
                    }
                    ExplosiveKind::Spawn if r.has_fan_out => {
                        println!("    - spawn @ {at}: governed by @fan_out");
                    }
                    ExplosiveKind::Spawn => {
                        println!(
                            "    - spawn @ {at}: no @fan_out(K); the DEFAULT fan-out ceiling of {DEFAULT_SPAWN_FANOUT} is reported policy, not enforced"
                        );
                    }
                }
            }
        }
    }
    println!(
        "\nnote: STATIC identification + a default-ceiling policy. Nothing enforces these ceilings \
         at run time; the @bounded / Wasmtime-fuel path (S39) is deferred — \
         no ceiling is faked."
    );
    ExitCode::SUCCESS
}
