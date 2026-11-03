# Bounded re-review corrections

Baseline: 2c7883a. Synthetic temporary roots and absolute temporary app-data only.

RED: `CARGO_INCREMENTAL=0 cargo test --offline -p topics-store --features test-support --test rereview_privacy -- --test-threads=1` ran 13 probes: 4 passed, 9 failed (n09, n10, n20, n21, n22, n24, n25, n35, n50). The fixture/probes derive from the independent keep2/new_probes.current.rs; n20/n21/n22/n24/n25 reproduce N1–N4 directly. No production changes precede this result.

First GREEN milestone: 20 focused tests pass, including all 13 original imported assertions. The full core/store/CLI/BDD crate test command passes 153 tests (133 previous + 20 focused), 0 failed/ignored. Warning-denied all-targets clippy passes. An intermediate full run caught the explicit-after-edit source-review regression; that path was corrected and the unchanged regression now passes.

| Finding | Bounded correction and evidence |
| --- | --- |
| N1 | Raw model IDs are checked once at the cloud boundary; a name-independent input pass validates dependencies before server lookup. Derived ambiguity remains human-only, never a tool result. n20/n21 and `model_validation_has_uniform_errors_and_never_publishes_private_ambiguity` compare success/error outputs, assert no private ID/name, no validation publication, preservation of unrelated candidates, and explicit unrelated subset approval. |
| N2 | Durable operation before/after values distinguish a real value edit from metadata-only provenance. Exact user statement evidence remains available to the human. n22, subject-only edit, genuine changed-value rebuild, and n23 redaction positives pass. |
| N3 | Source reviews bind path/hash/sequence. Prior local hash lineage dominates older destination consent; only a fresh human review of the current mapping releases it. n25, stale-ID negative/fresh-current-byte positive, and unchanged explicit-after-edit and undo-label tests pass. |
| N4 | Context connections require every source endpoint selected and every claim endpoint scoped, with private topic checks. n24 negative and fully selected/human-visible positive pass. |
| N5 | Case/whitespace and kind variants produce ambiguity, never identity merging. n09/n10 pass. Exact unique evidence-based correspondence remains the only automatic identity reuse. |
| N6 | Pending claims/connections dedupe semantically within a proposal, union citations and differing uncertainty, retain first identity, and remap dependent claim endpoints. n50 and multi-source dependent relationship test pass. |
| N7 | Topic-only same-name proposals include human unresolved identity review when selected passage evidence is available; no automatic merge/acceptance. Real second Joe remains reviewable. |
| N8 | Only root `topics/` is reserved; `Projects/topics/meeting.md` is inventoried and folder-selected, while app records stay excluded. |
| N9 | Inventory performance remains a prototype limit; no broad performance rewrite. |
| N10 | Surviving orphan correction/rule records without either a committed marker or a known interrupted journal warn and quarantine the superseded head. Orphans never authorize a replacement. n35 now also asserts no false current value; original interrupted/conflict recovery tests pass. |
| N11 | Source-only commit conflicts describe mapping ownership/history and recovery, rather than pretending every conflict follows proposal review. |
| N12 | Redacted current human-supported claim IDs, predicate and recorded_at remain intentional authorized-topic metadata; excluded evidence, source identifiers/paths, supersedes, dates and raw statement text are removed. No wire/API expansion. |

Guarded publication API and all public serde types are unchanged. No runtime/UI/provider acceptance is claimed here. Review must target the final diff; original independent 27-finding closure remains supporting baseline evidence, not a substitute for N1–N4 confirmation.
