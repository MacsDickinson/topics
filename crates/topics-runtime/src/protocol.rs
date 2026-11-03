//! Version-specific, fail-closed app-server state and effective settings checks.
use serde_json::Value;

pub const MAX_FRAME: usize = 512 * 1024;
pub const MAX_OUTPUT: usize = 2 * 1024 * 1024;
pub const DISABLED: [&str; 9] = [
    "shell_tool",
    "unified_exec",
    "hooks",
    "plugins",
    "apps",
    "view_image",
    "browser_use",
    "computer_use",
    "multi_agent",
];
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Completed,
    Cancelled,
    Failed,
}
pub struct TurnBuffer {
    thread: String,
    turn: String,
    cancelled: bool,
    terminal: bool,
    completed: bool,
    candidates: Vec<Value>,
}
impl TurnBuffer {
    pub fn new(thread: &str, turn: &str) -> Self {
        Self {
            thread: thread.into(),
            turn: turn.into(),
            cancelled: false,
            terminal: false,
            completed: false,
            candidates: vec![],
        }
    }
    pub fn matches(&self, thread: &str, turn: &str) -> bool {
        self.thread == thread && self.turn == turn
    }
    pub fn buffer(&mut self, value: Value) -> Result<(), String> {
        if self.cancelled || self.terminal || self.candidates.len() >= 32 {
            return Err("run_closed".into());
        }
        self.candidates.push(value);
        Ok(())
    }
    pub fn cancel(&mut self) {
        self.cancelled = true;
        self.candidates.clear();
    }
    pub fn finish(&mut self, status: &str) -> Result<Outcome, String> {
        if self.terminal {
            return Err("duplicate_terminal".into());
        }
        self.terminal = true;
        if self.cancelled {
            self.candidates.clear();
            return Ok(Outcome::Cancelled);
        }
        match status {
            "completed" => {
                self.completed = true;
                Ok(Outcome::Completed)
            }
            "interrupted" => {
                self.candidates.clear();
                Ok(Outcome::Cancelled)
            }
            "failed" => {
                self.candidates.clear();
                Ok(Outcome::Failed)
            }
            _ => {
                self.candidates.clear();
                Err("unknown_terminal_status".into())
            }
        }
    }
    pub fn take_candidates(&mut self) -> Vec<Value> {
        if self.completed && !self.cancelled {
            std::mem::take(&mut self.candidates)
        } else {
            vec![]
        }
    }
}
/// Validate one complete assembled inventory, including disabled inherited servers.
pub fn validate_inventory(value: &Value) -> Result<(), String> {
    if !value.get("nextCursor").is_some_and(Value::is_null) {
        return Err("incomplete_inventory".into());
    }
    let rows = value["data"].as_array().ok_or("invalid_inventory")?;
    let mut topics = 0;
    let mut names = std::collections::HashSet::new();
    for row in rows {
        let name = row["name"].as_str().ok_or("invalid_server")?;
        if !names.insert(name) {
            return Err("duplicate_server".into());
        }
        let tools = row["tools"].as_object().ok_or("invalid_tools")?;
        if name == "topics" {
            topics += 1;
            if row["runtimeStatus"] != "connected" {
                return Err(format!(
                    "Topics MCP not connected ({}); check packaged proxy",
                    row["runtimeStatus"]
                        .as_str()
                        .filter(|s| [
                            "notStarted",
                            "starting",
                            "authenticationRequired",
                            "failed",
                            "cancelled",
                            "disabled"
                        ]
                        .contains(s))
                        .unwrap_or("unknown")
                ));
            }
            if !row["toolsError"].is_null()
                || tools.len() != 2
                || !tools.contains_key("scope_sources")
                || !tools.contains_key("propose_changes")
                || row["resources"].as_array().is_none_or(|a| !a.is_empty())
                || row["resourceTemplates"]
                    .as_array()
                    .is_none_or(|a| !a.is_empty())
            {
                return Err(format!(
                    "unexpected_topics_surface tools={} required={} resources={:?} templates={:?} catalog_error={}",
                    tools.len(),
                    tools.contains_key("scope_sources") && tools.contains_key("propose_changes"),
                    row["resources"].as_array().map(Vec::len),
                    row["resourceTemplates"].as_array().map(Vec::len),
                    !row["toolsError"].is_null()
                ));
            }
            let expected = crate::bridge::tools();
            for (name, definition) in tools {
                if definition["name"] != name.as_str()
                    || expected
                        .iter()
                        .find(|tool| tool.name == name.as_str())
                        .is_none_or(|tool| {
                            serde_json::to_value(tool).unwrap()["inputSchema"]
                                != definition["inputSchema"]
                        })
                {
                    return Err("invalid_tool_schema".into());
                }
            }
        } else if row["runtimeStatus"] != "disabled" || !tools.is_empty() {
            return Err("unexpected_tool_surface".into());
        }
    }
    if topics != 1 {
        return Err("topics_unavailable".into());
    }
    Ok(())
}
pub fn validate_config(value: &Value, inherited: &[String]) -> Result<(), String> {
    let c = &value["config"];
    if c["web_search"] != "disabled" {
        return Err("web_search_enabled".into());
    }
    for flag in DISABLED {
        if c["features"][flag] != false {
            return Err("unsafe_feature".into());
        }
    }
    let servers = c["mcp_servers"].as_object().ok_or("missing_mcp_config")?;
    for (name, server) in servers {
        if name == "topics" {
            if server["enabled"] != true {
                return Err("topics_disabled".into());
            }
        } else if !inherited.contains(name) || server["enabled"] != false {
            return Err("config_changed".into());
        }
    }
    if !servers.contains_key("topics") {
        return Err("topics_missing".into());
    }
    for field in ["instructions", "developer_instructions"] {
        if c.get(field).is_some_and(|v| !v.is_null() && v != "") {
            return Err("inherited_instructions".into());
        }
    }
    Ok(())
}
pub fn validate_thread(value: &Value, cwd: &std::path::Path) -> Result<String, String> {
    if value["approvalPolicy"] != "never"
        || value["sandbox"]["type"] != "readOnly"
        || value["sandbox"]["networkAccess"] != false
        || value["cwd"].as_str() != cwd.to_str()
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
