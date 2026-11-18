//! Human-only local Friday pilot projection; never a model tool contract.
use crate::{Hash, Id, SourceRef, Timestamp};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PilotBriefing {
    pub root_id: Id,
    pub generated_at: Timestamp,
    pub briefing_token: Hash,
    pub rules: Vec<PilotRule>,
    pub occurrences: Vec<PilotOccurrence>,
    pub sources: Vec<PilotSource>,
    pub items: Vec<PilotItem>,
}
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PilotSource {
    pub source_id: Id,
    pub path: String,
    pub current_hash: Option<Hash>,
    pub analysed_hash: Option<Hash>,
    pub status: PilotSourceStatus,
    pub last_success_at: Option<Timestamp>,
}
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PilotSourceStatus {
    Checked,
    Pending,
    Unavailable,
    Skipped,
    Partial,
}
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PilotItem {
    pub item_id: Id,
    pub topic_id: Id,
    pub topic_name: String,
    pub revision_id: Id,
    pub predicate: String,
    pub value: String,
    pub sources: Vec<SourceRef>,
    pub rule_id: Option<Id>,
    pub original_value: Option<String>,
}

/// Approved scoped correction available at briefing time, including rules with no later example.
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PilotRule {
    pub rule_id: Id,
    pub topic_id: Id,
    pub topic_name: String,
    pub predicate: String,
    pub match_value: String,
    pub replacement: String,
}
/// An independently observed, post-approval occurrence in a completed analysis.
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PilotOccurrence {
    pub rule_id: Id,
    pub occurrence_id: Id,
    pub observed_at: Timestamp,
    pub recurred: bool,
}
