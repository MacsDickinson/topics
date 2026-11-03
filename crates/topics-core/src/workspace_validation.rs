//! Deterministic shape validation. References/current bytes are checked by the store.
use crate::workspace::*;
fn invalid(message: &str) -> ApiError {
    ApiError::new("invalid_input", message)
}
pub fn validate_workspace_id(id: &str) -> WorkspaceResult<()> {
    if uuid::Uuid::parse_str(id).is_ok_and(|id_value| id_value.to_string() == id) {
        Ok(())
    } else {
        Err(invalid("Expected a canonical UUID identifier"))
    }
}
pub fn validate_date_range(from: &Option<Date>, to: &Option<Date>) -> WorkspaceResult<()> {
    for d in [from, to].into_iter().flatten() {
        if d.len() != 10
            || d.starts_with("0000")
            || chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").is_err()
        {
            return Err(invalid(
                "Dates must be supported ISO calendar dates (YYYY-MM-DD)",
            ));
        }
    }
    if from.as_ref().zip(to.as_ref()).is_some_and(|(a, b)| a > b) {
        return Err(invalid("Date range is reversed"));
    }
    Ok(())
}
pub fn validate_candidate_shapes(candidates: &[Candidate]) -> WorkspaceResult<()> {
    if candidates.is_empty() || candidates.len() > 200 {
        return Err(invalid("Submit 1–200 review candidates"));
    }
    for c in candidates {
        match c {
            Candidate::Topic { topic } => {
                validate_workspace_id(&topic.id)?;
                if topic.name.trim().is_empty() {
                    return Err(invalid("Topic name is required"));
                }
            }
            Candidate::Claim { claim } | Candidate::Correction { claim, .. } => {
                for id in [&claim.id, &claim.revision_id, &claim.subject_id] {
                    validate_workspace_id(id)?;
                }
                if claim.predicate.trim().is_empty()
                    || claim.value.trim().is_empty()
                    || claim.sources.is_empty()
                {
                    return Err(invalid("Claims require a predicate, value and provenance"));
                }
                if !chrono::DateTime::parse_from_rfc3339(&claim.recorded_at)
                    .is_ok_and(|d| d.offset().local_minus_utc() == 0)
                {
                    return Err(invalid("Claim timestamp must be RFC3339 UTC"));
                }
                validate_date_range(&claim.valid_from, &claim.valid_to)?;
                if let Candidate::Correction { rule, .. } = c {
                    validate_workspace_id(&rule.id)?;
                }
            }
            Candidate::Connection { connection } => {
                validate_workspace_id(&connection.id)?;
                if connection.relation.trim().is_empty() || connection.sources.is_empty() {
                    return Err(invalid("Connections require a relation and evidence"));
                }
                validate_date_range(&connection.valid_from, &connection.valid_to)?;
            }
            Candidate::UnresolvedIdentity {
                name,
                candidates,
                sources,
            } => {
                if name.trim().is_empty() || candidates.len() < 2 || sources.is_empty() {
                    return Err(invalid(
                        "Unresolved identity requires evidence and at least two candidates",
                    ));
                }
                for id in candidates {
                    validate_workspace_id(id)?;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mutation_input_rejects_authority_unknown_fields_and_missing_nullable_properties() {
        for v in [
            serde_json::json!({"kind":"approve","candidates":[],"approved":true}),
            serde_json::json!({"kind":"reject","operation_id":"forged"}),
        ] {
            assert!(serde_json::from_value::<Decision>(v).is_err());
        }
        assert!(
            serde_json::from_value::<ScopeSpec>(serde_json::json!({"files":[],"folders":[]}))
                .is_err()
        );
        assert!(
            serde_json::from_value::<ScopeSpec>(
                serde_json::json!({"files":[],"folders":[],"date_from":null,"date_to":null})
            )
            .is_ok()
        );
        assert!(serde_json::from_value::<Candidate>(serde_json::json!({"kind":"topic","topic":{"id":"x","name":"X","topic_kind":"person","aliases":[],"sensitivity":"unknown","status":"accepted"}})).is_err());
    }
    #[test]
    fn invalid_dates_and_noncanonical_ids_are_explicit_errors() {
        for d in ["2026-02-30", "2026-1-1", "0000-01-01", "yesterday"] {
            assert!(validate_date_range(&Some(d.into()), &None).is_err());
        }
        assert!(
            validate_date_range(&Some("2026-11-03".into()), &Some("2026-11-01".into())).is_err()
        );
        assert!(validate_date_range(&Some("2024-02-29".into()), &None).is_ok());
        assert!(validate_workspace_id("../outside").is_err());
        assert!(validate_workspace_id("F55E947C-BA3B-4EC2-8678-43EF190E863D").is_err());
    }
}
