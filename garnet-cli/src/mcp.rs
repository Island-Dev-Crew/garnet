//! Released-MCP lifecycle core with an explicit, opt-in application boundary.
use crate::mcp_schema;
use serde_json::{json, Map, Value};
use std::collections::HashSet;

pub use crate::mcp_schema::MCP_PROTOCOL_VERSION;

const REQUEST_ID_LIMIT: usize = 1024;
const REQUEST_ID_STRING_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    AwaitInitialize,
    AwaitInitialized,
    Ready,
    Closed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum McpAction {
    Respond(String),
    NoResponse,
    Close(Option<String>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum McpApplicationResponse {
    Result(Value),
    Error { code: i64, message: String },
}

pub struct McpSession {
    phase: Phase,
    seen_ids: HashSet<Value>,
    capabilities: Value,
}

impl Default for McpSession {
    fn default() -> Self {
        Self::new()
    }
}

impl McpSession {
    pub fn new() -> Self {
        Self::with_capabilities(json!({}))
    }

    pub(crate) fn with_capabilities(capabilities: Value) -> Self {
        Self {
            phase: Phase::AwaitInitialize,
            seen_ids: HashSet::new(),
            capabilities,
        }
    }

    pub fn handle_message(&mut self, input: &str) -> McpAction {
        self.handle_message_with(input, &mut |_, _| None)
    }

    pub(crate) fn handle_message_with<F>(&mut self, input: &str, handler: &mut F) -> McpAction
    where
        F: FnMut(&str, Option<&Value>) -> Option<McpApplicationResponse>,
    {
        if self.phase == Phase::Closed {
            return McpAction::Close(None);
        }
        let value = match serde_json::from_str::<Value>(input) {
            Ok(value) => value,
            Err(_) => return McpAction::Respond(error(Value::Null, -32700, "Parse error")),
        };
        let Some(message) = value.as_object() else {
            return McpAction::Respond(error(Value::Null, -32600, "Invalid Request"));
        };
        if response_shaped(message) {
            self.phase = Phase::Closed;
            return McpAction::Close(None);
        }
        let readable_id = message
            .get("id")
            .filter(|id| valid_id(id))
            .cloned()
            .unwrap_or(Value::Null);
        if !valid_envelope(message) {
            if !message.contains_key("id")
                && message.get("jsonrpc").and_then(Value::as_str) == Some("2.0")
                && message.get("method").is_some_and(Value::is_string)
            {
                return self.handle_notification("", None);
            }
            return McpAction::Respond(error(readable_id, -32600, "Invalid Request"));
        }
        let Some(method) = message.get("method").and_then(Value::as_str) else {
            return McpAction::Respond(error(readable_id, -32600, "Invalid Request"));
        };
        let params = message.get("params");
        match message.get("id") {
            Some(id) if valid_id(id) => self.handle_request(id.clone(), method, params, handler),
            Some(_) => McpAction::Respond(error(Value::Null, -32600, "Invalid Request")),
            None => self.handle_notification(method, params),
        }
    }

    fn handle_request<F>(
        &mut self,
        id: Value,
        method: &str,
        params: Option<&Value>,
        handler: &mut F,
    ) -> McpAction
    where
        F: FnMut(&str, Option<&Value>) -> Option<McpApplicationResponse>,
    {
        if self.seen_ids.contains(&id) {
            return McpAction::Respond(error(id, -32600, "Request id already used"));
        }
        if self.seen_ids.len() >= REQUEST_ID_LIMIT {
            self.phase = Phase::Closed;
            return McpAction::Close(Some(error(id, -32000, "Request id budget exhausted")));
        }
        self.seen_ids.insert(id.clone());
        match self.phase {
            Phase::AwaitInitialize if method == "initialize" => self.initialize(id, params),
            Phase::AwaitInitialize => {
                self.phase = Phase::Closed;
                McpAction::Close(Some(error(
                    id,
                    -32002,
                    "Initialize must be the first request",
                )))
            }
            Phase::AwaitInitialized => match method {
                "initialize" => {
                    McpAction::Respond(error(id, -32600, "Initialize already received"))
                }
                "ping" => ping(id, params),
                _ => McpAction::Respond(error(id, -32002, "Server not initialized")),
            },
            Phase::Ready => match method {
                "initialize" => {
                    McpAction::Respond(error(id, -32600, "Initialize already received"))
                }
                "ping" => ping(id, params),
                _ => match handler(method, params) {
                    Some(McpApplicationResponse::Result(value)) => {
                        McpAction::Respond(result(id, value))
                    }
                    Some(McpApplicationResponse::Error { code, message }) => {
                        McpAction::Respond(error(id, code, &message))
                    }
                    None => McpAction::Respond(error(id, -32601, "Method not found")),
                },
            },
            Phase::Closed => McpAction::Close(None),
        }
    }

    fn handle_notification(&mut self, method: &str, params: Option<&Value>) -> McpAction {
        match self.phase {
            Phase::AwaitInitialize => {
                self.phase = Phase::Closed;
                McpAction::Close(None)
            }
            Phase::AwaitInitialized => {
                if method == "notifications/initialized"
                    && mcp_schema::valid_initialized_notification_params(params)
                {
                    self.phase = Phase::Ready;
                }
                McpAction::NoResponse
            }
            Phase::Ready => McpAction::NoResponse,
            Phase::Closed => McpAction::Close(None),
        }
    }

    fn initialize(&mut self, id: Value, params: Option<&Value>) -> McpAction {
        if !mcp_schema::valid_initialize_params(params) {
            return McpAction::Respond(error(id, -32602, "Invalid initialize params"));
        }
        self.phase = Phase::AwaitInitialized;
        McpAction::Respond(result(
            id,
            json!({
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "capabilities": self.capabilities.clone(),
                "serverInfo": {
                    "name": "garnet-minimum-shelf",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        ))
    }
}

fn valid_envelope(message: &Map<String, Value>) -> bool {
    message
        .keys()
        .all(|key| ["jsonrpc", "id", "method", "params"].contains(&key.as_str()))
        && message.get("jsonrpc").and_then(Value::as_str) == Some("2.0")
        && message.get("method").is_some_and(Value::is_string)
        && message.get("params").is_none_or(Value::is_object)
}

fn response_shaped(message: &Map<String, Value>) -> bool {
    !message.contains_key("method")
        && (message.contains_key("result") || message.contains_key("error"))
}

fn valid_id(id: &Value) -> bool {
    match id {
        Value::String(value) => value.len() <= REQUEST_ID_STRING_BYTES,
        Value::Number(value) => value.as_i64().is_some() || value.as_u64().is_some(),
        _ => false,
    }
}

fn ping(id: Value, params: Option<&Value>) -> McpAction {
    if mcp_schema::valid_request_params(params) {
        McpAction::Respond(result(id, json!({})))
    } else {
        McpAction::Respond(error(id, -32602, "Invalid ping params"))
    }
}

fn result(id: Value, value: Value) -> String {
    json!({"jsonrpc":"2.0","id":id,"result":value}).to_string()
}

fn error(id: Value, code: i64, message: &str) -> String {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}}).to_string()
}
