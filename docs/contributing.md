# Contributing to Phase 0

## Crates and dependency direction

- `topics-core`: serializable `Document`, `Person`, `SearchResult` and `IndexReport` values. It depends only on Serde, with no filesystem, SQL, UI or network behavior.
- `topics-store`: `TopicsService`, source traversal, Markdown parsing, JSON configuration, the disposable SQLite/FTS5 index and an owned polling watcher. It depends on core and owns all note access and app-data persistence.
- `topics-cli`: argument handling and JSON presentation over the same service and shared `application_data_dir()` resolver.
- `topics-bdd`: explicit Cucumber steps using the service and complete synthetic source-tree snapshots. Approved `.feature` files are the acceptance contract.
- `apps/desktop`: the Tauri 2 workspace member in `apps/desktop/src-tauri`, with React/TypeScript presentation. Its real IPC calls store services; it does not duplicate persistence or parsing.

Dependencies point inward from CLI/desktop/BDD to store to core. Neither store nor core knows about Tauri or React. Later-phase tasks, claims, chat and MCP are outside Phase 0.

## Search from UI to Markdown

The desktop resolves app data with `topics_store::application_data_dir()`. A resolver error remains managed application state, so an attempted search displays the error in a usable window rather than aborting or falling back. On the first valid search it creates a service and retains a `WatchHandle`. Blocking store work runs through Tauri's `spawn_blocking`, off the WebView event loop.

`TopicsService::open(app_data_dir)` prepares/canonicalizes app data **without scanning notes**. The first data read performs a full content-hash scan of the currently selected root; an explicit `index()` also always hashes content. Consequently, an unavailable old root does not prevent `use_notes_root` or `init_notes_root` from configuring a replacement. Search/people/documents report actionable errors for no root, an unavailable root or invalid config; these conditions are not empty search results.

The IPC command calls `service.search(&query)` and returns title, root-relative path and snippet. Serde carries owned values to React. The desktop adapter turns store failures into message rejections displayed by the existing error UI. The frontend debounces requests, ignores obsolete responses, and displays No matches only after a successful empty response.

Query terms are bound parameters and literal FTS terms, with punctuation-only terms omitted and NUL treated as a separator. FTS chooses the best excerpt column, including title-only matches. Snippets contain exact `<mark>` highlight tokens and untrusted source text; the React renderer emits text nodes for note content and only creates highlight elements for those tokens. It does not insert raw note HTML. The focused browser test verifies hostile title/snippet markup creates no injected image/script elements and executes nothing while highlighting still works. Literal-mark cosmetic behavior is deferred.

## Service usage and root changes

```rust,ignore
let app_data_dir = topics_store::application_data_dir()?;
let mut service = topics_store::TopicsService::open(app_data_dir)?;
let report = service.use_notes_root(synthetic_notes_dir)?; // builds/publishes immediately
let selected: Option<std::path::PathBuf> = service.notes_root()?;
let watcher = service.watch()?; // retain in application state
let hits = service.search("ledger")?;
let people = service.people()?;
let documents = service.documents()?;
```

The accessor is `notes_root(&self) -> Result<Option<PathBuf>>`: it reads current config under the shared lock and returns an owned canonical path, not a cached reference. A retained service and watcher follow changes made by another service or real CLI process using the same app-data directory.

`init_notes_root` is the only notes-writing operation in Phase 0 and accepts only empty/new directories. Other operations never rewrite sources. `index()` returns counts and warnings. Invalid UTF-8 and unreadable individual Markdown files remain plain document records with warnings; unreadable content never retains an old body/profile. An unreadable root is an error and does not wipe the previous index. Metadata comes from opened file handles so concurrent unlink does not abort a whole scan.

## Synchronization, recovery and Rust choices

Owned serializable values keep IPC/storage lifetimes simple. Optional profile fields and dates express missing data. Modification nanoseconds are decimal strings to avoid JSON/JavaScript integer precision loss. SHA-256 full scans catch offline edits even when mtimes are restored. Host-aware relative path components preserve legitimate Unix backslashes rather than turning them into directory separators.

One stable `store.lock` in canonical app data coordinates cooperating threads, instances and processes through an OS-backed exclusive lock. The guard covers current config discovery, full scans, publication and reads. Lock order is desktop session mutex (when applicable), store lock, then SQLite; watchers never take the desktop mutex. RAII closes the independent lock descriptor on success/error, and watcher sleep is outside the guard. Never replace or remove the lock file while an app is running, because that would split the lock domain.

Bundled SQLite provides FTS5 consistently. Each operation uses short-lived connections with a busy timeout. Indexed-root identity is committed in the same transaction as document, people and FTS rows. Config replacement occurs while that transaction is uncommitted, under the same OS guard: config-publication failure rolls back the SQL changes. A mismatch after interruption or a subsequent SQL commit failure is repaired before returning rows. Legacy indexes without identity rebuild on use. Recognized SQLite corruption and invalid serialized derived rows reset/rebuild the disposable index under the lock; config/notes are not deleted. Unrelated filesystem/permission errors remain errors. Arbitrary wrong-shape schemas require future schema-version handling; do not claim every SQLite failure is corruption.

The watcher polls metadata/structure about every 200 ms, including Unix ctime/inode/permissions, and reads/hashes note bodies when that snapshot changes. It does not hash the entire corpus at idle. Startup/explicit rebuild still use full content hashes. Dropping `WatchHandle` signals shutdown and joins the thread; `last_error()` reports the latest polling error. Existing watchers follow cross-process root changes. Reads can repair startup/root/index mismatches, but ordinary same-root source edits converge through the watcher; the BDD watcher tests do not manually index while polling.

Full scans and the shared lock can delay searches or root switches, and the current lock wait has no timeout. The two-second service-level target is tested on synthetic fixtures, not promised for arbitrary corpus sizes or stalled volumes. Large-corpus incremental indexing and bounded lock waits remain follow-ups.

## Configuration and source safety

`application_data_dir() -> Result<PathBuf>` resolves `TOPICS_APP_DATA_DIR` before OS discovery and creates nothing. Overrides must be nonempty absolute paths. The macOS default matches Tauri's identifier: `~/Library/Application Support/com.macsdickinson.topics`. An exported obsolete `TOPICS_APP_DATA` is an explicit error. On Unix, explicitly empty HOME without an override fails before OS fallback; unset HOME is normal OS behavior, not a test-isolation technique.

App data and notes must be disjoint trees. Initialization validates separation before creating the default layout and rejects parent traversal. Scanning does not follow source symlinks; a chosen root symlink is canonicalized. Store writes are limited to config, index/journals and lock in app data. It creates no notes Git repository and makes no network requests. Concurrent source edits are observed on a later poll/full scan; warnings and watcher errors remain inspectable.

## Validation and test safety

`bash scripts/check.sh --core-only` runs formatting, locked warning-denied clippy/tests excluding the desktop package, and all 15 non-UI Cucumber scenarios. This is also the Ubuntu core CI scope, requiring no desktop GTK/WebKit build. `bash scripts/check.sh` uses the complete workspace and additionally runs Playwright and the real macOS embedded-provider WebdriverIO suite. macOS CI invokes that full command with Rust 1.98.1, rustfmt/clippy and the pinned project-local Node runtime. Core-only results alone do not establish desktop readiness.

Cucumber uses explicit Given/When/Then definitions and `fail_on_skipped`; unknown steps fail instead of being absorbed by a catch-all. Source-tree snapshots cover files, directories and symlinks with contents and metadata. Service/CLI tests cover recovery, root switching across processes, publication consistency, source permissions/unlink, path identity, query edge cases, watcher lifecycle/errors, profiles/aliases and config resolver safety. A bounded real-CLI test holds an external store-lock descriptor, confirms use/search cannot complete before release, then verifies successful new-root results. One isolated no-op-lock mutant confirms that test detects omitted locking; this is not a general mutation campaign.

All fixtures are temporary and synthetic. CLI subprocesses use an absolute synthetic app-data override or a fully synthetic valid HOME; native smoke always uses an absolute temporary override. Never copy real notes into fixtures, follow the repository's notes symlink, or launch tests against a personal configured root. Never probe empty/missing HOME through real OS discovery without an absolute safe override: use injected resolver inputs and a fallback closure that fails if called. Every spawned child/helper must have bounded waits and cleanup, including assertion failures. Do not mutate global environment/runtime defaults, submit note content to cloud services or infer product approval from passing tests. Follow the approved BDD red/green process when changing product behavior.

Current evidence and exact commands are linked from [review fixes](phase-0-review-fixes.md), [BDD evidence](phase-0-bdd-evidence.md) and [final checks](phase-0-final-checks.md). Hosted CI, live read-only verification and approval records remain manager-owned gates; these local results do not invent sign-off.

## Topic workspace service and runtime seam

`topics-core::workspace::*` (also re-exported at the crate root) is the canonical serde API. It owns the UUID/date/candidate shape checks; store owns reference resolution, selected-byte checks, Markdown records and journalling. Mutation objects reject unknown properties, including tagged unit variants; nullable properties must be present as JSON `null`. Field names remain snake_case. UI commands receive one object named `request`.

`WorkspaceService::open(absolute_app_data)` opens the same configuration and stable `store.lock` as Phase 0. Every operation discovers the current configured root under that lock. Explicit root selection rotates the root capability ID, including reselecting the same path; accepted Markdown knowledge remains, while old scopes and pending approvals are invalidated. Root partitions in app data hold immutable preview snapshots, run capabilities, pending proposals, snapshots and operation journals. No provider or runtime process is invoked by this crate.

The runtime host first calls `scope_preview(ScopePreviewRequest)`, obtains explicit human consent, then `analysis_start(AnalysisStartRequest)`. Its tools may use only `scope_read(ScopeReadRequest)` and `source_context(SourceContextRequest)`. Both require the service-generated root/scope/run IDs on every lookup. `validate_proposal(StageProposalRequest)` checks a buffered `propose_changes` tool call without publishing or consuming a capability; it shares the preparation and validation used by staging. After a successful terminal turn only, `stage_proposal` consumes that capability and creates a pending proposal. Errors/cancellation must call `run_cancel`; a consumed/cancelled capability cannot read or publish late results. The runtime never receives `proposal_decide`, root changes, human inventory, undo or write capabilities.

Human browsing uses `sources_list(RootRequest)` and `source_get(SourceGetRequest)`, including local-only sources without cloud consent. `statement_get(StatementGetRequest)` returns persisted correction/edit text and timestamp for the evidence UI. These are not model tools. Neither legacy Phase 0 search nor human inventory may be registered as a model retrieval tool. A fresh root needs no provider: `manual_propose(ManualProposeRequest)` creates a pending proposal and uses exactly the same human approval service.

The CLI mirrors those requests: `topics workspace <root-get|root-set|sources|source|statement|source-repair|preview|manual|proposals|decide|topics|topic|history|undo|sensitivity> 'REQUEST_JSON'`. It is a recovery/testing adapter, not another apply implementation. JSON errors include the canonical error code in their displayed message. CLI tests pass absolute synthetic `TOPICS_APP_DATA_DIR` values.

## Workspace records, evidence and write recovery

App-owned records use `topics/{topics,claims,connections,rules,sources,statements,unresolved,operations}/<uuid>.md`. Claim filenames use revision IDs. Each begins with YAML frontmatter containing `schema: topics/v1`, `record_kind`, `id`, `operation_id`, and `data: <typed JSON object>`; JSON is valid YAML flow syntax. The Markdown body gives a readable kind heading and assertion/name/text. This v1 parser deliberately accepts the documented five-property frontmatter and typed JSON data, rather than arbitrary hand-written YAML. Malformed records, payload/filename identity mismatches and duplicate IDs are excluded with warnings. External edits invalidate the recorded hash and remain visible as conflicts/warnings, rather than becoming an implicit approval.

Operation records are durable commit markers. Their data records ordered target paths, expected hashes/absence, new hashes, dependency IDs, tombstones, undo lineage, and original/applied proposal candidates. Accepted knowledge is reconstructed directly from committed Markdown; deleting SQLite/FTS and restarting loses no topics, claim history, connections, rules, statements or decisions. Pending proposals and run state are disposable. A canonical fingerprint normalizes provisional UUIDs/timestamps while retaining accepted endpoint identity; repeat analysis shows the durable decision. Explicit manual creation of two same-name people retains two identities.

Before the first target replacement, a flushed app-data journal stores every target's previous bytes or explicit absence (`old_bytes: null`) and staged bytes. File replacements use unique same-directory temporary files, flush, rename and parent-directory flush. All expected target hashes are checked, and the operation marker is written last; projections ignore uncommitted records. Restart resumes only when current target hashes match either expected or already-written new hashes. Otherwise, it preserves old, staged and current bytes and records a conflict, without publishing incomplete knowledge. Proposal conflicts additionally preserve the original reviewed record bytes and proposed Markdown. This is cooperative process coordination and recoverable per-file replacement, not cross-file atomic rename or protection against every race with an uncooperative editor.

Undo creates another committed operation. Dependency checks identify later live operations; creations get durable tombstones, while updates restore previous data under a new operation ID. Undo retains original records and operation history. An update can be undone before undoing its original creation. Compensating undo operations cannot themselves be undone; restore intended knowledge through a reviewed proposal instead. Snapshot recovery depends on retaining the root-partitioned app-data snapshots; the disposable SQLite database is independent.

Citations validate source ID, root-relative path, full-document SHA-256, UTF-8 byte boundaries, passage hash and exact excerpt. A changed or missing source is stale; it is never silently reanchored. Explicit human `source_identity_repair(SourceRepairRequest)` journals a mapping repair for rename/edit with expected source hash, restrictive sensitivity and dependency checks. It never changes old citation paths/hashes or source files; replacing stale evidence requires a separate reviewed revision. Date filtering uses validated filename dates first, then exact ISO `date:` frontmatter; absent/invalid dates are visibly excluded and mtime is never used as document date.

A correction cites its persisted user statement, supersedes the old revision, and records a deterministic topic/predicate/source-scoped suppression rule. Old transcript assertions remain in history; a rule suppresses false candidates but never accepts a replacement. Editing a pending assertion during human approval adds a persisted user-edit statement to its evidence in the same operation, retaining the original transcript only as context. Source/topic sensitivity is durable; local-only evidence taints derived topics and denies model context, including raw scoped source access after a related topic becomes local-only. Unknown source confirmation is per run and never lifts local-only labels.

The synthetic-only `topics-store/test-support` feature exposes a deterministic interruption seam used by BDD; normal application callers do not need it. Core validation runs are documented in `features/topic-workspace/evidence-core-green.md`; desktop and runtime-only scenarios require their owning streams' evidence.
