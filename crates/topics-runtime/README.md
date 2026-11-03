# Pinned Topics runtime

This crate owns installed Codex process lifetime, exact selected-scope tool
transport, the app-hosted official MCP SDK, and native containment. Canonical
values come from `topics-core::workspace`; there is no store dependency, raw
filesystem model tool, approval tool or apply tool.

## Effective model tools

Codex **0.166.0** is pinned. Its ordinary MCP configuration adds generic resource
handlers, and default model metadata forces Code Mode with additional built-ins;
disabling namespaces alone does not remove handlers. The earlier native-MCP
launch is superseded and is not approved for real-note use.

The implemented launch uses the supported startup `model_catalog_json` override,
pinned to the complete `gpt-6.1-sol` catalog entry and digest. It selects direct
mode, disables patch/shell/experimental native tools and agents, and disables all
Codex-internal MCP servers individually. `thread/start.dynamicTools` contains only
namespace `topics` with `scope_sources` and `propose_changes`. Their exact schemas
are fetched with the official **rmcp 3.8.0 client** from the **same authenticated,
scoped app-hosted rmcp server**. `item/tool/call` is only a transport adapter into
that MCP client; it never calls HostTools or storage directly. Unknown names,
namespace, thread/turn/call identity, errors, cancellation and drift fail closed.
No parallel data authority or alternate proposal path exists.

Specialist source/actual offline capture demonstrated exactly two tools and 18
forced forbidden calls rejected as unsupported, including hidden MCP resource
helpers. Its existing-auth live probe demonstrated subscription acceptance. The
production runtime tests additionally exercise the actual SDK client/server,
cited discovery and persisted-statement correction. Offline capture is not a
separately captured subscription request inventory or universal adversarial proof.

## Host API and guarded staging

```rust
pub trait HostTools: Send + Sync + 'static {
    fn call(&self, tool: &str, arguments: serde_json::Value)
        -> WorkspaceResult<serde_json::Value>;
    fn commit_candidates(&self, candidates: Vec<Candidate>, permit: CommitPermit)
        -> WorkspaceResult<()>;
}
```

`Runtime::new(RuntimeConfig { codex_path, proxy_path, app_data })` exposes
`status() -> RuntimeStatusResult`, `analysis_start` / `chat_start` taking
`(RunRequest, Arc<dyn HostTools>, EventSink) -> WorkspaceResult<String>`, and
`cancel(root_id, run_id) -> bool`. `RunRequest` contains core-assigned canonical
UUID `root_id`, `run_id`, `scope_id`, absolute `notes_root`, and bounded authorized
`prompt`. `EventSink` receives canonical `WorkspaceRunEvent`, increasing sequence
and exactly one terminal event.

The host captures immutable root/scope/run identity. `scope_sources` uses the
same scope/revision/sensitivity service as UI. `propose_changes` validates only,
including the combined buffered provisional candidate set, without staging.
Every read, source/topic/derived reference, citation, error and proposal must deny
unselected/local-only content and stale revisions. Chat persists the user
statement first and uses only bounded authorized current claims/rules/citations;
runtime never infers a correction with string heuristics.

After successful terminal model completion only, runtime calls
`commit_candidates` on a bounded detached preparation worker. Production desktop
must call the core's guarded staging helper:

```rust
service.stage_proposal_guarded(request, |proposal, action| {
    permit.publish(vec![proposal.id.clone()], action)
}).map(|_| ())
```

`CommitPermit::publish(ids, action)` is deadline-bound and revocable. The action
must be the **core-provided final short publication action**, after validation,
blocking preparation and temp write/flush. It must contain no arbitrary host
callbacks or preparation. Cancellation/timeout revoke before a late publication;
if the final publication fence is busy or publication already linearized, cancellation
returns false immediately and published IDs
produce proposal/completed even if the host then hangs, panics or errors. Runtime
holds no state lock during arbitrary host code. The final atomic OS rename cannot
be universally preempted; the brief publication action is the explicit
linearization point, not a claim that arbitrary filesystem I/O can be cancelled.
Core remains independent of runtime types.

The desktop also invalidates core capabilities on cancel/error/root switch and
must not hold a core/service lock while calling Runtime cancellation. No runtime
method approves or applies proposals. A host that stages from `call`, ignores the
permit, or places arbitrary blocking code inside `publish` violates this API.

## Privacy and confinement

Before selected bytes, exact effective config pins the native ChatGPT subscription
endpoint `https://chatgpt.com/backend-api/codex` and ChatGPT service base, provider
`openai`, and account type `chatgpt`. It forces `notify=[]`, inherited skills and
agent instructions off, empty developer instructions, zero project-doc bytes,
history off, telemetry exporters off and content logging false, empty compact
prompt and local thread store. Custom provider maps, selected profiles, prompt
files and extra endpoint/telemetry settings are rejected. Every child starts with
`env_clear`: only validated HOME/CODEX_HOME, PATH, validated TMPDIR, bounded locale
values and forced RUST_LOG=off cross the boundary. Trace, custom CA, proxy, auth,
API and OTEL knobs are excluded without enumerating their values; no global env/config
is changed, and no login, token IO/copy, deliberate refresh or API-key fallback
exists. All inherited feature names are disabled, and effective enabled/unknown
features fail closed. All Codex MCP entries must be disabled with empty inventory.

The returned thread must be ephemeral, empty instruction/workspace roots,
read-only/network false, approval never, pinned model, and empty app-owned cwd.
Canonical notes root, prospective app data and cwd must be disjoint in both
ancestor directions **before any directory/file write**, then checked again after
setup. UUID aliases and missing required nullable Candidate fields are rejected.

On macOS, a static parameterized sandbox profile denies file-read-data and file
writes beneath the canonical notes root, and literal unlink/rename of **each
ancestor**, plus file-clone denial on each ancestor literal. Untrusted paths are separate `-D` argv values, never Scheme text.
Bounded synthetic preflight checks canonical/symlink reads, overwrite/create,
ancestor `mv`, `RENAME_SWAP`, `clonefile` and `fclonefileat`, quoted Unicode
paths and allowed sibling reads/writes/clones. Syscall helpers require actual
successful unsandboxed positive controls and permission-specific denials; missing
helpers or unsupported operations cannot count as confinement. Probe fixtures must be outside the selected root and on its same
filesystem; unsupported volumes fail closed. No live note bytes are probed.

This path boundary does not protect pre-existing outside-root hardlinks/copies or
hide metadata. It is not universal machine confidentiality or protection against
a malicious same-user process reading memory/environment. The loopback bootstrap
capability authenticates a run; its client-chosen nonce only rejects exact
bootstrap replay, not a stolen-token holder minting a fresh nonce, and provides
no separate cryptographic server proof. Endpoint lifetime is bounded; unauthenticated
connection rate limits do not abort authenticated clients. SDK tracing remains
disabled in the app logger to avoid malformed-input diagnostics.

## SDK/lifecycle bounds and tests

The parent SDK is outside the notes denial. In-app calls use its official client;
the separately tested minimal stdio proxy remains the shared external transport
entry point. Desktop may dispatch `--topics-mcp-proxy` before Tauri/root startup
to `run_stdio_proxy() -> i32` and exit. Proxy EOF/deadline shutdown does not wait
for Tokio's uncancellable stdin reader. The in-app route no longer gives its
capability to the Codex child environment.

Frames: 512 KiB; host responses/prompt: 256 KiB; stream output: 2 MiB; calls and
candidates: 64; SDK host call timeout: 5s; callback timeout: 6s; staging preparation:
5s; execution: 150s; endpoint: 180s. Stalled read/preparation callbacks are detached
with process-wide bounded worker permits; they cannot publish after revocation.
Validated JSON is reserialized before SDK forwarding, so duplicate-key last-wins
semantics cannot diverge between the guard and SDK. Strict MCP envelope, SDK decode
and handshake checks precede rmcp, which otherwise
ignores malformed frames. Client cancellation poisons/discards the buffer. Child
process-group termination supplements pipe/endpoint teardown; Codex can place a
proxy in another group, so proxy lifetime relies on tested EOF/endpoint shutdown,
not an unsupported claim that one group kill reaches every descendant.

```
CARGO_INCREMENTAL=0 cargo fmt -p topics-runtime --check
CARGO_INCREMENTAL=0 cargo clippy -p topics-runtime --all-targets --all-features -- -D warnings
CARGO_INCREMENTAL=0 cargo test -p topics-runtime
CARGO_INCREMENTAL=0 cargo test -p topics-runtime --test live_codex --test schema_pin --test config_override_native --features native-live-tests -- --nocapture
```

All fixtures are synthetic temp roots/app-data. Native checks are macOS-only;
normal Linux tests do not claim native confinement proof. Live tests require an
actual scoped tool read and proposals, and adversarial tests require a successful
model baseline before denial proof; an unavailable runtime cannot pass them.
The real store adapter, guarded core helper and desktop native flow are separate
manager integration/review gates. Historical evidence in `evidence/REPORT.md` is
superseded where the Opus review and remediation report say so.

Catalog provenance: upstream OpenAI Codex commit
`6ff1e1ca3b30fc8b914720681fcf5fe62e95f856`,
`codex-rs/models-manager/models.json`, complete gpt-6.1-sol entry with only
`tool_mode`, `multi_agent_version`, `apply_patch_tool_type`,
`experimental_supported_tools` and `shell_type` changed as documented above.
SHA256 `63fd24dc2fff977b4ae4418e5ea2585ab88289784e8e1ddf8bbf84a9e3431672`.
The upstream Apache-2.0 license is retained in `CODEX-CATALOG-LICENSE.txt`.

Review findings, exact final test counts and remaining integration gates: [evidence/REVIEW-REMEDIATION.md](evidence/REVIEW-REMEDIATION.md).
