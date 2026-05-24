//! `core::cmp` — ordering helpers (Layer 0, no caps).
//!
//! Generic over any `PartialOrd` host type; the interpreter instantiates
//! these for Garnet's comparable values. `@stability(experimental)`.

/// The lesser of two values (returns `a` on a tie).
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if b < a {
        b
    } else {
        a
    }
}

/// The greater of two values (returns `a` on a tie).
pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if b > a {
        b
    } else {
        a
    }
}

/// Clamp `v` into the inclusive `[lo, hi]` range. `lo` must not exceed `hi`.
pub fn clamp<T: PartialOrd>(v: T, lo: T, hi: T) -> T {
    if v < lo {
        lo
    } else if v > hi {
        hi
    } else {
        v
    }
}

/// Three-way compare: `-1` if `a < b`, `0` if equal, `1` if `a > b`.
/// Returns `0` for incomparable values (e.g. NaN), matching Garnet's
/// total-order-by-default surface contract.
pub fn ordering<T: PartialOrd>(a: T, b: T) -> i32 {
    if a < b {
        -1
    } else if a > b {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn min_max_pick_correctly() {
        assert_eq!(min(3, 7), 3);
        assert_eq!(min(7, 3), 3);
        assert_eq!(max(3, 7), 7);
        assert_eq!(max(7, 3), 7);
    }

    #[test]
    fn min_max_work_on_strings() {
        assert_eq!(min("apple", "banana"), "apple");
        assert_eq!(max("apple", "banana"), "banana");
    }

    #[test]
    fn clamp_bounds() {
        assert_eq!(clamp(5, 0, 10), 5);
        assert_eq!(clamp(-3, 0, 10), 0);
        assert_eq!(clamp(99, 0, 10), 10);
        assert_eq!(clamp(0, 0, 10), 0);
        assert_eq!(clamp(10, 0, 10), 10);
    }

    #[test]
    fn ordering_three_way() {
        assert_eq!(ordering(1, 2), -1);
        assert_eq!(ordering(2, 2), 0);
        assert_eq!(ordering(3, 2), 1);
    }

    #[test]
    fn ordering_incomparable_is_zero() {
        assert_eq!(ordering(f64::NAN, 1.0), 0);
    }
}
