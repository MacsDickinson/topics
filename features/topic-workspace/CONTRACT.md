# Topic workspace prototype contract v1 — in progress

This is the implementation baseline, not evidence of working code. Semantic types live in `crates/topics-core/src/workspace.rs`, re-exported from `lib.rs`; all serde JSON uses snake_case fields and `#[serde(tag = "kind", rename_all = "snake_case")]` for unions. IDs are opaque UUID strings, timestamps RFC3339 UTC, dates ISO 8601, hashes lowercase SHA-256 hex. Required fields are required; `?` means explicit nullable field; arrays may be empty. Unknown fields are rejected for incoming mutations. All paths are root-relative UTF-8 slash paths, validated inside the selected root; source bytes are never provided by a model as authority.

## Wire types

```text
RootRef { root_id: Id, path: String }
SourceRef { source_id: Id, path: String, document_hash: Hash, locator: Locator, passage_hash: Hash, excerpt: String }
Evidence = {kind:"passage",source:SourceRef} | {kind:"user_statement",statement_id:Id}
UserStatement { id:Id, topic_id:Id, text:String, recorded_at:Timestamp } # persisted in topics/statements/<id>.md
Locator { start_byte: u64, end_byte: u64, heading: String? } # UTF-8 byte offsets, end exclusive
ScopeSpec { files: [String], folders: [String], date_from: Date?, date_to: Date? }
ScopePreview { scope_id: Id, root_id: Id, sources: [SourceSummary], excluded: [Exclusion] }
SourceSummary { source_id: Id, path: String, hash: Hash, date: Date?, sensitivity: Sensitivity }
Exclusion { path: String, reason: String }
Sensitivity = "cloud_allowed" | "local_only" | "unknown"
Topic { id: Id, topic_kind: TopicKind, name: String, aliases: [String], sensitivity: Sensitivity }
TopicKind = "person" | "project" | "thread"
Endpoint = {kind:"topic",id:Id} | {kind:"claim",id:Id} | {kind:"source",source_id:Id,locator:Locator}
Claim { id: Id, revision_id: Id, subject_id: Id, predicate: String, value: String, uncertainty: String?, sources: [Evidence], supersedes: [Id], recorded_at: Timestamp, valid_from: Date?, valid_to: Date? }
Connection { id: Id, from: Endpoint, relation: String, to: Endpoint, uncertainty: String?, sources: [SourceRef], valid_from: Date?, valid_to: Date? }
Rule { id: Id, topic_id: Id, predicate: String, match_value: String, replacement: String, scope: RuleScope, source_revision_id: Id }
RuleScope { source_paths: [String], context: String? } # empty paths means all eligible sources for this topic/predicate, not all topics
Candidate = {kind:"topic",topic:Topic} | {kind:"claim",claim:Claim} | {kind:"connection",connection:Connection} | {kind:"correction",claim:Claim,rule:Rule} | {kind:"unresolved_identity",name:String,candidates:[Id],sources:[SourceRef]}
Proposal { id: Id, root_id: Id, scope_id: Id?, origin: Origin, status: ProposalStatus, candidates: [Candidate], fingerprint: Hash, created_at: Timestamp }
Origin = {kind:"analysis",run_id:Id} | {kind:"chat",topic_id:Id,message_id:Id} | {kind:"manual"}
ProposalStatus = "pending" | "accepted" | "rejected" | "conflict"
TopicView { topic: Topic, current_claims: [Claim], connections: [Connection], unresolved: [Candidate], history: [Claim], warnings: [String] }
Operation { id: Id, status: String, description: String, created_at: Timestamp }
ApiError { code: String, message: String, details: Json? }
```

Source IDs are durable per-root mappings recorded under `topics/sources/`; reuse exact path+hash and offer explicit identity repair for rename/edit ambiguity. A SourceRef must match source ID, path, document hash, valid UTF-8 byte range and passage hash against current bytes before analysis/apply; display stale citations without claiming they still resolve. ID creation in a proposal is provisional until accepted. Connections accept arbitrary nonempty relation strings; endpoint and source existence are checked against accepted records or same approved set. `supersedes` holds revision IDs; accepted heads with multiple descendants are conflicts, not last-write-wins. A correction's source is the user's chat message, not falsely attributed to the transcript. A rule is deterministic and scoped; it suppresses the specified false candidate for the target and requires review of replacement, not automatic knowledge acceptance.

## Requests and results

Every IPC command takes one JSON object named `request` and resolves to the indicated JSON value or rejects with `ApiError` (`invalid_input`, `no_root`, `stale_scope`, `stale_evidence`, `conflict`, `forbidden`, `runtime_unavailable`, `cancelled`, `internal`). No paths in errors or logs outside the selected root. `root_id` guards every operation against root switches; service checks again at commit.

```text
workspace_root_get {} -> {root: RootRef?}
workspace_root_set {path:String,create:bool} -> {root:RootRef,index:IndexReport}
workspace_scope_preview {root_id:Id,spec:ScopeSpec} -> ScopePreview
workspace_sensitivity_set {root_id:Id,target:Endpoint,sensitivity:Sensitivity} -> {operation:Operation}
workspace_analysis_start {root_id:Id,scope_id:Id,confirm_unknown:bool} -> {run_id:Id}
workspace_run_cancel {root_id:Id,run_id:Id} -> {cancelled:bool}
workspace_proposals_list {root_id:Id} -> {proposals:[Proposal]}
workspace_proposal_decide {root_id:Id,proposal_id:Id,decision:Decision,expected_fingerprint:Hash} -> {operation:Operation?,proposal:Proposal}
Decision = {kind:"approve",candidates:[Candidate]} | {kind:"reject"}
workspace_topics_list {root_id:Id} -> {topics:[Topic]}
workspace_topic_get {root_id:Id,topic_id:Id} -> TopicView
workspace_chat_start {root_id:Id,topic_id:Id,scope_id:Id,confirm_unknown:bool,message:String} -> {run_id:Id,message_id:Id}
workspace_history_list {root_id:Id} -> {operations:[Operation]}
workspace_undo {root_id:Id,operation_id:Id} -> {operation:Operation}
workspace_runtime_status {} -> {available:bool,version:String?,message:String}
```

Approve with edited candidates is the only apply route; validate fingerprint, references, dependencies, identity, rules and hashes again inside the service. Reject makes no knowledge writes. Approval accepts exactly the submitted candidates and rejects the omitted candidates; repeat decisions on a decided proposal are idempotent only when identical. Fingerprint-based duplicate analysis presents existing decisions rather than silently accepting duplicates. Candidate sets with dependencies must be reviewed together; deleting a required topic makes approval fail rather than generating dangling references. Manual capture creates a pending proposal through the same proposal service, never a bypass. Operations produce stable IDs before commit and are idempotent on retry. UI listens to `workspace_run_event` payload `{root_id,run_id,seq,event}` where `event` is `{kind:"text_delta",text:String}`, `{kind:"proposal",proposal_id:Id}`, `{kind:"completed"}`, `{kind:"cancelled"}`, or `{kind:"error",error:ApiError}`. Sequence is monotonic; one terminal event closes the run, late/duplicate/malformed/wrong-root events cannot mutate state. Model proposals remain buffered until a successful terminal turn; cancellation, errors and late messages discard the buffer and never apply proposals. Explicit user confirmation of scope precedes `analysis_start`; scope snapshots expire on root or hash change.

## Durable format and ownership

`topics/topics/<id>.md`, `topics/claims/<revision_id>.md`, `topics/connections/<id>.md`, `topics/rules/<id>.md`, `topics/sources/<id>.md`, and `topics/operations/<id>.md` use YAML frontmatter `{schema: topics/v1, record_kind, id, operation_id, data}` with `data` matching the typed JSON above, and human-readable Markdown body. Operation records include ordered target paths, expected hashes/absence, new hashes, dependency IDs and committed status. Only records belonging to a committed operation are visible; startup reconciliation resolves incomplete journals. Proposals and run state may be disposable under root-partitioned Application Support; acceptance records and committed operation metadata are durable in `topics/`. Snapshot manifest and bytes are in root-partitioned Application Support, keyed by operation and target. Rebuild reconstructs current state from committed records; malformed or duplicate IDs are reported and excluded, not silently merged. Undo records compensating operations; if a later record depends on the target, refuse until dependency resolution. For creations, undo removes visibility through a durable tombstone rather than erasing prior bytes.

`topics-core` owns all types and deterministic validation. `topics-store` owns `WorkspaceService` methods corresponding to the commands, file/journal/locking and derived index; existing `TopicsService` root selection remains the source of configuration. `topics-cli` adds `workspace` commands using the same service, particularly list/get/decide/history/undo, for synthetic BDD and recovery; no alternate apply. `topics-bdd` calls services directly. `topics-runtime` (new crate) owns Codex process and app-owned MCP adapter, consumes core types and service trait; no filesystem write or approval capability for model tools. Minimal model tools are `scope_sources {scope_id}` and `propose_changes {candidates:[Candidate]}`; chat receives only its authorized topic context. Both validate root/scope/sensitivity on every call. The server assigns origin/status and validates unique provisional UUIDs supplied by the model; accepted IDs are retained so dependent references remain stable. Reject collisions and model attempts to supply authority. Model cannot invoke `workspace_proposal_decide`. App shell `apps/desktop/src-tauri/src/main.rs` maps named commands and events to these services; `apps/desktop/src/` owns UI and mocked IPC, no domain policy. Integration manager owns root Cargo workspace, lockfiles and shared wiring. Runtime-specific JSON-RPC fields and launch flags are provisional until verified discovery is integrated, not invented here.

Unlabelled sources default to unknown; explicit confirmation permits precisely the previewed unknown sources for one run. Local-only always denies cloud access. Source labels are persisted in source records and topic sensitivity in topic records; evidence from a local-only source taints derived topics and citations local-only until explicit user review. Every model-facing lookup checks both source and topic labels. Topic chat uses a separately previewed bounded context, with the same consent and taint rules.

Verified runtime 0.166.0: initialize then initialized; `account/read` with `refreshToken:false`; `thread/start` uses empty app-owned cwd, `approvalPolicy:never`, `sandbox:read-only`, `ephemeral:true`, `environments:[]`; `turn/start` uses thread ID and text input; interrupt needs thread and turn IDs. Pin version and generated schema, match interleaved request IDs, reject unknown mandatory requests and deny approvals. Stdio MCP was proven only with a synthetic bridge; authenticate and bind bridge capabilities to one scope. Disable inherited servers individually and verify config and paginated inventory, but do not treat flags, empty cwd or read-only mode as sufficient read isolation. Real-note calls require independently tested OS confinement and preflight; fail closed on uncertainty or drift. No credential reads, auth refresh or login implementation is authorized here.

macOS containment discovery: a tested `sandbox-exec` deny of file-read-data and file-write under the canonical root blocks canonical and symlink paths while allowing synthetic Codex/MCP initialization; the MCP service must remain outside that sandbox, reached by a token-scoped sandboxed stdio proxy. A pre-existing hardlink outside the root is not covered by that path restriction. Root switch terminates and relaunches the child with a new canonical-root profile; combine OS containment with verified disabled tools and inventory, and do not claim universal machine isolation. Runtime implementation must verify its exact profile and capability proxy in integration tests before enabling real-note analysis.

## Agreed integration additions (2026-11-03)

The coordinator approved these missing human-interface endpoints without changing scenario semantics: `workspace_manual_propose {root_id,candidates} -> {proposal}` creates a pending proposal through the same service; `workspace_sources_list {root_id} -> {sources:[SourceSummary]}` lists human-visible source inventory, including local-only labels; `workspace_source_get {root_id,source_id} -> ScopedSource` opens evidence for the human without granting cloud consent. They are not model tools. Canonical Rust requests are `ManualProposeRequest`, `RootRequest`, and `SourceGetRequest`; results are `ManualProposeResult`, `SourcesListResult`, and `ScopedSource`. Source-label requests use `Endpoint::Source` with a zero-length locator as a whole-source label marker.

Validated ISO `date: YYYY-MM-DD` YAML frontmatter supplies an indexed date when a supported filename date is absent; filename dates take precedence. Invalid or absent dates remain undated and are excluded visibly under date filters. Modification time never substitutes for document date.

Runtime tool validation uses `WorkspaceService::validate_proposal(StageProposalRequest) -> WorkspaceResult<()>`, which shares the terminal staging policy but publishes no proposal and does not consume the run capability. Human persisted statement inspection uses `workspace_statement_get {root_id,statement_id} -> UserStatement` (`StatementGetRequest`); it is not a model tool and keeps `TopicView` wire-compatible.

The bounded human-only source repair seam is `source_identity_repair(SourceRepairRequest{root_id,source_id,path,expected_hash}) -> SourceSummary`. It journals the mapping change, requires an unambiguous moved original for a rename, refuses a replacement identity with live dependents, and preserves restrictive sensitivity. Old citation paths/hashes remain unchanged and stale until separately reviewed; repair never rewrites a meeting. The CLI exposes it as `workspace source-repair`; desktop UI expansion is outside this integration addition.
