//! Contextual validation of accepted knowledge. Confirm and correct build the
//! same exact receipts as the rest of review; `setup_decide` applies them.
use super::review::{blank, current, finalize, pin, pin_candidate};
use super::*;
use crate::workspace::knowledge::{Caller, norm, project};

fn fresh_passages(ctx: &Context, state: &State, claim: &Claim) -> (Vec<Evidence>, bool) {
    let mut stale = false;
    let fresh = claim
        .sources
        .iter()
        .filter(|e| match e {
            Evidence::Passage { source } => {
                let ok = scopes::validate_source(ctx, state, source).is_ok();
                stale |= !ok;
                ok
            }
            Evidence::UserStatement { .. } => true,
        })
        .cloned()
        .collect();
    (fresh, stale)
}
fn preview(ctx: &Context, request: ValidationPreviewRequest) -> WResult<ValidationPreview> {
    valid_id(&request.topic_id)?;
    valid_id(&request.item_id)?;
    let mut state = load(ctx)?;
    let changed = || {
        error(
            "conflict",
            "This check changed; refresh the topic and review it again",
        )
    };
    let topic = project(ctx, &state, Caller::Human)?
        .topics
        .remove(&request.topic_id)
        .ok_or_else(|| error("invalid_input", "Topic does not exist"))?;
    let item = topic
        .validation
        .iter()
        .find(|i| i.id == request.item_id)
        .ok_or_else(changed)?;
    if let Some(connection_id) = &item.connection_id {
        let ValidationAction::Confirm { id } = &request.action else {
            return Err(error(
                "invalid_input",
                "A relationship can be confirmed or deferred; undo it in History to remove it",
            ));
        };
        if id != connection_id {
            return Err(changed());
        }
        let connection = state
            .connections
            .get(connection_id)
            .cloned()
            .ok_or_else(changed)?;
        let mut receipt = blank(ctx, SetupDecisionKind::Confirm);
        receipt.requested_ids = vec![connection.id.clone()];
        receipt.included_ids = receipt.requested_ids.clone();
        pin_candidate(
            ctx,
            &state,
            &Candidate::Connection {
                connection: connection.clone(),
            },
            &mut receipt.expected_hashes,
        )
        .map_err(|_| {
            error(
                "stale_evidence",
                "This relationship's source changed. Analyse the source again or defer the check.",
            )
        })?;
        receipt.connections.push(connection);
        receipt.operation_count = 1;
        return Ok(ValidationPreview {
            receipt: finalize(ctx, receipt, None, None, false, vec![])?,
            statement: None,
            before: vec![],
            after: vec![],
            confirmed: vec![],
        });
    }
    // Pending assertions keep their own decision; a judgment covers accepted revisions.
    let mut accepted = vec![];
    for id in &item.revision_ids {
        if state.claims.contains_key(id) {
            accepted.push(current(&state, id)?);
        }
    }
    if accepted.is_empty() || accepted.len() != item.revision_ids.len() {
        return Err(error(
            "invalid_input",
            "These assertions are still pending; decide them where they were discovered",
        ));
    }
    let resolved = match &request.action {
        ValidationAction::Confirm { id } => accepted
            .iter()
            .find(|c| &c.revision_id == id)
            .map(|c| c.value.clone())
            .ok_or_else(changed)?,
        ValidationAction::Correct { value } => {
            if value.trim().is_empty() {
                return Err(error("invalid_input", "Enter the right value"));
            }
            value.trim().to_string()
        }
    };
    let mut agreeing = vec![];
    let mut differing = vec![];
    for claim in &accepted {
        let (fresh, stale) = fresh_passages(ctx, &state, claim);
        if norm(&claim.value) == norm(&resolved) {
            agreeing.push((claim.clone(), fresh, stale));
        } else {
            differing.push(claim);
        }
    }
    if differing.is_empty() && agreeing.iter().all(|(_, _, stale)| !stale) {
        // Nothing changes: the existing statement confirmation receipt.
        let mut receipt = blank(ctx, SetupDecisionKind::Confirm);
        receipt.requested_ids = agreeing
            .iter()
            .map(|(c, ..)| c.revision_id.clone())
            .collect();
        receipt.included_ids = receipt.requested_ids.clone();
        for (claim, ..) in &agreeing {
            pin_candidate(
                ctx,
                &state,
                &Candidate::Claim {
                    claim: claim.clone(),
                },
                &mut receipt.expected_hashes,
            )?;
            receipt.revisions.push(claim.clone());
        }
        receipt.operation_count = 1;
        return Ok(ValidationPreview {
            receipt: finalize(ctx, receipt, None, None, false, vec![])?,
            statement: None,
            before: vec![],
            after: vec![],
            confirmed: agreeing.into_iter().map(|(c, ..)| c).collect(),
        });
    }
    let text = {
        let rejected: Vec<_> = differing.iter().map(|c| format!("“{}”", c.value)).collect();
        if rejected.is_empty() {
            format!(
                "I confirm {} {} is “{resolved}”.",
                topic.topic.name, item.predicate
            )
        } else {
            format!(
                "{} {} is “{resolved}”, not {}.",
                topic.topic.name,
                item.predicate,
                rejected.join(" or ")
            )
        }
    };
    let statement = UserStatement {
        id: new_id(),
        topic_id: request.topic_id.clone(),
        text,
        recorded_at: now(),
    };
    let judgment = Evidence::UserStatement {
        statement_id: statement.id.clone(),
    };
    let mut before = vec![];
    let mut after = vec![];

    let mut candidates = vec![];
    // Each rejected source assertion keeps its history and gains a superseding
    // revision that cites the human's judgment.
    for old in differing {
        let mut claim = old.clone();
        claim.revision_id = new_id();
        claim.value = resolved.clone();
        claim.uncertainty = None;
        claim.supersedes = vec![old.revision_id.clone()];
        claim.sources = vec![judgment.clone()];
        claim.recorded_at = now();
        before.push(old.clone());
        after.push(claim.clone());
        candidates.push(Candidate::Claim { claim });
    }
    // An agreeing assertion whose passage went stale is restated on the
    // human's word; its earlier revision keeps the original citation.
    let mut confirmed = vec![];
    for (old, fresh, stale) in &agreeing {
        if !stale {
            confirmed.push(old.clone());
            continue;
        }
        let mut claim = old.clone();
        claim.revision_id = new_id();
        claim.uncertainty = None;
        claim.supersedes = vec![old.revision_id.clone()];
        claim.sources = fresh.clone();
        claim.sources.push(judgment.clone());
        claim.recorded_at = now();
        before.push(old.clone());
        after.push(claim.clone());
        candidates.push(Candidate::Claim { claim });
    }
    let mut receipt = blank(ctx, SetupDecisionKind::Save);
    receipt.requested_ids = accepted.iter().map(|c| c.revision_id.clone()).collect();
    receipt.included_ids = after.iter().map(|c| c.revision_id.clone()).collect();
    receipt
        .included_ids
        .extend(confirmed.iter().map(|c| c.revision_id.clone()));
    receipt.revisions = accepted.clone();
    receipt.candidates = candidates;
    for claim in &accepted {
        for evidence in &claim.sources {
            if let Evidence::Passage { source } = evidence
                && scopes::validate_source(ctx, &state, source).is_ok()
            {
                pin(ctx, &mut receipt.expected_hashes, &source.path)?;
            }
        }
    }
    for claim in accepted.iter().chain(&after) {
        pin(
            ctx,
            &mut receipt.expected_hashes,
            &format!("topics/claims/{}.md", claim.revision_id),
        )?;
    }
    pin(
        ctx,
        &mut receipt.expected_hashes,
        &format!("topics/topics/{}.md", request.topic_id),
    )?;
    receipt.operation_count = decision_chunks(&receipt.candidates, &[], true)?.len() as u32;
    if receipt.operation_count != 1 {
        return Err(error(
            "invalid_input",
            "Too many competing assertions for one judgment; correct them individually",
        ));
    }
    // The same checks approval runs, so a judgment that cannot be saved says so now.
    state
        .statements
        .insert(statement.id.clone(), statement.clone());
    proposal::validate(ctx, &state, &receipt.candidates, None, true, false)?;
    let confirm = confirmed.iter().map(|c| c.revision_id.clone()).collect();
    Ok(ValidationPreview {
        receipt: finalize(ctx, receipt, Some(statement.clone()), None, true, confirm)?,
        statement: Some(statement),
        before,
        after,
        confirmed,
    })
}
impl WorkspaceService {
    /// Preview a human judgment on one validation item. Nothing is saved until
    /// the returned receipt is approved through `setup_decide`.
    pub fn validation_preview(
        &self,
        request: ValidationPreviewRequest,
    ) -> WResult<ValidationPreview> {
        self.with(&request.root_id, |ctx| preview(ctx, request.clone()))
    }
}
