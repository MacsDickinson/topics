# Phase 0 final evidence and contributor documentation

Scope: two bounded test-evidence gaps and stale documentation, based on clean `f31bf55a2bc002bc8b623d53b869b16318893dad`. **Production behavior is unchanged.** The independent final [Opus re-review](reviews/phase-0-switching-rereview.md) is preserved byte-for-byte, including its ACCEPT/no-blocker verdict, safety statement and nonblocking follow-ups. That verdict applies to the reviewed baseline; this report records the subsequent tests/docs-only change and does not invent user approval.

## T1: actual cross-process lock regression

`cli_use_and_search_wait_for_external_store_lock_then_complete` uses fresh synthetic old/new notes and an absolute temporary app-data override. The parent test process opens the existing stable `store.lock` through an independent descriptor and acquires its OS lock before launching either real CLI command. Successful acquisition is the readiness barrier: the test does not race a sleeping lock-holder's startup. `Command::spawn` launches the real executable after that barrier.

For **both `topics use NEW` and `topics search ledger`**, the CLI must remain alive and incomplete throughout a bounded 750ms observation window while the parent holds the lock. The parent explicitly drops the descriptor to release it, then the CLI must complete successfully within five seconds. Assertions check the published new-root config/count and the actual new-root title/path returned by search. The child owner kills and waits for a subprocess on every assertion/timeout path; there are no detached threads or helper processes.

Commands/results:

- `cargo test --locked -p topics-cli --test commands cli_use_and_search_wait_for_external_store_lock_then_complete -- --exact`: **1 passed**, seven other tests filtered by the explicit exact selection. [Targeted result](../apps/desktop/evidence/final-lock-test.txt).
- `cargo test --locked -p topics-cli --test commands`: **8 passed**, zero failed/ignored/filtered. [All CLI integration results](../apps/desktop/evidence/final-cli-tests.txt).

One disposable `git archive` export of the baseline received the new test file and exactly one production mutation, replacing `file.lock()?;` with a no-op comment. It used a separate temporary Cargo target, so no mutant artifact replaced a working checkout binary. The exact test was run with `cargo test --locked --offline --manifest-path <temporary-export>/Cargo.toml -p topics-cli --test commands cli_use_and_search_wait_for_external_store_lock_then_complete -- --exact`: **expected exit 101**, failed at **“CLI completed while another process still held store.lock”**, after successful compilation. [Single mutant result](../apps/desktop/evidence/final-lock-mutant.txt). The export/target were removed in `finally`; this was one mutation, not a campaign. Test subprocesses used synthetic HOME and absolute app-data overrides throughout.

The test bounds its own observation and cleanup; it does not add a production lock timeout or resolve the already-deferred stalled-volume behavior.

## H0: hostile markup evidence

The added Playwright case injects a synthetic IPC response with hostile image/onerror and script markup in **both title and snippet**, plus a legitimate `<mark>ledger</mark>` span. It checks exact literal title/snippet text, zero image elements, zero result script elements, unchanged baseline script count and zero execution counter after a browser rendering/task turn. It separately requires the legitimate mark element and text. This directly covers hostile markup rather than describing the older literal `<b>` test as such; no renderer or literal-mark cosmetic behavior changed.

`npm test --prefix apps/desktop`: **exactly 5 passed**, zero skipped: the two approved feature-driven UI scenarios, two existing async regressions and one hostile-markup case. [Browser log](../apps/desktop/evidence/final-hostile-browser-tests.txt).

## D1: documentation and comments

`docs/contributing.md` now describes lazy service open, first-data-read full hash scan, dynamic fallible owned root access, cross-process OS locking, config/SQLite identity and recovery, metadata-first polling, actionable no-root/unavailable errors, current explicit BDD/source snapshots and strict test isolation. It also states existing nonblocking limits without changing behavior or claiming approval. The two stale BDD observation comments were corrected; every non-comment byte in those Rust files remains identical.

Additional checks:

- `cargo fmt --all --check`: **exit 0**. [Formatting result](../apps/desktop/evidence/final-checks-fmt.txt).
- `cargo clippy --locked -p topics-cli -p topics-bdd --all-targets -- -D warnings`: **exit 0**. [Relevant clippy result](../apps/desktop/evidence/final-checks-clippy.txt).
- `npm run build --prefix apps/desktop`: **exit 0**, strict TypeScript plus ordinary 35-module frontend build. [Build result](../apps/desktop/evidence/final-checks-build.txt).

## Unchanged-production proof and remaining gates

`git diff --exit-code f31bf55` over `crates/topics-core`, `crates/topics-store`, CLI `src`, desktop `src`/`src-tauri`, manifests/lockfiles, approved features/plans/research, README, scripts and workflows is empty. For the two BDD files, removing only full-line `//`/`///` comments leaves byte-identical source. The preserved re-review is byte-identical to the supplied original. [Proof record](../apps/desktop/evidence/final-unchanged-production.txt).

No production or native logic changed, so the accepted cold full/native checks were not repeated; hosted CI will exercise the full suite. All temporary lock-test notes/config/index, scratch export and child processes were cleaned up. No live notes/default configuration, global runtime setting, push or main merge was touched. Hosted CI, live read-only verification, approval records and the review's nonblocking future items remain manager-owned follow-ups.
