#[path = "support/setup.rs"]
mod support;
use support::*;
use topics_core::*;

#[test]
fn later_independent_rule_occurrence_is_counted_once_and_snapshot_survives_undo() {
    let mut f = Fixture::new();
    f.single(1);
    let preview = f
        .service
        .setup_correction_preview(SetupCorrectionPreviewRequest {
            root_id: f.root_id.clone(),
            topic_id: f.topics[0].id.clone(),
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
    let topic = f.topics[0].clone();
    f.write(
        "later.md",
        "# Transcript\nAlex uses he/him. Morgan uses he/him.\n",
    );
    f.preview(2);
    f.authorize();
    let cap = f.start();
    let refs = f.read(&cap);
    let source = refs
        .iter()
        .find(|s| s.summary.path == "later.md")
        .unwrap()
        .citation
        .clone();
    let other = support::topic("Morgan", TopicKind::Person);
    f.publish(
        cap,
        vec![
            Candidate::Claim {
                claim: claim(&topic, "pronouns", "he/him", &source),
            },
            Candidate::Topic {
                topic: other.clone(),
            },
            Candidate::Claim {
                claim: claim(&other, "pronouns", "he/him", &source),
            },
        ],
    );
    let briefing = f.service.pilot_briefing(f.root_request()).unwrap();
    let applied = briefing
        .items
        .iter()
        .find(|item| item.rule_id.is_some())
        .unwrap();
    assert_eq!(applied.value, "they/them");
    assert_eq!(applied.sources[0].path, "later.md");
    let unrelated = briefing
        .items
        .iter()
        .find(|item| item.topic_id == other.id)
        .unwrap();
    assert_eq!(unrelated.value, "he/him");
    assert!(unrelated.rule_id.is_none());
    assert_eq!(briefing.occurrences.len(), 1);
    assert!(!briefing.occurrences[0].recurred);
    let snapshot = f
        .service
        .pilot_snapshot_create(PilotSnapshotCreateRequest {
            root_id: f.root_id.clone(),
            briefing_token: briefing.briefing_token,
        })
        .unwrap();
    let week = PilotWeek {
        week: "2026-11-20".into(),
        project: "Fictional".into(),
        phase: PilotPhase::Pilot,
        snapshot_id: Some(snapshot.id.clone()),
        source_count: 2,
        decision_count: 1,
        commitment_count: 0,
        total_minutes: Some(5),
        correction_minutes: Some(0),
        useful_change: String::new(),
        workload_comparison: PilotWorkloadComparison::Unknown,
        useful_change_judgement: PilotUsefulChange::Unknown,
        manual_commitments: 0,
        item_adjustments: vec![],
        material_judgements: vec![],
        rule_occurrences: vec![],
        largest_cost_and_next_change: String::new(),
    };
    let result = f
        .service
        .pilot_week_save(PilotWeekSaveRequest {
            root_id: f.root_id.clone(),
            week,
        })
        .unwrap();
    assert_eq!(
        (
            result.measures.learning[0].eligible,
            result.measures.learning[0].recurrences
        ),
        (1, 0)
    );
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
            .rules
            .is_empty()
    );
    assert_eq!(
        f.service
            .pilot_snapshot_get(PilotSnapshotGetRequest {
                root_id: f.root_id,
                snapshot_id: snapshot.id.clone()
            })
            .unwrap(),
        snapshot
    );
}
