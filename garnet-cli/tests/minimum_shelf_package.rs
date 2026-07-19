//! Lane 2B fail-closed package boundary and sealed flagship traps.

use garnet_cli::minimum_shelf::{MinimumShelfPackage, TIER1_TOOL_NAME};
use std::fs;
use std::path::{Path, PathBuf};

fn flagship() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/minimum-shelf-flagship")
}

fn copy_flagship() -> tempfile::TempDir {
    let temp = tempfile::tempdir().expect("temp package root");
    for name in ["SHELF_PACKAGE.json", "tool.garnet", "tool.seal.json"] {
        fs::copy(flagship().join(name), temp.path().join(name)).expect("copy flagship fixture");
    }
    temp
}

fn assert_rejected(path: &Path, expected: &str) {
    let error = MinimumShelfPackage::load(path).expect_err("package must be rejected");
    assert!(error.to_string().contains(expected), "{error}");
}

#[test]
fn sealed_flagship_loads_end_to_end() {
    let package = MinimumShelfPackage::load(&flagship()).expect("flagship seal must verify");
    assert_eq!(package.tool_name(), TIER1_TOOL_NAME);
    assert_eq!(package.ring_tier(), 1);
}

#[test]
fn rejects_when_seal_is_missing() {
    let temp = copy_flagship();
    fs::remove_file(temp.path().join("tool.seal.json")).expect("remove seal");
    assert_rejected(temp.path(), "seal");
}

#[test]
fn rejects_when_seal_is_empty() {
    let temp = copy_flagship();
    fs::write(temp.path().join("tool.seal.json"), b"{}\n").expect("strip seal");
    assert_rejected(temp.path(), "seal");
}

#[test]
fn rejects_when_source_bytes_change() {
    let temp = copy_flagship();
    let path = temp.path().join("tool.garnet");
    let source = fs::read_to_string(&path).expect("source");
    fs::write(&path, source.replace("value * 2", "value * 3")).expect("tamper source");
    assert_rejected(temp.path(), "source");
}

#[test]
fn rejects_when_seal_bytes_change() {
    let temp = copy_flagship();
    let path = temp.path().join("tool.seal.json");
    let mut seal = fs::read(&path).expect("seal");
    seal.push(b'\n');
    fs::write(&path, seal).expect("tamper seal");
    assert_rejected(temp.path(), "seal");
}

#[test]
fn rejects_when_manifest_is_rebound_to_tampered_source() {
    let temp = copy_flagship();
    let source_path = temp.path().join("tool.garnet");
    let source = fs::read_to_string(&source_path).expect("source");
    let tampered = source.replace("value * 2", "value * 3");
    fs::write(&source_path, &tampered).expect("tamper source");

    let manifest_path = temp.path().join("SHELF_PACKAGE.json");
    let manifest = fs::read_to_string(&manifest_path).expect("manifest");
    let original_hash = blake3::hash(source.as_bytes()).to_hex().to_string();
    let rebound_hash = blake3::hash(tampered.as_bytes()).to_hex().to_string();
    fs::write(&manifest_path, manifest.replace(&original_hash, &rebound_hash))
        .expect("rebind manifest");

    assert_rejected(temp.path(), "trusted flagship");
}
