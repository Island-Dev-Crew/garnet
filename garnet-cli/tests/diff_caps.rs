//! S37 — `garnet diff-caps` integration test (runs the built binary).
//!
//! The contract gate: non-zero exit iff the program GAINED authority between two
//! revisions; zero when caps only shrink or stay the same.

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn fresh(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("garnet_s37_{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(dir: &Path, name: &str, body: &str) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, body).unwrap();
    p
}

#[test]
fn authority_expansion_exits_nonzero() {
    let dir = fresh("expand");
    let old = write(dir.as_path(), "old.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let new = write(
        dir.as_path(),
        "new.garnet",
        "@caps(fs, net)\ndef main() { 1 }\n",
    );
    let out = garnet()
        .arg("diff-caps")
        .arg(&old)
        .arg(&new)
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(1),
        "authority expansion must exit non-zero"
    );
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("AUTHORITY EXPANDED"), "{s}");
    assert!(s.contains("caps GAINED") && s.contains("net"), "{s}");
}

#[test]
fn capability_reduction_exits_zero() {
    let dir = fresh("reduce");
    let old = write(
        dir.as_path(),
        "old.garnet",
        "@caps(fs, net)\ndef main() { 1 }\n",
    );
    let new = write(dir.as_path(), "new.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let out = garnet()
        .arg("diff-caps")
        .arg(&old)
        .arg(&new)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "a capability reduction must exit zero"
    );
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("no authority expansion"), "{s}");
    assert!(s.contains("caps removed") && s.contains("net"), "{s}");
}

#[test]
fn identical_surface_exits_zero_with_no_changes() {
    let dir = fresh("same");
    let a = write(dir.as_path(), "a.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let b = write(dir.as_path(), "b.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let out = garnet().arg("diff-caps").arg(&a).arg(&b).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("no capability changes"), "{s}");
}

#[test]
fn verify_caps_baseline_caps_the_fused_band() {
    // Completes the S33 graft: with a baseline, the diff-caps capability signal
    // feeds verify's fused `min`. A current tree that gained `net` vs the
    // baseline caps the fused merge confidence at 2/5.
    let dir = fresh("verify_baseline");
    let old = write(dir.as_path(), "old.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let cur = write(
        dir.as_path(),
        "cur.garnet",
        "@caps(fs, net)\ndef main() { 1 }\n",
    );
    let out = garnet()
        .arg("verify")
        .arg(&cur)
        .arg("--caps-baseline")
        .arg(&old)
        .output()
        .unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(
        s.contains("capability signal (diff-caps vs baseline): 2/5"),
        "{s}"
    );
    assert!(s.contains("Merge confidence (fused): 2/5"), "{s}");
}

// ── RB-1 (Directive 15): --machine JSON verdict ─────────────────────────

#[test]
fn machine_expansion_emits_json_verdict_and_same_exit_code() {
    let dir = fresh("machine_expand");
    let old = write(dir.as_path(), "old.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let new = write(
        dir.as_path(),
        "new.garnet",
        "@caps(fs, net)\ndef main() { 1 }\n",
    );
    let out = garnet()
        .arg("diff-caps")
        .arg("--machine")
        .arg(&old)
        .arg(&new)
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(1),
        "--machine must keep the gating exit code"
    );
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(
        s.contains("\"schema\":\"garnet.diff-caps.machine/1\""),
        "{s}"
    );
    assert!(s.contains("\"verdict\":\"authority-expanded\""), "{s}");
    assert!(s.contains("\"capability_band\":\"2/5\""), "{s}");
    assert!(s.contains("\"aggregate_gained\":[\"net\"]"), "{s}");
    assert!(
        !s.contains("AUTHORITY EXPANDED"),
        "machine mode must emit only the JSON payload: {s}"
    );
}

#[test]
fn machine_no_expansion_emits_json_verdict_and_exit_zero() {
    let dir = fresh("machine_same");
    let a = write(dir.as_path(), "a.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let b = write(dir.as_path(), "b.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let out = garnet()
        .arg("diff-caps")
        .arg("--machine")
        .arg(&a)
        .arg(&b)
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("\"verdict\":\"no-authority-expansion\""), "{s}");
    assert!(s.contains("\"capability_band\":\"5/5\""), "{s}");
    assert!(s.contains("\"exit_code\":0"), "{s}");
}

#[test]
fn machine_output_is_deterministic_and_single_line() {
    let dir = fresh("machine_det");
    let old = write(dir.as_path(), "old.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let new = write(
        dir.as_path(),
        "new.garnet",
        "@caps(fs, net, time)\ndef main() { 1 }\n",
    );
    let run = || {
        let out = garnet()
            .arg("diff-caps")
            .arg("--machine")
            .arg(&old)
            .arg(&new)
            .output()
            .unwrap();
        String::from_utf8(out.stdout).unwrap()
    };
    let first = run();
    let second = run();
    assert_eq!(first, second, "machine output must be deterministic");
    assert_eq!(
        first.trim().lines().count(),
        1,
        "machine output is a single JSON line: {first}"
    );
}

#[test]
fn human_output_is_unchanged_without_machine_flag() {
    // The CI gate scripts parse the human text; --machine must be purely
    // additive. Golden assertion: everything below the path-bearing header
    // line is pinned byte-for-byte.
    let dir = fresh("human_stable");
    let old = write(dir.as_path(), "old.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let new = write(
        dir.as_path(),
        "new.garnet",
        "@caps(fs, net)\ndef main() { 1 }\n",
    );
    let out = garnet()
        .arg("diff-caps")
        .arg(&old)
        .arg(&new)
        .output()
        .unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    let (header, rest) = s.split_once('\n').expect("multi-line output");
    assert!(header.starts_with("garnet diff-caps: "), "{s}");
    assert_eq!(
        rest,
        "  + caps GAINED:  net\n\
         \x20 ~ main gained: net\n\
         \n\
         diff-caps: AUTHORITY EXPANDED — review required (capability band 2/5)\n",
        "human output below the header must be byte-stable"
    );
}

#[test]
fn unknown_diff_caps_flag_is_rejected() {
    // A typo'd flag must not be silently treated as a path (mirrors the
    // adjacent `verify` arm's explicit unknown-flag rejection).
    let out = garnet()
        .arg("diff-caps")
        .arg("--machin")
        .arg("a.garnet")
        .arg("b.garnet")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(err.contains("unknown diff-caps flag: --machin"), "{err}");
}

// ── crown C B-1: the walker must not hide DECLARED authority ────────────
//
// `collect_targets` (the shared collector behind `surface_for_path`) used to
// skip any directory *named* `vendor` or `node_modules` at any depth. A
// `.garnet` file placed there declared `@caps(net, fs)` and the authority gate
// never read it: `diff-caps --machine` said `no-authority-expansion`, band
// 5/5, exit 0. The verdict's `scope` string does not cover that case — it
// disclaims *undeclared* authority, while this authority is declared and
// simply unread.

/// An old/new pair whose ONLY difference is a `.garnet` file added at
/// `hidden_rel` in the new tree, declaring `@caps(net, fs)`.
fn hidden_authority_pair(tag: &str, hidden_rel: &str) -> (PathBuf, PathBuf) {
    let root = fresh(tag);
    let old = root.join("old");
    let new = root.join("new");
    std::fs::create_dir_all(&old).unwrap();
    std::fs::create_dir_all(&new).unwrap();
    write(old.as_path(), "tool.garnet", "@caps()\ndef main() { 1 }\n");
    write(new.as_path(), "tool.garnet", "@caps()\ndef main() { 1 }\n");
    let hidden = new.join(hidden_rel);
    std::fs::create_dir_all(hidden.parent().unwrap()).unwrap();
    std::fs::write(&hidden, "@caps(net, fs)\ndef reach() { 1 }\n").unwrap();
    (old, new)
}

fn machine_diff(old: &Path, new: &Path) -> (i32, String) {
    let out = garnet()
        .arg("diff-caps")
        .arg("--machine")
        .arg(old)
        .arg(new)
        .output()
        .unwrap();
    (
        out.status.code().unwrap(),
        String::from_utf8(out.stdout).unwrap(),
    )
}

/// The reviewer's exact reproduction: a bare `vendor/` is NOT the documented
/// vendored path, so authority declared under it must reach the gate.
#[test]
fn declared_authority_under_bare_vendor_is_not_hidden() {
    let (old, new) = hidden_authority_pair("bare_vendor", "vendor/evil.garnet");
    let (code, s) = machine_diff(&old, &new);
    assert_eq!(code, 1, "authority declared under vendor/ must gate: {s}");
    assert!(s.contains("\"verdict\":\"authority-expanded\""), "{s}");
    assert!(s.contains("\"capability_band\":\"2/5\""), "{s}");
    assert!(s.contains("\"aggregate_gained\":[\"fs\",\"net\"]"), "{s}");
}

/// `node_modules` is an equally arbitrary directory name — a `.garnet` file
/// there was just as invisible to the gate.
#[test]
fn declared_authority_under_node_modules_is_not_hidden() {
    let (old, new) = hidden_authority_pair("node_modules", "node_modules/evil.garnet");
    let (code, s) = machine_diff(&old, &new);
    assert_eq!(
        code, 1,
        "authority declared under node_modules/ must gate: {s}"
    );
    assert!(s.contains("\"verdict\":\"authority-expanded\""), "{s}");
    assert!(s.contains("\"aggregate_gained\":[\"fs\",\"net\"]"), "{s}");
}

/// The old skip matched on the bare directory NAME at any depth, so nesting
/// the same directory deeper hid authority just as well.
#[test]
fn declared_authority_under_nested_vendor_is_not_hidden() {
    let (old, new) = hidden_authority_pair("nested_vendor", "a/b/vendor/evil.garnet");
    let (code, s) = machine_diff(&old, &new);
    assert_eq!(code, 1, "authority under a/b/vendor/ must gate: {s}");
    assert!(s.contains("\"verdict\":\"authority-expanded\""), "{s}");
    assert!(s.contains("\"aggregate_gained\":[\"fs\",\"net\"]"), "{s}");
}

/// The ONE documented vendored path (`.garnet/vendor/<name>`, garnet-cli
/// AGENTS.md) stays skipped — but the omission is now DISCLOSED in the machine
/// verdict, so a reviewer can see the walk was not total.
#[test]
fn documented_vendored_path_stays_skipped_and_the_omission_is_disclosed() {
    let (old, new) = hidden_authority_pair("dot_garnet_vendor", ".garnet/vendor/dep/lib.garnet");
    let (code, s) = machine_diff(&old, &new);
    assert_eq!(code, 0, "the documented vendored path stays skipped: {s}");
    assert!(s.contains("\"verdict\":\"no-authority-expansion\""), "{s}");
    assert!(s.contains("\"skipped_path_count\":1"), "{s}");
    assert!(
        s.contains("{\"rule\":\"vendored-dependencies\",\"count\":1}"),
        "{s}"
    );
}

/// Legitimate skips survive — build output, VCS internals, and the tool cache
/// are still not walked, and each is named in the disclosure.
#[test]
fn build_output_vcs_and_cache_stay_skipped_and_are_disclosed() {
    let root = fresh("legit_skips");
    let old = root.join("old");
    let new = root.join("new");
    std::fs::create_dir_all(&old).unwrap();
    std::fs::create_dir_all(&new).unwrap();
    write(old.as_path(), "tool.garnet", "@caps()\ndef main() { 1 }\n");
    write(new.as_path(), "tool.garnet", "@caps()\ndef main() { 1 }\n");
    for dir in ["target", ".git", ".garnet-cache"] {
        let d = new.join(dir);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("build.garnet"), "@caps(net, fs)\ndef g() { 1 }\n").unwrap();
    }
    let (code, s) = machine_diff(&old, &new);
    assert_eq!(code, 0, "build/VCS/cache trees stay skipped: {s}");
    assert!(s.contains("\"verdict\":\"no-authority-expansion\""), "{s}");
    assert!(s.contains("\"skipped_path_count\":3"), "{s}");
    assert!(s.contains("{\"rule\":\"build-output\",\"count\":1}"), "{s}");
    assert!(s.contains("{\"rule\":\"tool-cache\",\"count\":1}"), "{s}");
    assert!(s.contains("{\"rule\":\"vcs-metadata\",\"count\":1}"), "{s}");
}

/// A walk that declined nothing says so: zero skipped paths, empty rule list.
/// Absence of the field means a pre-cure binary, NOT a clean walk.
#[test]
fn a_total_walk_reports_zero_skipped_paths() {
    let dir = fresh("total_walk");
    let a = write(dir.as_path(), "a.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let b = write(dir.as_path(), "b.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let (code, s) = machine_diff(&a, &b);
    assert_eq!(code, 0, "{s}");
    assert!(s.contains("\"skipped_path_count\":0"), "{s}");
    assert!(s.contains("\"skipped_paths\":[]"), "{s}");
}

/// Cross-family review of this change (Codex, B1): a DIRECTORY reached through
/// a symlink was neither walked nor tallied — `DirEntry::file_type()` does not
/// follow links, so `src -> ../external` holding `evil.garnet` vanished while
/// the verdict still reported `skipped_path_count: 0`. The walk still does not
/// FOLLOW links (a link loop must terminate); it now DISCLOSES each one it
/// declines under `symlinked-directory`. Old/new pair whose only difference is
/// a `src` link, in the root named by `link_in`, to a real directory holding
/// `@caps(net, fs)`.
#[cfg(unix)]
fn symlinked_authority_pair(tag: &str, link_in: &str) -> (PathBuf, PathBuf) {
    let root = fresh(tag);
    let old = root.join("old");
    let new = root.join("new");
    let external = root.join("external");
    for d in [&old, &new, &external] {
        std::fs::create_dir_all(d).unwrap();
    }
    write(old.as_path(), "tool.garnet", "@caps()\ndef main() { 1 }\n");
    write(new.as_path(), "tool.garnet", "@caps()\ndef main() { 1 }\n");
    std::fs::write(
        external.join("evil.garnet"),
        "@caps(net, fs)\ndef reach() { 1 }\n",
    )
    .unwrap();
    let side = if link_in == "old" { &old } else { &new };
    std::os::unix::fs::symlink(&external, side.join("src")).unwrap();
    (old, new)
}

/// A linked directory in the NEW root: not followed, so no expansion is seen —
/// and the verdict must say so instead of claiming a complete walk.
#[cfg(unix)]
#[test]
fn symlinked_directory_in_the_new_root_is_disclosed_not_followed() {
    let (old, new) = symlinked_authority_pair("symlink_new", "new");
    let (code, s) = machine_diff(&old, &new);
    assert_eq!(code, 0, "a linked directory is not followed: {s}");
    assert!(s.contains("\"verdict\":\"no-authority-expansion\""), "{s}");
    assert!(s.contains("\"skipped_path_count\":1"), "{s}");
    assert!(
        s.contains("{\"rule\":\"symlinked-directory\",\"count\":1}"),
        "{s}"
    );
}

/// The verdict covers both trees, so a link in the OLD root is tallied too.
#[cfg(unix)]
#[test]
fn symlinked_directory_in_the_old_root_is_disclosed_too() {
    let (old, new) = symlinked_authority_pair("symlink_old", "old");
    let (code, s) = machine_diff(&old, &new);
    assert_eq!(code, 0, "{s}");
    assert!(s.contains("\"skipped_path_count\":1"), "{s}");
    assert!(
        s.contains("{\"rule\":\"symlinked-directory\",\"count\":1}"),
        "{s}"
    );
}

/// A link back onto its own tree must terminate — and be tallied, not hidden.
#[cfg(unix)]
#[test]
fn symlink_loop_terminates_and_is_disclosed() {
    let root = fresh("symlink_loop");
    let old = root.join("old");
    let new = root.join("new");
    std::fs::create_dir_all(&old).unwrap();
    std::fs::create_dir_all(&new).unwrap();
    write(old.as_path(), "tool.garnet", "@caps()\ndef main() { 1 }\n");
    write(new.as_path(), "tool.garnet", "@caps()\ndef main() { 1 }\n");
    std::os::unix::fs::symlink(&new, new.join("loop")).unwrap();
    let (code, s) = machine_diff(&old, &new);
    assert_eq!(code, 0, "{s}");
    assert!(s.contains("\"skipped_path_count\":1"), "{s}");
    assert!(
        s.contains("{\"rule\":\"symlinked-directory\",\"count\":1}"),
        "{s}"
    );
}

/// A linked `.garnet` FILE is read through the link as before — only linked
/// DIRECTORIES are declined — so authority declared in one still gates.
#[cfg(unix)]
#[test]
fn symlinked_garnet_file_is_still_read() {
    let root = fresh("symlink_file");
    let old = root.join("old");
    let new = root.join("new");
    std::fs::create_dir_all(&old).unwrap();
    std::fs::create_dir_all(&new).unwrap();
    write(old.as_path(), "tool.garnet", "@caps()\ndef main() { 1 }\n");
    write(new.as_path(), "tool.garnet", "@caps()\ndef main() { 1 }\n");
    let real = root.join("real.garnet");
    std::fs::write(&real, "@caps(net, fs)\ndef reach() { 1 }\n").unwrap();
    std::os::unix::fs::symlink(&real, new.join("linked.garnet")).unwrap();
    let (code, s) = machine_diff(&old, &new);
    assert_eq!(code, 1, "authority behind a linked file must gate: {s}");
    assert!(s.contains("\"aggregate_gained\":[\"fs\",\"net\"]"), "{s}");
    assert!(s.contains("\"skipped_path_count\":0"), "{s}");
}

/// Review v2 (Codex, B1-v2): `path.is_dir()` collapses an I/O error to
/// `false`, so a link whose EXISTING directory target sat behind a
/// non-searchable parent was neither followed, tallied, nor errored — exit 0,
/// `skipped_path_count: 0`, while the target held `@caps(net, fs)`. Now a
/// link the walk cannot resolve is an error (exit 2, no verdict): a gate must
/// not print a green it could not earn. Restores the parent's mode on drop so
/// the fixture can be cleaned up even when an assertion fails.
#[cfg(unix)]
struct RestoreMode(PathBuf);
#[cfg(unix)]
impl Drop for RestoreMode {
    fn drop(&mut self) {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&self.0, std::fs::Permissions::from_mode(0o700));
    }
}

/// Old/new pair; in the root named by `link_in`, `src` links to a real
/// directory holding `@caps(net, fs)` whose PARENT is mode 000.
#[cfg(unix)]
fn locked_link_pair(tag: &str, link_in: &str) -> (PathBuf, PathBuf, RestoreMode) {
    use std::os::unix::fs::PermissionsExt;
    let root = fresh(tag);
    let old = root.join("old");
    let new = root.join("new");
    let locked = root.join("locked");
    let actual = locked.join("actual");
    for d in [&old, &new, &actual] {
        std::fs::create_dir_all(d).unwrap();
    }
    write(old.as_path(), "tool.garnet", "@caps()\ndef main() { 1 }\n");
    write(new.as_path(), "tool.garnet", "@caps()\ndef main() { 1 }\n");
    std::fs::write(
        actual.join("evil.garnet"),
        "@caps(net, fs)\ndef reach() { 1 }\n",
    )
    .unwrap();
    let side = if link_in == "old" { &old } else { &new };
    std::os::unix::fs::symlink(&actual, side.join("src")).unwrap();
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();
    (old, new, RestoreMode(locked))
}

#[cfg(unix)]
fn is_root() -> bool {
    std::process::Command::new("id")
        .arg("-u")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "0")
        .unwrap_or(false)
}

#[cfg(unix)]
fn unresolvable_link_is_an_error(link_in: &str) {
    if is_root() {
        eprintln!("skipped: root bypasses directory permissions");
        return;
    }
    let (old, new, _guard) = locked_link_pair(&format!("locked_link_{link_in}"), link_in);
    let out = garnet()
        .arg("diff-caps")
        .arg("--machine")
        .arg(&old)
        .arg(&new)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(2),
        "an unresolvable link must be an error, never a silent zero: {stderr}"
    );
    assert!(out.stdout.is_empty(), "no verdict on error");
    assert!(stderr.contains("src"), "the error names the link: {stderr}");
}

#[cfg(unix)]
#[test]
fn unresolvable_link_in_the_new_root_is_an_error_not_a_silent_zero() {
    unresolvable_link_is_an_error("new");
}

#[cfg(unix)]
#[test]
fn unresolvable_link_in_the_old_root_is_an_error_not_a_silent_zero() {
    unresolvable_link_is_an_error("old");
}

/// A link that points at itself resolves to nothing readable; it is an error
/// (ELOOP) rather than a silent pass — the walk does not guess.
#[cfg(unix)]
#[test]
fn self_referential_link_is_an_error() {
    let root = fresh("self_link");
    let old = root.join("old");
    let new = root.join("new");
    std::fs::create_dir_all(&old).unwrap();
    std::fs::create_dir_all(&new).unwrap();
    write(old.as_path(), "tool.garnet", "@caps()\ndef main() { 1 }\n");
    write(new.as_path(), "tool.garnet", "@caps()\ndef main() { 1 }\n");
    std::os::unix::fs::symlink("cycle", new.join("cycle")).unwrap();
    let out = garnet()
        .arg("diff-caps")
        .arg("--machine")
        .arg(&old)
        .arg(&new)
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.stdout.is_empty());
}

/// The supplied root itself may be a link: the OS resolves it and the tree
/// behind it is walked — only links met BELOW the root are declined.
#[cfg(unix)]
#[test]
fn explicit_root_that_is_a_link_is_resolved_and_walked() {
    let root = fresh("root_link");
    let old = root.join("old");
    let real = root.join("real");
    std::fs::create_dir_all(&old).unwrap();
    std::fs::create_dir_all(&real).unwrap();
    write(old.as_path(), "tool.garnet", "@caps()\ndef main() { 1 }\n");
    write(
        real.as_path(),
        "tool.garnet",
        "@caps(net, fs)\ndef main() { 1 }\n",
    );
    std::os::unix::fs::symlink(&real, root.join("new")).unwrap();
    let (code, s) = machine_diff(&old, &root.join("new"));
    assert_eq!(code, 1, "{s}");
    assert!(s.contains("\"aggregate_gained\":[\"fs\",\"net\"]"), "{s}");
    assert!(s.contains("\"skipped_path_count\":0"), "{s}");
}
