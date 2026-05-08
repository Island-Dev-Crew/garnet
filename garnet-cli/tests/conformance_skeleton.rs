//! Mini-Spec conformance skeleton.
//!
//! Active tests cover rows from the v0.4.2 conformance matrix that are
//! implemented today. Ignored tests name partial/deferred rows so language
//! completeness work has stable test handles before the implementation lands.

use garnet_memory::{CycleGraph, CycleScan, MemoryKind};
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
fn actor_sendable_rejects_nonsendable_protocol_payloads() {
    let src = r#"
@nonsendable
struct LocalSocket {
  fd: Int
}

actor Worker {
  protocol submit(socket: LocalSocket) -> Int
  on submit(socket) {
    1
  }
}

@caps()
def main() {
  1
}
"#;
    let path = temp_source("actor_sendable_nonsendable_payload", src);
    assert_ok(&["parse"], &path);
    let out = run(&["check"], &path);
    assert!(
        !out.status.success(),
        "actor protocol payloads must reject @nonsendable types"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("@nonsendable type `LocalSocket`")
            && stdout.contains("actor `Worker` protocol `submit`"),
        "expected actor Sendable boundary diagnostic, got:\n{stdout}"
    );
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
fn deferred_arc_cycle_detection() {
    let mut graph = CycleGraph::new();
    let rooted = graph.add_node(MemoryKind::Working, "rooted");
    let reachable = graph.add_node(MemoryKind::Episodic, "reachable");
    let cycle_a = graph.add_node(MemoryKind::Working, "cycle_a");
    let cycle_b = graph.add_node(MemoryKind::Semantic, "cycle_b");

    graph.add_root(rooted).unwrap();
    graph.add_edge(rooted, reachable).unwrap();
    graph.add_edge(cycle_a, cycle_b).unwrap();
    graph.add_edge(cycle_b, cycle_a).unwrap();

    let report = graph.collect_cycles(CycleScan::Kind(MemoryKind::Working));

    assert_eq!(report.collected, vec![cycle_a, cycle_b]);
    assert!(graph.contains(rooted));
    assert!(graph.contains(reachable));
    assert!(!graph.contains(cycle_a));
    assert!(!graph.contains(cycle_b));
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
fn deferred_nll_lifetime_inference() {
    let missing_input_src = r#"
fn leak() -> &Buffer {
  0
}
"#;
    let missing_input_path = temp_source("lifetime_missing_input", missing_input_src);
    assert_ok(&["parse"], &missing_input_path);
    let out = run(&["check"], &missing_input_path);
    assert!(
        !out.status.success(),
        "safe-mode lifetime elision must reject reference returns without borrowed inputs"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("missing lifetime specifier"),
        "expected missing lifetime diagnostic, got:\n{stdout}"
    );

    let ambiguous_src = r#"
fn choose(borrow a: Buffer, borrow b: Buffer) -> &Buffer {
  a
}
"#;
    let ambiguous_path = temp_source("lifetime_ambiguous_inputs", ambiguous_src);
    assert_ok(&["parse"], &ambiguous_path);
    let out = run(&["check"], &ambiguous_path);
    assert!(
        !out.status.success(),
        "safe-mode lifetime elision must reject ambiguous reference returns with multiple borrowed inputs"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("multiple borrowed inputs"),
        "expected ambiguous lifetime diagnostic, got:\n{stdout}"
    );

    let single_input_src = r#"
fn view(borrow item: Buffer) -> &Buffer {
  item
}
"#;
    let single_input_path = temp_source("lifetime_single_input", single_input_src);
    assert_ok(&["parse"], &single_input_path);
    assert_ok(&["check"], &single_input_path);
}

#[test]
fn partial_borrow_rule_suite() {
    let use_after_move_src = r#"
fn consume(own x: Buffer) -> Int {
  0
}

fn caller(own b: Buffer) -> Int {
  consume(b)
  consume(b)
  0
}
"#;
    let use_after_move_path = temp_source("borrow_use_after_move", use_after_move_src);
    assert_ok(&["parse"], &use_after_move_path);
    let out = run(&["check"], &use_after_move_path);
    assert!(
        !out.status.success(),
        "safe-mode B4 must reject use-after-move through own parameters"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("use-after-move"),
        "expected use-after-move diagnostic, got:\n{stdout}"
    );

    let aliasing_src = r#"
fn frob(mut a: Buffer, borrow b: Buffer) -> Int {
  0
}

fn caller(mut x: Buffer) -> Int {
  frob(x, x)
}
"#;
    let aliasing_path = temp_source("borrow_aliasing_xor_mut", aliasing_src);
    assert_ok(&["parse"], &aliasing_path);
    let out = run(&["check"], &aliasing_path);
    assert!(
        !out.status.success(),
        "safe-mode B1/B2 must reject mut+borrow aliasing in one call"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("aliasing violation"),
        "expected aliasing diagnostic, got:\n{stdout}"
    );

    let method_move_src = r#"
impl Buffer {
  fn consume(own self) -> Int {
    0
  }
}

impl Socket {
  fn consume(borrow self) -> Int {
    0
  }
}

fn caller(own b: Buffer) -> Int {
  b.consume()
  b.consume()
  0
}
"#;
    let method_move_path = temp_source("borrow_method_use_after_move", method_move_src);
    assert_ok(&["parse"], &method_move_path);
    let out = run(&["check"], &method_move_path);
    assert!(
        !out.status.success(),
        "safe-mode B4 must reject use-after-move through own method receivers"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("use-after-move"),
        "expected method receiver use-after-move diagnostic, got:\n{stdout}"
    );

    let managed_arc_src = r#"
fn consume(own x: Buffer) -> Int {
  0
}

def caller(b) {
  consume(b)
  consume(b)
  0
}
"#;
    let managed_arc_path = temp_source("managed_arc_not_affine", managed_arc_src);
    assert_ok(&["parse"], &managed_arc_path);
    assert_ok(&["check"], &managed_arc_path);

    let place_aliasing_src = r#"
struct Pair {
  left: Buffer,
  right: Buffer,
}

fn frob(mut a: Buffer, borrow b: Buffer) -> Int {
  0
}

fn caller(mut p: Pair) -> Int {
  frob(p.left, p.left)
}
"#;
    let place_aliasing_path = temp_source("borrow_place_aliasing", place_aliasing_src);
    assert_ok(&["parse"], &place_aliasing_path);
    let out = run(&["check"], &place_aliasing_path);
    assert!(
        !out.status.success(),
        "safe-mode B1/B2 must reject mut+borrow aliasing of the same field projection"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("aliasing violation"),
        "expected field-place aliasing diagnostic, got:\n{stdout}"
    );

    let sibling_ok_src = r#"
struct Pair {
  left: Buffer,
  right: Buffer,
}

fn consume(own x: Buffer) -> Int {
  0
}

fn read(borrow x: Buffer) -> Int {
  0
}

fn caller(mut p: Pair) -> Int {
  consume(p.left)
  read(p.right)
  0
}
"#;
    let sibling_ok_path = temp_source("borrow_moved_field_sibling_ok", sibling_ok_src);
    assert_ok(&["parse"], &sibling_ok_path);
    assert_ok(&["check"], &sibling_ok_path);

    let index_aliasing_src = r#"
fn frob(mut a: Buffer, borrow b: Buffer) -> Int {
  0
}

fn caller(mut items: Buffers) -> Int {
  frob(items[0], items[1])
}
"#;
    let index_aliasing_path = temp_source("borrow_index_aliasing", index_aliasing_src);
    assert_ok(&["parse"], &index_aliasing_path);
    let out = run(&["check"], &index_aliasing_path);
    assert!(
        !out.status.success(),
        "safe-mode B1/B2 must conservatively reject mut+borrow aliasing across indexes of the same receiver"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("aliasing violation"),
        "expected index-place aliasing diagnostic, got:\n{stdout}"
    );
}

#[test]
#[ignore = "Mini-Spec §8.6 full place-granular B1-B5 borrow rules are partial in v0.4.2"]
fn deferred_full_borrow_rule_suite() {
    pending("full place-granular borrow-check B1-B5 conformance suite");
}

#[test]
fn deferred_trait_coherence() {
    let duplicate_impl_src = r#"
trait Renderable {
  def render() -> String
}

struct Widget {
  name: String,
}

impl Widget for Renderable {
  def render() -> String {
    "one"
  }
}

impl Widget for Renderable {
  def render() -> String {
    "two"
  }
}
"#;
    let duplicate_impl_path = temp_source("trait_duplicate_impl", duplicate_impl_src);
    assert_ok(&["parse"], &duplicate_impl_path);
    let out = run(&["check"], &duplicate_impl_path);
    assert!(
        !out.status.success(),
        "trait coherence must reject duplicate impls for the same trait/type pair"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("duplicate impl"),
        "expected duplicate impl diagnostic, got:\n{stdout}"
    );

    let orphan_impl_src = r#"
impl ExternalWidget for ExternalRenderable {
  def render() -> String {
    "external"
  }
}
"#;
    let orphan_impl_path = temp_source("trait_orphan_impl", orphan_impl_src);
    assert_ok(&["parse"], &orphan_impl_path);
    let out = run(&["check"], &orphan_impl_path);
    assert!(
        !out.status.success(),
        "trait coherence must reject impls where neither trait nor type is local"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("orphan rule"),
        "expected orphan-rule diagnostic, got:\n{stdout}"
    );

    let local_trait_src = r#"
trait LocalRenderable {
  def render() -> String
}

impl ExternalWidget for LocalRenderable {
  def render() -> String {
    "external"
  }
}
"#;
    let local_trait_path = temp_source("trait_local_trait_external_type", local_trait_src);
    assert_ok(&["parse"], &local_trait_path);
    assert_ok(&["check"], &local_trait_path);

    let local_type_src = r#"
struct LocalWidget {
  name: String,
}

impl LocalWidget for ExternalRenderable {
  def render() -> String {
    "local"
  }
}
"#;
    let local_type_path = temp_source("trait_external_trait_local_type", local_type_src);
    assert_ok(&["parse"], &local_type_path);
    assert_ok(&["check"], &local_type_path);
}

#[test]
fn generic_instantiation_runs_without_monomorphization_claims() {
    let src = r#"
struct Box<T> {
  value: T,
}

impl<T> Box<T> {
  def get(receiver) {
    receiver.value
  }
}

def identity<T>(value: T) {
  value
}

@caps()
def main() {
  let n: Box<Int> = Box(40)
  let s: Box<String> = Box("gem")
  identity(n.get()) + s.get().len()
}
"#;
    let path = temp_source("generic_instantiation_runs", src);
    assert_ok(&["parse"], &path);
    assert_ok(&["check"], &path);
    let out = run(&["run"], &path);
    assert!(
        out.status.success(),
        "generic struct/function/impl instantiation should run as interpreter evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("=> 43"),
        "expected generic runtime evidence to return 43, got:\n{stdout}"
    );
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
fn dynamic_impl_dispatch_tables() {
    let src = r#"
protocol Renderable {
  def render() -> String
}

@dynamic
struct TraitWidget {
  name: String
}

@dynamic
impl TraitWidget for Renderable {
  def render(receiver) -> String {
    receiver.name + "|trait"
  }
}

@caps()
def accept_renderable(item: Renderable) -> String {
  item.render()
}

@caps()
def main() {
  let widget = TraitWidget("panel")
  let knows = if widget.responds_to(:render) { "yes" } else { "no" }
  let before = accept_renderable(widget)
  widget.def_method(:render) do |receiver|
    receiver.name + "|instance"
  end
  let after = widget.render()
  knows + "|" + before + "|" + after
}
"#;
    let path = temp_source("dynamic_impl_dispatch_tables", src);
    assert_ok(&["parse"], &path);
    assert_ok(&["check"], &path);
    let out = run(&["run"], &path);
    assert!(
        out.status.success(),
        "dynamic impl table should participate in protocol satisfaction and dispatch\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("=> yes|panel|trait|panel|instance"),
        "expected dynamic impl dispatch before per-instance override, got:\n{stdout}"
    );

    let non_dynamic_trait_impl_src = r#"
protocol Renderable {
  def render() -> String
}

struct StaticTraitWidget {
  name: String
}

impl StaticTraitWidget for Renderable {
  def render(receiver) -> String {
    receiver.name
  }
}

@caps()
def accept_renderable(item: Renderable) -> String {
  item.render()
}

@caps()
def main() {
  accept_renderable(StaticTraitWidget("plain"))
}
"#;
    let non_dynamic_trait_impl_path = temp_source(
        "non_dynamic_trait_impl_stays_deferred",
        non_dynamic_trait_impl_src,
    );
    assert_ok(&["parse"], &non_dynamic_trait_impl_path);
    assert_ok(&["check"], &non_dynamic_trait_impl_path);
    let non_dynamic_trait_impl_out = run(&["run"], &non_dynamic_trait_impl_path);
    assert!(
        !non_dynamic_trait_impl_out.status.success(),
        "plain trait impl coherence remains deferred; only @dynamic impl is active in Phase 2H"
    );
    let stderr = String::from_utf8_lossy(&non_dynamic_trait_impl_out.stderr);
    assert!(
        stderr.contains("does not satisfy protocol Renderable"),
        "expected non-dynamic trait impl to remain deferred, got:\n{stderr}"
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

    let cast_ok_src = r#"
protocol CastRenderable {
  def render() -> String
}

struct CastWidget {
  name: String
}

impl CastWidget {
  def render(receiver) -> String {
    receiver.name
  }
}

@caps()
def main() {
  let widget = CastWidget("cast-panel")
  let renderable = widget as CastRenderable
  renderable.render()
}
"#;
    let cast_ok_path = temp_source("structural_protocol_cast_ok", cast_ok_src);
    assert_ok(&["parse"], &cast_ok_path);
    assert_ok(&["check"], &cast_ok_path);
    let cast_ok_out = run(&["run"], &cast_ok_path);
    assert!(
        cast_ok_out.status.success(),
        "protocol cast should accept a structurally compatible value\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&cast_ok_out.stdout),
        String::from_utf8_lossy(&cast_ok_out.stderr)
    );
    let stdout = String::from_utf8_lossy(&cast_ok_out.stdout);
    assert!(
        stdout.contains("=> cast-panel"),
        "expected successful protocol cast to preserve the receiver, got:\n{stdout}"
    );

    let cast_missing_src = r#"
protocol CastRenderable {
  def render() -> String
}

struct CastPlain {
  name: String
}

@caps()
def main() {
  let plain = CastPlain("box")
  plain as CastRenderable
}
"#;
    let cast_missing_path = temp_source("structural_protocol_cast_missing", cast_missing_src);
    assert_ok(&["parse"], &cast_missing_path);
    assert_ok(&["check"], &cast_missing_path);
    let cast_missing_out = run(&["run"], &cast_missing_path);
    assert!(
        !cast_missing_out.status.success(),
        "protocol cast must reject a structurally incompatible value"
    );
    let stderr = String::from_utf8_lossy(&cast_missing_out.stderr);
    assert!(
        stderr.contains("does not satisfy protocol CastRenderable"),
        "expected structural protocol cast diagnostic, got:\n{stderr}"
    );

    let cast_non_protocol_src = r#"
@caps()
def main() {
  1 as Int
}
"#;
    let cast_non_protocol_path = temp_source(
        "structural_protocol_cast_non_protocol",
        cast_non_protocol_src,
    );
    assert_ok(&["parse"], &cast_non_protocol_path);
    assert_ok(&["check"], &cast_non_protocol_path);
    let cast_non_protocol_out = run(&["run"], &cast_non_protocol_path);
    assert!(
        !cast_non_protocol_out.status.success(),
        "protocol cast must reject a non-protocol target"
    );
    let stderr = String::from_utf8_lossy(&cast_non_protocol_out.stderr);
    assert!(
        stderr.contains("cast target Int is not a protocol"),
        "expected non-protocol cast diagnostic, got:\n{stderr}"
    );

    let generic_protocol_ok_src = r#"
protocol BoxLike<T> {
  def value() -> T
}

struct TextBox {
  text: String
}

impl TextBox {
  def value(receiver) -> String {
    receiver.text
  }
}

@caps()
def unwrap_text(box: BoxLike<String>) -> String {
  box.value()
}

@caps()
def main() {
  let box = TextBox("generic-panel")
  let box_like = box as BoxLike<String>
  unwrap_text(box_like)
}
"#;
    let generic_protocol_ok_path =
        temp_source("structural_protocol_generic_ok", generic_protocol_ok_src);
    assert_ok(&["parse"], &generic_protocol_ok_path);
    assert_ok(&["check"], &generic_protocol_ok_path);
    let generic_protocol_ok_out = run(&["run"], &generic_protocol_ok_path);
    assert!(
        generic_protocol_ok_out.status.success(),
        "generic protocol type arguments should substitute into required method signatures\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&generic_protocol_ok_out.stdout),
        String::from_utf8_lossy(&generic_protocol_ok_out.stderr)
    );
    let stdout = String::from_utf8_lossy(&generic_protocol_ok_out.stdout);
    assert!(
        stdout.contains("=> generic-panel"),
        "expected generic protocol cast and parameter binding to preserve TextBox value, got:\n{stdout}"
    );

    let generic_protocol_bad_src = r#"
protocol BoxLike<T> {
  def value() -> T
}

struct IntBox {
  value: Int
}

impl IntBox {
  def value(receiver) -> Int {
    receiver.value
  }
}

@caps()
def unwrap_text(box: BoxLike<String>) -> String {
  box.value()
}

@caps()
def main() {
  unwrap_text(IntBox(7))
}
"#;
    let generic_protocol_bad_path =
        temp_source("structural_protocol_generic_bad", generic_protocol_bad_src);
    assert_ok(&["parse"], &generic_protocol_bad_path);
    assert_ok(&["check"], &generic_protocol_bad_path);
    let generic_protocol_bad_out = run(&["run"], &generic_protocol_bad_path);
    assert!(
        !generic_protocol_bad_out.status.success(),
        "generic protocol substitution must reject an incompatible concrete method return type"
    );
    let stderr = String::from_utf8_lossy(&generic_protocol_bad_out.stderr);
    assert!(
        stderr.contains("does not satisfy protocol BoxLike"),
        "expected generic protocol mismatch diagnostic, got:\n{stderr}"
    );

    let typed_builtin_protocol_ok_src = r#"
protocol TypedTextShape {
  def len() -> Int
  def upcase() -> String
  def starts_with(prefix: String) -> Bool
}

@caps()
def accept_text(value: TypedTextShape) -> Int {
  value.len()
}

@caps()
def main() {
  accept_text("garnet")
}
"#;
    let typed_builtin_protocol_ok_path = temp_source(
        "structural_protocol_typed_builtin_ok",
        typed_builtin_protocol_ok_src,
    );
    assert_ok(&["parse"], &typed_builtin_protocol_ok_path);
    assert_ok(&["check"], &typed_builtin_protocol_ok_path);
    let typed_builtin_protocol_ok_out = run(&["run"], &typed_builtin_protocol_ok_path);
    assert!(
        typed_builtin_protocol_ok_out.status.success(),
        "typed built-in String method signatures should satisfy compatible protocols\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&typed_builtin_protocol_ok_out.stdout),
        String::from_utf8_lossy(&typed_builtin_protocol_ok_out.stderr)
    );
    let stdout = String::from_utf8_lossy(&typed_builtin_protocol_ok_out.stdout);
    assert!(
        stdout.contains("=> 6"),
        "expected typed built-in protocol call to return String length, got:\n{stdout}"
    );

    let typed_builtin_protocol_bad_src = r#"
protocol BadTextShape {
  def len() -> String
}

@caps()
def accept_bad(value: BadTextShape) -> String {
  value.len()
}

@caps()
def main() {
  accept_bad("garnet")
}
"#;
    let typed_builtin_protocol_bad_path = temp_source(
        "structural_protocol_typed_builtin_bad",
        typed_builtin_protocol_bad_src,
    );
    assert_ok(&["parse"], &typed_builtin_protocol_bad_path);
    assert_ok(&["check"], &typed_builtin_protocol_bad_path);
    let typed_builtin_protocol_bad_out = run(&["run"], &typed_builtin_protocol_bad_path);
    assert!(
        !typed_builtin_protocol_bad_out.status.success(),
        "typed built-in protocol signatures must reject incompatible return types"
    );
    let stderr = String::from_utf8_lossy(&typed_builtin_protocol_bad_out.stderr);
    assert!(
        stderr.contains("does not satisfy protocol BadTextShape"),
        "expected typed built-in signature diagnostic, got:\n{stderr}"
    );
}
