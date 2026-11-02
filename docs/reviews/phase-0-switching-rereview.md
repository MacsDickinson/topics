# Phase 0 switching/startup re-review: 0c8bb3a..f31bf55

**Verdict: ACCEPT. I found no blocking behaviour defect, and it is fine to proceed to the live read-only step.** One docs correction (D1) should land before final sign-off, and one test (T1) is strongly recommended.

- Reviewer: independent Claude session. Provider Anthropic, model reported by the session: `claude-opus-5-5` (Opus 5.5), Opus tier. The session cannot independently confirm the requested effort level.
- Target: f31bf55, confirmed final by the coordinator. Reviewed in `git archive` exports under the scratchpad. `topics-p0-shell` stayed at f31bf55 with 0 dirty files and `topics-p0-core` at e2faa28 with 0 dirty files.
- Safety: every CLI run used a synthetic HOME and an absolute `TOPICS_APP_DATA_DIR`. No live notes, no real config, no empty-HOME run. `~/Library/Application Support/com.macsdickinson.topics` was checked absent after each major stage.
- Line numbers refer to f31bf55.

## Focus-area results
| Focus | Result | Evidence |
|---|---|---|
| OS lock across independent processes | PASS | `lib.rs:75-84`, one stable `store.lock`. 150 real CLI switches against a retained watcher process with 3 searching threads: 0 mixed, 0 stale, 0 errors in 3,252 quiescent and 1,037 during-switch searches |
| Lock order and release | PASS | One order: desktop session mutex, then store lock, then SQLite. No nested acquisition found by reading every path. RAII guard; released after SIGKILL (tested) and on error paths |
| Atomic config/index publication | PASS, see B3 | Config renamed inside the open transaction (`lib.rs:122-131`, `:310-311`). A failed publication leaves config and raw index on the old root (tested with `config.json.tmp` as a directory) |
| Indexed-root mismatch recovery | PASS | 120 SIGKILLs of `topics use` (1,500 files per root): 98 left hot journals and 10 left real config≠index states, seen in both directions. **Next search correct every time: 0 bad, 0 hung** |
| No old-root results after CLI change with live watcher | PASS | Post-switch CLI wrong-root 0, raw database wrong-root 0, native test passes across 5 watcher intervals |
| Dynamic root API | PASS | `notes_root()` returns `Result<Option<PathBuf>>` and reads config under the lock. Nothing caches the root; the desktop session holds only the service and watcher |
| No user-config fallback on invalid env | PASS | Empty, space, relative and `.` overrides and `TOPICS_APP_DATA` all error, with 0 entries created in synthetic HOME. Empty-HOME guard verified by its injected unit test, not run for real |
| Readable startup errors | PASS | Native empty and relative overrides show "Search failed: TOPICS_APP_DATA_DIR must be a non-empty absolute path" in a usable window (screenshot inspected). Missing root and no root give readable errors |
| Native watcher edit and root switch | PASS | My own run: search.ts 2 passing, startup.ts 1+1 passing. Screenshots inspected. Search does not repair here, so the convergence comes from the watcher |

## Approved-scenario evidence
- `bash scripts/check.sh` at f31bf55, my own cold build in a scratch export: **exit 0**.
  - fmt and clippy clean, all Rust tests pass.
  - **15 scenarios (15 passed), 78 steps (78 passed)**.
  - Playwright 4 passed.
  - Native: 2 passing + 1 + 1 across 3 app sessions.
- Scenario names from the feature files vs runner output: identical, 15 of 15.
- `features/`, `docs/plan.md` and `docs/research` unchanged since c1051f7.
- No skipped or undefined. `fail_on_skipped` now really works: with the explicit steps, an unknown step gives rc 101, and removing the flag gives rc 0 with "1 skipped". The original T2 is fixed.
- Read-only: BDD snapshot now covers directories, and the "scan creates a directory" and "scan writes a file" mutants each turn 2 scenarios red. A CLI run across index, unreadable and restored states left path/mtime/size identical. No app files appeared in notes.
- Watcher scenarios are real despite lazy read repair: stopping the watcher turns exactly the 4 watcher scenarios red.

## Behaviour defects (none blocking)

### B1. Medium-low: wrong-shape index schema is not treated as disposable
`crates/topics-store/src/lib.rs:206-215` recovers only `DatabaseCorrupt`, `NotADatabase` and bad JSON. With a differently shaped table, search, index **and `use`** all fail until the file is deleted by hand:
```
sqlite3 $APP/index.sqlite3 'DROP TABLE documents; CREATE TABLE documents(path TEXT);'
topics use $ROOT  -> rc=1 "table documents has 1 columns but 3 values were supplied"
```
Same for `index_state` with the wrong shape. Truncated, garbage, zero-length, overwritten pages, missing FTS shadow table, malformed JSON and a garbage journal all recover correctly.

No Phase 0 code path produces this today; the first schema change would. Fix before Phase 1: stamp `PRAGMA user_version` and rebuild on mismatch.

### B2. Medium-low: the lock wait has no timeout
`lib.rs:82`. With another process holding `store.lock` via `flock`, `topics search` waited the full 9 seconds until the holder released it. By reading the code:
- `use` takes the same lock (`lib.rs:117`), so it would block too and can't be used to switch away from a stalled volume.
- The desktop would stay on "Searching…", holding its session mutex (`apps/desktop/src-tauri/src/main.rs:31-34`).
- `docs/phase-0-review-fixes.md` covers scan delay but not an indefinite wait.

Fix later: bounded `try_lock` retry with a readable error.

### B3. Low: commit failure after config rename returns Err with the switch half-applied
`lib.rs:310-311`. If `tx.commit()` fails after the rename, `use` reports failure while config already names the new root, which contradicts the comment at `lib.rs:124-125`. The next read repairs the index, so no wrong-root results result. By reading only; I did not force a commit failure.

### B4. Low: 2-second target and search latency degrade with size
Each edit does a full scan and rewrite under the lock. Release build, synthetic data:
| Corpus | Idle CPU | Edit to search | Max search stall |
|---|---|---|---|
| 525 × 20KB | ~5% of a core (was ~21%) | 509-541ms | 328ms |
| 5000 × 20KB | ~19% of a core (was ~68%) | **3264-3758ms** | **3379ms** |

M5's idle cost is much better and the research-scale ~525 files are comfortably inside 2s. The doc makes no claim for arbitrary sizes. Edit latency at 5,000 files was 1.8-2.9s in my earlier measurement, so it may have regressed, though machine load differed between runs.

### B5. Low, by reading only
- No `sync_all` before config rename (`lib.rs:127-128`). SIGKILL is safe (tested); power loss is untested. An empty config would give a readable error, and `use` recovers.
- Unset (not empty) HOME still reaches the passwd home (`crates/topics-store/src/app_data.rs:33`). Normal OS behaviour; **not executed**, per the safety rules.
- `to_string_lossy` root identity (`lib.rs:308`) would never match on a non-UTF-8 root on Linux, causing repeated rewrites. Not reachable on macOS; not reproduced.
- Usage errors still create app data and `store.lock` (`crates/topics-cli/src/main.rs:4`), but no longer scan.

## Test coverage (not behaviour defects)
**T1. Strongly recommended: nothing in the repo verifies the OS lock.** The no-op `file.lock()` mutant survives all store and CLI tests and all 15 scenarios. My stress probe does catch it: raw database wrong-root 37 of 150 and 1 stale desktop result. Add a deterministic test: hold `store.lock` from a separate descriptor and assert that `use`/`search` block until release.

Lesser gaps:
- Removing the watcher's identity check also survives, but it is largely redundant with the read-side check, so low value.
- **No hostile-markup test exists.** Grep across `apps/desktop/tests` and `crates/topics-store/tests` finds none, which differs from the coordinator's summary. The H0 rejection stands on code reading alone.
- `coherence.rs:66` describes publishing index before config, the reverse of the real order. The repair path is the same, and my kill fuzz covers the real order.
- Native convergence uses a 5s bound (`apps/desktop/tests/native/search.ts:52`); the 2s bound is enforced at service level only.

Mutants killed as they should be: read identity check, publish after commit, no recovery, startup scan skipped, snapshot without ctime/inode, `use` reading the old root, empty-HOME guard, watcher stopped, source-tree writes.

## Documentation
**D1. Should fix before final sign-off: `docs/contributing.md` still describes the old design.** It was not touched in this range, and Phase 0 deliverable 8 requires it.
- Line 15: `open` "scans the selected root before returning". False.
- Line 33: `notes_root` and "follows root changes made through its owning service". Stale.
- Line 39: "in-process writer mutex". That mutex is gone.
- Line 41: "polls and hashes Markdown every 200 ms". False.
- Line 43: "Avoid concurrent processes... not multiple app instances". The opposite of the fix.

The correct text is already in `docs/phase-0-coherence-evidence.md`; it mainly needs moving over.

Also stale:
- `crates/topics-bdd/src/main.rs:92` and `steps.rs:194` say reads never rebuild or repair, but `lib.rs:248-256` can.
- `docs/phase-0-core-evidence.md` still says 8 tests; `phase-0-review-fixes.md` supersedes it.

## Original core findings: status
| Finding | Status | Checked by |
|---|---|---|
| B1 missing root / corrupt index | **FIXED** for missing root and real corruption; gap in new B1 above | Live CLI, damage variants |
| B2 cross-process roots | **FIXED** | Stress, kill fuzz, native |
| H0 raw HTML | **Rejection agreed**: `apps/desktop/src/main.tsx:14-17` emits React text only, no `innerHTML`, plus CSP. Raw note text still in the snippet; literal-mark spoof is cosmetic and deferred | Code reading |
| H1 file vanishes mid-scan | **FIXED**: metadata from the open handle, `open` no longer scans | Repo churn test, code |
| H2 backslash filenames | **FIXED** | Live CLI: distinct paths, kind `other` |
| M1 punctuation/NUL | **FIXED** | Live CLI for punctuation; repo test for NUL |
| M2 title-only highlight | **FIXED** | Live CLI |
| M3 unreadable | **FIXED**: file stays a plain document with no stale body; root fails closed and keeps rows. An unreadable subdirectory drops its documents with a warning and recovers on restore | Live CLI |
| M4 watcher warnings | Deferred by coordinator | – |
| M5 polling cost | **FIXED** for idle; see B4 | Probe |
| M6 env fallthrough | **FIXED**; unset HOME noted in B5 | Live CLI + unit test |
| M7 failed config write | **FIXED**; commit-failure edge in B3 | Live CLI, raw database |
| M8 no-root reads | **FIXED** | Live CLI |
| T1 survivors | **FIXED**: watcher errors and drop, stale people, aliases, directory snapshot all covered | Repo tests read; snapshot by mutant |
| T2 catch-all steps | **FIXED**: explicit steps, flag meaningful | Unknown-step run |
| T3 red evidence | **FIXED**: red logs committed; I sampled `core-review-red.txt` and cannot independently replay history | `apps/desktop/evidence` |
| T4 `--locked`, CI, approval | `--locked` **FIXED**. Hosted CI still never run. Persistence approval remains coordinator-owned | check.sh |
| Parser and other Low items | Deferred, unchanged | – |

## Limits
- macOS only. Stress and fuzz give strong evidence, not proof.
- Power loss, commit failure, stalled volumes and Windows/Linux were not exercised.
- 12 targeted mutants this round (9 + 2 + the watcher-stopped rerun), plus the `fail_on_skipped` removal check; stopped when the coordinator asked for no further expansion.
- Not done here: hosted CI, the live read-only run, persistence approval.

## Commands and results
```
bash scripts/check.sh (f31bf55, cold, scratch)      exit 0; 15/78 BDD, 4 Playwright, native 2+1+1
diff feature scenario names vs runner               identical, 15
probes stress <topics> <dir> 150 300                0 failures; MIXED=0 STALE-ROOT=0 errors=0
probes killfuzz <topics> <dir> 120 1500             98 hot journals, 10 config≠index, BAD=0 hung=0
  (first run reported 45 windows: inflated by a bug in my probe's read-only connection; corrected and rerun)
no-op lock build + stress                           raw wrong-root 37/150, STALE-ROOT=1 (probe detects it)
probes watcher <dir> 525 20 / 5000 20               table in B4
cli2.sh / pend.sh                                   recovery, config, resolver, lock and pending-finding results above
mut2.py / mut3.py + watcher-stopped rerun           results above (a first watcher mutant failed to compile; result discarded and redone)
```
Artifacts, all in the scratchpad: `rr-full-check.log`, `cli2.sh`, `pend.sh`, `mut2.py`, `mut3.py`, `rrp/crates/probes/src/main.rs`.
