# Independent-review corrections

Baseline `85e9b77`. Adopted the reviewer's 53 synthetic service probes before implementation; actual RED was 5 passed / 30 failed in review_probes and 6 passed / 12 failed in review_reanalysis (42 failures total). Original reviewer sources remain unchanged in their scratch directory; committed regression copies will document necessary fixture changes when model-supplied sensitivity becomes non-authoritative.

```text
topics-review-red.log
     Locking 1 package to latest compatible version
      Adding sha1_smol v1.0.1
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.93s
     Running tests/review_probes.rs (target/debug/deps/review_probes-3403d70422631994)
    p01_crash_leftover_tmp_in_proposals_must_not_brick
    p02_crash_between_mkdir_and_journal_must_not_break_history
    p03_recovered_approval_must_show_accepted_in_proposals_list
    p04_recovery_conflict_must_not_hide_previously_accepted_topic
    p10_edited_local_only_label_record_must_fail_closed
    p11_deleted_local_only_label_record_must_fail_closed
    p12_restoring_older_committed_bytes_must_not_downgrade_label
    p13_restoring_older_topic_bytes_must_warn_or_keep_local_only
    p14_deleted_superseding_claim_must_warn_not_silently_revive_false_claim
    p20_renamed_local_only_source_must_stay_restricted
    p21_case_alias_must_not_bypass_local_only_label
    p22_case_alias_must_not_expose_app_records
    p23_model_must_not_self_label_new_topic_cloud_allowed
    p24_repair_onto_local_only_path_must_not_relax_label
    p25_undo_of_repair_must_keep_local_only_label_of_tombstoned_duplicate
    p26_chat_must_stay_within_authorized_topic
    p30_edited_connection_must_not_keep_unqualified_transcript_citation
    p31_edited_claim_subject_must_record_provenance
    p32_citation_heading_must_be_verified
    p40_undo_after_decide_allows_reanalysis_to_propose_again
    p41_undo_claim_with_connection_on_logical_id_must_be_refused
    p42_undo_creation_must_not_skip_live_intermediate_updates
    p50_unrelated_label_change_must_not_permanently_conflict_pending_proposal
    p51_stale_unrelated_scope_file_must_not_strand_fresh_reanalysis
    p52_equivalent_claim_must_not_duplicate_via_different_candidate_set
    p53_fingerprint_stable_for_connection_to_provisional_claim
    p55_second_real_person_same_name_can_still_be_proposed_by_model
    p56_model_claim_on_ambiguous_same_name_must_be_unresolved
    p57_new_provisional_same_name_with_connection_must_be_unresolved
    p58_suppressed_only_output_must_not_fail_the_run
test result: FAILED. 5 passed; 30 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.36s

topics-reanalysis-red.log
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.14s
     Running tests/review_reanalysis.rs (target/debug/deps/review_reanalysis-58feda63a8273819)
    r02_random_topic_id_bypasses_rule_when_fingerprint_differs_by_uncertainty
    r03_random_topic_id_bypasses_rule_when_extra_fact_added
    r05_reused_topic_candidate_with_known_id_is_usable
    r06_reusing_logical_claim_id_with_unchanged_value_is_not_terminal_error
    r07_known_context_reproposal_of_corrected_value_must_not_duplicate
    r08_id_reuse_connection_must_dedupe_against_first_run
    r10_context_without_original_source_must_still_show_correction
    r12_new_meeting_random_topic_false_claim_bypasses_rule
    r13_human_edited_claim_must_appear_in_model_context
    r15_topic_named_from_newly_local_only_connection_source_denied_as_unit
    r16_old_revision_of_path_labelled_local_only_denied_as_unit
    r17_context_must_not_include_other_topics_unresolved_identities
test result: FAILED. 6 passed; 12 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.85s
```

## Final implementation and checks

Code milestones after the RED: `f56fb37` closes the privacy/reanalysis blockers; `d3202eb` adds final review regressions and the guarded publication seam. The final follow-up also requires matching temporal bounds for provisional identity proof and rechecks the actual returned scoped byte buffer against its confirmed hash. The working tree was held idle for the independent review before these corrections; no original reviewer scratch files were edited.

Final actual commands, all exit 0 (all builds use this worktree's target, with incremental artifacts disabled):

```text
cargo fmt -p topics-core -p topics-store -p topics-cli -p topics-bdd --check
CARGO_INCREMENTAL=0 cargo clippy --offline -p topics-core -p topics-store -p topics-cli -p topics-bdd --all-targets -- -D warnings
CARGO_INCREMENTAL=0 cargo test --offline -p topics-core -p topics-store -p topics-cli -p topics-bdd --features topics-store/test-support --no-fail-fast
  131 passed, 0 failed, 0 ignored
CARGO_INCREMENTAL=0 cargo run --offline -p topics-bdd --bin workspace-bdd
  4 features; 20 scenarios, 110 steps passed; zero skipped/undefined
CARGO_INCREMENTAL=0 cargo run --offline -p topics-bdd -- --tags 'not @ui'
  3 features; 15 scenarios, 78 steps passed; zero skipped/undefined
```

Breakdown: the prior 65 unit/integration regressions remain green; `review_probes` has 45 passing probes and `review_reanalysis` has 21. The original 53 reviewer probes are included, with one explicit human-label fixture change required by M8 (the model can no longer authorize CloudAllowed); p54/r14/r18 were strengthened from diagnostic/weak checks to actual assertions. Other additions cover redacted output bytes/metadata, missing commit markers, registered rename destinations, distinct same-name identities with changed evidence/endpoints/dates, edited-subset dependency narrowing, endpoint headings, useful human FTS, recovery-conflict reanalysis, unknown target preservation, and two guarded-publication tests. The unknown-target test was corrected to use a service-issued canonical source ID rather than an incorrect path-derived fixture ID; its initial fixture error is not counted as a product RED.

Additional local RED results were actually observed before their local corrections: missing committed marker, previously registered unknown rename destination, two distinct same-name identity cases, out-of-scope statement-only redaction, omitted-subset dependencies, source-endpoint heading, recovery-conflict pending status, and temporal identity mismatch. Guarded publication follows the runtime review's F6 failure; this worker's tests prove the core callback boundary, not the actual runtime permit/process behavior.

Both BDD runners retain `fail_on_skipped`. The four unchanged presentation/runtime-owned scenarios remain mapped in [the initial all-24 integration table](evidence-core-green.md); they are not counted as tested here. No feature text or Phase 0 step/snapshot suite was changed. Two workspace step assertions now correctly expect a successful empty no-change outcome and persisted Conflict status, matching the coordinator's approved review decisions rather than preserving the original bugs.

## Finding disposition

| Finding | Concrete correction and evidence |
|---|---|
| H1 | Safe provisional matching requires a unique accepted identity, compatible aliases, exact cited assertion or complete relationship endpoints, and matching temporal bounds; otherwise same-name identity stays unresolved. Rules then apply to the canonical identity. r02/r03/r12 and new distinct-person/temporal probes pass. |
| H2 | Claim/connection dedupe uses current accepted records independently of whole candidate sets, retaining accepted IDs and user-statement provenance. Reused logical/revision/endpoint IDs work; p52/r06/r07/r08 pass. |
| H3 | Latest target hashes and typed committed snapshots detect missing, edited and rolled-back mappings; recovered source/topic labels are restricted, not Unknown. Unknown/damaged target bytes are not silently overwritten by registration. p10/p11/p12 and target-preservation probe pass. |
| H4 | Restrictive same-content identity history survives rename, including a destination already registered Unknown; explicit human review can release normal source labels. p20 and registered-destination probe pass. |
| H5 | Selection resolves actual filesystem identity before policy, including APFS case aliases; reserved root topics spellings are denied. p21/p22 pass on this macOS synthetic filesystem. |
| H6 | Expected hashes are limited to actual selected/edited dependencies. Conflict status invalidates stale pending dedupe; changed unrelated scope documents cannot strand approval/fresh analysis. p50/p51 and subset/recovery-refresh probes pass. |
| H7 | Canonical UUID JSON readers ignore hidden atomic temp artifacts, including proposal/run directories; malformed disposable entries do not brick unrelated operations. p01 plus guarded staging tests pass. |
| M1 | Undone operations reconcile as Conflict and are excluded from live decision dedupe; fresh provisional IDs can re-propose absent facts. p40 passes. Historical identifiers remain reserved. |
| M2 | Recovery first preserves three versions, then restores only the app's still-matching staged replacements to prior accepted bytes. p04 passes without overwriting the external target. |
| M3 | Projection uses latest committed targets and warns on missing/rollback; typed snapshots preserve the superseding revision. Retained committed journals also detect a missing/changed marker, preventing silent false-head revival. p13/p14 and marker-loss probe pass. |
| M4 | Undo includes logical claim IDs and later target writes in dependencies. p41 passes. |
| M5 | Subject/uncertainty edits attach persisted full before/after user provenance while preserving the submitted uncertainty. Relationship edits are visibly HUMAN-EDITED and link persisted statements or durable operation assertions. p30/p31 and existing edit-provenance regression pass. |
| M6 | Claims as well as connections resolve ambiguous known/provisional names into unresolved candidates. Chat's explicitly selected identity remains scoped. p56/p57 pass. |
| M7 | Topic-only second identities and ambiguous same-name graphs retain distinct fingerprints; names alone do not collapse entities. p54/p55 and distinct-evidence/endpoint/temporal probes pass. |
| M8 | New model topics cannot self-grant CloudAllowed; human sensitivity decisions remain authoritative. p23 passes; context fixture now performs that explicit human action. |
| M9 | Context checks current same-path labels and connection-only evidence before any redaction and denies tainted topics as a unit. r15/r16 plus redaction/taint probe pass. |
| M10 | Human FTS excludes root app-owned topics records and retains source search, including human access to their own private notes. It is never an MCP retrieval surface. Human-search regression passes. |
| M11 | Repair inherits restrictive target-path policy even when the target hash is not registered; label history survives duplicate tombstones/repair undo. p24/p25 pass. |
| M12 | Empty/incomplete journals are ignored; malformed journals are retained with warnings. p02 passes. |
| M13 | Proposal listing reconciles committed recovery and conflict journals without another decision request. p03 and recovery-refresh probe pass. |
| M14 | Known topic echoes, already-known values and suppression-only outputs succeed; a zero-candidate accepted result creates no operation/knowledge. p58/r04/r05/r06/r11 and guarded no-change tests pass. |
| M15 | Every chat candidate kind is checked against its authorized topic; unrelated topic creation/relationships/unresolved sets are denied. p26 passes. |
| L1 | Undo cannot skip live intermediate target updates; compensation must correspond to the latest write. p42 and prior undo-order regression pass. |
| L2 | Canonical fingerprints map provisional logical and revision claim endpoints. p53 passes. |
| L3 | Model unresolved context is filtered to the requested topic, selected evidence and eligible candidates. r17 passes. |
| L4 | Heading hints are verified for cited passages and source endpoints, including UTF-8 byte/range/hash checks. p32 and endpoint-heading probe pass. |
| L5 | Missing/private context UUIDs return the same sanitized forbidden error; no name leaks. Strengthened r18 passes. |

## Guarded publication and integration

Exact runtime-independent API:

```rust,ignore
WorkspaceService::stage_proposal_guarded(
    request: StageProposalRequest,
    publisher: impl FnOnce(
        &Proposal,
        &mut dyn FnMut() -> WorkspaceResult<()>,
    ) -> WorkspaceResult<()>,
) -> WorkspaceResult<Proposal>
```

Preparation and flushed hidden JSON happen before the publisher. The publisher wraps only the final idempotent rename; denied publication drops the temp and creates no pending record. Ordinary staging calls the action unconditionally. Pending dedupe and accepted empty outcomes also traverse the permit. The caller passes `|proposal, action| permit.publish(vec![proposal.id.clone()], action)`. The runtime owns deadline/revocation arbitration and successful-terminal buffering; core does not import runtime types. The delayed callback test checks that a fully prepared temporary file exists while final visibility does not, then revokes the permit and confirms no pending/root knowledge appears. A success test also covers duplicate and empty outcomes, once-only visibility and no new accepted operation. Validation-only remains non-consuming/non-publishing.

Pending caches are disposable; file bytes flush before publication and directory durability is best effort outside the gate. Accepted Markdown writes retain the snapshot/journal/marker protocol. Scope reads also compare the actual outgoing bytes with their confirmed hash, rather than relying only on an earlier validation read. No API/type changes beyond the requested Rust guarded staging method, no runtime dependency, no root manifest/check-script ownership change and no new dependencies were introduced.

## Limits and remaining acceptance

All data was synthetic temp-root/app-data data; no live notes, provider, personal config/auth or notes Git. Only the idle generated incremental cache in this exact worktree was removed with path/symlink guards; no other target/cache/source/evidence was cleaned. Remaining acceptance is the independent Opus re-run on the final diff and manager-owned native/runtime/integration checks. This self-verification is not an independent ACCEPT.

The coordinator approved redacted current human-supported values for explicitly authorized nonprivate CloudAllowed topics: excluded evidence/source/path/superseded IDs and unrequested statement text are absent, history remains scoped, and taint is checked first. The analysis adapter must still select context only through eligible evidence, not an all-topic export. Plain edits do not create implicit rules; deterministic rules match exact predicates/values, and replacements always require review.

New operation ordering is root-local monotonic; legacy ordering is retained for legacy records. Same-app-data cooperative locking, per-file replacement and snapshot retention remain required. There is no cross-file atomicity, universal uncooperative-writer/power-loss guarantee, arbitrary-copy recognition, cross-app-data writer coordination or conflict auto-merge. Missing/changed committed payloads are recoverable from surviving typed snapshots/journals with warnings; wholesale erasure of all later Markdown metadata and app-data history cannot be distinguished from a truly older root. Conservative private-history taint may require explicit source/citation review. Source/citation repair never silently reanchors old evidence.
