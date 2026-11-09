//! Bounded folder setup orchestration over the canonical workspace service.
//!
//! The desktop shares its service handle, Runtime and lifecycle gate with this
//! facade. Call blocking methods on the desktop blocking executor. No domain
//! persistence or approval authority lives in this crate.
use std::sync::{Arc, Mutex};
use tokio::sync::watch;
use topics_core::{ApiError, SetupJob, SetupJobControlRequest, SetupJobRequest, WorkspaceResult};
use topics_runtime::Runtime;
use topics_store::WorkspaceService;

/// Desktop adapter and job worker share the same lifecycle and service handles.
#[derive(Clone)]
pub struct SetupCoordinator {
    _service: Arc<Mutex<WorkspaceService>>,
    _runtime: Arc<Runtime>,
    _lifecycle: Arc<Mutex<()>>,
    snapshots: watch::Sender<Option<SetupJob>>,
}
impl SetupCoordinator {
    pub fn new(
        service: Arc<Mutex<WorkspaceService>>,
        runtime: Arc<Runtime>,
        lifecycle: Arc<Mutex<()>>,
    ) -> Self {
        let (snapshots, _) = watch::channel(None);
        Self {
            _service: service,
            _runtime: runtime,
            _lifecycle: lifecycle,
            snapshots,
        }
    }
    /// Start an already consented job; this method cannot create consent.
    pub fn start(&self, _request: SetupJobRequest) -> WorkspaceResult<SetupJob> {
        Err(ApiError::new(
            "not_implemented",
            "Setup orchestration pending",
        ))
    }
    /// Pause/cancel fences the active Runtime before core capability revocation.
    /// Resume/retry use core state and never recreate completed checkpoints.
    pub fn control(&self, _request: SetupJobControlRequest) -> WorkspaceResult<SetupJob> {
        Err(ApiError::new(
            "not_implemented",
            "Setup orchestration pending",
        ))
    }
    pub fn get(&self, request: SetupJobRequest) -> WorkspaceResult<SetupJob> {
        self._service
            .lock()
            .map_err(|_| ApiError::new("internal", "Setup service unavailable"))?
            .setup_job_get(request)
    }
    /// Call before changing root, while the desktop holds its lifecycle gate.
    /// This method itself does not acquire that gate, so it can share root_set's
    /// critical section. It never waits on a runtime callback or worker thread.
    pub fn cancel_active(&self) -> WorkspaceResult<()> {
        Err(ApiError::new(
            "not_implemented",
            "Setup orchestration pending",
        ))
    }
    /// Bounded coalescing snapshots, consumed with `changed().await` in the host.
    /// Snapshots carry canonical job/root IDs and active batch run IDs; callers
    /// use get() for a requested job when subscribing or reconnecting.
    pub fn subscribe(&self) -> watch::Receiver<Option<SetupJob>> {
        self.snapshots.subscribe()
    }
}
