//! Mini-Spec conformance skeleton.
//!
//! Active tests cover rows from the v0.4.2 conformance matrix that are
//! implemented today. Ignored tests name partial/deferred rows so language
//! completeness work has stable test handles before the implementation lands.

use garnet_memory::{CycleGraph, CycleRootBuffer, CycleScan, MemoryKind};
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
        .current_dir(path.parent().unwrap_or_else(|| Path::new(".")))
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
    let safe_affine = graph.add_safe_node(MemoryKind::Working, "safe_affine");

    graph.add_root(rooted).unwrap();
    graph.add_edge(rooted, reachable).unwrap();
    graph.add_edge(cycle_a, cycle_b).unwrap();
    graph.add_edge(cycle_b, cycle_a).unwrap();
    graph.add_edge(safe_affine, safe_affine).unwrap();

    let report = graph.collect_cycles(CycleScan::Kind(MemoryKind::Working));

    assert_eq!(report.trial_candidates, vec![cycle_a]);
    assert!(report.trial_retained.is_empty());
    assert_eq!(report.finalization_order, vec![cycle_b, cycle_a]);
    assert_eq!(report.collected, vec![cycle_a, cycle_b]);
    assert!(graph.contains(rooted));
    assert!(graph.contains(reachable));
    assert!(graph.contains(safe_affine));
    assert!(!graph.contains(cycle_a));
    assert!(!graph.contains(cycle_b));

    let mut buffered = CycleGraph::new();
    let buffered_root = buffered.add_node(MemoryKind::Working, "buffered_root");
    let buffered_child = buffered.add_node(MemoryKind::Working, "buffered_child");
    let mut roots = CycleRootBuffer::with_threshold(CycleScan::Kind(MemoryKind::Working), 1);

    buffered.add_root(buffered_root).unwrap();
    buffered.add_edge(buffered_root, buffered_child).unwrap();
    buffered.add_edge(buffered_child, buffered_root).unwrap();

    let buffered_report = buffered
        .release_root_to_buffer(buffered_root, &mut roots)
        .unwrap()
        .expect("threshold should collect immediately");

    assert!(roots.is_empty());
    assert_eq!(buffered_report.trial_candidates, vec![buffered_root]);
    assert_eq!(
        buffered_report.collected,
        vec![buffered_root, buffered_child]
    );
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
fn deferred_full_borrow_rule_suite() {
    let double_own_src = r#"
fn consume_pair(own left: Buffer, own right: Buffer) -> Int {
  0
}

fn caller(own b: Buffer) -> Int {
  consume_pair(b, b)
}
"#;
    let double_own_path = temp_source("borrow_drop_double_own", double_own_src);
    assert_ok(&["parse"], &double_own_path);
    let out = run(&["check"], &double_own_path);
    assert!(
        !out.status.success(),
        "safe-mode B5 must reject double-own of the same binding in one call"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("drop discipline"),
        "expected drop-discipline diagnostic, got:\n{stdout}"
    );

    let returning_branch_src = r#"
fn consume(own x: Buffer) -> Int {
  0
}

fn read(borrow x: Buffer) -> Int {
  0
}

fn caller(own b: Buffer, c: Bool) -> Int {
  if c {
    consume(b)
    return 0
  } else {
    0
  }
  read(b)
  0
}
"#;
    let returning_branch_path =
        temp_source("borrow_returning_branch_liveness", returning_branch_src);
    assert_ok(&["parse"], &returning_branch_path);
    assert_ok(&["check"], &returning_branch_path);

    let returning_loop_src = r#"
fn consume(own x: Buffer) -> Int {
  0
}

fn read(borrow x: Buffer) -> Int {
  0
}

fn caller(own b: Buffer, c: Bool) -> Int {
  while c {
    consume(b)
    return 0
  }
  read(b)
  0
}
"#;
    let returning_loop_path = temp_source("borrow_returning_loop_liveness", returning_loop_src);
    assert_ok(&["parse"], &returning_loop_path);
    assert_ok(&["check"], &returning_loop_path);

    let returning_for_src = r#"
fn consume(own x: Buffer) -> Int {
  0
}

fn read(borrow x: Buffer) -> Int {
  0
}

fn caller(own b: Buffer, xs: Array<Int>) -> Int {
  for x in xs {
    consume(b)
    return 0
  }
  read(b)
  0
}
"#;
    let returning_for_path = temp_source("borrow_returning_for_liveness", returning_for_src);
    assert_ok(&["parse"], &returning_for_path);
    assert_ok(&["check"], &returning_for_path);

    let for_shadow_src = r#"
fn consume(own x: Buffer) -> Int {
  0
}

fn read(borrow x: Buffer) -> Int {
  0
}

fn caller(own item: Buffer, xs: Array<Int>) -> Int {
  consume(item)
  for item in xs {
    0
  }
  read(item)
  0
}
"#;
    let for_shadow_path = temp_source(
        "borrow_for_loop_shadow_preserves_outer_move",
        for_shadow_src,
    );
    assert_ok(&["parse"], &for_shadow_path);
    let out = run(&["check"], &for_shadow_path);
    assert!(
        !out.status.success(),
        "safe-mode for-loop variables must not rebind a moved outer binding"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("use-after-move"),
        "expected use-after-move diagnostic, got:\n{stdout}"
    );

    let match_shadow_src = r#"
fn consume(own x: Buffer) -> Int {
  0
}

fn read(borrow x: Buffer) -> Int {
  0
}

fn caller(borrow item: Buffer, own subject: Buffer) -> Int {
  match subject {
    item => consume(item)
  }
  read(item)
  0
}
"#;
    let match_shadow_path = temp_source(
        "borrow_match_pattern_shadow_does_not_poison_outer",
        match_shadow_src,
    );
    assert_ok(&["parse"], &match_shadow_path);
    assert_ok(&["check"], &match_shadow_path);

    let match_outer_move_src = r#"
fn consume(own x: Buffer) -> Int {
  0
}

fn read(borrow x: Buffer) -> Int {
  0
}

fn caller(own item: Buffer, n: Int) -> Int {
  match n {
    _ => consume(item)
  }
  read(item)
  0
}
"#;
    let match_outer_move_path = temp_source(
        "borrow_match_arm_outer_move_propagates",
        match_outer_move_src,
    );
    assert_ok(&["parse"], &match_outer_move_path);
    let out = run(&["check"], &match_outer_move_path);
    assert!(
        !out.status.success(),
        "safe-mode match arms must still propagate real outer moves"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("use-after-move"),
        "expected use-after-move diagnostic, got:\n{stdout}"
    );

    let match_block_move_src = r#"
fn consume(own x: Buffer) -> Int {
  0
}

fn read(borrow x: Buffer) -> Int {
  0
}

fn caller(own item: Buffer, n: Int) -> Int {
  match n {
    _ => {
      consume(item)
      0
    }
  }
  read(item)
  0
}
"#;
    let match_block_move_path = temp_source(
        "borrow_match_arm_block_statement_move_propagates",
        match_block_move_src,
    );
    assert_ok(&["parse"], &match_block_move_path);
    let out = run(&["check"], &match_block_move_path);
    assert!(
        !out.status.success(),
        "safe-mode match-arm block statements must be preserved for later move diagnostics"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("use-after-move"),
        "expected use-after-move diagnostic, got:\n{stdout}"
    );

    let match_guard_move_src = r#"
fn consumes_false(own x: Buffer) -> Bool {
  false
}

fn read(borrow x: Buffer) -> Int {
  0
}

fn caller(own item: Buffer, n: Int) -> Int {
  match n {
    _ if consumes_false(item) => {
      return 0
    },
    _ => 0
  }
  read(item)
  0
}
"#;
    let match_guard_move_path = temp_source(
        "borrow_match_guard_move_propagates_after_false_guard",
        match_guard_move_src,
    );
    assert_ok(&["parse"], &match_guard_move_path);
    let out = run(&["check"], &match_guard_move_path);
    assert!(
        !out.status.success(),
        "safe-mode match guards must merge moves that can continue when the guard is false"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("use-after-move"),
        "expected use-after-move diagnostic, got:\n{stdout}"
    );
}

#[test]
fn deferred_match_exhaustiveness_and_reachability() {
    let bool_non_exhaustive_src = r#"
fn bool_code(flag: Bool) -> Int {
  match flag {
    true => 1
  }
}
"#;
    let bool_non_exhaustive_path =
        temp_source("match_bool_non_exhaustive", bool_non_exhaustive_src);
    assert_ok(&["parse"], &bool_non_exhaustive_path);
    let out = run(&["check"], &bool_non_exhaustive_path);
    assert!(
        !out.status.success(),
        "safe-mode Bool matches must reject missing finite-domain cases"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("non-exhaustive match") && stdout.contains("`false`"),
        "expected missing Bool pattern diagnostic, got:\n{stdout}"
    );

    let bool_initializer_src = r#"
fn bool_code() -> Int {
  let flag = true
  match flag {
    true => 1
  }
}
"#;
    let bool_initializer_path = temp_source(
        "match_bool_initializer_non_exhaustive",
        bool_initializer_src,
    );
    assert_ok(&["parse"], &bool_initializer_path);
    let out = run(&["check"], &bool_initializer_path);
    assert!(
        !out.status.success(),
        "safe-mode Bool initializer matches must reject missing finite-domain cases"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("non-exhaustive match") && stdout.contains("`false`"),
        "expected initializer-driven Bool missing pattern diagnostic, got:\n{stdout}"
    );

    let mutable_bool_assignment_src = r#"
fn bool_code() -> Int {
  let mut flag = 1
  flag = true
  match flag {
    true => 1
  }
}
"#;
    let mutable_bool_assignment_path = temp_source(
        "match_mutable_bool_assignment_non_exhaustive",
        mutable_bool_assignment_src,
    );
    assert_ok(&["parse"], &mutable_bool_assignment_path);
    let out = run(&["check"], &mutable_bool_assignment_path);
    assert!(
        !out.status.success(),
        "safe-mode mutable Bool finite assignments must reject missing finite-domain cases"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("non-exhaustive match") && stdout.contains("`false`"),
        "expected mutable assignment-driven Bool missing pattern diagnostic, got:\n{stdout}"
    );

    let mutable_invalidation_src = r#"
fn bool_code() -> Int {
  let mut flag = true
  flag = 1
  match flag {
    true => 1
  }
}
"#;
    let mutable_invalidation_path = temp_source(
        "match_mutable_assignment_invalidation_open",
        mutable_invalidation_src,
    );
    assert_ok(&["parse"], &mutable_invalidation_path);
    assert_ok(&["check"], &mutable_invalidation_path);

    let if_else_mutable_bool_src = r#"
fn bool_code(cond: Bool) -> Int {
  let mut flag = 1
  if cond {
    flag = true
  } else {
    flag = false
  }
  match flag {
    true => 1
  }
}
"#;
    let if_else_mutable_bool_path = temp_source(
        "match_if_else_mutable_bool_assignment_non_exhaustive",
        if_else_mutable_bool_src,
    );
    assert_ok(&["parse"], &if_else_mutable_bool_path);
    let out = run(&["check"], &if_else_mutable_bool_path);
    assert!(
        !out.status.success(),
        "safe-mode if/else mutable Bool assignments must reject missing finite-domain cases"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("non-exhaustive match") && stdout.contains("`false`"),
        "expected if/else mutable Bool assignment diagnostic, got:\n{stdout}"
    );

    let if_else_mixed_invalidation_src = r#"
fn bool_code(cond: Bool) -> Int {
  let mut flag = true
  if cond {
    flag = false
  } else {
    flag = 1
  }
  match flag {
    true => 1
  }
}
"#;
    let if_else_mixed_invalidation_path = temp_source(
        "match_if_else_mixed_assignment_invalidation_open",
        if_else_mixed_invalidation_src,
    );
    assert_ok(&["parse"], &if_else_mixed_invalidation_path);
    assert_ok(&["check"], &if_else_mixed_invalidation_path);

    let nested_if_assignment_src = r#"
fn bool_code(first: Bool, second: Bool) -> Int {
  let mut flag = 1
  if first {
    if second {
      flag = true
    } else {
      flag = false
    }
  } else {
    flag = true
  }
  match flag {
    true => 1
  }
}
"#;
    let nested_if_assignment_path = temp_source(
        "match_nested_if_assignment_non_exhaustive",
        nested_if_assignment_src,
    );
    assert_ok(&["parse"], &nested_if_assignment_path);
    let out = run(&["check"], &nested_if_assignment_path);
    assert!(
        !out.status.success(),
        "safe-mode nested if assignments must reject missing finite-domain cases when every nested branch assigns"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("non-exhaustive match") && stdout.contains("`false`"),
        "expected nested if assignment diagnostic, got:\n{stdout}"
    );

    let nested_if_missing_else_src = r#"
fn bool_code(first: Bool, second: Bool) -> Int {
  let mut flag = 1
  if first {
    if second {
      flag = true
    }
  } else {
    flag = false
  }
  match flag {
    true => 1
  }
}
"#;
    let nested_if_missing_else_path = temp_source(
        "match_nested_if_missing_else_invalidation_open",
        nested_if_missing_else_src,
    );
    assert_ok(&["parse"], &nested_if_missing_else_path);
    assert_ok(&["check"], &nested_if_missing_else_path);

    let compound_invalidation_src = r#"
fn bool_code() -> Int {
  let mut flag = true
  flag += 1
  match flag {
    true => 1
  }
}
"#;
    let compound_invalidation_path = temp_source(
        "match_compound_assignment_invalidation_open",
        compound_invalidation_src,
    );
    assert_ok(&["parse"], &compound_invalidation_path);
    assert_ok(&["check"], &compound_invalidation_path);

    let if_else_compound_invalidation_src = r#"
fn bool_code(cond: Bool) -> Int {
  let mut flag = true
  if cond {
    flag += 1
  } else {
    flag += 1
  }
  match flag {
    true => 1
  }
}
"#;
    let if_else_compound_invalidation_path = temp_source(
        "match_if_else_compound_assignment_invalidation_open",
        if_else_compound_invalidation_src,
    );
    assert_ok(&["parse"], &if_else_compound_invalidation_path);
    assert_ok(&["check"], &if_else_compound_invalidation_path);

    let while_assignment_invalidation_src = r#"
fn bool_code(cond: Bool) -> Int {
  let mut flag = true
  while cond {
    flag = 1
  }
  match flag {
    true => 1
  }
}
"#;
    let while_assignment_invalidation_path = temp_source(
        "match_while_assignment_invalidation_open",
        while_assignment_invalidation_src,
    );
    assert_ok(&["parse"], &while_assignment_invalidation_path);
    assert_ok(&["check"], &while_assignment_invalidation_path);

    let for_assignment_invalidation_src = r#"
fn bool_code() -> Int {
  let mut flag = true
  for item in [1] {
    flag = item
  }
  match flag {
    true => 1
  }
}
"#;
    let for_assignment_invalidation_path = temp_source(
        "match_for_assignment_invalidation_open",
        for_assignment_invalidation_src,
    );
    assert_ok(&["parse"], &for_assignment_invalidation_path);
    assert_ok(&["check"], &for_assignment_invalidation_path);

    let try_body_assignment_invalidation_src = r#"
fn bool_code() -> Int {
  let mut flag = true
  try {
    flag = 1
  } rescue e {
    0
  }
  match flag {
    true => 1
  }
}
"#;
    let try_body_assignment_invalidation_path = temp_source(
        "match_try_body_assignment_invalidation_no_stale_match_diag",
        try_body_assignment_invalidation_src,
    );
    assert_ok(&["parse"], &try_body_assignment_invalidation_path);
    let out = run(&["check"], &try_body_assignment_invalidation_path);
    assert!(
        !out.status.success(),
        "safe-mode try/rescue should remain rejected"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("uses `try`/`rescue`"),
        "expected safe-mode try/rescue diagnostic, got:\n{stdout}"
    );
    assert!(
        !stdout.contains("non-exhaustive match"),
        "try assignment invalidation must not leave a stale match coverage diagnostic, got:\n{stdout}"
    );

    let closure_assignment_boundary_src = r#"
fn bool_code(cond: Bool) -> Int {
  let mut flag = 1
  let updater = |value| if cond {
    flag = true
  } else {
    flag = false
  }
  match flag {
    true => 1
  }
}
"#;
    let closure_assignment_boundary_path = temp_source(
        "match_uninvoked_closure_assignment_boundary_open",
        closure_assignment_boundary_src,
    );
    assert_ok(&["parse"], &closure_assignment_boundary_path);
    assert_ok(&["check"], &closure_assignment_boundary_path);

    let immediate_closure_assignment_invalidation_src = r#"
fn bool_code() -> Int {
  let mut flag = true
  (|value| {
    flag = value
  })(1)
  match flag {
    true => 1
  }
}
"#;
    let immediate_closure_assignment_invalidation_path = temp_source(
        "match_immediate_closure_assignment_invalidation_open",
        immediate_closure_assignment_invalidation_src,
    );
    assert_ok(&["parse"], &immediate_closure_assignment_invalidation_path);
    assert_ok(&["check"], &immediate_closure_assignment_invalidation_path);

    let local_closure_assignment_invalidation_src = r#"
fn bool_code() -> Int {
  let mut flag = true
  let updater = |value| {
    flag = value
  }
  updater(1)
  match flag {
    true => 1
  }
}
"#;
    let local_closure_assignment_invalidation_path = temp_source(
        "match_local_closure_assignment_invalidation_open",
        local_closure_assignment_invalidation_src,
    );
    assert_ok(&["parse"], &local_closure_assignment_invalidation_path);
    assert_ok(&["check"], &local_closure_assignment_invalidation_path);

    let branch_joined_closure_assignment_invalidation_src = r#"
fn bool_code(cond: Bool) -> Int {
  let mut flag = true
  let updater = if cond {
    |value| {
      flag = value
    }
  } else {
    |value| {
      flag = false
    }
  }
  updater(1)
  match flag {
    true => 1
  }
}
"#;
    let branch_joined_closure_assignment_invalidation_path = temp_source(
        "match_branch_joined_closure_assignment_invalidation_open",
        branch_joined_closure_assignment_invalidation_src,
    );
    assert_ok(
        &["parse"],
        &branch_joined_closure_assignment_invalidation_path,
    );
    assert_ok(
        &["check"],
        &branch_joined_closure_assignment_invalidation_path,
    );

    let branch_rebound_closure_assignment_invalidation_src = r#"
fn bool_code(cond: Bool) -> Int {
  let mut flag = true
  let mut updater = |value| {
    value
  }
  if cond {
    updater = |value| {
      flag = value
    }
  } else {
    updater = |value| {
      flag = false
    }
  }
  updater(1)
  match flag {
    true => 1
  }
}
"#;
    let branch_rebound_closure_assignment_invalidation_path = temp_source(
        "match_branch_rebound_closure_assignment_invalidation_open",
        branch_rebound_closure_assignment_invalidation_src,
    );
    assert_ok(
        &["parse"],
        &branch_rebound_closure_assignment_invalidation_path,
    );
    assert_ok(
        &["check"],
        &branch_rebound_closure_assignment_invalidation_path,
    );

    let local_closure_alias_assignment_invalidation_src = r#"
fn bool_code() -> Int {
  let mut flag = true
  let updater = |value| {
    flag = value
  }
  let alias = updater
  alias(1)
  match flag {
    true => 1
  }
}
"#;
    let local_closure_alias_assignment_invalidation_path = temp_source(
        "match_local_closure_alias_assignment_invalidation_open",
        local_closure_alias_assignment_invalidation_src,
    );
    assert_ok(
        &["parse"],
        &local_closure_alias_assignment_invalidation_path,
    );
    assert_ok(
        &["check"],
        &local_closure_alias_assignment_invalidation_path,
    );

    let branch_joined_closure_alias_assignment_invalidation_src = r#"
fn bool_code(cond: Bool) -> Int {
  let mut flag = true
  let update_from_arg = |value| {
    flag = value
  }
  let update_to_false = |value| {
    flag = false
  }
  let alias = if cond {
    update_from_arg
  } else {
    update_to_false
  }
  alias(1)
  match flag {
    true => 1
  }
}
"#;
    let branch_joined_closure_alias_assignment_invalidation_path = temp_source(
        "match_branch_joined_closure_alias_assignment_invalidation_open",
        branch_joined_closure_alias_assignment_invalidation_src,
    );
    assert_ok(
        &["parse"],
        &branch_joined_closure_alias_assignment_invalidation_path,
    );
    assert_ok(
        &["check"],
        &branch_joined_closure_alias_assignment_invalidation_path,
    );

    let direct_branch_selected_closure_call_invalidation_src = r#"
fn bool_code(cond: Bool) -> Int {
  let mut flag = true
  let update_from_arg = |value| {
    flag = value
  }
  let update_to_false = |value| {
    flag = false
  }
  (if cond {
    update_from_arg
  } else {
    update_to_false
  })(1)
  match flag {
    true => 1
  }
}
"#;
    let direct_branch_selected_closure_call_invalidation_path = temp_source(
        "match_direct_branch_selected_closure_call_invalidation_open",
        direct_branch_selected_closure_call_invalidation_src,
    );
    assert_ok(
        &["parse"],
        &direct_branch_selected_closure_call_invalidation_path,
    );
    assert_ok(
        &["check"],
        &direct_branch_selected_closure_call_invalidation_path,
    );

    let enum_complete_src = r#"
enum Status { Ready, Done }

fn status_code(status: Status) -> Int {
  match status {
    Status::Ready => 1
    Status::Done => 2
  }
}
"#;
    let enum_complete_path = temp_source("match_enum_complete", enum_complete_src);
    assert_ok(&["parse"], &enum_complete_path);
    assert_ok(&["check"], &enum_complete_path);

    let enum_unreachable_src = r#"
enum Status { Ready, Done }

fn status_code(status: Status) -> Int {
  match status {
    Status::Ready => 1
    Status::Ready => 2
    Status::Done => 3
  }
}
"#;
    let enum_unreachable_path =
        temp_source("match_enum_duplicate_unreachable", enum_unreachable_src);
    assert_ok(&["parse"], &enum_unreachable_path);
    let out = run(&["check"], &enum_unreachable_path);
    assert!(
        !out.status.success(),
        "safe-mode enum matches must reject arms already covered by prior unguarded arms"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("unreachable match arm") && stdout.contains("Status::Ready"),
        "expected unreachable duplicate variant diagnostic, got:\n{stdout}"
    );

    let enum_initializer_missing_src = r#"
enum Status { Ready, Done }

fn status_code() -> Int {
  let status = Status::Ready()
  match status {
    Status::Ready => 1
  }
}
"#;
    let enum_initializer_missing_path = temp_source(
        "match_enum_initializer_missing",
        enum_initializer_missing_src,
    );
    assert_ok(&["parse"], &enum_initializer_missing_path);
    let out = run(&["check"], &enum_initializer_missing_path);
    assert!(
        !out.status.success(),
        "safe-mode enum initializer matches must reject missing finite-domain cases"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("non-exhaustive match") && stdout.contains("Status::Done"),
        "expected initializer-driven enum missing variant diagnostic, got:\n{stdout}"
    );

    let mutable_enum_initializer_missing_src = r#"
enum Status { Ready, Done }

fn status_code() -> Int {
  let mut status = Status::Ready()
  match status {
    Status::Ready => 1
  }
}
"#;
    let mutable_enum_initializer_missing_path = temp_source(
        "match_mutable_enum_initializer_missing",
        mutable_enum_initializer_missing_src,
    );
    assert_ok(&["parse"], &mutable_enum_initializer_missing_path);
    let out = run(&["check"], &mutable_enum_initializer_missing_path);
    assert!(
        !out.status.success(),
        "safe-mode mutable enum initializers must reject missing finite-domain cases"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("non-exhaustive match") && stdout.contains("Status::Done"),
        "expected mutable enum initializer missing variant diagnostic, got:\n{stdout}"
    );

    let if_else_enum_assignment_src = r#"
enum Status { Ready, Done }

fn status_code(cond: Bool) -> Int {
  let mut status = 1
  if cond {
    status = Status::Ready()
  } else {
    status = Status::Done()
  }
  match status {
    Status::Ready => 1
  }
}
"#;
    let if_else_enum_assignment_path = temp_source(
        "match_if_else_enum_assignment_missing",
        if_else_enum_assignment_src,
    );
    assert_ok(&["parse"], &if_else_enum_assignment_path);
    let out = run(&["check"], &if_else_enum_assignment_path);
    assert!(
        !out.status.success(),
        "safe-mode if/else mutable enum assignments must reject missing finite-domain cases"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("non-exhaustive match") && stdout.contains("Status::Done"),
        "expected if/else mutable enum assignment missing variant diagnostic, got:\n{stdout}"
    );

    let nested_missing_src = r#"
enum Inner { Left, Right }
enum Outer { Wrap(Inner), Empty }

fn nested_code(outer: Outer) -> Int {
  match outer {
    Outer::Wrap(Inner::Left) => 1
    Outer::Empty => 0
  }
}
"#;
    let nested_missing_path = temp_source("match_nested_enum_payload_missing", nested_missing_src);
    assert_ok(&["parse"], &nested_missing_path);
    let out = run(&["check"], &nested_missing_path);
    assert!(
        !out.status.success(),
        "safe-mode nested enum payload matches must reject missing finite payload cases"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("non-exhaustive match") && stdout.contains("Outer::Wrap(Inner::Right)"),
        "expected missing nested payload diagnostic, got:\n{stdout}"
    );

    let nested_complete_src = r#"
enum Inner { Left, Right }
enum Outer { Wrap(Inner), Empty }

fn nested_code(outer: Outer) -> Int {
  match outer {
    Outer::Wrap(Inner::Left) => 1
    Outer::Wrap(Inner::Right) => 2
    Outer::Empty => 0
  }
}
"#;
    let nested_complete_path =
        temp_source("match_nested_enum_payload_complete", nested_complete_src);
    assert_ok(&["parse"], &nested_complete_path);
    assert_ok(&["check"], &nested_complete_path);

    let imported_complete_src = r#"
module Types {
  enum Status { Ready, Done }
}

use Types::{Status}

fn imported_status_code(status: Status) -> Int {
  match status {
    Status::Ready => 1
    Status::Done => 2
  }
}
"#;
    let imported_complete_path =
        temp_source("match_imported_enum_named_complete", imported_complete_src);
    assert_ok(&["parse"], &imported_complete_path);
    assert_ok(&["check"], &imported_complete_path);

    let imported_missing_src = r#"
module Types {
  enum Status { Ready, Done }
}

use Types::{Status}

fn imported_status_code(status: Status) -> Int {
  match status {
    Status::Ready => 1
  }
}
"#;
    let imported_missing_path =
        temp_source("match_imported_enum_named_missing", imported_missing_src);
    assert_ok(&["parse"], &imported_missing_path);
    let out = run(&["check"], &imported_missing_path);
    assert!(
        !out.status.success(),
        "safe-mode imported enum matches must reject missing imported cases"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("non-exhaustive match") && stdout.contains("Types::Status::Done"),
        "expected missing imported enum diagnostic, got:\n{stdout}"
    );

    let true_guard_complete_src = r#"
enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if true => 1
    Status::Done => 2
  }
}
"#;
    let true_guard_complete_path =
        temp_source("match_true_guard_enum_complete", true_guard_complete_src);
    assert_ok(&["parse"], &true_guard_complete_path);
    assert_ok(&["check"], &true_guard_complete_path);

    let local_true_guard_complete_src = r#"
enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  let always = true
  match status {
    Status::Ready if always => 1
    Status::Done => 2
  }
}
"#;
    let local_true_guard_complete_path = temp_source(
        "match_local_true_guard_enum_complete",
        local_true_guard_complete_src,
    );
    assert_ok(&["parse"], &local_true_guard_complete_path);
    assert_ok(&["check"], &local_true_guard_complete_path);

    let const_true_guard_complete_src = r#"
const ALWAYS = true
enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if ALWAYS => 1
    Status::Done => 2
  }
}
"#;
    let const_true_guard_complete_path = temp_source(
        "match_const_true_guard_enum_complete",
        const_true_guard_complete_src,
    );
    assert_ok(&["parse"], &const_true_guard_complete_path);
    assert_ok(&["check"], &const_true_guard_complete_path);

    let imported_const_true_guard_complete_src = r#"
module Flags {
  const ALWAYS = true
}
use Flags::{ALWAYS}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if ALWAYS => 1
    Status::Done => 2
  }
}
"#;
    let imported_const_true_guard_complete_path = temp_source(
        "match_imported_const_true_guard_enum_complete",
        imported_const_true_guard_complete_src,
    );
    assert_ok(&["parse"], &imported_const_true_guard_complete_path);
    assert_ok(&["check"], &imported_const_true_guard_complete_path);

    let module_relative_imported_const_true_guard_complete_src = r#"
module Outer {
  module Flags {
    const ALWAYS = true
  }
  use Flags::{ALWAYS}

  enum Status { Ready, Done }

  fn guarded_status_code(status: Status) -> Int {
    match status {
      Status::Ready if ALWAYS => 1
      Status::Done => 2
    }
  }
}
"#;
    let module_relative_imported_const_true_guard_complete_path = temp_source(
        "match_module_relative_imported_const_true_guard_enum_complete",
        module_relative_imported_const_true_guard_complete_src,
    );
    assert_ok(
        &["parse"],
        &module_relative_imported_const_true_guard_complete_path,
    );
    assert_ok(
        &["check"],
        &module_relative_imported_const_true_guard_complete_path,
    );

    let mutable_guard_missing_src = r#"
enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  let mut always = true
  match status {
    Status::Ready if always => 1
    Status::Done => 2
  }
}
"#;
    let mutable_guard_missing_path = temp_source(
        "match_mutable_guard_enum_missing",
        mutable_guard_missing_src,
    );
    assert_ok(&["parse"], &mutable_guard_missing_path);
    let out = run(&["check"], &mutable_guard_missing_path);
    assert!(
        !out.status.success(),
        "safe-mode mutable boolean guards must remain unknown"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("non-exhaustive match") && stdout.contains("Status::Ready"),
        "expected mutable guard to stay non-covering, got:\n{stdout}"
    );

    let shadowed_const_guard_missing_src = r#"
const ALWAYS = true
enum Status { Ready, Done }

fn guarded_status_code(status: Status, ALWAYS: Bool) -> Int {
  match status {
    Status::Ready if ALWAYS => 1
    Status::Done => 2
  }
}
"#;
    let shadowed_const_guard_missing_path = temp_source(
        "match_const_guard_shadowed_by_param_missing",
        shadowed_const_guard_missing_src,
    );
    assert_ok(&["parse"], &shadowed_const_guard_missing_path);
    let out = run(&["check"], &shadowed_const_guard_missing_path);
    assert!(
        !out.status.success(),
        "safe-mode parameter shadowing must keep same-name const boolean guards unknown"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("non-exhaustive match") && stdout.contains("Status::Ready"),
        "expected parameter-shadowed const guard to stay non-covering, got:\n{stdout}"
    );

    let shadowed_imported_const_guard_missing_src = r#"
module Flags {
  const ALWAYS = true
}
use Flags::{ALWAYS}

enum Status { Ready, Done }

fn guarded_status_code(status: Status, ALWAYS: Bool) -> Int {
  match status {
    Status::Ready if ALWAYS => 1
    Status::Done => 2
  }
}
"#;
    let shadowed_imported_const_guard_missing_path = temp_source(
        "match_imported_const_guard_shadowed_by_param_missing",
        shadowed_imported_const_guard_missing_src,
    );
    assert_ok(&["parse"], &shadowed_imported_const_guard_missing_path);
    let out = run(&["check"], &shadowed_imported_const_guard_missing_path);
    assert!(
        !out.status.success(),
        "safe-mode parameter shadowing must keep same-name imported const boolean guards unknown"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("non-exhaustive match") && stdout.contains("Status::Ready"),
        "expected parameter-shadowed imported const guard to stay non-covering, got:\n{stdout}"
    );

    let false_guard_missing_src = r#"
enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if false => 1
    Status::Done => 2
  }
}
"#;
    let false_guard_missing_path =
        temp_source("match_false_guard_enum_missing", false_guard_missing_src);
    assert_ok(&["parse"], &false_guard_missing_path);
    let out = run(&["check"], &false_guard_missing_path);
    assert!(
        !out.status.success(),
        "safe-mode false literal guards must be unreachable and non-covering"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("statically false guard")
            && stdout.contains("non-exhaustive match")
            && stdout.contains("Status::Ready"),
        "expected false-guard and missing-case diagnostics, got:\n{stdout}"
    );

    let local_false_guard_missing_src = r#"
enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  let never = false
  match status {
    Status::Ready if never => 1
    Status::Done => 2
  }
}
"#;
    let local_false_guard_missing_path = temp_source(
        "match_local_false_guard_enum_missing",
        local_false_guard_missing_src,
    );
    assert_ok(&["parse"], &local_false_guard_missing_path);
    let out = run(&["check"], &local_false_guard_missing_path);
    assert!(
        !out.status.success(),
        "safe-mode local false guards must be unreachable and non-covering"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("statically false guard")
            && stdout.contains("non-exhaustive match")
            && stdout.contains("Status::Ready"),
        "expected local false-guard and missing-case diagnostics, got:\n{stdout}"
    );

    let const_false_guard_missing_src = r#"
const NEVER = false
enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if NEVER => 1
    Status::Done => 2
  }
}
"#;
    let const_false_guard_missing_path = temp_source(
        "match_const_false_guard_enum_missing",
        const_false_guard_missing_src,
    );
    assert_ok(&["parse"], &const_false_guard_missing_path);
    let out = run(&["check"], &const_false_guard_missing_path);
    assert!(
        !out.status.success(),
        "safe-mode const false guards must be unreachable and non-covering"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("statically false guard")
            && stdout.contains("non-exhaustive match")
            && stdout.contains("Status::Ready"),
        "expected const false-guard and missing-case diagnostics, got:\n{stdout}"
    );

    let imported_const_false_guard_missing_src = r#"
module Flags {
  const NEVER = false
}
use Flags::*

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if NEVER => 1
    Status::Done => 2
  }
}
"#;
    let imported_const_false_guard_missing_path = temp_source(
        "match_imported_const_false_guard_enum_missing",
        imported_const_false_guard_missing_src,
    );
    assert_ok(&["parse"], &imported_const_false_guard_missing_path);
    let out = run(&["check"], &imported_const_false_guard_missing_path);
    assert!(
        !out.status.success(),
        "safe-mode imported const false guards must be unreachable and non-covering"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("statically false guard")
            && stdout.contains("non-exhaustive match")
            && stdout.contains("Status::Ready"),
        "expected imported const false-guard and missing-case diagnostics, got:\n{stdout}"
    );

    let path_const_true_guard_complete_src = r#"
module Flags {
  const ALWAYS = true
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Flags::ALWAYS => 1
    Status::Done => 2
  }
}
"#;
    let path_const_true_guard_complete_path = temp_source(
        "match_path_const_true_guard_enum_complete",
        path_const_true_guard_complete_src,
    );
    assert_ok(&["parse"], &path_const_true_guard_complete_path);
    assert_ok(&["check"], &path_const_true_guard_complete_path);

    let path_const_false_guard_missing_src = r#"
module Flags {
  const NEVER = false
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Flags::NEVER => 1
    Status::Done => 2
  }
}
"#;
    let path_const_false_guard_missing_path = temp_source(
        "match_path_const_false_guard_enum_missing",
        path_const_false_guard_missing_src,
    );
    assert_ok(&["parse"], &path_const_false_guard_missing_path);
    let out = run(&["check"], &path_const_false_guard_missing_path);
    assert!(
        !out.status.success(),
        "safe-mode path-qualified const false guards must be unreachable and non-covering"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("statically false guard")
            && stdout.contains("non-exhaustive match")
            && stdout.contains("Status::Ready"),
        "expected path const false-guard and missing-case diagnostics, got:\n{stdout}"
    );

    let path_const_alias_true_guard_complete_src = r#"
module Core {
  const RAW = true
}

module Flags {
  const ALWAYS = Core::RAW
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Flags::ALWAYS => 1
    Status::Done => 2
  }
}
"#;
    let path_const_alias_true_guard_complete_path = temp_source(
        "match_path_const_alias_true_guard_enum_complete",
        path_const_alias_true_guard_complete_src,
    );
    assert_ok(&["parse"], &path_const_alias_true_guard_complete_path);
    assert_ok(&["check"], &path_const_alias_true_guard_complete_path);

    let path_const_alias_false_guard_missing_src = r#"
module Core {
  const RAW = false
}

module Flags {
  const NEVER = Core::RAW
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Flags::NEVER => 1
    Status::Done => 2
  }
}
"#;
    let path_const_alias_false_guard_missing_path = temp_source(
        "match_path_const_alias_false_guard_enum_missing",
        path_const_alias_false_guard_missing_src,
    );
    assert_ok(&["parse"], &path_const_alias_false_guard_missing_path);
    let out = run(&["check"], &path_const_alias_false_guard_missing_path);
    assert!(
        !out.status.success(),
        "safe-mode path-qualified const false aliases must be unreachable and non-covering"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("statically false guard")
            && stdout.contains("non-exhaustive match")
            && stdout.contains("Status::Ready"),
        "expected path const false-alias and missing-case diagnostics, got:\n{stdout}"
    );

    let boolean_const_expr_true_guard_complete_src = r#"
module Core {
  const RAW = true
}

module Flags {
  const ALWAYS = Core::RAW and not false
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Flags::ALWAYS => 1
    Status::Done => 2
  }
}
"#;
    let boolean_const_expr_true_guard_complete_path = temp_source(
        "match_boolean_const_expr_true_guard_enum_complete",
        boolean_const_expr_true_guard_complete_src,
    );
    assert_ok(&["parse"], &boolean_const_expr_true_guard_complete_path);
    assert_ok(&["check"], &boolean_const_expr_true_guard_complete_path);

    let boolean_const_expr_false_guard_missing_src = r#"
module Core {
  const RAW = true
}

module Flags {
  const NEVER = not Core::RAW or false
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Flags::NEVER => 1
    Status::Done => 2
  }
}
"#;
    let boolean_const_expr_false_guard_missing_path = temp_source(
        "match_boolean_const_expr_false_guard_enum_missing",
        boolean_const_expr_false_guard_missing_src,
    );
    assert_ok(&["parse"], &boolean_const_expr_false_guard_missing_path);
    let out = run(&["check"], &boolean_const_expr_false_guard_missing_path);
    assert!(
        !out.status.success(),
        "safe-mode false boolean const expressions must be unreachable and non-covering"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("statically false guard")
            && stdout.contains("non-exhaustive match")
            && stdout.contains("Status::Ready"),
        "expected boolean const expression false-guard and missing-case diagnostics, got:\n{stdout}"
    );

    let short_circuit_or_true_guard_complete_src = r#"
module Flags {
  const ALWAYS = true or Missing::VALUE
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Flags::ALWAYS => 1
    Status::Done => 2
  }
}
"#;
    let short_circuit_or_true_guard_complete_path = temp_source(
        "match_short_circuit_or_true_guard_enum_complete",
        short_circuit_or_true_guard_complete_src,
    );
    assert_ok(&["parse"], &short_circuit_or_true_guard_complete_path);
    assert_ok(&["check"], &short_circuit_or_true_guard_complete_path);

    let short_circuit_and_false_guard_missing_src = r#"
module Flags {
  const NEVER = false and Missing::VALUE
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Flags::NEVER => 1
    Status::Done => 2
  }
}
"#;
    let short_circuit_and_false_guard_missing_path = temp_source(
        "match_short_circuit_and_false_guard_enum_missing",
        short_circuit_and_false_guard_missing_src,
    );
    assert_ok(&["parse"], &short_circuit_and_false_guard_missing_path);
    let out = run(&["check"], &short_circuit_and_false_guard_missing_path);
    assert!(
        !out.status.success(),
        "safe-mode short-circuit false boolean const expressions must be unreachable and non-covering"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("statically false guard")
            && stdout.contains("non-exhaustive match")
            && stdout.contains("Status::Ready"),
        "expected short-circuit boolean const expression false-guard and missing-case diagnostics, got:\n{stdout}"
    );

    let bool_equality_true_guard_complete_src = r#"
module Core {
  const RAW = true
}

module Flags {
  const ALWAYS = Core::RAW == true
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Flags::ALWAYS => 1
    Status::Done => 2
  }
}
"#;
    let bool_equality_true_guard_complete_path = temp_source(
        "match_boolean_const_equality_true_guard_enum_complete",
        bool_equality_true_guard_complete_src,
    );
    assert_ok(&["parse"], &bool_equality_true_guard_complete_path);
    assert_ok(&["check"], &bool_equality_true_guard_complete_path);

    let bool_inequality_false_guard_missing_src = r#"
module Core {
  const RAW = true
}

module Flags {
  const NEVER = Core::RAW != true
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Flags::NEVER => 1
    Status::Done => 2
  }
}
"#;
    let bool_inequality_false_guard_missing_path = temp_source(
        "match_boolean_const_inequality_false_guard_enum_missing",
        bool_inequality_false_guard_missing_src,
    );
    assert_ok(&["parse"], &bool_inequality_false_guard_missing_path);
    let out = run(&["check"], &bool_inequality_false_guard_missing_path);
    assert!(
        !out.status.success(),
        "safe-mode false boolean const inequalities must be unreachable and non-covering"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("statically false guard")
            && stdout.contains("non-exhaustive match")
            && stdout.contains("Status::Ready"),
        "expected boolean const inequality false-guard and missing-case diagnostics, got:\n{stdout}"
    );

    let direct_bool_equality_true_guard_complete_src = r#"
module Core {
  const RAW = true
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Core::RAW == true => 1
    Status::Done => 2
  }
}
"#;
    let direct_bool_equality_true_guard_complete_path = temp_source(
        "match_direct_boolean_const_equality_true_guard_enum_complete",
        direct_bool_equality_true_guard_complete_src,
    );
    assert_ok(&["parse"], &direct_bool_equality_true_guard_complete_path);
    assert_ok(&["check"], &direct_bool_equality_true_guard_complete_path);

    let direct_bool_inequality_false_guard_missing_src = r#"
module Core {
  const RAW = true
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Core::RAW != true => 1
    Status::Done => 2
  }
}
"#;
    let direct_bool_inequality_false_guard_missing_path = temp_source(
        "match_direct_boolean_const_inequality_false_guard_enum_missing",
        direct_bool_inequality_false_guard_missing_src,
    );
    assert_ok(&["parse"], &direct_bool_inequality_false_guard_missing_path);
    let out = run(&["check"], &direct_bool_inequality_false_guard_missing_path);
    assert!(
        !out.status.success(),
        "safe-mode direct false boolean const inequalities must be unreachable and non-covering"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("statically false guard")
            && stdout.contains("non-exhaustive match")
            && stdout.contains("Status::Ready"),
        "expected direct boolean const inequality false-guard and missing-case diagnostics, got:\n{stdout}"
    );

    let int_equality_true_guard_complete_src = r#"
module Core {
  const LIMIT = 2
}

module Flags {
  const ALWAYS = Core::LIMIT == 2
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Flags::ALWAYS => 1
    Status::Done => 2
  }
}
"#;
    let int_equality_true_guard_complete_path = temp_source(
        "match_integer_const_equality_true_guard_enum_complete",
        int_equality_true_guard_complete_src,
    );
    assert_ok(&["parse"], &int_equality_true_guard_complete_path);
    assert_ok(&["check"], &int_equality_true_guard_complete_path);

    let int_inequality_false_guard_missing_src = r#"
module Core {
  const LIMIT = 2
}

module Flags {
  const NEVER = Core::LIMIT != 2
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Flags::NEVER => 1
    Status::Done => 2
  }
}
"#;
    let int_inequality_false_guard_missing_path = temp_source(
        "match_integer_const_inequality_false_guard_enum_missing",
        int_inequality_false_guard_missing_src,
    );
    assert_ok(&["parse"], &int_inequality_false_guard_missing_path);
    let out = run(&["check"], &int_inequality_false_guard_missing_path);
    assert!(
        !out.status.success(),
        "safe-mode false integer const inequalities must be unreachable and non-covering"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("statically false guard")
            && stdout.contains("non-exhaustive match")
            && stdout.contains("Status::Ready"),
        "expected integer const inequality false-guard and missing-case diagnostics, got:\n{stdout}"
    );

    let direct_int_equality_true_guard_complete_src = r#"
module Core {
  const LIMIT = 2
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Core::LIMIT == 2 => 1
    Status::Done => 2
  }
}
"#;
    let direct_int_equality_true_guard_complete_path = temp_source(
        "match_direct_integer_const_equality_true_guard_enum_complete",
        direct_int_equality_true_guard_complete_src,
    );
    assert_ok(&["parse"], &direct_int_equality_true_guard_complete_path);
    assert_ok(&["check"], &direct_int_equality_true_guard_complete_path);

    let int_relational_true_guard_complete_src = r#"
module Core {
  const LIMIT = 2
}

module Flags {
  const ALWAYS = Core::LIMIT < 3
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Flags::ALWAYS => 1
    Status::Done => 2
  }
}
"#;
    let int_relational_true_guard_complete_path = temp_source(
        "match_integer_const_less_than_true_guard_enum_complete",
        int_relational_true_guard_complete_src,
    );
    assert_ok(&["parse"], &int_relational_true_guard_complete_path);
    assert_ok(&["check"], &int_relational_true_guard_complete_path);

    let int_relational_false_guard_missing_src = r#"
module Core {
  const LIMIT = 2
}

module Flags {
  const NEVER = Core::LIMIT > 3
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Flags::NEVER => 1
    Status::Done => 2
  }
}
"#;
    let int_relational_false_guard_missing_path = temp_source(
        "match_integer_const_greater_than_false_guard_enum_missing",
        int_relational_false_guard_missing_src,
    );
    assert_ok(&["parse"], &int_relational_false_guard_missing_path);
    let out = run(&["check"], &int_relational_false_guard_missing_path);
    assert!(
        !out.status.success(),
        "safe-mode false integer relational guards must be unreachable and non-covering"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("statically false guard")
            && stdout.contains("non-exhaustive match")
            && stdout.contains("Status::Ready"),
        "expected integer relational false-guard and missing-case diagnostics, got:\n{stdout}"
    );

    let direct_int_relational_true_guard_complete_src = r#"
module Core {
  const LIMIT = 2
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Core::LIMIT >= 2 => 1
    Status::Done => 2
  }
}
"#;
    let direct_int_relational_true_guard_complete_path = temp_source(
        "match_direct_integer_const_greater_equal_true_guard_enum_complete",
        direct_int_relational_true_guard_complete_src,
    );
    assert_ok(&["parse"], &direct_int_relational_true_guard_complete_path);
    assert_ok(&["check"], &direct_int_relational_true_guard_complete_path);

    let int_arithmetic_true_guard_complete_src = r#"
module Core {
  const LIMIT = 2
  const OFFSET = 1
}

module Flags {
  const ALWAYS = Core::LIMIT + Core::OFFSET == 3
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Flags::ALWAYS => 1
    Status::Done => 2
  }
}
"#;
    let int_arithmetic_true_guard_complete_path = temp_source(
        "match_integer_const_arithmetic_equality_true_guard_enum_complete",
        int_arithmetic_true_guard_complete_src,
    );
    assert_ok(&["parse"], &int_arithmetic_true_guard_complete_path);
    assert_ok(&["check"], &int_arithmetic_true_guard_complete_path);

    let int_arithmetic_false_guard_missing_src = r#"
module Core {
  const LIMIT = 2
}

module Flags {
  const NEVER = Core::LIMIT * 2 < 4
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Flags::NEVER => 1
    Status::Done => 2
  }
}
"#;
    let int_arithmetic_false_guard_missing_path = temp_source(
        "match_integer_const_arithmetic_relational_false_guard_enum_missing",
        int_arithmetic_false_guard_missing_src,
    );
    assert_ok(&["parse"], &int_arithmetic_false_guard_missing_path);
    let out = run(&["check"], &int_arithmetic_false_guard_missing_path);
    assert!(
        !out.status.success(),
        "safe-mode false integer arithmetic guards must be unreachable and non-covering"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("statically false guard")
            && stdout.contains("non-exhaustive match")
            && stdout.contains("Status::Ready"),
        "expected integer arithmetic false-guard and missing-case diagnostics, got:\n{stdout}"
    );

    let direct_int_arithmetic_true_guard_complete_src = r#"
module Core {
  const LIMIT = 2
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Core::LIMIT + 1 >= 3 => 1
    Status::Done => 2
  }
}
"#;
    let direct_int_arithmetic_true_guard_complete_path = temp_source(
        "match_direct_integer_const_arithmetic_relational_true_guard_enum_complete",
        direct_int_arithmetic_true_guard_complete_src,
    );
    assert_ok(&["parse"], &direct_int_arithmetic_true_guard_complete_path);
    assert_ok(&["check"], &direct_int_arithmetic_true_guard_complete_path);

    let same_module_int_identifier_guard_complete_src = r#"
const LIMIT = 2
const OFFSET = 1

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if LIMIT + OFFSET == 3 => 1
    Status::Done => 2
  }
}
"#;
    let same_module_int_identifier_guard_complete_path = temp_source(
        "match_same_module_integer_const_identifier_guard_enum_complete",
        same_module_int_identifier_guard_complete_src,
    );
    assert_ok(&["parse"], &same_module_int_identifier_guard_complete_path);
    assert_ok(&["check"], &same_module_int_identifier_guard_complete_path);

    let imported_named_int_identifier_guard_complete_src = r#"
module Core {
  const LIMIT = 2
  const OFFSET = 1
}
use Core::{LIMIT, OFFSET}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if LIMIT + OFFSET == 3 => 1
    Status::Done => 2
  }
}
"#;
    let imported_named_int_identifier_guard_complete_path = temp_source(
        "match_imported_named_integer_const_identifier_guard_enum_complete",
        imported_named_int_identifier_guard_complete_src,
    );
    assert_ok(
        &["parse"],
        &imported_named_int_identifier_guard_complete_path,
    );
    assert_ok(
        &["check"],
        &imported_named_int_identifier_guard_complete_path,
    );

    let imported_glob_int_identifier_false_guard_missing_src = r#"
module Core {
  const LIMIT = 2
}
use Core::*

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if LIMIT * 2 < 4 => 1
    Status::Done => 2
  }
}
"#;
    let imported_glob_int_identifier_false_guard_missing_path = temp_source(
        "match_imported_glob_integer_const_identifier_false_guard_enum_missing",
        imported_glob_int_identifier_false_guard_missing_src,
    );
    assert_ok(
        &["parse"],
        &imported_glob_int_identifier_false_guard_missing_path,
    );
    let out = run(
        &["check"],
        &imported_glob_int_identifier_false_guard_missing_path,
    );
    assert!(
        !out.status.success(),
        "safe-mode false imported integer identifier guards must be unreachable and non-covering"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("statically false guard")
            && stdout.contains("non-exhaustive match")
            && stdout.contains("Status::Ready"),
        "expected imported integer identifier false-guard and missing-case diagnostics, got:\n{stdout}"
    );

    let imported_int_identifier_shadowed_missing_src = r#"
module Core {
  const LIMIT = 2
  const OFFSET = 1
}
use Core::{LIMIT, OFFSET}

enum Status { Ready, Done }

fn guarded_status_code(status: Status, LIMIT: Int) -> Int {
  match status {
    Status::Ready if LIMIT + OFFSET == 3 => 1
    Status::Done => 2
  }
}
"#;
    let imported_int_identifier_shadowed_missing_path = temp_source(
        "match_imported_integer_const_identifier_shadowed_by_param_missing",
        imported_int_identifier_shadowed_missing_src,
    );
    assert_ok(&["parse"], &imported_int_identifier_shadowed_missing_path);
    let out = run(&["check"], &imported_int_identifier_shadowed_missing_path);
    assert!(
        !out.status.success(),
        "parameter-shadowed imported integer identifiers must remain unknown/non-covering"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("statically false guard")
            && stdout.contains("non-exhaustive match")
            && stdout.contains("Status::Ready"),
        "expected parameter-shadowed imported integer identifier to stay unknown, got:\n{stdout}"
    );

    let symbol_const_equality_true_guard_complete_src = r#"
module Core {
  const MODE = :ready
}

module Flags {
  const ALWAYS = Core::MODE == :ready
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Flags::ALWAYS => 1
    Status::Done => 2
  }
}
"#;
    let symbol_const_equality_true_guard_complete_path = temp_source(
        "match_symbol_const_equality_true_guard_enum_complete",
        symbol_const_equality_true_guard_complete_src,
    );
    assert_ok(&["parse"], &symbol_const_equality_true_guard_complete_path);
    assert_ok(&["check"], &symbol_const_equality_true_guard_complete_path);

    let string_const_inequality_false_guard_missing_src = r#"
module Core {
  const LABEL = "ready"
}

module Flags {
  const NEVER = Core::LABEL != "ready"
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Flags::NEVER => 1
    Status::Done => 2
  }
}
"#;
    let string_const_inequality_false_guard_missing_path = temp_source(
        "match_string_const_inequality_false_guard_enum_missing",
        string_const_inequality_false_guard_missing_src,
    );
    assert_ok(
        &["parse"],
        &string_const_inequality_false_guard_missing_path,
    );
    let out = run(
        &["check"],
        &string_const_inequality_false_guard_missing_path,
    );
    assert!(
        !out.status.success(),
        "safe-mode false string const inequality guards must be unreachable and non-covering"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("statically false guard")
            && stdout.contains("non-exhaustive match")
            && stdout.contains("Status::Ready"),
        "expected string const inequality false-guard and missing-case diagnostics, got:\n{stdout}"
    );

    let nil_const_equality_true_guard_complete_src = r#"
module Core {
  const EMPTY = nil
}

module Flags {
  const ALWAYS = Core::EMPTY == nil
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Flags::ALWAYS => 1
    Status::Done => 2
  }
}
"#;
    let nil_const_equality_true_guard_complete_path = temp_source(
        "match_nil_const_equality_true_guard_enum_complete",
        nil_const_equality_true_guard_complete_src,
    );
    assert_ok(&["parse"], &nil_const_equality_true_guard_complete_path);
    assert_ok(&["check"], &nil_const_equality_true_guard_complete_path);

    let nil_const_inequality_false_guard_missing_src = r#"
module Core {
  const EMPTY = nil
}

module Flags {
  const NEVER = Core::EMPTY != nil
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Flags::NEVER => 1
    Status::Done => 2
  }
}
"#;
    let nil_const_inequality_false_guard_missing_path = temp_source(
        "match_nil_const_inequality_false_guard_enum_missing",
        nil_const_inequality_false_guard_missing_src,
    );
    assert_ok(&["parse"], &nil_const_inequality_false_guard_missing_path);
    let out = run(&["check"], &nil_const_inequality_false_guard_missing_path);
    assert!(
        !out.status.success(),
        "safe-mode false nil const inequality guards must be unreachable and non-covering"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("statically false guard")
            && stdout.contains("non-exhaustive match")
            && stdout.contains("Status::Ready"),
        "expected nil const inequality false-guard and missing-case diagnostics, got:\n{stdout}"
    );

    let mixed_literal_const_inequality_true_guard_complete_src = r#"
module Core {
  const EMPTY = nil
}

module Flags {
  const ALWAYS = Core::EMPTY != false
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Flags::ALWAYS => 1
    Status::Done => 2
  }
}
"#;
    let mixed_literal_const_inequality_true_guard_complete_path = temp_source(
        "match_mixed_literal_const_inequality_true_guard_enum_complete",
        mixed_literal_const_inequality_true_guard_complete_src,
    );
    assert_ok(
        &["parse"],
        &mixed_literal_const_inequality_true_guard_complete_path,
    );
    assert_ok(
        &["check"],
        &mixed_literal_const_inequality_true_guard_complete_path,
    );

    let mixed_literal_const_equality_false_guard_missing_src = r#"
module Core {
  const EMPTY = nil
}

module Flags {
  const NEVER = Core::EMPTY == false
}

enum Status { Ready, Done }

fn guarded_status_code(status: Status) -> Int {
  match status {
    Status::Ready if Flags::NEVER => 1
    Status::Done => 2
  }
}
"#;
    let mixed_literal_const_equality_false_guard_missing_path = temp_source(
        "match_mixed_literal_const_equality_false_guard_enum_missing",
        mixed_literal_const_equality_false_guard_missing_src,
    );
    assert_ok(
        &["parse"],
        &mixed_literal_const_equality_false_guard_missing_path,
    );
    let out = run(
        &["check"],
        &mixed_literal_const_equality_false_guard_missing_path,
    );
    assert!(
        !out.status.success(),
        "safe-mode false mixed-literal const equality guards must be unreachable and non-covering"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("statically false guard")
            && stdout.contains("non-exhaustive match")
            && stdout.contains("Status::Ready"),
        "expected mixed-literal const equality false-guard and missing-case diagnostics, got:\n{stdout}"
    );

    let open_literal_duplicate_src = r#"
fn classify(value: Int) -> Int {
  match value {
    1 => 10
    1 => 11
    _ => 0
  }
}
"#;
    let open_literal_duplicate_path = temp_source(
        "match_open_literal_duplicate_unreachable",
        open_literal_duplicate_src,
    );
    assert_ok(&["parse"], &open_literal_duplicate_path);
    let out = run(&["check"], &open_literal_duplicate_path);
    assert!(
        !out.status.success(),
        "safe-mode open-domain matches must reject duplicate literal arms"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("unreachable match arm") && stdout.contains("`1`"),
        "expected duplicate open literal diagnostic, got:\n{stdout}"
    );

    let open_catch_all_src = r#"
fn classify(value: Int) -> Int {
  match value {
    _ => 0
    1 => 10
  }
}
"#;
    let open_catch_all_path = temp_source(
        "match_open_arm_after_catch_all_unreachable",
        open_catch_all_src,
    );
    assert_ok(&["parse"], &open_catch_all_path);
    let out = run(&["check"], &open_catch_all_path);
    assert!(
        !out.status.success(),
        "safe-mode open-domain matches must reject arms after catch-all arms"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("unreachable match arm") && stdout.contains("covered by prior catch-all"),
        "expected open-domain catch-all reachability diagnostic, got:\n{stdout}"
    );
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

    let generic_overlap_src = r#"
trait Renderable {
  def render() -> String
}

struct Box<T> {
  value: T,
}

impl<T> Box<T> for Renderable {
  def render() -> String {
    "generic"
  }
}

impl Box<String> for Renderable {
  def render() -> String {
    "string"
  }
}
"#;
    let generic_overlap_path = temp_source("trait_generic_overlap", generic_overlap_src);
    assert_ok(&["parse"], &generic_overlap_path);
    let out = run(&["check"], &generic_overlap_path);
    assert!(
        !out.status.success(),
        "trait coherence must reject generic blanket impls that overlap concrete impls"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("overlapping impl"),
        "expected overlapping impl diagnostic, got:\n{stdout}"
    );

    let qualified_external_src = r#"
struct Widget {
  name: String,
}

impl Remote::Widget for ExternalRenderable {
  def render() -> String {
    "remote"
  }
}
"#;
    let qualified_external_path = temp_source(
        "trait_qualified_external_short_name",
        qualified_external_src,
    );
    assert_ok(&["parse"], &qualified_external_path);
    let out = run(&["check"], &qualified_external_path);
    assert!(
        !out.status.success(),
        "trait coherence must not treat qualified external types as local by short-name collision"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("orphan rule"),
        "expected orphan-rule diagnostic for qualified external type, got:\n{stdout}"
    );
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
