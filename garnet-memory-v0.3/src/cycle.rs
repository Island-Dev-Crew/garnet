//! Bounded observable cycle-collection reference path.
//!
//! This module is not the production allocator-integrated ARC collector.
//! It gives Mnemos a deterministic graph model for Mini-Spec §4.5
//! fixtures: rooted nodes stay live, unrooted acyclic nodes remain available
//! for normal eviction, and unrooted cycles are collected by a bounded
//! trial-deletion pass with kind-aware scan scheduling. Safe-mode allocations
//! can be modeled as affine nodes that are retained but excluded from ARC
//! cycle detection.

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

/// Allocation discipline for a node in the observable cycle fixture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleAllocationMode {
    /// Managed-mode ARC allocation participating in cycle collection.
    ManagedArc,
    /// Safe-mode affine allocation; not ARC-managed and not scanned.
    SafeAffine,
}

/// Bounded root-candidate buffer for trial-deletion fixtures.
///
/// Production ARC will own this buffer inside the allocator. This reference
/// type only models the observable scheduling rule: a decrement can enqueue a
/// still-referenced object, and collection scans the buffered roots instead of
/// every unrooted object in the graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CycleRootBuffer {
    scan: CycleScan,
    threshold: usize,
    roots: BTreeSet<CycleNodeId>,
}

impl CycleRootBuffer {
    pub const DEFAULT_THRESHOLD: usize = 256;

    pub fn new(scan: CycleScan) -> Self {
        Self::with_threshold(scan, Self::DEFAULT_THRESHOLD)
    }

    pub fn with_threshold(scan: CycleScan, threshold: usize) -> Self {
        Self {
            scan,
            threshold: threshold.max(1),
            roots: BTreeSet::new(),
        }
    }

    pub fn scan(&self) -> CycleScan {
        self.scan
    }

    pub fn threshold(&self) -> usize {
        self.threshold
    }

    pub fn len(&self) -> usize {
        self.roots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }

    pub fn buffered_roots(&self) -> Vec<CycleNodeId> {
        self.roots.iter().copied().collect()
    }

    fn insert(&mut self, id: CycleNodeId) {
        self.roots.insert(id);
    }

    fn clear(&mut self) {
        self.roots.clear();
    }
}

/// Result of a cycle-collection pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CycleCollectReport {
    pub scan: CycleScan,
    pub trial_candidates: Vec<CycleNodeId>,
    pub trial_retained: Vec<CycleNodeId>,
    pub finalization_order: Vec<CycleNodeId>,
    pub retained_roots: Vec<CycleNodeId>,
    pub retained: Vec<CycleNodeId>,
    pub collected: Vec<CycleNodeId>,
}

/// Errors returned by graph mutation helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CycleGraphError {
    MissingNode(CycleNodeId),
    NonArcNode(CycleNodeId),
    RootUnderflow(CycleNodeId),
}

impl fmt::Display for CycleGraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingNode(id) => write!(f, "missing cycle graph node {}", id.index()),
            Self::NonArcNode(id) => write!(f, "cycle graph node {} is not ARC-managed", id.index()),
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
    mode: CycleAllocationMode,
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
        self.add_node_with_mode(kind, CycleAllocationMode::ManagedArc, label)
    }

    pub fn add_safe_node(&mut self, kind: MemoryKind, label: impl Into<String>) -> CycleNodeId {
        self.add_node_with_mode(kind, CycleAllocationMode::SafeAffine, label)
    }

    fn add_node_with_mode(
        &mut self,
        kind: MemoryKind,
        mode: CycleAllocationMode,
        label: impl Into<String>,
    ) -> CycleNodeId {
        let id = CycleNodeId(self.nodes.len());
        self.nodes.push(CycleNode {
            kind,
            mode,
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

    pub fn allocation_mode(&self, id: CycleNodeId) -> Option<CycleAllocationMode> {
        self.nodes.get(id.index()).map(|node| node.mode)
    }

    pub fn add_root(&mut self, id: CycleNodeId) -> Result<(), CycleGraphError> {
        self.ensure_arc_tracked(id)?;
        let node = self.node_mut(id)?;
        node.roots += 1;
        Ok(())
    }

    pub fn release_root(&mut self, id: CycleNodeId) -> Result<(), CycleGraphError> {
        self.ensure_arc_tracked(id)?;
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
        self.collect_candidate_cycles(scan, trial_candidates, live)
    }

    /// Release one ARC root and enqueue it for buffered trial deletion when it
    /// remains referenced only by other ARC nodes.
    pub fn release_root_to_buffer(
        &mut self,
        id: CycleNodeId,
        buffer: &mut CycleRootBuffer,
    ) -> Result<Option<CycleCollectReport>, CycleGraphError> {
        self.release_root(id)?;

        if self.should_buffer_after_release(id, buffer.scan()) {
            buffer.insert(id);
        }

        if buffer.len() >= buffer.threshold() {
            Ok(Some(self.collect_buffered_cycles(buffer)))
        } else {
            Ok(None)
        }
    }

    /// Collect cycles reachable from the buffered roots and clear the buffer.
    pub fn collect_buffered_cycles(&mut self, buffer: &mut CycleRootBuffer) -> CycleCollectReport {
        let scan = buffer.scan();
        let live = self.live_from_roots();
        let counts = self.reference_counts();
        let trial_candidates = buffer
            .buffered_roots()
            .into_iter()
            .filter(|id| self.contains(*id))
            .filter(|id| !live.contains(id))
            .filter(|id| counts.get(id).copied().unwrap_or(0) > 0)
            .filter(|id| self.scan_matches_node(scan, *id))
            .collect();

        buffer.clear();
        self.collect_candidate_cycles(scan, trial_candidates, live)
    }

    fn collect_candidate_cycles(
        &mut self,
        scan: CycleScan,
        trial_candidates: Vec<CycleNodeId>,
        live: BTreeSet<CycleNodeId>,
    ) -> CycleCollectReport {
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
            finalization_order: trial.finalization_order,
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

    fn ensure_arc_tracked(&self, id: CycleNodeId) -> Result<(), CycleGraphError> {
        self.ensure_active(id)?;
        if self.is_arc_tracked(id) {
            Ok(())
        } else {
            Err(CycleGraphError::NonArcNode(id))
        }
    }

    fn is_arc_tracked(&self, id: CycleNodeId) -> bool {
        self.nodes
            .get(id.index())
            .map(|node| node.mode == CycleAllocationMode::ManagedArc)
            .unwrap_or(false)
    }

    fn active_ids(&self) -> Vec<CycleNodeId> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(idx, node)| (!node.collected).then_some(CycleNodeId(idx)))
            .collect()
    }

    fn active_arc_ids(&self) -> Vec<CycleNodeId> {
        self.active_ids()
            .into_iter()
            .filter(|id| self.is_arc_tracked(*id))
            .collect()
    }

    fn root_ids(&self) -> Vec<CycleNodeId> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(idx, node)| {
                (!node.collected && node.mode == CycleAllocationMode::ManagedArc && node.roots > 0)
                    .then_some(CycleNodeId(idx))
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

    fn outgoing_arc(&self, id: CycleNodeId) -> Vec<CycleNodeId> {
        self.outgoing_active(id)
            .into_iter()
            .filter(|child| self.is_arc_tracked(*child))
            .collect()
    }

    fn live_from_roots(&self) -> BTreeSet<CycleNodeId> {
        let mut live = BTreeSet::new();
        let mut queue: VecDeque<_> = self.root_ids().into_iter().collect();

        while let Some(id) = queue.pop_front() {
            if !live.insert(id) {
                continue;
            }
            for child in self.outgoing_arc(id) {
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

    fn should_buffer_after_release(&self, id: CycleNodeId, scan: CycleScan) -> bool {
        if !self.contains(id) || !self.is_arc_tracked(id) || !self.scan_matches_node(scan, id) {
            return false;
        }

        let live = self.live_from_roots();
        let counts = self.reference_counts();
        !live.contains(&id) && counts.get(&id).copied().unwrap_or(0) > 0
    }

    fn reference_counts(&self) -> BTreeMap<CycleNodeId, usize> {
        let mut counts = BTreeMap::new();
        for id in self.active_arc_ids() {
            counts.insert(id, self.nodes[id.index()].roots);
        }

        for id in self.active_arc_ids() {
            for child in self.outgoing_arc(id) {
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
            .active_arc_ids()
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
        let mut finalization_order = Vec::new();
        for candidate in candidates {
            self.collect_white(
                *candidate,
                &mut colors,
                &mut collected,
                &mut finalization_order,
            );
        }

        let retained = candidates
            .iter()
            .copied()
            .filter(|id| !collected.contains(id))
            .collect();

        TrialOutcome {
            retained,
            collected,
            finalization_order,
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

        for child in self.outgoing_arc(id) {
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
        for child in self.outgoing_arc(id) {
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
        for child in self.outgoing_arc(id) {
            *counts.entry(child).or_insert(0) += 1;
            self.scan_black(child, colors, counts);
        }
    }

    fn collect_white(
        &self,
        id: CycleNodeId,
        colors: &mut BTreeMap<CycleNodeId, TrialColor>,
        collected: &mut BTreeSet<CycleNodeId>,
        finalization_order: &mut Vec<CycleNodeId>,
    ) {
        if colors.get(&id) != Some(&TrialColor::White) {
            return;
        }

        colors.insert(id, TrialColor::Black);
        for child in self.outgoing_arc(id) {
            self.collect_white(child, colors, collected, finalization_order);
        }
        collected.insert(id);
        finalization_order.push(id);
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
    finalization_order: Vec<CycleNodeId>,
}
