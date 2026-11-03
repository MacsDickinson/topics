# Initial independent desktop review

Claude Sonnet 5.5 reviewed the actual `apps/desktop` diff from `f3e1ed0` through worker commit `5492d5b`, read-only, after the implementer became idle. Verdict: conditional acceptance pending the findings below. Orca Task `task_a46d287b8a03`; report message `msg_d0e8d68b75b5`.

The reviewer reproduced 21 browser tests and seven Rust adapter tests, and ran a separate scratch Playwright typing probe. It inspected actual native screenshots and evidence but did not rerun native or live-provider acceptance. No personal notes, credentials, or configuration were accessed, and no repository files were changed.

## Findings returned to the implementer

- **High:** `Scope.tsx` normalises controlled input on every keystroke. Typing `Team Notes/one.md` produces `TeamNotes/one.md`; typing `a.md`, Enter, then `b.md` produces `a.mdb.md`. Alias and correction-rule path editors have the same delimiter-loss pattern. Existing tests used `fill()` and missed it. Preserve drafts while typing and add real keystroke regressions.
- **Medium:** initial root/configuration errors are hidden unless settings is already open. A bad app-data setting appears as a misleading runtime retry message. Show the primary workspace error and recovery guidance independently.
- **Medium/low:** date-only selection enables Preview although the core requires explicit files or folders. Require a source selection and explain that dates filter it.
- **Low:** tighten proxy argument dispatch; replace CLI-only folder guidance in the GUI; distinguish source loading from empty results; identify each candidate in checkbox labels; preserve the chat draft and scope when refreshing the same topic after approval.

The reviewer found the model tool boundary sound: captured root/scope/run capability, only `scope_sources` and validation-only `propose_changes`, cumulative validation, terminal-only staging, cancellation on terminal paths, human-only inventory/statement APIs, and selected-source association for exported topic context. It confirmed the UI handles explicit human cloud permission and accepted empty proposals as no new changes.

Packaged Finder startup and the signed release artifact were assessed through implementation evidence, not independently launched. The final integrated native run and the separate core/runtime reviews remain required. Fixes and targeted independent re-review will be recorded separately.
