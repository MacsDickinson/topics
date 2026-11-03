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
