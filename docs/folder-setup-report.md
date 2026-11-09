# Folder setup prototype

2026-11-09. The working development prototype is at `target/prototype/Topics.app`. It includes the accepted redesign, folder setup and topic-centred review. The bounded genuine native journey passes. The user-required Claude reviews remain blocked by expired CLI authentication, so final delivery approval is pending.

## Using it

Open the app and connect a notes folder. Setup inventories its structure locally, suggests folder roles and presents the exact source selection and exclusions before a model run. Review that manifest, then build the starting map. Browse its topics, relationships and cited assertions as a graph or list, and save the whole baseline or a dependency-consistent group in one approval.

Imported understanding is labelled **From your notes**. Import does not require individually marking every assertion reviewed. Confirm important assertions when useful, use **That's wrong** to approve a precise replacement and scoped correction rule, or **Add context** to retain a comment without turning it into a fact or confirmation. Original evidence and revision history remain available. Deferred material stays deferred rather than being rejected as a side effect of saving another group.

Analysis runs in bounded batches and can be paused or resumed. Changed inputs require renewed exact consent while retaining eligible completed work. Selected-source completion and exclusions are reported separately. This milestone provides the starting topic map and manageable review flow; the persistent task backlog and daily bullet-journal workflow follow it.

## Data and runtime

Markdown remains authoritative. Accepted records, revisions, receipts and app-owned inventory metadata live under the notes root's reserved `topics/` directory. Existing source notes are not rewritten. Writes retain snapshots, journal/recovery metadata and compensating undo; SQLite is disposable. Automated work used only synthetic notes and isolated app data, with original source hashes checked. No live notes or repository notes symlink were used.

Every model change remains a proposal for explicit approval. The installed pinned Codex 0.166.0 owns authentication and uses the existing two-tool surface. Finder discovery now checks the exact retained compatible installation before ordinary fallbacks, while explicit executable overrides retain priority. It inspects executable metadata and does not modify the user's current Codex 0.167.0 installation or inspect credentials.

The final signed bundle was built from `76107ad` without the native-test feature and passed `codesign --verify --deep --strict`. Prior apps are preserved under `target/prototype/backups/`; `target/prototype/BUILD.txt` identifies the artifact. No personal configured app was launched during validation.

## Validation

- One complete `bash scripts/check.sh` invocation passed at `8d35973`: formatting, workspace all-target warning-denied Clippy, **344 Rust tests**, **55 BDD scenarios / 330 steps**, canonical TypeScript drift/build checks, **7 observer/lineage helper tests**, **114 browser tests**, native search/startup, workspace, scripted 100-note setup, restart, renewed consent, failure/retry and strict scripted review/rebuild rehearsal. Rust counts use the last outer summary per Cargo test executable and exclude duplicate child summaries.
- The subsequent Finder-only repair passed formatting, desktop Clippy, **13 desktop Rust tests**, and an independent seven-case metadata-discovery review before the signed production rebuild. The earlier complete-check result is not represented as a local rerun after this last patch.
- Actual installed Codex 0.166.0 passed the shared strict native setup and restart specifications on freshly built source `ec9f2e64e800a80679c10d6040bda65b846b168f`. One 110-byte fictional selected note used one manifest authorization and one model call. Tests approved the sourced baseline, checked the exact corrected revision, supersession, source/user-statement evidence and persisted rule, added context without increasing confirmations, verified unchanged source hashes, then compared exact knowledge after a fresh-process SQLite rebuild. This is bounded synthetic acceptance, not a corpus-quality or import-performance benchmark.
- Hosted Linux Core and macOS Desktop passed prototype source `8e7d0c9`: [Core](https://github.com/MacsDickinson/topics/actions/runs/42563820546), [Desktop](https://github.com/MacsDickinson/topics/actions/runs/42563820387). That snapshot predates the final harness/default-rehearsal/Finder additions. A final source push and source-bound hosted verification are separate steps after this report is committed.

Full assembled output is retained in `features/folder-setup/evidence/assembled-final-check.txt`; the Finder check/build output is adjacent. The genuine run, prior failures and explicitly separate scripted rehearsal are recorded in `apps/desktop/evidence/live-setup-acceptance/REPORT.md` and `apps/desktop/evidence/scripted-setup-rehearsal/REPORT.md`.

## Findings resolved

Independent review caught a real approval mismatch: switching previews could leave a correction displayed while submitting another receipt. Unified approval state, stale-response guards and outgoing ID/fingerprint regressions closed it. Other bounded reviews closed diagnostic provenance spoofing and strengthened the genuine harness to require the exact corrected claim lineage through restart.

A real-model rejection was reproduced against the unmodified setup host: the model supplied a date-only `recorded_at` where core requires RFC3339 UTC. Three offline replays hit the same gate; changing that field alone passed full core validation. Explicit format guidance fixed the missing instruction without coercing output or weakening validators. Later genuine extraction and the full native journey passed. Earlier discarded native payloads are not claimed to have the identical defect.

Native verification also exposed test assumptions about app-owned inventory records, empty-state headings, labelled controls, animation-frame selection and WebDriver asynchronous collections. Repairs retain source-byte, privacy, full-selection, approval, undo and identity assertions. Historical RED/failure evidence is preserved.

## Review board and remaining gate

| Stream | Implementation | Independent verification | Required Claude review |
| --- | --- | --- | --- |
| Core/store/BDD | Orca Codex Sol, high | Synthetic service/BDD/regression evidence | Ready; authentication blocked |
| Setup coordinator/runtime | Orca Codex Sol, high | Fresh Sol reviews closed scoped provenance, diagnostics and timestamp findings | Ready; authentication blocked |
| Desktop setup/review | Orca Codex Terra, medium | Browser/native/genuine evidence; scoped state/receipt review | Pending; authentication blocked |
| Structural redesign | Separate Orca design run | Full design, canonical integration and receipt-repair reviews accepted | Included in remaining desktop review |
| Genuine harness/rehearsal | Orca Codex Terra, medium | Fresh Sol reviews, original-counterexample replays and exact native rehearsal accepted | Included in remaining review |

Orca run `run_ee41ef4a4c2c` retains the work and reports. Tool-free Claude probes returned `OAuth session expired and could not be refreshed`. The user must run `claude auth login` to enable the requested cross-provider reviews. No credentials were inspected or copied. Final approval remains open until those reviews and any resulting fixes are complete.
