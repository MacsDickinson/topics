# Storage and knowledge architecture for Topics

Research date: 1 November 2026. Recommendation for the first macOS release.

## Decision

Use **markdown as the human source of truth, plus a disposable SQLite index in Rust**. Add a small, explicit claim-revision model for accepted corrections. Do not introduce a graph database or automatically turn every transcript sentence into a fact.

Use `rusqlite`, bundled SQLite with FTS5, `notify`, and optional `sqlite-vec`. Begin with local embeddings through `fastembed-rs` on CPU. Keep semantic search optional so tasks, browsing and lexical search work without a downloaded model. All durable task state, accepted claims, provenance and correction records must survive deletion of the database.

This combines option (b) with a narrow part of (d). It supports graph-shaped relationships through SQL tables without making the graph the authority. At approximately 525 files and probably thousands of entities, storage capacity is unlikely to be the difficult part. Identity resolution, evidence quality and concurrent editing are the harder problems. This scale judgement is an architectural inference, not a benchmark.

## Requirements and assumptions

The supplied context establishes these requirements:

- Obsidian, Claude skills and the app must share ordinary markdown. Existing meeting and log files remain useful without Topics.
- A displayed fact must explain which file, meeting date and passage support it. “Unverified” statements remain proposals. Repeated summaries do not become independent corroboration.
- Corrections must retain the earlier assertion, the replacement, who accepted it and when. A mistaken transcript and a real change over time are different events.
- Topics, people and tasks have many-to-many relationships. A task persists through daily review, migration, comments and completion.
- Search and chat retrieval must cover the corpus quickly and cite evidence. Corrections must override stale passages during retrieval.
- Scheduled jobs can edit files while the app runs. Edits must not silently disappear, and missed watcher events must not permanently stale the index.
- Data stays local by default. Storage and retrieval must not depend on a chat provider or authentication method.

Assume one user and one Mac initially, with external local writers rather than simultaneous multi-device collaboration. No note contents were inspected or sent externally for this report. The file count and layout come from the brief; latency, corpus size in bytes, hardware and job write behaviour remain unmeasured.

## Options compared

| Option | Strengths | Costs and failure modes | Decision |
| --- | --- | --- | --- |
| (a) Markdown plus frontmatter only | Minimal dependencies; transparent editing, links and git history | Repeated parsing; bespoke search and joins; corrections require conventions; task identity can drift between logs | Suitable for a prototype, insufficient as the complete app architecture |
| (b) Markdown plus derived SQLite | Transactions for index updates; FTS5; vector search; relational links; rebuildable and easy to inspect | Requires ingestion, schema migrations, stale-state handling and disciplined write-back | Recommended foundation |
| (c) Embedded graph database | Natural traversal and graph queries; useful for complex relationship analysis | Extra query language, packaging and maintenance risk; still needs file sync and provenance; graph does not establish truth | Defer |
| (d) Temporal claims layered on (b) | Explicit corrections, historical understanding, evidence and uncertainty | More schema and authoring rules; unconstrained extraction creates noisy contradictory facts | Adopt explicit accepted revisions now; defer automatic temporal extraction |

Many-to-many relationships need join tables, not necessarily a graph engine. SQLite supports recursive queries, including graph traversal, and FTS5 provides ranked text search. Most initial queries are bounded: a topic's tasks, a person's meetings, and evidence supporting a claim. Use joins and one or two relationship expansions first. [SQLite recursive queries](https://sqlite.org/lang_with.html), [FTS5](https://sqlite.org/fts5.html).

### Dependency status in late 2026

Versions below were checked against crates.io's API and upstream repositories on 1 November 2026. “Current” means observed publication/activity, not verified security, compatibility or production quality. Pre-1.0 crates can still be mature; pin dependencies and review changes.

| Component | Observed status | Implication |
| --- | --- | --- |
| `rusqlite` | 0.41.1, published 28 October 2026; repository active | Established SQLite binding; use bundled SQLite for reproducible capabilities. [Crate metadata](https://crates.io/api/v1/crates/rusqlite), [upstream](https://github.com/rusqlite/rusqlite) |
| `notify` | Stable 8.2.1, 25 October 2026; 9.0.0-rc.7 also published | Active; start with stable. Documentation describes editor replacement and platform limitations. Treat notifications as hints. [Crate](https://crates.io/api/v1/crates/notify), [limitations](https://docs.rs/notify/latest/notify/) |
| `sqlite-vec` | Stable-labelled 0.1.9; newer 0.1.10-alpha.4 published 18 May 2026; upstream explicitly warns it is pre-v1 | Useful optional C extension with Rust bindings. Prefer the non-prerelease; avoid alpha ANN features initially. Keep vectors replaceable. [Crate](https://crates.io/api/v1/crates/sqlite-vec), [upstream](https://github.com/asg017/sqlite-vec) |
| Kuzu / `kuzu` | 0.11.3, 10 October 2025; upstream archived that day | Reject for a new dependency. Rust bindings do not remove the risk of an unmaintained C++ engine. [Crate](https://crates.io/api/v1/crates/kuzu), [archive notice](https://github.com/kuzudb/kuzu/) |
| CozoDB / `cozo` | 0.7.6, December 2023; repository not archived, last push observed December 2024 | Attractive embedded Datalog, relational/vector queries and time travel, but maintenance is uncertain. Reject for the first release. An open maintenance question is further evidence, not proof of abandonment. [Crate](https://crates.io/api/v1/crates/cozo), [repository](https://github.com/cozodb/cozo), [issues](https://github.com/cozodb/cozo/issues) |
| Oxigraph / `oxigraph` | 0.5.13, 22 October 2026; active Rust RDF/SPARQL project | Credible if RDF interoperability becomes a requirement. RDF modelling and provenance conventions add work; persistent RocksDB adds native build cost. [Crate](https://crates.io/api/v1/crates/oxigraph), [upstream](https://github.com/oxigraph/oxigraph) |
| SurrealDB / `surrealdb` | 3.3.5, 29 October 2026; 4.0 beta also available; active | Supports embedded Rust and graph/document queries, but adds substantial engine and upgrade scope. Pin stable and review the selected release's licence before adoption; do not assume permissive licensing. [Crate](https://crates.io/api/v1/crates/surrealdb), [embedding](https://surrealdb.com/docs/reference/rust/embedding), [licence](https://github.com/surrealdb/surrealdb/blob/main/LICENSE) |
| `petgraph` | 0.8.3, September 2025; repository active in October 2026 | Mature graph data structures and algorithms, not a transactional database. Suitable for a disposable in-memory view later. [Crate](https://crates.io/api/v1/crates/petgraph), [upstream](https://github.com/petgraph/petgraph) |

Do not equate `sqlite-vec` with a pure Rust implementation, or `petgraph` with durable graph storage. SQLite and optional native inference libraries still require macOS packaging validation.

### Local embeddings

| Option | Status checked on research date | Choice |
| --- | --- | --- |
| `fastembed-rs` (`fastembed`) | 7.5.0, 23 October 2026; active; integrates tokenisation and ONNX inference via `ort` | Best first integration. Bundle or explicitly download a pinned, licensed small model; CPU baseline. [Crate](https://crates.io/api/v1/crates/fastembed), [upstream](https://github.com/anush008/fastembed-rs) |
| Candle | `candle-core` 0.11.1, 31 October 2026; active Rust ML framework with Metal support | Useful for Rust-controlled acceleration; more responsibility for architecture, pooling, tokenisation and model parity. [Crate](https://crates.io/api/v1/crates/candle-core), [upstream](https://github.com/huggingface/candle) |
| `ort` directly | 2.0.0-rc.15, 19 October 2026; current 2.x remains a release candidate | Flexible ONNX integration but more model plumbing. Pin the exact wrapper/runtime combination, including fastembed's transitive runtime. [Crate](https://crates.io/api/v1/crates/ort), [upstream](https://github.com/pykeio/ort) |
| Apple MLX sidecar | Upstream active on 1 November 2026; official APIs include Python, C++, C and Swift | Apple silicon option through a bundled Swift/Python helper over stdio. Adds process lifecycle, distribution and model integration work; not the initial dependency. [Upstream](https://github.com/ml-explore/mlx) |

No model quality, acceleration or battery benchmark was run. Evaluate candidate models using representative local retrieval questions before choosing one. Record model checksum, dimensions, normalisation and chunker version. Changing any of these creates a new embedding generation; never mix incompatible vectors. Model downloads must not upload notes.

## Recommended data contract

### Markdown ownership and identifiers

Preserve existing `Meetings/` and `Log/` history. Keep topic and person pages under `memory/`, with human prose and links. Add `memory/tasks/` and `memory/claims/YYYY.MM/` for durable records. These are proposed future paths, not changes made by this research.

Use UUIDv7 identifiers for app-created documents, tasks, logical claims and revisions. Titles and paths are mutable labels, never identities. Common frontmatter:

```yaml
schema: topics/v1
id: "<document UUID>"
kind: topic # person, task, claim_revision, journal
created_at: "2026-11-01T10:00:00Z"
updated_at: "2026-11-01T10:00:00Z"
aliases: []
related_ids: []
```

Existing files without frontmatter remain valid. A git-tracked markdown catalogue under `memory/catalogue/` maps legacy document IDs to paths and content fingerprints, without rewriting every transcript. Exact rename detection can use matching hashes; a simultaneous rename and edit may require confirmation. Duplicate IDs are errors, not an instruction to merge people. Keep readable Obsidian links alongside IDs and repair paths after recognised renames. Unknown frontmatter and formatting survive app edits.

### Tasks and daily bullet journal

Use **one file per persistent task**, with a single canonical `- [ ]` or `- [x]` action line and timestamped comments below. Frontmatter adds `status` (`open`, `scheduled`, `done`, `cancelled`), `topic_ids`, `person_ids`, nullable `due_on`/`scheduled_on`, `source_refs` and `completed_at`. Validate checkbox/status consistency; external inconsistent edits need resolution.

Daily logs record reflection, completion and migration events with links to task IDs. They do not copy editable backlog checkboxes. Postponing a task updates its schedule and appends a dated migration comment; it does not create a replacement task. Topic pages link to task files rather than own separate task records. Import historical actions as proposals, with source/block references, to avoid silently duplicating the backlog.

### Claims, provenance and correction

Only promote useful explicit claims: accepted decisions, ownership, identities and corrected facts. Raw transcript passages remain evidence. Each immutable markdown claim revision contains:

- Document/revision ID, stable `claim_id`, subject ID, predicate and object ID or literal.
- Acceptance state and actor; `recorded_at` supplied by the app; nullable `valid_from`/`valid_to` plus date precision.
- `supersedes` revision IDs, reason and correction kind (`error` or `world_change`).
- Source references: document ID, path hint, meeting date, heading/block, passage hash and a short evidence excerpt. Include extractor/model metadata for proposals.

The body explains the assertion in plain English. A new revision file records the replacement and retains the previous assertion in its correction explanation; the old revision remains. User corrections can cite the chat acceptance as evidence without pretending a meeting said the corrected text. Ambiguous dates stay unknown.

Current knowledge is the accepted, unconflicted head of each revision chain. Two concurrent replacements produce a visible conflict, not last-write-wins. A resolution revision names both parents. Transaction time answers “what did the app believe then?”; validity time answers “when was this true?”. A correction of an old mistake retracts that belief; an actual role change closes the former real-world interval. Preserve both questions in the index.

This borrows Graphiti/Zep's separation of recorded and real-world time. Graphiti itself is an active Python framework with additional database/LLM dependencies, not a drop-in Rust embedded library. Borrow the semantics without adopting its runtime or automatic LLM invalidation. [Temporal model](https://www.getzep.com/blog/beyond-static-knowledge-graphs/), [Graphiti](https://github.com/getzep/graphiti).

Correction flow: resolve the target claim and evidence; accept the replacement; durably create its revision file; ingest it; invalidate dependent summaries and retrieval caches; refresh topic understanding. Default retrieval prefers accepted current claims and includes relevant correction context whenever old supporting passages are retrieved. Superseded assertions remain available in history mode, not as current facts. Search still exposes original evidence, clearly labelled.

Do not silently rewrite historical meetings. Offer an explicit annotation or a reviewed patch if useful. A patch retains the correction record and original evidence. Topic summary blocks may be refreshed separately and carry their input revision IDs; they are derived views. Human prose remains editable; contradictory prose is flagged rather than silently overwritten.

### SQLite index and retrieval

Store the database outside the notes repository, under macOS Application Support. Derived tables cover documents/hashes, chunks, entities/aliases, typed links, task state/comments, claim revisions, evidence and current-claim projections. Add FTS5 and optional vector tables. Never store the sole copy of an accepted correction, task comment or identity mapping there.

Parse incrementally by content hash. Update each document's rows and FTS entries in one transaction. Remove deleted-file rows; mark missing evidence unresolved. Preserve a stale warning if a new version fails parsing rather than presenting old rows as current. Build full replacements in a separate database and swap after validation. One writer queue and WAL allow reads during indexing; SQLite still has one writer at a time. [WAL concurrency](https://sqlite.org/wal.html).

Chunk by heading and bounded paragraphs, retaining offsets, heading context and document dates. Combine FTS5/BM25 with vector results using rank fusion, then expand linked people/topics and deduplicate source passages. Include current accepted claims and their corrections before conflicting transcript text. Recheck file hashes before constructing chat evidence. Cite source paths, dates and locators; show uncertainty and abstain where evidence conflicts.

Exact vector search is a reasonable first experiment. For illustration, 10,000 float32 vectors of 384 dimensions occupy about 15.4 MB before metadata/index overhead. This is an estimate, not the measured corpus. Measure retrieval quality and latency before considering ANN or a different database.

### External writers, recovery and backup

Watch the notes root with `notify`, debounce bursts and rescan affected directories. Reconcile hashes on launch, wake, watcher errors and periodically. Watch directories because editors may replace files. Files are authoritative; watcher delivery is not a change log.

For app writes, capture the original bytes/hash, calculate a narrow patch, reread, and three-way merge only disjoint edits. Conflicting edits preserve both versions for review. Write through a same-directory temporary file, flush and atomically rename; acknowledge success only after durable markdown exists. Reindex from that file, and recognise self-generated hashes without ignoring later external edits.

**Hash checks plus rename cannot guarantee freedom from lost edits with uncooperative writers**: another process can write between the check and replacement. Standardise a shared per-file advisory lock and expected-hash write helper for Claude jobs and app writes. Alternatively, have jobs create unique proposal files which the app accepts. Obsidian cannot be assumed to honour this protocol; avoid automatic rewrites of actively edited human pages, retain pre-write snapshots and expose detected races. Never claim SQLite transactions solve filesystem concurrency.

Unique immutable correction files reduce write contention. Single-file task edits avoid multi-file commits. Commit the authoritative record first; journal/summary updates are retryable projections identified by operation ID. A crash after the record write is repaired by reconciliation. No atomicity across unrelated markdown files is assumed.

Git-track markdown, catalogue and claim revisions; exclude database, WAL, vectors and model cache. Make small labelled commits after settled writes, without staging unrelated work or auto-resolving conflicts. Git preserves committed history, not every keystroke, and is not an off-device backup. Retain local snapshots and use a user-approved encrypted backup destination. Rebuild the index after checkout or restore.

## Risks, migration and remaining decisions

The main risks are inconsistent external edits, entity conflation, stale summaries, noisy extraction, model/runtime packaging and audit gaps from edits made while the app is closed. Immutable records are a convention, not tamper-proof evidence. Git and validation reveal many changes but cannot recover an unobserved, uncommitted overwrite. Dates and aliases need explicit human resolution.

Ship in stages: index existing markdown and lexical search; add persistent tasks and daily review; add explicit correction revisions; then embeddings and narrowly scoped extraction proposals. Keep relationship and retrieval interfaces independent of the database implementation. BDD acceptance scenarios should cover rebuild after deleting SQLite, rename preservation, crash recovery, concurrent task edits, competing corrections and stale transcript retrieval; no code tests were required for this research.

If measured needs grow, improve chunking, SQL indexes and retrieval evaluation first. Add `petgraph` for algorithms or a visual graph without changing storage. Consider a derived Oxigraph projection for RDF/SPARQL requirements, or another maintained graph engine for frequent deep traversals that SQL demonstrably handles poorly. Export stable IDs, edges, evidence and temporal intervals from markdown/SQLite, run both projections in parallel, and compare answers before switching. A larger node count alone is not a migration trigger.

User decisions still needed during product discovery: whether jobs can adopt the shared write protocol; whether historical corrections should annotate source files; preferred daily migration notation; supported Mac hardware and offline model budget; and any future multi-device requirement. None changes the initial recommendation, but multi-device concurrent writing would require a separate sync design.
