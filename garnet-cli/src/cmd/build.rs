//! `garnet build [--deterministic] [--sign <keyfile>] <file>` — emit a
//! (deterministic, optionally signed) provenance manifest beside the source.

use crate::manifest::{self, Manifest};
use crate::read_file;
use std::path::PathBuf;
use std::process::ExitCode;

pub fn run(path: PathBuf, deterministic: bool, sign_keyfile: Option<String>) -> ExitCode {
    let src = match read_file(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };
    let module = match garnet_parser::parse_source(&src) {
        Ok(m) => m,
        Err(e) => {
            let report = miette::Report::new(e).with_source_code(src);
            eprintln!("{report:?}");
            return ExitCode::from(1);
        }
    };
    if deterministic {
        let mut manifest = Manifest::build(&src, &module);

        // v3.4.1 ManifestSig: if --sign <keyfile> was passed, load the key
        // and sign the manifest in place. The signature covers every field
        // except signer_pubkey + signature themselves.
        if let Some(keyfile) = sign_keyfile.as_deref() {
            let key_hex = match std::fs::read_to_string(keyfile) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("failed to read keyfile {keyfile}: {e}");
                    return ExitCode::from(1);
                }
            };
            let signing_key = match manifest::signing_key_from_hex(&key_hex) {
                Ok(k) => k,
                Err(e) => {
                    eprintln!("bad keyfile {keyfile}: {e}");
                    return ExitCode::from(1);
                }
            };
            let _payload = manifest.sign(&signing_key);
        }

        let json = manifest.to_canonical_json();
        let manifest_path = path.with_extension(
            path.extension()
                .map(|e| format!("{}.manifest.json", e.to_string_lossy()))
                .unwrap_or_else(|| "manifest.json".to_string()),
        );
        if let Err(e) = std::fs::write(&manifest_path, &json) {
            eprintln!("failed to write manifest: {e}");
            return ExitCode::from(1);
        }
        println!("built {} ({} items)", path.display(), module.items.len());
        println!("  source_hash = {}", manifest.source_hash);
        println!("  ast_hash    = {}", manifest.ast_hash);
        println!("  manifest    = {}", manifest_path.display());
        if manifest.is_signed() {
            println!("  signed_by   = {}", manifest.signer_pubkey);
            println!("  signature   = {}", manifest.signature);
        }
    } else {
        println!("built {} ({} items)", path.display(), module.items.len());
        println!("  hint: pass --deterministic to emit a provenance manifest");
    }
    ExitCode::SUCCESS
}
