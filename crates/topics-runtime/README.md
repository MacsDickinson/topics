# Topics installed runtime

`topics-runtime` owns Codex process lifetime, pinned app-server NDJSON, the fixed
app-hosted MCP surface and the sandboxed stdio proxy. It has no store dependency,
filesystem tool, proposal approval or apply API. Canonical domain and event values
come from `topics-core::workspace`.

The desktop may use its own executable as `RuntimeConfig.proxy_path`. Before
starting Tauri or reading root settings it dispatches `--topics-mcp-proxy` to
`topics_runtime::run_stdio_proxy()` and exits with that returned status. The
standalone `topics-mcp-proxy` binary calls the same function.

## Host adapter

`Runtime::new(RuntimeConfig { codex_path, proxy_path, app_data })` exposes
`status() -> RuntimeStatusResult`, `analysis_start` and `chat_start` with
`(RunRequest, Arc<dyn HostTools>, EventSink) -> WorkspaceResult<String>`, and
`cancel(root_id, run_id) -> bool`. `RunRequest` contains `root_id`, `run_id`,
`scope_id`, absolute `notes_root`, and bounded authorized `prompt`. The core
service assigns all three IDs. `EventSink` receives canonical `WorkspaceRunEvent`
with increasing sequence and one terminal event. Event delivery runs separately
from state locks. Cancelling must also invalidate the store run capability.

`HostTools::call(tool, arguments)` accepts only `scope_sources` or
`propose_changes`. The adapter captures immutable root/scope/run identity and uses
the same scope/revision/sensitivity checked core service as the UI. It must reject
unselected and local-only sources/topics, including evidence, derived references,
chat context and stale revisions. `propose_changes` validates only, without
publishing anything. `commit_candidates` revalidates the active run and stages a
pending proposal only after a successful matching terminal model turn. It never
approves or applies. Runtime serializes this call against cancellation. A host
implementation that stages from `call` violates the interface contract.

Chat prompt construction belongs to the trusted host: persist the user statement
first and include only the authorized topic/current claims/rules/citations and
that statement, subject to the same scope and sensitivity checks. Runtime does
not infer corrections using string matching.

## Isolation and lifecycle

Every run checks exactly `codex-cli 0.166.0`. It initializes without a `jsonrpc`
field, sends `initialized`, and calls `account/read` with refresh disabled. Only
an existing ChatGPT account is accepted; there is no login, credential reading,
auth refresh or API-key fallback. Codex owns its installed authentication.

On macOS the parent canonicalizes the root and launches `/usr/bin/sandbox-exec`
with a static Scheme profile and a separate `-D TOPICS_ROOT=<canonical root>`
argument. It denies file-read-data and all file writes below that root. Preflight
runs exact-profile sentinel checks in temporary synthetic directories, including
canonical and symlink aliases, quoted Unicode paths and an allowed sibling. It
never reads live root bytes to preflight. Other operating systems fail analysis
closed while the rest of the app remains useful.

This is a path boundary: pre-existing hardlinks and other copies outside the root
are not protected. Metadata is not hidden. This does not claim universal machine
confidentiality or protection from a malicious same-user process reading another
process's environment/memory. The per-run bootstrap capability is inherited in
three explicitly forwarded environment variables, never argv or diagnostics;
proxy connections authenticate a fresh UUID nonce and run token before MCP.
Repeated bootstrap nonces, other sessions, wrong/expired tokens and expired
endpoints are denied. There are at most eight authenticated connections and 24
connection attempts per run. All connections use the same captured host scope.
The endpoint and SDK services are closed on completion, error or cancellation.

Multiple connections are necessary for pinned Codex: its unfiltered
`mcpServerStatus/list` opens a separate discovery client even with `threadId`;
only `threadId` plus `serverName` reuses a thread connection. This behavior was
verified against the official rust-v0.166.0 `mcp_processor.rs`. We retain complete
paginated unfiltered inventory rather than ignoring a failed catalog probe.

The runtime first reads inherited MCP *names* without logging config, stops the
process, and disables each name before enabling Topics. Names must be a safe
single CLI key segment; a reserved `topics` collision or unusual name fails
closed with remediation. Codex CLI splits dotted override keys literally, so
quoted key segments are not safe quoting. Empty `mcp_servers={}` is not used.
The disabled features are shell_tool, unified_exec, hooks, plugins, apps,
view_image, browser_use, computer_use and multi_agent; web_search is disabled.
The effective config, complete inventory and exact two tool schemas are checked
before authorized prompt bytes. Threads are ephemeral, approval never,
read-only/network false, with empty environments and instruction sources in an
empty app-owned cwd. Configuration/account/root changes require relaunch and
reauthorization; host checks protect every read and stage from stale capabilities.

MCP uses official `rmcp = 3.8.0`, advertising protocol 2025-06-18 and exactly
`scope_sources {scope_id}` and `propose_changes {candidates}`. The SDK runs in the
parent, outside the root denial. The child forwards bounded frames only. Errors
from the host are replaced by generic denials; no source bytes or capabilities
are logged. Desktop logging should keep SDK transport tracing disabled because
protocol library diagnostics can contain malformed peer input.

App-server and MCP frames are limited to 512 KiB, model/bridge output to 2 MiB,
tool responses and prompt to 256 KiB, candidates to 64 and calls to 64. Host calls
have a five-second deadline; process requests 40 seconds; execution 150 seconds;
bootstrap/endpoint lifetime 180 seconds. Interrupt acknowledgement is followed by
a terminal wait or a bounded kill. Pipes are drained and children reaped. Malformed,
unknown, wrong-ID, failed, cancelled and late messages cannot stage candidates. An authenticated invalid or denied tool call poisons the entire run and discards earlier buffered candidates.

## Evidence and commands

`cargo test -p topics-runtime` runs deterministic protocol, authenticated SDK,
full fake-process cancellation/isolation and native macOS sentinel tests. The
native tests are `cfg(target_os="macos")`; no skipped native test is reported as
proof on Linux. A non-macOS analysis reports confinement unavailable.

Explicit native acceptance uses only synthetic temporary notes/app-data and the
installed runtime's existing subscription. It never accesses personal notes:

```
cargo test -p topics-runtime --test schema_pin --features native-live-tests -- --nocapture
cargo test -p topics-runtime --test live_codex --features native-live-tests -- --nocapture
cargo fmt -p topics-runtime --check
cargo clippy -p topics-runtime --all-targets -- -D warnings
```

The committed schema projection checks required fields, property shapes and
reachable enums for 16 pinned methods/results/notifications against the installed
generator. The live test requires an actual cited discovery proposal and a
correction citing a persisted user statement; a successful empty model turn does
not pass. `evidence/` records RED and sanitized results, not raw transcripts,
account data, configuration values or source content.

Final scenario mapping, API signatures, RED provenance and limits: [evidence/REPORT.md](evidence/REPORT.md).
