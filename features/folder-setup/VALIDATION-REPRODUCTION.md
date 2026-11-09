# Scratch actual-host validation reproduction

2026-11-09; one authorized genuine fictional Runtime attempt, completed in 23.717s. The new scratch reproduction failed at the exact app-owned core gate: `invalid_input: Claim timestamp must be RFC3339 UTC`. The scratch investigation edited no product source/test/manifests; the subsequently approved prompt/test followup is recorded below. No additional provider attempt is authorized or performed.

## Observed root cause and bounds of attribution

The actual installed runtime returned four fictional candidates: two topics, one pronoun claim and one cited leads connection. The claim's `recorded_at` was exactly `2026-11-09`, a date without a time or UTC offset. Runtime canonical shape parsing admits this structurally valid string; the real core's `workspace_validation.rs:58-61` requires RFC3339 UTC and rejected it before publication. The core error returned by the actual service equals the host error exactly. The runtime retained `host/validation_denied`, one scope read and one proposal call. Canonical job coverage is selected1/processed0/failed1, one attempt, one authorization, no graph items and no publication. Source and private bytes stayed unchanged.

The exact captured buffer was replayed three times against `setup_batch_validate` under the same original live capability after the runtime terminal and before canonical failure/revocation: three identical timestamp errors, zero further provider calls. This is deterministic semantic RED. The capture hash is `97700d0ec9dc90f45bbb846ca59db5fd98e9c41e83cd9f0ddb3ea5d210716ee7`.

The prior actual Tauri run on built90f3276 retained only host/validation_denied, not candidates or raw core error. The scratch result faithfully reproduces that category and establishes this new attempt's exact gate; it cannot prove the discarded prior native proposal had the identical malformed field. Earlier unknown/relationship-only attempts remain separate events.

## Offline one-field ablation

`offline-ablation` reopens a COPY of this wholly synthetic app-data, keeps the same source root/IDs/citations and canonical manifest, and obtains an offline retry capability there without constructing any Runtime or provider client. Original four candidates still fail with the timestamp error. Changing only the single claim's `recorded_at` to `2026-11-09T00:00:00Z` passes FULL `setup_batch_validate`, including dependency/source/privacy/identity checks. No stage/publication/acceptance is called, processed and graph remain zero, the original live job remains untouched. The test-only extra attempt/call counter belongs exclusively to the copied offline app-data and is not another provider call. See offline-ablation.json.

## Faithful host and source parity

Dependencies are actual core/store/runtime source from checkout `/Users/macs/orca/workspaces/topics/topics-setup-live-acceptance`, HEAD `c937a2f201ae6eb79e1feab2e83d54348dab5172`, with product source verified identical to built `90f3276b358e94d4ae719a9874297fc7c87658cc` across core/store/runtime/setup. The scratch binary path-compiles actual unmodified `crates/topics-setup/src/host.rs` AND `context.rs`; it does not invent a response or validator. `Inner` has the same real service mutex/closure semantics; ONLY UI watch publication is a no-op. The recording decorator delegates every call and both commit methods. Actual core validation, candidate accumulation, guarded durable publication, unread-scope signal, privacy checks and real CommitPermit run unchanged. No GUI or coordinator-thread/watch delivery is exercised.

The actual `host::PROMPT` plus the coordinator's exact `This attempt's reviewed candidate budget is 64.` suffix is used. A separate local actual-host read verifies first-batch scope/context parity, then a NEW actual host with unread guard false is used for the model attempt. Scoped response has one exact110-byte source, no accepted topics or draft records, no private path or bytes; its pre-bridge serialized size was 857 bytes. The real bridge adds the same candidate schema and provides the same two tools. `candidate.schema.json:193-195` describes recorded_at only as type:string, and current PROMPT gives no timestamp format; missing guidance is observed, while why this model chose a date-only value is not proven.

Forty-eight product source/schema/manifests were hashed before and after all live work (source-provenance.json) and remained unchanged. During the genuine scratch attempt the worker checkout stayed at d271cae with only its original manager-owned dirty manifests/lock; later approved edits are limited to the prompt and regression/report files below. Acceptance source's pre-existing untracked evidence was untouched. All generated Rust source, manifest/lock/build artifacts and data are scratch-only; own target `/tmp/topics-setup-target`, no main/genuine/manager target or source writes.

## Isolation and preflight evidence

Exact selected fixture is native-live-setup.mjs's110 bytes, SHA256 `4243c2bfdfd37862a5ac55a01da0c6cc0f745e753d094dfa335b35a4c240c367`; exact private fixture SHA256 `813fc16fa24dea6fc5b7048d44462706d49357966e443da4621d56e898c74e65`. Fresh absolute notes/app-data in this scratch directory; no note symlinks. Private source is marked LocalOnly by actual canonical API before preview; exact selected-file manifest, unchanged20file/65536byte/100batch/64candidate/4attempt limits, explicit canonical test consent and one real batch capability.

Dry runs under synthetic HOME and scripted children, before live attempt:
- dry-success: one scope/proposal, four graph items, one real guarded publication, processed1,1392ms, no provider.
- dry-denial: deliberate synthetic citation-hash corruption reaches actual core, host/validation_denied, zero publication/coverage, three same-capability offline replays,1362ms, no provider.

Live parent HOME is an empty synthetic directory; CODEX_HOME is absent. Only the installed executable's wrapper restores the existing runtime HOME. Exact executable `/Users/macs/.codex/packages/standalone/releases/0.166.0-aarch64-apple-darwin/bin/codex`; version, account, effective config, catalog, containment and all preflight checks are owned by the unmodified Runtime. No credential or personal/provider configuration contents were inspected/copied (the offline ablation copies only synthetic canonical app-data and its fictional root setting), no raw provider frames, text deltas or stderr were captured. Parent probe stderr is zero bytes. The only retained model values are the explicitly authorized fictional candidate JSON; the only raw validation error is actual app-owned core error. Renderer-only fake transmission scratch files were removed after dry runs.

The live binary requires `TOPICS_SYNTHETIC_LIVE_PROBE_CONSENT=one-fictional-attempt`, creates a global exclusive LIVE_ATTEMPT_STARTED marker immediately before its sole Runtime::analysis_start, and refuses reused session/marker. Marker remains present. Exactly one genuine Runtime attempt was made; no retry. Runtime contains only original two tools, no approvals, existing64candidate/call/512KiBframe/256KiBhost-response/2MiBoutput budgets,150sexecution,5shost/6sSDK/40sread and5shelper/30saggregate preflight bounds. Probe terminal wait200s and parent process230s remain bounded; no production timeout changed.

## Commands and files

Build (offline, CARGO_INCREMENTAL=0, CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0, own target):
`cargo build --offline --manifest-path /private/tmp/topics-host-validation-probe.xg8m0cer/Cargo.toml`

With synthetic parent HOME, no CODEX_HOME and no provider credentials in parent environment:
`/tmp/topics-setup-target/debug/topics-host-validation-scratch-probe dry-success`
`/tmp/topics-setup-target/debug/topics-host-validation-scratch-probe dry-denial`
`TOPICS_SYNTHETIC_LIVE_PROBE_CONSENT=one-fictional-attempt /tmp/topics-setup-target/debug/topics-host-validation-scratch-probe live-once`

Offline ablation only (no Runtime construction/provider):
`cargo run --offline --manifest-path /private/tmp/topics-host-validation-probe.xg8m0cer/Cargo.toml --bin offline-ablation`

Scratch-only source: src/main.rs, src/bin/offline-ablation.rs, Cargo.toml/Cargo.lock. Evidence: source-provenance.json; live-once/isolation-and-consent.json, captured-proposal.json, offline-replay-red.json, summary.json; offline-ablation.json; dry-success/ and dry-denial/ counterparts; build and bounded process outputs.

## Approved bounded guidance followup

A one-sentence addition to the OWNED setup prompt explicitly separates claim recording timestamp from validity dates: `For every claim, recorded_at must be a full RFC3339 UTC timestamp with date and time (zero UTC offset, such as Z), never a date alone; valid_from and valid_to are separate nullable YYYY-MM-DD source-validity dates.` Reviewable exact diff is the scratch `proposed-guidance.patch` under `/private/tmp/topics-host-validation-probe.xg8m0cer`; the same one-sentence addition is the sole product behavior change in this branch. No core validator/schema/identity/source/approval changes or automatic timestamp coercion. A deterministic captured-fixture regression preserves invalid-date rejection and verifies a full UTC timestamp passes the complete core; prompt contract coverage ensures the model receives this requirement. That cannot establish future model compliance or GUI acceptance without separately authorized genuine validation.

The coordinator approved this minimal prompt addition and a captured-fictional regression in this dispatch, requiring no frozen current calendar date in product guidance and no further provider calls. The exact fictional candidate array is committed as `crates/topics-setup/tests/fixtures/native-date-only-recorded-at.json`, with the canonical payload hash above. Only fixture-bound source ID/path and capability are rebound; locator, source hashes/excerpt, candidate identities and assertions remain identical. The ablation then changes exactly one recorded_at value and passes FULL core validation without staging or acceptance.

Before changing the prompt, `cargo test --offline -p topics-setup --test protocol captured_date_only_timestamp -- --nocapture` was **RED: 0 passed, 1 failed**, specifically `transmitted setup instructions omit recorded_at`. The invalid captured buffer rejection, one-field UTC full-core success, empty graph/coverage/proposal and source-unchanged assertions had already passed. Actual setup instructions were observed from the scripted child's received runtime turn input, not imported from a production constant. This regression establishes semantic format enforcement and instruction delivery, not future model compliance.

No core validator, schema, serialization coercion, dependency, privacy/identity rule, cap, retry or approval tool is changed. The original actual native payload is still unavailable; the new genuine scratch capture establishes this attempt's timestamp rejection only. Required Claude review remains auth-blocked, and the complete native review journey remains FAILED/not accepted. No source patch is a promise of a passing GUI run.

## Final approved source checks

RED tests/fixture/provenance were committed as `020e2b2` before changing the prompt. The same focused test is now GREEN. The sole product change adds explicit RFC3339 UTC date/time guidance with no literal calendar date; invalid input remains rejected and no timestamp is automatically rewritten.

Using `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=/tmp/topics-setup-target`, with synthetic Codex homes and default features (no native-live/provider tests):

- `cargo test --offline -p topics-runtime -p topics-setup`: **66 runtime + 30 setup passed** (37 runtime unit + 29 runtime integration; 3 setup unit + 27 setup protocol).
- `cargo test --offline -p topics-store --lib --test setup_service --test setup_amendments`: **42 passed** (12 unit + 23 service + 7 amendments).
- Total: **138 passed**; `/tmp/topics-recorded-at-final-tests.txt`. Existing 100-document/multi-batch, source privacy/copies, failed retry/deduplication, changed manifest, root switch, pause/real publication fence, stable IDs, human edits and SQLite rebuild cases remain green.
- `cargo fmt -p topics-setup -p topics-runtime -p topics-store -- --check`: passed.
- `cargo clippy --offline -p topics-runtime -p topics-setup -p topics-store --all-targets --features topics-store/test-support -- -D warnings`: passed.
- `git diff --check`: passed.

Four owned files change: SetupHost prompt, owned protocol regression, fictional captured candidate fixture and this report. Manager-owned manifests/lock, core/store/runtime/schema/BDD/UI source remain untouched. Exactly one authorized genuine scratch attempt was consumed, and no provider calls followed the guidance change. The scratch process exited zero to indicate completed diagnostic capture; the actual genuine runtime/job outcome was FAILED with zero coverage, not model or GUI success.

The final source is ready for the independent reviewer's focused recheck. The manager must separately authorize any further genuine GUI validation; the full native journey and required Claude authentication remain pending.
