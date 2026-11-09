# Setup job worker handoff

Task: `task_50b086b5058f`; dispatch: `ctx_420c8d463718`.
Dedicated worktree: `/Users/macs/orca/workspaces/topics/topics-setup-jobs`; branch: `MacsDickinson/topics-setup-jobs`.
Baseline: `f9b6508`; all work is local commits, with no push, merge, main changes
or nested workers. Owned implementation is complete and ready for independent
Claude review. No genuine-model, native UI, live-note or hosted readiness is claimed.

## Result and key commits

The bounded SetupCoordinator uses canonical core/store DTOs and the real Runtime,
HostTools and CommitPermit. It supports start/control/get plus bounded asynchronous
watch snapshots; exact consent is core-owned, every batch uses a fresh capability,
success stages draft work only, and core atomically publishes graph/mappings and
checkpoint under the runtime fence. Cancellation/root switch revoke runtime
publication before core capability changes. Completed work and human edits survive
retry, restart and SQLite rebuild. Changed inputs need a new exact consent, and
reconsent preserves original checkpoint IDs while sending only outstanding sources.

- `109f000`: final optional-context packing implementation and full GREEN evidence.
- `1d10d66`: actual combined-budget RED (9/12 completed before the fix).
- `f3b0fd5`: source-delivery guard; no scoped read means zero processed files.
- `112b6ba`: actual unread-source coverage RED.
- `6f2436c`: separate graph_claim fixture/native renderer, with focused RED/GREEN.
- `5090f0f`: actionable exhausted attempt budgets and async consent-conflict snapshots.
- `95e0c8c`: minimal reviewed-attempt guard correction for desktop consumers.
- `79538ac`: coordinator and guarded host mechanics; `4260dfc` is the real facade
  baseline stub used by `2c507ba` to capture ten semantic RED cases on runnable core.
- `70970f1` + `285bf89`: runtime opt-in empty-terminal hook RED and implementation;
  preserve both when integrating. Existing hosts default to their previous behavior.
- `5532026`: outstanding-only reconsent protocol RED before the canonical repair.

Authorized core checkpoints were cherry-picked exactly; no uncommitted core
behavior or downstream types were copied or independently redefined:

| Authorized source | Local cherry-pick |
| --- | --- |
| `bfe9f65` | `1ad3eb2` |
| `4815346` | `47112e9` |
| `b57be61` | `a79d3b8` |
| `7488504` | `6fe7676` |
| `7154524` | `7a64c24` |
| `ddc2c4d` | `efa16df` |

## Files and integration

Owned production files are `crates/topics-setup/src/lib.rs`, `host.rs`, `context.rs`
and the authorized new crate manifest. Unit/protocol evidence is in `src/tests.rs`,
`tests/protocol.rs`, `tests/support/scripted_child.py` and `tests/support/render.rs`.
`examples/render_fixture.rs` is development-only and never imported by the production
library. The only existing runtime source edit is the default-false
requires_empty_commit hook and terminal condition in `crates/topics-runtime/src/lib.rs`;
focused tests extend `crates/topics-runtime/tests/scripted_runtime.rs`.

Owned docs are JOB-DISCOVERY, JOB-ADAPTER, JOB-FIXTURE and this handoff, plus the
tracked `features/folder-setup/evidence/job-*.txt` logs. Shared BDD/core/store/UI
changes on this branch come only from the explicitly authorized core commits.
Manager-owned root Cargo.toml, core Cargo.toml and Cargo.lock edits remain
uncommitted and excluded from worker commits, as requested.

Facade details and lock ordering are in JOB-ADAPTER. Call start/control/get on the
host blocking executor. The desktop must hold its shared lifecycle gate while
calling cancel_active before root_set; cancel_active must not acquire that gate
again. Clone watch snapshots promptly before making service calls. Resume/retry
can queue until a stopping child drains, without polling or invented durable states.
After explicit core setup_job_reconsent, control(Resume) starts/queues the outstanding
work using the new canonical revision; it never grants consent itself.

## Final validation

All Cargo commands used `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=/tmp/topics-setup-target`.
Runtime-bearing tests use synthetic temp notes, absolute isolated app-data and
synthetic Codex homes; only scripted child protocol is used. No token, personal
configuration, provider or paid-service call was performed.

| Command | Final result | Evidence |
| --- | --- | --- |
| cargo test -p topics-setup --all-targets --offline -- --test-threads=1 | 22 pass: 19 protocol + 3 unit; no ignored/skipped cases | job-final-green.txt |
| cargo test -p topics-runtime --all-targets --offline -- --test-threads=1 | 58 pass; default features only; native-live/provider tests disabled | job-runtime-final-green.txt |
| cargo fmt -p topics-setup -p topics-runtime --check | pass | job-final-checks.txt |
| cargo clippy -p topics-setup -p topics-runtime --all-targets --offline -- -D warnings | pass | job-final-checks.txt |
| native fixture renderer smoke, quoted synthetic directory and overwrite refusal | pass | job-fixture-renderer-smoke.txt |

The complete setup run took 93.96s for 19 protocol cases and 3.07s for the three
unit cases. The 100-document case uses 100 exact copied 45-byte notes (4,500 source
bytes), one consent, five bounded turns, 100 distinct citations, stable graph IDs
and one independent evidence group; it took 22.22s in the final run. This exercises
coverage and duplicate/copy handling, not realistic corpus extraction performance.
The combined-budget case uses twelve 24 KiB bodies plus 45-byte headers, twelve
turns and one consent; all exact source bodies remain delivered, with a partial
prior-context notice and intact dependencies where needed (17.34s in the full run).
The reconsent case takes two completed sources and two cancelled outstanding sources,
changes one waiting source and adds one new source, combines all five in one new
manifest batch, then transmits only its three outstanding sources under a second
explicit consent while retaining original scope/run/proposal identities (4.07s).

Required RED precedes implementation: real facade stub RED, runtime empty fence
RED, retry/exhaustion/snapshot RED, graph_claim RED, reconsent RED, unread-source
RED and optional-context RED are retained separately from upstream stub failures.
Original source-count/call-count/privacy/complete assertions were not relaxed to
hide upstream repairs. Core fixed completion to mean exact selected manifest
coverage; excluded locations remain disclosed and are never claimed analysed.

## Approved scenario mapping

These are ownership mappings, not a claim that 22 Rust tests prove all 21 product
scenarios or UI behavior. Exact test names are in JOB-ADAPTER and the final log.

| # | Approved scenario | Owned evidence / remaining owner |
| --- | --- | --- |
| 1 | Connecting a new folder starts local setup without transmission | local inventory/no-transmission/unchanged-note protocol; core + desktop journey |
| 2 | Empty and returning folders have useful entry points | core/desktop entry behavior; coordinator starts only an existing consented job |
| 3 | One consent covers an exact unchanged job manifest | multiple exact scopes, one consent, private/unrelated sentinels, no-read coverage guard |
| 4 | Changed inputs require renewed consent only for the changed scope | changed-manifest block + actual outstanding-only reconsent preserving completed IDs |
| 5 | Batches build one sourced graph instead of independent topic lists | 100-document graph, all citations, one copy evidence group, stable IDs, combined buffers and scoped prior context |
| 6 | Ambiguous identities and historic disagreement remain visible | two Joe identities/issue, competing dated reference/meeting claims, no model acceptance |
| 7 | A topic map is usable as a graph and as a list | desktop-owned presentation; canonical draft graph supplied |
| 8 | Save the eligible starting map with one explicit approval | core/desktop-owned; adapter never exposes save/approve tools |
| 9 | Saving a topic group preserves deferred material | core/desktop-owned |
| 10 | A group save handles visible dependencies | core/desktop-owned; optional model context packing preserves retained dependencies |
| 11 | Confirming a section does not confirm unseen topic facts | core/desktop-owned; no confirmation tool in model surface |
| 12 | Correct a fact in its topic without editing storage records | core/desktop-owned human action; adapter verifies scoped corrected context and suppression on subsequent analysis |
| 13 | Several occurrences of a problem create one review question | core/desktop grouping; adapter keeps same-name identity questions unresolved |
| 14 | One correction can cover a clearly previewed matching set | core exact matching/approval; adapter proves precise suppression over subsequent batches and human-text privacy |
| 15 | New evidence shows a topic diff without overriding confirmed understanding | core/desktop diff; adapter preserves accepted correction and competing dated accounts |
| 16 | Cancellation and restart retain completed draft work | real HostTools/CommitPermit pause/root-switch publication barriers; completed batch + edit retained; cancelled buffer absent |
| 17 | Failed inputs do not masquerade as complete folder coverage | failed turn discarded, honest coverage, retry preserves completed graph and no duplicates |
| 18 | A stale receipt cannot save or confirm changed knowledge | core/desktop-owned; adapter controls retain canonical optimistic revisions |
| 19 | Index rebuild preserves imported and personally confirmed distinctions | actual SQLite deletion/reopen keeps draft IDs, completed checkpoints and human draft edits; accepted review metadata core-owned |
| 20 | Undo preserves independent later work | core/desktop-owned; adapter never accepts or undoes knowledge |
| 21 | Folder scale does not turn setup into a record checklist | 100 documents / five calls / one consent / exact source coverage; one-save action core/desktop-owned |

## Limits and remaining review

Source scopes retain 100 files / 2 MiB and raw core proposal limits retain 200
candidates / 2 MiB. The frozen runtime is stricter: 64 combined candidates,
512 KiB frames, 256 KiB host replies and one existing 150s turn per attempt.
max_turns_per_batch is a finite reviewed attempt budget, including failures and
cancellations; four attempts do not cause four automatic turns. No limit changed.
Native preflight remains failclosed at five seconds per helper / 30 seconds aggregate.

The optional context packer accounts for the actual runtime candidate schema and
serialized response envelopes. It never trims current source bytes or mandatory
eligible accepted/correction context. Whole optional records/citations and available
endpoint dependencies are retained when they fit; omitted prior context is explicit
and is not evidence of absence. Oversized mandatory payloads fail closed with a
smaller-batch action; a single oversized file needs narrower upstream projection or
exclusion, not a larger runtime frame. Accepted-context local traversal remains
subject to the existing bounded host timeout. Exact-copy grouping does not promise
arbitrary modified-copy recognition, fuzzy identity merging or semantic truth checks.

What remains is independent Claude review (include the actual runtime trait/terminal
diff and new context packer), manager integration/manifests/lock, genuine pinned-Codex
synthetic/native UI acceptance and hosted CI. The manager owns those stages. This
worker makes no final native/live/hosted/product acceptance claim and will stop idle
following its worker_done receipt.

## Local commit history

```text
b995e5d docs: map setup job runtime and core contract seams
a633158 chore: scaffold setup adapter pending canonical contract
1ad3eb2 feat(setup): freeze canonical contracts and capture approved service BDD RED
70970f1 test(runtime): capture RED for guarded empty setup completions
285bf89 fix(runtime): guard opted-in empty setup checkpoints on terminal success
4260dfc feat(setup): expose agreed facade signatures before orchestration behavior
5ae1028 test(setup): draft real runtime protocol scenarios against canonical contract
79538ac feat(setup): implement bounded worker and permit-bound model host mechanics
91106fc test(setup): pin combined-buffer and model-tool boundaries with validation notes
47112e9 feat(setup): implement guarded draft jobs and exact journalled review decisions
2c507ba test(setup): capture actual facade baseline RED against runnable core
3d69aab test(setup): expose reviewed retry budget versus single turn mismatch
1247944 test(setup): expose empty checkpoint and completed-context service gaps
fd50fe3 test(setup): expose exhausted attempt and consent snapshot regressions
95e0c8c fix(setup): honor reviewed retry attempts with one bounded turn each
5090f0f fix(setup): surface exhausted budgets and consent snapshots without polling
6f2436c test(setup): add graph claim mode and shared native fixture renderer
a79d3b8 feat(setup): freeze additive reconsent and review seams with semantic RED
5532026 test(setup): capture outstanding-only reconsent protocol RED
6fe7676 feat(setup): implement exact reconsent, connection review and supplemental context
112b6ba test(setup): expose terminal success without model source read
f3b0fd5 fix(setup): require scoped source delivery before coverage publication
7a64c24 test(setup): capture selected coverage RED while preserving exclusions
efa16df fix(setup): complete exact selected coverage while retaining exclusions
1d10d66 test(setup): reproduce optional context crowding out selected sources
109f000 fix(setup): pack optional draft context within the frozen response budget
```
