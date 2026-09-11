//! NetDefaults on both backends: an IPv6 address that carries an internal
//! IPv4 address is refused under the strict policy, like the IPv4 address
//! itself. Before the fix, `@caps(net)` plus
//! `net::tcp_connect("::ffff:127.0.0.1", port)` reached a local listener.

use std::io::ErrorKind;
use std::net::TcpListener;
use std::process::Command;

fn run_backend(program: &str, backend: &str) -> std::process::Output {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("prog.garnet");
    std::fs::write(&path, program).unwrap();
    Command::new(env!("CARGO_BIN_EXE_garnet"))
        .args(["run", backend])
        .arg(&path)
        .output()
        .unwrap()
}

#[test]
fn ipv4_mapped_loopback_is_refused_on_both_backends() {
    for backend in ["--interp", "--vm"] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let port = listener.local_addr().unwrap().port();
        let program = format!(
            "@caps(net)\ndef main() {{ net::tcp_connect(\"::ffff:127.0.0.1\", {port}) }}\n"
        );
        let out = run_backend(&program, backend);
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        // connect() returns only after the handshake, so a connection the
        // program made is already queued on the listener.
        match listener.accept() {
            Err(e) if e.kind() == ErrorKind::WouldBlock => {}
            Ok((_, peer)) => {
                panic!("{backend}: the program reached the local listener from {peer}: {text}")
            }
            Err(e) => panic!("{backend}: accept failed: {e}"),
        }
        assert!(
            text.contains("denied"),
            "{backend}: expected a NetDefaults denial, got: {text}"
        );
    }
}
