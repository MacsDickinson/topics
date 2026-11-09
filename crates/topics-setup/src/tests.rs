//! Deterministic publication races through the real Runtime, host and core fence.
#![cfg(target_os = "macos")]
use super::*;
use std::{fs, time::Instant};
use topics_runtime::RuntimeConfig;

fn isolated(name: &str, test: impl FnOnce()) {
    if std::env::var("TOPICS_SETUP_RACE_CHILD").as_deref() == Ok(name) {
        test();
        return;
    }
    let home = tempfile::tempdir().unwrap();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name, "--nocapture"])
        .env("TOPICS_SETUP_RACE_CHILD", name)
        .env("CODEX_HOME", home.path())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(50);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "race child failed");
            return;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("bounded race child timeout");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn publication_race(switch_root: bool) {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().unwrap();
    let notes = temp.path().join("notes");
    fs::create_dir(&notes).unwrap();
    fs::write(
        notes.join("source.md"),
        "# Synthetic Alex A42\n\nAlex leads Ledger.\n",
    )
    .unwrap();
    let mut service = WorkspaceService::open(temp.path().join("app-data")).unwrap();
    let root = service
        .root_set(RootSetRequest {
            path: notes.to_str().unwrap().into(),
            create: false,
        })
        .unwrap()
        .root;
    let manifest = service
        .setup_manifest_preview(SetupManifestPreviewRequest {
            root_id: root.root_id.clone(),
            spec: ScopeSpec {
                files: vec!["source.md".into()],
                folders: vec![],
                date_from: None,
                date_to: None,
            },
            roles: vec![],
            limits: SetupLimits {
                max_files_per_batch: 1,
                max_bytes_per_batch: 65536,
                max_batches: 1,
                max_candidates_per_turn: 64,
                max_turns_per_batch: 1,
            },
            runtime: "codex-cli 0.166.0".into(),
        })
        .unwrap();
    let job = service
        .setup_job_create(SetupJobCreateRequest {
            root_id: root.root_id.clone(),
            manifest_id: manifest.manifest_id,
            expected_fingerprint: manifest.fingerprint,
        })
        .unwrap();
    let script = temp.path().join("scripted-codex");
    let source = include_str!("../tests/support/scripted_child.py")
        .replace(
            "@SETTINGS@",
            &serde_json::to_string(&temp.path().join("mode")).unwrap(),
        )
        .replace(
            "@LOG@",
            &serde_json::to_string(&temp.path().join("log.jsonl")).unwrap(),
        )
        .replace(
            "@FLAGS@",
            &serde_json::to_string(topics_runtime::protocol::DISABLED).unwrap(),
        );
    fs::write(&script, source).unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(temp.path().join("mode"), "graph").unwrap();
    let runtime = Arc::new(Runtime::new(RuntimeConfig {
        codex_path: script,
        proxy_path: "/usr/bin/false".into(),
        app_data: temp.path().join("app-data"),
    }));
    let service = Arc::new(Mutex::new(service));
    let coordinator = SetupCoordinator::new(service.clone(), runtime, Arc::new(Mutex::new(())));
    let (prepared_tx, prepared_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let release_rx = Mutex::new(release_rx);
    *coordinator.inner.before_publish.lock().unwrap() = Some(Arc::new(move || {
        prepared_tx.send(()).unwrap();
        release_rx
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(10))
            .unwrap();
    }));
    let (cancelled_tx, cancelled_rx) = mpsc::sync_channel(1);
    *coordinator.inner.after_cancel.lock().unwrap() = Some(Arc::new(move || {
        cancelled_tx.send(()).unwrap();
    }));
    coordinator
        .start(SetupJobRequest {
            root_id: root.root_id.clone(),
            job_id: job.job_id.clone(),
        })
        .unwrap();
    prepared_rx
        .recv_timeout(Duration::from_secs(20))
        .expect("real core publication must prepare before permit");
    // This snapshot was emitted before preparation, so reading it needs no
    // service/store lock while the real publisher deliberately holds that lock.
    let snapshot = coordinator.subscribe().borrow().as_ref().unwrap().clone();
    let controller = coordinator.clone();
    let next_notes = temp.path().join("next-notes");
    fs::create_dir(&next_notes).unwrap();
    let (result_tx, result_rx) = mpsc::sync_channel(1);
    let thread = std::thread::spawn(move || {
        let result = if switch_root {
            let _gate = controller.inner.lifecycle.lock().unwrap();
            controller.cancel_active().unwrap();
            let paused = controller
                .get(SetupJobRequest {
                    root_id: snapshot.root_id,
                    job_id: snapshot.job_id,
                })
                .unwrap();
            controller
                .inner
                .with_service(|service| {
                    service.root_set(RootSetRequest {
                        path: next_notes.to_str().unwrap().into(),
                        create: false,
                    })
                })
                .unwrap();
            paused
        } else {
            controller
                .control(SetupJobControlRequest {
                    root_id: snapshot.root_id,
                    job_id: snapshot.job_id,
                    expected_revision: snapshot.revision,
                    action: SetupJobAction::Pause,
                })
                .unwrap()
        };
        result_tx.send(result).unwrap();
    });
    cancelled_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("Runtime.cancel must finish without the service/store lock");
    release_tx.send(()).unwrap();
    let paused = result_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    thread.join().unwrap();
    assert!(
        paused.graph.items.is_empty(),
        "revoked preparation must not append to draft"
    );
    assert!(
        paused.coverage.processed.is_empty(),
        "revoked preparation must not checkpoint success"
    );
    assert!(
        paused
            .batches
            .iter()
            .all(|b| b.status != SetupBatchStatus::Completed)
    );
    assert_eq!(paused.status, SetupJobStatus::Paused);
    if switch_root {
        assert!(
            coordinator.subscribe().borrow().is_none(),
            "old run context must be cleared on root switch"
        );
    } else {
        let durable = WorkspaceService::open(temp.path().join("app-data"))
            .unwrap()
            .setup_job_get(SetupJobRequest {
                root_id: root.root_id,
                job_id: job.job_id,
            })
            .unwrap();
        assert_eq!(durable.graph, paused.graph);
        assert_eq!(durable.coverage.processed, paused.coverage.processed);
    }
}
#[test]
fn pause_denies_real_prepared_publication_without_waiting_for_store_lock() {
    isolated(
        "tests::pause_denies_real_prepared_publication_without_waiting_for_store_lock",
        || publication_race(false),
    );
}
#[test]
fn root_switch_denies_real_prepared_publication_and_clears_context() {
    isolated(
        "tests::root_switch_denies_real_prepared_publication_and_clears_context",
        || publication_race(true),
    );
}
