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
        targets: targets
            .iter()
            .map(|t| TargetMeta {
                path: t.path.clone(),
                expected_hash: t.expected_hash.clone(),
                new_hash: t.new_hash.clone(),
            })
            .collect(),
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
            "Record changed since proposal review; snapshots and staged bytes preserved",
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
    save_json(&dir.join("journal.json"), journal)
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
pub(super) fn recover(ctx: &Context) -> WResult<()> {
    let base = ctx.partition.join("journals");
    if !base.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(base).map_err(internal)? {
        let entry = entry.map_err(internal)?;
        if entry.file_type().map_err(internal)?.is_symlink() {
            return Err(error("forbidden", "Invalid journal directory"));
        }
        let path = entry.path().join("journal.json");
        if !path.exists() {
            continue;
        }
        let mut journal: Journal = read_json(&path)?;
        valid_id(&journal.commit.operation.id)?;
        if journal.status == "staged" {
            // Conflicts are durable and inspectable; they never expose incomplete records.
            if let Err(e) = apply_journal(ctx, &entry.path(), &mut journal, None)
                && e.code != "conflict"
            {
                return Err(e);
            }
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
    pub operations: Vec<Commit>,
    pub warnings: Vec<String>,
    pub hashes: BTreeMap<String, Option<Hash>>,
}
pub(super) fn load(ctx: &Context) -> WResult<State> {
    let mut state = State::default();
    let mut records = vec![];
    for dir in [
        "operations",
        "topics",
        "claims",
        "connections",
        "rules",
        "sources",
        "statements",
        "unresolved",
    ] {
        let parent = ctx.root.join("topics").join(dir);
        if !parent.exists() {
            continue;
        }
        // Validate all ancestors before reading app-owned bytes.
        target_path(
            &ctx.root,
            &format!("topics/{dir}/{}.md", uuid::Uuid::nil()),
            false,
        )?;
        for entry in fs::read_dir(parent).map_err(internal)? {
            let entry = entry.map_err(internal)?;
            let Some(name) = entry.file_name().to_str().map(str::to_string) else {
                state
                    .warnings
                    .push("Non UTF-8 record filename excluded".into());
                continue;
            };
            if name.starts_with('.') {
                continue;
            }
            let path = format!("topics/{dir}/{name}");
            let parsed = (|| {
                let full = target_path(&ctx.root, &path, false)?;
                let bytes = fs::read(full).map_err(internal)?;
                let record = Record::parse(&bytes)?;
                let expected_kind = match dir {
                    "topics" => "topic",
                    "claims" => "claim",
                    "connections" => "connection",
                    "rules" => "rule",
                    "sources" => "source",
                    "statements" => "statement",
                    "operations" => "operation",
                    _ => "unresolved",
                };
                if record.record_kind != expected_kind {
                    return Err(error(
                        "invalid_input",
                        "Record kind differs from its directory",
                    ));
                }
                valid_id(&record.id)?;
                valid_id(&record.operation_id)?;
                if name != format!("{}.md", record.id) {
                    return Err(error(
                        "invalid_input",
                        "Record identity differs from filename",
                    ));
                }
                Ok((record, hash(&bytes)))
            })();
            match parsed {
                Ok((record, h)) => {
                    state.hashes.insert(path.clone(), Some(h));
                    records.push((path, record));
                }
                Err(_) => state
                    .warnings
                    .push(format!("Malformed or unsafe record excluded: {path}")),
            }
        }
    }
    for (_, r) in records.iter().filter(|(_, r)| r.record_kind == "operation") {
        match serde_json::from_value::<Commit>(r.data.clone()) {
            Ok(c) if c.operation.id == r.id && c.operation.status == "committed" => {
                state.operations.push(c)
            }
            _ => state.warnings.push("Malformed operation excluded".into()),
        }
    }
    state
        .operations
        .sort_by_key(|c| (c.operation.created_at.clone(), c.operation.id.clone()));
    let committed: BTreeMap<_, _> = state
        .operations
        .iter()
        .map(|c| (c.operation.id.clone(), c))
        .collect();
    let operation_order: BTreeMap<_, _> = state
        .operations
        .iter()
        .enumerate()
        .map(|(i, c)| (c.operation.id.clone(), i))
        .collect();
    let mut tombstones = BTreeMap::new();
    for (order, commit) in state.operations.iter().enumerate() {
        for id in &commit.tombstones {
            tombstones.insert(id.clone(), order);
        }
    }
    let mut ids = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for (_, r) in &records {
        if r.record_kind != "operation" && !ids.insert(r.id.clone()) {
            duplicates.insert(r.id.clone());
        }
    }
    for (path, r) in records.iter().filter(|(_, r)| r.record_kind != "operation") {
        if duplicates.contains(&r.id) {
            state
                .warnings
                .push(format!("Duplicate record identity excluded: {}", r.id));
            continue;
        }
        if tombstones.get(&r.id).is_some_and(|order| {
            operation_order
                .get(&r.operation_id)
                .is_none_or(|record_order| order >= record_order)
        }) {
            continue;
        }
        let Some(op) = committed.get(&r.operation_id) else {
            continue;
        };
        if !op
            .targets
            .iter()
            .any(|t| t.path == *path && state.hashes.get(path) == Some(&Some(t.new_hash.clone())))
        {
            state
                .warnings
                .push(format!("Externally changed record excluded: {path}"));
            continue;
        }
        let identity_field = match r.record_kind.as_str() {
            "claim" => "revision_id",
            "source" => "source_id",
            "unresolved" => "",
            _ => "id",
        };
        if !identity_field.is_empty()
            && r.data.get(identity_field).and_then(Value::as_str) != Some(&r.id)
        {
            state.warnings.push(format!(
                "Record payload identity differs from filename: {path}"
            ));
            continue;
        }
        macro_rules! insert {
            ($map:ident,$ty:ty) => {
                match serde_json::from_value::<$ty>(r.data.clone()) {
                    Ok(v) => {
                        state.$map.insert(r.id.clone(), v);
                    }
                    Err(_) => state
                        .warnings
                        .push(format!("Invalid record data excluded: {path}")),
                }
            };
        }
        match r.record_kind.as_str() {
            "topic" => insert!(topics, Topic),
            "claim" => insert!(claims, Claim),
            "connection" => insert!(connections, Connection),
            "rule" => insert!(rules, Rule),
            "source" => insert!(sources, SourceSummary),
            "statement" => insert!(statements, UserStatement),
            "unresolved" => match serde_json::from_value(r.data.clone()) {
                Ok(c) => state.unresolved.push(c),
                Err(_) => state
                    .warnings
                    .push("Invalid unresolved identity excluded".into()),
            },
            _ => state
                .warnings
                .push(format!("Unknown record kind excluded: {path}")),
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
