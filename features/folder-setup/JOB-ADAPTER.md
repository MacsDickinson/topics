# Setup adapter implementation and verification status

Current canonical checkpoint: authorized `4815346` (local cherry-pick `47112e9`),
following the `bfe9f65` contract. Runnable-core baseline RED is captured in
`job-adapter-red.txt`: all 10 original protocol cases reached the previous
facade `4260dfc` and failed on its absent orchestration behavior. The current
adapter has passed 13 protocol cases across subset/focused runs, plus three
publication/tool-boundary unit cases; complete job coverage remains blocked by upstream empty-checkpoint
and completed-draft-context regressions. Evidence separates these failures from
the earlier service-stub failures; no assertion was relaxed to claim full GREEN.

## Desktop facade

`topics_setup::SetupCoordinator` takes shared `Arc<Mutex<WorkspaceService>>`,
`Arc<Runtime>` and the desktop `Arc<Mutex<()>>` lifecycle gate. Blocking methods
belong on the desktop blocking executor:

- `start(SetupJobRequest) -> WorkspaceResult<SetupJob>` starts an already
  consented Ready job; it cannot create transmission consent or accept knowledge.
- `control(SetupJobControlRequest) -> WorkspaceResult<SetupJob>` pauses/cancels
  or explicitly resumes/retries. A requested resume may queue until the previous
  cancelled child drains, rather than race the runtime's one-active-run slot.
  Core consent conflicts also emit the newly persisted canonical snapshot before
  returning the error, so subscribers observe NeedsConsent without polling.
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
hook; other hosts retain their existing behavior. The setup host also requires a
successful scoped source read before it can checkpoint any analysed-file coverage,
including an empty extraction. A successful terminal with no scoped read records
failure and zero processed files; it cannot pretend that a selected folder was
analysed. This tracks capability-bound delivery, not semantic verification.
Model calls receive exact
selected bytes, evidence-associated core accepted context and capability-bound
core draft context. Whole inventory/graph, excluded names and human stage results
are not serialized to the model.

`coverage.complete` describes the exact eligible **selected manifest**: all its
sources processed, with none selected failed or outstanding. Exclusions remain
explicit and are never claimed analysed; selected completion never means whole
folder coverage. The coordinator confirmed this interpretation after reserved
app-record disclosure exposed a core predicate regression; its repair is upstream.

`max_turns_per_batch` is the total reviewed **attempt** budget, including failed
and cancelled attempts. Every attempt launches exactly one runtime turn with its
existing 150s deadline and 64 combined candidate cap. A budget of four authorizes
up to four attempts, not four automatic turns; unchanged successful batches never
rerun. Exhaustion returns an actionable conflict before changing the job into
unlaunchable Ready work. Review a new manifest to authorize further attempts.
Resume/retry fixtures explicitly consent four attempts; the exhaustion fixture
consents one. The facade does not enlarge any runtime deadline or payload limit.

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
| setup 6: ambiguity / disagreement | `ambiguity_stays_an_issue_and_model_success_has_no_approval_surface`, `historic_reference_and_recent_meeting_accounts_retain_dates_without_selecting_truth` |
| review 14/15: corrections and subsequent analysis | `scoped_accepted_correction_is_available_without_human_statement_text_and_cannot_be_revived`; human approval and review projections remain core-owned |
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
- `job-runtime-regressions.txt`: all 58 default-feature runtime regression tests
  pass, including the 15 scripted cases. Native-live/provider tests are disabled.
- `job-subset-green.txt`: 10 unaffected real orchestrator/service/scripted-child
  protocol cases pass in 42.64s, including 100 documents / five turns, retry,
  privacy, ambiguity, root switch and SQLite rebuild. Empty success and completed
  context remain explicit exclusions pending upstream repairs.
- `job-budget-snapshot-green.txt`: exhaustion, reviewed retry budget and async
  consent-conflict snapshot regressions pass after their recorded RED cases.
- `job-correction-history.txt`: scoped accepted correction, human-text privacy,
  suppression on later analysis, and historical/current disagreement pass with
  the actual service and scripted child.
- `cargo fmt -p topics-setup -p topics-runtime --check` and warning-denied clippy
  for both crates/all targets pass at the implementation checkpoint; rerun after
  integrating runnable core and any resulting changes.

`job-protocol-red.txt`, `job-service-stub-red.txt` and
`job-protocol-stub-red.txt` capture real upstream not_implemented failures,
**not adapter coverage**. They cannot substantiate GREEN, identity correctness,
privacy containment of live notes, restart correctness or completed source coverage.

## Remaining integration and review

Await authorized core repairs and canonical amendments for reconsent, inference
and confirmation; do not invent signatures. Empty terminal extraction currently
fails core's general minimum-one-candidate validator before publication;
completed draft context currently omits prior batches. Their exact failures are
`job-empty-service-red.txt` and `job-context-red.txt`. Retain both assertions and
verify the complete suite after the authorized fix. Check privacy and cross-batch
mapping, empty checkpoint IDs, changed
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
