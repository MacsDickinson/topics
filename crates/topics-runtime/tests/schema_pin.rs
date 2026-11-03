//! Compare the installed pinned generator to committed protocol projections.
#![cfg(all(target_os = "macos", feature = "native-live-tests"))]
use serde_json::{Value, json};
fn enums(value: &Value, path: String, out: &mut serde_json::Map<String, Value>) {
    match value {
        Value::Object(obj) => {
            if let Some(e) = obj.get("enum") {
                out.insert(path.clone(), e.clone());
            }
            for (k, v) in obj {
                enums(v, format!("{path}/{k}"), out);
            }
        }
        Value::Array(a) => {
            for (i, v) in a.iter().enumerate() {
                enums(v, format!("{path}/{i}"), out);
            }
        }
        _ => {}
    }
}
#[test]
fn installed_codex_generated_protocol_matches_pinned_required_fields_and_enums() {
    let root = tempfile::tempdir().unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let output = rt.block_on(async {
        let mut cmd = tokio::process::Command::new(
            std::env::var_os("TOPICS_TEST_CODEX").unwrap_or_else(|| "codex".into()),
        );
        cmd.args([
            "app-server",
            "generate-json-schema",
            "--experimental",
            "--out",
        ])
        .arg(root.path())
        .kill_on_drop(true);
        tokio::time::timeout(std::time::Duration::from_secs(15), cmd.output())
            .await
            .unwrap()
            .unwrap()
    });
    assert!(output.status.success());
    let expected: Value = serde_json::from_str(include_str!("schema/codex-0.166.0.json")).unwrap();
    for (file, projection) in expected.as_object().unwrap() {
        let value: Value =
            serde_json::from_slice(&std::fs::read(root.path().join(file)).unwrap()).unwrap();
        let mut variants = serde_json::Map::new();
        enums(&value, "".into(), &mut variants);
        assert_eq!(
            projection,
            &json!({"required":value.get("required").cloned().unwrap_or(json!([])),"properties":value["properties"],"enums":variants}),
            "schema drift in {file}"
        );
    }
    println!(
        "sanitized evidence: pinned Codex 0.166.0 schema projections match 16 methods/results/notifications"
    );
}
