use super::*;
pub(super) fn key(c: &Candidate) -> Id {
    match c {
        Candidate::Topic { topic } => topic.id.clone(),
        Candidate::Claim { claim } | Candidate::Correction { claim, .. } => {
            claim.revision_id.clone()
        }
        Candidate::Connection { connection } => connection.id.clone(),
        Candidate::UnresolvedIdentity { name, .. } => stable_id(&format!("setup-identity:{name}")),
    }
}
fn candidate_topics(candidates: &[Candidate]) -> BTreeMap<Id, Topic> {
    candidates
        .iter()
        .filter_map(|c| {
            if let Candidate::Topic { topic } = c {
                Some((topic.id.clone(), topic.clone()))
            } else {
                None
            }
        })
        .collect()
}
fn normalized_name(s: &str) -> String {
    s.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
fn corroborates(
    new: &Topic,
    old: &Topic,
    input: &[Candidate],
    draft: &[DraftItem],
    topics: &BTreeMap<Id, Topic>,
) -> bool {
    input.iter().any(|c|match c{
  Candidate::Claim{claim} if claim.subject_id==new.id=>draft.iter().any(|i|matches!(&i.candidate,Candidate::Claim{claim:o} if o.subject_id==old.id&&o.predicate==claim.predicate&&o.value==claim.value)),
  Candidate::Connection{connection:c}=>draft.iter().any(|i|if let Candidate::Connection{connection:o}=&i.candidate{
   let ep=|e:&Endpoint,subject:&Id|match e{Endpoint::Topic{id} if id==subject=>Some("self".to_string()),Endpoint::Topic{id}=>topics.get(id).map(|t|format!("{:?}:{}",t.topic_kind,normalized_name(&t.name))),_=>None};
   c.relation==o.relation&&ep(&c.from,&new.id).is_some()&&ep(&c.to,&new.id).is_some()&&ep(&c.from,&new.id)==ep(&o.from,&old.id)&&ep(&c.to,&new.id)==ep(&o.to,&old.id)
  }else{false}),_=>false})
}
pub(super) fn validation_state(ctx: &Context, job: &SetupJob) -> WResult<State> {
    let mut state = load(ctx)?;
    for item in &job.graph.items {
        if item.disposition != DraftDisposition::Dismissed
            && let Candidate::Topic { topic } = &item.candidate
        {
            state
                .topics
                .entry(topic.id.clone())
                .or_insert(topic.clone());
        }
    }
    Ok(state)
}
pub(super) fn validate_input(
    ctx: &Context,
    job: &SetupJob,
    request: &SetupBatchStageRequest,
    selected: &BTreeSet<Id>,
) -> WResult<()> {
    if request.candidates.len() > job.manifest.limits.max_candidates_per_turn as usize {
        return Err(error(
            "invalid_input",
            "Candidate payload exceeds reviewed turn budget",
        ));
    }
    let state = validation_state(ctx, job)?;
    let origin = Origin::Analysis {
        run_id: request.capability.run_id.clone(),
    };
    proposal::validate_model_boundary(ctx, &state, &request.candidates, selected, &origin)?;
    for candidate in &request.candidates {
        let ids = proposal::candidate_ids(std::slice::from_ref(candidate));
        for existing in &job.graph.items {
            if proposal::candidate_ids(std::slice::from_ref(&existing.candidate))
                .iter()
                .any(|id| ids.contains(id))
                && !equivalent(&existing.candidate, candidate)
            {
                return Err(error(
                    "invalid_input",
                    "A draft identity was reused for different content; propose a fresh revision",
                ));
            }
        }
    }
    let input = proposal::normalize_model_candidates(
        &state,
        &request.candidates,
        &origin,
        proposal::NormalizationMode::Input,
        None,
    )?;
    if !input.is_empty() {
        proposal::validate(ctx, &state, &input, Some(selected), false, false)?;
    }
    Ok(())
}
fn merge_sources(target: &mut Candidate, incoming: &Candidate) {
    match (target, incoming) {
        (Candidate::Claim { claim: a }, Candidate::Claim { claim: b }) => {
            if a.uncertainty.is_none() {
                a.uncertainty.clone_from(&b.uncertainty);
            }
            for e in &b.sources {
                if !a.sources.contains(e) {
                    a.sources.push(e.clone())
                }
            }
        }
        (Candidate::Connection { connection: a }, Candidate::Connection { connection: b }) => {
            if a.uncertainty.is_none() {
                a.uncertainty.clone_from(&b.uncertainty);
            }
            for s in &b.sources {
                if !a.sources.contains(s) {
                    a.sources.push(s.clone())
                }
            }
        }
        (
            Candidate::UnresolvedIdentity {
                sources: a,
                candidates: ids,
                ..
            },
            Candidate::UnresolvedIdentity {
                sources: b,
                candidates: others,
                ..
            },
        ) => {
            for s in b {
                if !a.contains(s) {
                    a.push(s.clone())
                }
            }
            for id in others {
                if !ids.contains(id) {
                    ids.push(id.clone())
                }
            }
        }
        _ => {}
    }
}
fn equivalent(a: &Candidate, b: &Candidate) -> bool {
    match (a, b) {
        (Candidate::Topic { topic: a }, Candidate::Topic { topic: b }) => a.id == b.id,
        (Candidate::Claim { claim: a }, Candidate::Claim { claim: b }) => {
            proposal::same_claim(a, b) && a.supersedes == b.supersedes
        }
        (Candidate::Connection { connection: a }, Candidate::Connection { connection: b }) => {
            a.from == b.from
                && a.to == b.to
                && a.relation == b.relation
                && a.valid_from == b.valid_from
                && a.valid_to == b.valid_to
        }
        (
            Candidate::UnresolvedIdentity { name: a, .. },
            Candidate::UnresolvedIdentity { name: b, .. },
        ) => normalized_name(a) == normalized_name(b),
        _ => false,
    }
}
pub(super) fn merge(
    ctx: &Context,
    job: &mut SetupJob,
    mut candidates: Vec<Candidate>,
    proposal_id: &str,
    batch_id: &str,
) -> WResult<()> {
    let mut topics = candidate_topics(&candidates);
    for i in &job.graph.items {
        if let Candidate::Topic { topic } = &i.candidate {
            topics.insert(topic.id.clone(), topic.clone());
        }
    }
    let mut mapping = job.graph.canonical_ids.clone();
    for c in &candidates {
        if let Candidate::Topic { topic } = c {
            let matches: Vec<_> = job
                .graph
                .items
                .iter()
                .filter_map(|i| {
                    if i.disposition != DraftDisposition::Dismissed
                        && let Candidate::Topic { topic: old } = &i.candidate
                        && old.id != topic.id
                        && old.topic_kind == topic.topic_kind
                        && old.sensitivity != Sensitivity::LocalOnly
                        && normalized_name(&old.name) == normalized_name(&topic.name)
                    {
                        Some(old)
                    } else {
                        None
                    }
                })
                .collect();
            if matches.len() == 1
                && corroborates(topic, matches[0], &candidates, &job.graph.items, &topics)
            {
                mapping.insert(topic.id.clone(), matches[0].id.clone());
            }
        }
    }
    proposal::remap(&mut candidates, &mapping)?;
    job.graph.canonical_ids = mapping;
    let state = load(ctx)?;
    for mut c in candidates {
        if let Candidate::Topic { topic } = &mut c
            && topic.sensitivity == Sensitivity::CloudAllowed
        {
            topic.sensitivity = Sensitivity::Unknown;
        }
        // Accepted correction rules suppress exact bad candidates, never create replacements.
        if matches!(&c,Candidate::Claim{claim} if state.rules.values().any(|r|proposal::rule_matches(r,claim)))
        {
            continue;
        }
        if let Some(existing) =
            job.graph.items.iter_mut().find(|i| {
                i.disposition != DraftDisposition::Dismissed && equivalent(&i.candidate, &c)
            })
        {
            let old_ids = proposal::candidate_ids(std::slice::from_ref(&c));
            let kept = proposal::candidate_ids(std::slice::from_ref(&existing.candidate));
            for (a, b) in old_ids.into_iter().zip(kept) {
                job.graph.canonical_ids.insert(a, b);
            }
            merge_sources(&mut existing.candidate, &c);
            if !existing.batch_ids.contains(&batch_id.to_string()) {
                existing.batch_ids.push(batch_id.into())
            }
            if !existing.proposal_ids.contains(&proposal_id.to_string()) {
                existing.proposal_ids.push(proposal_id.into())
            }
        } else {
            let id = key(&c);
            job.graph.items.push(DraftItem {
                item_id: id,
                candidate: c,
                disposition: DraftDisposition::Pending,
                batch_ids: vec![batch_id.into()],
                proposal_ids: vec![proposal_id.into()],
                independent_evidence_groups: vec![],
                assertion_kind: DraftAssertionKind::Unclassified,
            });
        }
    }
    let mapping = job.graph.canonical_ids.clone();
    for item in &mut job.graph.items {
        proposal::remap(std::slice::from_mut(&mut item.candidate), &mapping)?;
        item.independent_evidence_groups = proposal::evidence_refs(&item.candidate)
            .iter()
            .map(|s| s.passage_hash.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
    }
    issues(ctx, job)?;
    Ok(())
}
pub(super) fn issues(ctx: &Context, job: &mut SetupJob) -> WResult<()> {
    let state = load(ctx)?;
    let deferred: BTreeSet<_> = job
        .graph
        .issues
        .iter()
        .filter(|i| i.deferred)
        .map(|i| i.id.clone())
        .collect();
    let mut groups: BTreeMap<String, SetupIssue> = BTreeMap::new();
    let claims: Vec<_> = job
        .graph
        .items
        .iter()
        .filter_map(|i| {
            if let Candidate::Claim { claim } = &i.candidate {
                Some((i.item_id.clone(), claim))
            } else {
                None
            }
        })
        .collect();
    let mut ambiguous = BTreeSet::new();
    let mut names: BTreeMap<String, Vec<Id>> = BTreeMap::new();
    for item in &job.graph.items {
        if let Candidate::Topic { topic } = &item.candidate {
            names
                .entry(normalized_name(&topic.name))
                .or_default()
                .push(topic.id.clone());
        }
    }
    for topic in state.topics.values() {
        let ids = names.entry(normalized_name(&topic.name)).or_default();
        if !ids.contains(&topic.id) {
            ids.push(topic.id.clone());
        }
    }
    for ids in names.values() {
        if ids.len() > 1 {
            ambiguous.extend(ids.iter().cloned());
        }
    }
    for item in &job.graph.items {
        if matches!(
            item.disposition,
            DraftDisposition::Dismissed | DraftDisposition::Saved
        ) {
            continue;
        }
        let c = &item.candidate;
        let mut reasons = vec![];
        if proposal::evidence_refs(c)
            .iter()
            .any(|s| scopes::validate_source(ctx, &state, s).is_err())
        {
            reasons.push((
                SetupIssueKind::Unsupported,
                format!("stale:{}", item.item_id),
            ));
        }
        match c {
            Candidate::UnresolvedIdentity { name, .. } => reasons.push((
                SetupIssueKind::Identity,
                format!("identity:{}", normalized_name(name)),
            )),
            Candidate::Topic { topic } if ambiguous.contains(&topic.id) => reasons.push((
                SetupIssueKind::Identity,
                format!("identity:{}", normalized_name(&topic.name)),
            )),
            Candidate::Claim { claim } => {
                if !claim
                    .sources
                    .iter()
                    .any(|e| matches!(e, Evidence::Passage { .. }))
                {
                    reasons.push((
                        SetupIssueKind::Unsupported,
                        format!("unsupported:{}:{}", claim.subject_id, claim.predicate),
                    ))
                }
                if claim.uncertainty.is_some() {
                    reasons.push((
                        SetupIssueKind::Unsupported,
                        format!("uncertain:{}:{}", claim.subject_id, claim.predicate),
                    ))
                }
                let other = claims.iter().any(|(_, o)| {
                    o.subject_id == claim.subject_id
                        && o.predicate == claim.predicate
                        && o.value != claim.value
                });
                if other {
                    reasons.push((
                        SetupIssueKind::Contradiction,
                        format!("contradiction:{}:{}", claim.subject_id, claim.predicate),
                    ))
                }
                if !claim.supersedes.is_empty()
                    || state
                        .claims
                        .values()
                        .any(|o| o.subject_id == claim.subject_id && o.predicate == claim.predicate)
                {
                    reasons.push((
                        SetupIssueKind::Replacement,
                        format!("replacement:{}:{}", claim.subject_id, claim.predicate),
                    ))
                }
            }
            Candidate::Connection { connection } => {
                if state.connections.values().any(|old| {
                    old.from == connection.from
                        && old.to == connection.to
                        && old.relation == connection.relation
                        && old.valid_from == connection.valid_from
                        && old.valid_to == connection.valid_to
                }) {
                    reasons.push((
                        SetupIssueKind::Replacement,
                        format!("relationship-evidence:{}", connection.id),
                    ));
                }
                if connection.sources.is_empty() {
                    reasons.push((
                        SetupIssueKind::Unsupported,
                        format!("unsupported:{}", connection.id),
                    ))
                }
                if connection.uncertainty.is_some() {
                    reasons.push((
                        SetupIssueKind::Inferred,
                        format!(
                            "inferred:{:?}:{}:{:?}",
                            connection.from, connection.relation, connection.to
                        ),
                    ))
                }
            }
            Candidate::Correction { claim, .. } => reasons.push((
                SetupIssueKind::Replacement,
                format!("replacement:{}:{}", claim.subject_id, claim.predicate),
            )),
            _ => {}
        }
        for dep in proposal::dependencies(std::slice::from_ref(c)) {
            if ambiguous.contains(&dep) {
                let name = names
                    .iter()
                    .find(|(_, ids)| ids.contains(&dep))
                    .map(|(n, _)| n.clone())
                    .unwrap_or(dep);
                reasons.push((SetupIssueKind::Identity, format!("identity:{name}")));
            }
        }
        for (kind, key) in reasons {
            let id = stable_id(&format!("{}:{key}", job.job_id));
            let issue = groups.entry(key.clone()).or_insert_with(|| SetupIssue {
                id: id.clone(),
                issue_kind: kind,
                key,
                item_ids: vec![],
                sources: vec![],
                deferred: deferred.contains(&id),
            });
            if !issue.item_ids.contains(&item.item_id) {
                issue.item_ids.push(item.item_id.clone())
            }
            for s in proposal::evidence_refs(c) {
                if !issue.sources.contains(s) {
                    issue.sources.push(s.clone())
                }
            }
        }
    }
    job.graph.issues = groups.into_values().collect();
    job.graph.issues.sort_by_key(|issue| {
        let against_confirmed = issue.issue_kind == SetupIssueKind::Replacement
            && issue.item_ids.iter().any(|id| {
                job.graph
                    .items
                    .iter()
                    .find(|item| &item.item_id == id)
                    .is_some_and(|item| match &item.candidate {
                        Candidate::Claim { claim } => state.claims.values().any(|old| {
                            old.subject_id == claim.subject_id
                                && old.predicate == claim.predicate
                                && old.value != claim.value
                                && !state
                                    .claims
                                    .values()
                                    .any(|next| next.supersedes.contains(&old.revision_id))
                                && (state.reviews.values().any(|review| {
                                    review.record_id == old.revision_id
                                        && review.state == ReviewState::ConfirmedByYou
                                }) || scopes::direct_human_value_supported(&state, old))
                        }),
                        _ => false,
                    })
            });
        let rank = if against_confirmed {
            0
        } else {
            match issue.issue_kind {
                SetupIssueKind::Identity => 1,
                SetupIssueKind::Contradiction => 2,
                SetupIssueKind::Replacement => 3,
                SetupIssueKind::Unsupported => 4,
                SetupIssueKind::Inferred => 5,
            }
        };
        (rank, issue.key.clone())
    });
    Ok(())
}
