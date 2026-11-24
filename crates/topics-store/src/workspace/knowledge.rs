//! Read-only knowledge projection: the whole-workspace graph, topic lookup and
//! topic detail. The desktop and the local retrieval MCP server share it; the
//! caller decides what is eligible. Nothing here writes accepted knowledge.
use super::*;
use std::cell::RefCell;

const DEFERRED: &str = "validation-deferred.json";
const RETRIEVAL: &str = "retrieval.json";
const SEARCH_DEFAULT: usize = 20;
const SEARCH_MAX: usize = 50;

/// Who is asking. The human at the desktop sees everything they saved,
/// including local-only and pending knowledge. An external model client sees
/// accepted knowledge whose topic and cited sources the human released.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Caller<'a> {
    Human,
    External(&'a BTreeSet<Id>),
}

pub(super) struct Projection {
    pub topics: BTreeMap<Id, TopicKnowledge>,
    pub edges: Vec<GraphEdge>,
    pub discoveries: Vec<PendingDiscovery>,
    pub warnings: Vec<String>,
}

pub(super) fn norm(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
fn no_checks() -> CheckCounts {
    CheckCounts {
        conflicting: 0,
        stale: 0,
        uncertain: 0,
        unverified: 0,
        deferred: 0,
    }
}
fn deferred(ctx: &Context) -> WResult<BTreeMap<Id, Timestamp>> {
    let path = ctx.partition.join(DEFERRED);
    if path.try_exists().map_err(internal)? {
        read_json(&path)
    } else {
        Ok(BTreeMap::new())
    }
}
pub(super) fn save_deferred(ctx: &Context, value: &BTreeMap<Id, Timestamp>) -> WResult<()> {
    save_json(&ctx.partition.join(DEFERRED), value)
}

/// Citation freshness and release are asked many times per projection; each
/// distinct passage is validated against its source once.
struct Citations<'a> {
    ctx: &'a Context,
    state: &'a State,
    fresh: RefCell<BTreeMap<String, bool>>,
    released: RefCell<BTreeMap<(String, Hash), bool>>,
}
impl Citations<'_> {
    fn fresh(&self, source: &SourceRef) -> bool {
        let key = serde_json::to_string(source).expect("serializable citation");
        if let Some(known) = self.fresh.borrow().get(&key) {
            return *known;
        }
        let fresh = validate_source(self.ctx, self.state, source).is_ok();
        self.fresh.borrow_mut().insert(key, fresh);
        fresh
    }
    fn passage(&self, source: &SourceRef, caller: Caller) -> Option<AssertionEvidence> {
        if matches!(caller, Caller::External(_)) && !self.released(source) {
            return None;
        }
        let stale = !self.fresh(source);
        if stale && matches!(caller, Caller::External(_)) {
            return None;
        }
        Some(AssertionEvidence::Passage {
            source: source.clone(),
            stale,
        })
    }
    // Released means the human explicitly labelled these exact bytes Cloud allowed.
    fn released(&self, source: &SourceRef) -> bool {
        let key = (source.path.clone(), source.document_hash.clone());
        if let Some(known) = self.released.borrow().get(&key) {
            return *known;
        }
        let released = self
            .state
            .reviewed_sources
            .get(&key)
            .is_some_and(|(label, _)| *label == Sensitivity::CloudAllowed)
            && scopes::source_sensitivity(self.state, &source.path, &source.document_hash)
                == Sensitivity::CloudAllowed;
        self.released.borrow_mut().insert(key, released);
        released
    }
}

struct Pending {
    topics: BTreeMap<Id, (Topic, PendingRef)>,
    claims: Vec<(Claim, PendingRef)>,
    connections: Vec<(Connection, PendingRef)>,
    discoveries: Vec<PendingDiscovery>,
}
fn pending(ctx: &Context, state: &State) -> WResult<Pending> {
    let mut out = Pending {
        topics: BTreeMap::new(),
        claims: vec![],
        connections: vec![],
        discoveries: vec![],
    };
    let add =
        |out: &mut Pending, candidate: &Candidate, decision: &PendingRef| -> (u32, u32, u32, u32) {
            match candidate {
                Candidate::Topic { topic } => {
                    if state.topics.contains_key(&topic.id) || out.topics.contains_key(&topic.id) {
                        return (0, 0, 0, 0);
                    }
                    out.topics
                        .insert(topic.id.clone(), (topic.clone(), decision.clone()));
                    (1, 0, 0, 0)
                }
                Candidate::Claim { claim } | Candidate::Correction { claim, .. } => {
                    if state.claims.contains_key(&claim.revision_id) {
                        return (0, 0, 0, 0);
                    }
                    out.claims.push((claim.clone(), decision.clone()));
                    (0, 1, 0, 0)
                }
                Candidate::Connection { connection } => {
                    if state.connections.contains_key(&connection.id) {
                        return (0, 0, 0, 0);
                    }
                    out.connections.push((connection.clone(), decision.clone()));
                    (0, 0, 1, 0)
                }
                Candidate::UnresolvedIdentity { .. } => (0, 0, 0, 1),
            }
        };
    for proposal in proposals::pending_proposals(ctx)? {
        let decision = PendingRef {
            kind: PendingKind::Proposal,
            id: proposal.id.clone(),
        };
        let mut counts = (0, 0, 0, 0);
        for candidate in &proposal.candidates {
            let (t, a, r, n) = add(&mut out, candidate, &decision);
            counts = (counts.0 + t, counts.1 + a, counts.2 + r, counts.3 + n);
        }
        out.discoveries.push(PendingDiscovery {
            decision,
            state: PendingState::AwaitingDecision,
            topics: counts.0,
            assertions: counts.1,
            relationships: counts.2,
            needs_input: counts.3,
            created_at: proposal.created_at,
        });
    }
    for job in setup::projected_jobs(ctx, state)? {
        let decision = PendingRef {
            kind: PendingKind::SetupDraft,
            id: job.job_id.clone(),
        };
        let mut counts = (0, 0, 0, 0);
        for item in job.graph.items.iter().filter(|item| {
            matches!(
                item.disposition,
                DraftDisposition::Pending | DraftDisposition::Deferred
            )
        }) {
            let (t, a, r, _) = add(&mut out, &item.candidate, &decision);
            counts = (counts.0 + t, counts.1 + a, counts.2 + r, 0);
        }
        let waiting = counts.0 + counts.1 + counts.2;
        let state = match job.status {
            SetupJobStatus::NeedsConsent
            | SetupJobStatus::Paused
            | SetupJobStatus::FinishedWithGaps => PendingState::NeedsAttention,
            SetupJobStatus::Running => PendingState::Analysing,
            SetupJobStatus::Ready if !job.coverage.outstanding.is_empty() => {
                PendingState::NeedsAttention
            }
            _ if waiting > 0 => PendingState::AwaitingDecision,
            _ => continue,
        };
        out.discoveries.push(PendingDiscovery {
            decision,
            state,
            topics: counts.0,
            assertions: counts.1,
            relationships: counts.2,
            needs_input: job.graph.issues.iter().filter(|i| !i.deferred).count() as u32,
            created_at: job.created_at,
        });
    }
    out.discoveries
        .sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(out)
}

fn evidence(
    state: &State,
    citations: &Citations,
    claim: &Claim,
    caller: Caller,
    withheld: &mut Withheld,
) -> Vec<AssertionEvidence> {
    let mut out = vec![];
    for source in &claim.sources {
        match source {
            Evidence::Passage { source } => {
                if let Some(passage) = citations.passage(source, caller) {
                    out.push(passage)
                } else {
                    withheld.passages += 1;
                }
            }
            // The human's own words stay with the human.
            Evidence::UserStatement { statement_id } => {
                if caller == Caller::Human
                    && let Some(statement) = state.statements.get(statement_id)
                {
                    out.push(AssertionEvidence::Statement {
                        statement: statement.clone(),
                    });
                }
            }
        }
    }
    out
}
fn passages(
    citations: &Citations,
    sources: &[SourceRef],
    caller: Caller,
    withheld: &mut Withheld,
) -> Vec<AssertionEvidence> {
    let mut out = vec![];
    for source in sources {
        if let Some(passage) = citations.passage(source, caller) {
            out.push(passage)
        } else {
            withheld.passages += 1;
        }
    }
    out
}
fn has_stale(evidence: &[AssertionEvidence]) -> bool {
    evidence
        .iter()
        .any(|e| matches!(e, AssertionEvidence::Passage { stale: true, .. }))
}
fn assertion(
    claim: &Claim,
    status: KnowledgeStatus,
    review: ReviewState,
    evidence: Vec<AssertionEvidence>,
    pending: Option<PendingRef>,
) -> Assertion {
    Assertion {
        claim_id: claim.id.clone(),
        revision_id: claim.revision_id.clone(),
        predicate: claim.predicate.clone(),
        value: claim.value.clone(),
        uncertainty: claim.uncertainty.clone(),
        valid_from: claim.valid_from.clone(),
        valid_to: claim.valid_to.clone(),
        recorded_at: claim.recorded_at.clone(),
        status,
        review,
        check: None,
        evidence,
        pending,
    }
}
/// Effective dates are inclusive. Missing bounds leave that end open.
pub(super) fn periods_disjoint(
    a_from: Option<&str>,
    a_to: Option<&str>,
    b_from: Option<&str>,
    b_to: Option<&str>,
) -> bool {
    a_to.zip(b_from).is_some_and(|(end, start)| end < start)
        || b_to.zip(a_from).is_some_and(|(end, start)| end < start)
}
pub(super) fn item_id(kind: CheckKind, topic: &str, ids: &[Id]) -> Id {
    let mut ids = ids.to_vec();
    ids.sort();
    stable_id(&format!(
        "topics-validation-v1:{kind:?}:{topic}:{}",
        ids.join(",")
    ))
}
/// Classify one topic's assertions. Conflicts are different values for one
/// predicate that could hold at the same time; non-overlapping periods are a
/// timeline instead. Unconfirmed single assertions are stale, uncertain or
/// simply unverified, in that order.
fn check(topic: &mut TopicKnowledge, deferred: &BTreeMap<Id, Timestamp>) {
    let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, a) in topic.assertions.iter().enumerate() {
        groups.entry(norm(&a.predicate)).or_default().push(index);
    }
    let mut conflicted = BTreeSet::new();
    for members in groups.values() {
        let differing = |x: usize, y: usize| {
            norm(&topic.assertions[x].value) != norm(&topic.assertions[y].value)
        };
        let mut changed = false;
        for (i, &x) in members.iter().enumerate() {
            for &y in &members[i + 1..] {
                if !differing(x, y) {
                    continue;
                }
                let (a, b) = (&topic.assertions[x], &topic.assertions[y]);
                if periods_disjoint(
                    a.valid_from.as_deref(),
                    a.valid_to.as_deref(),
                    b.valid_from.as_deref(),
                    b.valid_to.as_deref(),
                ) {
                    changed = true;
                    continue;
                }
                let ids = vec![
                    topic.assertions[x].revision_id.clone(),
                    topic.assertions[y].revision_id.clone(),
                ];
                topic.validation.push(ValidationItem {
                    id: item_id(CheckKind::Conflicting, &topic.topic.id, &ids),
                    kind: CheckKind::Conflicting,
                    predicate: topic.assertions[x].predicate.clone(),
                    revision_ids: ids,
                    connection_id: None,
                    deferred: false,
                });
                conflicted.extend([x, y]);
            }
        }
        if changed {
            let mut ordered = members.clone();
            ordered.sort_by(|&x, &y| {
                let key = |i: usize| {
                    let a = &topic.assertions[i];
                    (
                        a.valid_from.clone().or(a.valid_to.clone()),
                        a.recorded_at.clone(),
                    )
                };
                key(x).cmp(&key(y))
            });
            topic.timelines.push(Timeline {
                predicate: topic.assertions[members[0]].predicate.clone(),
                revision_ids: ordered
                    .iter()
                    .map(|&i| topic.assertions[i].revision_id.clone())
                    .collect(),
            });
        }
    }
    let topic_id = topic.topic.id.clone();
    for (index, a) in topic.assertions.iter_mut().enumerate() {
        if conflicted.contains(&index) {
            a.check = Some(CheckKind::Conflicting);
            continue;
        }
        let kind = if has_stale(&a.evidence) {
            CheckKind::Stale
        } else if a.review == ReviewState::ConfirmedByYou {
            continue;
        } else if a.uncertainty.is_some() {
            CheckKind::Uncertain
        } else {
            if a.status == KnowledgeStatus::Pending {
                continue;
            }
            a.check = Some(CheckKind::Unverified);
            continue;
        };
        a.check = Some(kind);
        topic.validation.push(ValidationItem {
            id: item_id(kind, &topic_id, std::slice::from_ref(&a.revision_id)),
            kind,
            predicate: a.predicate.clone(),
            revision_ids: vec![a.revision_id.clone()],
            connection_id: None,
            deferred: false,
        });
    }
    for item in &mut topic.validation {
        item.deferred = deferred.contains_key(&item.id);
    }
    topic.validation.sort_by_key(|i| (i.deferred, i.kind));
    let mut counts = no_checks();
    for item in &topic.validation {
        if item.deferred {
            counts.deferred += 1;
            continue;
        }
        match item.kind {
            CheckKind::Conflicting => counts.conflicting += 1,
            CheckKind::Stale => counts.stale += 1,
            CheckKind::Uncertain => counts.uncertain += 1,
            CheckKind::Unverified => {}
        }
    }
    counts.unverified = topic
        .assertions
        .iter()
        .filter(|a| a.check == Some(CheckKind::Unverified))
        .count() as u32;
    topic.checks = counts;
}

pub(super) fn project(ctx: &Context, state: &State, caller: Caller) -> WResult<Projection> {
    let citations = Citations {
        ctx,
        state,
        fresh: RefCell::default(),
        released: RefCell::default(),
    };
    let human = caller == Caller::Human;
    let waiting = if human {
        pending(ctx, state)?
    } else {
        Pending {
            topics: BTreeMap::new(),
            claims: vec![],
            connections: vec![],
            discoveries: vec![],
        }
    };
    let deferrals = if human {
        deferred(ctx)?
    } else {
        BTreeMap::new()
    };
    let blank =
        |topic: &Topic, status: KnowledgeStatus, pending: Option<PendingRef>| TopicKnowledge {
            topic: topic.clone(),
            status,
            pending,
            assertions: vec![],
            relationships: vec![],
            validation: vec![],
            timelines: vec![],
            earlier: vec![],
            context: vec![],
            checks: no_checks(),
            withheld: Withheld {
                assertions: 0,
                relationships: 0,
                passages: 0,
            },
            warnings: vec![],
        };
    let mut topics: BTreeMap<Id, TopicKnowledge> = BTreeMap::new();
    for topic in state.topics.values() {
        // Anything not explicitly released, or tainted by private evidence, stays local.
        if let Caller::External(allowed) = caller
            && (!allowed.contains(&topic.id)
                || topic.sensitivity != Sensitivity::CloudAllowed
                || scopes::topic_restricted(state, &topic.id))
        {
            continue;
        }
        topics.insert(
            topic.id.clone(),
            blank(topic, KnowledgeStatus::Accepted, None),
        );
    }
    for (topic, decision) in waiting.topics.values() {
        topics.insert(
            topic.id.clone(),
            blank(topic, KnowledgeStatus::Pending, Some(decision.clone())),
        );
    }
    let superseded: BTreeSet<&Id> = state
        .claims
        .values()
        .flat_map(|c| c.supersedes.iter())
        .chain(state.orphan_superseded.iter())
        .collect();
    for claim in state.claims.values() {
        let Some(topic) = topics.get_mut(&claim.subject_id) else {
            continue;
        };
        if superseded.contains(&claim.revision_id) {
            if human {
                let mut none = Withheld {
                    assertions: 0,
                    relationships: 0,
                    passages: 0,
                };
                topic.earlier.push(assertion(
                    claim,
                    KnowledgeStatus::Accepted,
                    ReviewState::EarlierUnderstanding,
                    evidence(state, &citations, claim, caller, &mut none),
                    None,
                ));
            }
            continue;
        }
        let cited = evidence(state, &citations, claim, caller, &mut topic.withheld);
        // Every citation must be current and released for external retrieval.
        if !human && (cited.is_empty() || cited.len() != claim.sources.len()) {
            topic.withheld.assertions += 1;
            continue;
        }
        let cited = if human { cited } else { vec![] };
        let review = setup::claim_review(state, claim, false).state;
        topic.assertions.push(assertion(
            claim,
            KnowledgeStatus::Accepted,
            review,
            cited,
            None,
        ));
    }
    for (claim, decision) in &waiting.claims {
        let Some(topic) = topics.get_mut(&claim.subject_id) else {
            continue;
        };
        let cited = evidence(state, &citations, claim, caller, &mut topic.withheld);
        topic.assertions.push(assertion(
            claim,
            KnowledgeStatus::Pending,
            ReviewState::NeedsInput,
            cited,
            Some(decision.clone()),
        ));
    }
    for topic in topics.values_mut() {
        topic.assertions.sort_by(|a, b| {
            (norm(&a.predicate), &a.recorded_at).cmp(&(norm(&b.predicate), &b.recorded_at))
        });
        topic
            .earlier
            .sort_by(|a, b| b.recorded_at.cmp(&a.recorded_at));
        check(topic, &deferrals);
        if human && topic.status == KnowledgeStatus::Accepted {
            topic.context = setup::context_notes(state, &topic.topic.id);
        }
    }
    let mut edges = vec![];
    let accepted = state.connections.values().map(|c| (c, None)).chain(
        waiting
            .connections
            .iter()
            .map(|(c, d)| (c, Some(d.clone()))),
    );
    for (connection, decision) in accepted {
        let endpoint = |e: &Endpoint| match e {
            Endpoint::Topic { id } => Some(id.clone()),
            Endpoint::Claim { id } => topics
                .values()
                .find(|t| {
                    t.assertions
                        .iter()
                        .any(|a| &a.revision_id == id || &a.claim_id == id)
                })
                .map(|t| t.topic.id.clone()),
            Endpoint::Source { .. } => None,
        };
        let (Some(from), Some(to)) = (endpoint(&connection.from), endpoint(&connection.to)) else {
            continue;
        };
        if !topics.contains_key(&from) || !topics.contains_key(&to) {
            // The other end is private or unknown to this caller.
            for id in [&from, &to] {
                if let Some(topic) = topics.get_mut(id) {
                    topic.withheld.relationships += 1;
                }
            }
            continue;
        }
        let mut withheld = Withheld {
            assertions: 0,
            relationships: 0,
            passages: 0,
        };
        let cited = passages(&citations, &connection.sources, caller, &mut withheld);
        if !human && (cited.is_empty() || cited.len() != connection.sources.len()) {
            for id in [&from, &to] {
                topics.get_mut(id).unwrap().withheld.relationships += 1;
            }
            continue;
        }
        let cited = if human { cited } else { vec![] };
        let status = if decision.is_some() {
            KnowledgeStatus::Pending
        } else {
            KnowledgeStatus::Accepted
        };
        let review = if decision.is_some() {
            ReviewState::NeedsInput
        } else {
            setup::connection_review(state, connection).state
        };
        let confirmed = review == ReviewState::ConfirmedByYou;
        let kind = if has_stale(&cited) {
            Some(CheckKind::Stale)
        } else if confirmed {
            None
        } else if connection.uncertainty.is_some() {
            Some(CheckKind::Uncertain)
        } else if status == KnowledgeStatus::Pending {
            None
        } else {
            Some(CheckKind::Unverified)
        };
        // A relationship is validated once, from the topic it starts at.
        if let Some(kind) = kind.filter(|k| *k != CheckKind::Unverified) {
            let id = item_id(kind, &from, std::slice::from_ref(&connection.id));
            let is_deferred = deferrals.contains_key(&id);
            let owner = topics.get_mut(&from).unwrap();
            owner.validation.push(ValidationItem {
                id,
                kind,
                predicate: connection.relation.clone(),
                revision_ids: vec![],
                connection_id: Some(connection.id.clone()),
                deferred: is_deferred,
            });
            if is_deferred {
                owner.checks.deferred += 1;
            } else if kind == CheckKind::Stale {
                owner.checks.stale += 1;
            } else {
                owner.checks.uncertain += 1;
            }
        }
        for (own, other, outgoing) in [(&from, &to, true), (&to, &from, false)] {
            if !outgoing && from == to {
                continue;
            }
            let reference = {
                let other = &topics[other].topic;
                TopicRef {
                    id: other.id.clone(),
                    name: other.name.clone(),
                    topic_kind: other.topic_kind,
                }
            };
            let topic = topics.get_mut(own).unwrap();
            topic.withheld.passages += withheld.passages;
            topic.relationships.push(Relationship {
                id: connection.id.clone(),
                relation: connection.relation.clone(),
                outgoing,
                other: reference,
                uncertainty: connection.uncertainty.clone(),
                valid_from: connection.valid_from.clone(),
                valid_to: connection.valid_to.clone(),
                status,
                review,
                check: kind,
                evidence: cited.clone(),
                pending: decision.clone(),
            });
        }
        edges.push(GraphEdge {
            id: connection.id.clone(),
            from,
            to,
            relation: connection.relation.clone(),
            status,
            inferred: connection.uncertainty.is_some(),
            confirmed,
            valid_from: connection.valid_from.clone(),
            valid_to: connection.valid_to.clone(),
            sources: cited.len() as u32,
            pending: decision,
        });
    }
    for topic in topics.values_mut() {
        topic.validation.sort_by_key(|i| (i.deferred, i.kind));
    }
    Ok(Projection {
        topics,
        edges,
        discoveries: waiting.discoveries,
        warnings: if human {
            state.warnings.clone()
        } else {
            vec![]
        },
    })
}

fn graph(ctx: &Context, projection: Projection) -> KnowledgeGraph {
    let mut checks = no_checks();
    let nodes: Vec<_> = projection
        .topics
        .into_values()
        .map(|topic| {
            checks.conflicting += topic.checks.conflicting;
            checks.stale += topic.checks.stale;
            checks.uncertain += topic.checks.uncertain;
            checks.unverified += topic.checks.unverified;
            checks.deferred += topic.checks.deferred;
            let open = topic.checks.conflicting + topic.checks.stale + topic.checks.uncertain;
            GraphNode {
                verified: topic.status == KnowledgeStatus::Accepted
                    && open == 0
                    && !topic.assertions.is_empty()
                    && topic
                        .assertions
                        .iter()
                        .all(|a| a.review == ReviewState::ConfirmedByYou),
                assertions: topic.assertions.len() as u32,
                status: topic.status,
                checks: topic.checks,
                pending: topic.pending,
                topic: topic.topic,
            }
        })
        .collect();
    KnowledgeGraph {
        root_id: ctx.root_id.clone(),
        nodes,
        edges: projection.edges,
        pending: projection.discoveries,
        checks,
        warnings: projection.warnings,
    }
}

fn search(projection: &Projection, query: &str, limit: Option<u32>) -> TopicSearchResult {
    let query = norm(query);
    let terms: Vec<&str> = query.split(' ').filter(|t| !t.is_empty()).collect();
    let all = |text: &str| {
        let text = norm(text);
        terms.iter().all(|term| text.contains(term))
    };
    let mut hits: Vec<(u8, TopicHit)> = vec![];
    for topic in projection.topics.values() {
        let name = norm(&topic.topic.name);
        let found = if terms.is_empty() || name == query {
            Some((0, MatchField::Name, None))
        } else if name.starts_with(&query) {
            Some((1, MatchField::Name, None))
        } else if all(&topic.topic.name) {
            Some((2, MatchField::Name, None))
        } else if let Some(alias) = topic.topic.aliases.iter().find(|a| all(a)) {
            Some((3, MatchField::Alias, Some(alias.clone())))
        } else {
            topic
                .assertions
                .iter()
                .find(|a| all(&format!("{} {}", a.predicate, a.value)))
                .map(|a| {
                    (
                        4,
                        MatchField::Assertion,
                        Some(format!("{}: {}", a.predicate, a.value)),
                    )
                })
        };
        if let Some((rank, matched, snippet)) = found {
            hits.push((
                rank,
                TopicHit {
                    topic: topic.topic.clone(),
                    status: topic.status,
                    matched,
                    snippet,
                    checks: topic.checks.clone(),
                },
            ));
        }
    }
    hits.sort_by(|a, b| {
        (a.0, norm(&a.1.topic.name), &a.1.topic.id).cmp(&(
            b.0,
            norm(&b.1.topic.name),
            &b.1.topic.id,
        ))
    });
    let total = hits.len() as u32;
    let limit = limit.map_or(SEARCH_DEFAULT, |l| (l as usize).clamp(1, SEARCH_MAX));
    TopicSearchResult {
        hits: hits.into_iter().take(limit).map(|(_, hit)| hit).collect(),
        total,
    }
}

impl WorkspaceService {
    /// The whole workspace as topics and typed relationships, with what is
    /// pending and what is worth checking. Human desktop projection.
    pub fn knowledge_graph(&self, request: RootRequest) -> WResult<KnowledgeGraph> {
        self.with(&request.root_id, |ctx| {
            let state = load(ctx)?;
            Ok(graph(ctx, project(ctx, &state, Caller::Human)?))
        })
    }
    /// Topic lookup over names, aliases and accepted or pending assertions.
    pub fn topic_search(&self, request: TopicSearchRequest) -> WResult<TopicSearchResult> {
        self.with(&request.root_id, |ctx| {
            let state = load(ctx)?;
            Ok(search(
                &project(ctx, &state, Caller::Human)?,
                &request.query,
                request.limit,
            ))
        })
    }
    pub fn topic_knowledge(&self, request: TopicGetRequest) -> WResult<TopicKnowledge> {
        self.with(&request.root_id, |ctx| {
            valid_id(&request.topic_id)?;
            let state = load(ctx)?;
            project(ctx, &state, Caller::Human)?
                .topics
                .remove(&request.topic_id)
                .ok_or_else(|| error("invalid_input", "Topic does not exist"))
        })
    }
    /// Put one validation item aside, or bring it back. This is triage only:
    /// it changes no assertion and is kept with the workspace's app data.
    pub fn validation_defer(&self, request: ValidationDeferRequest) -> WResult<TopicKnowledge> {
        self.with(&request.root_id, |ctx| {
            valid_id(&request.topic_id)?;
            valid_id(&request.item_id)?;
            let state = load(ctx)?;
            let mut deferrals = deferred(ctx)?;
            if request.deferred {
                let known = project(ctx, &state, Caller::Human)?
                    .topics
                    .get(&request.topic_id)
                    .is_some_and(|t| t.validation.iter().any(|i| i.id == request.item_id));
                if !known {
                    return Err(error(
                        "conflict",
                        "This check changed; refresh the topic before deferring it",
                    ));
                }
                deferrals.insert(request.item_id.clone(), now());
            } else {
                deferrals.remove(&request.item_id);
            }
            save_deferred(ctx, &deferrals)?;
            project(ctx, &state, Caller::Human)?
                .topics
                .remove(&request.topic_id)
                .ok_or_else(|| error("invalid_input", "Topic does not exist"))
        })
    }

    pub fn retrieval_settings_get(&self, request: RootRequest) -> WResult<RetrievalSettings> {
        self.with(&request.root_id, |ctx| Ok(retrieval_settings(ctx)))
    }
    /// The human switches local MCP retrieval on or off for this notes folder.
    pub fn retrieval_settings_set(
        &self,
        request: RetrievalSettingsSetRequest,
    ) -> WResult<RetrievalSettings> {
        self.with(&request.root_id, |ctx| {
            let settings = RetrievalSettings {
                enabled: request.enabled,
                updated_at: Some(now()),
            };
            save_json(&ctx.partition.join(RETRIEVAL), &settings)?;
            Ok(settings)
        })
    }
    // External callers never name a root, never trigger recovery and never
    // rotate the root capability: this path only reads.
    fn external<T>(
        &self,
        root_id: &str,
        operation: impl FnOnce(&Context, &State) -> WResult<T>,
    ) -> WResult<T> {
        let _guard = self.base.lock().map_err(internal)?;
        let unavailable = || {
            error(
                "no_root",
                "Topics has no available notes folder. Open Topics and choose one.",
            )
        };
        let root = self
            .base
            .configured_root()
            .map_err(|_| unavailable())?
            .ok_or_else(unavailable)?;
        let path = root.to_str().ok_or_else(unavailable)?.to_string();
        let epoch: RootEpoch = read_json(&self.base.app_data.join("workspace-root.json"))
            .map_err(|_| unavailable())?;
        if epoch.path != path || epoch.root_id != root_id {
            return Err(unavailable());
        }
        let ctx = Context {
            root,
            root_id: epoch.root_id,
            partition: self.base.app_data.join("workspaces").join(stable_id(&path)),
            interrupt_after: None,
            instance_id: self.instance_id.clone(),
        };
        if !retrieval_settings(&ctx).enabled {
            return Err(error(
                "forbidden",
                "Local MCP retrieval is turned off for this notes folder. Turn it on in Topics under Sources, MCP access.",
            ));
        }
        let state = load(&ctx)?;
        if !state.warnings.is_empty() {
            return Err(error("forbidden", "Knowledge integrity is unknown"));
        }
        operation(&ctx, &state)
    }
    /// Topic lookup for a local MCP client: accepted, released knowledge only.
    pub fn retrieval_search(
        &self,
        root_id: &str,
        allowed: &BTreeSet<Id>,
        request: RetrievalSearchRequest,
    ) -> WResult<TopicSearchResult> {
        self.external(root_id, |ctx, state| {
            Ok(search(
                &project(ctx, state, Caller::External(allowed))?,
                &request.query,
                request.limit,
            ))
        })
    }
    /// Topic detail for a local MCP client. An ineligible topic is reported
    /// exactly like one that does not exist.
    pub fn retrieval_topic(
        &self,
        root_id: &str,
        allowed: &BTreeSet<Id>,
        request: RetrievalTopicRequest,
    ) -> WResult<TopicKnowledge> {
        self.external(root_id, |ctx, state| {
            valid_id(&request.topic_id)?;
            project(ctx, state, Caller::External(allowed))?
                .topics
                .remove(&request.topic_id)
                .ok_or_else(|| error("invalid_input", "Topic is not available"))
        })
    }
}
fn retrieval_settings(ctx: &Context) -> RetrievalSettings {
    read_json(&ctx.partition.join(RETRIEVAL)).unwrap_or(RetrievalSettings {
        enabled: false,
        updated_at: None,
    })
}
