use super::*;
use std::{collections::VecDeque, sync::Mutex};

// Only the digest and original display time are retained. A window can keep
// an earlier view open while another refreshes the same folder.
#[derive(Default)]
pub(super) struct IssuedViews(Mutex<VecDeque<IssuedView>>);

struct IssuedView {
    token: Id,
    digest: Hash,
    checked_at: Timestamp,
}

impl WorkspaceService {
    pub(super) fn issue_pilot_view(&self, mut briefing: PilotBriefing) -> WResult<PilotBriefing> {
        let mut views = self.pilot_views.0.lock().map_err(internal)?;
        let token = new_id();
        let digest = std::mem::replace(&mut briefing.briefing_token, token.clone());
        views.push_back(IssuedView {
            token,
            digest,
            checked_at: briefing.generated_at.clone(),
        });
        if views.len() > 128 {
            views.pop_front();
        }
        Ok(briefing)
    }

    // The caller holds the workspace lock through verification and snapshot commit.
    pub(super) fn observed_pilot_briefing(
        &self,
        ctx: &Context,
        token: &str,
    ) -> WResult<PilotBriefing> {
        let stale = || {
            error(
                "stale_scope",
                "Refresh Today and review the briefing before saving.",
            )
        };
        let (digest, checked_at) = self
            .pilot_views
            .0
            .lock()
            .map_err(internal)?
            .iter()
            .find(|view| view.token == token)
            .map(|view| (view.digest.clone(), view.checked_at.clone()))
            .ok_or_else(stale)?;
        let mut briefing = pilot_briefing::project(ctx)?;
        if digest != briefing.briefing_token {
            return Err(stale());
        }
        briefing.briefing_token = token.to_owned();
        briefing.generated_at = checked_at;
        Ok(briefing)
    }
}
