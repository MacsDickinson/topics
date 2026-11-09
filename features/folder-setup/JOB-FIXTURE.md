# Synthetic native-test protocol fixture

This helper is development/test-only. The production setup library does not
import it. It implements a scripted pinned app-server child, never a provider
client or alternative production backend. Use only caller-created synthetic temp
notes, disjoint absolute app-data and synthetic runtime home/configuration.

Render into an **existing empty absolute synthetic directory**:

```text
cargo run -q -p topics-setup --example render_fixture -- /absolute/empty-temp-dir graph_claim
```

The example prints one JSON object with `executable`, `mode_file`, `log_file` and
`mode`. Pass the executable path to the existing desktop native-test harness as
`TOPICS_CODEX_PATH`. Retain the real Runtime/HostTools/MCP/containment guards;
use the harness's synthetic HOME/CODEX_HOME and app-data isolation. Put fixture
output outside the protected synthetic notes root. The renderer does not read or
modify HOME, Codex configuration, tokens, providers or source notes.

The output contains `scripted-codex` (executable) and `mode`; the child creates
`transmissions.jsonl` only when a turn calls its exact scoped source tool. Log
entries contain only that synthetic turn's prompt and returned scoped context.
Mode changes for fail/retry tests are explicit writes to this synthetic mode file.
Existing nonempty directories, symlink output directories, relative paths and
unknown modes are rejected; existing files are never overwritten.

`tests/support/render.rs` is shared by the example and the real protocol tests.
It uses serde_json to quote absolute paths and serializes the actual
`topics_runtime::protocol::DISABLED` constant. The native harness needs no second
JavaScript flag list or custom path escaping. Spaces, apostrophes and Unicode
paths have been smoke-tested.

Modes used by the native harness:

- `graph` remains unchanged: two synthetic topics and one cited leads connection.
- `graph_claim` adds one canonical cited Claim on Synthetic Alex A42 with
  predicate `responsibility`, value `leads Ledger`, all required nullable fields
  present, and passage evidence from the real scoped source response. Sources
  for this fixture should say “Alex A42 leads Ledger.”
- `empty`, `ambiguous`, `split_calls`, `fail_first` and `hold_second` exercise
  focused protocol behavior; `pronouns` and `historic` support isolated correction
  and dated-disagreement regressions. They are synthetic test modes only.

The fixed graph IDs end in `000001` (Alex), `000002` (Ledger), `000003`
(connection), `000004` (logical responsibility claim) and `000005` (its revision),
all under the `10000000-0000-4000-8000-` prefix. The claim/revision are separate
canonical UUIDs. Nothing is accepted by a scripted terminal success; human group
save and subsequent correction still use the real canonical decision APIs.

Evidence:

- `job-graph-claim-red.txt`: the real service/protocol test failed on the missing
  claim before the new mode was added.
- `job-graph-claim-green.txt`: the same test passes with cited draft-only output.
- `job-fixture-renderer-smoke.txt`: quoted-path construction, executable fake
  version and nonempty-output refusal pass.
- `job-renderer-default-graph.txt`: the unchanged graph mode still processes
  100 documents in five bounded turns, preserving one graph and source coverage.

This is scripted protocol evidence. It does not claim genuine-model, native UI,
user-authentication or hosted acceptance; those remain manager-owned.
