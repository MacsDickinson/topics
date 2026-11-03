# Final deterministic assertion authority corrections

Baseline c637101. Only synthetic temporary roots and absolute temporary app-data are used.

RED before production changes:

- `CARGO_INCREMENTAL=0 cargo test --offline -p topics-store --features test-support --test final_confirmation f1_ -- --test-threads=1`: **0 passed, 2 failed**. `Zed` with `ask Zedekiah in legal` and `legal` with `check with legal` both wrongly exported their unselected transcript values after approval.
- The same command filtered to `f2_`: **1 passed, 1 failed**. The real-correction metadata-follow-up positive fails; the passage-only / unchanged-transcript lineage negative remains withheld.
- The previously added r2 literal positive is explicitly amended under the manager's stricter policy: its complete statement equals the value after trimming, with original whitespace retained in persisted human evidence. The earlier `Confirmed: VALUE` fixture was a substring grant and is deliberately no longer accepted as literal authority. The independent 31 probe prefix and assertions remain unchanged.

The requested bounded policy is exact complete trimmed statement/value equality, plus genuine changed-value supersession or proven UI value edits. Token, substring and semantic inference are not assertion authority. Same-value lineage may retain an actual human value authority; passage-only or metadata-only history cannot manufacture it.
