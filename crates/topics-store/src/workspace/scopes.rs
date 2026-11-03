use super::*;
#[derive(Clone, Serialize, Deserialize)]
pub(super) struct Scope {
    pub preview: ScopePreview,
}
#[derive(Clone, Serialize, Deserialize)]
pub(super) struct Run {
    pub root_id: Id,
    pub run_id: Id,
    pub scope_id: Id,
    pub active: bool,
    pub message_id: Option<Id>,
    pub topic_id: Option<Id>,
}
const MAX_SOURCES: usize = 100;
const MAX_BYTES: usize = 2 * 1024 * 1024;

pub(super) fn source_bytes(ctx: &Context, path: &str) -> WResult<Vec<u8>> {
    relative(path)?;
    if path.split('/').next() == Some("topics") || !path.to_lowercase().ends_with(".md") {
        return Err(error(
            "forbidden",
            "Only selected source Markdown is readable",
        ));
    }
    let mut at = ctx.root.clone();
    for part in path.split('/') {
        at.push(part);
        let m = fs::symlink_metadata(&at)
            .map_err(|_| error("stale_evidence", "Selected source is unavailable"))?;
        if m.file_type().is_symlink() {
            return Err(error("forbidden", "Source symlinks are excluded"));
        }
    }
    let m = fs::metadata(&at).map_err(internal)?;
    if !m.is_file() || m.len() > MAX_BYTES as u64 {
        return Err(error(
            "invalid_input",
            "Source must be a regular Markdown file within the size limit",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if m.nlink() > 1 {
            return Err(error(
                "forbidden",
                "Multiply linked source files are excluded",
            ));
        }
    }
    let bytes = fs::read(at).map_err(internal)?;
    if bytes.len() > MAX_BYTES || std::str::from_utf8(&bytes).is_err() {
        return Err(error(
            "invalid_input",
            "Source must be UTF-8 Markdown within the size limit",
        ));
    }
    Ok(bytes)
}
pub(super) fn validate_source(ctx: &Context, state: &State, source: &SourceRef) -> WResult<()> {
    let stale = || {
        error(
            "stale_evidence",
            "Citation changed, is unavailable, or has an invalid UTF-8 passage locator",
        )
    };
    let summary = state.sources.get(&source.source_id).ok_or_else(stale)?;
    if summary.path != source.path || summary.hash != source.document_hash {
        return Err(stale());
    }
    let bytes = source_bytes(ctx, &source.path).map_err(|_| stale())?;
    if hash(&bytes) != source.document_hash {
        return Err(stale());
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| stale())?;
    let start = usize::try_from(source.locator.start_byte).map_err(|_| stale())?;
    let end = usize::try_from(source.locator.end_byte).map_err(|_| stale())?;
    let passage = text
        .get(start..end)
        .filter(|_| start < end)
        .ok_or_else(stale)?;
    if hash(passage.as_bytes()) != source.passage_hash || passage != source.excerpt {
        return Err(stale());
    }
    Ok(())
}
pub(super) use topics_core::validate_date_range as validate_dates;

fn restricted_sources(state: &State) -> BTreeSet<Id> {
    let private: BTreeSet<_> = state
        .topics
        .values()
        .filter(|t| t.sensitivity == Sensitivity::LocalOnly)
        .map(|t| t.id.clone())
        .collect();
    let mut sources: BTreeSet<_> = state
        .sources
        .values()
        .filter(|s| s.sensitivity == Sensitivity::LocalOnly)
        .map(|s| s.source_id.clone())
        .collect();
    for c in state
        .claims
        .values()
        .filter(|c| private.contains(&c.subject_id))
    {
        for e in &c.sources {
            if let Evidence::Passage { source } = e {
                sources.insert(source.source_id.clone());
            }
        }
    }
    for c in state.connections.values() {
        if [&c.from, &c.to]
            .iter()
            .any(|e| matches!(e,Endpoint::Topic{id} if private.contains(id)))
        {
            sources.extend(c.sources.iter().map(|s| s.source_id.clone()));
        }
    }
    sources
}
fn path_sensitivity(state: &State, path: &str) -> Sensitivity {
    let mut label = Sensitivity::Unknown;
    for op in &state.operations {
        for target in &op.targets {
            for source in state.sources.values().filter(|s| s.path == path) {
                if target.path == format!("topics/sources/{}.md", source.source_id)
                    && state.hashes.get(&target.path) == Some(&Some(target.new_hash.clone()))
                {
                    label = source.sensitivity;
                }
            }
        }
    }
    label
}
pub(super) fn scope(ctx: &Context, id: &str) -> WResult<Scope> {
    valid_id(id)?;
    let scope: Scope = read_json(&ctx.partition.join("scopes").join(format!("{id}.json")))
        .map_err(|_| error("stale_scope", "Preview selection again"))?;
    if scope.preview.root_id != ctx.root_id {
        return Err(error("stale_scope", "Notes root changed; preview again"));
    }
    let state = load(ctx)?;
    let restricted = restricted_sources(&state);
    for s in &scope.preview.sources {
        if restricted.contains(&s.source_id) {
            return Err(error(
                "forbidden",
                "Selection contains restricted source context",
            ));
        }
        if state
            .sources
            .get(&s.source_id)
            .is_none_or(|current| current.sensitivity != s.sensitivity)
        {
            return Err(error(
                "stale_scope",
                "Source sensitivity changed; preview again",
            ));
        }
        if s.sensitivity == Sensitivity::LocalOnly {
            return Err(error("forbidden", "Selection contains restricted sources"));
        }
        let bytes = source_bytes(ctx, &s.path)
            .map_err(|_| error("stale_scope", "Selected sources changed; preview again"))?;
        if hash(&bytes) != s.hash {
            return Err(error(
                "stale_scope",
                "Selected sources changed; preview again",
            ));
        }
    }
    Ok(scope)
}
pub(super) fn authorized(ctx: &Context, scope_id: &str, run_id: &str) -> WResult<Scope> {
    valid_id(run_id)?;
    let run: Run = read_json(&ctx.partition.join("runs").join(format!("{run_id}.json")))
        .map_err(|_| error("forbidden", "Run capability is unavailable"))?;
    if !run.active || run.root_id != ctx.root_id || run.scope_id != scope_id {
        return Err(error(
            "forbidden",
            "Run capability is inactive or out of scope",
        ));
    }
    scope(ctx, scope_id)
}
impl WorkspaceService {
    pub fn scope_preview(&self, request: ScopePreviewRequest) -> WResult<ScopePreview> {
        self.with(&request.root_id, |ctx| {
            validate_dates(&request.spec.date_from, &request.spec.date_to)?;
            if request.spec.files.is_empty() && request.spec.folders.is_empty() {
                return Err(error("invalid_input", "Select explicit files or folders"));
            }
            let state = load(ctx)?;
            let mut files: BTreeSet<String> = BTreeSet::new();
            let mut excluded = vec![];
            for path in &request.spec.files {
                relative(path)?;
                files.insert(path.clone());
            }
            for folder in &request.spec.folders {
                relative(folder)?;
                if folder.split('/').next() == Some("topics") {
                    excluded.push(Exclusion {
                        path: folder.clone(),
                        reason: "App records are excluded".into(),
                    });
                    continue;
                }
                let mut at = ctx.root.clone();
                let mut unsafe_path = false;
                for part in folder.split('/') {
                    at.push(part);
                    if fs::symlink_metadata(&at).is_ok_and(|m| m.file_type().is_symlink()) {
                        unsafe_path = true;
                        break;
                    }
                }
                if unsafe_path {
                    excluded.push(Exclusion {
                        path: folder.clone(),
                        reason: "Symlinks are excluded".into(),
                    });
                    continue;
                }
                if !at.is_dir() {
                    return Err(error("invalid_input", "Selected folder is unavailable"));
                }
                for entry in walkdir::WalkDir::new(at)
                    .follow_links(false)
                    .into_iter()
                    .filter_entry(|e| e.file_name() != "topics")
                {
                    let entry = entry.map_err(|_| {
                        error("invalid_input", "Selected folder cannot be enumerated")
                    })?;
                    if entry.file_type().is_dir() {
                        continue;
                    }
                    let path = entry
                        .path()
                        .strip_prefix(&ctx.root)
                        .map_err(internal)?
                        .to_str()
                        .ok_or_else(|| error("invalid_input", "Selected paths must be UTF-8"))?
                        .replace(std::path::MAIN_SEPARATOR, "/");
                    files.insert(path);
                }
            }
            if files.len() > MAX_SOURCES {
                return Err(error(
                    "invalid_input",
                    "Selection exceeds 100 files; choose a smaller scope",
                ));
            }
            let mut sources = vec![];
            let op = new_id();
            let mut records = vec![];
            let mut total = 0;
            for path in files {
                if path.split('/').next() == Some("topics") || !path.to_lowercase().ends_with(".md")
                {
                    excluded.push(Exclusion {
                        path,
                        reason: "App records and non-Markdown files are excluded".into(),
                    });
                    continue;
                }
                let sensitivity = path_sensitivity(&state, &path);
                let restricted = restricted_sources(&state);
                let tainted = state
                    .sources
                    .values()
                    .any(|s| s.path == path && restricted.contains(&s.source_id));
                if sensitivity == Sensitivity::LocalOnly || tainted {
                    excluded.push(Exclusion {
                        path,
                        reason: "Local-only source".into(),
                    });
                    continue;
                }
                let bytes = match source_bytes(ctx, &path) {
                    Ok(v) => v,
                    Err(_) => {
                        excluded.push(Exclusion {
                            path,
                            reason: "Unavailable, unsafe, oversized or non-UTF-8 source".into(),
                        });
                        continue;
                    }
                };
                total += bytes.len();
                if total > MAX_BYTES {
                    return Err(error(
                        "invalid_input",
                        "Selection exceeds 2 MiB; choose a smaller scope",
                    ));
                }
                let h = hash(&bytes);
                let text = std::str::from_utf8(&bytes).map_err(internal)?;
                let d = crate::parse::document(&path, text, h.clone(), "0".into()).date;
                if request.spec.date_from.is_some() || request.spec.date_to.is_some() {
                    let Some(date) = d.as_ref() else {
                        excluded.push(Exclusion {
                            path,
                            reason: "Undated source".into(),
                        });
                        continue;
                    };
                    if request
                        .spec
                        .date_from
                        .as_ref()
                        .is_some_and(|from| date < from)
                        || request.spec.date_to.as_ref().is_some_and(|to| date > to)
                    {
                        excluded.push(Exclusion {
                            path,
                            reason: "Outside inclusive date range".into(),
                        });
                        continue;
                    }
                }
                let id = stable_id(&format!("{}\n{path}\n{h}", ctx.root.display()));
                let summary = SourceSummary {
                    source_id: id.clone(),
                    path,
                    hash: h,
                    date: d,
                    sensitivity,
                };
                if !state.sources.contains_key(&id) {
                    records.push((
                        format!("topics/sources/{id}.md"),
                        Record::new("source", &id, &op, &summary)?,
                    ));
                }
                sources.push(summary);
            }
            if !records.is_empty() {
                commit(
                    ctx,
                    "Register previewed sources",
                    records,
                    CommitIntent {
                        dependencies: vec![],
                        tombstones: vec![],
                        undo_of: None,
                        expected: None,
                        proposal: None,
                    },
                )?;
            }
            let preview = ScopePreview {
                scope_id: new_id(),
                root_id: ctx.root_id.clone(),
                sources,
                excluded,
            };
            save_json(
                &ctx.partition
                    .join("scopes")
                    .join(format!("{}.json", preview.scope_id)),
                &Scope {
                    preview: preview.clone(),
                },
            )?;
            Ok(preview)
        })
    }
    pub fn analysis_start(&self, request: AnalysisStartRequest) -> WResult<AnalysisStartResult> {
        self.with(&request.root_id, |ctx| {
            let s = scope(ctx, &request.scope_id)?;
            if s.preview.sources.is_empty() {
                return Err(error("invalid_input", "Selection has no eligible sources"));
            }
            if !request.confirm_unknown
                && s.preview
                    .sources
                    .iter()
                    .any(|s| s.sensitivity == Sensitivity::Unknown)
            {
                return Err(error(
                    "forbidden",
                    "Confirm the listed unknown sources for this run",
                ));
            }
            let run_id = new_id();
            let run = Run {
                root_id: ctx.root_id.clone(),
                run_id: run_id.clone(),
                scope_id: request.scope_id.clone(),
                active: true,
                message_id: None,
                topic_id: None,
            };
            save_json(
                &ctx.partition.join("runs").join(format!("{run_id}.json")),
                &run,
            )?;
            Ok(AnalysisStartResult { run_id })
        })
    }
    pub fn run_cancel(&self, request: RunCancelRequest) -> WResult<RunCancelResult> {
        self.with(&request.root_id, |ctx| {
            valid_id(&request.run_id)?;
            let path = ctx
                .partition
                .join("runs")
                .join(format!("{}.json", request.run_id));
            let mut run: Run =
                read_json(&path).map_err(|_| error("invalid_input", "Run does not exist"))?;
            if run.root_id != ctx.root_id {
                return Err(error(
                    "stale_scope",
                    "Run belongs to a previous root selection",
                ));
            }
            let cancelled = run.active;
            run.active = false;
            save_json(&path, &run)?;
            Ok(RunCancelResult { cancelled })
        })
    }
    pub fn scope_read(&self, request: ScopeReadRequest) -> WResult<ScopeReadResult> {
        self.with(&request.root_id, |ctx| {
            let s = authorized(ctx, &request.scope_id, &request.run_id)?;
            let mut sources = vec![];
            // Validate the entire immutable snapshot before returning any bytes.
            for summary in s.preview.sources {
                let content =
                    String::from_utf8(source_bytes(ctx, &summary.path)?).map_err(internal)?;
                let citation = SourceRef {
                    source_id: summary.source_id.clone(),
                    path: summary.path.clone(),
                    document_hash: summary.hash.clone(),
                    locator: Locator {
                        start_byte: 0,
                        end_byte: content.len() as u64,
                        heading: None,
                    },
                    passage_hash: hash(content.as_bytes()),
                    excerpt: content.clone(),
                };
                sources.push(ScopedSource {
                    summary,
                    content,
                    citation,
                });
            }
            Ok(ScopeReadResult { sources })
        })
    }
    pub fn source_context(&self, request: SourceContextRequest) -> WResult<TopicView> {
        self.with(&request.root_id,|ctx|{
        let selected=authorized(ctx,&request.scope_id,&request.run_id)?;let state=load(ctx)?;
        let mut view=topic_view(ctx,&state,&request.topic_id)?;
        if view.topic.sensitivity!=Sensitivity::CloudAllowed {return Err(error("forbidden","Topic context requires an explicit cloud-allowed label"));}
        let allowed:BTreeSet<_>=selected.preview.sources.iter().map(|s|s.source_id.clone()).collect();
        // A topic derived from a restricted source is denied as a unit, including its name.
        if view.history.iter().flat_map(|c|&c.sources).any(|e|matches!(e,Evidence::Passage{source} if state.sources.get(&source.source_id).is_none_or(|s|s.sensitivity==Sensitivity::LocalOnly))) {return Err(error("forbidden","Topic context contains restricted evidence"));}
        fn claim_allowed(c:&Claim,state:&State,allowed:&BTreeSet<Id>,topic:&str,depth:usize)->bool {
            depth<100 && !c.sources.is_empty() && c.sources.iter().all(|e|match e {
                Evidence::Passage{source}=>allowed.contains(&source.source_id),
                Evidence::UserStatement{statement_id}=>state.statements.get(statement_id).is_some_and(|s|s.topic_id==topic)&&!c.supersedes.is_empty()&&c.supersedes.iter().all(|id|state.claims.get(id).is_some_and(|old|claim_allowed(old,state,allowed,topic,depth+1))),
            })
        }
        let eligible=|c:&Claim|claim_allowed(c,&state,&allowed,&request.topic_id,0);
        view.current_claims.retain(eligible);view.history.retain(eligible);
        view.connections.retain(|c|c.sources.iter().all(|s|allowed.contains(&s.source_id)&&validate_source(ctx,&state,s).is_ok()) && endpoint_cloud(&state,&c.from) && endpoint_cloud(&state,&c.to));
        view.unresolved.retain(|c|matches!(c,Candidate::UnresolvedIdentity{sources,..} if sources.iter().all(|s|allowed.contains(&s.source_id))));
        view.warnings.clear();Ok(view)
    })
    }
    pub fn chat_start(&self, request: ChatStartRequest) -> WResult<ChatStartResult> {
        // Consent and bounded context checks precede persisting the user statement.
        let run = self.analysis_start(AnalysisStartRequest {
            root_id: request.root_id.clone(),
            scope_id: request.scope_id.clone(),
            confirm_unknown: request.confirm_unknown,
        })?;
        let context = self.source_context(SourceContextRequest {
            root_id: request.root_id.clone(),
            scope_id: request.scope_id.clone(),
            run_id: run.run_id.clone(),
            topic_id: request.topic_id.clone(),
        });
        if let Err(e) = context {
            let _ = self.run_cancel(RunCancelRequest {
                root_id: request.root_id.clone(),
                run_id: run.run_id,
            });
            return Err(e);
        }
        let statement = self.user_statement(UserStatementRequest {
            root_id: request.root_id.clone(),
            topic_id: request.topic_id.clone(),
            text: request.message,
        })?;
        self.with(&request.root_id, |ctx| {
            let path = ctx
                .partition
                .join("runs")
                .join(format!("{}.json", run.run_id));
            let mut value: Run = read_json(&path)?;
            value.message_id = Some(statement.id.clone());
            value.topic_id = Some(request.topic_id.clone());
            save_json(&path, &value)?;
            Ok(ChatStartResult {
                run_id: run.run_id,
                message_id: statement.id,
            })
        })
    }
}
fn endpoint_cloud(state: &State, endpoint: &Endpoint) -> bool {
    match endpoint {
        Endpoint::Topic { id } => state
            .topics
            .get(id)
            .is_some_and(|t| t.sensitivity == Sensitivity::CloudAllowed),
        Endpoint::Claim { id } => state
            .claims
            .values()
            .find(|c| &c.id == id || &c.revision_id == id)
            .is_some_and(|c| {
                state
                    .topics
                    .get(&c.subject_id)
                    .is_some_and(|t| t.sensitivity == Sensitivity::CloudAllowed)
            }),
        Endpoint::Source { source_id, .. } => state
            .sources
            .get(source_id)
            .is_some_and(|s| s.sensitivity != Sensitivity::LocalOnly),
    }
}

impl WorkspaceService {
    /// Human-only inventory. Never register this method as a model tool.
    pub fn sources_list(&self, request: RootRequest) -> WResult<SourcesListResult> {
        self.with(&request.root_id, |ctx| {
            let state = load(ctx)?;
            let op = new_id();
            let mut records = vec![];
            let mut sources = vec![];
            for entry in walkdir::WalkDir::new(&ctx.root)
                .follow_links(false)
                .into_iter()
                .filter_entry(|e| e.depth() == 0 || e.file_name() != "topics")
            {
                let entry = entry
                    .map_err(|_| error("invalid_input", "Cannot enumerate source inventory"))?;
                if !entry.file_type().is_file() {
                    continue;
                }
                let Some(path) = entry
                    .path()
                    .strip_prefix(&ctx.root)
                    .ok()
                    .and_then(Path::to_str)
                else {
                    continue;
                };
                if !path.to_lowercase().ends_with(".md") {
                    continue;
                }
                let bytes = match source_bytes(ctx, path) {
                    Ok(b) => b,
                    Err(_) => continue,
                };
                let h = hash(&bytes);
                let id = stable_id(&format!("{}\n{path}\n{h}", ctx.root.display()));
                let sensitivity = path_sensitivity(&state, path);
                let source = SourceSummary {
                    source_id: id.clone(),
                    path: path.into(),
                    hash: h.clone(),
                    date: crate::parse::document(
                        path,
                        std::str::from_utf8(&bytes).map_err(internal)?,
                        h,
                        "0".into(),
                    )
                    .date,
                    sensitivity,
                };
                if !state.sources.contains_key(&id) {
                    records.push((
                        format!("topics/sources/{id}.md"),
                        Record::new("source", &id, &op, &source)?,
                    ));
                }
                sources.push(source);
            }
            if !records.is_empty() {
                commit(
                    ctx,
                    "Register human source inventory",
                    records,
                    CommitIntent {
                        dependencies: vec![],
                        tombstones: vec![],
                        undo_of: None,
                        expected: None,
                        proposal: None,
                    },
                )?;
            }
            sources.sort_by_key(|s| s.path.clone());
            Ok(SourcesListResult { sources })
        })
    }
    /// Human evidence inspection, including unknown/local-only. No model capability is granted.
    pub fn source_get(&self, request: SourceGetRequest) -> WResult<ScopedSource> {
        self.with(&request.root_id, |ctx| {
            valid_id(&request.source_id)?;
            let state = load(ctx)?;
            let summary = state
                .sources
                .get(&request.source_id)
                .cloned()
                .ok_or_else(|| error("invalid_input", "Source does not exist"))?;
            let content = String::from_utf8(source_bytes(ctx, &summary.path)?).map_err(internal)?;
            if hash(content.as_bytes()) != summary.hash {
                return Err(error(
                    "stale_evidence",
                    "Source changed; refresh the inventory",
                ));
            }
            let citation = SourceRef {
                source_id: summary.source_id.clone(),
                path: summary.path.clone(),
                document_hash: summary.hash.clone(),
                locator: Locator {
                    start_byte: 0,
                    end_byte: content.len() as u64,
                    heading: None,
                },
                passage_hash: hash(content.as_bytes()),
                excerpt: content.clone(),
            };
            Ok(ScopedSource {
                summary,
                content,
                citation,
            })
        })
    }
}
