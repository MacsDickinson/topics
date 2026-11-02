# Phase 0 desktop evidence

Scope: `apps/desktop`, with no changes to approved features, root Cargo workspace or core crates. All browser fixtures are synthetic; native fixtures use a fresh temporary directory, isolated config/index, and the real store. No repository notes symlink is accessed.

## BDD red then green

`cd apps/desktop && npm test` against a placeholder heading: **2 failed**, missing search box / Cmd+K focus, recorded before UI implementation in [playwright-red.txt](../apps/desktop/evidence/playwright-red.txt). An earlier browser-install infrastructure failure was resolved before recording this behavioral red.

The same command after implementation: **4 passed, zero skipped** in [playwright-green.txt](../apps/desktop/evidence/playwright-green.txt). Both approved scenarios are registered from `features/phase-0/desktop_search.feature`; each step is executed by an explicit mapping, undefined/unsupported lines throw, and the runner requires both scenarios. Additional tests cover late results, clearing input, literal snippet text, errors and recovery.

`npm run build`: **passed** (TypeScript strict check and Vite production build). Local runtime is **Node v24.22.2**, pinned as an npm dependency; no global runtime was changed. Bootstrap `npm ci` with existing Node, then npm scripts use the local runtime.

## Real native smoke

Pending core workspace integration and execution; no passing native result is claimed yet. Run `cd apps/desktop && npm run test:native`. The harness configures the synthetic root through `topics use`, builds an actual Tauri debug binary, runs WebdriverIO with `driverProvider: 'embedded'`, verifies title/path/snippet and No matches, saves screenshots, checks the synthetic source hash and removes only its own temporary fixture directory. No mocked IPC is installed by this suite.

Native screenshot destinations: `apps/desktop/evidence/native-ledger.png` and `apps/desktop/evidence/native-no-matches.png` (created only by a successful real app run).

Current official [Tauri WebDriver documentation](https://tauri.app/develop/tests/webdriver/) and [WebdriverIO platform support](https://webdriver.io/docs/desktop-testing/tauri/platform-support) support macOS through the embedded `tauri-plugin-wdio-webdriver` provider without an external driver. [Plugin setup](https://webdriver.io/docs/desktop-testing/tauri/plugin-setup) and [configuration](https://webdriver.io/docs/desktop-testing/tauri/configuration) describe the Rust plugins, capabilities and service setup used here. Research checked 2026-11-02. Rust plugins require the explicit `native-test` feature and debug assertions; frontend plugin import requires `VITE_NATIVE_TEST=1`, and native-test permissions/global API live only in the test config overlay.

## UI to core

React calls `invoke('search_notes', { query })`. Tauri clones managed state and runs the blocking work on `spawn_blocking`; the first search opens the configured `TopicsService`, runs the startup hash scan/index, starts and retains `WatchHandle`, then searches. Later queries use the same real store. Rust returns serializable `{ title, path, snippet }` values or rejects with an error. UI debounces requests, clears previous results immediately, ignores obsolete promises, renders errors separately from empty matches, and treats snippets as plain text.

The notes root comes from the existing core app config. `TOPICS_APP_DATA_DIR` isolates native-test config and index. No production fixtures or fallback results exist. A missing configuration is a visible search error with CLI setup guidance.

## Integration boundary / residual checks

Core owns root Cargo files and must add `apps/desktop/src-tauri` to workspace members. Native integration must confirm shared Application Support location with the CLI and compile the exact store API. Root check can call `cd apps/desktop && npm ci && npm run build && npm test`; a macOS app session additionally runs `npm run test:native` and must treat failure as failure, not skip it.

Initial npm audit reports 30 development-tool dependency advisories (16 moderate, 14 high), primarily in the current WebdriverIO toolchain; no blanket major-version downgrade or force fix was applied. These packages are build/test tooling and are absent from the production frontend dependency graph.
