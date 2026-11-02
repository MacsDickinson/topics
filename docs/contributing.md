# Contributing to Phase 0

## Crates and dependency direction

- `topics-core`: serializable `Document`, `Person`, `SearchResult`, `IndexReport` values. Depends only on Serde; it has no filesystem, SQL, UI or network behavior.
- `topics-store`: `TopicsService`, Markdown parsers, isolated JSON configuration, SQLite/FTS5 index and owned polling watcher. Depends on core. It owns all note access and application-data persistence.
- `topics-cli`: argument validation and JSON presentation over the same service. Depends on store and uses its shared `application_data_dir()` resolver.
- `topics-bdd`: Cucumber service-level specifications with per-scenario temporary app-data and notes directories. Depends on store. The approved `.feature` files remain the acceptance contract.
- `apps/desktop`: separate shell workstream, also depends on store. Add its Rust package (`apps/desktop/src-tauri`) to the root workspace at integration; it must not duplicate persistence or parsing logic in IPC commands.

Dependencies point inward from CLI/desktop/BDD to store to core. Neither store nor core knows about Tauri, React or CLI output. LLM/MCP crates are later phases.

## Search from UI to Markdown

The desktop creates `TopicsService::open(app_data_dir)` using `topics_store::application_data_dir()?`, wraps it in managed state, and keeps a `WatchHandle` alive in that state. `open` reads configuration and scans the selected root before returning; no root is selected on a fresh installation.

A typed IPC command accepts a query and calls `service.search(&query)`. SQLite FTS5 returns `Vec<SearchResult>` with title, root-relative path and snippet; Serde carries these values to the frontend. Paths do not contain an assumed home directory. The source Markdown is read during indexing and is never modified. The frontend renders the response and links a result using its configured root. Query terms are bound as parameters and quoted as literal FTS terms; raw SQL and arbitrary FTS operators never come from UI text.

Snippets include `<mark>` match spans and may also contain untrusted Markdown text. Render as text or use a narrowly sanitizing renderer; do not insert raw note HTML. `StoreError` implements serialization as `{code,message}` for IPC, and implements standard Rust error display for CLI use.

Public service methods:

```rust,ignore
let app_data_dir = topics_store::application_data_dir()?;
let mut service = topics_store::TopicsService::open(app_data_dir)?;
let report = service.use_notes_root(synthetic_notes_dir)?; // immediately builds index
let watcher = service.watch()?; // keep alive in application state
let hits = service.search("ledger")?;
let people = service.people()?;
let documents = service.documents()?;
```

`init_notes_root` is the only notes-writing operation in Phase 0, and accepts only empty/new directories. `index` returns document count and warnings; invalid UTF-8 remains a plain searchable document with a warning. `notes_root` exposes the canonical selected directory. Dropping the watcher requests shutdown and joins its thread; `last_error` exposes the most recent scan error. An existing watcher follows root changes made through its owning service.

## Consequential Rust choices

Owned serializable values keep storage and IPC lifetimes simple. Optional profile fields and dates express missing data instead of invented defaults. Modification nanoseconds are decimal strings so they survive JSON/JavaScript without integer precision loss; SHA-256 hashes detect same-mtime content edits. The index stores only derived data; future accepted knowledge must remain in approved durable files outside SQLite.

Bundled SQLite enables FTS5 consistently without requiring a system SQLite build. Each operation opens a short-lived connection with a busy timeout. A transaction publishes a whole consistent scan, so concurrent searches never see half a rename or a partially switched root. The in-process writer mutex serializes manual and watcher indexing. The watcher owns its worker thread and stop signal through RAII rather than detaching a background task that outlives the app.

The watcher polls and hashes Markdown every 200 ms, reusing the same scan/index service as startup and rebuild. This favors predictable behavior across editors' atomic saves and offline edits in Phase 0; it is not optimized for very large corpora. The two-second acceptance target is verified on synthetic fixtures, not a claim about arbitrary folder sizes. No filesystem notifications are needed. Scanning does not follow symlinks, preventing loops and indexing files outside the root; users may choose a root symlink, which is canonicalized once.

Application data and notes must be disjoint trees in both directions. Initialization checks separation before creation and rejects parent traversal. Configuration uses temp-file plus rename inside app data. The notes index performs no writes to source files, creates no notes Git repository and makes no network requests. A concurrent editor can change a file during a scan; the next poll/startup scan converges, and transient watcher errors remain inspectable. Avoid concurrent processes changing configuration: the service coordinates its own threads, not multiple app instances.

## Validation

Run `scripts/check.sh --core-only` for formatting, Clippy with warnings denied, workspace unit/integration tests and all 15 non-UI Cucumber scenarios. Cucumber uses `fail_on_skipped`. The installed runner is 0.23: its Gherkin parser correctly handles escaped table pipes in the approved glossary fixture; 0.21 silently truncated that cell.

The full `scripts/check.sh` additionally requires the shell-owned `scripts/check-desktop.sh`. UI-tagged scenarios are exercised separately through the shell tests. Keep the full shell and real-app evidence with that workstream; core checks alone do not establish desktop readiness.

Boundary tests cover source bytes/mtime invariance, init refusal, application-data overlap, traversal, symlink loops/escape, invalid UTF-8, malformed configuration, exact-mtime offline edits, hostile search syntax, watcher root switching, CLI configuration persistence and failure exit codes. Tests use only temporary synthetic data. Do not copy production notes into tests or run tests against a configured personal root. Follow the approved red/green BDD process before expanding product behavior.

## Shared application-data contract regression

`pub fn application_data_dir() -> Result<PathBuf>` is exported by `topics-store`.
It resolves `TOPICS_APP_DATA_DIR` before OS discovery, does not create directories,
and rejects empty/relative overrides instead of falling back. The macOS default
matches Tauri's identifier: `~/Library/Application Support/com.macsdickinson.topics`.
Use an absolute temporary override for native smoke tests as well as CLI tests;
never launch a native test with a missing override against the real user home.
The obsolete `TOPICS_APP_DATA` variable is not supported.

Before the fix, `cargo test -p topics-cli --test commands` failed at
`override config missing`: the CLI ignored the new override. The regression used
an isolated subprocess environment with synthetic home/data paths and a temporary
obsolete-variable trap, so even the failing implementation could not open personal
configuration. CLI checks now assert configuration/index creation under the new
override, failure without writes for invalid overrides, and the macOS/Linux default
identifier under a synthetic home. Resolver unit tests use injected defaults,
without mutating process-wide environment or probing personal paths.

Green verification on macOS: `scripts/check.sh --core-only` passed formatting,
Clippy with warnings denied, 13 unit/integration checks and all 15 non-UI
Cucumber scenarios (78 steps). Windows/Linux native execution remains unverified;
desktop native-build and smoke evidence belongs to the shell workstream.
