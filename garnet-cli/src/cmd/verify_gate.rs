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
    /// Optional baseline path (an older revision of the same tree) for the S37
    /// capability signal: `diff-caps(baseline, current)` feeds the fuse.
    pub caps_baseline: Option<PathBuf>,
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
    let capability = resolve_capability_signal(&args);
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
    match capability {
        CapabilitySignal::Surface(b) => {
            println!("  capability signal (diff-caps vs baseline): {}/5", b.get())
        }
        CapabilitySignal::Pending => {
            println!("  capability signal: pending (pass --caps-baseline <old> for diff-caps)")
        }
    }
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

/// Compute the S37 capability signal. With a `--caps-baseline`, run
/// `diff-caps(baseline, current)` and map an authority change to a band;
/// otherwise the slot stays pending (back-compat with S33).
fn resolve_capability_signal(args: &GateArgs) -> CapabilitySignal {
    let Some(baseline) = &args.caps_baseline else {
        return CapabilitySignal::Pending;
    };
    match (
        crate::cap_manifest::surface_for_path(baseline),
        crate::cap_manifest::surface_for_path(&args.path),
    ) {
        (Ok(base), Ok(current)) => {
            let diff = garnet_check::diff_caps(&base, &current);
            CapabilitySignal::Surface(capability_band(&diff))
        }
        _ => {
            eprintln!(
                "garnet verify: could not build capability surfaces for --caps-baseline; \
                 capability signal left pending"
            );
            CapabilitySignal::Pending
        }
    }
}

/// Map a capability diff to the capability signal band: `5` when the program did
/// not gain authority, `2` when it did (so the fused `min` flags the change for
/// review).
pub fn capability_band(diff: &garnet_check::CapsDiff) -> Band {
    if diff.authority_expanded() {
        Band::new(2)
    } else {
        Band::new(5)
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
/// deterministic output. `pub(crate)` so `garnet caps` (S36) reuses the walk.
pub(crate) fn collect_targets(path: &Path) -> std::io::Result<Vec<PathBuf>> {
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
        } else if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("garnet"))
        {
            // Case-insensitive so Windows' case-insensitive filesystem does not
            // silently skip an uppercase `.GARNET` file (WIN-S33/36/37/46). This
            // shared collector is reused by `cap_manifest::surface_for_path`, so
            // the one fix covers verify / caps / diff-caps / sandbox-policy walks.
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::collect_targets;
    use std::fs;
    use tempfile::TempDir;

    /// The shared collector must discover an uppercase `.GARNET` file. macOS
    /// preserves filename case, so this reproduces the Windows-only *skip*
    /// (WIN-S33/36/37/46) on Mac: pre-fix the case-sensitive `== "garnet"`
    /// dropped `BAD.GARNET`; post-fix the case-insensitive compare keeps it.
    #[test]
    fn collect_targets_discovers_uppercase_garnet_extension() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("main.garnet"),
            "@caps()\ndef main() { 1 }\n",
        )
        .unwrap();
        fs::write(tmp.path().join("BAD.GARNET"), "def main( { 1 }\n").unwrap();

        let found = collect_targets(tmp.path()).unwrap();
        let names: Vec<String> = found
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(
            names.iter().any(|n| n == "BAD.GARNET"),
            "uppercase .GARNET must be discovered, got {names:?}"
        );
        assert!(names.iter().any(|n| n == "main.garnet"), "got {names:?}");
        assert_eq!(found.len(), 2, "both targets discovered, got {names:?}");
    }

    /// A single uppercase `.GARNET` *file* target also resolves (the file-path
    /// branch is already case-agnostic; this pins it against regressions).
    #[test]
    fn collect_targets_accepts_single_uppercase_file() {
        let tmp = TempDir::new().unwrap();
        let f = tmp.path().join("LIB.GARNET");
        fs::write(&f, "def f() { 1 }\n").unwrap();
        assert_eq!(collect_targets(&f).unwrap(), vec![f]);
    }
}
