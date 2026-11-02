# Topics — agreed plan

Date: 2026-11-01. Revised after plan review on 2026-11-02. Based on the four reports in `docs/research/`; decisions below supersede their earlier recommendations.

Phase 0's plan and 17 feature scenarios were approved on 2026-11-02 with the instruction to start the Orca team. Its implementation and independent Claude reviews are recorded in [the Phase 0 delivery report](phase-0-report.md). Later phases still require their own plan and scenario sign-off.

## What we are building

A local macOS app that is your daily driver for two things:

1. **Context** — topics (people, projects, threads). Each topic shows what the app currently believes, where each fact came from, and lets you correct it via chat.
2. **Tasks** — one persistent backlog with stable IDs, comments and history. Bullet-journal daily log and migration replace the Monday rewrite and the big Friday correction.

## Findings from the original work-laptop notes

These findings describe the original corpus. The personal-laptop notes are essentially a fresh start; do not assume the same files, counts or imports exist here.

- Tasks have no identity. Each Monday the actions page is rewritten, so age, history and duplicates are lost.
- Transcript errors repeat: pronouns, misheard names ("Talon" for Tallinn), two "Joe"s, invented explanations. "Unverified" appears 67 times with no owner or way to close it.
- Fixes live in prose ("always use he/him"), so nothing applies them on import.
- Live docs drift: `AGENTS.md` is a stale copy of `CLAUDE.md` with a bad find-replace. People pages are months old. Skills use paths that do not match the folders.
- `~/notes` is not under git. There is no history.

## Persistence and connected knowledge

**Decided: Markdown is authoritative; Topics models a knowledge graph over it.** Source format, knowledge model and query engine are separate choices.

- Connections can link topics, people, meetings, tasks, claims, documents and identifiable passages within documents. Their types must be extensible to an individual's use cases; attendance and discussion are examples, not a fixed ontology.
- Each connection has stable endpoints, a type, provenance and review status, plus temporal validity when relevant. Document mentions, explicit relationships and inferred relationships remain distinguishable. LLM-inferred connections are proposals requiring approval.
- Facts and relationships keep their sources. Corrections supersede earlier revisions rather than deleting history.
- Use Markdown properties, wikilinks and stable identifiers for Obsidian compatibility. Obsidian can browse and filter the knowledge, but Topics must work without it. App-created records live under `topics/`; source meeting files remain untouched.
- **Phase 0 uses SQLite/FTS5 as a disposable document index.** The [persistence assessment](persistence-assessment.md) records the bounded rationale. This does not fix the engine or durable format for future connected knowledge; a graph model does not itself require a dedicated graph database.

**Decided:** one small `.md` file per task and per claim revision, with durable connection records also in the app-owned `topics/` folder. The derived index lives under Application Support and can be deleted and rebuilt without losing accepted knowledge.

The first retrieval example is 1:1 preparation: find shared meetings, topics and decisions discussed, open actions, and changes since the person's last recorded exposure. Show evidence and uncertainty: attendance suggests possible exposure, not confirmed understanding. Shared attendance, ambiguous identity, a later decision and a correction are design cases in the assessment, not implemented graph queries. The connected-knowledge feature must turn them into executable scenarios for sign-off before its build. Other use cases must be able to introduce different connections without redesigning the core model.

Research precedents: [Basic Memory's file-first graph](https://docs.basicmemory.com/reference/technical-information), [Graphiti's temporal relationships and provenance](https://github.com/getzep/graphiti), [Glean's content, people and activity model](https://docs.glean.com/security/knowledge-graph), and [Obsidian properties](https://obsidian.md/help/properties) and [Bases](https://obsidian.md/help/bases).

## Core concepts

| Concept | What it is |
|---|---|
| Topic | Person, project, team or thread. Has aliases (including known mishearings). |
| Source | A meeting file, Slack link, Jira key, chat message or hand edit. Has a reliability level. Transcripts are low. |
| Claim | One fact on a topic, with source, status (proposed, accepted, superseded) and dates. |
| Connection | An extensible typed relationship between knowledge items or source passages, with evidence, review status and relevant dates. |
| Correction | A rule applied to every future import ("Talon → Tallinn", "Codie: he/him", "'Joe' in Banking Services → ask"). |
| Task | Stable ID, owner, topics, status, due/scheduled date, source, times migrated, comment log. Kinds: task, waiting-on, verify, decision. |
| Log entry | One line in the daily log with a bullet-journal signifier. Can create a task, claim or event. |

Meeting files are never rewritten. Corrections apply on read and in generated pages.

## Daily and weekly rituals

- **Daily (under 10 min):** review proposals from new meetings (accept, edit, reject), then today's tasks (done, migrate, schedule, cancel), then quick capture.
- **Weekly (about 20 min):** migrate every open task older than 7 days. Tasks migrated 3+ times are flagged "decide or drop". Review verify items and stale topics. Generate the actions page and roundup from the store.

## Chat and LLM

- **The app hosts an MCP server.** It is the one tool surface for in-app chat, your Claude scheduled jobs and Codex.
- **Every LLM change is a proposal.** The model calls `propose_changes`. You approve, edit or reject a diff card. Nothing writes without approval.
- **Decided: Codex first, then Claude.** Test the first in-app chat release on the personal laptop with Codex. Claude is required for meaningful work-laptop testing, but does not gate the first personal release. Select the connection per chat once alternatives exist.
- **Installed runtimes:** start with `codex app-server`, which owns its sign-in and credentials; add the unmodified `claude -p` runtime with its own authentication. Never extract or reuse installed applications' session tokens.
- **Direct Sign in with ChatGPT:** a separate feature with its own sign-off, worth testing with the user's personal Pro plan and potentially demonstrating through the OpenAI ambassador network. Official [plan usage](https://developers.openai.com/siwc/token-sharing-open-source) supports open-source and locally hosted apps, with [eligible Plus and Pro users](https://developers.openai.com/siwc/quickstart). Do not assume the same eligibility for work accounts.
- **API connections:** plan for provider API keys, local model endpoints and configurable gateways such as Bifrost and OpenRouter. Connection configuration includes protocol, endpoint, credential reference, model and headers; verify streaming and tool-calling capabilities per integration. Keep credentials in secure OS storage.
- **Two entry points:** external agents drive Topics through MCP; in-app chat uses an installed runtime or an API connection. For direct API connections, Topics owns the conversation/tool loop and invokes the same tool contracts through an internal MCP client. All routes share proposal approval rules.
- **Ollama** is for small local jobs only: tag new meetings, pull out candidate tasks.
- Sensitive topics (HR, health, family) can be flagged local-only. The MCP server never sends them to cloud models.

## Tech stack

- **Rust core** (cargo workspace): `topics-core` (domain), `topics-store` (Markdown + derived index + watcher), `topics-llm` (backends), `topics-mcp` (MCP server, `rmcp`), `topics-bdd` (cucumber-rs).
- **UI shell — decided: Tauri 2, React and TypeScript** in the system WebView. Rust owns domain and persistence logic. UI tests: Playwright against the front end with mocked IPC, plus WebdriverIO embedded-provider smoke tests against the real macOS app.
- **Learning and maintainability:** document crate responsibilities, dependency direction, a traced UI-to-core example and consequential Rust choices. Keep modules focused and BDD scenarios readable as service usage examples.
- **Toolchain:** Rust is pinned to 1.98.1. The desktop pins project-local Node 24.22.2 LTS; the existing global Node installation is unchanged. Full Xcode is not needed.

## BDD process (each feature)

1. I draft the feature plan and Gherkin scenarios with synthetic data.
2. **You sign off** the plan and scenarios.
3. A worker writes failing scenarios, records the failing evidence, then writes the code through core services (the UI calls the same services).
4. A reviewer from the other provider checks it (Claude reviews Codex, Codex reviews Claude).
5. I show you passing results and UI evidence.

CI: `cargo fmt`, `clippy -D warnings`, `cargo test`, cucumber with `fail_on_skipped`, then UI smoke tests. One local check script is also called by CI. Tests never touch your live notes. Build work uses Orca teams with one worker per independent workstream and cross-provider review.

## Delivery phases

Each phase is one feature with its own sign-off.

| # | Feature | You get |
|---|---|---|
| 0 | Foundations: toolchain, workspace, BDD harness, choose or create a notes folder, read-only index of people, projects, meetings and logs. Full-text search. CLI. | Searchable index. Nothing writes to notes. |
| 1 | Backlog + daily log + task UI. Fresh capture and optional import from an existing actions page (you confirm matches). Comments, status, migration. | **First daily driver.** Can replace "Monday actions prep". |
| 2 | Topics + claims + extensible connections + corrections. Topic view with sources. Correction rules applied on import. Verify tasks replace "Unverified". | See and fix current understanding and retrieve connected context. |
| 3 | MCP server. Claude jobs read the backlog, then send proposals into the inbox. | Scheduled jobs feed the app instead of rewriting files. |
| 4 | Chat: Codex app-server first, then Claude for work-laptop testing; diff-card approvals and connection selection. Direct ChatGPT plan usage and API/gateway connections get separate feature plans and sign-off. | Correct facts by chat, first at home and then at work. |
| 5 | Generated views: actions page, people pages, one generator for `CLAUDE.md` + `AGENTS.md`. Retire Monday actions prep. | No more drift. |
| 6 | Import inbox: new meetings auto-tagged locally, corrections applied, claims and tasks proposed for the daily review. | Smaller, daily corrections. |

## Risks

- The Codex app-server is marked experimental. Mitigation: pin the version, add a schema contract test, and add Claude as an alternative in the follow-up release.
- Codex has shell tools. Mitigation: run it in an empty folder with a read-only sandbox, deny approvals, and add a BDD test that checks this.
- Hand edits to generated files in Obsidian. Mitigation: a generated header plus an editable region that is read back.
- Over-structuring killed the February system. Capture must stay one line.
- Employer data policy decides which provider may see the notes.

## Notes folder and safety (decided)

- **No git in the notes folder.** The app works on top of any notes folder. On a fresh install it can create one with the default layout.
- The notes root is a setting. The app never assumes `~/notes`.
- Before any write, the app saves a snapshot of the file's old bytes in Application Support. You can undo any app write from a history view.
- Writes are atomic (temp file then rename) and check the file hash first. On conflict the app keeps both versions for you to review.
- Your employer policy on providers is still yours to confirm before you use chat on work notes.

Smaller items for later phases: the daily ritual time (morning or evening), Jira sync direction, and whether waiting-on tasks share the main backlog.
