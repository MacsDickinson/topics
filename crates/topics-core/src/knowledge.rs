//! Read-only knowledge projection shared by the desktop and the local retrieval
//! MCP server, plus the human validation requests that sit beside it.
//! Nothing here is model authority: retrieval callers receive values only.
use crate::*;
use serde::{Deserialize, Serialize};

fn required<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<T, D::Error> {
    T::deserialize(d)
}
macro_rules! value {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        #[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name { $(#[serde(deserialize_with="required")] pub $field: $ty),* }
    };
}
macro_rules! choices {
    ($name:ident { $($variant:ident),* }) => {
        #[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(rename_all="snake_case")]
        pub enum $name { $($variant),* }
    };
}

// Accepted knowledge is committed Markdown. Pending knowledge is a discovery
// that still waits for the existing human decision.
choices!(KnowledgeStatus { Accepted, Pending });
// Why an assertion is worth a second look. Conflicting, uncertain and stale
// ask for a decision; unverified is informational and never blocks browsing.
choices!(CheckKind {
    Conflicting,
    Stale,
    Uncertain,
    Unverified
});
choices!(PendingKind {
    Proposal,
    SetupDraft
});
// Where a pending discovery stands before it can become accepted knowledge.
choices!(PendingState {
    Analysing,
    AwaitingDecision,
    NeedsAttention
});
choices!(MatchField {
    Name,
    Alias,
    Assertion
});
value!(CheckCounts {
    conflicting: u32,
    stale: u32,
    uncertain: u32,
    unverified: u32,
    deferred: u32
});
// The route to the existing decision for a pending discovery.
value!(PendingRef {
    kind: PendingKind,
    id: Id
});
value!(GraphNode { topic: Topic, status: KnowledgeStatus, assertions: u32, verified: bool, checks: CheckCounts, pending: Option<PendingRef> });
value!(GraphEdge { id: Id, from: Id, to: Id, relation: String, status: KnowledgeStatus, inferred: bool, confirmed: bool, valid_from: Option<Date>, valid_to: Option<Date>, sources: u32, pending: Option<PendingRef> });
value!(PendingDiscovery {
    decision: PendingRef,
    state: PendingState,
    topics: u32,
    assertions: u32,
    relationships: u32,
    needs_input: u32,
    created_at: Timestamp
});
value!(KnowledgeGraph { root_id: Id, nodes: Vec<GraphNode>, edges: Vec<GraphEdge>, pending: Vec<PendingDiscovery>, checks: CheckCounts, warnings: Vec<String> });

value!(TopicSearchRequest { root_id: Id, query: String, limit: Option<u32> });
value!(TopicHit { topic: Topic, status: KnowledgeStatus, matched: MatchField, snippet: Option<String>, checks: CheckCounts });
value!(TopicSearchResult { hits: Vec<TopicHit>, total: u32 });

#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AssertionEvidence {
    // The cited passage. `stale` means the source no longer matches the citation.
    Passage { source: SourceRef, stale: bool },
    // The human's own recorded words: a correction, edit or validation judgment.
    Statement { statement: UserStatement },
}
value!(Assertion { claim_id: Id, revision_id: Id, predicate: String, value: String, uncertainty: Option<String>, valid_from: Option<Date>, valid_to: Option<Date>, recorded_at: Timestamp, status: KnowledgeStatus, review: ReviewState, check: Option<CheckKind>, evidence: Vec<AssertionEvidence>, pending: Option<PendingRef> });
value!(TopicRef {
    id: Id,
    name: String,
    topic_kind: TopicKind
});
value!(Relationship { id: Id, relation: String, outgoing: bool, other: TopicRef, uncertainty: Option<String>, valid_from: Option<Date>, valid_to: Option<Date>, status: KnowledgeStatus, review: ReviewState, check: Option<CheckKind>, evidence: Vec<AssertionEvidence>, pending: Option<PendingRef> });
// One thing to validate in context. `revision_ids` are the competing or
// affected assertions; `connection_id` is set for a relationship instead.
value!(ValidationItem { id: Id, kind: CheckKind, predicate: String, revision_ids: Vec<Id>, connection_id: Option<Id>, deferred: bool });
// Different values for one predicate whose effective periods do not overlap:
// a change over time, not a disagreement.
value!(Timeline { predicate: String, revision_ids: Vec<Id> });
// What this caller was not given. Counts only, never content.
value!(Withheld {
    assertions: u32,
    relationships: u32,
    passages: u32
});
value!(TopicKnowledge { topic: Topic, status: KnowledgeStatus, pending: Option<PendingRef>, assertions: Vec<Assertion>, relationships: Vec<Relationship>, validation: Vec<ValidationItem>, timelines: Vec<Timeline>, earlier: Vec<Assertion>, context: Vec<UserStatement>, checks: CheckCounts, withheld: Withheld, warnings: Vec<String> });

// Human validation. Both actions end in the existing receipt and setup_decide.
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ValidationAction {
    // Keep this assertion (or relationship) as the right one.
    Confirm { id: Id },
    // None of the displayed values is right; record the human's value.
    Correct { value: String },
}
value!(ValidationPreviewRequest {
    root_id: Id,
    topic_id: Id,
    item_id: Id,
    action: ValidationAction,
    note: String
});
value!(ValidationPreview { receipt: SetupReceipt, statement: Option<UserStatement>, before: Vec<Claim>, after: Vec<Claim>, confirmed: Vec<Claim>, rules: Vec<Rule> });
value!(ValidationDeferRequest {
    root_id: Id,
    topic_id: Id,
    item_id: Id,
    deferred: bool
});

// Local MCP retrieval. The caller never names a root; the server is bound to
// the configured notes folder and to what its owner switched on.
value!(RetrievalSettings { enabled: bool, updated_at: Option<Timestamp> });
value!(RetrievalSettingsSetRequest {
    root_id: Id,
    enabled: bool
});
value!(RetrievalSearchRequest { query: String, limit: Option<u32> });
value!(RetrievalTopicRequest { topic_id: Id });
