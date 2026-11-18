#[path = "support/setup.rs"]
mod support;

use support::*;
use topics_core::PilotSnapshotCreateRequest;

#[test]
fn capture_preserves_the_displayed_check_time_and_detects_an_unseen_edit() {
    let mut fixture = Fixture::new();
    fixture.write("meetings/first.md", "# Synthetic\nFirst meeting.");
    let root = fixture.root_request();
    let displayed = fixture.service.pilot_briefing(root.clone()).unwrap();
    let refreshed = fixture.service.pilot_briefing(root.clone()).unwrap();
    let snapshot = fixture
        .service
        .pilot_snapshot_create(PilotSnapshotCreateRequest {
            root_id: root.root_id.clone(),
            briefing_token: displayed.briefing_token.clone(),
        })
        .unwrap();
    assert_eq!(snapshot.briefing, displayed);

    fixture.write("meetings/first.md", "# Synthetic\nDifferent decision.");
    let error = fixture
        .service
        .pilot_snapshot_create(PilotSnapshotCreateRequest {
            root_id: root.root_id,
            briefing_token: refreshed.briefing_token,
        })
        .unwrap_err();
    assert_eq!(error.code, "stale_scope");
}
