use garnet_cli::mcp_schema::{
    valid_initialize_params, valid_initialized_notification_params, MCP_PROTOCOL_VERSION as V,
};
use serde_json::{json, Value};

fn valid_params() -> Value {
    json!({"protocolVersion":V,"capabilities":{"experimental":{"x":{}}},"clientInfo":{
        "name":"test","version":"1","title":"Test","description":"client",
        "icons":[{"src":"https://example.test/x.png","mimeType":"image/png","sizes":["48x48"],"theme":"dark"}],
        "websiteUrl":"https://example.test"},"_meta":{"trace":"t","progressToken":1.5}})
}

#[test]
fn released_initialize_shape_and_notification_meta_are_accepted() {
    assert!(valid_initialize_params(Some(&valid_params())));
    let mut alternate = valid_params();
    alternate["clientInfo"]["icons"] = json!([]);
    alternate["capabilities"] = json!({});
    assert!(valid_initialize_params(Some(&alternate)));
    alternate["clientInfo"]["icons"] = json!([{"src":"data:image/png;base64,AA=="}]);
    assert!(valid_initialize_params(Some(&alternate)));
    assert!(valid_initialized_notification_params(None));
    assert!(valid_initialized_notification_params(Some(
        &json!({"_meta":{"vendor":"value"}})
    )));
}

#[test]
fn malformed_implementation_metadata_is_rejected() {
    for info in [
        json!({"name":"x","version":"1","icons":[7]}),
        json!({"name":"x","version":"1","icons":[{}]}),
        json!({"name":"x","version":"1","icons":[{"src":7}]}),
        json!({"name":"x","version":"1","icons":[{"src":"not a uri"}]}),
        json!({"name":"x","version":"1","icons":[{"src":"data:image/png;base64,AA==","mimeType":7}]}),
        json!({"name":"x","version":"1","icons":[{"src":"https://example.test/x.png","sizes":[7]}]}),
        json!({"name":"x","version":"1","icons":[{"src":"https://example.test/x.png","theme":"sepia"}]}),
        json!({"name":"x","version":"1","websiteUrl":"not a uri"}),
    ] {
        let mut params = valid_params();
        params["clientInfo"] = info;
        assert!(!valid_initialize_params(Some(&params)));
    }
}

#[test]
fn known_capabilities_are_typed_and_top_level_extensions_remain_open() {
    for capabilities in [
        json!({"roots":false}),
        json!({"roots":{"listChanged":1}}),
        json!({"sampling":7}),
        json!({"sampling":{"context":false}}),
        json!({"experimental":[]}),
        json!({"experimental":{"bad":[]}}),
        json!({"elicitation":{"form":true}}),
        json!({"tasks":{"requests":{"sampling":{"createMessage":false}}}}),
    ] {
        let mut params = valid_params();
        params["capabilities"] = capabilities;
        assert!(!valid_initialize_params(Some(&params)));
    }
    let mut params = valid_params();
    params["capabilities"] = json!({"roots":{"listChanged":true},"sampling":{"context":{},"tools":{}},
        "elicitation":{"form":{},"url":{}},"tasks":{"list":{},"cancel":{},"requests":{
        "sampling":{"createMessage":{}},"elicitation":{"create":{}}}},"vendor.example":{"mode":"strict"}});
    assert!(valid_initialize_params(Some(&params)));
}

#[test]
fn malformed_outer_and_meta_shapes_are_rejected() {
    for params in [
        json!({}),
        json!({"protocolVersion":7,"capabilities":{},"clientInfo":{"name":"x","version":"1"}}),
        json!({"protocolVersion":V,"capabilities":{},"clientInfo":{"name":"x","version":"1"},"extra":1}),
        json!({"protocolVersion":V,"capabilities":{},"clientInfo":{"name":"x","version":"1"},"_meta":[]}),
        json!({"protocolVersion":V,"capabilities":{},"clientInfo":{"name":"x","version":"1"},"_meta":{"progressToken":true}}),
    ] {
        assert!(!valid_initialize_params(Some(&params)));
    }
    assert!(!valid_initialized_notification_params(Some(
        &json!({"extra":1})
    )));
    assert!(!valid_initialized_notification_params(Some(
        &json!({"_meta":[]})
    )));
}
