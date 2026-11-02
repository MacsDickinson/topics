use std::{
    fs,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};
use topics_store::TopicsService;

fn setup() -> (tempfile::TempDir, std::path::PathBuf, TopicsService) {
    let temp = tempfile::tempdir().unwrap();
    let notes = temp.path().join("notes");
    fs::create_dir(&notes).unwrap();
    let service = TopicsService::open(temp.path().join("app")).unwrap();
    (temp, notes, service)
}

#[test]
fn punctuation_terms_and_nul_do_not_remove_real_search_hits() {
    let (_temp, notes, mut service) = setup();
    fs::write(notes.join("ledger.md"), "# Ledger\nLedger vendor migration").unwrap();
    service.use_notes_root(notes).unwrap();
    for query in [
        "ledger &",
        "& ledger",
        "ledger -",
        "ledger - vendor",
        "ledger\0",
        "\0ledger\0 &",
    ] {
        assert_eq!(service.search(query).unwrap().len(), 1, "{query:?}");
    }
    assert!(service.search("& - \0").unwrap().is_empty());
}

#[test]
fn title_only_match_has_a_highlighted_excerpt() {
    let (_temp, notes, mut service) = setup();
    fs::create_dir(notes.join("Meetings")).unwrap();
    fs::write(
        notes.join("Meetings/2026-01-01-standup.md"),
        "# Notes\nNothing relevant",
    )
    .unwrap();
    service.use_notes_root(notes).unwrap();
    let rows = service.search("standup").unwrap();
    assert_eq!(rows.len(), 1);
    assert!(
        rows[0].snippet.contains("<mark>standup</mark>"),
        "{}",
        rows[0].snippet
    );
}

#[cfg(unix)]
#[test]
fn unix_backslashes_and_real_subdirectories_are_distinct_paths() {
    let (_temp, notes, mut service) = setup();
    fs::create_dir(notes.join("a")).unwrap();
    fs::write(notes.join("a/b.md"), "# Nested\nledger").unwrap();
    fs::write(notes.join("a\\b.md"), "# Backslash\nledger").unwrap();
    fs::write(
        notes.join("memory\\people\\literal.md"),
        "# Literal\nledger",
    )
    .unwrap();
    assert_eq!(service.use_notes_root(notes).unwrap().document_count, 3);
    let paths: Vec<_> = service
        .documents()
        .unwrap()
        .into_iter()
        .map(|d| d.path)
        .collect();
    assert_eq!(paths, ["a/b.md", "a\\b.md", "memory\\people\\literal.md"]);
    assert!(service.people().unwrap().is_empty());
}

#[test]
fn config_publication_failure_keeps_selected_root_and_raw_index_unchanged() {
    let (temp, notes, mut service) = setup();
    fs::write(notes.join("old.md"), "OldRoot").unwrap();
    service.use_notes_root(&notes).unwrap();
    let app = temp.path().join("app");
    let original_config = fs::read(app.join("config.json")).unwrap();
    let other = temp.path().join("other");
    fs::create_dir(&other).unwrap();
    fs::write(other.join("new.md"), "NewRoot").unwrap();
    fs::create_dir(app.join("config.json.tmp")).unwrap();
    assert!(service.use_notes_root(&other).is_err());
    assert_eq!(fs::read(app.join("config.json")).unwrap(), original_config);
    let db = rusqlite::Connection::open(app.join("index.sqlite3")).unwrap();
    let path: String = db
        .query_row("SELECT path FROM documents", [], |r| r.get(0))
        .unwrap();
    assert_eq!(path, "old.md");
    assert_eq!(
        service.notes_root().unwrap().unwrap(),
        fs::canonicalize(notes).unwrap()
    );
}

#[test]
fn no_root_reads_report_choose_root_instead_of_no_matches() {
    let (_temp, _notes, service) = setup();
    for result in [
        service.search("ledger").map(|_| ()),
        service.people().map(|_| ()),
        service.documents().map(|_| ()),
    ] {
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Choose a notes root")
        );
    }
}

#[test]
fn corrupt_disposable_index_rebuilds_and_missing_root_can_be_reconfigured() {
    let (temp, notes, mut service) = setup();
    fs::write(notes.join("old.md"), "OldRoot").unwrap();
    service.use_notes_root(&notes).unwrap();
    let app = temp.path().join("app");
    fs::write(app.join("index.sqlite3"), "not a database").unwrap();
    let recovered = TopicsService::open(&app).unwrap();
    assert_eq!(recovered.search("OldRoot").unwrap().len(), 1);
    fs::rename(&notes, temp.path().join("gone")).unwrap();
    let mut unavailable = TopicsService::open(&app).unwrap();
    assert!(unavailable.search("OldRoot").is_err());
    let replacement = temp.path().join("replacement");
    fs::create_dir(&replacement).unwrap();
    fs::write(replacement.join("new.md"), "NewRoot").unwrap();
    unavailable.use_notes_root(replacement).unwrap();
    assert!(unavailable.search("OldRoot").unwrap().is_empty());
    assert_eq!(unavailable.search("NewRoot").unwrap().len(), 1);
}

#[cfg(unix)]
struct RestorePermissions(std::path::PathBuf, fs::Permissions);
#[cfg(unix)]
impl Drop for RestorePermissions {
    fn drop(&mut self) {
        fs::set_permissions(&self.0, self.1.clone()).unwrap();
    }
}
#[cfg(unix)]
fn deny(path: &Path) -> RestorePermissions {
    use std::os::unix::fs::PermissionsExt;
    let original = fs::metadata(path).unwrap().permissions();
    fs::set_permissions(path, fs::Permissions::from_mode(0o0)).unwrap();
    RestorePermissions(path.to_path_buf(), original)
}

#[cfg(unix)]
#[test]
fn unreadable_file_keeps_plain_record_without_stale_body_and_unreadable_root_fails() {
    let (temp, notes, mut service) = setup();
    let note = notes.join("ledger.md");
    fs::write(&note, "# Ledger\nStaleSecret").unwrap();
    service.use_notes_root(&notes).unwrap();
    let permission = deny(&note);
    let report = service.index().unwrap();
    assert_eq!(report.document_count, 1);
    assert_eq!(report.warnings.len(), 1);
    assert!(report.warnings[0].starts_with("ledger.md:"));
    assert!(!report.warnings[0].contains(temp.path().to_str().unwrap()));
    assert_eq!(service.documents().unwrap()[0].kind, "other");
    assert!(service.search("StaleSecret").unwrap().is_empty());
    drop(permission);
    service.index().unwrap();
    let permission = deny(&notes);
    assert!(service.index().is_err());
    assert!(service.search("StaleSecret").is_err());
    let db = rusqlite::Connection::open(temp.path().join("app/index.sqlite3")).unwrap();
    assert_eq!(
        db.query_row("SELECT count(*) FROM documents", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
    drop(permission);
}

#[test]
fn concurrent_unlinks_warn_instead_of_aborting_scans() {
    let (_temp, notes, mut service) = setup();
    for index in 0..300 {
        fs::write(notes.join(format!("{index:04}.md")), "Synthetic churn").unwrap();
    }
    service.use_notes_root(&notes).unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let flag = stop.clone();
    let root = notes.clone();
    let churn = thread::spawn(move || {
        while !flag.load(Ordering::Acquire) {
            for index in 0..300 {
                let path = root.join(format!("{index:04}.md"));
                let _ = fs::remove_file(&path);
                let _ = fs::write(path, "Synthetic churn");
            }
        }
    });
    let mut failure = None;
    for _ in 0..100 {
        if let Err(error) = service.index() {
            failure = Some(error);
            break;
        }
    }
    stop.store(true, Ordering::Release);
    churn.join().unwrap();
    assert!(failure.is_none(), "{failure:?}");
}

#[test]
fn watcher_reports_root_loss_recovers_and_stops_on_drop() {
    use std::time::{Duration, Instant};
    let (temp, notes, mut service) = setup();
    fs::write(notes.join("note.md"), "Before").unwrap();
    service.use_notes_root(&notes).unwrap();
    let watcher = service.watch().unwrap();
    fs::rename(&notes, temp.path().join("gone")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while watcher.last_error().is_none() {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(20));
    }
    fs::create_dir(&notes).unwrap();
    fs::write(notes.join("note.md"), "Recovered").unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while watcher.last_error().is_some() || service.search("Recovered").unwrap().is_empty() {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(20));
    }
    drop(watcher);
    fs::write(notes.join("note.md"), "AfterDrop").unwrap();
    thread::sleep(Duration::from_millis(450));
    assert!(service.search("AfterDrop").unwrap().is_empty());
    assert_eq!(service.search("Recovered").unwrap().len(), 1);
    service.index().unwrap();
    assert_eq!(service.search("AfterDrop").unwrap().len(), 1);
}

#[test]
fn edited_deleted_profiles_and_multiple_glossary_aliases_are_replaced() {
    let (_temp, notes, mut service) = setup();
    fs::create_dir_all(notes.join("memory/people")).unwrap();
    let person = notes.join("memory/people/alex.md");
    let other = notes.join("memory/people/blair.md");
    fs::write(&person, "# Alex\n**Role:** Engineer").unwrap();
    fs::write(&other, "# Blair\n**Role:** Designer").unwrap();
    let glossary = notes.join("memory/glossary.md");
    fs::write(
        &glossary,
        "## Nicknames\n| Nickname | Name |\n| --- | --- |\n| Al | Alex |\n| Lex | Alex |\n",
    )
    .unwrap();
    service.use_notes_root(&notes).unwrap();
    let people = service.people().unwrap();
    assert_eq!(people.len(), 2);
    assert_eq!(
        people.iter().find(|p| p.name == "Alex").unwrap().aliases,
        ["Al", "Lex"]
    );
    fs::write(person, "# Alexis\n**Role:** Lead").unwrap();
    fs::remove_file(other).unwrap();
    fs::remove_file(glossary).unwrap();
    service.index().unwrap();
    let people = service.people().unwrap();
    assert_eq!(people.len(), 1);
    assert_eq!(people[0].name, "Alexis");
    assert_eq!(people[0].role.as_deref(), Some("Lead"));
    assert!(people[0].aliases.is_empty());
}
