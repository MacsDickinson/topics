#[path = "support/setup.rs"]
mod support;
use support::*;
use topics_core::*;

#[test]
fn exact_manifest_multibatch_dedupe_save_confirm_rebuild() {
    let mut f = Fixture::new();
    f.prepare_default(4);
    f.finish();
    assert_eq!(f.job.as_ref().unwrap().graph.items.len(), 4);
    f.all_receipt();
    f.decide();
    assert_eq!(f.saved().len(), 2);
    assert_eq!(f.save_actions, 1);
    for t in f.saved() {
        let r = f.review(&t);
        assert!(r.reviews.iter().all(|v| v.state == ReviewState::FromNotes));
    }
    f.restart();
    assert_eq!(f.saved().len(), 2);
    f.unchanged();
}
#[test]
fn revoked_publisher_cannot_attach_draft_or_reuse_attempt() {
    let mut f = Fixture::new();
    f.prepare_default(2);
    let cap = f.start();
    let s = f.read(&cap)[0].citation.clone();
    let t = topic("Alex", TopicKind::Person);
    let request = SetupBatchStageRequest {
        capability: cap.clone(),
        candidates: vec![
            Candidate::Topic { topic: t.clone() },
            Candidate::Claim {
                claim: claim(&t, "role", "lead", &s),
            },
        ],
    };
    let e = f
        .service
        .setup_batch_stage_guarded(request.clone(), |_, _| {
            Err(ApiError::new("cancelled", "revoked"))
        })
        .unwrap_err();
    assert_eq!(e.code, "cancelled");
    f.refresh();
    assert!(f.job.as_ref().unwrap().graph.items.is_empty());
    assert!(f.service.setup_batch_validate(request).is_err());
    f.restart();
    assert_eq!(f.job.as_ref().unwrap().status, SetupJobStatus::Paused);
    f.control(SetupJobAction::Resume);
    assert!(f.job.as_ref().unwrap().graph.items.is_empty());
}
#[test]
fn current_root_and_scope_provenance_are_not_rebased() {
    let mut f = Fixture::new();
    f.prepare_default(2);
    let cap = f.start();
    let old = f.root_id.clone();
    let other = f.temp.path().join("other");
    std::fs::create_dir(&other).unwrap();
    f.service
        .root_set(RootSetRequest {
            path: other.to_str().unwrap().into(),
            create: false,
        })
        .unwrap();
    let e = f
        .service
        .setup_draft_context(SetupDraftContextRequest { capability: cap })
        .unwrap_err();
    assert_eq!(e.code, "stale_scope");
    let new = f
        .service
        .root_set(RootSetRequest {
            path: f.root.to_str().unwrap().into(),
            create: false,
        })
        .unwrap()
        .root
        .root_id;
    assert_ne!(new, old);
    assert!(
        f.service
            .setup_job_get(SetupJobRequest {
                root_id: new,
                job_id: f.job.unwrap().job_id
            })
            .is_err()
    );
}
#[test]
fn stale_source_invalidates_receipt_and_preserves_draft() {
    let mut f = Fixture::new();
    f.prepare_default(2);
    f.finish();
    f.all_receipt();
    let p = f.manifest.as_ref().unwrap().batches[0].sources[0]
        .source
        .path
        .clone();
    std::fs::write(f.root.join(p), "changed").unwrap();
    let r = f.receipt.as_ref().unwrap();
    let e = f
        .service
        .setup_decide(SetupDecisionRequest {
            root_id: f.root_id.clone(),
            receipt_id: r.receipt_id.clone(),
            expected_fingerprint: r.fingerprint.clone(),
        })
        .unwrap_err();
    assert_eq!(e.code, "conflict");
    assert!(f.saved().is_empty());
    f.refresh();
    assert!(
        f.job
            .as_ref()
            .unwrap()
            .graph
            .items
            .iter()
            .all(|i| i.disposition == DraftDisposition::Pending)
    );
}
#[test]
fn inventory_excludes_hidden_symlinks_and_private_manifest_metadata() {
    let mut f = Fixture::new();
    f.write("odd/public.md", "# Public\nAlex leads Ledger");
    f.write("secret/private.md", "# Private sentinel\nprivate bytes");
    f.write(".config/hidden.md", "hidden");
    f.write("asset.txt", "attachment");
    #[cfg(unix)]
    std::os::unix::fs::symlink(f.root.join("secret"), f.root.join("copy")).unwrap();
    let s = f
        .inventory()
        .documents
        .into_iter()
        .find(|d| d.source.path.contains("private"))
        .unwrap()
        .source;
    f.service
        .sensitivity_set(SensitivitySetRequest {
            root_id: f.root_id.clone(),
            target: Endpoint::Source {
                source_id: s.source_id,
                locator: Locator {
                    start_byte: 0,
                    end_byte: 0,
                    heading: None,
                },
            },
            sensitivity: Sensitivity::LocalOnly,
        })
        .unwrap();
    let inv = f.inventory();
    assert_eq!(inv.documents.len(), 2);
    assert!(inv.excluded.iter().any(|e| e.path == ".config"));
    f.preview(1);
    f.authorize();
    let cap = f.start();
    let sources = f.read(&cap);
    let wire = serde_json::to_string(&sources).unwrap();
    assert!(!wire.contains("Private sentinel") && !wire.contains("private.md"));
    assert!(
        f.service
            .setup_draft_context(SetupDraftContextRequest { capability: cap })
            .unwrap()
            .topics
            .is_empty()
    );
}
#[test]
fn explicit_correction_survives_app_cache_loss_and_keeps_exact_statement() {
    let mut f = Fixture::new();
    f.single(2);
    f.correct();
    f.decide();
    let topic = f.topics[0].clone();
    let review = f.review(&topic);
    assert!(
        review
            .reviews
            .iter()
            .any(|r| r.state == ReviewState::ConfirmedByYou)
    );
    assert!(
        review
            .reviews
            .iter()
            .any(|r| r.state == ReviewState::FromNotes)
    );
    let before = review.clone();
    f.service = topics_store::WorkspaceService::open(f.temp.path().join("fresh-app")).unwrap();
    f.root_id = f
        .service
        .root_set(RootSetRequest {
            path: f.root.to_str().unwrap().into(),
            create: false,
        })
        .unwrap()
        .root
        .root_id;
    let after = f.review(&topic);
    assert_eq!(before, after);
    let s = f
        .service
        .statement_get(StatementGetRequest {
            root_id: f.root_id.clone(),
            statement_id: after
                .reviews
                .iter()
                .find_map(|r| r.statement_id.clone())
                .unwrap(),
        })
        .unwrap();
    assert_eq!(s.text, "Alex uses they/them");
    f.unchanged();
}
#[test]
fn model_validation_is_not_a_private_identity_oracle() {
    fn run(private: bool) -> Result<(), ApiError> {
        let mut f = Fixture::new();
        f.prepare_default(1);
        if private {
            let hidden = Topic {
                id: id(),
                topic_kind: TopicKind::Person,
                name: "Alex".into(),
                aliases: vec![],
                sensitivity: Sensitivity::LocalOnly,
            };
            let p = f
                .service
                .manual_propose(ManualProposeRequest {
                    root_id: f.root_id.clone(),
                    candidates: vec![Candidate::Topic { topic: hidden }],
                })
                .unwrap()
                .proposal;
            f.service
                .proposal_decide(ProposalDecideRequest {
                    root_id: f.root_id.clone(),
                    proposal_id: p.id,
                    expected_fingerprint: p.fingerprint,
                    decision: Decision::Approve {
                        candidates: p.candidates,
                    },
                })
                .unwrap();
        }
        let cap = f.start();
        let s = f.read(&cap)[0].citation.clone();
        let t = topic("Alex", TopicKind::Person);
        f.service.setup_batch_validate(SetupBatchStageRequest {
            capability: cap,
            candidates: vec![
                Candidate::Topic { topic: t.clone() },
                Candidate::Claim {
                    claim: claim(&t, "role", "lead", &s),
                },
            ],
        })
    }
    assert_eq!(run(false), run(true));
}
#[test]
fn partial_selection_closes_endpoints_without_sibling_claims() {
    let mut f = Fixture::new();
    f.prepare_default(2);
    f.finish();
    let j = f.job.as_ref().unwrap();
    let relation = j
        .graph
        .items
        .iter()
        .find(|i| matches!(i.candidate, Candidate::Connection { .. }))
        .unwrap()
        .item_id
        .clone();
    let receipt = f
        .service
        .setup_save_preview(SetupSavePreviewRequest {
            root_id: f.root_id.clone(),
            job_id: j.job_id.clone(),
            item_ids: vec![relation],
        })
        .unwrap();
    assert_eq!(receipt.dependency_ids.len(), 2);
    assert_eq!(receipt.candidates.len(), 3);
    assert!(
        !receipt
            .candidates
            .iter()
            .any(|c| matches!(c, Candidate::Claim { .. }))
    );
    f.receipt = Some(receipt);
    f.decide();
    assert!(f.job.as_ref().unwrap().graph.items.iter().any(|i| matches!(
        i.candidate,
        Candidate::Claim { .. }
    ) && i.disposition
        == DraftDisposition::Pending));
}
#[test]
fn confirmation_hashes_reject_changed_revision_and_preserve_other_groups() {
    let mut f = Fixture::new();
    f.single(2);
    let c = f.claims[0].clone();
    let r = f
        .service
        .setup_confirm_preview(SetupConfirmPreviewRequest {
            root_id: f.root_id.clone(),
            revision_ids: vec![c.revision_id.clone()],
        })
        .unwrap();
    let p = f.root.join(format!("topics/claims/{}.md", c.revision_id));
    std::fs::write(p, "externally changed").unwrap();
    let e = f
        .service
        .setup_decide(SetupDecisionRequest {
            root_id: f.root_id.clone(),
            receipt_id: r.receipt_id,
            expected_fingerprint: r.fingerprint,
        })
        .unwrap_err();
    assert_eq!(e.code, "conflict");
    assert!(
        f.review(&f.topics[0])
            .reviews
            .iter()
            .all(|m| m.state != ReviewState::ConfirmedByYou)
    );
}
#[test]
fn legacy_acceptance_does_not_invent_verification() {
    let mut f = Fixture::new();
    f.write("source.md", "# Source\nAlex uses he/him");
    let s = f.citation("source.md");
    let t = topic("Alex", TopicKind::Person);
    let c = claim(&t, "pronouns", "he/him", &s);
    let p = f
        .service
        .manual_propose(ManualProposeRequest {
            root_id: f.root_id.clone(),
            candidates: vec![
                Candidate::Topic { topic: t.clone() },
                Candidate::Claim { claim: c },
            ],
        })
        .unwrap()
        .proposal;
    f.service
        .proposal_decide(ProposalDecideRequest {
            root_id: f.root_id.clone(),
            proposal_id: p.id,
            expected_fingerprint: p.fingerprint,
            decision: Decision::Approve {
                candidates: p.candidates,
            },
        })
        .unwrap();
    assert_eq!(
        f.review(&t).reviews[0].state,
        ReviewState::AcceptedUnverified
    );
}
#[test]
fn bounded_multigroup_decision_reports_partial_and_recovers_exact_receipt() {
    let mut f = Fixture::new();
    f.write("many.md", "# Synthetic\nMultiple explicit topics.\n");
    f.preview(1);
    f.authorize();
    let cap = f.start();
    let source = f.read(&cap)[0].citation.clone();
    let mut candidates = vec![];
    for n in 0..31 {
        let t = topic(&format!("Topic {n}"), TopicKind::Project);
        candidates.push(Candidate::Topic { topic: t.clone() });
        candidates.push(Candidate::Claim {
            claim: claim(&t, "status", "reported", &source),
        });
    }
    f.publish(cap, candidates);
    f.all_receipt();
    assert_eq!(f.receipt.as_ref().unwrap().operation_count, 3);
    f.service.interrupt_next_operation_after(45);
    let r = f.receipt.as_ref().unwrap();
    let result = f
        .service
        .setup_decide(SetupDecisionRequest {
            root_id: f.root_id.clone(),
            receipt_id: r.receipt_id.clone(),
            expected_fingerprint: r.fingerprint.clone(),
        })
        .unwrap();
    assert_eq!(result.status, SetupDecisionStatus::Partial);
    assert_eq!(result.operations.len(), 1);
    assert_eq!(result.saved_ids.len(), 30);
    let current = f.service.topics_list(f.root_request()).unwrap();
    assert_eq!(current.topics.len(), 31);
    assert!(
        f.service
            .setup_decide(SetupDecisionRequest {
                root_id: f.root_id.clone(),
                receipt_id: r.receipt_id.clone(),
                expected_fingerprint: r.fingerprint.clone()
            })
            .is_err()
    );
    f.refresh();
    assert!(
        f.job
            .as_ref()
            .unwrap()
            .graph
            .items
            .iter()
            .any(|i| i.disposition != DraftDisposition::Saved)
    );
}
