# Independent review: Topics durable core, f3e1ed0..85e9b77

Reviewer: Claude Opus 5.5. Read-only; no repo edits, commits or real notes.

## Verdict: FAIL for acceptance (fix the High findings first)

The evidence is real: the 65 tests and both BDD suites reproduce green. But those suites do not cover the failures below. The architecture holds (commit markers, journals, path whitelist, no model apply), and most fixes look local to one function, though I did not implement any.

Nothing here accepts knowledge silently. H1 and H2 still need a human approval, and in H1 the original topic keeps its corrected value. The defect in those cases is that the review shows a false or duplicate fact with no warning.

## Probe repro

Probes assert what I judge to be correct behaviour, so a failure confirms the observed behaviour below.
- Probes: keep/review_probes.rs (pNN), keep/review_reanalysis.rs (rNN), both beside this report. Output: probes.out, reanalysis.out, baseline.out.
- To run: copy a probe file into tree/crates/topics-store/tests/, then from tree/ run
  `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=<core target> cargo test --offline -p topics-store --features test-support --test <name> -- --test-threads=1 --nocapture`

## Reanalysis / known-context seam (manager's question)

Answer: random provisional topic IDs do bypass the correction rule, unless the whole proposal is fingerprint-identical to the earlier one. Reusing IDs from context avoids the bypass but defeats dedupe.

| Case after correcting Alex he/him -> they/them | Observed | Probe |
|---|---|---|
| Identical candidates, random topic ID | Deduped to Accepted; correct | r01 pass |
| Random topic ID, reworded uncertainty | Rule bypassed; approval creates a second Alex with current he/him | r02 |
| Random topic ID, one extra fact | Rule bypassed | r03 |
| Random topic ID, new meeting | Rule bypassed | r12 |
| Reused topic ID, false claim plus another fact | False claim suppressed; correct | r04, r11 pass |
| Reused topic ID, only the false claim | validate/stage fail: invalid_input "Submit 1–200 review candidates" | p58 |
| Reused topic ID, re-proposes they/them | Pending; approval gives two current claims, one citing the transcript | r07 |
| Reused IDs, same connection | Pending; approval gives two connections | r08 |
| Same claim with reused subject ID | Pending; approval gives two current claims | p52 |
| Echoes known Topic candidate beside a new claim | Whole proposal fails: conflict | r05 |
| Reuses logical claim ID, no supersedes | Whole proposal fails: conflict | r06 |
| source_context, original source in scope | Current they/them plus 2-item history; correct | r09 pass |
| source_context, original source not in scope | Topic returned with 0 claims and 0 history | r10 |
| source_context on a human-edited accepted claim | Claim omitted | r13 |

## High

H1. Correction rule bypassed by a provisional topic ID. proposals.rs:498-514 (rule_matches compares topic_id), proposals.rs:647. New topics are never canonicalised against accepted identities. Probes r02, r03, r12.

H2. Reanalysis duplicates accepted facts once IDs are reused. proposals.rs:35-90: the fingerprint is whole-proposal, and its identity map only holds topics in that same proposal, so reuse changes the hash. There is no claim- or connection-level duplicate check. "Repeated analysis does not duplicate accepted facts" passes in BDD only for byte-equivalent proposals. Probes p52, r07, r08.

H3. Local-only label fails open when its record is edited, deleted or restored. A source record that is edited, deleted or rolled back to older committed bytes makes the source Unknown (`[Unknown]` in all three probes). persistence.rs:531-543 excludes the edited record or accepts any committed hash; scopes.rs:124-138 defaults to Unknown. In the edit case scopes.rs:336-341 with persistence.rs:254-257 also replaces the external bytes with an Unknown label; the old bytes survive only in the app-data journal. Probes p10, p11, p12. Should fail closed.

H4. Renaming a local-only source loses the label. Identical bytes at a new path become Unknown and eligible under confirm_unknown. scopes.rs:656-670, 124-138. No rename ambiguity is surfaced, though the contract calls for it. Probe p20.

H5. Case aliases bypass privacy checks on default macOS APFS. `meetings/PRIVATE.md` skips the local-only label, and scope_read returned "secret\n". `Topics/topics/<id>.md` or folder `Topics` passes the app-record exclusion, and scope_read returned a local-only topic's name. scopes.rs:20, 207, 236, 265 and exact-match path_sensitivity. Trigger is a caller-supplied ScopeSpec path (UI/CLI), not the model; native pickers may never produce one. Probes p21, p22.

H6. Stale pending proposals strand re-analysis. Changing an unrelated file in the scope gives stale_evidence even when the citation is unchanged (proposals.rs:845-849). A fresh analysis then returns the same old proposal and old scope through dedupe (proposals.rs:756-762), and approval fails again. Changing any record's bytes, such as an unrelated source label, makes every older pending proposal conflict for good (proposals.rs:914-919). Status stays Pending; ProposalStatus::Conflict is never assigned by the store. The only exits are rejecting the proposal or using a manual proposal. Probes p50, p51.

H7. A leftover temp file bricks proposals after a crash. atomic() creates `.topics-*.tmp` in the target dir (persistence.rs:191). saved_proposals parses every entry with no filter (proposals.rs:22-27), so list, stage and validate all return internal permanently. bound_model_run reads runs/ the same way (proposals.rs:594-595), so the same applies to chat staging by inspection. Probe p01 (crash state planted by hand).

## Medium

M1. After undo, re-analysis reports the fact as Accepted though it is absent, and it cannot be re-proposed by a model. proposals.rs:732-754 ignores undo_of. Probe p40: status=Accepted, topics=0.
M2. Recovery conflict hides an accepted topic. A rewritten target keeps its uncommitted op ID, so persistence.rs:531-533 drops it, with no restore path beyond JSON byte arrays in conflict-N.json. Probe p04.
M3. Committed records deleted or rolled back give no warning. Deleting a superseding claim revives he/him as current with warnings=[]. A restored older topic record silently drops local_only. persistence.rs:531-543 does not check last-write. Probes p13, p14.
M4. Undo leaves a dangling connection endpoint. A connection using the logical claim ID is not seen as a dependent, because ids come from revision filenames. mod.rs:329-356. Probe p41.
M5. Human edits keep unqualified transcript citations. Editing a connection relation keeps the citation with no provenance (Connection has no statement evidence). Editing claim subject_id or uncertainty adds no statement; proposals.rs:858 omits those fields. So a claim can be re-pointed from one Joe to another. Probes p30, p31.
M6. Same-name handling is partial. Ambiguity is only detected for connections to existing topics (proposals.rs:619-646). A claim on one of two Joes, or a new third Joe with a connection, is not unresolved. Probes p56, p57.
M7. Same-name identities collapse in the fingerprint (proposals.rs:36-46). A genuine second "Joe" topic proposed by a model returns Accepted against the first. Probe p55; p54 is informational.
M8. A model can set its new topic to cloud_allowed, and that survives approval and satisfies the "explicit" gate at scopes.rs:462. proposals.rs:469-476, 545-549. Probe p23. Exposure is limited: context claims still come only from selected sources.
M9. source_context does not deny as a unit despite its comment. scopes.rs:465 checks only claim evidence and the per-hash label. A topic returned its name when connection-only evidence was later labelled local-only (r15), or when a newer revision at the same path was labelled (r16).
M10. Legacy search indexes topics/. scan.rs:28 has no exclusion, and `topics search` returned a local_only topic name from its topic and operation records. The plan says local-only knowledge must not leak through search. Human-only today, so not a model leak; by inspection undone records would also stay searchable.
M11. Repair can relax labels. Repair onto a path labelled local-only returned Unknown and made it eligible (scopes.rs:714-722, probe p24). Undoing a repair loses a tombstoned duplicate's local-only label (p25).
M12. Empty journal dir breaks history. Crash between mkdir and journal write: recover tolerates it, history_list returns internal permanently. mod.rs:210-213. Probe p02 (planted by hand).
M13. After recovery, proposals_list shows Pending though knowledge is accepted, until the same decision is re-sent. No reconciliation. Probe p03.
M14. Reanalysis terminal errors (p58, r05, r06 above): suppression alone or echoing known IDs fails the whole proposal, and stage has already consumed the run. proposals.rs:647-649, 180-195, 287-293, 716-721.
M15. Chat can propose Topic and Connection candidates outside its authorized topic; only claims are checked. proposals.rs:450-457. Probe p26: chat on A staged B->C.

## Low

L1. Undo false positive: mod.rs:375 treats a record as restored by matching data and any undo op. Create, label, relabel, label, undo last, then undo creation succeeds past live updates. Probe p42.
L2. Fingerprint not stable for connections to a provisional claim (Endpoint::Claim not normalised). Probe p53.
L3. source_context returns unresolved identities unrelated to the topic. scopes.rs:475. Probe r17.
L4. Locator heading is unverified, so a model can cite a forged heading. Probe p32.
L5. Local-only vs missing topic returns distinct error codes (forbidden vs invalid_input). Needs a UUID, no name leaks. Adapter must discard both, as planned. Probe r18.

## Design calls, not bugs (need a decision)

- r10: hiding a correction when the original source is out of scope prevents excerpt leakage, but leaves the model blind to the correction. Consider a redacted current claim with no passage.
- r13/r14: an edit creates no rule, so an edited-away "Friday" can be re-proposed for review. Consistent with the contract.
- Rule matching is exact-string and exact-predicate ("He/him" or another predicate is not matched). Contract says deterministic.

## Documented limits (not findings)

No cross-file atomicity or uncooperative-writer guarantee. Undo needs app-data snapshots. Undo restores data under a new op ID, so bytes differ by operation_id only. Reselecting a root orphans pending proposals (documented as disposable). No conflict auto-merge.

## Residual risks, not probed

- Ordering uses wall-clock created_at (persistence.rs:492): a backward clock step could misorder tombstones and labels.
- store.lock lives in app data, so two app-data dirs on one root are uncoordinated.
- New directories are not parent-fsynced, and no real power-loss test ran.
- A corrupt journal.json blocks every call for that root (persistence.rs:379), by inspection.
- Active runs survive restart with no reconciliation; the runtime owns this.

## Verified sound

Probes: p05 replayed marker idempotent; p06 knowledge rebuilt after full app-data loss; p43 undo of a correction restores the claim, source bytes unchanged; p59 rule does not touch another person and does not accept a replacement.
Reading only: strict fields and canonical UUIDs; write whitelist limited to topics/<kind>/<uuid>.md with symlink checks (workspace records cannot target source files); no model apply path in the service; validate_proposal non-consuming and stage consuming; run capability bound to root and scope; errors carry no outside paths.

## Checks run

- Baseline: cargo test -p topics-core -p topics-store -p topics-cli -p topics-bdd: 65 passed. workspace-bdd: 20 scenarios, 110 steps. topics-bdd 'not @ui': 15 scenarios, 78 steps.
- Probes: 53 written. 42 failed (each confirming the behaviour above). 11 passed, of which p54 and r14 have no assertion and r18 is a weak check.
- Legacy search: one synthetic CLI probe.

Not run: clippy, fmt, --locked, scripts/check.sh, full workspace, desktop or runtime. The four desktop/runtime-owned scenarios were not exercised, and the real adapter was not run; seam probes call the core directly. No model was invoked.

Inspected: commits 1b8ac16, ed401eb, cc402a6, b91a0db, cbce881, 85e9b77. Read: store workspace/{mod,persistence,proposals,scopes}.rs, parse.rs and scan.rs, core workspace.rs and workspace_validation.rs, CLI diff, service test fixture and test list, docs/contributing.md diff, PLAN, CONTRACT, all .feature files except runtime.feature, evidence red/green. Not read closely: docs/plan.md (only its size), runtime.feature, the workspace-bdd.rs step bodies, individual test bodies.

Environment: scratch tree with the manager's Cargo.lock; reused the core worktree's git-ignored target (it grew); tracked tree clean at 85e9b77; synthetic temp dirs only.
