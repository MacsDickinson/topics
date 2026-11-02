# Phase 0 delivery report

Date: 2026-11-02. Orca Run: `run_dc62697a69ad`.

## Outcome and authorization

The user approved the Phase 0 plan and all 17 existing Gherkin scenarios, then requested the Orca team build. The approved feature files are unchanged. Production implementation `f31bf55` received independent Claude Opus acceptance; the tests/docs-only follow-up `bb88282` received fresh Claude Sonnet acceptance and leaves production behavior unchanged.

Delivered: Rust workspace and CLI; configurable existing/new notes roots; read-only Markdown indexing, person/glossary parsing and SQLite FTS5 search; rebuild and startup reconciliation; metadata-first watcher; Tauri 2 React/TypeScript search window; isolated BDD/browser/native checks; and contributor documentation. Tasks, claims, connection editing, chat and MCP remain future features with separate sign-off.

SQLite is the Phase 0 derived document index. The [assessment](persistence-assessment.md) is a design assessment of future connected-retrieval examples, not an implementation or benchmark of graph queries. Extensible connections remain a product requirement for the later knowledge feature.

## Verification

- Actual red results precede core/UI implementation and accepted defect fixes; logs are linked from [core evidence](phase-0-core-evidence.md), [desktop evidence](phase-0-desktop-evidence.md) and [review-fix evidence](phase-0-review-fixes.md).
- Full `bash scripts/check.sh` at the accepted production commit passed formatting, strict Clippy, 34 Rust tests, all 15 core scenarios/78 steps, four Playwright checks and four real macOS native assertions across three sessions. No skipped or undefined scenarios.
- The final tests/docs delta adds one CLI lock regression and one hostile-markup browser case. All eight CLI integration tests and exactly five Playwright checks passed, alongside formatting, relevant Clippy and the production frontend build. The final Rust suite contains 35 tests. See [final checks](phase-0-final-checks.md).
- An independent Opus cold full/native run passed. Its multi-process probes found no mixed/stale results across 150 CLI switches and thousands of searches; 120 interrupted switches recovered without bad results or hangs. These probes support the focused repository regressions; they are not a universal performance guarantee.
- The new lock regression fails on an isolated no-op-lock mutation. Explicit BDD steps fail closed on undefined input, and snapshots cover files and directories. The hostile-markup case verifies literal rendering, no injected image/script nodes, no execution and working highlighting.
- Real synthetic native screenshots were visually inspected: [search](../apps/desktop/evidence/native-ledger.png), [no matches](../apps/desktop/evidence/native-no-matches.png), [root switch](../apps/desktop/evidence/native-root-switch.png), [watcher update](../apps/desktop/evidence/native-watcher-edit.png) and [startup error](../apps/desktop/evidence/native-startup-relative.png).
- [Linux core CI](../.github/workflows/core.yml) runs the core-only command; [macOS CI](../.github/workflows/desktop.yml) runs the full check. Hosted results are available in the repository's GitHub Actions runs after push.

## Read-only acceptance and confidentiality

The repo's malformed, ignored `notes` symlink was repaired to its existing local target. No note file was edited to repair the link.

The separate real-folder CLI acceptance indexed **26 documents with zero warnings**, using temporary app data that was removed. Only aggregate counts were reported; note contents were not printed, sent to web searches or committed.

The initial before/after snapshot differed. Its detailed comparison was not retained, so its cause is unconfirmed; iCloud hydration or background activity is a possibility, not an established explanation. Two subsequent snapshots without an app invocation were identical. A repeated read-only index then verified all **45 captured source entries** unchanged in paths, content hashes, modes, sizes, modification times and link targets, with no added/removed paths. The acceptance was repeated once specifically to investigate the failed guard. It was not added to the automated suite, whose notes fixtures remain synthetic temporary directories.

One initial reviewer probe violated the temp-only protocol by creating an empty index under real Application Support after clearing HOME. No notes root was configured there, so no notes were read or written; the reviewer verified the index was empty and removed the newly created file/directory. This is disclosed in the preserved [initial review](reviews/phase-0-core-review-original.md). Follow-up checks used explicit isolated data directories or injected resolver inputs. The OS fallback is now guarded and tested without probing personal defaults.

## Team rollup

| Role | Provider/model | Branch or commit | Evidence and review | Next action |
| --- | --- | --- | --- | --- |
| Core foundation | Codex GPT-6.1 Sol, medium | `topics-p0-core`, `e2faa28` (imported as `dab0b31`) | 15 core BDD scenarios; Opus findings fixed in integration | Landed in main with fixes |
| Shared app-data resolver | Codex GPT-6.1 Sol, medium | `MacsDickinson/topics-p0-config`, `335ba24` (imported as `84125eb`) | Isolated CLI/resolver checks; included in Opus review | Landed in main |
| Desktop and integration fixes | Codex GPT-6.1 Sol, medium | `topics-p0-shell`, through `bb88282` | Full/native checks; Opus production acceptance and Sonnet UI/delta acceptance | Landed in main |
| Explicit BDD harness | Codex GPT-6.1 Sol, medium | `MacsDickinson/topics-p0-bdd-refactor`, `1f73e46`/`21001f8` | 15/78 BDD, snapshot tests and undefined-step rejection; reviewed in final integration | Landed as `6a67d4c`/`8650eeb` |
| Core/final production review | Anthropic Claude Opus 5.5 | `0c8bb3a..f31bf55` | Actual diff, scratch full/native run, consistency/recovery probes; **accept** | Settled; report preserved |
| Desktop review | Anthropic Claude Sonnet 5.5 | `0c8bb3a` | Actual IPC/UI/native checks; switching finding fixed | Settled/released |
| Final tests/docs review | Anthropic Claude Sonnet 5.5 | `f31bf55..bb88282` | Actual diff and focused CLI/hostile-markup checks; **accept** | Settled/released |

Sol followed the user's explicit implementation choice. The bundled router was invoked per distinct task but had no TypeSafe key and returned deterministic fallbacks; no key was requested or exposed. Claude cross-provider review was explicitly requested, with Opus used for filesystem/multi-process correctness and Sonnet for bounded UI and test/doc review. Initial Orca readiness attempts failed before delivery; recorded recovery dispatches completed the work without duplicate writers.

All implementation/review Tasks are settled and their release decisions recorded. Orca retained terminals marked external or user-owned; no claim is made that those processes exited. Their committed branches remain preserved. No unfinished implementation worker is being left to continue independently.

## Nonblocking follow-ups

- Before schema evolution, stamp the derived schema version and rebuild on mismatched shape; current corruption recovery does not handle every manually altered table shape.
- Lock waits are unbounded if another process stalls. Scans serialize queries; the 2-second convergence target was verified on the synthetic acceptance fixtures and representative hundreds-of-files probes, not arbitrary folder sizes.
- Literal `<mark>` text can cosmetically resemble highlighting. Source markup is rendered as text; hostile execution is covered by the new browser regression. Watcher-warning UI remains later scope.
- Existing npm development-tool advisories are recorded in desktop evidence; those tools are absent from the production frontend dependency graph.

Phase 1 build work has not started. Its plan and scenarios need the user's sign-off.

## Hosted CI follow-up

After `268b4d8`, hosted [Linux core 41446356310](https://github.com/MacsDickinson/topics/actions/runs/41446356310) passed, while [macOS native 41446356300](https://github.com/MacsDickinson/topics/actions/runs/41446356300) passed its normal session and failed the empty app-data startup session on macOS 26.6.2. The native test harness now has a locally verified synthetic-session/WebKit isolation correction and bounded process/backend diagnostics; all four real native assertions pass locally on macOS 15.5. This is **not yet a confirmed hosted CI fix**: one earlier local disconnect remains unexplained, and the actual hosted WebKit mechanism is unconfirmed. See [CI investigation, evidence and causal limits](phase-0-ci-fix.md). The original Sol worker owns this focused follow-up; production behavior is unchanged, and independent review plus hosted rerun remain required.
