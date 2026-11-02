# Phase 0 core execution evidence

## Red baseline, before service implementation

On 2026-11-02, Rust 1.98.1, ran:

```sh
cargo run -p topics-bdd -- --tags 'not @ui'
```

Actual exit: 101. Cucumber summary: **3 features, 15 scenarios (15 failed), 15 steps (15 failed)**. Every scenario entered its first registered step and panicked with `Phase 0 service not implemented`. The runner uses `fail_on_skipped`; UI scenarios are tagged and excluded for the shell worker. This result was recorded before implementing persistence or core service behavior. The temporary raw log is not committed.

The approved feature files were not changed. All fixtures are synthetic temporary directories; no live notes were accessed.

## Green result

Final command on 2026-11-02:

```sh
scripts/check.sh --core-only
```

Actual exit: **0**. This ran `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `cargo run -p topics-bdd -- --tags 'not @ui'`.

- Cucumber 0.23.0: **3 features, 15 scenarios (15 passed), 78 steps (78 passed)**, with `fail_on_skipped` enabled and no skipped scenarios.
- Unit/integration checks: **8 passed** (2 parser tests, 5 store boundary tests, 1 CLI process integration test); no failures or ignored tests.
- `git diff --check`: passed.

The runner was upgraded from 0.21.1 after finding its bundled Gherkin 0.14 parser silently truncated the approved escaped-pipe glossary table. Gherkin 0.16 in Cucumber 0.23 reads that unchanged fixture correctly. Early service-backed runs exposed this and canonical-path/timing issues in the harness before the final green run.

Validated behavior includes canonical root persistence, non-empty initialization refusal and source invariance, root switching, document kinds and dates, meeting filename metadata, profile fields and glossary aliases, FTS snippets, invalid UTF-8 warning/count, missing-database rebuild, same-mtime content hashing on startup, and add/edit/rename/delete watcher convergence within two seconds on the synthetic fixtures. Boundary checks cover application-data overlap and parent traversal before creation, symlinks escaping the root, malformed configuration, literal search syntax, watcher lifetime/root switching, and CLI failure exit codes.

## Integration and limits

The service/API contract and `scripts/check-desktop.sh` contract were sent to the coordinator. The default `scripts/check.sh` delegates to that shell-owned script after core checks; this core checkout has no desktop files and only `--core-only` was run. Desktop workspace membership, UI tests, real macOS smoke evidence and independent Claude review remain with the coordinator and shell workstream. This report does not claim those passed.

The watcher hashes the entire synthetic notes tree every 200 ms and avoids SQLite rewrites when document metadata/hashes are unchanged. It favors simple, portable convergence over large-corpus efficiency; the two-second result is not a benchmark for arbitrary notes sizes. The index serializes in-process writers, not competing application instances. No live notes were read or written, no approved features or research reports were edited, and no graph/task/claim/chat/MCP functionality was introduced.
