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
    let state = load(ctx)?;
    for path in json_entries(&dir)? {
        let Ok(mut p) = read_json::<SavedProposal>(&path) else {
            continue;
        };
        if p.proposal.root_id != ctx.root_id {
            continue;
        }
        if let Some(commit) = state
            .operations
            .iter()
            .find(|c| c.operation.id == p.operation_id)
        {
            if undone(&state, &commit.operation.id) {
                p.proposal.status = ProposalStatus::Conflict;
            } else if let Some(decision) = &commit.proposal {
                p.proposal.status = ProposalStatus::Accepted;
                p.operation = Some(commit.operation.clone());
                p.decision = Some(decision.decision.clone());
                if !decision.applied_candidates.is_empty() {
                    p.proposal.candidates = decision.applied_candidates.clone();
                }
            }
        } else if p.proposal.status == ProposalStatus::Pending
            && p.expected
                .iter()
                .any(|(path, h)| state.hashes.get(path) != Some(h))
        {
            p.proposal.status = ProposalStatus::Conflict;
        }
        save_json(&path, &p)?;
        out.push(p);
    }
    out.sort_by_key(|p| (p.proposal.created_at.clone(), p.proposal.id.clone()));
    Ok(out)
}
fn fingerprint(candidates: &[Candidate]) -> WResult<Hash> {
    let mut identities = BTreeMap::new();
    let mut counts = BTreeMap::new();
    for candidate in candidates {
        if let Candidate::Topic { topic } = candidate {
            *counts
                .entry((topic.name.clone(), format!("{:?}", topic.topic_kind)))
                .or_insert(0) += 1;
        }
    }
    for c in candidates {
        if let Candidate::Topic { topic } = c {
            let referenced = candidates
                .iter()
                .any(|c| dependencies(std::slice::from_ref(c)).contains(&topic.id));
            if referenced
                && counts.get(&(topic.name.clone(), format!("{:?}", topic.topic_kind))) == Some(&1)
            {
                let mut v = json_value(topic)?;
                v.as_object_mut().unwrap().remove("id");
                identities.insert(
                    topic.id.clone(),
                    hash(serde_json::to_string(&v).map_err(internal)?.as_bytes()),
                );
            }
        }
    }
    for c in candidates {
        if let Candidate::Claim { claim } | Candidate::Correction { claim, .. } = c {
            let key = serde_json::json!({"subject":identities.get(&claim.subject_id).unwrap_or(&claim.subject_id),"predicate":claim.predicate,"value":claim.value,"sources":claim.sources,"valid_from":claim.valid_from,"valid_to":claim.valid_to});
            let token = hash(serde_json::to_string(&key).map_err(internal)?.as_bytes());
            identities.insert(claim.id.clone(), token.clone());
            identities.insert(claim.revision_id.clone(), token);
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
            if !matches!(c,Candidate::Topic{topic} if !identities.contains_key(&topic.id)) {
                map.remove("id");
            }
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
        state.topics.get(id).is_none_or(|t| {
            t.sensitivity != Sensitivity::LocalOnly && !scopes::topic_restricted(state, id)
        })
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
        if let Origin::Chat { topic_id, .. } = origin {
            let authorized = match candidate {
                Candidate::Topic { topic } => &topic.id == topic_id,
                Candidate::Connection { connection } => [&connection.from, &connection.to]
                    .iter()
                    .any(|e| scopes::endpoint_topic(state, e).as_ref() == Some(topic_id)),
                Candidate::UnresolvedIdentity { candidates, .. } => candidates.contains(topic_id),
                _ => true,
            };
            if !authorized {
                return Err(error(
                    "forbidden",
                    "Chat mutation is outside its authorized topic",
                ));
            }
        }
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
                for path in json_entries(&dir)? {
                    let Ok(r) = read_json::<Run>(&path) else {
                        continue;
                    };
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
fn same_claim(a: &Claim, b: &Claim) -> bool {
    a.subject_id == b.subject_id
        && a.predicate == b.predicate
        && a.value == b.value
        && a.valid_from == b.valid_from
        && a.valid_to == b.valid_to
}
fn endpoint_identity(state: &State, e: &Endpoint) -> Endpoint {
    match e {
        Endpoint::Claim { id } => Endpoint::Claim {
            id: state
                .claims
                .values()
                .find(|c| &c.id == id || &c.revision_id == id)
                .map(|c| c.id.clone())
                .unwrap_or_else(|| id.clone()),
        },
        _ => e.clone(),
    }
}
fn remap(candidates: &mut [Candidate], mapping: &BTreeMap<Id, Id>) -> WResult<()> {
    fn replace(value: &mut Value, mapping: &BTreeMap<Id, Id>) {
        match value {
            Value::Object(map) => {
                for (k, v) in map {
                    if [
                        "id",
                        "revision_id",
                        "subject_id",
                        "topic_id",
                        "source_revision_id",
                    ]
                    .contains(&k.as_str())
                    {
                        if let Value::String(id) = v
                            && let Some(new) = mapping.get(id)
                        {
                            *id = new.clone();
                        }
                    } else {
                        replace(v, mapping)
                    }
                }
            }
            Value::Array(items) => {
                for item in items {
                    replace(item, mapping)
                }
            }
            _ => {}
        }
    }
    for candidate in candidates {
        let mut value = json_value(&*candidate)?;
        replace(&mut value, mapping);
        *candidate = serde_json::from_value(value).map_err(internal)?;
    }
    Ok(())
}
fn prepared_candidates(
    ctx: &Context,
    state: &State,
    request: &StageProposalRequest,
    selected: Option<&BTreeSet<Id>>,
) -> WResult<Vec<Candidate>> {
    let mut candidates = request.candidates.clone();
    if matches!(request.origin, Origin::Manual) {
        taint(state, &mut candidates);
        validate(ctx, state, &candidates, selected, false)?;
        return Ok(candidates);
    }
    let mut mapping = BTreeMap::new();
    let mut ambiguous = BTreeMap::<Id, (String, Vec<Id>)>::new();
    for candidate in &candidates {
        if let Candidate::Topic { topic } = candidate {
            if let Some(accepted) = state.topics.get(&topic.id) {
                if accepted.name != topic.name || accepted.topic_kind != topic.topic_kind {
                    return Err(error(
                        "invalid_input",
                        "Existing topic echo differs from the accepted identity",
                    ));
                }
                mapping.insert(topic.id.clone(), topic.id.clone());
                continue;
            }
            let matches: Vec<_> = state
                .topics
                .values()
                .filter(|t| {
                    t.topic_kind == topic.topic_kind
                        && (t.name == topic.name || t.aliases.contains(&topic.name))
                })
                .collect();
            let referenced = candidates
                .iter()
                .any(|c| dependencies(std::slice::from_ref(c)).contains(&topic.id));
            if matches.len() > 1 && referenced {
                ambiguous.insert(
                    topic.id.clone(),
                    (
                        topic.name.clone(),
                        matches.iter().map(|t| t.id.clone()).collect(),
                    ),
                );
                continue;
            }
            if matches.len() == 1 && referenced {
                let accepted = matches[0];
                let evidence_match = candidates.iter().any(|c| match c {
                    Candidate::Claim { claim } | Candidate::Correction { claim, .. }
                        if claim.subject_id == topic.id =>
                    {
                        state.claims.values().any(|old| {
                            old.subject_id == accepted.id
                                && old.predicate == claim.predicate
                                && old.value == claim.value
                        }) || state.rules.values().any(|rule| {
                            rule.topic_id == accepted.id
                                && rule.predicate == claim.predicate
                                && rule.match_value == claim.value
                        })
                    }
                    Candidate::Connection { connection } => state.connections.values().any(|old| {
                        old.relation == connection.relation
                            && [&old.from, &old.to]
                                .iter()
                                .any(|e| matches!(e,Endpoint::Topic{id} if id==&accepted.id))
                            && connection.sources.iter().any(|s| {
                                old.sources.iter().any(|old| {
                                    old.document_hash == s.document_hash
                                        && old.passage_hash == s.passage_hash
                                })
                            })
                    }),
                    _ => false,
                });
                if evidence_match {
                    mapping.insert(topic.id.clone(), accepted.id.clone());
                }
            }
        }
    }
    remap(&mut candidates, &mapping)?;
    candidates.retain(|c|!matches!(c,Candidate::Topic{topic} if mapping.values().any(|id|id==&topic.id)||ambiguous.contains_key(&topic.id)));
    for candidate in &mut candidates {
        if let Candidate::Topic { topic } = candidate
            && topic.sensitivity == Sensitivity::CloudAllowed
        {
            topic.sensitivity = Sensitivity::Unknown;
        }
    }
    let mut prepared = vec![];
    for candidate in candidates {
        let mut ambiguity = None;
        if !matches!(request.origin, Origin::Chat { .. }) {
            for id in dependencies(std::slice::from_ref(&candidate)) {
                if let Some(a) = ambiguous.get(&id) {
                    ambiguity = Some(a.clone());
                    break;
                }
                if let Some(topic) = state.topics.get(&id) {
                    let ids: Vec<_> = state
                        .topics
                        .values()
                        .filter(|t| t.name == topic.name && t.topic_kind == topic.topic_kind)
                        .map(|t| t.id.clone())
                        .collect();
                    if ids.len() > 1 {
                        ambiguity = Some((topic.name.clone(), ids));
                        break;
                    }
                }
            }
        }
        if let Some((name, ids)) = ambiguity {
            let sources = evidence_refs(&candidate).into_iter().cloned().collect();
            prepared.push(Candidate::UnresolvedIdentity {
                name,
                candidates: ids,
                sources,
            });
            continue;
        }
        if matches!(&candidate,Candidate::Claim{claim} if state.rules.values().any(|rule|rule_matches(rule,claim)))
        {
            continue;
        }
        prepared.push(candidate);
    }
    let superseded: BTreeSet<_> = state
        .claims
        .values()
        .flat_map(|c| c.supersedes.clone())
        .collect();
    let mut duplicates = BTreeMap::new();
    prepared.retain(|candidate| match candidate {
        Candidate::Claim { claim } => {
            if let Some(old) = state
                .claims
                .values()
                .find(|old| !superseded.contains(&old.revision_id) && same_claim(old, claim))
            {
                duplicates.insert(claim.id.clone(), old.id.clone());
                duplicates.insert(claim.revision_id.clone(), old.revision_id.clone());
                false
            } else {
                true
            }
        }
        _ => true,
    });
    remap(&mut prepared, &duplicates)?;
    prepared.retain(|candidate|!matches!(candidate,Candidate::Connection{connection} if state.connections.values().any(|old|old.relation==connection.relation&&endpoint_identity(state,&old.from)==endpoint_identity(state,&connection.from)&&endpoint_identity(state,&old.to)==endpoint_identity(state,&connection.to)&&old.valid_from==connection.valid_from&&old.valid_to==connection.valid_to)));
    // Known echoes and suppression are successful no-change outcomes.
    if prepared.is_empty() {
        return Ok(prepared);
    }
    validate_model_boundary(
        ctx,
        state,
        &prepared,
        selected.ok_or_else(|| error("forbidden", "Model scope is required"))?,
        &request.origin,
    )?;
    taint(state, &mut prepared);
    validate(ctx, state, &prepared, selected, false)?;
    Ok(prepared)
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
            let candidates = prepared_candidates(ctx, &state, &request, selected.as_ref())?;
            let fingerprint = fingerprint(if candidates.is_empty() {
                &request.candidates
            } else {
                &candidates
            })?;
            if let Some(existing) = saved_proposals(ctx)?.into_iter().find(|p| {
                p.proposal.status != ProposalStatus::Conflict
                    && p.proposal.status == ProposalStatus::Pending
                    && p.proposal.fingerprint == fingerprint
                    && (!matches!(request.origin, Origin::Manual)
                        || p.proposal.candidates == request.candidates)
            }) {
                return Ok(existing.proposal);
            }
            let proposal = Proposal {
                id: new_id(),
                root_id: ctx.root_id.clone(),
                scope_id: request.scope_id.clone(),
                origin: request.origin.clone(),
                status: if candidates.is_empty() {
                    ProposalStatus::Accepted
                } else {
                    ProposalStatus::Pending
                },
                candidates,
                fingerprint,
                created_at: now(),
            };
            let dependency_ids: BTreeSet<_> =
                dependencies(&proposal.candidates).into_iter().collect();
            let expected: BTreeMap<_, _> = state
                .hashes
                .iter()
                .filter(|(path, _)| {
                    dependency_ids
                        .iter()
                        .any(|id| path.ends_with(&format!("/{id}.md")))
                        || state.claims.values().any(|c| {
                            dependency_ids.contains(&c.id)
                                && path.ends_with(&format!("/{}.md", c.revision_id))
                        })
                })
                .map(|(p, h)| (p.clone(), h.clone()))
                .collect();
            let expected_bytes = expected
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
                expected,
                expected_bytes,
                operation_id: new_id(),
                decision: if proposal.candidates.is_empty() {
                    Some(Decision::Approve { candidates: vec![] })
                } else {
                    None
                },
                operation: None,
            };
            save_json(&proposal_file(ctx, &proposal.id)?, &saved)?;
            Ok(proposal)
        })
    }
    pub fn proposal_decide(&self, request: ProposalDecideRequest) -> WResult<ProposalDecideResult> {
        let result = self.decide_inner(request.clone());
        if result
            .as_ref()
            .is_err_and(|e| e.code == "conflict" || e.code == "stale_evidence")
        {
            let _ = self.with(&request.root_id, |ctx| {
                let path = proposal_file(ctx, &request.proposal_id)?;
                let mut saved: SavedProposal = read_json(&path)?;
                if saved.proposal.status == ProposalStatus::Pending {
                    saved.proposal.status = ProposalStatus::Conflict;
                    save_json(&path, &saved)?;
                }
                Ok(())
            });
        }
        result
    }
    fn decide_inner(&self, request: ProposalDecideRequest) -> WResult<ProposalDecideResult> {
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
            let current_state=load(ctx)?;
            if undone(&current_state,&saved.operation_id){return Err(error("conflict","Accepted operation was undone; create a fresh proposal"));}
            if let Some(commit)=current_state.operations.iter().find(|c|c.operation.id==saved.operation_id) {
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
                let mut approved = candidates.clone();
                let private = taint(&state, &mut approved);
                let op = saved.operation_id.clone();
                let mut records = vec![];
                // A human edit is new provenance, never a fabricated transcript assertion.
                for candidate in &mut approved {
                    if let Candidate::Claim{claim}|Candidate::Correction{claim,..}=candidate {
                        let original=saved.proposal.candidates.iter().find_map(|c|match c {Candidate::Claim{claim:old}|Candidate::Correction{claim:old,..} if old.revision_id==claim.revision_id=>Some(old),_=>None});
                        if let Some(original)=original && (original.value!=claim.value||original.subject_id!=claim.subject_id||original.uncertainty!=claim.uncertainty||original.predicate!=claim.predicate||original.valid_from!=claim.valid_from||original.valid_to!=claim.valid_to||original.sources!=claim.sources) {
                            let statement=UserStatement{id:new_id(),topic_id:claim.subject_id.clone(),text:format!("User edited proposed assertion from {} to {}.",serde_json::to_string(original).map_err(internal)?,serde_json::to_string(&claim).map_err(internal)?),recorded_at:now()};
                            claim.sources.push(Evidence::UserStatement{statement_id:statement.id.clone()});
                            claim.uncertainty=Some(format!("{}; user edit recorded as evidence",claim.uncertainty.as_deref().unwrap_or("User-reviewed assertion")));
                            records.push((format!("topics/statements/{}.md",statement.id),Record::new("statement",&statement.id,&op,&statement)?));state.statements.insert(statement.id.clone(),statement);
                        }
                    }
                }
                for candidate in &mut approved {
                    if let Candidate::Connection{connection}=candidate {
                        let original=saved.proposal.candidates.iter().find_map(|c|match c{Candidate::Connection{connection:old} if old.id==connection.id=>Some(old),_=>None});
                        if let Some(original)=original&&original!=connection {
                            let topic_id=scopes::endpoint_topic(&state,&connection.from).or_else(||scopes::endpoint_topic(&state,&connection.to)).or_else(||scopes::endpoint_topic(&state,&original.from)).or_else(||scopes::endpoint_topic(&state,&original.to)).unwrap_or_else(||connection.id.clone());
                            let statement=UserStatement{id:new_id(),topic_id,text:format!("User edited connection {} from {} to {}.",connection.id,serde_json::to_string(original).map_err(internal)?,serde_json::to_string(&connection).map_err(internal)?),recorded_at:now()};
                            connection.uncertainty=Some(format!("{}; HUMAN-EDITED relationship, statement {} (original passage is context, not support for the edited relation)",connection.uncertainty.as_deref().unwrap_or("User-reviewed relationship"),statement.id));
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
