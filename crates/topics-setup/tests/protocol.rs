//! Owned protocol scenarios: real orchestrator + canonical store + scripted child.
//! No configured provider, personal source folder or model call is used.
#![cfg(target_os = "macos")]
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use topics_core::*;
use topics_runtime::{Runtime, RuntimeConfig};
use topics_setup::SetupCoordinator;
use topics_store::WorkspaceService;

// Run runtime-bearing tests in a child with a synthetic Codex home so the
// runtime's environment checks never inspect personal configuration metadata.
fn isolated(name: &str, test: impl FnOnce()) {
    if std::env::var("TOPICS_SETUP_SYNTHETIC_TEST").as_deref() == Ok(name) {
        test();
        return;
    }
    let home = tempfile::tempdir().unwrap();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name, "--nocapture"])
        .env("TOPICS_SETUP_SYNTHETIC_TEST", name)
        .env("CODEX_HOME", home.path())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(150);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "synthetic protocol child failed: {name}");
            break;
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("bounded synthetic protocol child deadline: {name}");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
struct Fixture {
    temp: tempfile::TempDir,
    service: Arc<Mutex<WorkspaceService>>,
    coordinator: SetupCoordinator,
    root: RootRef,
    files: Vec<String>,
}
impl Fixture {
    fn new(documents: usize, mode: &str) -> Self {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().unwrap();
        let notes = temp.path().join("notes");
        fs::create_dir_all(notes.join("profiles")).unwrap();
        fs::create_dir_all(notes.join("meetings")).unwrap();
        let files: Vec<_> = (0..documents)
            .map(|i| {
                let path = format!("{}/{i:03}.md", if i < 2 { "profiles" } else { "meetings" });
                fs::write(
                    notes.join(&path),
                    "# Synthetic Alex A42\n\nAlex A42 leads Ledger.\n",
                )
                .unwrap();
                path
            })
            .collect();
        let mut service = WorkspaceService::open(temp.path().join("app-data")).unwrap();
        let root = service
            .root_set(RootSetRequest {
                path: notes.to_str().unwrap().into(),
                create: false,
            })
            .unwrap()
            .root;
        let script = temp.path().join("scripted-codex");
        let source = include_str!("support/scripted_child.py")
            .replace(
                "@SETTINGS@",
                &serde_json::to_string(&temp.path().join("mode")).unwrap(),
            )
            .replace(
                "@LOG@",
                &serde_json::to_string(&temp.path().join("transmissions.jsonl")).unwrap(),
            )
            .replace(
                "@FLAGS@",
                &serde_json::to_string(topics_runtime::protocol::DISABLED).unwrap(),
            );
        fs::write(&script, source).unwrap();
        fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(temp.path().join("mode"), mode).unwrap();
        let runtime = Arc::new(Runtime::new(RuntimeConfig {
            codex_path: script,
            proxy_path: PathBuf::from("/usr/bin/false"), // Direct-mode compatibility field; unused.
            app_data: temp.path().join("app-data"),
        }));
        let service = Arc::new(Mutex::new(service));
        let coordinator = SetupCoordinator::new(service.clone(), runtime, Arc::new(Mutex::new(())));
        Self {
            temp,
            service,
            coordinator,
            root,
            files,
        }
    }
    fn authorize(&self, per_batch: u32) -> SetupJob {
        let service = self.service.lock().unwrap();
        let manifest = service
            .setup_manifest_preview(SetupManifestPreviewRequest {
                root_id: self.root.root_id.clone(),
                spec: ScopeSpec {
                    files: self.files.clone(),
                    folders: vec![],
                    date_from: None,
                    date_to: None,
                },
                roles: vec![
                    FolderRoleOverride {
                        path: "profiles".into(),
                        role: FolderRole::Reference,
                    },
                    FolderRoleOverride {
                        path: "meetings".into(),
                        role: FolderRole::Meeting,
                    },
                ],
                limits: SetupLimits {
                    max_files_per_batch: per_batch,
                    max_bytes_per_batch: 65536,
                    max_batches: 100,
                    max_candidates_per_turn: 64,
                    max_turns_per_batch: 1,
                },
                runtime: "codex-cli 0.166.0".into(),
            })
            .expect("reviewed manifest must be available without transmitting");
        service
            .setup_job_create(SetupJobCreateRequest {
                root_id: self.root.root_id.clone(),
                manifest_id: manifest.manifest_id,
                expected_fingerprint: manifest.fingerprint,
            })
            .unwrap()
    }
    fn request(&self, job: &SetupJob) -> SetupJobRequest {
        SetupJobRequest {
            root_id: self.root.root_id.clone(),
            job_id: job.job_id.clone(),
        }
    }
    fn wait(&self, job: &SetupJob, predicate: impl Fn(&SetupJob) -> bool) -> SetupJob {
        let deadline = Instant::now() + Duration::from_secs(90);
        loop {
            let snapshot = self.coordinator.get(self.request(job)).unwrap();
            if predicate(&snapshot) {
                return snapshot;
            }
            assert!(
                Instant::now() < deadline,
                "job did not reach expected snapshot: {:?}",
                snapshot.status
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    fn done(&self, job: &SetupJob) -> SetupJob {
        self.wait(job, |s| {
            matches!(
                s.status,
                SetupJobStatus::Finished | SetupJobStatus::FinishedWithGaps
            )
        })
    }
    fn transmissions(&self) -> Vec<Value> {
        fs::read_to_string(self.temp.path().join("transmissions.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
}
#[test]
fn inventory_is_local_and_consent_starts_one_bounded_job() {
    isolated(
        "inventory_is_local_and_consent_starts_one_bounded_job",
        || {
            let f = Fixture::new(6, "empty");
            let inventory = f
                .service
                .lock()
                .unwrap()
                .setup_inventory(SetupInventoryRequest {
                    root_id: f.root.root_id.clone(),
                    roles: vec![],
                })
                .unwrap();
            assert_eq!(inventory.documents.len(), 6);
            let job = f.authorize(2);
            assert!(
                f.transmissions().is_empty(),
                "preview and consent do not transmit"
            );
            f.coordinator
                .start(f.request(&job))
                .expect("authorized setup must schedule bounded turns");
            let done = f.done(&job);
            assert_eq!(done.coverage.processed.len(), 6);
            assert_eq!(done.coverage.transmission_authorizations, 1);
            assert_eq!(f.transmissions().len(), 3);
            assert!(
                done.graph.items.is_empty(),
                "empty success must not fabricate candidates"
            );
            assert!(
                done.batches
                    .iter()
                    .all(|b| b.status == SetupBatchStatus::Completed)
            );
            for path in &f.files {
                assert_eq!(
                    fs::read_to_string(PathBuf::from(&f.root.path).join(path)).unwrap(),
                    "# Synthetic Alex A42\n\nAlex A42 leads Ledger.\n"
                );
            }
        },
    );
}
#[test]
fn hundred_documents_use_exact_multiple_scopes_and_one_draft_graph() {
    isolated(
        "hundred_documents_use_exact_multiple_scopes_and_one_draft_graph",
        || {
            let f = Fixture::new(100, "graph");
            let job = f.authorize(20);
            f.coordinator.start(f.request(&job)).unwrap();
            let done = f.done(&job);
            assert!(done.coverage.complete);
            assert_eq!(done.coverage.processed.len(), 100);
            assert_eq!(done.coverage.model_calls, 5);
            assert_eq!(done.coverage.transmission_authorizations, 1);
            assert_eq!(
                done.graph
                    .items
                    .iter()
                    .filter(|i| matches!(i.candidate, Candidate::Topic { .. }))
                    .count(),
                2
            );
            assert_eq!(
                done.graph
                    .items
                    .iter()
                    .filter(|i| matches!(i.candidate, Candidate::Connection { .. }))
                    .count(),
                1
            );
            let relationship = done
                .graph
                .items
                .iter()
                .find(|i| matches!(i.candidate, Candidate::Connection { .. }))
                .unwrap();
            if let Candidate::Connection { connection } = &relationship.candidate {
                assert_eq!(connection.sources.len(), 100);
            }
            assert_eq!(
                relationship.independent_evidence_groups.len(),
                1,
                "exact copies are not independent corroboration"
            );
            let sent = f.transmissions();
            assert_eq!(sent.len(), 5);
            let mut paths = std::collections::BTreeSet::new();
            for batch in sent {
                for source in batch["context"]["sources"].as_array().unwrap() {
                    assert!(
                        paths.insert(source["summary"]["path"].as_str().unwrap().to_owned()),
                        "source retransmitted"
                    );
                }
            }
            assert_eq!(paths.len(), 100);
            assert!(
                f.service
                    .lock()
                    .unwrap()
                    .topics_list(RootRequest {
                        root_id: f.root.root_id.clone()
                    })
                    .unwrap()
                    .topics
                    .is_empty(),
                "model completion cannot accept the graph"
            );
        },
    );
}
#[test]
fn failed_turn_preserves_other_batches_and_retry_does_not_duplicate() {
    isolated(
        "failed_turn_preserves_other_batches_and_retry_does_not_duplicate",
        || {
            let f = Fixture::new(6, "fail_first");
            let job = f.authorize(2);
            f.coordinator.start(f.request(&job)).unwrap();
            let gaps = f.done(&job);
            assert!(!gaps.coverage.complete);
            assert_eq!(gaps.coverage.failed.len(), 2);
            assert_eq!(gaps.coverage.processed.len(), 4);
            assert_eq!(gaps.coverage.model_calls, 3);
            let ids: Vec<_> = gaps.graph.items.iter().map(|i| i.item_id.clone()).collect();
            f.coordinator
                .control(SetupJobControlRequest {
                    root_id: f.root.root_id.clone(),
                    job_id: job.job_id.clone(),
                    expected_revision: gaps.revision,
                    action: SetupJobAction::RetryFailed,
                })
                .unwrap();
            let done = f.done(&job);
            assert!(done.coverage.complete);
            assert_eq!(done.coverage.model_calls, 4);
            assert_eq!(done.coverage.processed.len(), 6);
            assert_eq!(
                done.graph
                    .items
                    .iter()
                    .map(|i| i.item_id.clone())
                    .collect::<Vec<_>>(),
                ids
            );
        },
    );
}
#[test]
fn pause_reopen_and_changed_waiting_manifest_never_retransmit_completed_inputs() {
    isolated(
        "pause_reopen_and_changed_waiting_manifest_never_retransmit_completed_inputs",
        || {
            let f = Fixture::new(6, "hold_second");
            let job = f.authorize(2);
            f.coordinator.start(f.request(&job)).unwrap();
            let deadline = Instant::now() + Duration::from_secs(30);
            while f.transmissions().len() < 2 {
                assert!(Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(10));
            }
            let running = f.coordinator.get(f.request(&job)).unwrap();
            let paused = f
                .coordinator
                .control(SetupJobControlRequest {
                    root_id: f.root.root_id.clone(),
                    job_id: job.job_id.clone(),
                    expected_revision: running.revision,
                    action: SetupJobAction::Pause,
                })
                .unwrap();
            assert_eq!(paused.status, SetupJobStatus::Paused);
            assert_eq!(paused.coverage.processed.len(), 2);
            let completed = paused
                .batches
                .iter()
                .find(|b| b.status == SetupBatchStatus::Completed)
                .unwrap();
            let waiting = paused
                .manifest
                .batches
                .iter()
                .find(|b| b.batch_id != completed.batch_id)
                .unwrap();
            fs::write(
                PathBuf::from(&f.root.path).join(&waiting.sources[0].source.path),
                "# Changed unsent bytes\n",
            )
            .unwrap();
            fs::write(
                PathBuf::from(&f.root.path).join("meetings/new.md"),
                "# New source not consented\n",
            )
            .unwrap();
            let reopened = WorkspaceService::open(f.temp.path().join("app-data")).unwrap();
            let durable = reopened.setup_job_get(f.request(&job)).unwrap();
            assert_eq!(durable.graph, paused.graph);
            assert_eq!(durable.coverage.processed, paused.coverage.processed);
            let result = f.coordinator.control(SetupJobControlRequest {
                root_id: f.root.root_id.clone(),
                job_id: job.job_id.clone(),
                expected_revision: durable.revision,
                action: SetupJobAction::Resume,
            });
            if let Err(error) = result {
                assert!(matches!(
                    error.code.as_str(),
                    "stale_scope" | "conflict" | "needs_consent"
                ));
            }
            let blocked = f.wait(&job, |s| s.status == SetupJobStatus::NeedsConsent);
            assert_eq!(
                f.transmissions().len(),
                2,
                "changed bytes cannot launch another child turn"
            );
            assert_eq!(blocked.coverage.processed.len(), 2);
            assert_eq!(
                blocked.coverage.selected, 6,
                "new sources never enter old manifest"
            );
        },
    );
}
