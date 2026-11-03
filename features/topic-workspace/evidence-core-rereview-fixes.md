# Bounded re-review corrections

Baseline: 2c7883a. Synthetic temporary roots and absolute temporary app-data only.

RED: `CARGO_INCREMENTAL=0 cargo test --offline -p topics-store --features test-support --test rereview_privacy -- --test-threads=1` ran 13 probes: 4 passed, 9 failed (n09, n10, n20, n21, n22, n24, n25, n35, n50). The fixture/probes derive from the independent keep2/new_probes.current.rs; n20/n21/n22/n24/n25 reproduce N1–N4 directly. No production changes precede this result.
