//! RB-1 — [`CapSet`]: the checker's capability bitset.
//!
//! Replaces the `BTreeSet<String>` capability representation inside the
//! CapCaps propagator with a `Copy` `u16` bitset over the closed capability
//! set. Propagation is bitwise OR over the call graph; subset checking is
//! `required & !declared == 0`; the diff-caps delta is XOR.
//!
//! ## Bit layout (`u16`)
//!
//! | bit  | capability     | meaning                                          |
//! |------|----------------|--------------------------------------------------|
//! | 0    | `*`            | wildcard/star (debug-only at the surface)        |
//! | 1    | `env`          | process-environment access (S17)                 |
//! | 2    | `ffi`          | `extern "C"` calls                               |
//! | 3    | `fs`           | file-system read + write                         |
//! | 4    | `net`          | public TCP/UDP                                   |
//! | 5    | `net_internal` | RFC1918/loopback TCP/UDP                         |
//! | 6    | `proc`         | process spawn, signals                           |
//! | 7    | `time`         | wall clock + sleep                               |
//! | 8    | (other)        | at least one unknown/user-defined cap declared   |
//! | 9–15 | reserved       | must be zero; reserved for future canonical caps |
//!
//! Bits 0..=7 are assigned in **lexicographic order of the canonical name**
//! (`*` is ASCII 0x2A, before lowercase letters), so ascending-bit iteration
//! yields names in exactly the order a `BTreeSet<String>` iterates. That
//! ordering equivalence is what keeps diagnostics byte-identical with the
//! set-based implementation this module replaces.
//!
//! ## The `OTHER` bit — claim boundary
//!
//! The parser admits user-defined capability names (`Capability::Other`);
//! the checker's annotation audit rejects unknown names with an error, but
//! checking continues and the propagator still runs over the module. The
//! old set-based propagator carried unknown declared names verbatim; the
//! only propagator behavior that depended on them was **presence** (a fn
//! whose only declared cap is unknown still counts as "annotated" for the
//! coverage-check gate in `has_caps_annotation`). [`CapSet`] preserves
//! exactly that presence semantics via [`CapSet::OTHER`] and nothing more:
//! the *identity* of unknown names lives at the surface/audit layers, which
//! stay string-typed. Required (propagated) caps can never contain `OTHER`
//! in-tree: primitive caps come from `garnet_stdlib::registry`, whose
//! strings are all canonical — the `registry_caps_all_canonical` trap test
//! below turns any future violation of that invariant into a test failure.

/// A set of capabilities as a `u16` bitset. `Copy` — passing one around is
/// two bytes, and call-graph propagation is branch-free bitwise OR.
#[derive(Copy, Clone, PartialEq, Eq, Default, Hash, PartialOrd, Ord)]
pub struct CapSet(u16);

/// Canonical capability names and their bits, sorted lexicographically.
/// Ascending-bit iteration over this table reproduces `BTreeSet<String>`
/// iteration order for the closed set.
const NAMED: [(&str, u16); 8] = [
    ("*", 1 << 0),
    ("env", 1 << 1),
    ("ffi", 1 << 2),
    ("fs", 1 << 3),
    ("net", 1 << 4),
    ("net_internal", 1 << 5),
    ("proc", 1 << 6),
    ("time", 1 << 7),
];

impl CapSet {
    /// The empty capability set.
    pub const EMPTY: CapSet = CapSet(0);
    /// `*` — the wildcard/star capability (surface-level only).
    pub const STAR: CapSet = CapSet(1 << 0);
    /// `env` — process-environment access.
    pub const ENV: CapSet = CapSet(1 << 1);
    /// `ffi` — `extern "C"` calls.
    pub const FFI: CapSet = CapSet(1 << 2);
    /// `fs` — file-system read + write.
    pub const FS: CapSet = CapSet(1 << 3);
    /// `net` — public TCP/UDP.
    pub const NET: CapSet = CapSet(1 << 4);
    /// `net_internal` — RFC1918/loopback TCP/UDP.
    pub const NET_INTERNAL: CapSet = CapSet(1 << 5);
    /// `proc` — process spawn, signals.
    pub const PROC: CapSet = CapSet(1 << 6);
    /// `time` — wall clock + sleep.
    pub const TIME: CapSet = CapSet(1 << 7);
    /// Presence marker for unknown/user-defined declared cap names. See the
    /// module docs for the exact (deliberately narrow) semantics.
    pub const OTHER: CapSet = CapSet(1 << 8);

    /// Look up a canonical capability name. Returns `None` for unknown
    /// (user-defined) names — callers that need old `BTreeSet` presence
    /// semantics for those use [`CapSet::from_name_or_other`].
    pub fn from_name(name: &str) -> Option<CapSet> {
        NAMED
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, bit)| CapSet(*bit))
    }

    /// Like [`CapSet::from_name`], but maps unknown names to [`CapSet::OTHER`]
    /// so that *presence* of a declaration is never lost.
    pub fn from_name_or_other(name: &str) -> CapSet {
        Self::from_name(name).unwrap_or(CapSet::OTHER)
    }

    /// Whether no capability (named or other) is present.
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Whether the unknown-declared marker is set.
    pub fn has_other(self) -> bool {
        self.0 & CapSet::OTHER.0 != 0
    }

    /// Whether the canonical capability `name` is present. Unknown names
    /// always return `false`: identity of user-defined caps is not tracked
    /// here (module docs, "claim boundary").
    pub fn contains(self, name: &str) -> bool {
        match Self::from_name(name) {
            Some(bit) => self.0 & bit.0 != 0,
            None => false,
        }
    }

    /// Whether every capability in `other` is also in `self`:
    /// `other & !self == 0`.
    pub fn contains_all(self, other: CapSet) -> bool {
        other.0 & !self.0 == 0
    }

    /// Whether `self` is a subset of `superset`: `self & !superset == 0`.
    pub fn is_subset(self, superset: CapSet) -> bool {
        self.0 & !superset.0 == 0
    }

    /// The capabilities in `self` but not in `other` (`self & !other`).
    pub fn difference(self, other: CapSet) -> CapSet {
        CapSet(self.0 & !other.0)
    }

    /// The symmetric difference (`self ^ other`) — the diff-caps delta.
    pub fn delta(self, other: CapSet) -> CapSet {
        CapSet(self.0 ^ other.0)
    }

    /// The intersection (`self & other`). diff-caps splits its XOR delta
    /// into gained/removed by intersecting with the new/old side.
    pub fn intersect(self, other: CapSet) -> CapSet {
        CapSet(self.0 & other.0)
    }

    /// Iterate the canonical names present, in lexicographic order (the
    /// same order `BTreeSet<String>` iteration produced). The `OTHER`
    /// marker has no canonical name and is never yielded.
    pub fn iter_names(self) -> impl Iterator<Item = &'static str> {
        NAMED
            .iter()
            .filter(move |(_, bit)| self.0 & bit != 0)
            .map(|(name, _)| *name)
    }

    /// The canonical names present, sorted lexicographically.
    pub fn names(self) -> Vec<&'static str> {
        self.iter_names().collect()
    }

    /// Raw bits — for tests and debug rendering only.
    pub fn bits(self) -> u16 {
        self.0
    }
}

impl std::ops::BitOr for CapSet {
    type Output = CapSet;
    fn bitor(self, rhs: CapSet) -> CapSet {
        CapSet(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for CapSet {
    fn bitor_assign(&mut self, rhs: CapSet) {
        self.0 |= rhs.0;
    }
}

impl std::fmt::Debug for CapSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CapSet({:?}", self.names())?;
        if self.has_other() {
            write!(f, " +other")?;
        }
        write!(f, ")")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::BTreeSet;

    /// Mask of every named (canonical) capability bit — bits 0..=7.
    const NAMED_MASK: u16 = (1 << 8) - 1;

    const ALL_NAMES: [&str; 8] = [
        "*",
        "env",
        "ffi",
        "fs",
        "net",
        "net_internal",
        "proc",
        "time",
    ];

    fn capset_of(names: &[&str]) -> CapSet {
        names.iter().fold(CapSet::EMPTY, |acc, n| {
            acc | CapSet::from_name(n).expect("known name")
        })
    }

    fn model_of<'a>(names: &[&'a str]) -> BTreeSet<&'a str> {
        names.iter().copied().collect()
    }

    #[test]
    fn from_name_roundtrips_every_canonical_name() {
        for name in ALL_NAMES {
            let set = CapSet::from_name(name).expect("canonical name resolves");
            assert!(set.contains(name), "{name} must be in its own set");
            assert_eq!(set.names(), vec![name]);
        }
    }

    #[test]
    fn unknown_name_is_none_and_other_preserves_presence() {
        assert_eq!(CapSet::from_name("custom"), None);
        let other = CapSet::from_name_or_other("custom");
        assert!(other.has_other());
        assert!(!other.is_empty(), "presence of an unknown cap must survive");
        assert!(other.names().is_empty(), "OTHER has no canonical name");
        assert!(!other.contains("custom"), "identity is not tracked");
    }

    #[test]
    fn empty_is_empty_and_named_bits_stay_below_reserve() {
        assert!(CapSet::EMPTY.is_empty());
        for (name, bit) in NAMED {
            assert!(
                bit & !NAMED_MASK == 0,
                "named cap {name} must sit in bits 0..=7"
            );
        }
        assert_eq!(CapSet::OTHER.bits(), 1 << 8, "OTHER is bit 8");
    }

    #[test]
    fn subset_uses_a_and_not_b() {
        let fs_net = capset_of(&["fs", "net"]);
        let fs = capset_of(&["fs"]);
        assert!(fs.is_subset(fs_net));
        assert!(!fs_net.is_subset(fs));
        assert!(CapSet::EMPTY.is_subset(fs));
        assert!(fs.contains_all(CapSet::EMPTY));
        assert!(fs_net.contains_all(fs));
    }

    #[test]
    fn delta_is_xor() {
        let a = capset_of(&["fs", "net"]);
        let b = capset_of(&["net", "time"]);
        assert_eq!(a.delta(b).names(), vec!["fs", "time"]);
        assert!(a.delta(a).is_empty());
    }

    /// Registry-drift trap: every capability string any stdlib primitive
    /// requires must map to a canonical `CapSet` bit. If a new capability
    /// name enters the registry without a `CapSet` bit, required caps would
    /// degrade to the nameless `OTHER` marker and a coverage diagnostic
    /// could lose its name — this test makes that drift a deterministic
    /// failure instead.
    #[test]
    fn registry_caps_all_canonical() {
        for (qualified, meta) in garnet_stdlib::registry::all_prims() {
            for cap in &meta.required_caps.0 {
                assert!(
                    CapSet::from_name(cap).is_some(),
                    "primitive `{qualified}` requires cap `{cap}` which has no \
                     CapSet bit — add it to garnet-check-v0.3/src/capset.rs \
                     (closed set + reserve bits) before extending the registry"
                );
            }
        }
    }

    proptest! {
        /// Union via `|` agrees with the `BTreeSet` model.
        #[test]
        fn union_matches_set_model(
            a in proptest::sample::subsequence(ALL_NAMES.to_vec(), 0..=8),
            b in proptest::sample::subsequence(ALL_NAMES.to_vec(), 0..=8),
        ) {
            let bits = capset_of(&a) | capset_of(&b);
            let mut model = model_of(&a);
            model.extend(model_of(&b));
            prop_assert_eq!(bits.names(), model.into_iter().collect::<Vec<_>>());
        }

        /// Subset agrees with the `BTreeSet` model.
        #[test]
        fn subset_matches_set_model(
            a in proptest::sample::subsequence(ALL_NAMES.to_vec(), 0..=8),
            b in proptest::sample::subsequence(ALL_NAMES.to_vec(), 0..=8),
        ) {
            let (sa, sb) = (capset_of(&a), capset_of(&b));
            let (ma, mb) = (model_of(&a), model_of(&b));
            prop_assert_eq!(sa.is_subset(sb), ma.is_subset(&mb));
        }

        /// Difference agrees with the `BTreeSet` model, including iteration order.
        #[test]
        fn difference_matches_set_model(
            a in proptest::sample::subsequence(ALL_NAMES.to_vec(), 0..=8),
            b in proptest::sample::subsequence(ALL_NAMES.to_vec(), 0..=8),
        ) {
            let bits = capset_of(&a).difference(capset_of(&b));
            let model: Vec<&str> = model_of(&a)
                .difference(&model_of(&b))
                .copied()
                .collect();
            prop_assert_eq!(bits.names(), model);
        }

        /// XOR delta agrees with the `BTreeSet` symmetric difference.
        #[test]
        fn delta_matches_symmetric_difference(
            a in proptest::sample::subsequence(ALL_NAMES.to_vec(), 0..=8),
            b in proptest::sample::subsequence(ALL_NAMES.to_vec(), 0..=8),
        ) {
            let bits = capset_of(&a).delta(capset_of(&b));
            let model: Vec<&str> = model_of(&a)
                .symmetric_difference(&model_of(&b))
                .copied()
                .collect();
            prop_assert_eq!(bits.names(), model);
        }

        /// Intersection agrees with the `BTreeSet` model, and splitting the
        /// XOR delta by intersection reproduces both set differences —
        /// the exact identity diff-caps relies on.
        #[test]
        fn intersect_matches_set_model_and_splits_delta(
            a in proptest::sample::subsequence(ALL_NAMES.to_vec(), 0..=8),
            b in proptest::sample::subsequence(ALL_NAMES.to_vec(), 0..=8),
        ) {
            let (sa, sb) = (capset_of(&a), capset_of(&b));
            let (ma, mb) = (model_of(&a), model_of(&b));
            let inter: Vec<&str> = ma.intersection(&mb).copied().collect();
            prop_assert_eq!(sa.intersect(sb).names(), inter);
            let delta = sa.delta(sb);
            prop_assert_eq!(delta.intersect(sb), sb.difference(sa));
            prop_assert_eq!(delta.intersect(sa), sa.difference(sb));
        }

        /// Name iteration is exactly `BTreeSet<String>` order.
        #[test]
        fn iteration_order_matches_btreeset(
            a in proptest::sample::subsequence(ALL_NAMES.to_vec(), 0..=8),
        ) {
            let bits = capset_of(&a);
            let model: Vec<&str> = model_of(&a).into_iter().collect();
            prop_assert_eq!(bits.names(), model);
        }
    }
}
