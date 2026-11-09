# Folder setup development status

2026-11-09. **Implemented and packaged for development; not accepted for delivery.**

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

## Genuine provider acceptance is failing

The real installed pinned Codex 0.166.0 was invoked only with fictional notes, explicit source consent and isolated Topics/WebKit data. Codex alone retained its existing HOME to own authentication; credentials were not inspected or copied. The user's current Codex 0.167.0 executable was not changed.

The first small fixture completed source coverage and produced two topics with a cited relationship, but no claim for the claim-specific correction/context controls. A second fixture failed with zero processed notes and an uninformative generic runtime error. One subsequent exploratory attempt with the same claim fixture failed after 25 seconds, again with zero processed and an unknown category. No full genuine save/correction/context/restart journey has passed, and no initiating cause has been established. Scripted success is not substituted for this gate.

The original diagnostic loss was reproduced with distinct synthetic failures. Runtime now records one of 24 locally generated phase/reason pairs without retaining raw provider messages, source content, credentials, frames or stderr. The store reconstructs only exact allowlisted pairs; malformed, extra-key and mismatched values become unknown. Model-facing denials, tools, validation, privacy, caps, deadlines, retries and publication rules remain unchanged. Tests distinguish actual candidate-schema rejection from store host validation and exercise transport, callback, preflight and deadlines. See `features/folder-setup/FAILURE-ORIGINS.md`. This observes enforcement stages; it does not prove the cause of the earlier genuine failures.

The genuine test harness remains on its separate branch pending independent review and integration. Its restricted observer was updated to retain all 24 safe pairs and fails promptly on terminal failures. Another provider attempt is withheld until that review settles and a concrete bounded follow-up is authorized.

## Remaining gates

Required Claude reviews of the Rust core, runtime/coordinator and desktop are blocked by expired CLI OAuth. Tool-free probes returned that error; the user was asked to run `claude auth login`. A fresh Codex review of the new diagnostics and live harness is additional interim verification, not a replacement for the user-required Claude review.

Remaining work: close review findings; diagnose and fix the real setup failure; demonstrate the full genuine journey; verify final assembled native/browser checks and rebuilt signed app; then commit/push the final source to the personal remote and run hosted Linux/macOS CI. No final delivery, hosted CI or remote source update is claimed here.
