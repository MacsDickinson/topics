use super::*;
use scopes::{Run, authorized, validate_dates};
#[derive(Serialize, Deserialize)]
struct SavedProposal {
    proposal: Proposal,
    expected: BTreeMap<String, Option<Hash>>,
    expected_bytes: BTreeMap<String, Vec<u8>>,
    operation_id: Id,
    decision: Option<Decision>,
    operation: Option<Operation>,
}
fn proposal_file(ctx: &Context, id: &str) -> WResult<PathBuf> {
    valid_id(id)?;
    Ok(ctx.partition.join("proposals").join(format!("{id}.json")))
}
fn saved_proposals(ctx: &Context) -> WResult<Vec<SavedProposal>> {
    let dir = ctx.partition.join("proposals");
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut out = vec![];
    for e in fs::read_dir(dir).map_err(internal)? {
        let e = e.map_err(internal)?;
        if e.file_type().map_err(internal)?.is_symlink() {
            return Err(error("forbidden", "Invalid proposal storage"));
        }
        let p: SavedProposal = read_json(&e.path())?;
        if p.proposal.root_id == ctx.root_id {
            out.push(p);
        }
    }
    out.sort_by_key(|p| (p.proposal.created_at.clone(), p.proposal.id.clone()));
    Ok(out)
}
fn fingerprint(candidates: &[Candidate]) -> WResult<Hash> {
    let mut identities = BTreeMap::new();
    for c in candidates {
        if let Candidate::Topic { topic } = c {
            let mut v = json_value(topic)?;
            v.as_object_mut().unwrap().remove("id");
            identities.insert(
                topic.id.clone(),
                hash(serde_json::to_string(&v).map_err(internal)?.as_bytes()),
            );
        }
    }
    let mut values = vec![];
    for c in candidates {
        let mut v = json_value(c)?;
        let key = match c {
            Candidate::Topic { .. } => "topic",
            Candidate::Claim { .. } | Candidate::Correction { .. } => "claim",
            Candidate::Connection { .. } => "connection",
            Candidate::UnresolvedIdentity { .. } => "",
        };
        if let Some(map) = v.get_mut(key).and_then(Value::as_object_mut) {
            map.remove("id");
            map.remove("revision_id");
            map.remove("recorded_at");
        }
        if let Some(map) = v.get_mut("rule").and_then(Value::as_object_mut) {
            map.remove("id");
            map.remove("source_revision_id");
        }
        fn normalize(v: &mut Value, ids: &BTreeMap<String, String>) {
            match v {
                Value::Object(map) => {
                    for v in map.values_mut() {
                        normalize(v, ids)
                    }
                }
                Value::Array(a) => {
                    for v in a {
                        normalize(v, ids)
                    }
                }
                Value::String(s) => {
                    if let Some(value) = ids.get(s) {
                        *s = value.clone();
                    }
                }
                _ => {}
            }
        }
        normalize(&mut v, &identities);
        values.push(serde_json::to_string(&v).map_err(internal)?);
    }
    values.sort();
    Ok(hash(values.join("\n").as_bytes()))
}

fn candidate_ids(candidates: &[Candidate]) -> Vec<Id> {
    candidates
        .iter()
        .flat_map(|c| match c {
            Candidate::Topic { topic } => vec![topic.id.clone()],
            Candidate::Claim { claim } => vec![claim.revision_id.clone()],
            Candidate::Correction { claim, rule } => {
                vec![claim.revision_id.clone(), rule.id.clone()]
            }
            Candidate::Connection { connection } => vec![connection.id.clone()],
            Candidate::UnresolvedIdentity { .. } => vec![],
        })
        .collect()
}
fn evidence_refs(c: &Candidate) -> Vec<&SourceRef> {
    match c {
        Candidate::Claim { claim } | Candidate::Correction { claim, .. } => claim
            .sources
            .iter()
            .filter_map(|s| {
                if let Evidence::Passage { source } = s {
                    Some(source)
                } else {
                    None
                }
            })
            .collect(),
        Candidate::Connection { connection } => connection.sources.iter().collect(),
        Candidate::UnresolvedIdentity { sources, .. } => sources.iter().collect(),
        _ => vec![],
    }
}
fn dependencies(candidates: &[Candidate]) -> Vec<Id> {
    let mut out = BTreeSet::new();
    for c in candidates {
        match c {
            Candidate::Claim { claim } | Candidate::Correction { claim, .. } => {
                out.insert(claim.subject_id.clone());
                out.extend(claim.supersedes.iter().cloned());
                for e in &claim.sources {
                    match e {
                        Evidence::Passage { source } => {
                            out.insert(source.source_id.clone());
                        }
                        Evidence::UserStatement { statement_id } => {
                            out.insert(statement_id.clone());
                        }
                    }
                }
            }
            Candidate::Connection { connection } => {
                for ep in [&connection.from, &connection.to] {
                    match ep {
                        Endpoint::Topic { id } | Endpoint::Claim { id } => {
                            out.insert(id.clone());
                        }
                        Endpoint::Source { source_id, .. } => {
                            out.insert(source_id.clone());
                        }
                    }
                }
            }
            Candidate::UnresolvedIdentity { candidates, .. } => {
                out.extend(candidates.iter().cloned())
            }
            _ => {}
        }
        for s in evidence_refs(c) {
            out.insert(s.source_id.clone());
        }
    }
    out.into_iter().collect()
}
fn validate(
    ctx: &Context,
    state: &State,
    candidates: &[Candidate],
    selected: Option<&BTreeSet<Id>>,
    approval: bool,
) -> WResult<()> {
    validate_candidate_shapes(candidates)?;
    let ids = candidate_ids(candidates);
    let mut unique = BTreeSet::new();
    for id in &ids {
        valid_id(id)?;
        if !unique.insert(id) {
            return Err(error("invalid_input", "Duplicate provisional IDs"));
        }
        if state
            .hashes
            .keys()
            .any(|path| path.ends_with(&format!("/{id}.md")))
            || state.topics.contains_key(id)
            || state.claims.contains_key(id)
            || state.connections.contains_key(id)
            || state.rules.contains_key(id)
            || state.sources.contains_key(id)
            || state.statements.contains_key(id)
        {
            return Err(error(
                "conflict",
                "Candidate ID collides with accepted records",
            ));
        }
    }
    let mut topics = state.topics.clone();
    let mut claims = state.claims.clone();
    for c in candidates {
        match c {
            Candidate::Topic { topic } => {
                valid_id(&topic.id)?;
                if topic.name.trim().is_empty() {
                    return Err(error("invalid_input", "Topic name is required"));
                }
                topics.insert(topic.id.clone(), topic.clone());
            }
            Candidate::Claim { claim } | Candidate::Correction { claim, .. } => {
                claims.insert(claim.revision_id.clone(), claim.clone());
            }
            _ => {}
        }
    }
    let endpoint = |e: &Endpoint| -> WResult<()> {
        match e {
            Endpoint::Topic { id } => {
                if topics.contains_key(id) {
                    Ok(())
                } else {
                    Err(error("invalid_input", "Missing dependency: topic"))
                }
            }
            Endpoint::Claim { id } => {
                if claims.values().any(|c| &c.id == id || &c.revision_id == id) {
                    Ok(())
                } else {
                    Err(error("invalid_input", "Missing dependency: claim"))
                }
            }
            Endpoint::Source { source_id, locator } => {
                let source = state
                    .sources
                    .get(source_id)
                    .ok_or_else(|| error("invalid_input", "Missing dependency: source"))?;
                let bytes = scopes::source_bytes(ctx, &source.path)?;
                let text = std::str::from_utf8(&bytes).map_err(internal)?;
                if locator.start_byte >= locator.end_byte
                    || text
                        .get(locator.start_byte as usize..locator.end_byte as usize)
                        .is_none()
                {
                    return Err(error("invalid_input", "Invalid endpoint passage locator"));
                }
                Ok(())
            }
        }
    };
    let mut superseded = BTreeSet::new();
    let mut logical_claims = BTreeSet::new();
    for c in candidates {
        for source in evidence_refs(c) {
            validate_source(ctx, state, source)?;
            if selected.is_some_and(|ids| !ids.contains(&source.source_id)) {
                return Err(error(
                    "forbidden",
                    "Evidence is outside the confirmed scope",
                ));
            }
        }
        match c {
            Candidate::Topic { .. } => {}
            Candidate::Claim { claim } | Candidate::Correction { claim, .. } => {
                valid_id(&claim.id)?;
                if !logical_claims.insert(claim.id.clone())
                    || topics.contains_key(&claim.id)
                    || state.sources.contains_key(&claim.id)
                    || state.connections.contains_key(&claim.id)
                    || state.rules.contains_key(&claim.id)
                    || state.statements.contains_key(&claim.id)
                    || ids
                        .iter()
                        .any(|id| id == &claim.id && id != &claim.revision_id)
                {
                    return Err(error(
                        "invalid_input",
                        "Claim identity collides with another entity",
                    ));
                }
                valid_id(&claim.revision_id)?;
                valid_id(&claim.subject_id)?;
                if !topics.contains_key(&claim.subject_id) {
                    return Err(error(
                        "invalid_input",
                        "Missing dependency: claim subject topic",
                    ));
                }
                let prior: Vec<_> = state.claims.values().filter(|c| c.id == claim.id).collect();
                if !prior.is_empty() && claim.supersedes.is_empty() {
                    return Err(error(
                        "conflict",
                        "Existing claim needs an explicit superseding revision",
                    ));
                }
                for revision in &claim.supersedes {
                    let old = state.claims.get(revision).ok_or_else(|| {
                        error("invalid_input", "Missing dependency: superseded revision")
                    })?;
                    if old.id != claim.id
                        || old.subject_id != claim.subject_id
                        || old.predicate != claim.predicate
                    {
                        return Err(error(
                            "invalid_input",
                            "Superseded revision belongs to another assertion",
                        ));
                    }
                    if !superseded.insert(revision)
                        || state
                            .claims
                            .values()
                            .any(|c| c.supersedes.contains(revision))
                    {
                        return Err(error(
                            "conflict",
                            "Claim revision already has a successor; refresh review",
                        ));
                    }
                }
                for evidence in &claim.sources {
                    if let Evidence::UserStatement { statement_id } = evidence {
                        let s = state.statements.get(statement_id).ok_or_else(|| {
                            error("invalid_input", "Missing persisted user statement")
                        })?;
                        if s.topic_id != claim.subject_id {
                            return Err(error(
                                "invalid_input",
                                "User statement belongs to another topic",
                            ));
                        }
                    }
                }
                if let Candidate::Correction { rule, .. } = c {
                    valid_id(&rule.id)?;
                    if claim.supersedes.is_empty()
                        || rule.topic_id != claim.subject_id
                        || rule.predicate != claim.predicate
                        || rule.source_revision_id != claim.revision_id
                        || rule.replacement != claim.value
                        || rule.match_value.is_empty()
                        || !claim
                            .sources
                            .iter()
                            .any(|s| matches!(s, Evidence::UserStatement { .. }))
                    {
                        return Err(error(
                            "invalid_input",
                            "Correction must supersede a claim and cite a persisted user statement with a matching scoped rule",
                        ));
                    }
                    if !claim.supersedes.iter().any(|r| {
                        state
                            .claims
                            .get(r)
                            .is_some_and(|c| c.value == rule.match_value)
                    }) {
                        return Err(error(
                            "invalid_input",
                            "Rule must match the superseded assertion",
                        ));
                    }
                    for path in &rule.scope.source_paths {
                        relative(path)?;
                        if path.starts_with("topics/") {
                            return Err(error(
                                "invalid_input",
                                "Rule source scope cannot target app records",
                            ));
                        }
                    }
                }
                if approval && state.rules.values().any(|r| rule_matches(r, claim)) {
                    return Err(error(
                        "conflict",
                        "Correction rule suppresses this assertion; review a replacement",
                    ));
                }
            }
            Candidate::Connection { connection } => {
                endpoint(&connection.from)?;
                endpoint(&connection.to)?;
                if connection.relation.trim().is_empty() || connection.sources.is_empty() {
                    return Err(error(
                        "invalid_input",
                        "Connections require a relation and evidence",
                    ));
                }
                validate_dates(&connection.valid_from, &connection.valid_to)?;
            }
            Candidate::UnresolvedIdentity {
                name,
                candidates,
                sources,
            } => {
                if name.trim().is_empty()
                    || candidates.len() < 2
                    || sources.is_empty()
                    || candidates.iter().any(|id| !topics.contains_key(id))
                {
                    return Err(error(
                        "invalid_input",
                        "Unresolved identity requires evidence and at least two existing candidates",
                    ));
                }
            }
        }
    }
    Ok(())
}
fn validate_model_boundary(
    ctx: &Context,
    state: &State,
    candidates: &[Candidate],
    selected: &BTreeSet<Id>,
    origin: &Origin,
) -> WResult<()> {
    validate_candidate_shapes(candidates)?;
    let allowed_topic = |id: &str| {
        state
            .topics
            .get(id)
            .is_none_or(|t| t.sensitivity == Sensitivity::CloudAllowed)
    };
    let check_endpoint = |endpoint: &Endpoint| -> WResult<()> {
        let allowed=match endpoint {
            Endpoint::Topic{id}=>allowed_topic(id),
            Endpoint::Claim{id}=>state.claims.values().find(|c|&c.id==id||&c.revision_id==id).is_none_or(|c|allowed_topic(&c.subject_id)&&c.sources.iter().all(|e|matches!(e,Evidence::Passage{source} if selected.contains(&source.source_id)))),
            Endpoint::Source{source_id,..}=>selected.contains(source_id),
        };
        if allowed {
            Ok(())
        } else {
            Err(error(
                "forbidden",
                "Candidate references restricted or unselected context",
            ))
        }
    };
    for candidate in candidates {
        if let Candidate::Claim { claim } | Candidate::Correction { claim, .. } = candidate {
            for evidence in &claim.sources {
                if let Evidence::UserStatement { statement_id } = evidence
                    && !matches!(origin,Origin::Chat{topic_id,message_id} if topic_id==&claim.subject_id&&message_id==statement_id)
                {
                    return Err(error(
                        "forbidden",
                        "User-statement evidence must belong to this chat turn",
                    ));
                }
            }
            if let Origin::Chat { topic_id, .. } = origin
                && topic_id != &claim.subject_id
            {
                return Err(error(
                    "forbidden",
                    "Chat mutation is outside its authorized topic",
                ));
            }
        }
        for source in evidence_refs(candidate) {
            if !selected.contains(&source.source_id) {
                return Err(error(
                    "forbidden",
                    "Evidence is outside the confirmed scope",
                ));
            }
            validate_source(ctx, state, source)?;
        }
        match candidate {
            Candidate::Topic { topic } => {
                if !allowed_topic(&topic.id) {
                    return Err(error(
                        "forbidden",
                        "Candidate references restricted context",
                    ));
                }
            }
            Candidate::Claim { claim } | Candidate::Correction { claim, .. } => {
                if !allowed_topic(&claim.subject_id) {
                    return Err(error(
                        "forbidden",
                        "Candidate references restricted context",
                    ));
                }
            }
            Candidate::Connection { connection } => {
                check_endpoint(&connection.from)?;
                check_endpoint(&connection.to)?;
            }
            Candidate::UnresolvedIdentity { candidates, .. } => {
                if candidates.iter().any(|id| !allowed_topic(id)) {
                    return Err(error("forbidden", "Identity includes restricted context"));
                }
            }
        }
    }
    Ok(())
}
fn rule_matches(rule: &Rule, claim: &Claim) -> bool {
    rule.topic_id == claim.subject_id
        && rule.predicate == claim.predicate
        && rule.match_value == claim.value
        && claim.sources.iter().any(|s| match s {
            Evidence::Passage { source } => {
                (rule.scope.source_paths.is_empty()
                    || rule.scope.source_paths.contains(&source.path))
                    && rule
                        .scope
                        .context
                        .as_ref()
                        .is_none_or(|c| source.excerpt.contains(c))
            }
            _ => false,
        })
}
fn taint(state: &State, candidates: &mut [Candidate]) -> BTreeSet<Id> {
    let mut private: BTreeSet<_> = state
        .topics
        .values()
        .filter(|t| t.sensitivity == Sensitivity::LocalOnly)
        .map(|t| t.id.clone())
        .collect();
    for c in candidates.iter() {
        if evidence_refs(c).iter().any(|s| {
            state
                .sources
                .get(&s.source_id)
                .is_some_and(|s| s.sensitivity == Sensitivity::LocalOnly)
        }) {
            match c {
                Candidate::Claim { claim } | Candidate::Correction { claim, .. } => {
                    private.insert(claim.subject_id.clone());
                }
                Candidate::Connection { connection } => {
                    for e in [&connection.from, &connection.to] {
                        if let Endpoint::Topic { id } = e {
                            private.insert(id.clone());
                        }
                    }
                }
                _ => {}
            }
        }
    }
    for c in candidates {
        if let Candidate::Topic { topic } = c
            && private.contains(&topic.id)
        {
            topic.sensitivity = Sensitivity::LocalOnly;
        }
    }
    private
}
fn bound_model_run(
    ctx: &Context,
    request: &StageProposalRequest,
) -> WResult<(Option<BTreeSet<Id>>, Option<Id>)> {
    let mut selected = None;
    let mut run = None;
    match &request.origin {
        Origin::Manual => {
            if request.scope_id.is_some() {
                return Err(error(
                    "invalid_input",
                    "Manual proposals do not have a model scope",
                ));
            }
        }
        Origin::Analysis { run_id } => {
            let id = request
                .scope_id
                .as_ref()
                .ok_or_else(|| error("forbidden", "Model proposals require a confirmed scope"))?;
            let s = authorized(ctx, id, run_id)?;
            selected = Some(
                s.preview
                    .sources
                    .into_iter()
                    .map(|s| s.source_id)
                    .collect::<BTreeSet<_>>(),
            );
            run = Some(run_id.clone());
        }
        Origin::Chat {
            topic_id,
            message_id,
        } => {
            let scope_id = request
                .scope_id
                .as_ref()
                .ok_or_else(|| error("forbidden", "Chat proposals require a confirmed scope"))?;
            let mut found = None;
            let dir = ctx.partition.join("runs");
            if dir.exists() {
                for e in fs::read_dir(dir).map_err(internal)? {
                    let r: Run = read_json(&e.map_err(internal)?.path())?;
                    if r.message_id.as_ref() == Some(message_id)
                        && r.topic_id.as_ref() == Some(topic_id)
                        && r.active
                    {
                        found = Some(r);
                    }
                }
            }
            let r = found.ok_or_else(|| error("forbidden", "Chat capability is inactive"))?;
            let s = authorized(ctx, scope_id, &r.run_id)?;
            selected = Some(s.preview.sources.into_iter().map(|s| s.source_id).collect());
            run = Some(r.run_id);
        }
    }
    Ok((selected, run))
}
fn prepared_candidates(
    ctx: &Context,
    state: &State,
    request: &StageProposalRequest,
    selected: Option<&BTreeSet<Id>>,
) -> WResult<Vec<Candidate>> {
    let mut candidates = request.candidates.clone();
    // Inferred endpoints with ambiguous names stay reviewable as unresolved candidates.
    if !matches!(request.origin, Origin::Manual) {
        for candidate in &mut candidates {
            if let Candidate::Connection { connection } = candidate {
                let ambiguous = [&connection.from, &connection.to].iter().find_map(|ep| {
                    if let Endpoint::Topic { id } = ep {
                        let topic = state.topics.get(id)?;
                        let ids: Vec<_> = state
                            .topics
                            .values()
                            .filter(|t| t.name == topic.name)
                            .map(|t| t.id.clone())
                            .collect();
                        (ids.len() > 1).then(|| (topic.name.clone(), ids))
                    } else {
                        None
                    }
                });
                if let Some((name, ids)) = ambiguous {
                    *candidate = Candidate::UnresolvedIdentity {
                        name,
                        candidates: ids,
                        sources: connection.sources.clone(),
                    };
                }
            }
        }
    }
    candidates.retain(|c|!matches!(c,Candidate::Claim{claim} if state.rules.values().any(|r|rule_matches(r,claim))));
    taint(state, &mut candidates);
    validate(ctx, state, &candidates, selected, false)?;
    Ok(candidates)
}
impl WorkspaceService {
    /// Validate one buffered tool call without publishing a proposal or consuming its run.
    pub fn validate_proposal(&self, request: StageProposalRequest) -> WResult<()> {
        self.with(&request.root_id, |ctx| {
            let state = load(ctx)?;
            let (selected, _) = bound_model_run(ctx, &request)?;
            if let Some(selected) = &selected {
                validate_model_boundary(
                    ctx,
                    &state,
                    &request.candidates,
                    selected,
                    &request.origin,
                )?;
            }
            let fingerprint = fingerprint(&request.candidates)?;
            if !matches!(request.origin, Origin::Manual)
                && state.operations.iter().any(|c| {
                    c.proposal
                        .as_ref()
                        .is_some_and(|p| p.fingerprint == fingerprint)
                })
            {
                return Ok(());
            }
            if saved_proposals(ctx)?.iter().any(|p| {
                p.proposal.fingerprint == fingerprint
                    && (!matches!(request.origin, Origin::Manual)
                        || p.proposal.candidates == request.candidates)
            }) {
                return Ok(());
            }
            prepared_candidates(ctx, &state, &request, selected.as_ref())?;
            Ok(())
        })
    }

    pub fn manual_propose(&self, request: ManualProposeRequest) -> WResult<ManualProposeResult> {
        Ok(ManualProposeResult {
            proposal: self.stage_proposal(StageProposalRequest {
                root_id: request.root_id,
                scope_id: None,
                origin: Origin::Manual,
                candidates: request.candidates,
            })?,
        })
    }

    pub fn proposals_list(&self, request: RootRequest) -> WResult<ProposalsListResult> {
        self.with(&request.root_id, |ctx| {
            Ok(ProposalsListResult {
                proposals: saved_proposals(ctx)?
                    .into_iter()
                    .map(|p| p.proposal)
                    .collect(),
            })
        })
    }
    pub fn stage_proposal(&self, request: StageProposalRequest) -> WResult<Proposal> {
        self.with(&request.root_id, |ctx| {
            let state = load(ctx)?;
            let (selected, run) = bound_model_run(ctx, &request)?;
            // This application-only call consumes the terminal turn's capability, including
            // duplicate or malformed completions, so late tool activity cannot reuse it.
            if let Some(id) = &run {
                let path = ctx.partition.join("runs").join(format!("{id}.json"));
                let mut r: Run = read_json(&path)?;
                r.active = false;
                save_json(&path, &r)?;
            }
            if let Some(selected) = &selected {
                validate_model_boundary(
                    ctx,
                    &state,
                    &request.candidates,
                    selected,
                    &request.origin,
                )?;
            }
            let fingerprint = fingerprint(&request.candidates)?;
            if !matches!(request.origin, Origin::Manual)
                && let Some(committed) = state
                    .operations
                    .iter()
                    .find_map(|c| c.proposal.as_ref().filter(|p| p.fingerprint == fingerprint))
            {
                return Ok(Proposal {
                    id: committed.proposal_id.clone(),
                    root_id: ctx.root_id.clone(),
                    scope_id: request.scope_id.clone(),
                    origin: request.origin.clone(),
                    status: ProposalStatus::Accepted,
                    candidates: if !committed.applied_candidates.is_empty() {
                        committed.applied_candidates.clone()
                    } else {
                        match &committed.decision {
                            Decision::Approve { candidates } => candidates.clone(),
                            Decision::Reject => vec![],
                        }
                    },
                    fingerprint,
                    created_at: now(),
                });
            }
            if let Some(existing) = saved_proposals(ctx)?.into_iter().find(|p| {
                p.proposal.fingerprint == fingerprint
                    && (!matches!(request.origin, Origin::Manual)
                        || p.proposal.candidates == request.candidates)
            }) {
                return Ok(existing.proposal);
            }
            let candidates = prepared_candidates(ctx, &state, &request, selected.as_ref())?;
            let proposal = Proposal {
                id: new_id(),
                root_id: ctx.root_id.clone(),
                scope_id: request.scope_id.clone(),
                origin: request.origin.clone(),
                status: ProposalStatus::Pending,
                candidates,
                fingerprint,
                created_at: now(),
            };
            let expected_bytes = state
                .hashes
                .keys()
                .map(|path| {
                    Ok((
                        path.clone(),
                        fs::read(target_path(&ctx.root, path, false)?).map_err(internal)?,
                    ))
                })
                .collect::<WResult<BTreeMap<_, _>>>()?;
            let saved = SavedProposal {
                proposal: proposal.clone(),
                expected: state.hashes,
                expected_bytes,
                operation_id: new_id(),
                decision: None,
                operation: None,
            };
            save_json(&proposal_file(ctx, &proposal.id)?, &saved)?;
            Ok(proposal)
        })
    }
    pub fn proposal_decide(&self, request: ProposalDecideRequest) -> WResult<ProposalDecideResult> {
        self.with(&request.root_id, |ctx| {
            let path = proposal_file(ctx, &request.proposal_id)?;
            let mut saved: SavedProposal =
                read_json(&path).map_err(|_| error("invalid_input", "Proposal does not exist"))?;
            if saved.proposal.root_id != ctx.root_id {
                return Err(error(
                    "stale_scope",
                    "Proposal belongs to another root selection",
                ));
            }
            if saved.proposal.fingerprint != request.expected_fingerprint {
                return Err(error(
                    "conflict",
                    "Proposal fingerprint changed; refresh review",
                ));
            }
            if let Some(commit)=load(ctx)?.operations.iter().find(|c|c.operation.id==saved.operation_id) {
                let durable=commit.proposal.as_ref().ok_or_else(||error("conflict","Operation identity collision"))?;
                if durable.decision!=request.decision{return Err(error("conflict","Proposal already has a different durable decision"));}
                saved.proposal.status=ProposalStatus::Accepted;if !durable.applied_candidates.is_empty(){saved.proposal.candidates=durable.applied_candidates.clone();}saved.operation=Some(commit.operation.clone());saved.decision=Some(request.decision.clone());save_json(&path,&saved)?;
            }
            if let Some(decision) = &saved.decision {
                if decision == &request.decision {
                    return Ok(ProposalDecideResult {
                        operation: saved.operation,
                        proposal: saved.proposal,
                    });
                }
                return Err(error(
                    "conflict",
                    "Proposal already has a different decision",
                ));
            }
            if let Decision::Approve { candidates } = &request.decision {
                let mut state = load(ctx)?;
                let proposed: BTreeSet<_> = candidate_ids(&saved.proposal.candidates)
                    .into_iter()
                    .collect();
                if candidate_ids(candidates)
                    .iter()
                    .any(|id| !proposed.contains(id))
                {
                    return Err(error(
                        "invalid_input",
                        "Edited approval cannot introduce unrelated record IDs",
                    ));
                }
                // Scope bytes remain a precondition even after the runtime capability closes.
                if let Some(id) = &saved.proposal.scope_id {
                    scopes::scope(ctx, id).map_err(|_| {
                        error("stale_evidence", "Selection changed before approval")
                    })?;
                }
                let mut approved = candidates.clone();
                let private = taint(&state, &mut approved);
                let op = saved.operation_id.clone();
                let mut records = vec![];
                // A human edit is new provenance, never a fabricated transcript assertion.
                for candidate in &mut approved {
                    if let Candidate::Claim{claim}|Candidate::Correction{claim,..}=candidate {
                        let original=saved.proposal.candidates.iter().find_map(|c|match c {Candidate::Claim{claim:old}|Candidate::Correction{claim:old,..} if old.revision_id==claim.revision_id=>Some(old),_=>None});
                        if let Some(original)=original && (original.value!=claim.value||original.predicate!=claim.predicate||original.valid_from!=claim.valid_from||original.valid_to!=claim.valid_to||original.sources!=claim.sources) {
                            let statement=UserStatement{id:new_id(),topic_id:claim.subject_id.clone(),text:format!("User edited proposed {} assertion from {:?} to {:?}.",claim.predicate,original.value,claim.value),recorded_at:now()};
                            claim.sources.push(Evidence::UserStatement{statement_id:statement.id.clone()});
                            claim.uncertainty=Some(format!("{}; user edit recorded as evidence",claim.uncertainty.as_deref().unwrap_or("User-reviewed assertion")));
                            records.push((format!("topics/statements/{}.md",statement.id),Record::new("statement",&statement.id,&op,&statement)?));state.statements.insert(statement.id.clone(),statement);
                        }
                    }
                }
                let mut expected = saved.expected.clone();
                for c in &approved {
                    match c {
                        Candidate::Topic { topic } => records.push((
                            format!("topics/topics/{}.md", topic.id),
                            Record::new("topic", &topic.id, &op, topic)?,
                        )),
                        Candidate::Claim { claim } => records.push((
                            format!("topics/claims/{}.md", claim.revision_id),
                            Record::new("claim", &claim.revision_id, &op, claim)?,
                        )),
                        Candidate::Correction { claim, rule } => {
                            records.push((
                                format!("topics/claims/{}.md", claim.revision_id),
                                Record::new("claim", &claim.revision_id, &op, claim)?,
                            ));
                            records.push((
                                format!("topics/rules/{}.md", rule.id),
                                Record::new("rule", &rule.id, &op, rule)?,
                            ));
                        }
                        Candidate::Connection { connection } => records.push((
                            format!("topics/connections/{}.md", connection.id),
                            Record::new("connection", &connection.id, &op, connection)?,
                        )),
                        Candidate::UnresolvedIdentity { .. } => {
                            let id = new_id();
                            records.push((
                                format!("topics/unresolved/{id}.md"),
                                Record::new("unresolved", &id, &op, c)?,
                            ));
                        }
                    }
                }
                for id in private {
                    if let Some(topic) = state.topics.get(&id)
                        && topic.sensitivity != Sensitivity::LocalOnly {
                            let mut topic = topic.clone();
                            topic.sensitivity = Sensitivity::LocalOnly;
                            records.push((
                                format!("topics/topics/{id}.md"),
                                Record::new("topic", &id, &op, topic)?,
                            ));
                        }
                }
                for (path, _) in &records {
                    expected.entry(path.clone()).or_insert(None);
                }
                let mut stale_parent=false;
                for (path,expected) in &saved.expected {if bytes_optional(&target_path(&ctx.root,path,false)?)?.as_deref().map(hash)!=*expected {stale_parent=true;}}
                if stale_parent {
                    let current=saved.expected.keys().map(|p|Ok((p.clone(),bytes_optional(&target_path(&ctx.root,p,false)?)?))).collect::<WResult<BTreeMap<_,_>>>()?;
                    save_json(&ctx.partition.join("conflicts").join(format!("{}.json",saved.proposal.id)),&serde_json::json!({"old_bytes":saved.expected_bytes,"current_bytes":current,"staged_records":records.iter().map(|(p,r)|Ok((p.clone(),r.bytes()?))).collect::<WResult<BTreeMap<_,_>>>()?}))?;
                    return Err(error("conflict","Referenced record changed since proposal review; versions preserved"));
                }
                validate(ctx, &state, candidates, None, true)?;
                let operation = commit(ctx,"Approve reviewed proposal",records,CommitIntent{dependencies:dependencies(&approved),tombstones:vec![],undo_of:None,expected:Some(&expected),proposal:Some(CommittedDecision{proposal_id:saved.proposal.id.clone(),fingerprint:saved.proposal.fingerprint.clone(),decision:request.decision.clone(),applied_candidates:approved.clone(),proposed_candidates:saved.proposal.candidates.clone()})})?;
                saved.operation = Some(operation);
                saved.proposal.candidates=approved;
                saved.proposal.status = ProposalStatus::Accepted;
            } else {
                saved.proposal.status = ProposalStatus::Rejected;
            }
            saved.decision = Some(request.decision);
            save_json(&path, &saved)?;
            Ok(ProposalDecideResult {
                operation: saved.operation,
                proposal: saved.proposal,
            })
        })
    }
}
