//! S13 — Integration test for `garnet add --registry`.
//!
//! Builds a temp filesystem registry (package dir + generated `index.json`),
//! a temp consuming project, then runs `garnet add --registry <dir>
//! hello_lib@0.1.0` and asserts the package is vendored + recorded. Because
//! S12 is merged, it also runs `garnet run` and asserts the registry-vendored
//! symbol resolves at run time — the full registry → vendor → resolve loop.

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
    if garnet_binary().exists() {
        return;
    }
    let status = Command::new("cargo")
        .args(["build", "-p", "garnet-cli"])
        .current_dir(workspace_root())
        .status()
        .expect("cargo build invocation");
    assert!(status.success(), "cargo build -p garnet-cli failed");
}

/// Build a filesystem registry with one package version + its index.json.
fn build_registry(tmp: &TempDir) -> PathBuf {
    let registry = tmp.path().join("registry");
    let pkg = registry.join("hello_lib/0.1.0");
    fs::create_dir_all(&pkg).expect("create package dir");
    fs::write(
        pkg.join("lib.garnet"),
        "def registry_hello() { \"hi from the registry stub\" }\n",
    )
    .expect("write package source");
    let index = garnet_registry_stub::build_index(&registry).expect("build index");
    garnet_registry_stub::write_index(&registry, &index).expect("write index");
    registry
}

/// Build a consuming project that `use`s the package.
fn build_project(tmp: &TempDir) -> PathBuf {
    let project = tmp.path().join("project");
    fs::create_dir_all(project.join("src")).expect("create project");
    fs::write(
        project.join("Garnet.toml"),
        "[package]\nname = \"consumer\"\nversion = \"0.1.0\"\n\n[dependencies]\n",
    )
    .expect("write Garnet.toml");
    fs::write(
        project.join("src/main.garnet"),
        "use hello_lib::*\ndef main() { registry_hello() }\n",
    )
    .expect("write main.garnet");
    project
}

#[test]
fn garnet_add_registry_vendors_and_records() {
    ensure_binary_built();
    let tmp = TempDir::new().expect("temp dir");
    let registry = build_registry(&tmp);
    let project = build_project(&tmp);

    let output = Command::new(garnet_binary())
        .arg("add")
        .arg("--registry")
        .arg(&registry)
        .arg("hello_lib@0.1.0")
        .current_dir(&project)
        .output()
        .expect("running garnet add --registry");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "garnet add --registry failed\nstdout: {stdout}\nstderr: {stderr}"
    );

    // Vendored.
    assert!(
        project.join(".garnet/vendor/hello_lib/lib.garnet").exists(),
        "package was not vendored"
    );
    // Manifest records the registry shape.
    let manifest = fs::read_to_string(project.join("Garnet.toml")).unwrap();
    assert!(
        manifest.contains("hello_lib = { registry =") && manifest.contains("version = \"0.1.0\""),
        "Garnet.toml missing registry entry:\n{manifest}"
    );
    // Lockfile records the dep.
    let lock = fs::read_to_string(project.join("Garnet.lock")).unwrap();
    assert!(
        lock.contains("hello_lib"),
        "Garnet.lock missing dep:\n{lock}"
    );
}

#[test]
fn garnet_run_resolves_registry_vendored_symbol() {
    ensure_binary_built();
    let tmp = TempDir::new().expect("temp dir");
    let registry = build_registry(&tmp);
    let project = build_project(&tmp);

    let add = Command::new(garnet_binary())
        .arg("add")
        .arg("--registry")
        .arg(&registry)
        .arg("hello_lib@0.1.0")
        .current_dir(&project)
        .output()
        .expect("garnet add");
    assert!(add.status.success());

    // S12 resolver loads the registry-vendored symbol at run time.
    let run = Command::new(garnet_binary())
        .arg("run")
        .arg("--interp")
        .arg(project.join("src/main.garnet"))
        .current_dir(&project)
        .output()
        .expect("garnet run");
    let stdout = String::from_utf8_lossy(&run.stdout);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        run.status.success(),
        "garnet run failed\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("hi from the registry stub"),
        "registry-vendored symbol did not resolve\nstdout: {stdout}\nstderr: {stderr}"
    );
}

#[test]
fn garnet_add_registry_rejects_missing_version() {
    ensure_binary_built();
    let tmp = TempDir::new().expect("temp dir");
    let registry = build_registry(&tmp);
    let project = build_project(&tmp);

    let output = Command::new(garnet_binary())
        .arg("add")
        .arg("--registry")
        .arg(&registry)
        .arg("hello_lib@9.9.9")
        .current_dir(&project)
        .output()
        .expect("running garnet add --registry");
    assert!(
        !output.status.success(),
        "garnet add for a missing version must fail"
    );
    assert!(
        !project.join(".garnet/vendor/hello_lib").exists(),
        "nothing should be vendored on a failed resolve"
    );
}

#[test]
fn garnet_add_registry_warns_on_slopsquatting_near_miss() {
    // S45: an unknown name that closely resembles a known one (here a single
    // adjacent transposition of `hello_lib`) must still fail to resolve, but the
    // error is enriched with a slopsquatting near-miss hint.
    ensure_binary_built();
    let tmp = TempDir::new().expect("temp dir");
    let registry = build_registry(&tmp); // contains hello_lib@0.1.0
    let project = build_project(&tmp);

    let output = Command::new(garnet_binary())
        .arg("add")
        .arg("--registry")
        .arg(&registry)
        .arg("hello_lbi@0.1.0") // transposition of hello_lib
        .current_dir(&project)
        .output()
        .expect("running garnet add --registry");
    assert!(
        !output.status.success(),
        "an unknown package must fail to resolve"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("did you mean") && stderr.contains("hello_lib"),
        "expected a near-miss hint naming hello_lib, got:\n{stderr}"
    );
    assert!(
        stderr.contains("slopsquatting"),
        "expected the slopsquatting label:\n{stderr}"
    );
    assert!(
        !project.join(".garnet/vendor").exists(),
        "nothing should be vendored on a failed resolve"
    );
}

#[test]
fn garnet_add_registry_missing_version_has_no_slop_warning() {
    // A version miss on a *known* name is not slopsquatting — the guard must
    // stay quiet so the signal isn't diluted.
    ensure_binary_built();
    let tmp = TempDir::new().expect("temp dir");
    let registry = build_registry(&tmp);
    let project = build_project(&tmp);

    let output = Command::new(garnet_binary())
        .arg("add")
        .arg("--registry")
        .arg(&registry)
        .arg("hello_lib@9.9.9") // name known, version absent
        .current_dir(&project)
        .output()
        .expect("running garnet add --registry");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("slopsquatting"),
        "a version miss on a known name must not trigger the slop guard:\n{stderr}"
    );
}
