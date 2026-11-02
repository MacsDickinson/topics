# Phase 0 — Foundations (draft for sign-off)

## Goal

A working Rust workspace that indexes any notes folder read-only and lets you search it. A bare Tauri window proves the shell builds and talks to the core. Nothing in your notes changes.

## In scope

1. **Toolchain.** Verify the installed Rust/rustup, pin stable in `rust-toolchain.toml`, and set up Node LTS (the personal Mac currently has Node v23.6.0). CI and local development share `scripts/check.sh`, running fmt, clippy, unit tests, cucumber, and front-end tests.
2. **Workspace.** Crates: `topics-core`, `topics-store`, `topics-bdd`, `topics-cli`, plus `apps/desktop` (Tauri 2). `topics-llm` and `topics-mcp` come in later phases.
3. **Notes root setting.** Point the app at any existing folder, or create a new one with the default layout (`memory/people`, `memory/projects`, `Meetings`, `Log`, `topics/`). Stored in the app config under Application Support.
4. **Read-only index** under Application Support. SQLite with FTS5 is the leading candidate, subject to the persistence review below:
   - Every `.md` file as a document (path, hash, modified time, kind guessed from folder: person, project, meeting, log, other).
   - People profiles parsed into Person rows: name, known as, pronouns, role, team. Aliases from the glossary nickname table.
   - Meetings: date and title from the file name.
   - Full-text search (FTS5) over all documents.
5. **Rebuild and watch.** Deleting the database triggers a full rebuild. A file watcher re-indexes changed, added, renamed and deleted files. A hash scan on start catches changes made while the app was closed.
6. **CLI:** `topics init <path>`, `topics use <path>`, `topics index`, `topics search <query>`, `topics people`.
7. **Desktop shell:** a Tauri window with React and TypeScript, a search box that calls the core and lists results. One Playwright test (mocked IPC) and one WebdriverIO smoke test against the real app.
8. **Contributor documentation:** crate responsibilities and dependency direction, a traced UI-to-core search example, and explanations of consequential Rust choices.

## Persistence review before implementation

Markdown remains authoritative. The knowledge model includes extensible typed connections between topics, people, meetings, tasks, claims, documents and source passages, with provenance, review status and relevant dates. The 1:1 example does not define a fixed set of connection types for every user.

Before signing off the persistence implementation, draft and approve synthetic retrieval scenarios for shared attendance, ambiguous identity, a later decision and a correction. Assess SQLite against these queries and rebuild requirements; a dedicated graph engine remains an option if the evidence warrants it. This assessment informs the foundation without pulling full 1:1 preparation, LLM extraction or connection editing into Phase 0. The current Gherkin scenarios are unchanged and remain under review.

## Out of scope

Tasks, claims, corrections, chat, MCP, writes to existing notes. These are Phases 1–6.

## Rules for the build

- Tests use synthetic notes folders in temp directories only. Never the live `~/notes`.
- The index never writes to the notes root. Only `topics init` on an empty or new folder creates files.
- Unparseable files are indexed as plain documents with a warning. They never stop the index.
- Record failing BDD evidence before implementation; completion requires no skipped scenarios.

## Proposed workers

| Worker | Model | Scope |
|---|---|---|
| `p0-core` | GPT-6.1 Sol, medium | Toolchain, workspace, store, index, watcher, CLI, cucumber steps |
| `p0-shell` | GPT-6.1 Sol, medium | Tauri shell, search UI, Playwright + WebdriverIO smoke tests (starts after `p0-core` exposes the search service) |

Reviews: independent Claude contexts review both Codex workstreams. Select the Claude review tier for each task's complexity at dispatch.

## Done when

- All scenarios in `features/phase-0/*.feature` pass with `fail_on_skipped`.
- `scripts/check.sh` is green.
- `topics index` over the configured real notes root (read-only) completes, and I report counts and warnings to you. This is the one run against live data, and it is read-only. Resolve the repository's `notes` symlink; do not assume `~/notes` or the original work-laptop corpus.
