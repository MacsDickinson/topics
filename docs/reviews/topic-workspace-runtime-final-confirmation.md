# Targeted confirmation: Topics runtime 44212f4..1430eba (evidence 09b484e)

Reviewer: Claude Opus 5.5. Read-only, no repo edits. No live provider call. No real auth, config, header or notes access.
Everything ran on synthetic throwaway homes with outbound network denied except loopback.

## Verdict: NOT a full ACCEPT - one concrete counterexample on R2

R1, R3-R8 and chronology: ACCEPT.
R2: the code does what was specified (env_clear plus allowlist) and closes the original trace repro, but the property it claims does not hold.
Codex re-imports variables from `$CODEX_HOME/.env` after exec.
No re-architecture is needed. The fix is small, so the gate can close once it lands, without a waiver.

## Open finding

### R2b MEDIUM-LOW - `$CODEX_HOME/.env` re-injects proxy and CA settings after env_clear

Where: crates/topics-runtime/src/environment.rs:33-69 (no guard; the claim is at lines 2-3, "excluded by construction"), lib.rs:325, lib.rs:746, README.md:96.
`grep` finds no `.env` handling anywhere in runtime src.
The same runtime tree is already on main (9be6ed6, tree identical), so this applies there too.

Setup: production argv (DISABLED, PRIVACY_OVERRIDES, catalog and profile parsed from source) and the exact allowlisted env, with a synthetic CODEX_HOME containing `.env`.
Codex 0.166.0, with deny-network-except-loopback added to the profile.

| Run | `.env` contents | Result |
|---|---|---|
| control, synthetic ChatGPT auth | none | 0 proxy connections, 0 stderr bytes |
| dotenv, synthetic ChatGPT auth | proxy vars, RUST_LOG=trace, others | `CONNECT chatgpt.com:443` reached the loopback listener during account/read; 253,476 stderr bytes (forced RUST_LOG=off overridden) |
| mitm, synthetic API key | HTTPS_PROXY + SSL_CERT_FILE (synthetic CA) | TLS terminated by the loopback proxy. 6 POST /backend-api/codex/responses with the prompt marker in plaintext; synthetic bearer seen on all 13 requests |
| mitm-noca, synthetic API key | HTTPS_PROXY only | Same proxy rejected: TLSV1_ALERT_UNKNOWN_CA. So the `.env` CA is what enables decryption |

In the mitm run the following all passed, unchanged: my Python port of the key config/read checks (destination URLs, provider, notify, features), thread model and provider, ephemeral, instructionSources empty.
config/read showed no trace of the proxy or CA.
Those checks would not detect it, and the production validators read the same config/read data.

Why it is in scope: hostile config.toml in the same directory is already rejected fail-closed (F2 ACCEPT). `.env` is the same trust class and bypasses all of that.
It is also not only a malicious case: a benign user `~/.codex/.env` with a corporate proxy would silently route note analysis through it.

Why MEDIUM-LOW: it needs a same-user write, or a pre-existing user file, so it is not a model path. Same precondition class as the original R2.

Not established:
- The full turn was only reached with API-key auth, which production rejects at account/read.
- Synthetic ChatGPT auth stopped at "workspace routing discovery failed" in account/read. So prompt and bearer exposure on the subscription path is inferred, not observed; only the CONNECT was observed on that path.
- I drove the production argv with my own driver, not the Runtime binary, because sandbox-exec cannot nest and an unconfined run could have reached the real provider if the proxy were not honoured.
- SSLKEYLOGFILE was never created even with completed TLS, so no keylog effect was demonstrated.
- Not tested: whether HOME/.codex/.env is read when CODEX_HOME points elsewhere. cwd is verified empty by the runtime, so a cwd `.env` should not apply.
- I did not check whether the real ~/.codex/.env exists (out of scope). The owner should check before real notes.

Suggested fix (not implemented or tested): in environment::apply_from, the single choke point used by version checks, children and probes, fail closed with an actionable error if `.env` exists in the effective Codex home (CODEX_HOME, else HOME/.codex).
Use lstat only and never read it. Add a regression test. A same-user write after the check remains the documented limit.

## Confirmed

### R1 ancestor clones: ACCEPT
Exact production profile, parsed from source, on macOS 15.5. 46 cases, all as expected:
- Positive controls: unsandboxed clonefile and fclonefileat copied the sentinel.
- Allowed: sandboxed sibling clone (both syscalls) and a clone of a plain file inside an ancestor.
- Denied with EPERM/EACCES and no sentinel copied, for both syscalls: parent, trailing slash, /., Notes/.., grandparent, symlink, firmlink (/System/Volumes/Data), case variant, NFD variant, /.vol.
- Also denied: clonefileat (relative, including NFD), chdir-relative clone, copyfile CLONE|RECURSIVE, cp -cR and ditto (rc=1, no leak), direct clone of root and secret through aliases, new hardlink creation, reads through all aliases, openat from an ancestor fd.

The preflight code runs both syscalls against two synthetic ancestors with positive controls.
Actual Codex under the profile with the file-clone rules: the real subscription log (implementer's) plus my synthetic runs above, which reached thread and turn.

### R2 allowlist as specified: code confirmed, property open (R2b)
- env_clear at all three sites (bounded_probe, command, check_version); only HOME, CODEX_HOME, PATH, TMPDIR, LANG, LC_ALL, LC_CTYPE, RUST_LOG=off.
- Original trace repro is closed. Positive control: CODEX_ROLLOUT_TRACE_ROOT passed directly to the process creates a trace with the prompt in 2 files. Passed via `.env`, no trace was created, so that path does not reopen it.
- The real scripted-child regression and the implementer's harness (hostile HTTPS_PROXY in the parent while the run succeeds) support stripping of parent env.
- Note: PATH crosses unvalidated by design. Not exploited; sandbox-exec and codex are launched by absolute path.

### R3-R7: ACCEPT
Code read and the named tests pass.
- R3: the validated Value is reserialized before forwarding.
- R4: require_success and require_denial reject nonzero helper exit, bad JSON, ENOENT and a successful operation.
- R5: static codes map to remediation; the version error filters the installed string.
- R6: revoke uses try_lock and returns false immediately.
- R7: authorize_commit checks failed.

Residuals by code reading only, not run, no action needed for the gate:
- INFO lib.rs:298-299 + 716-717: if the 150s timeout fires exactly during a hung publication action, finish_publication holds the state lock while blocked in published(), so cancel() would block in that narrow case.
- INFO commit.rs revoke: a nanosecond collision with published() gives a spurious false from cancel; the permit deadline still bounds it.
- INFO isolation.rs:158: preflight requires /usr/bin/python3, so a Mac without Command Line Tools likely fails closed and analysis is unavailable (not tested). Correct for safety, but an availability limit for a desktop app.

### R8 chronology: ACCEPT, high confidence
- Recomputed source digest 669c6f5c... and latest mtime 1793685618239791665 (06:00:18) match the JSON exactly.
- Source scope is identical between 1430eba and 09b484e; topics-core and the committed root manifests have no diff in that range.
- Root Cargo.toml and Cargo.lock SHA256 match the recorded build-time values.
- All three executable SHA256 and ns mtimes match the JSON (06:07:42, after the 06:07:35 commit).
- Logs were written 06:07:43-06:08:22, chronology 06:08:22.466, evidence commit 06:09:16. Reflog is linear, with no amend.
- Trace and keylog absence are enforced by asserts before the report is written, and fake HTTPS_PROXY in the parent would have failed a leaking run.

Precision on what ran when:
- Only the 4 native tests ran in the post-freeze harness. The 45 deterministic ran at 06:00:38, and fmt and clippy at 06:07:04: after the last source edit, before the commit. The implementer's report says this correctly; the "all in one invocation" wording applies only to the native 4.
- live_attempts:1 is hard-coded and cannot be proven from artifacts. The timing makes a post-freeze retry very unlikely.
- Harness SCOPE excludes topics-core; covered by the git diff above.

## Checks run
- Prebuilt frozen-source binaries, no compile, clean env: 2+22+5+4+1+11 = 45 pass, 0 failed, exit 0. Implementer's fmt and clippy logs were inspected, not rerun.
- Native tests not rerun, per instruction: subscription, schema_pin, config_override_native.
- Probes: r1.py, helper.py and r2env.py in the probes directory beside this report. All fixtures were deleted. Worktree status unchanged (only the manager's Cargo.toml and Cargo.lock), HEAD 09b484e.

## Documented limits, not defects
Existing outside-root hardlinks and copies; metadata via stat; malicious same-user memory or process reader; a same-user write after any check; final OS rename not preemptible; publication's short cooperative action; unsupported filesystems fail closed; Linux unavailable. launchctl and LaunchServices escape remains untested (same-user class, requires execution capability the verified tool surface lacks).
