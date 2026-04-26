//! `garnet keygen <keyfile>` — create an Ed25519 signing keypair; write the
//! hex-encoded 32-byte signing key to `<keyfile>`, print the corresponding
//! hex-encoded public key to stdout. The caller is responsible for
//! protecting the keyfile (e.g., `chmod 0600` on UNIX).

use crate::manifest;
use std::path::PathBuf;
use std::process::ExitCode;

pub fn run(keyfile: PathBuf) -> ExitCode {
    let (signing_key, pubkey_hex) = manifest::generate_signing_key();
    let key_hex = manifest::signing_key_to_hex(&signing_key);
    // Write with a trailing newline — POSIX-friendly.
    let body = format!("{key_hex}\n");
    if let Err(e) = std::fs::write(&keyfile, body) {
        eprintln!("failed to write keyfile {}: {e}", keyfile.display());
        return ExitCode::from(1);
    }
    // Best-effort UNIX permission tightening. On Windows, this is a no-op —
    // caller should use an ACL or keep the file in a protected directory.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        if let Err(e) = std::fs::set_permissions(&keyfile, perms) {
            eprintln!("warning: could not chmod 0600 {}: {e}", keyfile.display());
        }
    }
    println!("generated Ed25519 signing keypair");
    println!(
        "  keyfile = {} (keep private; chmod 0600)",
        keyfile.display()
    );
    println!("  pubkey  = {pubkey_hex}");
    ExitCode::SUCCESS
}
