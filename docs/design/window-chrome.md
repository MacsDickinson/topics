# Integrated macOS window chrome

The main window uses the native decorated macOS overlay title bar with its text title hidden. Native close, minimize and full-screen controls remain system controls. The same window geometry and chrome are configured in production and the incognito native-test override. No simulated traffic lights are drawn.

The title bar and navigation share `--color-background-sidebar` in every palette. The 36px fixed bar reserves the first 80px for native controls. Its empty middle accepts dragging. A compact status disclosure sits on the right, outside the drag region. The sidebar starts directly below the bar. The skip link is the first keyboard stop and clears the bar.

The disclosure summarises workspace startup, analysis progress, completion and cancellation, errors and runtime availability. Open it to read a long message, inspect related changes, dismiss a notification or recheck an unavailable runtime. Escape returns focus to the trigger. Clicking outside or tabbing away closes the disclosure. Blocking startup recovery instructions remain beside the folder settings. Form validation and consent stay with the action they concern.

The main-window capability grants `core:window:allow-start-dragging`. The shell places the drag attribute on the empty element.

```tsx
<header className="window-chrome" aria-label="Workspace status">
  <div className="window-chrome-drag" aria-hidden="true" data-tauri-drag-region />
  {/* Status disclosure and its actions */}
</header>
```

## Verification and limits

The focused Playwright checks use mocked synthetic workspace data: both color schemes at 560×420, text selection, horizontal overflow, connected navigation, setup and search focus. Before implementation both theme cases failed because the drag region was absent; after implementation all three cases passed. `npm run build` passed. Browser checks cannot establish native title-bar appearance, movement, native controls or window operations; those require an isolated real macOS window. See the worker report for native results and remaining acceptance gaps.

Tauri's [window customization guide](https://v2.tauri.app/learn/window-customization/) documents drag regions and the minimal drag permission. The installed configuration schema requires decorated overlay windows for the hidden title and retains native decorations.

Native evidence (isolated native-test build, synthetic absolute app-data and fictional note): on the machine's dark appearance, the first-run and setup captures in `apps/desktop/evidence/window-chrome/` show the native traffic lights and app surface. The accessibility tree exposed native close, minimize and full-screen buttons. A foreground drag from window-local (400,15) to (460,55) changed the measured window origin from (166,57) to (226,97); a background synthetic drag did not move it. Entering the synthetic folder path and clicking Use notes folder opened the setup screen. The close-button action ended the test process, and the synthetic source retained its original content; the automation returned an accessibility error after the window closed, so this observation is limited to the resulting process state. No analysis was authorized.

Follow-up native evidence used a separate native-test build with a per-window `Light` theme and initial 560×420 size; system appearance was unchanged. The light minimum and scrolled screenshots show the traffic lights above the app and a visible labelled path field. Accessibility verified selection of `synthetic text` and replacement with the synthetic folder path without moving the window; connecting opened setup, and selecting Search focused its search field. At 560×420 these screens require scrolling; this was not a complete keyboard-only traversal of every setup control.

Foreground dragging the resize corner changed the measured bounds from 560×420 to 760×570 and back. Pressing the native minimize control produced `window_not_found` during the resulting transition; the process remained running, and `get-app-state --restore-window` restored the same window at 760×570 with setup still present. Pressing the native full-screen control changed its measured bounds from 760×570 to 1512×945 and removed the standard window controls; the foreground Control-Command-F shortcut restored 760×570 and its controls. Native close again ended the isolated process with the automation returning `permission_denied` while attempting its post-action accessibility read; the synthetic note remained unchanged. No provider call or analysis authorization was made. Independent cross-provider review remains outstanding.

A follow-up browser regression measured the original desktop sidebar bottom at 804px in an 800px viewport. The fixed header and adjusted sidebar height now pass that bound, and the header remains at viewport top while desktop and narrow content scroll; the desktop sidebar sticks below it. These DOM measurements use synthetic fixtures and do not replace native checks.

The skip-link regression was reproduced at y=12 in both themes at 560px and 1180px. After moving it below the header, the four focused geometry cases and the original first-run keyboard scenario passed in the browser.

A focused follow-up used a fresh isolated light native-test window at 560×420. First Tab focused the skip link, whose visible focus ring cleared the traffic lights (see `native-skip-focus-light.png`); Return focused the main region, then Tab reached the chooser and labelled path field. After accessibility replacement entered a synthetic path, Tab reached the create checkbox and Use notes folder, and Return opened setup. Tab and Return on the skip link then Tab focused the setup root checkbox with a visible ring (`native-setup-focus-light.png`); Command-K focused search, and accessibility replacement of a fictional query produced the synthetic result (`native-search-focus-light.png`). The window did not move during these interactions and the fictional source remained unchanged. Path and query entry used AXReplaceSelection, so this is not proof of a wholly physical-keyboard text-entry flow or exhaustive setup traversal. Close again ended the isolated process while Orca's post-action read returned `permission_denied`; no provider was invoked.
