//! No model requests or auth reads: app-server reads only a throwaway config.
#![cfg(target_os = "macos")]
#[test]
fn installed_runtime_force_disables_notify_skills_and_telemetry_before_any_turn() {
    use std::{
        io::{BufRead, BufReader, Write},
        process::Stdio,
        time::{Duration, Instant},
    };
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("home");
    let ch = home.join(".codex");
    let cwd = temp.path().join("cwd");
    let root = temp.path().join("notes");
    for dir in [&ch, &cwd, &root] {
        std::fs::create_dir_all(dir).unwrap();
    }
    std::fs::write(ch.join("config.toml"),"notify=[\"/usr/bin/false\"]\n[skills]\ninclude_instructions=true\n[otel]\nlog_user_prompt=true\nexporter=\"statsig\"\n").unwrap();
    let mut cmd = topics_runtime::isolation::command(
        &root.canonicalize().unwrap(),
        std::path::Path::new("/Users/macs/.local/bin/codex"),
    );
    cmd.current_dir(&cwd)
        .env_clear()
        .env("HOME", &home)
        .env("CODEX_HOME", &ch)
        .env("PATH", "/usr/bin:/bin");
    for f in topics_runtime::protocol::DISABLED {
        cmd.args(["--disable", *f]);
    }
    for c in topics_runtime::protocol::PRIVACY_OVERRIDES {
        cmd.args(["-c", *c]);
    }
    cmd.args(["-c", "web_search=\"disabled\"", "app-server"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = cmd.spawn().unwrap();
    let mut input = child.stdin.take().unwrap();
    let output = child.stdout.take().unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(output).lines() {
            let Ok(line) = line else {
                break;
            };
            let _ = tx.send(line);
        }
    });
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        input.write_all(b"{\"id\":1,\"method\":\"initialize\",\"params\":{\"clientInfo\":{\"name\":\"topics-synthetic-config\",\"version\":\"1\"},\"capabilities\":{\"experimentalApi\":true}}}\n").unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let v: serde_json::Value = serde_json::from_str(
                &rx.recv_timeout(deadline.saturating_duration_since(Instant::now()))
                    .unwrap(),
            )
            .unwrap();
            if v["id"] == 1 {
                assert!(v.get("result").is_some());
                break;
            }
        }
        input.write_all(format!("{}\n{}\n",serde_json::json!({"method":"initialized","params":{}}),serde_json::json!({"id":2,"method":"config/read","params":{"cwd":cwd,"includeLayers":false}})).as_bytes()).unwrap();
        loop {
            let v: serde_json::Value = serde_json::from_str(
                &rx.recv_timeout(deadline.saturating_duration_since(Instant::now()))
                    .unwrap(),
            )
            .unwrap();
            if v["id"] == 2 {
                assert!(
                    topics_runtime::protocol::validate_privacy(&v["result"]["config"]).is_ok(),
                    "effective privacy override mismatch: {:?}",
                    topics_runtime::protocol::validate_privacy(&v["result"]["config"]).err()
                );
                break;
            }
        }
    }));
    let _ = child.kill();
    let _ = child.wait();
    assert!(result.is_ok());
}
