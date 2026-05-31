//! `garnet verify <path>` — the S33 acceptance gate.
//!
//! Runs edition-aware parse + safe-mode check over the target(s), emits a fused
//! merge-confidence band, and exits non-zero iff any target fails fatally.
//! (Distinct from `garnet verify <file> <manifest.json>`, the 2-arg
//! deterministic-manifest verify in `cmd/verify.rs`; the dispatcher routes on
//! positional-arg count.)

use crate::verify_gate::{fuse, Band, CapabilitySignal, GateTally};
use crate::{edition_manifest, read_file};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Parsed arguments for the acceptance gate.
pub struct GateArgs {
    pub path: PathBuf,
    /// Optional external-reviewer band (1..=5); in CI/PR this is Greptile.
    pub external_band: Option<u8>,
}

pub fn run(args: GateArgs) -> ExitCode {
    let tally = match gate_tally(&args.path) {
        Ok(tally) => tally,
        Err(message) => {
            eprintln!("garnet verify: {message}");
            return ExitCode::from(2);
        }
    };

    let internal = tally.internal_band();
    let external = args.external_band.map(Band::new);
    let capability = CapabilitySignal::pending_until_s37();
    let fused = fuse(internal, external, capability);

    println!();
    println!(
        "Verified {} target(s): {} failing, {} advisory diagnostic(s).",
        tally.targets, tally.failing, tally.advisories
    );
    println!("Merge confidence (fused): {}/5", fused.get());
    println!(
        "  internal (local parse + safe-mode check): {}/5",
        internal.get()
    );
    match external {
        Some(b) => println!("  external reviewer: {}/5", b.get()),
        None => println!("  external reviewer: not supplied (Greptile wires in at PR time)"),
    }
    println!("  capability signal: stub (pending S37 diff-caps)");
    println!("  fusion rule: min of the present signals");

    if tally.passes() {
        println!("\ngate: PASS");
        ExitCode::SUCCESS
    } else {
        println!(
            "\ngate: FAIL ({} target(s) with fatal diagnostics)",
            tally.failing
        );
        ExitCode::from(1)
    }
}

/// Parse + check a single target, folding the outcome into `tally`.
fn verify_one(target: &Path, tally: &mut GateTally) {
    let src = match read_file(target) {
        Ok(s) => s,
        Err(e) => {
            println!("  ✗ {} : read error: {e}", target.display());
            tally.failing += 1;
            return;
        }
    };
    let edition = match edition_manifest::resolve_edition_for(target) {
        Ok(resolved) => {
            if let Some(warning) = resolved.warning {
                eprintln!("{warning}");
            }
            resolved.edition
        }
        Err(message) => {
            println!("  ✗ {} : {message}", target.display());
            tally.failing += 1;
            return;
        }
    };
    match garnet_parser::parse_source_with_edition(&src, edition) {
        Ok(module) => {
            let report = garnet_check::check_module(&module);
            if report.ok() {
                let advisories = report.errors.len();
                if advisories > 0 {
                    tally.advisories += advisories;
                    println!(
                        "  ~ {} : ok ({advisories} advisor{})",
                        target.display(),
                        if advisories == 1 { "y" } else { "ies" }
                    );
                } else {
                    println!("  ✓ {} : clean", target.display());
                }
            } else {
                tally.failing += 1;
                println!(
                    "  ✗ {} : {} diagnostic(s)",
                    target.display(),
                    report.errors.len()
                );
                for err in &report.errors {
                    println!("      {err}");
                }
            }
        }
        Err(e) => {
            tally.failing += 1;
            println!("  ✗ {} : parse error: {e}", target.display());
        }
    }
}

/// Collect the targets under `path`, run edition-aware parse + safe-mode check
/// over each (printing a per-target line), and return the aggregate tally.
/// `Err` signals a *usage* problem (unreadable path / no `.garnet` files) — not
/// a gate failure. Exposed for integration tests so the accept/reject verdict is
/// asserted without scraping stdout or process exit codes.
pub fn gate_tally(path: &Path) -> Result<GateTally, String> {
    let targets = collect_targets(path).map_err(|e| e.to_string())?;
    if targets.is_empty() {
        return Err(format!("no .garnet files found under {}", path.display()));
    }
    println!(
        "garnet verify: acceptance gate over {} target(s)",
        targets.len()
    );
    let mut tally = GateTally::default();
    for target in &targets {
        tally.targets += 1;
        verify_one(target, &mut tally);
    }
    Ok(tally)
}

/// Resolve the target list: a single `.garnet` file, or every `.garnet` file
/// under a directory (skipping build/vendor dirs). Returned sorted for
/// deterministic output.
fn collect_targets(path: &Path) -> std::io::Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    let mut out = Vec::new();
    walk(path, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            // Skip build output, VCS, and vendored dependency trees.
            if matches!(
                name.as_ref(),
                "target" | ".git" | "node_modules" | "vendor" | ".garnet-cache"
            ) {
                continue;
            }
            walk(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "garnet") {
            out.push(path);
        }
    }
    Ok(())
}
