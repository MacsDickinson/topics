//! Explicit Gherkin bindings: the service call under test is visible in each action.
use crate::{Notes, eventually};
use cucumber::{given, then, when};
use std::{fs, io::Write};
use topics_store::TopicsService;

// Source fixtures and root selection.
#[given("a notes root with these files:")]
fn notes_table(w: &mut Notes, step: &cucumber::gherkin::Step) {
    for row in step
        .table
        .as_ref()
        .expect("notes fixture table")
        .rows
        .iter()
        .skip(1)
    {
        w.write(
            &row[0],
            row[1..].join("|").replace("\\|", "|").replace("\\n", "\n"),
        );
    }
    w.remember_source();
    w.service.use_notes_root(&w.root).unwrap();
}

#[given(regex = r#"^a folder "([^"]+)" containing (\d+) markdown files$"#)]
fn existing_folder(w: &mut Notes, name: String, count: usize) {
    w.root = w.fixture_folder(&name, count);
    w.remember_source();
}

#[given(regex = r#"^the notes root is "([^"]+)" with (\d+) markdown files$"#)]
fn selected_folder(w: &mut Notes, name: String, count: usize) {
    existing_folder(w, name, count);
    w.service.use_notes_root(&w.root).unwrap();
}

#[given("a notes root with 3 markdown files including \"memory/projects/ledger-migration.md\"")]
fn recovery_fixture(w: &mut Notes) {
    selected_folder(w, "notes".into(), 3);
}

#[given(regex = r#"^the folder "([^"]+)" does not exist$"#)]
fn absent_folder(w: &mut Notes, name: String) {
    w.root = w.temp.path().join(name);
    assert!(!w.root.exists());
}

#[when(regex = r#"^I set the notes root to "([^"]+)"$"#)]
fn use_root(w: &mut Notes, name: String) {
    w.root = w.temp.path().join(name);
    w.service.use_notes_root(&w.root).unwrap();
}

#[when(regex = r#"^I initialise a notes root at "([^"]+)"$"#)]
fn initialise_root(w: &mut Notes, name: String) {
    w.error = w
        .service
        .init_notes_root(w.temp.path().join(name))
        .err()
        .map(|error| error.to_string())
        .unwrap_or_default();
}

#[then(regex = r#"^the notes root setting is "([^"]+)"$"#)]
fn root_setting(w: &mut Notes, name: String) {
    assert_eq!(
        w.service.notes_root().unwrap().unwrap(),
        fs::canonicalize(w.temp.path().join(name)).unwrap()
    );
}

#[then(
    regex = r#"^"([^"]+)" contains the folders "memory/people", "memory/projects", "Meetings", "Log" and "topics"$"#
)]
fn initialised_folders(w: &mut Notes, name: String) {
    for folder in [
        "memory/people",
        "memory/projects",
        "Meetings",
        "Log",
        "topics",
    ] {
        assert!(
            w.temp.path().join(&name).join(folder).is_dir(),
            "missing {folder}"
        );
    }
}

#[then("I am told the folder is not empty and to use it instead")]
fn nonempty_error(w: &mut Notes) {
    assert!(
        w.error.contains("not empty") && w.error.contains("use"),
        "{}",
        w.error
    );
}

#[then(regex = r#"^no file in "([^"]+)" has changed$"#)]
fn named_source_unchanged(w: &mut Notes, name: String) {
    w.assert_source_unchanged(&w.temp.path().join(name));
}

#[then("every file in the notes root has the same bytes and modified time as before")]
fn source_unchanged(w: &mut Notes) {
    w.assert_source_unchanged(&w.root);
}

// Indexing and domain data.
#[given("the index is built")]
#[when("the index is built")]
fn build_index(w: &mut Notes) {
    w.warnings = w.service.index().unwrap().warnings;
}

#[then(regex = r#"^document "([^"]+)" has kind "([^"]+)"$"#)]
fn document_kind(w: &mut Notes, path: String, kind: String) {
    let documents = w.service.documents().unwrap();
    let document = documents
        .iter()
        .find(|document| document.path == path)
        .expect("indexed document");
    assert_eq!(document.kind, kind);
}

#[then(regex = r#"^document "([^"]+)" has kind "([^"]+)" dated "([^"]+)"$"#)]
fn dated_document(w: &mut Notes, path: String, kind: String, date: String) {
    document_kind(w, path.clone(), kind);
    let documents = w.service.documents().unwrap();
    let document = documents
        .iter()
        .find(|document| document.path == path)
        .unwrap();
    assert_eq!(document.date.as_deref(), Some(date.as_str()));
}

#[then(regex = r#"^there is a person "([^"]+)" known as "([^"]+)" with pronouns "([^"]+)"$"#)]
fn person_profile(w: &mut Notes, name: String, known_as: String, pronouns: String) {
    let people = w.service.people().unwrap();
    let person = people
        .iter()
        .find(|person| person.name == name)
        .expect("indexed person");
    assert_eq!(person.known_as.as_deref(), Some(known_as.as_str()));
    assert_eq!(person.pronouns.as_deref(), Some(pronouns.as_str()));
}

#[then(regex = r#"^"([^"]+)" has the alias "([^"]+)"$"#)]
fn person_alias(w: &mut Notes, name: String, alias: String) {
    let people = w.service.people().unwrap();
    let person = people
        .iter()
        .find(|person| person.name == name)
        .expect("indexed person");
    assert!(person.aliases.contains(&alias));
}

#[then(regex = r#"^"([^"]+)" has role "([^"]+)" in team "([^"]+)"$"#)]
fn person_role(w: &mut Notes, name: String, role: String, team: String) {
    let people = w.service.people().unwrap();
    let person = people
        .iter()
        .find(|person| person.name == name)
        .expect("indexed person");
    assert_eq!(person.role.as_deref(), Some(role.as_str()));
    assert_eq!(person.team.as_deref(), Some(team.as_str()));
}

#[given(regex = r#"^the file "([^"]+)" contains invalid UTF-8$"#)]
fn invalid_utf8(w: &mut Notes, path: String) {
    w.write(&path, [0xff, 0xfe]);
}

#[then(regex = r#"^there is 1 warning naming "([^"]+)"$"#)]
fn parse_warning(w: &mut Notes, path: String) {
    assert_eq!(w.warnings.len(), 1);
    assert!(w.warnings[0].contains(&path), "{:?}", w.warnings);
}

#[then(regex = r"^the index contains (\d+) documents$")]
fn document_count(w: &mut Notes, count: usize) {
    assert_eq!(w.service.documents().unwrap().len(), count);
}

#[then(regex = r"^within 2 seconds the index contains (\d+) documents$")]
fn eventual_document_count(w: &mut Notes, count: usize) {
    eventually(&format!("index contains {count} documents"), || {
        w.service.documents().unwrap().len() == count
    });
}

// Search observes the index; it does not repair it.
#[when(regex = r#"^I search for "([^"]+)"$"#)]
fn search(w: &mut Notes, query: String) {
    w.results = w.service.search(&query).unwrap();
}

#[then(regex = r#"^the results include "([^"]+)"$"#)]
fn result_path(w: &mut Notes, path: String) {
    assert!(
        w.results.iter().any(|result| result.path == path),
        "missing {path}"
    );
}

#[then(regex = r#"^each result shows a snippet with "([^"]+)" highlighted$"#)]
fn highlighted_snippets(w: &mut Notes, query: String) {
    assert!(!w.results.is_empty());
    for result in &w.results {
        assert!(
            result
                .snippet
                .to_lowercase()
                .contains(&format!("<mark>{}</mark>", query.to_lowercase()))
        );
    }
}

#[then(regex = r#"^no document from "([^"]+)" appears in search results$"#)]
fn previous_root_absent(w: &mut Notes, query: String) {
    assert!(w.service.search(&query).unwrap().is_empty());
}

#[then(regex = r#"^searching for "([^"]+)" returns "([^"]+)"$"#)]
fn search_returns(w: &mut Notes, query: String, path: String) {
    // A rename can leave the count unchanged before its watcher event is processed.
    if w.watcher.is_some() {
        eventual_search_returns(w, query, path);
    } else {
        assert!(w.has_result(&query, &path), "missing {path} for {query}");
    }
}

#[then(regex = r#"^within 2 seconds searching for "([^"]+)" returns "([^"]+)"$"#)]
fn eventual_search_returns(w: &mut Notes, query: String, path: String) {
    eventually(&format!("search {query} returns {path}"), || {
        w.has_result(&query, &path)
    });
}

#[then(regex = r#"^no result points to "([^"]+)"$"#)]
fn stale_result_absent(w: &mut Notes, path: String) {
    assert!(!w.has_result("ledger", &path), "stale result: {path}");
}

// App lifecycle and external filesystem operations.
#[when("the index database is deleted")]
fn delete_database(w: &mut Notes) {
    fs::remove_file(w.temp.path().join("app-data/index.sqlite3")).unwrap();
}

#[when("the app starts")]
fn start_app(w: &mut Notes) {
    w.service = TopicsService::open(w.temp.path().join("app-data")).unwrap();
}

#[given("the app is stopped")]
fn stop_app(w: &mut Notes) {
    w.watcher = None;
}

#[given("the app is running and watching the notes root")]
fn watch_root(w: &mut Notes) {
    w.watcher = Some(w.service.watch().unwrap());
}

#[when(regex = r#"^another program appends "([^"]+)" to "([^"]+)"$"#)]
fn external_append(w: &mut Notes, content: String, path: String) {
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(w.root.join(path))
        .unwrap();
    writeln!(file, "{content}").unwrap();
}

#[when(regex = r#"^another program creates "([^"]+)" containing "([^"]+)"$"#)]
fn external_create(w: &mut Notes, path: String, content: String) {
    w.write(&path, content);
}

#[when(regex = r#"^another program renames "([^"]+)" to "([^"]+)"$"#)]
fn external_rename(w: &mut Notes, old: String, new: String) {
    fs::rename(w.root.join(old), w.root.join(new)).unwrap();
}

#[when(regex = r#"^another program deletes "([^"]+)"$"#)]
fn external_delete(w: &mut Notes, path: String) {
    fs::remove_file(w.root.join(path)).unwrap();
}
