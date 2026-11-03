# Independent re-review: Topics runtime remediation a612725..44212f4

Reviewer: Claude Opus 5.5. Read-only; no repo changes. Synthetic throwaway homes and loopback only. No live provider call, no auth/config/header/notes reads.

## Verdict: CONDITIONAL

F1-F3, the previous blockers, are fixed and verified on the real code path. Nothing needs re-architecture. Before real notes, fix R1 and R2 (both small) or have the user explicitly accept them. R3-R10 are low/info and need not block.

## How it was tested

- Real path: the implementer's compiled live_codex test binary (real Runtime, SDK bridge, McpClient, test HostTools) -> real Codex 0.166.0 -> scripted loopback fake provider. Runtime's sandbox-exec launched a thin shim (TOPICS_TEST_CODEX). sandbox-exec cannot nest, so the shim relayed to a Codex started by the driver under the production root denial plus deny-network and deny ~/.codex (packages allowed).
- The shim changed only: openai_base_url -> loopback; spoofed config/read openai_base_url and account/read type (production correctly requires both); dropped WebSocket "Reconnecting"/fallback notifications my fake provider caused. Auth was a synthetic API key.
- Caveats: this is API-key auth offline, not an intercepted subscription request. That binary was built 05:10:47, before the last bridge.rs edit (see R8). Because of the relay, Codex was not the Runtime's child in these runs, so they say nothing about lifecycle.
- Scratch build of exact 44212f4: 15 lib + 5 acceptance + 4 privacy + 10 scripted + 1 proxy = 35, matches the claim, no ignored. proxy_lifecycle failed once on its first run in that batch while the disk was nearly full; cause not determined (panic text not captured). It then passed 3 + 40 of 43 reruns. It is not on the in-app path.
- Not rerun by me: fmt, clippy, schema_pin, config_override_native, live model tests. My scratch binaries were lost when disk pressure forced cleanup. The bridge frame matrix below ran on a current-source scratch build before that.

## F1-F11 mapping

| ID | Status | Verified evidence |
|---|---|---|
| F1 tool surface | FIXED | Captured request offered exactly topics.propose_changes and topics.scope_sources on clean and hostile config. 26 forced calls (exec, shell, shell_command, exec_command, apply_patch, read_file, list_dir, grep_files, view_image, web_search, tool_search, update_plan, request_user_input*, sleep, spawn_agent x2, 3 MCP resource helpers, goals, mcp__topics__*, topics.read_file) all returned "unsupported call" with no callback and no host call. custom_tool_call exec: no item, no host call, turn completed (its output not inspected). Discovery ran end to end: scoped_reads=1, candidates=3, proposal_events=1. |
| F1 forwarding | VERIFIED | item/tool/call -> McpClient (rmcp client, no HostTools field) -> authenticated TCP -> same SDK server, SessionGate, HostTools. Descriptors come from real tools/list with schema and annotation equality. Token stays in parent, not in child env. dynamicToolCall is the only tool item accepted; mcpToolCall now rejected. |
| F1 catalog | VERIFIED | SHA256 63fd24dc... matches the code constant and the specialist file; five-field diff confirmed; Apache license present. thread_manager.rs passes config.model_catalog to provider.models_manager, consistent with the auth-independence claim; I did not read the manager implementation or observe a subscription request. |
| F2 notify/provider | FIXED | Hostile config: notify did not run. Custom model_providers and inherited profile: 0 provider posts. No prompt or tool-result bytes in CODEX_HOME after a run (including sqlite). |
| F2 env | PARTIAL | See R2. |
| F3 instructions | FIXED | No skills block, multi_agent_role, spawn_agent text or synthetic skill text, including with hostile skills and developer_instructions. Only base instructions, <permissions instructions>, environment context and prompt reach the model. ~/.codex/AGENTS.md fails closed with 0 posts (see R5). |
| F4 ancestor rename | FIXED, but see R1 | Under the exact new profile, denied: mv, RENAME_SWAP in both argument orders and on grandparent, /System/Volumes/Data, /.vol, case-variant and relative-dirfd renames. |
| F5 frames | FIXED for all original shapes, see R3 | All 10 original frames now poison with pending=0. Also poison: BOM, empty line, resources/read, logging/setLevel, extra task param, reused id. Valid ping does not poison. |
| F6/F7 | FIXED | Scripted slow-host, timeout, late publish, post-publication stall and panic tests pass. Reviewed: shutdown_background, bounded workers, revoke vs publish; cancel returns false once published; no false cancellation. Desktop on main (2f2de7e) uses stage_proposal_guarded + permit.publish as specified. |
| F8 | FIXED | Raw cancel and RequestContext cancellation both poison; regression test passes. |
| F9 | FIXED | validate_layout checks both directions before any mkdir; regression passes. |
| F10 | FIXED | Aliases rejected, cross-call duplicate denied through the real client, nullable required, ephemeral checked, honest annotations confirmed in code, rate limit no longer aborts sessions. |
| F11 | FIXED | TurnBuffer removed, group-kill comment corrected. |

## Remaining findings

### R1 MEDIUM - clonefile of an ancestor directory defeats the root read denial
- Where: isolation.rs:8, 16-24; not covered by preflight (isolation.rs:93-218). I missed this in the first review; it is not a regression.
- Repro under the exact profile including ancestor rules: clonefile(parent_of_root, /outside/c) or fclonefileat(open(parent_of_root), ...) returned 0, then read /outside/c/Notes/secret.md: sentinel returned.
- Not the documented hardlink/copy limit: it needs no pre-existing copy and creates one from inside the sandbox.
- Reachability: needs native syscall ability inside the sandbox. No tool on the verified F1 surface provides that, so this is defence-in-depth, same class as original F4.
- Verified mitigation: adding `(deny file-clone (literal <ancestor>))` for each ancestor blocked it while a sibling `cp -c` still worked. Not yet tested with Codex running under it. Add a preflight case with a positive control.
- Direct clonefile(root), cp -cR, ditto, recursive copyfile, openbyid_np, exchangedata were all denied.

### R2 MEDIUM-LOW - Environment scrub is a denylist and misses honoured variables
- Where: lib.rs:327-354.
- Repro: launch with CODEX_ROLLOUT_TRACE_ROOT=<dir>. Run completes normally and <dir>/trace-*/payloads/*.json contain prompt text despite the ephemeral thread. (I scanned for prompt text only, not tool results.)
- Also present in the binary and not removed: CODEX_CA_CERTIFICATE (REVIEW-REMEDIATION claims custom-CA overrides are removed), SSLKEYLOGFILE, CODEX_ACCESS_TOKEN, CODEX_APP_SERVER_CHATGPT_BASE_URL, CODEX_AUTHAPI_BASE_URL, OPENAI_ORGANIZATION, OPENAI_PROJECT. Only the trace variable's effect was demonstrated; the others are string evidence from the binary.
- Limit: requires the launching environment to set them (same-user or launchctl setenv), so not a model path.
- Fix: env_clear() plus a small allowlist (HOME, CODEX_HOME, PATH, TMPDIR, locale), keeping RUST_LOG=off.

### R3 LOW - Duplicate JSON keys still evade poison
- Where: bridge.rs:486-503 validates parsed last-wins Value but forwards the raw bytes.
- Repro: valid buffer, then `{"jsonrpc":"2.0","id":9,"method":"tools/call","method":"tools/call","params":{"name":"read_file","arguments":{}}}`. Reply error, failed=false, pending=1. Same for duplicate id, params, jsonrpc and params.name.
- Only a token holder can send this, and the token now stays in the parent, so not model-reachable.
- Fix: forward re-serialized validated value, or poison on any SDK error response.

### R4 LOW - RENAME_SWAP preflight can pass vacuously
- Where: isolation.rs:173-190. Any failure counts as denied, with no positive control, so a missing or broken /usr/bin/python3 passes. On a Mac without Command Line Tools that path is expected to be the install stub (not tested). The profile rule itself is verified above.

### R5 LOW - Fail-closed errors are raw codes, not actionable
- Where: lib.rs:396, 408, 415; protocol.rs strings such as unsafe_thread_configuration, custom_model_providers_unsupported.
- runtime.feature requires an actionable isolation error. A user with ~/.codex/AGENTS.md is blocked from all analysis (0 posts: correct) but, by code reading, sees only that code.

### R6 LOW - Hung final publication action blocks cancel()
- Where: commit.rs:47-63, 88; lib.rs:238-239. publish holds the permit mutex through the action; cancel holds the state lock while waiting on it. The final rename not being preemptible is an accepted limit, but cancel could return false immediately via a Publishing state instead of blocking its caller.

### R7 LOW - failed not rechecked at second lock
- Where: lib.rs:567-576 rechecks cancelled only. Currently unreachable: callbacks are serial and the parent is the sole client.

### R8 LOW - "Final" evidence predates the last source edit
- review-final-native-live.log 05:11:14, review-final-sdk.log 05:09:12, live binary 05:10:47; src/bridge.rs last modified 05:19:21. Only the deterministic log (05:19:38) follows it. Rerun native-live and SDK once at 44212f4 and replace the logs. I did not do this because extra live calls were out of scope.

### R9 INFO - Leftover proxy surface
- status() still requires proxy_path (lib.rs:90) although in-app runs never launch it and no child receives the token. Stale topics branches at lib.rs:642-654 and lib.rs:909-918.

### R10 INFO - Availability
- Any `error` notification (for example "Reconnecting") or configWarning fails the whole run. Correct fail-closed, but a transient network retry will surface as run failure.

## Limits preserved, not defects
Outside-root hardlinks/copies; metadata via stat; same-user memory/env compromise; final OS rename not preemptible; client-chosen nonce; tracing must stay off; Linux unavailable.

## Residual risk / not tested
Subscription request tool inventory (only offline capture plus source and the specialist's live success); whether upstream behaviour differs on ChatGPT auth; launchctl/LaunchServices escape from the allow-default profile (would mutate user state, same-user class, and needs execution capability the verified surface lacks); non-APFS roots (preflight fails closed on a different device); real-store acceptance (separate).

## Probes
probes2/ beside this report: f4.py, f4m.py (sandbox), shim.py, drive2.py (harness), drive.py (superseded). Rust frame-matrix probe source was in the deleted scratch copy; results are recorded above.
