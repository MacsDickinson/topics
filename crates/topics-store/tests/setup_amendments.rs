//! Semantic RED/GREEN for the additive coordinator-reviewed contract seams.
#[path = "support/setup.rs"]
mod support;
use support::*;
use topics_core::*;
#[test]
fn successful_empty_terminal_completion_is_guarded_without_empty_model_proposals() {
    let mut f = Fixture::new();
    f.prepare_default(1);
    let cap = f.start();
    let request = SetupBatchStageRequest {
        capability: cap,
        candidates: vec![],
    };
    assert!(
        f.service.setup_batch_validate(request.clone()).is_err(),
        "model tool payload remains 1–200"
    );
    let job = f
        .service
        .setup_batch_stage_guarded(request, |proposal, action| {
            assert!(!proposal.id.is_empty());
            action()
        })
        .expect("successful app-owned empty terminal completion needs a checkpoint");
    assert_eq!(job.coverage.processed.len(), 1);
    assert!(job.graph.items.is_empty());
    assert!(f.saved().is_empty());
}
#[test]
fn explicit_reconsent_preserves_completed_graph_and_issues_only_unsent_subsets() {
    let mut f = Fixture::new();
    f.prepare_default(3);
    f.finish_one(false);
    let before = f.job.as_ref().unwrap().clone();
    let completed = before.coverage.processed[0].clone();
    let changed = before.manifest.batches[1].sources[0].source.path.clone();
    std::fs::write(f.root.join(&changed), "# Changed\nUpdated unsent content\n").unwrap();
    f.write("added.md", "# Added\nNew source requires this consent\n");
    f.preview(20);
    let manifest = f.manifest.as_ref().unwrap().clone();
    let job = f
        .service
        .setup_job_reconsent(SetupJobReconsentRequest {
            root_id: f.root_id.clone(),
            job_id: before.job_id.clone(),
            expected_revision: before.revision,
            manifest_id: manifest.manifest_id.clone(),
            expected_fingerprint: manifest.fingerprint.clone(),
        })
        .expect("one exact updated consent preserves existing completed work");
    assert_eq!(job.job_id, before.job_id);
    assert_eq!(job.graph, before.graph);
    assert_eq!(job.prior_consents[0].manifest, before.manifest);
    assert_eq!(job.prior_consents[0].batches, before.batches);
    assert_eq!(job.coverage.transmission_authorizations, 2);
    assert_eq!(job.coverage.processed, vec![completed.clone()]);
    f.job = Some(job);
    let cap = f.start();
    let sent = f.read(&cap);
    assert_eq!(sent.len(), 3);
    assert!(!sent.iter().any(|s| s.summary.path == completed));
    assert!(sent.iter().any(|s| s.summary.path == changed));
    assert!(sent.iter().any(|s| s.summary.path == "added.md"));
}
#[test]
fn exact_connection_confirmation_preserves_claim_verification_and_sources() {
    let mut f = Fixture::new();
    f.prepare_default(1);
    f.finish();
    f.all_receipt();
    f.decide();
    let alex = f.saved().into_iter().find(|t| t.name == "Alex").unwrap();
    let before = f.review(&alex);
    let connection = before.topic.connections[0].clone();
    let receipt = f
        .service
        .setup_connection_confirm_preview(SetupConnectionConfirmPreviewRequest {
            root_id: f.root_id.clone(),
            connection_ids: vec![connection.id.clone()],
        })
        .expect("connections have explicit per-record confirmation");
    assert_eq!(receipt.connections, vec![connection.clone()]);
    let result = f
        .service
        .setup_decide(SetupDecisionRequest {
            root_id: f.root_id.clone(),
            receipt_id: receipt.receipt_id,
            expected_fingerprint: receipt.fingerprint,
        })
        .unwrap();
    assert_eq!(result.status, SetupDecisionStatus::Completed);
    let after = f.review(&alex);
    assert_eq!(after.topic, before.topic);
    assert_eq!(
        after
            .reviews
            .iter()
            .find(|m| m.record_id == connection.id)
            .unwrap()
            .state,
        ReviewState::ConfirmedByYou
    );
    assert!(after.topic.current_claims.iter().all(|c| {
        after
            .reviews
            .iter()
            .find(|m| m.record_id == c.revision_id)
            .unwrap()
            .state
            == ReviewState::FromNotes
    }));
    f.unchanged();
}
#[test]
fn supplemental_context_is_durable_without_claims_rules_or_duplicated_corrections() {
    let mut f = Fixture::new();
    f.single(2);
    let t = f.topics[0].clone();
    let before = f.review(&t);
    let statement = f
        .service
        .setup_context_add(UserStatementRequest {
            root_id: f.root_id.clone(),
            topic_id: t.id.clone(),
            text: "My supplemental context, not a correction".into(),
        })
        .expect("an explicit human Save records supplemental topic context");
    f.correct();
    f.decide();
    f.restart();
    let after = f.review(&t);
    assert_eq!(after.context, vec![statement]);
    assert_eq!(
        after.topic.current_claims.len(),
        before.topic.current_claims.len()
    );
    assert!(
        after
            .reviews
            .iter()
            .any(|r| r.state == ReviewState::FromNotes)
    );
    f.unchanged();
}
#[test]
fn classification_is_explicit_projection_separate_from_verification() {
    let mut f = Fixture::new();
    f.prepare_default(1);
    f.finish();
    assert!(
        f.job
            .as_ref()
            .unwrap()
            .graph
            .items
            .iter()
            .all(|item| item.assertion_kind == DraftAssertionKind::ReportedExplicit)
    );
    assert!(f.saved().is_empty());
}
#[test]
fn completed_draft_context_respects_record_and_frame_budgets() {
    let mut f = Fixture::new();
    f.prepare_default(3);
    let t = topic("Ledger", TopicKind::Project);
    for batch in 0..2 {
        let cap = f.start();
        let source = f.read(&cap)[0].citation.clone();
        let mut candidates = vec![Candidate::Topic { topic: t.clone() }];
        for n in 0..199 {
            candidates.push(Candidate::Claim {
                claim: claim(&t, &format!("field-{batch}-{n}"), "reported", &source),
            });
        }
        f.publish(cap, candidates);
    }
    let cap = f.start();
    let context = f
        .service
        .setup_draft_context(SetupDraftContextRequest { capability: cap })
        .unwrap();
    assert_eq!(
        context.topics.len() + context.claims.len() + context.connections.len(),
        200
    );
    assert!(serde_json::to_vec(&context).unwrap().len() <= 1024 * 1024);
    assert!(context.claims.iter().all(|c| {
        c.sources
            .iter()
            .all(|e| matches!(e, Evidence::Passage { .. }))
    }));
}
#[test]
fn selected_manifest_completion_retains_exclusions_without_claiming_full_vault_coverage() {
    let mut f = Fixture::new();
    f.prepare_default(100);
    f.preview(20);
    f.authorize();
    f.finish();
    let job = f.job.as_ref().unwrap();
    assert_eq!(job.coverage.selected, 100);
    assert_eq!(job.coverage.processed.len(), 100);
    assert_eq!(job.coverage.model_calls, 5);
    assert!(job.coverage.excluded.iter().any(|e| e.path == "topics"));
    println!(
        "selected={}, processed={}, excluded={}, status={:?}, complete={}",
        job.coverage.selected,
        job.coverage.processed.len(),
        job.coverage.excluded.len(),
        job.status,
        job.coverage.complete
    );
    assert!(
        job.coverage.complete,
        "all exact selected sources completed; exclusions must remain disclosed separately"
    );
    assert_eq!(job.status, SetupJobStatus::Finished);
    assert!(!f.seen.iter().any(|s| s.summary.path.starts_with("topics/")));
    f.unchanged();
}
