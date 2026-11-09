//! Internal local observations only; never serialized onto the model tool wire.
use crate::RunState;
use serde_json::{Value, json};
use std::{
    future::Future,
    sync::{Arc, Mutex},
    time::Duration,
};
use topics_core::workspace::{ApiError, WorkspaceResult};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Failure {
    Version,
    Layout,
    Native,
    Startup,
    Discovery,
    DiscoveryDeadline,
    Transport,
    ReadDeadline,
    Protocol,
    McpSession,
    McpInventory,
    McpRequest,
    McpDeadline,
    Callback,
    Schema,
    HostValidation,
    HostDeadline,
    Host,
    Execution,
    ExecutionDeadline,
    Publication,
    PublicationDeadline,
    Terminal,
    ScopeNotRead,
}
impl Failure {
    pub(crate) fn details(self) -> Value {
        let (phase, reason) = match self {
            Self::Version => ("preflight", "version_check_failed"),
            Self::Layout => ("preflight", "layout_denied"),
            Self::Native => ("preflight", "native_denied"),
            Self::Startup => ("startup", "setup_failed"),
            Self::Discovery => ("discovery", "discovery_failed"),
            Self::DiscoveryDeadline => ("discovery", "discovery_deadline"),
            Self::Transport => ("transport", "transport_failed"),
            Self::ReadDeadline => ("transport", "read_deadline"),
            Self::Protocol => ("transport", "protocol_denied"),
            Self::McpSession => ("mcp", "session_failed"),
            Self::McpInventory => ("mcp", "inventory_denied"),
            Self::McpRequest => ("mcp", "request_failed"),
            Self::McpDeadline => ("mcp", "request_deadline"),
            Self::Callback => ("callback", "authorization_denied"),
            Self::Schema => ("candidate", "schema_denied"),
            Self::HostValidation => ("host", "validation_denied"),
            Self::HostDeadline => ("host", "call_deadline"),
            Self::Host => ("host", "host_failed"),
            Self::Execution => ("execution", "validation_denied"),
            Self::ExecutionDeadline => ("execution", "execution_deadline"),
            Self::Publication => ("publication", "preparation_failed"),
            Self::PublicationDeadline => ("publication", "preparation_deadline"),
            Self::Terminal => ("turn", "model_terminal_failed"),
            Self::ScopeNotRead => ("publication", "scope_not_read"),
        };
        json!({"phase":phase,"reason":reason})
    }
    pub(crate) fn from_details(details: Option<&Value>) -> Option<Self> {
        let object = details?.as_object()?;
        if object.len() != 2 {
            return None;
        }
        match (
            object.get("phase")?.as_str()?,
            object.get("reason")?.as_str()?,
        ) {
            ("preflight", "version_check_failed") => Some(Self::Version),
            ("preflight", "layout_denied") => Some(Self::Layout),
            ("preflight", "native_denied") => Some(Self::Native),
            ("startup", "setup_failed") => Some(Self::Startup),
            ("discovery", "discovery_failed") => Some(Self::Discovery),
            ("discovery", "discovery_deadline") => Some(Self::DiscoveryDeadline),
            ("transport", "transport_failed") => Some(Self::Transport),
            ("transport", "read_deadline") => Some(Self::ReadDeadline),
            ("transport", "protocol_denied") => Some(Self::Protocol),
            ("mcp", "session_failed") => Some(Self::McpSession),
            ("mcp", "inventory_denied") => Some(Self::McpInventory),
            ("mcp", "request_failed") => Some(Self::McpRequest),
            ("mcp", "request_deadline") => Some(Self::McpDeadline),
            ("callback", "authorization_denied") => Some(Self::Callback),
            ("candidate", "schema_denied") => Some(Self::Schema),
            ("host", "validation_denied") => Some(Self::HostValidation),
            ("host", "call_deadline") => Some(Self::HostDeadline),
            ("host", "host_failed") => Some(Self::Host),
            ("execution", "validation_denied") => Some(Self::Execution),
            ("execution", "execution_deadline") => Some(Self::ExecutionDeadline),
            ("publication", "preparation_failed") => Some(Self::Publication),
            ("publication", "preparation_deadline") => Some(Self::PublicationDeadline),
            ("turn", "model_terminal_failed") => Some(Self::Terminal),
            ("publication", "scope_not_read") => Some(Self::ScopeNotRead),
            _ => None,
        }
    }
    pub(crate) fn tag(self, mut error: ApiError) -> ApiError {
        if error.code != "cancelled" {
            error.details = Some(
                Self::from_details(error.details.as_ref())
                    .unwrap_or(self)
                    .details(),
            );
        }
        error
    }
    pub(crate) fn error(self, code: &str, message: &str) -> ApiError {
        self.tag(crate::error(code, message))
    }
    pub(crate) fn record(self, state: &Arc<Mutex<RunState>>) {
        if let Ok(mut state) = state.lock() {
            self.record_locked(&mut state);
        }
    }
    pub(crate) fn record_locked(self, state: &mut RunState) {
        // No publication/locking action here: diagnostics never change visibility.
        if !state.cancelled && !state.terminal && !state.publication_emitted {
            state.failure.get_or_insert(self);
        }
    }
}
/// Existing production budgets are passed unchanged; tests use a short synthetic future.
pub(crate) async fn bounded<T>(
    duration: Duration,
    future: impl Future<Output = WorkspaceResult<T>>,
    failure: Failure,
    message: &str,
) -> WorkspaceResult<T> {
    tokio::time::timeout(duration, future)
        .await
        .unwrap_or_else(|_| Err(failure.tag(crate::runtime_error(message))))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finite_contract_round_trips_and_unknown_or_extra_details_remain_unknown() {
        let pairs: Vec<_> = include_str!("../../../features/folder-setup/FAILURE-ORIGINS.md")
            .lines()
            .filter(|line| line.starts_with("| `"))
            .map(|line| {
                let fields: Vec<_> = line.split('`').collect();
                (fields[1], fields[3])
            })
            .collect();
        assert_eq!(pairs.len(), 24);
        for (phase, reason) in &pairs {
            let value = json!({"phase":phase,"reason":reason});
            assert_eq!(
                Failure::from_details(Some(&value)).unwrap().details(),
                value
            );
            let mut extra = value;
            extra["private"] = json!("PRIVATE_SENTINEL");
            assert_eq!(Failure::from_details(Some(&extra)), None);
        }
        for value in [
            json!({"phase":"PRIVATE_SENTINEL","reason":"PRIVATE_SENTINEL"}),
            json!({"phase":"host","reason":7}),
            json!(null),
        ] {
            assert_eq!(Failure::from_details(Some(&value)), None);
        }
    }
    #[tokio::test]
    async fn stage_deadlines_retain_local_origin_and_ignore_private_payload() {
        for failure in [Failure::DiscoveryDeadline, Failure::ExecutionDeadline] {
            let result: WorkspaceResult<()> = bounded(
                Duration::from_millis(5),
                std::future::pending(),
                failure,
                "Runtime operation timed out",
            )
            .await;
            assert_eq!(result.unwrap_err().details, Some(failure.details()));
        }
        let error = Failure::Publication.tag(ApiError {
            code: "forbidden".into(),
            message: "unchanged internal error".into(),
            details: Some(
                json!({"phase":"host","reason":"validation_denied","raw":"PRIVATE_SENTINEL"}),
            ),
        });
        assert_eq!(error.details, Some(Failure::Publication.details()));
    }
}
