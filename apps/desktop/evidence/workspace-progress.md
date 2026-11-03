# Desktop workspace delivery evidence

All new desktop work is in `apps/desktop/**`. No push or main merge. Core/runtime commits were imported only for integration; root manifest/lock changes are local coordinator-owned integration metadata.

## Baseline and design

Committed semantic baseline: f3e1ed0b53a7a7611bbf8ad1b00aa0f521f33e91. `da7e551` recorded focused Playwright/native RED before UI behavior: five existing Phase 0 browser tests passed, seven new workspace tests failed; the real native app failed on missing Analyse notes. See workspace-red.md, workspace-red.txt, workspace-native-red.txt. Source inventory and statement-text follow-ups have additional real failing output in workspace-sources-red.txt and workspace-statements-red.txt; proxy startup has workspace-proxy-red.txt.

Quiet macOS workspace: paper #ffffff, sidebar #edf0f4, ink #20252c, secondary #5b6570, line #dce0e5, action/focus #245f9e. System type, left alignment, sidebar/navigation, topic list/detail and cited evidence. No hero/demo data. Review removed negative pane spacing overlapping notices, corrected unknown-versus-local-only guidance, exposed proposed topic sensitivity, and replaced correction IDs with exact persisted statement text/date. Visible keyboard focus, Cmd+K, reduced motion, text selection and narrow desktop layout are covered.

## Passing checks

- `npm run build` with project-local Node 24.22.2: workspace-build.txt.
- `npm test -- --workers=1`: 21/21 browser tests, including all five Phase 0 tests, source text XSS, root setup, actual consent/request shape, edited/subset/rejected decisions, correction diff/history, stale scope/evidence, malformed/duplicate/wrong-root/late streams, root and list/detail races, cancellation failure/retry, automatic selected-topic refresh after approval, accepted empty no-change outcomes, manual missing-runtime fallback, keyboard and narrow layout. Transport is mocked only in browser tests; workspace-green.txt.
- `cargo test -p topics-desktop` with isolated absolute application-data override, debug info disabled: 6/6 tests against actual core/runtime types and services. Validation-only host does not publish; terminal stage remains pending; model has no approval/unselected scope; cancelled capability cannot stage; split tool calls validate one dependency buffer; unrelated/local-only topic inventory never enters scoped context. workspace-rust-tests.txt.
- Warning-denied desktop clippy with all targets/features: workspace-clippy.txt.
- `npm run test:native:workspace`: real embedded WebDriver + Rust IPC, create fresh root, add/read/preview actual synthetic Markdown, persist local-only label, deliberately missing Codex, manual pending proposal, human approval, topic, history and undo, original bytes unchanged; proxy-before-GUI and empty/relative app-data startup checks. workspace-native-green.txt.
- `npm run test:native`: retained Phase 0 real native search, CLI root switch and watcher convergence, plus proxy/startup isolation. workspace-phase0-native.txt.

## Genuine installed-runtime native acceptance

Authorized explicitly through coordinator. Command: `TOPICS_NATIVE_LIVE=1 TOPICS_CODEX_PATH=/Users/macs/.local/bin/codex CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 npm run test:native:live`.

`workspace-native-live.txt` records the complete passing flow through actual Tauri commands, real WorkspaceService and installed Codex 0.166.0 using the packaged desktop executable as its scoped MCP proxy: bounded fictional meeting preview and per-run unknown consent; cited pending discovery with no accepted topics; explicit human approval; accepted Alex and Ledger; topic chat proposes he/him -> they/them with a persisted user statement and scoped rule; explicit human approval; same-source reanalysis introduces no duplicate topic proposals and preserves accepted topic/current revision IDs; fresh native process after deleting the derived SQLite index, real search rebuild, same accepted topic/current revision/history; original source bytes unchanged. Reanalysis context includes only core-authorized, evidenced topic views; unrelated names/IDs and failed context lookup errors are omitted. No fake provider/extraction fallback.

App/WebKit/root/application data remain synthetic. Only an ephemeral wrapper delegates the installed runtime to its own existing HOME/subscription. No test/app credential or personal configuration reads, login changes, config rewrites, note copies, or live root access. Processes, waits and temporary fixtures have bounded cleanup. Transport tracing is not enabled. Native test-only validation diagnostics record only core validation code/message, never frames or credentials.

Earlier attempts are retained honestly: workspace-native-live-initial-failure.txt shows a fail-closed model-tool error with every candidate discarded; workspace-native-live-selector-failure.txt reached genuine accepted discovery/correction then failed on test selector timing before reanalysis. The latter was fixed with paint/enabled waits. Embedded WebDriver lacks native option default-action selection, so the documented DOM selection/change helper preserves real React and every real Rust command. The successful final native flow uses actual provider output throughout.

## Screenshots and production artifact

Selected actual native evidence, visually inspected: native-live-topic.png (current correction, exact user statement and source), native-live-correction.png (old/new/scoped reason awaiting human action), native-live-discovery.png (cited pending review), native-live-restart.png (old revision after native restart/rebuild). native-workspace.png and native-workspace-history.png cover manual fallback/undo. Browser-only evidence is labelled workspace-topic.png, workspace-correction.png, workspace-narrow.png.

Production Tauri bundle: `target/release/bundle/macos/Topics.app`; normal release configuration, no native-test feature or VITE_NATIVE_TEST import. Exact final output is workspace-production-build.txt. Final bundle succeeded at 17.88 MiB with explicit local ad-hoc signing; codesign --verify --deep --strict passes. The initial linker-only signature failed resource verification, so Tauri macOS signingIdentity is now configured as "-". No signing credentials or notarization were requested or changed.

## Integration and remaining review gates

Local implementation commits: da7e551 (RED), 33cc8b6 (React), 5472d43 (real IPC/native). Canonical imported bases: core 1b8ac16 + cc402a6 + b91a0db + cbce881 + 85e9b77 equivalents; runtime c8fcdb0 + 3089600 + f779487 + 7a6e9f5 + a612725 equivalents. Final evidence/fix commit follows the final passing checks; selected-topic refresh has its own regression RED in workspace-current-view-red.txt.

All workspace commands take `{request}` and reject ApiError. Additional human-only `workspace_statement_get` delegates canonical StatementGetRequest. Runtime emits the canonical event envelope and sequence directly. Root/start/cancel use a lifecycle gate; all terminal paths revoke core capabilities; model tools cannot decide/apply. No remaining adapter stub, production mock or invented Rust domain type.

Coordinator must integrate root workspace member/dependency `topics-runtime` and resolved lock metadata. Desktop lock dependencies: serde_json, tauri, tauri-build, optional tauri-plugin-wdio / tauri-plugin-wdio-webdriver, tempfile (dev), topics-core, topics-runtime, topics-store. Runtime final direct libc is required in the lock graph. Full repository checks and independent cross-provider reviews remain coordinator-owned. Core Opus findings about stable identity/rules, privacy classification/aliases, stale proposal/recovery behavior are separate known integration blockers: this passing synthetic path does not dismiss those findings or establish universal runtime confinement. Core fixes and independent UI/runtime review must land before whole-milestone readiness is claimed.
