use super::*;
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
value!(SetupBatch { batch_id: Id, status: SetupBatchStatus, scope_id: Option<Id>, run_id: Option<Id>, attempt: u32, error: Option<ApiError>, proposal_ids: Vec<Id>, completed_source_ids: Vec<Id> });
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
value!(DraftItem { item_id: Id, candidate: Candidate, disposition: DraftDisposition, batch_ids: Vec<Id>, proposal_ids: Vec<Id>, independent_evidence_groups: Vec<Hash>, assertion_kind: DraftAssertionKind });
value!(SetupIssue { id: Id, issue_kind: SetupIssueKind, key: String, item_ids: Vec<Id>, sources: Vec<SourceRef>, deferred: bool });
value!(DraftGraph { items: Vec<DraftItem>, issues: Vec<SetupIssue>, canonical_ids: BTreeMap<Id, Id> });
value!(SetupJob { schema: u32, job_id: Id, root_id: Id, manifest: SetupManifest, prior_consents: Vec<SetupConsent>, status: SetupJobStatus, revision: u64, batches: Vec<SetupBatch>, graph: DraftGraph, coverage: SetupCoverage, created_at: Timestamp });
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
value!(SetupConsent { manifest: SetupManifest, batches: Vec<SetupBatch> });
value!(SetupJobReconsentRequest { root_id: Id, job_id: Id, expected_revision: u64, manifest_id: Id, expected_fingerprint: Hash });
choices!(DraftAssertionKind { Unclassified, ReportedExplicit, InferredOrUncertain, Unsupported, IdentityQuestion, Correction });
