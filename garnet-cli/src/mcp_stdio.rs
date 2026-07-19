//! Raw-byte stdio transport for the bounded Minimum Shelf MCP host.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::minimum_shelf::MinimumShelfHost;
    use serde_json::{json, Value};
    use std::io::Cursor;

    fn request(id: Value, method: &str, params: Option<Value>) -> Vec<u8> {
        let mut message = json!({"jsonrpc":"2.0","id":id,"method":method});
        if let Some(params) = params {
            message["params"] = params;
        }
        encode_frame(message.to_string().as_bytes())
    }

    fn notification(method: &str) -> Vec<u8> {
        encode_frame(json!({"jsonrpc":"2.0","method":method}).to_string().as_bytes())
    }

    fn decode_all(bytes: Vec<u8>) -> Vec<Value> {
        let mut reader = Cursor::new(bytes);
        let mut messages = Vec::new();
        while let Some(frame) = read_frame(&mut reader).expect("valid response frame") {
            messages.push(serde_json::from_slice(&frame).expect("response JSON"));
        }
        messages
    }

    #[test]
    fn mcp_stdio_raw_bytes_initialize_list_call_and_error() {
        let source = "@caps()\ndef main(value) { value * 2 }\n";
        let mut host = MinimumShelfHost::from_verified_source(source.to_string());
        let init = json!({
            "protocolVersion": crate::mcp::MCP_PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": {"name":"raw-byte-test","version":"1"}
        });
        let mut input = Vec::new();
        input.extend(request(json!(1), "initialize", Some(init)));
        input.extend(notification("notifications/initialized"));
        input.extend(request(json!(2), "tools/list", None));
        input.extend(request(
            json!(3),
            "tools/call",
            Some(json!({"name":"garnet.core.double","arguments":{"value":21}})),
        ));
        input.extend(request(json!(4), "unknown/method", None));

        let mut output = Vec::new();
        serve(&mut host, &mut Cursor::new(input), &mut output).expect("stdio session");
        assert!(!output.windows(2).any(|pair| pair == b"\n\n"));
        let messages = decode_all(output);
        assert_eq!(messages.len(), 4, "initialized notification has no response");
        assert_eq!(messages[0]["result"]["capabilities"], json!({"tools":{"listChanged":false}}));
        assert_eq!(messages[1]["result"]["tools"][0]["name"], "garnet.core.double");
        assert_eq!(messages[2]["result"]["structuredContent"], json!({"value":42}));
        assert_eq!(messages[3]["error"]["code"], -32601);
    }

    #[test]
    fn mcp_stdio_rejects_lf_headers_and_emits_one_parse_error_frame() {
        let source = "@caps()\ndef main(value) { value * 2 }\n";
        let mut host = MinimumShelfHost::from_verified_source(source.to_string());
        let mut output = Vec::new();
        let error = serve(
            &mut host,
            &mut Cursor::new(b"Content-Length: 2\n\n{}".to_vec()),
            &mut output,
        )
        .expect_err("text-mode LF framing must fail");
        assert!(error.to_string().contains("CRLF"), "{error}");
        let messages = decode_all(output);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["error"]["code"], -32700);
        assert_eq!(messages[0]["id"], Value::Null);
    }
}
