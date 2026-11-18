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
fn substantive_edit(item: &PilotItem, before: &Candidate, after: &Candidate) -> bool {
    let (Candidate::Claim { claim: before }, Candidate::Claim { claim: after }) = (before, after)
    else {
        return false;
    };
    let original_sources: Vec<_> = after
        .sources
        .iter()
        .filter(|e| !matches!(e, Evidence::UserStatement { .. }))
        .collect();
    let before_sources: Vec<_> = before
        .sources
        .iter()
        .filter(|e| !matches!(e, Evidence::UserStatement { .. }))
        .collect();
    before.revision_id == item.revision_id
        && before.subject_id == item.topic_id
        && before.predicate == item.predicate
        && before.value.trim() == item.value.trim()
        && before.value.split_whitespace().collect::<Vec<_>>()
            != after.value.split_whitespace().collect::<Vec<_>>()
        && before.subject_id == after.subject_id
        && before.predicate == after.predicate
        && before.valid_from == after.valid_from
        && before.valid_to == after.valid_to
        && before.uncertainty == after.uncertainty
        && before_sources == original_sources
}
fn apply_item_adjustments(wrong: &mut BTreeSet<Id>, adjustments: &[PilotItemAdjustment]) {
    for adjustment in adjustments {
        if adjustment.wrong {
            wrong.insert(adjustment.item_id.clone());
        } else {
            wrong.remove(&adjustment.item_id);
        }
    }
}
fn add_observed_wrong(
    ctx: &Context,
    state: &State,
    week: Option<&PilotWeek>,
    result: &mut PilotWeekMeasures,
) -> WResult<()> {
    let Some(week) = week else { return Ok(()) };
    let Some(snapshot) = week
        .snapshot_id
        .as_ref()
        .and_then(|id| state.pilot_snapshots.get(id))
    else {
        return Ok(());
    };
    let edits = setup::pilot_edits(ctx)?;
    let mut wrong: BTreeSet<_> = state
        .pilot_feedback
        .values()
        .filter(|f| f.snapshot_id == snapshot.id && f.kind == PilotFeedbackKind::Wrong)
        .filter_map(|f| f.item_id.clone())
        .collect();
    for item in &snapshot.briefing.items {
        if edits.iter().any(|(before, after, at)| {
            at > &snapshot.captured_at && substantive_edit(item, before, after)
        }) {
            wrong.insert(item.item_id.clone());
        }
        if state.operations.iter().filter(|op| op.operation.created_at > snapshot.captured_at && !undone(state, &op.operation.id)).filter_map(|op| op.proposal.as_ref()).flat_map(|p| &p.applied_candidates).any(|candidate| matches!(candidate, Candidate::Correction { claim, .. } if claim.supersedes.contains(&item.revision_id) && claim.value.split_whitespace().collect::<Vec<_>>() != item.value.split_whitespace().collect::<Vec<_>>())) {
            wrong.insert(item.item_id.clone());
        }
    }
    apply_item_adjustments(&mut wrong, &week.item_adjustments);
    result.wrong_items = wrong.len() as u32;
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
        let classification = item
            .feedback_id
            .as_ref()
            .and_then(|id| feedback.get(id))
            .map_or(item.evidence_at_cutoff, |entry| entry.evidence_at_cutoff);
        match classification {
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
fn classified_week(mut week: PilotWeek, feedback: &BTreeMap<Id, PilotFeedback>) -> PilotWeek {
    for item in &mut week.material_judgements {
        if let Some(entry) = item.feedback_id.as_ref().and_then(|id| feedback.get(id)) {
            item.evidence_at_cutoff = entry.evidence_at_cutoff;
        }
    }
    week
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
        .rfind(|w| w.phase == PilotPhase::Baseline);
    let pilots: Vec<_> = weeks
        .iter()
        .copied()
        .filter(|w| w.phase == PilotPhase::Pilot && baseline.is_none_or(|b| w.week > b.week))
        .rev()
        .take(2)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
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
        let comparable = [*a, *b].iter().all(|w| match w.workload_comparison {
            PilotWorkloadComparison::Comparable => true,
            PilotWorkloadComparison::MateriallyDifferent => false,
            PilotWorkloadComparison::Unknown => {
                w.source_count == base.source_count
                    && w.decision_count == base.decision_count
                    && w.commitment_count == base.commitment_count
            }
        });
        if comparable {
            out.preparation_time =
                if a.total_minutes < base.total_minutes && b.total_minutes < base.total_minutes {
                    Pass
                } else {
                    Miss
                };
        }
        out.useful_change = if [a.useful_change_judgement, b.useful_change_judgement]
            .contains(&PilotUsefulChange::Observed)
        {
            Pass
        } else if [a.useful_change_judgement, b.useful_change_judgement]
            .iter()
            .all(|v| *v == PilotUsefulChange::None)
        {
            Miss
        } else {
            Inconclusive
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
            let week = state
                .pilot_weeks
                .get(&id)
                .cloned()
                .map(|week| classified_week(week, &state.pilot_feedback));
            let mut measures = measures(week.as_ref(), &state.pilot_feedback);
            add_observed_wrong(ctx, &state, week.as_ref(), &mut measures)?;
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
            let mut week = request.week;
            let id = week_id(&week.week, &week.project)?;
            if week.correction_minutes > week.total_minutes {
                return Err(error(
                    "invalid_input",
                    "Correction time cannot exceed total time",
                ));
            }
            unique_ids(week.material_judgements.iter().map(|j| j.id.as_str()))?;
            unique_ids(week.rule_occurrences.iter().map(|j| j.id.as_str()))?;
            unique_ids(week.item_adjustments.iter().map(|j| j.item_id.as_str()))?;
            let state = load(ctx)?;
            if week
                .snapshot_id
                .as_ref()
                .is_some_and(|id| !state.pilot_snapshots.contains_key(id))
            {
                return Err(error("invalid_input", "Snapshot does not exist"));
            }
            if !week.item_adjustments.is_empty() {
                let snapshot = week
                    .snapshot_id
                    .as_ref()
                    .and_then(|id| state.pilot_snapshots.get(id))
                    .ok_or_else(|| error("invalid_input", "Item adjustments require a snapshot"))?;
                if week.item_adjustments.iter().any(|a| {
                    !snapshot
                        .briefing
                        .items
                        .iter()
                        .any(|i| i.item_id == a.item_id)
                }) {
                    return Err(error(
                        "invalid_input",
                        "Adjustment item is not in the snapshot",
                    ));
                }
            }
            week = classified_week(week, &state.pilot_feedback);
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
            if let Some(snapshot_id) = &week.snapshot_id
                && let Some(op) = state
                    .latest_operations
                    .get(&format!("topics/pilot-snapshots/{snapshot_id}.md"))
            {
                dependencies.push(op.clone());
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
            let mut measures = measures(Some(&week), &state.pilot_feedback);
            add_observed_wrong(ctx, &state, Some(&week), &mut measures)?;
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
    fn substantive_edits_and_reports_count_one_item_and_can_be_adjusted() {
        let item = PilotItem {
            item_id: "item".into(),
            topic_id: "topic".into(),
            topic_name: "Fiction".into(),
            revision_id: "revision".into(),
            predicate: "owner".into(),
            value: "Sam".into(),
            sources: vec![],
            rule_id: None,
            original_value: None,
        };
        let claim = |value: &str| Claim {
            id: "claim".into(),
            revision_id: "revision".into(),
            subject_id: "topic".into(),
            predicate: "owner".into(),
            value: value.into(),
            uncertainty: None,
            sources: vec![],
            supersedes: vec![],
            recorded_at: "2026-01-01T00:00:00Z".into(),
            valid_from: None,
            valid_to: None,
        };
        let before = Candidate::Claim {
            claim: claim("Sam"),
        };
        let whitespace = Candidate::Claim {
            claim: claim(" Sam "),
        };
        let after = Candidate::Claim {
            claim: claim("Alex"),
        };
        assert!(!substantive_edit(&item, &before, &whitespace));
        assert!(substantive_edit(&item, &before, &after));
        let mut counted = BTreeSet::from([item.item_id.clone()]);
        counted.insert(item.item_id.clone());
        apply_item_adjustments(
            &mut counted,
            &[PilotItemAdjustment {
                item_id: item.item_id,
                wrong: false,
            }],
        );
        assert!(counted.is_empty());
    }
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
