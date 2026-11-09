use super::*;
#[test]
fn setup_capability_rejects_scope_widening_and_missing_required_members() {
    let capability = SetupBatchCapability {
        root_id: "root-generation".into(),
        job_id: "job".into(),
        batch_id: "batch".into(),
        scope_id: "scope".into(),
        run_id: "run".into(),
        manifest_fingerprint: "manifest-hash".into(),
        attempt: 1,
    };
    let mut value = serde_json::to_value(&capability).unwrap();
    value["allow_all_files"] = serde_json::json!(true);
    assert!(serde_json::from_value::<SetupBatchCapability>(value.clone()).is_err());
    value.as_object_mut().unwrap().remove("allow_all_files");
    value.as_object_mut().unwrap().remove("scope_id");
    assert!(serde_json::from_value::<SetupBatchCapability>(value).is_err());
}
#[test]
fn nullable_review_members_are_required_and_verification_is_explicit() {
    let metadata = ReviewMetadata {
        id: "review".into(),
        record_id: "revision".into(),
        state: ReviewState::FromNotes,
        receipt_id: None,
        statement_id: None,
        recorded_at: "2026-11-09T00:00:00Z".into(),
    };
    let mut wire = serde_json::to_value(&metadata).unwrap();
    assert_eq!(wire["state"], "from_notes");
    assert_eq!(
        serde_json::from_value::<ReviewMetadata>(wire.clone()).unwrap(),
        metadata
    );
    wire.as_object_mut().unwrap().remove("statement_id");
    assert!(serde_json::from_value::<ReviewMetadata>(wire).is_err());
}
