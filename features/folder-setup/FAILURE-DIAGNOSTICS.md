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

## Limits and readiness

These two reasons diagnose future locally observed failures. They do not recover
the initiating cause of the genuine native attempt whose artifacts retained
only `runtime_unavailable`. All production caps, timeouts, cancellation fences,
retry policy and buffer discard semantics stay unchanged. Scripted regressions
do not establish genuine native acceptance; independent review is still pending.
