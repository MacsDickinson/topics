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
