//! `garnet keygen <keyfile>` — create an Ed25519 signing keypair; write the
//! hex-encoded 32-byte signing key to `<keyfile>`, print the corresponding
//! hex-encoded public key to stdout. On Unix the key only ever exists in a
//! file with mode 0600.

use crate::manifest;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub fn run(keyfile: PathBuf) -> ExitCode {
    let (signing_key, pubkey_hex) = manifest::generate_signing_key();
    let key_hex = manifest::signing_key_to_hex(&signing_key);
    // Write with a trailing newline — POSIX-friendly.
    let body = format!("{key_hex}\n");
    if let Err(e) = write_private(&keyfile, &body) {
        eprintln!("failed to write keyfile {}: {e}", keyfile.display());
        return ExitCode::from(1);
    }
    println!("generated Ed25519 signing keypair");
    println!(
        "  keyfile = {} (keep private; chmod 0600)",
        keyfile.display()
    );
    println!("  pubkey  = {pubkey_hex}");
    ExitCode::SUCCESS
}

/// Unix: write `body` to a new mode-0600 file beside `keyfile`, then rename it
/// over `keyfile`. The key never enters an existing file, so a keyfile that
/// was world-readable, or held open by another process, never contains it.
#[cfg(unix)]
fn write_private(keyfile: &Path, body: &str) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let name = keyfile.file_name().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "keyfile has no file name")
    })?;
    let dir = match keyfile.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    let tmp = dir.join(format!(
        ".{}.garnet-keygen-{}",
        name.to_string_lossy(),
        std::process::id()
    ));
    let result = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&tmp)
        .and_then(|mut f| {
            f.write_all(body.as_bytes())?;
            f.sync_all()
        })
        .and_then(|()| std::fs::rename(&tmp, keyfile));
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// Elsewhere: a plain write. Protect the key with an ACL or keep it in a
/// protected directory.
#[cfg(not(unix))]
fn write_private(keyfile: &Path, body: &str) -> std::io::Result<()> {
    std::fs::write(keyfile, body)
}
