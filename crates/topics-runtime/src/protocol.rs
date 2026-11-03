//! Version-specific, fail-closed app-server state and effective settings checks.
use serde_json::Value;

pub const MAX_FRAME: usize = 512 * 1024;
pub const MAX_OUTPUT: usize = 2 * 1024 * 1024;
pub const DISABLED: &[&str] = &[
    "shell_tool",
    "unified_exec",
    "hooks",
    "plugins",
    "apps",
    "view_image",
    "browser_use",
    "computer_use",
    "multi_agent",
    "multi_agent_v2",
    "sleep_tool",
    "goals",
    "tool_suggest",
    "skill_search",
    "image_generation",
    "token_budget",
    "request_permissions_tool",
    "deferred_executor",
    "send_message_to_user_async",
];
/// Validate one complete assembled inventory, including disabled inherited servers.
pub fn validate_inventory(value: &Value) -> Result<(), String> {
    if !value.get("nextCursor").is_some_and(Value::is_null) {
        return Err("incomplete_inventory".into());
    }
    let rows = value["data"].as_array().ok_or("invalid_inventory")?;

    let mut names = std::collections::HashSet::new();
    for row in rows {
        let name = row["name"].as_str().ok_or("invalid_server")?;
        if !names.insert(name) {
            return Err("duplicate_server".into());
        }
        let tools = row["tools"].as_object().ok_or("invalid_tools")?;
        if row["runtimeStatus"] != "disabled" || !tools.is_empty() || !row["toolsError"].is_null() {
            return Err("unexpected_codex_mcp_surface".into());
        }
    }
    Ok(())
}
pub fn validate_config(value: &Value, inherited: &[String]) -> Result<(), String> {
    let c = &value["config"];
    validate_privacy(c)?;
    if c["web_search"] != "disabled" {
        return Err("web_search_enabled".into());
    }
    for flag in DISABLED {
        let disabled = if *flag == "multi_agent_v2" {
            c["features"][*flag]["enabled"] == false
                && c["features"][*flag]["disable_native_tools"] == true
        } else {
            c["features"][*flag] == false
        };
        if !disabled {
            return Err(format!("unsafe_feature_{flag}"));
        }
    }
    for (name, value) in c["features"].as_object().ok_or("missing_feature_map")? {
        if name != "multi_agent_v2" && *value != false {
            return Err("unexpected_enabled_feature".into());
        }
    }
    let empty = serde_json::Map::new();
    let servers = if c["mcp_servers"].is_null() {
        &empty
    } else {
        c["mcp_servers"].as_object().ok_or("invalid_mcp_config")?
    };
    for (name, server) in servers {
        if !inherited.contains(name) || server["enabled"] != false {
            return Err("config_changed".into());
        }
    }
    for field in ["instructions", "developer_instructions"] {
        if c.get(field).is_some_and(|v| !v.is_null() && v != "") {
            return Err("inherited_instructions".into());
        }
    }
    Ok(())
}
pub fn validate_thread(value: &Value, cwd: &std::path::Path) -> Result<String, String> {
    if value["thread"]["ephemeral"] != true
        || value["approvalPolicy"] != "never"
        || value["sandbox"]["type"] != "readOnly"
        || value["sandbox"]["networkAccess"] != false
        || value["cwd"].as_str() != cwd.to_str()
        || value["runtimeWorkspaceRoots"]
            .as_array()
            .is_none_or(|roots| !roots.is_empty())
        || value["instructionSources"]
            .as_array()
            .is_none_or(|a| !a.is_empty())
    {
        return Err("unsafe_thread_configuration".into());
    }
    value["thread"]["id"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .ok_or("missing_thread_id".into())
}

/// Required identity fields are checked, not merely optional if present.
pub fn validate_event_context(frame: &Value, thread: &str, turn: &str) -> Result<(), String> {
    let method = frame["method"].as_str().ok_or("missing_method")?;
    let p = &frame["params"];
    if method.starts_with("item/") || method == "turn/moderationMetadata" {
        if p["threadId"] != thread || p["turnId"] != turn {
            return Err("wrong_or_missing_event_identity".into());
        }
    } else if method.starts_with("turn/") {
        if p["threadId"] != thread {
            return Err("wrong_or_missing_thread_identity".into());
        }
        if matches!(method, "turn/started" | "turn/completed") && p["turn"]["id"] != turn {
            return Err("wrong_or_missing_turn_identity".into());
        }
    } else if p.get("threadId").is_some_and(|id| id != thread) {
        return Err("wrong_thread_identity".into());
    }
    Ok(())
}

/// Known destinations and instruction/telemetry controls for the pinned runtime.
/// Empty tables merge in Codex, so custom providers/files are rejected, not erased.
pub const PRIVACY_OVERRIDES: &[&str] = &[
    "notify=[]",
    "openai_base_url=\"https://chatgpt.com/backend-api/codex\"",
    "chatgpt_base_url=\"https://chatgpt.com/backend-api/\"",
    "model_provider=\"openai\"",
    "skills.include_instructions=false",
    "agents.enabled=false",
    "features.multi_agent_v2.disable_native_tools=true",
    "tools.experimental_request_user_input.enabled=false",
    "tools.update_plan.enabled=false",
    "developer_instructions=\"\"",
    "project_doc_max_bytes=0",
    "history.persistence=\"none\"",
    "analytics.enabled=false",
    "feedback.enabled=false",
    "compact_prompt=\"\"",
    "otel.exporter=\"none\"",
    "otel.trace_exporter=\"none\"",
    "otel.metrics_exporter=\"none\"",
    "otel.log_user_prompt=false",
    "otel.log_agent_responses=false",
    "otel.log_guardian_assessments=false",
    "experimental_thread_store.type=\"local\"",
];
pub fn validate_privacy(c: &Value) -> Result<(), String> {
    if c["notify"].as_array().is_none_or(|a| !a.is_empty())
        || c["model_provider"] != "openai"
        || c["skills"]["include_instructions"] != false
        || c["agents"]["enabled"] != false
        || c["analytics"]["enabled"] != false
        || c["feedback"]["enabled"] != false
        || c["compact_prompt"] != ""
        || c["experimental_thread_store"]["type"] != "local"
    {
        return Err("unsafe_privacy_controls".into());
    }
    for field in [
        "profile",
        "experimental_thread_store_endpoint",
        "experimental_compact_prompt_file",
        "model_instructions_file",
        "experimental_realtime_ws_base_url",
        "experimental_realtime_webrtc_call_base_url",
    ] {
        if c.get(field).is_some_and(|v| !v.is_null()) {
            return Err(format!("inherited_{field}_unsupported"));
        }
    }
    if c.get("model_providers")
        .is_some_and(|v| !v.is_null() && v.as_object().is_none_or(|o| !o.is_empty()))
    {
        return Err("custom_model_providers_unsupported".into());
    }
    if c["chatgpt_base_url"] != "https://chatgpt.com/backend-api/"
        || c["openai_base_url"] != "https://chatgpt.com/backend-api/codex"
    {
        return Err("custom_provider_destination_unsupported".into());
    }
    let otel = c["otel"].as_object().ok_or("missing_telemetry_controls")?;
    for key in ["exporter", "trace_exporter", "metrics_exporter"] {
        if c["otel"][key] != "none" {
            return Err("telemetry_export_unsupported".into());
        }
    }
    for key in [
        "log_user_prompt",
        "log_agent_responses",
        "log_guardian_assessments",
    ] {
        if c["otel"][key] != false {
            return Err("telemetry_content_logging_unsupported".into());
        }
    }
    for (key, value) in otel {
        if ![
            "exporter",
            "trace_exporter",
            "metrics_exporter",
            "log_user_prompt",
            "log_agent_responses",
            "log_guardian_assessments",
        ]
        .contains(&key.as_str())
            && !value.is_null()
        {
            return Err("inherited_telemetry_settings_unsupported".into());
        }
    }
    Ok(())
}
