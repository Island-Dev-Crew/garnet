//! Minimum Shelf: one frozen Core Ring Tier 1 tool.
//!
//! The application host binds the released MCP lifecycle to this one tool.
//! Package discovery and seal verification remain a separate, fail-closed
//! boundary; no registry or hosted-service surface exists here.

pub const TIER1_TOOL_NAME: &str = "garnet.core.double";

use garnet_check::capability_surface;
use garnet_interp::{Interpreter, Value as GarnetValue};
use serde_json::{json, Value};
use std::fmt;
use std::fs;
use std::path::Path;

use crate::cap_manifest::CapabilityManifest;
use crate::manifest::Manifest;
use crate::mcp::{McpAction, McpApplicationResponse, McpSession};

const PACKAGE_SCHEMA: &str = "garnet.minimum-shelf-package/v1";
const PACKAGE_NAME: &str = "garnet-minimum-shelf-flagship";
const PACKAGE_RING: &str = "core";
const PACKAGE_TIER: u64 = 1;
const PACKAGE_SOURCE: &str = "tool.garnet";
const PACKAGE_SEAL: &str = "tool.seal.json";
const PACKAGE_SEAL_KIND: &str = "in-toto-predicate-unsigned";
const MAX_PACKAGE_FILE_BYTES: u64 = 1024 * 1024;
const TRUSTED_SOURCE_BLAKE3: &str =
    "38db718f35ec1dd034010a9c3d52b4236ce30523a117abefdf41196d5cc9cce9";
const TRUSTED_SEAL_BLAKE3: &str =
    "9e54835294d37829da8deb294dd1bac353325d2b18342c37ab22413ea482fffa";
const TRUSTED_MANIFEST_BLAKE3: &str =
    "8d81f053bc09c36285b8ef7d3f630c1642923300c0f8073e4b25744b0e79d770";

#[derive(Debug)]
pub struct MinimumShelfPackage {
    source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageError(String);

impl fmt::Display for PackageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Minimum Shelf package rejected: {}", self.0)
    }
}

impl std::error::Error for PackageError {}

impl MinimumShelfPackage {
    pub fn load(root: &Path) -> Result<Self, PackageError> {
        reject_symlink(root, "package root")?;
        if !root.is_dir() {
            return Err(reject("package root is not a directory"));
        }

        let manifest_bytes = read_regular(&root.join("SHELF_PACKAGE.json"), "package manifest")?;
        if canonical_text_blake3(&manifest_bytes)? != TRUSTED_MANIFEST_BLAKE3 {
            return Err(reject(
                "package manifest bytes do not match the trusted flagship",
            ));
        }
        let manifest: Value = serde_json::from_slice(&manifest_bytes)
            .map_err(|error| reject(format!("package manifest is invalid JSON: {error}")))?;
        verify_package_manifest(&manifest)?;

        let source_bytes = read_regular(&root.join(PACKAGE_SOURCE), "source")?;
        if blake3_hex(&source_bytes) != TRUSTED_SOURCE_BLAKE3 {
            return Err(reject("source bytes do not match the trusted flagship"));
        }
        let seal_bytes = read_regular(&root.join(PACKAGE_SEAL), "seal")?;
        if canonical_text_blake3(&seal_bytes)? != TRUSTED_SEAL_BLAKE3 {
            return Err(reject("seal bytes do not match the trusted flagship"));
        }

        let source =
            String::from_utf8(source_bytes).map_err(|_| reject("source is not valid UTF-8"))?;
        let module = garnet_parser::parse_source(&source)
            .map_err(|error| reject(format!("source does not parse: {error}")))?;
        let build = Manifest::build(&source, &module);
        let caps = CapabilityManifest::from_surface(capability_surface(&module));
        verify_seal(&seal_bytes, &build, &caps)?;

        Ok(Self { source })
    }

    pub fn tool_name(&self) -> &'static str {
        TIER1_TOOL_NAME
    }

    pub fn ring_tier(&self) -> u64 {
        PACKAGE_TIER
    }

    pub(crate) fn into_host(self) -> MinimumShelfHost {
        MinimumShelfHost::from_verified_source(self.source)
    }
}

fn verify_package_manifest(manifest: &Value) -> Result<(), PackageError> {
    let Some(object) = manifest.as_object() else {
        return Err(reject("package manifest must be an object"));
    };
    const KEYS: &[&str] = &[
        "name",
        "ring",
        "schema",
        "seal",
        "sealBlake3",
        "sealKind",
        "source",
        "sourceBlake3",
        "tier",
        "tool",
    ];
    if object.len() != KEYS.len() || !object.keys().all(|key| KEYS.contains(&key.as_str())) {
        return Err(reject("package manifest fields are not exact"));
    }
    for (key, expected) in [
        ("schema", PACKAGE_SCHEMA),
        ("name", PACKAGE_NAME),
        ("ring", PACKAGE_RING),
        ("tool", TIER1_TOOL_NAME),
        ("source", PACKAGE_SOURCE),
        ("seal", PACKAGE_SEAL),
        ("sourceBlake3", TRUSTED_SOURCE_BLAKE3),
        ("sealBlake3", TRUSTED_SEAL_BLAKE3),
        ("sealKind", PACKAGE_SEAL_KIND),
    ] {
        if object.get(key).and_then(Value::as_str) != Some(expected) {
            return Err(reject(format!(
                "package manifest `{key}` does not match the trusted flagship"
            )));
        }
    }
    if object.get("tier").and_then(Value::as_u64) != Some(PACKAGE_TIER) {
        return Err(reject(
            "package manifest `tier` does not match the trusted flagship",
        ));
    }
    Ok(())
}

fn verify_seal(
    bytes: &[u8],
    build: &Manifest,
    caps: &CapabilityManifest,
) -> Result<(), PackageError> {
    let seal: Value = serde_json::from_slice(bytes)
        .map_err(|error| reject(format!("seal is invalid JSON: {error}")))?;
    let Some(root) = seal.as_object() else {
        return Err(reject("seal must be an object"));
    };
    if root.len() != 4
        || root.get("_type").and_then(Value::as_str) != Some(crate::seal::STATEMENT_TYPE)
        || root.get("predicateType").and_then(Value::as_str) != Some(crate::seal::PREDICATE_TYPE)
    {
        return Err(reject("seal statement envelope is not exact"));
    }
    let Some(subjects) = root.get("subject").and_then(Value::as_array) else {
        return Err(reject("seal subject is missing"));
    };
    if subjects.len() != 1
        || subjects[0].get("name").and_then(Value::as_str) != Some("tool")
        || subjects[0]
            .pointer("/digest/blake3")
            .and_then(Value::as_str)
            != Some(build.ast_hash.as_str())
    {
        return Err(reject("seal subject does not bind the flagship AST"));
    }
    let Some(predicate) = root.get("predicate").and_then(Value::as_object) else {
        return Err(reject("seal predicate is missing"));
    };
    if predicate.len() != 4
        || predicate.get("source_blake3").and_then(Value::as_str)
            != Some(build.source_hash.as_str())
    {
        return Err(reject("seal predicate does not bind the flagship source"));
    }
    let expected_build: Value = serde_json::from_str(&build.to_canonical_json())
        .map_err(|error| reject(format!("internal build manifest is invalid: {error}")))?;
    if predicate.get("build_manifest") != Some(&expected_build) {
        return Err(reject("seal build manifest does not match current Garnet"));
    }
    let expected_caps: Value = serde_json::from_str(&caps.to_json())
        .map_err(|error| reject(format!("internal capability manifest is invalid: {error}")))?;
    if predicate.get("capability_manifest") != Some(&expected_caps) {
        return Err(reject(
            "seal capability manifest is not the empty Tier 1 surface",
        ));
    }
    if !predicate
        .get("tooling")
        .and_then(|tooling| tooling.get("cosign"))
        .and_then(Value::as_str)
        .is_some_and(|note| note.contains("UNSIGNED"))
    {
        return Err(reject("seal must state its unsigned status honestly"));
    }
    Ok(())
}

fn read_regular(path: &Path, label: &str) -> Result<Vec<u8>, PackageError> {
    reject_symlink(path, label)?;
    let metadata =
        fs::metadata(path).map_err(|error| reject(format!("{label} is unavailable: {error}")))?;
    if !metadata.is_file() {
        return Err(reject(format!("{label} is not a regular file")));
    }
    if metadata.len() > MAX_PACKAGE_FILE_BYTES {
        return Err(reject(format!("{label} exceeds the byte limit")));
    }
    fs::read(path).map_err(|error| reject(format!("{label} could not be read: {error}")))
}

fn reject_symlink(path: &Path, label: &str) -> Result<(), PackageError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(reject(format!("{label} must not be a symlink")))
        }
        Ok(_) => Ok(()),
        Err(error) => Err(reject(format!("{label} is unavailable: {error}"))),
    }
}

fn canonical_text_blake3(bytes: &[u8]) -> Result<String, PackageError> {
    let text = std::str::from_utf8(bytes).map_err(|_| reject("seal is not valid UTF-8"))?;
    Ok(blake3_hex(text.replace("\r\n", "\n").as_bytes()))
}

fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn reject(message: impl Into<String>) -> PackageError {
    PackageError(message.into())
}

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
