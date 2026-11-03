# Final targeted confirmation corrections

Baseline ebfbe67; synthetic temporary notes roots and absolute isolated app-data only.

RED before production changes: copied the independent `final-probes.rs` unchanged to `crates/topics-store/tests/final_confirmation.rs` and ran `CARGO_INCREMENTAL=0 cargo test --offline -p topics-store --features test-support --test final_confirmation -- --test-threads=1`. Actual result: **31 tests, 20 passed, 11 failed, 0 ignored**. Failures: a1/a1b/a1c/a1d (R1), b3/b6/b7/b8 (R2), e3 (R3), a4 (R4), c7 (R5). The independent source remains untouched.
