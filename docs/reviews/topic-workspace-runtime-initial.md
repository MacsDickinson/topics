# Independent review: Topics runtime/MCP at a612725 (f3e1ed0..a612725)

Reviewer: Claude Opus 5.5. Read-only; no repo changes. Probes ran in a scratch copy with synthetic temp dirs only.

## Verdict: CONDITIONAL

Accept for synthetic use and continued integration. Do NOT enable real-note analysis until F1-F3 are fixed or explicitly accepted by the user. PLAN treats non-excludable tools and unknown tool/config isolation as a hard stop (features/topic-workspace/PLAN.md "Runtime startup must fail closed if non-Topics tools cannot be excluded").

The bridge, staging path, process lifecycle and root denial are otherwise solid. No probe read the selected root through the OS boundary except via F4, and no failure path staged candidates.

## How the offline Codex evidence was obtained

Real codex-cli 0.166.0, the runtime's exact --disable/-c flags and thread/start params, throwaway HOME/CODEX_HOME, synthetic API-key auth.json, sandbox denying all non-loopback network, `openai_base_url` pointed at a local fake provider, stdio stub in place of the Topics MCP proxy. No provider call, no personal auth/config/notes.

Caveat for F1-F3: this is the API-key auth path on the default model (gpt-6.1-sol). The ChatGPT-subscription path was NOT captured, and the tool surface could differ there. Runtime code never checks either way.

## Findings

### F1 HIGH - Built-in tool surface is not excluded and cannot be detected
- Where: protocol.rs:6-16 (DISABLED), lib.rs:301-304, lib.rs:365 and protocol.rs:93-167 (inventory covers MCP only), lib.rs:618-649 (validate_item).
- Observed: captured request's `additional_tools` item offers functions.exec, wait, request_user_input, request_user_input_async; clock.sleep; collaboration.spawn_agent, followup_task, send_message, list_agents, interrupt_agent, wait_agent. Nested in exec: apply_patch, create_goal, get_goal, update_goal, list_mcp_resources, list_mcp_resource_templates, read_mcp_resource, clock__curr_time, mcp__topics__*.
- Topics tools were reachable only through exec; a direct call returned "unsupported call".
- exec, wait, sleep and failed calls emit no app-server item, so validate_item never sees them. README's "exact two tool schemas are checked" describes the MCP inventory only and gives a misleading picture of what the model can call.
- Also tried: --disable code_mode_host, multi_agent_v2, sleep_tool, goals, tool_suggest, skill_search, image_generation. Only sleep disappeared; exec and collaboration tools stayed, and code mode then fails closed. I found no flag set that excludes them.
- Containment actually observed: nested apply_patch failed with "No usable execution environment" (environments:[]), for relative, outside-root and inside-root paths; files unchanged. exec reports typeof require/process/fetch/Deno all undefined. create_goal failed with "Goal tools require a persistent thread". spawn_agent emits subAgentActivity, which validate_item rejects at item/started.
- Unproven: whether a spawned child has an environment or other tools (child made no provider POST in my 14s window); request_user_input_async effect (my arguments failed its schema; any server request would be denied by transport.rs:144-150).
- Needed: positive model-visible tool allowlist evidence, or an explicit user decision accepting this contained surface; correct the README either way. A live assertion or capture on the ChatGPT path is required to settle that path.

### F2 HIGH - Inherited `notify` receives prompt bytes; config validation is a narrow denylist
- Where: protocol.rs:168-197 (validate_config); lib.rs:292-331 (no override).
- Repro: synthetic CODEX_HOME config `notify=["/usr/bin/python3","-I","<script>"]` with the run's flags. After the turn the script's argv contained SYNTH_PROMPT and it wrote it to a file. --disable hooks does not stop it.
- Impact: chat prompts carry authorized topic context and the user statement to an arbitrary inherited program, unsandboxed apart from the root denial. The analysis prompt is smaller, but notify payloads also carry assistant text by Codex's format (assistant-text path not separately verified).
- Also unchecked by validate_config: openai_base_url, chatgpt_base_url, model_providers, otel, experimental_thread_store_endpoint, compact_prompt, experimental_compact_prompt_file. A probe with overridden openai_base_url still returned modelProvider "openai", so lib.rs:358 does not establish the destination. Whether the ChatGPT auth route honours that override is unproven.
- Needed: force `notify=[]` and reject or override endpoint and provider keys, preferably using an allowlist of effective config.

### F3 MEDIUM-HIGH - Inherited skills reach the model while instructionSources is []
- Where: protocol.rs:198-214, protocol.rs:191-195.
- Repro: synthetic $CODEX_HOME/skills/synth/SKILL.md and ~/.agents/skills/synth2/SKILL.md. Their description text appears in a developer message; thread/start still returns instructionSources:[], so validation passes. Also injected: bundled skill list with the absolute CODEX_HOME path, and <multi_agent_role> text telling the model it can spawn agents.
- Violates "no inherited instructions". AGENTS.md itself is correctly caught (non-empty instructionSources fails closed).
- Verified mitigation: `[skills] include_instructions=false` removed the skills message in the probe (set in config.toml; I did not test the equivalent -c override). <permissions instructions> then appeared in its place.

### F4 MEDIUM - Renaming an ancestor of the root defeats read and write denial
- Where: isolation.rs:8; preflight does not cover it (isolation.rs:33-119).
- Repro under the exact PROFILE: os.rename(parent_of_root, elsewhere), then read elsewhere/Notes/secret.md: sentinel returned. RENAME_SWAP of the parent also works. It also moves the user's folder on disk.
- This is not the documented hardlink/copy limit: it needs no pre-existing link. It does need native rename capability inside the sandbox, and no tool I could exercise provides that today (see F1), so this is defence-in-depth rather than a current model path.
- Verified mitigation: adding `(deny file-write-unlink (literal <ancestor>))` for each ancestor blocked the rename while sibling writes still worked. Tested for two ancestor levels and for mv only, not RENAME_SWAP; add a preflight case.

### F5 MEDIUM - "Any invalid authenticated frame poisons" is false for frames rmcp rejects
- Where: bridge.rs:318-344 filter; bridge.rs:382; rmcp 3.8 async_rw ignores parse errors (`receive_ignores_parse_error`).
- Repro: valid propose_changes buffered, then send a frame. All left failed=false, pending=1:
  tools/call read_file with no id (silently dropped); wrong-scope call with no id (dropped); forbidden tool with id null, object or float; bad _meta type; malformed initialize or tools/list params; malformed notifications/cancelled; ping with id []; method+result frame; pre-initialize tools/call on a second authenticated connection.
- Still poisons correctly: not-JSON, unknown method, a parseable forbidden call that reaches the handler, oversize.
- Impact: only a token holder can send these, and candidates are still revalidated by host at commit, but the stated invariant does not hold and tests only cover two frame shapes.

### F6 MEDIUM - Terminal event and run slot wait on a slow or hung host call
- Where: lib.rs:163-173 (Runtime dropped before terminal; tokio waits for spawn_blocking from bridge.rs:176), lib.rs:174-189.
- Repro: host.call sleeping 12s, cancel after buffer start: cancel() returned in 0ms, Cancelled terminal arrived 11.7s later, and a new analysis_start returned conflict meanwhile. A permanently hung host call means no terminal and a permanently blocked runtime. The 5s host deadline does not bound this. Stages stayed 0.
- commit_candidates (lib.rs:479) has no timeout and runs holding the state lock, so a hung commit also blocks cancel().

### F7 LOW-MEDIUM - Panic in commit_candidates wedges the Runtime
- Where: lib.rs:479, 174, 212.
- Repro: panicking host: no terminal event, next start returns conflict, cancel() panics on the poisoned state and poisons `active`, later starts return `internal` until restart.

### F8 LOW-MEDIUM - Client-cancelled propose_changes still buffers
- Where: bridge.rs:153-156 ignores the RequestContext cancellation token.
- Repro: notifications/cancelled for an in-flight call: pending=1, failed=false. Those candidates can stage even though the client abandoned the call.

### F9 LOW - Root/app-data separation is incomplete and writes before validating
- Where: lib.rs:231-243, 249-255.
- Repro 1: app_data inside the root is rejected, but the directories were already created in the notes root (created_inside_root=true).
- Repro 2: notes_root=<app_data>/runtime passes: cwd lives under the denied root, the parent creates dirs inside the notes root, and turn/start (prompt) was reached before the fixture failed.
- Canonical cwd is never compared with root.

### F10 LOW - Smaller gaps
- validate_thread ignores thread.ephemeral (fixture returning false proceeds and stages). Real Codex returned true.
- Provisional ID dedupe compares strings (bridge.rs:101-105): uppercase, simple-hex, urn: and braced aliases of the same UUID all pass, and cross-call duplicates buffer (pending=2). lib.rs:105 likewise accepts urn: run IDs despite "canonical". Not path traversal. Coordinator says core now dedupes; I did NOT verify that f56fb37 normalizes UUID aliases - check this seam.
- Missing nullable Option fields deserialize as None although schema and CONTRACT require them explicit (core serde).
- propose_changes annotated readOnlyHint:true (bridge.rs:122-127). No write occurs at call time, so I do not rate this a bypass, but offline readOnlyHint:false + destructiveHint:false + mode=auto also completed with no approval request, so the honest annotation looks viable. Retest on the ChatGPT path; this narrows REPORT's root-cause claim to absent annotations, which I did not test.
- 25+ unauthenticated loopback connections end the accept loop and abort authenticated connections (bridge.rs:263-266, 385): listener closed, no poison. Availability only; each idle attempt also blocks accept for 2s.
- The nonce is client-chosen, so it stops exact-line replay only; anyone with the token can mint a new one. No server proof to the proxy. Documented-limit class.
- Preflight probes use blocking .output() with no timeout (isolation.rs:25-30), uncancellable. Fixture checks TMPDIR's filesystem, not the selected root's.

### F11 LOW - Evidence and comment accuracy
- 3 of the 22 deterministic tests exercise protocol::TurnBuffer, which production never uses (only protocol.rs and tests/acceptance.rs reference it). Scripted tests do provide the real coverage.
- transport.rs:215-216 says the group kill reaches proxy descendants. Real Codex gave the MCP child its own pgid (codex 66830, stub 66831). Result is still correct: survivors=[] after group SIGKILL, and proxy.rs exits on stdin EOF by code reading (the live test used the stub, not the real proxy).
- Live fixtures use a test HostTools, not the real store. That is independent evidence for the runtime; desktop owns real-store/native acceptance and has not been run here.

## Verified correct
- Baseline in scratch at a612725: 8 unit + 8 acceptance + 1 proxy + 5 scripted = 22 pass, 0 ignored. Live tests deliberately not rerun; live logs are counts only.
- Exact PROFILE denies: direct read, ls, xattr listing, hardlink, clonefile, moving file or root out, /System/Volumes/Data, /.nofollow, /.resolve, /Volumes/Macintosh HD, /.vol, openat from parent, case and NFC/NFD variants, `=` and odd characters in the path; write, mkfifo, symlink, unlink, chmod, truncate, rename. Sentinel intact. libc realpath returned true-case/stored-form paths.
- Inherited MCP: fake server disabled via -c, inventory shows disabled with no tools; dotted names rejected at lib.rs:268; config drift fails at protocol.rs:184.
- Commit linearization: cancel and commit serialize on the state lock; terminal once; duplicate completion stages once; unknown/failed/wrong-turn/exit discard; a denied call reaching the handler poisons.
- No filesystem, approve or apply tool in the bridge; scope fixed by SessionGate. Token only in env, not argv; errors generic; no stdout diagnostics from proxy. rmcp pinned =3.8.0.
- Unknown notifications and all server-initiated requests fail closed; output, frame, queue, call and time bounds present. No surviving process after group kill.
- Ephemeral: with the default log level, prompt and tool-result sentinels were absent from all files in CODEX_HOME after a run (only notify's own output contained them).

## Documented limits, not defects
Outside-root hardlinks/copies; metadata via stat; same-user process inspection; env-held capability; Linux unavailable; rmcp tracing must stay off in desktop.

## Not tested
ChatGPT-auth tool surface; launchctl/LaunchServices escape from the allow-default profile (deliberately not run: would mutate user launchd state; same-user class); non-APFS or network roots; real store adapter; core f56fb37 itself.

## Probes
Kept beside this report in probes/: sb.py (sandbox), capture.py and script.py (fake provider), life.py (lifecycle), anc.py (mitigation), mcpstub.py. Rust probes were in the deleted scratch copy; their results are recorded above.
