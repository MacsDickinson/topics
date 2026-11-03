# Bounded runtime re-review patch

This patch follows Opus re-review of the exact two-tool production Runtime/SDK
route; it does not replace the accepted architecture or public host interfaces.
The manager preserves the independent report at
`docs/reviews/topic-workspace-runtime-rereview.md`. Earlier test counts in
REVIEW-REMEDIATION.md describe the earlier source revision.

| Review | Change and regression |
|---|---|
| R1 ancestor clones | Static profile denies file-clone on every canonical ancestor literal, in addition to unlink/rename denial. Native clonefile and fclonefileat tests first prove a real unsandboxed copy, then require EPERM/EACCES and no destination, and prove an allowed sibling clone succeeds. Both original bypasses failed before implementation in rereview-clone-red.log. Preflight runs both syscalls against two synthetic ancestors. |
| R2 ambient trace/auth/CA/destination environment | All installed-runtime version checks, app-server children and native preflight helpers start with env_clear. Only existing absolute canonical HOME/CODEX_HOME, PATH, validated TMPDIR and bounded locale cross the boundary; RUST_LOG is forced off. No ambient variable enumeration or credential-file reads. A real Runtime scripted-child regression fails before implementation and passes afterward with hostile trace, custom CA, endpoint and fake auth values. |
| R3 duplicate keys | Guard reserializes the validated JSON Value before SDK forwarding. A five-case duplicate envelope/params matrix poisons a previously buffered candidate on forbidden last-wins calls. |
| R4 vacuous helper denial | RENAME_SWAP and clone helpers must exit successfully with valid syscall result JSON. Unsandboxed positive controls must actually execute. Denial requires result=-1 and EPERM/EACCES; missing/broken helper, malformed JSON, ENOENT/unsupported result and a successful operation fail closed. |
| R5 actionable errors | Static safety codes map to sanitized remediation for inherited AGENTS instructions, conflicting features/privacy controls and custom provider/endpoint overrides. Unknown codes give restart/configuration guidance without raw frames or configuration. |
| R6 cancellation during publication | CommitPermit revocation uses try_lock and returns false immediately while the final publication fence is busy; a bounded test holds the action and verifies nonblocking revocation. A final atomic OS rename remains nonpreemptible. |
| R7 second commit gate | Failed state is checked again alongside cancellation when installing the preparation permit; a targeted test rejects failed state without installing it. |
| R8 source chronology | Final native verification is run once by run-final-native.py after a source-freeze commit, with source and executable hashes, build timestamps and unchanged-source checks. Final results are appended only after that run. |
| R9 stale route branches | Removed in-app requirements for an unused proxy binary and stale Topics-native-MCP startup polling/progress allowances. The external stdio proxy remains supported and independently tested. |
| R10 conservative retry | Retained: a transient protocol/tool failure discards the entire run and requires a new authorized run, rather than silently replaying a partial turn. |

Before source freeze: `cargo fmt -p topics-runtime --check`,
`cargo clippy -p topics-runtime --all-targets --all-features -- -D warnings`,
and `cargo test -p topics-runtime`, all with CARGO_INCREMENTAL=0, pass.
The deterministic suite contains 45 passing tests, no ignored tests:
22 library, five acceptance/native boundary, two direct native clone, four privacy,
11 full scripted Runtime and one proxy lifecycle. Linux ordinary tests retain
native cfg boundaries and do not claim macOS proof.

The final genuine tests use only temporary synthetic notes/app-data and existing
runtime-managed subscription auth. The application never reads/copies tokens.
The final harness passes fake hostile trace/CA/auth/endpoint/proxy/OTEL values to
the live test process to verify the production runtime strips them; no actual
custom destination or notify service receives synthetic selected bytes.

Remaining limits: existing hardlinks/copies already outside the root and a
malicious same-user memory/environment reader are outside this path boundary.
Unsupported filesystem/helper controls block analysis. Publication has a short
cooperative atomic action and cannot promise universal OS syscall preemption.
No desktop/core acceptance or release approval is claimed by this worker.

## Final frozen-source verification

Source-freeze commit: `1430eba0c9644abfe67851bc9ed1f3b5167acbcc` at
2026-11-03 06:07:35 UTC. The final harness ran from 06:07:38.822 to
06:08:22.467 UTC, after the last product/test source edit. No product source or
test changed after freeze; the final evidence commit has the same runtime source
as this tested commit. `rereview-final-chronology.json` records SHA256 of every
tracked runtime src/test file plus crate manifest in one ordered source digest:
`669c6f5c66893f4eac6892903ee0658c597bcf168dab4d22d5a6756791037609`.
All three test executables were built after source freeze, hashed before/after
execution, and unchanged. The live binary SHA256 is
`1abbb03a1b77fb75611591897842b3e6ece36b90fce9b34105d73c8db6818b59`.
The JSON includes the separate schema/config executable hashes and nanosecond
mtimes. Root manager-owned files were not committed here; build-time SHA256:
Cargo.toml `221b13f8ba1ef245d0f69b50ef76ad4c60b6a14af611d0c7b9619df4c9c10062`,
Cargo.lock `4fc5dc5ecd4264dfbfe118107ef3f35b98f558bffd1ac315dd2240fc302d1969`.

Final results: **45 deterministic tests + four explicit native tests pass, none
ignored**, plus fmt and all-target/all-feature clippy with warnings denied.
The four explicit tests are schema (16 pinned projections), isolated effective
config, and two genuine subscription tests comprising three synthetic turns:

- Discovery: one successful scoped read, three cited candidates, one pending
  proposal event, one completed terminal; source unchanged.
- Correction: one successful scoped read, one typed correction citing the persisted
  user statement and accepted revision, one proposal event, one completed terminal;
  source unchanged.
- Adversary: successful real scoped baseline read required, no private randomized
  sentinel returned, zero staged candidates, one terminal, source unchanged.

This was **one final genuine native test invocation, no live retries**; it passed
with ten fake hostile trace/CA/auth/destination/proxy/OTEL parent variables. The
forbidden trace path and keylog were never created. Only sanitized counts/status
are retained, not raw prompt, model/account/config/capability or auth payloads.

Exact final commands:

```
CARGO_INCREMENTAL=0 cargo fmt -p topics-runtime --check
CARGO_INCREMENTAL=0 cargo clippy -p topics-runtime --all-targets --all-features -- -D warnings
CARGO_INCREMENTAL=0 cargo test -p topics-runtime
python3 crates/topics-runtime/evidence/run-final-native.py
```

The Python harness builds `cargo test -p topics-runtime --features
native-live-tests --test live_codex --test schema_pin --test config_override_native
--no-run --message-format=json` with CARGO_INCREMENTAL=0, then invokes each exact
built binary with `--nocapture --test-threads=1`. Its environment is built by
per-name safe lookups; it never reads auth files or enumerates ambient credentials.

Evidence: rereview-clone-red/green.log, rereview-environment-red/green.log,
rereview-deterministic-green.log, rereview-final-clippy.log,
rereview-final-fmt.log, rereview-final-schema_pin.log,
rereview-final-config_override_native.log, rereview-final-live_codex.log,
and rereview-final-chronology.json. Historical logs remain labeled by revision.

## Public API remains unchanged

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
pub type EventSink = Arc<dyn Fn(WorkspaceRunEvent) + Send + Sync>;
impl Runtime {
    pub fn new(config: RuntimeConfig) -> Self;
    pub fn status(&self) -> RuntimeStatusResult;
    pub fn analysis_start(&self, request: RunRequest, host: Arc<dyn HostTools>,
        events: EventSink) -> WorkspaceResult<String>;
    pub fn chat_start(&self, request: RunRequest, host: Arc<dyn HostTools>,
        events: EventSink) -> WorkspaceResult<String>;
    pub fn cancel(&self, root_id: &str, run_id: &str) -> bool;
}
```

The accepted core/desktop guarded adapter continues using
`service.stage_proposal_guarded(request, |proposal, action| {
permit.publish(vec![proposal.id.clone()], action) })`. No new domain types,
dependencies, approvals/apply capabilities, or store coupling were introduced.
Ready for targeted independent confirmation of R1/R2 and chronology; manager
retains integration/push ownership.
