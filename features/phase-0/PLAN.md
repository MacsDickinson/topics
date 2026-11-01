# Phase 0 — Foundations (draft for sign-off)

## Goal

A working Rust workspace that indexes any notes folder read-only and lets you search it. A bare Tauri window proves the shell builds and talks to the core. Nothing in your notes changes.

## In scope

1. **Toolchain.** Install `rustup` (stable, pinned in `rust-toolchain.toml`) and Node LTS. CI script `scripts/check.sh` runs fmt, clippy, unit tests, cucumber, and front-end tests.
2. **Workspace.** Crates: `topics-core`, `topics-store`, `topics-bdd`, `topics-cli`, plus `apps/desktop` (Tauri 2). `topics-llm` and `topics-mcp` come in later phases.
3. **Notes root setting.** Point the app at any existing folder, or create a new one with the default layout (`memory/people`, `memory/projects`, `Meetings`, `Log`, `topics/`). Stored in the app config under Application Support.
4. **Read-only index** in SQLite under Application Support:
   - Every `.md` file as a document (path, hash, modified time, kind guessed from folder: person, project, meeting, log, other).
   - People profiles parsed into Person rows: name, known as, pronouns, role, team. Aliases from the glossary nickname table.
   - Meetings: date and title from the file name.
   - Full-text search (FTS5) over all documents.
5. **Rebuild and watch.** Deleting the database triggers a full rebuild. A file watcher re-indexes changed, added, renamed and deleted files. A hash scan on start catches changes made while the app was closed.
6. **CLI:** `topics init <path>`, `topics use <path>`, `topics index`, `topics search <query>`, `topics people`.
7. **Desktop shell:** a Tauri window with a search box that calls the core and lists results. One Playwright test (mocked IPC) and one WebdriverIO smoke test against the real app.

## Out of scope

Tasks, claims, corrections, chat, MCP, writes to existing notes. These are Phases 1–6.

## Rules for the build

- Tests use synthetic notes folders in temp directories only. Never the live `~/notes`.
- The index never writes to the notes root. Only `topics init` on an empty or new folder creates files.
- Unparseable files are indexed as plain documents with a warning. They never stop the index.

## Proposed workers

| Worker | Model | Scope |
|---|---|---|
| `p0-core` | GPT-6.1 Sol, medium | Toolchain, workspace, store, index, watcher, CLI, cucumber steps |
| `p0-shell` | Opus 5.5, medium | Tauri shell, search UI, Playwright + WebdriverIO smoke tests (starts after `p0-core` exposes the search service) |

Reviews: Opus 5.5 high reviews `p0-core`; GPT-6-Astra high reviews `p0-shell`.

## Done when

- All scenarios in `features/phase-0/*.feature` pass with `fail_on_skipped`.
- `scripts/check.sh` is green.
- `topics index` over your real `~/notes` (read-only) completes, and I report counts and warnings to you. This is the one run against live data, and it is read-only.
