# Folder setup development status

2026-11-09. **Implemented and packaged for development; not accepted for delivery.**

**Latest checkpoint:** the bounded genuine native journey passes, including exact corrected-revision lineage, supplemental context, unchanged source hashes and fresh-process SQLite rebuild. The complete assembled check passed at `8d35973`; the subsequent Finder discovery repair passed its focused tests/review and signed production rebuild at `76107ad`. Hosted Linux/macOS checks passed prototype snapshot `8e7d0c9`; final source CI and user-required Claude reviews remain outstanding. See the [current prototype report](folder-setup-report.md). Earlier checkpoints below retain historical failures and source identities.

The approved milestone adds local notes-folder inventory and role suggestions, exact-manifest consent, bounded resumable analysis, a sourced topic graph/list, group baseline save and topic-centred review. Imported assertions remain “From your notes”. Explicit confirmation, correction and supplemental context are separate operations; added context does not verify facts. Original source notes remain unchanged, and approved app records and receipts live under the root's reserved `topics/` folder.

## Source and local app

Main includes accepted redesign and approval repair `cfbb694`, native test repairs through `1f76d9a`, and expanded local failure origins through `25777fd`. The production bundle currently at `target/prototype/Topics.app` was built from `1f76d9a` without the native-test feature and passed `codesign --verify --deep --strict`. The previous app is retained under `target/prototype/backups/`. `target/prototype/BUILD.txt` records its actual source and limitations. The newer diagnostic patch requires another bundle build before the packaged app can expose those categories. No personal configured app or notes were launched for verification.

## Verification so far

- Assembled formatting, workspace all-target warning-denied Clippy and workspace Rust tests pass, including the expanded failure-origin patch. Latest log: `/tmp/topics-origins-main-check.log`.
- BDD passes 55 service scenarios and 330 steps: foundation 15/78, workspace 20/110, setup 20/142. The setup graph/list presentation scenario has browser/native owners; it is not counted as a service pass.
- An assembled browser run passed 113 tests before the later native-predicate test was added. Subsequent focused checks verify that predicate and current source-sensitivity controls. Browser fixtures do not establish native or provider acceptance.
- Fresh native stages pass search/CLI coherence/startup, workspace/manual approval/privacy/undo, scripted setup of 100 notes across five bounded turns, group save, correction, context, restart/index rebuild and renewed consent. They were completed in stages after failures below; a single full `scripts/check.sh` run has not yet completed green on the final assembled source.
- Independent design review accepted the full redesign, canonical-state integration and a real P1 approval repair. The original bug displayed a correction while submitting a different confirmation receipt after a preview switch. The repaired discriminated state and stale-response guards passed 112 implementation browser checks and an independent 16-check re-review. See `apps/desktop/evidence/redesign/approval-repair.md`.

The first assembled native failure compared the entire notes root and mistook new app-owned inventory/journal records for source changes. Reproduction proved the original Markdown hash unchanged. The repaired guard excludes only the reserved root `topics/` directory, still protects nested directories named topics, and rejects symlinks. A later search test treated an empty-state heading as a search result; it now waits for the exact result title and path. Legacy native selectors were reconciled to controls' actual associated labels/IDs. These failures and subsequent native passes are retained in `apps/desktop/evidence/`.

## Earlier genuine provider failures

The real installed pinned Codex 0.166.0 was invoked only with fictional notes, explicit source consent and isolated Topics/WebKit data. Codex alone retained its existing HOME to own authentication; credentials were not inspected or copied. The user's current Codex 0.167.0 executable was not changed.

The first small fixture completed source coverage and produced two topics with a cited relationship, but no claim for the claim-specific correction/context controls. A second fixture failed with zero processed notes and an uninformative generic runtime error. One subsequent exploratory attempt with the same claim fixture failed after 25 seconds, again with zero processed and an unknown category. No full genuine save/correction/context/restart journey has passed, and no initiating cause has been established. Scripted success is not substituted for this gate.

The original diagnostic loss was reproduced with distinct synthetic failures. Runtime now records one of 24 locally generated phase/reason pairs without retaining raw provider messages, source content, credentials, frames or stderr. The store reconstructs only exact allowlisted pairs; malformed, extra-key and mismatched values become unknown. Model-facing denials, tools, validation, privacy, caps, deadlines, retries and publication rules remain unchanged. Tests distinguish actual candidate-schema rejection from store host validation and exercise transport, callback, preflight and deadlines. See `features/folder-setup/FAILURE-ORIGINS.md`. This observes enforcement stages; it does not prove the cause of the earlier genuine failures.

The genuine test harness remains on its separate branch pending independent review and integration. Its restricted observer was updated to retain all 24 safe pairs and fails promptly on terminal failures. Another provider attempt is withheld until that review settles and a concrete bounded follow-up is authorized.

## Remaining gates

Required Claude reviews of the Rust core, runtime/coordinator and desktop are blocked by expired CLI OAuth. Tool-free probes returned that error; the user was asked to run `claude auth login`. A fresh Codex review of the new diagnostics and live harness is additional interim verification, not a replacement for the user-required Claude review.

Remaining work: close review findings; diagnose and fix the real setup failure; demonstrate the full genuine journey; verify final assembled native/browser checks and rebuilt signed app; then commit/push the final source to the personal remote and run hosted Linux/macOS CI. No final delivery, hosted CI or remote source update is claimed here.

## Subsequent investigation checkpoint

Independent interim review found and closed two P2 issues: valid-shaped arbitrary host error details could spoof diagnostic provenance, and the genuine harness could accept confirmation of a different claim revision. Typed internal error provenance and exact corrected-claim/history/evidence/rule/context comparisons passed re-review, 137 independently rerun Rust tests, six Node tests and both original counterexamples. These are additional Codex reviews; the required Claude review remains blocked.

A later native attempt narrowed its failure to `host/validation_denied`. An actual-host scratch probe then captured a fictional claim with `recorded_at: 2026-11-09`; core requires a full RFC3339 UTC timestamp. Three same-capability offline replays reproduced the precise error. In copied synthetic app data, changing only that field to a valid UTC timestamp passed full core validation without publishing. This establishes the scratch attempt's cause, not the discarded payload of every earlier native failure. A minimal prompt-format clarification and captured-fixture regression are authorized; schema, validator, coercion, privacy and acceptance rules remain unchanged. No further genuine journey is accepted.

Committed source `1cfd6f5568afecb38bbec6dbd11fda8ba9cc9450` is backed up on the personal remote branch `folder-setup-prototype-2026-11-09`. Hosted checks for that exact WIP source are being monitored; no CI result or final acceptance is implied. The newer timestamp fix will need separate integration and validation.

## Passing genuine checkpoint

The timestamp-format guidance and captured-fixture regression passed focused independent review. The first native follow-up extracted and saved a valid baseline, then exposed an asynchronous WebDriver collection bug in the harness. Its repair and explicit scripted rehearsal passed independent review; the rehearsal exercised the exact shared genuine setup/restart specifications without a provider call.

The final genuine run freshly built source `ec9f2e64e800a80679c10d6040bda65b846b168f`, merging main `eda6b8f`. With scripted flags unset, actual installed Codex 0.166.0 processed the same 110-byte synthetic note in one model call under one explicit manifest authorization. The native tests passed baseline approval/From your notes, exact replacement-head supersession, statement provenance, persisted correction rule, no extra confirmation from context, source-byte preservation and fresh-process comparison after SQLite deletion. Evidence is in `apps/desktop/evidence/live-setup-acceptance/REPORT.md`; its earlier failures remain historical, not the current outcome.

Core and Desktop both passed hosted CI at `8e7d0c9` (runs `42563820546` and `42563820387`). This snapshot predates the final harness/default-rehearsal additions and must not be represented as final-head CI. A new complete local check includes the strict scripted rehearsal and helper tests. The current production bundle was built from `2208024`; production source remained unchanged through the passing genuine run. A subsequent Finder usability repair is necessary because normal executable discovery otherwise selects updated Codex 0.167.0 instead of the already installed pinned 0.166.0; it will retain explicit overrides and inspect only known executable metadata. No user Codex installation, credentials or notes are changed.
