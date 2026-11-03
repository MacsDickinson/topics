//! Executable runtime.feature boundary scenarios; fixtures contain no personal data.
use serde_json::json;
use topics_runtime::bridge::SessionGate;
#[test]
fn tool_boundary_denies_spoofed_wrong_session_expired_and_unknown() {
    let mut gate = SessionGate::new("session", "scope");
    let token = gate.capability().to_owned();
    assert!(
        gate.authorize(
            "session",
            &token,
            "scope_sources",
            &json!({"scope_id":"scope"})
        )
        .is_ok()
    );
    for (session, cap, tool, args) in [
        (
            "other",
            token.as_str(),
            "scope_sources",
            json!({"scope_id":"scope"}),
        ),
        (
            "session",
            "wrong",
            "scope_sources",
            json!({"scope_id":"scope"}),
        ),
        (
            "session",
            token.as_str(),
            "workspace_proposal_decide",
            json!({}),
        ),
        (
            "session",
            token.as_str(),
            "scope_sources",
            json!({"scope_id":"private"}),
        ),
        (
            "session",
            token.as_str(),
            "scope_sources",
            json!({"scope_id":"scope","source_id":"unselected"}),
        ),
    ] {
        assert!(gate.authorize(session, cap, tool, &args).is_err());
    }
    gate.close();
    assert!(
        gate.authorize(
            "session",
            &token,
            "scope_sources",
            &json!({"scope_id":"scope"})
        )
        .is_err()
    );
}
#[test]
fn malformed_model_output_and_authority_fields_are_rejected() {
    let gate = SessionGate::new("session", "scope");
    for args in [
        json!({"candidates":"bad"}),
        json!({"candidates":[],"origin":{"kind":"manual"}}),
        json!({"candidates":[{"kind":"claim","claim":{"sources":[{"path":"../private"}]}}]}),
    ] {
        assert!(
            gate.authorize("session", gate.capability(), "propose_changes", &args)
                .is_err()
        );
    }
}
#[cfg(target_os = "macos")]
#[test]
fn native_root_canonical_symlink_unicode_quotes_denied_sibling_allowed() {
    let temp = tempfile::tempdir().unwrap();
    let protected = temp.path().join("notes ' quoted ü");
    std::fs::create_dir(&protected).unwrap();
    topics_runtime::isolation::preflight(&protected).unwrap();
}
#[test]
fn unknown_effective_tool_surface_blocks_selected_bytes() {
    assert!(
        topics_runtime::protocol::validate_inventory(
            &json!({"data":[{"name":"unexpected","status":"ready","tools":{}}],"nextCursor":null})
        )
        .is_err()
    );
}

#[test]
fn missing_event_identity_cannot_be_mistaken_for_current_run() {
    for frame in [
        json!({"method":"item/agentMessage/delta","params":{"turnId":"turn","delta":"late"}}),
        json!({"method":"turn/completed","params":{"threadId":"thread","turn":{"id":"old"}}}),
    ] {
        assert!(
            topics_runtime::protocol::validate_event_context(&frame, "thread", "turn").is_err()
        );
    }
}
#[cfg(not(target_os = "macos"))]
#[test]
fn unsupported_os_cannot_enable_analysis() {
    let temp = tempfile::tempdir().unwrap();
    assert!(topics_runtime::isolation::preflight(temp.path()).is_err());
}
