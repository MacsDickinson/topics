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
    let (evidence, incomplete_history) = setup::pilot_evidence(ctx)?;
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
    let mut occurrences: Vec<PilotOccurrence> = vec![];
    if available {
        recover(ctx)?;
        let inventory = setup::inventory::pilot_inventory(ctx)?;
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
        for (job, runs) in &evidence {
            for (spec, batch) in job.manifest.batches.iter().zip(&job.batches) {
                for source in &spec.sources {
                    if let Some(entry) = sources.get_mut(&source.source.path)
                        && entry.current_hash.as_ref() == Some(&source.source.hash)
                        && entry.status != PilotSourceStatus::Checked
                        && (matches!(
                            batch.status,
                            SetupBatchStatus::Failed
                                | SetupBatchStatus::Cancelled
                                | SetupBatchStatus::Running
                        ) || (batch.status == SetupBatchStatus::Completed
                            && !runs.iter().any(|(_, versions, _)| {
                                versions.iter().any(|version| {
                                    version.path == source.source.path
                                        && version.hash == source.source.hash
                                })
                            })))
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
        let current_job_id = evidence.last().map(|(job, _)| job.job_id.as_str());
        let all_runs: Vec<_> = evidence.iter().flat_map(|(_, runs)| runs.iter()).collect();
        for (job, job_runs) in &evidence {
            if Some(job.job_id.as_str()) != current_job_id {
                continue;
            }
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
                let refs: Vec<_> = refs
                    .into_iter()
                    .filter(|source| {
                        sources.get(&source.path).is_some_and(|status| {
                            status.status == PilotSourceStatus::Checked
                                && status.current_hash.as_ref() == Some(&source.document_hash)
                        })
                    })
                    .collect();
                if refs.is_empty() {
                    continue;
                }
                let mut matching_times = job_runs.iter().filter(|(_, versions, candidates)| {
                    candidates.iter().any(|candidate| matches!(candidate, Candidate::Claim { claim: raw } if raw.subject_id == claim.subject_id && raw.predicate == claim.predicate && raw.value == claim.value))
                        && versions.iter().any(|version| refs.iter().any(|source| source.path == version.path && source.document_hash == version.hash))
                }).map(|(at, _, _)| at);
                let Some(first_current) = matching_times.next() else {
                    continue;
                };
                if all_runs.iter().any(|(at, _, candidates)| at < first_current && candidates.iter().any(|candidate| matches!(candidate, Candidate::Claim { claim: raw } if raw.subject_id == claim.subject_id && raw.predicate == claim.predicate && raw.value == claim.value))) { continue; }
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
        // Analysis checkpoints are committed under the same lock as rule approvals.
        // Compare against the committed rule operation, never the preview timestamp.
        let mut runs: Vec<_> = evidence.iter().flat_map(|(_, runs)| runs.iter()).collect();
        runs.sort_by(|a, b| a.0.cmp(&b.0));
        for rule in state.rules.values() {
            let path = format!("topics/rules/{}.md", rule.id);
            let Some(operation_id) = state.latest_operations.get(&path) else {
                continue;
            };
            let Some(approval) = state
                .operations
                .iter()
                .find(|op| &op.operation.id == operation_id && op.undo_of.is_none())
            else {
                continue;
            };
            let Some(corrected) = state.claims.get(&rule.source_revision_id) else {
                continue;
            };
            let Some(topic) = state.topics.get(&rule.topic_id) else {
                continue;
            };
            let mut prior_paths: BTreeSet<String> = corrected
                .supersedes
                .iter()
                .filter_map(|id| state.claims.get(id))
                .flat_map(|old| old.sources.iter())
                .filter_map(|e| match e {
                    Evidence::Passage { source } => Some(source.path.clone()),
                    _ => None,
                })
                .collect();
            let mut prior_passages = BTreeSet::new();
            for (at, versions, candidates) in &runs {
                if at <= &approval.operation.created_at {
                    prior_paths.extend(versions.iter().map(|v| v.path.clone()));
                    for candidate in candidates {
                        if let Candidate::Claim { claim } = candidate
                            && claim.subject_id == rule.topic_id
                            && claim.predicate == rule.predicate
                        {
                            prior_passages.extend(claim.sources.iter().filter_map(|e| match e {
                                Evidence::Passage { source } => Some(source.passage_hash.clone()),
                                _ => None,
                            }));
                        }
                    }
                    continue;
                }
                for candidate in candidates {
                    let Candidate::Claim { claim } = candidate else {
                        continue;
                    };
                    if claim.value != rule.match_value && claim.value != rule.replacement {
                        continue;
                    }
                    for source in claim.sources.iter().filter_map(|e| match e {
                        Evidence::Passage { source } => Some(source),
                        _ => None,
                    }) {
                        let mut comparable = claim.clone();
                        comparable.value = rule.match_value.clone();
                        comparable.sources = vec![Evidence::Passage {
                            source: source.clone(),
                        }];
                        if !proposals::rule_matches(rule, &comparable)
                            || !versions
                                .iter()
                                .any(|v| v.path == source.path && v.hash == source.document_hash)
                        {
                            continue;
                        }
                        let duplicate = prior_paths.contains(&source.path)
                            || prior_passages.contains(&source.passage_hash);
                        let occurrence_id = stable_id(&format!(
                            "pilot-rule:{}:{}:{}",
                            rule.id, claim.subject_id, source.path
                        ));
                        let observed = occurrences
                            .iter()
                            .any(|occurrence| occurrence.occurrence_id == occurrence_id)
                            || items.iter().any(|item| {
                                item.rule_id.as_ref() == Some(&rule.id)
                                    && item.sources.iter().any(|existing| {
                                        existing.path == source.path
                                            || existing.passage_hash == source.passage_hash
                                    })
                            });
                        if duplicate && !observed {
                            continue;
                        }
                        prior_paths.insert(source.path.clone());
                        prior_passages.insert(source.passage_hash.clone());
                        // A matching correct raw draft must not appear a second time as a generic card.
                        for item in &mut items {
                            if item.rule_id.is_none()
                                && item.topic_id == claim.subject_id
                                && item.predicate == claim.predicate
                                && item.value == claim.value
                            {
                                item.sources.retain(|existing| {
                                    !(existing.path == source.path
                                        && existing.document_hash == source.document_hash)
                                });
                            }
                        }
                        items.retain(|item| !item.sources.is_empty());
                        let visible = sources.get(&source.path).is_some_and(|status| {
                            status.status == PilotSourceStatus::Checked
                                && status.current_hash.as_ref() == Some(&source.document_hash)
                        });
                        if duplicate {
                            if visible
                                && let Some(item) = items.iter_mut().find(|item| {
                                    item.rule_id.as_ref() == Some(&rule.id)
                                        && item.sources.iter().any(|old| {
                                            old.path == source.path
                                                || old.passage_hash == source.passage_hash
                                        })
                                })
                            {
                                item.sources = vec![source.clone()];
                                item.revision_id = claim.revision_id.clone();
                                item.original_value = Some(claim.value.clone());
                            } else if visible && observed {
                                items.push(PilotItem {
                                    item_id: occurrence_id.clone(),
                                    topic_id: topic.id.clone(),
                                    topic_name: topic.name.clone(),
                                    revision_id: claim.revision_id.clone(),
                                    predicate: claim.predicate.clone(),
                                    value: rule.replacement.clone(),
                                    sources: vec![source.clone()],
                                    rule_id: Some(rule.id.clone()),
                                    original_value: Some(claim.value.clone()),
                                });
                            }
                            continue;
                        }
                        if visible {
                            items.push(PilotItem {
                                item_id: occurrence_id.clone(),
                                topic_id: topic.id.clone(),
                                topic_name: topic.name.clone(),
                                revision_id: claim.revision_id.clone(),
                                predicate: claim.predicate.clone(),
                                value: rule.replacement.clone(),
                                sources: vec![source.clone()],
                                rule_id: Some(rule.id.clone()),
                                original_value: Some(claim.value.clone()),
                            });
                        }
                        occurrences.push(PilotOccurrence {
                            rule_id: rule.id.clone(),
                            occurrence_id,
                            observed_at: at.clone(),
                            recurred: false,
                        });
                    }
                }
                // Any already analysed source version is a prior snapshot, even if
                // that analysis contained no eligible matching claim.
                prior_paths.extend(versions.iter().map(|version| version.path.clone()));
            }
        }
        // A recurrence means the wrong value actually reached the displayed projection.
        for occurrence in &mut occurrences {
            if let Some(rule) = state.rules.get(&occurrence.rule_id)
                && let Some(card) = items
                    .iter()
                    .find(|item| item.item_id == occurrence.occurrence_id)
            {
                occurrence.recurred = items.iter().any(|item| {
                    item.topic_id == rule.topic_id
                        && item.predicate == rule.predicate
                        && item.value == rule.match_value
                        && item.sources.iter().any(|source| {
                            card.sources.iter().any(|matched| {
                                source.path == matched.path
                                    && source.document_hash == matched.document_hash
                            })
                        })
                });
            }
        }
        occurrences
            .sort_by(|a, b| (&a.rule_id, &a.occurrence_id).cmp(&(&b.rule_id, &b.occurrence_id)));
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
    if incomplete_history {
        sources.insert(
            "Analysis history incomplete".into(),
            PilotSource {
                source_id: stable_id(&format!("pilot-history:{}", ctx.root_id)),
                path: "Analysis history incomplete".into(),
                current_hash: None,
                analysed_hash: None,
                status: PilotSourceStatus::Partial,
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
