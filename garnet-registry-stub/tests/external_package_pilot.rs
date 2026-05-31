//! S77 — external package pilot.
//!
//! Drives the external-package flow end-to-end against the filesystem registry
//! stub: an external package is published into a registry, resolved by name +
//! version, content-address-verified (BLAKE3), a *nonexistent* dependency is
//! refused, and the slopsquatting guard flags a hallucinated near-miss of the
//! package's name. Runs in the `cargo test --workspace` matrix on every OS.
//!
//! Honest scope: a LOCAL filesystem registry-stub pilot, NOT a live public
//! ecosystem. No HTTP, no publish/auth, no SemVer ranges, no signatures. The
//! slopguard is a deterministic heuristic ("prompt to verify"), not a security
//! guarantee — "known names" are the local index, not a global feed.

use garnet_registry_stub::slopguard::{nearest, SuspicionKind};
use garnet_registry_stub::{build_index, package_dir, resolve, verify_package, RegistryError};
use std::fs;
use tempfile::TempDir;

/// Publish a one-file external package `<name>/<version>/lib.garnet` into a
/// fresh filesystem registry and return the temp registry root.
fn registry_with_package(name: &str, version: &str, body: &str) -> TempDir {
    let tmp = TempDir::new().unwrap();
    let pkg = tmp.path().join(name).join(version);
    fs::create_dir_all(&pkg).unwrap();
    fs::write(pkg.join("lib.garnet"), body).unwrap();
    tmp
}

#[test]
fn external_package_resolves_and_verifies() {
    let reg = registry_with_package(
        "acme-logger",
        "1.0.0",
        "@caps()\ndef log(line) {\n  line\n}\n",
    );
    let index = build_index(reg.path()).unwrap();

    // The external package is in the index and resolves by name + version.
    assert!(index.known_names().contains(&"acme-logger"));
    let entry = resolve(&index, "acme-logger", "1.0.0").unwrap();

    // Its on-disk tree content-address-verifies (BLAKE3) against the index.
    let dir = package_dir(reg.path(), &entry).unwrap();
    verify_package(&dir, &entry).expect("freshly built package must verify");
}

#[test]
fn tampered_external_package_fails_verification() {
    let reg = registry_with_package("acme-logger", "1.0.0", "@caps()\ndef log(x) {\n  x\n}\n");
    let index = build_index(reg.path()).unwrap();
    let entry = resolve(&index, "acme-logger", "1.0.0").unwrap();
    let dir = package_dir(reg.path(), &entry).unwrap();

    // Tamper with the vendored bytes after indexing → integrity check fails.
    fs::write(
        dir.join("lib.garnet"),
        "@caps()\ndef log(x) {\n  evil()\n}\n",
    )
    .unwrap();
    assert!(matches!(
        verify_package(&dir, &entry),
        Err(RegistryError::Integrity(_))
    ));
}

#[test]
fn nonexistent_dependency_is_refused() {
    let reg = registry_with_package("acme-logger", "1.0.0", "@caps()\ndef log(x) {\n  x\n}\n");
    let index = build_index(reg.path()).unwrap();
    // A hallucinated dependency simply is not in the registry → NotFound.
    assert!(matches!(
        resolve(&index, "acme-loggr", "1.0.0"),
        Err(RegistryError::NotFound(_))
    ));
}

#[test]
fn slopguard_flags_hallucinated_near_miss() {
    let reg = registry_with_package("acme-logger", "1.0.0", "@caps()\ndef log(x) {\n  x\n}\n");
    let index = build_index(reg.path()).unwrap();
    let known = index.known_names();

    // Separator-confusable near-miss (`-` vs `_`) — a common slopsquatting vector.
    let sep = nearest("acme_logger", known.iter().copied(), 2);
    assert!(
        sep.iter().any(|s| s.candidate == "acme-logger"
            && matches!(s.kind, SuspicionKind::SeparatorConfusable)),
        "expected a separator-confusable suspicion, got {sep:?}"
    );

    // Edit-distance near-miss (one deletion) is also surfaced.
    let edit = nearest("acme-loggr", known.iter().copied(), 2);
    assert!(
        edit.iter().any(|s| s.candidate == "acme-logger"),
        "expected an edit-distance suspicion, got {edit:?}"
    );
}
