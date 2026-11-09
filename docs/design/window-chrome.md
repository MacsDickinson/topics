# Integrated macOS window chrome

The main window uses the native decorated macOS overlay title bar with its text title hidden. Native close, minimize and full-screen controls remain system controls. The same window geometry and chrome are configured in production and the incognito native-test override. No simulated traffic lights are drawn.

The shell places a 28px app-surface header above both sidebar and content. Its first 80px are reserved for native controls; only the remaining empty element has `data-tauri-drag-region`. Keep buttons, links, fields and selectable text outside that element. The main-window capability adds only `core:window:allow-start-dragging`; existing core permissions remain unchanged. The app surface comes from `--color-background-app`, which follows the existing light/dark mappings. At narrow widths the header precedes the stacked navigation and content.

```tsx
<div className="window-chrome" aria-hidden="true">
  <div className="window-chrome-drag" data-tauri-drag-region />
</div>
```

## Verification and limits

The focused Playwright checks use mocked synthetic workspace data: both color schemes at 560×420, text selection, horizontal overflow, connected navigation, setup and search focus. Before implementation both theme cases failed because the drag region was absent; after implementation all three cases passed. `npm run build` passed. Browser checks cannot establish native title-bar appearance, movement, native controls or window operations; those require an isolated real macOS window. See the worker report for native results and remaining acceptance gaps.

Tauri's [window customization guide](https://v2.tauri.app/learn/window-customization/) documents drag regions and the minimal drag permission. The installed configuration schema requires decorated overlay windows for the hidden title and retains native decorations.

Native evidence (isolated native-test build, synthetic absolute app-data and fictional note): on the machine's dark appearance, the first-run and setup captures in `apps/desktop/evidence/window-chrome/` show the native traffic lights and app surface. The accessibility tree exposed native close, minimize and full-screen buttons. A foreground drag from window-local (400,15) to (460,55) changed the measured window origin from (166,57) to (226,97); a background synthetic drag did not move it. Entering the synthetic folder path and clicking Use notes folder opened the setup screen. The close-button action ended the test process, and the synthetic source retained its original content; the automation returned an accessibility error after the window closed, so this observation is limited to the resulting process state. No analysis was authorized.

Native light appearance, minimum-size layout, text selection, resize, minimize/restore and full-screen transitions remain unverified on a real window. Browser evidence does not close those gaps. Independent cross-provider review remains outstanding.
