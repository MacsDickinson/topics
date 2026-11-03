//! The proxy must exit on parent EOF even while Codex holds stdin open.
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
#[test]
fn proxy_exits_after_authenticated_parent_disconnect_with_open_stdin() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = listener.local_addr().unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_topics-mcp-proxy"))
        .env("TOPICS_MCP_ENDPOINT", endpoint.to_string())
        .env("TOPICS_MCP_SESSION", "synthetic-session")
        .env("TOPICS_MCP_TOKEN", "synthetic-capability")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    listener.set_nonblocking(true).unwrap();
    let accept_deadline = Instant::now() + Duration::from_secs(3);
    let (stream, _) = loop {
        match listener.accept() {
            Ok(connection) => break connection,
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    && Instant::now() < accept_deadline =>
            {
                std::thread::sleep(Duration::from_millis(10))
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("Proxy did not connect within deadline");
            }
        }
    };
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    let mut reader = BufReader::new(stream);
    let mut bootstrap = String::new();
    reader.read_line(&mut bootstrap).unwrap();
    let value: serde_json::Value = serde_json::from_str(&bootstrap).unwrap();
    assert_eq!(value["session"], "synthetic-session");
    assert!(uuid::Uuid::parse_str(value["nonce"].as_str().unwrap()).is_ok());
    reader.get_mut().write_all(b"ok\n").unwrap();
    drop(reader);
    drop(listener);
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        if Instant::now() > deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("Proxy hung on blocking stdin after parent EOF");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
