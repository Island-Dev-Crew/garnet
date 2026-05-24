//! `core::iter` — iterator combinators (Layer 0, no caps).
//!
//! Pure host helpers over `Vec`/iterators. The higher-order combinators
//! (`map`, `filter`, `fold`) are generic over a Rust closure; the
//! interpreter supplies a closure that invokes the Garnet callable, so the
//! combinator itself stays a small, totally-tested function and only the
//! closure bridge is interpreter-side. `@stability(experimental)`.

/// Apply `f` to each element, returning a new sequence.
pub fn map<T, U, F: Fn(T) -> U>(items: Vec<T>, f: F) -> Vec<U> {
    items.into_iter().map(f).collect()
}

/// Keep elements for which `pred` returns true.
pub fn filter<T, F: Fn(&T) -> bool>(items: Vec<T>, pred: F) -> Vec<T> {
    items.into_iter().filter(|x| pred(x)).collect()
}

/// Reduce a sequence to a single value with an accumulator seeded by `init`.
pub fn fold<T, A, F: Fn(A, T) -> A>(items: Vec<T>, init: A, f: F) -> A {
    items.into_iter().fold(init, f)
}

/// Pair up elements of two sequences, truncating to the shorter.
pub fn zip<A, B>(a: Vec<A>, b: Vec<B>) -> Vec<(A, B)> {
    a.into_iter().zip(b).collect()
}

/// Take the first `n` elements (or all, if fewer).
pub fn take<T>(items: Vec<T>, n: usize) -> Vec<T> {
    items.into_iter().take(n).collect()
}

/// Skip the first `n` elements, returning the rest.
pub fn drop<T>(items: Vec<T>, n: usize) -> Vec<T> {
    items.into_iter().skip(n).collect()
}

/// Materialize any iterable into an owned `Vec`.
pub fn collect<T, I: IntoIterator<Item = T>>(items: I) -> Vec<T> {
    items.into_iter().collect()
}

/// Concatenate two sequences end to end.
pub fn chain<T>(a: Vec<T>, b: Vec<T>) -> Vec<T> {
    a.into_iter().chain(b).collect()
}

/// Pair each element with its zero-based index.
pub fn enumerate<T>(items: Vec<T>) -> Vec<(usize, T)> {
    items.into_iter().enumerate().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_applies_closure() {
        assert_eq!(map(vec![1, 2, 3], |x| x * 2), vec![2, 4, 6]);
        let lens = map(vec!["a", "bb", "ccc"], |s| s.len());
        assert_eq!(lens, vec![1, 2, 3]);
    }

    #[test]
    fn filter_keeps_matching() {
        assert_eq!(filter(vec![1, 2, 3, 4], |x| x % 2 == 0), vec![2, 4]);
        assert!(filter(vec![1, 3, 5], |x| x % 2 == 0).is_empty());
    }

    #[test]
    fn fold_accumulates() {
        assert_eq!(fold(vec![1, 2, 3, 4], 0, |acc, x| acc + x), 10);
        assert_eq!(fold(vec![1, 2, 3], 1, |acc, x| acc * x), 6);
        // fold over empty returns the seed
        assert_eq!(fold(Vec::<i32>::new(), 42, |acc, x| acc + x), 42);
    }

    #[test]
    fn zip_truncates_to_shorter() {
        assert_eq!(zip(vec![1, 2, 3], vec!["a", "b"]), vec![(1, "a"), (2, "b")]);
    }

    #[test]
    fn take_and_drop_partition() {
        assert_eq!(take(vec![1, 2, 3, 4], 2), vec![1, 2]);
        assert_eq!(take(vec![1, 2], 5), vec![1, 2]); // n > len
        assert_eq!(drop(vec![1, 2, 3, 4], 2), vec![3, 4]);
        assert!(drop(vec![1, 2], 5).is_empty());
    }

    #[test]
    fn chain_concatenates_in_order() {
        assert_eq!(chain(vec![1, 2], vec![3, 4]), vec![1, 2, 3, 4]);
    }

    #[test]
    fn collect_materializes() {
        let v: Vec<i32> = collect(0..3);
        assert_eq!(v, vec![0, 1, 2]);
    }

    #[test]
    fn enumerate_indexes() {
        assert_eq!(enumerate(vec!["a", "b"]), vec![(0, "a"), (1, "b")]);
    }
}
