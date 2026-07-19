//! Minimum Shelf: one frozen Core Ring Tier 1 tool.
//!
//! Slice 1 deliberately contains no transport, package discovery, registry,
//! or hosted-service surface. The sealed package boundary is added only after
//! this in-process interpreter path is mechanically proven.

pub const TIER1_TOOL_NAME: &str = "garnet.core.double";

use garnet_interp::{Interpreter, Value as GarnetValue};
use serde_json::{json, Value};

fn invoke_tier1_source(source: &str, arguments: &Value) -> Result<Value, String> {
    let object = arguments
        .as_object()
        .ok_or_else(|| "Tier 1 arguments must be an object".to_string())?;
    if object.len() != 1 || !object.contains_key("value") {
        return Err("Tier 1 arguments must contain exactly `value`".to_string());
    }
    let input = object["value"]
        .as_i64()
        .ok_or_else(|| "Tier 1 `value` must be a signed 64-bit integer".to_string())?;

    let evaluated = crate::panic_firewall::firewalled(|| {
        let mut interpreter = Interpreter::new();
        interpreter
            .load_source_with_entry_caps(source, "main")
            .map_err(|error| format!("Tier 1 load failed: {error}"))?;
        interpreter
            .call_entry("main", vec![GarnetValue::Int(input)])
            .map_err(|error| format!("Tier 1 execution failed: {error}"))
    })
    .map_err(|panic| format!("Tier 1 interpreter panic: {panic}"))??;

    match evaluated {
        GarnetValue::Int(value) => Ok(json!({"value": value})),
        other => Err(format!(
            "Tier 1 entry must return an integer, got {}",
            other.type_name()
        )),
    }
}

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
