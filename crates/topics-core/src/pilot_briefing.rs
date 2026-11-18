//! Human-only local Friday pilot projection; never a model tool contract.
use crate::{Hash, Id, SourceRef, Timestamp};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PilotBriefing {
    pub root_id: Id,
    pub generated_at: Timestamp,
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
