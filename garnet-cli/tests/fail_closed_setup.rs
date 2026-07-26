//! Lane 1 item 6: setup and discovery failures must never degrade to green.
//!
//! These fixtures deliberately use cross-platform filesystem type/content
//! mismatches instead of permission bits, whose behavior varies by account and
//! operating system.

use std::path::Path;
use std::process::{Command, Output};

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn output_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn write_main(root: &Path) {
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("src/main.garnet"),
        "@caps()\ndef main() { \"MAIN-RAN\" }\n",
    )
    .unwrap();
}

fn write_main_requiring_dependency(root: &Path) {
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("src/main.garnet"),
        "@caps()\ndef main() { probe_value() }\n",
    )
    .unwrap();
}

fn write_probe_vendor(root: &Path) {
    std::fs::create_dir_all(root.join(".garnet/vendor/probe")).unwrap();
    std::fs::write(
        root.join(".garnet/vendor/probe/lib.garnet"),
        "def probe_value() { \"DEP-LOADED\" }\n",
    )
    .unwrap();
}

fn assert_toml_dependency_loads(manifest: &str, declaration: &str) {
    let dir = tempfile::TempDir::new().unwrap();
    write_main_requiring_dependency(dir.path());
    write_probe_vendor(dir.path());
    std::fs::write(dir.path().join("Garnet.toml"), manifest).unwrap();

    let output = run_interp(dir.path());
    let combined = output_text(&output);
    assert!(
        output.status.success() && combined.contains("DEP-LOADED"),
        "valid TOML 1.0 {declaration} dependency declaration must load; output:\n{combined}"
    );
}

fn write_dependency_manifest(root: &Path, vendor: &str) {
    std::fs::write(
        root.join("Garnet.toml"),
        format!(
            "[package]\nname = \"setup-probe\"\n\n[dependencies]\n\
             probe = {{ path = \"probe\", vendor = \"{vendor}\" }}\n"
        ),
    )
    .unwrap();
}

fn run_interp(root: &Path) -> Output {
    garnet()
        .arg("run")
        .arg("--interp")
        .arg(root.join("src/main.garnet"))
        .output()
        .unwrap()
}

fn assert_setup_red(output: Output, reason: &str) {
    let combined = output_text(&output);
    assert!(
        !output.status.success(),
        "{reason} must fail setup instead of running main; output:\n{combined}"
    );
    assert!(
        !combined.contains("MAIN-RAN"),
        "{reason} must stop before main; output:\n{combined}"
    );
}

fn write_passing_test(root: &Path) {
    std::fs::create_dir_all(root.join("tests")).unwrap();
    std::fs::write(
        root.join("tests/probe.garnet"),
        "@caps()\ndef test_ok() -> bool { true }\n",
    )
    .unwrap();
}

fn run_tests(root: &Path) -> Output {
    garnet().arg("test").arg(root).output().unwrap()
}

#[test]
fn malformed_dependency_entry_fails_before_main() {
    let dir = tempfile::TempDir::new().unwrap();
    write_main(dir.path());
    std::fs::write(
        dir.path().join("Garnet.toml"),
        "[dependencies]\nprobe = { path = \"probe\" }\n",
    )
    .unwrap();

    assert_setup_red(run_interp(dir.path()), "malformed dependency entry");
}

#[test]
fn dependency_row_with_trailing_non_toml_tokens_fails_before_main() {
    let dir = tempfile::TempDir::new().unwrap();
    write_main(dir.path());
    std::fs::write(
        dir.path().join("Garnet.toml"),
        "[dependencies]\nprobe = { vendor = \".garnet/vendor/probe\" THIS IS NOT TOML }\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.path().join(".garnet/vendor/probe")).unwrap();

    assert_setup_red(
        run_interp(dir.path()),
        "dependency row with trailing non-TOML tokens",
    );
}

#[test]
fn malformed_dependency_header_fails_before_main() {
    let dir = tempfile::TempDir::new().unwrap();
    write_main(dir.path());
    std::fs::write(
        dir.path().join("Garnet.toml"),
        "[dependencies\nprobe = { path = \"probe\", vendor = \".garnet/vendor/probe\" }\n",
    )
    .unwrap();

    assert_setup_red(run_interp(dir.path()), "malformed dependency header");
}

#[test]
fn spaced_and_quoted_dependency_header_loads() {
    assert_toml_dependency_loads(
        "[ \"dependencies\" ]\n\"probe\"={path=\"probe\",vendor=\".garnet/vendor/probe\"}\n",
        "spaced and quoted header",
    );
}

#[test]
fn dotted_dependency_key_loads() {
    assert_toml_dependency_loads(
        "dependencies.probe={path=\"probe\",vendor=\".garnet/vendor/probe\"}\n",
        "dotted key",
    );
}

#[test]
fn inline_top_level_dependency_table_loads() {
    assert_toml_dependency_loads(
        "dependencies={probe={path=\"probe\",vendor=\".garnet/vendor/probe\"}}\n",
        "inline top-level table",
    );
}

#[test]
fn dependency_subtable_loads() {
    assert_toml_dependency_loads(
        "[dependencies.probe]\npath=\"probe\"\nvendor=\".garnet/vendor/probe\"\n",
        "dependency subtable",
    );
}

#[test]
fn duplicate_dependency_keys_fail_before_main() {
    let dir = tempfile::TempDir::new().unwrap();
    write_main(dir.path());
    write_probe_vendor(dir.path());
    std::fs::write(
        dir.path().join("Garnet.toml"),
        "[dependencies]\nprobe={path=\"one\",vendor=\".garnet/vendor/probe\"}\nprobe={path=\"two\",vendor=\".garnet/vendor/probe\"}\n",
    )
    .unwrap();

    assert_setup_red(run_interp(dir.path()), "duplicate dependency keys");
}

#[test]
fn unreadable_manifest_shape_fails_before_main() {
    let dir = tempfile::TempDir::new().unwrap();
    write_main(dir.path());
    std::fs::create_dir(dir.path().join("Garnet.toml")).unwrap();

    assert_setup_red(run_interp(dir.path()), "unreadable Garnet.toml");
}

#[test]
fn missing_declared_vendor_path_fails_before_main() {
    let dir = tempfile::TempDir::new().unwrap();
    write_main(dir.path());
    write_dependency_manifest(dir.path(), ".garnet/vendor/missing");

    assert_setup_red(run_interp(dir.path()), "missing declared vendor path");
}

#[test]
fn mismatched_dependency_vendor_name_fails_before_main() {
    let dir = tempfile::TempDir::new().unwrap();
    write_main(dir.path());
    write_dependency_manifest(dir.path(), ".garnet/vendor/other");
    std::fs::create_dir_all(dir.path().join(".garnet/vendor/other")).unwrap();

    assert_setup_red(
        run_interp(dir.path()),
        "vendor path not bound to dependency name",
    );
}

#[test]
fn dot_vendor_path_fails_before_main() {
    let dir = tempfile::TempDir::new().unwrap();
    write_main(dir.path());
    write_dependency_manifest(dir.path(), ".");

    assert_setup_red(run_interp(dir.path()), "dot vendor path");
}

#[test]
fn parent_vendor_path_fails_before_main() {
    let outer = tempfile::TempDir::new().unwrap();
    let root = outer.path().join("project");
    std::fs::create_dir(&root).unwrap();
    write_main(&root);
    write_dependency_manifest(&root, "..");

    assert_setup_red(run_interp(&root), "parent vendor path");
}

#[test]
fn traversing_vendor_path_fails_before_main() {
    let dir = tempfile::TempDir::new().unwrap();
    write_main(dir.path());
    write_dependency_manifest(dir.path(), ".garnet/vendor/probe/../../..");
    std::fs::create_dir_all(dir.path().join(".garnet/vendor/probe")).unwrap();

    assert_setup_red(run_interp(dir.path()), "traversing vendor path");
}

#[test]
fn absolute_vendor_path_fails_before_main() {
    let dir = tempfile::TempDir::new().unwrap();
    let external = tempfile::TempDir::new().unwrap();
    write_main(dir.path());
    write_dependency_manifest(dir.path(), &external.path().to_string_lossy());

    assert_setup_red(run_interp(dir.path()), "absolute vendor path");
}

#[cfg(unix)]
#[test]
fn symlinked_canonical_vendor_root_fails_before_main() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::TempDir::new().unwrap();
    let external = tempfile::TempDir::new().unwrap();
    write_main(dir.path());
    write_dependency_manifest(dir.path(), ".garnet/vendor/probe");
    std::fs::create_dir_all(dir.path().join(".garnet/vendor")).unwrap();
    symlink(external.path(), dir.path().join(".garnet/vendor/probe")).unwrap();

    assert_setup_red(run_interp(dir.path()), "symlinked vendor escape");
}

#[cfg(unix)]
#[test]
fn symlinked_vendor_ancestor_escape_fails_before_main() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::TempDir::new().unwrap();
    let external = tempfile::TempDir::new().unwrap();
    write_main(dir.path());
    write_dependency_manifest(dir.path(), ".garnet/vendor/probe");
    std::fs::create_dir_all(external.path().join("vendor/probe")).unwrap();
    symlink(external.path(), dir.path().join(".garnet")).unwrap();

    assert_setup_red(run_interp(dir.path()), "symlinked vendor ancestor escape");
}

#[test]
fn non_directory_vendor_root_fails_before_main() {
    let dir = tempfile::TempDir::new().unwrap();
    write_main(dir.path());
    write_dependency_manifest(dir.path(), ".garnet/vendor/probe");
    std::fs::create_dir_all(dir.path().join(".garnet/vendor")).unwrap();
    std::fs::write(dir.path().join(".garnet/vendor/probe"), b"not a directory").unwrap();

    assert_setup_red(run_interp(dir.path()), "non-directory vendor root");
}

#[test]
fn non_regular_vendor_source_fails_before_main() {
    let dir = tempfile::TempDir::new().unwrap();
    write_main(dir.path());
    write_dependency_manifest(dir.path(), ".garnet/vendor/probe");
    std::fs::create_dir_all(dir.path().join(".garnet/vendor/probe/lib.garnet")).unwrap();

    assert_setup_red(run_interp(dir.path()), "non-regular vendored Garnet source");
}

#[test]
fn malformed_vendor_source_fails_before_main() {
    let dir = tempfile::TempDir::new().unwrap();
    write_main(dir.path());
    write_dependency_manifest(dir.path(), ".garnet/vendor/probe");
    std::fs::create_dir_all(dir.path().join(".garnet/vendor/probe")).unwrap();
    std::fs::write(
        dir.path().join(".garnet/vendor/probe/lib.garnet"),
        "def broken( {\n",
    )
    .unwrap();

    assert_setup_red(run_interp(dir.path()), "malformed vendored Garnet source");
}

#[test]
fn non_utf8_vendor_source_fails_before_main() {
    let dir = tempfile::TempDir::new().unwrap();
    write_main(dir.path());
    write_dependency_manifest(dir.path(), ".garnet/vendor/probe");
    std::fs::create_dir_all(dir.path().join(".garnet/vendor/probe")).unwrap();
    std::fs::write(
        dir.path().join(".garnet/vendor/probe/lib.garnet"),
        [0xff, 0xfe],
    )
    .unwrap();

    assert_setup_red(
        run_interp(dir.path()),
        "unreadable-as-Garnet vendored source",
    );
}

#[test]
fn well_formed_vendor_still_runs() {
    let dir = tempfile::TempDir::new().unwrap();
    write_main(dir.path());
    write_dependency_manifest(dir.path(), ".garnet/vendor/probe");
    std::fs::create_dir_all(dir.path().join(".garnet/vendor/probe")).unwrap();
    std::fs::write(
        dir.path().join(".garnet/vendor/probe/lib.garnet"),
        "def helper() { 1 }\n",
    )
    .unwrap();

    let output = run_interp(dir.path());
    let combined = output_text(&output);
    assert!(
        output.status.success() && combined.contains("MAIN-RAN"),
        "well-formed vendor control must run; output:\n{combined}"
    );
}

#[test]
fn bare_file_without_manifest_still_runs() {
    let dir = tempfile::TempDir::new().unwrap();
    write_main(dir.path());

    let output = run_interp(dir.path());
    let combined = output_text(&output);
    assert!(
        output.status.success() && combined.contains("MAIN-RAN"),
        "bare-file control must run; output:\n{combined}"
    );
}

#[test]
fn project_with_empty_dependency_table_still_runs() {
    let dir = tempfile::TempDir::new().unwrap();
    write_main(dir.path());
    std::fs::write(
        dir.path().join("Garnet.toml"),
        "[package]\nname = \"no-deps\"\n\n[dependencies]\n",
    )
    .unwrap();

    let output = run_interp(dir.path());
    let combined = output_text(&output);
    assert!(
        output.status.success() && combined.contains("MAIN-RAN"),
        "empty dependency-table control must run; output:\n{combined}"
    );
}

#[test]
fn existing_non_directory_tests_input_fails_closed() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(dir.path().join("tests"), b"not a directory").unwrap();

    let output = run_tests(dir.path());
    let combined = output_text(&output);
    assert!(
        !output.status.success(),
        "existing non-directory tests input must not become no-files success; output:\n{combined}"
    );
}

#[test]
fn nonexistent_explicit_project_root_fails_closed() {
    let dir = tempfile::TempDir::new().unwrap();
    let missing = dir.path().join("missing-project");

    let output = run_tests(&missing);
    let combined = output_text(&output);
    assert!(
        !output.status.success(),
        "nonexistent explicit project root must not become no-files success; output:\n{combined}"
    );
}

#[test]
fn existing_empty_project_root_remains_a_no_tests_success() {
    let dir = tempfile::TempDir::new().unwrap();

    let output = run_tests(dir.path());
    let combined = output_text(&output);
    assert!(
        output.status.success() && combined.contains("no .garnet files found"),
        "existing empty project remains a legitimate no-tests control; output:\n{combined}"
    );
}

#[test]
fn multiple_positional_test_roots_are_a_cli_error() {
    let first = tempfile::TempDir::new().unwrap();
    let second = tempfile::TempDir::new().unwrap();
    let output = garnet()
        .arg("test")
        .arg(first.path())
        .arg(second.path())
        .output()
        .unwrap();
    let combined = output_text(&output);
    assert_eq!(
        output.status.code(),
        Some(2),
        "multiple positional roots must be a CLI usage error; output:\n{combined}"
    );
}

#[test]
fn non_regular_discovered_test_entry_fails_closed() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("tests/probe.garnet")).unwrap();

    let output = run_tests(dir.path());
    let combined = output_text(&output);
    assert!(
        !output.status.success(),
        "non-regular discovered test entry must fail discovery; output:\n{combined}"
    );
}

#[test]
fn non_regular_main_helper_fails_closed() {
    let dir = tempfile::TempDir::new().unwrap();
    write_passing_test(dir.path());
    std::fs::create_dir_all(dir.path().join("src/main.garnet")).unwrap();

    let output = run_tests(dir.path());
    let combined = output_text(&output);
    assert!(
        !output.status.success(),
        "non-regular src/main.garnet must not be omitted from a green run; output:\n{combined}"
    );
}

#[test]
fn non_utf8_main_helper_fails_closed() {
    let dir = tempfile::TempDir::new().unwrap();
    write_passing_test(dir.path());
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/main.garnet"), [0xff, 0xfe]).unwrap();

    let output = run_tests(dir.path());
    let combined = output_text(&output);
    assert!(
        !output.status.success(),
        "unreadable-as-Garnet src/main.garnet must not be omitted from a green run; output:\n{combined}"
    );
}

#[test]
fn ordinary_test_discovery_still_passes() {
    let dir = tempfile::TempDir::new().unwrap();
    write_passing_test(dir.path());

    let output = run_tests(dir.path());
    let combined = output_text(&output);
    assert!(
        output.status.success() && combined.contains("1 passed; 0 failed"),
        "ordinary test control must stay green; output:\n{combined}"
    );
}
