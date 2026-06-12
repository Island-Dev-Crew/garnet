//! S37 — capability-surface diff (the headline novelty).
//!
//! Diffs two S35 [`CapabilitySurface`]s and answers the gating question: did the
//! program **gain authority**? A refactor or edition bump may move caps between
//! functions, but a *new program-level capability* (or an introduced wildcard)
//! is the thing `diff-caps` gates on. Pure and deterministic (all lists sorted).

use crate::capset::CapSet;
use crate::CapabilitySurface;
use std::collections::{BTreeMap, BTreeSet};

/// The capability difference from an `old` surface to a `new` one.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapsDiff {
    /// Capabilities in the new aggregate but not the old — **new program authority**.
    pub aggregate_added: Vec<String>,
    /// Capabilities in the old aggregate but not the new — authority reduced.
    pub aggregate_removed: Vec<String>,
    /// Functions present in new but not old (by name).
    pub functions_added: Vec<String>,
    /// Functions present in old but not new (by name).
    pub functions_removed: Vec<String>,
    /// Functions present in both whose declared caps grew: `(name, caps gained)`.
    pub functions_caps_expanded: Vec<(String, Vec<String>)>,
    /// A `@caps(*)` wildcard appears in new but not old.
    pub wildcard_introduced: bool,
}

impl CapsDiff {
    /// Whether the program **gained authority** — the gating condition. A
    /// function re-declaring a capability already in the program's aggregate is
    /// NOT new program authority; only a new aggregate capability or an
    /// introduced wildcard is.
    pub fn authority_expanded(&self) -> bool {
        !self.aggregate_added.is_empty() || self.wildcard_introduced
    }

    /// Whether nothing changed at all (for no-op reporting).
    pub fn is_empty(&self) -> bool {
        self.aggregate_added.is_empty()
            && self.aggregate_removed.is_empty()
            && self.functions_added.is_empty()
            && self.functions_removed.is_empty()
            && self.functions_caps_expanded.is_empty()
            && !self.wildcard_introduced
    }
}

/// Split a sorted cap-name list into its canonical-bitset core and the
/// (still sorted) unknown/user-defined remainder. The surface layer keeps
/// full string fidelity for unknown names — a gained *unknown* capability
/// must still gate as authority expansion — so the bitset carries the
/// closed set and the remainder rides alongside as strings.
fn split_caps(caps: &[String]) -> (CapSet, Vec<&String>) {
    let mut known = CapSet::EMPTY;
    let mut unknown: Vec<&String> = Vec::new();
    for c in caps {
        match CapSet::from_name(c) {
            Some(bit) => known |= bit,
            None => unknown.push(c),
        }
    }
    (known, unknown)
}

/// RB-1: one side of a capability delta. `gained` keeps only the bits also
/// present on the `keep` side (the XOR-split identity: gained = delta & new,
/// removed = delta & old), then merges the canonical names with the unknown
/// remainder, preserving the lexicographic order the set-difference
/// implementation produced.
fn delta_side(
    delta: CapSet,
    keep: CapSet,
    keep_unknown: &[&String],
    drop_unknown: &[&String],
) -> Vec<String> {
    let drop_set: BTreeSet<&String> = drop_unknown.iter().copied().collect();
    let mut out: BTreeSet<String> = delta
        .intersect(keep)
        .iter_names()
        .map(|n| n.to_string())
        .collect();
    out.extend(
        keep_unknown
            .iter()
            .filter(|c| !drop_set.contains(*c))
            .map(|c| (*c).clone()),
    );
    out.into_iter().collect()
}

/// Compute the capability diff from `old` to `new`. Input surfaces are already
/// sorted (S35); every output list preserves sorted order. RB-1: the delta
/// over the closed capability set is XOR on [`CapSet`] bitsets; unknown
/// (user-defined) names are diffed as strings so no authority change can
/// hide behind an unrepresentable name.
pub fn diff_caps(old: &CapabilitySurface, new: &CapabilitySurface) -> CapsDiff {
    let (old_known, old_unknown) = split_caps(&old.aggregate);
    let (new_known, new_unknown) = split_caps(&new.aggregate);
    let delta = old_known.delta(new_known);
    let aggregate_added = delta_side(delta, new_known, &new_unknown, &old_unknown);
    let aggregate_removed = delta_side(delta, old_known, &old_unknown, &new_unknown);

    let old_fns: BTreeMap<&String, &Vec<String>> =
        old.per_function.iter().map(|(n, c)| (n, c)).collect();
    let new_fns: BTreeMap<&String, &Vec<String>> =
        new.per_function.iter().map(|(n, c)| (n, c)).collect();

    let mut functions_added: Vec<String> = new
        .per_function
        .iter()
        .filter(|(n, _)| !old_fns.contains_key(n))
        .map(|(n, _)| n.clone())
        .collect();
    let mut functions_removed: Vec<String> = old
        .per_function
        .iter()
        .filter(|(n, _)| !new_fns.contains_key(n))
        .map(|(n, _)| n.clone())
        .collect();
    let mut functions_caps_expanded: Vec<(String, Vec<String>)> = Vec::new();
    for (name, old_caps) in &old.per_function {
        if let Some(new_caps) = new_fns.get(name) {
            let (fn_old_known, fn_old_unknown) = split_caps(old_caps);
            let (fn_new_known, fn_new_unknown) = split_caps(new_caps);
            let fn_delta = fn_old_known.delta(fn_new_known);
            let gained = delta_side(fn_delta, fn_new_known, &fn_new_unknown, &fn_old_unknown);
            if !gained.is_empty() {
                functions_caps_expanded.push((name.clone(), gained));
            }
        }
    }
    functions_added.sort();
    functions_removed.sort();
    functions_caps_expanded.sort_by(|a, b| a.0.cmp(&b.0));

    CapsDiff {
        aggregate_added,
        aggregate_removed,
        functions_added,
        functions_removed,
        functions_caps_expanded,
        wildcard_introduced: new.has_wildcard && !old.has_wildcard,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn surf(agg: &[&str], per: &[(&str, &[&str])], wild: bool) -> CapabilitySurface {
        CapabilitySurface {
            aggregate: agg.iter().map(|s| s.to_string()).collect(),
            per_function: per
                .iter()
                .map(|(n, c)| (n.to_string(), c.iter().map(|x| x.to_string()).collect()))
                .collect(),
            has_wildcard: wild,
        }
    }

    #[test]
    fn added_cap_is_authority_expansion() {
        let d = diff_caps(
            &surf(&["fs"], &[("a", &["fs"])], false),
            &surf(&["fs", "net"], &[("a", &["fs", "net"])], false),
        );
        assert_eq!(d.aggregate_added, vec!["net"]);
        assert!(d.authority_expanded());
    }

    #[test]
    fn removed_only_is_not_expansion() {
        let d = diff_caps(
            &surf(&["fs", "net"], &[("a", &["fs", "net"])], false),
            &surf(&["fs"], &[("a", &["fs"])], false),
        );
        assert_eq!(d.aggregate_removed, vec!["net"]);
        assert!(d.aggregate_added.is_empty());
        assert!(!d.authority_expanded());
    }

    #[test]
    fn identical_is_empty_no_expansion() {
        let s = surf(&["fs"], &[("a", &["fs"])], false);
        let d = diff_caps(&s, &s);
        assert!(d.is_empty());
        assert!(!d.authority_expanded());
    }

    #[test]
    fn wildcard_introduced_is_expansion() {
        let d = diff_caps(&surf(&["fs"], &[], false), &surf(&["fs", "*"], &[], true));
        assert!(d.wildcard_introduced);
        assert!(d.authority_expanded());
    }

    #[test]
    fn function_added_and_removed_tracked() {
        let d = diff_caps(
            &surf(&["fs"], &[("old_fn", &["fs"])], false),
            &surf(&["fs"], &[("new_fn", &["fs"])], false),
        );
        assert_eq!(d.functions_added, vec!["new_fn"]);
        assert_eq!(d.functions_removed, vec!["old_fn"]);
        assert!(!d.authority_expanded()); // same aggregate
    }

    #[test]
    fn function_caps_expanded_without_program_authority_gain() {
        // `b` gains `fs`, but `fs` was already in the aggregate (via `a`) — that
        // is NOT new program authority.
        let d = diff_caps(
            &surf(&["fs"], &[("a", &["fs"]), ("b", &[])], false),
            &surf(&["fs"], &[("a", &["fs"]), ("b", &["fs"])], false),
        );
        assert_eq!(
            d.functions_caps_expanded,
            vec![("b".to_string(), vec!["fs".to_string()])]
        );
        assert!(!d.authority_expanded());
    }

    #[test]
    fn gained_unknown_cap_still_gates_as_authority_expansion() {
        // A user-defined capability name entering the aggregate must still
        // flag authority expansion — unknown names ride alongside the
        // bitset as strings precisely so this cannot regress.
        let d = diff_caps(
            &surf(&["fs"], &[("a", &["fs"])], false),
            &surf(
                &["custom_cap", "fs"],
                &[("a", &["custom_cap", "fs"])],
                false,
            ),
        );
        assert_eq!(d.aggregate_added, vec!["custom_cap"]);
        assert!(d.authority_expanded());
    }

    // ── RB-1 permanent reference suite ─────────────────────────────────
    //
    // `reference_diff_caps` is the pre-RB-1 set-difference implementation,
    // kept verbatim as a test-only oracle. The proptest below feeds both
    // implementations random invariant-respecting surfaces (canonical AND
    // unknown cap names, wildcard included) and requires identical output.

    /// The old (pre-RB-1) `diff_caps`, verbatim — test oracle only.
    fn reference_diff_caps(old: &CapabilitySurface, new: &CapabilitySurface) -> CapsDiff {
        let old_agg: BTreeSet<&String> = old.aggregate.iter().collect();
        let new_agg: BTreeSet<&String> = new.aggregate.iter().collect();
        let aggregate_added = new
            .aggregate
            .iter()
            .filter(|c| !old_agg.contains(c))
            .cloned()
            .collect();
        let aggregate_removed = old
            .aggregate
            .iter()
            .filter(|c| !new_agg.contains(c))
            .cloned()
            .collect();

        let new_fns: BTreeMap<&String, &Vec<String>> =
            new.per_function.iter().map(|(n, c)| (n, c)).collect();
        let old_fns: BTreeMap<&String, &Vec<String>> =
            old.per_function.iter().map(|(n, c)| (n, c)).collect();

        let mut functions_added: Vec<String> = new
            .per_function
            .iter()
            .filter(|(n, _)| !old_fns.contains_key(n))
            .map(|(n, _)| n.clone())
            .collect();
        let mut functions_removed: Vec<String> = old
            .per_function
            .iter()
            .filter(|(n, _)| !new_fns.contains_key(n))
            .map(|(n, _)| n.clone())
            .collect();
        let mut functions_caps_expanded: Vec<(String, Vec<String>)> = Vec::new();
        for (name, old_caps) in &old.per_function {
            if let Some(new_caps) = new_fns.get(name) {
                let old_set: BTreeSet<&String> = old_caps.iter().collect();
                let gained: Vec<String> = new_caps
                    .iter()
                    .filter(|c| !old_set.contains(c))
                    .cloned()
                    .collect();
                if !gained.is_empty() {
                    functions_caps_expanded.push((name.clone(), gained));
                }
            }
        }
        functions_added.sort();
        functions_removed.sort();
        functions_caps_expanded.sort_by(|a, b| a.0.cmp(&b.0));

        CapsDiff {
            aggregate_added,
            aggregate_removed,
            functions_added,
            functions_removed,
            functions_caps_expanded,
            wildcard_introduced: new.has_wildcard && !old.has_wildcard,
        }
    }

    use proptest::prelude::*;

    /// Cap-name pool: the closed set, the wildcard, and unknown names.
    const POOL: [&str; 10] = [
        "*",
        "env",
        "ffi",
        "fs",
        "net",
        "net_internal",
        "proc",
        "time",
        "custom_cap",
        "zz_unknown",
    ];

    /// Build an invariant-respecting surface the way `capability_surface`
    /// does: per_function sorted by unique fn name, each cap list sorted +
    /// deduplicated, aggregate = sorted union, wildcard = any `*`.
    fn surface_from(fns: Vec<(usize, Vec<usize>)>) -> CapabilitySurface {
        let mut per: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (fn_idx, cap_idxs) in fns {
            let entry = per.entry(format!("f{fn_idx}")).or_default();
            entry.extend(cap_idxs.iter().map(|&i| POOL[i].to_string()));
        }
        let mut aggregate: BTreeSet<String> = BTreeSet::new();
        let mut has_wildcard = false;
        for caps in per.values() {
            aggregate.extend(caps.iter().cloned());
            has_wildcard |= caps.contains("*");
        }
        CapabilitySurface {
            aggregate: aggregate.into_iter().collect(),
            per_function: per
                .into_iter()
                .map(|(n, c)| (n, c.into_iter().collect()))
                .collect(),
            has_wildcard,
        }
    }

    fn surface_strategy() -> impl Strategy<Value = CapabilitySurface> {
        proptest::collection::vec(
            (0..6usize, proptest::collection::vec(0..POOL.len(), 0..4)),
            0..6,
        )
        .prop_map(surface_from)
    }

    proptest! {
        /// RB-1 differential: the XOR-based diff equals the old
        /// set-difference diff on random surfaces — aggregate deltas,
        /// per-function expansions, function adds/removes, wildcard.
        #[test]
        fn xor_diff_matches_set_reference(
            old in surface_strategy(),
            new in surface_strategy(),
        ) {
            prop_assert_eq!(diff_caps(&old, &new), reference_diff_caps(&old, &new));
        }
    }
}
