//! Mini-Spec conformance skeleton.
//!
//! Active tests cover rows from the v0.4.2 conformance matrix that are
//! implemented today. Ignored tests name partial/deferred rows so language
//! completeness work has stable test handles before the implementation lands.

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_garnet"))
}

fn temp_source(name: &str, src: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "garnet_conformance_{}_{}",
        name,
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.garnet"));
    std::fs::write(&path, src).unwrap();
    path
}

fn run(args: &[&str], path: &Path) -> std::process::Output {
    Command::new(garnet_bin())
        .args(args)
        .arg(path)
        .output()
        .unwrap()
}

fn assert_ok(args: &[&str], path: &Path) {
    let out = run(args, path);
    assert!(
        out.status.success(),
        "garnet {} {} failed\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        path.display(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

fn pending(feature: &str) {
    eprintln!("conformance placeholder pending implementation: {feature}");
}

#[test]
fn implemented_control_flow_and_interpreter_smoke_runs() {
    let src = r#"
@caps()
def main() {
  let mut i = 0
  let mut total = 0
  while i < 4 {
    if i > 1 {
      total += i
    }
    i += 1
  }
  total
}
"#;
    let path = temp_source("control_flow", src);
    assert_ok(&["parse"], &path);
    assert_ok(&["check"], &path);
    assert_ok(&["run"], &path);
}

#[test]
fn implemented_memory_declaration_parses() {
    let src = r#"
memory episodic events : EpisodeStore<String>

@caps()
def main() {
  1
}
"#;
    let path = temp_source("memory_decl", src);
    assert_ok(&["parse"], &path);
}

#[test]
fn implemented_capcaps_rejects_missing_fs_authority() {
    let src = r#"
@caps()
def main() {
  read_file("config.toml")
}
"#;
    let path = temp_source("missing_fs_cap", src);
    let out = run(&["check"], &path);
    assert!(
        !out.status.success(),
        "CapCaps must reject fs primitive without @caps(fs)"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("does not declare `fs`") || stdout.contains("does not declare .fs."),
        "expected missing fs diagnostic, got:\n{stdout}"
    );
}

#[test]
fn implemented_reproducible_manifest_smoke_builds() {
    let src = r#"
@caps()
def main() {
  42
}
"#;
    let path = temp_source("manifest_smoke", src);
    let out = Command::new(garnet_bin())
        .args(["build", "--deterministic", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "deterministic build failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(path.with_extension("garnet.manifest.json").exists());
}

#[test]
fn parser_parity_top_level_protocol_and_dyn_trait_parse() {
    let src = r#"
protocol Renderable {
  def render() -> String
}

@caps()
def inspect(item: dyn Renderable) {
  1
}
"#;
    let path = temp_source("protocol_dyn_trait", src);
    assert_ok(&["parse"], &path);
}

#[test]
fn parser_parity_yield_next_dynamic_and_nonsendable_parse() {
    let src = r#"
@dynamic
@nonsendable
struct DynamicObject {
  id: Int
}

@dynamic
impl DynamicObject {
  def label(self) {
    "dynamic"
  }
}

@caps()
def staged_block_surface() {
  yield 1
  next 2
  3
}
"#;
    let path = temp_source("parser_parity_surface", src);
    assert_ok(&["parse"], &path);
}

#[test]
#[ignore = "Mini-Spec §4.5 ARC cycle detection is deferred in v0.4.2"]
fn deferred_arc_cycle_detection() {
    pending("ARC + Bacon-Rajan cycle detection");
}

#[test]
fn deferred_blocks_and_yield() {
    let src = r#"
@caps()
def emit_each() {
  yield 1
  yield 2
}

@caps()
def main() {
  let mut total = 0
  emit_each() do |x|
    total += x
    next x
    total += 100
  end
  total
}
"#;
    let path = temp_source("blocks_yield_next", src);
    assert_ok(&["parse"], &path);
    assert_ok(&["check"], &path);
    let out = run(&["run"], &path);
    assert!(
        out.status.success(),
        "garnet run {} failed\nstdout:\n{}\nstderr:\n{}",
        path.display(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("=> 3"),
        "expected yielded block to accumulate 3 without post-next statements, got:\n{stdout}"
    );
}

#[test]
fn explicit_closure_argument_does_not_become_implicit_block() {
    let src = r#"
@caps()
def no_args() {
  7
}

@caps()
def main() {
  let f = |x| x
  no_args(f)
}
"#;
    let path = temp_source("explicit_closure_is_not_block", src);
    assert_ok(&["parse"], &path);
    assert_ok(&["check"], &path);
    let out = run(&["run"], &path);
    assert!(
        !out.status.success(),
        "ordinary closure argument must not be silently consumed as a do/end block"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("arity mismatch"),
        "expected arity mismatch for ordinary closure arg, got:\n{stderr}"
    );
}

#[test]
#[ignore = "Mini-Spec §8.5 full NLL/lifetime inference is deferred in v0.4.2"]
fn deferred_nll_lifetime_inference() {
    pending("full non-lexical lifetime inference");
}

#[test]
#[ignore = "Mini-Spec §8.6 formal borrow rules are partial in v0.4.2"]
fn partial_borrow_rule_suite() {
    pending("complete borrow-check B1-B5 conformance suite");
}

#[test]
#[ignore = "Mini-Spec §11.5 trait coherence is spec-only in v0.4.2"]
fn deferred_trait_coherence() {
    pending("formal trait coherence/orphan-rule enforcement");
}

#[test]
#[ignore = "Mini-Spec §11.6 monomorphization is parsed-only in v0.4.2"]
fn parsed_only_monomorphization() {
    pending("generic monomorphization and zero-cost theorem evidence");
}

#[test]
#[ignore = "Mini-Spec §11.7 dynamic dispatch is deferred in v0.4.2"]
fn deferred_dynamic_dispatch() {
    pending("@dynamic method dispatch table");
}

#[test]
#[ignore = "Mini-Spec §11.8 structural protocols are deferred in v0.4.2"]
fn deferred_structural_protocols() {
    pending("structural protocol checking and runtime casts");
}
