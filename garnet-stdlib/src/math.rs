//! `core::math` — total numeric functions (Layer 0, no caps).
//!
//! These are pure host helpers over `f64`. The interpreter marshals Garnet
//! numbers into `f64` and back; the surface contract is documented in
//! `GARNET_STDLIB_LAYER_POLICY.md` (Layer 0). `@stability(experimental)`.

use crate::StdError;

pub fn abs(x: f64) -> f64 {
    x.abs()
}

/// Square root. Errors on negative input rather than returning NaN, so the
/// managed-mode bridge surfaces a catchable error instead of a silent NaN.
pub fn sqrt(x: f64) -> Result<f64, StdError> {
    if x < 0.0 {
        return Err(StdError::Arithmetic(format!(
            "sqrt of negative number: {x}"
        )));
    }
    Ok(x.sqrt())
}

pub fn pow(base: f64, exp: f64) -> f64 {
    base.powf(exp)
}

pub fn floor(x: f64) -> f64 {
    x.floor()
}

pub fn ceil(x: f64) -> f64 {
    x.ceil()
}

/// Round half away from zero (matches Garnet's documented rounding mode,
/// which is `f64::round`, not banker's rounding).
pub fn round(x: f64) -> f64 {
    x.round()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abs_handles_sign_and_zero() {
        assert_eq!(abs(-3.5), 3.5);
        assert_eq!(abs(3.5), 3.5);
        assert_eq!(abs(0.0), 0.0);
    }

    #[test]
    fn sqrt_exact_and_irrational() {
        assert_eq!(sqrt(4.0).unwrap(), 2.0);
        assert_eq!(sqrt(0.0).unwrap(), 0.0);
        let r = sqrt(2.0).unwrap();
        assert!((r * r - 2.0).abs() < 1e-12);
    }

    #[test]
    fn sqrt_negative_is_error() {
        match sqrt(-1.0) {
            Err(StdError::Arithmetic(_)) => {}
            other => panic!("expected Arithmetic error, got {other:?}"),
        }
    }

    #[test]
    fn pow_identities() {
        assert_eq!(pow(2.0, 10.0), 1024.0);
        assert_eq!(pow(5.0, 0.0), 1.0);
        assert_eq!(pow(9.0, 0.5), 3.0);
    }

    #[test]
    fn floor_ceil_round_boundaries() {
        assert_eq!(floor(2.9), 2.0);
        assert_eq!(floor(-2.1), -3.0);
        assert_eq!(ceil(2.1), 3.0);
        assert_eq!(ceil(-2.9), -2.0); // ceil moves toward +inf
                                      // round-half-away-from-zero
        assert_eq!(round(2.5), 3.0);
        assert_eq!(round(-2.5), -3.0);
        assert_eq!(round(2.4), 2.0);
    }
}
