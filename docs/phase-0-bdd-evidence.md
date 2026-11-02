# Phase 0 BDD harness evidence

Verified on 2026-11-02 in `topics-p0-bdd-refactor`, intentionally stacked from
`0c8bb3a` with Orca parent `topics-p0-shell`. Scope is `crates/topics-bdd` and
this report; approved feature files are unchanged.

## Baseline and refactor

Both the original baseline and explicit-step implementation produced:

```text
cargo run -p topics-bdd -- --tags 'not @ui'
3 features
15 scenarios (15 passed)
78 steps (78 passed)
```

There were no skipped or undefined steps. The runner retains `fail_on_skipped()`.
`steps.rs` binds actual phrases to Given, When and Then explicitly; there is no
catch-all registration or string dispatcher. Service calls remain visible in the
step functions. Shared fixtures and polling assertions live in `main.rs`.
Warnings are captured from the actual index action rather than a second index.
Search and document-count assertions observe the service without rebuilding the
index. Watcher checks retain the two-second deadline and 20ms polling interval;
the unqualified search after rename still waits for the active watcher because
an unchanged document count can precede its rename event.

Each scenario owns a `TempDir` containing separate notes and app-data directories.
No live/default application data is opened. `@ui` scenarios remain owned by the
desktop harness and are excluded by the repository's normal non-UI command.

## Snapshot contract

Snapshots record relative paths (including the root), entry type and modified
time for every directory, regular file, symlink and other entry. Regular files
also record bytes; symlinks record link targets. `symlink_metadata` and recursion
only into actual directories ensure links are never followed.

```text
cargo test -p topics-bdd
2 passed; 0 failed
```

`extra_empty_directory_changes_snapshot` creates a synthetic extra directory,
checks that the directory entry exists only in the new snapshot, and verifies
the snapshots differ independently of the parent's mtime. On Unix,
`symlinks_are_recorded_without_following_them` changes a file outside the fixture
through a linked directory and verifies the snapshot is unchanged and contains
only root and link entries.

## Undefined steps fail closed

An isolated temporary feature was run through Cucumber's supported `--input`
glob override. The actual approved feature tree was never edited. Reproduce
from the worktree root after building the harness:

```sh
python3 - <<'PY'
import pathlib, subprocess, tempfile
with tempfile.TemporaryDirectory(prefix='topics-bdd-undefined-') as folder:
    root = pathlib.Path(folder).resolve()
    (root / 'undefined.feature').write_text(
        'Feature: Fail closed\n  Scenario: Unknown step\n'
        '    Given this step is deliberately undefined\n')
    result = subprocess.run(
        ['target/debug/topics-bdd', '--input', str(root / '*.feature')],
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    print(result.stdout)
    assert result.returncode == 101
    assert "Step doesn't match any function" in result.stdout
    assert '1 step (1 failed)' in result.stdout
PY
```

Observed: one failed scenario, one failed step, `Step doesn't match any function`,
exit 101. With `fail_on_skipped`, Cucumber reports the unmatched step as failed.
The glob is intentional: a literal absolute file input matched zero features in
this Cucumber/globwalk version, so that is not valid negative-test evidence.

## Static checks and integration boundary

Passed `cargo fmt -p topics-bdd --check` and
`cargo clippy -p topics-bdd --all-targets -- -D warnings`.

The first implementation commit `1f73e46` uses baseline
`notes_root() -> Option<&Path>`. The coordinator then supplied immutable core
checkpoint `3b5c70a`; it was merged locally for verification, retaining the
explicit harness when resolving the old dispatcher accessor conflict. The
follow-up adapts only the root-setting assertion to the dynamic
`topics_store::Result<Option<PathBuf>>` accessor. Against that checkpoint,
15 scenarios/78 steps, both snapshot tests, formatting and strict Clippy pass
again; the scratch undefined step still fails closed with exit 101.

Integration should cherry-pick `1f73e46` plus the accessor/report follow-up,
not the local verification merge. Final combined checks and independent Claude
review belong to integration; the 33-mutant campaign was not rerun, and no
desktop validation was attempted here.
