//! Explicit native acceptance, executed with --features native-live-tests.
//! Uses existing runtime subscription auth and exclusively synthetic temporary data.
#![cfg(all(target_os = "macos", feature = "native-live-tests"))]
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};
use topics_core::workspace::*;
use topics_runtime::{HostTools, RunRequest, Runtime, RuntimeConfig};
fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
struct Host {
    scope: String,
    path: PathBuf,
    source: SourceRef,
    staged: Mutex<Vec<Candidate>>,
    reads: Mutex<usize>,
    statement: String,
    topic: String,
    old_revision: String,
    correction: bool,
}
impl HostTools for Host {
    fn call(&self, tool: &str, args: Value) -> WorkspaceResult<Value> {
        println!("sanitized host tool_call={tool}");
        match tool {
            "scope_sources" if args == json!({"scope_id":self.scope}) => {
                let text = std::fs::read_to_string(&self.path).unwrap();
                assert_eq!(hash(text.as_bytes()), self.source.document_hash);
                *self.reads.lock().unwrap() += 1;
                Ok(
                    json!({"sources":[{"summary":{"source_id":self.source.source_id,"path":self.source.path,"hash":self.source.document_hash,"date":"2026-11-01","sensitivity":"cloud_allowed"},"content":text,"citation":self.source}]}),
                )
            }
            "propose_changes" => {
                let values: Vec<Candidate> = serde_json::from_value(args["candidates"].clone())
                    .map_err(|_| ApiError::new("invalid_input", "Invalid candidate"))?;
                for c in &values {
                    let sources = match c {
                        Candidate::Connection { connection } => connection.sources.clone(),
                        Candidate::Claim { claim } | Candidate::Correction { claim, .. } => claim
                            .sources
                            .iter()
                            .filter_map(|e| {
                                if let Evidence::Passage { source } = e {
                                    Some(source.clone())
                                } else {
                                    None
                                }
                            })
                            .collect(),
                        Candidate::UnresolvedIdentity { sources, .. } => sources.clone(),
                        _ => vec![],
                    };
                    if sources.iter().any(|s| s != &self.source) {
                        return Err(ApiError::new(
                            "forbidden",
                            "Source outside authorized snapshot",
                        ));
                    }
                    if self.correction {
                        match c {
                            Candidate::Correction { claim, rule }
                                if claim.subject_id == self.topic
                                    && claim.value == "they/them"
                                    && claim.supersedes == vec![self.old_revision.clone()]
                                    && claim.sources
                                        == vec![Evidence::UserStatement {
                                            statement_id: self.statement.clone(),
                                        }]
                                    && rule.topic_id == self.topic
                                    && rule.predicate == "pronouns"
                                    && rule.match_value == "he/him"
                                    && rule.replacement == "they/them" => {}
                            _ => {
                                return Err(ApiError::new(
                                    "invalid_input",
                                    "Correction must cite the persisted statement and scoped current revision",
                                ));
                            }
                        }
                    }
                }
                Ok(json!({"validated":true}))
            }
            _ => Err(ApiError::new("forbidden", "Outside selected scope")),
        }
    }
    fn commit_candidates(&self, values: Vec<Candidate>) -> WorkspaceResult<Vec<String>> {
        assert!(self.staged.lock().unwrap().is_empty());
        self.call("propose_changes", json!({"candidates":values}))?;
        *self.staged.lock().unwrap() = values;
        Ok(vec![id()])
    }
}
#[test]
fn subscription_codex_production_sdk_bridge_discovers_cited_candidate_and_corrects() {
    let fixture = tempfile::tempdir().unwrap();
    let notes = fixture.path().join("notes ' ü");
    std::fs::create_dir(&notes).unwrap();
    let bytes = "Alex leads Ledger. Alex uses he/him pronouns.\n";
    let source_path = notes.join("one.md");
    std::fs::write(&source_path, bytes).unwrap();
    std::fs::write(notes.join("private.md"), "PRIVATE_SENTINEL_NEVER_SELECTED").unwrap();
    let source = SourceRef {
        source_id: id(),
        path: "one.md".into(),
        document_hash: hash(bytes.as_bytes()),
        locator: Locator {
            start_byte: 0,
            end_byte: bytes.len() as u64,
            heading: None,
        },
        passage_hash: hash(bytes.as_bytes()),
        excerpt: bytes.into(),
    };
    let runtime = Runtime::new(RuntimeConfig {
        codex_path: PathBuf::from(
            std::env::var_os("TOPICS_TEST_CODEX").unwrap_or_else(|| "codex".into()),
        ),
        proxy_path: PathBuf::from(env!("CARGO_BIN_EXE_topics-mcp-proxy")),
        app_data: fixture.path().join("app-data"),
    });
    assert!(runtime.status().available, "{}", runtime.status().message);
    let topic = id();
    let old_revision = id();
    let statement = id();
    for correction in [false, true] {
        let host = Arc::new(Host {
            scope: id(),
            path: source_path.clone(),
            source: source.clone(),
            staged: Mutex::new(vec![]),
            reads: Mutex::new(0),
            statement: statement.clone(),
            topic: topic.clone(),
            old_revision: old_revision.clone(),
            correction,
        });
        let prompt = if !correction {
            "Discover person Alex, project Ledger, and a leads connection with the provided full passage citation. Propose these three candidates together with unique UUID IDs, cloud_allowed sensitivity, explicit null optional fields, and stable provisional dependencies. Do not mention or access private/unselected sources.".to_owned()
        } else {
            // The app persists the statement before providing bounded accepted context.
            let statement_record=UserStatement{id:statement.clone(),topic_id:topic.clone(),text:"Alex uses they/them pronouns. Correct that and remember it for this person's pronouns.".into(),recorded_at:"2026-11-03T00:00:00Z".into()};
            let statements = fixture.path().join("statements");
            std::fs::create_dir_all(&statements).unwrap();
            std::fs::write(
                statements.join(format!("{statement}.json")),
                serde_json::to_vec(&statement_record).unwrap(),
            )
            .unwrap();
            format!(
                "Authorized persisted topic context: {}. Propose exactly one correction Candidate with claim predicate pronouns, value they/them, subject_id from topic, new UUID revision_id, supersedes containing only current revision_id, and sources ONLY user_statement evidence with persisted statement id. Rule must target same topic/predicate pronouns, match_value he/him, replacement they/them, source_revision_id equal new revision, scope source_paths [] and context null. Do not infer identity or apply the correction.",
                json!({"topic":{"id":topic,"name":"Alex"},"current_claim":{"id":id(),"revision_id":old_revision,"predicate":"pronouns","value":"he/him"},"rules":[],"statement":statement_record})
            )
        };
        let request = RunRequest {
            root_id: id(),
            run_id: id(),
            scope_id: host.scope.clone(),
            notes_root: notes.clone(),
            prompt,
        };
        let (tx, rx) = mpsc::channel();
        runtime
            .analysis_start(
                request.clone(),
                host.clone(),
                Arc::new(move |e| {
                    let _ = tx.send(e);
                }),
            )
            .unwrap();
        let mut terminal = 0;
        let mut last_seq = 0;
        let mut proposals = 0;
        let mut diagnostic_text = String::new();
        loop {
            let event = rx
                .recv_timeout(Duration::from_secs(180))
                .expect("bounded model terminal event");
            assert_eq!(event.root_id, request.root_id);
            assert_eq!(event.run_id, request.run_id);
            assert!(event.seq > last_seq);
            last_seq = event.seq;
            match event.event {
                RunEvent::Completed => {
                    terminal += 1;
                    break;
                }
                RunEvent::Error { error } => panic!("sanitized runtime failure: {}", error),
                RunEvent::Cancelled => panic!("unexpected cancellation"),
                RunEvent::Proposal { .. } => proposals += 1,
                RunEvent::TextDelta { text } => diagnostic_text.push_str(&text),
                _ => {}
            }
        }
        assert_eq!(terminal, 1);
        assert_eq!(
            proposals,
            1,
            "reads={} staged={}",
            host.reads.lock().unwrap(),
            host.staged.lock().unwrap().len()
        );
        assert!(*host.reads.lock().unwrap() > 0);
        let staged = host.staged.lock().unwrap();
        assert!(!staged.is_empty());
        if correction {
            assert!(matches!(staged[0], Candidate::Correction { .. }));
        } else {
            assert!(staged.iter().any(|c|matches!(c,Candidate::Connection{connection} if connection.relation=="leads" && !connection.sources.is_empty())));
        }
        assert_eq!(std::fs::read_to_string(&source_path).unwrap(), bytes);
        println!(
            "sanitized evidence: mode={} terminal=completed scoped_reads={} candidates={} proposal_events={} source_unchanged=true",
            if correction {
                "correction"
            } else {
                "discovery"
            },
            host.reads.lock().unwrap(),
            staged.len(),
            proposals
        );
        // Completion event precedes final child bookkeeping by only this bounded wait.
        for _ in 0..100 {
            if !runtime.cancel(&request.root_id, &request.run_id) {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
