# Bounded native preflight after hosted macOS timeout

The manager supplied hosted macOS Desktop run 41728961231, job 126240993654 at
main c9dfe9c: actual native acceptance failed after 2.16 seconds with "Native
confinement probe exceeded deadline", while four other acceptance tests passed.
The original failure is preserved in native-probe-ci-red.log. The old log has no
helper label, so it does not identify Python or any particular syscall. Prior
hosted macOS had passed; Linux Core 41728961320 passes the previous test fix.

## Measurements and deadline choice

`cargo run -p topics-runtime --example measure_native_probes` calls the actual
production sandbox command and bounded probe, using synthetic temp roots only.
It measures eight sandbox true launches and eight Python isolated imports of the
same ctypes/json/os/sys modules per phase, then a complete actual preflight.
The loaded phase has two owned yes processes whose output is discarded and which
are killed/reaped on scope exit. It does not purge OS caches; "cold" means first
invocation in this measurement process, not a simulated pristine hosted runner.

Before the patch all 34 samples passed locally: unloaded/loaded sandbox launch
maxima 13.30/13.64 ms, Python-import maxima 100.35/81.76 ms, full preflight
971.88/1130.53 ms. These measurements do not reproduce the hosted stall and cannot
establish a hosted cold-start percentile. The hosted timeout is a right-censored
observation at the old two-second limit. A controlled healthy helper delayed 2.2
seconds records a separate behavioral RED against the old implementation, then
passes with its actual success status and exact output after the patch.

The new default is five seconds per helper, a modest startup margin beyond that
observed two-second failure rather than a huge retry window. There is also a
shared **30-second aggregate preflight budget**, clamped into every helper's
remaining deadline, so repeated slow steps cannot multiply five seconds across
all controls. This is an engineering safety margin, not a claim measured hosted
latency is always below five seconds. A host slower than these finite budgets
still fails closed. There is no retry or timeout-as-success path. Errors identify
only a fixed helper executable/syscall class and give responsiveness/retry advice,
never selected paths, captured error output or note content.

## Process and pipe lifetime

Owned child stdin/stdout/stderr are nonblocking Unix descriptors. Polling drains
stdout and stderr together, retaining at most 8192 bytes each while discarding
excess, and caps work per poll so an infinite writer cannot starve the deadline.
Input is limited to 4096 bytes before spawning and written within the same
budget. Successful completion requires direct-child exit and EOF on both pipes.
A descendant retaining a pipe therefore cannot make an unbounded reader-thread
join; the old blocking output-reader threads are removed.

Each helper starts in its own process group. Timeout or pipe/exit-observation
error sends SIGKILL to that group, kills the direct child and waits/reaps it before
returning failure. Ordinary inherited-pipe descendants are covered by group
termination; this is not a universal claim about children deliberately escaping
the group. Kernel spawn/filesystem/reap syscalls cannot be universally preempted
in an unresponsive OS. No detached output threads or hidden per-step retries exist.

All static profile text, literal ancestor unlink/clone denial rules, synthetic
fixtures, canonical/symlink read and write checks, clone and swap positive controls,
permission-specific negative controls, sibling controls and final selected-root
sandbox launch remain unchanged. native-probe-controls-preserved.json verifies
byte-identical profile/ancestor code and the identical normalized preflight body
after removing only budget arguments/declaration and rustfmt changes. Runtime
still completes preflight before bridge creation, thread creation or selected-byte
transmission; an error cannot advance to inference.

## Regression checks

Five added tests cover healthy startup beyond the old cutoff, short-deadline
kill/reap, inherited output pipe timeout, pre-spawn oversized-input rejection plus
bounded input roundtrip, and expired/remaining aggregate budget enforcement.
The cleanup tests use 500 ms deadlines, assert failure well below two seconds,
and verify direct PID is absent and waitpid reports ECHILD (already reaped).
The inherited-pipe helper exits successfully itself while a child retains its
pipes, and must still return a deadline error. Expired aggregate budget proves
no marker-producing process is launched; remaining budget clamps the next helper.
Existing permission-helper-failure checks remain. The public default timeout
regression now tests the five-second deadline with the same one-second wall-clock
allowance previously used for the two-second deadline; it continues to require
an error. These tests do not relax any actual confinement assertion.

On final source before freeze: six focused isolation tests and **54 ordinary
runtime tests pass**, none ignored (31 library, five acceptance/native confinement,
two native clone, four privacy, 11 scripted Runtime, one proxy lifecycle).
Formatting and all-target/all-feature clippy with warnings denied pass.

Commands use CARGO_INCREMENTAL=0:

```
cargo test -p topics-runtime --lib isolation::tests
cargo fmt -p topics-runtime --check
cargo clippy -p topics-runtime --all-targets --all-features -- -D warnings
cargo test -p topics-runtime
cargo run -p topics-runtime --example measure_native_probes
```

Evidence: native-probe-ci-red.log, native-probe-startup-red.log,
native-probe-timing-before.json, native-probe-timing-after.json,
native-probe-lifecycle-green.log, native-probe-clippy.log,
native-probe-fmt.log, native-probe-suite-green.log and
native-probe-controls-preserved.json. Source-frozen local native timing chronology
will accompany the final evidence receipt. No model calls or auth/config/note
content reads were performed; all fixtures and helper input are synthetic.

This is a production lifecycle/deadline change. Manager must obtain focused
cross-provider review and rebuild/run the final synthetic native live integration
on the final source before integration, then rerun hosted macOS; this worker does
not claim a local measurement is a new hosted macOS pass. Root manifests, public
Runtime/CommitPermit APIs, provider configuration, tool surface and core/UI remain
unchanged, and no push is performed here.

## Final local source chronology

RED/baseline commit: `f16a770a6e036e4ded1d1e854c3bd8c0ae3572a1`.
Frozen production fix: `4e01d9ff875be20a0ae4fe754b37cf1c026b933e`.
The final local synthetic OS measurement ran 2026-11-03 22:37:04.892–22:37:08.912
UTC against that unchanged source; only evidence/docs changed afterward.
Source scope (runtime src/tests/examples/crate manifest) SHA256:
`ee9f06be7d20f1e5106d39c22fd51c916444396ef5de1fcf321fd2a866d19401`.
Measurement executable SHA256:
`c7712e1467bb31722326dfed4720941913b1d5cf6a5e62e2fe9f4c1c072d4321`;
its recorded mtime is newer than the final source edit, and it was unchanged
through execution. The executable may be reused by cargo after source freeze;
this record does not claim a fresh native model rebuild after commit creation.

All 34 final samples pass. Unloaded/loaded sandbox-launch maxima were
13.10/12.88 ms, Python-import maxima 105.61/81.11 ms, and actual complete
preflight 988.17/966.63 ms. native-probe-timing-after.json contains every sample;
native-probe-final-chronology.json records source and executable identities and
zero model calls. These remain local measurements, not a reproduction or pass of
the hosted macOS cold runner. The manager's fresh native live rebuild and focused
cross-provider review are expressly still required for this production change.
