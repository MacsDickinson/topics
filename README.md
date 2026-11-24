# Topics

Topics opens a local knowledge graph of the people, projects and threads in your Markdown notes. Select a topic or press **⌘K** to find it by name, alias or claim. Follow a typed relationship to its other topic, or switch to **Text view**.

**Analyse sources** opens the local folder inventory and exact manifest review. Analysis produces pending discoveries. Open a discovery to review the proposal or folder job that produced it. Accepted knowledge reopens from disk.

**Needs validation** brings conflicting values, uncertainty and changed citations into the topic reader. Compare the passages, confirm a value, enter a correction or defer the question. Approval records the judgment and preserves earlier evidence. **More tools** contains note search, Inbox, History and the Friday pilot.

An optional [local MCP retrieval server](#local-mcp-retrieval) gives a configured client read-only access to selected approved topics.

Documentation: [How Topics works](https://macsdickinson.github.io/topics/) is the architecture, user guide and feature breakdown. GitHub Actions publishes the validated site from `main` and keeps the generated files on `gh-pages`. See the [docs site guide](docs/site/README.md) for editing, preview and Pages setup. Every change must keep the docs current under [AGENTS.md](AGENTS.md).

UI contributors: follow the [design system](docs/design-system.md) for tokens, components, patterns and review requirements.

## Run

To run from source with the pinned Rust toolchain and project-local Node:

```sh
cd apps/desktop
npm ci
npm run tauri dev
```

Choose or create a notes folder in the app. Use Sources to inspect notes and label private files local-only. Choose Analyse notes, select files or folders, optionally filter by dates, and inspect the preview before confirming a cloud run. Dates filter the explicit selection; undated notes are excluded when dates are specified.

Review changes in Inbox, then open an accepted topic to inspect its understanding or ask for a correction. Chat requires an explicit human Cloud allowed label and confirmed source preview. Unknown-source consent applies to one run and cannot override local-only labels. Model-created topics start unknown.

Chat uses the already signed-in, unmodified **Codex CLI 0.166.0**. Codex owns its credentials; Topics does not read tokens or implement a separate login. `TOPICS_CODEX_PATH` can select an absolute executable; the app also checks standard installation locations. The experimental protocol and tool catalog are pinned. Incompatible or unsupported runtime settings fail with guidance; source browsing, manual capture and accepted knowledge remain usable.

## Local MCP retrieval

Build the executable with `cargo build --release -p topics-cli`. In **Sources → Local MCP access**, enable retrieval for the connected folder. Choose the topic IDs for each client. Use the same application data directory as the desktop. Example stdio client configuration:

```json
{
  "mcpServers": {
    "topics": {
      "command": "/absolute/path/to/target/release/topics",
      "args": ["mcp", "FOLDER_ID", "TOPIC_ID"]
    }
  }
}
```

Replace the placeholders with the IDs displayed in Sources. Append more topic IDs as separate arguments. In a client using a custom desktop data directory, set its `TOPICS_APP_DATA_DIR` environment variable to that same absolute path. The two tools are `topic_search` (query and optional limit) and `topic_detail` (topic_id). The root selection and allowlist come from the process configuration. A new root selection requires an updated folder ID.

A topic must be accepted, explicitly Cloud allowed and eligible under source privacy rules. Retrieved assertions and relationships require current, explicitly Cloud allowed evidence for **every** citation, and relationships require both endpoints in scope. Passage text, paths, pending records, human statements and mutation tools stay outside this interface. The folder switch takes effect on the next call. This server is separate from the model bridge used for a single analysis.

## Data

Original source and meeting notes are not rewritten. Accepted records live in small Markdown files under the notes root's app-owned `topics/` folder, with stable IDs and durable operation metadata. SQLite/FTS is a disposable source index; rebuilding preserves accepted knowledge and history. No Git repository is created inside notes.

App writes retain previous bytes or an absence marker under Application Support. Recoverable operations, hash checks and preserved conflicts protect cooperating writes. History provides compensating undo with dependency checks; retain the app-data snapshots for undo. The [contributor guide](docs/contributing.md) documents formats, recovery and limits.

All cloud calls use one scoped MCP service, through exactly two model tools with no approval/apply capability. Local-only data and unselected evidence are excluded at that boundary. The [runtime investigation](docs/research/codex-effective-tools-discovery.md) records the pinned configuration. The local app is ad-hoc signed.

## Verification

```sh
bash scripts/check.sh --core-only
bash scripts/check.sh
```

The core-only command runs formatting, warning-denied Clippy, Rust tests and three BDD runners. The full macOS command adds browser tests and real native search/workspace/setup suites. Native setup uses a scripted provider with synthetic notes; genuine model tests are separate explicit opt-ins. Presentation/protocol scenarios are mapped to their native/runtime owners rather than counted as service BDD passes. See [desktop verification](apps/desktop/README.md).

Development and automated verification use synthetic temporary notes and an absolute isolated `TOPICS_APP_DATA_DIR`. Never point tests at personal notes or follow the repository's `notes` symlink. Normal app data defaults to `~/Library/Application Support/com.macsdickinson.topics`; CLI and desktop share configuration. Rust is pinned to 1.98.1 and desktop Node to 24.22.2 without changing global Node.

[Topic workspace delivery and validation](docs/topic-workspace-report.md) records this prototype. [Phase 0 delivery](docs/phase-0-report.md) remains the foundation record. Milestone evidence and independent findings are retained in `features/topic-workspace/`, `crates/topics-runtime/evidence/`, `apps/desktop/evidence/` and `docs/reviews/`.
