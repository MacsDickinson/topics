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
   current-batch eligible citations plus previously completed, unchanged, still-eligible
   job-manifest citations and associated draft nodes, no
   human inventory, waiting files, private alternative IDs/counts/name-collision errors.
   This same explicit context subset can support proposals; scope_read still returns
   only the current batch documents. Superseded accepted history and human
   statement text never enter this draft context.
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

## Additive coordinator amendment (2026-11-09)

- `setup_job_reconsent(SetupJobReconsentRequest{root_id,job_id,expected_revision,
  manifest_id,expected_fingerprint}) -> SetupJob` reviews one exact new immutable
  manifest for the existing root generation/job. `prior_consents: [SetupConsent]`
  retains original manifest and batch/scope/run identities; each current batch's
  `completed_source_ids` retains unaffected completed coverage even when the new
  manifest puts completed and waiting sources in one batch. A new capability
  contains only the outstanding subset of its reviewed manifest batch. No stale
  source/new file inherits old consent, and no original IDs are rebased.
- `setup_connection_confirm_preview(SetupConnectionConfirmPreviewRequest{
  root_id,connection_ids}) -> SetupReceipt` enumerates exact accepted connections,
  source/dependency hashes and `connections` snapshots. The existing `setup_decide`
  approves this receipt, confirming only enumerated connection IDs. Claim-only
  `setup_confirm_preview` is unchanged; verification is separate for both kinds.
- `setup_context_add(UserStatementRequest) -> UserStatement` reuses the canonical
  human statement record format with a distinct journal purpose for supplemental
  topic notes. It records exact human text only, no claim, replacement, rule or
  verification. `SetupTopicReview.context: [UserStatement]` returns those notes
  after restart/rebuild, excluding correction/edit/chat evidence. A distinct
  journal purpose is necessary to avoid guessing whether old chat questions or
  correction evidence were supplemental context; existing `user_statement` keeps
  its prior evidence semantics.
- `DraftItem.assertion_kind` explicitly projects reported_explicit,
  inferred_or_uncertain, unsupported, identity_question or correction separately
  from verification and disposition. These are conservative extraction/review
  categories derived from declared candidate kind, cited evidence and an explicit
  uncertainty flag; roles, recency and confidence never select truth. Absence of
  uncertainty is not semantic proof or personal verification. Unclassified items
  remain ineligible until classified. All data still enters through canonical
  bounded Candidate values; no new model approval/type surface.
- Empty successful terminal output is an application-only guarded-stage outcome,
  after the exact capability/hash/privacy/attempt checks and run consumption. It
  gets a stable proposal UUID and one atomic checkpoint publication. Model-facing
  `setup_batch_validate` and legacy proposal schemas still require 1–200 candidates.
- `max_turns_per_batch` counts total one-turn attempts, including cancelled and
  failed attempts. It remains finite; the desktop default is four. Setup adapters
  may retain a stricter 64-candidate combined buffer than core's 200 raw cap.

Draft context is a bounded retrieval subset, never a claim of complete knowledge:
at most 200 topic/claim/connection records and 1 MiB of serialized context, after
privacy filtering, including necessary topic labels. It never reports counts of
excluded private identities. `independent_evidence_groups` conservatively groups
verbatim passage hashes (including copies under different document headings);
its count is not independent confirmation or personal verification. Root folder
role overrides use the inventory root's empty path; longer folder overrides win.

## Coverage clarification — coordinator ruling 2026-11-09

`coverage.complete` and Finished mean completion of the exact eligible **selected**
manifest sources: every selected source is processed, and none selected failed or
remain outstanding. They never mean the entire filesystem/folder/vault was
analysed or understood. `coverage.excluded` always retains intentional preview
exclusions (app records, hidden/config/private files, filters and unselected
notes); those files are never counted as processed or sent. FinishedWithGaps
identifies missing/failed selected coverage. The 100/100 selected + app-owned
exclusion regression was captured RED before changing the predicate.
