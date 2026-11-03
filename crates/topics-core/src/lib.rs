//! Serializable domain data; no filesystem or persistence dependencies.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Document {
    pub path: String,
    pub title: String,
    pub kind: String,
    pub date: Option<String>,
    pub hash: String,
    pub modified_ns: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Person {
    pub name: String,
    pub path: String,
    pub known_as: Option<String>,
    pub pronouns: Option<String>,
    pub role: Option<String>,
    pub team: Option<String>,
    pub aliases: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchResult {
    pub title: String,
    pub path: String,
    /// FTS5 excerpt. Match spans use <mark> tags; render as text unless safely sanitized.
    pub snippet: String,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct IndexReport {
    pub document_count: usize,
    pub warnings: Vec<String>,
}

pub mod workspace;
pub use workspace::*;

mod workspace_validation;
pub use workspace_validation::{
    validate_candidate_shapes, validate_date_range, validate_workspace_id,
};
