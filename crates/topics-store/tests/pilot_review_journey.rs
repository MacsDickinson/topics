#[path = "support/setup.rs"]
mod support;
use support::*;
use topics_core::*;

#[test]
fn meeting_snapshot_keeps_two_items_and_reports_one_catchable_miss_in_three() {
    let mut f = Fixture::new();
    f.write(
        "meeting.md",
        "# Transcript\nAlex uses he/him, leads Ledger, and the release needs approval.\n",
    );
    f.preview(1);
    f.authorize();
    let capability = f.start();
    let source = f.read(&capability)[0].citation.clone();
    let alex = topic("Alex", TopicKind::Person);
    f.publish(
        capability,
        vec![
            Candidate::Topic {
                topic: alex.clone(),
            },
            Candidate::Claim {
                claim: claim(&alex, "pronouns", "he/him", &source),
            },
            Candidate::Claim {
                claim: claim(&alex, "team", "Ledger", &source),
            },
        ],
    );
    f.all_receipt();
    f.decide();
    let displayed = f.service.pilot_briefing(f.root_request()).unwrap();
    assert_eq!(displayed.items.len(), 2);
    assert_eq!(displayed.sources[0].status, PilotSourceStatus::Checked);
    assert_eq!(
        displayed.sources[0].current_hash,
        displayed.sources[0].analysed_hash
    );
    assert!(source.excerpt.contains("release needs approval"));
    assert!(
        displayed
            .items
            .iter()
            .all(|item| !item.value.contains("release needs approval"))
    );
    let snapshot = f
        .service
        .pilot_snapshot_create(PilotSnapshotCreateRequest {
            root_id: f.root_id.clone(),
            briefing_token: displayed.briefing_token.clone(),
        })
        .unwrap();
    let feedback = |id: &str, text: &str, evidence_at_cutoff| PilotFeedback {
        id: id.into(),
        snapshot_id: snapshot.id.clone(),
        item_id: None,
        kind: PilotFeedbackKind::Omitted,
        text: text.into(),
        evidence: None,
        evidence_at_cutoff,
        recorded_at: String::new(),
        recorded_by: String::new(),
    };
    let catchable = f
        .service
        .pilot_feedback_save(PilotFeedbackSaveRequest {
            root_id: f.root_id.clone(),
            feedback: feedback(
                "3474b27e-5757-48ec-af52-0b6d370d1ead",
                "The release needs approval",
                PilotEvidenceAtCutoff::Analysed,
            ),
        })
        .unwrap();
    let new = f
        .service
        .pilot_feedback_save(PilotFeedbackSaveRequest {
            root_id: f.root_id.clone(),
            feedback: feedback(
                "5acec084-82d6-4e3f-b68c-403c9b057eff",
                "A fourth decision was first learned in the meeting",
                PilotEvidenceAtCutoff::FirstLearned,
            ),
        })
        .unwrap();
    let mut missed = feedback(
        "b7930e88-88c2-437c-9fd4-d606813b8eca",
        "A personal commitment was missed",
        PilotEvidenceAtCutoff::FirstLearned,
    );
    missed.kind = PilotFeedbackKind::MissedCommitment;
    f.service
        .pilot_feedback_save(PilotFeedbackSaveRequest {
            root_id: f.root_id.clone(),
            feedback: missed,
        })
        .unwrap();
    f.write(
        "meeting.md",
        "# Transcript\nNew minutes after the meeting.\n",
    );
    let refreshed = f.service.pilot_briefing(f.root_request()).unwrap();
    assert_ne!(
        refreshed.sources[0].current_hash,
        snapshot.briefing.sources[0].current_hash
    );
    let preserved = f
        .service
        .pilot_snapshot_get(PilotSnapshotGetRequest {
            root_id: f.root_id.clone(),
            snapshot_id: snapshot.id.clone(),
        })
        .unwrap();
    assert_eq!(preserved.briefing, displayed);
    let mut judgements: Vec<_> = snapshot
        .briefing
        .items
        .iter()
        .map(|item| PilotMaterialJudgement {
            id: item.item_id.clone(),
            description: item.value.clone(),
            outcome: PilotMaterialOutcome::SurfacedCorrectly,
            evidence_at_cutoff: PilotEvidenceAtCutoff::Analysed,
            feedback_id: None,
        })
        .collect();
    judgements.extend([
        PilotMaterialJudgement {
            id: "third".into(),
            description: catchable.text.clone(),
            outcome: PilotMaterialOutcome::Omitted,
            evidence_at_cutoff: PilotEvidenceAtCutoff::Unknown,
            feedback_id: Some(catchable.id),
        },
        PilotMaterialJudgement {
            id: "fourth".into(),
            description: new.text.clone(),
            outcome: PilotMaterialOutcome::Omitted,
            evidence_at_cutoff: PilotEvidenceAtCutoff::Unknown,
            feedback_id: Some(new.id),
        },
    ]);
    let week = PilotWeek {
        week: "2026-11-20".into(),
        project: "Fictional".into(),
        phase: PilotPhase::Pilot,
        snapshot_id: Some(snapshot.id),
        source_count: 1,
        decision_count: 4,
        commitment_count: 0,
        total_minutes: Some(20),
        correction_minutes: Some(5),
        useful_change: String::new(),
        workload_comparison: PilotWorkloadComparison::Unknown,
        useful_change_judgement: PilotUsefulChange::Unknown,
        manual_commitments: 0,
        item_adjustments: vec![],
        material_judgements: judgements,
        rule_occurrences: vec![],
        largest_cost_and_next_change: String::new(),
    };
    let saved = f
        .service
        .pilot_week_save(PilotWeekSaveRequest {
            root_id: f.root_id.clone(),
            week,
        })
        .unwrap();
    assert_eq!(
        (
            saved.measures.known_omissions,
            saved.measures.eligible_changes,
            saved.measures.first_learned
        ),
        (1, 3, 1)
    );
    assert_eq!(saved.measures.missed_commitments, 1);
}
