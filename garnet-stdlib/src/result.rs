//! `core::result` — `Result` combinators (Layer 0, no caps).
//!
//! Pure host helpers over Rust `Result`, which the interpreter maps onto
//! Garnet's `Result<T, E>`. Higher-order combinators take a Rust closure
//! supplied by the interpreter's callable bridge. `@stability(experimental)`.

/// Wrap a value as `Ok`.
pub fn ok<T, E>(value: T) -> Result<T, E> {
    Ok(value)
}

/// Wrap a value as `Err`.
pub fn err<T, E>(error: E) -> Result<T, E> {
    Err(error)
}

/// Transform the `Ok` value, leaving `Err` untouched.
pub fn map<T, U, E, F: Fn(T) -> U>(r: Result<T, E>, f: F) -> Result<U, E> {
    r.map(f)
}

/// Chain a `Result`-returning function on the `Ok` value (monadic bind).
pub fn and_then<T, U, E, F: Fn(T) -> Result<U, E>>(r: Result<T, E>, f: F) -> Result<U, E> {
    r.and_then(f)
}

/// Recover from an `Err` with a `Result`-returning function.
pub fn or_else<T, E, G, F: Fn(E) -> Result<T, G>>(r: Result<T, E>, f: F) -> Result<T, G> {
    r.or_else(f)
}

/// Return the `Ok` value, or `default` if `Err`.
pub fn unwrap_or<T, E>(r: Result<T, E>, default: T) -> T {
    r.unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_err_constructors() {
        let a: Result<i32, String> = ok(5);
        let b: Result<i32, String> = err("nope".to_string());
        assert_eq!(a, Ok(5));
        assert_eq!(b, Err("nope".to_string()));
    }

    #[test]
    fn map_only_touches_ok() {
        let a: Result<i32, &str> = Ok(3);
        let b: Result<i32, &str> = Err("e");
        assert_eq!(map(a, |x| x * 2), Ok(6));
        assert_eq!(map(b, |x| x * 2), Err("e"));
    }

    #[test]
    fn and_then_chains_and_short_circuits() {
        let safe_div = |x: i32| if x == 0 { Err("zero") } else { Ok(100 / x) };
        assert_eq!(and_then(Ok(4), safe_div), Ok(25));
        assert_eq!(and_then(Ok(0), safe_div), Err("zero"));
        assert_eq!(and_then(Err("pre"), safe_div), Err("pre"));
    }

    #[test]
    fn or_else_recovers_from_err() {
        let recover = |_e: &str| -> Result<i32, &str> { Ok(0) };
        assert_eq!(or_else(Ok(9), recover), Ok(9));
        assert_eq!(or_else(Err("boom"), recover), Ok(0));
    }

    #[test]
    fn unwrap_or_supplies_default() {
        assert_eq!(unwrap_or(Ok::<i32, &str>(7), 0), 7);
        assert_eq!(unwrap_or(Err::<i32, &str>("e"), 0), 0);
    }
}
