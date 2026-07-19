//! Minimum Shelf: one frozen Core Ring Tier 1 tool.
//!
//! The application host binds the released MCP lifecycle to this one tool.
//! Package discovery and seal verification remain a separate, fail-closed
//! boundary; no registry or hosted-service surface exists here.

pub const TIER1_TOOL_NAME: &str = "garnet.core.double";

use garnet_interp::{Interpreter, Value as GarnetValue};
use serde_json::{json, Value};

use crate::mcp::{McpAction, McpApplicationResponse, McpSession};

struct Tier1Tool {
    source: String,
}

impl Tier1Tool {
    fn invoke(&self, arguments: &Value) -> Result<Value, String> {
        invoke_tier1_source(&self.source, arguments)
    }
}

pub(crate) struct MinimumShelfHost {
    session: McpSession,
    tool: Tier1Tool,
}

impl MinimumShelfHost {
    pub(crate) fn from_verified_source(source: String) -> Self {
        Self {
            session: McpSession::with_capabilities(json!({"tools":{"listChanged":false}})),
            tool: Tier1Tool { source },
        }
    }

    pub(crate) fn handle_message(&mut self, input: &str) -> McpAction {
        let tool = &self.tool;
        self.session
            .handle_message_with(input, &mut |method, params| {
                handle_tool_request(tool, method, params)
            })
    }
}

fn handle_tool_request(
    tool: &Tier1Tool,
    method: &str,
    params: Option<&Value>,
) -> Option<McpApplicationResponse> {
    match method {
        "tools/list" => Some(if crate::mcp_schema::valid_request_params(params) {
            McpApplicationResponse::Result(json!({
                "tools": [{
                    "name": TIER1_TOOL_NAME,
                    "title": "Garnet Core Double",
                    "description": "Doubles one signed 64-bit integer in the Garnet interpreter.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {"value": {"type": "integer"}},
                        "required": ["value"],
                        "additionalProperties": false
                    },
                    "outputSchema": {
                        "type": "object",
                        "properties": {"value": {"type": "integer"}},
                        "required": ["value"],
                        "additionalProperties": false
                    }
                }]
            }))
        } else {
            application_error(-32602, "Invalid tools/list params")
        }),
        "tools/call" => Some(call_tool(tool, params)),
        _ => None,
    }
}

fn call_tool(tool: &Tier1Tool, params: Option<&Value>) -> McpApplicationResponse {
    let Some(params) = params.and_then(Value::as_object) else {
        return application_error(-32602, "Invalid tools/call params");
    };
    if params.len() != 2
        || params.get("name").and_then(Value::as_str) != Some(TIER1_TOOL_NAME)
        || !params.get("arguments").is_some_and(Value::is_object)
    {
        return application_error(-32602, "Invalid tools/call params");
    }
    let arguments = &params["arguments"];
    match tool.invoke(arguments) {
        Ok(structured) => {
            let rendered = structured["value"].to_string();
            McpApplicationResponse::Result(json!({
                "content": [{"type":"text", "text": rendered}],
                "isError": false,
                "structuredContent": structured
            }))
        }
        Err(message) => application_error(-32602, &message),
    }
}

fn application_error(code: i64, message: &str) -> McpApplicationResponse {
    McpApplicationResponse::Error {
        code,
        message: message.to_string(),
    }
}

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

    #[test]
    fn minimum_shelf_router_rejects_wrong_tool_and_params() {
        let tool = Tier1Tool {
            source: "@caps()\ndef main(value) { value * 2 }\n".to_string(),
        };
        for params in [
            None,
            Some(json!({})),
            Some(json!({"name":"other","arguments":{"value":21}})),
            Some(json!({
                "name": TIER1_TOOL_NAME,
                "arguments": {"value":21},
                "extra": true
            })),
        ] {
            let response = call_tool(&tool, params.as_ref());
            assert!(
                matches!(response, McpApplicationResponse::Error { code: -32602, .. }),
                "{response:?}"
            );
        }
    }
}
