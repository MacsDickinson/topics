# Phase 0 accepted review fixes

Integrated follow-up to [the original independent Opus core review](reviews/phase-0-core-review-original.md), preserved verbatim at its original `335ba24` target. Its initial verdict and disclosure are historical; this worker never ran the unsafe empty-HOME/no-override probe. Folder-switch/startup work is in `3b5c70a`, extended core fixes in `9a3c1f0`; independent BDD `1f73e46` / `21001f8` are imported as `6a67d4c` / `8650eeb`. Independent verification of this final diff remains the coordinator's next gate.

## Accepted findings and disposition

| Finding | Implementation and evidence |
| --- | --- |
| B1 missing selected root prevents use/init; corrupt index prevents recovery | `open` only prepares app data. The full startup hash scan runs on first data read, while reconfiguration scans only the proposed root. Actual CLI use and init recover from a missing old root. Reads return unavailable-root errors. Recognized SQLite corruption or malformed derived JSON resets only the disposable database/journals under the shared OS lock, then retries/rebuilds. Missing/corrupt sources never trigger deletion of notes or config. |
| B2 / desktop MEDIUM cross-process folder mixing | One stable OS lock spans current config discovery, complete scans, SQLite/config publication and reads; watchers discover current config dynamically. SQLite root identity is committed with indexed rows and compared before reads. Real app + CLI, separate store instances, raw SQLite before read-time repair, and repeated watcher intervals are covered. |
| Desktop LOW invalid app-data setup abort | Managed resolver error keeps the window open. Search displays the error, without opening any store or falling back. Empty and relative overrides have separate real native sessions. |
| H1 unlink between read and metadata | Content and metadata come from an open file handle. A synthetic deletion/recreation workload that failed the old code now completes 100 scans without a scan-wide error. Per-file failures warn; roots are checked for availability before publication. |
| H2 Unix backslash collision | Relative path serialization joins host path components. A literal `a\b.md`, real `a/b.md`, and literal `memory\people\literal.md` remain separate; the last is not misclassified as a person. |
| M1 punctuation/NUL query handling | NUL becomes a separator and punctuation-only terms are omitted. Tests assert actual ledger hits for `ledger &`, `& ledger`, `ledger -`, `ledger - vendor` and NUL-bearing queries. |
| M2 title-only highlight | FTS chooses the best snippet column (`-1`); a filename-only meeting title match must contain its highlight. Source text is not escaped or transformed in Rust. |
| M3 unreadable source / root | Unreadable individual Markdown remains a plain document with empty content and a relative warning, never its stale body/person record. An unreadable root fails closed and leaves the prior derived rows untouched. Permission fixtures restore permissions on scope exit. |
| M7 failed config publication | The matching SQLite transaction remains uncommitted while atomic config replacement is attempted. A failed publication rolls the transaction back; a direct raw-index probe verifies old rows remain, together with old config and selected root. If interrupted after config replacement but before SQL commit, root identity forces repair on the next read. |
| M8 unconfigured reads | Search, documents and people return `Choose a notes root first`; the real CLI exits unsuccessfully with that guidance rather than returning empty results. |
| M5 idle watcher battery cost | Polling compares source metadata, structure, and (on Unix) ctime/inode/permissions; it reads and hashes bodies only when the snapshot changes. Explicit index and the first read after open still hash every file, including equal-mtime offline edits. A unit check verifies repeated idle polls take the no-rescan branch and same-size/restored-mtime changes are detected on Unix. No new CPU-percentage claim or benchmark framework is introduced. |
| M6 unsafe environment fallthrough | Any obsolete `TOPICS_APP_DATA` produces an explicit error, including when the new override is also set. On Unix, explicitly empty HOME without an override fails before OS default discovery. That edge case is tested with injected inputs and a panic fallback closure, never through the real OS resolver. CLI tests always use an absolute synthetic override or valid synthetic HOME. |
| T1 accepted small coverage | Watcher error/recovery and stop-on-Drop, stale profile removal/replacement, multiple glossary aliases and alias removal have focused tests. |
| T2 / source-tree snapshot T1 | Independent BDD owner supplied explicit step definitions and complete source-tree snapshots, integrated without changing approved feature files. See [BDD evidence](phase-0-bdd-evidence.md). |
| T3 / T4 evidence and locking | Actual red logs are committed, as are full local/native results. Core check and native fixture CLI use `--locked`; no dependency changes are required. Hosted CI has not been run by this local-only task. |

H0 raw HTML was **not accepted as an injection defect**: the existing React renderer escapes note text and only interprets exact mark tokens, with no `innerHTML`. Literal-mark cosmetic handling remains deferred. M4 watcher-warning UI expansion, optional parser/date/filename policies, graph work and later-phase features are unchanged. Persistence approval and final summary-document reconciliation remain coordinator-owned decisions; this task does not invent a new sign-off record.

## Actual red before fixes

- `cargo test -p topics-store --test review_regressions`: **8 failed**, with each accepted behavior failing at its own assertion. [Core red](../apps/desktop/evidence/core-review-red.txt).
- Actual CLI missing-root use/init and obsolete-variable probes failed their new assertions. [CLI red](../apps/desktop/evidence/core-recovery-cli-red.txt).
- Pure empty-HOME and obsolete-environment guards failed safely, including the injected fallback panic. [Resolver red](../apps/desktop/evidence/resolver-safety-red.txt). No real default discovery with empty HOME occurred.
- Earlier real-app switch and startup red logs remain linked in [coherence evidence](phase-0-coherence-evidence.md).

## Synchronization and recovery details

The public accessor is `notes_root(&self) -> topics_store::Result<Option<PathBuf>>`. It reads current config under the shared lock and propagates unavailable-root/config errors. Internal helpers use an already-held guard; no recursive acquisition occurs. The desktop session mutex precedes the store lock, which precedes SQLite. Watchers never take the desktop mutex and sleep only after releasing the store guard. The stable lock inode is never replaced or removed. Source traversal is factored into `scan.rs`; parsing responsibilities remain unchanged.

The lock covers cooperating Topics processes. An unavailable root or unrelated filesystem/permission error is surfaced, not treated as index corruption. Only recognized derived SQLite corruption and invalid serialized index rows permit removal of the named index/journal files. Config errors remain config errors. Root identity repairs valid-but-mismatched config/index files after interruption. Full scans can delay queries or folder switches while holding the guard; correctness takes precedence over simultaneous indexing for Phase 0.

Synthetic source mutations are explicit test inputs (churn, external editor, profile edits and permission changes). Production store code only reads notes; all config/index/lock writes stay in isolated app data. Native fixture source bytes are restored before snapshot verification, and temporary roots/config/index are removed. No live notes, malformed repository notes symlink, cloud note submission, global runtime changes, push or main merge occurred.

## Final verification

`bash scripts/check.sh`: **exit 0** after both fix checkpoints and independent BDD integration. [Final full log](../apps/desktop/evidence/final-review-check.txt) records fmt, clippy with warnings denied, **34 Rust tests** (including 2 BDD source-tree snapshot tests), **15 core scenarios / 78 steps**, **4 Playwright checks**, and **4 real native assertions across 3 macOS app sessions**; no skips.

After the final CI routing correction, `bash -n scripts/check.sh scripts/check-desktop.sh` and `bash scripts/check.sh --core-only` passed. The latter excludes `topics-desktop` from clippy/test, so Ubuntu core CI does not require GTK/WebKit; default/full mode retains the exact already-verified workspace scope. [Core-only log](../apps/desktop/evidence/final-core-only-check.txt). The macOS desktop workflow now invokes the **full** check and explicitly installs Rust 1.98.1 plus rustfmt/clippy. Explicit toolchain selection follows the [action's documented input form](https://github.com/dtolnay/rust-toolchain#inputs); hosted workflow execution is still pending.

`npm run build --prefix apps/desktop` passed after native smoke, restoring ordinary production assets; [final production build](../apps/desktop/evidence/final-production-build.txt). Default `cargo tree --locked -p topics-desktop --edges normal` contains no WDIO plugin dependency, and built assets contain none of `wdioTauri`, `__wdio` or `WDIO Plugin`.

Fresh real native screenshots were captured and visually inspected: [selected new root](../apps/desktop/evidence/native-root-switch.png), [external edit convergence](../apps/desktop/evidence/native-watcher-edit.png), [empty override](../apps/desktop/evidence/native-startup-empty.png) and [relative override](../apps/desktop/evidence/native-startup-relative.png). Both approved original native result/no-match captures were refreshed as well. Native fixtures and test workers stopped and their temporary source/config/index directories were removed.
