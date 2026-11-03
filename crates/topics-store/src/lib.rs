//! Read-only notes reader and replaceable SQLite index. No cloud access.
mod app_data;
mod parse;
mod scan;
mod workspace;
pub use app_data::application_data_dir;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};
use thiserror::Error;
pub use topics_core::{Document, IndexReport, Person, SearchResult};
pub use workspace::WorkspaceService;

pub type Result<T> = std::result::Result<T, StoreError>;
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Filesystem error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Index error: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("Configuration error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid derived index row: {0}")]
    IndexData(String),
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
            Self::IndexData(_) => "index",
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
    startup_scan: AtomicBool,
}
impl TopicsService {
    pub fn open(app_data_dir: impl AsRef<Path>) -> Result<Self> {
        fs::create_dir_all(app_data_dir.as_ref())?;
        let service = Self {
            app_data: fs::canonicalize(app_data_dir)?,
            startup_scan: AtomicBool::new(true),
        };
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
        let root = fs::canonicalize(root)
            .map_err(|error| StoreError::Invalid(format!("Notes root unavailable: {error}")))?;
        if !root.is_dir() {
            return Err(StoreError::Invalid("Notes root must be a directory".into()));
        }
        if self.app_data.starts_with(&root) || root.starts_with(&self.app_data) {
            return Err(StoreError::Invalid(
                "Notes root and application data must be separate directories".into(),
            ));
        }
        fs::read_dir(&root)
            .map_err(|error| StoreError::Invalid(format!("Notes root unavailable: {error}")))?;
        Ok(root)
    }
    pub fn use_notes_root(&mut self, path: impl AsRef<Path>) -> Result<IndexReport> {
        let _guard = self.lock()?;
        let root = self.validate_root(path.as_ref())?;
        let bytes = serde_json::to_vec_pretty(&Config {
            notes_root: Some(root.clone()),
        })?;
        self.recover_index(|| {
            self.index_root(&root, || {
                // The matching index transaction is still uncommitted here. Any
                // publication error rolls it back, preserving the selected old root.
                let temporary = self.app_data.join("config.json.tmp");
                fs::write(&temporary, &bytes)?;
                fs::rename(temporary, self.app_data.join("config.json"))?;
                Ok(())
            })
        })
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
        self.recover_index(|| {
            let root = self.selected_root()?;
            self.index_root(&root, || Ok(()))
        })
    }
    fn selected_root(&self) -> Result<PathBuf> {
        self.configured_root()?
            .ok_or_else(|| StoreError::Invalid("Choose a notes root first".into()))
    }
    fn recover_index<T>(&self, operation: impl Fn() -> Result<T>) -> Result<T> {
        let first = operation();
        let corrupt = match &first {
            Err(StoreError::Sql(error)) => error.sqlite_error_code().is_some_and(|code| {
                matches!(
                    code,
                    rusqlite::ErrorCode::DatabaseCorrupt | rusqlite::ErrorCode::NotADatabase
                )
            }),
            Err(StoreError::IndexData(_)) => true,
            _ => false,
        };
        if !corrupt {
            return first;
        }
        // Caller holds the shared OS guard; no Topics process has an index
        // transaction open here. Remove only the disposable index and journals.
        for file in [
            "index.sqlite3",
            "index.sqlite3-wal",
            "index.sqlite3-shm",
            "index.sqlite3-journal",
        ] {
            match fs::remove_file(self.app_data.join(file)) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
        self.startup_scan.store(true, Ordering::Release);
        operation()
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
    // A crash between config rename and SQLite commit can leave two valid
    // files identifying different roots. Rebuild the disposable index before
    // serving any rows. Legacy indexes without identity rebuild on first use.
    fn ensure_current_index(&self) -> Result<()> {
        let root = self.selected_root()?;
        if self.startup_scan.load(Ordering::Acquire)
            || self.indexed_root()?.as_deref() != Some(root.as_path())
        {
            self.index_root(&root, || Ok(()))?;
        }
        Ok(())
    }
    fn index_root(
        &self,
        root: &Path,
        publish_config: impl FnOnce() -> Result<()>,
    ) -> Result<IndexReport> {
        self.validate_root(root)?;
        let (records, warnings) = scan::scan(root)?;
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
            publish_config()?;
            self.startup_scan.store(false, Ordering::Release);
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
        publish_config()?;
        tx.commit()?;
        self.startup_scan.store(false, Ordering::Release);
        Ok(report)
    }
    pub fn documents(&self) -> Result<Vec<Document>> {
        let _guard = self.lock()?;
        self.recover_index(|| {
            self.ensure_current_index()?;
            self.read_rows("SELECT data FROM documents ORDER BY path")
        })
    }
    pub fn people(&self) -> Result<Vec<Person>> {
        let _guard = self.lock()?;
        self.recover_index(|| {
            self.ensure_current_index()?;
            self.read_rows("SELECT data FROM people ORDER BY path")
        })
    }
    fn read_rows<T: serde::de::DeserializeOwned>(&self, sql: &str) -> Result<Vec<T>> {
        let c = self.connection()?;
        let mut stmt = c.prepare(sql)?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        rows.map(|r| {
            serde_json::from_str(&r?).map_err(|error| StoreError::IndexData(error.to_string()))
        })
        .collect()
    }
    pub fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let _guard = self.lock()?;
        self.recover_index(|| {
            self.ensure_current_index()?;
            self.search_rows(query)
        })
    }
    fn search_rows(&self, query: &str) -> Result<Vec<SearchResult>> {
        // Treat user input as literal terms, not SQL or unrestricted FTS syntax.
        let query = query.replace('\0', " ");
        let terms: Vec<String> = query
            .split_whitespace()
            .filter(|term| term.chars().any(char::is_alphanumeric))
            .map(|t| format!("\"{}\"", t.replace('"', "\"\"")))
            .collect();
        if terms.is_empty() {
            return Ok(vec![]);
        }
        let c = self.connection()?;
        let mut stmt = c.prepare("SELECT title,path,snippet(search,-1,'<mark>','</mark>','…',24) FROM search WHERE search MATCH ?1 ORDER BY rank,path")?;
        let rows = stmt.query_map([terms.join(" AND ")], |r| {
            Ok(SearchResult {
                title: r.get(0)?,
                path: r.get(1)?,
                snippet: r.get(2)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }
    // No content reads/hashes at idle: only a metadata snapshot, including Unix
    // ctime/inode, is compared. Startup and explicit index always hash all bytes.
    fn poll_changes(&self, previous: &mut Option<Vec<String>>) -> Result<bool> {
        let _guard = self.lock()?;
        self.recover_index(|| {
            let root = self.selected_root()?;
            let snapshot = scan::snapshot(&root)?;
            if previous.as_ref() != Some(&snapshot)
                || self.indexed_root()?.as_deref() != Some(root.as_path())
            {
                self.index_root(&root, || Ok(()))?;
                Ok(Some(snapshot))
            } else {
                Ok(None)
            }
        })
        .map(|snapshot| {
            let changed = snapshot.is_some();
            if let Some(snapshot) = snapshot {
                *previous = Some(snapshot);
            }
            changed
        })
    }
    pub fn watch(&self) -> Result<WatchHandle> {
        if self.notes_root()?.is_none() {
            return Err(StoreError::Invalid("Choose a notes root first".into()));
        }
        let worker = Self {
            app_data: self.app_data.clone(),
            startup_scan: AtomicBool::new(true),
        };
        let stop = Arc::new(AtomicBool::new(false));
        let stop_worker = stop.clone();
        let last_error = Arc::new(Mutex::new(None));
        let errors = last_error.clone();
        let handle = thread::spawn(move || {
            let mut previous = None;
            while !stop_worker.load(Ordering::Acquire) {
                let result = worker.poll_changes(&mut previous);
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

#[cfg(test)]
mod watcher_tests {
    use super::*;

    #[test]
    fn idle_poll_does_not_rescan_bodies_but_metadata_change_does() {
        let temp = tempfile::tempdir().unwrap();
        let notes = temp.path().join("notes");
        fs::create_dir(&notes).unwrap();
        let note = notes.join("note.md");
        fs::write(&note, "OldRoot").unwrap();
        let mut service = TopicsService::open(temp.path().join("app")).unwrap();
        service.use_notes_root(&notes).unwrap();
        let mut previous = None;
        assert!(service.poll_changes(&mut previous).unwrap());
        for _ in 0..4 {
            assert!(!service.poll_changes(&mut previous).unwrap());
        }
        let mtime = fs::metadata(&note).unwrap().modified().unwrap();
        fs::write(&note, "NewRoot").unwrap();
        fs::File::options()
            .write(true)
            .open(&note)
            .unwrap()
            .set_modified(mtime)
            .unwrap();
        #[cfg(unix)]
        assert!(service.poll_changes(&mut previous).unwrap());
        #[cfg(not(unix))]
        service.index().unwrap();
        assert_eq!(service.search("NewRoot").unwrap().len(), 1);
        assert!(service.search("OldRoot").unwrap().is_empty());
    }
}
