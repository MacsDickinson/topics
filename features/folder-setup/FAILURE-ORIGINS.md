# Local failure origin diagnostics

## Valid-pair provenance repair followup

Independent review at `fefcfa1` reproduced a real public host-seam bug:
generic `HostTools::commit_candidates` errors with an exactly valid preflight
pair were promoted to runtime provenance. This is a diagnostic integrity bug;
the review demonstrated no current model route, privacy leak or publication
authority change. The genuine initiating cause is still unknown.

Semantic RED: `cargo test --offline -p topics-runtime --test scripted_runtime
arbitrary_host_valid_pair -- --nocapture` returned **0 passed, 1 failed** on
`fefcfa1`, exporting `preflight/version_check_failed` where the observed boundary
requires `publication/preparation_failed`. Only a local scripted completed empty
turn and synthetic source/home were used; evidence:
`/tmp/topics-host-provenance-red.txt`. This is the exact valid-pair spoof,
not merely an unknown or extra-key shape test.

The genuine 25.1s followup on built `4d3420e` failed with one attempt, zero
processed sources, empty graph and unknown diagnostics. This instrumentation
does not establish that failure's initiating cause. No further provider call is
authorized or performed here.

## Frozen finite contract

Existing `ApiError.details` is null (unknown) or exactly `{phase, reason}` with
one of the pairs below. Store reconstructs constants, rejects extra/missing
keys, wrong types, unknown and mismatched pairs; no new wire DTO.

| phase | reason |
| --- | --- |
| `preflight` | `version_check_failed` |
| `preflight` | `layout_denied` |
| `preflight` | `native_denied` |
| `startup` | `setup_failed` |
| `discovery` | `discovery_failed` |
| `discovery` | `discovery_deadline` |
| `transport` | `transport_failed` |
| `transport` | `read_deadline` |
| `transport` | `protocol_denied` |
| `mcp` | `session_failed` |
| `mcp` | `inventory_denied` |
| `mcp` | `request_failed` |
| `mcp` | `request_deadline` |
| `callback` | `authorization_denied` |
| `candidate` | `schema_denied` |
| `host` | `validation_denied` |
| `host` | `call_deadline` |
| `host` | `host_failed` |
| `execution` | `validation_denied` |
| `execution` | `execution_deadline` |
| `publication` | `preparation_failed` |
| `publication` | `preparation_deadline` |
| `turn` | `model_terminal_failed` |
| `publication` | `scope_not_read` |

These are locally generated boundary observations, never classifications of raw
provider/SDK messages. The first bridge failure wins over a later generic SDK
denial; model-facing denials are unchanged and contain no diagnostics. No
frames, stderr, error strings, candidate values, source paths/bytes, private
context, configuration or auth are recorded in details. No phase is inferred
from elapsed time. Native preflight's combined reason does not distinguish its
helper timeout from other local preflight rejection.

## Semantic RED

`cargo test --offline -p topics-setup --test protocol local_failure_origin --
--nocapture` before behavior changes: five controlled cases expect distinct
pairs through real Runtime/MCP/SetupCoordinator/WorkspaceService, each preserving
zero processed/no publication/unchanged notes and rejecting sentinel bytes.
Cases exercise malformed canonical candidates at the gate, valid-shaped
candidates denied by actual store source validation, malformed process frame,
wrong scope callback and wrong executable version.

RED result: **0 passed, 5 failed**, each `left: None` versus its expected
distinct pair. The initial host-denial fixture had a Python nesting error; it
was corrected and all five RED cases were rerun against pre-fix product source
before accepting this evidence (`/tmp/topics-origin-corrected-red.txt`).

RED commits: `44cf779` and corrected-fixture evidence `fdbe61a`; both precede
the behavior commit and preserve `46dc9ee` in history. The correction matters:
the accepted host RED is a real well-shaped candidate denied by source passage
validation, not a synthetic child exception.

## Implementation boundaries

- Private runtime `Failure` has 24 variants and no serialization trait. The
  existing `RunState` gains one optional typed first failure; no candidate,
  source, error text, provider payload or diagnostic log is added to state.
- `SessionGate` keeps the same public validation and String results. A private
  typed result distinguishes its actual authorization and candidate-schema
  branches before SDK normalization. Host inner error, blocking worker failure,
  five-second timeout and invalid/oversized response record local variants.
- First bridge origin survives later poisoned-run/SDK rejection. Execution
  seals its typed result before teardown; final error details are reconstructed
  from the enum, while cancellation and published outcomes retain their existing
  precedence. A stopped or terminal generation cannot append durable results.
- Transport tags existing spawn/write/read/decode/request rejection boundaries.
  Startup, native/version/layout checks, configuration discovery, MCP session
  and inventory, execution and guarded publication annotate their existing
  boundaries. No new source/config/auth reads or subprocesses are added.
- The existing coordinator passes errors to the store. The store reconstructs
  the exact constant pair and keeps its existing generic code/action message;
  no new DTO, canonical source change, store dependency or persisted job schema.

`transport/protocol_denied` identifies the local parser/request adapter boundary,
including a well-formed server error response; it does not prove malformed model
output or a provider protocol defect. `host/host_failed` intentionally groups
local worker/capacity/response failures. MCP setup and inventory failures are
grouped. All categories describe enforcement stages, not upstream causes,
auth/transience, source truth or model quality.

## GREEN verification

Environment: `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=/tmp/topics-setup-target`; runtime
suites use an empty temporary `CODEX_HOME`, setup tests isolate each runtime
child again. Only scripted temporary notes and absolute app data are used;
provider calls: **zero**. Main/genuine targets were not used.

- `cargo test --offline -p topics-runtime -p topics-setup`: **64 runtime + 29
  setup passed**, native-live feature disabled; original 100-document, copy,
  privacy, retry/deduplication, cancellation/publication, root-switch and
  restart/SQLite-rebuild regressions remain green.
- `cargo test --offline -p topics-store --lib --test setup_service --test
  setup_amendments`: **42 passed** (12 unit + 23 service + 7 amendments).
- Sanitizer integration checks 86 persisted input shapes, including all 24
  allowed pairs and extra/private fields; reopen preserves only safe constants,
  notes stay unchanged and a late second failure update is rejected. Unit
  tests cover all 576 pair combinations and corresponding extra-key rejection.
- Five owned protocol RED cases are GREEN, each one attempt/call with zero
  processed sources, no graph/proposals/checkpoints, unchanged source bytes and
  no retained private error sentinel. The original two-pair cases stay GREEN.
- Real SDK bridge test validates a buffer, then tests candidate schema rejection
  versus host denial: distinct first enum values, identical generic MCP error,
  no `data`, private sentinel or reason on the wire, and discarded buffer. A
  subsequent denied callback cannot replace the first reason.
- Deadline tests use actual unchanged **5s host, 6s SDK and 40s process-read**
  waits with local synthetic workers and bounded process cleanup. Existing
  real-permit publication timeout test additionally requires
  `publication/preparation_deadline` and rejects later publication. Discovery
  and execution use the same bounded helper with unchanged production **15s /
  150s**; its deterministic unit test uses a 5ms pending synthetic future.
- Following the final teardown-origin sealing and preservation of the original
  timeout human message, the 15 scripted runtime tests, five service-origin
  cases and deadline helper test were rerun. Full evidence is in
  `/tmp/topics-origin-final-tests.txt`; corrected RED is recorded above.
- Package `cargo fmt -p topics-runtime -p topics-setup -p topics-store --
  --check`, warning-denied all-target clippy with
  `--features topics-store/test-support`, and `git diff --check` pass. The first
  clippy run caught test module placement; it was moved, then clippy passed.
  Branch-local pre-existing BDD formatting remains outside this lane; the
  coordinator owns integration formatting and has its upstream correction.

## Bounds, unknowns and handoff

All production bounds remain unchanged: version 5s; native helper 5s and aggregate
30s; discovery 15s; process read 40s; execution 150s; SDK call 6s; host 5s;
publication preparation 5s; coordinator terminal receive 200s; bridge lifetime
180s; stop wait 3s; frames 512KiB, host responses 256KiB, output 2MiB,
64 runtime candidates/calls and existing canonical 200-candidate/2MiB ceiling.
Tools, validation order/conditions, retries, capabilities and approval fences
are unchanged. Tests exercise diagnostics, not relaxed success criteria.

Unmapped early input failures, poisoned synchronization/outer worker panics and
coordinator missing-terminal paths can still remain unknown. Known broad phases
do not expose internal subprocess/provider reasons. No initiating cause can be
recovered for the already-failed genuine attempt, and no native/live/hosted
acceptance claim is made. No UI, BDD/shared bindings, core source, manifests,
lockfile, notes or personal configuration were edited.

Scenario ownership: approved `continuity.feature`, "Failed inputs do not
masquerade as complete folder coverage", plus its cancellation/restart safety
scenario; protocol tests provide real orchestrator/service boundary evidence.
Independent Claude review remains pending authentication. Before any separately
authorized genuine attempt, the harness owner must update
`apps/desktop/scripts/live-setup-observation.mjs` from its two-pair allowlist to
this frozen 24-pair contract; otherwise its safe artifact drops new pairs back
to unknown. Existing two static UI explanations remain; other pairs may use the
generic UI fallback. Final assembled source/harness review and genuine acceptance
remain the coordinator's next steps.
