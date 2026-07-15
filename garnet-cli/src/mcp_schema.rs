//! Released MCP initialize-schema validation. Session and transport are deferred.
use serde_json::{Map, Value};

pub const MCP_PROTOCOL_VERSION: &str = "2025-11-25";
const INIT_FIELDS: &[&str] = &["protocolVersion", "capabilities", "clientInfo", "_meta"];

pub fn valid_initialize_params(value: Option<&Value>) -> bool {
    let Some(params) = value.and_then(Value::as_object) else {
        return false;
    };
    allowed_keys(params, INIT_FIELDS)
        && valid_request_meta(params)
        && params.get("protocolVersion").is_some_and(Value::is_string)
        && params
            .get("capabilities")
            .is_some_and(valid_client_capabilities)
        && params.get("clientInfo").is_some_and(valid_client_info)
}

pub fn valid_initialized_notification_params(value: Option<&Value>) -> bool {
    value.is_none()
        || value.is_some_and(|value| {
            value.as_object().is_some_and(|params| {
                allowed_keys(params, &["_meta"]) && params.get("_meta").is_none_or(Value::is_object)
            })
        })
}

pub(crate) fn valid_request_params(value: Option<&Value>) -> bool {
    value.is_none()
        || value.is_some_and(|value| {
            value.as_object().is_some_and(|params| {
                allowed_keys(params, &["_meta"]) && valid_request_meta(params)
            })
        })
}

fn valid_client_info(value: &Value) -> bool {
    let Some(info) = value.as_object() else {
        return false;
    };
    allowed_keys(
        info,
        &[
            "name",
            "version",
            "title",
            "description",
            "icons",
            "websiteUrl",
        ],
    ) && ["name", "version"]
        .iter()
        .all(|key| info.get(*key).is_some_and(Value::is_string))
        && ["title", "description"]
            .iter()
            .all(|key| info.get(*key).is_none_or(Value::is_string))
        && info.get("websiteUrl").is_none_or(valid_uri)
        && info.get("icons").is_none_or(|value| {
            value
                .as_array()
                .is_some_and(|icons| icons.iter().all(valid_icon))
        })
}

fn valid_uri(value: &Value) -> bool {
    value
        .as_str()
        .is_some_and(|uri| fluent_uri::Uri::parse(uri).is_ok())
}

fn valid_icon(value: &Value) -> bool {
    let Some(icon) = value.as_object() else {
        return false;
    };
    allowed_keys(icon, &["src", "mimeType", "sizes", "theme"])
        && icon.get("src").is_some_and(valid_uri)
        && icon.get("mimeType").is_none_or(Value::is_string)
        && icon.get("sizes").is_none_or(|value| {
            value
                .as_array()
                .is_some_and(|sizes| sizes.iter().all(Value::is_string))
        })
        && icon
            .get("theme")
            .is_none_or(|value| matches!(value.as_str(), Some("light" | "dark")))
}

fn valid_client_capabilities(value: &Value) -> bool {
    let Some(capabilities) = value.as_object() else {
        return false;
    };
    capabilities.iter().all(|(key, value)| match key.as_str() {
        "experimental" => value
            .as_object()
            .is_some_and(|map| map.values().all(Value::is_object)),
        "roots" => value.as_object().is_some_and(|map| {
            allowed_keys(map, &["listChanged"])
                && map.get("listChanged").is_none_or(Value::is_boolean)
        }),
        "sampling" => object_options(value, &["context", "tools"]),
        "elicitation" => object_options(value, &["form", "url"]),
        "tasks" => valid_tasks(value),
        _ => true,
    })
}

fn object_options(value: &Value, keys: &[&str]) -> bool {
    value
        .as_object()
        .is_some_and(|map| allowed_keys(map, keys) && map.values().all(Value::is_object))
}

fn valid_tasks(value: &Value) -> bool {
    let Some(tasks) = value.as_object() else {
        return false;
    };
    allowed_keys(tasks, &["list", "cancel", "requests"])
        && ["list", "cancel"]
            .iter()
            .all(|key| tasks.get(*key).is_none_or(Value::is_object))
        && tasks.get("requests").is_none_or(|value| {
            let Some(requests) = value.as_object() else {
                return false;
            };
            allowed_keys(requests, &["sampling", "elicitation"])
                && requests
                    .get("sampling")
                    .is_none_or(|value| object_options(value, &["createMessage"]))
                && requests
                    .get("elicitation")
                    .is_none_or(|value| object_options(value, &["create"]))
        })
}

fn valid_request_meta(map: &Map<String, Value>) -> bool {
    map.get("_meta").is_none_or(|value| {
        value.as_object().is_some_and(|meta| {
            meta.get("progressToken")
                .is_none_or(|token| token.is_string() || token.is_number())
        })
    })
}

fn allowed_keys(map: &Map<String, Value>, keys: &[&str]) -> bool {
    map.keys().all(|key| keys.contains(&key.as_str()))
}
