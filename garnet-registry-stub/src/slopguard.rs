//! Slopsquatting guard (S45).
//!
//! AI-generated code frequently references a *hallucinated* package name that is
//! a near-miss of a real one; attackers pre-register the near-miss. When a
//! requested package is **not** in the registry but closely resembles a name
//! that **is**, this module surfaces the resemblance so the resolver can warn
//! before anything is added.
//!
//! This is a pure, deterministic heuristic (string in, [`Suspicion`]s out) — no
//! I/O and no network. It is a *prompt to verify*, not a security guarantee:
//! the registry is a filesystem stub, so "known names" are whatever the local
//! index contains, not a global ecosystem feed.

/// Why a known name is suspected to be the intended target of a near-miss.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuspicionKind {
    /// Identical to the query once `-`/`_` separators are normalized
    /// (e.g. `foo-bar` vs `foo_bar`) — a common slopsquatting vector.
    SeparatorConfusable,
    /// Within `max_distance` edits (Damerau–Levenshtein / optimal string
    /// alignment) of the query. Carries the edit distance.
    EditDistance(usize),
}

impl SuspicionKind {
    /// Lower sorts first (more suspicious). Separator confusables outrank
    /// edit-distance matches; closer edits outrank farther ones.
    fn rank(&self) -> (u8, usize) {
        match self {
            SuspicionKind::SeparatorConfusable => (0, 0),
            SuspicionKind::EditDistance(d) => (1, *d),
        }
    }
}

/// A known package name the query closely resembles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suspicion {
    pub candidate: String,
    pub kind: SuspicionKind,
}

/// Damerau–Levenshtein distance restricted to the optimal-string-alignment
/// variant (handles a single transposition of adjacent characters), computed
/// over Unicode scalar values.
pub fn osa_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (n, m) = (a.len(), b.len());
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }
    // Three rolling rows: prev-prev (for transposition), prev, and current.
    let mut prev2 = vec![0usize; m + 1];
    let mut prev = (0..=m).collect::<Vec<_>>();
    let mut cur = vec![0usize; m + 1];
    for i in 1..=n {
        cur[0] = i;
        for j in 1..=m {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            let mut best = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                best = best.min(prev2[j - 2] + 1);
            }
            cur[j] = best;
        }
        std::mem::swap(&mut prev2, &mut prev);
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[m]
}

/// Lowercase and collapse `-`/`_` to a single canonical separator so
/// separator-only differences compare equal.
fn separator_normalized(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '-' | '_' => '_',
            other => other.to_ascii_lowercase(),
        })
        .collect()
}

/// Return the known names that `query` closely resembles, best match first.
///
/// `query` is assumed to be an *unknown* name (an exact match is never a
/// near-miss and is filtered out). A match is reported when:
/// - it is separator-confusable with `query`, or
/// - its OSA distance to `query` is in `1..=max_distance` **and** strictly less
///   than the longer of the two lengths (so unrelated short names are not
///   flagged just because everything is within two edits of everything).
///
/// Ordering is deterministic: separator confusables first, then by ascending
/// edit distance, then by candidate name.
pub fn nearest<'a, I>(query: &str, known: I, max_distance: usize) -> Vec<Suspicion>
where
    I: IntoIterator<Item = &'a str>,
{
    let q_norm = separator_normalized(query);
    let mut out: Vec<Suspicion> = Vec::new();
    for candidate in known {
        if candidate == query {
            continue;
        }
        if separator_normalized(candidate) == q_norm {
            out.push(Suspicion {
                candidate: candidate.to_string(),
                kind: SuspicionKind::SeparatorConfusable,
            });
            continue;
        }
        let d = osa_distance(query, candidate);
        let longer = query.chars().count().max(candidate.chars().count());
        if d >= 1 && d <= max_distance && d < longer {
            out.push(Suspicion {
                candidate: candidate.to_string(),
                kind: SuspicionKind::EditDistance(d),
            });
        }
    }
    out.sort_by(|a, b| {
        a.kind
            .rank()
            .cmp(&b.kind.rank())
            .then_with(|| a.candidate.cmp(&b.candidate))
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn osa_handles_transposition_and_edits() {
        assert_eq!(osa_distance("requests", "requests"), 0);
        assert_eq!(osa_distance("reqests", "requests"), 1); // deletion
        assert_eq!(osa_distance("reuqests", "requests"), 1); // transposition
        assert_eq!(osa_distance("", "abc"), 3);
    }

    #[test]
    fn flags_a_single_typo() {
        let known = ["requests", "numpy", "left_pad"];
        let hits = nearest("reqests", known, 2);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].candidate, "requests");
        assert_eq!(hits[0].kind, SuspicionKind::EditDistance(1));
    }

    #[test]
    fn flags_separator_confusable_first() {
        let known = ["left_pad", "left_pat"];
        let hits = nearest("left-pad", known, 2);
        // `left_pad` is separator-confusable (rank 0); `left_pat` is distance 1.
        assert_eq!(hits[0].candidate, "left_pad");
        assert_eq!(hits[0].kind, SuspicionKind::SeparatorConfusable);
    }

    #[test]
    fn exact_match_is_not_a_near_miss() {
        let known = ["requests", "numpy"];
        assert!(nearest("requests", known, 2).is_empty());
    }

    #[test]
    fn distant_and_tiny_names_are_not_flagged() {
        let known = ["numpy", "ab"];
        assert!(nearest("requests", known, 2).is_empty());
        // "cd" vs "ab": distance 2 but not < longer length (2) -> not flagged.
        assert!(nearest("cd", ["ab"], 2).is_empty());
    }

    #[test]
    fn ordering_is_deterministic_best_first() {
        let known = ["aaa", "aab", "aXc"];
        // query "aac": "aab" d1, "aaa" d1, "aXc" d1 -> all distance 1, name order.
        let hits = nearest("aac", known, 2);
        let names: Vec<&str> = hits.iter().map(|h| h.candidate.as_str()).collect();
        assert_eq!(names, ["aXc", "aaa", "aab"]);
    }
}
