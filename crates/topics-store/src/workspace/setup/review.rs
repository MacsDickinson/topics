use super::*;
#[derive(Serialize, Deserialize)]
pub(super) struct Reviewed {
    pub receipt: SetupReceipt,
    pub statement: Option<UserStatement>,
    pub rule: Option<Rule>,
    // A validation judgment: every saved revision is the human's own call, and
    // `confirm` names untouched current revisions the same judgment confirms.
    #[serde(default)]
    pub judgment: bool,
    #[serde(default)]
    pub confirm: Vec<Id>,
}
pub(super) fn receipt_path(ctx: &Context, id: &str) -> WResult<PathBuf> {
    valid_id(id)?;
    Ok(setup_dir(ctx).join("receipts").join(format!("{id}.json")))
}
pub(super) fn record_path(c: &Candidate) -> Option<String> {
    match c {
        Candidate::Topic { topic } => Some(format!("topics/topics/{}.md", topic.id)),
        Candidate::Claim { claim } | Candidate::Correction { claim, .. } => {
            Some(format!("topics/claims/{}.md", claim.revision_id))
        }
        Candidate::Connection { connection } => {
            Some(format!("topics/connections/{}.md", connection.id))
        }
        _ => None,
    }
}
pub(super) fn pin(
    ctx: &Context,
    hashes: &mut BTreeMap<String, Option<Hash>>,
    path: &str,
) -> WResult<()> {
    let bytes = if path.starts_with("topics/") {
        bytes_optional(&target_path(&ctx.root, path, false)?)?
    } else {
        Some(scopes::source_bytes(ctx, path)?)
    };
    hashes.insert(path.into(), bytes.as_deref().map(hash));
    Ok(())
}
pub(super) fn pin_candidate(
    ctx: &Context,
    state: &State,
    c: &Candidate,
    hashes: &mut BTreeMap<String, Option<Hash>>,
) -> WResult<()> {
    if let Some(path) = record_path(c) {
        pin(ctx, hashes, &path)?;
    }
    for source in proposal::evidence_refs(c) {
        scopes::validate_source(ctx, state, source)?;
        pin(ctx, hashes, &source.path)?;
        pin(
            ctx,
            hashes,
            &format!("topics/sources/{}.md", source.source_id),
        )?;
    }
    let deps = proposal::dependencies(std::slice::from_ref(c));
    for path in state.hashes.keys() {
        if deps.iter().any(|id| path.ends_with(&format!("/{id}.md")))
            || state.claims.values().any(|claim| {
                deps.contains(&claim.id) && path.ends_with(&format!("/{}.md", claim.revision_id))
            })
        {
            pin(ctx, hashes, path)?;
        }
    }
    Ok(())
}
pub(super) fn finalize(
    ctx: &Context,
    mut r: SetupReceipt,
    statement: Option<UserStatement>,
    rule: Option<Rule>,
    judgment: bool,
    confirm: Vec<Id>,
) -> WResult<SetupReceipt> {
    r.fingerprint = if judgment {
        digest(&(&r, &statement, &confirm))?
    } else {
        digest(&r)?
    };
    save_json(
        &receipt_path(ctx, &r.receipt_id)?,
        &Reviewed {
            receipt: r.clone(),
            statement,
            rule,
            judgment,
            confirm,
        },
    )?;
    Ok(r)
}
pub(super) fn blank(ctx: &Context, kind: SetupDecisionKind) -> SetupReceipt {
    SetupReceipt {
        schema: SCHEMA,
        receipt_id: new_id(),
        root_id: ctx.root_id.clone(),
        kind,
        job_id: None,
        job_revision: None,
        requested_ids: vec![],
        included_ids: vec![],
        dependency_ids: vec![],
        deferred_ids: vec![],
        candidates: vec![],
        revisions: vec![],
        connections: vec![],
        expected_hashes: BTreeMap::new(),
        stale_passages: vec![],
        fingerprint: String::new(),
        max_records_per_operation: MAX_RECORDS as u32,
        operation_count: 0,
        created_at: now(),
    }
}
pub(super) fn current(state: &State, id: &str) -> WResult<Claim> {
    let c = state
        .claims
        .get(id)
        .ok_or_else(|| error("invalid_input", "Select an existing claim revision"))?;
    if state
        .claims
        .values()
        .any(|n| n.supersedes.iter().any(|r| r == id))
        || state.orphan_superseded.contains(id)
    {
        return Err(error(
            "conflict",
            "Claim has a newer revision; refresh the section before confirming",
        ));
    }
    Ok(c.clone())
}
pub(super) fn save_preview(
    ctx: &Context,
    request: SetupSavePreviewRequest,
) -> WResult<SetupReceipt> {
    let mut job = get_job(ctx, &request.job_id)?;
    graph::issues(ctx, &mut job)?;
    let state = load(ctx)?;
    let requested = if request.item_ids.is_empty() {
        job.graph
            .items
            .iter()
            .filter(|i| i.disposition == DraftDisposition::Pending)
            .map(|i| i.item_id.clone())
            .collect()
    } else {
        request.item_ids
    };
    if requested.iter().any(|id| {
        !job.graph.items.iter().any(|i| {
            &i.item_id == id
                && matches!(
                    i.disposition,
                    DraftDisposition::Pending | DraftDisposition::Deferred
                )
        })
    }) {
        return Err(error(
            "invalid_input",
            "Select existing pending or deferred draft items",
        ));
    }
    let blocked: BTreeSet<_> = job
        .graph
        .issues
        .iter()
        .flat_map(|i| i.item_ids.clone())
        .collect();
    let mut included: BTreeSet<_> = requested
        .iter()
        .filter(|id| !blocked.contains(*id))
        .cloned()
        .collect();
    let mut deferred: BTreeSet<_> = requested
        .iter()
        .filter(|id| blocked.contains(*id))
        .cloned()
        .collect();
    let mut dependencies = BTreeSet::new();
    // Fixed point dependency closure. Only endpoint records join; never sibling claims.
    loop {
        let prior = included.clone();
        for id in &prior {
            let item = job.graph.items.iter().find(|i| &i.item_id == id).unwrap();
            for dep in proposal::dependencies(std::slice::from_ref(&item.candidate)) {
                if state.topics.contains_key(&dep)
                    || state.claims.contains_key(&dep)
                    || state.claims.values().any(|c| c.id == dep)
                    || state.sources.contains_key(&dep)
                    || state.statements.contains_key(&dep)
                {
                    continue;
                }
                if let Some(required) = job.graph.items.iter().find(|i| {
                    proposal::candidate_ids(std::slice::from_ref(&i.candidate)).contains(&dep)
                        || matches!(&i.candidate,Candidate::Claim{claim} if claim.id==dep)
                }) {
                    if blocked.contains(&required.item_id)
                        || required.disposition == DraftDisposition::Dismissed
                    {
                        included.remove(id);
                        deferred.insert(id.clone());
                        break;
                    }
                    included.insert(required.item_id.clone());
                    if !requested.contains(&required.item_id) {
                        dependencies.insert(required.item_id.clone());
                    }
                } else if !matches!(&item.candidate,Candidate::Claim{claim} if claim.sources.iter().any(|e|matches!(e,Evidence::UserStatement{statement_id} if statement_id==&dep)))
                {
                    included.remove(id);
                    deferred.insert(id.clone());
                    break;
                }
            }
        }
        if included == prior {
            break;
        }
    }
    let mut candidates: Vec<_> = job
        .graph
        .items
        .iter()
        .filter(|i| included.contains(&i.item_id))
        .map(|i| i.candidate.clone())
        .collect();
    // Topics have no citation field. A bare node is eligible only as an evidenced endpoint/subject.
    let needed: BTreeSet<_> = candidates
        .iter()
        .filter(|c| !matches!(c, Candidate::Topic { .. }))
        .flat_map(|c| proposal::dependencies(std::slice::from_ref(c)))
        .collect();
    candidates.retain(|c| {
        if let Candidate::Topic { topic } = c {
            if !needed.contains(&topic.id) {
                included.remove(&topic.id);
                deferred.insert(topic.id.clone());
                return false;
            }
            true
        } else {
            true
        }
    });
    candidates.sort_by_key(|c| match c {
        Candidate::Topic { .. } => 0,
        Candidate::Claim { .. } => 1,
        Candidate::Connection { .. } => 2,
        _ => 3,
    });
    let mut r = blank(ctx, SetupDecisionKind::Save);
    r.job_id = Some(job.job_id);
    r.job_revision = Some(job.revision);
    r.requested_ids = requested;
    r.included_ids = included.into_iter().collect();
    r.dependency_ids = dependencies
        .into_iter()
        .filter(|id| r.included_ids.contains(id))
        .collect();
    r.deferred_ids = deferred.into_iter().collect();
    r.candidates = candidates;
    for c in &r.candidates {
        pin_candidate(ctx, &state, c, &mut r.expected_hashes)?;
    }
    let edits = checkpoint(ctx, r.job_id.as_ref().unwrap())?.edits;
    r.operation_count = decision_chunks(&r.candidates, &edits, false)?.len() as u32;
    finalize(ctx, r, None, None, false, vec![])
}
pub(super) fn collected(state: &State, receipt: &SetupReceipt) -> Option<SetupDecisionResult> {
    let mut parts: Vec<_> = state
        .receipts
        .values()
        .filter(|p| p.receipt.receipt_id == receipt.receipt_id)
        .cloned()
        .collect();
    if parts.is_empty() {
        return None;
    }
    parts.sort_by_key(|p| p.saved_ids.len());
    let mut result = parts.pop().unwrap();
    result.operations.retain(|op| {
        state
            .operations
            .iter()
            .any(|c| c.operation.id == op.id && !undone(state, &op.id))
    });
    for op in &mut result.operations {
        if let Some(c) = state.operations.iter().find(|c| c.operation.id == op.id) {
            *op = c.operation.clone();
        }
    }
    Some(result)
}
fn review_id(record: &str) -> Id {
    stable_id(&format!("topics-review-v1:{record}"))
}
/// The recorded human review of one claim revision, or the default for a
/// revision nobody has reviewed. Shared by topic review and the knowledge projection.
pub(in crate::workspace) fn claim_review(
    state: &State,
    claim: &Claim,
    superseded: bool,
) -> ReviewMetadata {
    let mut review = state
        .reviews
        .get(&review_id(&claim.revision_id))
        .cloned()
        .unwrap_or_else(|| {
            let statement = claim.sources.iter().find_map(|e| {
                if let Evidence::UserStatement { statement_id } = e
                    && scopes::direct_human_value_supported(state, claim)
                {
                    Some(statement_id.clone())
                } else {
                    None
                }
            });
            ReviewMetadata {
                id: review_id(&claim.revision_id),
                record_id: claim.revision_id.clone(),
                state: if statement.is_some() {
                    ReviewState::ConfirmedByYou
                } else {
                    ReviewState::AcceptedUnverified
                },
                receipt_id: None,
                statement_id: statement,
                recorded_at: claim.recorded_at.clone(),
            }
        });
    if superseded {
        review.state = ReviewState::EarlierUnderstanding
    }
    review
}
pub(in crate::workspace) fn connection_review(
    state: &State,
    connection: &Connection,
) -> ReviewMetadata {
    state
        .reviews
        .get(&review_id(&connection.id))
        .cloned()
        .unwrap_or(ReviewMetadata {
            id: review_id(&connection.id),
            record_id: connection.id.clone(),
            state: ReviewState::AcceptedUnverified,
            receipt_id: None,
            statement_id: None,
            recorded_at: String::new(),
        })
}
pub(super) fn metadata_record(
    op: &str,
    id: &str,
    state: ReviewState,
    receipt: &SetupReceipt,
    statement: Option<Id>,
) -> WResult<(String, Record)> {
    let review = ReviewMetadata {
        id: review_id(id),
        record_id: id.into(),
        state,
        receipt_id: Some(receipt.receipt_id.clone()),
        statement_id: statement,
        recorded_at: now(),
    };
    Ok((
        format!("topics/reviews/{}.md", review.id),
        Record::new("review", &review.id, op, &review)?,
    ))
}
impl WorkspaceService {
    pub fn setup_save_preview(&self, request: SetupSavePreviewRequest) -> WResult<SetupReceipt> {
        self.with(&request.root_id, |ctx| save_preview(ctx, request.clone()))
    }
    pub fn setup_confirm_preview(
        &self,
        request: SetupConfirmPreviewRequest,
    ) -> WResult<SetupReceipt> {
        self.with(&request.root_id, |ctx| {
            if request.revision_ids.is_empty() || request.revision_ids.len() > 1000 {
                return Err(error(
                    "invalid_input",
                    "Enumerate 1–1000 displayed claim revisions",
                ));
            }
            let state = load(ctx)?;
            let mut r = blank(ctx, SetupDecisionKind::Confirm);
            r.requested_ids = request
                .revision_ids
                .into_iter()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            r.included_ids = r.requested_ids.clone();
            for id in &r.requested_ids {
                let c = current(&state, id)?;
                pin_candidate(
                    ctx,
                    &state,
                    &Candidate::Claim { claim: c.clone() },
                    &mut r.expected_hashes,
                )?;
                r.revisions.push(c);
            }
            r.operation_count = r.included_ids.len().div_ceil(30) as u32;
            finalize(ctx, r, None, None, false, vec![])
        })
    }
    pub fn setup_connection_confirm_preview(
        &self,
        request: SetupConnectionConfirmPreviewRequest,
    ) -> WResult<SetupReceipt> {
        self.with(&request.root_id, |ctx| {
            if request.connection_ids.is_empty() || request.connection_ids.len() > 1000 {
                return Err(error(
                    "invalid_input",
                    "Enumerate 1–1000 displayed connections",
                ));
            }
            let state = load(ctx)?;
            let mut receipt = blank(ctx, SetupDecisionKind::Confirm);
            receipt.requested_ids = request
                .connection_ids
                .into_iter()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            receipt.included_ids = receipt.requested_ids.clone();
            for id in &receipt.requested_ids {
                valid_id(id)?;
                let connection = state
                    .connections
                    .get(id)
                    .cloned()
                    .ok_or_else(|| error("invalid_input", "Select an existing connection"))?;
                pin_candidate(
                    ctx,
                    &state,
                    &Candidate::Connection {
                        connection: connection.clone(),
                    },
                    &mut receipt.expected_hashes,
                )?;
                receipt.connections.push(connection);
            }
            receipt.operation_count = receipt.connections.len().div_ceil(30) as u32;
            finalize(ctx, receipt, None, None, false, vec![])
        })
    }
    pub fn setup_correction_preview(
        &self,
        request: SetupCorrectionPreviewRequest,
    ) -> WResult<SetupCorrectionPreview> {
        self.with(&request.root_id, |ctx| {
            if request.revision_ids.is_empty()
                || request.revision_ids.len() > 30
                || request.statement.trim().is_empty()
                || request.statement.len() > 32768
                || request.replacement.trim().is_empty()
            {
                return Err(error(
                    "invalid_input",
                    "Preview 1–30 exact revisions and your correction statement",
                ));
            }
            let state = load(ctx)?;
            valid_id(&request.topic_id)?;
            let topic = state
                .topics
                .get(&request.topic_id)
                .ok_or_else(|| error("invalid_input", "Correction topic unavailable"))?;
            if state
                .topics
                .values()
                .filter(|t| t.name.to_lowercase() == topic.name.to_lowercase())
                .count()
                > 1
                && request.rule_scope.context.is_none()
                && request.rule_scope.source_paths.is_empty()
            {
                return Err(error(
                    "conflict",
                    "Same-name identities require explicit correction source/context scope",
                ));
            }
            let mut before = vec![];
            let mut seen = BTreeSet::new();
            for id in &request.revision_ids {
                if !seen.insert(id) {
                    return Err(error("invalid_input", "Duplicate correction revision"));
                }
                let c = current(&state, id)?;
                if c.subject_id != request.topic_id || c.predicate != request.predicate {
                    return Err(error(
                        "invalid_input",
                        "Correction set contains another topic or predicate",
                    ));
                }
                before.push(c);
            }
            if before.iter().any(|c| c.value != before[0].value) {
                return Err(error(
                    "invalid_input",
                    "Preview one matching wrong value at a time",
                ));
            }
            for p in &request.rule_scope.source_paths {
                relative(p)?;
                if p.split('/').any(|c| c.starts_with('.')) || p.starts_with("topics/") {
                    return Err(error(
                        "forbidden",
                        "Rule scope must name eligible source paths",
                    ));
                }
            }
            let statement = UserStatement {
                id: new_id(),
                topic_id: request.topic_id.clone(),
                text: request.statement.clone(),
                recorded_at: now(),
            };
            let after: Vec<_> = before
                .iter()
                .map(|old| {
                    let mut c = old.clone();
                    c.revision_id = new_id();
                    c.value = request.replacement.clone();
                    c.uncertainty = None;
                    c.supersedes = vec![old.revision_id.clone()];
                    c.sources = vec![Evidence::UserStatement {
                        statement_id: statement.id.clone(),
                    }];
                    c.recorded_at = now();
                    c
                })
                .collect();
            let rule = Rule {
                id: new_id(),
                topic_id: request.topic_id,
                predicate: request.predicate,
                match_value: before[0].value.clone(),
                replacement: request.replacement,
                scope: request.rule_scope,
                source_revision_id: after[0].revision_id.clone(),
            };
            let mut r = blank(ctx, SetupDecisionKind::Save);
            r.requested_ids = before.iter().map(|c| c.revision_id.clone()).collect();
            r.included_ids = after.iter().map(|c| c.revision_id.clone()).collect();
            r.revisions = before.clone();
            r.candidates = after
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    if i == 0 {
                        Candidate::Correction {
                            claim: c.clone(),
                            rule: rule.clone(),
                        }
                    } else {
                        Candidate::Claim { claim: c.clone() }
                    }
                })
                .collect();
            for c in before.iter().chain(&after) {
                pin_candidate(
                    ctx,
                    &state,
                    &Candidate::Claim { claim: c.clone() },
                    &mut r.expected_hashes,
                )?;
            }
            r.operation_count = 1;
            validate_candidate_shapes(&r.candidates)?;
            let receipt = finalize(
                ctx,
                r,
                Some(statement.clone()),
                Some(rule.clone()),
                false,
                vec![],
            )?;
            Ok(SetupCorrectionPreview {
                receipt,
                statement,
                before,
                after,
                rule,
            })
        })
    }
    pub fn setup_topic_review(&self, request: TopicGetRequest) -> WResult<SetupTopicReview> {
        self.with(&request.root_id, |ctx| {
            let state = load(ctx)?;
            let topic = topic_view(ctx, &state, &request.topic_id)?;
            let superseded: BTreeSet<_> = topic
                .history
                .iter()
                .flat_map(|c| c.supersedes.clone())
                .collect();
            let mut reviews = vec![];
            for c in &topic.history {
                reviews.push(claim_review(&state, c, superseded.contains(&c.revision_id)));
            }
            for c in &topic.connections {
                reviews.push(connection_review(&state, c));
            }
            Ok(SetupTopicReview {
                topic,
                reviews,
                context: super::context::notes(&state, &request.topic_id),
            })
        })
    }
}
