//! Recoverable per-file replacements. The final Markdown operation is the visibility marker.
use super::*;
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Record {
    pub schema: String,
    pub record_kind: String,
    pub id: Id,
    pub operation_id: Id,
    pub data: Value,
}
impl Record {
    pub fn new(kind: &str, id: &str, op: &str, data: impl Serialize) -> WResult<Self> {
        Ok(Self {
            schema: "topics/v1".into(),
            record_kind: kind.into(),
            id: id.into(),
            operation_id: op.into(),
            data: json_value(data)?,
        })
    }
    pub fn bytes(&self) -> WResult<Vec<u8>> {
        let title = self
            .data
            .get("name")
            .or_else(|| self.data.get("value"))
            .or_else(|| self.data.get("text"))
            .and_then(Value::as_str)
            .unwrap_or(&self.record_kind);
        Ok(format!("---\nschema: topics/v1\nrecord_kind: {}\nid: {}\noperation_id: {}\ndata: {}\n---\n# {}\n\n{}\n", self.record_kind, self.id, self.operation_id, serde_json::to_string(&self.data).map_err(internal)?, self.record_kind, title).into_bytes())
    }
    pub fn parse(bytes: &[u8]) -> WResult<Self> {
        let text = std::str::from_utf8(bytes).map_err(internal)?;
        let front = text
            .strip_prefix("---\n")
            .and_then(|s| s.split_once("\n---\n"))
            .map(|s| s.0)
            .ok_or_else(|| error("invalid_input", "Malformed record frontmatter"))?;
        let mut fields = BTreeMap::new();
        for line in front.lines() {
            let (k, v) = line
                .split_once(": ")
                .ok_or_else(|| error("invalid_input", "Malformed record property"))?;
            if fields.insert(k, v).is_some() {
                return Err(error("invalid_input", "Duplicate record property"));
            }
        }
        if fields.len() != 5 || fields.get("schema") != Some(&"topics/v1") {
            return Err(error("invalid_input", "Unsupported record schema"));
        }
        let field = |name| {
            fields
                .get(name)
                .map(|v| v.to_string())
                .ok_or_else(|| error("invalid_input", "Missing record property"))
        };
        Ok(Self {
            schema: field("schema")?,
            record_kind: field("record_kind")?,
            id: field("id")?,
            operation_id: field("operation_id")?,
            data: serde_json::from_str(
                fields
                    .get("data")
                    .ok_or_else(|| error("invalid_input", "Missing record data"))?,
            )
            .map_err(internal)?,
        })
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Target {
    pub path: String,
    pub expected_hash: Option<Hash>,
    pub new_hash: Hash,
    // Snapshots and staged bytes are explicitly persisted before the first target changes.
    pub old_bytes: Option<Vec<u8>>,
    pub new_bytes: Vec<u8>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Commit {
    pub operation: Operation,
    #[serde(default)]
    pub sequence: u64,
    pub targets: Vec<TargetMeta>,
    pub dependencies: Vec<Id>,
    pub tombstones: Vec<Id>,
    pub undo_of: Option<Id>,
    pub proposal: Option<CommittedDecision>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct CommittedDecision {
    pub proposal_id: Id,
    pub fingerprint: Hash,
    pub decision: Decision,
    #[serde(default)]
    pub applied_candidates: Vec<Candidate>,
    #[serde(default)]
    pub proposed_candidates: Vec<Candidate>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct TargetMeta {
    pub path: String,
    pub expected_hash: Option<Hash>,
    pub new_hash: Hash,
    #[serde(default)]
    pub record: Option<Record>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Journal {
    pub commit: Commit,
    pub targets: Vec<Target>,
    pub status: String,
}

pub(super) use topics_core::validate_workspace_id as valid_id;

pub(super) fn relative(path: &str) -> WResult<()> {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|c| c.is_empty() || c == "." || c == ".." || c.contains(':') || c.contains('\0'))
    {
        return Err(error(
            "invalid_input",
            "Expected a root-relative slash path without traversal",
        ));
    }
    Ok(())
}
pub(super) fn target_path(root: &Path, path: &str, create: bool) -> WResult<PathBuf> {
    relative(path)?;
    let parts: Vec<_> = path.split('/').collect();
    if parts.len() != 3
        || parts[0] != "topics"
        || ![
            "topics",
            "claims",
            "connections",
            "rules",
            "sources",
            "statements",
            "operations",
            "unresolved",
            "reviews",
            "receipts",
        ]
        .contains(&parts[1])
    {
        return Err(error("forbidden", "Write target is outside app records"));
    }
    valid_id(
        parts[2]
            .strip_suffix(".md")
            .ok_or_else(|| error("forbidden", "Invalid record target"))?,
    )?;
    let mut at = root.to_path_buf();
    for (i, part) in parts.iter().enumerate() {
        at.push(part);
        match fs::symlink_metadata(&at) {
            Ok(m) if m.file_type().is_symlink() => {
                return Err(error(
                    "forbidden",
                    "Symlinks are forbidden in app record targets",
                ));
            }
            Ok(m) if i < 2 && !m.is_dir() => {
                return Err(error("forbidden", "Record parent is not a directory"));
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                if i < 2 && create {
                    fs::create_dir(&at).map_err(internal)?;
                }
            }
            Err(e) => return Err(internal(e)),
        }
    }
    Ok(at)
}
pub(super) fn bytes_optional(path: &Path) -> WResult<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(v) => Ok(Some(v)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(internal(e)),
    }
}
pub(super) fn atomic(path: &Path, bytes: &[u8]) -> WResult<()> {
    let parent = path
        .parent()
        .ok_or_else(|| error("internal", "Missing write parent"))?;
    fs::create_dir_all(parent).map_err(internal)?;
    let temporary = parent.join(format!(".topics-{}.tmp", new_id()));
    let result = (|| {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(internal)?;
        f.write_all(bytes).map_err(internal)?;
        f.sync_all().map_err(internal)?;
        fs::rename(&temporary, path).map_err(internal)?;
        fs::File::open(parent)
            .and_then(|f| f.sync_all())
            .map_err(internal)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}
pub(super) struct PreparedPublication {
    temporary: PathBuf,
    target: PathBuf,
    pub published: bool,
    replace: bool,
}
impl PreparedPublication {
    pub fn publish(&mut self) -> WResult<()> {
        if self.published {
            return Ok(());
        }
        if !self.replace && self.target.exists() {
            return Err(error("conflict", "Proposal target already exists"));
        }
        fs::rename(&self.temporary, &self.target).map_err(internal)?;
        self.published = true;
        Ok(())
    }
}
impl Drop for PreparedPublication {
    fn drop(&mut self) {
        if !self.published {
            let _ = fs::remove_file(&self.temporary);
        }
    }
}
pub(super) fn prepare_checkpoint(
    path: &Path,
    value: &impl Serialize,
) -> WResult<PreparedPublication> {
    let mut prepared = prepare_publication(path, value)?;
    prepared.replace = true;
    Ok(prepared)
}
pub(super) fn prepare_publication(
    path: &Path,
    value: &impl Serialize,
) -> WResult<PreparedPublication> {
    let parent = path
        .parent()
        .ok_or_else(|| error("internal", "Missing proposal parent"))?;
    fs::create_dir_all(parent).map_err(internal)?;
    let prepared = PreparedPublication {
        temporary: parent.join(format!(".topics-{}.tmp", new_id())),
        target: path.into(),
        published: false,
        replace: false,
    };
    let bytes = serde_json::to_vec_pretty(value).map_err(internal)?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&prepared.temporary)
        .map_err(internal)?;
    file.write_all(&bytes).map_err(internal)?;
    file.sync_all().map_err(internal)?;
    Ok(prepared)
}
pub(super) fn save_json(path: &Path, value: &impl Serialize) -> WResult<()> {
    atomic(path, &serde_json::to_vec_pretty(value).map_err(internal)?)
}
pub(super) fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> WResult<T> {
    serde_json::from_slice(&fs::read(path).map_err(internal)?).map_err(internal)
}

#[derive(Default)]
pub(super) struct CommitIntent<'a> {
    pub dependencies: Vec<Id>,
    pub tombstones: Vec<Id>,
    pub undo_of: Option<Id>,
    pub expected: Option<&'a BTreeMap<String, Option<Hash>>>,
    pub proposal: Option<CommittedDecision>,
}
pub(super) fn commit(
    ctx: &Context,
    description: &str,
    records: Vec<(String, Record)>,
    intent: CommitIntent<'_>,
) -> WResult<Operation> {
    let CommitIntent {
        dependencies,
        tombstones,
        undo_of,
        expected,
        proposal,
    } = intent;
    let id = records
        .first()
        .map(|r| r.1.operation_id.clone())
        .unwrap_or_else(new_id);
    let operation = Operation {
        id: id.clone(),
        status: "committed".into(),
        description: description.into(),
        created_at: now(),
    };
    let mut targets = vec![];
    for (path, record) in records {
        let target = target_path(&ctx.root, &path, true)?;
        let old_bytes = bytes_optional(&target)?;
        let actual = old_bytes.as_deref().map(hash);
        let expected_hash = expected
            .and_then(|map| map.get(&path))
            .cloned()
            .unwrap_or_else(|| actual.clone());
        let new_bytes = record.bytes()?;
        targets.push(Target {
            path,
            expected_hash,
            new_hash: hash(&new_bytes),
            old_bytes,
            new_bytes,
        });
    }
    let metadata = Commit {
        operation: operation.clone(),
        sequence: next_sequence(ctx)?,
        targets: targets
            .iter()
            .map(|t| {
                Ok(TargetMeta {
                    path: t.path.clone(),
                    expected_hash: t.expected_hash.clone(),
                    new_hash: t.new_hash.clone(),
                    record: Some(Record::parse(&t.new_bytes)?),
                })
            })
            .collect::<WResult<Vec<_>>>()?,
        dependencies,
        tombstones,
        undo_of,
        proposal,
    };
    let mut journal = Journal {
        commit: metadata,
        targets,
        status: "staged".into(),
    };
    let dir = ctx.partition.join("journals").join(&id);
    fs::create_dir_all(&dir).map_err(internal)?;
    // The journal's explicit null old_bytes marks absence; each byte array is a complete snapshot.
    save_json(&dir.join("journal.json"), &journal)?;
    if journal
        .targets
        .iter()
        .any(|t| t.old_bytes.as_deref().map(hash) != t.expected_hash)
    {
        preserve_conflict(ctx, &dir, &mut journal)?;
        return Err(error(
            "conflict",
            if journal
                .targets
                .iter()
                .all(|t| t.path.starts_with("topics/sources/"))
            {
                "Source mapping ownership or history changed; restore or explicitly repair mappings; versions preserved"
            } else {
                "Record changed since proposal review; snapshots and staged bytes preserved"
            },
        ));
    }
    apply_journal(ctx, &dir, &mut journal, ctx.interrupt_after)?;
    Ok(operation)
}
pub(super) fn preserve_conflict(ctx: &Context, dir: &Path, journal: &mut Journal) -> WResult<()> {
    journal.status = "conflict".into();
    for (i, t) in journal.targets.iter().enumerate() {
        let current = bytes_optional(&target_path(&ctx.root, &t.path, false)?)?;
        save_json(
            &dir.join(format!("conflict-{i}.json")),
            &serde_json::json!({"path":t.path,"old_bytes":t.old_bytes,"staged_bytes":t.new_bytes,"current_bytes":current}),
        )?;
    }
    save_json(&dir.join("journal.json"), journal)?;
    for target in &journal.targets {
        let path = target_path(&ctx.root, &target.path, false)?;
        if let Some(old) = &target.old_bytes
            && bytes_optional(&path)?.as_deref().map(hash) == Some(target.new_hash.clone())
        {
            atomic(&path, old)?;
        }
    }
    Ok(())
}
pub(super) fn apply_journal(
    ctx: &Context,
    dir: &Path,
    journal: &mut Journal,
    stop_after: Option<usize>,
) -> WResult<()> {
    // Validate every remaining expected target before resuming. Written new hashes are retry-safe.
    for t in &journal.targets {
        let current = bytes_optional(&target_path(&ctx.root, &t.path, false)?)?
            .as_deref()
            .map(hash);
        if current != t.expected_hash && current != Some(t.new_hash.clone()) {
            preserve_conflict(ctx, dir, journal)?;
            return Err(error(
                "conflict",
                "Interrupted operation conflicts with current records; versions preserved",
            ));
        }
    }
    for (i, t) in journal.targets.iter().enumerate() {
        let path = target_path(&ctx.root, &t.path, true)?;
        let current = bytes_optional(&path)?.as_deref().map(hash);
        if current != Some(t.new_hash.clone()) {
            if current != t.expected_hash {
                preserve_conflict(ctx, dir, journal)?;
                return Err(error("conflict", "Target changed during operation"));
            }
            atomic(&path, &t.new_bytes)?;
        }
        if stop_after == Some(i + 1) {
            return Err(error(
                "internal",
                "Synthetic interruption after target replacement",
            ));
        }
    }
    let id = &journal.commit.operation.id;
    let marker = Record::new("operation", id, id, &journal.commit)?;
    let marker_path = target_path(&ctx.root, &format!("topics/operations/{id}.md"), true)?;
    if let Some(existing) = bytes_optional(&marker_path)? {
        if existing != marker.bytes()? {
            return Err(error("conflict", "Operation marker collision"));
        }
    } else {
        atomic(&marker_path, &marker.bytes()?)?;
    }
    journal.status = "committed".into();
    save_json(&dir.join("journal.json"), journal)
}
pub(super) fn json_entries(dir: &Path) -> WResult<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut paths = vec![];
    for entry in fs::read_dir(dir).map_err(internal)? {
        let entry = entry.map_err(internal)?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if name.starts_with('.') || !entry.file_type().map_err(internal)?.is_file() {
            continue;
        }
        if name
            .strip_suffix(".json")
            .is_some_and(|id| valid_id(id).is_ok())
        {
            paths.push(entry.path());
        }
    }
    paths.sort();
    Ok(paths)
}
type JournalScan = (Vec<(PathBuf, Journal)>, Vec<String>);
pub(super) fn journals(ctx: &Context) -> WResult<JournalScan> {
    let base = ctx.partition.join("journals");
    let mut out = vec![];
    let mut warnings = vec![];
    if !base.exists() {
        return Ok((out, warnings));
    }
    for entry in fs::read_dir(base).map_err(internal)? {
        let entry = entry.map_err(internal)?;
        if !entry.file_type().map_err(internal)?.is_dir()
            || entry
                .file_name()
                .to_str()
                .is_none_or(|id| valid_id(id).is_err())
        {
            continue;
        }
        let path = entry.path().join("journal.json");
        if !path.exists() {
            continue;
        }
        match read_json::<Journal>(&path) {
            Ok(j) if valid_id(&j.commit.operation.id).is_ok() => out.push((entry.path(), j)),
            _ => warnings.push("Malformed operation journal retained for manual recovery".into()),
        }
    }
    Ok((out, warnings))
}
pub(super) fn recover(ctx: &Context) -> WResult<()> {
    for (dir, mut journal) in journals(ctx)?.0 {
        if journal.status != "staged" {
            continue;
        }
        let marker = target_path(
            &ctx.root,
            &format!("topics/operations/{}.md", journal.commit.operation.id),
            false,
        )?;
        if let Some(bytes) = bytes_optional(&marker)? {
            // A durable marker already committed these bytes. Never replay an old
            // journal over a subsequent write, deletion, or external conflict.
            if Record::parse(&bytes).is_ok_and(|r| {
                r.operation_id == journal.commit.operation.id
                    && r.record_kind == "operation"
                    && serde_json::from_value::<Commit>(r.data)
                        .is_ok_and(|c| json_value(c).ok() == json_value(&journal.commit).ok())
            }) {
                journal.status = "committed".into();
                save_json(&dir.join("journal.json"), &journal)?;
            }
            continue;
        }
        if let Err(e) = apply_journal(ctx, &dir, &mut journal, None)
            && e.code != "conflict"
        {
            return Err(e);
        }
    }
    Ok(())
}

#[derive(Default)]
pub(super) struct State {
    pub topics: BTreeMap<Id, Topic>,
    pub claims: BTreeMap<Id, Claim>,
    pub connections: BTreeMap<Id, Connection>,
    pub rules: BTreeMap<Id, Rule>,
    pub sources: BTreeMap<Id, SourceSummary>,
    pub statements: BTreeMap<Id, UserStatement>,
    pub unresolved: Vec<Candidate>,
    pub reviews: BTreeMap<Id, ReviewMetadata>,
    pub receipts: BTreeMap<Id, SetupDecisionResult>,
    pub operations: Vec<Commit>,
    pub warnings: Vec<String>,
    pub hashes: BTreeMap<String, Option<Hash>>,
    pub conflicts: BTreeSet<String>,
    pub path_labels: BTreeMap<String, Sensitivity>,
    pub reviewed_sources: BTreeMap<(String, Hash), (Sensitivity, u64)>,
    pub private_hashes: BTreeMap<Hash, u64>,
    pub orphan_superseded: BTreeSet<Id>,
    pub orphan_topics: BTreeSet<Id>,
    pub source_history: Vec<SourceSummary>,
    pub source_integrity_unknown: bool,
    pub latest_operations: BTreeMap<String, Id>,
}
pub(super) fn next_sequence(ctx: &Context) -> WResult<u64> {
    load(ctx)?
        .operations
        .iter()
        .map(|c| c.sequence)
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| error("conflict", "Operation sequence exhausted"))
}

pub(super) fn undone(state: &State, id: &str) -> bool {
    state
        .operations
        .iter()
        .any(|c| c.undo_of.as_deref() == Some(id))
}
fn payload(
    ctx: &Context,
    target: &TargetMeta,
    op: &str,
    journals: &[(PathBuf, Journal)],
) -> Option<Record> {
    let matching =
        |r: &Record| r.operation_id == op && r.bytes().is_ok_and(|b| hash(&b) == target.new_hash);
    if let Some(record) = &target.record
        && matching(record)
    {
        return Some(record.clone());
    }
    for (_, journal) in journals.iter().filter(|(_, j)| j.commit.operation.id == op) {
        if let Some(t) = journal
            .targets
            .iter()
            .find(|t| t.path == target.path && hash(&t.new_bytes) == target.new_hash)
            && let Ok(record) = Record::parse(&t.new_bytes)
            && matching(&record)
        {
            return Some(record);
        }
    }
    let bytes = fs::read(target_path(&ctx.root, &target.path, false).ok()?).ok()?;
    if hash(&bytes) != target.new_hash {
        return None;
    }
    let record = Record::parse(&bytes).ok()?;
    matching(&record).then_some(record)
}
pub(super) fn load(ctx: &Context) -> WResult<State> {
    let mut state = State::default();
    let (journals, journal_warnings) = journals(ctx)?;
    state.warnings.extend(journal_warnings);
    let ops = ctx.root.join("topics/operations");
    if ops.exists() {
        target_path(
            &ctx.root,
            &format!("topics/operations/{}.md", uuid::Uuid::nil()),
            false,
        )?;
        for entry in fs::read_dir(&ops).map_err(internal)? {
            let entry = entry.map_err(internal)?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            if name.starts_with('.') {
                continue;
            }
            let parsed = (|| {
                let path = target_path(&ctx.root, &format!("topics/operations/{name}"), false)?;
                let bytes = fs::read(&path).map_err(internal)?;
                let r = Record::parse(&bytes)?;
                let c: Commit = serde_json::from_value(r.data).map_err(internal)?;
                valid_id(&c.operation.id)?;
                if r.record_kind != "operation"
                    || r.id != c.operation.id
                    || r.operation_id != r.id
                    || name != format!("{}.md", r.id)
                    || c.operation.status != "committed"
                {
                    return Err(error("invalid_input", "Malformed operation marker"));
                }
                Ok(c)
            })();
            match parsed {
                Ok(c) => state.operations.push(c),
                Err(_) => state
                    .warnings
                    .push("Malformed or unsafe operation marker excluded".into()),
            }
        }
    }
    let mut marker_conflicts = BTreeSet::new();
    for (_, journal) in &journals {
        if journal.status != "committed" {
            continue;
        }
        let id = &journal.commit.operation.id;
        let mismatch = state
            .operations
            .iter()
            .find(|c| &c.operation.id == id)
            .is_none_or(|c| json_value(c).ok() != json_value(&journal.commit).ok());
        if mismatch {
            state.operations.retain(|c| &c.operation.id != id);
            state.operations.push(journal.commit.clone());
            marker_conflicts.insert(id.clone());
            state.warnings.push(format!(
                "Committed operation marker missing or changed: {id}"
            ));
        }
    }
    state.operations.sort_by_key(|c| {
        (
            c.sequence,
            c.operation.created_at.clone(),
            c.operation.id.clone(),
        )
    });
    let mut latest = BTreeMap::new();
    let mut tombstones = BTreeMap::new();
    for (order, commit) in state.operations.iter().enumerate() {
        for target in &commit.targets {
            if target_path(&ctx.root, &target.path, false).is_err() {
                state
                    .warnings
                    .push("Unsafe committed target excluded".into());
                continue;
            }
            latest.insert(target.path.clone(), (order, target));
            if target.path.starts_with("topics/sources/")
                && let Some(r) = payload(ctx, target, &commit.operation.id, &journals)
                && let Ok(source) = serde_json::from_value::<SourceSummary>(r.data)
            {
                // Consent is for these exact bytes. A review of the previous
                // occupant of a destination path cannot declassify moved bytes.
                let human_review = commit.operation.description == "Set sensitivity"
                    || commit.undo_of.as_ref().is_some_and(|id| {
                        state.operations.iter().any(|old| {
                            &old.operation.id == id
                                && old.operation.description == "Set sensitivity"
                        })
                    });
                if human_review {
                    state.reviewed_sources.insert(
                        (source.path.clone(), source.hash.clone()),
                        (source.sensitivity, commit.sequence),
                    );
                }
                if source.sensitivity == Sensitivity::LocalOnly {
                    state
                        .private_hashes
                        .insert(source.hash.clone(), commit.sequence);
                }
                state
                    .path_labels
                    .insert(source.path.clone(), source.sensitivity);
                state.source_history.push(source);
            }
        }
        for id in &commit.tombstones {
            tombstones.insert(id.clone(), order);
        }
    }
    let mut ids = BTreeSet::new();
    for (path, (order, target)) in latest {
        let commit = &state.operations[order];
        let filename = path.rsplit('/').next().unwrap_or("");
        let id = filename.trim_end_matches(".md");
        state
            .latest_operations
            .insert(path.clone(), commit.operation.id.clone());
        let bytes = bytes_optional(&target_path(&ctx.root, &path, false)?)?;
        let current_hash = bytes.as_deref().map(hash);
        state.hashes.insert(path.clone(), current_hash.clone());
        if tombstones.get(id).is_some_and(|t| *t >= order) {
            continue;
        }
        let integrity = current_hash.as_ref() == Some(&target.new_hash)
            && !marker_conflicts.contains(&commit.operation.id);
        if !integrity {
            state.conflicts.insert(path.clone());
            state.warnings.push(format!(
                "Committed record missing, changed or rolled back: {path}"
            ));
        }
        let Some(record) = payload(ctx, target, &commit.operation.id, &journals) else {
            if path.starts_with("topics/sources/") {
                state.source_integrity_unknown = true;
            }
            state
                .warnings
                .push(format!("Committed record cannot be reconstructed: {path}"));
            continue;
        };
        if record.id != id
            || record.operation_id != commit.operation.id
            || !ids.insert(record.id.clone())
        {
            state.warnings.push(format!(
                "Duplicate or mismatched committed record excluded: {path}"
            ));
            if path.starts_with("topics/sources/") {
                state.source_integrity_unknown = true;
            }
            continue;
        }
        let kind = match path.split('/').nth(1).unwrap_or("") {
            "topics" => "topic",
            "claims" => "claim",
            "connections" => "connection",
            "rules" => "rule",
            "sources" => "source",
            "statements" => "statement",
            "unresolved" => "unresolved",
            "reviews" => "review",
            "receipts" => "receipt",
            _ => "",
        };
        let key = match kind {
            "claim" => "revision_id",
            "source" => "source_id",
            "unresolved" | "receipt" => "",
            _ => "id",
        };
        if record.record_kind != kind
            || (!key.is_empty() && record.data.get(key).and_then(Value::as_str) != Some(id))
        {
            state
                .warnings
                .push(format!("Record payload identity mismatch: {path}"));
            if kind == "source" {
                state.source_integrity_unknown = true;
            }
            continue;
        }
        macro_rules! insert {
            ($map:ident,$ty:ty) => {
                match serde_json::from_value::<$ty>(record.data.clone()) {
                    Ok(value) => {
                        state.$map.insert(id.into(), value);
                    }
                    Err(_) => state
                        .warnings
                        .push(format!("Malformed committed record: {path}")),
                }
            };
        }
        match kind {
            "topic" => {
                insert!(topics, Topic);
                if !integrity && let Some(topic) = state.topics.get_mut(id) {
                    topic.sensitivity = Sensitivity::LocalOnly;
                }
            }
            "source" => {
                insert!(sources, SourceSummary);
                if state.sources.get(id).is_none_or(|source| {
                    relative(&source.path).is_err()
                        || source
                            .path
                            .split('/')
                            .next()
                            .is_some_and(|p| p.eq_ignore_ascii_case("topics"))
                        || source.hash.len() != 64
                        || !source
                            .hash
                            .bytes()
                            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
                        || validate_date_range(&source.date, &None).is_err()
                }) {
                    state.source_integrity_unknown = true;
                    state
                        .warnings
                        .push("Malformed committed source mapping requires recovery".into());
                }
                if !integrity && let Some(source) = state.sources.get_mut(id) {
                    source.sensitivity = Sensitivity::LocalOnly;
                    state
                        .path_labels
                        .insert(source.path.clone(), Sensitivity::LocalOnly);
                }
            }
            "claim" => insert!(claims, Claim),
            "connection" => insert!(connections, Connection),
            "rule" => insert!(rules, Rule),
            "statement" => insert!(statements, UserStatement),
            "review" if integrity => insert!(reviews, ReviewMetadata),
            "receipt" if integrity => insert!(receipts, SetupDecisionResult),
            "unresolved" => {
                if let Ok(c) = serde_json::from_value(record.data.clone()) {
                    state.unresolved.push(c);
                }
            }
            _ => {}
        }
    }
    // Surviving correction records without either a commit marker or a known
    // interrupted journal are evidence of lost history, never authority for a
    // replacement. Quarantine the old head rather than resurrecting a false fact.
    for (dir, kind) in [("claims", "claim"), ("rules", "rule")] {
        let probe = format!("topics/{dir}/{}.md", uuid::Uuid::nil());
        let safe = target_path(&ctx.root, &probe, false)?;
        let parent = safe
            .parent()
            .ok_or_else(|| error("invalid_input", "Invalid record directory"))?;
        if !parent.exists() {
            continue;
        }
        for entry in fs::read_dir(parent).map_err(internal)? {
            let entry = entry.map_err(internal)?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            let Some(id) = name.strip_suffix(".md") else {
                continue;
            };
            if valid_id(id).is_err() {
                continue;
            }
            let path = target_path(&ctx.root, &format!("topics/{dir}/{name}"), false)?;
            let Ok(bytes) = fs::read(path) else {
                continue;
            };
            let Ok(record) = Record::parse(&bytes) else {
                continue;
            };
            if record.id != id
                || record.record_kind != kind
                || state
                    .operations
                    .iter()
                    .any(|c| c.operation.id == record.operation_id)
                || journals
                    .iter()
                    .any(|(_, j)| j.commit.operation.id == record.operation_id)
            {
                continue;
            }
            if kind == "claim" {
                if let Ok(claim) = serde_json::from_value::<Claim>(record.data) {
                    if claim.supersedes.is_empty() {
                        continue;
                    }
                    state.orphan_superseded.extend(claim.supersedes);
                    state.orphan_topics.insert(claim.subject_id);
                } else {
                    continue;
                }
            } else if let Ok(rule) = serde_json::from_value::<Rule>(record.data) {
                state.orphan_superseded.extend(
                    state
                        .claims
                        .values()
                        .filter(|c| {
                            c.subject_id == rule.topic_id
                                && c.predicate == rule.predicate
                                && c.value == rule.match_value
                        })
                        .map(|c| c.revision_id.clone()),
                );
                state.orphan_topics.insert(rule.topic_id);
            } else {
                continue;
            }
            state.warnings.push(
                "Orphan correction history requires recovery; prior assertion withheld".into(),
            );
        }
    }
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (tempfile::TempDir, Context, Vec<(String, Record)>, Id) {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("notes");
        fs::create_dir(&root).unwrap();
        let partition = temp.path().join("app-data/root");
        fs::create_dir_all(&partition).unwrap();
        let ctx = Context {
            instance_id: new_id(),
            root,
            root_id: new_id(),
            partition,
            interrupt_after: Some(1),
        };
        let op = new_id();
        let a = Topic {
            id: new_id(),
            topic_kind: TopicKind::Person,
            name: "A".into(),
            aliases: vec![],
            sensitivity: Sensitivity::Unknown,
        };
        let b = Topic {
            id: new_id(),
            topic_kind: TopicKind::Project,
            name: "B".into(),
            aliases: vec![],
            sensitivity: Sensitivity::Unknown,
        };
        let records = vec![
            (
                format!("topics/topics/{}.md", a.id),
                Record::new("topic", &a.id, &op, &a).unwrap(),
            ),
            (
                format!("topics/topics/{}.md", b.id),
                Record::new("topic", &b.id, &op, &b).unwrap(),
            ),
        ];
        (temp, ctx, records, op)
    }
    #[test]
    fn interrupted_replacements_are_invisible_until_commit_and_recovery_is_idempotent() {
        let (_temp, mut ctx, records, op) = setup();
        let e = commit(
            &ctx,
            "Synthetic interrupted operation",
            records,
            CommitIntent::default(),
        )
        .unwrap_err();
        assert_eq!(e.code, "internal");
        let journal: Journal = read_json(
            &ctx.partition
                .join("journals")
                .join(&op)
                .join("journal.json"),
        )
        .unwrap();
        assert_eq!(journal.status, "staged");
        assert!(journal.targets.iter().all(|t| t.old_bytes.is_none()));
        assert!(
            target_path(&ctx.root, &journal.targets[0].path, false)
                .unwrap()
                .exists()
        );
        assert!(
            !target_path(&ctx.root, &journal.targets[1].path, false)
                .unwrap()
                .exists()
        );
        assert!(load(&ctx).unwrap().topics.is_empty());
        ctx.interrupt_after = None;
        recover(&ctx).unwrap();
        let state = load(&ctx).unwrap();
        assert_eq!(state.topics.len(), 2);
        assert_eq!(state.operations.len(), 1);
        recover(&ctx).unwrap();
        assert_eq!(load(&ctx).unwrap().operations.len(), 1);
    }
    #[test]
    fn recovery_conflict_retains_three_versions_and_never_publishes_marker() {
        let (_temp, mut ctx, records, op) = setup();
        assert!(
            commit(
                &ctx,
                "Synthetic interrupted operation",
                records,
                CommitIntent::default()
            )
            .is_err()
        );
        let dir = ctx.partition.join("journals").join(&op);
        let j: Journal = read_json(&dir.join("journal.json")).unwrap();
        let target = target_path(&ctx.root, &j.targets[1].path, false).unwrap();
        fs::write(&target, b"# External synthetic version\n").unwrap();
        ctx.interrupt_after = None;
        recover(&ctx).unwrap();
        let j: Journal = read_json(&dir.join("journal.json")).unwrap();
        assert_eq!(j.status, "conflict");
        let conflict: Value = read_json(&dir.join("conflict-1.json")).unwrap();
        assert!(conflict["old_bytes"].is_null());
        assert_eq!(
            serde_json::from_value::<Vec<u8>>(conflict["staged_bytes"].clone()).unwrap(),
            j.targets[1].new_bytes
        );
        assert_eq!(
            serde_json::from_value::<Vec<u8>>(conflict["current_bytes"].clone()).unwrap(),
            b"# External synthetic version\n"
        );
        assert!(load(&ctx).unwrap().topics.is_empty());
        assert!(load(&ctx).unwrap().operations.is_empty());
    }
}
