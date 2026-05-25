//! `core::option` — `Option` combinators (Layer 0, no caps).
//!
//! Pure host helpers over Rust `Option`, which the interpreter maps onto
//! Garnet's `Option<T>` (`Some`/`None`). `@stability(experimental)`.

/// Wrap a value as `Some`.
pub fn some<T>(value: T) -> Option<T> {
    Some(value)
}

/// The `None` value.
pub fn none<T>() -> Option<T> {
    None
}

/// Transform the `Some` value, leaving `None` untouched.
pub fn map<T, U, F: Fn(T) -> U>(o: Option<T>, f: F) -> Option<U> {
    o.map(f)
}

/// Chain an `Option`-returning function on the `Some` value (monadic bind).
pub fn and_then<T, U, F: Fn(T) -> Option<U>>(o: Option<T>, f: F) -> Option<U> {
    o.and_then(f)
}

/// Return the `Some` value, or `default` if `None`.
pub fn unwrap_or<T>(o: Option<T>, default: T) -> T {
    o.unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn some_none_constructors() {
        assert_eq!(some(5), Some(5));
        assert_eq!(none::<i32>(), None);
    }

    #[test]
    fn map_only_touches_some() {
        assert_eq!(map(Some(3), |x| x + 1), Some(4));
        assert_eq!(map(None::<i32>, |x| x + 1), None);
    }

    #[test]
    fn and_then_chains_and_short_circuits() {
        let nonempty = |s: &str| if s.is_empty() { None } else { Some(s.len()) };
        assert_eq!(and_then(Some("hi"), nonempty), Some(2));
        assert_eq!(and_then(Some(""), nonempty), None);
        assert_eq!(and_then(None::<&str>, nonempty), None);
    }

    #[test]
    fn unwrap_or_supplies_default() {
        assert_eq!(unwrap_or(Some(7), 0), 7);
        assert_eq!(unwrap_or(None, 0), 0);
    }
}
