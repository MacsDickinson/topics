# Folder setup job seam discovery

2026-11-09; baseline `f9b6508`; implementation awaits the coordinator-authorized
core contract and compilable types/stubs. This document specifies requirements,
not a competing wire schema. All persisted and desktop-facing domain values belong
to topics-core; all durable manifests, graphs and checkpoints belong to topics-store.

## Existing seams and limits

- `WorkspaceService` owns source identity, sensitivity, scope preview/read, run
  capability issuance/revocation, candidate validation and guarded publication.
  Its calls take the existing store lock; retaining one service across root
  changes is supported. `root_set` creates a fresh root epoch even for the same
  path. Job snapshots must distinguish durable folder identity from live epoch.
- `scope_preview` limits selection to 100 files and 2 MiB, excludes app records,
  unsafe links, private sources and invalid Markdown, persists source records,
  and returns an immutable preview. It cannot inventory an arbitrary folder
  larger than that scope. Folder setup needs a local inventory seam owned by core,
  rather than widening the existing scope limits or scanning in the adapter.
- `analysis_start` issues a run for one scope; `scope_read` rechecks bytes and
  capability; `source_context` filters accepted context. Existing desktop
  `ScopedHost` supplies only `scope_sources` and `propose_changes`; it validates
  combined tool-call candidates before terminal success stages any proposal.
- `Runtime::analysis_start` is asynchronous, returns the stable run ID, and emits
  sequenced root/run envelopes through an event sink. One run may be active per
  Runtime. Keep the pinned runtime, direct catalog and containment unchanged.
- `HostTools::commit_candidates` receives a real `CommitPermit`. Core prepares
  validation and serialization before the permit invokes the final short
  visibility action. Existing `stage_proposal_guarded` is the reference pattern;
  publication IDs must be UUIDs and the permit accepts 1–64 IDs.
- Additional runtime integration gap: terminal success currently calls
  `commit_candidates` only if its candidate buffer is nonempty. A genuinely empty
  extraction therefore cannot publish a completed checkpoint through a permit.
  Request coordinator approval of a small opt-in host hook (default false for
  existing hosts) to require guarded empty-turn completion; do not checkpoint
  from a later Completed callback. No runtime edit is authorized yet.
- Model payload limits remain 200 candidates / 2 MiB and the existing bounded
  runtime turn. The desktop's current host additionally caps a combined buffer
  at 64 candidates. The job adapter must specify its conservative per-run bound
  explicitly and split batches; it must not expand the runtime budget.
- Accepted context today does not resolve job-local provisional IDs or expose
  durable draft context. Reusing `topics_list` as model-visible inventory would
  violate privacy. A new core scope-filtered job context is necessary.

## Required canonical service operations

Names and argument shapes must come from CONTRACT.md; the following are required
capabilities, not proposed parallel Rust types.

1. **Local inventory and selection preview:** root-bound editable roles with
   explanations, dates, links, byte counts, excluded/private human-only entries,
   and exact eligible source identities/hashes. Apply the existing source policy
   (plus hidden/config exclusions required by the plan). Inventory does not
   transmit or grant source permission. Core partitions an authorized selection
   into deterministic bounded batches, ordered reference/profile/explicit-link
   context before meeting/log material. Roles and dates are hints only.
2. **Create/authorize job:** one explicit consent pins the exact reviewed
   manifest fingerprint and privacy state, including unknown-source consent.
   Authorization is job-local and grants no durable CloudAllowed sensitivity.
   Stable job and batch IDs, source coverage and draft revision are returned.
3. **Acquire next batch:** atomically check active root epoch, job state,
   current unsent source hashes/privacy and consent before issuing the existing
   scope/run capability. Return the canonical root/scope/run/batch identities
   and notes root for RunRequest. No new files are added silently. Retry selects
   only failed/outstanding work; completed unchanged checkpoints are retained.
   An empty successful extraction still needs a completed checkpoint.
4. **Scoped model read:** return only capability-bound source bytes, citations,
   eligible accepted context/corrections and relevant job draft context. Private
   alternatives, inventory, exclusions, unrelated topic labels, human-only paths
   and stale/restricted facts must never enter model JSON or the prompt. Core
   owns association checks, context bounds and provisional/canonical ID mapping.
5. **Validate buffered candidates:** validate a whole turn's accumulated buffer
   without publishing it. Job-local references may target earlier authorized
   draft batches using core mappings. Repeated assertions retain source evidence
   without duplicate entities/facts/questions or false copy corroboration.
   Same-name people, incompatible accounts and inferred relationships remain
   unresolved; model confidence, role and recency do not select truth. Existing
   user corrections remain authoritative within their precise scope.
6. **Guarded terminal batch publication:** prepare candidates, normalization,
   stable ID mapping, graph revision and checkpoint together before invoking a
   publisher callback analogous to `stage_proposal_guarded`. The supplied final
   action atomically exposes the durable draft batch/checkpoint under the real
   CommitPermit. No callback after terminal success appends model results. The
   callback receives a stable UUID usable as the permit publication ID even for
   an empty/no-change batch. Rejection leaves no graph or completed checkpoint.
7. **Pause/failure/recovery:** core revokes the active batch capability and
   records pause/failure without removing prior completed batches or human edits.
   Resume detects changed unsent scope and returns an actionable consent conflict.
   Updated consent can retain unaffected completed results. Normal restart and
   disposable SQLite rebuild preserve jobs, graph IDs, edits and coverage.
8. **Human snapshot/progress:** get job state, draft graph/revision, completed,
   failed, outstanding and excluded coverage, remaining batches and active run.
   Completion distinguishes full selected coverage from gaps; excluded sources
   remain disclosed. Snapshot values contain the canonical states used by UI.
   Core owns durable state, not orchestration-local shadow copies.

The adapter facade should expose start/pause/resume/snapshot and a nonblocking
event subscription/callback using canonical job/root/run IDs and snapshots.
Desktop may schedule blocking facade calls on its existing blocking executor;
no frontend polling protocol or invented states are necessary. Manager owns the
desktop commands and workspace/dependency manifest integration.

## Cancellation, mutex ordering and races

Suggested adapter order: lifecycle gate -> local service handle -> store lock ->
publication permit. Runtime.cancel must be invoked without holding the service
handle or waiting on a worker that might be preparing publication. It revokes
the permit first; core capability revocation/pause follows. If publication already
linearized, that completed batch is retained. Pending preparation cannot append
after pause/root switch. No local state mutex may be held while calling Runtime,
core, a subscriber or a potentially blocking channel. Event callbacks enqueue
work; they must not run service calls on runtime emission paths or reenter locks.

Core must confirm whether its publication callback executes under store.lock and
whether it consumes the run before the gate; this determines safe cancellation
and retry ordering. Root switching needs a facade cancellation hook before
`root_set`; stale events are ignored by exact root/job/run identity, and model
context is recreated for every batch with no previous raw prompt/history.

## Synthetic protocol scenario ownership and RED evidence

Core owns shared service Gherkin bindings and all persistence/group-save tests.
This stream owns actual orchestrator + WorkspaceService + fake scripted runtime
integration tests under a new `crates/topics-setup` crate, after types freeze.
Before behavior, run these against the agreed stubs and retain exact failing
commands/assertions as RED evidence; merely failing to compile is not behavioral
RED. Do not edit the shared BDD crate or approved feature files.

| Approved scenario | Job protocol evidence owned here |
| --- | --- |
| setup: Connecting a new folder starts local setup without transmission | inventory/start separation; zero fake-child calls before consent; notes unchanged |
| setup: Empty and returning folders have useful entry points | empty job/no model and resume existing checkpoint without retransmission; UI entry points owned by desktop |
| setup: One consent covers an exact unchanged job manifest | multiple real bounded runs after one consent; exact subsets; private sentinel absent from prompts/tool JSON |
| setup: Changed inputs require renewed consent only for the changed scope | completed batch retained; changed waiting bytes block child launch; new source absent until reconsent |
| setup: Batches build one sourced graph instead of independent topic lists | stable mapped IDs across turns, copies/repeats, distinct citations; core owns normalization |
| setup: Ambiguous identities and historic disagreement remain visible | scripted ambiguous/counter-evidence remains draft/unresolved; role/date prompt never grants precedence |
| continuity: Cancellation and restart retain completed draft work | real HostTools/CommitPermit blocked-publication race; reopen, human edit retained, cancelled buffer absent, only outstanding work resumes |
| continuity: Failed inputs do not masquerade as complete folder coverage | child terminal failure discards current buffer; separate failed/excluded/outstanding coverage; retry no duplication |
| continuity: Index rebuild preserves imported and personally confirmed distinctions | orchestrator reopens persisted checkpoints after SQLite deletion/rebuild; core owns approved graph/verification reconstruction |
| continuity: Folder scale does not turn setup into a record checklist | >=100 synthetic docs, deterministic multiple batches, one consent, exact source coverage and child call count; one-save service evidence owned by core |

Additional protocol regressions: changed privacy blocks an unsent batch; root
switch during preparation leaves no late graph; successful empty extraction is
checkpointed; failed launch cleans its capability; malformed/overlarge output
does not stage; retry/repeated jobs do not duplicate accepted work. Use explicit
barriers for publication races and bounded process/channel waits, not timing luck.

## Integration requests and remaining limits

- Manager: add workspace member/dependency `topics-setup`; its crate will depend
  on topics-core, topics-store, topics-runtime, serde_json (plus synthetic test
  dependencies only as justified). Manager owns all manifests and lockfile edits.
- Coordinator: provide the exact core CONTRACT.md + types/stubs commit SHA and
  explicit cherry-pick authorization before implementation. Confirm ownership
  of inventory/batch planning and the guarded checkpoint/context seams above.
- No existing runtime source changes are planned. If a necessary gap appears,
  request permission before any edit; retain 5s helper / 30s aggregate failclosed
  native preflight, current candidate/payload caps and turn timeout.
- Scripted tests demonstrate protocol behavior only. Manager owns genuine pinned
  Codex synthetic acceptance, native responsiveness/containment, and hosted CI;
  this stream will not claim native/live/hosted readiness.
- All development uses synthetic temp source notes and isolated absolute app
  data. No live notes, personal config/auth/provider access, external calls,
  source-note writes or Git operations inside notes.
- Own build target and `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
  CARGO_PROFILE_TEST_DEBUG=0`; inspect disk before large builds. Final checks are
  crate tests, fmt and warning-denied clippy, with runtime regressions only if
  an authorized integration actually affects runtime behavior.
