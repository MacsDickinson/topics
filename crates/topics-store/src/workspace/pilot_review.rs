//! Local human pilot journal. No provider or proposal authority.
use super::*;

fn week_id(week: &str, project: &str) -> WResult<Id> {
    if chrono::NaiveDate::parse_from_str(week, "%Y-%m-%d").is_err()
        || project.trim().is_empty()
        || project.len() > 512
    {
        return Err(error("invalid_input", "Provide a date and project"));
    }
    Ok(stable_id(&format!("pilot-week:{}:{}", week, project)))
}
fn unique_ids<'a>(ids: impl Iterator<Item = &'a str>) -> WResult<()> {
    let mut seen = BTreeSet::new();
    for id in ids {
        if id.trim().is_empty() || !seen.insert(id) {
            return Err(error("invalid_input", "Each entry needs a distinct ID"));
        }
    }
    Ok(())
}
fn measures(week: Option<&PilotWeek>, feedback: &BTreeMap<Id, PilotFeedback>) -> PilotWeekMeasures {
    let mut result = PilotWeekMeasures {
        wrong_items: 0,
        reported_misses: 0,
        missed_commitments: 0,
        known_omissions: 0,
        eligible_changes: 0,
        pending_unavailable: 0,
        first_learned: 0,
        unknown: 0,
        draft_quality: draft_measures(None),
        learning: vec![],
    };
    let Some(week) = week else { return result };
    let mut wrong = BTreeSet::new();
    for entry in feedback
        .values()
        .filter(|f| Some(&f.snapshot_id) == week.snapshot_id.as_ref())
    {
        match entry.kind {
            PilotFeedbackKind::Wrong => {
                if let Some(id) = &entry.item_id {
                    wrong.insert(id.clone());
                }
            }
            PilotFeedbackKind::Omitted => result.reported_misses += 1,
            PilotFeedbackKind::MissedCommitment => result.missed_commitments += 1,
        }
    }
    result.wrong_items = wrong.len() as u32;
    for item in &week.material_judgements {
        match item.evidence_at_cutoff {
            PilotEvidenceAtCutoff::Analysed => {
                result.eligible_changes += 1;
                if item.outcome == PilotMaterialOutcome::Omitted {
                    result.known_omissions += 1;
                }
            }
            PilotEvidenceAtCutoff::PendingUnavailable => result.pending_unavailable += 1,
            PilotEvidenceAtCutoff::FirstLearned => result.first_learned += 1,
            PilotEvidenceAtCutoff::Unknown => result.unknown += 1,
        }
    }
    let mut learning = BTreeMap::<Id, (u32, u32)>::new();
    for occurrence in &week.rule_occurrences {
        let counts = learning.entry(occurrence.rule_id.clone()).or_default();
        counts.0 += 1;
        counts.1 += u32::from(occurrence.recurred);
    }
    result.learning = learning
        .into_iter()
        .map(|(rule_id, (eligible, recurrences))| PilotLearningMeasure {
            rule_id,
            eligible,
            recurrences,
        })
        .collect();
    result
}
fn comparisons(state: &State, project: &str, through: &str) -> PilotComparisons {
    use PilotComparisonStatus::{Inconclusive, Miss, Pass};
    let mut weeks: Vec<_> = state
        .pilot_weeks
        .values()
        .filter(|w| w.project == project && w.week.as_str() <= through)
        .collect();
    weeks.sort_by(|a, b| a.week.cmp(&b.week));
    let baseline = weeks
        .iter()
        .copied()
        .filter(|w| w.phase == PilotPhase::Baseline)
        .next_back();
    let pilots: Vec<_> = weeks
        .iter()
        .copied()
        .filter(|w| w.phase == PilotPhase::Pilot && baseline.is_none_or(|b| w.week > b.week))
        .take(2)
        .collect();
    let mut out = PilotComparisons {
        baseline_week: baseline.map(|w| w.week.clone()),
        pilot_weeks: pilots.iter().map(|w| w.week.clone()).collect(),
        preparation_time: Inconclusive,
        useful_change: Inconclusive,
        omission_trend: Inconclusive,
        source_coverage: Inconclusive,
    };
    if let (Some(base), [a, b]) = (baseline, pilots.as_slice()) {
        let comparable = [*a, *b].iter().all(|w| {
            w.source_count == base.source_count
                && w.decision_count == base.decision_count
                && w.commitment_count == base.commitment_count
        });
        if comparable {
            out.preparation_time =
                if a.total_minutes < base.total_minutes && b.total_minutes < base.total_minutes {
                    Pass
                } else {
                    Miss
                };
        }
        out.useful_change =
            if !a.useful_change.trim().is_empty() || !b.useful_change.trim().is_empty() {
                Pass
            } else {
                Miss
            };
        let am = measures(Some(a), &state.pilot_feedback);
        let bm = measures(Some(b), &state.pilot_feedback);
        if comparable && am.eligible_changes > 0 && bm.eligible_changes > 0 {
            out.omission_trend = if u64::from(bm.known_omissions) * u64::from(am.eligible_changes)
                <= u64::from(am.known_omissions) * u64::from(bm.eligible_changes)
            {
                Pass
            } else {
                Miss
            };
        }
        let has_coverage = |w: &PilotWeek| {
            w.snapshot_id
                .as_ref()
                .and_then(|id| state.pilot_snapshots.get(id))
                .is_some_and(|s| !s.briefing.sources.is_empty())
        };
        if has_coverage(a) && has_coverage(b) {
            out.source_coverage = Pass;
        }
    }
    out
}
impl WorkspaceService {
    pub fn pilot_week_get(&self, request: PilotWeekGetRequest) -> WResult<PilotWeekResult> {
        self.with(&request.root_id, |ctx| {
            let id = week_id(&request.week, &request.project)?;
            let state = load(ctx)?;
            let week = state.pilot_weeks.get(&id).cloned();
            let measures = measures(week.as_ref(), &state.pilot_feedback);
            let comparisons = comparisons(&state, &request.project, &request.week);
            Ok(PilotWeekResult {
                week,
                measures,
                comparisons,
            })
        })
    }
    pub fn pilot_week_save(&self, request: PilotWeekSaveRequest) -> WResult<PilotWeekResult> {
        self.with(&request.root_id, |ctx| {
            let week = request.week;
            let id = week_id(&week.week, &week.project)?;
            if week.correction_minutes > week.total_minutes {
                return Err(error(
                    "invalid_input",
                    "Correction time cannot exceed total time",
                ));
            }
            unique_ids(week.material_judgements.iter().map(|j| j.id.as_str()))?;
            unique_ids(week.rule_occurrences.iter().map(|j| j.id.as_str()))?;
            let state = load(ctx)?;
            if week
                .snapshot_id
                .as_ref()
                .is_some_and(|id| !state.pilot_snapshots.contains_key(id))
            {
                return Err(error("invalid_input", "Snapshot does not exist"));
            }
            let mut referenced_feedback = BTreeSet::new();
            for judgement in &week.material_judgements {
                if let Some(feedback_id) = &judgement.feedback_id {
                    if !referenced_feedback.insert(feedback_id) {
                        return Err(error(
                            "invalid_input",
                            "A feedback entry cannot count as multiple changes",
                        ));
                    }
                    let entry = state.pilot_feedback.get(feedback_id).ok_or_else(|| {
                        error("invalid_input", "Referenced feedback does not exist")
                    })?;
                    if Some(&entry.snapshot_id) != week.snapshot_id.as_ref() {
                        return Err(error(
                            "invalid_input",
                            "Feedback belongs to another briefing",
                        ));
                    }
                }
            }
            let mut dependencies = Vec::new();
            if let Some(snapshot_id) = &week.snapshot_id {
                if let Some(op) = state
                    .latest_operations
                    .get(&format!("topics/pilot-snapshots/{snapshot_id}.md"))
                {
                    dependencies.push(op.clone());
                }
            }
            for feedback_id in referenced_feedback {
                if let Some(op) = state
                    .latest_operations
                    .get(&format!("topics/pilot-feedback/{feedback_id}.md"))
                {
                    dependencies.push(op.clone());
                }
            }
            let path = format!("topics/pilot-weeks/{id}.md");
            if state.conflicts.contains(&path) {
                return Err(error("conflict", "Resolve the damaged weekly record first"));
            }
            let operation_id = new_id();
            let record = Record::new("pilot_week", &id, &operation_id, &week)?;
            commit(
                ctx,
                "Save Friday review",
                vec![(path, record)],
                CommitIntent {
                    dependencies,
                    ..Default::default()
                },
            )?;
            let measures = measures(Some(&week), &state.pilot_feedback);
            let mut updated = state;
            updated.pilot_weeks.insert(id, week.clone());
            let comparisons = comparisons(&updated, &week.project, &week.week);
            Ok(PilotWeekResult {
                week: Some(week),
                measures,
                comparisons,
            })
        })
    }
}

impl WorkspaceService {
    pub fn pilot_snapshot_list(&self, request: RootRequest) -> WResult<Vec<PilotSnapshot>> {
        self.with(&request.root_id, |ctx| {
            let mut snapshots: Vec<_> = load(ctx)?.pilot_snapshots.into_values().collect();
            snapshots.sort_by(|a, b| b.captured_at.cmp(&a.captured_at));
            Ok(snapshots)
        })
    }
    pub fn pilot_snapshot_get(&self, request: PilotSnapshotGetRequest) -> WResult<PilotSnapshot> {
        self.with(&request.root_id, |ctx| {
            valid_id(&request.snapshot_id)?;
            load(ctx)?
                .pilot_snapshots
                .get(&request.snapshot_id)
                .cloned()
                .ok_or_else(|| error("invalid_input", "Snapshot does not exist"))
        })
    }
    pub fn pilot_feedback_list(
        &self,
        request: PilotSnapshotGetRequest,
    ) -> WResult<Vec<PilotFeedback>> {
        self.with(&request.root_id, |ctx| {
            valid_id(&request.snapshot_id)?;
            let state = load(ctx)?;
            if !state.pilot_snapshots.contains_key(&request.snapshot_id) {
                return Err(error("invalid_input", "Snapshot does not exist"));
            }
            Ok(state
                .pilot_feedback
                .into_values()
                .filter(|f| f.snapshot_id == request.snapshot_id)
                .collect())
        })
    }
    pub fn pilot_feedback_save(&self, request: PilotFeedbackSaveRequest) -> WResult<PilotFeedback> {
        self.with(&request.root_id, |ctx| {
            let mut feedback = request.feedback;
            feedback.recorded_by = "user".into();
            valid_id(&feedback.id)?;
            valid_id(&feedback.snapshot_id)?;
            if feedback.text.trim().is_empty()
                || feedback.text.len() > 8192
                || feedback.evidence.as_ref().is_some_and(|s| s.len() > 8192)
            {
                return Err(error("invalid_input", "Provide a short feedback sentence"));
            }
            let state = load(ctx)?;
            let snapshot = state
                .pilot_snapshots
                .get(&feedback.snapshot_id)
                .ok_or_else(|| error("invalid_input", "Snapshot does not exist"))?;
            match feedback.kind {
                PilotFeedbackKind::Wrong
                    if feedback.item_id.as_ref().is_none_or(|id| {
                        !snapshot
                            .briefing
                            .items
                            .iter()
                            .any(|item| &item.item_id == id)
                    }) =>
                {
                    return Err(error("invalid_input", "Choose an item in the snapshot"));
                }
                PilotFeedbackKind::Omitted | PilotFeedbackKind::MissedCommitment
                    if feedback.item_id.is_some() =>
                {
                    return Err(error(
                        "invalid_input",
                        "A miss cannot reference a briefing item",
                    ));
                }
                _ => {}
            }
            let path = format!("topics/pilot-feedback/{}.md", feedback.id);
            if state.conflicts.contains(&path) {
                return Err(error("conflict", "Resolve damaged feedback first"));
            }
            if let Some(old) = state.pilot_feedback.get(&feedback.id) {
                if old.snapshot_id != feedback.snapshot_id
                    || old.item_id != feedback.item_id
                    || old.kind != feedback.kind
                {
                    return Err(error("conflict", "Feedback identity cannot be changed"));
                }
                feedback.recorded_at = old.recorded_at.clone();
            } else {
                feedback.recorded_at = now();
            }
            let op = new_id();
            let record = Record::new("pilot_feedback", &feedback.id, &op, &feedback)?;
            let snapshot_path = format!("topics/pilot-snapshots/{}.md", feedback.snapshot_id);
            let dependencies = state
                .latest_operations
                .get(&snapshot_path)
                .cloned()
                .into_iter()
                .collect();
            commit(
                ctx,
                "Save Friday feedback",
                vec![(path, record)],
                CommitIntent {
                    dependencies,
                    ..Default::default()
                },
            )?;
            Ok(feedback)
        })
    }
}

// A future draft source can provide lifecycle events in observation order. No
// source is wired in the pilot; an absent source is not an empty set of drafts.
#[derive(Clone, Copy)]
#[allow(dead_code)]
enum DraftDisposition {
    Undecided,
    Unchanged,
    Edited,
    Wrong,
}
#[allow(dead_code)]
struct DraftObservation {
    draft_id: Id,
    disposition: DraftDisposition,
}
fn draft_measures(events: Option<&[DraftObservation]>) -> Option<PilotDraftMeasures> {
    let mut latest = BTreeMap::new();
    for event in events? {
        latest.insert(&event.draft_id, event.disposition);
    }
    let mut result = PilotDraftMeasures {
        unchanged: 0,
        edited: 0,
        wrong: 0,
        undecided: 0,
    };
    for disposition in latest.values() {
        match disposition {
            DraftDisposition::Undecided => result.undecided += 1,
            DraftDisposition::Unchanged => result.unchanged += 1,
            DraftDisposition::Edited => result.edited += 1,
            DraftDisposition::Wrong => result.wrong += 1,
        }
    }
    Some(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn enabled_drafts_use_latest_outcome_per_id() {
        let events = [
            DraftObservation {
                draft_id: "one".into(),
                disposition: DraftDisposition::Undecided,
            },
            DraftObservation {
                draft_id: "one".into(),
                disposition: DraftDisposition::Unchanged,
            },
            DraftObservation {
                draft_id: "two".into(),
                disposition: DraftDisposition::Edited,
            },
            DraftObservation {
                draft_id: "three".into(),
                disposition: DraftDisposition::Wrong,
            },
            DraftObservation {
                draft_id: "four".into(),
                disposition: DraftDisposition::Undecided,
            },
        ];
        assert_eq!(
            draft_measures(Some(&events)),
            Some(PilotDraftMeasures {
                unchanged: 1,
                edited: 1,
                wrong: 1,
                undecided: 1
            })
        );
        assert_eq!(draft_measures(None), None);
    }
}
