# Folder setup contract v1 — frozen 2026-11-09

Approval baseline: f9b6508, approved PLAN.md and 21 scenarios, unchanged. Canonical
Rust/JSON types are `topics_core::setup` (re-exported at crate root); service
methods are `WorkspaceService::setup_*`. Downstream code imports these values,
never copies them. All structs deny unknown fields; nullable fields must be
present. Enums use snake_case strings; timestamps, hashes, IDs, paths and evidence
retain the topic-workspace contract. This is an implementation contract, not a
claim of completed acceptance.

## Ownership and sequence

Core/store own inventory, manifests, root generations, capabilities, draft
identity, checkpoints, grouping, receipts, review metadata and all serialization.
`topics-setup` owns bounded runtime orchestration and events; desktop owns Tauri
commands/React. Neither may invent store APIs or persistent types. Manager owns
shared manifests/lock and integration. No source rewrite or git in notes.

1. `setup_inventory(SetupInventoryRequest) -> SetupInventory`: human-only local
   tree, role suggestions/explanations, counts/date/link/bytes/exclusions,
   new/empty/returning state, resumable IDs. Role overrides change ordering only.
2. `setup_manifest_preview(SetupManifestPreviewRequest) -> SetupManifest`: exact
   immutable root generation, source IDs/path/hash/label, date/exclusions,
   role ordering, bounded batches and runtime/limits. Persisted app-data preview.
3. `setup_job_create(SetupJobCreateRequest) -> SetupJob`: one explicit consent
   for matching manifest ID/fingerprint; no durable sensitivity change.
4. `setup_batch_start(SetupBatchStartRequest) -> SetupBatchCapability`: subset
   revalidated against manifest and current privacy, original root generation,
   fresh scope/run capability. Changed sources fail stale_scope and require new
   preview/consent; added files never enter an existing manifest. No completed
   batch is rerun by resume. `setup_job_control` handles pause/resume/cancel/retry.
5. Model host calls `setup_batch_validate(SetupBatchStageRequest) -> ()` while
   buffering; on successful terminal turn call `setup_batch_stage_guarded` with
   the identical typed request and runtime `CommitPermit::publish` closure.
   Signature: `(request, impl FnOnce(&Proposal, &mut dyn FnMut() ->
   WorkspaceResult<()>) -> WorkspaceResult<()>) -> WorkspaceResult<SetupJob>`.
   Preparation, canonicalization, run consumption, draft merge and checkpoint
   serialization/flush precede callback; closure authorizes one final checkpoint
   rename. Cancellation denial exposes neither proposal nor draft. NO detached
   attach/publication call. No Markdown knowledge acceptance on batch success.
6. `setup_batch_fail(SetupBatchFailRequest) -> SetupJob` records sanitized
   failure and revokes capability. `setup_job_get`, `setup_jobs_list` reconstruct
   versioned checkpoints separate from SQLite. Running attempts recovered after
   restart are outstanding, not completed. Coverage accounts for each manifest
   document and excluded/failed/outstanding paths, actual started model calls,
   exact manifest authorizations, and honest finished_with_gaps status.
7. `setup_draft_context(SetupDraftContextRequest) -> SetupDraftContext`: only
   capability-selected eligible citations and associated draft nodes, no
   human inventory, private alternative IDs/counts/name-collision errors.
8. `setup_draft_edit` requires expected job revision and same item identity;
   human edit provenance is retained. `setup_draft_disposition` supports explicit
   defer/dismiss separately; omitted items remain pending. Read API includes
   stable item/proposal/batch provenance and provisional-to-canonical ID map.

Limits must remain within existing 100 sources/2 MiB per scope, 200 model
candidates per turn; max batch/turn counts are explicit bounded job budgets.
No model confidence or folder role determines truth. Exact copies share an
independent evidence group while retaining all distinct citations/dates; arbitrary
modified-copy recognition is not promised. Name alone never merges identities.
Unresolved identities, unsupported assertions, uncertainty, inferred relations,
competing values and accepted replacements become grouped issues.

## Decisions and review

`setup_save_preview(SetupSavePreviewRequest) -> SetupReceipt` expands requested
item IDs to visible dependency closure (empty IDs means whole eligible map).
Bare required endpoint topics are included; unrelated endpoint facts are not.
Ambiguous endpoints defer affected relations. Receipt pins exact candidates,
claim revisions, source/dependency hashes, root/job revisions, included/deferred
IDs, operation bound/count, and its own fingerprint. Unselected material is not
rejected. `setup_confirm_preview(SetupConfirmPreviewRequest)` enumerates exact
accepted revision IDs and evidence; it cannot confirm an unseen topic section.
`setup_correction_preview(SetupCorrectionPreviewRequest) ->
SetupCorrectionPreview` records exact human text, explicit before/after revisions
and deterministic future rule scope. Its receipt is approved by the same API.

`setup_decide(SetupDecisionRequest) -> SetupDecisionResult` revalidates all
pinned state, persists idempotent exact decision receipts and review metadata via
existing snapshot/journal/commit-marker writes. Bounded operations report
completed/partial/conflict and operation IDs; no cross-file atomicity promise.
Unfinished selections remain drafts. Stale receipt returns actionable conflict
without approving another group. Never call legacy reject-omitted approval for a
setup job. `setup_topic_review(TopicGetRequest) -> SetupTopicReview` projects:

- FromNotes = “From your notes”: sourced baseline explicitly saved, no human
  value authority or durable cloud permission.
- ConfirmedByYou = “Confirmed by you”: only enumerated revisions explicitly
  confirmed or assertions actually changed/supplied by the user.
- NeedsInput = “Needs your input”; EarlierUnderstanding = superseded history.
- AcceptedUnverified: legacy accepted records without identifiable confirmation;
  no invented verification. Actual old user statement corrections retain it.

Review metadata and exact decision receipts live in app-owned Markdown and
rebuild from committed operation markers. Drafts/checkpoints/user edits live in
versioned root-partitioned app data, independent of disposable SQLite. Existing
`undo(UndoRequest)` checks later dependencies, compensates only targeted work,
and preserves independent later operations and original source bytes.

## Scenario evidence ownership

`scenario-evidence.md` maps all 21 approved scenarios. The core Cucumber binary
uses exact explicit step bindings and fail_on_skipped, running service-owned
scenarios without changing signed-off feature text. Graph/list navigation is
presentation-owned; native runtime cancellation/containment, actual model calls,
and UI approval counts require downstream evidence and are not claimed from
synthetic store calls. Service privacy/stale/root/cancel/rebuild/100-doc tests
remain mandatory independently of those presentation proofs.
