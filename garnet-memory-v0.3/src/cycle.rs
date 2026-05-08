//! Bounded observable cycle-collection reference path.
//!
//! This module is not the production allocator-integrated ARC collector.
//! It gives Mnemos a deterministic graph model for Mini-Spec §4.5
//! fixtures: rooted nodes stay live, unrooted acyclic nodes remain available
//! for normal eviction, and unrooted cycles are collected with kind-aware
//! scan scheduling.

use crate::MemoryKind;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

/// Stable identifier for a node in a [`CycleGraph`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CycleNodeId(usize);

impl CycleNodeId {
    pub fn index(self) -> usize {
        self.0
    }
}

/// Which root partition should be scanned for collectable cycles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleScan {
    /// Scan every memory-kind partition.
    All,
    /// Scan components that include at least one node of the given kind.
    Kind(MemoryKind),
}

/// Result of a cycle-collection pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CycleCollectReport {
    pub scan: CycleScan,
    pub retained_roots: Vec<CycleNodeId>,
    pub retained: Vec<CycleNodeId>,
    pub collected: Vec<CycleNodeId>,
}

/// Errors returned by graph mutation helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CycleGraphError {
    MissingNode(CycleNodeId),
    RootUnderflow(CycleNodeId),
}

impl fmt::Display for CycleGraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingNode(id) => write!(f, "missing cycle graph node {}", id.index()),
            Self::RootUnderflow(id) => {
                write!(f, "cycle graph node {} has no root to release", id.index())
            }
        }
    }
}

impl std::error::Error for CycleGraphError {}

#[derive(Debug, Clone)]
struct CycleNode {
    kind: MemoryKind,
    label: String,
    roots: usize,
    edges: BTreeSet<CycleNodeId>,
    collected: bool,
}

/// Deterministic graph fixture for reference-counted Memory Core objects.
#[derive(Debug, Default, Clone)]
pub struct CycleGraph {
    nodes: Vec<CycleNode>,
}

impl CycleGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, kind: MemoryKind, label: impl Into<String>) -> CycleNodeId {
        let id = CycleNodeId(self.nodes.len());
        self.nodes.push(CycleNode {
            kind,
            label: label.into(),
            roots: 0,
            edges: BTreeSet::new(),
            collected: false,
        });
        id
    }

    pub fn contains(&self, id: CycleNodeId) -> bool {
        self.nodes
            .get(id.index())
            .map(|node| !node.collected)
            .unwrap_or(false)
    }

    pub fn label(&self, id: CycleNodeId) -> Option<&str> {
        self.nodes.get(id.index()).map(|node| node.label.as_str())
    }

    pub fn kind(&self, id: CycleNodeId) -> Option<MemoryKind> {
        self.nodes.get(id.index()).map(|node| node.kind)
    }

    pub fn add_root(&mut self, id: CycleNodeId) -> Result<(), CycleGraphError> {
        let node = self.node_mut(id)?;
        node.roots += 1;
        Ok(())
    }

    pub fn release_root(&mut self, id: CycleNodeId) -> Result<(), CycleGraphError> {
        let node = self.node_mut(id)?;
        if node.roots == 0 {
            return Err(CycleGraphError::RootUnderflow(id));
        }
        node.roots -= 1;
        Ok(())
    }

    pub fn add_edge(&mut self, from: CycleNodeId, to: CycleNodeId) -> Result<(), CycleGraphError> {
        self.ensure_active(to)?;
        self.node_mut(from)?.edges.insert(to);
        Ok(())
    }

    pub fn remove_edge(
        &mut self,
        from: CycleNodeId,
        to: CycleNodeId,
    ) -> Result<(), CycleGraphError> {
        self.node_mut(from)?.edges.remove(&to);
        Ok(())
    }

    /// Collect unrooted cyclic components that match the requested scan.
    ///
    /// The collector deliberately leaves unrooted acyclic components alone:
    /// those belong to ordinary retention/eviction policy, not the cycle
    /// detector. Cross-kind components are collected as a whole when any node
    /// in the component matches the requested scan kind.
    pub fn collect_cycles(&mut self, scan: CycleScan) -> CycleCollectReport {
        let live = self.live_from_roots();
        let mut collected_set = BTreeSet::new();

        for component in self.unrooted_components(&live) {
            if !self.is_cyclic_component(&component) || !self.component_matches(scan, &component) {
                continue;
            }
            collected_set.extend(component);
        }

        for node in &mut self.nodes {
            node.edges.retain(|child| !collected_set.contains(child));
        }

        for id in &collected_set {
            if let Some(node) = self.nodes.get_mut(id.index()) {
                node.collected = true;
                node.edges.clear();
            }
        }

        CycleCollectReport {
            scan,
            retained_roots: self.root_ids(),
            retained: self.active_ids(),
            collected: collected_set.into_iter().collect(),
        }
    }

    fn node_mut(&mut self, id: CycleNodeId) -> Result<&mut CycleNode, CycleGraphError> {
        self.ensure_active(id)?;
        Ok(&mut self.nodes[id.index()])
    }

    fn ensure_active(&self, id: CycleNodeId) -> Result<(), CycleGraphError> {
        if self.contains(id) {
            Ok(())
        } else {
            Err(CycleGraphError::MissingNode(id))
        }
    }

    fn active_ids(&self) -> Vec<CycleNodeId> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(idx, node)| (!node.collected).then_some(CycleNodeId(idx)))
            .collect()
    }

    fn root_ids(&self) -> Vec<CycleNodeId> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(idx, node)| {
                (!node.collected && node.roots > 0).then_some(CycleNodeId(idx))
            })
            .collect()
    }

    fn outgoing_active(&self, id: CycleNodeId) -> Vec<CycleNodeId> {
        self.nodes
            .get(id.index())
            .map(|node| {
                node.edges
                    .iter()
                    .copied()
                    .filter(|child| self.contains(*child))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn live_from_roots(&self) -> BTreeSet<CycleNodeId> {
        let mut live = BTreeSet::new();
        let mut queue: VecDeque<_> = self.root_ids().into_iter().collect();

        while let Some(id) = queue.pop_front() {
            if !live.insert(id) {
                continue;
            }
            for child in self.outgoing_active(id) {
                queue.push_back(child);
            }
        }

        live
    }

    fn unrooted_components(&self, live: &BTreeSet<CycleNodeId>) -> Vec<Vec<CycleNodeId>> {
        let mut tarjan = Tarjan::new(self, live);
        for id in self.active_ids() {
            if !live.contains(&id) && !tarjan.indices.contains_key(&id) {
                tarjan.connect(id);
            }
        }
        tarjan.components
    }

    fn is_cyclic_component(&self, component: &[CycleNodeId]) -> bool {
        if component.len() > 1 {
            return true;
        }

        component
            .first()
            .and_then(|id| self.nodes.get(id.index()).map(|node| (*id, node)))
            .map(|(id, node)| node.edges.contains(&id))
            .unwrap_or(false)
    }

    fn component_matches(&self, scan: CycleScan, component: &[CycleNodeId]) -> bool {
        match scan {
            CycleScan::All => true,
            CycleScan::Kind(kind) => component.iter().any(|id| self.kind(*id) == Some(kind)),
        }
    }
}

struct Tarjan<'a> {
    graph: &'a CycleGraph,
    live: &'a BTreeSet<CycleNodeId>,
    index: usize,
    indices: BTreeMap<CycleNodeId, usize>,
    lowlinks: BTreeMap<CycleNodeId, usize>,
    stack: Vec<CycleNodeId>,
    on_stack: BTreeSet<CycleNodeId>,
    components: Vec<Vec<CycleNodeId>>,
}

impl<'a> Tarjan<'a> {
    fn new(graph: &'a CycleGraph, live: &'a BTreeSet<CycleNodeId>) -> Self {
        Self {
            graph,
            live,
            index: 0,
            indices: BTreeMap::new(),
            lowlinks: BTreeMap::new(),
            stack: Vec::new(),
            on_stack: BTreeSet::new(),
            components: Vec::new(),
        }
    }

    fn connect(&mut self, id: CycleNodeId) {
        let current = self.index;
        self.indices.insert(id, current);
        self.lowlinks.insert(id, current);
        self.index += 1;
        self.stack.push(id);
        self.on_stack.insert(id);

        for child in self.graph.outgoing_active(id) {
            if self.live.contains(&child) {
                continue;
            }
            if !self.indices.contains_key(&child) {
                self.connect(child);
                let low = self.lowlinks[&id].min(self.lowlinks[&child]);
                self.lowlinks.insert(id, low);
            } else if self.on_stack.contains(&child) {
                let low = self.lowlinks[&id].min(self.indices[&child]);
                self.lowlinks.insert(id, low);
            }
        }

        if self.lowlinks[&id] == self.indices[&id] {
            let mut component = Vec::new();
            while let Some(member) = self.stack.pop() {
                self.on_stack.remove(&member);
                component.push(member);
                if member == id {
                    break;
                }
            }
            component.sort();
            self.components.push(component);
        }
    }
}
