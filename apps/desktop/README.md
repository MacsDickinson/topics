# Topics desktop workspace

React/TypeScript presentation over the real Rust `WorkspaceService` and installed Codex runtime, packaged with Tauri 2. Notes remain Markdown; every model change stays a proposal until explicit human approval. Search from Phase 0 remains available, including Cmd+K.

The sidebar opens Topics, Search, Inbox, History and Sources. Choose or create an absolute notes folder in the app. Sources can be read locally and labelled unknown, cloud allowed or local-only. Analyse explicit files/folders with optional inclusive dates, review included paths and exclusions, and confirm unknown sources for each run. Local-only sources never become eligible through confirmation. The inbox supports edited candidate subsets, rejection, cited passages and uncertainty. Topic detail shows accepted heads, source evidence, earlier revisions and natural-language correction chat; the human-only statement lookup displays the exact recorded user correction. History exposes undo and dependency conflicts. Missing or incompatible Codex leaves source browsing, manual topic proposals and review useful.

## IPC and runtime boundary

`src/workspace-types.ts` mirrors `topics_core::workspace`; canonical Rust values are imported rather than redeclared. All `workspace_*` calls use `{request: ...}` and reject typed `ApiError`, including malformed request fields decoded into canonical core types inside the command boundary. The legacy Phase 0 `search_notes({query})` remains compatible. Rust blocking service work runs off the WebView thread.

`src-tauri/src/workspace.rs` exposes root, scope, source inventory/read, sensitivity, manual proposal, analysis/chat, proposal decision, topic, persisted statement, history/undo and runtime-status commands. Source inventory/read and statement reads are human-only. The runtime host offers only `scope_sources` and validation-only `propose_changes`; it has no approval or arbitrary filesystem tool. A successful terminal turn alone calls the core staging service, which rechecks scope and dependencies. Only the human decision command applies accepted changes.

The runtime emits `workspace_run_event` with canonical root/run IDs, monotonic sequence and event; the shell forwards it without wrapping again. React discards wrong-root/run, duplicate, malformed, late and closed-run events, including events received before a start response. Root changes close the old event gate, cancel runtime and core capabilities, clear topic/proposal/consent state and reject stale reads. Cancellation errors remain retryable. The Rust lifecycle gate serializes start/cancel/root changes. App-level capabilities are revoked on every terminal path.

The packaged desktop executable doubles as the scoped MCP proxy: exact `--topics-mcp-proxy` dispatch occurs before Tauri, application data or root discovery. Production never enables native test plugins or a fake provider. `TOPICS_CODEX_PATH` can select an absolute installed executable; Finder discovery checks only known executable metadata (`~/.local/bin/codex`, Homebrew paths), then PATH. Codex owns its own sign-in; Topics does not read credentials or implement login.

## Local development

Node 24.22.2 is pinned in `.node-version` and as a local dependency. Use `npm ci`, then project-local Node (npm scripts include `node_modules/.bin` automatically). Do not change the global Node installation.

```sh
npm ci
npm test
npm run build
```

Every app launch/test must use synthetic notes and an absolute isolated `TOPICS_APP_DATA_DIR`; do not launch against personal configured roots or the repository notes symlink. Native runners create and remove their own synthetic fixture trees, use a pre-created synthetic HOME, and use a nonpersistent WebKit store. Production app configuration stays persistent. Release macOS bundles use local ad-hoc signing (no credentials); verify with `codesign --verify --deep --strict ../../target/release/bundle/macos/Topics.app`.

```sh
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 npm run test:native
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 npm run test:native:workspace
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 npm run test:native:setup
```

The first native suite retains Phase 0 real search/independent CLI root changes/watcher coverage. The workspace suite exercises fresh root creation, actual source preview/inspection/labels, unavailable runtime, manual proposal staging/review/approval, topic browsing and undo. Proxy entry and invalid app-data startup checks run alongside them. Embedded WebDriver does not implement the default action for option clicks; `tests/native/interaction.ts` performs DOM selection plus the change event while preserving React and every real IPC/service call.

The setup suite uses a local scripted provider with the actual Rust runtime and WebView. It covers 100 synthetic notes, exact scope consent, bounded progress, group save, correction, context, restart/index rebuild, renewed consent and safe failure/retry. The source-byte guard excludes only the root's app-owned `topics/` directory; other source changes and symlinks still fail the check.

Genuine installed-runtime testing is a separate explicit opt-in, authorized only for the fictional fixture:

```sh
TOPICS_NATIVE_LIVE=1 TOPICS_CODEX_PATH=/absolute/installed/codex \
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
npm run test:native:live
```

Only an ephemeral wrapper delegates the unmodified installed runtime to its existing HOME/subscription. GUI, WebKit, notes and Topics app data remain synthetic. The harness never reads credentials/config or prints account metadata. It tests real discovery and correction, fresh-process restart and derived-index rebuild with stable accepted IDs and source-byte checks; timeouts are bounded. This opt-in incurs actual provider requests. It is never silently substituted with scripted model output.

The separate genuine folder-setup journey requires the pinned Codex 0.166.0 and another explicit opt-in:

```sh
TOPICS_NATIVE_LIVE_SETUP=1 TOPICS_CODEX_PATH=/absolute/installed/codex \
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
npm run test:native:setup:live
```

It uses fictional notes, exact consent, real model output, explicit baseline/correction approval and exact corrected-revision lineage through restart. Failures preserve only allowlisted local diagnostic categories and restricted synthetic state. Genuine setup acceptance is currently failing; this command is a diagnostic acceptance test, not evidence that the milestone has passed. See [current status](../../docs/folder-setup-progress.md).

Browser transport fixtures are confined to tests and do not establish native/runtime readiness. RED, GREEN, selected screenshots and remaining limits are recorded in `evidence/workspace-progress.md` and adjacent logs. Root workspace/lockfile integration and cross-provider review remain coordinator-owned.
