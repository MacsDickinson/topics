# Desktop workspace implementation evidence

Baseline RED: [workspace-red.md](workspace-red.md), committed before behavior in da7e551.

Design: a quiet macOS document workspace with sidebar navigation, topic list/detail and cited evidence, followed by topic chat. Tokens: paper #ffffff, sidebar #edf0f4, ink #20252c, secondary #5b6570, line #dce0e5, action/focus #245f9e. System typography; 26px page titles, 17px section titles, 14px body; left alignment; changes/evidence carry hierarchy instead of repeated decorative cards. Initial review removed hero/card presentation; screenshot review removed negative pane spacing that overlapped notices.

`npm run build` using project-local Node 24.22.2 passes (workspace-build.txt). `npm test -- --workers=1` passes 17 focused browser tests, including all five Phase 0 search tests (workspace-green.txt). Browser tests substitute IPC transport and do not establish real runtime or native readiness. The application has no fake runtime or extraction fallback.

Browser evidence: workspace-topic.png, workspace-correction.png, workspace-history.png. Source-inventory extension has its own pre-implementation failing output in workspace-sources-red.txt, following manager-supplied cc402a6 contract addition.

Coverage: discovery.feature root setup, inclusive date preview/exclusions/consent, missing runtime/manual proposal; review-correction.feature cited edit and approval, reject, omitted dependencies, correction old/new/context and history, stale evidence; runtime.feature cancellation/late/duplicate/malformed/wrong-root events, isolation failure presentation; durability.feature root switch, stale detail reads, undo dependency conflict presentation. Core/runtime native behavior remains a separate integration gate.
