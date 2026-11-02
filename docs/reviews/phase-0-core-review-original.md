# Phase 0 core review: c1051f7..335ba24

**Verdict: NOT ACCEPTED. 2 blockers remain, plus 1 high finding that must be resolved before desktop integration.**
All 15 approved non-UI scenarios pass, so there is no Gherkin failure. The blockers are real product failures that those scenarios do not exercise.

- Reviewer: independent Claude session. Model reported by the session: `claude-opus-5-5` (Opus 5.5), Opus tier. The session cannot independently confirm the requested effort level.
- Commits: e2faa28 (core) and 335ba24 (config follow-up), scope widened by the coordinator.
- Checkout: `topics-p0-core` stayed at e2faa28 with a clean tree. All builds, probes and mutants ran on `git archive` exports in the scratchpad.
- Line numbers refer to 335ba24.

## Disclosure: one probe touched real application data
Probing `env -i PATH=$PATH HOME= topics people` with no override, I expected an error. `dirs_sys` instead fell back to the passwd home, so the run created `~/Library/Application Support/com.macsdickinson.topics/index.sqlite3`.
- No `config.json` existed there, so no notes root was selected and **no notes were read or written**.
- I confirmed it was mine: created 02:15:50, the same second as the adjacent probe steps, with 0 rows in all tables and no open handles.
- I removed that file and the empty directory (`rm` + `rmdir`). The path no longer exists.
- This was outside the temp-only brief. The behaviour is itself finding M6.

## Hard acceptance status
| Criterion | Status |
|---|---|
| 15 non-UI scenarios pass, exact mapping | PASS. `diff` of scenario names from feature files vs runner output is identical; 15 scenarios, 78 steps at e2faa28 and 335ba24 |
| No skipped/undefined scenarios | PASS. Unknown step gives 1 failed, rc 101. See T2: `fail_on_skipped` is vacuous |
| Approved features/plan/research unchanged | PASS. `git diff --quiet c1051f7 335ba24 -- features docs/plan.md docs/research` |
| Red evidence | VERIFIED from the worker's session log: 15-failed output 01:58:03Z, first FTS code 01:58:52Z, first 15-passed 02:02:32Z, commit 02:05:54Z. Not in git. See T3 |
| `scripts/check.sh --core-only` | PASS at both commits (8 tests at e2faa28, 13 at 335ba24) |
| Full `scripts/check.sh` green | NOT MET. `scripts/check-desktop.sh` is absent; expected, shell workstream |
| `topics index` over real notes root | NOT RUN. Coordinator's step, and I deliberately did not run it |
| Plan rule "never stop the index" | VIOLATED in H1 and H2 |
| CI | NEVER RUN. Only `main` is on the remote and no workflow runs are listed |

## Blockers

### B1. A missing root or corrupt index makes the CLI unrecoverable
`crates/topics-store/src/lib.rs:80`, `lib.rs:178-179`, `crates/topics-cli/src/main.rs:4`

`open` validates and indexes the configured root, and the CLI calls `open` before dispatching any command. If the root is gone, every command fails, including the two that would fix it. The desktop calling `open` at startup would hit the same error and could not offer a folder picker.

Reproducer (synthetic):
```
topics use $B/r1; mv $B/r1 $B/gone
topics search one   -> rc=1 "No such file or directory"
topics use $B/r2    -> rc=1 same
topics init $B/new  -> rc=1 same
```
Corrupt index: overwrite `index.sqlite3` with garbage, then `search`, `use` and `index` all fail with "file is not a database". Deleting the file recovers, but nothing tells the user that. The feature file says "The index can always be rebuilt".

Required fix:
- `open` must not fail on an unavailable root. Return a service in an "unavailable root" state and surface the status.
- `use` and `init` must not depend on the old root scan.
- Treat an unopenable or malformed index as disposable: delete and rebuild it.
- Add regression tests for both cases.

### B2. CLI and desktop serve each other's notes across roots
`lib.rs:184-196` and the commit at `lib.rs:269-289`, reached via the single `index.sqlite3`, the watcher's in-memory root (`lib.rs:340`) and the shared directory from 335ba24

There is one index per app-data directory and no root identity in it. A running desktop watcher rewrites the index to its in-memory root, and each CLI `open` rewrites it back. 335ba24 makes this reachable by design, because CLI and desktop now share the directory. `docs/contributing.md` only says to avoid it, and nothing enforces that.

Reproducer: a long-lived process with `watch()` on rA, then `topics use rB`:
- About 1s later: `config.json` says rB, the database contains rA's documents, and the desktop has not followed.
- Desktop process on rA, while the CLI ran: its search returned rB's notes in **159 of 727 samples**.
- 200 CLI rounds with config=rB: 1 returned an rA-only note, 1 was missing the rB note.

This is the exact isolation the "Switch to a different notes folder" scenario promises, and it matters for work vs home notes.

Required fix:
- Store root identity with the index, or use a per-root database.
- Make the watcher re-read config and follow it, or refuse to overwrite an index belonging to a newer root.
- Add a two-process regression test.

**Status:** independently corroborates the desktop review's root-switch finding. Per the coordinator, the fix is already assigned to the shell worker (OS-backed app-data lock, dynamic selected root, transactional indexed-root identity). It remains open at this review target, so targeted re-review is still needed. That fix should also cover B1, because lock or root-identity failures must not add new ways for `open` to fail, and `use`/`init` must still recover.

Same-root concurrency is fine: 20 parallel `search` on a fresh database and 80 CLI searches with a live watcher and edits gave 0 failures.

## High

### H0. Snippets and titles pass raw note HTML, and markers can be spoofed
`lib.rs:314`, `crates/topics-core/src/lib.rs:27`

A note containing `<script>alert(1)</script> ledger <img src=x onerror=alert(2)> </mark><b>spoof</b><mark>fake</mark>` returns that text unescaped in `snippet`. Titles are raw too. Rendering as plain text shows literal `<mark>` tags, which pushes a shell toward `innerHTML`, and that means script injection in a Tauri WebView. A safe shell can tokenize on the exact tags and render everything else as text, but note text can still fake highlights.

Blocker for integration if the desktop inserts HTML; I did not review the shell. Required fix in core: use private sentinel markers and return structured spans, or HTML-escape the note text and emit only real `<mark>` tags. The approved scenario still passes either way. Add a hostile-content test.

### H1. A file vanishing during a scan aborts the index, and at startup kills `open`
`lib.rs:233`

`fs::metadata(entry.path())?` runs after a successful read, so a file deleted in that window errors the whole scan. The read at `lib.rs:217` handles this correctly with a warning. Stress: 1,500 files being deleted and recreated gave **66 of 759 `open()` calls failing** with "No such file or directory". The stress is deliberately harsh. Combined with B1 it fails startup outright. Fix: take metadata from the opened file handle, and treat failure as a warning.

## Medium
- **H2. Backslash in a filename** (`lib.rs:216`, `.replace('\\', "/")`). On macOS, `a\b.md` next to `a/b.md` gives `UNIQUE constraint failed: documents.path`, so the index stops, and with B1 `open` is bricked. A lone `memory\people\x.md` is reported as `memory/people/x.md`, kind `person`, at a path that does not exist. Fix: join `components()` and do not rewrite on Unix.
- **M1. Punctuation-only query terms return nothing** (`lib.rs:306-315`). `ledger &`, `& ledger`, `ledger -` and `ledger - vendor` all return 0 hits on text containing those words, because an empty FTS phrase is ANDed in. A NUL byte returns `Err("unterminated string")`. Fix: drop terms that produce no tokens and strip NUL. Current test only asserts `is_ok`, not results (`crates/topics-store/tests/safety.rs:88`).
- **M2. Title-only matches have no highlight** (`lib.rs:314`, snippet column fixed to 2). `standup`, which appears only in a meeting filename title, returns `"# Notes\nnothing relevant"`. Fix: `snippet(search,-1,...)`.
- **M3. Unreadable content silently leaves the index** (`lib.rs:204`, `lib.rs:217-222`). `chmod 000` on a file drops an indexed document with only a warning. `chmod 000` on the root gives rc 0, 0 documents and a wiped index. Plan says "every .md file as a document". Fix: keep a plain document entry with a warning, and make an unreadable root an error. Walk warnings also contain absolute paths, unlike every other path in the API.
- **M4. Watcher and startup warnings are discarded** (`lib.rs:80`, `lib.rs:340-342`). Only errors reach `last_error`, so the desktop cannot show invalid UTF-8 or unreadable warnings without another `index()` call.
- **M5. Watcher is a full read and hash every 200ms** (`lib.rs:344`). Release build, synthetic data:
  | Corpus | No-change scan | Idle CPU (one core) | Edit to search |
  |---|---|---|---|
  | 50 × 5KB | 2ms | ~3% | 27-148ms |
  | 525 × 20KB | 44ms | ~21% | 336-427ms |
  | 5000 × 20KB | 423ms | ~68% | **1815-2929ms** |

  The 2s target holds at the research estimate of about 525 files, so this is not an acceptance failure, but it burns battery continuously and exceeds 2s at 5,000 files. Plan says "file watcher", and only startup needs a hash scan. Fix: filesystem events, or a stat-first poll.
- **M6. Override safety has two silent fallthroughs to real data** (`crates/topics-store/src/app_data.rs:13`).
  - An exported obsolete `TOPICS_APP_DATA` is ignored, so the run uses the default. Verified with a synthetic HOME: the index appeared under `com.macsdickinson.topics` and nothing under the obsolete path. That variable was in the e2faa28 README, so a stale shell can reach real data and scan the configured real root.
  - Empty `HOME` falls back to the passwd home (see Disclosure), so blanking HOME does not isolate.

  Empty, whitespace and relative overrides do fail correctly with no writes. Fix: error when `TOPICS_APP_DATA` is set. Native smoke harnesses should always set the override, and could verify the resolved path before launch.
- **M7. Failed config write leaves state half-switched** (`lib.rs:101`, `lib.rs:118-124`). With the write failing, `use` returns Err, but the in-memory root and index are on the new root while config has the old one. Reopen reverts. Fix: persist before committing, or roll back.
- **M8. No-root reads look like empty results.** `topics search x` and `topics people` give `[]` rc 0 with no root, and create `index.sqlite3`, while `index` errors. A fresh install would read as "No matches". Fix: return the same "Choose a notes root first" error.

## Low / optional future scope
- CLI parses arguments after `open` (`main.rs:4-5`): `topics` and `topics bogus` create app data and scan the root. Parse first.
- `std::env::args()` panics on a non-UTF-8 argument (`main.rs:5`). Use `args_os`.
- Overlap check bypass via firmlink (`lib.rs:92`): `/System/Volumes/Data/<app-data>/sub` was accepted as a root. Compare device and inode if this matters.
- `init` false rejections: `../x` is rejected although safe, and this also fails `commands.rs:18` when TMPDIR contains `..` (hit on my first baseline). A folder with only `.DS_Store` counts as non-empty. A plain file gets the "Folder is not empty" message.
- Symlink swap between walk and `fs::read` would follow the link. Narrow race; use `O_NOFOLLOW` if wanted.
- Parser (`crates/topics-store/src/parse.rs`):
  - `2026-+1-01-x.md` yields date `2026-+1-01`.
  - `2026-10-281.md` is accepted as dated.
  - `2026-10-28-16-05.md` gets title `16 05`.
  - A BOM hides the H1.
  - `.MD` is skipped.
  - `.git/` and `.trash/` Markdown is indexed; decide policy.
  - An empty profile field gives `Some("")` rather than `None`.
- Hand-off gap: no schema version, so a future parser change will not rebuild unchanged documents.

## Test quality
33 mutants, plus 1 corrected stale-people replacement, ran against unit tests and BDD. Scenario-level tautologies: none.
- Null `index()`: 12 of 15 scenarios red. The 3 greens do not depend on indexing.
- Null service: 14 of 15 red. "Indexing never modifies" passes as a negative property, but is killed by the write-a-file and bump-mtime mutants.
- Rename's count step is trivially true (3 before and after), but the following search steps cover it.
- BDD was stable: 5 runs plus 3 under CPU load, all 15/15.

**T1. Survivors, regression tests required:**
| Mutant | Gap |
|---|---|
| Drop does not stop the watcher | Lifetime untested (probe shows it works) |
| `last_error` never recorded | Watcher errors untested (probe shows it works) |
| Stale people rows (no delete + `INSERT OR REPLACE`) | Profile/alias edits and deletes untested (probe shows it works) |
| Unreadable file aborts index | Unreadable untested |
| Failed switch keeps new root | Rollback untested |
| Scan creates a directory in notes | `crates/topics-bdd/src/main.rs:48-51` snapshot ignores directories, weakening the read-only check |
| Glossary only first row | Multiple aliases untested |

Excluded from the count of real survivors: "root not canonicalised" (likely an artifact of my already-canonical TMPDIR), and the first stale-people mutant (equivalent; replaced by the row above).

**T2. `fail_on_skipped` is vacuous** (`crates/topics-bdd/src/main.rs:83-85`, `:264`). The `.*` catch-all makes every step defined, so removing the flag changes nothing. The guarantee actually comes from the final `panic!`. A single if/else dispatcher that treats Given/When/Then alike also falls short of the plan's "readable as service usage examples". Fix: discrete step definitions.

**T3. Red evidence is genuine but weak.** I reproduced the exact numbers (15 failed, 15 steps failed). Every scenario died on its first step, so no assertion ever ran. My null-implementation runs supply the missing confidence. The raw log is not committed.

**T4. Process and docs**
- Check.sh and CI lack `--locked`, CI is Linux-only for a macOS product, and CI has never run.
- `docs/phase-0-core-evidence.md` still says 8 tests and does not mention 335ba24.
- `docs/persistence-assessment.md` asserts rather than executes queries, and it landed with the implementation. Plan requires approved scenarios before persistence sign-off; I found no approval record. **Coordinator should confirm with the user.**

## Verified working
- Notes tree identical including directory mtimes after open, watch, search and people; no notes writes.
- Idle open and watcher leave the database mtime unchanged.
- Overlap rejected for same path, case variant, symlink and dangling symlink, with nothing created.
- `init` never overwrites.
- Symlinks are not followed.
- Invalid UTF-8 and NUL bodies stay plain documents.
- SQL and FTS syntax in queries is treated as literal.
- Profile and alias edits, glossary delete, person delete and offline edits converge in about 210ms.
- Watcher reports root loss, recovers, and stops on drop.
- 200 root switches under a live watcher: 0 inconsistent.
- 335ba24 resolver: creates nothing, rejects invalid overrides; tests are meaningful and use synthetic homes.

## Commands and results
```
scripts/check.sh --core-only @e2faa28   exit 0: 8 tests, 15 scenarios, 78 steps
scripts/check.sh --core-only @335ba24   exit 0: 13 tests, 15 scenarios, 78 steps
  (first e2faa28 attempt: exit 101, caused by my TMPDIR containing "..", see Low)
topics-bdd --tags 'not @ui' x5, x3 under CPU load   15/15 every run
topics-bdd (no tags)                    rc 101: the 2 UI scenarios fail, as expected
topics-bdd from wrong cwd               rc 101: fails closed
scratch probes: fts html consistency unreadable watcher parser backslash overlap readonly churn perf desktop
mutate.py / null.py                     33 mutants + null implementations + red repro
cargo check --locked --offline --workspace @335ba24   ok
orca search "Phase 0 service not implemented"         located the worker session; timestamps above
```
Not run: Windows or Linux native, the desktop shell, live notes, and CI.

Artifacts: scratchpad `baseline.log`, `cfg-baseline.log`, `mutation.log`, `mutate.py`, `null.py`, `twoproc.sh`, `probe/crates/probes/src/main.rs`.
