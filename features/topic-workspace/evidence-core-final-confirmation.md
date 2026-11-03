# Final targeted confirmation corrections

Later manager ruling tightens raw literal authority to complete trimmed equality and retains genuine authority through bounded same-value metadata lineage; see [evidence-core-authority-final.md](evidence-core-authority-final.md). Earlier counts remain evidence for their recorded commits, not a waiver of those final corrections.

Baseline ebfbe67; synthetic temporary notes roots and absolute isolated app-data only.

RED before production changes: copied the independent `final-probes.rs` unchanged to `crates/topics-store/tests/final_confirmation.rs` and ran `CARGO_INCREMENTAL=0 cargo test --offline -p topics-store --features test-support --test final_confirmation -- --test-threads=1`. Actual result: **31 tests, 20 passed, 11 failed, 0 ignored**. Failures: a1/a1b/a1c/a1d (R1), b3/b6/b7/b8 (R2), e3 (R3), a4 (R4), c7 (R5). The independent source remains untouched.

First GREEN source milestone: unchanged reviewer assertions **31/31 pass**, including a1/a1b/a1c/a1d, b3/b6/b7/b8, e3/a4/c7. The full four-crate command with `topics-store/test-support --no-fail-fast` also passes **195 tests (164 prior + 31 reviewer probes)**, 0 failed/ignored. Shared serde/request types, origin/scope authority, and the guarded publication signature are unchanged.

R1 computes a review dependency closure over logical and revision claim IDs before converting ambiguity to human-only records; suppressed claim dependents are pruned. R4 preserves declared unresolved alternatives without reinterpreting their sorted existing IDs. R2 requires raw-statement actual changed-value lineage or a literal statement value; questions/metadata alone cannot authorize the unchanged transcript value, and shape validation explicitly rejects no-op corrections. R3 retains the superseding candidate's established logical identity, unions meaningful supersedes/evidence and remaps dependent endpoints. R5 applies derived-topic taint before exact-byte source review so preview visibly excludes just the tainted source.

Final targeted checks and invariants:

| Request | Evidence |
| --- | --- |
| R1 private same-name/alias and visible Joe + private joe | Independent a1/a1b/a1c/a1d pass unchanged. Added logical/revision ID × both endpoint directions confirms uniform malformed raw missing-dependency errors, no private ID in error output, guard denial leaves no pending/temp visibility, and unrelated eligible facts can be explicitly approved as a subset. |
| R2 actual value authority | Independent b3/b6/b7 withhold unchanged transcript value; b8's no-op correction is rejected. b4 changed-value correction and b5 UI value edit remain usable with exact statements. Added literal raw-statement positive and explicit shape/service no-op rejection without history mutation. |
| R3 supersession semantics | e3 passes unchanged. Added either-order batch with three equivalent claims and dependent logical/revision endpoints preserves the existing logical ID, one current new value, prior history, and all endpoint dependencies. This found an intermediate C→A→B remap chain; a bounded final-ID resolution fixes it (34/35 RED → 35/35 GREEN). |
| R4 UUID-independent human review | Unedited independent a4 fixes proposed UUID at both ordering extremes and succeeds; declared unresolved alternatives are preserved rather than re-inferred from the first sorted existing ID. |
| R5 precise preview exclusions | c7 passes unchanged. Fresh source CloudAllowed cannot release derived private-topic taint; preview excludes A visibly, starts a run on B, returns only B, and requires explicit topic declassification to include A again. |

Final commands all exit 0:

- `cargo fmt -p topics-core -p topics-store -p topics-cli -p topics-bdd --check`.
- `CARGO_INCREMENTAL=0 cargo clippy --offline -p topics-core -p topics-store -p topics-cli -p topics-bdd --all-targets -- -D warnings`.
- `CARGO_INCREMENTAL=0 cargo test --offline -p topics-core -p topics-store -p topics-cli -p topics-bdd --features topics-store/test-support --no-fail-fast`: **199 passed, 0 failed, 0 ignored** (164 prior + 35 final confirmation tests). This includes all 31 previous rereview tests and original regression suites.
- `CARGO_INCREMENTAL=0 cargo run --offline -p topics-bdd --bin workspace-bdd`: **20 scenarios / 110 steps passed**, no skipped steps.
- `CARGO_INCREMENTAL=0 cargo run --offline -p topics-bdd -- --tags 'not @ui'`: **15 scenarios / 78 steps passed**, no skipped steps; Phase 0 remains unchanged.

The initial exact independent fixture and 31 probe functions compare byte-for-byte equal after rustfmt (ignoring rustfmt's stdout filename banner and final blank line); no assertions were weakened or unused fixture helpers removed. Four bounded regression tests were appended. Raw logs inspected at `/tmp/topics-r-{red,green1,green2,green3,final-tests,final-clippy,final-workspace-bdd,final-phase0-bdd}.log`; committed counts retain the actual results. The final commit includes formatting of the initially verbatim RED test file, addressing the manager's fmt gate.

No public API/serde or runtime/UI/native/provider changes, root dependency additions, root manifest/lock ownership changes, live notes or personal configuration access. Model tools still receive unit validation or sanitized raw-input errors; human pending candidates remain local. The initial source milestone is ee1ba22; final remap-chain correction, unchanged-probe formatting and evidence are in its follow-up commit.

Residuals intentionally unchanged: Unicode NFC/NFD name ambiguity; moved private bytes edited before inventory cannot be recognized by exact hash lineage and can inherit destination consent; logical claim endpoint revision selection may expose the opaque ID/relation without a value. Existing no-cross-file-atomicity, no-uncooperative-writer/cross-app-data coordination and detectable-versus-wholesale-history-loss limits remain. Inventory performance, authorized redacted claim ID/predicate/recorded_at and the empty-only-passage ambiguity limitation were not expanded. No claim of runtime/UI/native/provider acceptance is made by this worker; integration and final independent targeted confirmation remain with their owners.
