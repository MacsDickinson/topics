# Linux bridge CI synchronization correction

Manager-provided main source: `51caaa6`; Linux Core run `41602005434`, job
`125802721782`. The exact original failure excerpt is preserved in
bridge-ci-red.log from `/tmp/topics-m1-core-ci-job.log`: 25 of 26 library tests
passed, and review_sdk_ignored_frames_poison_previous_buffer observed failed=false.
This worker uses the existing runtime worktree without importing unrelated final
core/UI changes or modifying manager-owned manifests.

The test previously wrote each invalid frame, slept 40 milliseconds, snapshotted
failed, then explicitly closed the bridge. Neither successful TCP write nor an
elapsed timer acknowledged server processing. Thus closing the bridge could
cancel a not-yet-processed frame and fail the negative assertion. The precise CI
scheduling/TCP delay was not instrumented; the recorded failure is consistent
with that test synchronization race, not evidence that the guard accepted a
malformed envelope.

The fix is entirely inside the existing `#[cfg(test)]` module. After writing the
same eight invalid frames, the test now requires peer UnexpectedEof within three
seconds, before it closes the bridge or snapshots the shared state. This is the
observable connection termination produced when the guard rejects the frame.
The exact original assertion (`SDK ignored invalid envelope without poison`)
is retained. An additional assertion requires the previously seeded candidate
buffer to be empty. The test fails if the connection stays open, any SDK response
is received, the socket fails differently, poison is absent, or the buffer
survives. An accidental early connection close alone cannot pass the test.
This replaces elapsed-time guessing with a bounded transport acknowledgement;
it neither adds a longer sleep nor weakens an authorization/privacy check.

Production `validate_mcp_frame` rejects the matrix entries before SDK forwarding;
its rejection branch poisons synchronously before returning and dropping the
connection. Production code is byte-for-byte unchanged. The unchanged bridge
prefix before `#[cfg(test)]` has SHA256
`48ca07fcfe35b623d15097203a022939f704351d36ac4b64252480e47e9b32f8`.
No API, runtime launch, host authorization, SDK behavior, catalog, tool surface,
provider, core, UI, native containment or manifest changed.

Verification on this macOS worker:

- The exact focused regression passes.
- Fifty consecutive exact-test invocations pass all 400 invalid-frame checks,
  each requiring EOF, failed=true and empty buffer; repetition evidence includes
  executable SHA256 and the unchanged production-prefix assertion.
- All 49 ordinary runtime tests pass, none ignored (26 library, five acceptance,
  two clone, four privacy, 11 scripted Runtime, one proxy lifecycle).
- Format check and all-target/all-feature clippy with warnings denied pass.

Commands (cargo invocations use CARGO_INCREMENTAL=0):

```
cargo test -p topics-runtime --lib bridge::tests::review_sdk_ignored_frames_poison_previous_buffer
cargo fmt -p topics-runtime --check
cargo clippy -p topics-runtime --all-targets --all-features -- -D warnings
cargo test -p topics-runtime
```

Repetition invokes the compiled library test binary 50 times with
`--exact bridge::tests::review_sdk_ignored_frames_poison_previous_buffer
--test-threads=1`; every invocation is process-timeout bounded as well. Evidence:
bridge-ci-focused-green.log, bridge-ci-repetition-green.json,
bridge-ci-clippy.log and bridge-ci-suite-green.log. No new genuine model calls or
native live rebuild were made because production behavior did not change; prior
accepted native evidence is preserved. All fixtures are synthetic and no auth,
real notes, header or user configuration content was accessed.

Remaining confirmation: independent review of this test-only synchronization
change and a manager-owned Linux CI rerun. This macOS execution is not presented
as new Linux proof, nor is the exact Linux scheduling mechanism claimed measured.
