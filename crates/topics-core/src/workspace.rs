//! Canonical topic-workspace wire values. Store and runtime share these values.
use serde::{Deserialize, Serialize};

pub type Id = String;
pub type Hash = String;
pub type Timestamp = String;
pub type Date = String;
pub type WorkspaceResult<T> = Result<T, ApiError>;

macro_rules! value {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name { $(pub $field: $ty),* }
    };
}
value!(RootRef {
    root_id: Id,
    path: String
});
value!(Locator { start_byte: u64, end_byte: u64, heading: Option<String> });
value!(SourceRef {
    source_id: Id,
    path: String,
    document_hash: Hash,
    locator: Locator,
    passage_hash: Hash,
    excerpt: String
});
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Evidence {
    Passage { source: SourceRef },
    UserStatement { statement_id: Id },
}
value!(UserStatement {
    id: Id,
    topic_id: Id,
    text: String,
    recorded_at: Timestamp
});
value!(ScopeSpec { files: Vec<String>, folders: Vec<String>, date_from: Option<Date>, date_to: Option<Date> });
value!(ScopePreview { scope_id: Id, root_id: Id, sources: Vec<SourceSummary>, excluded: Vec<Exclusion> });
value!(SourceSummary { source_id: Id, path: String, hash: Hash, date: Option<Date>, sensitivity: Sensitivity });
value!(Exclusion {
    path: String,
    reason: String
});
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sensitivity {
    CloudAllowed,
    LocalOnly,
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopicKind {
    Person,
    Project,
    Thread,
}
value!(Topic { id: Id, topic_kind: TopicKind, name: String, aliases: Vec<String>, sensitivity: Sensitivity });
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Endpoint {
    Topic { id: Id },
    Claim { id: Id },
    Source { source_id: Id, locator: Locator },
}
value!(Claim { id: Id, revision_id: Id, subject_id: Id, predicate: String, value: String, uncertainty: Option<String>, sources: Vec<Evidence>, supersedes: Vec<Id>, recorded_at: Timestamp, valid_from: Option<Date>, valid_to: Option<Date> });
value!(Connection { id: Id, from: Endpoint, relation: String, to: Endpoint, uncertainty: Option<String>, sources: Vec<SourceRef>, valid_from: Option<Date>, valid_to: Option<Date> });
value!(Rule {
    id: Id,
    topic_id: Id,
    predicate: String,
    match_value: String,
    replacement: String,
    scope: RuleScope,
    source_revision_id: Id
});
value!(RuleScope { source_paths: Vec<String>, context: Option<String> });
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Candidate {
    Topic {
        topic: Topic,
    },
    Claim {
        claim: Claim,
    },
    Connection {
        connection: Connection,
    },
    Correction {
        claim: Claim,
        rule: Rule,
    },
    UnresolvedIdentity {
        name: String,
        candidates: Vec<Id>,
        sources: Vec<SourceRef>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Origin {
    Analysis { run_id: Id },
    Chat { topic_id: Id, message_id: Id },
    Manual,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Pending,
    Accepted,
    Rejected,
    Conflict,
}
value!(Proposal { id: Id, root_id: Id, scope_id: Option<Id>, origin: Origin, status: ProposalStatus, candidates: Vec<Candidate>, fingerprint: Hash, created_at: Timestamp });
value!(TopicView { topic: Topic, current_claims: Vec<Claim>, connections: Vec<Connection>, unresolved: Vec<Candidate>, history: Vec<Claim>, warnings: Vec<String> });
value!(Operation {
    id: Id,
    status: String,
    description: String,
    created_at: Timestamp
});
value!(ApiError { code: String, message: String, details: Option<serde_json::Value> });
impl ApiError {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }
}
impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for ApiError {}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Decision {
    Approve { candidates: Vec<Candidate> },
    Reject,
}
value!(RootGetRequest {});
value!(RootGetResult { root: Option<RootRef> });
value!(RootSetRequest {
    path: String,
    create: bool
});
value!(RootSetResult {
    root: RootRef,
    index: crate::IndexReport
});
value!(ScopePreviewRequest {
    root_id: Id,
    spec: ScopeSpec
});
value!(SensitivitySetRequest {
    root_id: Id,
    target: Endpoint,
    sensitivity: Sensitivity
});
value!(OperationResult {
    operation: Operation
});
value!(AnalysisStartRequest {
    root_id: Id,
    scope_id: Id,
    confirm_unknown: bool
});
value!(AnalysisStartResult { run_id: Id });
value!(RunCancelRequest {
    root_id: Id,
    run_id: Id
});
value!(RunCancelResult { cancelled: bool });
value!(RootRequest { root_id: Id });
value!(ProposalsListResult { proposals: Vec<Proposal> });
value!(ProposalDecideRequest {
    root_id: Id,
    proposal_id: Id,
    decision: Decision,
    expected_fingerprint: Hash
});
value!(ProposalDecideResult { operation: Option<Operation>, proposal: Proposal });
value!(TopicsListResult { topics: Vec<Topic> });
value!(TopicGetRequest {
    root_id: Id,
    topic_id: Id
});
value!(ChatStartRequest {
    root_id: Id,
    topic_id: Id,
    scope_id: Id,
    confirm_unknown: bool,
    message: String
});
value!(ChatStartResult {
    run_id: Id,
    message_id: Id
});
value!(HistoryListResult { operations: Vec<Operation> });
value!(UndoRequest {
    root_id: Id,
    operation_id: Id
});
value!(RuntimeStatusRequest {});
value!(RuntimeStatusResult { available: bool, version: Option<String>, message: String });
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RunEvent {
    TextDelta { text: String },
    Proposal { proposal_id: Id },
    Completed,
    Cancelled,
    Error { error: ApiError },
}
value!(WorkspaceRunEvent {
    root_id: Id,
    run_id: Id,
    seq: u64,
    event: RunEvent
});
// Internal application/runtime adapter requests. Never expose approval to a model.
value!(ScopeReadRequest {
    root_id: Id,
    scope_id: Id,
    run_id: Id
});
value!(ScopedSource {
    summary: SourceSummary,
    content: String,
    citation: SourceRef
});
value!(ScopeReadResult { sources: Vec<ScopedSource> });
value!(StageProposalRequest { root_id: Id, scope_id: Option<Id>, origin: Origin, candidates: Vec<Candidate> });
value!(SourceContextRequest {
    root_id: Id,
    scope_id: Id,
    run_id: Id,
    topic_id: Id
});
value!(UserStatementRequest {
    root_id: Id,
    topic_id: Id,
    text: String
});
