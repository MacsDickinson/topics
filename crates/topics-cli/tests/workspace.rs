use serde_json::{Value, json};
use std::{fs, process::Command};
#[test]
fn real_cli_reviews_and_undoes_manual_proposal_using_one_request_object() {
    let temp = tempfile::tempdir().unwrap();
    let app = temp.path().join("app-data");
    let root = temp.path().join("notes");
    let run = |command: &str, request: Value| {
        let output = Command::new(env!("CARGO_BIN_EXE_topics"))
            .env_clear()
            .env("TOPICS_APP_DATA_DIR", &app)
            .current_dir(temp.path())
            .args(["workspace", command, &request.to_string()])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice::<Value>(&output.stdout).unwrap()
    };
    let root_ref = run("root-set", json!({"path":root,"create":true}));
    let root_id = &root_ref["root"]["root_id"];
    let candidate = json!({"kind":"topic","topic":{"id":"62c0bec0-9676-49c0-b532-0c71c8fc38a2","topic_kind":"thread","name":"CLI synthetic topic","aliases":[],"sensitivity":"unknown"}});
    let staged = run(
        "manual",
        json!({"root_id":root_id,"candidates":[candidate]}),
    );
    assert_eq!(staged["proposal"]["status"], "pending");
    assert!(
        run("topics", json!({"root_id":root_id}))["topics"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let p = &staged["proposal"];
    let applied = run(
        "decide",
        json!({"root_id":root_id,"proposal_id":p["id"],"expected_fingerprint":p["fingerprint"],"decision":{"kind":"approve","candidates":[candidate]}}),
    );
    assert_eq!(
        run("topics", json!({"root_id":root_id}))["topics"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(
        fs::read_dir(root.join("topics/topics"))
            .unwrap()
            .next()
            .is_some()
    );
    run(
        "undo",
        json!({"root_id":root_id,"operation_id":applied["operation"]["id"]}),
    );
    assert!(
        run("topics", json!({"root_id":root_id}))["topics"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        run("history", json!({"root_id":root_id}))["operations"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(!root.join(".git").exists());
}
