use super::*;
#[derive(Serialize, Deserialize)]
struct Reviewed {
    receipt: SetupReceipt,
    statement: Option<UserStatement>,
    rule: Option<Rule>,
}
fn receipt_path(ctx: &Context, id: &str) -> WResult<PathBuf> {
    valid_id(id)?;
    Ok(setup_dir(ctx).join("receipts").join(format!("{id}.json")))
}
fn record_path(c: &Candidate) -> Option<String> {
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
fn pin(ctx: &Context, hashes: &mut BTreeMap<String, Option<Hash>>, path: &str) -> WResult<()> {
    let bytes = if path.starts_with("topics/") {
        bytes_optional(&target_path(&ctx.root, path, false)?)?
    } else {
        Some(scopes::source_bytes(ctx, path)?)
    };
    hashes.insert(path.into(), bytes.as_deref().map(hash));
    Ok(())
}
fn pin_candidate(
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
fn finalize(
    ctx: &Context,
    mut r: SetupReceipt,
    statement: Option<UserStatement>,
    rule: Option<Rule>,
) -> WResult<SetupReceipt> {
    r.fingerprint = digest(&r)?;
    save_json(
        &receipt_path(ctx, &r.receipt_id)?,
        &Reviewed {
            receipt: r.clone(),
            statement,
            rule,
        },
    )?;
    Ok(r)
}
fn blank(ctx: &Context, kind: SetupDecisionKind) -> SetupReceipt {
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
        expected_hashes: BTreeMap::new(),
        fingerprint: String::new(),
        max_records_per_operation: MAX_RECORDS as u32,
        operation_count: 0,
        created_at: now(),
    }
}
fn current(state: &State, id: &str) -> WResult<Claim> {
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
fn save_preview(ctx: &Context, request: SetupSavePreviewRequest) -> WResult<SetupReceipt> {
    let mut job = get_job(ctx, &request.job_id)?;
    graph::issues(ctx, &mut job)?;
    let state = load(ctx)?;
    let requested = if request.item_ids.is_empty() {
        job.graph
            .items
            .iter()
            .filter(|i| {
                matches!(
                    i.disposition,
                    DraftDisposition::Pending | DraftDisposition::Deferred
                )
            })
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
    r.operation_count = r.candidates.len().div_ceil(30) as u32;
    finalize(ctx, r, None, None)
}
fn collected(state: &State, receipt: &SetupReceipt) -> Option<SetupDecisionResult> {
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
fn metadata_record(
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
            finalize(ctx, r, None, None)
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
            let receipt = finalize(ctx, r, Some(statement.clone()), Some(rule.clone()))?;
            Ok(SetupCorrectionPreview {
                receipt,
                statement,
                before,
                after,
                rule,
            })
        })
    }
    pub fn setup_decide(&self, request: SetupDecisionRequest) -> WResult<SetupDecisionResult> {
        self.with(&request.root_id,|ctx|{
  let reviewed:Reviewed=read_json(&receipt_path(ctx,&request.receipt_id)?)?;let r=reviewed.receipt;
  let mut check=r.clone();check.fingerprint.clear();if r.root_id!=ctx.root_id||r.fingerprint!=request.expected_fingerprint||digest(&check)?!=r.fingerprint{return Err(error("conflict","Receipt changed; refresh the exact save or confirmation preview"))}
  let state=load(ctx)?;if let Some(done)=collected(&state,&r){if done.status==SetupDecisionStatus::Completed{return Ok(done)}return Err(error("conflict","A bounded decision was partially applied; inspect its operations and refresh the remaining selection"))}
  if let Some(id)=&r.job_id{let job=get_job(ctx,id)?;revision(&job,r.job_revision.ok_or_else(||error("conflict","Missing reviewed job revision"))?)?;
   let refreshed=save_preview(ctx,SetupSavePreviewRequest{root_id:ctx.root_id.clone(),job_id:id.clone(),item_ids:r.requested_ids.clone()})?;
   if refreshed.candidates!=r.candidates||refreshed.included_ids!=r.included_ids{return Err(error("conflict","Eligible knowledge or dependency closure changed; inspect a refreshed receipt"))}
  }
  for(path,expected)in &r.expected_hashes{let mut current=BTreeMap::new();pin(ctx,&mut current,path).map_err(|_|error("conflict","Referenced source unavailable; refresh the receipt"))?;if current.get(path)!=Some(expected)||state.conflicts.contains(path){return Err(error("conflict","Source, dependency or revision changed; refresh the exact receipt before approving"))}}
  for c in &r.revisions{current(&state,&c.revision_id)?;}
  if r.included_ids.is_empty(){return Ok(SetupDecisionResult{receipt:r.clone(),status:SetupDecisionStatus::Completed,operations:vec![],saved_ids:vec![],deferred_ids:r.deferred_ids.clone(),error:None})}
  let edits=if let Some(job)=&r.job_id{checkpoint(ctx,job)?.edits}else{vec![]};
  let mut result=SetupDecisionResult{receipt:r.clone(),status:SetupDecisionStatus::Partial,operations:vec![],saved_ids:vec![],deferred_ids:r.deferred_ids.clone(),error:None};
  let total=if r.kind==SetupDecisionKind::Confirm{r.revisions.len()}else{r.candidates.len()};
  for (chunk_index,start)in(0..total).step_by(30).enumerate(){
   let state=load(ctx)?;let op=new_id();let mut records=vec![];let end=(start+30).min(total);let mut dependencies=vec![];
   if r.kind==SetupDecisionKind::Confirm{for c in &r.revisions[start..end]{records.push(metadata_record(&op,&c.revision_id,ReviewState::ConfirmedByYou,&r,None)?);dependencies.push(c.revision_id.clone());result.saved_ids.push(c.revision_id.clone());}}
   else{
    let mut validate_state=load(ctx)?;
    if let Some(s)=&reviewed.statement{validate_state.statements.insert(s.id.clone(),s.clone());records.push((format!("topics/statements/{}.md",s.id),Record::new("statement",&s.id,&op,s)?));}
    let selected=&r.candidates[start..end];
    for edit in &edits{if selected.iter().any(|c|proposal::candidate_ids(std::slice::from_ref(c)).contains(&match &edit.after{Candidate::Claim{claim}=>claim.revision_id.clone(),_=>String::new()})){validate_state.statements.insert(edit.statement.id.clone(),edit.statement.clone());records.push((format!("topics/statements/{}.md",edit.statement.id),Record::new("statement",&edit.statement.id,&op,&edit.statement)?));}}
    proposal::validate(ctx,&validate_state,selected,None,true,false)?;
    for c in selected{let path=record_path(c).ok_or_else(||error("invalid_input","Issues cannot be saved as baseline facts"))?;let id=proposal::candidate_ids(std::slice::from_ref(c))[0].clone();let(kind,data)=match c{Candidate::Topic{topic}=>("topic",json_value(topic)?),Candidate::Claim{claim}|Candidate::Correction{claim,..}=>("claim",json_value(claim)?),Candidate::Connection{connection}=>("connection",json_value(connection)?),_=>unreachable!()};records.push((path,Record::new(kind,&id,&op,data)?));
     let statement=match c{Candidate::Claim{claim}|Candidate::Correction{claim,..}=>claim.sources.iter().find_map(|e|if let Evidence::UserStatement{statement_id}=e{Some(statement_id.clone())}else{None}),_=>None};
     if !matches!(c,Candidate::Topic{..}){records.push(metadata_record(&op,&id,if statement.is_some(){ReviewState::ConfirmedByYou}else{ReviewState::FromNotes},&r,statement)?);}
     if let Candidate::Correction{rule,..}=c{records.push((format!("topics/rules/{}.md",rule.id),Record::new("rule",&rule.id,&op,rule)?));}
     dependencies.extend(proposal::dependencies(std::slice::from_ref(c)));result.saved_ids.push(id);
    }
   }
   let planned=Operation{id:op.clone(),status:"committed".into(),description:"Apply exact setup review receipt".into(),created_at:now()};result.operations.push(planned);result.status=if end==total{SetupDecisionStatus::Completed}else{SetupDecisionStatus::Partial};
   let receipt_record_id=stable_id(&format!("{}:{chunk_index}",r.receipt_id));records.push((format!("topics/receipts/{receipt_record_id}.md"),Record::new("receipt",&receipt_record_id,&op,&result)?));
   if records.len()>MAX_RECORDS{return Err(error("invalid_input","Decision operation exceeds bounded record limit; preview a smaller matching set"))}
   let mut expected=state.hashes.clone();for(path,_)in &records{expected.entry(path.clone()).or_insert(None);}
   match commit(ctx,"Apply exact setup review receipt",records,CommitIntent{dependencies,tombstones:vec![],undo_of:None,expected:Some(&expected),proposal:None}){Ok(operation)=>{*result.operations.last_mut().unwrap()=operation;},Err(e)=>{result.operations.pop();result.saved_ids.truncate(start);result.status=if result.operations.is_empty(){SetupDecisionStatus::Conflict}else{SetupDecisionStatus::Partial};result.error=Some(e);return Ok(result)}}
  }
  if let Some(id)=&r.job_id{let mut job=get_job(ctx,id)?;for i in &mut job.graph.items{if result.saved_ids.contains(&i.item_id){i.disposition=DraftDisposition::Saved;}}bump(&mut job)?;save_job(ctx,&job)?;}
  Ok(result)
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
                let mut review = state
                    .reviews
                    .get(&review_id(&c.revision_id))
                    .cloned()
                    .unwrap_or_else(|| {
                        let statement = c.sources.iter().find_map(|e| {
                            if let Evidence::UserStatement { statement_id } = e
                                && state
                                    .statements
                                    .get(statement_id)
                                    .is_some_and(|s| s.topic_id == c.subject_id)
                            {
                                Some(statement_id.clone())
                            } else {
                                None
                            }
                        });
                        ReviewMetadata {
                            id: review_id(&c.revision_id),
                            record_id: c.revision_id.clone(),
                            state: if statement.is_some() {
                                ReviewState::ConfirmedByYou
                            } else {
                                ReviewState::AcceptedUnverified
                            },
                            receipt_id: None,
                            statement_id: statement,
                            recorded_at: c.recorded_at.clone(),
                        }
                    });
                if superseded.contains(&c.revision_id) {
                    review.state = ReviewState::EarlierUnderstanding
                }
                reviews.push(review);
            }
            for c in &topic.connections {
                reviews.push(state.reviews.get(&review_id(&c.id)).cloned().unwrap_or(
                    ReviewMetadata {
                        id: review_id(&c.id),
                        record_id: c.id.clone(),
                        state: ReviewState::AcceptedUnverified,
                        receipt_id: None,
                        statement_id: None,
                        recorded_at: String::new(),
                    },
                ));
            }
            Ok(SetupTopicReview { topic, reviews })
        })
    }
}
