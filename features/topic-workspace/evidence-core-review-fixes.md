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
