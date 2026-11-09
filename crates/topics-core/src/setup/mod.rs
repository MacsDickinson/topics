//! Canonical human setup/review API. These types are never model authority.
use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(rename_all="snake_case")]
        pub enum $name { $($variant),* }
    };
}
choices!(FolderRole {
    Reference,
    Project,
    Meeting,
    Log,
    Other
});
value!(FolderRoleOverride {
    path: String,
    role: FolderRole
});
value!(InventoryDocument {
    source: SourceSummary,
    bytes: u64,
    title: String,
    linked: bool,
    role: FolderRole,
    role_reason: String
});
value!(InventoryFolder { path: String, role: FolderRole, reason: String, documents: u64, bytes: u64, dated: u64, undated: u64, linked: u64, date_from: Option<Date>, date_to: Option<Date> });
choices!(SetupEntry {
    Empty,
    New,
    Returning
});
value!(SetupInventoryRequest { root_id: Id, roles: Vec<FolderRoleOverride> });
value!(SetupInventory { schema: u32, root_id: Id, entry: SetupEntry, folders: Vec<InventoryFolder>, documents: Vec<InventoryDocument>, excluded: Vec<Exclusion>, eligible_bytes: u64, saved_topic_count: u64, resumable_job_ids: Vec<Id> });
value!(SetupLimits {
    max_files_per_batch: u32,
    max_bytes_per_batch: u64,
    max_batches: u32,
    max_candidates_per_turn: u32,
    max_turns_per_batch: u32
});
value!(SetupManifestPreviewRequest { root_id: Id, spec: ScopeSpec, roles: Vec<FolderRoleOverride>, limits: SetupLimits, runtime: String });
value!(SetupManifestSource {
    source: SourceSummary,
    bytes: u64,
    role: FolderRole
});
value!(SetupManifestBatch { batch_id: Id, sources: Vec<SetupManifestSource> });
value!(SetupManifest { schema: u32, manifest_id: Id, root_id: Id, fingerprint: Hash, spec: ScopeSpec, roles: Vec<FolderRoleOverride>, batches: Vec<SetupManifestBatch>, excluded: Vec<Exclusion>, limits: SetupLimits, runtime: String, created_at: Timestamp });
value!(SetupJobCreateRequest {
    root_id: Id,
    manifest_id: Id,
    expected_fingerprint: Hash
});
value!(SetupJobRequest {
    root_id: Id,
    job_id: Id
});
choices!(SetupJobStatus {
    Ready,
    Running,
    Paused,
    Cancelled,
    NeedsConsent,
    Finished,
    FinishedWithGaps
});
choices!(SetupBatchStatus {
    Waiting,
    Running,
    Completed,
    Failed,
    Cancelled,
    NeedsConsent
});
value!(SetupBatch { batch_id: Id, status: SetupBatchStatus, scope_id: Option<Id>, run_id: Option<Id>, attempt: u32, error: Option<ApiError>, proposal_ids: Vec<Id> });
value!(SetupCoverage { selected: u64, processed: Vec<String>, failed: Vec<String>, outstanding: Vec<String>, excluded: Vec<Exclusion>, complete: bool, model_calls: u64, transmission_authorizations: u64 });
choices!(DraftDisposition {
    Pending,
    Saved,
    Deferred,
    Dismissed
});
choices!(SetupIssueKind {
    Identity,
    Contradiction,
    Unsupported,
    Inferred,
    Replacement
});
value!(DraftItem { item_id: Id, candidate: Candidate, disposition: DraftDisposition, batch_ids: Vec<Id>, proposal_ids: Vec<Id>, independent_evidence_groups: Vec<Hash> });
value!(SetupIssue { id: Id, issue_kind: SetupIssueKind, key: String, item_ids: Vec<Id>, sources: Vec<SourceRef>, deferred: bool });
value!(DraftGraph { items: Vec<DraftItem>, issues: Vec<SetupIssue>, canonical_ids: BTreeMap<Id, Id> });
value!(SetupJob { schema: u32, job_id: Id, root_id: Id, manifest: SetupManifest, status: SetupJobStatus, revision: u64, batches: Vec<SetupBatch>, graph: DraftGraph, coverage: SetupCoverage, created_at: Timestamp });
choices!(SetupJobAction {
    Pause,
    Resume,
    Cancel,
    RetryFailed
});
value!(SetupJobControlRequest {
    root_id: Id,
    job_id: Id,
    expected_revision: u64,
    action: SetupJobAction
});
value!(SetupBatchStartRequest {
    root_id: Id,
    job_id: Id,
    batch_id: Id
});
value!(SetupBatchCapability {
    root_id: Id,
    job_id: Id,
    batch_id: Id,
    scope_id: Id,
    run_id: Id,
    manifest_fingerprint: Hash,
    attempt: u32
});
value!(SetupBatchStageRequest { capability: SetupBatchCapability, candidates: Vec<Candidate> });
value!(SetupBatchFailRequest {
    capability: SetupBatchCapability,
    error: ApiError
});
value!(SetupDraftEditRequest {
    root_id: Id,
    job_id: Id,
    expected_revision: u64,
    item_id: Id,
    candidate: Candidate
});
value!(SetupDraftDispositionRequest { root_id: Id, job_id: Id, expected_revision: u64, item_ids: Vec<Id>, disposition: DraftDisposition });
value!(SetupDraftContextRequest {
    capability: SetupBatchCapability
});
value!(SetupDraftContext { topics: Vec<Topic>, claims: Vec<Claim>, connections: Vec<Connection> });
choices!(ReviewState {
    FromNotes,
    ConfirmedByYou,
    NeedsInput,
    EarlierUnderstanding,
    AcceptedUnverified
});
value!(ReviewMetadata { id: Id, record_id: Id, state: ReviewState, receipt_id: Option<Id>, statement_id: Option<Id>, recorded_at: Timestamp });
value!(SetupTopicReview { topic: TopicView, reviews: Vec<ReviewMetadata> });
choices!(SetupDecisionKind { Save, Confirm });
value!(SetupSavePreviewRequest { root_id: Id, job_id: Id, item_ids: Vec<Id> });
value!(SetupConfirmPreviewRequest { root_id: Id, revision_ids: Vec<Id> });
value!(SetupReceipt { schema: u32, receipt_id: Id, root_id: Id, kind: SetupDecisionKind, job_id: Option<Id>, job_revision: Option<u64>, requested_ids: Vec<Id>, included_ids: Vec<Id>, dependency_ids: Vec<Id>, deferred_ids: Vec<Id>, candidates: Vec<Candidate>, revisions: Vec<Claim>, expected_hashes: BTreeMap<String, Option<Hash>>, fingerprint: Hash, max_records_per_operation: u32, operation_count: u32, created_at: Timestamp });
value!(SetupDecisionRequest {
    root_id: Id,
    receipt_id: Id,
    expected_fingerprint: Hash
});
choices!(SetupDecisionStatus {
    Completed,
    Partial,
    Conflict
});
value!(SetupDecisionResult { receipt: SetupReceipt, status: SetupDecisionStatus, operations: Vec<Operation>, saved_ids: Vec<Id>, deferred_ids: Vec<Id>, error: Option<ApiError> });
value!(SetupCorrectionPreviewRequest { root_id: Id, topic_id: Id, revision_ids: Vec<Id>, predicate: String, replacement: String, statement: String, rule_scope: RuleScope });
value!(SetupCorrectionPreview { receipt: SetupReceipt, statement: UserStatement, before: Vec<Claim>, after: Vec<Claim>, rule: Rule });
