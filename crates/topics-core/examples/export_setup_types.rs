//! Export the canonical setup wire contract and all transitive workspace types.
use std::path::PathBuf;
use topics_core::setup::*;
use ts_rs::{Config, TS};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("expected output directory")?;
    let config = Config::default()
        .with_out_dir(output)
        .with_large_int("number");
    macro_rules! export { ($($ty:ty),+ $(,)?) => { $(<$ty>::export_all(&config)?;)+ }; }
    export!(
        SetupInventoryRequest,
        SetupInventory,
        SetupManifestPreviewRequest,
        SetupManifest,
        SetupJobCreateRequest,
        SetupJobRequest,
        SetupJob,
        SetupJobControlRequest,
        SetupBatchStartRequest,
        SetupBatchCapability,
        SetupBatchStageRequest,
        SetupBatchFailRequest,
        SetupDraftContextRequest,
        SetupDraftContext,
        SetupDraftEditRequest,
        SetupDraftDispositionRequest,
        SetupSavePreviewRequest,
        SetupConfirmPreviewRequest,
        SetupReceipt,
        SetupCorrectionPreviewRequest,
        SetupCorrectionPreview,
        SetupDecisionRequest,
        SetupDecisionResult,
        SetupTopicReview,
    );
    Ok(())
}
