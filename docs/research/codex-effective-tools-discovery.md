# Codex 0.166.0 effective-tool investigation

Date: 2026-11-03. Read-only product/source investigation; all source downloads, synthetic probes and captures are under `/tmp/topics-tool-investigation`. No product files were edited, installed binaries were unchanged, and this worker opened no credential or personal note files. Offline probes use throwaway HOME/CODEX_HOME, a fake API key, a loopback fake provider and a macOS sandbox denying external network and reads under `/Users/macs` except the installed runtime package. A separately authorized tiny live probe uses auth only through the installed runtime; its result is below.

## Result and recommendation

There **is** a supported configuration/API route to exactly two Topics tools without rebuilding Codex or replacing its authentication: pin an app-owned **startup** `model_catalog_json` with direct tool metadata, disable native utilities/agents, configure **no MCP servers inside Codex**, and supply exactly the two Topics descriptors through `thread/start.dynamicTools`. Every `item/tool/call` must be a transparent client transport to the **same scoped MCP server via the official rmcp client**, preserving schemas, scope, validation, failure/cancellation semantics and proposal authority. This adds no direct-note authority. Coordinator explicitly approved this transport route during investigation.

Offline actual-request capture contains exactly `topics.scope_sources` and `topics.propose_changes`, with no exec/wait, collaboration, questions, patch, clock, resource or search tools. Eighteen forced non-Topics calls returned unsupported; both allowed calls succeeded. A separate probe derived dynamic schemas from the same synthetic MCP server's `tools/list` and forwarded both callbacks to that server's `tools/call`, returning its sentinel. This is a demonstrated integration option, **not** an implemented/reviewed product fix or a real-note release approval.

Keeping a native Codex MCP server attached cannot satisfy the strict two-tool gate through the controls found: three generic MCP resource handlers are added. Merely hiding namespaces is insufficient: a forced direct call to a hidden resource handler still executed. The present product must remain fail-closed until the approved adapter, startup metadata/config checks, exact offline inventory test, negative tests and independent review are integrated.

## Version, artifacts and exact catalog

- Installed `codex --version`: `codex-cli 0.166.0`.
- Installed binary `/Users/macs/.codex/packages/standalone/current/bin/codex` SHA256: `39acf979729dd290f9a24ee38b2914e036c91b65952872e28ae2a6b88a7db7c1`.
- Official release `rust-v0.166.0` annotated tag object `593389380740f7b4762771573c447ccfdc59742f` resolves commit `6ff1e1ca3b30fc8b914720681fcf5fe62e95f856`. Saved `tag.json`, `tag-object.json`, selective source files under `source/`.
- Complete tested catalog: `/tmp/topics-tool-investigation/synthetic-catalog.json`, **66,917 bytes**, SHA256 **`63fd24dc2fff977b4ae4418e5ea2585ab88289784e8e1ddf8bbf84a9e3431672`**.
- It is the full pinned upstream `gpt-6.1-sol` entry in a `{ "models": [...] }` wrapper. Only these five metadata fields differ: `tool_mode:"direct"`, `multi_agent_version:"disabled"`, `apply_patch_tool_type:null`, `experimental_supported_tools:[]`, `shell_type:"disabled"`. Model slug, instructions and other capability metadata were retained. The probe explicitly overrides base instructions with a tiny synthetic instruction.
- Preserve this complete reviewed asset; do not synthesize an incomplete model entry. Pin binary version/catalog digest, reject model switching or fallback, and reject configuration drift. A changed catalog is a new reviewed security configuration, not a routine model-list refresh.

## Exact configuration and protocol

The minimal successful **tested** common disables were:

```text
--disable shell_tool --disable unified_exec --disable hooks
--disable plugins --disable apps --disable view_image
--disable browser_use --disable computer_use --disable multi_agent
--disable multi_agent_v2 --disable sleep_tool --disable goals
--disable tool_suggest --disable skill_search --disable image_generation
--disable token_budget --disable request_permissions_tool
--disable deferred_executor --disable send_message_to_user_async
-c 'web_search="disabled"'
-c 'notify=[]'
-c 'skills.include_instructions=false'
-c 'agents.enabled=false'
-c 'features.multi_agent_v2.disable_native_tools=true'
-c 'tools.experimental_request_user_input.enabled=false'
-c 'tools.update_plan.enabled=false'
-c 'model_catalog_json="/absolute/app-owned/catalog.json"'
-c 'model_provider="openai"'
-c 'openai_base_url="https://chatgpt.com/backend-api/codex"'
-c 'chatgpt_base_url="https://chatgpt.com/backend-api/"'
```

`model_catalog_json` is a **process-startup** setting: per-thread `config` overrides are accepted but are documented source-level no-ops. Model remains `gpt-6.1-sol`, using the built-in `openai` provider with runtime-owned ChatGPT auth. For production, **explicitly pin** the canonical subscription inference URL shown above and validate effective config equality; do not allow an inherited inference endpoint and do not force the API-key URL. Verify account/read with refreshToken:false reports ChatGPT before inference; never silently fall back to API-key auth. The offline-only fake-provider `openai_base_url` and synthetic API-key file in the probe are test instrumentation and **must not** become product configuration.

Disable **every inherited MCP server individually** and verify effective config and paginated status; `mcp_servers={}` merges rather than clears. Do not attach the Topics server to Codex in this mode: the Topics-owned rmcp client hosts the sole model-call forwarding path. Apply the runtime worker's separate endpoint, telemetry, notify, hooks, skills, filesystem and lifecycle privacy fixes as well; this report does not replace them.

Initialize:

```json
{"id":1,"method":"initialize","params":{"clientInfo":{"name":"topics","version":"0.1"},"capabilities":{"experimentalApi":true}}}
{"method":"initialized","params":{}}
```

Thread creation:

```json
{
  "id":2,"method":"thread/start","params":{
    "cwd":"/absolute/empty/app-owned/cwd",
    "model":"gpt-6.1-sol",
    "approvalPolicy":"never","sandbox":"read-only",
    "ephemeral":true,"environments":[],"allowProviderModelFallback":false,
    "baseInstructions":"<reviewed Topics instructions>",
    "dynamicTools":[{
      "type":"namespace","name":"topics",
      "description":"Topics tools",
      "tools":[
        {"type":"function","name":"scope_sources","description":"<exact MCP description>","inputSchema":"<EXACT MCP JSON schema object>","deferLoading":false},
        {"type":"function","name":"propose_changes","description":"<exact MCP description>","inputSchema":"<EXACT MCP JSON schema object>","deferLoading":false}
      ]
    }]
  }
}
```

The `inputSchema` placeholder above must be replaced with the actual object, never a string. The probe's synthetic MCP schema requires exactly `{scope_id:string}`; production must use existing Topics MCP descriptors verbatim. Ensure no deferred dynamic tools, extra namespace members or optional host-provided tools are added.

Turn creation remains `turn/start {threadId,input:[{type:"text",text:...}],environments:[]}`. No supported `turn/start` tool allowlist field exists in the installed schema. The environment list is sticky; keep it empty and reject unexpected environments/capability roots. Validate thread ephemeral/cwd/sandbox/approval/model/instruction sources.

For an allowed call, Codex emits:

```json
{"method":"item/tool/call","id":0,"params":{
  "threadId":"...","turnId":"...","callId":"...",
  "namespace":"topics","tool":"scope_sources",
  "arguments":{"scope_id":"synthetic-scope"}
}}
```

Match thread/turn/call IDs and exact namespace/name, pass the **same arguments** through the same scoped MCP client `tools/call`, and map MCP text content into:

```json
{"id":0,"result":{"success":true,"contentItems":[{"type":"inputText","text":"<MCP result text>"}]}}
```

`success` must reflect MCP `isError`; do not turn errors into successful results. Reject unsupported content kinds; Topics requires no images/audio. The surrounding `item/started` / `item/completed` are `dynamicToolCall`, not `mcpToolCall`, so production event validation needs the reviewed transport-specific item allowlist. Unknown server requests remain denied; only precisely scoped `item/tool/call` is newly accepted. Preserve interruption/root switch/deadlines/poison/staging checks through both forwarding and MCP; no direct HostTools shortcut.

## Offline observations

Full synthetic request/output evidence is in the named JSON files, not model self-reports:

| Artifact | Actual observed result |
|---|---|
| `source_controls.json` | Collaboration and both question tools removed; advertised exec/wait remain. Nested tools: apply_patch, clock, three resource helpers, two Topics tools. JS require/process/fetch/Deno all undefined. |
| `source_controls_dynamic_empty.json` | `dynamicTools:[]` does not remove builtins: exec/wait remain. |
| `catalog_direct_mcp.json` | Direct metadata removes exec/wait/patch/clock, but native MCP adds three resource helpers plus deferred tool search. |
| `catalog_direct_dynamic.json` | Exactly topics.propose_changes and topics.scope_sources; no top-level extra tools. |
| `dynamic_without_catalog.json` | Dynamic tool registration alone retains exec/wait because default model metadata forces Code Mode. |
| `direct_dynamic_negative.json` | Exact two-tool request; 18 forbidden calls rejected as unsupported; two Topics callbacks succeed; only two server requests, both item/tool/call; no subagent event. |
| `same_mcp_forwarding.json`, `mcp-forwarding-log.json` | Exactly two Topics tools; descriptors from tools/list; two callbacks become same server tools/call; both return SYNTH_TOOL_RESULT_SENTINEL. |
| `code_mode_exclusions.json` | Excluding functions/clock namespaces leaves exec/wait; nested visible object has only two Topics tools. |
| `excluded_hidden_direct_resources.json` | Despite above exclusion, forced direct functions.list_mcp_resources executes and returns resources:[]; namespace hiding is not removal. |
| `disabled_mcp_direct_dynamic.json` | Disabled inherited MCP entries do not recreate resource tools: exact two-tool inventory; forced list_mcp_resources unsupported. |
| `bundled_gpt55_mcp.json` | Native bundled gpt-5.5 with code_mode/code_mode_only disabled still offers apply_patch, three resource tools and tool_search. |

The 18 rejected names were functions.exec, wait, apply_patch, exec_command, shell_command, list_mcp_resources, list_mcp_resource_templates, read_mcp_resource, create_goal, get_goal, update_goal, request_user_input, request_user_input_async, view_image; collaboration.spawn_agent/list_agents; clock.sleep/curr_time. Forced calls came from the fake Responses stream, so they test dispatch rather than a model choosing to comply. Topics calls produced the two expected client callbacks. Registry absence caused unsupported errors before any environment action. The patch probe used a function payload; the source condition independently shows the patch handler is not registered when metadata is null.

Offline scripts have bounded RPC/turn waits, parent process-group teardown and temporary HOME removal. The fake provider is local only and never receives real tokens; captures contain no auth headers. Initial websocket upgrade attempts fail at loopback before HTTP fallback, which accounts for approximately seven seconds of each offline test; no external network is allowed.

## Primary-source evidence and why earlier flags failed

Links below are pinned to the verified release commit. Local copies retain original line numbers.

1. [Model tool-mode precedence](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/core/src/tools/mod.rs#L66): `model_info.tool_mode` wins over `features.code_mode` / `code_mode_only`. [Bundled catalog](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/models-manager/models.json) sets code_mode_only for every shipped model except gpt-5.5; all entries advertise patch. Disabling code_mode_host only disables a host implementation, not this model tool selection.
2. [Startup-only catalog config](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/config/src/config_toml.rs#L408); [catalog load/validation](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/core/src/config/mod.rs#L2257); [auth-preserving authoritative static manager](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/model-provider/src/provider.rs#L549). The supplied catalog is checked before remote discovery and retains the runtime auth manager.
3. [Native agent override](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/core/src/config/mod.rs#L1687): enabled multi_agent_v2 wins; otherwise agents.enabled=false forces Disabled before model metadata. [V2 hide-native control](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/core/src/tools/spec_plan.rs#L734) is an additional explicit control. Feature multi_agent alone is insufficient.
4. [Question tool config resolution](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/core/src/config/mod.rs#L2816) defaults enabled; [both question registrations](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/core/src/tools/spec_plan.rs#L1316) obey it. Instructions/mode are not an availability boundary.
5. [Resource tool registration](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/core/src/tools/spec_plan.rs#L1277) occurs when any MCP servers exist **or** model mode is code_mode_only. There is no corresponding user tools.* toggle. [Patch registration](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/core/src/tools/spec_plan.rs#L1426) depends on model metadata, not environments. Empty environments disable execution availability; they do not remove handlers.
6. [Tool router construction](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/core/src/tools/spec_plan.rs#L121) adds core, MCP, extension and **then** dynamic tools. [Dynamic tools append](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/core/src/tools/spec_plan.rs#L1589), not replace, the builtins. [Missing handler dispatch](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/core/src/tools/registry.rs#L753) returns unsupported; [router dispatch](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/core/src/tools/router.rs#L359) uses the registry, explaining why merely hidden tools can still execute.
7. A genuine internal [ToolPolicy.allowed_tools](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/ext/extension-api/src/tool_policy.rs#L5) exists, captured once from Rust ExtensionDataInit. [Session capture](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/core/src/session/session.rs#L1061), [app-server construction](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/app-server/src/request_processors/thread_processor.rs#L1574): app-server supplies selected capability roots only. Installed ThreadStartParams, TurnStartParams, InitializeCapabilities and config schema expose no ToolPolicy/allowed_tools field. This is a Rust embedding API, not an available JSON-RPC/config allowlist in the shipped binary. Do not invent one.
8. [Code Mode globals](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/code-mode-runtime/src/runtime/globals.rs#L15) install tools/helper functions in V8 and omit Node globals; [module loader](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/code-mode-runtime/src/runtime/module_loader.rs#L145) rejects unsupported imports. These are bounded capabilities, not proof that the default extra surface is inert: collaboration, resource handlers and other side effects were real. No default-mode confidentiality exemption is justified.

Installed local schemas inspected first: `/tmp/topics-codex-schema/v2/{ThreadStartParams,TurnStartParams}.json`, `v1/InitializeParams.json`, `DynamicToolCallResponse.json`; CLI help verified app-server -c/--disable and code-mode-host semantics. `InitializeCapabilities` negotiates experimental API, MCP elicitation extensions, notification opt-outs and attestation; it is not a tool allowlist. `MultiAgentMode` fields are deprecated/ignored and describe instruction policy, not enforceable tool restriction. baseInstructions/developerInstructions similarly cannot remove handlers.

Official documentation corroboration: [App Server dynamic tool callback flow](https://learn.chatgpt.com/docs/app-server#dynamic-tool-calls-experimental) documents dynamicTools and item/tool/call; [model catalogs](https://learn.chatgpt.com/docs/model-catalogs) documents version-specific complete metadata and local model_catalog_json taking precedence over remote discovery. The source and offline probes, rather than mutable documentation alone, establish the exact 0.166.0 behavior.

## Live subscription verification

**PASS**: `/tmp/topics-tool-investigation/live-summary.json` records existing ChatGPT auth, two inherited MCP servers disabled, thread safety checks passed, tool names `["scope_sources","propose_changes"]`, two callbacks, no unexpected server requests, `turn_status:"completed"`, and `success:true`. The final explicit canonical-endpoint run repeats all these successful checks in `live-canonical-summary.json`. Installed runtime retained credential ownership, used provider `openai` and model `gpt-6.1-sol`, and used the complete same pinned direct catalog. The wholly synthetic input instructed those two tool calls; returned content was fixed synthetic text with no notes. This live test verifies native subscription acceptance and client callback flow; the separate offline forwarding probe verifies forwarding to the same MCP server. It does not claim the production adapter was exercised.

Initial probe-startup attempts failed before initialize because I included an optional unrecognized `voice_transcription` feature flag; source checking removed it. An initial inference attempt explicitly set `openai_base_url=https://api.openai.com/v1` and failed with 401; this was a **probe setup error**, not a catalog incompatibility or account conclusion. [Default endpoint selection](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/model-provider-info/src/lib.rs#L467) uses the Codex subscription backend for ChatGPT auth unless `base_url` overrides it. The corrected default-route successful run omitted `openai_base_url`, checked no inherited inference override, and used normal runtime subscription routing. **A final production-configuration check also explicitly pinned `openai_base_url="https://chatgpt.com/backend-api/codex"` and validated the effective value: `/tmp/topics-tool-investigation/live-canonical-summary.json` records success, two Topics callbacks, no unexpected requests and completed status.** Use this explicit canonical pin in production to defeat inherited malicious overrides, together with `chatgpt_base_url="https://chatgpt.com/backend-api/"` and effective config validation. Both URLs are fixed public constants in the pinned primary source; no custom provider or credential proxy is needed. Do not force an API endpoint as a privacy “default” for ChatGPT auth. Failed-attempt sanitized summaries remain in the artifact directory; no credential values or headers were retained. No headers, credential values, auth files, model request payloads or account metadata were captured. Only tool names/counts and sanitized state booleans are retained. Runtime owns existing auth; no raw token reuse, login, forced refresh, paid API or alternate provider is used.

## Remaining limits and smallest decision

The coordinator approved the architecture-preserving dynamic-to-same-MCP adapter. No additional user relaxation of confidentiality or tool-removal policy is needed for that transport change. Implement that adapter and pin startup catalog/config; do not weaken the gate to accept Code Mode or hidden resource handlers. If that adapter is not integrated/reviewed, keep real-note analysis disabled and call the current build synthetic-only. This investigation changes no product behavior.

Offline proof establishes the local tool construction and forced-call rejection for this binary and configuration. A live successful direct tool turn establishes subscription/model acceptance and callback flow, **not** a separately captured subscription tool inventory, adversarial model guarantee, or end-to-end privacy certification. Inherited configuration and all runtime-worker isolation/lifecycle findings must still be fixed and independently verified. Every offline artifact, including failures/counterexamples, remains under the report directory for review.
