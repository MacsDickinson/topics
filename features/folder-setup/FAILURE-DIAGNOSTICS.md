# Bounded setup failure diagnostics

## Frozen human-facing contract

`SetupBatch.error` retains its existing generic code and human action message.
Its existing `ApiError.details` is either `null` (unknown) or exactly one of:

| phase | reason | Locally observed event |
| --- | --- | --- |
| `turn` | `model_terminal_failed` | Matching runtime terminal status is explicitly `failed` |
| `publication` | `scope_not_read` | Setup host refuses completion because its exact scoped sources were never read |

The store owns the persistence sanitizer. Only an exact two-key object with one
of these pairs is retained, reconstructed from constants. Unknown values, wrong
types, mismatched pairs, missing keys, extra keys and nested data become `null`.
No arbitrary message, provider error, stderr, model text, source metadata,
configuration or auth data is retained in these details. No inference is made
from broad error codes. Other existing failures remain unknown.

The desktop may render static explanations for these pairs with a generic
fallback. Model-facing MCP denials remain unchanged; these reasons are not tool
responses or model context. No wire DTO, protocol redesign, new retries, larger
limits or changed deadlines are part of this change.

## Semantic RED evidence (2026-11-09)

Before implementation, on parent `46dc9ee`, run with
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
CARGO_TARGET_DIR=/tmp/topics-setup-target`:

`cargo test --offline -p topics-setup --test protocol safe_reason -- --nocapture`

Result: **0 passed, 2 failed**, both at diagnostic equality (`left: None`):

- `failed_terminal_retains_safe_reason_and_discards_validated_candidates`:
  expected `{"phase":"turn","reason":"model_terminal_failed"}` after a
  scripted turn reads the scope, successfully validates candidates, then fails.
- `unread_scope_retains_distinct_safe_reason_without_publication`:
  expected `{"phase":"publication","reason":"scope_not_read"}` after a
  scripted turn completes without reading its sources.

Both already satisfied one attempt/call, zero processed, one failed source,
empty graph/proposal/checkpoint lists and unchanged source bytes. The synthetic
terminal envelope includes a private error sentinel which must not be retained.
Each runtime-bearing test runs in a child with a synthetic Codex home and a
scripted executable; no installed provider is called.

The store regression additionally exercises exact allowed pairs and malformed,
unknown, extra-key, nested and mismatched sentinel-bearing details, including
durable reopen and rejection of late failure updates.

`cargo test --offline -p topics-store --test setup_service
failed_batch_persists_only_exact_allowlisted_diagnostic_pairs -- --nocapture`
also failed before implementation: **0 passed, 1 failed**, at the first allowed
pair (`left: None`). RED tests were committed separately as `3088fae`.

## Limits and readiness

These two reasons diagnose future locally observed failures. They do not recover
the initiating cause of the genuine native attempt whose artifacts retained
only `runtime_unavailable`. All production caps, timeouts, cancellation fences,
retry policy and buffer discard semantics stay unchanged. Scripted regressions
do not establish genuine native acceptance; independent review is still pending.

Remaining blind spots intentionally stay unknown: startup/version/config/native
preflight failures, SDK/schema/host validation failures, deadlines and missing
terminal/disconnected worker paths. `model_terminal_failed` identifies a terminal
status, not its upstream cause, and says nothing about auth or transience.
`scope_not_read` identifies the local completion guard, not why the model did not
obtain its scope. These two pairs may not diagnose a future genuine failure.

## GREEN validation and handoff

With the same low-debug, nonincremental, dedicated target environment above:

| Command | Result |
| --- | --- |
| `cargo test --offline -p topics-setup --test protocol safe_reason -- --nocapture` | 2 passed; distinct exact pairs, zero coverage/publication, unchanged notes |
| `cargo test --offline -p topics-setup -p topics-runtime` | 24 setup tests (3 unit + 21 protocol), 58 runtime tests passed; native-live feature disabled |
| `cargo test --offline -p topics-store --test setup_service --test setup_amendments` | 30 passed (23 service + 7 amendments); sanitizer covers 14 input shapes |
| `cargo fmt -p topics-runtime -p topics-setup -p topics-store -- --check` | passed |
| `cargo clippy --offline -p topics-runtime -p topics-setup -p topics-store --all-targets --features topics-store/test-support -- -D warnings` | passed |
| `git diff --check` | passed |

Runtime-bearing suites used an empty temporary `CODEX_HOME`. The setup protocol
also isolates each test child with its own synthetic home. Full regressions
include 100-document coverage, human retry without duplicate graph records,
pause/root-switch publication races and restart/SQLite rebuild. The runtime
unknown/malformed/wrong-turn/exit cases also assert diagnostic `null`, while an
explicit failed terminal retains the known pair. Diagnostics and terminal error
sentinels are absent from model context. Store tests prove no sentinel retention,
durable reopen, and no second failure update on the same revoked attempt.

The first all-target clippy invocation omitted the existing store `test-support`
feature and could not compile unrelated fault-injection tests; the documented
feature-enabled command passed. Branch-local `cargo fmt --all -- --check` finds
pre-existing formatting in `crates/topics-bdd/src/bin/setup-bdd.rs`; this lane
does not edit BDD files. The coordinator confirms assembled integration already
contains that formatting correction and will recheck after applying this diff.

Scenario ownership remains the approved `continuity.feature` scenario
"Failed inputs do not masquerade as complete folder coverage" (failure/retry),
with the existing setup protocol tests owning runtime boundary evidence. No
Gherkin/shared binding, core source, UI, manifest or lock changes are included.
The desktop owner separately renders static explanations from the frozen pairs.
Independent review targets the final branch head; genuine provider retry remains
held for reviewed diagnostic/harness sources. Original `46dc9ee` stays in history.
