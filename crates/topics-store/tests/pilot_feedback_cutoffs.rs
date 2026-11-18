#[path = "support/setup.rs"]
mod support;
use support::*;
use topics_core::*;

fn omitted(snapshot: &PilotSnapshot, id: &str) -> PilotFeedback {
    PilotFeedback {
        id: id.into(),
        snapshot_id: snapshot.id.clone(),
        item_id: None,
        kind: PilotFeedbackKind::Omitted,
        text: "A fictional decision was missed".into(),
        evidence: Some("synthetic evidence".into()),
        evidence_at_cutoff: PilotEvidenceAtCutoff::Analysed,
        recorded_at: String::new(),
        recorded_by: String::new(),
    }
}
fn week(snapshot: &PilotSnapshot, date: &str) -> PilotWeek {
    PilotWeek {
        week: date.into(),
        project: "Fiction".into(),
        phase: PilotPhase::Pilot,
        snapshot_id: Some(snapshot.id.clone()),
        source_count: Some(1),
        decision_count: Some(1),
        commitment_count: Some(0),
        total_minutes: Some(10),
        correction_minutes: Some(1),
        useful_change: String::new(),
        workload_comparison: PilotWorkloadComparison::Unknown,
        useful_change_judgement: PilotUsefulChange::Unknown,
        manual_commitments: Some(0),
        item_adjustments: vec![],
        material_judgements: vec![],
        rule_occurrences: vec![],
        largest_cost_and_next_change: String::new(),
    }
}
#[test]
fn feedback_capture_does_not_advance_meeting_cutoff_and_survives_restart() {
    let mut f = Fixture::new();
    f.write("meeting.md", "# Fictional\nFirst version.\n");
    let view = f.service.pilot_briefing(f.root_request()).unwrap();
    let feedback = f
        .service
        .pilot_feedback_snapshot_create(PilotSnapshotCreateRequest {
            root_id: f.root_id.clone(),
            briefing_token: view.briefing_token,
        })
        .unwrap();
    assert!(feedback.feedback_only);
    let saved = f
        .service
        .pilot_feedback_save(PilotFeedbackSaveRequest {
            root_id: f.root_id.clone(),
            feedback: omitted(&feedback, "dbd37c4e-a19d-4e0e-9cfb-f2d6098d4d65"),
        })
        .unwrap();
    assert!(
        f.service
            .pilot_week_save(PilotWeekSaveRequest {
                root_id: f.root_id.clone(),
                week: week(&feedback, "2026-11-20")
            })
            .is_err()
    );
    let refreshed = f.service.pilot_briefing(f.root_request()).unwrap();
    assert_eq!(refreshed.since, None);
    let again = f
        .service
        .pilot_feedback_snapshot_create(PilotSnapshotCreateRequest {
            root_id: f.root_id.clone(),
            briefing_token: refreshed.briefing_token,
        })
        .unwrap();
    assert_eq!(again.id, feedback.id);
    let reopened = topics_store::WorkspaceService::open(&f.app).unwrap();
    let refreshed = reopened.pilot_briefing(f.root_request()).unwrap();
    let after_restart = reopened
        .pilot_feedback_snapshot_create(PilotSnapshotCreateRequest {
            root_id: f.root_id.clone(),
            briefing_token: refreshed.briefing_token.clone(),
        })
        .unwrap();
    assert_eq!(feedback.id, after_restart.id);
    assert_eq!(
        reopened
            .pilot_feedback_list(PilotSnapshotGetRequest {
                root_id: f.root_id.clone(),
                snapshot_id: after_restart.id.clone()
            })
            .unwrap(),
        vec![saved]
    );
    let meeting = reopened
        .pilot_snapshot_create(PilotSnapshotCreateRequest {
            root_id: f.root_id.clone(),
            briefing_token: refreshed.briefing_token,
        })
        .unwrap();
    let mut legacy = serde_json::to_value(&meeting).unwrap();
    legacy.as_object_mut().unwrap().remove("feedback_only");
    assert!(
        !serde_json::from_value::<PilotSnapshot>(legacy)
            .unwrap()
            .feedback_only
    );
    assert!(!meeting.feedback_only);
    assert_ne!(feedback.id, meeting.id);
    assert_eq!(
        reopened.pilot_briefing(f.root_request()).unwrap().since,
        Some(meeting.briefing.generated_at)
    );
}
#[test]
fn feedback_in_meeting_interval_is_visible_linkable_and_not_inherited() {
    let mut f = Fixture::new();
    f.write("meeting.md", "# Fictional\nInitial version.\n");
    let view = f.service.pilot_briefing(f.root_request()).unwrap();
    let prior = f
        .service
        .pilot_snapshot_create(PilotSnapshotCreateRequest {
            root_id: f.root_id.clone(),
            briefing_token: view.briefing_token,
        })
        .unwrap();
    f.write("meeting.md", "# Fictional\nA midweek version.\n");
    let view = f.service.pilot_briefing(f.root_request()).unwrap();
    assert_eq!(view.since, Some(prior.briefing.generated_at.clone()));
    let first = f
        .service
        .pilot_feedback_snapshot_create(PilotSnapshotCreateRequest {
            root_id: f.root_id.clone(),
            briefing_token: view.briefing_token,
        })
        .unwrap();
    let entry = f
        .service
        .pilot_feedback_save(PilotFeedbackSaveRequest {
            root_id: f.root_id.clone(),
            feedback: omitted(&first, "35c21d89-62d2-4a59-8800-594b669de0f9"),
        })
        .unwrap();
    f.write("meeting.md", "# Fictional\nLater version.\n");
    let view = f.service.pilot_briefing(f.root_request()).unwrap();
    assert_eq!(view.since, Some(prior.briefing.generated_at.clone()));
    let stale = f
        .service
        .pilot_feedback_snapshot_create(PilotSnapshotCreateRequest {
            root_id: f.root_id.clone(),
            briefing_token: first.briefing.briefing_token.clone(),
        })
        .unwrap_err();
    assert_eq!(stale.code, "stale_scope");
    let second = f
        .service
        .pilot_feedback_snapshot_create(PilotSnapshotCreateRequest {
            root_id: f.root_id.clone(),
            briefing_token: view.briefing_token.clone(),
        })
        .unwrap();
    assert_ne!(first.id, second.id);
    let meeting = f
        .service
        .pilot_snapshot_create(PilotSnapshotCreateRequest {
            root_id: f.root_id.clone(),
            briefing_token: view.briefing_token,
        })
        .unwrap();
    let list = f
        .service
        .pilot_feedback_list(PilotSnapshotGetRequest {
            root_id: f.root_id.clone(),
            snapshot_id: meeting.id.clone(),
        })
        .unwrap();
    assert_eq!(list, vec![entry.clone()]);
    let mut review = week(&meeting, "2026-11-20");
    let saved = f
        .service
        .pilot_week_save(PilotWeekSaveRequest {
            root_id: f.root_id.clone(),
            week: review.clone(),
        })
        .unwrap();
    assert_eq!(
        (
            saved.measures.reported_misses,
            saved.measures.known_omissions,
            saved.measures.eligible_changes
        ),
        (1, 1, 1)
    );
    review.material_judgements.push(PilotMaterialJudgement {
        id: "link".into(),
        description: "Fiction".into(),
        outcome: PilotMaterialOutcome::Omitted,
        evidence_at_cutoff: PilotEvidenceAtCutoff::Unknown,
        feedback_id: Some(entry.id.clone()),
    });
    let linked = f
        .service
        .pilot_week_save(PilotWeekSaveRequest {
            root_id: f.root_id.clone(),
            week: review,
        })
        .unwrap();
    assert_eq!(
        (
            linked.measures.known_omissions,
            linked.measures.eligible_changes
        ),
        (1, 1)
    );
    let view = f.service.pilot_briefing(f.root_request()).unwrap();
    let next = f
        .service
        .pilot_snapshot_create(PilotSnapshotCreateRequest {
            root_id: f.root_id.clone(),
            briefing_token: view.briefing_token,
        })
        .unwrap();
    let next_list = f
        .service
        .pilot_feedback_list(PilotSnapshotGetRequest {
            root_id: f.root_id.clone(),
            snapshot_id: next.id.clone(),
        })
        .unwrap();
    assert!(next_list.is_empty());
    let mut invalid = week(&next, "2026-11-27");
    invalid.material_judgements.push(PilotMaterialJudgement {
        id: "old".into(),
        description: "Old feedback".into(),
        outcome: PilotMaterialOutcome::Omitted,
        evidence_at_cutoff: PilotEvidenceAtCutoff::Unknown,
        feedback_id: Some(entry.id.clone()),
    });
    assert!(
        f.service
            .pilot_week_save(PilotWeekSaveRequest {
                root_id: f.root_id.clone(),
                week: invalid
            })
            .is_err()
    );
    let later = f
        .service
        .pilot_week_save(PilotWeekSaveRequest {
            root_id: f.root_id.clone(),
            week: week(&next, "2026-11-27"),
        })
        .unwrap();
    assert_eq!(later.measures.reported_misses, 0);
    assert_eq!(
        f.service
            .pilot_feedback_list(PilotSnapshotGetRequest {
                root_id: f.root_id.clone(),
                snapshot_id: first.id
            })
            .unwrap(),
        vec![entry]
    );
}
