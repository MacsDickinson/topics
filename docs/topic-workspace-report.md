# Topic workspace prototype

The prototype adds a context workflow to Phase 0: select notes, preview what will be sent, discover topics and cited claims, review changes, browse current understanding, and correct it through topic chat. Accepted corrections retain the original evidence and exact user statement, supersede the old value, and create a scoped suppression rule for later analysis. Human approval is required for every knowledge change. History supports compensating undo.

A persistent task backlog and the bullet-journal daily/weekly ritual remain the next milestone. This release does not yet replace Monday actions prep. Claude integration, automatic vault ingestion, generated pages and graph visualization are outside this milestone.

## Run the prototype

Open `target/prototype/Topics.app` from Finder, or run `open target/prototype/Topics.app` from the repository. The local production bundle is ad-hoc signed and passes `codesign --verify --deep --strict`; it is not notarized for distribution. Source instructions are in [README](../README.md).

Choose or create a notes folder. Use Sources to inspect files and mark private ones local-only. Analyse selected notes, review the resolved preview, and explicitly confirm transmission. Review proposed topics, claims and connections in Inbox. Open an accepted topic to inspect evidence; explicitly mark it Cloud allowed before using correction chat. Approve the resulting replacement. History displays operations and undo.

Codex CLI **0.166.0** must already be installed and signed in. The unmodified installed runtime owns ChatGPT authentication. Topics never reads tokens or implements its own login. Source browsing, manual capture and accepted knowledge remain available when Codex is unavailable.

## Data and scope

Accepted records are authoritative small Markdown files under the selected notes root's `topics/` directory. SQLite remains a derived index. Journalled snapshots, expected hashes, commit markers, recovery and compensating undo preserve history. No Git repository is created in notes. Original source files were not changed in development; automated and genuine-model verification used fictional temporary notes and isolated app data only.

Runtime discovery proved that read-only execution and disabled shell alone do not isolate a model. The implemented route uses a pinned direct tool catalog, exactly two dynamic model tools forwarded through the official MCP SDK, application-enforced scope and privacy, macOS file sandboxing, environment controls and fail-closed startup checks. The runtime has no approve/apply tool. Its experimental protocol is deliberately version-pinned. See [runtime investigation](research/codex-effective-tools-discovery.md) and [runtime acceptance](reviews/topic-workspace-runtime-accepted.md).

## Validation

Final source: core owner `636fbf2`, integrated as `44a0154`; runtime `2f5268d` and desktop guarded-publication `d42843e` were integrated earlier. Manager integration also covers locked dependencies, check scripts and shared-target native harness paths.

- `bash scripts/check.sh` at the final integrated source: **exit 0**, formatting, all-target warning-denied Clippy, **262 Rust tests passed, zero failed/ignored**, foundation BDD **15 scenarios / 78 steps**, workspace BDD **20 / 110**, **29 Playwright tests**, and native search/root-switch/watch, MCP-proxy entry, workspace approval/history/undo and startup-error suites.
- Four workspace presentation/protocol scenarios are mapped to their owning desktop/runtime evidence; they are not counted as service BDD passes. See [milestone evidence](../features/topic-workspace/PLAN.md), [core evidence](../features/topic-workspace/evidence-core-authority-final.md), and [desktop guide](../apps/desktop/README.md).
- Production build with `VITE_NATIVE_TEST=0 npm run tauri -- build --bundles app`: **exit 0**, approximately 20.3 MiB, strict signature verified. No native-test provider is included in the delivered bundle.
- `TOPICS_NATIVE_LIVE=1 TOPICS_CODEX_PATH=/Users/macs/.local/bin/codex npm run test:native:live` with the isolated shared build target: **exit 0**. Genuine Codex → actual Tauri/core/MCP discovery → human approval → chat correction → approval → repeat analysis → fresh native process → SQLite rebuild retained the same topic and correction; original synthetic note bytes were unchanged. Discovery/correction and restart specs both passed. [Screenshots](../apps/desktop/evidence/native-live-topic.png) and [logs](../features/topic-workspace/evidence/native-live.log) are retained.
- Hosted CI at `51caaa6`: [macOS Desktop passed](https://github.com/MacsDickinson/topics/actions/runs/41602005436); [Linux Core failed](https://github.com/MacsDickinson/topics/actions/runs/41602005434) one MCP negative test using a fixed 40 ms sleep. The test-only correction `883e2cf` replaces its fixed sleep with bounded peer-closure acknowledgement, retains rejection checks and additionally verifies buffer discard. All 49 runtime tests and 50 exact repetitions pass locally. Production bridge bytes are unchanged; hosted verification of this correction follows in the Actions link. Full local logs are retained in [integration evidence](../features/topic-workspace/evidence/checks.log).

During integration, insufficient disk space caused a failed link and left an empty non-executable generated CLI binary. Disposable build caches were cleared, package artifacts rebuilt, and the complete checks rerun successfully. No source/notes data was removed. The earlier failure is not counted as a passing run.

## Independent review

Build work used Orca run `run_826c6adf54d6`: separate Codex implementation streams for core, runtime and desktop, plus plan/discovery workers. Independent Claude reviewed every substantive stream. Source and review branches are preserved; settled workers were released. Model routing preferred Codex for implementation, Sonnet for bounded desktop review, and Opus for consequential core/runtime privacy and durability review. The final Opus choice over a low-confidence routing fallback was recorded in its standalone task.

| Stream | Review result | Evidence |
|---|---|---|
| Core Markdown, proposals, privacy and recovery | Accepted after original and targeted findings were fixed with retained RED/GREEN tests. Final independent confirmation reran 203 core tests, 39 focused cases, workspace BDD and 13 extra probes. | [Original findings](reviews/topic-workspace-core-initial.md), [R1–R5 confirmation](reviews/topic-workspace-core-r1-r5-confirmation.md), [final acceptance](reviews/topic-workspace-core-accepted.md) |
| Codex/MCP runtime, effective tools, cancellation and environment | Accepted; 49 runtime tests, effective-tool negatives and SDK forwarding confirmed. Dotenv autoload is rejected before child spawn. | [Initial findings](reviews/topic-workspace-runtime-initial.md), [final acceptance](reviews/topic-workspace-runtime-accepted.md) |
| Desktop and guarded proposal publication | Accepted; UI input/loading fixes and opaque permit publication reviewed independently. | [Desktop acceptance](reviews/topic-workspace-desktop-final.md) |

The original 31 final reviewer assertions remain unchanged apart from formatting. Two narrower authority failures were reproduced before fixes: substring text could declassify a transcript value, and metadata follow-ups lost genuine corrected-value authority. Final policy requires exact whole trimmed statement/value equality or proven human changed-value lineage; bounded same-value ancestry preserves genuine corrections and fails closed beyond 100 records. See [RED/GREEN record](../features/topic-workspace/evidence-core-authority-final.md).

## Limits and next milestone

Cooperative writes cannot eliminate every race with an external editor; undo needs retained app-data snapshots. Exact suppression rules do not establish semantic truth. Unicode-normalized alias equivalence, changed-content moved copies, arbitrary corpus performance and wholesale loss of authoritative history remain documented limits. A model-declared unresolved identity can omit an alternative from the human inbox; it does not leak that alternative to the model. Human approval of an actual changed value remains the authority boundary, including case changes and explicit returns to earlier values.

The next product milestone should be the persistent task backlog and daily review: stable Markdown tasks linked to topics, capture, comments, status, scheduling and bullet-journal migration, then optional reviewed import of an actions page. That closes the other half of the original brief and makes replacing Monday actions prep concrete. It needs its own plan and BDD sign-off.

The manager independently inspected the final CI correction: the invalid-frame matrix and original poison assertion are retained, while EOF and empty-buffer checks strengthen the oracle. The production prefix matches `51caaa6` byte-for-byte. A further model review and native rebuild were skipped for this test-only synchronization change; hosted Linux and macOS checks remain required. [Root cause and evidence](../crates/topics-runtime/evidence/BRIDGE-CI-SYNCHRONIZATION.md).
