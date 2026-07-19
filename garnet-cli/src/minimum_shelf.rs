//! Minimum Shelf: one frozen Core Ring Tier 1 tool.
//!
//! Slice 1 deliberately contains no transport, package discovery, registry,
//! or hosted-service surface. The sealed package boundary is added only after
//! this in-process interpreter path is mechanically proven.

pub const TIER1_TOOL_NAME: &str = "garnet.core.double";

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn minimum_shelf_tier1_tool_invokes_garnet_in_process() {
        let source = "@caps()\ndef main(value) { value * 2 }\n";
        let result = invoke_tier1_source(source, &json!({"value": 21}))
            .expect("frozen Tier 1 tool should execute");
        assert_eq!(result, json!({"value": 42}));
        assert_eq!(TIER1_TOOL_NAME, "garnet.core.double");
    }

    #[test]
    fn minimum_shelf_tier1_input_is_exact_and_bounded() {
        let source = "@caps()\ndef main(value) { value * 2 }\n";
        for invalid in [
            json!({}),
            json!({"value": 1, "extra": true}),
            json!({"value": "21"}),
            json!({"value": 9223372036854775808_u64}),
        ] {
            assert!(invoke_tier1_source(source, &invalid).is_err(), "{invalid}");
        }
    }
}
