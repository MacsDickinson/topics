//! Supplemental human notes have an explicit durable purpose, separate from claim evidence.
use super::*;
const PURPOSE: &str = "Record supplemental topic context";
pub(super) fn notes(state: &State, topic: &str) -> Vec<UserStatement> {
    state
        .statements
        .values()
        .filter(|statement| {
            statement.topic_id == topic
                && state.operations.iter().any(|operation| {
                    operation.operation.description == PURPOSE
                        && !undone(state, &operation.operation.id)
                        && operation.targets.iter().any(|target| {
                            target.path == format!("topics/statements/{}.md", statement.id)
                        })
                })
        })
        .cloned()
        .collect()
}
impl WorkspaceService {
    pub fn setup_context_add(&self, request: UserStatementRequest) -> WResult<UserStatement> {
        self.with(&request.root_id, |ctx| {
            let state = load(ctx)?;
            valid_id(&request.topic_id)?;
            if !state.topics.contains_key(&request.topic_id) {
                return Err(error(
                    "invalid_input",
                    "Choose an accepted topic for supplemental context",
                ));
            }
            if request.text.trim().is_empty() || request.text.len() > 32768 {
                return Err(error("invalid_input", "Context must contain 1–32768 bytes"));
            }
            let statement = UserStatement {
                id: new_id(),
                topic_id: request.topic_id.clone(),
                text: request.text.clone(),
                recorded_at: now(),
            };
            let op = new_id();
            commit(
                ctx,
                PURPOSE,
                vec![(
                    format!("topics/statements/{}.md", statement.id),
                    Record::new("statement", &statement.id, &op, &statement)?,
                )],
                CommitIntent {
                    dependencies: vec![request.topic_id.clone()],
                    tombstones: vec![],
                    undo_of: None,
                    expected: Some(&state.hashes),
                    proposal: None,
                },
            )?;
            Ok(statement)
        })
    }
}
