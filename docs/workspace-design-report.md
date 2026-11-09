# Workspace design prototype

The signed development candidate is at `target/prototype/workspace-design-17cdb27/Topics.app`. Quit the existing Topics app before opening this separate bundle. The existing running prototype and its files were left unchanged.

## Changes

- Integrated macOS overlay header with native traffic lights, a drag region and safe keyboard skip-link placement.
- Actual MIT BoardUI Select and DatePicker sources, adapted to Topics tokens with pinned React Aria dependencies. The calendar supports canonical date entry, Apply, Cancel, Clear and Escape focus return.
- Date-aware oversized-note blocking, explicit exclusion from this run, grouped informational exclusions and actionable empty/reversed-range explanations. A note outside the committed date range no longer blocks the manifest.

Eligible notes over the existing limit still require exclusion. Chunked large-note analysis is a separate proposed plan awaiting approval. Disconnect/reconnect and fresh-session features also remain proposed; they were not included in the window-design authorization.

## Verification

Implementation source `17cdb27fe16f9fa3d97c6f97aae06d59bb588d55` passed the full local `scripts/check.sh`: 346 Rust tests, 55 BDD scenarios / 330 steps, 133 browser tests, native search/startup/workspace/setup, the oversized/date/privacy/source-hash/focus regression, and scripted rehearsal/restart checks. All notes were synthetic with isolated application data; no real provider was invoked. Integrated diagnostics are retained at `/tmp/topics-design-integrated-evidence` and the log at `/tmp/topics-design-integration-check.log`.

The production release bundle was built with `VITE_NATIVE_TEST=0`, without the native-test feature, and passed strict ad-hoc signature verification before and after copying. Its source identity and limits are recorded beside the app. The build reported the existing large JavaScript chunk warning; the scripted rehearsal also reported a local `rust-objcopy` missing-LLVM stripping warning, but completed successfully. No global toolchain changes were made.

Independent Codex reviews accepted the window changes and setup tip `e9c7e9f`; the latter independently replayed the native regression and focused browser checks. Pointer evidence used PID-targeted synthetic input, not physical hardware. No screen-reader acceptance is claimed. See [window chrome](design/window-chrome.md), [setup controls](design/setup-controls.md), and the [team record](../features/workspace-controls/TEAM.md).

## Remaining checks

[Hosted Core passed](https://github.com/MacsDickinson/topics/actions/runs/42722393635) source `17cdb27`. [Hosted Desktop failed](https://github.com/MacsDickinson/topics/actions/runs/42722393676) its light / 1180px first-Tab skip-link assertion, with 132 browser cases passing. A separate worker is investigating the initial keyboard-focus cause; the local result does not supersede that failure.

The test-only follow-up `bd596f7` waits for the disconnected workspace, foregrounds the browser page, verifies initial document/body focus, and then retains the original single Tab, skip-link geometry and Enter assertions. Independent review accepted this ready-foreground test contract; 40 repeated review cases passed, in addition to the implementer's 80 repeated keyboard cases and 15 focused window/redesign cases. A controlled early-input probe showed a plausible timing mechanism, but did not prove the original hosted cause. Product code and the signed candidate remain unchanged. Hosted verification of the revised test is pending.

The user's required Claude review remains blocked by expired CLI OAuth authentication. The additional Codex reviews do not replace that requirement. This is a development candidate with these checks open.
