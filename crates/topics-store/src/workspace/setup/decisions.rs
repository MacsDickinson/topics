//! Apply exactly reviewed bounded groups; Markdown markers drive replay and undo.
use super::review::{
    Reviewed, collected, current, metadata_record, pin, receipt_path, record_path, save_preview,
};
use super::*;

fn edit_matches(edit: &DraftEdit, candidate: &Candidate) -> bool {
    let ids = proposal::candidate_ids(std::slice::from_ref(candidate));
    proposal::candidate_ids(std::slice::from_ref(&edit.after))
        .iter()
        .any(|id| ids.contains(id))
}
fn changed_assertion(edit: &DraftEdit) -> bool {
    match (&edit.before, &edit.after) {
        (Candidate::Claim { claim: before }, Candidate::Claim { claim: after }) => {
            before.value != after.value
        }
        (
            Candidate::Connection { connection: before },
            Candidate::Connection { connection: after },
        ) => {
            before.from != after.from || before.relation != after.relation || before.to != after.to
        }
        _ => false,
    }
}
fn assertion_records(
    ctx: &Context,
    reviewed: &Reviewed,
    edits: &[DraftEdit],
    selected: &[Candidate],
    op: &str,
) -> WResult<Vec<(String, Record)>> {
    let receipt = &reviewed.receipt;
    let mut state = load(ctx)?;
    let mut records = vec![];
    if let Some(statement) = &reviewed.statement {
        state
            .statements
            .insert(statement.id.clone(), statement.clone());
        records.push((
            format!("topics/statements/{}.md", statement.id),
            Record::new("statement", &statement.id, op, statement)?,
        ));
    }
    for edit in edits.iter().filter(|edit| {
        selected
            .iter()
            .any(|candidate| edit_matches(edit, candidate))
    }) {
        state
            .statements
            .insert(edit.statement.id.clone(), edit.statement.clone());
        records.push((
            format!("topics/statements/{}.md", edit.statement.id),
            Record::new("statement", &edit.statement.id, op, &edit.statement)?,
        ));
    }
    proposal::validate(ctx, &state, selected, None, true, false)?;
    for candidate in selected {
        let path = record_path(candidate)
            .ok_or_else(|| error("invalid_input", "Issues cannot be saved as baseline facts"))?;
        let id = proposal::candidate_ids(std::slice::from_ref(candidate))[0].clone();
        let (kind, data) = match candidate {
            Candidate::Topic { topic } => ("topic", json_value(topic)?),
            Candidate::Claim { claim } | Candidate::Correction { claim, .. } => {
                ("claim", json_value(claim)?)
            }
            Candidate::Connection { connection } => ("connection", json_value(connection)?),
            _ => unreachable!(),
        };
        records.push((path, Record::new(kind, &id, op, data)?));
        let edit = edits
            .iter()
            .rev()
            .find(|edit| edit_matches(edit, candidate));
        let mut statement = match candidate {
            Candidate::Claim { claim } | Candidate::Correction { claim, .. } => {
                claim.sources.iter().rev().find_map(|source| {
                    if let Evidence::UserStatement { statement_id } = source {
                        Some(statement_id.clone())
                    } else {
                        None
                    }
                })
            }
            _ => edit.map(|edit| edit.statement.id.clone()),
        };
        // Merely attaching a statement to unchanged transcript content is provenance,
        // never confirmation. A correction or a value actually changed by the human
        // is the only assertion authority on this save path.
        let corrected_by_user = reviewed.statement.is_some()
            && match candidate {
                Candidate::Claim { claim } | Candidate::Correction { claim, .. } => {
                    reviewed.receipt.revisions.iter().any(|old| {
                        claim.supersedes.contains(&old.revision_id) && old.value != claim.value
                    })
                }
                _ => false,
            };
        let assertion_edit = edits.iter().rev().find(|edit| {
            edit_matches(edit, candidate)
                && changed_assertion(edit)
                && match (&edit.after, candidate) {
                    (Candidate::Claim { claim: after }, Candidate::Claim { claim: current }) => {
                        after.subject_id == current.subject_id
                            && after.predicate == current.predicate
                            && after.value == current.value
                    }
                    (
                        Candidate::Connection { connection: after },
                        Candidate::Connection {
                            connection: current,
                        },
                    ) => {
                        after.from == current.from
                            && after.to == current.to
                            && after.relation == current.relation
                    }
                    _ => false,
                }
        });
        if let Some(edit) = assertion_edit {
            statement = Some(edit.statement.id.clone());
        }
        // A validation judgment is the human's explicit call on every revision it saves.
        let confirmed = corrected_by_user
            || assertion_edit.is_some()
            || (reviewed.judgment && !matches!(candidate, Candidate::Connection { .. }));
        if !matches!(candidate, Candidate::Topic { .. }) {
            records.push(metadata_record(
                op,
                &id,
                if confirmed {
                    ReviewState::ConfirmedByYou
                } else {
                    ReviewState::FromNotes
                },
                receipt,
                statement,
            )?);
        }
        if let Candidate::Correction { rule, .. } = candidate {
            records.push((
                format!("topics/rules/{}.md", rule.id),
                Record::new("rule", &rule.id, op, rule)?,
            ));
        }
    }
    Ok(records)
}
fn decide(ctx: &Context, request: SetupDecisionRequest) -> WResult<SetupDecisionResult> {
    let reviewed: Reviewed = read_json(&receipt_path(ctx, &request.receipt_id)?)?;
    let receipt = &reviewed.receipt;
    let mut check = receipt.clone();
    check.fingerprint.clear();
    if receipt.root_id != ctx.root_id
        || receipt.fingerprint != request.expected_fingerprint
        || (if reviewed.judgment {
            digest(&(&check, &reviewed.statement, &reviewed.confirm))?
        } else {
            digest(&check)?
        }) != receipt.fingerprint
    {
        return Err(error(
            "conflict",
            "Receipt changed; refresh the exact save or confirmation preview",
        ));
    }
    let state = load(ctx)?;
    if let Some(done) = collected(&state, receipt) {
        if done.status == SetupDecisionStatus::Completed {
            return Ok(done);
        }
        return Err(error(
            "conflict",
            "A bounded decision was partially applied; inspect its operations and refresh the remaining selection",
        ));
    }
    if reviewed.judgment {
        if validation::different_periods(&receipt.revisions)
            && receipt.kind != SetupDecisionKind::SaveAllPeriods
        {
            return Err(error(
                "conflict",
                "Review the full effective periods before approving this judgment",
            ));
        }
        for evidence in receipt.revisions.iter().flat_map(|claim| &claim.sources) {
            if let Evidence::Passage { source } = evidence {
                let fresh = scopes::validate_source(ctx, &state, source).is_ok();
                let expected_stale = receipt.stale_passages.contains(source);
                if fresh == expected_stale
                    || (!expected_stale
                        && receipt.expected_hashes.get(&source.path)
                            != Some(&Some(source.document_hash.clone())))
                {
                    return Err(error(
                        "conflict",
                        "Citation availability changed; review a fresh judgment receipt",
                    ));
                }
            }
        }
    }
    if let Some(id) = &receipt.job_id {
        let job = get_job(ctx, id)?;
        revision(
            &job,
            receipt
                .job_revision
                .ok_or_else(|| error("conflict", "Missing reviewed job revision"))?,
        )?;
        let refreshed = save_preview(
            ctx,
            SetupSavePreviewRequest {
                root_id: ctx.root_id.clone(),
                job_id: id.clone(),
                item_ids: receipt.requested_ids.clone(),
            },
        )?;
        if refreshed.candidates != receipt.candidates
            || refreshed.included_ids != receipt.included_ids
        {
            return Err(error(
                "conflict",
                "Eligible knowledge or dependency closure changed; inspect a refreshed receipt",
            ));
        }
    }
    for (path, expected) in &receipt.expected_hashes {
        let mut current_hash = BTreeMap::new();
        pin(ctx, &mut current_hash, path).map_err(|_| {
            error(
                "conflict",
                "Referenced source unavailable; refresh the receipt",
            )
        })?;
        if current_hash.get(path) != Some(expected) || state.conflicts.contains(path) {
            return Err(error(
                "conflict",
                "Source, dependency or revision changed; refresh the exact receipt before approving",
            ));
        }
    }
    for claim in &receipt.revisions {
        current(&state, &claim.revision_id)?;
    }
    for connection in &receipt.connections {
        if state.connections.get(&connection.id) != Some(connection) {
            return Err(error(
                "conflict",
                "Connection changed; refresh its exact confirmation receipt",
            ));
        }
    }
    let edits = if let Some(job) = &receipt.job_id {
        checkpoint(ctx, job)?.edits
    } else {
        vec![]
    };
    let mut result = SetupDecisionResult {
        receipt: receipt.clone(),
        status: SetupDecisionStatus::Completed,
        operations: vec![],
        saved_ids: vec![],
        deferred_ids: receipt.deferred_ids.clone(),
        error: None,
    };
    let total = if receipt.kind.confirms_existing() {
        receipt.revisions.len() + receipt.connections.len()
    } else {
        receipt.candidates.len()
    };
    let chunks = if receipt.kind.confirms_existing() {
        (0..total)
            .step_by(30)
            .map(|start| start..(start + 30).min(total))
            .collect()
    } else {
        decision_chunks(&receipt.candidates, &edits, reviewed.statement.is_some())?
    };
    if chunks.len() != receipt.operation_count as usize {
        return Err(error(
            "conflict",
            "Decision operation boundaries changed; preview the receipt again",
        ));
    }
    for (chunk_index, chunk) in chunks.into_iter().enumerate() {
        let start = chunk.start;
        let state = load(ctx)?;
        let op = new_id();
        let end = chunk.end;
        let mut dependencies = vec![];
        let mut records = if receipt.kind.confirms_existing() {
            let mut records = vec![];
            for index in start..end {
                let id = if index < receipt.revisions.len() {
                    receipt.revisions[index].revision_id.clone()
                } else {
                    receipt.connections[index - receipt.revisions.len()]
                        .id
                        .clone()
                };
                records.push(metadata_record(
                    &op,
                    &id,
                    ReviewState::ConfirmedByYou,
                    receipt,
                    None,
                )?);
                dependencies.push(id.clone());
                result.saved_ids.push(id);
            }
            records
        } else {
            let selected = &receipt.candidates[start..end];
            let mut records = assertion_records(ctx, &reviewed, &edits, selected, &op)?;
            dependencies.extend(proposal::dependencies(selected));
            if reviewed.judgment && chunk_index == 0 {
                // Untouched current revisions the same judgment confirmed.
                for id in &reviewed.confirm {
                    records.push(metadata_record(
                        &op,
                        id,
                        ReviewState::ConfirmedByYou,
                        receipt,
                        reviewed.statement.as_ref().map(|s| s.id.clone()),
                    )?);
                    dependencies.push(id.clone());
                    result.saved_ids.push(id.clone());
                }
            }
            result.saved_ids.extend(selected.iter().map(|candidate| {
                proposal::candidate_ids(std::slice::from_ref(candidate))[0].clone()
            }));
            records
        };
        let planned = Operation {
            id: op.clone(),
            status: "committed".into(),
            description: "Apply exact setup review receipt".into(),
            created_at: now(),
        };
        result.operations.push(planned);
        result.status = if end == total {
            SetupDecisionStatus::Completed
        } else {
            SetupDecisionStatus::Partial
        };
        let record_id = stable_id(&format!("{}:{chunk_index}", receipt.receipt_id));
        records.push((
            format!("topics/receipts/{record_id}.md"),
            Record::new("receipt", &record_id, &op, &result)?,
        ));
        if records.len() > MAX_RECORDS {
            return Err(error(
                "invalid_input",
                "Decision operation exceeds bounded record limit; preview a smaller matching set",
            ));
        }
        let mut expected = state.hashes.clone();
        for (path, _) in &records {
            expected.entry(path.clone()).or_insert(None);
        }
        match commit(
            ctx,
            "Apply exact setup review receipt",
            records,
            CommitIntent {
                dependencies,
                tombstones: vec![],
                undo_of: None,
                expected: Some(&expected),
                proposal: None,
            },
        ) {
            Ok(operation) => *result.operations.last_mut().unwrap() = operation,
            Err(error) => {
                result.operations.pop();
                result.saved_ids.truncate(start);
                result.status = if result.operations.is_empty() {
                    SetupDecisionStatus::Conflict
                } else {
                    SetupDecisionStatus::Partial
                };
                result.error = Some(error);
                return Ok(result);
            }
        }
    }
    if let Some(id) = &receipt.job_id {
        // get_job reconstructs acceptance from committed Markdown receipts, even
        // after a crash before a disposable checkpoint update.
        get_job(ctx, id)?;
    }
    Ok(result)
}
impl WorkspaceService {
    pub fn setup_decide(&self, request: SetupDecisionRequest) -> WResult<SetupDecisionResult> {
        self.with(&request.root_id, |ctx| decide(ctx, request.clone()))
    }
}
