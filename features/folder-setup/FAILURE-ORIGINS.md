# Local failure origin diagnostics

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
