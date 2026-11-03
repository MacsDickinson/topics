//! Adversarial service tests use synthetic roots and explicit absolute app data only.
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};
use topics_core::*;
use topics_store::WorkspaceService;
fn id() -> Id {
    uuid::Uuid::new_v4().to_string()
}
fn topic(name: &str) -> Topic {
    Topic {
        id: id(),
        name: name.into(),
        topic_kind: TopicKind::Person,
        aliases: vec![],
        sensitivity: Sensitivity::CloudAllowed,
    }
}
struct Fixture {
    temp: tempfile::TempDir,
    root: PathBuf,
    root_id: Id,
    service: WorkspaceService,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("notes");
        let mut service = WorkspaceService::open(temp.path().join("app-data")).unwrap();
        let root_id = service
            .root_set(RootSetRequest {
                path: root.to_str().unwrap().into(),
                create: true,
            })
            .unwrap()
            .root
            .root_id;
        Self {
            temp,
            root,
            root_id,
            service,
        }
    }
    fn req(&self) -> RootRequest {
        RootRequest {
            root_id: self.root_id.clone(),
        }
    }
    fn write(&self, path: &str, bytes: impl AsRef<[u8]>) {
        let p = self.root.join(path);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, bytes).unwrap();
    }
    fn source(&self, path: &str) -> ScopedSource {
        let s = self
            .service
            .sources_list(self.req())
            .unwrap()
            .sources
            .into_iter()
            .find(|s| s.path == path)
            .unwrap();
        self.service
            .source_get(SourceGetRequest {
                root_id: self.root_id.clone(),
                source_id: s.source_id,
            })
            .unwrap()
    }
    fn preview(&self, files: Vec<&str>) -> ScopePreview {
        self.service
            .scope_preview(ScopePreviewRequest {
                root_id: self.root_id.clone(),
                spec: ScopeSpec {
                    files: files.into_iter().map(str::to_string).collect(),
                    folders: vec![],
                    date_from: None,
                    date_to: None,
                },
            })
            .unwrap()
    }
    fn run(&self, p: &ScopePreview, confirm: bool) -> WorkspaceResult<AnalysisStartResult> {
        self.service.analysis_start(AnalysisStartRequest {
            root_id: self.root_id.clone(),
            scope_id: p.scope_id.clone(),
            confirm_unknown: confirm,
        })
    }
    fn stage(&self, candidates: Vec<Candidate>) -> WorkspaceResult<Proposal> {
        self.service
            .manual_propose(ManualProposeRequest {
                root_id: self.root_id.clone(),
                candidates,
            })
            .map(|p| p.proposal)
    }
    fn decide(&self, p: &Proposal, decision: Decision) -> WorkspaceResult<ProposalDecideResult> {
        self.service.proposal_decide(ProposalDecideRequest {
            root_id: self.root_id.clone(),
            proposal_id: p.id.clone(),
            expected_fingerprint: p.fingerprint.clone(),
            decision,
        })
    }
    fn accept(&self, candidates: Vec<Candidate>) -> ProposalDecideResult {
        let p = self.stage(candidates.clone()).unwrap();
        self.decide(&p, Decision::Approve { candidates }).unwrap()
    }
    fn set_source(&self, s: &SourceSummary, label: Sensitivity) -> Operation {
        self.service
            .sensitivity_set(SensitivitySetRequest {
                root_id: self.root_id.clone(),
                target: Endpoint::Source {
                    source_id: s.source_id.clone(),
                    locator: Locator {
                        start_byte: 0,
                        end_byte: 0,
                        heading: None,
                    },
                },
                sensitivity: label,
            })
            .unwrap()
            .operation
    }
    fn claim(&self, t: &Topic, s: &SourceRef, value: &str) -> Claim {
        Claim {
            id: id(),
            revision_id: id(),
            subject_id: t.id.clone(),
            predicate: "role".into(),
            value: value.into(),
            uncertainty: Some("Synthetic unverified assertion".into()),
            sources: vec![Evidence::Passage { source: s.clone() }],
            supersedes: vec![],
            recorded_at: "2026-11-03T00:00:00Z".into(),
            valid_from: None,
            valid_to: None,
        }
    }
}
#[test]
fn fresh_root_manual_proposals_survive_restart_and_decisions_are_idempotent() {
    let f = Fixture::new();
    assert!(f.service.sources_list(f.req()).unwrap().sources.is_empty());
    let t = topic("Fresh manual topic");
    let candidates = vec![Candidate::Topic { topic: t.clone() }];
    let p = f.stage(candidates.clone()).unwrap();
    assert!(f.service.topics_list(f.req()).unwrap().topics.is_empty());
    let a = f
        .decide(
            &p,
            Decision::Approve {
                candidates: candidates.clone(),
            },
        )
        .unwrap();
    let b = f.decide(&p, Decision::Approve { candidates }).unwrap();
    assert_eq!(a.operation, b.operation);
    assert_eq!(f.decide(&p, Decision::Reject).unwrap_err().code, "conflict");
    let reopened = WorkspaceService::open(f.temp.path().join("app-data")).unwrap();
    assert_eq!(reopened.topics_list(f.req()).unwrap().topics, vec![t]);
    assert!(!f.root.join(".git").exists());
}
#[test]
fn scope_consent_is_per_run_and_source_change_or_root_reselection_expires_it() {
    let mut f = Fixture::new();
    f.write("Meetings/one.md", "# Unknown\n");
    let p = f.preview(vec!["Meetings/one.md"]);
    assert_eq!(f.run(&p, false).unwrap_err().code, "forbidden");
    assert!(f.run(&p, true).is_ok());
    assert_eq!(f.run(&p, false).unwrap_err().code, "forbidden");
    f.write("Meetings/one.md", "# Different bytes\n");
    assert_eq!(f.run(&p, true).unwrap_err().code, "stale_scope");
    let old = f.root_id.clone();
    f.root_id = f
        .service
        .root_set(RootSetRequest {
            path: f.root.to_str().unwrap().into(),
            create: false,
        })
        .unwrap()
        .root
        .root_id;
    assert_ne!(old, f.root_id);
    assert_eq!(
        f.service
            .topics_list(RootRequest { root_id: old })
            .unwrap_err()
            .code,
        "stale_scope"
    );
    assert_eq!(f.run(&p, true).unwrap_err().code, "stale_scope");
}
#[test]
fn source_sensitivity_is_durable_and_taints_manual_derived_topics() {
    let f = Fixture::new();
    f.write("Meetings/private.md", "# Synthetic private material\n");
    let s = f.source("Meetings/private.md");
    f.set_source(&s.summary, Sensitivity::LocalOnly);
    let t = topic("Derived private identity");
    let c = f.claim(&t, &s.citation, "PRIVATE ROLE");
    f.accept(vec![
        Candidate::Topic { topic: t.clone() },
        Candidate::Claim { claim: c },
    ]);
    assert_eq!(
        f.service.topics_list(f.req()).unwrap().topics[0].sensitivity,
        Sensitivity::LocalOnly
    );
    assert_eq!(
        f.service.sources_list(f.req()).unwrap().sources[0].sensitivity,
        Sensitivity::LocalOnly
    );
    assert!(
        f.service
            .source_get(SourceGetRequest {
                root_id: f.root_id.clone(),
                source_id: s.summary.source_id
            })
            .is_ok()
    );
    let p = f.preview(vec!["Meetings/private.md"]);
    assert!(p.sources.is_empty());
    assert_eq!(p.excluded[0].reason, "Local-only source");
}
#[test]
fn labelling_a_topic_local_only_invalidates_related_source_capabilities() {
    let f = Fixture::new();
    f.write("Meetings/one.md", "# Synthetic meeting\nSensitive fact\n");
    let s = f.source("Meetings/one.md");
    let t = topic("Sensitive topic");
    let c = f.claim(&t, &s.citation, "Sensitive fact");
    f.accept(vec![
        Candidate::Topic { topic: t.clone() },
        Candidate::Claim { claim: c },
    ]);
    let p = f.preview(vec!["Meetings/one.md"]);
    let run = f.run(&p, true).unwrap();
    f.service
        .sensitivity_set(SensitivitySetRequest {
            root_id: f.root_id.clone(),
            target: Endpoint::Topic { id: t.id },
            sensitivity: Sensitivity::LocalOnly,
        })
        .unwrap();
    let result = f.service.scope_read(ScopeReadRequest {
        root_id: f.root_id.clone(),
        scope_id: p.scope_id,
        run_id: run.run_id,
    });
    assert!(
        result.is_err(),
        "local topic evidence must not leak through raw scope reads"
    );
}
#[test]
fn cancelled_capability_cannot_read_or_stage_even_valid_late_output() {
    let f = Fixture::new();
    f.write("Meetings/one.md", "# Meeting\n");
    let p = f.preview(vec!["Meetings/one.md"]);
    let run = f.run(&p, true).unwrap();
    f.service
        .run_cancel(RunCancelRequest {
            root_id: f.root_id.clone(),
            run_id: run.run_id.clone(),
        })
        .unwrap();
    assert_eq!(
        f.service
            .scope_read(ScopeReadRequest {
                root_id: f.root_id.clone(),
                scope_id: p.scope_id.clone(),
                run_id: run.run_id.clone()
            })
            .unwrap_err()
            .code,
        "forbidden"
    );
    assert_eq!(
        f.service
            .stage_proposal(StageProposalRequest {
                root_id: f.root_id.clone(),
                scope_id: Some(p.scope_id),
                origin: Origin::Analysis { run_id: run.run_id },
                candidates: vec![Candidate::Topic {
                    topic: topic("Late")
                }]
            })
            .unwrap_err()
            .code,
        "forbidden"
    );
    assert!(
        f.service
            .proposals_list(f.req())
            .unwrap()
            .proposals
            .is_empty()
    );
}
#[test]
fn source_locators_use_utf8_bytes_and_hashes_not_character_offsets_or_model_excerpt() {
    let f = Fixture::new();
    f.write("Meetings/one.md", "🦀 Alex\n");
    let s = f.source("Meetings/one.md");
    let t = topic("Alex");
    let mut reference = s.citation;
    reference.locator.start_byte = 4;
    reference.locator.end_byte = 9;
    reference.excerpt = " Alex".into();
    reference.passage_hash = format!("{:x}", Sha256::digest(reference.excerpt.as_bytes()));
    let c = f.claim(&t, &reference, "Leader");
    assert!(
        f.stage(vec![
            Candidate::Topic { topic: t.clone() },
            Candidate::Claim { claim: c }
        ])
        .is_ok()
    );
    reference.locator.start_byte = 1;
    let c = f.claim(&t, &reference, "Invalid byte boundary");
    assert_eq!(
        f.stage(vec![
            Candidate::Topic { topic: t.clone() },
            Candidate::Claim { claim: c }
        ])
        .unwrap_err()
        .code,
        "stale_evidence"
    );
    reference.locator.start_byte = 4;
    reference.excerpt = "Forged model text".into();
    let c = f.claim(&t, &reference, "Forged");
    assert_eq!(
        f.stage(vec![
            Candidate::Topic { topic: t },
            Candidate::Claim { claim: c }
        ])
        .unwrap_err()
        .code,
        "stale_evidence"
    );
}
#[test]
fn approved_subset_rejects_omissions_and_cannot_be_redecided_differently() {
    let f = Fixture::new();
    let a = Candidate::Topic { topic: topic("A") };
    let b = Candidate::Topic { topic: topic("B") };
    let p = f.stage(vec![a.clone(), b]).unwrap();
    f.decide(
        &p,
        Decision::Approve {
            candidates: vec![a.clone()],
        },
    )
    .unwrap();
    assert_eq!(f.service.topics_list(f.req()).unwrap().topics.len(), 1);
    assert!(
        f.decide(
            &p,
            Decision::Approve {
                candidates: vec![a]
            }
        )
        .is_ok()
    );
    assert_eq!(
        f.decide(
            &p,
            Decision::Approve {
                candidates: p.candidates.clone()
            }
        )
        .unwrap_err()
        .code,
        "conflict"
    );
    let mut stale = p.clone();
    stale.fingerprint = "0".repeat(64);
    assert_eq!(
        f.decide(&stale, Decision::Reject).unwrap_err().code,
        "conflict"
    );
}
#[test]
fn ids_collisions_are_rejected_and_same_names_never_silently_merge() {
    let f = Fixture::new();
    let a = topic("Joe");
    f.accept(vec![Candidate::Topic { topic: a.clone() }]);
    let mut changed = a.clone();
    changed.name = "Changed Joe".into();
    assert_eq!(
        f.stage(vec![Candidate::Topic { topic: changed }])
            .unwrap_err()
            .code,
        "conflict"
    );
    let b = topic("Joe");
    f.accept(vec![Candidate::Topic { topic: b }]);
    assert_eq!(f.service.topics_list(f.req()).unwrap().topics.len(), 2);
    let c = topic("Repeated provisional");
    assert_eq!(
        f.stage(vec![
            Candidate::Topic { topic: c.clone() },
            Candidate::Topic { topic: c }
        ])
        .unwrap_err()
        .code,
        "invalid_input"
    );
}
#[test]
fn deleting_derived_db_and_disposable_proposals_does_not_duplicate_accepted_decisions() {
    let f = Fixture::new();
    f.write(
        "Meetings/one.md",
        "# Synthetic scope\nExisting cited assertion\n",
    );
    let source = f.source("Meetings/one.md");
    let topic = topic("Durable fingerprint");
    let claim = f.claim(&topic, &source.citation, "Existing cited assertion");
    f.accept(vec![
        Candidate::Topic {
            topic: topic.clone(),
        },
        Candidate::Claim {
            claim: claim.clone(),
        },
    ]);
    let partition = fs::read_dir(f.temp.path().join("app-data/workspaces"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    fs::remove_dir_all(partition.join("proposals")).unwrap();
    fs::remove_file(f.temp.path().join("app-data/index.sqlite3")).unwrap();
    let mut repeated_topic = topic.clone();
    repeated_topic.id = id();
    let mut repeated_claim = claim.clone();
    repeated_claim.id = id();
    repeated_claim.revision_id = id();
    repeated_claim.subject_id = repeated_topic.id.clone();
    let preview = f.preview(vec!["Meetings/one.md"]);
    let run = f.run(&preview, true).unwrap();
    let proposal = f
        .service
        .stage_proposal(StageProposalRequest {
            root_id: f.root_id.clone(),
            scope_id: Some(preview.scope_id),
            origin: Origin::Analysis { run_id: run.run_id },
            candidates: vec![
                Candidate::Topic {
                    topic: repeated_topic,
                },
                Candidate::Claim {
                    claim: repeated_claim,
                },
            ],
        })
        .unwrap();
    assert_eq!(proposal.status, ProposalStatus::Accepted);
    assert!(proposal.candidates.is_empty());
    let view = f
        .service
        .topic_get(TopicGetRequest {
            root_id: f.root_id.clone(),
            topic_id: topic.id,
        })
        .unwrap();
    assert_eq!(view.current_claims.len(), 1);
    assert_eq!(view.current_claims[0], claim);
}
#[test]
fn fingerprints_preserve_accepted_endpoint_identity() {
    let f = Fixture::new();
    f.write("Meetings/one.md", "# Meeting\nAlex and Jo lead Ledger\n");
    let s = f.source("Meetings/one.md");
    let a = topic("Alex");
    let b = topic("Jo");
    let c = topic("Ledger");
    f.accept(vec![
        Candidate::Topic { topic: a.clone() },
        Candidate::Topic { topic: b.clone() },
        Candidate::Topic { topic: c.clone() },
    ]);
    let make = |from: &Topic| Candidate::Connection {
        connection: Connection {
            id: id(),
            from: Endpoint::Topic {
                id: from.id.clone(),
            },
            relation: "leads".into(),
            to: Endpoint::Topic { id: c.id.clone() },
            uncertainty: None,
            sources: vec![s.citation.clone()],
            valid_from: None,
            valid_to: None,
        },
    };
    let x = f.stage(vec![make(&a)]).unwrap();
    let y = f.stage(vec![make(&b)]).unwrap();
    assert_ne!(x.fingerprint, y.fingerprint);
}
#[test]
fn sensitivity_updates_undo_restore_previous_bytes_without_erasing_history() {
    let f = Fixture::new();
    f.write("Meetings/one.md", "# Source\n");
    let s = f.source("Meetings/one.md");
    let op = f.set_source(&s.summary, Sensitivity::LocalOnly);
    assert_eq!(
        f.service.sources_list(f.req()).unwrap().sources[0].sensitivity,
        Sensitivity::LocalOnly
    );
    f.service
        .undo(UndoRequest {
            root_id: f.root_id.clone(),
            operation_id: op.id.clone(),
        })
        .unwrap();
    assert_eq!(
        f.service.sources_list(f.req()).unwrap().sources[0].sensitivity,
        Sensitivity::Unknown
    );
    assert!(
        f.service
            .history_list(f.req())
            .unwrap()
            .operations
            .iter()
            .any(|o| o.id == op.id)
    );
}
#[test]
fn target_traversal_and_symlink_escape_are_rejected_without_external_writes() {
    let f = Fixture::new();
    let outside = f.temp.path().join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("source.md"), b"SYNTHETIC OUTSIDE").unwrap();
    let e = f
        .service
        .scope_preview(ScopePreviewRequest {
            root_id: f.root_id.clone(),
            spec: ScopeSpec {
                files: vec!["../outside/source.md".into()],
                folders: vec![],
                date_from: None,
                date_to: None,
            },
        })
        .unwrap_err();
    assert_eq!(e.code, "invalid_input");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(outside.join("source.md"), f.root.join("Meetings/escape.md"))
            .unwrap();
        let p = f.preview(vec!["Meetings/escape.md"]);
        assert!(p.sources.is_empty());
        let t = topic("Do not write outside");
        let staged = f
            .stage(vec![Candidate::Topic { topic: t.clone() }])
            .unwrap();
        std::os::unix::fs::symlink(&outside, f.root.join("topics/topics")).unwrap();
        let e = f
            .decide(
                &staged,
                Decision::Approve {
                    candidates: vec![Candidate::Topic { topic: t }],
                },
            )
            .unwrap_err();
        assert_eq!(e.code, "forbidden");
    }
    assert_eq!(fs::read_dir(&outside).unwrap().count(), 1);
    assert_eq!(
        fs::read(outside.join("source.md")).unwrap(),
        b"SYNTHETIC OUTSIDE"
    );
}
#[test]
fn existing_store_lock_blocks_workspace_calls_until_release() {
    use std::{sync::mpsc, time::Duration};
    let f = Fixture::new();
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(f.temp.path().join("app-data/store.lock"))
        .unwrap();
    lock.lock().unwrap();
    let service = WorkspaceService::open(f.temp.path().join("app-data")).unwrap();
    let request = f.req();
    let (tx, rx) = mpsc::channel();
    let thread = std::thread::spawn(move || tx.send(service.topics_list(request)).unwrap());
    assert!(rx.recv_timeout(Duration::from_millis(80)).is_err());
    drop(lock);
    assert!(rx.recv_timeout(Duration::from_secs(3)).unwrap().is_ok());
    thread.join().unwrap();
}
#[test]
fn invalid_dates_empty_selection_and_non_markdown_are_actionable() {
    let f = Fixture::new();
    f.write("Meetings/file.txt", "Synthetic text");
    let p = f.preview(vec!["Meetings/file.txt"]);
    assert!(p.sources.is_empty());
    assert!(!p.excluded.is_empty());
    let bad = f
        .service
        .scope_preview(ScopePreviewRequest {
            root_id: f.root_id.clone(),
            spec: ScopeSpec {
                files: vec!["Meetings/file.txt".into()],
                folders: vec![],
                date_from: Some("2026-02-30".into()),
                date_to: None,
            },
        })
        .unwrap_err();
    assert_eq!(bad.code, "invalid_input");
    let e = f
        .service
        .scope_preview(ScopePreviewRequest {
            root_id: f.root_id.clone(),
            spec: ScopeSpec {
                files: vec![],
                folders: vec![],
                date_from: None,
                date_to: None,
            },
        })
        .unwrap_err();
    assert_eq!(e.code, "invalid_input");
}

#[test]
fn model_cannot_reference_private_endpoints_or_deduplicate_into_private_decisions() {
    let f = Fixture::new();
    f.write("Meetings/public.md", "# Public selected meeting\n");
    f.write("Meetings/private.md", "# Private\nPRIVATE MARKER\n");
    let private = f.source("Meetings/private.md");
    let t = topic("Private person");
    let c = f.claim(&t, &private.citation, "PRIVATE CLAIM");
    let candidates = vec![
        Candidate::Topic { topic: t.clone() },
        Candidate::Claim { claim: c },
    ];
    f.accept(candidates.clone());
    f.set_source(&private.summary, Sensitivity::LocalOnly);
    let p = f.preview(vec!["Meetings/public.md"]);
    let stage = |candidates| {
        let run = f.run(&p, true).unwrap();
        f.service.stage_proposal(StageProposalRequest {
            root_id: f.root_id.clone(),
            scope_id: Some(p.scope_id.clone()),
            origin: Origin::Analysis {
                run_id: run.run_id.clone(),
            },
            candidates,
        })
    };
    assert_eq!(stage(candidates).unwrap_err().code, "forbidden");
    let source = f.source("Meetings/public.md");
    let conn = Connection {
        id: id(),
        from: Endpoint::Source {
            source_id: private.summary.source_id,
            locator: Locator {
                start_byte: 0,
                end_byte: 5,
                heading: None,
            },
        },
        relation: "mentions".into(),
        to: Endpoint::Topic { id: t.id },
        uncertainty: None,
        sources: vec![source.citation],
        valid_from: None,
        valid_to: None,
    };
    assert_eq!(
        stage(vec![Candidate::Connection { connection: conn }])
            .unwrap_err()
            .code,
        "forbidden"
    );
}
#[test]
fn historical_uuid_cannot_be_reused_after_creation_undo() {
    let f = Fixture::new();
    let t = topic("Historical identity");
    let r = f.accept(vec![Candidate::Topic { topic: t.clone() }]);
    let undo = f
        .service
        .undo(UndoRequest {
            root_id: f.root_id.clone(),
            operation_id: r.operation.unwrap().id,
        })
        .unwrap();
    let mut reused = t;
    reused.name = "Reused historical UUID".into();
    assert_eq!(
        f.stage(vec![Candidate::Topic { topic: reused }])
            .unwrap_err()
            .code,
        "conflict"
    );
    assert_eq!(
        f.service
            .undo(UndoRequest {
                root_id: f.root_id.clone(),
                operation_id: undo.operation.id
            })
            .unwrap_err()
            .code,
        "invalid_input"
    );
}
#[test]
fn explicit_source_review_can_lift_a_label_after_a_source_edit() {
    let f = Fixture::new();
    f.write("Meetings/one.md", "# Private old bytes\n");
    let old = f.source("Meetings/one.md");
    f.set_source(&old.summary, Sensitivity::LocalOnly);
    f.write("Meetings/one.md", "# Explicitly reviewed new bytes\n");
    let edited = f.source("Meetings/one.md");
    assert_ne!(old.summary.source_id, edited.summary.source_id);
    assert_eq!(edited.summary.sensitivity, Sensitivity::LocalOnly);
    f.set_source(&edited.summary, Sensitivity::CloudAllowed);
    let p = f.preview(vec!["Meetings/one.md"]);
    assert_eq!(p.sources.len(), 1);
    assert_eq!(p.sources[0].sensitivity, Sensitivity::CloudAllowed);
    assert!(f.run(&p, false).is_ok());
}

#[test]
fn source_metadata_can_be_registered_again_after_undo_without_losing_identity() {
    let f = Fixture::new();
    f.write("Meetings/one.md", "# Source\n");
    let source = f.source("Meetings/one.md");
    let operation = f
        .service
        .history_list(f.req())
        .unwrap()
        .operations
        .into_iter()
        .find(|o| o.description == "Register human source inventory")
        .unwrap();
    f.service
        .undo(UndoRequest {
            root_id: f.root_id.clone(),
            operation_id: operation.id,
        })
        .unwrap();
    let restored = f.source("Meetings/one.md");
    assert_eq!(restored.summary, source.summary);
}
#[test]
fn undo_of_update_then_original_creation_compensates_in_order() {
    let f = Fixture::new();
    let t = topic("Undoable topic");
    let creation = f
        .accept(vec![Candidate::Topic { topic: t.clone() }])
        .operation
        .unwrap();
    let label = f
        .service
        .sensitivity_set(SensitivitySetRequest {
            root_id: f.root_id.clone(),
            target: Endpoint::Topic { id: t.id },
            sensitivity: Sensitivity::LocalOnly,
        })
        .unwrap()
        .operation;
    f.service
        .undo(UndoRequest {
            root_id: f.root_id.clone(),
            operation_id: label.id,
        })
        .unwrap();
    f.service
        .undo(UndoRequest {
            root_id: f.root_id.clone(),
            operation_id: creation.id,
        })
        .unwrap();
    assert!(f.service.topics_list(f.req()).unwrap().topics.is_empty());
}

#[test]
fn validate_only_model_tool_call_does_not_publish_or_consume_capability() {
    let f = Fixture::new();
    f.write("Meetings/one.md", "# Synthetic selected meeting\n");
    let scope = f.preview(vec!["Meetings/one.md"]);
    let run = f.run(&scope, true).unwrap();
    let request = StageProposalRequest {
        root_id: f.root_id.clone(),
        scope_id: Some(scope.scope_id.clone()),
        origin: Origin::Analysis {
            run_id: run.run_id.clone(),
        },
        candidates: vec![Candidate::Topic {
            topic: topic("Buffered only"),
        }],
    };
    let count = f.service.history_list(f.req()).unwrap().operations.len();
    f.service.validate_proposal(request.clone()).unwrap();
    assert!(
        f.service
            .proposals_list(f.req())
            .unwrap()
            .proposals
            .is_empty()
    );
    assert!(f.service.topics_list(f.req()).unwrap().topics.is_empty());
    assert_eq!(
        f.service.history_list(f.req()).unwrap().operations.len(),
        count
    );
    assert!(
        f.service
            .scope_read(ScopeReadRequest {
                root_id: f.root_id.clone(),
                scope_id: scope.scope_id.clone(),
                run_id: run.run_id.clone()
            })
            .is_ok()
    );
    assert_eq!(
        f.service.stage_proposal(request).unwrap().status,
        ProposalStatus::Pending
    );
    assert_eq!(
        f.service
            .scope_read(ScopeReadRequest {
                root_id: f.root_id.clone(),
                scope_id: scope.scope_id,
                run_id: run.run_id
            })
            .unwrap_err()
            .code,
        "forbidden"
    );
}
#[test]
fn human_statement_retrieval_returns_persisted_text_after_restart() {
    let f = Fixture::new();
    let t = topic("Alex");
    f.accept(vec![Candidate::Topic { topic: t.clone() }]);
    let s = f
        .service
        .user_statement(UserStatementRequest {
            root_id: f.root_id.clone(),
            topic_id: t.id,
            text: "Use they/them; this is my correction".into(),
        })
        .unwrap();
    let reopened = WorkspaceService::open(f.temp.path().join("app-data")).unwrap();
    let got = reopened
        .statement_get(StatementGetRequest {
            root_id: f.root_id.clone(),
            statement_id: s.id.clone(),
        })
        .unwrap();
    assert_eq!(got, s);
    assert!(got.text.contains("they/them"));
}

#[test]
fn approving_an_edited_claim_records_the_human_edit_as_persisted_provenance() {
    let f = Fixture::new();
    f.write("Meetings/one.md", "# Meeting\nLedger launch is Friday\n");
    let source = f.source("Meetings/one.md");
    let t = topic("Ledger");
    let c = f.claim(&t, &source.citation, "Friday");
    let p = f
        .stage(vec![
            Candidate::Topic { topic: t.clone() },
            Candidate::Claim { claim: c.clone() },
        ])
        .unwrap();
    let mut edited = c;
    edited.value = "Monday".into();
    let decision = Decision::Approve {
        candidates: vec![
            Candidate::Topic { topic: t.clone() },
            Candidate::Claim { claim: edited },
        ],
    };
    let first = f.decide(&p, decision.clone()).unwrap();
    let retry = f.decide(&p, decision).unwrap();
    assert_eq!(first.operation, retry.operation);
    let view = f
        .service
        .topic_get(TopicGetRequest {
            root_id: f.root_id.clone(),
            topic_id: t.id,
        })
        .unwrap();
    let c = &view.current_claims[0];
    assert_eq!(c.value, "Monday");
    assert!(
        c.sources
            .iter()
            .any(|e| matches!(e,Evidence::Passage{source:s} if s.excerpt.contains("Friday")))
    );
    let statement = c
        .sources
        .iter()
        .find_map(|e| {
            if let Evidence::UserStatement { statement_id } = e {
                Some(statement_id)
            } else {
                None
            }
        })
        .unwrap();
    let text = f
        .service
        .statement_get(StatementGetRequest {
            root_id: f.root_id.clone(),
            statement_id: statement.clone(),
        })
        .unwrap();
    assert!(text.text.contains("Friday") && text.text.contains("Monday"));
    assert_eq!(
        fs::read_to_string(f.root.join("Meetings/one.md")).unwrap(),
        "# Meeting\nLedger launch is Friday\n"
    );
}

#[test]
fn explicit_source_repair_retains_identity_and_label_but_never_reanchors_old_citations() {
    let f = Fixture::new();
    f.write(
        "Meetings/one.md",
        "# Synthetic meeting\nOriginal assertion\n",
    );
    let source = f.source("Meetings/one.md");
    let t = topic("Alex");
    let c = f.claim(&t, &source.citation, "Original assertion");
    f.accept(vec![
        Candidate::Topic { topic: t.clone() },
        Candidate::Claim { claim: c },
    ]);
    f.set_source(&source.summary, Sensitivity::LocalOnly);
    fs::rename(
        f.root.join("Meetings/one.md"),
        f.root.join("Meetings/renamed.md"),
    )
    .unwrap();
    let target = f.source("Meetings/renamed.md");
    let repaired = f
        .service
        .source_identity_repair(SourceRepairRequest {
            root_id: f.root_id.clone(),
            source_id: source.summary.source_id.clone(),
            path: target.summary.path.clone(),
            expected_hash: target.summary.hash,
        })
        .unwrap();
    assert_eq!(repaired.source_id, source.summary.source_id);
    assert_eq!(repaired.sensitivity, Sensitivity::LocalOnly);
    assert_eq!(
        f.source("Meetings/renamed.md").summary.source_id,
        repaired.source_id
    );
    let view = f
        .service
        .topic_get(TopicGetRequest {
            root_id: f.root_id.clone(),
            topic_id: t.id,
        })
        .unwrap();
    assert!(view.warnings.iter().any(|w| w.contains("Stale citation")));
    assert_eq!(
        view.current_claims[0].sources[0],
        Evidence::Passage {
            source: source.citation
        }
    );
}

#[test]
fn logical_claim_identity_cannot_collide_with_a_proposed_topic() {
    let f = Fixture::new();
    f.write("Meetings/one.md", "# Synthetic source\n");
    let source = f.source("Meetings/one.md");
    let t = topic("Alex");
    let mut claim = f.claim(&t, &source.citation, "Leader");
    claim.id = t.id.clone();
    assert_eq!(
        f.stage(vec![
            Candidate::Topic { topic: t },
            Candidate::Claim { claim }
        ])
        .unwrap_err()
        .code,
        "invalid_input"
    );
}
