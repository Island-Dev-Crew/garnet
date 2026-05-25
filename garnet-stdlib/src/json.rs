//! `std::json` — JSON parse/serialize/access (Layer 1, no caps).
//!
//! Pure host helpers over `serde_json::Value`, tracking RFC 8259. The
//! interpreter marshals between Garnet values and `serde_json::Value`.
//! `@stability(experimental)`.

use crate::StdError;
use serde_json::Value;

/// Parse a JSON string into a value. Errors on malformed input.
pub fn parse(input: &str) -> Result<Value, StdError> {
    serde_json::from_str(input)
        .map_err(|e| StdError::InvalidInput(format!("json parse error: {e}")))
}

/// Serialize a value to a compact JSON string.
pub fn stringify(value: &Value) -> String {
    // `to_string` on a serde_json::Value is infallible.
    value.to_string()
}

/// Look up a key (object) or index (array) in a value. Returns a clone of
/// the child, or `None` if absent / wrong shape.
pub fn get(value: &Value, key: &str) -> Option<Value> {
    match value {
        Value::Object(map) => map.get(key).cloned(),
        Value::Array(arr) => key.parse::<usize>().ok().and_then(|i| arr.get(i)).cloned(),
        _ => None,
    }
}

/// Return a copy of a JSON object with `key` set to `value`. Errors if the
/// target is not an object.
pub fn set(target: &Value, key: &str, value: Value) -> Result<Value, StdError> {
    match target {
        Value::Object(map) => {
            let mut map = map.clone();
            map.insert(key.to_string(), value);
            Ok(Value::Object(map))
        }
        other => Err(StdError::InvalidInput(format!(
            "json set: target is {}, not an object",
            kind(other)
        ))),
    }
}

fn kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_then_get_object() {
        let v = parse(r#"{"name":"garnet","version":7}"#).unwrap();
        assert_eq!(get(&v, "name"), Some(Value::String("garnet".into())));
        assert_eq!(get(&v, "version"), Some(serde_json::json!(7)));
        assert_eq!(get(&v, "missing"), None);
    }

    #[test]
    fn parse_then_get_array_index() {
        let v = parse(r#"[10,20,30]"#).unwrap();
        assert_eq!(get(&v, "0"), Some(serde_json::json!(10)));
        assert_eq!(get(&v, "2"), Some(serde_json::json!(30)));
        assert_eq!(get(&v, "9"), None);
        assert_eq!(get(&v, "notanindex"), None);
    }

    #[test]
    fn parse_rejects_malformed() {
        match parse("{not json}") {
            Err(StdError::InvalidInput(_)) => {}
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    #[test]
    fn stringify_is_compact_and_roundtrips() {
        let v = parse(r#"{ "a" : 1 , "b" : [2,3] }"#).unwrap();
        let s = stringify(&v);
        assert_eq!(s, r#"{"a":1,"b":[2,3]}"#);
        // round-trips back to an equal value
        assert_eq!(parse(&s).unwrap(), v);
    }

    #[test]
    fn set_inserts_and_overwrites_without_mutating_input() {
        let v = parse(r#"{"a":1}"#).unwrap();
        let v2 = set(&v, "b", serde_json::json!(2)).unwrap();
        assert_eq!(get(&v2, "a"), Some(serde_json::json!(1)));
        assert_eq!(get(&v2, "b"), Some(serde_json::json!(2)));
        // original untouched
        assert_eq!(get(&v, "b"), None);
        // overwrite
        let v3 = set(&v2, "a", serde_json::json!(9)).unwrap();
        assert_eq!(get(&v3, "a"), Some(serde_json::json!(9)));
    }

    #[test]
    fn set_on_non_object_errors() {
        let v = parse("[1,2]").unwrap();
        match set(&v, "k", serde_json::json!(1)) {
            Err(StdError::InvalidInput(_)) => {}
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }
}
