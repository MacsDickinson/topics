# Topics — plan (draft for sign-off)

Date: 2026-11-01. Based on the four reports in `docs/research/`.

## What we are building

A local macOS app that is your daily driver for two things:

1. **Context** — topics (people, projects, threads). Each topic shows what the app currently believes, where each fact came from, and lets you correct it via chat.
2. **Tasks** — one persistent backlog with stable IDs, comments and history. Bullet-journal daily log and migration replace the Monday rewrite and the big Friday correction.

## Key findings from your notes

- Tasks have no identity. Each Monday the actions page is rewritten, so age, history and duplicates are lost.
- Transcript errors repeat: pronouns, misheard names ("Talon" for Tallinn), two "Joe"s, invented explanations. "Unverified" appears 67 times with no owner or way to close it.
- Fixes live in prose ("always use he/him"), so nothing applies them on import.
- Live docs drift: `AGENTS.md` is a stale copy of `CLAUDE.md` with a bad find-replace. People pages are months old. Skills use paths that do not match the folders.
- `~/notes` is not under git. There is no history.

## Answer: markdown or knowledge graph?

**Markdown stays. Add a SQLite index behind it. No graph database.**

- Scale is small (~35 people, ~10 projects, hundreds of tasks a year). SQL join tables and FTS5 search cover every query.
- Kuzu is archived. CozoDB looks unmaintained. A graph engine adds risk without a gain at this size.
- What markdown alone cannot do: stable task IDs, claim provenance, corrections that stick. These need structure. They do **not** need a graph.
- We borrow one idea from temporal knowledge graphs: every fact keeps its source, and a correction supersedes a fact instead of deleting it.

**Decided:** markdown is the source of truth for tasks and claims too. One small `.md` file per task and per claim revision, in an app-owned folder inside the notes root (`topics/`). SQLite is a disposable index under Application Support. Delete it and the app rebuilds it.

## Core concepts

| Concept | What it is |
|---|---|
| Topic | Person, project, team or thread. Has aliases (including known mishearings). |
| Source | A meeting file, Slack link, Jira key, chat message or hand edit. Has a reliability level. Transcripts are low. |
| Claim | One fact on a topic, with source, status (proposed, accepted, superseded) and dates. |
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
- **"Sign in with ChatGPT":** the official programme (launched 29 Sep 2026) lists only Plus and Pro plans. Your Codex login is ChatGPT Business (lhvbank.com). The working route: the app runs `codex app-server` and Codex owns the sign-in. You get a "Sign in with ChatGPT" button, and no tokens touch app code. Reusing Codex tokens directly is rejected.
- **Claude** (`claude -p`, LHV Enterprise org). **Decided: both backends ship together; you pick per chat.**
- **Ollama** is for small local jobs only: tag new meetings, pull out candidate tasks.
- Sensitive topics (HR, health, family) can be flagged local-only. The MCP server never sends them to cloud models.

## Tech stack

- **Rust core** (cargo workspace): `topics-core` (domain), `topics-store` (markdown + SQLite + watcher), `topics-llm` (backends), `topics-mcp` (MCP server, `rmcp`), `topics-bdd` (cucumber-rs).
- **UI shell — decided: Tauri 2** with a web front end in the system WebView. Rust owns all logic. UI tests: Playwright against the front end with mocked IPC, plus WebdriverIO embedded-provider smoke tests against the real macOS app.
- **Toolchain:** Rust is not installed on this Mac. Phase 0 installs `rustup` and Node. Full Xcode is not needed.

## BDD process (each feature)

1. I draft the feature plan and Gherkin scenarios with synthetic data.
2. **You sign off** the plan and scenarios.
3. A worker writes failing scenarios, then the code, through core services (the UI calls the same services).
4. A reviewer from the other provider checks it (Claude reviews Codex, Codex reviews Claude).
5. I show you passing results and UI evidence.

CI: `cargo fmt`, `clippy -D warnings`, `cargo test`, cucumber with `fail_on_skipped`, then UI smoke tests. Tests never touch your live notes.

## Delivery phases

Each phase is one feature with its own sign-off.

| # | Feature | You get |
|---|---|---|
| 0 | Foundations: toolchain, workspace, BDD harness, choose or create a notes folder, read-only index of people, projects, meetings and logs. Full-text search. CLI. | Searchable index. Nothing writes to notes. |
| 1 | Backlog + daily log + first UI. Import the 26 Oct actions page as tasks (you confirm matches). Comments, status, migration. | **First daily driver.** Can replace "Monday actions prep". |
| 2 | Topics + claims + corrections. Topic view with sources. Correction rules applied on import. Verify tasks replace "Unverified". | See and fix current understanding. |
| 3 | MCP server. Claude jobs read the backlog, then send proposals into the inbox. | Scheduled jobs feed the app instead of rewriting files. |
| 4 | Chat: Codex app-server (Sign in with ChatGPT) and Claude backends, picker per chat, diff-card approvals. | Correct facts by chat. |
| 5 | Generated views: actions page, people pages, one generator for `CLAUDE.md` + `AGENTS.md`. Retire Monday actions prep. | No more drift. |
| 6 | Import inbox: new meetings auto-tagged locally, corrections applied, claims and tasks proposed for the daily review. | Smaller, daily corrections. |

## Risks

- The Codex app-server is marked experimental. Mitigation: pin the version, add a schema contract test, keep Claude as a fallback.
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
