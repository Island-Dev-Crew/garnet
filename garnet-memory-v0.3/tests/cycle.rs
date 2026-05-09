//! Observable cycle-collection fixtures for the Memory Core reference path.

use garnet_memory::{
    CycleAllocatorFixture, CycleGraph, CycleNodeId, CycleRootBuffer, CycleScan, MemoryKind,
};

fn labels(graph: &CycleGraph, ids: &[CycleNodeId]) -> Vec<String> {
    let mut labels: Vec<_> = ids
        .iter()
        .map(|id| graph.label(*id).expect("node label").to_string())
        .collect();
    labels.sort();
    labels
}

fn labels_in_order(graph: &CycleGraph, ids: &[CycleNodeId]) -> Vec<String> {
    ids.iter()
        .map(|id| graph.label(*id).expect("node label").to_string())
        .collect()
}

#[test]
fn trial_deletion_collects_unrooted_cycle_and_retains_roots() {
    let mut graph = CycleGraph::new();
    let root = graph.add_node(MemoryKind::Working, "root");
    let child = graph.add_node(MemoryKind::Working, "child");
    let cycle_a = graph.add_node(MemoryKind::Working, "cycle_a");
    let cycle_b = graph.add_node(MemoryKind::Working, "cycle_b");

    graph.add_root(root).unwrap();
    graph.add_edge(root, child).unwrap();
    graph.add_edge(cycle_a, cycle_b).unwrap();
    graph.add_edge(cycle_b, cycle_a).unwrap();

    let report = graph.collect_cycles(CycleScan::All);

    assert_eq!(
        labels(&graph, &report.collected),
        vec!["cycle_a", "cycle_b"]
    );
    assert!(graph.contains(root));
    assert!(graph.contains(child));
    assert!(!graph.contains(cycle_a));
    assert!(!graph.contains(cycle_b));
}

#[test]
fn trial_deletion_leaves_unrooted_acyclic_nodes_for_later_eviction() {
    let mut graph = CycleGraph::new();
    let start = graph.add_node(MemoryKind::Episodic, "start");
    let leaf = graph.add_node(MemoryKind::Episodic, "leaf");

    graph.add_edge(start, leaf).unwrap();

    let report = graph.collect_cycles(CycleScan::All);

    assert!(report.collected.is_empty());
    assert_eq!(labels(&graph, &report.trial_candidates), vec!["leaf"]);
    assert_eq!(labels(&graph, &report.trial_retained), vec!["leaf"]);
    assert!(graph.contains(start));
    assert!(graph.contains(leaf));
}

#[test]
fn trial_deletion_collects_self_cycles() {
    let mut graph = CycleGraph::new();
    let node = graph.add_node(MemoryKind::Semantic, "self_cycle");

    graph.add_edge(node, node).unwrap();

    let report = graph.collect_cycles(CycleScan::All);

    assert_eq!(labels(&graph, &report.collected), vec!["self_cycle"]);
    assert!(!graph.contains(node));
}

#[test]
fn kind_partition_scan_collects_cross_kind_cycle_when_seed_kind_matches() {
    let mut graph = CycleGraph::new();
    let working = graph.add_node(MemoryKind::Working, "working_half");
    let episodic = graph.add_node(MemoryKind::Episodic, "episodic_half");
    let semantic = graph.add_node(MemoryKind::Semantic, "semantic_cycle");

    graph.add_edge(working, episodic).unwrap();
    graph.add_edge(episodic, working).unwrap();
    graph.add_edge(semantic, semantic).unwrap();

    let report = graph.collect_cycles(CycleScan::Kind(MemoryKind::Working));

    assert_eq!(
        labels(&graph, &report.collected),
        vec!["episodic_half", "working_half"]
    );
    assert_eq!(
        labels(&graph, &report.trial_candidates),
        vec!["working_half"]
    );
    assert!(!graph.contains(working));
    assert!(!graph.contains(episodic));
    assert!(graph.contains(semantic));

    let second_report = graph.collect_cycles(CycleScan::Kind(MemoryKind::Semantic));
    assert_eq!(
        labels(&graph, &second_report.collected),
        vec!["semantic_cycle"]
    );
}

#[test]
fn rooted_cross_kind_cycle_is_retained() {
    let mut graph = CycleGraph::new();
    let working = graph.add_node(MemoryKind::Working, "rooted_working");
    let semantic = graph.add_node(MemoryKind::Semantic, "rooted_semantic");

    graph.add_root(semantic).unwrap();
    graph.add_edge(working, semantic).unwrap();
    graph.add_edge(semantic, working).unwrap();

    let report = graph.collect_cycles(CycleScan::Kind(MemoryKind::Working));

    assert!(report.collected.is_empty());
    assert!(graph.contains(working));
    assert!(graph.contains(semantic));
}

#[test]
fn releasing_the_last_root_makes_cycle_collectable() {
    let mut graph = CycleGraph::new();
    let root = graph.add_node(MemoryKind::Working, "root");
    let child = graph.add_node(MemoryKind::Working, "child");

    graph.add_root(root).unwrap();
    graph.add_edge(root, child).unwrap();
    graph.add_edge(child, root).unwrap();

    let retained = graph.collect_cycles(CycleScan::All);
    assert!(retained.collected.is_empty());
    assert_eq!(labels(&graph, &retained.retained_roots), vec!["root"]);

    graph.release_root(root).unwrap();
    let collected = graph.collect_cycles(CycleScan::All);

    assert_eq!(labels(&graph, &collected.collected), vec!["child", "root"]);
    assert_eq!(
        labels_in_order(&graph, &collected.finalization_order),
        vec!["child", "root"]
    );
    assert!(!graph.contains(root));
    assert!(!graph.contains(child));
}

#[test]
fn finalization_order_follows_collect_white_postorder() {
    let mut graph = CycleGraph::new();
    let parent = graph.add_node(MemoryKind::Working, "parent");
    let child = graph.add_node(MemoryKind::Working, "child");

    graph.add_edge(parent, child).unwrap();
    graph.add_edge(child, parent).unwrap();

    let report = graph.collect_cycles(CycleScan::All);

    assert_eq!(
        labels_in_order(&graph, &report.finalization_order),
        vec!["child", "parent"]
    );
}

#[test]
fn safe_mode_allocations_are_not_cycle_collection_candidates() {
    let mut graph = CycleGraph::new();
    let safe = graph.add_safe_node(MemoryKind::Working, "safe_affine");

    graph.add_edge(safe, safe).unwrap();

    let report = graph.collect_cycles(CycleScan::All);

    assert!(report.trial_candidates.is_empty());
    assert!(report.collected.is_empty());
    assert!(report.finalization_order.is_empty());
    assert!(graph.contains(safe));
}

#[test]
fn root_buffer_release_collects_cycle_when_threshold_is_reached() {
    let mut graph = CycleGraph::new();
    let root = graph.add_node(MemoryKind::Working, "root");
    let child = graph.add_node(MemoryKind::Working, "child");
    let mut buffer = CycleRootBuffer::with_threshold(CycleScan::All, 1);

    graph.add_root(root).unwrap();
    graph.add_edge(root, child).unwrap();
    graph.add_edge(child, root).unwrap();

    let report = graph
        .release_root_to_buffer(root, &mut buffer)
        .unwrap()
        .expect("threshold should collect immediately");

    assert!(buffer.is_empty());
    assert_eq!(labels(&graph, &report.trial_candidates), vec!["root"]);
    assert_eq!(labels(&graph, &report.collected), vec!["child", "root"]);
    assert_eq!(
        labels_in_order(&graph, &report.finalization_order),
        vec!["child", "root"]
    );
}

#[test]
fn buffered_collection_scans_only_buffered_roots() {
    let mut graph = CycleGraph::new();
    let root = graph.add_node(MemoryKind::Working, "root");
    let child = graph.add_node(MemoryKind::Working, "child");
    let unrelated = graph.add_node(MemoryKind::Semantic, "unrelated");
    let mut buffer = CycleRootBuffer::with_threshold(CycleScan::All, 4);

    graph.add_root(root).unwrap();
    graph.add_edge(root, child).unwrap();
    graph.add_edge(child, root).unwrap();
    graph.add_edge(unrelated, unrelated).unwrap();

    let early = graph.release_root_to_buffer(root, &mut buffer).unwrap();
    assert!(early.is_none());
    assert_eq!(buffer.len(), 1);

    let report = graph.collect_buffered_cycles(&mut buffer);

    assert!(buffer.is_empty());
    assert_eq!(labels(&graph, &report.trial_candidates), vec!["root"]);
    assert_eq!(labels(&graph, &report.collected), vec!["child", "root"]);
    assert!(graph.contains(unrelated));
}

#[test]
fn allocator_owned_root_buffer_collects_after_root_release() {
    let mut allocator = CycleAllocatorFixture::with_threshold(CycleScan::All, 1);
    let root = allocator.allocate_arc(MemoryKind::Working, "root");
    let child = allocator.allocate_arc(MemoryKind::Working, "child");

    allocator.add_root(root).unwrap();
    allocator.add_edge(root, child).unwrap();
    allocator.add_edge(child, root).unwrap();

    let report = allocator
        .release_root(root)
        .unwrap()
        .expect("threshold should collect immediately");

    assert!(allocator.buffered_roots().is_empty());
    assert_eq!(
        labels(allocator.graph(), &report.trial_candidates),
        vec!["root"]
    );
    assert_eq!(
        labels(allocator.graph(), &report.collected),
        vec!["child", "root"]
    );
    assert!(!allocator.contains(root));
    assert!(!allocator.contains(child));
}

#[test]
fn allocator_edge_decrement_buffers_newly_unreachable_cycle() {
    let mut allocator = CycleAllocatorFixture::with_threshold(CycleScan::All, 1);
    let root = allocator.allocate_arc(MemoryKind::Working, "root");
    let cycle_a = allocator.allocate_arc(MemoryKind::Working, "cycle_a");
    let cycle_b = allocator.allocate_arc(MemoryKind::Working, "cycle_b");

    allocator.add_root(root).unwrap();
    allocator.add_edge(root, cycle_a).unwrap();
    allocator.add_edge(cycle_a, cycle_b).unwrap();
    allocator.add_edge(cycle_b, cycle_a).unwrap();

    let report = allocator
        .remove_edge(root, cycle_a)
        .unwrap()
        .expect("edge decrement should collect newly unreachable cycle");

    assert_eq!(
        labels(allocator.graph(), &report.trial_candidates),
        vec!["cycle_a"]
    );
    assert_eq!(
        labels(allocator.graph(), &report.collected),
        vec!["cycle_a", "cycle_b"]
    );
    assert!(allocator.contains(root));
    assert!(!allocator.contains(cycle_a));
    assert!(!allocator.contains(cycle_b));
}

#[test]
fn allocator_missing_edge_removal_does_not_schedule_cycle() {
    let mut allocator = CycleAllocatorFixture::with_threshold(CycleScan::All, 1);
    let root = allocator.allocate_arc(MemoryKind::Working, "root");
    let cycle_a = allocator.allocate_arc(MemoryKind::Working, "cycle_a");
    let cycle_b = allocator.allocate_arc(MemoryKind::Working, "cycle_b");

    allocator.add_root(root).unwrap();
    allocator.add_edge(cycle_a, cycle_b).unwrap();
    allocator.add_edge(cycle_b, cycle_a).unwrap();

    let report = allocator.remove_edge(root, cycle_a).unwrap();

    assert!(report.is_none());
    assert_eq!(allocator.buffer_len(), 0);
    assert!(allocator.contains(root));
    assert!(allocator.contains(cycle_a));
    assert!(allocator.contains(cycle_b));
}
