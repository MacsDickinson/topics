//! Human-only Friday pilot review contracts. Feedback never approves a fact or rule.
use crate::{Id, Timestamp};
use serde::{Deserialize, Serialize};

macro_rules! value {
    ($name:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        #[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name { $(pub $field: $ty),* }
    };
}
macro_rules! choices {
    ($name:ident { $($variant:ident),* $(,)? }) => {
        #[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name { $($variant),* }
    };
}

// The digest is supplied by the displayed briefing and verified under the workspace lock.
value!(PilotSnapshotCreateRequest {
    root_id: Id,
    briefing_token: String
});
value!(PilotSnapshotGetRequest {
    root_id: Id,
    snapshot_id: Id
});
value!(PilotSnapshot {
    id: Id,
    captured_at: Timestamp,
    briefing: serde_json::Value
});
choices!(PilotFeedbackKind { Wrong, Omitted });
choices!(PilotEvidenceAtCutoff {
    Analysed,
    PendingUnavailable,
    FirstLearned,
    Unknown
});
value!(PilotFeedback {
    id: Id, snapshot_id: Id, item_id: Option<Id>, kind: PilotFeedbackKind,
    text: String, evidence: Option<String>, evidence_at_cutoff: PilotEvidenceAtCutoff,
    recorded_at: Timestamp
});
value!(PilotFeedbackSaveRequest {
    root_id: Id,
    feedback: PilotFeedback
});
choices!(PilotPhase { Baseline, Pilot });
choices!(PilotMaterialOutcome {
    SurfacedCorrectly,
    Wrong,
    Omitted
});
value!(PilotMaterialJudgement {
    id: Id, description: String, outcome: PilotMaterialOutcome,
    evidence_at_cutoff: PilotEvidenceAtCutoff, feedback_id: Option<Id>
});
value!(PilotRuleOccurrence {
    id: Id,
    rule_id: Id,
    recurred: bool
});
value!(PilotWeek {
    week: String, project: String, phase: PilotPhase, snapshot_id: Option<Id>,
    source_count: u32, decision_count: u32, commitment_count: u32,
    total_minutes: u32, correction_minutes: u32, useful_change: String,
    manual_commitments: u32, material_judgements: Vec<PilotMaterialJudgement>,
    rule_occurrences: Vec<PilotRuleOccurrence>, largest_cost_and_next_change: String
});
value!(PilotWeekGetRequest {
    root_id: Id,
    week: String,
    project: String
});
value!(PilotWeekSaveRequest {
    root_id: Id,
    week: PilotWeek
});
value!(PilotDraftMeasures {
    unchanged: u32,
    edited: u32,
    wrong: u32,
    undecided: u32
});
value!(PilotLearningMeasure {
    rule_id: Id,
    eligible: u32,
    recurrences: u32
});
value!(PilotWeekMeasures {
    wrong_items: u32, reported_misses: u32, known_omissions: u32, eligible_changes: u32,
    pending_unavailable: u32, first_learned: u32, unknown: u32,
    draft_quality: Option<PilotDraftMeasures>, learning: Vec<PilotLearningMeasure>
});
value!(PilotWeekResult { week: Option<PilotWeek>, measures: PilotWeekMeasures });
