# Topics desktop

Bare Phase 0 search shell. React displays the response from `search_notes({ query: string })`, a Tauri command returning `Array<{ title: string; path: string; snippet: string }>`. Errors reject the invocation; they are displayed separately from empty results. Rust opens `topics_store::TopicsService` through the shared `application_data_dir()` config resolver, indexes on the first search, retains its watcher and executes searches off the UI thread. The frontend never reads note files.

## Local runtime

Node 24.22.2 LTS is pinned as a project dependency and in `.node-version`. The existing system Node may bootstrap `npm ci`; subsequent npm scripts put `node_modules/.bin/node` first in PATH. No global runtime changes are required.

```sh
cd apps/desktop
npm ci
./node_modules/.bin/node --version
npm exec playwright install chromium
npm test
npm run build
npm run tauri dev
```

Playwright substitutes only Tauri's transport in the browser harness. Native WebdriverIO tests must use the debug app and real store; they must never substitute IPC. Test fixtures and config/index directories are temporary and synthetic. See `../../docs/phase-0-desktop-evidence.md` for red/green results and native smoke invocation.

The full repository check invokes `bash scripts/check-desktop.sh`, including the required native macOS smoke. Run it in a macOS graphical session; unsupported hosts fail explicitly. Production builds use no `native-test` feature, test config overlay, or `VITE_NATIVE_TEST` environment variable.

To configure notes, run `cargo run -p topics-cli -- use /absolute/path/to/notes` from the repository root, then launch the desktop. The CLI and desktop share the same app config; `TOPICS_APP_DATA_DIR` can override their app data directory with an absolute path. The native smoke sets that override to its own temporary directory.
