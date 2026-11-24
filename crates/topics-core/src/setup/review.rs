use super::*;
choices!(ReviewState {
    FromNotes,
    ConfirmedByYou,
    NeedsInput,
    EarlierUnderstanding,
    AcceptedUnverified
});
value!(ReviewMetadata { id: Id, record_id: Id, state: ReviewState, receipt_id: Option<Id>, statement_id: Option<Id>, recorded_at: Timestamp });
value!(SetupTopicReview { topic: TopicView, reviews: Vec<ReviewMetadata>, context: Vec<UserStatement> });
choices!(SetupDecisionKind {
    Save,
    Confirm,
    ConfirmBoth,
    SaveAllPeriods
});
impl SetupDecisionKind {
    pub fn confirms_existing(self) -> bool {
        matches!(self, Self::Confirm | Self::ConfirmBoth)
    }
}
value!(SetupSavePreviewRequest { root_id: Id, job_id: Id, item_ids: Vec<Id> });
value!(SetupConfirmPreviewRequest { root_id: Id, revision_ids: Vec<Id> });
value!(SetupReceipt { schema: u32, receipt_id: Id, root_id: Id, kind: SetupDecisionKind, job_id: Option<Id>, job_revision: Option<u64>, requested_ids: Vec<Id>, included_ids: Vec<Id>, dependency_ids: Vec<Id>, deferred_ids: Vec<Id>, candidates: Vec<Candidate>, revisions: Vec<Claim>, connections: Vec<Connection>, expected_hashes: BTreeMap<String, Option<Hash>>, #[serde(default, skip_serializing_if = "Vec::is_empty")] stale_passages: Vec<SourceRef>, fingerprint: Hash, max_records_per_operation: u32, operation_count: u32, created_at: Timestamp });
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
value!(SetupConnectionConfirmPreviewRequest { root_id: Id, connection_ids: Vec<Id> });
