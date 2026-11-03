# Topics

Topics is a local-first macOS app for ongoing people, projects and threads in Markdown notes. The topic-workspace prototype adds reviewed discovery and sourced understanding to the Phase 0 search foundation.

- Analyse selected notes and review proposed topics, claims and connections.
- Accept, edit or reject changes before they become current knowledge.
- Open topics to inspect facts, evidence, uncertainty and revision history.
- Correct facts through topic chat, then approve the replacement and its scoped correction rule.
- Browse sources, set privacy labels, capture topics manually, and undo operations through History.

The persistent task backlog and bullet-journal daily review are the next milestone. See the [agreed plan](docs/plan.md) and [milestone plan and scenarios](features/topic-workspace/PLAN.md) for scope and overnight build authorization.

## Run

The delivered local app is placed at `target/prototype/Topics.app`. Open it from Finder or from this checkout:

```sh
open target/prototype/Topics.app
```

To run from source with the pinned Rust toolchain and project-local Node:

```sh
cd apps/desktop
npm ci
npm run tauri dev
```

Choose or create a notes folder in the app. Use Sources to inspect notes and label private files local-only. Choose Analyse notes, select files or folders, optionally filter by dates, and inspect the preview before confirming a cloud run. Dates filter the explicit selection; undated notes are excluded when dates are specified.

Review changes in Inbox, then open an accepted topic to inspect its understanding or ask for a correction. Chat requires an explicit human Cloud allowed label and confirmed source preview. Unknown-source consent applies to one run and cannot override local-only labels. Model-created topics start unknown.

Chat uses the already signed-in, unmodified **Codex CLI 0.166.0**. Codex owns its credentials; Topics does not read tokens or implement a separate login. `TOPICS_CODEX_PATH` can select an absolute executable; the app also checks standard installation locations. The experimental protocol and tool catalog are pinned. Incompatible or unsupported runtime settings fail with guidance; source browsing, manual capture and accepted knowledge remain usable.

## Data

Original source and meeting notes are not rewritten. Accepted records live in small Markdown files under the notes root's app-owned `topics/` folder, with stable IDs and durable operation metadata. SQLite/FTS is a disposable source index; rebuilding preserves accepted knowledge and history. No Git repository is created inside notes.

App writes retain previous bytes or an absence marker under Application Support. Recoverable operations, hash checks and preserved conflicts protect cooperating writes. History provides compensating undo with dependency checks; retain the app-data snapshots for undo. The [contributor guide](docs/contributing.md) documents formats, recovery and limits.

All cloud calls use one scoped MCP service, through exactly two model tools with no approval/apply capability. Local-only data and unselected evidence are excluded at that boundary. The [runtime investigation](docs/research/codex-effective-tools-discovery.md) records the pinned configuration. The local app is ad-hoc signed.

## Verification

```sh
bash scripts/check.sh --core-only
bash scripts/check.sh
```

The core-only command runs formatting, warning-denied Clippy, Rust tests and both BDD runners. The full macOS command adds browser tests and real native search/workspace suites. Four presentation/protocol scenarios are mapped to their native/runtime owners rather than counted as service BDD passes. Genuine model tests are separate explicit opt-ins; see [desktop verification](apps/desktop/README.md).

Development and automated verification use synthetic temporary notes and an absolute isolated `TOPICS_APP_DATA_DIR`. Never point tests at personal notes or follow the repository's `notes` symlink. Normal app data defaults to `~/Library/Application Support/com.macsdickinson.topics`; CLI and desktop share configuration. Rust is pinned to 1.98.1 and desktop Node to 24.22.2 without changing global Node.

[Phase 0 delivery](docs/phase-0-report.md) remains the foundation record. Milestone evidence and independent findings are retained in `features/topic-workspace/`, `crates/topics-runtime/evidence/`, `apps/desktop/evidence/` and `docs/reviews/`.
