# Phase 0 desktop evidence

Scope: `apps/desktop`, `scripts/check-desktop.sh`, `.github/workflows/desktop.yml`, this report, and the coordinator-authorized root workspace membership/lock integration. Approved feature files are unchanged; core changes are imported from their owning workers as listed below. All browser fixtures are synthetic; native fixtures use a fresh temporary directory, isolated config/index, and the real store. No repository notes symlink is accessed.

## BDD red then green

`cd apps/desktop && npm test` against a placeholder heading: **2 failed**, missing search box / Cmd+K focus, recorded before UI implementation in [playwright-red.txt](../apps/desktop/evidence/playwright-red.txt). An earlier browser-install infrastructure failure was resolved before recording this behavioral red.

The same command after implementation: **4 passed, zero skipped** in [playwright-green.txt](../apps/desktop/evidence/playwright-green.txt). Both approved scenarios are registered from `features/phase-0/desktop_search.feature`; each step is executed by an explicit mapping, undefined/unsupported lines throw, and the runner requires both scenarios. Additional tests cover late results, clearing input, literal snippet text, errors and recovery.

`npm run build`: **passed** (TypeScript strict check and Vite production build). Local runtime is **Node v24.22.2**, pinned as an npm dependency; no global runtime was changed. Bootstrap `npm ci` with existing Node, then npm scripts use the local runtime.

## Real native smoke

**Passed on the actual macOS WKWebView app** through WebdriverIO 1.5.1 embedded provider (WebKit 605.1.15): one native spec, no skips. The complete `bash scripts/check.sh` exited 0; see [full-check.txt](../apps/desktop/evidence/full-check.txt). Run `cd apps/desktop && npm run test:native`. The harness configures the synthetic root through `topics use`, builds an actual Tauri debug binary, runs WebdriverIO with `driverProvider: 'embedded'`, verifies title/path/snippet and No matches, saves screenshots, checks a snapshot of every synthetic source path and file hash and removes only its own temporary fixture directory. No mocked IPC is installed by this suite.

Actual native screenshots, both visually inspected: [ledger result](../apps/desktop/evidence/native-ledger.png) and [no matches](../apps/desktop/evidence/native-no-matches.png). These are real app webview screenshots with synthetic notes, not browser mock captures. The full synthetic notes directory snapshot was unchanged and its temporary config/index/notes were removed after the run.

Current official [Tauri WebDriver documentation](https://tauri.app/develop/tests/webdriver/) and [WebdriverIO platform support](https://webdriver.io/docs/desktop-testing/tauri/platform-support) support macOS through the embedded `tauri-plugin-wdio-webdriver` provider without an external driver. [Plugin setup](https://webdriver.io/docs/desktop-testing/tauri/plugin-setup) and [configuration](https://webdriver.io/docs/desktop-testing/tauri/configuration) describe the Rust plugins, capabilities and service setup used here. Research checked 2026-11-02. Rust plugins require the explicit `native-test` feature and debug assertions; frontend plugin import requires `VITE_NATIVE_TEST=1`, and native-test permissions/global API live only in the test config overlay.

## UI to core

React calls `invoke('search_notes', { query })`. Tauri clones managed state and runs the blocking work on `spawn_blocking`; the first search opens the configured `TopicsService`, runs the startup hash scan/index, starts and retains `WatchHandle`, then searches. Later queries use the same real store. Rust returns serializable `{ title, path, snippet }` values or rejects with an error. UI debounces requests, clears previous results immediately, ignores obsolete promises, renders errors separately from empty matches, and interprets only exact FTS `<mark>` markers as React highlight elements while keeping all note content as escaped text.

The notes root comes from the existing core app config. `TOPICS_APP_DATA_DIR` isolates native-test config and index. No production fixtures or fallback results exist. A missing configuration is a visible search error with CLI setup guidance.

## Integration and remaining risks

The shared `topics_store::application_data_dir()` helper resolves the CLI and desktop to the same config location (by default `~/Library/Application Support/com.macsdickinson.topics`), honoring `TOPICS_APP_DATA_DIR`. Coordinator authorized the narrow root workspace integration: `apps/desktop/src-tauri` is now a member and Cargo.lock includes the Tauri dependencies. Core baseline `e2faa28` was cherry-picked as `dab0b31`, and shared-config follow-up `335ba24` as `84125eb`; no core service files were independently edited here. The desktop compiles against that real API and uses the same config resolver as the CLI. Root full check delegates to `bash scripts/check-desktop.sh`, which installs pinned dependencies/browser, checks the local Node runtime, builds the frontend, runs all Playwright checks and requires the real native smoke. `.github/workflows/desktop.yml` runs that same script on macOS and retains synthetic evidence; missing native capability fails rather than skips.

Initial npm audit reports 30 development-tool dependency advisories (16 moderate, 14 high), primarily in the current WebdriverIO toolchain; no blanket major-version downgrade or force fix was applied. These packages are build/test tooling and are absent from the production frontend dependency graph.

Browser-only screenshots (mocked IPC, visually inspected): [ledger](../apps/desktop/evidence/browser-ledger.png) and [no matches](../apps/desktop/evidence/browser-no-matches.png). These are distinct from the real native evidence above.

## Final verification

- `bash scripts/check.sh`: **exit 0**, including fmt, clippy with warnings denied, 13 Rust tests, 15 core Gherkin scenarios / 78 steps, 4 Playwright tests (2 approved UI scenarios plus 2 async regressions), and 1 real native WebdriverIO smoke spec; no skipped scenarios/tests.
- `cargo check -p topics-desktop --features native-test`: **exit 0**, recorded in [native-build.txt](../apps/desktop/evidence/native-build.txt).
- Normal `cargo tree -p topics-desktop --edges normal` contains **no WDIO plugin dependencies**. Rust registration additionally requires debug assertions. Test permissions and global Tauri API are confined to the test config overlay.
- After native smoke, `npm run build` restored the ordinary frontend and passed; [production-build.txt](../apps/desktop/evidence/production-build.txt) shows 35 modules. Built assets contain none of `wdioTauri`, `__wdio` or `WDIO Plugin`; no production IPC mock exists.
- No live notes, malformed repository notes symlink, cloud submission of note content, push, or merge to main occurred.

Local shell commits before final evidence: `105c527` (UI/BDD), `c5ee637` (required local/CI smoke), `33b6480` (workspace/icon integration). Imported core commits are listed above so the coordinator can integrate originals or this branch without mistaking them for independently authored core work. Independent Claude review remains the coordinator's follow-up; macOS native CI is configured but has not been run on GitHub from this local-only task. Development-tool advisories remain as recorded above.
