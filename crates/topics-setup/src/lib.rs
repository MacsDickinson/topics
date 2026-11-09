//! Bounded folder setup orchestration over the canonical workspace service.
//!
//! Call blocking facade methods on the host's blocking executor. Core owns
//! consent, graph identity, checkpoints and all decisions; runtime owns the
//! process, containment and final publication permit. No approval tool lives here.
mod host;
#[cfg(test)]
mod tests;
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};
use tokio::sync::watch;
use topics_core::*;
use topics_runtime::{RunRequest, Runtime};
use topics_store::WorkspaceService;

fn error(code: &str, message: &str) -> ApiError {
    ApiError::new(code, message)
}

struct Active {
    request: SetupJobRequest,
    stopped: Arc<AtomicBool>,
    capability: Option<SetupBatchCapability>,
    restart: Option<SetupJobRequest>,
}
struct Inner {
    service: Arc<Mutex<WorkspaceService>>,
    runtime: Arc<Runtime>,
    lifecycle: Arc<Mutex<()>>,
    active: Mutex<Option<Active>>,
    snapshots: watch::Sender<Option<SetupJob>>,
    #[cfg(test)]
    before_publish: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
    #[cfg(test)]
    after_cancel: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
}
/// A single bounded sequential setup worker sharing the desktop runtime.
#[derive(Clone)]
pub struct SetupCoordinator {
    inner: Arc<Inner>,
}
impl SetupCoordinator {
    pub fn new(
        service: Arc<Mutex<WorkspaceService>>,
        runtime: Arc<Runtime>,
        lifecycle: Arc<Mutex<()>>,
    ) -> Self {
        let (snapshots, _) = watch::channel(None);
        Self {
            inner: Arc::new(Inner {
                service,
                runtime,
                lifecycle,
                active: Mutex::new(None),
                snapshots,
                #[cfg(test)]
                before_publish: Mutex::new(None),
                #[cfg(test)]
                after_cancel: Mutex::new(None),
            }),
        }
    }
    /// Start a consented Ready job. Consent is created by core, never here.
    /// Finished jobs return their snapshot without transmitting again.
    pub fn start(&self, request: SetupJobRequest) -> WorkspaceResult<SetupJob> {
        let _gate = self
            .inner
            .lifecycle
            .lock()
            .map_err(|_| error("internal", "Setup lifecycle unavailable"))?;
        self.start_locked(request)
    }
    fn start_locked(&self, request: SetupJobRequest) -> WorkspaceResult<SetupJob> {
        if self
            .inner
            .active
            .lock()
            .map_err(|_| error("internal", "Setup worker unavailable"))?
            .is_some()
        {
            return Err(error(
                "conflict",
                "A model job is running or stopping; resume after its terminal event",
            ));
        }
        let job = self.get(request.clone())?;
        if matches!(
            job.status,
            SetupJobStatus::Finished | SetupJobStatus::FinishedWithGaps
        ) {
            self.inner.publish(job.clone());
            return Ok(job);
        }
        if !matches!(job.status, SetupJobStatus::Ready | SetupJobStatus::Running) {
            return Err(error(
                "conflict",
                "Resume the job or review changed transmission consent first",
            ));
        }
        if job.batches.len() > job.manifest.limits.max_batches as usize
            || job.manifest.limits.max_turns_per_batch != 1
        {
            return Err(error(
                "invalid_input",
                "Setup needs bounded batches with one runtime turn each",
            ));
        }
        let stopped = Arc::new(AtomicBool::new(false));
        *self
            .inner
            .active
            .lock()
            .map_err(|_| error("internal", "Setup worker unavailable"))? = Some(Active {
            request: request.clone(),
            stopped: stopped.clone(),
            capability: None,
            restart: None,
        });
        self.inner.publish(job.clone());
        let worker = self.clone();
        let worker_stop = stopped.clone();
        if std::thread::Builder::new()
            .name("topics-setup".into())
            .spawn(move || {
                worker.work(request, worker_stop);
            })
            .is_err()
        {
            self.finish(&stopped);
            return Err(error(
                "runtime_unavailable",
                "Could not start the setup worker",
            ));
        }
        Ok(job)
    }
    /// Runtime cancellation occurs before core revocation and without service or
    /// store locks. Resume/retry process only the checkpoints selected by core.
    pub fn control(&self, request: SetupJobControlRequest) -> WorkspaceResult<SetupJob> {
        let _gate = self
            .inner
            .lifecycle
            .lock()
            .map_err(|_| error("internal", "Setup lifecycle unavailable"))?;
        let key = SetupJobRequest {
            root_id: request.root_id.clone(),
            job_id: request.job_id.clone(),
        };
        match request.action {
            SetupJobAction::Pause | SetupJobAction::Cancel => self.stop(Some(&key))?,
            SetupJobAction::Resume | SetupJobAction::RetryFailed => {
                if self
                    .inner
                    .active
                    .lock()
                    .map_err(|_| error("internal", "Setup worker unavailable"))?
                    .as_ref()
                    .is_some_and(|active| active.request != key)
                {
                    return Err(error(
                        "conflict",
                        "Another setup job is running or stopping",
                    ));
                }
            }
        }
        let job = self
            .inner
            .with_service(|service| service.setup_job_control(request.clone()))?;
        self.inner.publish(job.clone());
        if matches!(
            request.action,
            SetupJobAction::Resume | SetupJobAction::RetryFailed
        ) {
            // The previous turn may have committed its terminal checkpoint but
            // still be cleaning up its child. Queue this explicit resume until
            // its worker drains; do not race Runtime's one-active-run slot.
            let queued = {
                let mut active = self
                    .inner
                    .active
                    .lock()
                    .map_err(|_| error("internal", "Setup worker unavailable"))?;
                if let Some(active) = active.as_mut() {
                    active.restart = Some(key.clone());
                    active.stopped.store(true, Ordering::Release);
                    true
                } else {
                    false
                }
            };
            if queued {
                Ok(job)
            } else {
                self.start_locked(key)
            }
        } else {
            Ok(job)
        }
    }
    pub fn get(&self, request: SetupJobRequest) -> WorkspaceResult<SetupJob> {
        self.inner
            .with_service(|service| service.setup_job_get(request))
    }
    /// Caller holds the shared desktop lifecycle gate before root_set. This
    /// method must not acquire that gate again. Old context is removed from the
    /// subscription and the stopped worker cannot emit or start another batch.
    pub fn cancel_active(&self) -> WorkspaceResult<()> {
        let key = {
            let active = self
                .inner
                .active
                .lock()
                .map_err(|_| error("internal", "Setup worker unavailable"))?;
            active.as_ref().map(|a| a.request.clone())
        };
        self.stop(None)?;
        if let Some(key) = key {
            let job = self.get(key)?;
            if !matches!(
                job.status,
                SetupJobStatus::Finished | SetupJobStatus::FinishedWithGaps
            ) {
                self.inner.with_service(|service| {
                    service.setup_job_control(SetupJobControlRequest {
                        root_id: job.root_id,
                        job_id: job.job_id,
                        expected_revision: job.revision,
                        action: SetupJobAction::Pause,
                    })
                })?;
            }
        }
        self.inner.snapshots.send_replace(None);
        Ok(())
    }
    /// Bounded coalescing snapshots; consume changed().await in the host. A
    /// snapshot includes canonical job/root IDs and each batch's stable run ID.
    /// Use get() for a requested job on subscription/reconnection.
    pub fn subscribe(&self) -> watch::Receiver<Option<SetupJob>> {
        self.inner.snapshots.subscribe()
    }
    fn stop(&self, key: Option<&SetupJobRequest>) -> WorkspaceResult<()> {
        let capability = {
            let mut active = self
                .inner
                .active
                .lock()
                .map_err(|_| error("internal", "Setup worker unavailable"))?;
            match active
                .as_mut()
                .filter(|a| key.is_none_or(|key| key == &a.request))
            {
                Some(active) => {
                    active.stopped.store(true, Ordering::Release);
                    active.restart = None;
                    active.capability.clone()
                }
                None => None,
            }
        };
        if let Some(capability) = capability {
            self.inner
                .runtime
                .cancel(&capability.root_id, &capability.run_id);
            #[cfg(test)]
            if let Some(hook) = self
                .inner
                .after_cancel
                .lock()
                .map_err(|_| error("internal", "Synthetic cancellation hook unavailable"))?
                .clone()
            {
                hook();
            }
        }
        Ok(())
    }
    fn finish(&self, stopped: &Arc<AtomicBool>) {
        let restart = if let Ok(mut active) = self.inner.active.lock()
            && active
                .as_ref()
                .is_some_and(|a| Arc::ptr_eq(&a.stopped, stopped))
        {
            active.take().and_then(|a| a.restart)
        } else {
            None
        };
        if let Some(request) = restart {
            let _ = self.start(request);
        }
    }
    fn work(&self, request: SetupJobRequest, stopped: Arc<AtomicBool>) {
        // The loop is bounded by the canonical finite manifest; each selected
        // batch has exactly one bounded Runtime turn. Retrying is a human action.
        loop {
            let next = self.launch_next(&request, &stopped);
            let (capability, receiver) = match next {
                Ok(Some(next)) => next,
                Ok(None) | Err(_) => break,
            };
            let terminal = loop {
                match receiver.recv_timeout(Duration::from_secs(200)) {
                    Ok(envelope)
                        if envelope.root_id == capability.root_id
                            && envelope.run_id == capability.run_id =>
                    {
                        if matches!(
                            envelope.event,
                            RunEvent::Completed | RunEvent::Cancelled | RunEvent::Error { .. }
                        ) {
                            break envelope.event;
                        }
                    }
                    Ok(_) => {}
                    Err(_) => {
                        self.inner
                            .runtime
                            .cancel(&capability.root_id, &capability.run_id);
                        break RunEvent::Error {
                            error: error(
                                "runtime_unavailable",
                                "Model turn stopped without a terminal event",
                            ),
                        };
                    }
                }
            };
            if !stopped.load(Ordering::Acquire) {
                let result = self.inner.with_service(|service| {
                    let job = service.setup_job_get(request.clone())?;
                    let completed = job.batches.iter().any(|b| {
                        b.batch_id == capability.batch_id && b.status == SetupBatchStatus::Completed
                    });
                    if completed {
                        return Ok(job);
                    }
                    let failure = match terminal {
                        RunEvent::Cancelled => error(
                            "cancelled",
                            "Model turn cancelled; its buffered draft was discarded",
                        ),
                        _ => error(
                            "runtime_unavailable",
                            "Model turn failed; retry this batch to fill its coverage gap",
                        ),
                    };
                    service.setup_batch_fail(SetupBatchFailRequest {
                        capability: capability.clone(),
                        error: failure,
                    })
                });
                if let Ok(job) = result {
                    self.inner.publish_for(&stopped, job);
                }
            }
            // Control/root switch may have paused the batch already. No late
            // terminal callback mutates a stopped job or appends model output.
            if stopped.load(Ordering::Acquire) {
                break;
            }
            if let Ok(mut active) = self.inner.active.lock()
                && let Some(active) = active
                    .as_mut()
                    .filter(|a| Arc::ptr_eq(&a.stopped, &stopped))
            {
                active.capability = None;
            }
        }
        if !stopped.load(Ordering::Acquire)
            && let Ok(job) = self.get(request)
        {
            self.inner.publish_for(&stopped, job);
        }
        self.finish(&stopped);
    }
    fn launch_next(
        &self,
        request: &SetupJobRequest,
        stopped: &Arc<AtomicBool>,
    ) -> WorkspaceResult<Option<(SetupBatchCapability, mpsc::Receiver<WorkspaceRunEvent>)>> {
        let _gate = self
            .inner
            .lifecycle
            .lock()
            .map_err(|_| error("internal", "Setup lifecycle unavailable"))?;
        if stopped.load(Ordering::Acquire) {
            return Ok(None);
        }
        let prepared = self.inner.with_service(|service| {
            let job = service.setup_job_get(request.clone())?;
            if !matches!(job.status, SetupJobStatus::Ready | SetupJobStatus::Running) {
                return Ok(None);
            }
            let Some(batch) = job
                .batches
                .iter()
                .find(|b| b.status == SetupBatchStatus::Waiting)
            else {
                return Ok(None);
            };
            let root = service
                .root_get(RootGetRequest {})?
                .root
                .ok_or_else(|| error("no_root", "Select a notes folder"))?;
            if root.root_id != request.root_id {
                return Err(error("stale_scope", "Notes folder changed; preview again"));
            }
            let capability = service.setup_batch_start(SetupBatchStartRequest {
                root_id: request.root_id.clone(),
                job_id: request.job_id.clone(),
                batch_id: batch.batch_id.clone(),
            })?;
            let snapshot = service.setup_job_get(request.clone())?;
            Ok(Some((
                capability,
                PathBuf::from(root.path),
                job.manifest.limits.max_candidates_per_turn.min(64) as usize,
                snapshot,
            )))
        })?;
        let Some((capability, notes_root, max_candidates, snapshot)) = prepared else {
            return Ok(None);
        };
        if let Some(active) = self
            .inner
            .active
            .lock()
            .map_err(|_| error("internal", "Setup worker unavailable"))?
            .as_mut()
        {
            active.capability = Some(capability.clone());
        }
        self.inner.publish_for(stopped, snapshot);
        let host = Arc::new(host::SetupHost::new(
            self.inner.clone(),
            capability.clone(),
            stopped.clone(),
            max_candidates,
        ));
        let (sender, receiver) = mpsc::sync_channel(4);
        let sink = Arc::new(move |event: WorkspaceRunEvent| {
            // A setup host publishes one checkpoint ID; only its publication and
            // terminal envelopes enter this bounded queue. Text is untrusted and
            // does not become a topic briefing or an invented progress state.
            if !matches!(event.event, RunEvent::TextDelta { .. }) {
                let _ = sender.try_send(event);
            }
        });
        if let Err(failure) = self.inner.runtime.analysis_start(
            RunRequest {
                root_id: capability.root_id.clone(),
                scope_id: capability.scope_id.clone(),
                run_id: capability.run_id.clone(),
                notes_root,
                prompt: host::PROMPT.into(),
            },
            host,
            sink,
        ) {
            let job = self.inner.with_service(|service| {
                service.setup_batch_fail(SetupBatchFailRequest {
                    capability: capability.clone(),
                    error: error(
                        &failure.code,
                        "Could not start this model batch; retry when runtime is available",
                    ),
                })
            })?;
            self.inner.publish_for(stopped, job);
            return Err(failure);
        }
        Ok(Some((capability, receiver)))
    }
}
impl Inner {
    fn with_service<T>(
        &self,
        call: impl FnOnce(&mut WorkspaceService) -> WorkspaceResult<T>,
    ) -> WorkspaceResult<T> {
        let mut service = self
            .service
            .lock()
            .map_err(|_| error("internal", "Setup service unavailable"))?;
        call(&mut service)
    }
    fn publish(&self, job: SetupJob) {
        self.snapshots.send_if_modified(|current| {
            if current.as_ref().is_some_and(|old| {
                old.job_id == job.job_id
                    && old.root_id == job.root_id
                    && old.revision > job.revision
            }) {
                return false;
            }
            *current = Some(job);
            true
        });
    }
    fn publish_for(&self, stopped: &Arc<AtomicBool>, job: SetupJob) {
        // watch does not invoke user callbacks; no service/runtime calls occur
        // under this local identity mutex. A stopped generation cannot republish.
        if let Ok(active) = self.active.lock()
            && active.as_ref().is_some_and(|a| {
                Arc::ptr_eq(&a.stopped, stopped) && !stopped.load(Ordering::Acquire)
            })
        {
            self.publish(job);
        }
    }
}
