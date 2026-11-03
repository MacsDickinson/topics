use serde_json::json;
#[test]
fn inherited_notifications_providers_endpoints_telemetry_and_prompt_files_fail_closed() {
    let mut safe = json!({"openai_base_url":"https://chatgpt.com/backend-api/codex","chatgpt_base_url":"https://chatgpt.com/backend-api/","notify": [], "model_provider": "openai", "skills": {"include_instructions": false}, "agents": {"enabled": false}, "analytics": {"enabled": false}, "feedback": {"enabled": false}, "compact_prompt": "", "experimental_thread_store": {"type": "local"}, "otel": {"exporter": "none", "trace_exporter": "none", "metrics_exporter": "none", "log_user_prompt": false, "log_agent_responses": false, "log_guardian_assessments": false}, "web_search": "disabled", "features": {}, "mcp_servers": {}});
    for flag in topics_runtime::protocol::DISABLED {
        safe["features"][*flag] = json!(false);
    }
    safe["features"]["multi_agent_v2"] = json!({"enabled":false,"disable_native_tools":true});
    assert!(
        topics_runtime::protocol::validate_config(&json!({"config":safe.clone()}), &[]).is_ok()
    );
    for (key, value) in [
        ("notify", json!(["/synthetic/notify"])),
        ("openai_base_url", json!("http://127.0.0.1:1")),
        ("chatgpt_base_url", json!("http://127.0.0.1:1")),
        (
            "model_providers",
            json!({"custom":{"base_url":"http://127.0.0.1:1"}}),
        ),
        (
            "experimental_thread_store_endpoint",
            json!("http://127.0.0.1:1"),
        ),
        ("compact_prompt", json!("synthetic instructions")),
        (
            "experimental_compact_prompt_file",
            json!("/synthetic/prompt"),
        ),
        (
            "otel",
            json!({"exporter":{"otlp-http":{"endpoint":"http://127.0.0.1:1"}}}),
        ),
    ] {
        let mut c = safe.clone();
        c[key] = value;
        assert!(
            topics_runtime::protocol::validate_config(&json!({"config":c}), &[]).is_err(),
            "unchecked field {key}"
        );
    }
}
#[test]
fn returned_non_ephemeral_thread_cannot_transmit() {
    assert!(topics_runtime::protocol::validate_thread(&json!({"approvalPolicy":"never","sandbox":{"type":"readOnly","networkAccess":false},"cwd":"/synthetic","instructionSources":[],"thread":{"id":"thread","ephemeral":false}}),std::path::Path::new("/synthetic")).is_err());
}

#[test]
fn canonical_uuid_aliases_and_missing_nullable_candidates_are_rejected() {
    let canonical = "10000000-0000-4000-8000-00000000000a";
    for alias in [
        canonical.to_uppercase(),
        canonical.replace("-", ""),
        format!("urn:uuid:{canonical}"),
        format!("{{{canonical}}}"),
    ] {
        let value = json!({"candidates":[{"kind":"topic","topic":{"id":alias,"topic_kind":"person","name":"Synthetic","aliases":[],"sensitivity":"cloud_allowed"}}]});
        assert!(topics_runtime::bridge::parse_candidates(&value).is_err());
    }
    let value = json!({"candidates":[{"kind":"claim","claim":{"id":canonical,"revision_id":"10000000-0000-4000-8000-00000000000b","subject_id":canonical,"predicate":"synthetic","value":"synthetic","sources":[],"supersedes":[],"recorded_at":"2026-11-03T00:00:00Z","valid_from":null,"valid_to":null}}]});
    assert!(topics_runtime::bridge::parse_candidates(&value).is_err());
}
#[test]
fn native_probe_capture_and_wait_are_bounded() {
    let mut output = std::process::Command::new("python3");
    output.args([
        "-I",
        "-c",
        "import sys;sys.stdout.write('x'*1000000);sys.stdout.flush()",
    ]);
    let result = topics_runtime::isolation::bounded_probe(output, b"").unwrap();
    assert!(result.stdout.len() <= 8192);
    assert!(result.stderr.len() <= 8192);
    let start = std::time::Instant::now();
    let mut wait = std::process::Command::new("python3");
    wait.args(["-I", "-c", "import time;time.sleep(10)"]);
    assert!(topics_runtime::isolation::bounded_probe(wait, b"").is_err());
    assert!(start.elapsed() < std::time::Duration::from_secs(3));
}
