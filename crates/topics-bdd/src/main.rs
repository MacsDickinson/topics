//! Phase 0 service examples. Run with `--tags 'not @ui'`; desktop steps live elsewhere.
mod snapshot;
mod steps;

use cucumber::World;
use snapshot::Snapshot;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use topics_store::{SearchResult, TopicsService, WatchHandle};

#[derive(World)]
#[world(init = Self::new)]
struct Notes {
    temp: tempfile::TempDir,
    service: TopicsService,
    root: PathBuf,
    before: Snapshot,
    results: Vec<SearchResult>,
    error: String,
    warnings: Vec<String>,
    watcher: Option<WatchHandle>,
}

impl std::fmt::Debug for Notes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Synthetic notes world")
    }
}

impl Notes {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        // Every scenario gets private application data as well as private source notes.
        let service = TopicsService::open(temp.path().join("app-data")).unwrap();
        let root = temp.path().join("notes");
        Self {
            temp,
            service,
            root,
            before: Snapshot::default(),
            results: vec![],
            error: String::new(),
            warnings: vec![],
            watcher: None,
        }
    }

    fn write_at(root: &Path, path: &str, bytes: impl AsRef<[u8]>) {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn write(&self, path: &str, bytes: impl AsRef<[u8]>) {
        Self::write_at(&self.root, path, bytes);
    }

    fn fixture_folder(&self, name: &str, count: usize) -> PathBuf {
        assert!(count > 0);
        let root = self.temp.path().join(name);
        Self::write_at(
            &root,
            "memory/projects/ledger-migration.md",
            "# Ledger migration\nLedger work",
        );
        for i in 1..count {
            Self::write_at(&root, &format!("document-{i}.md"), format!("# {name} {i}"));
        }
        root
    }

    fn remember_source(&mut self) {
        self.before = Snapshot::capture(&self.root);
    }

    fn assert_source_unchanged(&self, root: &Path) {
        assert_eq!(Snapshot::capture(root), self.before, "source tree changed");
    }

    fn has_result(&self, query: &str, path: &str) -> bool {
        self.service
            .search(query)
            .unwrap()
            .iter()
            .any(|result| result.path == path)
    }
}

/// Poll through service reads without manually indexing. Reads may repair a
/// startup/root/index mismatch; ordinary same-root edits require the watcher.
fn eventually(description: &str, mut observed: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if observed() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out after 2 seconds: {description}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[tokio::main]
async fn main() {
    Notes::cucumber()
        .fail_on_skipped()
        .run_and_exit("features/phase-0")
        .await;
}
