# Runtime implementation evidence

Baseline: `f3e1ed0` and the approved-in-progress feature contract. Canonical core
baseline `1b8ac16` was cherry-picked as `77962e9`; no duplicate Rust domain types
were introduced. Root workspace/lock changes remain uncommitted for manager
integration. Runtime commits before the final hardening patch are `c8fcdb0`,
`3089600`, `f779487`, `7a6e9f5`.

## RED provenance

- `c8fcdb0` committed seven named acceptance tests before runtime implementation;
  `red-baseline.log` captures compile RED for missing runtime modules after local
  workspace registration. The first attempt also identified missing membership.
- `red-semantic.log` captures two behavior failures with a deliberately permissive
  stub authorization gate before implementing that gate/SDK bridge. The independent
  protocol state and native confinement implementation were already passing at
  this point; this is not claimed as behavioral RED for those implementations.
- `live-initial.log` records genuine runtime/model RED: successful terminal with
  zero selected reads/proposals was explicitly rejected by live acceptance.
- `red-tool-error-buffer.log` records a later semantic safety refinement before
  its fix: an authenticated denied tool call after a valid buffer could previously
  leave that earlier buffer publishable. The fix poisons the whole run and clears
  every candidate, including on malformed/unknown authenticated MCP frames.

## Production path and root causes

`Codex 0.166.0` runs under the static parameterized macOS root denial. The tiny
sandbox-inherited proxy forwards bounded stdio MCP over an authenticated local
channel to the official `rmcp = 3.8.0` server in the unsandboxed parent. Every
connection has a run capability and a fresh replay-checked nonce. All sessions
share only their own immutable host scope; caller tool arguments cannot choose
root, run, identity or authority. There is no dynamicTools or alternate tool path.

The first live inventory failure exposed a pinned Codex implementation detail:
unfiltered `mcpServerStatus/list` creates a separate catalog MCP connection even
with `threadId`; the official source reuses a connection only for
`threadId + serverName`. The bridge therefore permits up to eight authenticated
connections for the same run (24 bootstrap attempts), rather than ignoring an
inventory error. Complete unfiltered pagination and exact schemas remain checked.

The no-tool model RED was caused by absent MCP safety annotations: under approval
never, Codex regarded those tools as requiring approval. Both fixed tools only
read or validate/buffer within the process; they now declare readOnlyHint true,
destructiveHint false and openWorldHint false. Topics approval mode is explicitly
auto; the runtime still rejects every runtime-initiated approval/elicitation.
Only terminal success can invoke host staging, and staging cannot accept knowledge.

A final lifecycle test exposed the importance of Tokio's uncancellable blocking
stdin reader. The proxy uses `shutdown_background` when the endpoint/deadline
closes, so it exits while Codex still holds stdin open. App-server children have a
separate process group, bounded interrupt/terminal wait, group termination and
explicit reaping; stderr is drained without retaining content.

## Checks and scenario mapping

Final commands (run locally on macOS, exclusively synthetic temp roots/app-data):

```
cargo fmt -p topics-runtime --check
cargo clippy -p topics-runtime --all-targets --all-features -- -D warnings
cargo test -p topics-runtime
cargo test -p topics-runtime --test live_codex --test schema_pin --features native-live-tests -- --nocapture
```

The deterministic command runs 22 tests: eight protocol/SDK unit tests, eight
acceptance tests including the native OS preflight, five full scripted Runtime API
tests, and one proxy lifecycle test. No ignored tests. Explicit native live command
runs two actual existing-ChatGPT-subscription tests and one schema-generator test;
no API-key backend, login, token reads, bought API use or live notes are involved.

| Required behavior | Evidence |
|---|---|
| Tool boundary rejects unselected/local-only/derived evidence and private errors | fixed argument gate; SDK tests use private source IDs and generic sanitized errors; host validator contract; full store adapter belongs to desktop/core |
| Unexpected tools or isolation failure prevents selected transmission | full Runtime scripted test asserts no turn/start marker, zero host reads/stages; independent native preflight |
| Cancel and late model events cannot create proposal | real SDK-buffered candidate, cancel, denied late call and late completed; one cancelled terminal, zero stages |
| Malformed model output rejected | canonical Candidate parsing, host validation, malformed app-server/MCP tests; previous valid buffer is discarded after denied/invalid request |
| Unknown status/exit/wrong-turn/duplicate terminal | full Runtime fake-process tests; zero staging on failures and exactly one staging on successful duplicate completion |
| Native root read/write denial | canonical + symlink read, existing-file overwrite + creation denial; single/double quotes, Unicode and parentheses passed separately via -D; sibling read/write allowed |
| Genuine discovery | Codex model invokes production MCP; one scoped read, three cited person/project/leads candidates, one pending proposal event; source unchanged |
| Genuine correction | separate Codex turn; persisted user statement, current revision and topic supplied; one typed correction with user-statement evidence, supersedes and scoped pronouns rule; source unchanged |
| Adversarial synthetic model | attempts requested for built-ins/inherited tools/private references; random private sentinel not returned; zero staged changes; source bytes unchanged |
| SDK/version proof | actual rmcp 3.8.0 MCP 2025-06-18 negotiation with installed Codex; committed 16-method schema projection compared to pinned generator; wrong installed version actionable |
| Capability lifetime | wrong session/token, expired token, replayed bootstrap nonce denied; endpoint disabled on close; no cross-run host lookup |

Sanitized logs: `deterministic-green.log`, `native-live-green.log`, and the earlier
`live-green.log`. They contain counts/statuses rather than source bytes, raw model
text, account metadata, effective config values or capabilities.

## Exact Rust integration API

```rust
pub trait HostTools: Send + Sync + 'static {
    fn call(&self, tool: &str, arguments: serde_json::Value)
        -> topics_core::workspace::WorkspaceResult<serde_json::Value>;
    fn commit_candidates(&self, candidates: Vec<topics_core::workspace::Candidate>)
        -> topics_core::workspace::WorkspaceResult<Vec<String>>;
}
pub struct RuntimeConfig { pub codex_path: PathBuf, pub proxy_path: PathBuf, pub app_data: PathBuf }
pub struct RunRequest { pub root_id: String, pub run_id: String, pub scope_id: String, pub notes_root: PathBuf, pub prompt: String }
pub type EventSink = Arc<dyn Fn(WorkspaceRunEvent) + Send + Sync>;
Runtime::new(RuntimeConfig) -> Runtime;
Runtime::status(&self) -> RuntimeStatusResult;
Runtime::analysis_start(&self, RunRequest, Arc<dyn HostTools>, EventSink) -> WorkspaceResult<String>;
Runtime::chat_start(&self, RunRequest, Arc<dyn HostTools>, EventSink) -> WorkspaceResult<String>;
Runtime::cancel(&self, root_id: &str, run_id: &str) -> bool;
pub fn run_stdio_proxy() -> i32;
```

The host captures core-assigned IDs. `call(propose_changes)` uses the core's
nonmutating validator; `commit_candidates` revalidates root/scope/run/revisions and
stages once after terminal success. The desktop must invalidate the store run on
cancel/error/root switch and use its own executable's early `--topics-mcp-proxy`
dispatch before Tauri startup. No runtime-to-store dependency exists.

## Remaining integration work and limits

Manager owns root `Cargo.toml` membership/path dependency, Cargo.lock resolution,
main integration, personal remote push and independent Claude review. The runtime
crate additionally depends directly on libc for Unix process-group teardown.
Desktop/core acceptance exercises their real HostTools adapter and full human
approval/persistence; runtime live fixtures prove the real installed model and
production bridge, not the store's approval engine.

Native proof is for this tested macOS/filesystem/profile. Linux native confinement
is explicitly unavailable; normal tests use platform gates, but a Linux runner was
not available here and no Linux execution proof is claimed. The model negative
smoke is not proof against every undocumented tool or malicious same-user process.
The root denial does not protect hardlinks/copies outside the canonical root or
hide metadata. Bootstrap env capability is not a same-user process secrecy
boundary. SDK transport tracing must remain disabled in the desktop logger to
avoid library malformed-input diagnostics. Root/config drift requires cancel and
fresh authorization, and host checks remain mandatory on every read and stage.
