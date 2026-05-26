#![forbid(unsafe_code)]
//! Feature-gated LLM advisory tier for Garnet compiler suggestions.
//!
//! The default build keeps the crate inert. Enable the `llm` feature to use the
//! provider-compatible request/response layer and the non-deterministic
//! suggestion report.

#[cfg(not(feature = "llm"))]
pub const LLM_FEATURE_DISABLED: &str =
    "garnet-suggest-llm is inert until built with the `llm` Cargo feature";

#[cfg(feature = "llm")]
mod llm;

#[cfg(feature = "llm")]
pub use llm::*;

#[cfg(all(test, not(feature = "llm")))]
mod tests {
    use super::*;

    #[test]
    fn default_build_is_inert() {
        assert!(LLM_FEATURE_DISABLED.contains("llm"));
    }
}
