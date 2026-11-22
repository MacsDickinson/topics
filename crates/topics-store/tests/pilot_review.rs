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
        source_count: Some(3),
        decision_count: Some(3),
        commitment_count: Some(1),
        total_minutes: Some(20),
        correction_minutes: Some(5),
        useful_change: String::new(),
        workload_comparison: PilotWorkloadComparison::Unknown,
        useful_change_judgement: PilotUsefulChange::Unknown,
        manual_commitments: Some(1),
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
                id: "pending".into(),
                description: "Pending evidence".into(),
                outcome: PilotMaterialOutcome::Omitted,
                evidence_at_cutoff: PilotEvidenceAtCutoff::PendingUnavailable,
                feedback_id: None,
            },
            PilotMaterialJudgement {
                id: "unknown".into(),
                description: "Uncertain evidence".into(),
                outcome: PilotMaterialOutcome::Wrong,
                evidence_at_cutoff: PilotEvidenceAtCutoff::Unknown,
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
    assert_eq!(
        (result.measures.pending_unavailable, result.measures.unknown),
        (1, 1)
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
    let week = PilotWeek {
        week: "2026-11-20".into(),
        project: "Fictional".into(),
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
        material_judgements: vec![PilotMaterialJudgement {
            id: "change".into(),
            description: "decision".into(),
            outcome: PilotMaterialOutcome::Omitted,
            evidence_at_cutoff: PilotEvidenceAtCutoff::Unknown,
            feedback_id: Some(first.id.clone()),
        }],
        rule_occurrences: vec![],
        largest_cost_and_next_change: String::new(),
    };
    let mut classified = first.clone();
    classified.evidence_at_cutoff = PilotEvidenceAtCutoff::Analysed;
    service
        .pilot_feedback_save(PilotFeedbackSaveRequest {
            root_id: root_id.clone(),
            feedback: classified.clone(),
        })
        .unwrap();
    let mut without_rows = week.clone();
    without_rows.material_judgements.clear();
    let initial = service
        .pilot_week_save(PilotWeekSaveRequest {
            root_id: root_id.clone(),
            week: without_rows,
        })
        .unwrap();
    assert_eq!(
        (
            initial.measures.known_omissions,
            initial.measures.eligible_changes
        ),
        (1, 1)
    );
    let mut missed = first.clone();
    missed.id = "e6c06fe9-7a61-4b3b-a60d-cc1a3d14fa57".into();
    missed.kind = PilotFeedbackKind::MissedCommitment;
    let missed = service
        .pilot_feedback_save(PilotFeedbackSaveRequest {
            root_id: root_id.clone(),
            feedback: missed,
        })
        .unwrap();
    let mut invalid_commitment = week.clone();
    invalid_commitment.material_judgements[0].feedback_id = Some(missed.id);
    assert!(
        service
            .pilot_week_save(PilotWeekSaveRequest {
                root_id: root_id.clone(),
                week: invalid_commitment
            })
            .is_err()
    );
    let mut bad = week.clone();
    bad.material_judgements[0].outcome = PilotMaterialOutcome::SurfacedCorrectly;
    assert!(
        service
            .pilot_week_save(PilotWeekSaveRequest {
                root_id: root_id.clone(),
                week: bad
            })
            .is_err()
    );
    let mut duplicate = week.clone();
    duplicate.material_judgements.push(PilotMaterialJudgement {
        id: "duplicate".into(),
        ..duplicate.material_judgements[0].clone()
    });
    assert!(
        service
            .pilot_week_save(PilotWeekSaveRequest {
                root_id: root_id.clone(),
                week: duplicate
            })
            .is_err()
    );
    service
        .pilot_week_save(PilotWeekSaveRequest {
            root_id: root_id.clone(),
            week,
        })
        .unwrap();
    let mut classified = first.clone();
    classified.evidence_at_cutoff = PilotEvidenceAtCutoff::Analysed;
    service
        .pilot_feedback_save(PilotFeedbackSaveRequest {
            root_id: root_id.clone(),
            feedback: classified,
        })
        .unwrap();
    let review = service
        .pilot_week_get(PilotWeekGetRequest {
            root_id: root_id.clone(),
            week: "2026-11-20".into(),
            project: "Fictional".into(),
        })
        .unwrap();
    assert_eq!(
        (
            review.measures.known_omissions,
            review.measures.eligible_changes
        ),
        (1, 1)
    );
    assert_eq!(
        review.week.unwrap().material_judgements[0].evidence_at_cutoff,
        PilotEvidenceAtCutoff::Analysed
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
    let mut rebuilt = WorkspaceService::open(tmp.path().join("rebuilt-app-data")).unwrap();
    let rebuilt_root = rebuilt
        .root_set(RootSetRequest {
            path: root.to_string_lossy().into(),
            create: false,
        })
        .unwrap()
        .root
        .root_id;
    assert_eq!(
        rebuilt
            .pilot_snapshot_get(PilotSnapshotGetRequest {
                root_id: rebuilt_root.clone(),
                snapshot_id: snapshot.id.clone()
            })
            .unwrap(),
        snapshot
    );
    let rebuilt_week = rebuilt
        .pilot_week_get(PilotWeekGetRequest {
            root_id: rebuilt_root,
            week: "2026-11-20".into(),
            project: "Fictional".into(),
        })
        .unwrap();
    assert_eq!(
        (
            rebuilt_week.measures.known_omissions,
            rebuilt_week.measures.eligible_changes
        ),
        (1, 1)
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
        source_count: Some(sources),
        decision_count: Some(2),
        commitment_count: Some(1),
        total_minutes: Some(minutes),
        correction_minutes: Some(1),
        useful_change: String::new(),
        useful_change_judgement: useful,
        workload_comparison: workload,
        manual_commitments: Some(0),
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

#[test]
fn missing_measurements_and_blank_workloads_do_not_claim_time_savings() {
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
    let make = |date: &str, phase, minutes| PilotWeek {
        week: date.into(),
        project: "Fiction".into(),
        phase,
        snapshot_id: None,
        source_count: Some(0),
        decision_count: Some(0),
        commitment_count: Some(0),
        total_minutes: minutes,
        correction_minutes: None,
        useful_change: String::new(),
        workload_comparison: PilotWorkloadComparison::Unknown,
        useful_change_judgement: PilotUsefulChange::Unknown,
        manual_commitments: Some(0),
        item_adjustments: vec![],
        material_judgements: vec![],
        rule_occurrences: vec![],
        largest_cost_and_next_change: String::new(),
    };
    for w in [
        make("2026-11-06", PilotPhase::Baseline, Some(30)),
        make("2026-11-13", PilotPhase::Pilot, Some(20)),
        make("2026-11-20", PilotPhase::Pilot, Some(10)),
    ] {
        service
            .pilot_week_save(PilotWeekSaveRequest {
                root_id: root_id.clone(),
                week: w,
            })
            .unwrap();
    }
    let r_root = root_id.clone();
    let r = service
        .pilot_week_get(PilotWeekGetRequest {
            root_id,
            week: "2026-11-20".into(),
            project: "Fiction".into(),
        })
        .unwrap();
    assert_eq!(
        r.comparisons.preparation_time,
        PilotComparisonStatus::Inconclusive
    );
    let variant = service
        .pilot_week_get(PilotWeekGetRequest {
            root_id: r_root.clone(),
            week: "2026-11-20".into(),
            project: "  FICTION  ".into(),
        })
        .unwrap();
    let mut saved = variant.week.unwrap();
    assert_eq!(saved.project, "Fiction");
    // A blank count comes from a partially completed desktop scorecard.
    let mut form = serde_json::to_value(&saved).unwrap();
    form["decision_count"] = serde_json::Value::Null;
    form["manual_commitments"] = serde_json::json!(0);
    service
        .pilot_week_save(PilotWeekSaveRequest {
            root_id: r_root.clone(),
            week: serde_json::from_value(form).unwrap(),
        })
        .unwrap();
    let reopened = service
        .pilot_week_get(PilotWeekGetRequest {
            root_id: r_root.clone(),
            week: saved.week.clone(),
            project: saved.project.clone(),
        })
        .unwrap();
    let reopened = serde_json::to_value(reopened.week.unwrap()).unwrap();
    assert!(reopened["decision_count"].is_null());
    assert_eq!(reopened["manual_commitments"], 0);
    saved.project = "  FICTION  ".into();
    saved.workload_comparison = PilotWorkloadComparison::Comparable;
    saved.total_minutes = None;
    let saved = service
        .pilot_week_save(PilotWeekSaveRequest {
            root_id: r_root.clone(),
            week: saved,
        })
        .unwrap();
    assert_eq!(saved.week.unwrap().project, "Fiction");
    assert_eq!(
        saved.comparisons.preparation_time,
        PilotComparisonStatus::Inconclusive
    );
}
