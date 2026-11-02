use std::{fs, process::Command};
#[test]
fn commands_share_isolated_config_and_report_failures() {
    let temp = tempfile::tempdir().unwrap();
    let app = temp.path().join("app");
    let notes = temp.path().join("notes");
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_topics"))
            .env("TOPICS_APP_DATA", &app)
            .args(args)
            .output()
            .unwrap()
    };
    assert!(!run(&["index"]).status.success());
    assert!(!run(&["use", "missing"]).status.success());
    assert!(run(&["init", notes.to_str().unwrap()]).status.success());
    fs::write(
        notes.join("project.md"),
        "# Synthetic ledger\nLedger migration",
    )
    .unwrap();
    let index = run(&["index"]);
    assert!(index.status.success());
    assert!(String::from_utf8_lossy(&index.stdout).contains("\"document_count\": 1"));
    let search = run(&["search", "ledger"]);
    assert!(search.status.success());
    assert!(String::from_utf8_lossy(&search.stdout).contains("project.md"));
    assert!(run(&["people"]).status.success());
    assert!(!run(&["init", notes.to_str().unwrap()]).status.success());
    assert!(!run(&["search"]).status.success());
    assert!(!run(&["index", "extra"]).status.success());
}
