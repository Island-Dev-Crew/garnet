//! Native-process raw-byte stdio proof for the one sealed Minimum Shelf host.

use serde_json::{json, Value};
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn flagship() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/minimum-shelf-flagship")
}

fn frame(message: Value) -> Vec<u8> {
    let body = message.to_string();
    let mut bytes = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
    bytes.extend_from_slice(body.as_bytes());
    bytes
}

fn read_frames(bytes: &[u8]) -> Vec<Value> {
    let mut input = Cursor::new(bytes);
    let mut messages = Vec::new();
    loop {
        let mut header = Vec::new();
        let mut byte = [0_u8; 1];
        while !header.ends_with(b"\r\n\r\n") {
            if input.read(&mut byte).expect("read frame header") == 0 {
                assert!(header.is_empty(), "truncated response header");
                return messages;
            }
            header.push(byte[0]);
        }
        let header = std::str::from_utf8(&header[..header.len() - 4]).expect("ASCII header");
        let length: usize = header
            .strip_prefix("Content-Length: ")
            .expect("canonical Content-Length")
            .parse()
            .expect("decimal length");
        let mut body = vec![0_u8; length];
        input.read_exact(&mut body).expect("complete response body");
        messages.push(serde_json::from_slice(&body).expect("response JSON"));
    }
}

fn run_session(package: &std::path::Path, input: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_garnet"))
        .arg("mcp-serve")
        .arg("--package")
        .arg(package)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn Minimum Shelf host");
    let mut stdin = child.stdin.take().expect("piped stdin");
    stdin.write_all(input).expect("write raw transcript");
    drop(stdin);
    child.wait_with_output().expect("host output")
}

#[test]
fn native_stdio_initialize_list_call_and_error() {
    let init = json!({
        "jsonrpc":"2.0",
        "id":1,
        "method":"initialize",
        "params":{
            "protocolVersion":garnet_cli::mcp::MCP_PROTOCOL_VERSION,
            "capabilities":{},
            "clientInfo":{"name":"garnet-committed-trap","version":"1"}
        }
    });
    let mut input = frame(init);
    input.extend(frame(json!({
        "jsonrpc":"2.0", "method":"notifications/initialized"
    })));
    input.extend(frame(json!({
        "jsonrpc":"2.0", "id":2, "method":"tools/list"
    })));
    input.extend(frame(json!({
        "jsonrpc":"2.0", "id":3, "method":"tools/call",
        "params":{"name":"garnet.core.double","arguments":{"value":21}}
    })));
    input.extend(frame(json!({
        "jsonrpc":"2.0", "id":4, "method":"not/a/method"
    })));

    let output = run_session(&flagship(), &input);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!output.stdout.windows(2).any(|pair| pair == b"\n\n"));
    let messages = read_frames(&output.stdout);
    assert_eq!(messages.len(), 4);
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
fn native_stdio_rejects_tampered_package_before_protocol_output() {
    let temp = tempfile::tempdir().expect("temp package");
    for name in ["SHELF_PACKAGE.json", "tool.garnet", "tool.seal.json"] {
        fs::copy(flagship().join(name), temp.path().join(name)).expect("copy package");
    }
    fs::write(
        temp.path().join("tool.garnet"),
        b"@caps()\ndef main(value) { value * 3 }\n",
    )
    .expect("tamper source");

    let output = run_session(temp.path(), &[]);
    assert!(!output.status.success());
    assert!(
        output.stdout.is_empty(),
        "no MCP bytes before seal acceptance"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("package rejected"));
}
