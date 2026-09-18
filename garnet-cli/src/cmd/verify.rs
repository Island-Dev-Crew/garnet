//! `garnet verify <file> <artifact.json> [--caps-baseline <old>] [--signature]`.
//! Recheck and bind a seal/v2 to source, or verify a legacy deterministic
//! manifest (including its signature when present or required). A supplied
//! capability baseline is enforced on both artifact formats.

use crate::manifest::Manifest;
use crate::read_file;
use std::path::PathBuf;
use std::process::ExitCode;

pub fn run(file: PathBuf, manifest_path: PathBuf, require_signature: bool) -> ExitCode {
    run_with_baseline(file, manifest_path, require_signature, None)
}

pub fn run_with_baseline(
    file: PathBuf,
    manifest_path: PathBuf,
    require_signature: bool,
    baseline: Option<PathBuf>,
) -> ExitCode {
    let src = match read_file(&file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };
    let edition = match crate::edition_manifest::resolve_edition_for(&file) {
        Ok(resolved) => {
            if let Some(warning) = resolved.warning {
                eprintln!("{warning}");
            }
            resolved.edition
        }
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(1);
        }
    };
    let module = match garnet_parser::parse_source_with_edition(&src, edition) {
        Ok(m) => m,
        Err(e) => {
            let report = miette::Report::new(e).with_source_code(src);
            eprintln!("{report:?}");
            return ExitCode::from(1);
        }
    };
    let on_disk = match read_file(&manifest_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };
    match super::verify_gate::resolve_capability_signal(&file, baseline.as_deref()) {
        Err(error) => {
            eprintln!("FAIL --caps-baseline: {error}");
            return ExitCode::from(2);
        }
        Ok(crate::verify_gate::CapabilitySignal::Surface(band)) if band.get() < 5 => {
            eprintln!("FAIL declared authority expanded against --caps-baseline");
            return ExitCode::from(1);
        }
        _ => {}
    }
    // Format markers select a seal, never an attempted fallback to a manifest.
    if serde_json::from_str::<serde_json::Value>(&on_disk).is_ok_and(|v| {
        v.get("_type").is_some()
            || v.get("predicateType").is_some()
            || v.get("subject").is_some()
            || v.get("predicate").is_some()
    }) {
        if require_signature {
            eprintln!("FAIL seal content verification does not verify signatures; use external cosign verification");
            return ExitCode::from(2);
        }
        let report = garnet_check::check_module(&module);
        for diagnostic in &report.errors {
            eprintln!("{diagnostic}");
        }
        if !report.ok() {
            eprintln!("FAIL seal source failed checker");
            return ExitCode::from(1);
        }
        let build = Manifest::build(&src, &module);
        let caps = crate::cap_manifest::CapabilityManifest::from_surface(
            garnet_check::capability_surface(&module),
        );
        match crate::seal::verify_statement(&on_disk, &build, &caps) {
            Ok(()) => {
                println!("OK {} matches seal/v2 source and declared capabilities (content binding only; no signature verified)", file.display());
                return ExitCode::SUCCESS;
            }
            Err(error) => {
                eprintln!("FAIL {error}");
                return ExitCode::from(2);
            }
        }
    }
    let stored = match Manifest::from_canonical_json(&on_disk) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("manifest parse error: {e}");
            return ExitCode::from(1);
        }
    };
    let recomputed = Manifest::build(&src, &module);

    // Every field the manifest carries is compared, and each mismatch is
    // reported on its own with the field named — an operator must never have
    // to guess which input moved.
    let mismatches = stored.field_mismatches(&recomputed);
    if !mismatches.is_empty() {
        for mismatch in &mismatches {
            eprintln!("FAIL {mismatch}");
        }
        return ExitCode::from(2);
    }

    // v3.4.1 ManifestSig — signature verification.
    //
    // `--signature` flag forces a signed manifest + valid signature. Without
    // the flag, an unsigned manifest passes (backwards compat) but a signed
    // manifest whose signature is invalid is ALWAYS rejected — a tampered
    // signed manifest cannot quietly pass by dropping the flag.
    if stored.is_signed() {
        match stored.verify_signature() {
            Ok(()) => {
                println!("OK {} matches manifest + signature valid", file.display());
                println!("  source_hash = {}", stored.source_hash);
                println!("  ast_hash    = {}", stored.ast_hash);
                println!("  signed_by   = {}", stored.signer_pubkey);
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                eprintln!("FAIL signature verification: {e}");
                return ExitCode::from(2);
            }
        }
    } else if require_signature {
        eprintln!("FAIL manifest is unsigned but --signature was required");
        return ExitCode::from(2);
    }

    println!("OK {} matches manifest (unsigned)", file.display());
    println!("  source_hash = {}", stored.source_hash);
    println!("  ast_hash    = {}", stored.ast_hash);
    ExitCode::SUCCESS
}
