//! Per-machine HMAC key for the compiler-as-agent cache.
//! v3.3 Security Layer 1 — underpins CacheHMAC (#5) + ProvenanceStrategy (#6).
//!
//! ## Why a machine key?
//!
//! The compiler-as-agent cache (`.garnet-cache/`) contains two files that
//! directly influence compilation outcomes:
//!
//! - `episodes.log` — records every past invocation. The strategy miner
//!   reads this to synthesise heuristics.
//! - `strategies.db` — SQLite table of learned rules. The CLI consults
//!   these before each compile; a rule like `skip_check_if_unchanged`
//!   genuinely turns OFF the safety checker.
//!
//! If an attacker can write either file, they can suppress checks on
//! their own code. The v3.2 design trusted whatever rows already existed.
//! This module closes two specific threats from the pen-test model:
//!
//! 1. **Local `.garnet-cache/` poisoning** — shared dir, CI tmp, Nix
//!    sandbox co-tenant: attacker pre-seeds rows.
//! 2. **Committed-cache supply chain** — team commits `.garnet-cache/`
//!    for CI warmup; one developer's strategies silently become ambient
//!    law across the team, and a malicious dependency's install script
//!    pre-poisons them.
//!
//! ## The design
//!
//! Every machine gets a unique 32-byte random key at `~/.garnet/machine.key`
//! (generated on first cache operation). Every episode and every strategy
//! row carries a BLAKE3-keyed MAC over its canonical content. A row whose
//! MAC doesn't verify against *this machine's* key is treated as foreign
//! and ignored — not exploited.
//!
//! Cross-machine caches fail open (ignored), not closed (errors). This
//! avoids breaking developer workflows when someone legitimately checks
//! out a repo with a committed cache — they just don't benefit from the
//! learned strategies until this machine re-derives them.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const KEY_LEN: usize = 32;
const KEY_PATH_ENV: &str = "GARNET_MACHINE_KEY_PATH";
const KEY_PATH_RELATIVE: &str = ".garnet/machine.key";

/// Resolve the on-disk location of the machine key. Respects the
/// `GARNET_MACHINE_KEY_PATH` env var (for tests and containerised
/// deployments), otherwise falls back to `~/.garnet/machine.key`
/// (Windows: `%USERPROFILE%\.garnet\machine.key`).
pub fn default_key_path() -> PathBuf {
    if let Ok(p) = std::env::var(KEY_PATH_ENV) {
        return PathBuf::from(p);
    }
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(KEY_PATH_RELATIVE)
}

/// Load the machine key from `path`, or generate a fresh one if the
/// file does not exist. Returns a copy of the 32-byte key.
///
/// Errors only on genuine I/O failure (disk full, permission denied,
/// etc.) — a missing file triggers generation.
pub fn load_or_generate_key(path: &Path) -> io::Result<[u8; KEY_LEN]> {
    if path.exists() {
        let bytes = fs::read(path)?;
        if bytes.len() != KEY_LEN {
            // Corrupt key file — regenerate rather than panic. Log to
            // stderr so the operator knows why their strategies went
            // foreign overnight.
            eprintln!(
                "garnet: machine key at {:?} has wrong length ({} != {KEY_LEN}), regenerating",
                path,
                bytes.len()
            );
            return generate_and_save(path);
        }
        let mut key = [0u8; KEY_LEN];
        key.copy_from_slice(&bytes);
        Ok(key)
    } else {
        generate_and_save(path)
    }
}

fn generate_and_save(path: &Path) -> io::Result<[u8; KEY_LEN]> {
    let mut key = [0u8; KEY_LEN];
    getrandom::getrandom(&mut key).map_err(|e| io::Error::other(format!("getrandom: {e}")))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, key)?;
    // Best-effort restrictive permissions on Unix. Windows inherits
    // the parent directory ACL which for `~/.garnet/` under the user
    // profile is already user-restricted.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        let _ = fs::set_permissions(path, perms);
    }
    Ok(key)
}

/// Process-wide cached machine key. First access loads-or-generates
/// from disk; subsequent calls are lock-free reads.
///
/// Panics only if the disk operation fails — which is a deliberate
/// fail-hard choice: if we can't establish a machine key, we cannot
/// honour the cache integrity contract and running with an uninitialised
/// key would silently fail-open.
pub fn machine_key() -> &'static [u8; KEY_LEN] {
    static KEY: OnceLock<[u8; KEY_LEN]> = OnceLock::new();
    KEY.get_or_init(|| {
        let path = default_key_path();
        load_or_generate_key(&path)
            .expect("unable to load or generate machine key — cache integrity cannot be guaranteed")
    })
}

/// Compute an HMAC-equivalent keyed MAC over `data` using this
/// machine's key. Uses BLAKE3 keyed mode (which is cryptographically
/// equivalent to HMAC-BLAKE3 and faster than HMAC-SHA256).
pub fn mac(data: &[u8]) -> [u8; 32] {
    mac_with_key(machine_key(), data)
}

/// Same, but with an explicit key — used by tests to avoid touching
/// the real machine key.
pub fn mac_with_key(key: &[u8; KEY_LEN], data: &[u8]) -> [u8; 32] {
    blake3::keyed_hash(key, data).into()
}

/// Hex representation (64 chars) of a MAC byte array. Round-trip stable.
pub fn mac_to_hex(mac: &[u8; 32]) -> String {
    let mut out = String::with_capacity(64);
    for b in mac {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

/// Decode a 64-char hex string back into a 32-byte MAC. Returns `None`
/// on malformed input — callers treat that as a verification failure.
pub fn mac_from_hex(s: &str) -> Option<[u8; 32]> {
    if s.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        let pair = s.get(i * 2..i * 2 + 2)?;
        *byte = u8::from_str_radix(pair, 16).ok()?;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mac_is_deterministic_for_fixed_key_and_data() {
        let key = [0xAA; 32];
        let data = b"hello garnet";
        assert_eq!(mac_with_key(&key, data), mac_with_key(&key, data));
    }

    #[test]
    fn mac_differs_for_different_keys() {
        let k1 = [0xAA; 32];
        let k2 = [0x55; 32];
        assert_ne!(mac_with_key(&k1, b"x"), mac_with_key(&k2, b"x"));
    }

    #[test]
    fn mac_differs_for_different_data() {
        let k = [0xAA; 32];
        assert_ne!(mac_with_key(&k, b"a"), mac_with_key(&k, b"b"));
    }

    #[test]
    fn hex_roundtrip_is_identity() {
        let mac = [0x12, 0xab, 0xcd, 0xef].iter().cycle().take(32).copied().collect::<Vec<_>>();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&mac);
        let hex = mac_to_hex(&arr);
        assert_eq!(hex.len(), 64);
        let back = mac_from_hex(&hex).unwrap();
        assert_eq!(arr, back);
    }

    #[test]
    fn hex_rejects_malformed() {
        assert_eq!(mac_from_hex(""), None);
        assert_eq!(mac_from_hex("zz"), None);
        assert_eq!(mac_from_hex(&"0".repeat(63)), None);
        assert_eq!(mac_from_hex(&"0".repeat(65)), None);
        assert_eq!(mac_from_hex(&"gg".repeat(32)), None);
    }

    #[test]
    fn load_or_generate_creates_key_on_first_call() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("machine.key");
        assert!(!path.exists());
        let k1 = load_or_generate_key(&path).unwrap();
        assert!(path.exists());
        // Second call returns same key.
        let k2 = load_or_generate_key(&path).unwrap();
        assert_eq!(k1, k2);
    }

    #[test]
    fn load_or_generate_regenerates_on_wrong_length() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("machine.key");
        fs::write(&path, b"too short").unwrap();
        let k = load_or_generate_key(&path).unwrap();
        // Now the file is 32 bytes.
        let bytes = fs::read(&path).unwrap();
        assert_eq!(bytes.len(), 32);
        assert_eq!(bytes, k);
    }

    #[test]
    fn two_generated_keys_are_different() {
        let dir1 = tempfile::tempdir().unwrap();
        let dir2 = tempfile::tempdir().unwrap();
        let k1 = load_or_generate_key(&dir1.path().join("key")).unwrap();
        let k2 = load_or_generate_key(&dir2.path().join("key")).unwrap();
        assert_ne!(k1, k2, "fresh keys must be distinct");
    }
}
