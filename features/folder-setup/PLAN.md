# Folder setup and topic-centred review — approved for implementation

Date: 2026-11-09. Status: approved. The user approved this plan and its 21 scenarios with “plans good - LFG”. Implementation follows the BDD RED/GREEN process and independent review below.

## Problem and outcome

User testing of the topic-workspace prototype found two product gaps: connecting a notes folder leaves discovery as a manual file-selection operation, and reviewing individual extracted records is too much work even for a modest corpus. The next milestone should make a folder useful as a whole and make review part of understanding a topic.

Success: connect a folder, inspect its structure, authorize a bounded analysis, explore a sourced draft topic graph, and save a starting map with one explicit action. Begin using that map without having to certify every assertion. Resolve a small number of meaningful questions and correct facts naturally on topic pages. Subsequent imports show changes to understanding rather than another list of database records.

This takes priority over the task backlog based on the latest feedback. Tasks and bullet-journal migration remain planned separately.

## Evidence from the repo

- [Agreed plan](../../docs/plan.md): Markdown authoritative, provenance and extensible graph connections, all LLM writes proposed for human approval, snapshots and undo; future features require plan and BDD sign-off. The earlier overnight exception applied only to the delivered milestone.
- [Original notes research](../../docs/research/notes-analysis.md): few explicit links; context split between profiles, meetings and logs; repeated transcript errors, ambiguous names, stale live docs and copied summaries. Theme pages were sustainable; heavy record-by-record maintenance was not. These are historical findings, not a fresh scan of this machine.
- `apps/desktop/src/main.tsx`: root selection indexes and goes straight to Topics; discovery requires a separate selection and preview.
- `apps/desktop/src/ProposalEditor.tsx`: every topic, claim and connection is an editable row with an inclusion checkbox. It explicitly rejects omitted changes on approval. Topics and their dependencies are presented as work for the user to assemble.
- `topics-core::workspace`: already models topics, cited claim revisions, connections, unresolved identities, pending proposals and human decisions. It does not distinguish accepting a sourced import from verifying a statement as true.
- Existing runtime has bounded individual runs and candidate payloads; folder setup must orchestrate bounded batches rather than raise those limits or send the folder in one prompt.

Planning used repo code and existing reports only. Personal notes were not read, changed or sent anywhere.

## Proposed journey

### 1. Understand this folder

On connecting a previously unseen canonical folder, start a local-only inventory automatically. Show an editable folder tree grouped by likely purpose: people/reference pages, projects, meetings, logs and other notes. Roles are suggestions with an explanation, not facts inferred from a path. Unknown layouts work without forcing a particular notes convention.

Show actual file counts, dated/undated ranges, linked versus unlinked notes, eligible bytes and exclusions. A folder role can be changed in one action; individual file controls are available on demand. Existing app records, source symlinks, hidden/config folders and non-Markdown attachments are excluded from source discovery. Preserve the existing source-access policy rather than widening it silently.

Local inventory may read eligible Markdown locally to identify headings, dates and explicit links; it makes no model call and writes no original note. Existing local-only data can be counted in the human inventory but never sent as a path, heading, excerpt, graph label or classification hint to a cloud model.

A returning folder with accepted knowledge opens its existing map and offers incremental setup/resume. A newly created empty folder explains how to add notes and allows manual capture. Neither repeats an onboarding wizard on every launch.

### 2. Agree what to analyse

Offer the whole eligible folder as a proposed initial selection, with folder-level exclusions and optional history/date filters. No need to type paths. Display the resolved manifest, exclusions and approximate size before the user chooses **Build starting map**. Show the configured runtime and batch/turn limits; do not invent a monetary estimate for subscription usage.

This one authorization covers only the exact reviewed source manifest for this setup job. Each internal batch receives its own existing root/scope/run capability; the job cannot widen scope, override local-only labels or grant durable cloud permission. Source roles and privacy permission remain separate. Changed bytes, newly discovered files and new exclusions invalidate affected unsent scope; show the change and ask for updated transmission consent rather than rereading silently. Completed unaffected batches may be retained.

### 3. Build a draft map

Run a resumable job over bounded batches. Start with profiles/reference documents and explicit links, then incorporate meetings and logs into the same draft graph. The classification order is a retrieval aid, not a rule that a profile is always correct or a recent transcript always wins.

Extract people, projects and threads; cited statements; and typed relationships. Consolidate repeated assertions while retaining their distinct sources. Preserve dates and distinguish historic from current reports. An inferred association is visibly different from an explicit relationship. Do not merge identities by name alone or treat multiple copied summaries as independent corroboration.

Cross-batch topic resolution uses scoped draft context from this job plus eligible accepted context. Provisional IDs must map consistently across batches. Existing corrections and privacy restrictions apply to every batch. Repeated or overlapping runs do not produce a second person or another copy of an accepted fact.

Progress shows files processed, batches remaining, failures and coverage. The draft graph is browsable as batches finish, but no batch silently becomes accepted knowledge. A failed/cancelled model turn never publishes its buffered candidates. The overall job can finish with visible gaps; it must not call itself complete coverage when files failed or were excluded. User-edited drafts and checkpoints survive normal restart and deletion/rebuild of the disposable SQLite index.

### 4. Explore and save a starting map

The main screen is a map of topic cards with a simple graph/list toggle. People, projects and threads are navigable nodes; explicit and inferred connections have different labels/styles. Clicking a connection shows its evidence. The list offers the same information and actions without requiring graph navigation. No graph database migration is needed.

A topic page reads like a briefing: what this is, current reported context, related people/projects, recent changes and open questions. Each assertion can expand to citations and dates; evidence does not fill the whole page by default. A prose summary must map back to its underlying statements. Unsupported summary text cannot become a new fact by being placed in a heading or paragraph.

Offer **Save starting map** with an exact receipt: topics, sourced statements and relationships being saved, and the issues that remain open. This is one explicit human approval of the displayed eligible set, using existing journalled writes and undo. It is not an assertion that the user checked every statement.

No default approval of ambiguous identities, incompatible accounts, unsupported assertions, inferred relationships or replacements of existing accepted facts. Keep those as issues; saving the rest must not require resolving them. Bare endpoint topics needed by an approved relationship are included visibly in the receipt, without including unrelated facts about those topics. If an endpoint is ambiguous, defer the relationship rather than choose a person.

The user can save the whole eligible map or a topic group; the UI handles dependencies. Untouched or deferred material remains available, rather than being rejected because it was not selected. Explicit dismissal is a separate action. Partial approval is a service feature, not a frontend filter over the current reject-everything-else operation.

### 5. Review understanding, not records

Default states are presented in context:

| Label | Meaning |
|---|---|
| From your notes | Sourced material explicitly saved as part of the starting map; not personally verified |
| Confirmed by you | The user explicitly confirmed the identified statement(s), or supplied a correction |
| Needs your input | A contradiction, ambiguous identity or unsupported/inferred assertion requiring a decision |
| Earlier understanding | Superseded material retained with its evidence and history |

These are not per-row completion checkboxes. Saving a baseline never marks its statements confirmed, lifts source sensitivity, or turns transcript evidence into human-authored evidence. A correction is attributed to the user only for the actual assertion they changed, not unrelated facts in the same topic group.

Topic pages offer **Confirm**, **That's wrong**, **Add context** and **Not now** in place. Allow confirming a clearly enumerated section with its count, not unseen assertions elsewhere on the topic. Chat is another input to the same review flow. Show the proposed before/after and exact scope of any future rule. One approval can fix an explicitly previewed group of matching assertions; unresolved or unrelated identities are excluded. Preserve the original notes and superseded history.

The review queue groups questions by underlying problem, not occurrence: one uncertain identity or disputed assertion with all relevant evidence, rather than a separate card per mention. Prioritize contradictions with confirmed understanding, identity questions and new current changes. Do not silently pick truth based on model confidence, document recency or folder role. **Not now** leaves the question open without blocking the useful map; there is no required “reach inbox zero” setup gate.

Subsequent analysis presents topic-level diffs: what changed, what evidence is new and what needs input. Repeated unchanged sources create neither a new fact nor a new question. New facts remain proposed until a human saves the displayed group; background indexing is not permission to send new content or accept changes.

## Domain and persistence implications

Keep approval lifecycle separate from assertion verification. A saved import is approved for storage/use with provenance, while its verification is still “from notes.” Add durable per-revision/per-connection review metadata and an explicit decision receipt identifying the exact saved/confirmed IDs. Do not overload the free-text uncertainty field or silently treat all old accepted records as personally verified. Backfill old accepted records as accepted with no explicit verification; preserve identifiable user corrections without inventing unrelated confirmations.

Draft job manifests, scoped candidates, canonical ID mappings, coverage, user draft edits and checkpoints live in a versioned app-data store separate from the disposable search database. No model output writes authoritative notes before a human decision. Approved knowledge, verification decisions and decision lineage are reconstructible from app-owned Markdown records. Pin exact expected hashes/revisions in save receipts; stale material cannot be confirmed from an old view.

Group saving must support several dependency-consistent pending groups without rejecting the rest. A group may span model batches but publishes through the existing snapshot/journal/commit-marker protocol. No cross-file atomicity claim. If a group exceeds a safe write bound, disclose the operations and their completed/conflict states; never turn a partially applied import into an invisible global success. Undo obeys dependency checks and preserves later independently accepted work.

The model still has only the scoped read/propose surface. The human-facing job coordinator owns grouping, pause/resume and approval. Reading draft/accepted context for later batches must retain scope/privacy enforcement; no full-vault human inventory exposed through MCP.

## Delivery workstreams after sign-off

1. **Core/services and BDD:** draft job/decision contracts, review metadata and migration, cross-batch resolution and grouping, partial save/defer, durable receipts, recovery/undo and projections. Own the canonical API before downstream workers build against it.
2. **Setup job/runtime adapter:** inventory-to-manifest orchestration, bounded capability-bound batches, progress/cancel/resume, budget and error handling; preserve current pinned Codex/MCP containment. Independent after core contracts freeze.
3. **Desktop:** setup tree and consent summary, draft map/list, topic briefings, group save receipts, contextual correction and issues queue. Uses the same services; no persistence logic in React. Independent after contracts freeze.

Use Orca team for build work, synthetic temp notes only, real RED before behavior, and cross-provider review. Manager owns shared manifests, integration, local checks, native genuine-model acceptance and hosted CI. The user has signed off this plan and its scenarios; build work is authorized. A new graph layout dependency or contract migration must be justified in the implementation plan, not assumed now.

## Acceptance and scope

The 21 scenarios in [setup.feature](setup.feature), [review.feature](review.feature) and [continuity.feature](continuity.feature) are approved scenarios, not claims of existing behavior. Bind each to service, browser or native evidence before RED. Add lower-level security/recovery regression tests where the changed boundary requires them; preserve the existing suite.

Use synthetic corpora with an unfamiliar folder layout, at least 100 documents across bounded batches, repeated/copied assertions, two same-name people, a prior correction, historic/current disagreement, a local-only area and failed/changed inputs. Demonstrate initial map → one explicit baseline save → correction from a topic → incremental update → restart/rebuild → same provenance and review state. Measure visible approval actions and model/job coverage; do not claim correctness or responsiveness from a reduced fixture alone.

Acceptance bar: no per-document or per-fact marking required to start using the map; one transmission consent for an unchanged reviewed job manifest; one explicit approval to save its eligible starting map; deferred issues do not block unrelated knowledge; every displayed assertion and relationship retains evidence/trust state; no privacy or human-approval boundary weakened. Record corpus sizes, timings and call counts rather than promise an unmeasured import duration.

Not included: tasks/daily migration, Claude backend, automatic unsupervised cloud import, graph database replacement, global fuzzy identity merging, arbitrary semantic truth verification, rewriting source profiles/meetings or expanding to attachment formats. This is a correction to the setup and review product flow, not approval to trust every extraction automatically.
