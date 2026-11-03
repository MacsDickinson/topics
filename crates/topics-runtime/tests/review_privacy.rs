use serde_json::json;
#[test]
fn inherited_notifications_providers_endpoints_telemetry_and_prompt_files_fail_closed() {
    let mut safe = json!({"openai_base_url":"https://api.openai.com/v1","chatgpt_base_url":"https://chatgpt.com/backend-api/","notify": [], "model_provider": "openai", "skills": {"include_instructions": false}, "agents": {"enabled": false}, "analytics": {"enabled": false}, "feedback": {"enabled": false}, "compact_prompt": "", "experimental_thread_store": {"type": "local"}, "otel": {"exporter": "none", "trace_exporter": "none", "metrics_exporter": "none", "log_user_prompt": false, "log_agent_responses": false, "log_guardian_assessments": false}, "web_search": "disabled", "features": {}, "mcp_servers": {"topics": {"enabled": true}}});
    for flag in topics_runtime::protocol::DISABLED {
        safe["features"][*flag] = json!(false);
    }
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
