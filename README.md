# Topics

Topics is a local Markdown notes application. Phase 0 provides a read-only searchable index, people parsing and an isolated notes-root setting. Markdown is authoritative; the SQLite index can be deleted and rebuilt. Later product phases remain in [the plan](docs/plan.md).

Rust is pinned to stable 1.98.1. Install rustup, then run:

```sh
scripts/check.sh --core-only
```

On macOS, `scripts/check.sh` runs the full Rust, browser and real desktop checks. Core CI uses `--core-only`, excluding Tauri from the Linux build. See [contributing](docs/contributing.md) for crate boundaries, safety rules and an example UI-to-service call, and [the delivery report](docs/phase-0-report.md) for reviewed results.

To run the desktop with your chosen existing notes folder:

```sh
cargo run --locked -p topics-cli -- use /absolute/path/to/notes
cd apps/desktop
npm ci
npm run tauri dev
```

CLI and desktop must share the same `TOPICS_APP_DATA_DIR` override, if one is set. Without an override they use the same default app data directory. The desktop installs its pinned Node LTS locally through `npm ci`; no global Node change is required. Press Cmd+K to focus search. More details are in [the desktop README](apps/desktop/README.md).

Try the CLI with a new synthetic temporary folder (never point tests at personal notes):

```sh
sandbox=$(mktemp -d)
export TOPICS_APP_DATA_DIR="$sandbox/app-data"
cargo run -p topics-cli --bin topics -- init "$sandbox/synthetic-notes"
printf '# Synthetic ledger\nLedger migration example\n' > "$sandbox/synthetic-notes/memory/projects/ledger.md"
cargo run -p topics-cli --bin topics -- index
cargo run -p topics-cli --bin topics -- search ledger
cargo run -p topics-cli --bin topics -- people
```

Use `topics use <path>` to select an existing folder. `init` only accepts an empty or new folder and creates `memory/people`, `memory/projects`, `Meetings`, `Log` and `topics`; it refuses a non-empty folder. All commands emit JSON on success and exit nonzero on failure. No Git repository is required inside notes.

Without `TOPICS_APP_DATA_DIR`, the CLI uses the OS application data location (`~/Library/Application Support/com.macsdickinson.topics` on macOS). The setting lives in `config.json`, and the disposable index in `index.sqlite3`. CLI and desktop share `topics_store::application_data_dir() -> Result<PathBuf>`. Windows defaults to roaming application data plus `com.macsdickinson.topics`; Linux uses `$XDG_DATA_HOME/com.macsdickinson.topics` (or `~/.local/share/com.macsdickinson.topics`). Overrides must be non-empty absolute paths; invalid overrides fail without falling back. Tests use isolated temporary overrides or a synthetic home. No notes location is assumed.

[Core BDD evidence](docs/phase-0-core-evidence.md) records the red and green runs. [Persistence assessment](docs/persistence-assessment.md) explains the limited SQLite decision and future graph boundaries.
