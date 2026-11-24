//! Exercise the actual MCP executable and its configured caller scope.
use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};
use topics_core::*;
use topics_store::WorkspaceService;

#[test]
fn stdio_search_and_detail_only_return_released_knowledge_in_client_scope() {
    let temp = tempfile::tempdir().unwrap();
    let data = temp.path().join("data");
    let notes = temp.path().join("notes");
    let mut service = WorkspaceService::open(&data).unwrap();
    let root = service
        .root_set(RootSetRequest {
            path: notes.to_string_lossy().into_owned(),
            create: true,
        })
        .unwrap()
        .root
        .root_id;
    let allowed = "00000000-0000-4000-8000-000000000001";
    let mut candidates = vec![];
    for (suffix, name, sensitivity) in [
        (1, "Atlas", Sensitivity::CloudAllowed),
        (2, "OtherClientOnly", Sensitivity::CloudAllowed),
        (3, "PersonalOnly", Sensitivity::LocalOnly),
        (4, "UnknownSentinel", Sensitivity::Unknown),
    ] {
        candidates.push(Candidate::Topic {
            topic: Topic {
                id: format!("00000000-0000-4000-8000-{suffix:012}"),
                name: name.into(),
                aliases: vec![],
                topic_kind: TopicKind::Project,
                sensitivity,
            },
        });
    }
    let proposal = service
        .manual_propose(ManualProposeRequest {
            root_id: root.clone(),
            candidates,
        })
        .unwrap()
        .proposal;
    service
        .proposal_decide(ProposalDecideRequest {
            root_id: root.clone(),
            proposal_id: proposal.id,
            expected_fingerprint: proposal.fingerprint,
            decision: Decision::Approve {
                candidates: proposal.candidates,
            },
        })
        .unwrap();
    let session = || {
        let mut child = Command::new(env!("CARGO_BIN_EXE_topics"))
            .args([
                "mcp",
                &root,
                allowed,
                "00000000-0000-4000-8000-000000000003",
                "00000000-0000-4000-8000-000000000004",
            ])
            .env("TOPICS_APP_DATA_DIR", &data)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut input = child.stdin.take().unwrap();
        for request in [
            json!({"id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","clientInfo":{"name":"test","version":"1"},"capabilities":{}}}),
            json!({"method":"notifications/initialized"}),
            json!({"id":2,"method":"tools/list"}),
            json!({"id":3,"method":"tools/call","params":{"name":"topic_search","arguments":{"query":""}}}),
            json!({"id":4,"method":"tools/call","params":{"name":"topic_detail","arguments":{"topic_id":allowed}}}),
            json!({"id":5,"method":"tools/call","params":{"name":"topic_search","arguments":{"query":"","root_id":root}}}),
            json!({"id":6,"method":"tools/call","params":{"name":"topic_detail","arguments":{"topic_id":"00000000-0000-4000-8000-000000000002"}}}),
            json!({"id":7,"method":"tools/call","params":{"name":"source_get","arguments":{}}}),
        ] {
            let mut request = request;
            request["jsonrpc"] = json!("2.0");
            writeln!(input, "{request}").unwrap();
        }
        drop(input);
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        for secret in ["OtherClientOnly", "PersonalOnly", "UnknownSentinel"] {
            assert!(!text.contains(secret));
        }
        text.lines()
            .map(|s| serde_json::from_str::<Value>(s).unwrap())
            .collect::<Vec<_>>()
    };
    assert_eq!(session()[2]["result"]["isError"], true);
    service
        .retrieval_settings_set(RetrievalSettingsSetRequest {
            root_id: root.clone(),
            enabled: true,
        })
        .unwrap();
    let out = session();
    assert_eq!(out.len(), 7);
    assert_eq!(out[1]["result"]["tools"].as_array().unwrap().len(), 2);
    assert!(
        out[2]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Atlas")
    );
    assert!(
        out[3]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains(allowed)
    );
    assert_eq!(out[4]["error"]["code"], -32602);
    assert_eq!(out[5]["result"]["isError"], true);
    assert_eq!(out[6]["error"]["code"], -32602);
    service
        .retrieval_settings_set(RetrievalSettingsSetRequest {
            root_id: root.clone(),
            enabled: false,
        })
        .unwrap();
    assert_eq!(session()[3]["result"]["isError"], true);
}
