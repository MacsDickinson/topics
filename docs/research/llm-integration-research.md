# LLM integration research: how "topics" should provide chat

Task: `llm-integration-research` · Date: 1 November 2026 · Author: research worker (Claude Opus 5.5)

## 1. Recommendation

1. **Build the app's MCP server first.** Make it the only way any LLM reads or changes topics, facts and tasks. In-app chat, Codex, Claude Desktop jobs and local models all use the same tools. This is provider-neutral and useful before chat exists.
2. **MVP chat: run the Codex app-server as a child process and let Codex own sign-in.** The app shows a "Sign in with ChatGPT" button. It calls Codex's `account/login/start` (type `chatgpt`), so no token ever touches app code. This works today with the user's existing Codex login.
3. **Do not build on the official "Sign in with ChatGPT" (SIWC) plan-usage route yet.** It launched on 29 September 2026 and is technically a good fit. But it is documented only for Plus and Pro plans. This machine's Codex login is a **ChatGPT Business** workspace (`lhvbank.com`). Using a personal Plus plan for bank work notes is a policy problem anyway. Keep it as a later backend.
4. **Never reuse Codex's OAuth client id or `~/.codex/auth.json` tokens directly.** It breaks the intended trust model, it is fragile, and SIWC now makes it unnecessary.
5. **Second backend: `claude -p` (Claude Code headless).** The user is signed in to the **LHV Bank Limited Claude Enterprise** org. Existing scheduled jobs already send notes to Claude. So Claude may be the provider the employer has already approved. Ask before choosing which one is the default (see section 7).
6. **Local models (Ollama) for small private jobs only:** classify, tag, pull dates and candidate tasks out of new meeting notes. Not for chat or fact correction at MVP.
7. **Every LLM change is a proposal.** The LLM calls `propose_changes` with a structured diff. The user approves it in the app. Nothing writes to the store without approval, at least at MVP.
8. **In Rust, write a small `AgentBackend` trait of your own.** Do not adopt a framework (rig, genai) at MVP. Both chat backends are subprocess protocols, not HTTP APIs. Use `rmcp` for the MCP server.

## 2. "Sign in with ChatGPT": what exists in November 2026

### 2.1 Facts checked

- **SIWC is real and public.** OpenAI launched it at DevDay on 29 September 2026 ([The New Stack](https://thenewstack.io/sign-in-with-chatgpt/), [OpenAI quickstart](https://developers.openai.com/siwc/quickstart)). It has two parts: identity, and an optional "ChatGPT plan usage" permission. "Neither capability grants access to their ChatGPT conversations" ([OpenAI cookbook](https://developers.openai.com/cookbook/articles/sign-in-with-chatgpt)).
- **Who may use plan usage.** "ChatGPT plan usage is available to open-source projects, personal projects that run locally, and selected private apps." Paid or hosted apps must use an interest form ([cookbook](https://developers.openai.com/cookbook/articles/sign-in-with-chatgpt), [overview](https://developers.openai.com/siwc/token-sharing-open-source)). A personal local app like "topics" fits.
- **Which plans.** OpenAI's pages name only "eligible ChatGPT Plus and Pro" users ([help page](https://learn.chatgpt.com/docs/sign-in-with-chatgpt), [quickstart](https://developers.openai.com/siwc/quickstart)). Some secondary sources add Go. **No source names Business or Enterprise.** Ineligible users get `403 subscription_sharing_user_not_eligible` ([errors](https://developers.openai.com/siwc/token-sharing-open-source/errors-and-recovery)).
- **The flow.** Public client, no secret. Start with `client_id=dynamic_agent_client` and a stable, opaque `ext_agent_host_id`. Use PKCE (S256) and a loopback callback on `127.0.0.1` (not `localhost`). Scopes are `openid profile email` plus `offline_access resource.invoke chatgpt.tokens.use.direct`. Tokens come from `https://auth.openai.com/api/accounts/oauth/token`; refresh tokens rotate ([sign-in](https://developers.openai.com/siwc/token-sharing-open-source/sign-in), [sessions](https://developers.openai.com/siwc/token-sharing-open-source/profiles-and-sessions)).
- **Inference.** Plain `POST https://api.openai.com/v1/responses` with the bearer token. `store: false` and `stream: true` are required. Function and custom tools work. Hosted MCP, file search, `temperature`, `previous_response_id` and system-role messages do not ([models](https://developers.openai.com/siwc/token-sharing-open-source/models-and-inference), [limits](https://developers.openai.com/siwc/token-sharing-open-source/preview-limitations)).
- **Official SDK is TypeScript only** (`@siwc/local`, `@siwc/react`). A Rust app would do the OAuth itself (the `oauth2` crate and a loopback listener are enough).
- **Usage** counts against the user's plan. Users can set a weekly cap per app ([help page](https://learn.chatgpt.com/docs/sign-in-with-chatgpt)).
- **Data terms for plan-usage traffic are not stated.** I found no OpenAI statement on training or retention for SIWC requests. Flagged as unverified.

### 2.2 The three options

| | (a) Spawn Codex app-server, Codex owns auth | (b) Official SIWC + Responses API from Rust | (c) Reuse Codex OAuth client id/tokens |
|---|---|---|---|
| Works with user's Business login | **Yes** (Codex supports ChatGPT Business) | Probably **no** (Plus/Pro only per docs) | Technically, until it breaks |
| Terms | Documented integration surface for "your own product" ([app-server docs](https://learn.chatgpt.com/docs/app-server)) | Explicitly allowed for local personal apps | Not sanctioned. Impersonates Codex. Tokens are "like a password" ([auth docs](https://learn.chatgpt.com/docs/auth)) |
| Stability | `app-server` is marked `[experimental]` in CLI 0.166.0. Pin the version | Public preview with a long list of limits | Can break any day |
| Native feel | Extra process, but one Rust binary already installed | Best: pure Rust HTTP | — |
| Tokens in app code | None | Yes (app stores them) | Yes |
| Built-in agent tools (shell, file edits) | Present; must be locked down | None | — |
| **Verdict** | **MVP** | Later, if Business becomes eligible or a personal plan is allowed | **Reject** |

Note: the SIWC docs also describe running Codex app-server with a SIWC token (`model_provider` pointed at `/v1/responses` with `env_key = "ACCESS_TOKEN"`) ([SIWC Codex page](https://developers.openai.com/siwc/token-sharing-open-source/codex-app-server)). So route (a) can switch to SIWC auth later without changing the app's protocol code.

### 2.3 How route (a) works

**Process and protocol.** The app spawns `codex app-server` (stdio is the default). It speaks JSON-RPC 2.0 over newline-delimited JSON, without the `"jsonrpc"` field on the wire ([app-server docs](https://learn.chatgpt.com/docs/app-server)).

1. `initialize` with `clientInfo { name: "topics", title, version }`, then the `initialized` notification. The `name` shows in OpenAI's Compliance Logs, which suits a bank.
2. `account/read`. If not signed in, call `account/login/start { type: "chatgpt" }`, open the returned URL in the browser, and wait for `account/login/completed`. This is the in-app "Sign in with ChatGPT" button. Codex stores and refreshes the token.
3. `thread/start` with model, `cwd`, `sandboxPolicy`, `approvalPolicy`, and optional `dynamicTools`. Save `thread.id`.
4. `turn/start { threadId, input }`. Stream `item/agentMessage/delta` into the chat view. `item/started` and `item/completed` show tool calls. `turn/completed` ends the turn; only status `completed` means success.
5. `turn/interrupt` for a Stop button. `thread/resume` after a restart. `account/rateLimits/read` to show plan headroom.

**Exposing app tools.** Two ways:

- **MCP (recommended).** The app hosts an MCP server and passes it at spawn: `-c mcp_servers.topics.url="http://127.0.0.1:<port>/mcp"` (streamable HTTP, with `bearer_token_env_var`), or a stdio command. The same server also serves Claude and other clients.
- **`dynamicTools`** on `thread/start`. Codex sends `item/tool/call` back to the app as a server request. This is simpler in-process, but it is experimental and Codex-only. Keep it as a fallback.

**Locking Codex down.** Codex is a coding agent, so it has shell and file-edit tools. For a notes app:

- Set `cwd` to an empty app-owned folder, **not** `~/notes`. Codex should see notes only through MCP tools. This also gives a full audit log of what data the model saw.
- Use `sandboxPolicy: readOnly` with read roots limited to that folder, and `networkAccess: false` for commands.
- Decline every `item/commandExecution/requestApproval` and `item/fileChange/requestApproval` by default.
- Give strong developer instructions: "use only the topics tools".
- Check with Codex feature flags whether shell tools can be turned off fully. I did not verify a clean switch for this.

**Rust types.** Codex itself is Rust. Its protocol types live in `codex-rs/app-server-protocol` in [openai/codex](https://github.com/openai/codex/tree/main/codex-rs). Crates on crates.io with similar names are third-party. Two options: depend on the official crate by git tag, or run `codex app-server generate-json-schema` for the pinned version and hand-write serde types for the ~15 messages you need. I recommend the second, plus a CI contract test that regenerates the schema and fails on drift when Codex updates.

**Structured one-shot jobs.** `codex exec --json --output-schema schema.json --ephemeral` gives JSONL events and a schema-shaped final answer. It suits batch jobs (for example "propose fixes for this new meeting note"), without a long-lived process.

## 3. Claude: equivalents and coexistence

### 3.1 What the terms allow

From Anthropic's [Legal and compliance page](https://code.claude.com/docs/en/legal-and-compliance) (read 1 Nov 2026):

- Developers building products "should use API key authentication". Anthropic "does not permit third-party developers to offer Claude.ai login into their own applications". Developers "may not collect, store, or intermediate Claude.ai credentials or session tokens".
- But it does not "prevent an end user from signing in to the unmodified Claude Code binary with their own Claude subscription".
- The [Agent SDK overview](https://code.claude.com/docs/en/agent-sdk/overview) repeats: no claude.ai login in third-party products unless approved. The SDK is Python and TypeScript only. For other languages it says to run the CLI as a subprocess with `-p`.

So for a personal tool, the safe pattern is: **spawn the unmodified `claude` binary, never touch its tokens.** Do not ship "topics" to others with a Claude login. If it is ever shared, use API keys.

### 3.2 Two ways to use Claude

1. **Claude as an in-app chat backend.** Spawn `claude -p --output-format stream-json --input-format stream-json --mcp-config topics-mcp.json --allowed-tools "mcp__topics__*" --permission-mode <restrictive>`. Use `--json-schema` for structured one-shots and `--bare` to skip hooks and the user's own CLAUDE.md. The user's login is the **LHV Enterprise org**, so the Commercial Terms apply (checked with `claude auth status`).
2. **Claude jobs as MCP clients of the app (main point).** The current skills (monday-actions-prep, weekly-roundup, eng-leadership-meeting-notes, 121-prep) write markdown into `~/notes/Log/` and `memory/`. Instead, register the app's MCP server in Claude Desktop and Claude Code. Then the skills call tools such as `list_open_tasks`, `add_task_update`, `propose_fact_change` and `get_topic`.

### 3.3 Proposed coexistence plan

- **Phase 1, read-only.** The MCP server exposes `search_topics`, `get_topic`, `list_tasks`, `get_daily_log`. The skills keep writing markdown but read the backlog from the app.
- **Phase 2, writes as proposals.** Skills call `propose_changes`, and items land in the app's review inbox. The weekly roundup shrinks, because daily small proposals replace the big Friday correction.
- **Phase 3, the app owns the files.** The app renders the weekly actions page and people/project pages from its store. Skills stop writing those files directly. The app becomes the single writer, which ends edit conflicts between jobs.
- **Trust per client.** The MCP server knows which client called (bearer token per client). The app can auto-apply low-risk changes from trusted jobs (for example "add comment to task") and queue the rest.
- **Transport.** Serve stdio for Claude Desktop (it launches a small `topics-mcp` binary that talks to the running app over a Unix socket). Serve streamable HTTP on `127.0.0.1` for Codex. `rmcp` 3.x supports both. Use a current release: versions before 1.4.0 had a DNS-rebinding flaw in the HTTP transport ([docs.rs](https://docs.rs/crate/rmcp/latest)). Enterprise admins can limit local MCP servers in Claude Desktop, so check this.

## 4. Local models

**This machine:** Apple M5 Pro, **24 GB** unified memory, Ollama installed with `qwen3.6:latest` (23 GB). That model is too large to run next to normal apps on 24 GB.

| Job | Local fit | Notes |
|---|---|---|
| Classify a new meeting note to topics/people | Good | Small model plus a candidate list from the app's index |
| Pull candidate tasks, dates, owners | Good | Use JSON-schema output (Ollama's `format` field) |
| Flag "Unverified" claims, odd names | Fair | Good recall, so-so precision. Fine as a hint for review |
| Fix pronouns and misheard names across documents | Weak | Needs broad context and judgement. Use the cloud model |
| Interactive chat over the second brain | Weak at this size | Slow, poorer tool use |

- **Model size.** Aim for 4–12B models at 4-bit (about 3–8 GB). Test 2–3 current small models on a fixed, hand-labelled set of 30 notes before choosing. The scoring set stays on the machine.
- **Runtime.** Ollama moved to an MLX backend on Apple Silicon in 2026 ([Ollama blog](https://ollama.com/blog/mlx), [June update](https://ollama.com/blog/mlx-performance)). One third-party guide says MLX needs 32 GB or more and smaller Macs fall back to llama.cpp/Metal. I could not confirm this with Ollama. Either path is fine for small models.
- **Cost.** No money per call and no data leaves the machine. The costs are a few seconds of latency, RAM, battery, and a quality gap on judgement tasks.
- Use Ollama's HTTP API rather than embedding llama.cpp or MLX in the app. That keeps the app small and lets the user swap models.

## 5. Provider abstraction in Rust

### 5.1 Crates

| Crate | What it is | Use in "topics" |
|---|---|---|
| `rmcp` (3.6.0, Oct 2026) | Official Rust MCP SDK | **Yes**, for the MCP server |
| `async-openai` | Typed OpenAI client (chat, Responses), works with compatible endpoints | Later, for the SIWC direct backend. Optional for Ollama `/v1` |
| `genai` | Multi-provider client, native protocols, Ollama support | Possible for the local backend. Pre-1.0 |
| `rig-core` | Agent framework with tools and retrieval | Not at MVP. Its loop would duplicate what Codex/Claude already do |
| `oauth2` | OAuth 2.0 with PKCE | Later, for SIWC direct |

All of these are pre-1.0 and fast-moving ([crate landscape](https://encore.dev/articles/best-rust-framework-for-building-ai-agents), [genai](https://crates.io/crates/genai)). Pin versions.

### 5.2 Design

```rust
/// One chat session with some model runtime.
#[async_trait]
trait AgentBackend {
    async fn start(&self, ctx: SessionConfig) -> Result<SessionId>;
    async fn send(&self, s: &SessionId, msg: UserMessage) -> Result<EventStream>;
    async fn interrupt(&self, s: &SessionId) -> Result<()>;
    fn auth_state(&self) -> AuthState;          // SignedOut | SignedIn{plan} | Error
}

enum AgentEvent { TextDelta(String), ToolCall{..}, ToolResult{..}, Proposal(ChangeSetId),
                  UsageLimit{..}, Done{status}, Error(..) }
```

- **Backends:** `CodexAppServer` (MVP), `ClaudeHeadless` (MVP+1), `OllamaLocal` (batch jobs only), `SiwcResponses` (later).
- **Tools are not part of the backend.** All tools live in the MCP server. Backends only connect to it. A BDD test can run any backend against a fake MCP store.
- **A `FakeBackend`** replays scripted events, so BDD scenarios for chat-driven corrections run with no network and no tokens.

### 5.3 How chat-driven corrections work

1. The user types: "Alex is she/her, and the vendor is Acme not Akme."
2. The model calls read tools (`get_topic`, `search_mentions`) to find where the error appears.
3. The model calls `propose_changes` with a `ChangeSet`:
   ```json
   { "summary": "Fix pronoun and vendor name",
     "ops": [
       {"op": "update_fact", "fact_id": "...", "field": "pronouns", "from": "he/him", "to": "she/her"},
       {"op": "add_alias", "entity": "acme", "alias": "Akme", "kind": "mishearing"},
       {"op": "annotate_source", "source": "Meetings/2026.10/....md", "note": "Akme→Acme"} ],
     "evidence": [{"source": "...", "quote": "..."}] }
   ```
4. The app validates it (ids exist, the `from` values still match, so it is optimistic concurrency) and shows a diff card in the chat.
5. The user clicks Approve, Edit or Reject. The app applies it in one transaction and logs who proposed it (backend, model, client id).
6. Source meeting notes are never rewritten. Corrections live in the store as aliases and overrides, and future extraction uses them. This keeps static history honest and makes the "misheard names" list grow over time.

Write tools such as `update_fact` and `create_task` exist only inside `propose_changes` for LLM clients. The UI calls the same apply path directly.

### 5.4 Privacy: what leaves the machine

The user works for a bank, and the notes are confidential.

| Route | What leaves | To whom, under which terms | Local residue |
|---|---|---|---|
| Codex app-server, ChatGPT Business | Prompts, instructions, every MCP tool result (note text) | OpenAI, under the LHV ChatGPT Business workspace. OpenAI says business data is not used for training by default ([business data](https://openai.com/business-data/)), but that page does not name Codex. Verify | `~/.codex/sessions` thread history holds note content. Codex keeps the login in `~/.codex/auth.json` as plain text unless the keyring store is set |
| `claude -p`, LHV Enterprise | Same | Anthropic, under the Commercial Terms of the LHV org | `~/.claude/projects` transcripts |
| SIWC with a personal Plus/Pro plan | Same | OpenAI. Data terms for plan-usage traffic not stated | Tokens in the app's store |
| Ollama | Nothing | — | Model files only |

Mitigations:

- Send only the topic slice needed, never the whole repo.
- Show a "sent to provider" log per turn (the MCP audit gives this).
- Put a per-topic "local only" flag on sensitive topics such as HR and people matters. The MCP server refuses to return those to cloud clients.
- Run local classification before any cloud call.
- Set Codex `cli_auth_credentials_store = "keyring"`.
- Consider `--ephemeral` or periodic clean-up of CLI session logs.

## 6. Risks

- **Codex app-server is experimental**, and Codex ships often (0.166.0 today). Protocol drift can break chat. Mitigate with a pinned version, the schema contract test, and a Claude fallback backend.
- **Codex is a coding agent.** A wrong sandbox or approval setting could let it run shell commands or read `~/notes` directly. Mitigate with an empty `cwd`, a read-only sandbox, deny-by-default approvals, and a BDD scenario that asserts this.
- **SIWC eligibility may never include Business.** Then the official "Sign in with ChatGPT" brand stays out of reach for work use. Codex login gives the same experience.
- **Plan limits are shared.** In-app chat eats the same Codex or Claude allowance as the user's coding work.
- **Anthropic terms have changed more than once in 2026** ([The Register](https://www.theregister.com/software/2026/02/20/anthropic-clarifies-ban-on-third-party-tool-access-to-claude/5014546)). Spawning the unmodified CLI is allowed today. Re-check before relying on it long-term.
- **MCP spec churn.** `rmcp` 3.x targets the 2026-07-28 spec (sessionless). Clients may lag. Keep the legacy session mode on.
- **Prompt injection from notes.** Meeting summaries are untrusted text. A note could contain instructions. Proposals with human approval are the main defence. Never auto-apply from chat.

## 7. Open questions for the user

1. **Employer policy:** which providers may see work notes? Both a ChatGPT Business workspace (`lhvbank.com`) and a Claude Enterprise org (LHV Bank Limited) are signed in on this Mac. Is one approved for confidential notes and the other only for code? This decides the default backend.
2. **Is "Sign in with ChatGPT" a must**, or is "use your existing Codex/ChatGPT Business login" good enough? (The official SIWC route probably needs a personal Plus/Pro plan.)
3. **Do you have a personal Plus/Pro plan, and would you ever use it for this?** I assume not, for work notes.
4. **May Claude Desktop connect to a local MCP server** under LHV's Claude Enterprise admin settings?
5. **Which topics must stay local only?** For example people, HR and performance matters.
6. **Should the app become the single writer** of `Log/` actions pages and `memory/` pages (phase 3)? Or should skills keep writing markdown, with the app reading it?
7. **Would you accept a 3–8 GB local model** running in the background on a 24 GB Mac? And is the 23 GB `qwen3.6` worth keeping?

## 8. What I could not verify

- Whether SIWC plan usage works for ChatGPT Business or Enterprise. The docs are silent, and secondary sources disagree on Go.
- OpenAI's training and retention terms for SIWC plan-usage traffic, and whether the Business no-training default explicitly covers Codex.
- A clean Codex switch that disables all built-in shell and file tools for app-server threads.
- Ollama's exact memory threshold for its MLX backend.
- The exact current type names in the official `codex-app-server-protocol` crate.

Local facts were checked on this machine on 1 Nov 2026: Codex CLI 0.166.0 (`codex login status`, `codex doctor`, plan claim read from the local ID token), Claude Code 2.1.316 (`claude auth status`), and `ollama list`. No note content was read or sent anywhere.
