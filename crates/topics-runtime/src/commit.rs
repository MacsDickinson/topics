//! Cooperative publication fence. Blocking preparation belongs BEFORE publish;
//! its action must be the core-provided final short atomic visibility action.
use crate::{HostTools, diagnostics::Failure, error};
use std::{
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant},
};
use tokio::sync::{Semaphore, oneshot, watch};
use topics_core::workspace::{Candidate, WorkspaceResult};
#[derive(Clone, Debug)]
enum State {
    Open,
    Revoked,
    Published(Vec<String>),
}
struct Inner {
    deadline: Instant,
    state: Mutex<State>,
    changed: watch::Sender<State>,
}
#[derive(Clone)]
pub struct CommitPermit {
    inner: Arc<Inner>,
}
impl CommitPermit {
    pub(crate) fn new(timeout: Duration) -> Self {
        let (changed, _) = watch::channel(State::Open);
        Self {
            inner: Arc::new(Inner {
                deadline: Instant::now() + timeout,
                state: Mutex::new(State::Open),
                changed,
            }),
        }
    }
    /// Only pass the final action supplied by core.stage_proposal_guarded here.
    /// No arbitrary host code, preparation, callbacks, or blocking reads belong in
    /// this action. The final atomic rename cannot be universally preempted.
    pub fn publish(
        &self,
        ids: Vec<String>,
        action: &mut dyn FnMut() -> WorkspaceResult<()>,
    ) -> WorkspaceResult<()> {
        if ids.is_empty() || ids.len() > 64 || ids.iter().any(|id| !crate::canonical_id(id)) {
            return Err(error("invalid_input", "Invalid proposal publication IDs"));
        }
        let mut state = self
            .inner
            .state
            .lock()
            .map_err(|_| error("internal", "Publication fence unavailable"))?;
        if matches!(*state, State::Published(_)) {
            return Err(error("conflict", "Proposal publication already completed"));
        }
        if !matches!(*state, State::Open) || Instant::now() >= self.inner.deadline {
            *state = State::Revoked;
            self.inner.changed.send_replace(State::Revoked);
            return Err(error(
                "cancelled",
                "Proposal publication permit expired or revoked",
            ));
        }
        let result = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(action)) {
            Ok(result) => result,
            Err(_) => {
                *state = State::Revoked;
                self.inner.changed.send_replace(State::Revoked);
                return Err(error("internal", "Core publication action panicked"));
            }
        };
        if let Err(error) = result {
            *state = State::Revoked;
            self.inner.changed.send_replace(State::Revoked);
            return Err(error);
        }
        *state = State::Published(ids.clone());
        self.inner.changed.send_replace(State::Published(ids));
        Ok(())
    }
    pub(crate) fn published(&self) -> Option<Vec<String>> {
        match &*self.inner.state.lock().unwrap_or_else(|e| e.into_inner()) {
            State::Published(ids) => Some(ids.clone()),
            _ => None,
        }
    }
    /// false means publication is in progress/busy or already linearized; do not block the caller on the final atomic action.
    pub(crate) fn revoke(&self) -> bool {
        let mut state = match self.inner.state.try_lock() {
            Ok(state) => state,
            Err(std::sync::TryLockError::WouldBlock) => return false,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
        };
        if matches!(*state, State::Published(_)) {
            return false;
        }
        *state = State::Revoked;
        self.inner.changed.send_replace(State::Revoked);
        true
    }
}
pub(crate) async fn stage(
    host: Arc<dyn HostTools>,
    candidates: Vec<Candidate>,
    permit: CommitPermit,
) -> WorkspaceResult<Vec<String>> {
    static WORKERS: OnceLock<Arc<Semaphore>> = OnceLock::new();
    let slot = WORKERS
        .get_or_init(|| Arc::new(Semaphore::new(8)))
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            error(
                "runtime_unavailable",
                "Proposal preparation is busy; restart Topics if the host is unresponsive",
            )
        })?;
    let (tx, mut rx) = oneshot::channel();
    let worker_permit = permit.clone();
    std::thread::spawn(move || {
        let _slot = slot;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            host.commit_candidates(candidates, worker_permit)
        }))
        .unwrap_or_else(|_| Err(error("internal", "Host staging panicked")));
        let _ = tx.send(result);
    });
    let mut changed = permit.inner.changed.subscribe();
    loop {
        if let Some(ids) = permit.published() {
            return Ok(ids);
        }
        if matches!(*changed.borrow(), State::Revoked) {
            return Err(error(
                "cancelled",
                "Proposal preparation cancelled or expired",
            ));
        }
        let result = tokio::select! {
         _=tokio::time::sleep_until(tokio::time::Instant::from_std(permit.inner.deadline))=>{permit.revoke();Err(Failure::PublicationDeadline.error("runtime_unavailable","Proposal preparation timed out; later publication denied"))},
         result=&mut rx=>{permit.revoke();result.unwrap_or_else(|_|Err(error("internal","Proposal preparation disconnected"))).and_then(|_|Err(error("internal","Host did not use guarded proposal publication")))},
         _=changed.changed()=>continue,
        };
        // Published outcome wins even if host panics, stalls or errors afterwards.
        return permit.published().map_or(result, Ok);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn published_result_cannot_be_revoked_or_overwritten() {
        let permit = CommitPermit::new(Duration::from_secs(1));
        let id = uuid::Uuid::new_v4().to_string();
        let mut calls = 0;
        permit
            .publish(vec![id.clone()], &mut || {
                calls += 1;
                Ok(())
            })
            .unwrap();
        assert!(!permit.revoke());
        assert!(
            permit
                .publish(vec![uuid::Uuid::new_v4().to_string()], &mut || {
                    calls += 1;
                    Ok(())
                })
                .is_err()
        );
        assert_eq!(permit.published(), Some(vec![id]));
        assert_eq!(calls, 1);
    }
    #[test]
    fn revoke_does_not_wait_for_final_publication_action() {
        let permit = CommitPermit::new(Duration::from_secs(2));
        let worker = permit.clone();
        let id = uuid::Uuid::new_v4().to_string();
        let expected = id.clone();
        let (entered_tx, entered_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let thread = std::thread::spawn(move || {
            worker.publish(vec![id], &mut || {
                entered_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(1)).unwrap();
                Ok(())
            })
        });
        entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        let start = Instant::now();
        assert!(!permit.revoke());
        assert!(start.elapsed() < Duration::from_millis(100));
        release_tx.send(()).unwrap();
        thread.join().unwrap().unwrap();
        assert_eq!(permit.published(), Some(vec![expected]));
    }
}
