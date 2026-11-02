# Phase 0 folder-switch coherence and startup errors

Follow-up to independent Claude review of shell baseline `0c8bb3a`. All work is in the existing shell checkout; the core checkout and approved feature files are unchanged. This fixes the accepted folder-switch race and invalid app-data startup error, with synthetic regression and real macOS app evidence.

## Reproduced before implementation

- `cargo test -p topics-store --test coherence`: **2 failed** against the original service. An independently opened service selected the new root, but a retained instance restored old-root rows; a valid mismatched config/index pair also returned the wrong folder. [Rust red log](../apps/desktop/evidence/coherence-red.txt).
- `npm run test:native --prefix apps/desktop`: **failed** in the actual app plus real CLI test: expected `New ledger migration`, received `Ledger migration` after `topics use` selected the new root. [Native switch red log](../apps/desktop/evidence/native-coherence-red.txt).
- The focused startup probe (`TOPICS_TEST_ONLY_STARTUP=1 npm run test:native --prefix apps/desktop` in the pre-fix harness) **failed** because the app exited with **SIGABRT** before embedded WebDriver became ready when the override was empty. [Native startup red log](../apps/desktop/evidence/native-startup-red.txt). The final harness exposes the explicit focused command `npm run test:native --prefix apps/desktop -- --startup-only`; the ordinary full check always runs all native cases.

No production fix was applied before capturing the corresponding red behavior.

## Shared service boundary

`TopicsService` no longer caches the selected notes root. Every index pass, result read, and root-setting read obtains the current config while holding an exclusive OS-backed `store.lock` in the canonical app-data directory. `notes_root()` now returns `Result<Option<PathBuf>>`, so callers get a current owned value and configuration failures propagate. The BDD adapter and desktop use that fallible API; approved Gherkin is unchanged.

The lock spans complete source scans, SQLite publication, config replacement and result reads. A scanner that began before a switch must finish before the switch can publish; a scanner acquiring the lock afterward discovers the new root. Old watcher instances therefore cannot commit old-root rows after a successful CLI switch. The lock file is stable and never renamed or removed; each operation opens its own descriptor and releases it on scope exit, including errors. Rust's [File::lock documentation](https://doc.rust-lang.org/std/fs/struct.File.html#method.lock) specifies exclusivity across handles/processes and release when the descriptor is closed. All cooperating Topics processes use this same protocol.

There is one lock order: the desktop's session mutex (when applicable), then the store OS lock, then SQLite. Watchers do not take the desktop mutex, and their 200ms polling sleep occurs after the OS guard is dropped. Internal index/read helpers require an already-held store guard and do not recursively acquire it.

SQLite stores the indexed root identity in the **same transaction** as document, people and FTS rows. Following the extended core review, config is replaced atomically under the same OS guard while that transaction is still uncommitted, so publication failure rolls SQL changes back. If a process stops between config rename and SQL commit, the next read compares identities and rebuilds the disposable index from the selected root before returning anything. Legacy indexes without identity rebuild as well. Invalid config and unconfigured reads yield actionable errors; they do not appear as No matches. Source notes are never rewritten by the service.

This deliberately serializes scans and searches for Phase 0 correctness: a long scan can delay another query or root switch. It does not change approved search or folder-selection outcomes. Unsupported locking returns an error rather than proceeding without synchronization.

## Desktop startup behavior

App-data resolver failure is saved in managed state instead of escaping Tauri setup. The window opens, and an attempted search displays the original resolver error through existing error UI. Neither an empty nor relative override opens a store or falls back to personal config. Both cases are exercised in separate real app sessions; their app processes also receive synthetic home directories, and the harness verifies no fallback app-data directory was created.

## Green verification

`bash scripts/check.sh`: **exit 0**, with fmt, clippy (`-D warnings`), **16 Rust tests**, **15 core scenarios / 78 steps**, **4 Playwright checks**, and **4 native assertions across 3 real macOS app sessions**, no skips. [Full log](../apps/desktop/evidence/coherence-full-check.txt).

Targeted checks:

- `cargo test -p topics-store --test coherence`: both independent-instance / raw SQLite and interrupted-publication regressions pass; the full check includes their final direct SQLite assertions before any service read can repair state.
- `cargo test -p topics-store -p topics-cli`: all pass, including a retained watcher in one process plus real CLI child switching and querying across five polling intervals. [Targeted Rust log](../apps/desktop/evidence/coherence-rust-green.txt).
- `npm run test:native --prefix apps/desktop`: original search/no-match smoke, retained app + real CLI switching across five watcher intervals, external new-root file-edit convergence through desktop IPC, empty override and relative override all pass. [Targeted native log](../apps/desktop/evidence/native-coherence-green.txt).

The native test simulates an external editor writing only its synthetic new-root file, verifies watcher convergence, restores original fixture bytes in `finally`, and compares complete source path/hash snapshots before cleanup. Both temporary roots and isolated config/index are removed. No live notes or repository notes symlink are accessed; no note content is sent to any cloud service.

Fresh real-app screenshots (synthetic content), visually inspected:

- [New folder after CLI switch](../apps/desktop/evidence/native-root-switch.png)
- [External edit converged through watcher](../apps/desktop/evidence/native-watcher-edit.png)
- [Empty override error](../apps/desktop/evidence/native-startup-empty.png)
- [Relative override error](../apps/desktop/evidence/native-startup-relative.png)

The original [native result](../apps/desktop/evidence/native-ledger.png) and [No matches](../apps/desktop/evidence/native-no-matches.png) screenshots were refreshed by the full run too. After the smoke, `npm run build --prefix apps/desktop` passed and restored the ordinary frontend; [production build log](../apps/desktop/evidence/coherence-production-build.txt). Default Cargo dependencies contain no WDIO plugins, and production assets contain none of `wdioTauri`, `__wdio` or `WDIO Plugin`.

## Review and limits

Changes are confined to shared store synchronization, its synthetic regressions, CLI integration tests, the mechanical BDD API adjustment, desktop error handling and native harness/evidence. No optional literal-mark cosmetic changes, new UI settings or later-phase features were added. Independent review of this fix and hosted CI remain coordinator follow-ups; this task verifies local macOS execution. The previously documented development-tool advisories remain unchanged. No push or merge to main occurred.

The extended accepted core-review fixes and independent BDD integration are recorded in [final review-fix evidence](phase-0-review-fixes.md), which supersedes the checkpoint totals here.
