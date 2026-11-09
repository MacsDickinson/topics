#[path = "support/setup.rs"]
mod support;
use support::*;
use topics_core::*;

#[test]
fn failed_batch_persists_only_exact_allowlisted_diagnostic_pairs() {
    use serde_json::json;
    let turn = json!({"phase":"turn","reason":"model_terminal_failed"});
    let publication = json!({"phase":"publication","reason":"scope_not_read"});
    let sentinel = "PRIVATE_ERROR_SENTINEL";
    let mut cases = vec![
        (Some(turn.clone()), Some(turn.clone())),
        (Some(publication.clone()), Some(publication)),
        (None, None),
        (Some(json!(sentinel)), None),
        (Some(json!([turn])), None),
        (Some(json!({})), None),
        (Some(json!({"phase":"turn"})), None),
        (
            Some(json!({"phase":null,"reason":"model_terminal_failed"})),
            None,
        ),
        (
            Some(json!({"phase":"publication","reason":"model_terminal_failed"})),
            None,
        ),
        (
            Some(json!({"phase":"turn","reason":"scope_not_read"})),
            None,
        ),
        (Some(json!({"phase":"turn","reason":sentinel})), None),
        (
            Some(json!({"phase":"turn","reason":{"private":sentinel}})),
            None,
        ),
        (
            Some(json!({"phase":sentinel,"reason":"scope_not_read"})),
            None,
        ),
        (
            Some(json!({"phase":"turn","reason":"model_terminal_failed","raw":sentinel})),
            None,
        ),
    ];
    // Contract fixture is reviewable independently of the sanitizer implementation.
    for line in include_str!("../../../features/folder-setup/FAILURE-ORIGINS.md").lines() {
        if line.starts_with("| `") {
            let fields: Vec<_> = line.split('`').collect();
            let allowed = json!({"phase":fields[1],"reason":fields[3]});
            cases.push((Some(allowed.clone()), Some(allowed.clone())));
            let mut extra = allowed;
            extra["private"] = json!(sentinel);
            cases.push((Some(extra), None));
            cases.push((
                Some(json!({"phase":"PRIVATE_ERROR_SENTINEL","reason":fields[3]})),
                None,
            ));
        }
    }
    for (details, expected) in cases {
        let mut f = Fixture::new();
        f.prepare_default(1);
        let cap = f.start();
        let mut failure = ApiError::new(sentinel, sentinel);
        failure.details = details;
        let job = f
            .service
            .setup_batch_fail(SetupBatchFailRequest {
                capability: cap.clone(),
                error: failure,
            })
            .unwrap();
        let error = job.batches[0].error.as_ref().unwrap();
        assert_eq!(error.code, "runtime_unavailable");
        assert_eq!(error.details, expected);
        assert!(!serde_json::to_string(error).unwrap().contains(sentinel));
        assert!(job.coverage.processed.is_empty());
        assert!(job.graph.items.is_empty());
        assert!(job.batches[0].proposal_ids.is_empty());
        assert!(
            f.service
                .setup_batch_fail(SetupBatchFailRequest {
                    capability: cap,
                    error: ApiError::new(sentinel, sentinel),
                })
                .is_err(),
            "late diagnostics cannot change the completed failure checkpoint"
        );
        f.restart();
        assert_eq!(
            f.job.as_ref().unwrap().batches[0].error.as_ref().unwrap(),
            error
        );
        f.unchanged();
    }
}

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
#[cfg(feature = "test-support")]
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
#[test]
fn prior_completed_context_is_job_scoped_and_rechecks_current_privacy() {
    let mut f = Fixture::new();
    f.prepare_default(3);
    f.finish_one(false);
    let cap = f.start();
    let context = f
        .service
        .setup_draft_context(SetupDraftContextRequest {
            capability: cap.clone(),
        })
        .unwrap();
    assert_eq!(context.topics.len(), 2);
    assert_eq!(context.claims.len(), 1);
    assert_eq!(context.connections.len(), 1);
    assert!(context.claims[0].supersedes.is_empty());
    let prior = context.claims[0]
        .sources
        .iter()
        .find_map(|e| {
            if let Evidence::Passage { source } = e {
                Some(source)
            } else {
                None
            }
        })
        .unwrap()
        .clone();
    f.service
        .sensitivity_set(SensitivitySetRequest {
            root_id: f.root_id.clone(),
            target: Endpoint::Source {
                source_id: prior.source_id.clone(),
                locator: Locator {
                    start_byte: 0,
                    end_byte: 0,
                    heading: None,
                },
            },
            sensitivity: Sensitivity::LocalOnly,
        })
        .unwrap();
    let filtered = f
        .service
        .setup_draft_context(SetupDraftContextRequest {
            capability: cap.clone(),
        })
        .unwrap();
    assert!(
        filtered.claims.is_empty() && filtered.connections.is_empty() && filtered.topics.is_empty()
    );
    let c = claim(&context.topics[0], "new", "private-derived", &prior);
    assert!(
        f.service
            .setup_batch_validate(SetupBatchStageRequest {
                capability: cap,
                candidates: vec![Candidate::Claim { claim: c }]
            })
            .is_err()
    );
}
#[test]
fn identical_refreshed_manifest_reuses_pending_questions_without_new_calls() {
    let mut f = Fixture::new();
    f.prepare_default(2);
    f.finish_one(true);
    f.finish();
    let first = f.job.as_ref().unwrap().clone();
    f.preview(1);
    f.authorize();
    assert_eq!(f.job.as_ref().unwrap().job_id, first.job_id);
    assert_eq!(f.job.as_ref().unwrap().graph, first.graph);
    assert_eq!(f.job.as_ref().unwrap().coverage.model_calls, 2);
}
#[test]
fn explicitly_deferred_items_are_not_reselected_by_default_baseline_save() {
    let mut f = Fixture::new();
    f.prepare_default(2);
    f.finish();
    let j = f.job.as_ref().unwrap();
    let claim_id = j
        .graph
        .items
        .iter()
        .find(|i| matches!(i.candidate, Candidate::Claim { .. }))
        .unwrap()
        .item_id
        .clone();
    let updated = f
        .service
        .setup_draft_disposition(SetupDraftDispositionRequest {
            root_id: f.root_id.clone(),
            job_id: j.job_id.clone(),
            expected_revision: j.revision,
            item_ids: vec![claim_id.clone()],
            disposition: DraftDisposition::Deferred,
        })
        .unwrap();
    f.job = Some(updated);
    f.all_receipt();
    assert!(!f.receipt.as_ref().unwrap().included_ids.contains(&claim_id));
    f.decide();
    assert!(
        f.job
            .as_ref()
            .unwrap()
            .graph
            .items
            .iter()
            .any(|i| i.item_id == claim_id && i.disposition == DraftDisposition::Deferred)
    );
}
#[test]
fn legacy_statement_reference_does_not_invent_personal_verification() {
    let f = Fixture::new();
    let t = topic("Alex", TopicKind::Person);
    let p = f
        .service
        .manual_propose(ManualProposeRequest {
            root_id: f.root_id.clone(),
            candidates: vec![Candidate::Topic { topic: t.clone() }],
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
    let statement = f
        .service
        .user_statement(UserStatementRequest {
            root_id: f.root_id.clone(),
            topic_id: t.id.clone(),
            text: "check with legal".into(),
        })
        .unwrap();
    let c = Claim {
        id: id(),
        revision_id: id(),
        subject_id: t.id.clone(),
        predicate: "department".into(),
        value: "legal".into(),
        uncertainty: None,
        sources: vec![Evidence::UserStatement {
            statement_id: statement.id,
        }],
        supersedes: vec![],
        recorded_at: "2026-11-09T00:00:00Z".into(),
        valid_from: None,
        valid_to: None,
    };
    let p = f
        .service
        .manual_propose(ManualProposeRequest {
            root_id: f.root_id.clone(),
            candidates: vec![Candidate::Claim { claim: c }],
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
fn baseline_is_not_human_value_authority_but_exact_confirmation_is() {
    let mut f = Fixture::new();
    f.single(2);
    let t = f.topics[0].clone();
    f.service
        .sensitivity_set(SensitivitySetRequest {
            root_id: f.root_id.clone(),
            target: Endpoint::Topic { id: t.id.clone() },
            sensitivity: Sensitivity::CloudAllowed,
        })
        .unwrap();
    f.write("unrelated.md", "# Other\nUnrelated eligible context\n");
    let scope = f
        .service
        .scope_preview(ScopePreviewRequest {
            root_id: f.root_id.clone(),
            spec: ScopeSpec {
                files: vec!["unrelated.md".into()],
                folders: vec![],
                date_from: None,
                date_to: None,
            },
        })
        .unwrap();
    let run = f
        .service
        .analysis_start(AnalysisStartRequest {
            root_id: f.root_id.clone(),
            scope_id: scope.scope_id.clone(),
            confirm_unknown: true,
        })
        .unwrap();
    let request = SourceContextRequest {
        root_id: f.root_id.clone(),
        scope_id: scope.scope_id,
        run_id: run.run_id,
        topic_id: t.id.clone(),
    };
    assert!(
        f.service
            .source_context(request.clone())
            .unwrap()
            .current_claims
            .is_empty()
    );
    let r = f
        .service
        .setup_confirm_preview(SetupConfirmPreviewRequest {
            root_id: f.root_id.clone(),
            revision_ids: vec![f.claims[0].revision_id.clone()],
        })
        .unwrap();
    f.service
        .setup_decide(SetupDecisionRequest {
            root_id: f.root_id.clone(),
            receipt_id: r.receipt_id,
            expected_fingerprint: r.fingerprint,
        })
        .unwrap();
    let context = f.service.source_context(request).unwrap();
    assert_eq!(context.current_claims.len(), 1);
    assert_eq!(context.current_claims[0].value, "he/him");
    assert!(context.current_claims[0].sources.is_empty());
    assert_eq!(
        f.review(&t)
            .reviews
            .iter()
            .filter(|r| r.state == ReviewState::FromNotes)
            .count(),
        1
    );
}
#[test]
fn metadata_draft_edit_records_history_without_confirming_value() {
    let mut f = Fixture::new();
    f.prepare_default(1);
    f.finish();
    let job = f.job.as_ref().unwrap();
    let item = job
        .graph
        .items
        .iter()
        .find(|i| matches!(i.candidate, Candidate::Claim { .. }))
        .unwrap();
    let mut c = if let Candidate::Claim { claim } = &item.candidate {
        claim.clone()
    } else {
        unreachable!()
    };
    c.recorded_at = "2026-11-10T00:00:00Z".into();
    f.job = Some(
        f.service
            .setup_draft_edit(SetupDraftEditRequest {
                root_id: f.root_id.clone(),
                job_id: job.job_id.clone(),
                expected_revision: job.revision,
                item_id: item.item_id.clone(),
                candidate: Candidate::Claim { claim: c },
            })
            .unwrap(),
    );
    f.all_receipt();
    f.decide();
    let t = f.saved().into_iter().find(|t| t.name == "Alex").unwrap();
    let review = f.review(&t);
    assert!(
        review
            .reviews
            .iter()
            .all(|m| m.state == ReviewState::FromNotes)
    );
    let statement = review
        .reviews
        .iter()
        .find_map(|m| m.statement_id.clone())
        .unwrap();
    let statement = f
        .service
        .statement_get(StatementGetRequest {
            root_id: f.root_id.clone(),
            statement_id: statement,
        })
        .unwrap();
    assert!(statement.text.contains("Human draft edit from") && statement.text.contains(" to "));
    f.unchanged();
}
#[test]
fn model_local_only_draft_labels_remain_restrictive() {
    let mut f = Fixture::new();
    f.prepare_default(2);
    let cap = f.start();
    let source = f.read(&cap)[0].citation.clone();
    let mut private = topic("Restricted derived person", TopicKind::Person);
    private.sensitivity = Sensitivity::LocalOnly;
    f.publish(
        cap,
        vec![
            Candidate::Topic {
                topic: private.clone(),
            },
            Candidate::Claim {
                claim: claim(&private, "role", "sensitive", &source),
            },
        ],
    );
    assert!(f.job.as_ref().unwrap().graph.items.iter().any(|i|matches!(&i.candidate,Candidate::Topic{topic} if topic.sensitivity==Sensitivity::LocalOnly)));
    let cap = f.start();
    let context = f
        .service
        .setup_draft_context(SetupDraftContextRequest { capability: cap })
        .unwrap();
    assert!(context.topics.is_empty() && context.claims.is_empty());
}
#[test]
fn verbatim_passages_in_different_documents_are_not_independent_confirmation() {
    use sha2::Digest;
    let mut f = Fixture::new();
    f.write("first.md", "# First summary\nAlex leads Ledger\n");
    f.write("copy.md", "# Different heading\nAlex leads Ledger\n");
    f.preview(2);
    f.authorize();
    let cap = f.start();
    let sources = f.read(&cap);
    let mut citations = vec![];
    for source in &sources {
        let mut citation = source.citation.clone();
        let start = source.content.find("Alex leads Ledger").unwrap();
        citation.locator.start_byte = start as u64;
        citation.locator.end_byte = (start + "Alex leads Ledger".len()) as u64;
        citation.excerpt = "Alex leads Ledger".into();
        citation.passage_hash = format!("{:x}", sha2::Sha256::digest(citation.excerpt.as_bytes()));
        citations.push(citation);
    }
    assert_ne!(citations[0].document_hash, citations[1].document_hash);
    let alex = topic("Alex", TopicKind::Person);
    let ledger = topic("Ledger", TopicKind::Project);
    let mut rel = relation(&alex, &ledger, &citations[0]);
    rel.sources = citations;
    f.publish(
        cap,
        vec![
            Candidate::Topic { topic: alex },
            Candidate::Topic { topic: ledger },
            Candidate::Connection { connection: rel },
        ],
    );
    let item = f
        .job
        .as_ref()
        .unwrap()
        .graph
        .items
        .iter()
        .find(|i| matches!(i.candidate, Candidate::Connection { .. }))
        .unwrap();
    assert_eq!(item.independent_evidence_groups.len(), 1);
    if let Candidate::Connection { connection } = &item.candidate {
        assert_eq!(connection.sources.len(), 2);
    } else {
        panic!("expected relation")
    }
}
#[test]
fn concurrent_known_subject_duplicate_is_no_change_and_new_evidence_is_deferred() {
    let mut f = Fixture::new();
    f.single(1);
    let t = f.topics[0].clone();
    f.write("fresh.md", "# Fresh\nAlex leads Ledger.\n");
    let preview = |f: &Fixture, files| {
        f.service
            .setup_manifest_preview(SetupManifestPreviewRequest {
                root_id: f.root_id.clone(),
                spec: ScopeSpec {
                    files: vec!["fresh.md".into()],
                    folders: vec![],
                    date_from: None,
                    date_to: None,
                },
                roles: vec![],
                limits: SetupLimits {
                    max_files_per_batch: files,
                    max_bytes_per_batch: 2 * 1024 * 1024,
                    max_batches: 10,
                    max_candidates_per_turn: 200,
                    max_turns_per_batch: 2,
                },
                runtime: "synthetic".into(),
            })
            .unwrap()
    };
    f.manifest = Some(preview(&f, 1));
    f.authorize();
    let first_cap = f.start();
    let source = f.read(&first_cap)[0].citation.clone();
    let first_job = f.job.as_ref().unwrap().job_id.clone();
    f.manifest = Some(preview(&f, 2));
    f.authorize();
    let second_cap = f.start();
    let second_job = f.job.as_ref().unwrap().job_id.clone();
    assert_ne!(first_job, second_job);
    let first = claim(&t, "role", "Ledger lead", &source);
    let second = claim(&t, "role", "Ledger lead", &source);
    f.publish(first_cap, vec![Candidate::Claim { claim: first }]);
    f.publish(second_cap, vec![Candidate::Claim { claim: second }]);
    f.job = Some(
        f.service
            .setup_job_get(SetupJobRequest {
                root_id: f.root_id.clone(),
                job_id: first_job,
            })
            .unwrap(),
    );
    f.all_receipt();
    f.decide();
    f.job = Some(
        f.service
            .setup_job_get(SetupJobRequest {
                root_id: f.root_id.clone(),
                job_id: second_job,
            })
            .unwrap(),
    );
    f.refresh();
    assert!(
        f.job
            .as_ref()
            .unwrap()
            .graph
            .items
            .iter()
            .all(|i| i.disposition == DraftDisposition::Saved)
    );
    assert_eq!(
        f.review(&t)
            .topic
            .current_claims
            .iter()
            .filter(|c| c.predicate == "role")
            .count(),
        1
    );
    // A third document is new evidence, not another independently accepted fact.
    f.write(
        "new-evidence.md",
        "# New account\nAlex leads Ledger, with a new citation.\n",
    );
    let mut p = preview(&f, 1);
    p.spec.files = vec!["new-evidence.md".into()];
    let m = f
        .service
        .setup_manifest_preview(SetupManifestPreviewRequest {
            root_id: f.root_id.clone(),
            spec: p.spec,
            roles: vec![],
            limits: p.limits,
            runtime: p.runtime,
        })
        .unwrap();
    f.manifest = Some(m);
    f.authorize();
    let cap = f.start();
    let source = f.read(&cap)[0].citation.clone();
    f.publish(
        cap,
        vec![Candidate::Claim {
            claim: claim(&t, "role", "Ledger lead", &source),
        }],
    );
    f.all_receipt();
    assert!(f.receipt.as_ref().unwrap().candidates.is_empty());
    assert!(f.job.as_ref().unwrap().graph.items.iter().any(|i|matches!(&i.candidate,Candidate::Claim{claim} if claim.sources.len()==2&&!claim.supersedes.is_empty())));
    assert!(!f.job.as_ref().unwrap().graph.issues.is_empty());
}
#[test]
fn a_later_batch_cannot_reuse_an_edited_draft_revision_identity() {
    let mut f = Fixture::new();
    f.prepare_default(2);
    f.finish_one(false);
    let job = f.job.as_ref().unwrap();
    let item = job
        .graph
        .items
        .iter()
        .find(|i| matches!(i.candidate, Candidate::Claim { .. }))
        .unwrap();
    let mut edited = if let Candidate::Claim { claim } = &item.candidate {
        claim.clone()
    } else {
        unreachable!()
    };
    edited.value = "human edit".into();
    f.job = Some(
        f.service
            .setup_draft_edit(SetupDraftEditRequest {
                root_id: f.root_id.clone(),
                job_id: job.job_id.clone(),
                expected_revision: job.revision,
                item_id: item.item_id.clone(),
                candidate: Candidate::Claim {
                    claim: edited.clone(),
                },
            })
            .unwrap(),
    );
    let cap = f.start();
    let s = f.read(&cap)[0].citation.clone();
    let mut conflicting = edited;
    conflicting.value = "model overwrite".into();
    conflicting.sources = vec![Evidence::Passage { source: s }];
    let e = f
        .service
        .setup_batch_validate(SetupBatchStageRequest {
            capability: cap,
            candidates: vec![Candidate::Claim { claim: conflicting }],
        })
        .unwrap_err();
    assert_eq!(e.code, "invalid_input");
    f.refresh();
    assert!(
        f.job
            .as_ref()
            .unwrap()
            .graph
            .items
            .iter()
            .any(|i| matches!(&i.candidate,Candidate::Claim{claim} if claim.value=="human edit"))
    );
}
#[test]
fn receipt_operation_count_includes_retained_human_edit_history() {
    let mut f = Fixture::new();
    f.write(
        "fields.md",
        "# Report\nTwenty explicitly reported fields.\n",
    );
    f.preview(1);
    f.authorize();
    let cap = f.start();
    let source = f.read(&cap)[0].citation.clone();
    let t = topic("Ledger", TopicKind::Project);
    let mut candidates = vec![Candidate::Topic { topic: t.clone() }];
    for n in 0..20 {
        candidates.push(Candidate::Claim {
            claim: claim(&t, &format!("field-{n}"), "reported", &source),
        });
    }
    f.publish(cap, candidates);
    let ids: Vec<_> = f
        .job
        .as_ref()
        .unwrap()
        .graph
        .items
        .iter()
        .filter(|i| matches!(i.candidate, Candidate::Claim { .. }))
        .map(|i| i.item_id.clone())
        .collect();
    for id in ids {
        for n in 0..4 {
            f.refresh();
            let job = f.job.as_ref().unwrap();
            let mut c = job
                .graph
                .items
                .iter()
                .find_map(|i| {
                    if i.item_id == id
                        && let Candidate::Claim { claim } = &i.candidate
                    {
                        Some(claim.clone())
                    } else {
                        None
                    }
                })
                .unwrap();
            c.recorded_at = format!("2026-11-{:02}T00:00:00Z", 10 + n);
            f.job = Some(
                f.service
                    .setup_draft_edit(SetupDraftEditRequest {
                        root_id: f.root_id.clone(),
                        job_id: job.job_id.clone(),
                        expected_revision: job.revision,
                        item_id: id.clone(),
                        candidate: Candidate::Claim { claim: c },
                    })
                    .unwrap(),
            );
        }
    }
    f.all_receipt();
    assert_eq!(f.receipt.as_ref().unwrap().operation_count, 2);
    f.decide();
    assert_eq!(f.result.as_ref().unwrap().operations.len(), 2);
    assert!(
        f.review(&t)
            .reviews
            .iter()
            .all(|m| m.state == ReviewState::FromNotes)
    );
    f.unchanged();
}
#[test]
fn undo_can_be_followed_by_new_consent_for_a_fresh_setup_job() {
    let mut f = Fixture::new();
    f.prepare_default(2);
    f.finish();
    f.all_receipt();
    f.decide();
    let first = f.job.as_ref().unwrap().job_id.clone();
    let operation = f.result.as_ref().unwrap().operations[0].id.clone();
    f.service
        .undo(UndoRequest {
            root_id: f.root_id.clone(),
            operation_id: operation,
        })
        .unwrap();
    assert!(f.saved().is_empty());
    f.preview(1);
    f.authorize();
    assert_ne!(f.job.as_ref().unwrap().job_id, first);
    f.finish();
    f.all_receipt();
    f.decide();
    assert_eq!(f.saved().len(), 2);
    f.unchanged();
}
