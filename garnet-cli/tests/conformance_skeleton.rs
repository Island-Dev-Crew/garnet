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
fn deferred_dynamic_dispatch() {
    let src = r#"
@dynamic
struct Service {
  name: String
}

@caps()
def main() {
  let svc = Service("auth")
  svc.def_method(:label) do |receiver|
    receiver.name
  end
  svc.label()
}
"#;
    let path = temp_source("dynamic_dispatch", src);
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
        stdout.contains("=> auth"),
        "expected dynamic method dispatch to return field value, got:\n{stdout}"
    );
}

#[test]
fn static_impl_dispatch_and_method_missing() {
    let src = r#"
struct Service {
  name: String
}

impl Service {
  def label(receiver) {
    receiver.name
  }

  def method_missing(receiver, name, args) {
    "fallback"
  }
}

@caps()
def main() {
  let svc = Service("auth")
  svc.label() + "|" + svc.status()
}
"#;
    let path = temp_source("static_impl_dispatch_and_method_missing", src);
    assert_ok(&["parse"], &path);
    assert_ok(&["check"], &path);
    let out = run(&["run"], &path);
    assert!(
        out.status.success(),
        "static impl dispatch and method_missing should run\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("=> auth|fallback"),
        "expected static impl dispatch before method_missing fallback, got:\n{stdout}"
    );
}

#[test]
fn deferred_structural_protocols() {
    let ok_src = r#"
protocol Renderable {
  def render()
}

@dynamic
struct Widget {
  name: String
}

@caps()
def accept_renderable(item: Renderable) {
  item.render()
}

@caps()
def main() {
  let widget = Widget("card")
  widget.def_method(:render) do |receiver|
    receiver.name
  end
  accept_renderable(widget)
}
"#;
    let ok_path = temp_source("structural_protocol_dynamic_ok", ok_src);
    assert_ok(&["parse"], &ok_path);
    assert_ok(&["check"], &ok_path);
    let ok = run(&["run"], &ok_path);
    assert!(
        ok.status.success(),
        "dynamic value satisfying protocol should run\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&ok.stdout),
        String::from_utf8_lossy(&ok.stderr)
    );
    let stdout = String::from_utf8_lossy(&ok.stdout);
    assert!(
        stdout.contains("=> card"),
        "expected protocol-accepted dynamic method to return field value, got:\n{stdout}"
    );

    let static_src = r#"
protocol Renderable {
  def render()
}

struct StaticWidget {
  name: String
}

impl StaticWidget {
  def render(receiver) {
    receiver.name
  }
}

@caps()
def accept_renderable(item: Renderable) {
  item.render()
}

@caps()
def main() {
  let widget = StaticWidget("panel")
  accept_renderable(widget)
}
"#;
    let static_path = temp_source("structural_protocol_static_impl_ok", static_src);
    assert_ok(&["parse"], &static_path);
    assert_ok(&["check"], &static_path);
    let static_out = run(&["run"], &static_path);
    assert!(
        static_out.status.success(),
        "static impl method satisfying protocol should run\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&static_out.stdout),
        String::from_utf8_lossy(&static_out.stderr)
    );
    let stdout = String::from_utf8_lossy(&static_out.stdout);
    assert!(
        stdout.contains("=> panel"),
        "expected protocol-accepted static impl method to return field value, got:\n{stdout}"
    );

    let src = r#"
protocol Renderable {
  def render()
}

struct Plain {
  name: String
}

@caps()
def accept_renderable(item: Renderable) {
  1
}

@caps()
def main() {
  let plain = Plain("box")
  accept_renderable(plain)
}
"#;
    let path = temp_source("structural_protocol_missing_method", src);
    assert_ok(&["parse"], &path);
    assert_ok(&["check"], &path);
    let out = run(&["run"], &path);
    assert!(
        !out.status.success(),
        "protocol parameter must reject a value missing the required method"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("does not satisfy protocol Renderable"),
        "expected structural protocol diagnostic, got:\n{stderr}"
    );

    let arity_src = r#"
protocol Combiner {
  def combine(left, right)
}

struct OneArgCombiner {
  name: String
}

impl OneArgCombiner {
  def combine(receiver, left) {
    left
  }
}

@caps()
def accept_combiner(item: Combiner) {
  1
}

@caps()
def main() {
  accept_combiner(OneArgCombiner("bad"))
}
"#;
    let arity_path = temp_source("structural_protocol_arity_mismatch", arity_src);
    assert_ok(&["parse"], &arity_path);
    assert_ok(&["check"], &arity_path);
    let arity_out = run(&["run"], &arity_path);
    assert!(
        !arity_out.status.success(),
        "protocol parameter must reject a method with incompatible arity"
    );
    let stderr = String::from_utf8_lossy(&arity_out.stderr);
    assert!(
        stderr.contains("does not satisfy protocol Combiner"),
        "expected structural protocol arity diagnostic, got:\n{stderr}"
    );

    let param_type_src = r#"
protocol NumericCombiner {
  def combine(left: Int, right: Int) -> Int
}

struct StringCombiner {
  name: String
}

impl StringCombiner {
  def combine(receiver, left: String, right: String) -> String {
    left
  }
}

@caps()
def accept_numeric(item: NumericCombiner) {
  1
}

@caps()
def main() {
  accept_numeric(StringCombiner("bad"))
}
"#;
    let param_type_path = temp_source(
        "structural_protocol_parameter_type_mismatch",
        param_type_src,
    );
    assert_ok(&["parse"], &param_type_path);
    assert_ok(&["check"], &param_type_path);
    let param_type_out = run(&["run"], &param_type_path);
    assert!(
        !param_type_out.status.success(),
        "protocol parameter must reject a method with incompatible parameter types"
    );
    let stderr = String::from_utf8_lossy(&param_type_out.stderr);
    assert!(
        stderr.contains("does not satisfy protocol NumericCombiner"),
        "expected structural protocol parameter-type diagnostic, got:\n{stderr}"
    );

    let return_src = r#"
protocol RenderText {
  def render() -> String
}

struct NumericRenderer {
  value: Int
}

impl NumericRenderer {
  def render(receiver) -> Int {
    receiver.value
  }
}

@caps()
def accept_renderer(item: RenderText) {
  1
}

@caps()
def main() {
  accept_renderer(NumericRenderer(7))
}
"#;
    let return_path = temp_source("structural_protocol_return_mismatch", return_src);
    assert_ok(&["parse"], &return_path);
    assert_ok(&["check"], &return_path);
    let return_out = run(&["run"], &return_path);
    assert!(
        !return_out.status.success(),
        "protocol parameter must reject a method with incompatible return type"
    );
    let stderr = String::from_utf8_lossy(&return_out.stderr);
    assert!(
        stderr.contains("does not satisfy protocol RenderText"),
        "expected structural protocol return diagnostic, got:\n{stderr}"
    );

    let mode_src = r#"
protocol SafeHasher {
  fn hash(input: Int) -> Int
}

struct ManagedHasher {
  seed: Int
}

impl ManagedHasher {
  def hash(receiver, input: Int) -> Int {
    input
  }
}

@caps()
def accept_hasher(item: SafeHasher) {
  1
}

@caps()
def main() {
  accept_hasher(ManagedHasher(0))
}
"#;
    let mode_path = temp_source("structural_protocol_mode_mismatch", mode_src);
    assert_ok(&["parse"], &mode_path);
    assert_ok(&["check"], &mode_path);
    let mode_out = run(&["run"], &mode_path);
    assert!(
        !mode_out.status.success(),
        "protocol parameter must reject a method with incompatible mode"
    );
    let stderr = String::from_utf8_lossy(&mode_out.stderr);
    assert!(
        stderr.contains("does not satisfy protocol SafeHasher"),
        "expected structural protocol mode diagnostic, got:\n{stderr}"
    );
}
