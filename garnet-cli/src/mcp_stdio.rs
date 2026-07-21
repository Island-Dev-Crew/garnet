//! Raw-byte stdio transport for the bounded Minimum Shelf MCP host.

use crate::mcp::McpAction;
use crate::minimum_shelf::MinimumShelfHost;
use std::io::{self, Read, Write};

const MAX_HEADER_BYTES: usize = 8 * 1024;
const MAX_BODY_BYTES: usize = 1024 * 1024;

#[cfg(windows)]
pub(crate) fn set_binary_stdio() -> io::Result<()> {
    const STDIN_FILENO: i32 = 0;
    const STDOUT_FILENO: i32 = 1;
    const O_BINARY: i32 = 0x8000;

    unsafe extern "C" {
        fn _setmode(file_descriptor: i32, mode: i32) -> i32;
    }

    for descriptor in [STDIN_FILENO, STDOUT_FILENO] {
        // SAFETY: `_setmode` accepts a process-owned CRT descriptor and a
        // documented mode constant. We pass only stdin/stdout and check -1.
        if unsafe { _setmode(descriptor, O_BINARY) } == -1 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

#[cfg(not(windows))]
pub(crate) fn set_binary_stdio() -> io::Result<()> {
    Ok(())
}

pub(crate) fn encode_frame(body: &[u8]) -> Vec<u8> {
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    let mut frame = Vec::with_capacity(header.len() + body.len());
    frame.extend_from_slice(header.as_bytes());
    frame.extend_from_slice(body);
    frame
}

pub(crate) fn read_frame<R: Read>(reader: &mut R) -> io::Result<Option<Vec<u8>>> {
    let mut header = Vec::new();
    loop {
        let mut byte = [0_u8; 1];
        match reader.read(&mut byte)? {
            0 if header.is_empty() => return Ok(None),
            0 => return Err(invalid_data("incomplete header; CRLF framing required")),
            _ => {}
        }
        header.push(byte[0]);
        if byte[0] == b'\n' && header.get(header.len().saturating_sub(2)) != Some(&b'\r') {
            return Err(invalid_data("bare LF rejected; CRLF framing required"));
        }
        if header.len() >= 2
            && header[header.len() - 2] == b'\r'
            && header[header.len() - 1] != b'\n'
        {
            return Err(invalid_data("bare CR rejected; CRLF framing required"));
        }
        if header.ends_with(b"\r\n\r\n") {
            break;
        }
        if header.len() > MAX_HEADER_BYTES {
            return Err(invalid_data("MCP header exceeds byte limit"));
        }
    }

    let field = &header[..header.len() - 4];
    if field.iter().any(|byte| matches!(byte, b'\r' | b'\n')) {
        return Err(invalid_data(
            "exactly one Content-Length header is required",
        ));
    }
    let field = std::str::from_utf8(field)
        .map_err(|_| invalid_data("Content-Length header must be ASCII"))?;
    let digits = field
        .strip_prefix("Content-Length: ")
        .ok_or_else(|| invalid_data("exact Content-Length header is required"))?;
    if digits.is_empty()
        || !digits.bytes().all(|byte| byte.is_ascii_digit())
        || (digits.len() > 1 && digits.starts_with('0'))
    {
        return Err(invalid_data("invalid Content-Length value"));
    }
    let length = digits
        .parse::<usize>()
        .map_err(|_| invalid_data("Content-Length value overflows"))?;
    if length > MAX_BODY_BYTES {
        return Err(invalid_data("MCP body exceeds byte limit"));
    }
    let mut body = vec![0_u8; length];
    reader.read_exact(&mut body)?;
    Ok(Some(body))
}

pub(crate) fn serve<R: Read, W: Write>(
    host: &mut MinimumShelfHost,
    reader: &mut R,
    writer: &mut W,
) -> io::Result<()> {
    loop {
        let frame = match read_frame(reader) {
            Ok(Some(frame)) => frame,
            Ok(None) => break,
            Err(error) => {
                write_parse_error(writer)?;
                return Err(error);
            }
        };
        let input = match std::str::from_utf8(&frame) {
            Ok(input) => input,
            Err(_) => {
                write_parse_error(writer)?;
                return Err(invalid_data("MCP JSON body is not valid UTF-8"));
            }
        };
        match host.handle_message(input) {
            McpAction::Respond(response) => write_message(writer, &response)?,
            McpAction::NoResponse => {}
            McpAction::Close(Some(response)) => {
                write_message(writer, &response)?;
                break;
            }
            McpAction::Close(None) => break,
        }
    }
    writer.flush()
}

fn write_message<W: Write>(writer: &mut W, message: &str) -> io::Result<()> {
    writer.write_all(&encode_frame(message.as_bytes()))
}

fn write_parse_error<W: Write>(writer: &mut W) -> io::Result<()> {
    write_message(
        writer,
        r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}}"#,
    )?;
    writer.flush()
}

fn invalid_data(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;
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
        encode_frame(
            json!({"jsonrpc":"2.0","method":method})
                .to_string()
                .as_bytes(),
        )
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
        assert_eq!(
            messages.len(),
            4,
            "initialized notification has no response"
        );
        assert_eq!(
            messages[0]["result"]["capabilities"],
            json!({"tools":{"listChanged":false}})
        );
        assert_eq!(
            messages[1]["result"]["tools"][0]["name"],
            "garnet.core.double"
        );
        assert_eq!(
            messages[2]["result"]["structuredContent"],
            json!({"value":42})
        );
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

    #[test]
    fn mcp_stdio_rejects_noncanonical_and_unbounded_headers() {
        for invalid in [
            b"content-length: 2\r\n\r\n{}".as_slice(),
            b"Content-Length: 02\r\n\r\n{}".as_slice(),
            b"Content-Length: 2\r\nX-Test: no\r\n\r\n{}".as_slice(),
            b"Content-Length: 2\rX".as_slice(),
            b"Content-Length: 1048577\r\n\r\n".as_slice(),
        ] {
            let error = read_frame(&mut Cursor::new(invalid)).expect_err("must fail closed");
            assert_eq!(error.kind(), std::io::ErrorKind::InvalidData, "{error}");
        }
    }

    #[test]
    fn mcp_stdio_short_or_non_utf8_bodies_emit_one_parse_error() {
        let source = "@caps()\ndef main(value) { value * 2 }\n";
        for invalid in [
            b"Content-Length: 3\r\n\r\n{}".as_slice(),
            b"Content-Length: 1\r\n\r\n\xff".as_slice(),
        ] {
            let mut host = MinimumShelfHost::from_verified_source(source.to_string());
            let mut output = Vec::new();
            serve(&mut host, &mut Cursor::new(invalid), &mut output)
                .expect_err("invalid body must fail closed");
            let messages = decode_all(output);
            assert_eq!(messages.len(), 1);
            assert_eq!(messages[0]["error"]["code"], -32700);
        }
    }
}
