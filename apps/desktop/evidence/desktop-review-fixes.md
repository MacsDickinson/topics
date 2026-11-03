# Desktop Sonnet review fixes

Task task_67016a435363, targeted F1–F4 follow-up on 5492d5b. Scope remains apps/desktop; no provider calls, live notes or credential/config reads. Imported core RED/fixes ab7d4f6/f56fb37 as ff07dd7/3f91ee3 for deterministic integration; runtime remains the prior a612725 equivalent while the coordinator handles later privacy/tool-surface review.

## Reproduction and changes

Before editing behavior, desktop-review-red.txt records six focused failures; desktop-review-rulepaths-red.txt independently reproduces the correction-path keyboard failure (the combined alias case stops before reaching rule paths).

- **F1:** `ScopeFields` owns literal string drafts via `ScopeDraft`. Explicit Preview alone parses trimmed nonblank lines into canonical `ScopeSpec`; every keystroke preserves spaces, blank lines and caret position. Proposal aliases and rule paths have separate literal drafts, parsed only by explicit Approve. `pressSequentially` plus Enter regressions exercise spaced folders/files, multiline paths, comma aliases and rule paths; transport assertions verify normalized arrays rather than relying on fill.
- **F2:** Initial `workspace_root_get` failures render a primary workspace alert without waiting for Settings. An unavailable notes folder offers the recovery path form; invalid app-data override shows the original cause and restart instruction, disables the irrelevant folder action and hides misleading runtime retry advice. Native startup checks inspect the initial error without first submitting search. Visually inspected screenshots: desktop-review-startup-empty.png and desktop-review-startup-relative.png.
- **F3:** Preview requires a nonblank explicit file or folder. Dates are optional inclusive filters, with visible undated-exclusion guidance. Dates alone and whitespace-only paths keep Preview disabled, matching the bounded core API.
- **F4:** Proxy dispatch requires exactly the supported first argument and argument count, with a Rust regression. Legacy no-root search directs the user to choose a folder in the app. Source inventory loading is distinct from an empty folder. Include checkboxes identify candidate subject/title/value and index. Analysis and chat scopes/consent are independent; refreshing the same topic after approval preserves the current chat draft and consent, while actual topic/root changes, source/sensitivity changes and explicit send still invalidate the appropriate permission snapshot.

Existing Phase 0 browser fixtures now provide valid workspace startup replies, so search tests exercise successful startup rather than accidentally hiding failed IPC. Their actual search/error/XSS assertions and feature-driven registration are retained.

## Validation

- desktop-review-build.txt: TypeScript and Vite production build pass using pinned local Node 24.22.2.
- desktop-review-green.txt: **29/29 Playwright tests pass**, including original search/race/XSS tests and eight review regressions.
- desktop-review-rust.txt: **8/8 desktop Rust tests pass**, including exact proxy dispatch and real service host boundary checks.
- desktop-review-app-clippy.txt: desktop-only (`--no-deps`) all-target/all-feature clippy with `-D warnings` passes.
- desktop-review-native.txt: real macOS WebView + Rust workspace IPC/manual/sensitivity/history/undo, real typed malformed-request rejection, proxy startup and both invalid-app-data startup cases pass, using isolated synthetic data.
- desktop-review-phase0-native.txt: real search, independent CLI root switch, watcher convergence, proxy/startup isolation pass with synthetic data.
- desktop-review-production.txt: normal production Tauri app rebuilt and ad-hoc signed; `codesign --verify --deep --strict` passes, size **18.02 MiB** at target/release/bundle/macos/Topics.app. Hash: desktop-review-production-sha256.txt.

Full dependency clippy currently fails only in imported core code: persistence.rs:406 `type_complexity` on `journals` return and scopes.rs:299 `unnecessary_get_then_check`. Exact output is retained in desktop-review-clippy.txt and forwarded to the coordinator/core owner; no core file was edited here. Later core/runtime fixes, full integrated review and any future authorized genuine model acceptance remain coordinator-owned. No model rerun was performed in this task.
