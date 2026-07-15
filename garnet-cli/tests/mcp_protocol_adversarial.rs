use garnet_cli::mcp::{McpAction, McpSession, MCP_PROTOCOL_VERSION as V};
use serde_json::{json, Value};

fn params(version: Value) -> Value {
    json!({"protocolVersion":version,"capabilities":{},
        "clientInfo":{"name":"adversarial-client","version":"1"}})
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
    rpc(session, id, "initialize", Some(params(version)))
}

fn notify(session: &mut McpSession, method: &str, params: Option<Value>) -> McpAction {
    let mut value = json!({"jsonrpc":"2.0","method":method});
    if let Some(params) = params {
        value["params"] = params;
    }
    session.handle_message(&value.to_string())
}

fn error_code(value: &Value) -> i64 {
    value["error"]["code"].as_i64().expect("integer error code")
}

fn ready(session: &mut McpSession) {
    let initialized = initialize(session, json!("init"), json!(V));
    assert_eq!(initialized["result"]["protocolVersion"], V);
    assert_eq!(initialized["result"]["capabilities"], json!({}));
    assert_eq!(
        initialized["result"]
            .as_object()
            .expect("result object")
            .len(),
        3
    );
    assert!(initialized["result"]["serverInfo"]["name"].is_string());
    assert!(initialized["result"]["serverInfo"]["version"].is_string());
    assert_eq!(
        notify(session, "notifications/initialized", None),
        McpAction::NoResponse
    );
}

#[test]
fn invalid_envelope_matrix_echoes_only_readable_valid_ids() {
    let cases = [
        ("[]".to_owned(), json!(null)),
        ("{}".to_owned(), json!(null)),
        (
            json!({"jsonrpc":"1.0","id":7,"method":"x"}).to_string(),
            json!(7),
        ),
        (
            json!({"jsonrpc":"2.0","id":"m","method":7}).to_string(),
            json!("m"),
        ),
        (
            json!({"jsonrpc":"2.0","id":"p","method":"x","params":[]}).to_string(),
            json!("p"),
        ),
        (
            json!({"jsonrpc":"2.0","id":-7,"method":"x","extra":1}).to_string(),
            json!(-7),
        ),
    ];
    for (raw, id) in cases {
        let value = response(McpSession::new().handle_message(&raw));
        assert_eq!((error_code(&value), &value["id"]), (-32600, &id));
    }
    for id in [json!(null), json!(true), json!(1.5), json!({}), json!([])] {
        let value = response(McpSession::new().handle_message(&message(id, "initialize", None)));
        assert_eq!((error_code(&value), &value["id"]), (-32600, &Value::Null));
    }
}

#[test]
fn every_response_shaped_peer_message_closes_without_a_response() {
    for raw in [
        json!({"jsonrpc":"2.0","id":1,"result":{}}).to_string(),
        json!({"jsonrpc":"2.0","id":"x","error":{"code":-32601,"message":"no"}}).to_string(),
        json!({"id":1,"result":{},"error":{}}).to_string(),
    ] {
        assert_eq!(
            McpSession::new().handle_message(&raw),
            McpAction::Close(None)
        );
    }
}

#[test]
fn ping_params_and_unadvertised_operations_follow_each_lifecycle_phase() {
    let mut session = McpSession::new();
    initialize(&mut session, json!("init"), json!(V));
    let meta = json!({"_meta":{"progressToken":"p","vendor":true}});
    assert_eq!(
        rpc(&mut session, json!(1), "ping", Some(meta))["result"],
        json!({})
    );
    for (id, bad) in [(2, json!({"extra":1})), (3, json!({"_meta":[]}))] {
        assert_eq!(
            error_code(&rpc(&mut session, json!(id), "ping", Some(bad))),
            -32602
        );
    }
    for (id, method) in [(4, "tools/call"), (5, "unknown")] {
        assert_eq!(
            error_code(&rpc(&mut session, json!(id), method, None)),
            -32002
        );
    }
    assert_eq!(
        notify(&mut session, "notifications/initialized", None),
        McpAction::NoResponse
    );
    assert_eq!(
        rpc(&mut session, json!(6), "ping", None)["result"],
        json!({})
    );
    for (id, method) in [(7, "tools/call"), (8, "resources/list")] {
        assert_eq!(
            error_code(&rpc(&mut session, json!(id), method, None)),
            -32601
        );
    }
}

#[test]
fn premature_and_duplicate_notifications_cannot_corrupt_state() {
    for method in ["initialize", "notifications/initialized"] {
        let mut session = McpSession::new();
        let payload = (method == "initialize").then(|| params(json!(V)));
        assert_eq!(
            notify(&mut session, method, payload),
            McpAction::Close(None)
        );
    }
    let mut session = McpSession::new();
    ready(&mut session);
    assert_eq!(
        notify(&mut session, "unknown", Some(json!({}))),
        McpAction::NoResponse
    );
    assert_eq!(
        notify(&mut session, "notifications/initialized", None),
        McpAction::NoResponse
    );
    assert_eq!(
        error_code(&rpc(
            &mut session,
            json!(1),
            "initialize",
            Some(params(json!(V)))
        )),
        -32600
    );
    assert_eq!(
        rpc(&mut session, json!(2), "ping", None)["result"],
        json!({})
    );
}

#[test]
fn negative_numeric_string_and_escaped_ids_remain_distinct_and_exact() {
    let mut session = McpSession::new();
    initialize(&mut session, json!("init"), json!(V));
    for id in [json!(-1), json!(1), json!("1")] {
        assert_eq!(rpc(&mut session, id.clone(), "ping", None)["id"], id);
    }
    let escaped = "line\nbreak";
    let raw = raw_response(session.handle_message(&message(json!(escaped), "ping", None)));
    assert!(!raw.contains(['\r', '\n']));
    assert_eq!(
        serde_json::from_str::<Value>(&raw).expect("response JSON")["id"],
        escaped
    );
}
