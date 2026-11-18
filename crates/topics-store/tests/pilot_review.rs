use topics_core::*;
use topics_store::WorkspaceService;

#[test]
fn weekly_review_survives_restart_and_counts_distinct_material_changes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("fictional");
    std::fs::create_dir(&root).unwrap();
    let app = tmp.path().join("isolated-app-data");
    let mut service = WorkspaceService::open(&app).unwrap();
    let root_id = service
        .root_set(RootSetRequest {
            path: root.to_string_lossy().into(),
            create: false,
        })
        .unwrap()
        .root
        .root_id;
    let week = PilotWeek {
        week: "2026-11-20".into(),
        project: "Fictional".into(),
        phase: PilotPhase::Baseline,
        snapshot_id: None,
        source_count: 3,
        decision_count: 3,
        commitment_count: 1,
        total_minutes: 20,
        correction_minutes: 5,
        useful_change: String::new(),
        manual_commitments: 1,
        material_judgements: vec![
            PilotMaterialJudgement {
                id: "change-a".into(),
                description: "A".into(),
                outcome: PilotMaterialOutcome::SurfacedCorrectly,
                evidence_at_cutoff: PilotEvidenceAtCutoff::Analysed,
                feedback_id: None,
            },
            PilotMaterialJudgement {
                id: "change-b".into(),
                description: "B".into(),
                outcome: PilotMaterialOutcome::Omitted,
                evidence_at_cutoff: PilotEvidenceAtCutoff::Analysed,
                feedback_id: None,
            },
            PilotMaterialJudgement {
                id: "change-c".into(),
                description: "C".into(),
                outcome: PilotMaterialOutcome::Omitted,
                evidence_at_cutoff: PilotEvidenceAtCutoff::FirstLearned,
                feedback_id: None,
            },
        ],
        rule_occurrences: vec![],
        largest_cost_and_next_change: String::new(),
    };
    service
        .pilot_week_save(PilotWeekSaveRequest {
            root_id: root_id.clone(),
            week: week.clone(),
        })
        .unwrap();
    drop(service);
    let service = WorkspaceService::open(&app).unwrap();
    let result = service
        .pilot_week_get(PilotWeekGetRequest {
            root_id,
            week: week.week.clone(),
            project: week.project.clone(),
        })
        .unwrap();
    assert_eq!(result.week, Some(week));
    assert_eq!(
        (
            result.measures.known_omissions,
            result.measures.eligible_changes,
            result.measures.first_learned
        ),
        (1, 2, 1)
    );
    assert_eq!(result.measures.draft_quality, None);
}

#[test]
fn feedback_is_journalled_deduplicated_and_keeps_its_snapshot() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("fictional");
    std::fs::create_dir(&root).unwrap();
    let app = tmp.path().join("isolated-app-data");
    let mut service = WorkspaceService::open(&app).unwrap();
    let root_id = service
        .root_set(RootSetRequest {
            path: root.to_string_lossy().into(),
            create: false,
        })
        .unwrap()
        .root
        .root_id;
    let briefing = service
        .pilot_briefing(RootRequest {
            root_id: root_id.clone(),
        })
        .unwrap();
    let snapshot = service
        .pilot_snapshot_create(PilotSnapshotCreateRequest {
            root_id: root_id.clone(),
            briefing_token: briefing.briefing_token,
        })
        .unwrap();
    let mut entry = PilotFeedback {
        id: "b9f32a06-ec79-4750-b9d7-4fc8023cabec".into(),
        snapshot_id: snapshot.id.clone(),
        item_id: None,
        kind: PilotFeedbackKind::Omitted,
        text: "A fictional decision was missed".into(),
        evidence: None,
        evidence_at_cutoff: PilotEvidenceAtCutoff::Unknown,
        recorded_at: String::new(),
        recorded_by: String::new(),
    };
    let first = service
        .pilot_feedback_save(PilotFeedbackSaveRequest {
            root_id: root_id.clone(),
            feedback: entry.clone(),
        })
        .unwrap();
    entry.text = "Updated wording".into();
    let edited = service
        .pilot_feedback_save(PilotFeedbackSaveRequest {
            root_id: root_id.clone(),
            feedback: entry,
        })
        .unwrap();
    assert_eq!(first.recorded_at, edited.recorded_at);
    drop(service);
    let service = WorkspaceService::open(&app).unwrap();
    let entries = service
        .pilot_feedback_list(PilotSnapshotGetRequest {
            root_id: root_id.clone(),
            snapshot_id: snapshot.id.clone(),
        })
        .unwrap();
    assert_eq!(entries, vec![edited]);
    assert_eq!(
        service
            .pilot_snapshot_get(PilotSnapshotGetRequest {
                root_id,
                snapshot_id: snapshot.id
            })
            .unwrap(),
        snapshot
    );
}
