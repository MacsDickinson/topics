# Review: native helper deadline/lifecycle change (Topics)

Reviewed read-only: worktree topics-m1-runtime, diff 2c6d0bc..638ebaa (fix 4e01d9f, RED f16a770, final 638ebaa). Reviewer: Claude Sonnet 5.5. Scratch target dirs only; Cargo.lock/Cargo.toml byte-unchanged (cmp verified).

## Commands / results
- `CARGO_INCREMENTAL=0 cargo test --locked --offline -p topics-runtime` (scratch CARGO_TARGET_DIR): 31 lib (incl. 5 new isolation tests) + 5 acceptance (native_root_canonical_symlink_unicode_quotes_denied_sibling_allowed passes) + 2 clone_boundary + 1 proxy_lifecycle + 4 review_privacy + 11 scripted_runtime = 54 pass, 0 ignored.
- Controls-unchanged: normalized preflight body of 2c6d0bc vs 638ebaa (strip `&budget`, budget decl, whitespace, trailing commas) -> identical md5 a70216ea...; profile/ancestor-rule region byte-identical. Diff of preflight shows only the budget argument threaded through every call.
- Scratch probe crate (outside source, path dep, synthetic /bin/sh only):
  - 300x child exits immediately without reading 64B stdin: 0 errors.
  - `yes` flood: fails at 5.0s deadline (error, not success).
  - 5MB stdout + 100KB stderr then exit: OK in ~1s, captured capped at 8192 each.
  - Background descendant holding pipes, shell exits 0: deadline error at 5.0s, descendant killed (0 survivors) via group SIGKILL.
  - Descendant that setsid()s away holding pipes: deadline error at 5.0s, descendant survives (documented, accepted limit).

## Findings against the requested checks
- No confinement control weakened: only budget plumbing + label wrapping in preflight; same helpers, args, positive/negative controls, final selected-root launch. Timeout/IO/exit-observation errors are all `Err`, propagated by `?`; no retry, no timeout-as-success.
- Success requires direct-child exit AND EOF on both pipes AND stdin fully written/closed. Output capped 8192/stream (excess drained and discarded), input capped 4096 before spawn (oversized test: no process launched).
- Deadline: per-helper `min(remaining aggregate, 5s)`; expired aggregate returns before spawn; saturating arithmetic; sleep clamped to remaining. Fail-closed.
- Cleanup: process_group(0); stop_probe does SIGKILL to -pid, child.kill, child.wait on every error path after spawn; success path has already reaped via try_wait. Blocking reader threads removed, so no unbounded joins. `bounded_probe` is only used by preflight, tests and the example (Codex launch in lib.rs uses `isolation::command` only), so process-group/env behaviour of the real session is unchanged. Env still applied via environment::apply (env_clear, allowlist); no auth/tool-scope change. Error text uses only fixed exe/mode labels.
- preflight still runs before bridge/thread/selected bytes (lib.rs:256).

## Non-blocking observations (no blockers)
1. EPIPE on stdin write is now a hard error (previously ignored). A helper that exits without reading non-empty input before the first write would yield "pipe I/O failed" instead of its exit status. Fail-closed, and 300/300 quick-exit probe runs did not hit it; only /usr/bin/tee gets non-empty input in preflight. Possible rare flake on a very loaded host; consider treating BrokenPipe as "stop writing" and relying on exit status. Not a safety issue.
2. stop_probe kills `-pid` even after the direct child was already reaped (inherited-pipe case). POSIX keeps the pgid reserved while descendants exist, so this is safe in the tested case; a pid-reuse hit is theoretically possible only if the group is empty yet a pipe is held by an escaped process. Negligible.
3. Test gap: inherited-pipe test asserts the shell pid reaped but not that the backgrounded descendant died (shell is already reaped by try_wait, so that assertion is vacuous). I verified descendant kill manually (above); a pgrep/kill(0) check on the `sleep` would cover it.
4. Descendants that setsid/escape the group survive; successful helpers' lingering group members that closed their pipes are not killed. Same as before; documented. Spawn/fork/exec, filesystem fixture setup and waitpid are not preemptible; deadlines are best-effort against OS scheduling, not hard realtime. Poll throughput ~6.5MB/s, irrelevant for these helpers.
5. Evidence caveat (as the doc states): local timing does not reproduce the hosted stall; 5s/30s are margins, not measured hosted percentiles. A genuinely slower host still fails closed. Manager must rerun hosted macOS.

## Verdict: ACCEPT (no blockers)
