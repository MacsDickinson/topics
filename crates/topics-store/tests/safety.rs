use std::{
    fs,
    time::{Duration, Instant},
};
use topics_store::TopicsService;

#[test]
fn root_boundaries_and_nonempty_init_leave_sources_untouched() {
    let temp = tempfile::tempdir().unwrap();
    let app = temp.path().join("app");
    let mut service = TopicsService::open(&app).unwrap();
    assert!(service.index().is_err());
    assert!(service.watch().is_err());
    assert!(service.use_notes_root(&app).is_err());
    assert!(service.use_notes_root(temp.path()).is_err());
    assert!(service.init_notes_root(app.join("notes")).is_err());
    assert!(!app.join("notes").exists());
    assert!(
        service
            .init_notes_root(temp.path().join("missing/../app/notes"))
            .is_err()
    );
    assert!(!temp.path().join("missing").exists());
    let notes = temp.path().join("notes");
    fs::create_dir(&notes).unwrap();
    fs::write(notes.join("keep.md"), b"unchanged").unwrap();
    let modified = fs::metadata(notes.join("keep.md"))
        .unwrap()
        .modified()
        .unwrap();
    assert!(
        service
            .init_notes_root(&notes)
            .unwrap_err()
            .to_string()
            .contains("not empty")
    );
    assert_eq!(fs::read(notes.join("keep.md")).unwrap(), b"unchanged");
    assert_eq!(
        fs::metadata(notes.join("keep.md"))
            .unwrap()
            .modified()
            .unwrap(),
        modified
    );
    assert_eq!(fs::read_dir(notes).unwrap().count(), 1);
}
#[test]
fn startup_hash_scan_detects_equal_mtime_edit_and_invalid_utf8_stays_plain() {
    let temp = tempfile::tempdir().unwrap();
    let notes = temp.path().join("notes");
    let app = temp.path().join("app");
    fs::create_dir_all(notes.join("memory/people")).unwrap();
    let path = notes.join("meeting.md");
    fs::write(&path, "Ledger old").unwrap();
    fs::write(notes.join("memory/people/broken.md"), [0xff]).unwrap();
    let mut service = TopicsService::open(&app).unwrap();
    let report = service.use_notes_root(&notes).unwrap();
    assert_eq!(report.document_count, 2);
    assert_eq!(report.warnings.len(), 1);
    assert!(service.people().unwrap().is_empty());
    let old = service
        .documents()
        .unwrap()
        .into_iter()
        .find(|d| d.path == "meeting.md")
        .unwrap();
    let old_time = fs::metadata(&path).unwrap().modified().unwrap();
    drop(service);
    fs::write(&path, "Vendor new").unwrap();
    fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_modified(old_time)
        .unwrap();
    let service = TopicsService::open(&app).unwrap();
    let new = service
        .documents()
        .unwrap()
        .into_iter()
        .find(|d| d.path == "meeting.md")
        .unwrap();
    assert_eq!(old.modified_ns, new.modified_ns);
    assert_ne!(old.hash, new.hash);
    assert_eq!(service.search("Vendor").unwrap().len(), 1);
    assert!(service.search("Ledger").unwrap().is_empty());
    for query in ["", "   ", "\"", "OR", "*", "' OR 1=1 --", "NEAR(", "🦀"] {
        assert!(service.search(query).is_ok(), "{query}");
    }
}
#[cfg(unix)]
#[test]
fn symlinks_never_expand_scan_beyond_root() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    let notes = temp.path().join("notes");
    fs::create_dir(&notes).unwrap();
    let outside = temp.path().join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("secret.md"), "secret").unwrap();
    symlink(&outside, notes.join("external")).unwrap();
    symlink(outside.join("secret.md"), notes.join("secret.md")).unwrap();
    symlink(&notes, notes.join("loop")).unwrap();
    let mut service = TopicsService::open(temp.path().join("app")).unwrap();
    assert_eq!(service.use_notes_root(&notes).unwrap().document_count, 0);
    assert!(service.search("secret").unwrap().is_empty());
}
#[test]
fn malformed_config_is_an_explicit_error() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("config.json"), "not json").unwrap();
    assert!(TopicsService::open(temp.path()).is_err());
}
#[test]
fn retained_watcher_follows_root_switch() {
    let temp = tempfile::tempdir().unwrap();
    let a = temp.path().join("a");
    let b = temp.path().join("b");
    fs::create_dir(&a).unwrap();
    fs::create_dir(&b).unwrap();
    fs::write(a.join("old.md"), "OldRoot").unwrap();
    fs::write(b.join("new.md"), "NewRoot").unwrap();
    let mut service = TopicsService::open(temp.path().join("app")).unwrap();
    service.use_notes_root(&a).unwrap();
    let watcher = service.watch().unwrap();
    service.use_notes_root(&b).unwrap();
    fs::write(b.join("new.md"), "Northwind").unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while service.search("Northwind").unwrap().is_empty() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(service.search("OldRoot").unwrap().is_empty());
    assert!(watcher.last_error().is_none());
}
