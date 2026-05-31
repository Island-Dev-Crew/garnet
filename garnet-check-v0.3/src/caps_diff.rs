//! S37 — capability-surface diff (the headline novelty).
//!
//! Diffs two S35 [`CapabilitySurface`]s and answers the gating question: did the
//! program **gain authority**? A refactor or edition bump may move caps between
//! functions, but a *new program-level capability* (or an introduced wildcard)
//! is the thing `diff-caps` gates on. Pure and deterministic (all lists sorted).

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

/// Compute the capability diff from `old` to `new`. Input surfaces are already
/// sorted (S35); every output list preserves sorted order.
pub fn diff_caps(old: &CapabilitySurface, new: &CapabilitySurface) -> CapsDiff {
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
}
