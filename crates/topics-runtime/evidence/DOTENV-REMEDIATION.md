# R2b Codex dotenv launch guard

This bounded follow-up closes the one remaining finding in the manager-preserved
`docs/reviews/topic-workspace-runtime-final-confirmation.md`. The previously
accepted tools, SDK, sandbox, catalog, lifecycle and public API are unchanged.

Codex reloads effective CODEX_HOME/.env after the cleared child environment.
`environment::apply_from` now validates HOME and optional CODEX_HOME as before,
then calls only `symlink_metadata` on effective CODEX_HOME/.env, defaulting to
canonical HOME/.codex/.env. Only NotFound permits startup. Every existing entry
(file, directory, socket or symlink, including a dangling link) and all other
metadata errors reject before spawn. The common environment path guards installed
Codex version checks and both app-server launches as well as preflight helpers.
No dotenv content is opened, parsed, logged, copied, moved or deleted.

The actionable sanitized error explains that this runtime session does not
support Codex dotenv configuration and asks the user to choose an appropriate
existing CODEX_HOME without that entry. It includes no user path, entry content,
credential name or value. A metadata-only check of the expected actual runtime
home found no dotenv entry; no actual file contents or auth were inspected.

Synthetic regression RED against parent `09b484e` is saved in dotenv-red.log:
file/directory rejection, live/dangling symlink rejection and metadata-error
rejection fail before the implementation; the normal absent-entry control passes.
GREEN covers both default and explicit homes, absent default home without creating
it, absent existing default/explicit homes, and unchanged existing HOME/CODEX_HOME
validation. ENOTDIR provides a deterministic metadata-error case even when tests
run with privileges. Six focused environment tests pass, including the prior
allowlist and invalid-path checks.

Before freeze: fmt check, all-target/all-feature clippy with warnings denied, and
49 deterministic tests pass, none ignored (26 library, five acceptance/native
boundary, two clone boundary, four privacy, 11 scripted Runtime, one proxy).
Commands, all cargo checks with CARGO_INCREMENTAL=0:

```
cargo test -p topics-runtime --lib dotenv_             # saved RED before guard
cargo test -p topics-runtime --lib environment::tests # six GREEN
cargo fmt -p topics-runtime --check
cargo clippy -p topics-runtime --all-targets --all-features -- -D warnings
cargo test -p topics-runtime
```

Evidence before freeze: dotenv-red.log, dotenv-focused-green.log,
dotenv-fmt.log, dotenv-clippy.log, dotenv-deterministic-green.log.
Final native source chronology will be recorded after source freeze and one
synthetic subscription verification invocation, using the existing harness with
`--output-prefix dotenv-final` to preserve prior accepted evidence.

This is a fail-closed prelaunch metadata gate, not a guarantee against a malicious
same-user process replacing the entry between check and Codex dotenv import.
Existing outside-root hardlink/copy and final atomic-publication nonpreemption
limits remain documented in REREVIEW-REMEDIATION.md. No new capability, domain type,
external endpoint, login/auth access, root manifest change or push is added.

## Final source-frozen native result

Source patch: `8ae44f9cd5af2a514f1639def47996714cc9a4a0`, committed
2026-11-03 07:04:30 UTC. Final verification ran 07:04:42.337–07:05:48.151 UTC,
with no subsequent runtime src/test/manifest changes. The final evidence commit
therefore contains the identical tested product source. Source-scope SHA256:
`707dc53ffaa6a65dd086e13dedd55ba75b930806c279339ee7aa8000eafcce63`.

`python3 crates/topics-runtime/evidence/run-final-native.py --output-prefix
dotenv-final` passed all four explicit native tests in one invocation, with no
live retry: pinned schema (16 projections), isolated effective config, genuine
subscription discovery/correction and genuine adversarial isolation. This adds
to the 49 passing deterministic tests; none are ignored. Discovery yielded three
cited candidates and one proposal event after a real scoped read; persisted
statement correction yielded one typed candidate and one proposal event after a
real scoped read. The adversarial test required a successful real baseline tool
read, returned no randomized private sentinel, staged zero candidates and emitted
one terminal. All synthetic source bytes were unchanged.

The final model-test executable SHA256 is
`c7857afb3801bf7a1eeec71ae49f697981adfe7c66a1eb2aad65919c4b97f441`.
`dotenv-final-chronology.json` records all three executable SHA256 and mtimes,
source commit/digest and unchanged-source checks. Executables were built after
the last source edit and after the freeze commit. The final live run retained the
prior harness's ten fake hostile trace/CA/auth/endpoint/proxy/OTEL parent variables;
production children stripped them, and no trace or keylog path was created.
No model/account/config/capability/auth payloads were captured or logged.

Final native evidence is kept separately in dotenv-final-schema_pin.log,
dotenv-final-config_override_native.log, dotenv-final-live_codex.log and
dotenv-final-chronology.json; accepted previous logs are preserved. Public
Runtime new/status/analysis_start/chat_start/cancel, HostTools call and
guarded commit_candidates, CommitPermit publish, EventSink and RunRequest are
unchanged from the API recorded in REREVIEW-REMEDIATION.md. Ready for bounded
independent R2b-only confirmation; integration and push remain manager-owned.
