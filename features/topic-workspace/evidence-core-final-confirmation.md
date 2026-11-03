# Final targeted confirmation corrections

Baseline ebfbe67; synthetic temporary notes roots and absolute isolated app-data only.

RED before production changes: copied the independent `final-probes.rs` unchanged to `crates/topics-store/tests/final_confirmation.rs` and ran `CARGO_INCREMENTAL=0 cargo test --offline -p topics-store --features test-support --test final_confirmation -- --test-threads=1`. Actual result: **31 tests, 20 passed, 11 failed, 0 ignored**. Failures: a1/a1b/a1c/a1d (R1), b3/b6/b7/b8 (R2), e3 (R3), a4 (R4), c7 (R5). The independent source remains untouched.

First GREEN source milestone: unchanged reviewer assertions **31/31 pass**, including a1/a1b/a1c/a1d, b3/b6/b7/b8, e3/a4/c7. The full four-crate command with `topics-store/test-support --no-fail-fast` also passes **195 tests (164 prior + 31 reviewer probes)**, 0 failed/ignored. Shared serde/request types, origin/scope authority, and the guarded publication signature are unchanged.

R1 computes a review dependency closure over logical and revision claim IDs before converting ambiguity to human-only records; suppressed claim dependents are pruned. R4 preserves declared unresolved alternatives without reinterpreting their sorted existing IDs. R2 requires raw-statement actual changed-value lineage or a literal statement value; questions/metadata alone cannot authorize the unchanged transcript value, and shape validation explicitly rejects no-op corrections. R3 retains the superseding candidate's established logical identity, unions meaningful supersedes/evidence and remaps dependent endpoints. R5 applies derived-topic taint before exact-byte source review so preview visibly excludes just the tainted source.
