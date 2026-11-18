#[path = "support/setup.rs"]
mod support;
use support::*;
use topics_core::*;

#[test]
fn unanalysed_local_source_is_pending_and_never_claims_a_success() {
    let mut f = Fixture::new();
    f.write("project/meeting.md", "# Synthetic meeting\nA decision.\n");
    let briefing = f.service.pilot_briefing(f.root_request()).unwrap();
    assert_eq!(briefing.sources.len(), 1);
    assert_eq!(briefing.sources[0].status, PilotSourceStatus::Pending);
    assert_eq!(briefing.sources[0].last_success_at, None);
    assert!(briefing.items.is_empty());
}
