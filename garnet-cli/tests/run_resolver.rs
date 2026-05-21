//! S12 — Integration test for `garnet run` resolving vendored deps.
//!
//! Closes deferred line #1 of S3 ("interpreter does NOT yet load
//! `.garnet/vendor/` deps at `garnet run` time"). The test builds a temp
//! project with the same on-disk layout `garnet add ../local-lib`
//! produces, then runs `garnet run src/main.garnet` as a subprocess and
//! asserts the vendored lib's symbol resolved at run time.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("garnet-cli has a workspace parent")
        .to_path_buf()
}

fn garnet_binary() -> PathBuf {
    let target = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace_root().join("target"));
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    target.join(profile).join("garnet")
}

fn ensure_binary_built() {
    let binary = garnet_binary();
    if binary.exists() {
        return;
    }
    let status = Command::new("cargo")
        .args(["build", "-p", "garnet-cli"])
        .current_dir(workspace_root())
        .status()
        .expect("cargo build invocation");
    assert!(status.success(), "cargo build -p garnet-cli failed");
}

/// Build the minimum on-disk shape `garnet add ../local-lib` would
/// produce, plus a `src/main.garnet` that consumes the lib.
fn build_resolver_fixture(tmp: &TempDir) {
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).expect("create src/");
    fs::create_dir_all(root.join(".garnet/vendor/local_lib")).expect("create vendor/");

    fs::write(
        root.join("Garnet.toml"),
        "[package]\nname = \"resolver-demo\"\nversion = \"0.1.0\"\n\n\
         [dependencies]\nlocal_lib = { path = \"../local-lib\", vendor = \".garnet/vendor/local_lib\" }\n",
    )
    .expect("write Garnet.toml");

    fs::write(
        root.join("Garnet.lock"),
        "# Garnet lockfile v0.1\n[[dependency]]\nname = \"local_lib\"\n\
         path = \"../local-lib\"\nvendor = \".garnet/vendor/local_lib\"\n\
         [[dependency.file]]\npath = \"lib.garnet\"\nhash = \"deadbeef\"\n",
    )
    .expect("write Garnet.lock");

    fs::write(
        root.join(".garnet/vendor/local_lib/lib.garnet"),
        "def vendored_hello() { \"hi from local-lib\" }\n",
    )
    .expect("write vendor lib.garnet");

    fs::write(
        root.join("src/main.garnet"),
        "use local_lib::*\n\
         def main() { vendored_hello() }\n",
    )
    .expect("write src/main.garnet");
}

#[test]
fn garnet_run_resolves_vendored_dep_symbol() {
    ensure_binary_built();
    let tmp = TempDir::new().expect("temp dir");
    build_resolver_fixture(&tmp);

    let main_path = tmp.path().join("src/main.garnet");
    let output = Command::new(garnet_binary())
        .arg("run")
        .arg("--interp")
        .arg(&main_path)
        .current_dir(tmp.path())
        .output()
        .expect("running garnet run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "garnet run exited non-zero\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("hi from local-lib"),
        "S12 vendored symbol did not resolve at run time.\nstdout: {stdout}\nstderr: {stderr}",
    );
}

#[test]
fn garnet_run_without_garnet_toml_is_unchanged() {
    ensure_binary_built();
    // A bare .garnet file outside any project must still run, so users can
    // do `garnet run /tmp/scratch.garnet`. This guards against the pre-load
    // step accidentally requiring a Garnet.toml.
    let tmp = TempDir::new().expect("temp dir");
    let scratch = tmp.path().join("scratch.garnet");
    fs::write(&scratch, "def main() { \"scratch ok\" }\n").expect("write scratch.garnet");

    let output = Command::new(garnet_binary())
        .arg("run")
        .arg("--interp")
        .arg(&scratch)
        .output()
        .expect("running garnet run on bare file");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "garnet run on bare file exited non-zero\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("scratch ok"),
        "S12 must not break bare-file runs\nstdout: {stdout}\nstderr: {stderr}"
    );
}
