//! Read-only notes reader and replaceable SQLite index. No cloud access.
mod app_data;
mod parse;
pub use app_data::application_data_dir;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, UNIX_EPOCH},
};
use thiserror::Error;
pub use topics_core::{Document, IndexReport, Person, SearchResult};

pub type Result<T> = std::result::Result<T, StoreError>;
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Filesystem error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Index error: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("Configuration error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Invalid(String),
}
impl Serialize for StoreError {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let code = match self {
            Self::Io(_) => "filesystem",
            Self::Sql(_) => "index",
            Self::Json(_) => "configuration",
            Self::Invalid(_) => "invalid_input",
        };
        let mut value = serializer.serialize_struct("StoreError", 2)?;
        value.serialize_field("code", code)?;
        value.serialize_field("message", &self.to_string())?;
        value.end()
    }
}
#[derive(Default, Serialize, Deserialize)]
struct Config {
    notes_root: Option<PathBuf>,
}

pub struct TopicsService {
    app_data: PathBuf,
}
impl TopicsService {
    pub fn open(app_data_dir: impl AsRef<Path>) -> Result<Self> {
        fs::create_dir_all(app_data_dir.as_ref())?;
        let service = Self {
            app_data: fs::canonicalize(app_data_dir)?,
        };
        let _guard = service.lock()?;
        if let Some(root) = service.configured_root()? {
            service.index_root(&root)?;
        }
        Ok(service)
    }
    // Each descriptor owns an OS lock shared across threads, instances and
    // processes. Dropping it releases the lock on success, error or process exit.
    // Never remove this file: replacing its inode would split the lock domain.
    fn lock(&self) -> Result<fs::File> {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.app_data.join("store.lock"))?;
        file.lock()?;
        Ok(file)
    }
    // Caller holds store.lock throughout config discovery and any index use.
    fn configured_root(&self) -> Result<Option<PathBuf>> {
        let config: Config = match fs::read(self.app_data.join("config.json")) {
            Ok(bytes) => serde_json::from_slice(&bytes)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Config::default(),
            Err(e) => return Err(e.into()),
        };
        config
            .notes_root
            .map(|root| self.validate_root(&root))
            .transpose()
    }
    pub fn notes_root(&self) -> Result<Option<PathBuf>> {
        let _guard = self.lock()?;
        self.configured_root()
    }
    fn validate_root(&self, root: &Path) -> Result<PathBuf> {
        let root = fs::canonicalize(root)?;
        if !root.is_dir() {
            return Err(StoreError::Invalid("Notes root must be a directory".into()));
        }
        if self.app_data.starts_with(&root) || root.starts_with(&self.app_data) {
            return Err(StoreError::Invalid(
                "Notes root and application data must be separate directories".into(),
            ));
        }
        Ok(root)
    }
    pub fn use_notes_root(&mut self, path: impl AsRef<Path>) -> Result<IndexReport> {
        let _guard = self.lock()?;
        let root = self.validate_root(path.as_ref())?;
        // Readers and other scanners cannot observe the intermediate new index
        // with old config. The index's root identity repairs that pair after a crash.
        let report = self.index_root(&root)?;
        let bytes = serde_json::to_vec_pretty(&Config {
            notes_root: Some(root),
        })?;
        let temporary = self.app_data.join("config.json.tmp");
        fs::write(&temporary, bytes)?;
        fs::rename(temporary, self.app_data.join("config.json"))?;
        Ok(report)
    }
    pub fn init_notes_root(&mut self, path: impl AsRef<Path>) -> Result<IndexReport> {
        let path = path.as_ref();
        // Validate separation using the nearest existing ancestor before creating anything.
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };
        if absolute
            .components()
            .any(|c| c == std::path::Component::ParentDir)
        {
            return Err(StoreError::Invalid(
                "Use a notes path without parent traversal".into(),
            ));
        }
        let mut ancestor = absolute.as_path();
        while !ancestor.exists() {
            ancestor = ancestor
                .parent()
                .ok_or_else(|| StoreError::Invalid("Invalid notes root".into()))?;
        }
        let canonical_ancestor = fs::canonicalize(ancestor)?;
        let candidate = canonical_ancestor.join(
            absolute
                .strip_prefix(ancestor)
                .map_err(|e| StoreError::Invalid(e.to_string()))?,
        );
        if self.app_data.starts_with(&candidate) || candidate.starts_with(&self.app_data) {
            return Err(StoreError::Invalid(
                "Notes root and application data must be separate directories".into(),
            ));
        }
        if path.exists() && (!path.is_dir() || fs::read_dir(path)?.next().is_some()) {
            return Err(StoreError::Invalid(
                "Folder is not empty; use it instead".into(),
            ));
        }
        fs::create_dir_all(path)?;
        self.validate_root(path)?;
        for folder in [
            "memory/people",
            "memory/projects",
            "Meetings",
            "Log",
            "topics",
        ] {
            fs::create_dir_all(path.join(folder))?;
        }
        self.use_notes_root(path)
    }
    fn connection(&self) -> Result<Connection> {
        let c = Connection::open(self.app_data.join("index.sqlite3"))?;
        c.busy_timeout(Duration::from_secs(3))?;
        c.execute_batch("CREATE TABLE IF NOT EXISTS index_state(id INTEGER PRIMARY KEY CHECK(id=1), root TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS documents(path TEXT PRIMARY KEY, data TEXT NOT NULL, body TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS people(path TEXT PRIMARY KEY, data TEXT NOT NULL);
            CREATE VIRTUAL TABLE IF NOT EXISTS search USING fts5(path UNINDEXED, title, body);")?;
        Ok(c)
    }
    pub fn index(&self) -> Result<IndexReport> {
        let _guard = self.lock()?;
        let root = self
            .configured_root()?
            .ok_or_else(|| StoreError::Invalid("Choose a notes root first".into()))?;
        self.index_root(&root)
    }
    fn indexed_root(&self) -> Result<Option<PathBuf>> {
        Ok(self
            .connection()?
            .query_row("SELECT root FROM index_state WHERE id=1", [], |row| {
                row.get::<_, String>(0)
            })
            .optional()?
            .map(PathBuf::from))
    }
    // A crash between the SQLite commit and config rename leaves two valid
    // files identifying different roots. Rebuild the disposable index before
    // serving any rows. Legacy indexes without identity rebuild on first use.
    fn ensure_current_index(&self) -> Result<()> {
        let configured = self.configured_root()?;
        let indexed = self.indexed_root()?;
        if configured.is_none() || configured != indexed {
            if let Some(root) = configured {
                self.index_root(&root)?;
            } else {
                let mut c = self.connection()?;
                let tx = c.transaction()?;
                tx.execute_batch("DELETE FROM documents; DELETE FROM people; DELETE FROM search; DELETE FROM index_state;")?;
                tx.commit()?;
            }
        }
        Ok(())
    }
    fn index_root(&self, root: &Path) -> Result<IndexReport> {
        self.validate_root(root)?;
        let mut records = Vec::new();
        let mut warnings = Vec::new();
        for entry in walkdir::WalkDir::new(root).follow_links(false) {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warnings.push(e.to_string());
                    continue;
                }
            };
            if !entry.file_type().is_file() || entry.path().extension().is_none_or(|x| x != "md") {
                continue;
            }
            let path = entry
                .path()
                .strip_prefix(root)
                .map_err(|e| StoreError::Invalid(e.to_string()))?
                .to_string_lossy()
                .replace('\\', "/");
            let bytes = match fs::read(entry.path()) {
                Ok(b) => b,
                Err(e) => {
                    warnings.push(format!("{path}: {e}"));
                    continue;
                }
            };
            let body = match String::from_utf8(bytes.clone()) {
                Ok(s) => s,
                Err(_) => {
                    warnings.push(format!(
                        "{path}: invalid UTF-8; indexed as a plain document"
                    ));
                    String::from_utf8_lossy(&bytes).into_owned()
                }
            };
            let modified = fs::metadata(entry.path())?
                .modified()?
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
                .to_string();
            let mut document = parse::document(
                &path,
                &body,
                format!("{:x}", Sha256::digest(&bytes)),
                modified,
            );
            if std::str::from_utf8(&bytes).is_err() {
                document.kind = "other".into();
                document.date = None;
            }
            records.push((document, body));
        }
        records.sort_by(|a, b| a.0.path.cmp(&b.0.path));
        // Still hash every source: mtime alone cannot detect all offline edits.
        // Avoid rewriting SQLite on every idle polling tick.
        let report = IndexReport {
            document_count: records.len(),
            warnings,
        };
        if self.indexed_root()?.as_deref() == Some(root)
            && self.read_rows::<Document>("SELECT data FROM documents ORDER BY path")?
                == records.iter().map(|(d, _)| d.clone()).collect::<Vec<_>>()
        {
            return Ok(report);
        }
        let aliases = parse::aliases(
            records
                .iter()
                .filter(|(d, _)| d.path == "memory/glossary.md")
                .map(|(_, b)| b.as_str())
                .next()
                .unwrap_or(""),
        );
        let mut c = self.connection()?;
        let tx = c.transaction()?;
        tx.execute_batch("DELETE FROM documents; DELETE FROM people; DELETE FROM search;")?;
        for (document, body) in &records {
            tx.execute(
                "INSERT INTO documents VALUES (?1,?2,?3)",
                params![document.path, serde_json::to_string(document)?, body],
            )?;
            tx.execute(
                "INSERT INTO search(path,title,body) VALUES (?1,?2,?3)",
                params![document.path, document.title, body],
            )?;
            if document.kind == "person" {
                let person = parse::person(document, body, &aliases);
                tx.execute(
                    "INSERT INTO people VALUES (?1,?2)",
                    params![person.path, serde_json::to_string(&person)?],
                )?;
            }
        }
        tx.execute(
            "INSERT OR REPLACE INTO index_state(id,root) VALUES (1,?1)",
            [root.to_string_lossy().as_ref()],
        )?;
        tx.commit()?;
        Ok(report)
    }
    pub fn documents(&self) -> Result<Vec<Document>> {
        let _guard = self.lock()?;
        self.ensure_current_index()?;
        self.read_rows("SELECT data FROM documents ORDER BY path")
    }
    pub fn people(&self) -> Result<Vec<Person>> {
        let _guard = self.lock()?;
        self.ensure_current_index()?;
        self.read_rows("SELECT data FROM people ORDER BY path")
    }
    fn read_rows<T: serde::de::DeserializeOwned>(&self, sql: &str) -> Result<Vec<T>> {
        let c = self.connection()?;
        let mut stmt = c.prepare(sql)?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        rows.map(|r| Ok(serde_json::from_str(&r?)?)).collect()
    }
    pub fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let _guard = self.lock()?;
        self.ensure_current_index()?;
        // Treat user input as literal terms, not SQL or unrestricted FTS syntax.
        let terms: Vec<String> = query
            .split_whitespace()
            .map(|t| format!("\"{}\"", t.replace('"', "\"\"")))
            .collect();
        if terms.is_empty() {
            return Ok(vec![]);
        }
        let c = self.connection()?;
        let mut stmt = c.prepare("SELECT title,path,snippet(search,2,'<mark>','</mark>','…',24) FROM search WHERE search MATCH ?1 ORDER BY rank,path")?;
        let rows = stmt.query_map([terms.join(" AND ")], |r| {
            Ok(SearchResult {
                title: r.get(0)?,
                path: r.get(1)?,
                snippet: r.get(2)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }
    pub fn watch(&self) -> Result<WatchHandle> {
        if self.notes_root()?.is_none() {
            return Err(StoreError::Invalid("Choose a notes root first".into()));
        }
        let worker = Self {
            app_data: self.app_data.clone(),
        };
        let stop = Arc::new(AtomicBool::new(false));
        let stop_worker = stop.clone();
        let last_error = Arc::new(Mutex::new(None));
        let errors = last_error.clone();
        let handle = thread::spawn(move || {
            while !stop_worker.load(Ordering::Acquire) {
                let result = worker.index();
                if let Ok(mut error) = errors.lock() {
                    *error = result.err().map(|e| e.to_string());
                }
                thread::park_timeout(Duration::from_millis(200));
            }
        });
        Ok(WatchHandle {
            stop,
            handle: Some(handle),
            last_error,
        })
    }
}
/// Keep alive for the application's lifetime. Drop stops and joins the watcher.
pub struct WatchHandle {
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
    last_error: Arc<Mutex<Option<String>>>,
}
impl WatchHandle {
    pub fn last_error(&self) -> Option<String> {
        self.last_error.lock().ok().and_then(|e| e.clone())
    }
}
impl Drop for WatchHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            handle.thread().unpark();
            let _ = handle.join();
        }
    }
}
