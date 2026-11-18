//! Human-only, local projection of setup evidence.
use super::*;

impl WorkspaceService {
    pub fn pilot_briefing(&self, request: RootRequest) -> WResult<PilotBriefing> {
        let _guard = self.base.lock().map_err(internal)?;
        let (ctx, available) = match self.context(Some(&request.root_id)) {
            Ok(ctx) => (ctx, true),
            Err(e) if e.code == "no_root" => {
                let epoch: RootEpoch = read_json(&self.base.app_data.join("workspace-root.json"))?;
                let config: crate::Config = read_json(&self.base.app_data.join("config.json"))?;
                if config.notes_root.as_deref() != Some(Path::new(&epoch.path)) {
                    return Err(error("stale_scope", "Notes root changed"));
                }
                if epoch.root_id != request.root_id {
                    return Err(error("stale_scope", "Notes root changed"));
                }
                let partition = self
                    .base
                    .app_data
                    .join("workspaces")
                    .join(stable_id(&epoch.path));
                (
                    Context {
                        root: PathBuf::from(epoch.path),
                        root_id: epoch.root_id,
                        partition,
                        interrupt_after: None,
                        instance_id: self.instance_id.clone(),
                    },
                    false,
                )
            }
            Err(e) => return Err(e),
        };
        self.issue_pilot_view(project_with_availability(&ctx, available)?)
    }
}

/// Derive a briefing while the caller holds the workspace lock and context.
pub(super) fn project(ctx: &Context) -> WResult<PilotBriefing> {
    project_with_availability(ctx, true)
}

fn project_with_availability(ctx: &Context, available: bool) -> WResult<PilotBriefing> {
    let evidence = setup::pilot_evidence(ctx)?;
    let mut sources = BTreeMap::<String, PilotSource>::new();
    // Retain selected versions even if the root or an individual file disappears.
    for (job, runs) in &evidence {
        for batch in &job.manifest.batches {
            for entry in &batch.sources {
                sources
                    .entry(entry.source.path.clone())
                    .or_insert(PilotSource {
                        source_id: entry.source.source_id.clone(),
                        path: entry.source.path.clone(),
                        current_hash: None,
                        analysed_hash: None,
                        status: PilotSourceStatus::Unavailable,
                        last_success_at: None,
                    });
            }
        }
        for (at, versions, _) in runs {
            for version in versions {
                let source = sources.entry(version.path.clone()).or_insert(PilotSource {
                    source_id: version.source_id.clone(),
                    path: version.path.clone(),
                    current_hash: None,
                    analysed_hash: None,
                    status: PilotSourceStatus::Unavailable,
                    last_success_at: None,
                });
                if source.last_success_at.as_ref().is_none_or(|old| old < at) {
                    source.analysed_hash = Some(version.hash.clone());
                    source.last_success_at = Some(at.clone());
                }
            }
        }
    }
    let mut items = vec![];
    let mut rules = vec![];
    let occurrences: Vec<PilotOccurrence> = vec![];
    if available {
        recover(ctx)?;
        let inventory = setup::inventory::inventory(ctx, &[])?;
        for doc in inventory.documents {
            let entry = sources
                .entry(doc.source.path.clone())
                .or_insert(PilotSource {
                    source_id: doc.source.source_id.clone(),
                    path: doc.source.path.clone(),
                    current_hash: None,
                    analysed_hash: None,
                    status: PilotSourceStatus::Pending,
                    last_success_at: None,
                });
            entry.source_id = doc.source.source_id;
            entry.current_hash = Some(doc.source.hash);
            entry.status = if doc.source.sensitivity == Sensitivity::LocalOnly {
                PilotSourceStatus::Skipped
            } else if entry.analysed_hash == entry.current_hash && entry.last_success_at.is_some() {
                PilotSourceStatus::Checked
            } else {
                PilotSourceStatus::Pending
            };
        }
        for (job, _) in &evidence {
            for (spec, batch) in job.manifest.batches.iter().zip(&job.batches) {
                for source in &spec.sources {
                    if let Some(entry) = sources.get_mut(&source.source.path)
                        && entry.current_hash.as_ref() == Some(&source.source.hash)
                        && entry.status != PilotSourceStatus::Checked
                        && matches!(
                            batch.status,
                            SetupBatchStatus::Failed
                                | SetupBatchStatus::Cancelled
                                | SetupBatchStatus::Running
                        )
                    {
                        entry.status = PilotSourceStatus::Partial;
                    }
                }
            }
        }
        // The most recent reviewed selection defines the briefing scope.
        if let Some((job, _)) = evidence.last() {
            let selected: BTreeSet<_> = job
                .manifest
                .batches
                .iter()
                .flat_map(|batch| batch.sources.iter().map(|s| s.source.path.as_str()))
                .collect();
            for source in sources.values_mut() {
                if source.current_hash.is_some() && !selected.contains(source.path.as_str()) {
                    source.status = PilotSourceStatus::Skipped;
                }
            }
        }
        let state = load(ctx)?;
        for rule in state.rules.values() {
            if let Some(topic) = state.topics.get(&rule.topic_id) {
                rules.push(PilotRule {
                    rule_id: rule.id.clone(),
                    topic_id: topic.id.clone(),
                    topic_name: topic.name.clone(),
                    predicate: rule.predicate.clone(),
                    match_value: rule.match_value.clone(),
                    replacement: rule.replacement.clone(),
                });
            }
        }
        for (job, _) in &evidence {
            for item in &job.graph.items {
                if item.disposition == DraftDisposition::Dismissed {
                    continue;
                }
                let Candidate::Claim { claim } = &item.candidate else {
                    continue;
                };
                if state
                    .rules
                    .values()
                    .any(|rule| proposals::rule_matches(rule, claim))
                    || state
                        .claims
                        .values()
                        .any(|next| next.supersedes.contains(&claim.revision_id))
                {
                    continue;
                }
                let Some(topic) = state.topics.get(&claim.subject_id).or_else(|| {
                    job.graph.items.iter().find_map(|i| match &i.candidate {
                        Candidate::Topic { topic } if topic.id == claim.subject_id => Some(topic),
                        _ => None,
                    })
                }) else {
                    continue;
                };
                let refs: Vec<_> = claim
                    .sources
                    .iter()
                    .filter_map(|s| match s {
                        Evidence::Passage { source } => Some(source.clone()),
                        _ => None,
                    })
                    .collect();
                if refs.is_empty() {
                    continue;
                }
                let identity = serde_json::to_string(&(
                    topic.id.as_str(),
                    claim.predicate.as_str(),
                    claim.value.as_str(),
                ))
                .map_err(internal)?;
                let item_id = stable_id(&format!("pilot-item:{identity}"));
                if let Some(existing) = items
                    .iter_mut()
                    .find(|entry: &&mut PilotItem| entry.item_id == item_id)
                {
                    for source in refs {
                        if !existing.sources.contains(&source) {
                            existing.sources.push(source);
                        }
                    }
                    continue;
                }
                items.push(PilotItem {
                    item_id,
                    topic_id: topic.id.clone(),
                    topic_name: topic.name.clone(),
                    revision_id: claim.revision_id.clone(),
                    predicate: claim.predicate.clone(),
                    value: claim.value.clone(),
                    sources: refs,
                    rule_id: None,
                    original_value: None,
                });
            }
        }
        // Preserve the original observation while applying an approved, still-active
        // scoped rule only to an independent source, never an edited prior source.
        for (_, runs) in &evidence {
            for (at, _, candidates) in runs {
                for candidate in candidates {
                    let Candidate::Claim { claim } = candidate else {
                        continue;
                    };
                    let Some(rule) = state.rules.values().find(|rule| {
                        if claim.value != rule.match_value && claim.value != rule.replacement {
                            return false;
                        }
                        let mut comparable = claim.clone();
                        comparable.value = rule.match_value.clone();
                        proposals::rule_matches(rule, &comparable)
                    }) else {
                        continue;
                    };
                    let Some(corrected) = state.claims.get(&rule.source_revision_id) else {
                        continue;
                    };
                    if at <= &corrected.recorded_at {
                        continue;
                    }
                    let previous: BTreeSet<_> = corrected
                        .supersedes
                        .iter()
                        .filter_map(|id| state.claims.get(id))
                        .flat_map(|old| old.sources.iter())
                        .filter_map(|e| match e {
                            Evidence::Passage { source } => Some(source.path.as_str()),
                            _ => None,
                        })
                        .collect();
                    let refs: Vec<_> = claim
                        .sources
                        .iter()
                        .filter_map(|e| match e {
                            Evidence::Passage { source }
                                if !previous.contains(source.path.as_str()) =>
                            {
                                let mut one = claim.clone();
                                one.value = rule.match_value.clone();
                                one.sources = vec![Evidence::Passage {
                                    source: source.clone(),
                                }];
                                proposals::rule_matches(rule, &one).then(|| source.clone())
                            }
                            _ => None,
                        })
                        .collect();
                    if refs.is_empty() {
                        continue;
                    }
                    let Some(topic) = state.topics.get(&claim.subject_id) else {
                        continue;
                    };
                    let identity = format!(
                        "pilot-rule:{}:{}:{}",
                        rule.id, claim.subject_id, refs[0].path
                    );
                    let item_id = stable_id(&identity);
                    if items.iter().any(|item| {
                        item.rule_id.as_ref() == Some(&rule.id)
                            && item.topic_id == claim.subject_id
                            && item.sources.iter().any(|existing| {
                                refs.iter().any(|source| {
                                    existing.path == source.path
                                        || existing.passage_hash == source.passage_hash
                                })
                            })
                    }) {
                        continue;
                    }
                    items.push(PilotItem {
                        item_id,
                        topic_id: topic.id.clone(),
                        topic_name: topic.name.clone(),
                        revision_id: claim.revision_id.clone(),
                        predicate: claim.predicate.clone(),
                        value: rule.replacement.clone(),
                        sources: refs,
                        rule_id: Some(rule.id.clone()),
                        original_value: Some(claim.value.clone()),
                    });
                }
            }
        }
        items.sort_by(|a, b| {
            (&a.topic_name, &a.topic_id, &a.item_id).cmp(&(&b.topic_name, &b.topic_id, &b.item_id))
        });
        items.dedup_by(|a, b| a.item_id == b.item_id);
    }
    if !available && sources.is_empty() {
        sources.insert(
            ctx.root.to_string_lossy().into_owned(),
            PilotSource {
                source_id: stable_id(&format!("unavailable-root:{}", ctx.root_id)),
                path: ctx.root.to_string_lossy().into_owned(),
                current_hash: None,
                analysed_hash: None,
                status: PilotSourceStatus::Unavailable,
                last_success_at: None,
            },
        );
    }
    let sources: Vec<_> = sources.into_values().collect();
    let briefing_token = hash(
        &serde_json::to_vec(&(&ctx.root_id, &sources, &items, &rules, &occurrences))
            .map_err(internal)?,
    );
    Ok(PilotBriefing {
        root_id: ctx.root_id.clone(),
        generated_at: now(),
        briefing_token,
        sources,
        items,
        rules,
        occurrences,
    })
}
