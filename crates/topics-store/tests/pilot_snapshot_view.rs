#[path = "support/setup.rs"]
mod support;

use support::*;

#[test]
fn saving_the_same_meeting_view_twice_reuses_its_snapshot() {
    let fixture = Fixture::new();
    let briefing = fixture
        .service
        .pilot_briefing(fixture.root_request())
        .unwrap();
    let request = topics_core::PilotSnapshotCreateRequest {
        root_id: fixture.root_id.clone(),
        briefing_token: briefing.briefing_token,
    };
    let first = fixture
        .service
        .pilot_snapshot_create(request.clone())
        .unwrap();
    let second = fixture.service.pilot_snapshot_create(request).unwrap();
    assert_eq!(first.id, second.id);
    assert_eq!(
        fixture
            .service
            .pilot_snapshot_list(fixture.root_request())
            .unwrap(),
        vec![first]
    );
}

#[test]
fn reopening_an_offline_folder_keeps_its_identity_for_today() {
    let fixture = Fixture::new();
    std::fs::rename(&fixture.root, fixture.temp.path().join("offline")).unwrap();
    let reopened = topics_store::WorkspaceService::open(&fixture.app).unwrap();
    let root = reopened
        .root_get(topics_core::RootGetRequest {})
        .unwrap()
        .root
        .unwrap();
    assert_eq!(root.root_id, fixture.root_id);
    let briefing = reopened.pilot_briefing(fixture.root_request()).unwrap();
    assert!(
        briefing
            .sources
            .iter()
            .any(|s| s.status == topics_core::PilotSourceStatus::Unavailable)
    );
    assert!(!fixture.root.exists());
}
