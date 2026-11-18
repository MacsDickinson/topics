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
        workload_comparison: PilotWorkloadComparison::Unknown,
        useful_change_judgement: PilotUsefulChange::Unknown,
        manual_commitments: 1,
        item_adjustments: vec![],
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
    assert_eq!(entries, vec![edited.clone()]);
    let operation_id = service
        .history_list(RootRequest {
            root_id: root_id.clone(),
        })
        .unwrap()
        .operations
        .last()
        .unwrap()
        .id
        .clone();
    service
        .undo(UndoRequest {
            root_id: root_id.clone(),
            operation_id,
        })
        .unwrap();
    let restored = service
        .pilot_feedback_list(PilotSnapshotGetRequest {
            root_id: root_id.clone(),
            snapshot_id: snapshot.id.clone(),
        })
        .unwrap();
    assert_eq!(restored, vec![first.clone()]);
    let mut second = first.clone();
    second.id = "fa83e013-a40e-48ce-b7fd-20c16afc000c".into();
    second.text = "Another distinct miss".into();
    service
        .pilot_feedback_save(PilotFeedbackSaveRequest {
            root_id: root_id.clone(),
            feedback: second,
        })
        .unwrap();
    assert_eq!(
        service
            .pilot_feedback_list(PilotSnapshotGetRequest {
                root_id: root_id.clone(),
                snapshot_id: snapshot.id.clone()
            })
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        service
            .pilot_snapshot_get(PilotSnapshotGetRequest {
                root_id,
                snapshot_id: snapshot.id.clone()
            })
            .unwrap(),
        snapshot
    );
}

#[test]
fn comparable_workloads_and_extension_week_are_used_for_decision() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("fictional");
    std::fs::create_dir(&root).unwrap();
    let mut service = WorkspaceService::open(tmp.path().join("app")).unwrap();
    let root_id = service
        .root_set(RootSetRequest {
            path: root.to_string_lossy().into(),
            create: false,
        })
        .unwrap()
        .root
        .root_id;
    let make = |week: &str, phase, minutes, sources, workload, useful, outcome| PilotWeek {
        week: week.into(),
        project: "Fictional".into(),
        phase,
        snapshot_id: None,
        source_count: sources,
        decision_count: 2,
        commitment_count: 1,
        total_minutes: minutes,
        correction_minutes: 1,
        useful_change: String::new(),
        useful_change_judgement: useful,
        workload_comparison: workload,
        manual_commitments: 0,
        item_adjustments: vec![],
        material_judgements: vec![PilotMaterialJudgement {
            id: "a".into(),
            description: "decision".into(),
            outcome,
            evidence_at_cutoff: PilotEvidenceAtCutoff::Analysed,
            feedback_id: None,
        }],
        rule_occurrences: vec![],
        largest_cost_and_next_change: String::new(),
    };
    for w in [
        make(
            "2026-11-06",
            PilotPhase::Baseline,
            30,
            3,
            PilotWorkloadComparison::Unknown,
            PilotUsefulChange::Unknown,
            PilotMaterialOutcome::Omitted,
        ),
        make(
            "2026-11-13",
            PilotPhase::Pilot,
            40,
            3,
            PilotWorkloadComparison::Unknown,
            PilotUsefulChange::None,
            PilotMaterialOutcome::Omitted,
        ),
        make(
            "2026-11-20",
            PilotPhase::Pilot,
            20,
            4,
            PilotWorkloadComparison::Comparable,
            PilotUsefulChange::Observed,
            PilotMaterialOutcome::Omitted,
        ),
    ] {
        service
            .pilot_week_save(PilotWeekSaveRequest {
                root_id: root_id.clone(),
                week: w,
            })
            .unwrap();
    }
    let result = service
        .pilot_week_save(PilotWeekSaveRequest {
            root_id,
            week: make(
                "2026-11-27",
                PilotPhase::Pilot,
                15,
                5,
                PilotWorkloadComparison::Comparable,
                PilotUsefulChange::None,
                PilotMaterialOutcome::SurfacedCorrectly,
            ),
        })
        .unwrap();
    assert_eq!(
        result.comparisons.pilot_weeks,
        vec!["2026-11-20", "2026-11-27"]
    );
    assert_eq!(
        result.comparisons.preparation_time,
        PilotComparisonStatus::Pass
    );
    assert_eq!(
        result.comparisons.omission_trend,
        PilotComparisonStatus::Pass
    );
    assert_eq!(
        result.comparisons.useful_change,
        PilotComparisonStatus::Pass
    );
}

#[test]
fn snapshot_rejects_unobserved_token_and_old_root() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("one");
    let other = tmp.path().join("two");
    std::fs::create_dir(&root).unwrap();
    std::fs::create_dir(&other).unwrap();
    let mut service = WorkspaceService::open(tmp.path().join("app")).unwrap();
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
    assert!(
        service
            .pilot_snapshot_create(PilotSnapshotCreateRequest {
                root_id: root_id.clone(),
                briefing_token: "invented".into()
            })
            .is_err()
    );
    service
        .root_set(RootSetRequest {
            path: other.to_string_lossy().into(),
            create: false,
        })
        .unwrap();
    assert_eq!(
        service
            .pilot_snapshot_create(PilotSnapshotCreateRequest {
                root_id,
                briefing_token: briefing.briefing_token
            })
            .unwrap_err()
            .code,
        "stale_scope"
    );
}
