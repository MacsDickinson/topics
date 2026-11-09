# Setup adapter implementation and verification status

Current canonical checkpoint: authorized `ddc2c4d` (local `efa16df`), following
`bfe9f65`, `4815346`, `b57be61`, `7488504` and `7154524`. All 22 setup tests pass:
19 real orchestrator/service/scripted-child protocol cases and three unit cases,
including genuine runtime publication fences. Runnable-core baseline RED used
facade `4260dfc`; its ten original cases failed on absent orchestration behavior,
not compiler/runner failures. Original privacy, source-count, call-count, empty
completion and completed-context assertions were retained through core repairs.

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
app-record disclosure exposed a core predicate regression, repaired in `ddc2c4d`.

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

Final evidence:

- `job-final-green.txt`: all 22 setup tests pass; 19 protocol cases take 93.96s,
  three unit cases take 3.07s, with no skipped/ignored cases.
- `job-runtime-final-green.txt`: final default-feature runtime regressions;
  genuine/native-live/provider cases remain disabled.
- `job-context-budget-red.txt` / `job-context-budget-green.txt`: twelve 24 KiB
  source bodies, one per turn; before packing only nine completed, afterwards all
  twelve complete with exact source bytes, bounded replies, stable graph IDs,
  intact dependencies and explicit partial-context notice (17.70s focused run).
- `job-adapter-red.txt`: actual previous-facade RED against runnable core.
- `job-empty-red.txt` / `job-empty-green.txt`: real empty-terminal fence RED/GREEN
  and default-host/failure/cancel compatibility.
- `job-unread-red.txt` / `job-unread-green.txt`: no source read cannot become
  analysed-file coverage; failure keeps zero processed files.
- `job-reconsent-red.txt` and final GREEN: renewed exact consent retains original
  completed checkpoint IDs and sends three outstanding sources from a newly
  combined five-source manifest batch.
- `job-budget-snapshot-green.txt`, `job-correction-history.txt` and final GREEN:
  bounded retry, optimistic controls, async consent state, human correction/text
  privacy, suppression and dated disagreement.
- `JOB-FIXTURE.md` and its evidence: separate graph_claim mode, shared renderer,
  exact serde_json path quoting, unchanged graph mode and no production fake backend.

The initial `job-protocol-red.txt`, `job-service-stub-red.txt` and
`job-protocol-stub-red.txt` are historical upstream not_implemented evidence, not
adapter coverage. `job-empty-service-red.txt`, `job-context-red.txt` and
`job-reserved-exclusion-diagnostic.txt` record the real upstream failures fixed
by the explicitly authorized core checkpoints; the original assertions now pass.

## Combined context budget

Core's draft projection is bounded to 200 records / 1 MiB. The frozen runtime's
actual limits are stricter: 512 KiB per frame and 256 KiB per host tool response.
The adapter accounts for the runtime's actual candidate_schema and serialized SDK
and direct-mode envelopes. It keeps all selected source bytes and eligible
accepted context, including corrections, mandatory. It packs only optional prior
draft context into remaining space, retaining whole canonical records/citations,
identity hints and available endpoint dependencies. A partial-context notice says
that omitted context is not negative evidence. Nothing is removed from storage or
coverage because optional retrieval context did not fit; privacy still comes from
core's exact capability/hash/consent filters.

## Remaining integration and review

All owned protocol cases and required checks are complete. Independent review
and manager-owned product acceptance remain. The synthetic 100-document fixture
uses exact copied assertions to prove five bounded turns, one transmission consent,
all selected source coverage, stable IDs/citations and no false independent
corroboration. Separate fixtures cover ambiguity, private/unrelated data, prior
human correction, dates, errors, changed inputs, restart and index rebuild. These
are protocol tests, not an import-duration or semantic-extraction benchmark.

Core scopes remain bounded to 100 sources / 2 MiB; raw core proposal limits remain
200 candidates / 2 MiB. The adapter/runtime retain their existing stricter 64
combined candidates, 512 KiB frames and 256 KiB host replies. Mandatory sources and
accepted context that do not fit fail closed with a smaller-batch action; they are
never truncated. An individually oversized document requires narrower upstream
projection or exclusion rather than a larger frame. Accepted-context association
uses the existing core source_context boundary; large accepted inventories can
make that local traversal expensive and remain subject to the existing timeout.

Manager owns root/core manifests, lockfile, integration, independent Claude review
(including the actual runtime diff), genuine pinned-Codex synthetic acceptance and
hosted CI. No live/native/hosted readiness follows from scripted results here.
No push, merge, main changes or nested workers were performed.
