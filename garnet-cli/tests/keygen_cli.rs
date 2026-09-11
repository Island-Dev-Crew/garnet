//! `garnet keygen` never writes a signing key to a file named like a flag,
//! and creates the key file readable by its owner only.

use std::process::Command;

fn keygen_in(dir: &std::path::Path, arg: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
        .args(["keygen", arg])
        .current_dir(dir)
        .output()
        .unwrap()
}

#[test]
fn keygen_help_prints_usage_and_writes_no_key() {
    for flag in ["--help", "-h"] {
        let dir = tempfile::TempDir::new().unwrap();
        let out = keygen_in(dir.path(), flag);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(out.status.success(), "{flag}: {out:?}");
        assert!(stdout.contains("usage: garnet keygen"), "{flag}: {stdout}");
        assert!(
            !dir.path().join(flag).exists(),
            "{flag}: a signing key was written to a file named {flag}"
        );
    }
}

#[test]
fn keygen_rejects_other_flags_without_writing() {
    let dir = tempfile::TempDir::new().unwrap();
    let out = keygen_in(dir.path(), "-x");
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert!(!dir.path().join("-x").exists(), "a key was written to -x");
}

#[test]
fn keygen_writes_a_hex_key_and_prints_the_public_key() {
    let dir = tempfile::TempDir::new().unwrap();
    let out = keygen_in(dir.path(), "my.key");
    assert!(out.status.success(), "{out:?}");
    let key = std::fs::read_to_string(dir.path().join("my.key")).unwrap();
    let key = key.trim_end();
    assert_eq!(key.len(), 64, "32-byte key as hex: {key}");
    assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
    assert!(String::from_utf8_lossy(&out.stdout).contains("pubkey"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(dir.path().join("my.key"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600, "key file mode {mode:o}");
    }
}
