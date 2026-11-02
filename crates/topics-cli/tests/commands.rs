use std::{fs, process::Command};
#[test]
fn commands_share_isolated_config_and_report_failures() {
    let temp = tempfile::tempdir().unwrap();
    let app = temp.path().join("app");
    let notes = temp.path().join("notes");
    let run = |args: &[&str]| {
        isolated_command(temp.path())
            .env("TOPICS_APP_DATA_DIR", &app)
            // Keep the pre-fix regression safe: the obsolete resolver writes only here.
            .env("TOPICS_APP_DATA", temp.path().join("obsolete"))
            .args(args)
            .output()
            .unwrap()
    };
    assert!(!run(&["index"]).status.success());
    assert!(!run(&["use", "missing"]).status.success());
    assert!(run(&["init", notes.to_str().unwrap()]).status.success());
    assert!(app.join("config.json").is_file(), "override config missing");
    assert!(
        app.join("index.sqlite3").is_file(),
        "override index missing"
    );
    assert!(!temp.path().join("obsolete").exists());
    assert!(!temp.path().join("Library").exists());
    assert!(!temp.path().join("xdg").exists());
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

fn isolated_command(home: &std::path::Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_topics"));
    command
        .env_clear()
        .current_dir(home)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("APPDATA", home.join("roaming"))
        .env("LOCALAPPDATA", home.join("local"))
        .env("XDG_DATA_HOME", home.join("xdg"));
    command
}

#[test]
fn invalid_overrides_fail_without_creating_any_app_data() {
    for value in ["", "relative-app-data"] {
        let temp = tempfile::tempdir().unwrap();
        let output = isolated_command(temp.path())
            .env("TOPICS_APP_DATA_DIR", value)
            .arg("people")
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("TOPICS_APP_DATA_DIR"));
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[test]
fn os_default_matches_application_identifier_in_synthetic_home() {
    let temp = tempfile::tempdir().unwrap();
    let output = isolated_command(temp.path())
        .arg("people")
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    #[cfg(target_os = "macos")]
    let base = temp.path().join("Library/Application Support");
    #[cfg(target_os = "linux")]
    let base = temp.path().join("xdg");
    assert!(
        base.join("com.macsdickinson.topics/index.sqlite3")
            .is_file()
    );
    assert!(!base.join("Topics").exists());
}

#[test]
fn cli_switch_is_coherent_with_an_independent_process_retaining_a_watcher() {
    use std::{thread, time::Duration};
    use topics_store::TopicsService;
    let temp = tempfile::tempdir().unwrap();
    let app = temp.path().join("app");
    let old = temp.path().join("old");
    let new = temp.path().join("new");
    fs::create_dir(&old).unwrap();
    fs::create_dir(&new).unwrap();
    fs::write(old.join("old.md"), "OldRoot ledger").unwrap();
    fs::write(new.join("new.md"), "NewRoot ledger").unwrap();
    let mut retained = TopicsService::open(&app).unwrap();
    retained.use_notes_root(&old).unwrap();
    let watcher = retained.watch().unwrap();
    let run = |args: &[&str]| {
        isolated_command(temp.path())
            .env("TOPICS_APP_DATA_DIR", &app)
            .args(args)
            .output()
            .unwrap()
    };
    assert!(run(&["use", new.to_str().unwrap()]).status.success());
    for _ in 0..5 {
        thread::sleep(Duration::from_millis(250));
        assert!(retained.search("OldRoot").unwrap().is_empty());
        assert_eq!(retained.search("NewRoot").unwrap().len(), 1);
        let result = run(&["search", "ledger"]);
        assert!(result.status.success());
        let rows: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(rows.as_array().unwrap().len(), 1);
        assert_eq!(rows[0]["path"], "new.md");
    }
    assert!(watcher.last_error().is_none());
}
