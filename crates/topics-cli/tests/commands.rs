use std::{fs, process::Command};
#[test]
fn commands_share_isolated_config_and_report_failures() {
    let temp = tempfile::tempdir().unwrap();
    let app = temp.path().join("app");
    let notes = temp.path().join("notes");
    let run = |args: &[&str]| {
        isolated_command(temp.path())
            .env("TOPICS_APP_DATA_DIR", &app)
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
    let notes = temp.path().join("notes");
    fs::create_dir(&notes).unwrap();
    let output = isolated_command(temp.path())
        .args(["use", notes.to_str().unwrap()])
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

#[test]
fn missing_root_can_be_replaced_or_initialised_through_real_cli() {
    for initialise in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let app = temp.path().join("app");
        let notes = temp.path().join("old");
        let replacement = temp.path().join("replacement");
        fs::create_dir(&notes).unwrap();
        fs::write(notes.join("old.md"), "OldRoot").unwrap();
        let run = |args: &[&str]| {
            isolated_command(temp.path())
                .env("TOPICS_APP_DATA_DIR", &app)
                .args(args)
                .output()
                .unwrap()
        };
        assert!(run(&["use", notes.to_str().unwrap()]).status.success());
        fs::rename(notes, temp.path().join("gone")).unwrap();
        assert!(!run(&["search", "OldRoot"]).status.success());
        if !initialise {
            fs::create_dir(&replacement).unwrap();
            fs::write(replacement.join("new.md"), "NewRoot").unwrap();
        }
        let output = run(&[
            if initialise { "init" } else { "use" },
            replacement.to_str().unwrap(),
        ]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(run(&["search", "OldRoot"]).status.success());
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&run(&["search", "OldRoot"]).stdout)
                .unwrap(),
            serde_json::json!([])
        );
    }
}

#[test]
fn obsolete_override_is_an_explicit_error_in_a_synthetic_home() {
    let temp = tempfile::tempdir().unwrap();
    let output = isolated_command(temp.path())
        .env("TOPICS_APP_DATA", temp.path().join("obsolete"))
        .env("TOPICS_APP_DATA_DIR", temp.path().join("safe-override"))
        .arg("people")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("TOPICS_APP_DATA is obsolete"));
    assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
}

#[test]
fn unconfigured_reads_are_actionable_cli_errors() {
    let temp = tempfile::tempdir().unwrap();
    for args in [&["search", "ledger"][..], &["people"][..]] {
        let output = isolated_command(temp.path())
            .env("TOPICS_APP_DATA_DIR", temp.path().join("app"))
            .args(args)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("Choose a notes root first"));
    }
}

/// Own every spawned process, including assertion/timeout paths. A failed probe
/// must never leave a CLI blocked on the external lock after this test ends.
struct LockProbeChild(Option<std::process::Child>);
impl LockProbeChild {
    fn remains_blocked(&mut self) {
        use std::time::{Duration, Instant};
        let deadline = Instant::now() + Duration::from_millis(750);
        loop {
            assert!(
                self.0.as_mut().unwrap().try_wait().unwrap().is_none(),
                "CLI completed while another process still held store.lock"
            );
            if Instant::now() >= deadline {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    fn completes_after_release(&mut self) -> std::process::Output {
        use std::time::{Duration, Instant};
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if self.0.as_mut().unwrap().try_wait().unwrap().is_some() {
                return self.0.take().unwrap().wait_with_output().unwrap();
            }
            assert!(
                Instant::now() < deadline,
                "CLI did not complete after lock release"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}
impl Drop for LockProbeChild {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[test]
fn cli_use_and_search_wait_for_external_store_lock_then_complete() {
    use std::process::Stdio;
    let temp = tempfile::tempdir().unwrap();
    let app = temp.path().join("app");
    let old = temp.path().join("old");
    let new = temp.path().join("new");
    fs::create_dir(&old).unwrap();
    fs::create_dir(&new).unwrap();
    fs::write(old.join("old.md"), "# Old ledger").unwrap();
    fs::write(new.join("new.md"), "# New ledger").unwrap();
    let mut setup = isolated_command(temp.path());
    let output = setup
        .env("TOPICS_APP_DATA_DIR", &app)
        .args(["use", old.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    for args in [
        &["use", new.to_str().unwrap()][..],
        &["search", "ledger"][..],
    ] {
        // This parent's descriptor is independent of the real CLI process's
        // descriptor. Successful blocking acquisition is the readiness barrier:
        // the lock is held before Command::spawn launches the CLI (no race with
        // a timed lock-holder startup). Never replace/remove the lock inode.
        let external = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(app.join("store.lock"))
            .unwrap();
        external.lock().unwrap();
        let child = isolated_command(temp.path())
            .env("TOPICS_APP_DATA_DIR", &app)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut probe = LockProbeChild(Some(child));
        probe.remains_blocked();
        drop(external); // Explicit release signal, before waiting for completion.
        let result = probe.completes_after_release();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        if args[0] == "use" {
            assert_eq!(json["document_count"], 1);
            let config: serde_json::Value =
                serde_json::from_slice(&fs::read(app.join("config.json")).unwrap()).unwrap();
            assert_eq!(
                config["notes_root"],
                fs::canonicalize(&new).unwrap().to_str().unwrap()
            );
        } else {
            assert_eq!(json.as_array().unwrap().len(), 1);
            assert_eq!(json[0]["path"], "new.md");
            assert_eq!(json[0]["title"], "New ledger");
        }
    }
}
