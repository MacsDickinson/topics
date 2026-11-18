//! Capability-guarded draft publication, scoped context and human edits.
use super::graph::{issues, key, merge, validate_input, validation_state};
use super::jobs::capability;
use super::*;
impl WorkspaceService {
    pub fn setup_batch_validate(&self, request: SetupBatchStageRequest) -> WResult<()> {
        self.with(&request.capability.root_id, |ctx| {
            let (job, selected) = capability(ctx, &request.capability)?;
            validate_input(ctx, &job, &request, &selected)
        })
    }
    pub fn setup_batch_stage_guarded(
        &self,
        request: SetupBatchStageRequest,
        publisher: impl FnOnce(&Proposal, &mut dyn FnMut() -> WResult<()>) -> WResult<()>,
    ) -> WResult<SetupJob> {
        self.with(&request.capability.root_id, |ctx| {
            let (mut job, selected) = capability(ctx, &request.capability)?;
            // Consume even malformed terminal turns; the runtime owns cancellation before this action.
            revoke(ctx, &request.capability.run_id)?;
            if !request.candidates.is_empty(){validate_input(ctx, &job, &request, &selected)?;}
            let state = load(ctx)?;
            let origin = Origin::Analysis {
                run_id: request.capability.run_id.clone(),
            };
            // Review-only identity alternatives are never returned by model validation.
            let mut normalized = proposal::normalize_model_candidates(
                &state,
                &request.candidates,
                &origin,
                proposal::NormalizationMode::Review,
                None,
            )?;
            // An already accepted assertion can acquire genuinely new citations.
            // Keep that as a proposed revision/evidence change, rather than
            // dropping its sources or creating a second current fact.
            for candidate in &request.candidates {
                if let Candidate::Claim { claim } = candidate {
                    let heads: Vec<_> = state.claims.values().filter(|old| {
                        proposal::same_claim(old, claim)
                            && !state.claims.values().any(|next| next.supersedes.contains(&old.revision_id))
                    }).collect();
                    if heads.len() == 1 && claim.sources.iter().any(|source| !heads[0].sources.contains(source)) {
                        let old = heads[0];
                        let mut enriched = claim.clone();
                        enriched.id = old.id.clone();
                        enriched.supersedes = vec![old.revision_id.clone()];
                        for source in &old.sources { if !enriched.sources.contains(source) { enriched.sources.push(source.clone()); } }
                        job.graph.canonical_ids.insert(claim.id.clone(), old.id.clone());
                        normalized.retain(|candidate| !matches!(candidate, Candidate::Claim{claim:other} if other.revision_id == claim.revision_id));
                        normalized.push(Candidate::Claim { claim: enriched });
                    }
                }
            }
            // Preserve the raw-to-canonical identity seam even when accepted
            // context, rather than a previous draft batch, supplied the topic.
            for raw in &request.candidates {
                if let Candidate::Claim{claim:raw}=raw
                    && let Some(prepared)=normalized.iter().find_map(|candidate|match candidate {
                        Candidate::Claim{claim}|Candidate::Correction{claim,..} if claim.revision_id==raw.revision_id=>Some(claim),_=>None,
                    }) {
                        link(&mut job.graph.canonical_ids,&raw.subject_id,&prepared.subject_id);
                        link(&mut job.graph.canonical_ids,&raw.id,&prepared.id);
                }
            }
            let p = Proposal {
                id: new_id(),
                root_id: ctx.root_id.clone(),
                scope_id: Some(request.capability.scope_id.clone()),
                origin,
                status: ProposalStatus::Pending,
                fingerprint: proposal::fingerprint(&normalized)?,
                candidates: normalized,
                created_at: now(),
            };
            merge(
                ctx,
                &mut job,
                p.candidates.clone(),
                &p.id,
                &request.capability.batch_id,
            )?;
            let batch = job
                .batches
                .iter_mut()
                .find(|b| b.batch_id == request.capability.batch_id)
                .unwrap();
            batch.completed_source_ids=job.manifest.batches.iter().find(|b|b.batch_id==request.capability.batch_id).unwrap().sources.iter().map(|s|s.source.source_id.clone()).collect();
            batch.status = SetupBatchStatus::Completed;
            batch.proposal_ids.push(p.id.clone());
            batch.error = None;
            job.status = SetupJobStatus::Ready;
            bump(&mut job)?;
            coverage(&mut job);
            let mut saved = checkpoint(ctx, &job.job_id)?;
            let sources = job.manifest.batches.iter().find(|b| b.batch_id == request.capability.batch_id).unwrap().sources.iter().map(|s| s.source.clone()).collect();
            let mut raw = request.candidates.clone();
            proposal::remap(&mut raw, &job.graph.canonical_ids)?;
            saved.pilot_runs.push(PilotRun { at: now(), batch_id: request.capability.batch_id.clone(), sources, candidates: raw });
            saved.job = job.clone();
            saved.owner = ctx.instance_id.clone();
            let mut prepared = prepare_checkpoint(&job_path(ctx, &job.job_id)?, &saved)?;
            let mut action = || prepared.publish();
            publisher(&p, &mut action)?;
            if !prepared.published {
                return Err(error(
                    "internal",
                    "Publisher did not authorize setup checkpoint visibility",
                ));
            }
            Ok(job)
        })
    }
    pub fn setup_draft_context(
        &self,
        request: SetupDraftContextRequest,
    ) -> WResult<SetupDraftContext> {
        self.with(&request.capability.root_id, |ctx| {
            let (job, selected) = capability(ctx, &request.capability)?;
            let state = load(ctx)?;
            let context_state = validation_state(ctx, &job)?;
            let edited_ids: BTreeSet<_> = checkpoint(ctx, &job.job_id)?
                .edits
                .iter()
                .flat_map(|edit| proposal::candidate_ids(std::slice::from_ref(&edit.after)))
                .collect();
            let mut claims = vec![];
            let mut connections = vec![];
            let mut ids = BTreeSet::new();
            let mut context_bytes = 512usize;
            let mut context_records = 0usize;
            for item in &job.graph.items {
                if item.disposition == DraftDisposition::Dismissed {
                    continue;
                }
                if proposal::candidate_ids(std::slice::from_ref(&item.candidate))
                    .iter()
                    .chain(proposal::dependencies(std::slice::from_ref(&item.candidate)).iter())
                    .any(|id| edited_ids.contains(id))
                {
                    continue;
                }
                let refs = proposal::evidence_refs(&item.candidate);
                if refs.is_empty()
                    || refs.iter().any(|s| {
                        !selected.contains(&s.source_id)
                            || scopes::validate_source(ctx, &state, s).is_err()
                    })
                {
                    continue;
                }
                if proposal::validate_model_boundary(
                    ctx,
                    &context_state,
                    std::slice::from_ref(&item.candidate),
                    &selected,
                    &Origin::Analysis {
                        run_id: request.capability.run_id.clone(),
                    },
                )
                .is_err()
                {
                    continue;
                }
                let topic_ids: BTreeSet<_> = match &item.candidate {
                    Candidate::Claim { claim } => BTreeSet::from([claim.subject_id.clone()]),
                    Candidate::Connection { connection } => [&connection.from, &connection.to]
                        .iter()
                        .filter_map(|endpoint| scopes::endpoint_topic(&context_state, endpoint))
                        .collect(),
                    _ => continue,
                };
                if topic_ids
                    .iter()
                    .any(|id| scopes::topic_restricted(&context_state, id))
                {
                    continue;
                }
                let new_topics: Vec<_> = topic_ids
                    .iter()
                    .filter(|id| !ids.contains(*id))
                    .filter_map(|id| context_state.topics.get(id))
                    .collect();
                let bytes = serde_json::to_vec(&item.candidate).map_err(internal)?.len()
                    + new_topics
                        .iter()
                        .map(|topic| {
                            serde_json::to_vec(topic)
                                .map(|v| v.len() + 2)
                                .map_err(internal)
                        })
                        .collect::<WResult<Vec<_>>>()?
                        .into_iter()
                        .sum::<usize>()
                    + 2;
                if context_records + new_topics.len() + 1 > 200
                    || context_bytes + bytes > 1024 * 1024
                {
                    continue;
                }
                context_records += new_topics.len() + 1;
                context_bytes += bytes;
                match &item.candidate {
                    Candidate::Claim { claim }
                        if !scopes::topic_restricted(&context_state, &claim.subject_id) =>
                    {
                        ids.insert(claim.subject_id.clone());
                        let mut scoped = claim.clone();
                        // Draft context does not disclose accepted history outside
                        // the evidence set or promote a proposed replacement.
                        scoped.supersedes.clear();
                        claims.push(scoped)
                    }
                    Candidate::Connection { connection } => {
                        if [&connection.from, &connection.to].iter().any(|endpoint| {
                            scopes::endpoint_topic(&context_state, endpoint)
                                .is_some_and(|id| scopes::topic_restricted(&context_state, &id))
                        }) {
                            continue;
                        }
                        ids.extend(proposal::dependencies(std::slice::from_ref(
                            &item.candidate,
                        )));
                        connections.push(connection.clone())
                    }
                    _ => {}
                }
            }
            let topics = context_state
                .topics
                .values()
                .filter(|topic| {
                    ids.contains(&topic.id) && !scopes::topic_restricted(&context_state, &topic.id)
                })
                .cloned()
                .collect();
            Ok(SetupDraftContext {
                topics,
                claims,
                connections,
            })
        })
    }
    pub fn setup_draft_edit(&self, request: SetupDraftEditRequest) -> WResult<SetupJob> {
        self.with(&request.root_id, |ctx| {
            let mut job = get_job(ctx, &request.job_id)?;
            revision(&job, request.expected_revision)?;
            validate_candidate_shapes(std::slice::from_ref(&request.candidate))?;
            let statement_topic = match &request.candidate {
                Candidate::Claim { claim } => Some(claim.subject_id.clone()),
                Candidate::Connection { connection } => [&connection.from, &connection.to].iter().find_map(|e| match e {
                    Endpoint::Topic { id } => Some(id.clone()), _ => None,
                }),
                Candidate::Topic { topic } => Some(topic.id.clone()),
                _ => None,
            };
            let item = job
                .graph
                .items
                .iter_mut()
                .find(|i| i.item_id == request.item_id && i.disposition != DraftDisposition::Saved)
                .ok_or_else(|| error("invalid_input", "Editable draft item does not exist"))?;
            if key(&item.candidate) != key(&request.candidate) {
                return Err(error(
                    "invalid_input",
                    "Draft edit must preserve its record identity",
                ));
            }
            let before = item.candidate.clone();
            let mut after = request.candidate;
            if let (Candidate::Topic { topic: old }, Candidate::Topic { topic: new }) = (&before, &after)
                && old.sensitivity != new.sensitivity {
                return Err(error("forbidden", "Source roles and draft edits cannot change privacy; use explicit sensitivity review"));
            }
            // Human edits are exact statements about a specific assertion, not authority for siblings.
            let statement = if let Some(topic_id) = statement_topic {
                let s = UserStatement {
                    id: new_id(),
                    topic_id,
                    text: format!(
                        "Human draft edit from {} to {}",
                        serde_json::to_string(&before).map_err(internal)?,
                        serde_json::to_string(&after).map_err(internal)?
                    ),
                    recorded_at: now(),
                };
                if let Candidate::Claim { claim } = &mut after {
                    claim.sources.push(Evidence::UserStatement { statement_id: s.id.clone() });
                }
                Some(s)
            } else {
                None
            };
            item.candidate = after.clone();
            bump(&mut job)?;
            issues(ctx, &mut job)?;
            let mut saved = checkpoint(ctx, &job.job_id)?;
            if let Some(statement) = statement {
                saved.edits.push(DraftEdit {
                    before,
                    after,
                    statement,
                });
            }
            saved.job = job.clone();
            save_json(&job_path(ctx, &job.job_id)?, &saved)?;
            Ok(job)
        })
    }
    pub fn setup_draft_disposition(
        &self,
        request: SetupDraftDispositionRequest,
    ) -> WResult<SetupJob> {
        self.with(&request.root_id, |ctx| {
            if request.disposition == DraftDisposition::Saved {
                return Err(error(
                    "forbidden",
                    "Saving requires an exact reviewed decision receipt",
                ));
            }
            let mut job = get_job(ctx, &request.job_id)?;
            revision(&job, request.expected_revision)?;
            if request.item_ids.iter().any(|id| {
                !job.graph
                    .items
                    .iter()
                    .any(|i| &i.item_id == id && i.disposition != DraftDisposition::Saved)
            }) {
                return Err(error("invalid_input", "Select pending draft items"));
            }
            for i in &mut job.graph.items {
                if request.item_ids.contains(&i.item_id) {
                    i.disposition = request.disposition
                }
            }
            for issue in &mut job.graph.issues {
                if issue
                    .item_ids
                    .iter()
                    .any(|id| request.item_ids.contains(id))
                {
                    issue.deferred = request.disposition == DraftDisposition::Deferred;
                }
            }
            bump(&mut job)?;
            save_job(ctx, &job)?;
            Ok(job)
        })
    }
}
