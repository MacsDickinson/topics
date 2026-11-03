//! Service acceptance against the committed topic-workspace features.
//! Four presentation/protocol scenarios are explicitly assigned to desktop/runtime evidence.
use cucumber::{World, given, then, when};
use std::{fs, path::PathBuf};
use topics_core::*;
use topics_store::WorkspaceService;

#[derive(World)]
#[world(init=Self::new)]
struct Workspace {
    temp: tempfile::TempDir,
    service: WorkspaceService,
    root: PathBuf,
    root_id: Id,
    scope: Option<ScopePreview>,
    proposal: Option<Proposal>,
    other: Option<Proposal>,
    candidates: Vec<Candidate>,
    topic: Option<Topic>,
    claim: Option<Claim>,
    source: Option<SourceRef>,
    run_id: Option<Id>,
    error: Option<ApiError>,
    operations: Vec<Operation>,
    original: Vec<u8>,
    external: Vec<u8>,
    record: Option<PathBuf>,
    received: Vec<ScopedSource>,
    old_root_id: Option<Id>,
    old_proposal: Option<Proposal>,
    same_name: Vec<Id>,
}
impl std::fmt::Debug for Workspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Synthetic workspace world")
    }
}
fn id() -> Id {
    uuid::Uuid::new_v4().to_string()
}
fn topic(name: &str, kind: TopicKind) -> Topic {
    Topic {
        id: id(),
        topic_kind: kind,
        name: name.into(),
        aliases: vec![],
        sensitivity: Sensitivity::CloudAllowed,
    }
}
fn claim(t: &Topic, value: &str, source: &SourceRef) -> Claim {
    Claim {
        id: id(),
        revision_id: id(),
        subject_id: t.id.clone(),
        predicate: "status".into(),
        value: value.into(),
        uncertainty: Some("Transcript assertion; requires verification".into()),
        sources: vec![Evidence::Passage {
            source: source.clone(),
        }],
        supersedes: vec![],
        recorded_at: "2026-11-03T00:00:00Z".into(),
        valid_from: None,
        valid_to: None,
    }
}
fn connection(a: &Topic, b: &Topic, source: &SourceRef) -> Connection {
    Connection {
        id: id(),
        from: Endpoint::Topic { id: a.id.clone() },
        relation: "leads".into(),
        to: Endpoint::Topic { id: b.id.clone() },
        uncertainty: Some("Unverified transcript".into()),
        sources: vec![source.clone()],
        valid_from: None,
        valid_to: None,
    }
}
impl Workspace {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let app = temp.path().join("app-data");
        assert!(app.is_absolute());
        let service = WorkspaceService::open(&app).unwrap();
        let root = temp.path().join("notes");
        fs::create_dir(&root).unwrap();
        Self {
            temp,
            service,
            root,
            root_id: String::new(),
            scope: None,
            proposal: None,
            other: None,
            candidates: vec![],
            topic: None,
            claim: None,
            source: None,
            run_id: None,
            error: None,
            operations: vec![],
            original: vec![],
            external: vec![],
            record: None,
            received: vec![],
            old_root_id: None,
            old_proposal: None,
            same_name: vec![],
        }
    }
    fn request(&self) -> RootRequest {
        RootRequest {
            root_id: self.root_id.clone(),
        }
    }
    fn write(&self, path: &str, text: &str) {
        let p = self.root.join(path);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    }
    fn select(&mut self) {
        self.root_id = self
            .service
            .root_set(RootSetRequest {
                path: self.root.to_str().unwrap().into(),
                create: false,
            })
            .unwrap()
            .root
            .root_id;
    }
    fn preview(&mut self, files: Vec<&str>) {
        self.scope = Some(
            self.service
                .scope_preview(ScopePreviewRequest {
                    root_id: self.root_id.clone(),
                    spec: ScopeSpec {
                        files: files.into_iter().map(str::to_string).collect(),
                        folders: vec![],
                        date_from: None,
                        date_to: None,
                    },
                })
                .unwrap(),
        );
    }
    fn citation(&self, path: &str) -> SourceRef {
        let sources = self.service.sources_list(self.request()).unwrap();
        let source = sources
            .sources
            .into_iter()
            .find(|s| s.path == path)
            .unwrap();
        self.service
            .source_get(SourceGetRequest {
                root_id: self.root_id.clone(),
                source_id: source.source_id,
            })
            .unwrap()
            .citation
    }
    fn stage(&self, candidates: Vec<Candidate>) -> Proposal {
        self.service
            .manual_propose(ManualProposeRequest {
                root_id: self.root_id.clone(),
                candidates,
            })
            .unwrap()
            .proposal
    }
    fn decide(&self, p: &Proposal, decision: Decision) -> WorkspaceResult<ProposalDecideResult> {
        self.service.proposal_decide(ProposalDecideRequest {
            root_id: self.root_id.clone(),
            proposal_id: p.id.clone(),
            decision,
            expected_fingerprint: p.fingerprint.clone(),
        })
    }
    fn accept(&mut self, candidates: Vec<Candidate>) -> Proposal {
        let p = self.stage(candidates.clone());
        let result = self.decide(&p, Decision::Approve { candidates }).unwrap();
        self.operations.push(result.operation.unwrap());
        result.proposal
    }
    fn view(&self) -> TopicView {
        self.service
            .topic_get(TopicGetRequest {
                root_id: self.root_id.clone(),
                topic_id: self.topic.as_ref().unwrap().id.clone(),
            })
            .unwrap()
    }
    fn initial_claim(&mut self, value: &str) {
        self.write(
            "Meetings/one.md",
            &format!("# Synthetic meeting\n{value}\n"),
        );
        self.original = fs::read(self.root.join("Meetings/one.md")).unwrap();
        let source = self.citation("Meetings/one.md");
        let t = topic("Ledger", TopicKind::Project);
        let c = claim(&t, value, &source);
        self.candidates = vec![
            Candidate::Topic { topic: t.clone() },
            Candidate::Claim { claim: c.clone() },
        ];
        self.topic = Some(t);
        self.claim = Some(c);
        self.source = Some(source);
    }
    fn accepted_initial(&mut self, value: &str) {
        self.initial_claim(value);
        self.accept(self.candidates.clone());
    }
    fn correction(&mut self, value: &str) -> Proposal {
        let old = self.claim.as_ref().unwrap().clone();
        let t = self.topic.as_ref().unwrap().clone();
        self.preview(vec!["Meetings/one.md"]);
        let scope_id = self.scope.as_ref().unwrap().scope_id.clone();
        let chat = self
            .service
            .chat_start(ChatStartRequest {
                root_id: self.root_id.clone(),
                topic_id: t.id.clone(),
                scope_id: scope_id.clone(),
                confirm_unknown: true,
                message: format!("Correct {} to {value}", old.value),
            })
            .unwrap();
        let mut c = old.clone();
        c.revision_id = id();
        c.value = value.into();
        c.supersedes = vec![old.revision_id.clone()];
        c.sources = vec![Evidence::UserStatement {
            statement_id: chat.message_id.clone(),
        }];
        let rule = Rule {
            id: id(),
            topic_id: t.id.clone(),
            predicate: c.predicate.clone(),
            match_value: old.value,
            replacement: value.into(),
            scope: RuleScope {
                source_paths: vec!["Meetings/one.md".into()],
                context: None,
            },
            source_revision_id: c.revision_id.clone(),
        };
        self.service
            .stage_proposal(StageProposalRequest {
                root_id: self.root_id.clone(),
                scope_id: Some(scope_id),
                origin: Origin::Chat {
                    topic_id: t.id,
                    message_id: chat.message_id,
                },
                candidates: vec![Candidate::Correction { claim: c, rule }],
            })
            .unwrap()
    }
    fn restart(&mut self) {
        self.service = WorkspaceService::open(self.temp.path().join("app-data")).unwrap();
    }
    fn assert_source_unchanged(&self) {
        assert_eq!(
            fs::read(self.root.join("Meetings/one.md")).unwrap(),
            self.original
        );
    }
}
#[given("a synthetic notes root with absolute isolated application data")]
fn background(w: &mut Workspace) {
    w.select();
    assert!(w.temp.path().join("app-data").is_absolute());
}

#[given(expr = "dated notes {string} on {string} and {string} on {string}")]
fn dated(w: &mut Workspace, a: String, da: String, b: String, db: String) {
    w.write(&a, &format!("---\ndate: {da}\n---\n# One\n"));
    w.write(&b, &format!("---\ndate: {db}\n---\n# Two\n"));
}
#[given(expr = "an undated note {string}")]
fn undated(w: &mut Workspace, path: String) {
    w.write(&path, "# Undated\n");
}
#[when(expr = "I preview folder {string} for dates {string} through {string}")]
fn date_preview(w: &mut Workspace, folder: String, from: String, to: String) {
    w.scope = Some(
        w.service
            .scope_preview(ScopePreviewRequest {
                root_id: w.root_id.clone(),
                spec: ScopeSpec {
                    files: vec![],
                    folders: vec![folder],
                    date_from: Some(from),
                    date_to: Some(to),
                },
            })
            .unwrap(),
    );
}
#[then(expr = "the preview includes {string} and excludes {string} and {string}")]
fn date_result(w: &mut Workspace, yes: String, no: String, undated: String) {
    let p = w.scope.as_ref().unwrap();
    assert_eq!(
        p.sources
            .iter()
            .map(|s| s.path.as_str())
            .collect::<Vec<_>>(),
        vec![yes.as_str()]
    );
    assert!(
        p.excluded
            .iter()
            .any(|e| e.path == no && e.reason.contains("date"))
    );
    assert!(
        p.excluded
            .iter()
            .any(|e| e.path == undated && e.reason.contains("Undated"))
    );
}
#[then("no note bytes have been sent to a model")]
#[then("no note bytes are sent to a model")]
fn no_transmission(w: &mut Workspace) {
    assert!(w.received.is_empty());
    assert!(w.run_id.is_none());
}

#[given(expr = "{string} is unknown and {string} is local-only")]
fn labels(w: &mut Workspace, unknown: String, private: String) {
    w.write(&unknown, "# Public unknown\nAlex leads Ledger\n");
    w.write(&private, "# Private\nSYNTHETIC PRIVATE MARKER\n");
    let source = w.citation(&private);
    w.service
        .sensitivity_set(SensitivitySetRequest {
            root_id: w.root_id.clone(),
            target: Endpoint::Source {
                source_id: source.source_id.clone(),
                locator: Locator {
                    start_byte: 0,
                    end_byte: 0,
                    heading: None,
                },
            },
            sensitivity: Sensitivity::LocalOnly,
        })
        .unwrap();
    let t = topic("Private derived topic", TopicKind::Person);
    let c = claim(&t, "SYNTHETIC PRIVATE FACT", &source);
    w.accept(vec![
        Candidate::Topic { topic: t.clone() },
        Candidate::Claim { claim: c },
    ]);
    w.topic = Some(t);
}
#[when("I preview both files and confirm unknown sources for this run")]
fn consent(w: &mut Workspace) {
    w.preview(vec!["Meetings/one.md", "Meetings/private.md"]);
    let p = w.scope.as_ref().unwrap();
    w.run_id = Some(
        w.service
            .analysis_start(AnalysisStartRequest {
                root_id: w.root_id.clone(),
                scope_id: p.scope_id.clone(),
                confirm_unknown: true,
            })
            .unwrap()
            .run_id,
    );
}
#[then(expr = "the model can receive {string}")]
fn scoped_receive(w: &mut Workspace, path: String) {
    let result = w
        .service
        .scope_read(ScopeReadRequest {
            root_id: w.root_id.clone(),
            scope_id: w.scope.as_ref().unwrap().scope_id.clone(),
            run_id: w.run_id.clone().unwrap(),
        })
        .unwrap();
    assert_eq!(result.sources.len(), 1);
    assert_eq!(result.sources[0].summary.path, path);
    w.received = result.sources;
}
#[then(expr = "all model tools deny {string} and its derived topic context")]
fn private_denied(w: &mut Workspace, path: String) {
    assert!(
        w.scope
            .as_ref()
            .unwrap()
            .excluded
            .iter()
            .any(|e| e.path == path)
    );
    let e = w
        .service
        .source_context(SourceContextRequest {
            root_id: w.root_id.clone(),
            scope_id: w.scope.as_ref().unwrap().scope_id.clone(),
            run_id: w.run_id.clone().unwrap(),
            topic_id: w.topic.as_ref().unwrap().id.clone(),
        })
        .unwrap_err();
    assert_eq!(e.code, "forbidden");
    let out = serde_json::to_string(&w.received).unwrap();
    assert!(!out.contains("PRIVATE"));
}

#[given(expr = "I preview {string}")]
fn file_preview(w: &mut Workspace, path: String) {
    w.write(&path, "# Before\nSelected source\n");
    w.preview(vec![&path]);
}
#[when(expr = "another program changes {string} before analysis starts")]
fn changed_before_start(w: &mut Workspace, path: String) {
    w.write(&path, "# Changed outside Topics\n");
    let p = w.scope.as_ref().unwrap();
    w.error = Some(
        w.service
            .analysis_start(AnalysisStartRequest {
                root_id: w.root_id.clone(),
                scope_id: p.scope_id.clone(),
                confirm_unknown: true,
            })
            .unwrap_err(),
    );
}
#[then(expr = "analysis fails with {string}")]
#[then(expr = "approval fails with {string}")]
fn code(w: &mut Workspace, expected: String) {
    assert_eq!(w.error.as_ref().unwrap().code, expected);
}

#[given(expr = "a selected synthetic meeting says {string}")]
fn selected(w: &mut Workspace, text: String) {
    w.write("Meetings/one.md", &format!("# Meeting\n{text}\n"));
    w.original = fs::read(w.root.join("Meetings/one.md")).unwrap();
    w.preview(vec!["Meetings/one.md"]);
    w.source = Some(w.citation("Meetings/one.md"));
}
#[given(
    expr = "the scripted model proposes person {string}, project {string} and a cited {string} connection"
)]
fn discovery_model(w: &mut Workspace, person: String, project: String, relation: String) {
    let a = topic(&person, TopicKind::Person);
    let b = topic(&project, TopicKind::Project);
    let mut c = connection(&a, &b, w.source.as_ref().unwrap());
    c.relation = relation;
    w.candidates = vec![
        Candidate::Topic { topic: a },
        Candidate::Topic { topic: b },
        Candidate::Connection { connection: c },
    ];
}
#[when("analysis completes successfully")]
fn model_completed(w: &mut Workspace) {
    let scope_id = w.scope.as_ref().unwrap().scope_id.clone();
    let run_id = w
        .service
        .analysis_start(AnalysisStartRequest {
            root_id: w.root_id.clone(),
            scope_id: scope_id.clone(),
            confirm_unknown: true,
        })
        .unwrap()
        .run_id;
    w.proposal = Some(
        w.service
            .stage_proposal(StageProposalRequest {
                root_id: w.root_id.clone(),
                scope_id: Some(scope_id),
                origin: Origin::Analysis { run_id },
                candidates: w.candidates.clone(),
            })
            .unwrap(),
    );
}
#[then("the proposals show the passage and its source locator")]
fn cited_proposal(w: &mut Workspace) {
    let p = w.proposal.as_ref().unwrap();
    assert_eq!(p.status, ProposalStatus::Pending);
    let Candidate::Connection { connection } = p.candidates.last().unwrap() else {
        panic!("expected connection")
    };
    let s = &connection.sources[0];
    assert_eq!(s, w.source.as_ref().unwrap());
    assert!(s.locator.end_byte > s.locator.start_byte);
    assert!(s.excerpt.contains("Alex leads Ledger"));
}
#[then("no topic, claim or connection is accepted")]
#[then("neither record is accepted")]
fn none_accepted(w: &mut Workspace) {
    assert!(
        w.service
            .topics_list(w.request())
            .unwrap()
            .topics
            .is_empty()
    );
    assert!(
        w.service
            .history_list(w.request())
            .unwrap()
            .operations
            .iter()
            .all(|o| o.description != "Approve reviewed proposal")
    );
}
#[then("the meeting bytes are unchanged")]
#[then("the source meeting bytes are unchanged")]
#[then("the transcript bytes are unchanged")]
fn source_unchanged(w: &mut Workspace) {
    w.assert_source_unchanged();
}

#[given(expr = "two accepted people named {string} have different IDs")]
fn two_people(w: &mut Workspace, name: String) {
    let a = topic(&name, TopicKind::Person);
    let b = topic(&name, TopicKind::Person);
    w.same_name = vec![a.id.clone(), b.id.clone()];
    w.accept(vec![
        Candidate::Topic { topic: a },
        Candidate::Topic { topic: b },
    ]);
}
#[given(expr = "a selected meeting mentions {string} without distinguishing them")]
fn ambiguous_meeting(w: &mut Workspace, name: String) {
    selected(w, format!("{name} leads Ledger"));
}
#[when("the scripted model proposes an identity for that mention")]
fn ambiguous_proposal(w: &mut Workspace) {
    let a = w.service.topics_list(w.request()).unwrap().topics[0].clone();
    let b = topic("Ledger", TopicKind::Project);
    w.candidates = vec![
        Candidate::Topic { topic: b.clone() },
        Candidate::Connection {
            connection: connection(&a, &b, w.source.as_ref().unwrap()),
        },
    ];
    model_completed(w);
}
#[then("the review shows both candidates as unresolved")]
fn unresolved(w: &mut Workspace) {
    let p = w.proposal.as_ref().unwrap();
    assert!(p.candidates.iter().any(|c|matches!(c,Candidate::UnresolvedIdentity{candidates,..} if candidates.len()==2&&w.same_name.iter().all(|id|candidates.contains(id)))));
}
#[then("no connection to either person is accepted")]
fn no_identity_connection(w: &mut Workspace) {
    for id in &w.same_name {
        assert!(
            w.service
                .topic_get(TopicGetRequest {
                    root_id: w.root_id.clone(),
                    topic_id: id.clone()
                })
                .unwrap()
                .connections
                .is_empty()
        );
    }
}

#[given(expr = "a cited claim from {string} was accepted")]
fn accepted_cited(w: &mut Workspace, path: String) {
    assert_eq!(path, "Meetings/one.md");
    w.accepted_initial("Ledger launch is Friday");
}
#[when("the same unchanged meeting produces the same claim in a later analysis")]
fn repeat_analysis(w: &mut Workspace) {
    w.preview(vec!["Meetings/one.md"]);
    let mut c = w.claim.as_ref().unwrap().clone();
    c.id = id();
    c.revision_id = id();
    c.recorded_at = "2026-11-03T00:01:00Z".into();
    let t = w.topic.as_ref().unwrap().clone();
    let mut replacement = t.clone();
    replacement.id = id();
    c.subject_id = replacement.id.clone();
    w.candidates = vec![
        Candidate::Topic { topic: replacement },
        Candidate::Claim { claim: c },
    ];
    model_completed(w);
}
#[then("the existing decision is shown as a duplicate")]
fn duplicate(w: &mut Workspace) {
    assert_eq!(
        w.proposal.as_ref().unwrap().status,
        ProposalStatus::Accepted
    );
}
#[then("there is still one accepted claim")]
fn one_claim(w: &mut Workspace) {
    assert_eq!(w.view().current_claims.len(), 1);
    assert_eq!(w.view().history.len(), 1);
}

#[given(expr = "a pending claim says {string} with a meeting citation")]
fn pending_cited(w: &mut Workspace, value: String) {
    w.initial_claim(&value);
    w.proposal = Some(w.stage(w.candidates.clone()));
}
#[when(expr = "I edit the claim to {string} and approve it")]
fn edit_approve(w: &mut Workspace, value: String) {
    let p = w.proposal.as_ref().unwrap().clone();
    let mut edited = p.candidates.clone();
    for c in &mut edited {
        if let Candidate::Claim { claim } = c {
            claim.value = value.clone();
        }
    }
    let result = w
        .decide(&p, Decision::Approve { candidates: edited })
        .unwrap();
    w.operations.push(result.operation.unwrap());
    w.proposal = Some(result.proposal);
}
#[then("the topic shows the accepted Monday claim with its citation and uncertainty")]
fn accepted_monday(w: &mut Workspace) {
    let v = w.view();
    assert_eq!(v.current_claims.len(), 1);
    assert_eq!(v.current_claims[0].value, "Ledger launch is Monday");
    assert_eq!(
        v.current_claims[0].sources[0],
        Evidence::Passage {
            source: w.source.as_ref().unwrap().clone()
        }
    );
    assert!(v.current_claims[0].uncertainty.is_some());
}
#[given(expr = "a pending claim says {string}")]
fn pending_plain(w: &mut Workspace, value: String) {
    pending_cited(w, value);
}
#[when("I reject the proposal")]
fn reject(w: &mut Workspace) {
    w.proposal = Some(
        w.decide(w.proposal.as_ref().unwrap(), Decision::Reject)
            .unwrap()
            .proposal,
    );
}
#[then(expr = "no accepted claim says {string}")]
fn rejected_claim(w: &mut Workspace, _value: String) {
    assert!(
        w.service
            .topics_list(w.request())
            .unwrap()
            .topics
            .is_empty()
    );
}
#[then("the proposal is marked rejected")]
fn marked_rejected(w: &mut Workspace) {
    assert_eq!(
        w.service.proposals_list(w.request()).unwrap().proposals[0].status,
        ProposalStatus::Rejected
    );
}
#[given("a proposal contains a new person and a connection to that person")]
fn dependent(w: &mut Workspace) {
    selected(w, "Alex leads Ledger".into());
    discovery_model(w, "Alex".into(), "Ledger".into(), "leads".into());
    w.proposal = Some(w.stage(w.candidates.clone()));
}
#[when("I approve only the connection")]
fn omit_topic(w: &mut Workspace) {
    let p = w.proposal.as_ref().unwrap();
    let c = p
        .candidates
        .iter()
        .find(|c| matches!(c, Candidate::Connection { .. }))
        .unwrap()
        .clone();
    w.error = Some(
        w.decide(
            p,
            Decision::Approve {
                candidates: vec![c],
            },
        )
        .unwrap_err(),
    );
}
#[then("approval fails with a dependency error")]
fn dependency_error(w: &mut Workspace) {
    let e = w.error.as_ref().unwrap();
    assert_eq!(e.code, "invalid_input");
    assert!(e.message.contains("dependency"));
}

#[given(expr = "an accepted claim says person {string} uses {string} with a transcript citation")]
fn accepted_pronouns(w: &mut Workspace, name: String, value: String) {
    w.initial_claim(&value);
    let t = w.topic.as_mut().unwrap();
    t.name = name;
    t.topic_kind = TopicKind::Person;
    w.claim.as_mut().unwrap().predicate = "pronouns".into();
    w.candidates = vec![
        Candidate::Topic {
            topic: w.topic.as_ref().unwrap().clone(),
        },
        Candidate::Claim {
            claim: w.claim.as_ref().unwrap().clone(),
        },
    ];
    w.accept(w.candidates.clone());
}
#[given(expr = "topic chat proposes {string} and a rule scoped to Alex's pronouns")]
fn chat_correction(w: &mut Workspace, value: String) {
    w.proposal = Some(w.correction(&value));
}
#[when("I approve the correction")]
fn approve_correction(w: &mut Workspace) {
    let p = w.proposal.as_ref().unwrap();
    let result = w
        .decide(
            p,
            Decision::Approve {
                candidates: p.candidates.clone(),
            },
        )
        .unwrap();
    w.operations.push(result.operation.unwrap());
}
#[then("Alex's current claim says \"they/them\" and cites my persisted chat statement")]
fn current_correction(w: &mut Workspace) {
    let v = w.view();
    assert_eq!(v.current_claims.len(), 1);
    let c = &v.current_claims[0];
    assert_eq!(c.value, "they/them");
    let Evidence::UserStatement { statement_id } = &c.sources[0] else {
        panic!("must cite user")
    };
    let text =
        fs::read_to_string(w.root.join(format!("topics/statements/{statement_id}.md"))).unwrap();
    assert!(text.contains("Correct he/him to they/them"));
}
#[then("the prior claim is superseded but visible in history")]
fn prior_revision(w: &mut Workspace) {
    let v = w.view();
    assert_eq!(v.history.len(), 2);
    let old = w.claim.as_ref().unwrap();
    assert!(
        v.history
            .iter()
            .any(|c| c.revision_id == old.revision_id && c.value == "he/him")
    );
    assert_eq!(
        v.current_claims[0].supersedes,
        vec![old.revision_id.clone()]
    );
}
#[when("the same transcript is analyzed again and the app restarts")]
fn corrected_reanalysis(w: &mut Workspace) {
    w.preview(vec!["Meetings/one.md"]);
    let old = w.claim.as_ref().unwrap().clone();
    let mut repeated = old;
    repeated.id = id();
    repeated.revision_id = id();
    w.candidates = vec![Candidate::Claim { claim: repeated }];
    let scope_id = w.scope.as_ref().unwrap().scope_id.clone();
    let run = w
        .service
        .analysis_start(AnalysisStartRequest {
            root_id: w.root_id.clone(),
            scope_id: scope_id.clone(),
            confirm_unknown: true,
        })
        .unwrap();
    let no_change = w
        .service
        .stage_proposal(StageProposalRequest {
            root_id: w.root_id.clone(),
            scope_id: Some(scope_id),
            origin: Origin::Analysis { run_id: run.run_id },
            candidates: w.candidates.clone(),
        })
        .unwrap();
    assert_eq!(no_change.status, ProposalStatus::Accepted);
    assert!(no_change.candidates.is_empty());
    w.restart();
}
#[then("\"he/him\" is not restored as current knowledge")]
fn not_restored(w: &mut Workspace) {
    assert!(w.view().current_claims.iter().all(|c| c.value != "he/him"));
    assert_eq!(w.view().current_claims[0].value, "they/them");
}
#[then("the rule does not change another person's pronouns")]
fn scoped_rule(w: &mut Workspace) {
    let t = topic("Other person", TopicKind::Person);
    let mut c = claim(&t, "he/him", w.source.as_ref().unwrap());
    c.predicate = "pronouns".into();
    w.accept(vec![
        Candidate::Topic { topic: t.clone() },
        Candidate::Claim { claim: c },
    ]);
    let v = w
        .service
        .topic_get(TopicGetRequest {
            root_id: w.root_id.clone(),
            topic_id: t.id,
        })
        .unwrap();
    assert_eq!(v.current_claims[0].value, "he/him");
}

#[given(expr = "a pending claim cites a passage in {string}")]
fn pending_passage(w: &mut Workspace, path: String) {
    assert_eq!(path, "Meetings/one.md");
    pending_cited(w, "Ledger launch is Friday".into());
}
#[when("another program changes that passage before I approve it")]
fn stale_passage(w: &mut Workspace) {
    w.write("Meetings/one.md", "# Changed meeting\nEdited assertion\n");
    let p = w.proposal.as_ref().unwrap();
    w.error = Some(
        w.decide(
            p,
            Decision::Approve {
                candidates: p.candidates.clone(),
            },
        )
        .unwrap_err(),
    );
}
#[then("the proposed claim is not accepted")]
fn not_accepted(w: &mut Workspace) {
    assert!(
        w.service
            .topics_list(w.request())
            .unwrap()
            .topics
            .is_empty()
    );
}
#[given("two corrections supersede the same accepted claim revision")]
fn competing(w: &mut Workspace) {
    accepted_pronouns(w, "Alex".into(), "he/him".into());
    w.proposal = Some(w.correction("they/them"));
    w.other = Some(w.correction("she/her"));
}
#[when("the first correction is approved and the second is approved from its stale view")]
fn approve_competing(w: &mut Workspace) {
    approve_correction(w);
    let p = w.other.as_ref().unwrap();
    w.error = Some(
        w.decide(
            p,
            Decision::Approve {
                candidates: p.candidates.clone(),
            },
        )
        .unwrap_err(),
    );
}
#[then("the second approval reports a conflict")]
fn competing_conflict(w: &mut Workspace) {
    assert_eq!(w.error.as_ref().unwrap().code, "conflict");
}
#[then("neither correction silently overwrites the other")]
fn no_overwrite(w: &mut Workspace) {
    assert_eq!(w.view().current_claims[0].value, "they/them");
    assert_eq!(w.view().history.len(), 2);
    assert_eq!(
        w.service
            .proposals_list(w.request())
            .unwrap()
            .proposals
            .iter()
            .filter(|p| p.status == ProposalStatus::Conflict)
            .count(),
        1
    );
}

#[given("an accepted topic, claim, connection and correction rule")]
fn accepted_graph(w: &mut Workspace) {
    accepted_pronouns(w, "Alex".into(), "he/him".into());
    let t = topic("Ledger", TopicKind::Project);
    let c = connection(w.topic.as_ref().unwrap(), &t, w.source.as_ref().unwrap());
    w.accept(vec![
        Candidate::Topic { topic: t },
        Candidate::Connection { connection: c },
    ]);
    w.proposal = Some(w.correction("they/them"));
    approve_correction(w);
}
#[when("the derived database is deleted and the app restarts")]
fn rebuild(w: &mut Workspace) {
    let db = w.temp.path().join("app-data/index.sqlite3");
    assert!(db.exists());
    fs::remove_file(db).unwrap();
    w.restart();
}
#[then("the same stable IDs, current claim, history and citation are recovered")]
fn recovered_graph(w: &mut Workspace) {
    let v = w.view();
    assert_eq!(v.topic.id, w.topic.as_ref().unwrap().id);
    assert_eq!(v.history.len(), 2);
    assert_eq!(v.current_claims[0].value, "they/them");
    assert_eq!(v.connections.len(), 1);
    assert!(v.history.iter().any(|c| c.sources
        == vec![Evidence::Passage {
            source: w.source.as_ref().unwrap().clone()
        }]));
    let rules = fs::read_dir(w.root.join("topics/rules")).unwrap().count();
    assert_eq!(rules, 1);
}
#[given("a pending update with an expected record hash")]
fn update_hash(w: &mut Workspace) {
    w.accepted_initial("Ledger launch is Friday");
    w.proposal = Some(w.correction("Monday"));
    w.record = Some(
        w.root
            .join(format!("topics/topics/{}.md", w.topic.as_ref().unwrap().id)),
    );
}
#[when("another program changes that record before approval")]
fn external_update(w: &mut Workspace) {
    w.external = b"# External record version\nMust be preserved\n".to_vec();
    fs::write(w.record.as_ref().unwrap(), &w.external).unwrap();
    let p = w.proposal.as_ref().unwrap();
    w.error = Some(
        w.decide(
            p,
            Decision::Approve {
                candidates: p.candidates.clone(),
            },
        )
        .unwrap_err(),
    );
}
#[then("approval reports a conflict and preserves the external bytes and proposed bytes")]
fn conflict_bytes(w: &mut Workspace) {
    assert_eq!(w.error.as_ref().unwrap().code, "conflict");
    assert_eq!(fs::read(w.record.as_ref().unwrap()).unwrap(), w.external);
    let partition = w.temp.path().join("app-data/workspaces");
    let p = fs::read_dir(partition)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path()
        .join("conflicts")
        .join(format!("{}.json", w.proposal.as_ref().unwrap().id));
    let data: serde_json::Value = serde_json::from_slice(&fs::read(p).unwrap()).unwrap();
    assert!(!data["old_bytes"].as_object().unwrap().is_empty());
    assert!(!data["staged_records"].as_object().unwrap().is_empty());
    assert!(!data["current_bytes"].as_object().unwrap().is_empty());
}
#[then("no dependent record becomes visible")]
fn no_dependent(w: &mut Workspace) {
    assert_eq!(
        fs::read_dir(w.root.join("topics/claims")).unwrap().count(),
        1
    );
}

fn interrupted(w: &mut Workspace) {
    w.initial_claim("Ledger launch is Friday");
    let p = w.stage(w.candidates.clone());
    w.service.interrupt_next_operation_after(1);
    let e = w
        .decide(
            &p,
            Decision::Approve {
                candidates: p.candidates.clone(),
            },
        )
        .unwrap_err();
    assert_eq!(e.code, "internal");
    w.proposal = Some(p);
}
#[given("approval of a topic and dependent claim is interrupted after the first file")]
#[given("approval is interrupted after its first file")]
fn interrupted_setup(w: &mut Workspace) {
    interrupted(w);
}
#[given("another program changes a remaining target")]
fn remaining_changed(w: &mut Workspace) {
    let p = w.root.join(format!(
        "topics/claims/{}.md",
        w.claim.as_ref().unwrap().revision_id
    ));
    w.external = b"# Outside pending claim\n".to_vec();
    fs::write(&p, &w.external).unwrap();
    w.record = Some(p);
}
#[when("the app restarts")]
fn restart(w: &mut Workspace) {
    w.restart();
    let _ = w.service.topics_list(w.request()).unwrap();
}
#[then("recovery completes the operation if remaining expected hashes match")]
fn recovery_complete(w: &mut Workspace) {
    assert!(
        w.service
            .history_list(w.request())
            .unwrap()
            .operations
            .iter()
            .any(|o| o.description == "Approve reviewed proposal" && o.status == "committed")
    );
}
#[then("the topic and claim become visible together")]
fn together(w: &mut Workspace) {
    assert_eq!(w.service.topics_list(w.request()).unwrap().topics.len(), 1);
    assert_eq!(w.view().current_claims.len(), 1);
    let p = w.proposal.as_ref().unwrap();
    assert_eq!(
        w.decide(
            p,
            Decision::Approve {
                candidates: p.candidates.clone()
            }
        )
        .unwrap()
        .proposal
        .status,
        ProposalStatus::Accepted
    );
}
#[then("recovery reports a conflict and preserves old, staged and current versions")]
fn recovery_conflict(w: &mut Workspace) {
    assert!(
        w.service
            .history_list(w.request())
            .unwrap()
            .operations
            .iter()
            .any(|o| o.status == "conflict")
    );
    assert_eq!(fs::read(w.record.as_ref().unwrap()).unwrap(), w.external);
    let base = fs::read_dir(w.temp.path().join("app-data/workspaces"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path()
        .join("journals");
    let mut seen = false;
    for e in fs::read_dir(base).unwrap() {
        let p = e.unwrap().path().join("conflict-1.json");
        if p.exists() {
            let v: serde_json::Value = serde_json::from_slice(&fs::read(p).unwrap()).unwrap();
            assert!(v["old_bytes"].is_null());
            assert!(v["staged_bytes"].as_array().is_some());
            assert!(v["current_bytes"].as_array().is_some());
            seen = true;
        }
    }
    assert!(seen);
}
#[then("incomplete records are not visible as accepted")]
fn incomplete_hidden(w: &mut Workspace) {
    assert!(
        w.service
            .topics_list(w.request())
            .unwrap()
            .topics
            .is_empty()
    );
}

#[given("a topic was created and a later claim depends on it")]
fn undo_setup(w: &mut Workspace) {
    w.initial_claim("Ledger launch is Friday");
    w.accept(vec![Candidate::Topic {
        topic: w.topic.as_ref().unwrap().clone(),
    }]);
    w.accept(vec![Candidate::Claim {
        claim: w.claim.as_ref().unwrap().clone(),
    }]);
}
#[when("I undo the topic creation")]
fn undo_topic(w: &mut Workspace) {
    w.error = Some(
        w.service
            .undo(UndoRequest {
                root_id: w.root_id.clone(),
                operation_id: w.operations[0].id.clone(),
            })
            .unwrap_err(),
    );
}
#[then("undo is refused with the dependent claim identified")]
fn undo_refused(w: &mut Workspace) {
    let e = w.error.as_ref().unwrap();
    assert_eq!(e.code, "conflict");
    assert!(
        e.details.as_ref().unwrap()["dependent_operations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str() == Some(w.operations[1].id.as_str()))
    );
}
#[when("I undo the claim and then undo the topic creation")]
fn undo_order(w: &mut Workspace) {
    for index in [1, 0] {
        w.service
            .undo(UndoRequest {
                root_id: w.root_id.clone(),
                operation_id: w.operations[index].id.clone(),
            })
            .unwrap();
    }
}
#[then("both are absent from current knowledge and remain visible in history")]
fn undo_history(w: &mut Workspace) {
    assert!(
        w.service
            .topics_list(w.request())
            .unwrap()
            .topics
            .is_empty()
    );
    let h = w.service.history_list(w.request()).unwrap();
    assert!(
        w.operations
            .iter()
            .all(|o| h.operations.iter().any(|x| x.id == o.id))
    );
    assert_eq!(
        h.operations
            .iter()
            .filter(|o| o.description == "Undo creations")
            .count(),
        2
    );
    assert!(
        w.root
            .join(format!(
                "topics/claims/{}.md",
                w.claim.as_ref().unwrap().revision_id
            ))
            .exists()
    );
}
#[given(expr = "root {string} has an accepted topic and pending proposal")]
fn root_a(w: &mut Workspace, name: String) {
    assert_eq!(name, "A");
    w.accept(vec![Candidate::Topic {
        topic: topic("Root A accepted", TopicKind::Project),
    }]);
    w.proposal = Some(w.stage(vec![Candidate::Topic {
        topic: topic("Root A pending", TopicKind::Thread),
    }]));
    w.old_root_id = Some(w.root_id.clone());
    w.old_proposal = w.proposal.clone();
}
#[when(expr = "I switch to root {string}")]
fn root_b(w: &mut Workspace, name: String) {
    w.root = w.temp.path().join(name);
    fs::create_dir(&w.root).unwrap();
    w.select();
}
#[then("neither root A's topic nor its proposal appears")]
fn isolated_roots(w: &mut Workspace) {
    assert!(
        w.service
            .topics_list(w.request())
            .unwrap()
            .topics
            .is_empty()
    );
    assert!(
        w.service
            .proposals_list(w.request())
            .unwrap()
            .proposals
            .is_empty()
    );
}
#[then("an old root A approval request is rejected")]
fn old_approval(w: &mut Workspace) {
    let p = w.old_proposal.as_ref().unwrap();
    let e = w
        .service
        .proposal_decide(ProposalDecideRequest {
            root_id: w.old_root_id.as_ref().unwrap().clone(),
            proposal_id: p.id.clone(),
            expected_fingerprint: p.fingerprint.clone(),
            decision: Decision::Approve {
                candidates: p.candidates.clone(),
            },
        })
        .unwrap_err();
    assert_eq!(e.code, "stale_scope");
}

#[given(expr = "the run is authorized only for {string}")]
fn authorized_only(w: &mut Workspace, path: String) {
    selected(w, "Public selected content".into());
    assert_eq!(path, "Meetings/one.md");
    w.run_id = Some(
        w.service
            .analysis_start(AnalysisStartRequest {
                root_id: w.root_id.clone(),
                scope_id: w.scope.as_ref().unwrap().scope_id.clone(),
                confirm_unknown: true,
            })
            .unwrap()
            .run_id,
    );
}
#[given(expr = "{string} is local-only")]
fn runtime_private(w: &mut Workspace, path: String) {
    w.write(&path, "# Secret\nSYNTHETIC PRIVATE MARKER\n");
    let s = w.citation(&path);
    w.service
        .sensitivity_set(SensitivitySetRequest {
            root_id: w.root_id.clone(),
            target: Endpoint::Source {
                source_id: s.source_id,
                locator: s.locator,
            },
            sensitivity: Sensitivity::LocalOnly,
        })
        .unwrap();
    w.write(
        "Meetings/unselected.md",
        "# Unselected\nSYNTHETIC UNSELECTED MARKER\n",
    );
}
#[when(expr = "the model requests {string} or an unselected source by ID")]
fn request_private(w: &mut Workspace, path: String) {
    assert_eq!(path, "Meetings/private.md");
    let result = w
        .service
        .scope_read(ScopeReadRequest {
            root_id: w.root_id.clone(),
            scope_id: w.scope.as_ref().unwrap().scope_id.clone(),
            run_id: w.run_id.as_ref().unwrap().clone(),
        })
        .unwrap();
    w.received = result.sources;
    // There is deliberately no source-id/path read in the model API: a source endpoint
    // cannot be deserialized as the bounded ScopeReadRequest and cannot broaden it.
    let value = serde_json::json!({"root_id":w.root_id,"scope_id":w.scope.as_ref().unwrap().scope_id,"run_id":w.run_id,"source_id":w.citation(&path).source_id});
    assert!(serde_json::from_value::<ScopeReadRequest>(value).is_err());
}
#[then("the tool returns no source content or derived private context")]
fn no_private_content(w: &mut Workspace) {
    let out = serde_json::to_string(&w.received).unwrap();
    assert!(!out.contains("PRIVATE"));
    assert!(!out.contains("UNSELECTED"));
    assert_eq!(w.received.len(), 1);
    assert_eq!(w.received[0].summary.path, "Meetings/one.md");
}
#[given("a model run returns a proposal with an invalid source locator")]
fn malformed_output(w: &mut Workspace) {
    selected(w, "Alex leads Ledger".into());
    let t = topic("Alex", TopicKind::Person);
    let mut source = w.source.as_ref().unwrap().clone();
    source.locator.end_byte = u64::MAX;
    w.candidates = vec![
        Candidate::Topic { topic: t.clone() },
        Candidate::Claim {
            claim: claim(&t, "Leader", &source),
        },
    ];
}
#[when("the run completes")]
fn malformed_complete(w: &mut Workspace) {
    let scope_id = w.scope.as_ref().unwrap().scope_id.clone();
    let run = w
        .service
        .analysis_start(AnalysisStartRequest {
            root_id: w.root_id.clone(),
            scope_id: scope_id.clone(),
            confirm_unknown: true,
        })
        .unwrap();
    w.error = Some(
        w.service
            .stage_proposal(StageProposalRequest {
                root_id: w.root_id.clone(),
                scope_id: Some(scope_id),
                origin: Origin::Analysis { run_id: run.run_id },
                candidates: w.candidates.clone(),
            })
            .unwrap_err(),
    );
}
#[then("the proposal is rejected with an actionable error")]
fn malformed_rejected(w: &mut Workspace) {
    assert_eq!(w.error.as_ref().unwrap().code, "stale_evidence");
    assert!(w.error.as_ref().unwrap().message.contains("locator"));
    assert!(
        w.service
            .proposals_list(w.request())
            .unwrap()
            .proposals
            .is_empty()
    );
}
#[then("no accepted record or source file changes")]
fn malformed_no_changes(w: &mut Workspace) {
    assert!(
        w.service
            .topics_list(w.request())
            .unwrap()
            .topics
            .is_empty()
    );
    w.assert_source_unchanged();
}

#[tokio::main]
async fn main() {
    Workspace::cucumber()
        .fail_on_skipped()
        .filter_run_and_exit("features/topic-workspace", |_, _, scenario| {
            !matches!(
                scenario.name.as_str(),
                "Choose a fresh root from the desktop"
                    | "Missing Codex is actionable"
                    | "Unexpected tools or isolation failure block a real-note run"
                    | "Cancel and late model events cannot create a proposal"
            )
        })
        .await;
}
