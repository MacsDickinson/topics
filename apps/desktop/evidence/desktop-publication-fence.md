# Desktop F6 publication fence

Task task_c264421e57fd; only the Rust adapter and its tests changed. The accepted frontend at 1c2070b is untouched. No native launch, provider request, live notes/config/auth access, or other-worktree edits were performed.

## RED and implementation

`desktop-publication-red.txt` records two behavioral failures against the extracted old unfenced order: core staged a proposal before the runtime publication handoff, so rejection or revocation still left pending data visible. Both named tests failed before the adapter change.

Production `HostTools::commit_candidates` now takes the opaque `topics_runtime::CommitPermit` and returns `WorkspaceResult<()>`. Its private `stage_with_publisher` helper checks the bound core run/scope and calls `WorkspaceService::stage_proposal_guarded`; the supplied callback is exactly `permit.publish(vec![proposal.id.clone()], action)`. The adapter has no direct production `stage_proposal` call. Core performs validation, preparation and flushes before the callback; only the core-supplied final visibility action enters the runtime permit fence. Authoritative published IDs and deadline/revocation remain runtime-owned.

Tests use the same private helper with publisher closures; they do not construct, forge or expand access to CommitPermit. Denial and bounded channel-controlled revocation assert no pending proposal in the retained service or after reopening the real service. Core cancellation fails before the publisher can run. Success invokes the final action and checks that the published ID matches the actual pending proposal; topics remain unaccepted. Existing validation, cumulative dependency, scope/privacy and request-shape tests remain intact.

## Checks

- `cargo test -p topics-desktop` with an absolute synthetic TOPICS_APP_DATA_DIR and reduced debug/incremental settings: **10/10 pass**, including both RED regressions (desktop-publication-green.txt).
- `cargo clippy -p topics-desktop --all-targets --all-features --no-deps -- -D warnings`: **passes** (desktop-publication-app-clippy.txt).
- Full dependency warning-denied clippy was attempted and currently fails only in imported runtime code: bridge.rs:173 `collapsible_match` and lib.rs:300 `collapsible_if` (desktop-publication-clippy.txt). Forwarded to the coordinator; no runtime code was edited here.
- `git diff --check`: passes.

Shared commits imported for compilation: f66c5bf -> 2a43177, daf2094 -> ba07259, d3202eb -> c37561a. Canonical signatures match the requested seam; no workaround or replacement domain type was introduced. Existing local root Cargo integration metadata remains coordinator-owned; rmcp client/tokio-stream and runtime sha2 are resolved in the local lock.

Targeted independent adapter confirmation and final integrated native/production bundle acceptance remain coordinator-owned. No redundant UI tests, model requests or rebundle was run in this Rust-only task.
