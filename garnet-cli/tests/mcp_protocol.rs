use garnet_cli::mcp::{McpAction, McpSession, MCP_PROTOCOL_VERSION as V};
use serde_json::{json, Value};

fn init_params(version: Value) -> Value {
    json!({"protocolVersion":version,"capabilities":{"roots":{"listChanged":true},
        "vendor.example":{"mode":"strict"}},"clientInfo":{"name":"test-client",
        "version":"1","websiteUrl":"https://example.test"},"_meta":{"trace":"test"}})
}

fn message(id: Value, method: &str, params: Option<Value>) -> String {
    let mut value = json!({"jsonrpc":"2.0","id":id,"method":method});
    if let Some(params) = params {
        value["params"] = params;
    }
    value.to_string()
}

fn raw_response(action: McpAction) -> String {
    match action {
        McpAction::Respond(raw) | McpAction::Close(Some(raw)) => raw,
        other => panic!("expected response action, got {other:?}"),
    }
}

fn response(action: McpAction) -> Value {
    let raw = raw_response(action);
    assert!(!raw.contains(['\r', '\n']));
    let value: Value = serde_json::from_str(&raw).expect("response JSON");
    let object = value.as_object().expect("response object");
    assert_eq!(object.get("jsonrpc"), Some(&json!("2.0")));
    assert_eq!(object.contains_key("result"), !object.contains_key("error"));
    assert_eq!(object.len(), 3);
    value
}

fn rpc(session: &mut McpSession, id: Value, method: &str, params: Option<Value>) -> Value {
    response(session.handle_message(&message(id, method, params)))
}

fn initialize(session: &mut McpSession, id: Value, version: Value) -> Value {
    rpc(session, id, "initialize", Some(init_params(version)))
}

fn assert_error(
    session: &mut McpSession,
    id: Value,
    method: &str,
    params: Option<Value>,
    expected: i64,
) -> Value {
    let value = rpc(session, id, method, params);
    assert_eq!(value["error"]["code"], expected);
    value
}

fn notify(session: &mut McpSession, method: &str, params: Option<Value>) -> McpAction {
    let mut value = json!({"jsonrpc":"2.0","method":method});
    if let Some(params) = params {
        value["params"] = params;
    }
    session.handle_message(&value.to_string())
}

#[test]
fn malformed_input_errors_and_orphan_responses_close_silently() {
    let parse = response(McpSession::new().handle_message("{"));
    assert_eq!(
        (&parse["error"]["code"], &parse["id"]),
        (&json!(-32700), &Value::Null)
    );
    let mut orphan = McpSession::new();
    let raw = json!({"jsonrpc":"2.0","id":1,"result":{}}).to_string();
    assert_eq!(orphan.handle_message(&raw), McpAction::Close(None));
    assert_eq!(
        orphan.handle_message(&message(
            json!(1),
            "initialize",
            Some(init_params(json!(V)))
        )),
        McpAction::Close(None)
    );
}

#[test]
fn initialize_is_first_schema_validated_and_version_negotiated() {
    let mut request_first = McpSession::new();
    assert_error(&mut request_first, json!(1), "ping", None, -32002);
    assert_eq!(
        request_first.handle_message(&message(
            json!(2),
            "initialize",
            Some(init_params(json!(V)))
        )),
        McpAction::Close(None)
    );
    let mut notification_first = McpSession::new();
    assert_eq!(
        notify(&mut notification_first, "unknown", Some(json!([]))),
        McpAction::Close(None)
    );

    let mut session = McpSession::new();
    let mut malformed = init_params(json!(V));
    malformed["clientInfo"]["icons"] = json!([7]);
    assert_error(
        &mut session,
        json!(1),
        "initialize",
        Some(malformed),
        -32602,
    );
    let initialized = initialize(&mut session, json!(2), json!("unsupported"));
    let result = &initialized["result"];
    assert_eq!(
        (
            result["protocolVersion"].clone(),
            result["capabilities"].clone()
        ),
        (json!(V), json!({}))
    );
}

#[test]
fn lifecycle_allows_ping_and_only_initialized_notification_opens_operations() {
    let mut session = McpSession::new();
    initialize(&mut session, json!("init"), json!(V));
    assert_eq!(
        rpc(&mut session, json!(1), "ping", None)["result"],
        json!({})
    );
    assert_error(&mut session, json!(2), "tools/list", None, -32002);
    assert_eq!(
        notify(&mut session, "notifications/initialized", Some(json!([]))),
        McpAction::NoResponse
    );
    assert_error(&mut session, json!(3), "tools/list", None, -32002);
    let open_meta = json!({"_meta":{"progressToken":true,"vendor":[]}});
    assert_eq!(
        notify(&mut session, "notifications/initialized", Some(open_meta)),
        McpAction::NoResponse
    );
    assert_eq!(
        notify(&mut session, "vendor/event", Some(json!([]))),
        McpAction::NoResponse
    );
    assert_error(&mut session, json!(4), "tools/list", None, -32601);
}

#[test]
fn request_ids_are_exact_unique_memory_bounded_and_close_on_exhaustion() {
    let mut exact = McpSession::new();
    initialize(&mut exact, json!("same"), json!(V));
    assert_error(&mut exact, json!("same"), "ping", None, -32600);
    let oversized = "x".repeat(257);
    assert_eq!(
        response(McpSession::new().handle_message(&message(json!(oversized), "initialize", None)))
            ["id"],
        Value::Null
    );

    let mut bounded = McpSession::new();
    initialize(&mut bounded, json!(0), json!(V));
    assert_eq!(
        notify(&mut bounded, "notifications/initialized", None),
        McpAction::NoResponse
    );
    for id in 1..1024 {
        assert_eq!(
            rpc(&mut bounded, json!(id), "ping", None)["result"],
            json!({})
        );
    }
    let exhausted = response(bounded.handle_message(&message(json!(1024), "ping", None)));
    assert_eq!(exhausted["error"]["code"], -32000);
    assert_eq!(bounded.handle_message("{"), McpAction::Close(None));
}
