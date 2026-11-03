# Final deterministic assertion authority corrections

Baseline c637101. Only synthetic temporary roots and absolute temporary app-data are used.

RED before production changes:

- `CARGO_INCREMENTAL=0 cargo test --offline -p topics-store --features test-support --test final_confirmation f1_ -- --test-threads=1`: **0 passed, 2 failed**. `Zed` with `ask Zedekiah in legal` and `legal` with `check with legal` both wrongly exported their unselected transcript values after approval.
- The same command filtered to `f2_`: **1 passed, 1 failed**. The real-correction metadata-follow-up positive fails; the passage-only / unchanged-transcript lineage negative remains withheld.
- The previously added r2 literal positive is explicitly amended under the manager's stricter policy: its complete statement equals the value after trimming, with original whitespace retained in persisted human evidence. The earlier `Confirmed: VALUE` fixture was a substring grant and is deliberately no longer accepted as literal authority. The independent 31 probe prefix and assertions remain unchanged.

The requested bounded policy is exact complete trimmed statement/value equality, plus genuine changed-value supersession or proven UI value edits. Token, substring and semantic inference are not assertion authority. Same-value lineage may retain an actual human value authority; passage-only or metadata-only history cannot manufacture it.

GREEN and policy evidence:

- The amended exact-equality positive passed before the source change and passes afterward, including preservation of the exact original whitespace in the human statement.
- Both F1 instruction/substring negatives now withhold the value. Approval does not upgrade a question or instruction into literal assertion authority.
- F2 follows only the same trimmed value, logical claim ID, subject and predicate. A visited set prevents cycles, and at most 100 revision records are queued/examined; exceeding the bound fails closed. It cannot cross a different-value ancestor to resurrect an older value's authority. The positive checks two date/uncertainty follow-ups after a genuine correction in both chat and manual history. Negative chains cover unchanged transcript values and a passage-only value change, neither of which acquires human authority.
- Focused `final_confirmation` result: **39 passed, 0 failed, 0 ignored**, including all 31 independent assertions. A rustfmt-normalized comparison confirms the independent fixture/probe prefix remains identical; only the previously owned r2 positive was amended under explicit manager direction, and four F1/F2 tests appended.
- F3 is documented only: preserved model-declared alternatives may omit a server-known private alternative from the human inbox. The list is not exhaustive. No identity/proposal behavior or model boundary is expanded.

Final verification commands all exit 0:

1. `cargo fmt -p topics-core -p topics-store -p topics-cli -p topics-bdd --check`.
2. `CARGO_INCREMENTAL=0 cargo clippy --offline -p topics-core -p topics-store -p topics-cli -p topics-bdd --all-targets -- -D warnings`.
3. `CARGO_INCREMENTAL=0 cargo test --offline -p topics-core -p topics-store -p topics-cli -p topics-bdd --features topics-store/test-support --no-fail-fast`: **203 passed, 0 failed, 0 ignored** (199 prior + four final F1/F2 regressions).
4. `CARGO_INCREMENTAL=0 cargo run --offline -p topics-bdd --bin workspace-bdd`: **20 scenarios, 110 steps passed**, no skips.
5. `CARGO_INCREMENTAL=0 cargo run --offline -p topics-bdd -- --tags 'not @ui'`: **15 scenarios, 78 steps passed**, no skips; Phase 0 unchanged.

Inspected logs: `/tmp/topics-f1-red.log`, `/tmp/topics-f2-red.log`, `/tmp/topics-f-equality-baseline.log`, `/tmp/topics-f-green.log`, `/tmp/topics-f-final-{clippy,tests,workspace-bdd,phase0-bdd}.log`. Production diff is confined to the private authority classifier in the store scope service. Public serde/API, guarded publication, runtime/desktop and scope/origin authority remain unchanged. No live notes, personal config, provider/native tests, root manifests or dependency additions. Cargo's locally regenerated root lock is restored before commit; integration retains its manager-owned lock. Existing recorded hash/name/opaque-ID and recovery limits remain unchanged. Independent focused confirmation and main integration belong to the coordinator.
