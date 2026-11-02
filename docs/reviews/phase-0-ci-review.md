# Phase 0 native CI harness review

Reviewer: fresh Claude Sonnet 5.5 context, Orca Task `task_7deb754c1b96`, Dispatch `ctx_69c516fac408`. Reviewed `268b4d8..712014f` after the Sol implementation worker was idle. Date: 2026-11-02.

**Disposition: ACCEPT, no blocking findings.** This accepts the test-isolation correction and diagnostic instrumentation; it does not establish the cause of the hosted WebKit failure.

The reviewer inspected the actual native runner, environment helper, process launcher, WDIO config, test-only Tauri overlay, evidence and documentation. Both Node environment checks passed, and syntax checks passed for the three scripts. The reviewer independently confirmed no diff in production Rust/React/config, manifests, lockfiles, workflows, native assertions or approved feature files. The GUI suite was not repeated by the reviewer; the retained implementer run passed normal search/coherence (two assertions), empty-override startup and relative-override startup (one each).

The unchanged startup assertion still requires the app-data resolver error. No retry, skip or timeout change was introduced. The reviewer recorded these nonblocking limitations:

- Several harness settings changed together: prepared HOME, nonpersistent WebKit, log capture and the process wrapper. Log capture itself adds plugin initialization scripts, so a hosted result cannot establish that incognito alone resolved the previous failure.
- The launcher forwards SIGTERM/SIGINT; force-killing it could orphan its child. Process logs append, so separate local runs can mix diagnostic entries. The accepted local run verified all three recorded app processes exited.
- The synthetic relative-path fallback guard assumes the app's working directory is the desktop directory and covers known paths. It is a regression check, not proof that every macOS framework honors HOME.

These limits remain recorded alongside the unexplained first local disconnect in [the investigation](../phase-0-ci-fix.md). Hosted verification is a separate gate.
