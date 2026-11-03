# Runtime Opus review remediation

This report supersedes security and API claims in the historical REPORT.md for
`a612725`. Implementation is ready for independent re-review; this is not release
approval or a claim that desktop/core acceptance was run by this worker.

The initial independent review is preserved by the manager at
`docs/reviews/topic-workspace-runtime-initial.md`; detailed probes remain in the
reviewer's scratchpad. Runtime remediation commits start with `f66c5bf` and
`daf2094`, followed by the final verification/evidence commit named in the worker
receipt. Root Cargo manifest/lock changes stay manager-owned and uncommitted here.

## Consequential fixes

| Finding | Implemented behavior and evidence |
|---|---|
| F1 effective non-Topics built-ins | Ordinary Codex MCP mode is removed from the in-app launch. The complete pinned startup catalog forces supported direct mode for gpt-6.1-sol; all internal MCP servers are disabled and their complete paginated inventory must contain only those disabled names. Exactly two namespace descriptors come from the actual official rmcp client reading the same parent SDK server's tools/list. Exact item/tool/call callbacks forward through that client, never directly to HostTools. Unknown/stale namespace/name/thread/turn/call IDs fail closed. Specialist actual offline capture observed only two tools and rejected 18 forced forbidden calls, including hidden resource helpers; source proves startup static catalog applies to ChatGPT auth too. Production genuine subscription tests use this adapter and same SDK client/server. No namespace-hiding waiver or synthetic-only completion substitute is used. |
| F2 inherited notify/destinations | notify is forced empty before initialization/turns. Exact canonical subscription/provider bases, ChatGPT account, provider and model are checked. Custom provider maps, selected profiles, prompt files, experimental thread-store/realtime and unknown top-level endpoints are rejected. OTEL exporters/content logging, analytics, feedback/history are forced off; compact prompt is empty; thread store local. Child provider/proxy/API-key/OTEL/custom-CA environment overrides are removed without reading values; globals are unchanged. `review-privacy-red.log` -> green; installed metadata-only native test uses isolated HOME/CODEX_HOME and a hostile synthetic notify/skills/telemetry config with no auth read, turn or model call. |
| F3 inherited skill/developer instructions | skills.include_instructions=false, agents.enabled=false, V2 native disable and question/plan controls are passed via startup -c and verified effective. Developer instructions are empty, project-doc bytes zero, instruction sources/workspace roots empty. The supplied offline actual developer-context capture contains neither a skills block nor multi_agent_role; sanitized booleans/digests are recorded in specialist-tool-surface-summary.json. Native -c behavior is directly tested. Generated permission/collaboration-mode instructions are distinguished from inherited skills; no raw subscription prompt/header interception was used. |
| F4 ancestor rename | The parameterized profile denies file-write-unlink on each canonical ancestor using literal(param) rules. No path becomes Scheme text. Native preflight now tests mv and RENAME_SWAP for two ancestors, canonical and symlink reads/overwrites/creates, quotes/Unicode and allowed sibling reads/writes. Review supplied the original bypass RED; the strengthened direct OS preflight passes. |
| F5 SDK-silently-ignored frames | Strict envelope/id/params/_meta/known-method validation plus the SDK's own ClientJsonRpcMessage decode and per-connection initialization state run BEFORE SDK forwarding. Missing/null/object/array/float IDs, malformed initialize/list/cancel, method+result, pre-init calls and invalid notifications poison shared run state and discard prior buffers. Authenticated poisoning also revokes any unpublished preparation permit. `review-frame-red.log` records the SDK-ignore failure; full and raw-connection regressions pass. |
| F6 slow/hung host and commit | Runtime teardown uses shutdown_background rather than waiting on uncancellable spawn_blocking callbacks. Read-host workers and detached preparation workers have process-wide bounded permits. Host preparation runs outside state locks with a five-second revocable CommitPermit. Production core helper performs validation/preparation/temp write+flush before a short guarded final publication action. Cancel/deadline deny late publication and release terminal/run slot; published IDs win if the host later stalls/errors. Full Runtime tests cover slow read cancellation, prepublication cancellation and timeout, late guarded publish denial, next-run availability, and postpublication stall/error. |
| F7 commit panic | Host staging panics are caught outside runtime state locks; one error terminal is emitted and the runtime remains usable. A full subsequent successful run is tested. Publication result remains authoritative if a later host failure occurs. |
| F8 MCP cancellation | Both raw notifications/cancelled and RequestContext cancellation poison/discard before a callback can append candidates. An actual in-flight blocking-host SDK regression verifies cancellation leaves no buffer. |
| F9 path separation | Prospective canonical root/app-data/cwd are checked in BOTH ancestor directions before any directory or file write; canonical paths are checked again after setup. Preflight fixtures cannot be inside the selected root. `review-layout-red.log` -> green, including appdata nested in notes and notes nested in appdata/runtime. |
| F10 smaller issues | Returned thread.ephemeral must be true. UUIDs/references require lowercase hyphenated canonical spelling, and repeated provisional IDs are checked across the complete buffer. Required nullable Candidate fields are checked against the committed schema independently of older serde Option behavior. propose_changes is honestly readOnlyHint=false, non-destructive/closed-world; genuine ChatGPT direct-route tests pass with that descriptor fetched from MCP. Unauthenticated connection rate limits preserve authenticated sessions; fresh per-second buckets recover. Native probe output and process lifetime are bounded; probe and selected root must share a filesystem or analysis fails closed. |
| F11 evidence accuracy | Unused TurnBuffer and its three surrogate tests are removed; real Runtime/SDK regressions provide the coverage. Group-kill claims now explicitly exclude descendants that Codex places in another group; tested proxy EOF/endpoint shutdown is its lifetime mechanism. Runtime live fixtures remain independent of the real store; desktop/core integration and native human approval are separate manager checks. |

## Configuration-route correction

The intermediate `f66c5bf` privacy milestone forced the public OpenAI API base as a
canonical default. Specialist source/live investigation found that this overrides
the native subscription route. **It is superseded**: final code explicitly pins
`openai_base_url=https://chatgpt.com/backend-api/codex` and
`chatgpt_base_url=https://chatgpt.com/backend-api/`, with ChatGPT account type,
provider openai, pinned model and no fallback. The specialist and production
runtime live tests both pass this canonical subscription route. This worker did
not infer with the public-API override. No token was read/copied, no login/refresh
implementation or API-key backend was added, and no notes were sent to a custom
provider or notify command.

## Current public host seam

```rust
pub trait HostTools: Send + Sync + 'static {
    fn call(&self, tool: &str, arguments: serde_json::Value)
        -> WorkspaceResult<serde_json::Value>;
    fn commit_candidates(&self, candidates: Vec<Candidate>, permit: CommitPermit)
        -> WorkspaceResult<()>;
}
impl CommitPermit {
    pub fn publish(&self, ids: Vec<String>,
        action: &mut dyn FnMut() -> WorkspaceResult<()>) -> WorkspaceResult<()>;
}
```

Only runtime can create a CommitPermit, after a successful matching terminal
model turn. Desktop must use the independent core helper, not ordinary staging:

```rust
service.stage_proposal_guarded(request, |proposal, action| {
    permit.publish(vec![proposal.id.clone()], action)
}).map(|_| ())
```

The final core action is the sole brief publication linearization point. No
blocking preparation, arbitrary callback or model code belongs inside it. The
atomic OS rename itself cannot be universally preempted; cancellation at/after
publication is explicitly too late, not falsely reported as cancelled. A host
that ignores the permit or blocks arbitrarily inside the core-provided action
violates the cooperative interface. Core depends on no runtime type. Public
Runtime new/status/analysis_start/chat_start/cancel, RunRequest, EventSink and
run_stdio_proxy signatures otherwise remain as previously agreed. Runtime has no
approve/apply or direct-store API.

## Verification

Final normal suite: **35 deterministic tests**, no ignored tests, on this macOS:
15 library SDK/protocol/fence tests, five acceptance/boundary tests including native
OS preflight, four privacy/UUID/probe tests, ten full scripted Runtime tests, and
one proxy lifecycle test. Three unused TurnBuffer tests are no longer counted.

Explicit native verification: **four tests**: two genuine existing-subscription
model tests (discovery+correction, and adversarial private sentinel), one installed
schema-generator comparison covering 16 projections, and one isolated effective
configuration override test. The adversarial model test now requires a successful
scoped model tool read first, so a preflight/unavailable-runtime error cannot
masquerade as negative-model proof. Source bytes remain unchanged and no private
random sentinel is returned.

Commands:

```
CARGO_INCREMENTAL=0 cargo fmt -p topics-runtime --check
CARGO_INCREMENTAL=0 cargo clippy -p topics-runtime --all-targets --all-features -- -D warnings
CARGO_INCREMENTAL=0 cargo test -p topics-runtime
CARGO_INCREMENTAL=0 cargo test -p topics-runtime --test live_codex --test schema_pin --features native-live-tests -- --nocapture
CARGO_INCREMENTAL=0 cargo test -p topics-runtime --test config_override_native --features native-live-tests -- --nocapture
```

Final evidence: review-final-deterministic.log, review-final-sdk.log,
review-final-native-live.log and review-final-native-config.log. Genuine direct
SDK discovery yielded three cited candidates and one proposal event; correction
yielded one typed persisted-user-statement correction and one proposal event;
all scoped source bytes were unchanged. The model tests passed in 26.37 seconds,
schema comparison in 0.12 seconds and metadata-only native check in 0.10 seconds.
Logs contain counts/statuses, never raw model/source/account/config/capability
payloads. Private account/auth handling remains inside the installed runtime.

RED provenance includes the initial independent review's verified F1-F11 probes,
our explicit review-privacy-red.log, review-frame-red.log and review-layout-red.log,
and the earlier whole-buffer-error RED. Later focused lifecycle tests verify the
review-supplied RED cases; they are not falsely described as newly recorded local
preimplementation behavioral RED for every individual review issue.

## Catalog/source evidence and limits

The complete upstream entry is 66,917 bytes, SHA256
`63fd24dc2fff977b4ae4418e5ea2585ab88289784e8e1ddf8bbf84a9e3431672`,
from verified Codex release commit
`6ff1e1ca3b30fc8b914720681fcf5fe62e95f856`. Five reviewed metadata changes select
direct mode and remove native capabilities. It is written only under validated
app data, checksum checked, and the effective startup path/bytes/model checked
before selected bytes. The upstream Apache license is retained. Primary source
and actual-request/forced-call evidence are in the specialist's manager-preserved
report; sanitized digests/counts/context booleans are in
specialist-tool-surface-summary.json. The source confirms a startup static catalog
is authoritative for existing auth, not merely the API-key test path.

Limits remain explicit: this tested macOS/filesystem/profile is not universal
machine isolation; outside-root hardlinks/copies and metadata are not hidden;
same-user memory/environment compromise is outside the guarantee. Client-chosen
nonces reject exact-line replay, not stolen-token minting, and there is no separate
cryptographic server proof. In-app capability now stays in the parent rather
than being forwarded into Codex. Final atomic publication I/O is not universally
preemptible. SDK tracing must stay disabled in the app logger. Linux native
confinement is unavailable; Linux execution was not available here and is not
claimed. The offline exact surface proof and genuine subscription flow are
separate observations, not an intercepted live subscription request inventory.

Manager owns core guarded-helper/desktop adapter/native acceptance, root lock
integration, personal push and independent Opus re-review. This worker did not
integrate to main or push remotely.
