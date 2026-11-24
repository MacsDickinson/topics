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

pub(super) fn canonical_relative(ctx: &Context, path: &str) -> WResult<String> {
    relative(path)?;
    let mut at = ctx.root.clone();
    let mut parts = vec![];
    for requested in path.split('/') {
        let metadata = fs::symlink_metadata(at.join(requested))
            .map_err(|_| error("stale_evidence", "Selected path is unavailable"))?;
        if metadata.file_type().is_symlink() {
            return Err(error("forbidden", "Source symlinks are excluded"));
        }
        let mut actual = None;
        for entry in fs::read_dir(&at).map_err(internal)? {
            let entry = entry.map_err(internal)?;
            let same = entry.file_name() == requested || {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    entry
                        .metadata()
                        .is_ok_and(|m| m.dev() == metadata.dev() && m.ino() == metadata.ino())
                }
                #[cfg(not(unix))]
                {
                    false
                }
            };
            if same {
                actual = entry.file_name().to_str().map(str::to_string);
                break;
            }
        }
        let actual = actual.ok_or_else(|| {
            error(
                "invalid_input",
                "Selected path must have a UTF-8 filesystem identity",
            )
        })?;
        if parts.is_empty() && actual.eq_ignore_ascii_case("topics") {
            return Err(error(
                "forbidden",
                "App records are excluded from source selection",
            ));
        }
        at.push(&actual);
        parts.push(actual);
    }
    Ok(parts.join("/"))
}
pub(super) fn source_bytes(ctx: &Context, path: &str) -> WResult<Vec<u8>> {
    let canonical = canonical_relative(ctx, path)?;
    let path = canonical.as_str();
    if path
        .split('/')
        .next()
        .is_some_and(|p| p.eq_ignore_ascii_case("topics"))
        || !path.to_lowercase().ends_with(".md")
    {
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
pub(super) fn locator_valid(text: &str, locator: &Locator) -> bool {
    let (Ok(start), Ok(end)) = (
        usize::try_from(locator.start_byte),
        usize::try_from(locator.end_byte),
    ) else {
        return false;
    };
    if start >= end || text.get(start..end).is_none() {
        return false;
    }
    if let Some(heading) = &locator.heading {
        let before = text[..start]
            .lines()
            .rev()
            .find(|line| line.starts_with('#'))
            .map(|line| line.trim_start_matches('#').trim());
        let first = if start == 0 {
            text.lines()
                .next()
                .filter(|line| line.starts_with('#'))
                .map(|line| line.trim_start_matches('#').trim())
        } else {
            None
        };
        if before.or(first) != Some(heading.as_str()) {
            return false;
        }
    }
    true
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
    if !locator_valid(text, &source.locator) {
        return Err(stale());
    }
    if canonical_relative(ctx, &source.path).map_err(|_| stale())? != source.path {
        return Err(stale());
    }
    Ok(())
}
pub(super) use topics_core::validate_date_range as validate_dates;

fn topic_tainted_sources(state: &State) -> BTreeSet<Id> {
    let private: BTreeSet<_> = state
        .topics
        .values()
        .filter(|t| topic_restricted(state, &t.id))
        .map(|t| t.id.clone())
        .collect();
    let mut sources = BTreeSet::new();
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
fn restricted_sources(state: &State) -> BTreeSet<Id> {
    let mut sources: BTreeSet<_> = state
        .sources
        .values()
        .filter(|s| source_policy(state, &s.path, &s.hash) == Sensitivity::LocalOnly)
        .map(|s| s.source_id.clone())
        .collect();
    sources.extend(topic_tainted_sources(state));
    sources
}
pub(super) fn path_sensitivity(state: &State, path: &str) -> Sensitivity {
    if state.source_integrity_unknown {
        return Sensitivity::LocalOnly;
    }
    state
        .path_labels
        .get(path)
        .copied()
        .unwrap_or(Sensitivity::Unknown)
}
// Primitive policy deliberately excludes topic traversal so source-derived
// topic taint cannot recurse through identical-content lineage.
fn source_policy(state: &State, path: &str, h: &str) -> Sensitivity {
    let own = path_sensitivity(state, path);
    if own == Sensitivity::LocalOnly {
        return own;
    }
    let review = state.reviewed_sources.get(&(path.into(), h.into()));
    if let Some(private_sequence) = state.private_hashes.get(h)
        && review.is_none_or(|(_, sequence)| sequence < private_sequence)
    {
        return Sensitivity::LocalOnly;
    }
    review.map(|(label, _)| *label).unwrap_or(own)
}
pub(super) fn source_sensitivity(state: &State, path: &str, h: &str) -> Sensitivity {
    let own = source_policy(state, path, h);
    if own == Sensitivity::LocalOnly {
        return own;
    }
    // Reviewing source bytes cannot declassify a private derived topic. Apply
    // topic taint before the exact-byte source-consent override, so preview can
    // exclude only this source and retain the unrelated eligible selection.
    let topic_tainted = topic_tainted_sources(state);
    if state
        .source_history
        .iter()
        .any(|s| (s.hash == h || s.path == path) && topic_tainted.contains(&s.source_id))
    {
        return Sensitivity::LocalOnly;
    }
    if state
        .reviewed_sources
        .contains_key(&(path.into(), h.into()))
    {
        return own;
    }
    let restricted = restricted_sources(state);
    if state
        .source_history
        .iter()
        .any(|s| (s.hash == h || s.path == path) && restricted.contains(&s.source_id))
    {
        return Sensitivity::LocalOnly;
    }
    own
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
pub(super) fn endpoint_topic(state: &State, endpoint: &Endpoint) -> Option<Id> {
    match endpoint {
        Endpoint::Topic { id } => Some(id.clone()),
        Endpoint::Claim { id } => state
            .claims
            .values()
            .find(|c| &c.id == id || &c.revision_id == id)
            .map(|c| c.subject_id.clone()),
        _ => None,
    }
}
pub(super) fn topic_restricted(state: &State, id: &str) -> bool {
    if state.source_integrity_unknown
        || state.orphan_topics.contains(id)
        || state
            .topics
            .get(id)
            .is_none_or(|t| t.sensitivity == Sensitivity::LocalOnly)
    {
        return true;
    }
    let restricted = |source: &SourceRef| {
        !state.sources.contains_key(&source.source_id)
            || source_policy(state, &source.path, &source.document_hash) == Sensitivity::LocalOnly
    };
    state
        .claims
        .values()
        .filter(|c| c.subject_id == id)
        .flat_map(|c| &c.sources)
        .any(|e| matches!(e,Evidence::Passage{source} if restricted(source)))
        || state
            .connections
            .values()
            .filter(|c| {
                [&c.from, &c.to]
                    .iter()
                    .any(|e| endpoint_topic(state, e).as_deref() == Some(id))
            })
            .any(|c| c.sources.iter().any(restricted))
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
                if folder
                    .split('/')
                    .next()
                    .is_some_and(|p| p.eq_ignore_ascii_case("topics"))
                {
                    excluded.push(Exclusion {
                        path: folder.clone(),
                        reason: "App records are excluded".into(),
                    });
                    continue;
                }
                let folder = match canonical_relative(ctx, folder) {
                    Ok(path) => path,
                    Err(_) => {
                        excluded.push(Exclusion {
                            path: folder.clone(),
                            reason: "Unsafe or reserved folder is excluded".into(),
                        });
                        continue;
                    }
                };
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
                for entry in walkdir::WalkDir::new(at).follow_links(false).into_iter() {
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
            for requested in files {
                let path = match canonical_relative(ctx, &requested) {
                    Ok(path) => path,
                    Err(_) => {
                        excluded.push(Exclusion {
                            path: requested,
                            reason: "Unsafe, unavailable or app-owned source".into(),
                        });
                        continue;
                    }
                };
                if path
                    .split('/')
                    .next()
                    .is_some_and(|p| p.eq_ignore_ascii_case("topics"))
                    || !path.to_lowercase().ends_with(".md")
                {
                    excluded.push(Exclusion {
                        path,
                        reason: "App records and non-Markdown files are excluded".into(),
                    });
                    continue;
                }
                let sensitivity = path_sensitivity(&state, &path);
                if sensitivity == Sensitivity::LocalOnly {
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
                if source_sensitivity(&state, &path, &h) == Sensitivity::LocalOnly {
                    excluded.push(Exclusion {
                        path,
                        reason: "Local-only or unresolved same-content source identity".into(),
                    });
                    continue;
                }
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
                let id = source_identity(ctx, &state, &path, &h);
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
                let expected: BTreeMap<_, _> = records
                    .iter()
                    .map(|(path, _)| (path.clone(), state.hashes.get(path).cloned().flatten()))
                    .collect();
                commit(
                    ctx,
                    "Register previewed sources",
                    records,
                    CommitIntent {
                        dependencies: vec![],
                        tombstones: vec![],
                        undo_of: None,
                        expected: Some(&expected),
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
                if hash(content.as_bytes()) != summary.hash {
                    return Err(error(
                        "stale_scope",
                        "Selected source changed during retrieval; preview again",
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
        if state.topics.get(&request.topic_id).is_none_or(|t|t.sensitivity!=Sensitivity::CloudAllowed)||topic_restricted(&state,&request.topic_id){return Err(error("forbidden","Topic context is unavailable for this run"));}
        let mut view=topic_projection(&state,&request.topic_id)?;let allowed:BTreeSet<_>=selected.preview.sources.iter().map(|s|s.source_id.clone()).collect();
            fn scoped_claim(ctx:&Context,state:&State,claim:&Claim,allowed:&BTreeSet<Id>,topic:&str,depth:usize)->bool {
                if depth>100||claim.sources.is_empty(){return false;}
                let mut passage=false;
                for evidence in &claim.sources {match evidence {
                    Evidence::Passage{source}=>{passage=true;if !allowed.contains(&source.source_id)||validate_source(ctx,state,source).is_err(){return false;}},
                    Evidence::UserStatement{statement_id}=>if state.statements.get(statement_id).is_none_or(|s|s.topic_id!=topic){return false;},
                }}
                passage||(!claim.supersedes.is_empty()&&claim.supersedes.iter().all(|id|state.claims.get(id).is_some_and(|old|scoped_claim(ctx,state,old,allowed,topic,depth+1))))
            }
            let eligible=|claim:&Claim|scoped_claim(ctx,&state,claim,&allowed,&request.topic_id,0);
        let mut redacted=false;
        view.current_claims.retain_mut(|claim|{
            if eligible(claim){return true;}
            if human_value_supported(&state,claim) {
                claim.sources.clear();claim.supersedes.clear();claim.valid_from=None;claim.valid_to=None;claim.uncertainty=Some("Human-supported current value; evidence outside selected scope is redacted".into());redacted=true;return true;
            }
            false
        });
        view.history.retain(eligible);
        let endpoint_scoped = |endpoint: &Endpoint| match endpoint {
            Endpoint::Source {source_id,..} => allowed.contains(source_id) && endpoint_cloud(&state,endpoint),
            Endpoint::Claim {id} => state.claims.values().find(|c| &c.id==id || &c.revision_id==id).is_some_and(|c| scoped_claim(ctx,&state,c,&allowed,&c.subject_id,0)) && endpoint_cloud(&state,endpoint),
            Endpoint::Topic {..} => endpoint_cloud(&state,endpoint),
        };
        view.connections.retain(|c|!c.sources.is_empty()&&c.sources.iter().all(|s|allowed.contains(&s.source_id)&&validate_source(ctx,&state,s).is_ok())&&endpoint_scoped(&c.from)&&endpoint_scoped(&c.to));
        view.unresolved.retain(|c|matches!(c,Candidate::UnresolvedIdentity{sources,candidates,..} if candidates.contains(&request.topic_id)&&sources.iter().all(|s|allowed.contains(&s.source_id)&&validate_source(ctx,&state,s).is_ok())&&candidates.iter().all(|id|state.topics.get(id).is_some_and(|t|t.sensitivity==Sensitivity::CloudAllowed)&&!topic_restricted(&state,id))));
        view.warnings.clear();if redacted{view.warnings.push("Current human-supported value included with out-of-scope evidence and metadata redacted".into());}Ok(view)
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
// Edit statements retain exact UI evidence, but metadata-only edits do not
// make the transcript value human-authored. Authority comes from the durable
// before/after decision, not the mere presence of a statement ID on a claim.
fn human_value_supported(state: &State, claim: &Claim) -> bool {
    if state.reviews.values().any(|review| {
        review.record_id == claim.revision_id && review.state == ReviewState::ConfirmedByYou
            && state.receipts.values().any(|decision| {
                review.receipt_id.as_ref() == Some(&decision.receipt.receipt_id)
                    && decision.saved_ids.contains(&claim.revision_id)
                    && (decision.receipt.kind.confirms_existing()
                        && decision.receipt.revisions.contains(claim)
                        || decision.receipt.candidates.iter().any(|candidate| matches!(candidate,
                            Candidate::Claim{claim:confirmed} | Candidate::Correction{claim:confirmed,..} if confirmed == claim)))
            })
    }) { return true; }
    // Metadata revisions may preserve earlier value authority, but never cross
    // a value/identity change to revive authority for an old transcript value.
    // A visited set bounds both cycles and total queued history to 100 records.
    const MAX_VALUE_LINEAGE: usize = 100;
    let mut pending = vec![claim];
    let mut visited = BTreeSet::from([claim.revision_id.clone()]);
    while let Some(current) = pending.pop() {
        if direct_human_value_supported(state, current) {
            return true;
        }
        for id in &current.supersedes {
            let Some(old) = state.claims.get(id) else {
                continue;
            };
            if old.id != claim.id
                || old.subject_id != claim.subject_id
                || old.predicate != claim.predicate
                || old.value.trim() != claim.value.trim()
            {
                continue;
            }
            if visited.insert(old.revision_id.clone()) {
                if visited.len() > MAX_VALUE_LINEAGE {
                    return false;
                }
                pending.push(old);
            }
        }
    }
    false
}
pub(super) fn direct_human_value_supported(state: &State, claim: &Claim) -> bool {
    claim.sources.iter().any(|e| {
        let Evidence::UserStatement { statement_id } = e else { return false; };
        if state.statements.get(statement_id).is_none_or(|s| s.topic_id != claim.subject_id) { return false; }
        let path = format!("topics/statements/{statement_id}.md");
        state.operations.iter().any(|operation| {
            if !operation.targets.iter().any(|t| t.path == path) { return false; }
            if operation.operation.description == "Record user statement" {
                // Approval of a question or metadata revision does not author
                // the unchanged transcript value. A raw statement supports an
                // actual changed value, or the entire literal value itself.
                // Substrings, instructions and token/semantic guesses grant no
                // authority even if the candidate was subsequently approved.
                let changed_value = claim.supersedes.iter().any(|id| state.claims.get(id).is_some_and(|old| old.id == claim.id && old.subject_id == claim.subject_id && old.predicate == claim.predicate && old.value.trim() != claim.value.trim()));
                let literal_value = state.statements.get(statement_id).is_some_and(|statement| !claim.value.trim().is_empty() && statement.text.trim() == claim.value.trim());
                return changed_value || literal_value;
            }
            operation.proposal.as_ref().is_some_and(|decision| {
                decision.applied_candidates.iter().any(|candidate| {
                    let (Candidate::Claim {claim:applied} | Candidate::Correction {claim:applied,..}) = candidate else { return false; };
                    applied.subject_id == claim.subject_id && applied.predicate == claim.predicate && applied.value == claim.value
                        && applied.sources.contains(e)
                        && decision.proposed_candidates.iter().any(|old| matches!(old,Candidate::Claim{claim:old}|Candidate::Correction{claim:old,..} if old.revision_id==applied.revision_id && old.value!=applied.value))
                })
            })
        })
    })
}
fn endpoint_cloud(state: &State, endpoint: &Endpoint) -> bool {
    match endpoint {
        Endpoint::Topic { id } => state.topics.get(id).is_some_and(|t| {
            t.sensitivity == Sensitivity::CloudAllowed && !topic_restricted(state, &t.id)
        }),
        Endpoint::Claim { id } => state
            .claims
            .values()
            .find(|c| &c.id == id || &c.revision_id == id)
            .is_some_and(|c| {
                state.topics.get(&c.subject_id).is_some_and(|t| {
                    t.sensitivity == Sensitivity::CloudAllowed && !topic_restricted(state, &t.id)
                })
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
            if state.source_integrity_unknown{return Err(error("conflict","Committed source mappings cannot be reconstructed; restore their snapshots before registering inventory"));}
            let op = new_id();
            let mut records = vec![];
            let mut sources = vec![];
            for entry in walkdir::WalkDir::new(&ctx.root)
                .follow_links(false)
                .into_iter()
                .filter_entry(|e| {
                    e.depth() != 1
                        || !e
                            .file_name()
                            .to_str()
                            .is_some_and(|n| n.eq_ignore_ascii_case("topics"))
                })
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
                let id = source_identity(ctx, &state, path, &h);
                let sensitivity = source_sensitivity(&state, path, &h);
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
                let expected:BTreeMap<_,_>=records.iter().map(|(path,_)|(path.clone(),state.hashes.get(path).cloned().flatten())).collect();
                commit(
                    ctx,
                    "Register human source inventory",
                    records,
                    CommitIntent {
                        dependencies: vec![],
                        tombstones: vec![],
                        undo_of: None,
                        expected: Some(&expected),
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

pub(super) fn source_identity(ctx: &Context, state: &State, path: &str, h: &str) -> Id {
    if let Some(source) = state
        .sources
        .values()
        .find(|s| s.path == path && s.hash == h)
    {
        return source.source_id.clone();
    }
    let deterministic = stable_id(&format!("{}\n{path}\n{h}", ctx.root.display()));
    if state.sources.contains_key(&deterministic) {
        new_id()
    } else {
        deterministic
    }
}
impl WorkspaceService {
    /// Explicit human mapping repair. Existing citation paths/hashes never change.
    pub fn source_identity_repair(&self, request: SourceRepairRequest) -> WResult<SourceSummary> {
        self.with(&request.root_id, |ctx| {
            let request = SourceRepairRequest {
                path: canonical_relative(ctx, &request.path)?,
                ..request.clone()
            };
            valid_id(&request.source_id)?;
            let state = load(ctx)?;
            let old = state
                .sources
                .get(&request.source_id)
                .ok_or_else(|| error("invalid_input", "Source identity does not exist"))?;
            if old.path != request.path && ctx.root.join(&old.path).exists() {
                return Err(error(
                    "conflict",
                    "Original source still exists; a rename repair requires an unambiguous move",
                ));
            }
            let bytes = source_bytes(ctx, &request.path)?;
            let h = hash(&bytes);
            if h != request.expected_hash {
                return Err(error(
                    "stale_evidence",
                    "Repair target changed; inspect it again",
                ));
            }
            let duplicates: Vec<_> = state
                .sources
                .values()
                .filter(|s| s.path == request.path && s.hash == h && s.source_id != old.source_id)
                .collect();
            if duplicates.iter().any(|s| {
                state.operations.iter().any(|c| {
                    c.dependencies.contains(&s.source_id)
                        && !state
                            .operations
                            .iter()
                            .any(|u| u.undo_of.as_ref() == Some(&c.operation.id))
                })
            }) {
                return Err(error(
                    "conflict",
                    "Repair target identity has dependents; review citation revisions explicitly",
                ));
            }
            let sensitivity = if old.sensitivity == Sensitivity::LocalOnly
                || source_sensitivity(&state, &request.path, &h) == Sensitivity::LocalOnly
                || duplicates
                    .iter()
                    .any(|s| s.sensitivity == Sensitivity::LocalOnly)
            {
                Sensitivity::LocalOnly
            } else {
                old.sensitivity
            };
            let repaired = SourceSummary {
                source_id: old.source_id.clone(),
                path: request.path.clone(),
                hash: h.clone(),
                date: crate::parse::document(
                    &request.path,
                    std::str::from_utf8(&bytes).map_err(internal)?,
                    h,
                    "0".into(),
                )
                .date,
                sensitivity,
            };
            if old == &repaired {
                return Ok(repaired);
            }
            let op = new_id();
            commit(
                ctx,
                "Repair source identity (citations remain stale)",
                vec![(
                    format!("topics/sources/{}.md", old.source_id),
                    Record::new("source", &old.source_id, &op, &repaired)?,
                )],
                CommitIntent {
                    dependencies: vec![old.source_id.clone()],
                    tombstones: duplicates.iter().map(|s| s.source_id.clone()).collect(),
                    expected: Some(&state.hashes),
                    ..CommitIntent::default()
                },
            )?;
            Ok(repaired)
        })
    }
}
