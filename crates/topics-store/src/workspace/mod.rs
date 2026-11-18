//! Authoritative Markdown knowledge service. No provider access or model approval surface.
mod persistence;
mod pilot_review;
mod proposals;
mod scopes;
mod setup;
use super::TopicsService;
use persistence::*;
use scopes::validate_source;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
use topics_core::*;
type WResult<T> = WorkspaceResult<T>;
pub(super) fn error(code: &str, message: &str) -> ApiError {
    ApiError::new(code, message)
}
pub(super) fn internal(_: impl std::fmt::Display) -> ApiError {
    error(
        "internal",
        "Workspace storage operation failed; inspect synthetic diagnostics or check access permissions",
    )
}
pub(super) fn hash(bytes: &[u8]) -> Hash {
    format!("{:x}", Sha256::digest(bytes))
}
pub(super) fn new_id() -> Id {
    uuid::Uuid::new_v4().to_string()
}
pub(super) fn stable_id(value: &str) -> Id {
    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, value.as_bytes()).to_string()
}
pub(super) fn now() -> Timestamp {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
}
pub(super) fn json_value(value: impl Serialize) -> WResult<Value> {
    serde_json::to_value(value).map_err(internal)
}
struct Context {
    root: PathBuf,
    root_id: Id,
    partition: PathBuf,
    interrupt_after: Option<usize>,
    instance_id: Id,
}
#[derive(Serialize, Deserialize)]
struct RootEpoch {
    path: String,
    root_id: Id,
}

/// One service is safe to retain across root changes. Each call takes the existing store.lock.
/// The runtime calls stage_proposal only after a successful terminal turn, never per tool call.
pub struct WorkspaceService {
    base: TopicsService,
    fault: std::sync::Mutex<Option<usize>>,
    instance_id: Id,
}
impl WorkspaceService {
    /// Human-only persisted correction/edit evidence retrieval; never a model tool.
    pub fn statement_get(&self, request: StatementGetRequest) -> WResult<UserStatement> {
        self.with(&request.root_id, |ctx| {
            valid_id(&request.statement_id)?;
            load(ctx)?
                .statements
                .get(&request.statement_id)
                .cloned()
                .ok_or_else(|| error("invalid_input", "User statement does not exist"))
        })
    }

    /// Deterministic interruption seam for synthetic persistence/BDD tests only.
    #[cfg(any(test, feature = "test-support"))]
    pub fn interrupt_next_operation_after(&self, files: usize) {
        *self.fault.lock().expect("synthetic fault lock") = Some(files);
    }

    pub fn open(app_data: impl AsRef<Path>) -> WResult<Self> {
        Ok(Self {
            base: TopicsService::open(app_data).map_err(internal)?,
            fault: std::sync::Mutex::new(None),
            instance_id: new_id(),
        })
    }
    fn context(&self, expected: Option<&str>) -> WResult<Context> {
        let root = self
            .base
            .configured_root()
            .map_err(|_| error("no_root", "Choose an available notes root"))?
            .ok_or_else(|| error("no_root", "Choose a notes root"))?;
        let path = root
            .to_str()
            .ok_or_else(|| error("invalid_input", "Root path must be UTF-8"))?
            .to_string();
        let epoch_path = self.base.app_data.join("workspace-root.json");
        let epoch = if epoch_path.exists() {
            Some(read_json::<RootEpoch>(&epoch_path)?)
        } else {
            None
        };
        let epoch = match epoch {
            Some(e) if e.path == path => e,
            _ => {
                let e = RootEpoch {
                    path: path.clone(),
                    root_id: new_id(),
                };
                save_json(&epoch_path, &e)?;
                e
            }
        };
        if expected.is_some_and(|id| id != epoch.root_id) {
            return Err(error(
                "stale_scope",
                "Notes root changed; select and preview again",
            ));
        }
        let partition = self.base.app_data.join("workspaces").join(stable_id(&path));
        fs::create_dir_all(&partition).map_err(internal)?;
        Ok(Context {
            root,
            root_id: epoch.root_id,
            instance_id: self.instance_id.clone(),
            partition,
            interrupt_after: self.fault.lock().map_err(internal)?.take(),
        })
    }
    fn with<T>(&self, root_id: &str, operation: impl FnOnce(&Context) -> WResult<T>) -> WResult<T> {
        let _guard = self.base.lock().map_err(internal)?;
        let ctx = self.context(Some(root_id))?;
        recover(&ctx)?;
        operation(&ctx)
    }
    pub fn root_get(&self, _request: RootGetRequest) -> WResult<RootGetResult> {
        let _guard = self.base.lock().map_err(internal)?;
        if self
            .base
            .configured_root()
            .map_err(|_| error("no_root", "Choose an available notes root"))?
            .is_none()
        {
            return Ok(RootGetResult { root: None });
        }
        let ctx = self.context(None)?;
        recover(&ctx)?;
        Ok(RootGetResult {
            root: Some(RootRef {
                root_id: ctx.root_id,
                path: ctx.root.to_string_lossy().into_owned(),
            }),
        })
    }
    pub fn root_set(&mut self, request: RootSetRequest) -> WResult<RootSetResult> {
        if !Path::new(&request.path).is_absolute() {
            return Err(error("invalid_input", "Notes root must be absolute"));
        }
        let index = if request.create {
            self.base.init_notes_root(&request.path)
        } else {
            self.base.use_notes_root(&request.path)
        }
        .map_err(|_| {
            error(
                "invalid_input",
                "Cannot select root; use a separate accessible folder, or create an empty folder",
            )
        })?;
        let _guard = self.base.lock().map_err(internal)?;
        // A new explicit selection invalidates capabilities even when selecting the same path again.
        let root = self
            .base
            .configured_root()
            .map_err(internal)?
            .ok_or_else(|| error("no_root", "Choose a notes root"))?;
        save_json(
            &self.base.app_data.join("workspace-root.json"),
            &RootEpoch {
                path: root.to_string_lossy().into_owned(),
                root_id: new_id(),
            },
        )?;
        let ctx = self.context(None)?;
        recover(&ctx)?;
        Ok(RootSetResult {
            root: RootRef {
                root_id: ctx.root_id,
                path: ctx.root.to_string_lossy().into_owned(),
            },
            index,
        })
    }
    pub fn topics_list(&self, request: RootRequest) -> WResult<TopicsListResult> {
        self.with(&request.root_id, |ctx| {
            Ok(TopicsListResult {
                topics: load(ctx)?.topics.into_values().collect(),
            })
        })
    }
    pub fn topic_get(&self, request: TopicGetRequest) -> WResult<TopicView> {
        self.with(&request.root_id, |ctx| {
            topic_view(ctx, &load(ctx)?, &request.topic_id)
        })
    }
    pub fn history_list(&self, request: RootRequest) -> WResult<HistoryListResult> {
        self.with(&request.root_id, |ctx| {
            let mut operations: Vec<_> = load(ctx)?
                .operations
                .into_iter()
                .map(|c| c.operation)
                .collect();
            for (_, j) in journals(ctx)?.0 {
                if j.status == "conflict" {
                    let mut operation = j.commit.operation;
                    operation.status = "conflict".into();
                    operations.push(operation);
                }
            }
            Ok(HistoryListResult { operations })
        })
    }
    pub fn user_statement(&self, request: UserStatementRequest) -> WResult<UserStatement> {
        self.with(&request.root_id, |ctx| {
            let state = load(ctx)?;
            if !state.topics.contains_key(&request.topic_id) {
                return Err(error("invalid_input", "Statement topic does not exist"));
            }
            if request.text.trim().is_empty() || request.text.len() > 32768 {
                return Err(error(
                    "invalid_input",
                    "Statement must contain 1–32768 bytes",
                ));
            }
            let statement = UserStatement {
                id: new_id(),
                topic_id: request.topic_id.clone(),
                text: request.text.clone(),
                recorded_at: now(),
            };
            let op = new_id();
            commit(
                ctx,
                "Record user statement",
                vec![(
                    format!("topics/statements/{}.md", statement.id),
                    Record::new("statement", &statement.id, &op, &statement)?,
                )],
                CommitIntent {
                    dependencies: vec![request.topic_id.clone()],
                    tombstones: vec![],
                    undo_of: None,
                    expected: None,
                    proposal: None,
                },
            )?;
            Ok(statement)
        })
    }
    pub fn sensitivity_set(&self, request: SensitivitySetRequest) -> WResult<OperationResult> {
        self.with(&request.root_id, |ctx| {
            let state = load(ctx)?;
            let op = new_id();
            let (path, record) = match &request.target {
                Endpoint::Topic { id } => {
                    let mut topic = state
                        .topics
                        .get(id)
                        .cloned()
                        .ok_or_else(|| error("invalid_input", "Topic does not exist"))?;
                    topic.sensitivity = request.sensitivity;
                    (
                        format!("topics/topics/{id}.md"),
                        Record::new("topic", id, &op, topic)?,
                    )
                }
                Endpoint::Source { source_id, .. } => {
                    let mut source = state
                        .sources
                        .get(source_id)
                        .cloned()
                        .ok_or_else(|| error("invalid_input", "Source does not exist"))?;
                    source.sensitivity = request.sensitivity;
                    (
                        format!("topics/sources/{source_id}.md"),
                        Record::new("source", source_id, &op, source)?,
                    )
                }
                _ => {
                    return Err(error(
                        "invalid_input",
                        "Only sources and topics have sensitivity labels",
                    ));
                }
            };
            let operation = commit(
                ctx,
                "Set sensitivity",
                vec![(path, record)],
                CommitIntent {
                    dependencies: vec![],
                    tombstones: vec![],
                    undo_of: None,
                    expected: Some(&state.hashes),
                    proposal: None,
                },
            )?;
            Ok(OperationResult { operation })
        })
    }
    pub fn undo(&self, request: UndoRequest) -> WResult<OperationResult> {
        self.with(&request.root_id, |ctx| {
            let state = load(ctx)?;
            let target = state
                .operations
                .iter()
                .find(|c| c.operation.id == request.operation_id)
                .ok_or_else(|| error("invalid_input", "Operation does not exist"))?;
            if target.undo_of.is_some() {return Err(error("invalid_input","A compensating undo cannot itself be undone; restore through a reviewed proposal"));}
            if state
                .operations
                .iter()
                .any(|c| c.undo_of.as_ref() == Some(&request.operation_id))
            {
                return Err(error("conflict", "Operation was already undone"));
            }
            let mut ids: BTreeSet<_> = target
                .targets
                .iter()
                .filter_map(|t| {
                    t.path
                        .rsplit('/')
                        .next()?
                        .strip_suffix(".md")
                        .map(str::to_string)
                })
                .collect();
            for record in target.targets.iter().filter_map(|t|t.record.as_ref()) {if record.record_kind=="claim"&&let Some(id)=record.data.get("id").and_then(Value::as_str){ids.insert(id.into());}}
            let logical:Vec<_>=state.claims.values().filter(|claim|ids.contains(&claim.revision_id)).map(|claim|claim.id.clone()).collect();ids.extend(logical);
            let target_order=state.operations.iter().position(|c|c.operation.id==target.operation.id).unwrap();
            let paths:BTreeSet<_>=target.targets.iter().map(|t|t.path.clone()).collect();
            let undone: BTreeSet<_> = state
                .operations
                .iter()
                .filter_map(|c| c.undo_of.clone())
                .collect();
            let dependents:Vec<_>=state.operations.iter().enumerate().filter(|(order,c)|*order>target_order&&c.undo_of.is_none()&&!undone.contains(&c.operation.id)&&(c.dependencies.iter().any(|d|ids.contains(d))||c.targets.iter().any(|t|paths.contains(&t.path)))).map(|(_,c)|c.operation.id.clone()).collect();
            if !dependents.is_empty() {
                let mut e = error("conflict", "Undo dependent operations first");
                e.details = Some(serde_json::json!({"dependent_operations":dependents}));
                return Err(e);
            }
            let journal: Journal = read_json(
                &ctx.partition
                    .join("journals")
                    .join(&request.operation_id)
                    .join("journal.json"),
            )?;
            let op = new_id();
            let mut records = vec![];
            let mut tombstones = vec![];
            let mut expected = BTreeMap::new();
            for t in &journal.targets {
                if state.conflicts.contains(&t.path){return Err(error("conflict","Committed target is damaged or rolled back; resolve its conflict before undo"));}
                let current = bytes_optional(&target_path(&ctx.root, &t.path, false)?)?
                    .as_deref()
                    .map(hash);
                if current != Some(t.new_hash.clone()) {
                    let restored=bytes_optional(&target_path(&ctx.root,&t.path,false)?)?.and_then(|bytes|Record::parse(&bytes).ok()).zip(Record::parse(&t.new_bytes).ok()).is_some_and(|(current,original)|current.id==original.id&&current.record_kind==original.record_kind&&current.data==original.data&&state.latest_operations.get(&t.path)==Some(&current.operation_id)&&state.operations.iter().any(|c|c.operation.id==current.operation_id&&c.undo_of.is_some()));
                    if !restored {return Err(error("conflict","Cannot undo an externally or subsequently changed record"));}
                }
                expected.insert(t.path.clone(), current);
                if let Some(old) = &t.old_bytes {
                    let mut record = Record::parse(old)?;
                    record.operation_id = op.clone();
                    records.push((t.path.clone(), record));
                } else {
                    let id = t
                        .path
                        .rsplit('/')
                        .next()
                        .and_then(|v| v.strip_suffix(".md"))
                        .ok_or_else(|| error("internal", "Invalid undo target"))?;
                    tombstones.push(id.into());
                }
            }
            // Empty compensations still have an operation ID independent from their target records.
            let operation = if records.is_empty() {
                commit_empty(
                    ctx,
                    &op,
                    "Undo creations",
                    tombstones,
                    Some(request.operation_id.clone()),
                )?
            } else {
                commit(
                    ctx,
                    "Undo record update",
                    records,
                    CommitIntent {
                        dependencies: vec![],
                        tombstones,
                        undo_of: Some(request.operation_id.clone()),
                        expected: Some(&expected),
                        proposal: None,
                    },
                )?
            };
            Ok(OperationResult { operation })
        })
    }
}
fn commit_empty(
    ctx: &Context,
    id: &str,
    description: &str,
    tombstones: Vec<Id>,
    undo_of: Option<Id>,
) -> WResult<Operation> {
    let operation = Operation {
        id: id.into(),
        status: "committed".into(),
        description: description.into(),
        created_at: now(),
    };
    let mut journal = Journal {
        commit: Commit {
            operation: operation.clone(),
            sequence: next_sequence(ctx)?,
            targets: vec![],
            dependencies: vec![],
            tombstones,
            undo_of,
            proposal: None,
        },
        targets: vec![],
        status: "staged".into(),
    };
    let dir = ctx.partition.join("journals").join(id);
    fs::create_dir_all(&dir).map_err(internal)?;
    save_json(&dir.join("journal.json"), &journal)?;
    apply_journal(ctx, &dir, &mut journal, None)?;
    Ok(operation)
}
fn topic_projection(state: &State, id: &str) -> WResult<TopicView> {
    valid_id(id)?;
    let topic = state
        .topics
        .get(id)
        .cloned()
        .ok_or_else(|| error("invalid_input", "Topic does not exist"))?;
    let history: Vec<_> = state
        .claims
        .values()
        .filter(|c| c.subject_id == id)
        .cloned()
        .collect();
    let superseded: BTreeSet<_> = history
        .iter()
        .flat_map(|c| c.supersedes.iter().cloned())
        .collect();
    let current_claims = history
        .iter()
        .filter(|c| {
            !superseded.contains(&c.revision_id)
                && !state.orphan_superseded.contains(&c.revision_id)
        })
        .cloned()
        .collect();
    let connections = state
        .connections
        .values()
        .filter(|c| {
            matches!(&c.from,Endpoint::Topic{id:other} if other==id)
                || matches!(&c.to,Endpoint::Topic{id:other} if other==id)
        })
        .cloned()
        .collect();
    let warnings = state.warnings.clone();
    Ok(TopicView {
        topic,
        current_claims,
        connections,
        unresolved: state.unresolved.clone(),
        history,
        warnings,
    })
}

fn topic_view(ctx: &Context, state: &State, id: &str) -> WResult<TopicView> {
    let mut view = topic_projection(state, id)?;
    for claim in &view.history {
        for evidence in &claim.sources {
            if let Evidence::Passage { source } = evidence
                && validate_source(ctx, state, source).is_err()
            {
                view.warnings
                    .push(format!("Stale citation: {}", source.path));
            }
        }
    }
    Ok(view)
}
