use std::{fs, thread, time::Duration};
use topics_store::TopicsService;

#[test]
fn independent_instances_and_old_watcher_never_restore_previous_root() {
    let temp = tempfile::tempdir().unwrap();
    let app = temp.path().join("app");
    let old = temp.path().join("old");
    let new = temp.path().join("new");
    fs::create_dir(&old).unwrap();
    fs::create_dir(&new).unwrap();
    fs::write(old.join("old.md"), "OldRoot ledger").unwrap();
    fs::write(new.join("new.md"), "NewRoot ledger").unwrap();
    let mut first = TopicsService::open(&app).unwrap();
    first.use_notes_root(&old).unwrap();
    let watcher = first.watch().unwrap();
    let mut independent = TopicsService::open(&app).unwrap();
    independent.use_notes_root(&new).unwrap();
    // An already-open instance must discover the current selection even when it
    // explicitly indexes immediately after another instance has switched roots.
    first.index().unwrap();
    for _ in 0..4 {
        // Probe SQLite directly before any service read can repair it: an old
        // watcher must never restore old-root rows and hide that behind refresh.
        let db = rusqlite::Connection::open(app.join("index.sqlite3")).unwrap();
        let paths: String = db
            .query_row("SELECT group_concat(path) FROM documents", [], |r| r.get(0))
            .unwrap();
        assert_eq!(paths, "new.md");
        assert_eq!(
            first.notes_root().unwrap().unwrap(),
            fs::canonicalize(&new).unwrap()
        );
        assert!(first.search("OldRoot").unwrap().is_empty());
        assert_eq!(first.search("NewRoot").unwrap().len(), 1);
        assert!(independent.search("OldRoot").unwrap().is_empty());
        assert_eq!(independent.search("NewRoot").unwrap().len(), 1);
        thread::sleep(Duration::from_millis(250));
    }
    assert!(watcher.last_error().is_none());
    assert_eq!(
        fs::read_to_string(old.join("old.md")).unwrap(),
        "OldRoot ledger"
    );
    assert_eq!(
        fs::read_to_string(new.join("new.md")).unwrap(),
        "NewRoot ledger"
    );
}

#[test]
fn reads_recover_a_config_index_mismatch_after_interrupted_switch() {
    let temp = tempfile::tempdir().unwrap();
    let app = temp.path().join("app");
    let old = temp.path().join("old");
    let new = temp.path().join("new");
    fs::create_dir(&old).unwrap();
    fs::create_dir(&new).unwrap();
    fs::write(old.join("old.md"), "OldRoot").unwrap();
    fs::write(new.join("new.md"), "NewRoot").unwrap();
    let mut retained = TopicsService::open(&app).unwrap();
    retained.use_notes_root(&old).unwrap();
    let old_config = fs::read(app.join("config.json")).unwrap();
    let mut independent = TopicsService::open(&app).unwrap();
    independent.use_notes_root(&new).unwrap();
    // Simulate a process terminating after publishing the new index but before
    // replacing the old config: both files remain individually valid.
    fs::write(app.join("config.json"), old_config).unwrap();
    assert_eq!(retained.search("OldRoot").unwrap().len(), 1);
    assert!(retained.search("NewRoot").unwrap().is_empty());
    assert_eq!(retained.documents().unwrap()[0].path, "old.md");
}
