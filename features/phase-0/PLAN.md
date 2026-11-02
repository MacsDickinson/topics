# Phase 0 — Foundations (approved and implemented)

The user approved this plan and all 17 existing feature scenarios on 2026-11-02 and authorized the Orca team to build them. The feature files remain unchanged. See [delivery and verification](../../docs/phase-0-report.md) for the implementation, reviews and read-only acceptance results.

## Goal

A working Rust workspace that indexes any notes folder read-only and lets you search it. A bare Tauri window proves the shell builds and talks to the core. Nothing in your notes changes.

## In scope

1. **Toolchain.** Verify the installed Rust/rustup, pin stable in `rust-toolchain.toml`, and set up Node LTS (the personal Mac currently has Node v23.6.0). CI and local development share `scripts/check.sh`, running fmt, clippy, unit tests, cucumber, and front-end tests.
2. **Workspace.** Crates: `topics-core`, `topics-store`, `topics-bdd`, `topics-cli`, plus `apps/desktop` (Tauri 2). `topics-llm` and `topics-mcp` come in later phases.
3. **Notes root setting.** Point the app at any existing folder, or create a new one with the default layout (`memory/people`, `memory/projects`, `Meetings`, `Log`, `topics/`). Stored in the app config under Application Support.
4. **Read-only index** under Application Support, implemented with disposable SQLite/FTS5:
   - Every `.md` file as a document (path, hash, modified time, kind guessed from folder: person, project, meeting, log, other).
   - People profiles parsed into Person rows: name, known as, pronouns, role, team. Aliases from the glossary nickname table.
   - Meetings: date and title from the file name.
   - Full-text search (FTS5) over all documents.
5. **Rebuild and watch.** Deleting or corrupting the derived database triggers a rebuild. A metadata-first watcher re-indexes changed, added, renamed and deleted files. The first data read after opening the service performs a full hash scan to catch changes made while closed. A missing old root does not prevent choosing a replacement.
6. **CLI:** `topics init <path>`, `topics use <path>`, `topics index`, `topics search <query>`, `topics people`.
7. **Desktop shell:** a Tauri window with React and TypeScript, a search box that calls the core and lists results. One Playwright test (mocked IPC) and one WebdriverIO smoke test against the real app.
8. **Contributor documentation:** crate responsibilities and dependency direction, a traced UI-to-core search example, and explanations of consequential Rust choices.

## Persistence assessment and future sign-off

Markdown remains authoritative. The knowledge model includes extensible typed connections between topics, people, meetings, tasks, claims, documents and source passages, with provenance, review status and relevant dates. The 1:1 example does not define a fixed set of connection types for every user.

The user accepted the file-first graph direction and then approved the existing Phase 0 features for implementation. The [assessment](../../docs/persistence-assessment.md) considers shared attendance, ambiguous identity, a later decision and a correction at design level. Phase 0 implements only the approved document index and search. Executable connected-retrieval scenarios, durable connection formats and any dedicated graph engine decision remain for that feature's own sign-off. Full 1:1 preparation, LLM extraction and connection editing were not added here.

## Out of scope

Tasks, claims, corrections, chat, MCP, writes to existing notes. These are Phases 1–6.

## Rules for the build

- Tests use synthetic notes folders in temp directories only. Never the live `~/notes`.
- The index never writes to the notes root. Only `topics init` on an empty or new folder creates files.
- Unparseable files are indexed as plain documents with a warning. They never stop the index.
- Record failing BDD evidence before implementation; completion requires no skipped scenarios.

## Team and review

| Worker | Model | Scope |
|---|---|---|
| `p0-core` | GPT-6.1 Sol, medium | Toolchain, workspace, store, index, watcher, CLI, cucumber steps |
| `p0-shell` | GPT-6.1 Sol, medium | Tauri shell, search UI, Playwright + WebdriverIO smoke tests; native integration follows the core service contract |

The frontend began independently against the agreed IPC contract. Separate Sol follow-ups handled the shared config resolver and explicit BDD step definitions. Claude Opus 5.5 reviewed core consistency and the final production fixes; fresh Sonnet 5.5 contexts reviewed the UI and final test/documentation delta. All final reviews accepted the implementation. Branches, evidence and known limits are in the delivery report.

## Done when

- All scenarios in `features/phase-0/*.feature` pass with `fail_on_skipped`.
- `scripts/check.sh` is green.
- A separate read-only acceptance index over the configured real root completes and reports counts/warnings. It is not an automated test. Resolve the repository's `notes` symlink; do not assume `~/notes` or the original work-laptop corpus. The delivery report records the initial snapshot discrepancy and the repeated acceptance check used to investigate it.
