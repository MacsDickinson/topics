#[path = "support/setup.rs"]
mod support;
use support::*;
use topics_core::*;

#[test]
fn unanalysed_local_source_is_pending_and_never_claims_a_success() {
    let mut f = Fixture::new();
    f.write("project/meeting.md", "# Synthetic meeting\nA decision.\n");
    let briefing = f.service.pilot_briefing(f.root_request()).unwrap();
    assert_eq!(briefing.sources.len(), 1);
    assert_eq!(briefing.sources[0].status, PilotSourceStatus::Pending);
    assert_eq!(briefing.sources[0].last_success_at, None);
    assert!(briefing.items.is_empty());
}

#[test]
fn successful_version_stays_historical_when_source_changes_or_disappears() {
    let mut f = Fixture::new();
    f.prepare_default(1);
    f.finish();
    let initial = f.service.pilot_briefing(f.root_request()).unwrap();
    let source = &initial.sources[0];
    assert_eq!(source.status, PilotSourceStatus::Checked);
    assert!(source.last_success_at.is_some());
    assert_eq!(source.analysed_hash, source.current_hash);
    let path = source.path.clone();
    let old_hash = source.analysed_hash.clone();
    let old_time = source.last_success_at.clone();
    std::fs::write(
        f.root.join(&path),
        "# Synthetic revision\nA different decision.\n",
    )
    .unwrap();
    let changed = f.service.pilot_briefing(f.root_request()).unwrap();
    assert_eq!(changed.sources[0].status, PilotSourceStatus::Pending);
    assert_eq!(changed.sources[0].analysed_hash, old_hash);
    assert_eq!(changed.sources[0].last_success_at, old_time);
    std::fs::remove_file(f.root.join(&path)).unwrap();
    let missing = f.service.pilot_briefing(f.root_request()).unwrap();
    assert_eq!(missing.sources[0].status, PilotSourceStatus::Unavailable);
    assert_eq!(missing.sources[0].analysed_hash, old_hash);
}

#[test]
fn unavailable_root_is_reported_at_briefing_boundary() {
    let mut f = Fixture::new();
    f.prepare_default(1);
    f.finish();
    std::fs::rename(&f.root, f.temp.path().join("offline")).unwrap();
    let briefing = f.service.pilot_briefing(f.root_request()).unwrap();
    assert_eq!(briefing.sources[0].status, PilotSourceStatus::Unavailable);
    assert!(briefing.sources[0].last_success_at.is_some());
}

#[test]
fn approved_rule_applies_only_to_later_independent_source_and_undo_stops_it() {
    let mut f = Fixture::new();
    f.single(1);
    let topic = f.topics[0].clone();
    let preview = f
        .service
        .setup_correction_preview(SetupCorrectionPreviewRequest {
            root_id: f.root_id.clone(),
            topic_id: topic.id.clone(),
            revision_ids: vec![f.claims[0].revision_id.clone()],
            predicate: "pronouns".into(),
            replacement: "they/them".into(),
            statement: "Alex uses they/them".into(),
            rule_scope: RuleScope {
                source_paths: vec![],
                context: Some("Transcript".into()),
            },
        })
        .unwrap();
    f.receipt = Some(preview.receipt);
    f.decide();
    let operation = f.result.as_ref().unwrap().operations[0].id.clone();
    f.write("later.md", "# Transcript\nAlex uses he/him.\n");
    f.preview(2);
    f.authorize();
    let cap = f.start();
    let refs = f.read(&cap);
    let source = &refs
        .iter()
        .find(|s| s.summary.path == "later.md")
        .unwrap()
        .citation;
    f.publish(
        cap,
        vec![Candidate::Claim {
            claim: claim(&topic, "pronouns", "he/him", source),
        }],
    );
    let briefing = f.service.pilot_briefing(f.root_request()).unwrap();
    let corrected: Vec<_> = briefing
        .items
        .iter()
        .filter(|i| i.rule_id.is_some())
        .collect();
    assert_eq!(corrected.len(), 1);
    assert_eq!(corrected[0].value, "they/them");
    assert_eq!(corrected[0].sources[0].path, "later.md");
    f.service
        .undo(UndoRequest {
            root_id: f.root_id.clone(),
            operation_id: operation,
        })
        .unwrap();
    assert!(
        f.service
            .pilot_briefing(f.root_request())
            .unwrap()
            .items
            .iter()
            .all(|i| i.rule_id.is_none())
    );
}

#[test]
fn failed_batch_is_incomplete_while_other_completed_source_is_checked() {
    let mut f = Fixture::new();
    f.prepare_default(2);
    f.finish_one(false);
    let cap = f.start();
    f.service
        .setup_batch_fail(SetupBatchFailRequest {
            capability: cap,
            error: ApiError::new("synthetic", "synthetic"),
        })
        .unwrap();
    let briefing = f.service.pilot_briefing(f.root_request()).unwrap();
    assert_eq!(
        briefing
            .sources
            .iter()
            .filter(|s| s.status == PilotSourceStatus::Checked)
            .count(),
        1
    );
    assert_eq!(
        briefing
            .sources
            .iter()
            .filter(|s| s.status == PilotSourceStatus::Partial)
            .count(),
        1
    );
    assert!(
        briefing
            .sources
            .iter()
            .find(|s| s.status == PilotSourceStatus::Partial)
            .unwrap()
            .last_success_at
            .is_none()
    );
}
