//! Folder setup persistence and group decisions.
use super::*;
fn pending<T>() -> WResult<T> { Err(error("not_implemented", "Folder setup implementation pending")) }
impl WorkspaceService {
    pub fn setup_inventory(&self, _request: SetupInventoryRequest) -> WResult<SetupInventory> { pending() }
    pub fn setup_manifest_preview(&self, _request: SetupManifestPreviewRequest) -> WResult<SetupManifest> { pending() }
    pub fn setup_job_create(&self, _request: SetupJobCreateRequest) -> WResult<SetupJob> { pending() }
    pub fn setup_job_get(&self, _request: SetupJobRequest) -> WResult<SetupJob> { pending() }
    pub fn setup_jobs_list(&self, _request: RootRequest) -> WResult<Vec<SetupJob>> { pending() }
    pub fn setup_job_control(&self, _request: SetupJobControlRequest) -> WResult<SetupJob> { pending() }
    pub fn setup_batch_start(&self, _request: SetupBatchStartRequest) -> WResult<SetupBatchCapability> { pending() }
    pub fn setup_batch_validate(&self, _request: SetupBatchStageRequest) -> WResult<()> { pending() }
    pub fn setup_batch_stage_guarded(&self, _request: SetupBatchStageRequest, _publisher: impl FnOnce(&Proposal, &mut dyn FnMut() -> WResult<()>) -> WResult<()>) -> WResult<SetupJob> { pending() }
    pub fn setup_batch_fail(&self, _request: SetupBatchFailRequest) -> WResult<SetupJob> { pending() }
    pub fn setup_draft_context(&self, _request: SetupDraftContextRequest) -> WResult<SetupDraftContext> { pending() }
    pub fn setup_draft_edit(&self, _request: SetupDraftEditRequest) -> WResult<SetupJob> { pending() }
    pub fn setup_draft_disposition(&self, _request: SetupDraftDispositionRequest) -> WResult<SetupJob> { pending() }
    pub fn setup_save_preview(&self, _request: SetupSavePreviewRequest) -> WResult<SetupReceipt> { pending() }
    pub fn setup_confirm_preview(&self, _request: SetupConfirmPreviewRequest) -> WResult<SetupReceipt> { pending() }
    pub fn setup_correction_preview(&self, _request: SetupCorrectionPreviewRequest) -> WResult<SetupCorrectionPreview> { pending() }
    pub fn setup_decide(&self, _request: SetupDecisionRequest) -> WResult<SetupDecisionResult> { pending() }
    pub fn setup_topic_review(&self, _request: TopicGetRequest) -> WResult<SetupTopicReview> { pending() }
}
