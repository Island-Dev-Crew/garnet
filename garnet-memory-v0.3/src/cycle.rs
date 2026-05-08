//! Bounded observable cycle-collection reference path.
//!
//! This module is not the production allocator-integrated ARC collector.
//! It gives Mnemos a deterministic graph model for Mini-Spec §4.5
//! fixtures: rooted nodes stay live, unrooted acyclic nodes remain available
//! for normal eviction, and unrooted cycles are collected by a bounded
//! trial-deletion pass with kind-aware scan scheduling.

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
    pub trial_candidates: Vec<CycleNodeId>,
    pub trial_retained: Vec<CycleNodeId>,
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

    /// Collect unrooted cycles that match the requested scan.
    ///
    /// The collector deliberately leaves unrooted acyclic components alone:
    /// those belong to ordinary retention/eviction policy, not the cycle
    /// detector. Cross-kind components are collected as a whole when any node
    /// in the component is reached from a matching trial-deletion candidate.
    pub fn collect_cycles(&mut self, scan: CycleScan) -> CycleCollectReport {
        let live = self.live_from_roots();
        let trial_candidates = self.trial_candidates(scan, &live);
        let trial = self.run_trial_deletion(&trial_candidates, &live);
        let collected_set = trial.collected;

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
            trial_candidates,
            trial_retained: trial.retained,
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

    fn scan_matches_node(&self, scan: CycleScan, id: CycleNodeId) -> bool {
        match scan {
            CycleScan::All => true,
            CycleScan::Kind(kind) => self.kind(id) == Some(kind),
        }
    }

    fn reference_counts(&self) -> BTreeMap<CycleNodeId, usize> {
        let mut counts = BTreeMap::new();
        for id in self.active_ids() {
            counts.insert(id, self.nodes[id.index()].roots);
        }

        for id in self.active_ids() {
            for child in self.outgoing_active(id) {
                *counts.entry(child).or_insert(0) += 1;
            }
        }
        counts
    }

    fn trial_candidates(&self, scan: CycleScan, live: &BTreeSet<CycleNodeId>) -> Vec<CycleNodeId> {
        let counts = self.reference_counts();
        self.active_ids()
            .into_iter()
            .filter(|id| !live.contains(id))
            .filter(|id| counts.get(id).copied().unwrap_or(0) > 0)
            .filter(|id| self.scan_matches_node(scan, *id))
            .collect()
    }

    fn run_trial_deletion(
        &self,
        candidates: &[CycleNodeId],
        live: &BTreeSet<CycleNodeId>,
    ) -> TrialOutcome {
        let mut counts = self.reference_counts();
        let mut colors: BTreeMap<_, _> = self
            .active_ids()
            .into_iter()
            .map(|id| (id, TrialColor::Black))
            .collect();

        for candidate in candidates {
            self.mark_gray(*candidate, &mut colors, &mut counts);
        }
        for candidate in candidates {
            self.scan_candidate(*candidate, live, &mut colors, &mut counts);
        }

        let mut collected = BTreeSet::new();
        for candidate in candidates {
            self.collect_white(*candidate, &mut colors, &mut collected);
        }

        let retained = candidates
            .iter()
            .copied()
            .filter(|id| !collected.contains(id))
            .collect();

        TrialOutcome {
            retained,
            collected,
        }
    }

    fn mark_gray(
        &self,
        id: CycleNodeId,
        colors: &mut BTreeMap<CycleNodeId, TrialColor>,
        counts: &mut BTreeMap<CycleNodeId, usize>,
    ) {
        if colors.get(&id) == Some(&TrialColor::Gray) {
            return;
        }
        colors.insert(id, TrialColor::Gray);

        for child in self.outgoing_active(id) {
            if let Some(count) = counts.get_mut(&child) {
                *count = count.saturating_sub(1);
            }
            self.mark_gray(child, colors, counts);
        }
    }

    fn scan_candidate(
        &self,
        id: CycleNodeId,
        live: &BTreeSet<CycleNodeId>,
        colors: &mut BTreeMap<CycleNodeId, TrialColor>,
        counts: &mut BTreeMap<CycleNodeId, usize>,
    ) {
        if colors.get(&id) != Some(&TrialColor::Gray) {
            return;
        }

        if live.contains(&id) || counts.get(&id).copied().unwrap_or(0) > 0 {
            self.scan_black(id, colors, counts);
            return;
        }

        colors.insert(id, TrialColor::White);
        for child in self.outgoing_active(id) {
            self.scan_candidate(child, live, colors, counts);
        }
    }

    fn scan_black(
        &self,
        id: CycleNodeId,
        colors: &mut BTreeMap<CycleNodeId, TrialColor>,
        counts: &mut BTreeMap<CycleNodeId, usize>,
    ) {
        if colors.get(&id) == Some(&TrialColor::Black) {
            return;
        }

        colors.insert(id, TrialColor::Black);
        for child in self.outgoing_active(id) {
            *counts.entry(child).or_insert(0) += 1;
            self.scan_black(child, colors, counts);
        }
    }

    fn collect_white(
        &self,
        id: CycleNodeId,
        colors: &mut BTreeMap<CycleNodeId, TrialColor>,
        collected: &mut BTreeSet<CycleNodeId>,
    ) {
        if colors.get(&id) != Some(&TrialColor::White) {
            return;
        }

        colors.insert(id, TrialColor::Black);
        collected.insert(id);
        for child in self.outgoing_active(id) {
            self.collect_white(child, colors, collected);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrialColor {
    Black,
    Gray,
    White,
}

struct TrialOutcome {
    retained: Vec<CycleNodeId>,
    collected: BTreeSet<CycleNodeId>,
}
