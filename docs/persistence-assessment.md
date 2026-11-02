# Phase 0 persistence assessment

This limited assessment selects SQLite with bundled FTS5 for a replaceable document index. Markdown remains authoritative; no accepted knowledge will exist only in this database. Phase 0 does not implement graph retrieval, claims, correction records or relationship editing.

Synthetic future retrieval cases:

- Shared attendance: Alex and Sam attend meeting M1, which discusses Ledger. A future query joins sourced attendance edges and discussion edges; attendance implies possible exposure, never confirmed understanding.
- Ambiguous identity: M1 mentions “Joe” while two people have that alias. Preserve the source passage and unresolved identity rather than choosing an endpoint silently.
- Later decision: M2 changes the Ledger cutover after Alex last attended M1. Compare source dates and validity intervals, preserving both versions and review status.
- Correction: an accepted correction supersedes an earlier claim revision. Retain provenance and supersession history; rebuilding must recover both from authoritative records outside SQLite.

SQLite can represent extensible edge types as values, stable endpoints and passage identifiers, source references, review status, validity intervals and supersession references in ordinary relational tables. Indexed joins and recursive CTEs can serve these bounded queries; a dedicated graph engine is not justified by these examples alone. Unknown identities need explicit unresolved records; SQL cannot infer identity or certainty. Future phases must define and approve durable Markdown record formats before adding that functionality and benchmark representative graph workloads before committing to an engine.

For Phase 0, the only stored entities are derived documents, parsed profiles and full-text content. The database can be deleted and recreated from a configured root; its schema is deliberately not a durable knowledge API. Application Support holds configuration and index separately from notes. A startup content scan detects offline edits even with unchanged modification timestamps; the live polling watcher uses the same service path. No network, notes Git repository, or note write is required for indexing.
