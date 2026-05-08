//! Observable cycle-collection fixtures for the Memory Core reference path.

use garnet_memory::{CycleGraph, CycleNodeId, CycleScan, MemoryKind};

fn labels(graph: &CycleGraph, ids: &[CycleNodeId]) -> Vec<String> {
    let mut labels: Vec<_> = ids
        .iter()
        .map(|id| graph.label(*id).expect("node label").to_string())
        .collect();
    labels.sort();
    labels
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
    assert!(!graph.contains(root));
    assert!(!graph.contains(child));
}
