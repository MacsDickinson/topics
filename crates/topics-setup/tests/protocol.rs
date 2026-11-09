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
#[path = "support/render.rs"]
mod renderer;

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
    lifecycle: Arc<Mutex<()>>,
}
impl Fixture {
    fn new(documents: usize, mode: &str) -> Self {
        let temp = tempfile::tempdir().unwrap();
        renderer::render(temp.path(), mode).unwrap();
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
        let runtime = Arc::new(Runtime::new(RuntimeConfig {
            codex_path: script,
            proxy_path: PathBuf::from("/usr/bin/false"), // Direct-mode compatibility field; unused.
            app_data: temp.path().join("app-data"),
        }));
        let service = Arc::new(Mutex::new(service));
        let lifecycle = Arc::new(Mutex::new(()));
        let coordinator = SetupCoordinator::new(service.clone(), runtime, lifecycle.clone());
        Self {
            temp,
            service,
            coordinator,
            root,
            files,
            lifecycle,
        }
    }
    fn authorize(&self, per_batch: u32) -> SetupJob {
        self.authorize_attempts(per_batch, 1)
    }
    fn authorize_attempts(&self, per_batch: u32, attempts: u32) -> SetupJob {
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
                    max_turns_per_batch: attempts,
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
        let snapshot = self.wait(job, |s| {
            matches!(
                s.status,
                SetupJobStatus::Finished | SetupJobStatus::FinishedWithGaps
            )
        });
        if !snapshot.coverage.complete {
            eprintln!(
                "Synthetic coverage: {:?}; batch outcomes: {:?}",
                snapshot.coverage,
                snapshot
                    .batches
                    .iter()
                    .map(|batch| (&batch.status, &batch.error, &batch.completed_source_ids))
                    .collect::<Vec<_>>()
            );
        }
        snapshot
    }
    fn transmissions(&self) -> Vec<Value> {
        fs::read_to_string(self.temp.path().join("transmissions.jsonl"))
            .unwrap_or_default()
            .split_inclusive('\n')
            .filter(|line| line.ends_with('\n'))
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
            let job = f.authorize_attempts(2, 4);
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
            let job = f.authorize_attempts(2, 4);
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
            let snapshots = f.coordinator.subscribe();
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
                snapshots.borrow().as_ref().unwrap().status,
                SetupJobStatus::NeedsConsent,
                "a persisted consent conflict must reach async subscribers even when control returns Err"
            );
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

#[test]
fn private_sources_and_unrelated_topic_inventory_never_reach_the_child() {
    isolated(
        "private_sources_and_unrelated_topic_inventory_never_reach_the_child",
        || {
            let mut f = Fixture::new(4, "graph");
            let private = "meetings/PRIVATE_PATH_SENTINEL.md";
            fs::write(
                PathBuf::from(&f.root.path).join(private),
                "# PRIVATE_TITLE_SENTINEL\n\nPRIVATE_BYTES_SENTINEL\n",
            )
            .unwrap();
            f.files.push(private.into());
            {
                let service = f.service.lock().unwrap();
                let preview = service
                    .scope_preview(ScopePreviewRequest {
                        root_id: f.root.root_id.clone(),
                        spec: ScopeSpec {
                            files: vec![private.into()],
                            folders: vec![],
                            date_from: None,
                            date_to: None,
                        },
                    })
                    .unwrap();
                service
                    .sensitivity_set(SensitivitySetRequest {
                        root_id: f.root.root_id.clone(),
                        target: Endpoint::Source {
                            source_id: preview.sources[0].source_id.clone(),
                            locator: Locator {
                                start_byte: 0,
                                end_byte: 1,
                                heading: None,
                            },
                        },
                        sensitivity: Sensitivity::LocalOnly,
                    })
                    .unwrap();
                for (id, name, sensitivity) in [
                    (
                        "20000000-0000-4000-8000-000000000001",
                        "PRIVATE_TOPIC_SENTINEL",
                        Sensitivity::LocalOnly,
                    ),
                    (
                        "20000000-0000-4000-8000-000000000002",
                        "UNRELATED_TOPIC_SENTINEL",
                        Sensitivity::CloudAllowed,
                    ),
                ] {
                    let candidate = Candidate::Topic {
                        topic: Topic {
                            id: id.into(),
                            topic_kind: TopicKind::Thread,
                            name: name.into(),
                            aliases: vec![],
                            sensitivity,
                        },
                    };
                    let proposal = service
                        .manual_propose(ManualProposeRequest {
                            root_id: f.root.root_id.clone(),
                            candidates: vec![candidate.clone()],
                        })
                        .unwrap()
                        .proposal;
                    service
                        .proposal_decide(ProposalDecideRequest {
                            root_id: f.root.root_id.clone(),
                            proposal_id: proposal.id,
                            decision: Decision::Approve {
                                candidates: vec![candidate],
                            },
                            expected_fingerprint: proposal.fingerprint,
                        })
                        .unwrap();
                }
            }
            let job = f.authorize(2);
            f.coordinator.start(f.request(&job)).unwrap();
            let done = f.done(&job);
            assert_eq!(done.coverage.processed.len(), 4);
            assert!(done.coverage.excluded.iter().any(|e| e.path == private));
            let transmitted = serde_json::to_string(&f.transmissions()).unwrap();
            for sentinel in [
                "PRIVATE_PATH_SENTINEL",
                "PRIVATE_TITLE_SENTINEL",
                "PRIVATE_BYTES_SENTINEL",
                "PRIVATE_TOPIC_SENTINEL",
                "UNRELATED_TOPIC_SENTINEL",
            ] {
                assert!(
                    !transmitted.contains(sentinel),
                    "model-visible leak: {sentinel}"
                );
            }
        },
    );
}

#[test]
fn ambiguity_stays_an_issue_and_model_success_has_no_approval_surface() {
    isolated(
        "ambiguity_stays_an_issue_and_model_success_has_no_approval_surface",
        || {
            let f = Fixture::new(4, "ambiguous");
            let job = f.authorize(2);
            f.coordinator.start(f.request(&job)).unwrap();
            let done = f.done(&job);
            assert!(done.coverage.complete);
            assert_eq!(done.graph.items.iter().filter(|i| matches!(&i.candidate, Candidate::Topic { topic } if topic.name == "Joe")).count(), 2);
            assert!(
                done.graph
                    .issues
                    .iter()
                    .any(|i| i.issue_kind == SetupIssueKind::Identity)
            );
            assert!(
                !done
                    .graph
                    .items
                    .iter()
                    .any(|i| matches!(i.candidate, Candidate::Connection { .. }))
            );
            assert!(
                done.graph
                    .items
                    .iter()
                    .all(|i| i.disposition != DraftDisposition::Saved)
            );
            assert!(
                f.service
                    .lock()
                    .unwrap()
                    .topics_list(RootRequest {
                        root_id: f.root.root_id.clone()
                    })
                    .unwrap()
                    .topics
                    .is_empty()
            );
        },
    );
}

#[test]
fn unchanged_pause_resume_preserves_checkpoint_ids_and_does_not_resend_completed_sources() {
    isolated(
        "unchanged_pause_resume_preserves_checkpoint_ids_and_does_not_resend_completed_sources",
        || {
            let f = Fixture::new(6, "hold_second");
            let job = f.authorize_attempts(2, 4);
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
            let ids: Vec<_> = paused
                .graph
                .items
                .iter()
                .map(|i| i.item_id.clone())
                .collect();
            let completed_paths = paused.coverage.processed.clone();
            fs::write(f.temp.path().join("mode"), "graph").unwrap();
            f.coordinator
                .control(SetupJobControlRequest {
                    root_id: f.root.root_id.clone(),
                    job_id: job.job_id.clone(),
                    expected_revision: paused.revision,
                    action: SetupJobAction::Resume,
                })
                .unwrap();
            let done = f.done(&job);
            assert!(done.coverage.complete);
            assert_eq!(
                done.graph
                    .items
                    .iter()
                    .map(|i| i.item_id.clone())
                    .collect::<Vec<_>>(),
                ids
            );
            let sent = f.transmissions();
            assert_eq!(
                sent.len(),
                4,
                "one cancelled attempt plus three completed batches"
            );
            for path in completed_paths {
                let copies = sent
                    .iter()
                    .flat_map(|run| run["context"]["sources"].as_array().unwrap())
                    .filter(|source| source["summary"]["path"] == path)
                    .count();
                assert_eq!(copies, 1, "completed bytes retransmitted after resume");
            }
        },
    );
}

#[test]
fn root_switch_clears_old_context_and_rejects_old_job_launches() {
    isolated(
        "root_switch_clears_old_context_and_rejects_old_job_launches",
        || {
            let f = Fixture::new(6, "hold_second");
            let job = f.authorize(2);
            let snapshots = f.coordinator.subscribe();
            f.coordinator.start(f.request(&job)).unwrap();
            let deadline = Instant::now() + Duration::from_secs(30);
            while f.transmissions().len() < 2 {
                assert!(Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(10));
            }
            let next = f.temp.path().join("different-notes");
            fs::create_dir(&next).unwrap();
            let old_capability;
            {
                let _gate = f.lifecycle.lock().unwrap();
                f.coordinator.cancel_active().unwrap();
                let paused = f.coordinator.get(f.request(&job)).unwrap();
                assert_eq!(paused.coverage.processed.len(), 2);
                old_capability = paused
                    .batches
                    .iter()
                    .find(|b| b.status != SetupBatchStatus::Completed && b.run_id.is_some())
                    .map(|b| (b.scope_id.clone().unwrap(), b.run_id.clone().unwrap()));
                f.service
                    .lock()
                    .unwrap()
                    .root_set(RootSetRequest {
                        path: next.to_str().unwrap().into(),
                        create: false,
                    })
                    .unwrap();
            }
            assert!(
                snapshots.borrow().is_none(),
                "root switch must clear old model progress context"
            );
            assert!(f.coordinator.start(f.request(&job)).is_err());
            if let Some((scope_id, run_id)) = old_capability {
                assert!(
                    f.service
                        .lock()
                        .unwrap()
                        .scope_read(ScopeReadRequest {
                            root_id: f.root.root_id.clone(),
                            scope_id,
                            run_id
                        })
                        .is_err()
                );
            }
            assert_eq!(f.transmissions().len(), 2);
        },
    );
}

#[test]
fn restart_and_sqlite_rebuild_preserve_draft_edits_and_completed_batch_ids() {
    isolated(
        "restart_and_sqlite_rebuild_preserve_draft_edits_and_completed_batch_ids",
        || {
            let f = Fixture::new(6, "hold_second");
            let job = f.authorize_attempts(2, 4);
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
            let item = paused
                .graph
                .items
                .iter()
                .find(|i| matches!(i.candidate, Candidate::Topic { .. }))
                .unwrap();
            let mut candidate = item.candidate.clone();
            if let Candidate::Topic { topic } = &mut candidate {
                topic.aliases.push("Human draft alias".into());
            }
            let edited = f
                .service
                .lock()
                .unwrap()
                .setup_draft_edit(SetupDraftEditRequest {
                    root_id: f.root.root_id.clone(),
                    job_id: job.job_id.clone(),
                    expected_revision: paused.revision,
                    item_id: item.item_id.clone(),
                    candidate,
                })
                .unwrap();
            let app_data = f.temp.path().join("app-data");
            for path in [
                "index.sqlite3",
                "index.sqlite3-wal",
                "index.sqlite3-shm",
                "index.sqlite3-journal",
            ] {
                match fs::remove_file(app_data.join(path)) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => panic!("synthetic SQLite removal failed: {error}"),
                }
            }
            let service = Arc::new(Mutex::new(WorkspaceService::open(&app_data).unwrap()));
            let durable = service
                .lock()
                .unwrap()
                .setup_job_get(f.request(&job))
                .unwrap();
            assert_eq!(durable.graph, edited.graph);
            assert_eq!(durable.coverage.processed, edited.coverage.processed);
            assert_eq!(
                durable
                    .batches
                    .iter()
                    .filter(|b| b.status == SetupBatchStatus::Completed)
                    .map(|b| b.batch_id.clone())
                    .collect::<Vec<_>>(),
                edited
                    .batches
                    .iter()
                    .filter(|b| b.status == SetupBatchStatus::Completed)
                    .map(|b| b.batch_id.clone())
                    .collect::<Vec<_>>()
            );
            let runtime = Arc::new(Runtime::new(RuntimeConfig {
                codex_path: f.temp.path().join("scripted-codex"),
                proxy_path: PathBuf::from("/usr/bin/false"),
                app_data,
            }));
            let resumed = SetupCoordinator::new(service, runtime, Arc::new(Mutex::new(())));
            fs::write(f.temp.path().join("mode"), "graph").unwrap();
            resumed
                .control(SetupJobControlRequest {
                    root_id: f.root.root_id.clone(),
                    job_id: job.job_id.clone(),
                    expected_revision: durable.revision,
                    action: SetupJobAction::Resume,
                })
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(40);
            let complete = loop {
                let snapshot = resumed.get(f.request(&job)).unwrap();
                if snapshot.status == SetupJobStatus::Finished {
                    break snapshot;
                }
                assert!(
                    Instant::now() < deadline,
                    "restart resume did not finish: {:?}",
                    snapshot.status
                );
                std::thread::sleep(Duration::from_millis(10));
            };
            assert_eq!(complete.coverage.processed.len(), 6);
            assert!(complete.graph.items.iter().any(|i| matches!(&i.candidate, Candidate::Topic { topic } if topic.aliases.iter().any(|a| a == "Human draft alias"))));
        },
    );
}

#[test]
fn multiple_tool_calls_validate_the_runtime_combined_buffer_once() {
    isolated(
        "multiple_tool_calls_validate_the_runtime_combined_buffer_once",
        || {
            let f = Fixture::new(2, "split_calls");
            let job = f.authorize(2);
            f.coordinator.start(f.request(&job)).unwrap();
            let done = f.done(&job);
            assert!(done.coverage.complete);
            assert_eq!(
                done.graph.items.len(),
                3,
                "host must validate combined buffer without appending prior tool calls again"
            );
            assert_eq!(done.batches[0].proposal_ids.len(), 1);
        },
    );
}

#[test]
fn reviewed_retry_budget_keeps_each_attempt_to_one_runtime_turn() {
    isolated(
        "reviewed_retry_budget_keeps_each_attempt_to_one_runtime_turn",
        || {
            let f = Fixture::new(2, "graph");
            let job = f.authorize_attempts(2, 4);
            f.coordinator
                .start(f.request(&job))
                .expect("reviewed retry budget permits one turn per attempt");
            let done = f.done(&job);
            assert!(done.coverage.complete);
            assert_eq!(
                done.coverage.model_calls, 1,
                "a retry budget does not cause automatic extra turns"
            );
            assert_eq!(f.transmissions().len(), 1);
        },
    );
}

#[test]
fn next_turn_receives_only_eligible_completed_draft_context_from_the_same_manifest() {
    isolated(
        "next_turn_receives_only_eligible_completed_draft_context_from_the_same_manifest",
        || {
            let f = Fixture::new(4, "graph");
            let job = f.authorize(2);
            f.coordinator.start(f.request(&job)).unwrap();
            let done = f.done(&job);
            assert!(done.coverage.complete);
            let sent = f.transmissions();
            assert_eq!(sent.len(), 2);
            assert!(
                sent[0]["context"]["draft_context"]["topics"]
                    .as_array()
                    .unwrap()
                    .is_empty()
            );
            let context = &sent[1]["context"]["draft_context"];
            assert_eq!(
                context["topics"].as_array().unwrap().len(),
                2,
                "later turn needs scoped canonical IDs from the completed draft, not independent topic lists"
            );
            assert_eq!(context["connections"].as_array().unwrap().len(), 1);
            let reviewed: std::collections::BTreeSet<_> = done
                .manifest
                .batches
                .iter()
                .flat_map(|b| b.sources.iter().map(|s| s.source.path.as_str()))
                .collect();
            for source in context["connections"][0]["sources"].as_array().unwrap() {
                assert!(
                    reviewed.contains(source["path"].as_str().unwrap()),
                    "completed draft context cannot widen the reviewed manifest"
                );
            }
        },
    );
}

#[test]
fn exhausted_reviewed_attempt_budget_returns_an_actionable_resume_error() {
    isolated(
        "exhausted_reviewed_attempt_budget_returns_an_actionable_resume_error",
        || {
            let f = Fixture::new(4, "hold_second");
            let job = f.authorize(2); // Explicitly reviewed one attempt per batch.
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
            let failure = f
                .coordinator
                .control(SetupJobControlRequest {
                    root_id: f.root.root_id.clone(),
                    job_id: job.job_id.clone(),
                    expected_revision: paused.revision,
                    action: SetupJobAction::Resume,
                })
                .expect_err(
                    "exhausted budget must not silently leave a Ready job with unlaunchable work",
                );
            assert_eq!(failure.code, "conflict");
            assert!(failure.message.contains("budget"));
            let durable = f.coordinator.get(f.request(&job)).unwrap();
            assert_eq!(durable.status, SetupJobStatus::Paused);
            assert_eq!(durable.coverage.processed, paused.coverage.processed);
            assert_eq!(f.transmissions().len(), 2);
        },
    );
}

#[test]
fn scoped_accepted_correction_is_available_without_human_statement_text_and_cannot_be_revived() {
    isolated(
        "scoped_accepted_correction_is_available_without_human_statement_text_and_cannot_be_revived",
        || {
            let f = Fixture::new(2, "pronouns");
            for path in &f.files {
                fs::write(
                    PathBuf::from(&f.root.path).join(path),
                    "# Synthetic Alex A42\n\nAlex A42 uses he/him.\n",
                )
                .unwrap();
            }
            let person_id = "10000000-0000-4000-8000-000000000001";
            {
                let service = f.service.lock().unwrap();
                let preview = service
                    .scope_preview(ScopePreviewRequest {
                        root_id: f.root.root_id.clone(),
                        spec: ScopeSpec {
                            files: vec![f.files[0].clone()],
                            folders: vec![],
                            date_from: None,
                            date_to: None,
                        },
                    })
                    .unwrap();
                let run = service
                    .analysis_start(AnalysisStartRequest {
                        root_id: f.root.root_id.clone(),
                        scope_id: preview.scope_id.clone(),
                        confirm_unknown: true,
                    })
                    .unwrap();
                let source = service
                    .scope_read(ScopeReadRequest {
                        root_id: f.root.root_id.clone(),
                        scope_id: preview.scope_id,
                        run_id: run.run_id.clone(),
                    })
                    .unwrap()
                    .sources[0]
                    .citation
                    .clone();
                service
                    .run_cancel(RunCancelRequest {
                        root_id: f.root.root_id.clone(),
                        run_id: run.run_id,
                    })
                    .unwrap();
                let claim = Claim {
                    id: "40000000-0000-4000-8000-000000000001".into(),
                    revision_id: "40000000-0000-4000-8000-000000000002".into(),
                    subject_id: person_id.into(),
                    predicate: "pronouns".into(),
                    value: "he/him".into(),
                    uncertainty: None,
                    sources: vec![Evidence::Passage { source }],
                    supersedes: vec![],
                    recorded_at: "2026-11-09T00:00:00Z".into(),
                    valid_from: None,
                    valid_to: None,
                };
                let candidates = vec![
                    Candidate::Topic {
                        topic: Topic {
                            id: person_id.into(),
                            topic_kind: TopicKind::Person,
                            name: "Synthetic Alex A42".into(),
                            aliases: vec!["employee A42".into()],
                            sensitivity: Sensitivity::CloudAllowed,
                        },
                    },
                    Candidate::Claim {
                        claim: claim.clone(),
                    },
                ];
                let proposal = service
                    .manual_propose(ManualProposeRequest {
                        root_id: f.root.root_id.clone(),
                        candidates: candidates.clone(),
                    })
                    .unwrap()
                    .proposal;
                service
                    .proposal_decide(ProposalDecideRequest {
                        root_id: f.root.root_id.clone(),
                        proposal_id: proposal.id,
                        decision: Decision::Approve { candidates },
                        expected_fingerprint: proposal.fingerprint,
                    })
                    .unwrap();
                let correction = service
                    .setup_correction_preview(SetupCorrectionPreviewRequest {
                        root_id: f.root.root_id.clone(),
                        topic_id: person_id.into(),
                        revision_ids: vec![claim.revision_id],
                        predicate: "pronouns".into(),
                        replacement: "they/them".into(),
                        statement: "Alex uses they/them. PRIVATE_UNRELATED_HUMAN_SENTINEL".into(),
                        rule_scope: RuleScope {
                            source_paths: f.files.clone(),
                            context: None,
                        },
                    })
                    .unwrap();
                let result = service
                    .setup_decide(SetupDecisionRequest {
                        root_id: f.root.root_id.clone(),
                        receipt_id: correction.receipt.receipt_id,
                        expected_fingerprint: correction.receipt.fingerprint,
                    })
                    .unwrap();
                assert_eq!(result.status, SetupDecisionStatus::Completed);
            }
            let job = f.authorize(1);
            f.coordinator.start(f.request(&job)).unwrap();
            let done = f.done(&job);
            assert_eq!(done.coverage.processed.len(), 2);
            assert!(
                done.batches
                    .iter()
                    .all(|b| b.status == SetupBatchStatus::Completed)
            );
            assert!(!done.graph.items.iter().any(|i| matches!(&i.candidate, Candidate::Claim { claim } if claim.predicate == "pronouns" && claim.value == "he/him")), "analysis must not revive the precise suppressed assertion");
            let sent = f.transmissions();
            assert!(
                sent[0]["context"]["topic_contexts"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .flat_map(|view| view["current_claims"].as_array().unwrap())
                    .any(|claim| claim["predicate"] == "pronouns" && claim["value"] == "they/them")
            );
            assert!(
                !serde_json::to_string(&sent)
                    .unwrap()
                    .contains("PRIVATE_UNRELATED_HUMAN_SENTINEL")
            );
            let view = f
                .service
                .lock()
                .unwrap()
                .setup_topic_review(TopicGetRequest {
                    root_id: f.root.root_id.clone(),
                    topic_id: person_id.into(),
                })
                .unwrap();
            assert_eq!(
                view.topic
                    .current_claims
                    .iter()
                    .find(|c| c.predicate == "pronouns")
                    .unwrap()
                    .value,
                "they/them"
            );
            assert!(
                view.reviews
                    .iter()
                    .any(|r| r.state == ReviewState::ConfirmedByYou)
            );
        },
    );
}

#[test]
fn historic_reference_and_recent_meeting_accounts_retain_dates_without_selecting_truth() {
    isolated(
        "historic_reference_and_recent_meeting_accounts_retain_dates_without_selecting_truth",
        || {
            let f = Fixture::new(3, "historic");
            for path in &f.files {
                let text = if path.starts_with("profiles/") {
                    "# 2020-01-01 Ledger profile\n\nLedger leader is Taylor.\n"
                } else {
                    "# 2026-10-01 Ledger meeting\n\nLedger leader is Alex.\n"
                };
                fs::write(PathBuf::from(&f.root.path).join(path), text).unwrap();
            }
            let job = f.authorize(1);
            f.coordinator.start(f.request(&job)).unwrap();
            let done = f.done(&job);
            assert!(done.coverage.complete);
            let claims: Vec<_> = done
                .graph
                .items
                .iter()
                .filter_map(|i| {
                    if let Candidate::Claim { claim } = &i.candidate {
                        Some(claim)
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(claims.len(), 2);
            assert!(
                claims
                    .iter()
                    .any(|c| c.value == "Taylor" && c.valid_from.as_deref() == Some("2020-01-01"))
            );
            assert!(
                claims
                    .iter()
                    .any(|c| c.value == "Alex" && c.valid_from.as_deref() == Some("2026-10-01"))
            );
            assert_eq!(claims.iter().map(|c| c.sources.len()).sum::<usize>(), 3);
            assert!(
                done.graph
                    .issues
                    .iter()
                    .any(|i| i.issue_kind == SetupIssueKind::Contradiction)
            );
            assert!(
                f.service
                    .lock()
                    .unwrap()
                    .topics_list(RootRequest {
                        root_id: f.root.root_id.clone()
                    })
                    .unwrap()
                    .topics
                    .is_empty()
            );
        },
    );
}

#[test]
fn graph_claim_fixture_stages_canonical_cited_claim_without_accepting_it() {
    isolated(
        "graph_claim_fixture_stages_canonical_cited_claim_without_accepting_it",
        || {
            let f = Fixture::new(2, "graph_claim");
            let job = f.authorize(2);
            f.coordinator.start(f.request(&job)).unwrap();
            let done = f.done(&job);
            assert!(done.coverage.complete);
            let claim = done
                .graph
                .items
                .iter()
                .find_map(|item| match &item.candidate {
                    Candidate::Claim { claim } if claim.predicate == "responsibility" => {
                        Some(claim)
                    }
                    _ => None,
                })
                .expect("graph_claim fixture must stage a canonical cited responsibility claim");
            assert_eq!(claim.subject_id, "10000000-0000-4000-8000-000000000001");
            assert_eq!(claim.value, "leads Ledger");
            assert_ne!(claim.id, claim.revision_id);
            assert_eq!(claim.sources.len(), 2);
            assert!(
                claim
                    .sources
                    .iter()
                    .all(|e| matches!(e, Evidence::Passage { .. }))
            );
            assert!(
                done.graph
                    .items
                    .iter()
                    .all(|i| i.disposition == DraftDisposition::Pending)
            );
            assert!(
                f.service
                    .lock()
                    .unwrap()
                    .topics_list(RootRequest {
                        root_id: f.root.root_id.clone()
                    })
                    .unwrap()
                    .topics
                    .is_empty()
            );
        },
    );
}

#[test]
fn renewed_manifest_consent_sends_only_outstanding_subset_and_preserves_original_checkpoints() {
    isolated(
        "renewed_manifest_consent_sends_only_outstanding_subset_and_preserves_original_checkpoints",
        || {
            let mut f = Fixture::new(4, "hold_second");
            let job = f.authorize_attempts(2, 4);
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
            let completed_paths = paused.coverage.processed.clone();
            let original = paused
                .batches
                .iter()
                .find(|batch| batch.status == SetupBatchStatus::Completed)
                .unwrap()
                .clone();
            let changed = paused.coverage.outstanding[0].clone();
            fs::write(
                PathBuf::from(&f.root.path).join(&changed),
                "# Synthetic changed account\n\nAlex A42 leads Ledger. New date context.\n",
            )
            .unwrap();
            let added = "meetings/extra.md";
            fs::write(
                PathBuf::from(&f.root.path).join(added),
                "# Synthetic new account\n\nAlex A42 leads Ledger.\n",
            )
            .unwrap();
            f.files.push(added.into());
            let reconsented = {
                let service = f.service.lock().unwrap();
                let mut limits = job.manifest.limits.clone();
                limits.max_files_per_batch = 5; // Intentionally mix completed + outstanding in one new batch.
                let manifest = service
                    .setup_manifest_preview(SetupManifestPreviewRequest {
                        root_id: f.root.root_id.clone(),
                        spec: ScopeSpec {
                            files: f.files.clone(),
                            folders: vec![],
                            date_from: None,
                            date_to: None,
                        },
                        roles: job.manifest.roles.clone(),
                        limits,
                        runtime: job.manifest.runtime.clone(),
                    })
                    .unwrap();
                service
                    .setup_job_reconsent(SetupJobReconsentRequest {
                        root_id: f.root.root_id.clone(),
                        job_id: job.job_id.clone(),
                        expected_revision: paused.revision,
                        manifest_id: manifest.manifest_id,
                        expected_fingerprint: manifest.fingerprint,
                    })
                    .expect("explicit renewed manifest consent must retain the existing draft job")
            };
            assert_eq!(reconsented.job_id, job.job_id);
            assert_eq!(reconsented.prior_consents.len(), 1);
            let retained = reconsented.prior_consents[0]
                .batches
                .iter()
                .find(|batch| batch.batch_id == original.batch_id)
                .unwrap();
            assert_eq!(retained.run_id, original.run_id);
            assert_eq!(retained.scope_id, original.scope_id);
            assert_eq!(retained.proposal_ids, original.proposal_ids);
            assert_eq!(reconsented.manifest.batches.len(), 1);
            assert_eq!(reconsented.batches[0].completed_source_ids.len(), 2);
            fs::write(f.temp.path().join("mode"), "graph").unwrap();
            f.coordinator
                .control(SetupJobControlRequest {
                    root_id: f.root.root_id.clone(),
                    job_id: job.job_id.clone(),
                    expected_revision: reconsented.revision,
                    action: SetupJobAction::Resume,
                })
                .unwrap();
            let done = f.done(&job);
            assert_eq!(done.coverage.processed.len(), 5);
            assert_eq!(done.coverage.transmission_authorizations, 2);
            assert_eq!(done.coverage.model_calls, 3);
            assert_eq!(done.graph.items.len(), 3);
            let sent = f.transmissions();
            assert_eq!(sent.len(), 3);
            assert_eq!(
                sent[2]["context"]["sources"].as_array().unwrap().len(),
                3,
                "fresh capability must contain only outstanding sources, not the five-file manifest batch"
            );
            for path in completed_paths {
                let count = sent
                    .iter()
                    .flat_map(|turn| turn["context"]["sources"].as_array().unwrap())
                    .filter(|source| source["summary"]["path"] == path)
                    .count();
                assert_eq!(count, 1, "completed source retransmitted after reconsent");
            }
        },
    );
}

#[test]
fn successful_turn_without_scoped_read_cannot_claim_source_coverage() {
    isolated(
        "successful_turn_without_scoped_read_cannot_claim_source_coverage",
        || {
            let f = Fixture::new(2, "no_read");
            let job = f.authorize(2);
            f.coordinator.start(f.request(&job)).unwrap();
            let done = f.done(&job);
            assert_eq!(
                done.coverage.processed.len(),
                0,
                "terminal success cannot report analysed files the model never requested"
            );
            assert_eq!(done.coverage.failed.len(), 2);
            assert!(!done.coverage.complete);
            assert!(f.transmissions().is_empty());
            assert!(done.graph.items.is_empty());
        },
    );
}

fn assert_failed_diagnostic(mode: &str, phase: &str, reason: &str) {
    let f = Fixture::new(1, mode);
    let job = f.authorize(1);
    let path = f.temp.path().join("notes").join(&f.files[0]);
    let original = fs::read(&path).unwrap();
    f.coordinator.start(f.request(&job)).unwrap();
    let done = f.done(&job);
    assert_eq!(done.status, SetupJobStatus::FinishedWithGaps);
    assert!(!done.coverage.complete);
    assert_eq!(done.coverage.processed.len(), 0);
    assert_eq!(done.coverage.failed.len(), 1);
    assert_eq!(done.coverage.model_calls, 1);
    assert_eq!(done.batches[0].attempt, 1);
    assert!(done.graph.items.is_empty());
    assert!(done.batches[0].proposal_ids.is_empty());
    assert!(done.batches[0].completed_source_ids.is_empty());
    assert_eq!(fs::read(path).unwrap(), original);
    assert_eq!(f.transmissions().len(), usize::from(mode == "fail_first"));
    let model_context = serde_json::to_string(&f.transmissions()).unwrap();
    for diagnostic in [
        "model_terminal_failed",
        "scope_not_read",
        "PRIVATE_ERROR_SENTINEL",
    ] {
        assert!(!model_context.contains(diagnostic));
    }
    let failure = done.batches[0].error.as_ref().unwrap();
    assert_eq!(failure.code, "runtime_unavailable");
    assert!(
        !serde_json::to_string(failure)
            .unwrap()
            .contains("PRIVATE_ERROR_SENTINEL")
    );
    assert_eq!(
        failure.details,
        Some(serde_json::json!({"phase":phase,"reason":reason}))
    );
    // Rereading the durable snapshot cannot publish the failed turn's buffer.
    assert_eq!(f.coordinator.get(f.request(&job)).unwrap(), done);
}

#[test]
fn failed_terminal_retains_safe_reason_and_discards_validated_candidates() {
    isolated(
        "failed_terminal_retains_safe_reason_and_discards_validated_candidates",
        || assert_failed_diagnostic("fail_first", "turn", "model_terminal_failed"),
    );
}

#[test]
fn unread_scope_retains_distinct_safe_reason_without_publication() {
    isolated(
        "unread_scope_retains_distinct_safe_reason_without_publication",
        || assert_failed_diagnostic("no_read", "publication", "scope_not_read"),
    );
}

#[test]
fn optional_prior_context_cannot_crowd_out_deliverable_selected_sources() {
    isolated(
        "optional_prior_context_cannot_crowd_out_deliverable_selected_sources",
        || {
            let f = Fixture::new(12, "graph");
            let note = format!(
                "# Synthetic Alex A42\n\nAlex A42 leads Ledger.\n{}",
                "x".repeat(24 * 1024)
            );
            for path in &f.files {
                fs::write(PathBuf::from(&f.root.path).join(path), &note).unwrap();
            }
            let job = f.authorize(1);
            f.coordinator.start(f.request(&job)).unwrap();
            let done = f.done(&job);
            let sent = f.transmissions();
            assert!(!sent.is_empty());
            let mandatory = serde_json::json!({"sources": sent[0]["context"]["sources"], "topic_contexts": [], "draft_context": {"topics":[],"claims":[],"connections":[]}, "candidate_schema": sent[0]["context"]["candidate_schema"]});
            assert!(
                mandatory.to_string().len() < topics_runtime::protocol::MAX_FRAME / 2,
                "the exact selected source is independently deliverable"
            );
            assert_eq!(
                done.coverage.processed.len(),
                12,
                "optional completed context must not cause otherwise deliverable selected batches to fail"
            );
            assert!(done.coverage.failed.is_empty());
            assert!(done.coverage.complete);
            assert_eq!(done.coverage.model_calls, 12);
            assert_eq!(sent.len(), 12);
            assert!(
                sent.iter().any(|turn| turn["context"]["context_notice"]
                    .as_str()
                    .is_some_and(|notice| notice.contains("partial"))),
                "omitting optional prior context requires an explicit partial-context notice"
            );
            for turn in &sent {
                let context = &turn["context"];
                assert!(context.to_string().len() <= topics_runtime::protocol::MAX_FRAME / 2);
                assert_eq!(context["sources"].as_array().unwrap().len(), 1);
                assert_eq!(
                    context["sources"][0]["content"], note,
                    "mandatory source bytes must be exact, never truncated"
                );
                let topics: std::collections::BTreeSet<_> = context["draft_context"]["topics"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|topic| topic["id"].as_str().unwrap())
                    .collect();
                for connection in context["draft_context"]["connections"].as_array().unwrap() {
                    for endpoint in [&connection["from"], &connection["to"]] {
                        if endpoint["kind"] == "topic" {
                            assert!(
                                topics.contains(endpoint["id"].as_str().unwrap()),
                                "packed optional graph must keep endpoint dependencies"
                            );
                        }
                    }
                }
            }
            assert_eq!(done.graph.items.len(), 3);
        },
    );
}
