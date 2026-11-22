//! Internal local observations only; never serialized onto the model tool wire.
use crate::RunState;
use serde_json::{Value, json};
use std::{
    future::Future,
    sync::{Arc, Mutex},
    time::Duration,
};
use topics_core::workspace::ApiError;
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
    pub(crate) fn tag(self, error: impl Into<RuntimeFailure>) -> RuntimeFailure {
        let mut failure = error.into();
        if failure.reason.is_none() && failure.error.code != "cancelled" {
            failure.reason = Some(self);
        }
        failure
    }
    pub(crate) fn error(self, code: &str, message: &str) -> RuntimeFailure {
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
/// Runtime-owned provenance travels as a type, never through ApiError JSON.
#[derive(Debug)]
pub(crate) struct RuntimeFailure {
    error: ApiError,
    pub(crate) reason: Option<Failure>,
}
pub(crate) type RuntimeResult<T> = Result<T, RuntimeFailure>;
impl From<ApiError> for RuntimeFailure {
    fn from(mut error: ApiError) -> Self {
        // Generic errors cross an arbitrary host/service seam: even an exact
        // allowlisted pair is data, not evidence of a locally observed stage.
        error.details = None;
        Self {
            error,
            reason: None,
        }
    }
}
impl From<RuntimeFailure> for ApiError {
    fn from(failure: RuntimeFailure) -> Self {
        let mut error = failure.error;
        error.details = failure.reason.map(Failure::details);
        error
    }
}
impl std::ops::Deref for RuntimeFailure {
    type Target = ApiError;
    fn deref(&self) -> &Self::Target {
        &self.error
    }
}
impl std::fmt::Display for RuntimeFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}
/// Existing production budgets are passed unchanged; tests use a short synthetic future.
pub(crate) async fn bounded<T>(
    duration: Duration,
    future: impl Future<Output = RuntimeResult<T>>,
    failure: Failure,
    message: &str,
) -> RuntimeResult<T> {
    tokio::time::timeout(duration, future)
        .await
        .unwrap_or_else(|_| Err(failure.tag(crate::runtime_error(message))))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn contract_pairs_are_output_only_and_generic_error_details_never_become_provenance() {
        let expected: Vec<_> = include_str!("../../../features/folder-setup/FAILURE-ORIGINS.md")
            .lines()
            .filter(|line| line.starts_with("| `"))
            .map(|line| {
                let fields: Vec<_> = line.split('`').collect();
                json!({"phase":fields[1],"reason":fields[3]})
            })
            .collect();
        let all = [
            Failure::Version,
            Failure::Layout,
            Failure::Native,
            Failure::Startup,
            Failure::Discovery,
            Failure::DiscoveryDeadline,
            Failure::Transport,
            Failure::ReadDeadline,
            Failure::Protocol,
            Failure::McpSession,
            Failure::McpInventory,
            Failure::McpRequest,
            Failure::McpDeadline,
            Failure::Callback,
            Failure::Schema,
            Failure::HostValidation,
            Failure::HostDeadline,
            Failure::Host,
            Failure::Execution,
            Failure::ExecutionDeadline,
            Failure::Publication,
            Failure::PublicationDeadline,
            Failure::Terminal,
            Failure::ScopeNotRead,
        ];
        assert_eq!(all.map(Failure::details).as_slice(), expected.as_slice());
        for details in expected
            .into_iter()
            .chain([json!({"private":"PRIVATE_SENTINEL"}), json!(null)])
        {
            let raw = ApiError {
                code: "arbitrary_host_error".into(),
                message: "PRIVATE_SENTINEL".into(),
                details: Some(details),
            };
            let converted = RuntimeFailure::from(raw.clone());
            assert_eq!(converted.reason, None);
            assert_eq!(ApiError::from(converted).details, None);
            let typed = Failure::Publication.tag(raw);
            assert_eq!(typed.reason, Some(Failure::Publication));
            let layered = Failure::Execution.tag(typed);
            assert_eq!(layered.reason, Some(Failure::Publication));
            assert_eq!(
                ApiError::from(layered).details,
                Some(Failure::Publication.details())
            );
        }
    }
    #[tokio::test]
    async fn stage_deadlines_retain_local_origin_and_ignore_private_payload() {
        for failure in [Failure::DiscoveryDeadline, Failure::ExecutionDeadline] {
            let result: RuntimeResult<()> = bounded(
                Duration::from_millis(5),
                std::future::pending(),
                failure,
                "Runtime operation timed out",
            )
            .await;
            assert_eq!(
                ApiError::from(result.unwrap_err()).details,
                Some(failure.details())
            );
        }
        let error = Failure::Publication.tag(ApiError {
            code: "forbidden".into(),
            message: "unchanged internal error".into(),
            details: Some(
                json!({"phase":"host","reason":"validation_denied","raw":"PRIVATE_SENTINEL"}),
            ),
        });
        assert_eq!(
            ApiError::from(error).details,
            Some(Failure::Publication.details())
        );
    }
}
