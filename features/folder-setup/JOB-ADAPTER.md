# Setup adapter implementation and verification status

Current canonical baseline: authorized `bfe9f65` (local cherry-pick `1ad3eb2`).
The service implementation is still stubbed in this worktree; **job protocol
coverage is not verified**. Coordinator will provide the authorized runnable
checkpoint. Preserve the concrete facade stub `4260dfc` to repeat the same
protocol cases as adapter-specific RED on runnable core, then test the current
implementation without weakening assertions. Existing evidence files explicitly
separate upstream-stub failures from runtime RED/GREEN.

## Desktop facade

`topics_setup::SetupCoordinator` takes shared `Arc<Mutex<WorkspaceService>>`,
`Arc<Runtime>` and the desktop `Arc<Mutex<()>>` lifecycle gate. Blocking methods
belong on the desktop blocking executor:

- `start(SetupJobRequest) -> WorkspaceResult<SetupJob>` starts an already
  consented Ready job; it cannot create transmission consent or accept knowledge.
- `control(SetupJobControlRequest) -> WorkspaceResult<SetupJob>` pauses/cancels
  or explicitly resumes/retries. A requested resume may queue until the previous
  cancelled child drains, rather than race the runtime's one-active-run slot.
- `get(SetupJobRequest) -> WorkspaceResult<SetupJob>` reads canonical persistence.
- `subscribe() -> watch::Receiver<Option<SetupJob>>` supplies bounded coalescing
  snapshots for async consumption. Job/root/batch/run IDs and revision are core
  values; no frontend polling or independent durable job state is introduced.
- `cancel_active() -> WorkspaceResult<()>` is called **while the desktop already
  holds the shared lifecycle gate**, before root_set. It deliberately does not
  acquire the gate again. Runtime cancellation precedes core pause/revocation;
  the subscription is cleared, and stopped generations cannot emit old context.

The worker launches one canonical batch at a time. Runtime callbacks forward
only bounded publication/terminal events; untrusted text deltas are not briefings
or persisted facts. Failed turns discard their own buffer; successful work is
staged through `setup_batch_stage_guarded` with graph/mapping/checkpoint atomic
visibility under the real CommitPermit. There is no later attach operation.
Empty successful turns use the narrowly authorized default-false runtime opt-in
hook; other hosts retain their existing behavior. Model calls receive exact
selected bytes, evidence-associated core accepted context and capability-bound
core draft context. Whole inventory/graph, excluded names and human stage results
are not serialized to the model.

## Evidence and scenario ownership

The approved feature files and shared service BDD bindings are unchanged by this
stream. Core owns normalization, durability, grouping, privacy/hash rechecks and
all save/confirm/correction decisions; desktop owns UI behavior and action counts.

| Approved scenario | Owned protocol test / evidence |
| --- | --- |
| setup 1: local folder setup | `inventory_is_local_and_consent_starts_one_bounded_job` |
| setup 2: empty/returning | empty terminal runtime tests; core/desktop entry behavior remains upstream |
| setup 3: one exact-manifest consent | inventory test, 100-document test, `private_sources_and_unrelated_topic_inventory_never_reach_the_child` |
| setup 4: changed waiting scope | `pause_reopen_and_changed_waiting_manifest_never_retransmit_completed_inputs` |
| setup 5: one sourced graph / copied evidence | `hundred_documents_use_exact_multiple_scopes_and_one_draft_graph`, `multiple_tool_calls_validate_the_runtime_combined_buffer_once` |
| setup 6: ambiguity / disagreement | `ambiguity_stays_an_issue_and_model_success_has_no_approval_surface`; dated disagreement remains core-owned |
| continuity 16: cancel/restart | `unchanged_pause_resume_preserves_checkpoint_ids_and_does_not_resend_completed_sources`; two real host/permit preparation-race unit tests |
| continuity 17: failed coverage/retry | `failed_turn_preserves_other_batches_and_retry_does_not_duplicate` |
| continuity 19: index rebuild | `restart_and_sqlite_rebuild_preserve_draft_edits_and_completed_batch_ids`; accepted review metadata remains core-owned |
| continuity 21: >=100 documents | 100-document test records 5 scopes/turns and one transmission consent; save action remains core/desktop-owned |

Additional regressions cover root-switch context/capability invalidation,
combined tool-call buffers, forbidden approval/privacy tools and scope widening.
Fake children implement the pinned app-server protocol and use the real runtime
and service. Runtime-bearing test processes use synthetic Codex homes; sources
and absolute app-data paths are synthetic temp directories. Channel/process waits
and job budgets are bounded; deterministic barriers exercise publication races.

Completed independent of core stubs:

- `job-empty-red.txt`: 4 scripted tests, 2 behavioral failures before the terminal
  branch change, 2 compatibility/failure tests passing; exit 101.
- `job-empty-green.txt`: all 15 real scripted runtime tests pass after the change,
  including successful empty checkpoints, revocation, failed/cancelled empty
  turns and existing default-host behavior; 36.46s, exit 0.
- Scope/approval/privacy-tool denial unit test: 1 passes without core setup APIs.
- `cargo fmt -p topics-setup -p topics-runtime --check` and warning-denied clippy
  for both crates/all targets pass at the implementation checkpoint; rerun after
  integrating runnable core and any resulting changes.

`job-protocol-red.txt`, `job-service-stub-red.txt` and
`job-protocol-stub-red.txt` capture real upstream not_implemented failures,
**not adapter coverage**. They cannot substantiate GREEN, identity correctness,
privacy containment of live notes, restart correctness or completed source coverage.

## Remaining integration and review

Await authorized runnable core commits and canonical amendments for reconsent,
inference and confirmation; do not invent signatures. Capture adapter-specific
RED with facade `4260dfc`, then fix/verify all protocol cases against real service
behavior. Check privacy and cross-batch mapping, empty checkpoint IDs, changed
unsent scope, exact completed coverage, human edit preservation and lock ordering.
The test corpus deliberately includes copies, same-name people, private and
unrelated sentinels, failed turns and changed bytes. Core/manager must also verify
accepted corrections and historical disagreement across scoped context.

Core scope limits stay at 100 sources / 2 MiB; the adapter/runtime retain the
stricter existing 64 combined candidate buffer and 2 MiB transport payload.
A scope near the raw-byte cap may exceed the transport cap after JSON/citation
encoding; fail closed and preview smaller batches rather than increasing limits.
Accepted-context association uses the existing core source_context boundary;
large accepted inventories may make that existing local traversal expensive.
These paths require measurement with runnable services and the synthetic corpus.

Manager owns root/core manifests, lockfile, integration, independent Claude review
(including the actual runtime diff), genuine pinned-Codex synthetic acceptance and
hosted CI. No live/native/hosted readiness follows from scripted results here.
No push, merge, main changes or nested workers were performed.
