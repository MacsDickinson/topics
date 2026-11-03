//! Reviewer scratch probes. Synthetic temp roots and absolute app data only. Each test asserts the
//! CORRECT behaviour, so a failure confirms a defect.
use std::{fs, path::PathBuf};
use topics_core::*;
use topics_store::WorkspaceService;
fn id() -> Id {
    uuid::Uuid::new_v4().to_string()
}
fn topic(name: &str) -> Topic {
    Topic {
        id: id(),
        name: name.into(),
        topic_kind: TopicKind::Person,
        aliases: vec![],
        sensitivity: Sensitivity::CloudAllowed,
    }
}
struct F {
    temp: tempfile::TempDir,
    root: PathBuf,
    root_id: Id,
    service: WorkspaceService,
}
impl F {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("notes");
        let mut service = WorkspaceService::open(temp.path().join("app-data")).unwrap();
        let root_id = service
            .root_set(RootSetRequest {
                path: root.to_str().unwrap().into(),
                create: true,
            })
            .unwrap()
            .root
            .root_id;
        Self {
            temp,
            root,
            root_id,
            service,
        }
    }
    fn req(&self) -> RootRequest {
        RootRequest {
            root_id: self.root_id.clone(),
        }
    }
    fn write(&self, path: &str, bytes: impl AsRef<[u8]>) {
        let p = self.root.join(path);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, bytes).unwrap();
    }
    fn summary(&self, path: &str) -> SourceSummary {
        self.service
            .sources_list(self.req())
            .unwrap()
            .sources
            .into_iter()
            .find(|s| s.path == path)
            .unwrap()
    }
    fn source(&self, path: &str) -> ScopedSource {
        let s = self.summary(path);
        self.service
            .source_get(SourceGetRequest {
                root_id: self.root_id.clone(),
                source_id: s.source_id,
            })
            .unwrap()
    }
    fn preview(&self, files: Vec<&str>) -> ScopePreview {
        self.preview_spec(files, vec![]).unwrap()
    }
    fn preview_spec(&self, files: Vec<&str>, folders: Vec<&str>) -> WorkspaceResult<ScopePreview> {
        self.service.scope_preview(ScopePreviewRequest {
            root_id: self.root_id.clone(),
            spec: ScopeSpec {
                files: files.into_iter().map(str::to_string).collect(),
                folders: folders.into_iter().map(str::to_string).collect(),
                date_from: None,
                date_to: None,
            },
        })
    }
    fn run(&self, p: &ScopePreview) -> Id {
        self.service
            .analysis_start(AnalysisStartRequest {
                root_id: self.root_id.clone(),
                scope_id: p.scope_id.clone(),
                confirm_unknown: true,
            })
            .unwrap()
            .run_id
    }
    fn model_stage(
        &self,
        files: Vec<&str>,
        candidates: Vec<Candidate>,
    ) -> WorkspaceResult<Proposal> {
        let p = self.preview(files);
        let run_id = self.run(&p);
        self.service.stage_proposal(StageProposalRequest {
            root_id: self.root_id.clone(),
            scope_id: Some(p.scope_id),
            origin: Origin::Analysis { run_id },
            candidates,
        })
    }
    fn stage(&self, candidates: Vec<Candidate>) -> WorkspaceResult<Proposal> {
        self.service
            .manual_propose(ManualProposeRequest {
                root_id: self.root_id.clone(),
                candidates,
            })
            .map(|p| p.proposal)
    }
    fn decide(&self, p: &Proposal, decision: Decision) -> WorkspaceResult<ProposalDecideResult> {
        self.service.proposal_decide(ProposalDecideRequest {
            root_id: self.root_id.clone(),
            proposal_id: p.id.clone(),
            expected_fingerprint: p.fingerprint.clone(),
            decision,
        })
    }
    fn approve(&self, p: &Proposal) -> WorkspaceResult<ProposalDecideResult> {
        self.decide(
            p,
            Decision::Approve {
                candidates: p.candidates.clone(),
            },
        )
    }
    fn accept(&self, c: Vec<Candidate>) -> ProposalDecideResult {
        let p = self.stage(c).unwrap();
        self.approve(&p).unwrap()
    }
    fn label(&self, s: &SourceSummary, l: Sensitivity) -> Operation {
        self.service
            .sensitivity_set(SensitivitySetRequest {
                root_id: self.root_id.clone(),
                target: Endpoint::Source {
                    source_id: s.source_id.clone(),
                    locator: Locator {
                        start_byte: 0,
                        end_byte: 0,
                        heading: None,
                    },
                },
                sensitivity: l,
            })
            .unwrap()
            .operation
    }
    fn label_topic(&self, t: &Topic, l: Sensitivity) -> Operation {
        self.service
            .sensitivity_set(SensitivitySetRequest {
                root_id: self.root_id.clone(),
                target: Endpoint::Topic { id: t.id.clone() },
                sensitivity: l,
            })
            .unwrap()
            .operation
    }
    fn view(&self, t: &Topic) -> WorkspaceResult<TopicView> {
        self.service.topic_get(TopicGetRequest {
            root_id: self.root_id.clone(),
            topic_id: t.id.clone(),
        })
    }
    fn partition(&self) -> PathBuf {
        fs::read_dir(self.temp.path().join("app-data/workspaces"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path()
    }
    fn claim(&self, t: &Topic, s: &SourceRef, v: &str) -> Claim {
        Claim {
            id: id(),
            revision_id: id(),
            subject_id: t.id.clone(),
            predicate: "role".into(),
            value: v.into(),
            uncertainty: Some("synthetic".into()),
            sources: vec![Evidence::Passage { source: s.clone() }],
            supersedes: vec![],
            recorded_at: "2026-11-03T00:00:00Z".into(),
            valid_from: None,
            valid_to: None,
        }
    }
    fn conn(&self, a: &Topic, b: &Topic, s: &SourceRef) -> Connection {
        Connection {
            id: id(),
            from: Endpoint::Topic { id: a.id.clone() },
            relation: "leads".into(),
            to: Endpoint::Topic { id: b.id.clone() },
            uncertainty: None,
            sources: vec![s.clone()],
            valid_from: None,
            valid_to: None,
        }
    }
    fn correct(&self, t: &Topic, old: &Claim, new_value: &str) -> (Claim, Rule) {
        let st = self
            .service
            .user_statement(UserStatementRequest {
                root_id: self.root_id.clone(),
                topic_id: t.id.clone(),
                text: "correction".into(),
            })
            .unwrap();
        let mut c = old.clone();
        c.revision_id = id();
        c.value = new_value.into();
        c.supersedes = vec![old.revision_id.clone()];
        c.sources = vec![Evidence::UserStatement {
            statement_id: st.id,
        }];
        let r = Rule {
            id: id(),
            topic_id: t.id.clone(),
            predicate: old.predicate.clone(),
            match_value: old.value.clone(),
            replacement: new_value.into(),
            scope: RuleScope {
                source_paths: vec![],
                context: None,
            },
            source_revision_id: c.revision_id.clone(),
        };
        (c, r)
    }
}
fn tc(t: &Topic) -> Candidate {
    Candidate::Topic { topic: t.clone() }
}
fn cc(c: &Claim) -> Candidate {
    Candidate::Claim { claim: c.clone() }
}

fn fresh(c: &Claim) -> Claim {
    let mut n = c.clone();
    n.id = id();
    n.revision_id = id();
    n
}
fn code<T>(r: &WorkspaceResult<T>) -> String {
    match r {
        Ok(_) => "ok".into(),
        Err(e) => e.code.clone(),
    }
}
fn human_cloud(f: &F, t: &Topic) {
    f.label_topic(t, Sensitivity::CloudAllowed);
}
fn alex_flow(f: &F) -> (Topic, Claim, SourceRef) {
    f.write("M/a.md", "alex he/him\n");
    let s = f.source("M/a.md");
    let t = topic("Alex");
    let c = f.claim(&t, &s.citation, "he/him");
    let p = f.model_stage(vec!["M/a.md"], vec![tc(&t), cc(&c)]).unwrap();
    f.approve(&p).unwrap();
    human_cloud(f, &t);
    let (n, r) = f.correct(&t, &c, "they/them");
    f.accept(vec![Candidate::Correction { claim: n, rule: r }]);
    (t, c, s.citation)
}

// ---------- Independent final probes. Assert CORRECT behaviour; failure = counterexample. ----------
fn ctx(f: &F, scope: &ScopePreview, run: &str, t: &Id) -> WorkspaceResult<TopicView> {
    f.service.source_context(SourceContextRequest {
        root_id: f.root_id.clone(),
        scope_id: scope.scope_id.clone(),
        run_id: run.into(),
        topic_id: t.clone(),
    })
}
fn req(f: &F, scope: &ScopePreview, run: &str, candidates: Vec<Candidate>) -> StageProposalRequest {
    StageProposalRequest {
        root_id: f.root_id.clone(),
        scope_id: Some(scope.scope_id.clone()),
        origin: Origin::Analysis { run_id: run.into() },
        candidates,
    }
}
fn private_fixture(name: &str) -> (F, Topic, SourceRef) {
    let f = F::new();
    f.write("M/a.md", "selected evidence about things\n");
    let s = f.source("M/a.md").citation;
    let mut p = topic(name);
    p.sensitivity = Sensitivity::LocalOnly;
    f.accept(vec![tc(&p)]);
    (f, p, s)
}
fn claim_endpoint_batch(f: &F, name: &str, s: &SourceRef) -> Vec<Candidate> {
    let t = topic(name);
    let other = topic("Unrelated Ledger");
    let c = f.claim(&t, s, "a fact");
    let conn = Connection {
        id: id(),
        from: Endpoint::Claim {
            id: c.revision_id.clone(),
        },
        relation: "concerns".into(),
        to: Endpoint::Topic {
            id: other.id.clone(),
        },
        uncertainty: None,
        sources: vec![s.clone()],
        valid_from: None,
        valid_to: None,
    };
    vec![
        tc(&t),
        cc(&c),
        tc(&other),
        Candidate::Connection { connection: conn },
    ]
}

/// N1: name collision must not change the unit result when a connection cites the claim of a colliding topic.
#[test]
fn a1_n1_claim_endpoint_connection_private_name_oracle() {
    let (f, _p, s) = private_fixture("SecretName");
    let scope = f.preview(vec!["M/a.md"]);
    let run = f.run(&scope);
    let hit = f.service.validate_proposal(req(
        &f,
        &scope,
        &run,
        claim_endpoint_batch(&f, "SecretName", &s),
    ));
    let miss = f.service.validate_proposal(req(
        &f,
        &scope,
        &run,
        claim_endpoint_batch(&f, "Nobody", &s),
    ));
    eprintln!("OBS a1 validate hit={hit:?} miss={miss:?}");
    assert_eq!(
        hit, miss,
        "validate_proposal result depends on a local-only topic name"
    );
}
#[test]
fn a1b_n1_claim_endpoint_terminal_stage_private_name_poison() {
    let (f, _p, s) = private_fixture("SecretName");
    let scope = f.preview(vec!["M/a.md"]);
    let run = f.run(&scope);
    let hit = f.service.stage_proposal(req(
        &f,
        &scope,
        &run,
        claim_endpoint_batch(&f, "SecretName", &s),
    ));
    eprintln!(
        "OBS a1b stage hit={:?}",
        hit.as_ref()
            .map(|p| p.status)
            .map_err(|e| (e.code.clone(), e.message.clone()))
    );
    assert!(
        hit.is_ok(),
        "the unrelated analysis batch is lost because of a local-only name"
    );
}
/// Same shape with a reversed endpoint and a local-only ALIAS.
#[test]
fn a1c_n1_claim_endpoint_alias_collision() {
    let f = F::new();
    f.write("M/a.md", "selected evidence\n");
    let s = f.source("M/a.md").citation;
    let mut p = topic("Other Name");
    p.aliases = vec!["HiddenAlias".into()];
    p.sensitivity = Sensitivity::LocalOnly;
    f.accept(vec![tc(&p)]);
    let scope = f.preview(vec!["M/a.md"]);
    let run = f.run(&scope);
    let mk = |name: &str| {
        let mut b = claim_endpoint_batch(&f, name, &s);
        if let Candidate::Connection { connection } = &mut b[3] {
            std::mem::swap(&mut connection.from, &mut connection.to);
        }
        b
    };
    let hit = f
        .service
        .validate_proposal(req(&f, &scope, &run, mk("hiddenalias")));
    let miss = f
        .service
        .validate_proposal(req(&f, &scope, &run, mk("Nobody")));
    eprintln!("OBS a1c hit={hit:?} miss={miss:?}");
    assert_eq!(hit, miss);
}
/// N1 positive: ordinary shapes, malformed uniformity, no leak, subset approval and whole approval.
#[test]
fn a2_n1_ordinary_shapes_uniform_and_human_only() {
    let (f, p, s) = private_fixture("SecretName");
    let scope = f.preview(vec!["M/a.md"]);
    let run = f.run(&scope);
    let shapes = |name: &str| -> Vec<Vec<Candidate>> {
        let t = topic(name);
        let u = topic("Unrelated");
        let c = f.claim(&t, &s, "fact");
        let uc = f.claim(&u, &s, "fact2");
        vec![
            vec![tc(&t)],
            vec![tc(&t), cc(&c)],
            vec![
                tc(&t),
                tc(&u),
                cc(&uc),
                Candidate::Connection {
                    connection: f.conn(&t, &u, &s),
                },
            ],
            vec![
                tc(&t),
                tc(&u),
                Candidate::Connection {
                    connection: f.conn(&u, &t, &s),
                },
                Candidate::UnresolvedIdentity {
                    name: name.into(),
                    candidates: vec![t.id.clone(), u.id.clone()],
                    sources: vec![s.clone()],
                },
            ],
        ]
    };
    for name in ["SecretName", "secretname", "  SECRETNAME "] {
        for (a, b) in shapes(name).into_iter().zip(shapes("Nobody")) {
            let ra = f.service.validate_proposal(req(&f, &scope, &run, a));
            let rb = f.service.validate_proposal(req(&f, &scope, &run, b));
            assert_eq!(ra, rb, "name {name}");
            assert!(ra.is_ok());
        }
    }
    assert!(
        f.service
            .proposals_list(f.req())
            .unwrap()
            .proposals
            .iter()
            .all(|x| x.status == ProposalStatus::Accepted),
        "validation published"
    );
    // Terminal: pending holds the private alternative for the human; subset works; private untouched.
    let t = topic("SecretName");
    let u = topic("Unrelated");
    let c = f.claim(&t, &s, "fact");
    let uc = f.claim(&u, &s, "fact2");
    let pr = f
        .service
        .stage_proposal(req(&f, &scope, &run, vec![tc(&t), cc(&c), tc(&u), cc(&uc)]))
        .unwrap();
    assert_eq!(pr.status, ProposalStatus::Pending);
    assert!(pr.candidates.iter().any(|x| matches!(x, Candidate::UnresolvedIdentity { candidates, .. } if candidates.contains(&p.id))));
    let subset: Vec<_> = pr
        .candidates
        .iter()
        .filter(|x| {
            matches!(x, Candidate::Topic { topic } if topic.id == u.id)
                || matches!(x, Candidate::Claim { claim } if claim.subject_id == u.id)
        })
        .cloned()
        .collect();
    assert_eq!(subset.len(), 2);
    f.decide(&pr, Decision::Approve { candidates: subset })
        .unwrap();
    assert_eq!(f.view(&u).unwrap().current_claims.len(), 1);
    assert!(f.view(&p).unwrap().current_claims.is_empty());
}
/// Whole approval including the private alternative must never enter later model context.
#[test]
fn a3_n1_approved_ambiguity_never_reaches_model_context() {
    let (f, p, s) = private_fixture("SecretName");
    let scope = f.preview(vec!["M/a.md"]);
    let run = f.run(&scope);
    let t = topic("SecretName");
    let c = f.claim(&t, &s, "fact");
    let pr = f
        .service
        .stage_proposal(req(&f, &scope, &run, vec![tc(&t), cc(&c)]))
        .unwrap();
    let r = f.approve(&pr);
    eprintln!(
        "OBS a3 whole approval={:?}",
        r.as_ref()
            .map(|x| x.proposal.status)
            .map_err(|e| e.message.clone())
    );
    r.unwrap();
    f.label_topic(&t, Sensitivity::CloudAllowed);
    let scope2 = f.preview(vec!["M/a.md"]);
    let run2 = f.run(&scope2);
    let v = ctx(&f, &scope2, &run2, &t.id).unwrap();
    let j = serde_json::to_string(&v).unwrap();
    assert!(!j.contains(&p.id), "private ID in model context: {j}");
    assert!(f.view(&t).unwrap().unresolved.iter().any(|x| matches!(x, Candidate::UnresolvedIdentity { candidates, .. } if candidates.contains(&p.id))), "human keeps it");
    assert_eq!(code(&ctx(&f, &scope2, &run2, &p.id)), "forbidden");
}
/// Whole approval of a topic-only ambiguity against two accepted same-name topics; UUID order fixed.
#[test]
fn a4_topic_only_ambiguity_whole_approval_is_order_independent() {
    for proposed_id in [
        "00000000-0000-4000-8000-000000000001",
        "ffffffff-ffff-4fff-bfff-ffffffffffff",
    ] {
        let f = F::new();
        f.write("M/a.md", "selected evidence\n");
        f.source("M/a.md");
        let a = topic("Joe");
        let b = topic("Joe");
        f.accept(vec![tc(&a)]);
        f.accept(vec![tc(&b)]);
        let scope = f.preview(vec!["M/a.md"]);
        let run = f.run(&scope);
        let mut t = topic("Joe");
        t.id = proposed_id.into();
        let pr = f
            .service
            .stage_proposal(req(&f, &scope, &run, vec![tc(&t)]))
            .unwrap();
        let r = f.approve(&pr);
        eprintln!(
            "OBS a4 id={proposed_id} approval={:?} candidates={}",
            r.as_ref()
                .map(|x| x.proposal.status)
                .map_err(|e| (e.code.clone(), e.message.clone())),
            pr.candidates.len()
        );
        assert!(
            r.is_ok(),
            "reviewing exactly what was proposed conflicts depending on UUID order"
        );
    }
}
/// N2 raw chat: uncertainty-only revision of an unchanged transcript value.
fn deal_fixture() -> (F, Topic, Claim, SourceRef) {
    let f = F::new();
    f.write("M/secret.md", "acquisition target zed\n");
    f.write("M/ok.md", "ok\n");
    let s = f.source("M/secret.md").citation;
    f.source("M/ok.md");
    let t = topic("Deal");
    let c = f.claim(&t, &s, "acquisition target is Zed");
    f.accept(vec![tc(&t), cc(&c)]);
    (f, t, c, s)
}
fn chat_revision(
    f: &F,
    t: &Topic,
    old: &Claim,
    message: &str,
    edit: impl FnOnce(&mut Claim, Evidence),
) -> Proposal {
    let scope = f.preview(vec!["M/secret.md"]);
    let chat = f
        .service
        .chat_start(ChatStartRequest {
            root_id: f.root_id.clone(),
            topic_id: t.id.clone(),
            scope_id: scope.scope_id.clone(),
            confirm_unknown: true,
            message: message.into(),
        })
        .unwrap();
    let mut n = old.clone();
    n.revision_id = id();
    n.supersedes = vec![old.revision_id.clone()];
    edit(
        &mut n,
        Evidence::UserStatement {
            statement_id: chat.message_id.clone(),
        },
    );
    f.service
        .stage_proposal(StageProposalRequest {
            root_id: f.root_id.clone(),
            scope_id: Some(scope.scope_id),
            origin: Origin::Chat {
                topic_id: t.id.clone(),
                message_id: chat.message_id,
            },
            candidates: vec![cc(&n)],
        })
        .unwrap()
}
fn model_context_json(f: &F, t: &Topic) -> String {
    let scope = f.preview(vec!["M/ok.md"]);
    let run = f.run(&scope);
    serde_json::to_string(&ctx(f, &scope, &run, &t.id).unwrap()).unwrap()
}
#[test]
fn b0_n2_control_no_statement_value_withheld() {
    let (f, t, _, _) = deal_fixture();
    assert!(!model_context_json(&f, &t).contains("Zed"));
}
#[test]
fn b1_n2_chat_uncertainty_only_statement_must_not_declassify() {
    let (f, t, old, _) = deal_fixture();
    let p = chat_revision(&f, &t, &old, "I'm not sure about this", |n, e| {
        n.uncertainty = Some("user unsure".into());
        n.sources = vec![e];
    });
    eprintln!(
        "OBS b1 plain uncertainty-only chat status={:?} candidates={}",
        p.status,
        p.candidates.len()
    );
    if p.status == ProposalStatus::Pending {
        f.approve(&p).unwrap();
    }
    let j = model_context_json(&f, &t);
    eprintln!("OBS b1 model context={j}");
    assert!(
        !j.contains("Zed"),
        "unchanged transcript value exported from unselected source after uncertainty-only chat"
    );
}
#[test]
fn b2_n2_chat_keeps_passage_plus_statement_must_not_declassify() {
    let (f, t, old, _) = deal_fixture();
    let p = chat_revision(&f, &t, &old, "Is that really the target?", |n, e| {
        n.uncertainty = Some("question raised".into());
        n.sources.push(e);
    });
    f.approve(&p).unwrap();
    let j = model_context_json(&f, &t);
    eprintln!("OBS b2 contains_value={}", j.contains("Zed"));
    assert!(!j.contains("Zed"));
}
#[test]
fn b3_n2_statement_on_manual_metadata_revision_must_not_declassify() {
    let (f, t, old, _) = deal_fixture();
    let st = f
        .service
        .user_statement(UserStatementRequest {
            root_id: f.root_id.clone(),
            topic_id: t.id.clone(),
            text: "check this with legal".into(),
        })
        .unwrap();
    let mut n = old.clone();
    n.revision_id = id();
    n.supersedes = vec![old.revision_id.clone()];
    n.uncertainty = Some("check with legal".into());
    n.sources = vec![Evidence::UserStatement {
        statement_id: st.id,
    }];
    f.accept(vec![cc(&n)]);
    let j = model_context_json(&f, &t);
    eprintln!("OBS b3 contains_value={}", j.contains("Zed"));
    assert!(!j.contains("Zed"));
}
#[test]
fn b4_n2_positive_chat_correction_changed_value_stays_usable() {
    let (f, t, old, _) = deal_fixture();
    let p = chat_revision(&f, &t, &old, "No, the target is Acme", |n, e| {
        n.value = "acquisition target is Acme".into();
        n.sources = vec![e];
    });
    f.approve(&p).unwrap();
    let j = model_context_json(&f, &t);
    assert!(
        j.contains("Acme")
            && !j.contains("Zed")
            && !j.contains("M/secret.md")
            && !j.contains("No, the target"),
        "{j}"
    );
    let human = serde_json::to_string(&f.view(&t).unwrap()).unwrap();
    assert!(human.contains("Zed"));
}
#[test]
fn b5_n2_positive_ui_value_edit_and_statement_retained_after_metadata_edit_blocked() {
    let (f, t, _, s) = deal_fixture();
    // value edit: exported; statement exists for the human.
    let c = f.claim(&t, &s, "headcount is forty");
    let c = Claim {
        predicate: "headcount".into(),
        ..c
    };
    let p = f.stage(vec![cc(&c)]).unwrap();
    let mut e = c.clone();
    e.value = "headcount is fifty".into();
    f.decide(
        &p,
        Decision::Approve {
            candidates: vec![cc(&e)],
        },
    )
    .unwrap();
    // metadata edit: withheld, but exact statement remains for the human.
    let m = f.claim(&t, &s, "budget is ten");
    let m = Claim {
        predicate: "budget".into(),
        ..m
    };
    let p = f.stage(vec![cc(&m)]).unwrap();
    let mut e = m.clone();
    e.uncertainty = Some("low".into());
    let d = f
        .decide(
            &p,
            Decision::Approve {
                candidates: vec![cc(&e)],
            },
        )
        .unwrap();
    let applied = d
        .proposal
        .candidates
        .iter()
        .find_map(|c| match c {
            Candidate::Claim { claim } => Some(claim.clone()),
            _ => None,
        })
        .unwrap();
    let sid = applied
        .sources
        .iter()
        .find_map(|e| match e {
            Evidence::UserStatement { statement_id } => Some(statement_id.clone()),
            _ => None,
        })
        .expect("UI statement persisted");
    let st = f
        .service
        .statement_get(StatementGetRequest {
            root_id: f.root_id.clone(),
            statement_id: sid,
        })
        .unwrap();
    assert!(st.text.contains("budget is ten"));
    let j = model_context_json(&f, &t);
    assert!(
        j.contains("headcount is fifty") && !j.contains("forty") && !j.contains("budget is ten"),
        "{j}"
    );
}
/// N3 battery.
fn moved_fixture() -> (F, SourceSummary, SourceSummary, String) {
    let f = F::new();
    f.write("M/p.md", "private bytes\n");
    let p = f.summary("M/p.md");
    f.label(&p, Sensitivity::LocalOnly);
    f.write("M/q.md", "public bytes\n");
    let q = f.summary("M/q.md");
    f.label(&q, Sensitivity::CloudAllowed);
    assert_eq!(
        f.preview(vec!["M/q.md"]).sources[0].sensitivity,
        Sensitivity::CloudAllowed
    );
    let h = p.hash.clone();
    (f, p, q, h)
}
#[test]
fn c1_n3_move_copy_never_inherits_destination_or_confirm_override() {
    let (f, _, q, h) = moved_fixture();
    fs::copy(f.root.join("M/p.md"), f.root.join("M/q.md")).unwrap(); // copy keeps p too
    let pv = f.preview(vec!["M/q.md"]);
    assert!(pv.sources.is_empty(), "copied bytes eligible");
    assert_eq!(
        f.service
            .analysis_start(AnalysisStartRequest {
                root_id: f.root_id.clone(),
                scope_id: pv.scope_id.clone(),
                confirm_unknown: true
            })
            .unwrap_err()
            .code,
        "invalid_input"
    );
    fs::remove_file(f.root.join("M/p.md")).unwrap(); // now a true move
    assert!(f.preview(vec!["M/q.md"]).sources.is_empty());
    // New unlabelled destination.
    fs::copy(f.root.join("M/q.md"), f.root.join("M/new.md")).unwrap();
    assert!(
        f.preview_spec(vec![], vec!["M"])
            .unwrap()
            .sources
            .is_empty()
    );
    // Stale human decision about old q identity cannot release.
    f.label(&q, Sensitivity::CloudAllowed);
    assert!(f.preview(vec!["M/q.md"]).sources.is_empty());
    // Inventory shows it restricted and does not rewrite an existing mapping as allowed.
    assert_eq!(f.summary("M/q.md").hash, h);
    assert_eq!(f.summary("M/q.md").sensitivity, Sensitivity::LocalOnly);
}
#[test]
fn c2_n3_older_review_of_same_hash_does_not_override_newer_private_label() {
    let f = F::new();
    f.write("M/q.md", "shared bytes\n");
    let q = f.summary("M/q.md");
    f.label(&q, Sensitivity::CloudAllowed);
    f.write("M/r.md", "shared bytes\n");
    let r = f.summary("M/r.md");
    f.label(&r, Sensitivity::LocalOnly);
    let pv = f.preview(vec!["M/q.md"]);
    eprintln!("OBS c2 q eligible={}", pv.sources.len());
    assert!(pv.sources.is_empty());
}
#[test]
fn c3_n3_fresh_review_then_undo_returns_restricted() {
    let (f, _, _, h) = moved_fixture();
    fs::rename(f.root.join("M/p.md"), f.root.join("M/q.md")).unwrap();
    let moved = f.summary("M/q.md");
    assert_eq!(moved.hash, h);
    let op = f.label(&moved, Sensitivity::CloudAllowed);
    assert_eq!(
        f.preview(vec!["M/q.md"]).sources.len(),
        1,
        "explicit human fresh review releases"
    );
    f.service
        .undo(UndoRequest {
            root_id: f.root_id.clone(),
            operation_id: op.id,
        })
        .unwrap();
    assert!(
        f.preview(vec!["M/q.md"]).sources.is_empty(),
        "undo re-restricts"
    );
}
#[test]
fn c4_n3_repair_cannot_launder_private_hash() {
    let (f, p, q, h) = moved_fixture();
    fs::rename(f.root.join("M/p.md"), f.root.join("M/q.md")).unwrap();
    // repair q's stale cloud identity over the moved private bytes.
    let r = f
        .service
        .source_identity_repair(SourceRepairRequest {
            root_id: f.root_id.clone(),
            source_id: q.source_id.clone(),
            path: "M/q.md".into(),
            expected_hash: h.clone(),
        })
        .unwrap();
    assert_eq!(r.sensitivity, Sensitivity::LocalOnly);
    assert!(f.preview(vec!["M/q.md"]).sources.is_empty());
    // repair p's identity at new path keeps private.
    let r = f.service.source_identity_repair(SourceRepairRequest {
        root_id: f.root_id.clone(),
        source_id: p.source_id.clone(),
        path: "M/q.md".into(),
        expected_hash: h,
    });
    eprintln!(
        "OBS c4 second repair={:?}",
        r.as_ref().map(|s| s.sensitivity)
    );
    assert!(f.preview(vec!["M/q.md"]).sources.is_empty());
}
#[test]
fn c5_n3_private_by_topic_taint_moved_over_reviewed_path() {
    let f = F::new();
    f.write("M/s.md", "tainted bytes\n");
    let s = f.source("M/s.md").citation;
    let mut private = topic("HR Case");
    private.sensitivity = Sensitivity::LocalOnly;
    let c = f.claim(&private, &s, "secret");
    f.accept(vec![tc(&private), cc(&c)]);
    f.write("M/q.md", "public bytes\n");
    let q = f.summary("M/q.md");
    f.label(&q, Sensitivity::CloudAllowed);
    fs::rename(f.root.join("M/s.md"), f.root.join("M/q.md")).unwrap();
    assert!(
        f.preview(vec!["M/q.md"]).sources.is_empty(),
        "hash-lineage topic taint lost"
    );
}
/// Documented bounded limit: observed and reported, not asserted as a defect.
#[test]
fn c6_n3_limit_observation_moved_and_edited() {
    let (f, _, _, _) = moved_fixture();
    fs::rename(f.root.join("M/p.md"), f.root.join("M/q.md")).unwrap();
    fs::write(f.root.join("M/q.md"), "private bytes\nplus one line\n").unwrap();
    let pv = f.preview(vec!["M/q.md"]);
    eprintln!(
        "OBS c6 moved+edited eligible={} sens={:?}",
        pv.sources.len(),
        pv.sources.first().map(|s| s.sensitivity)
    );
    let f2 = F::new();
    f2.write("M/p.md", "v1\n");
    let p = f2.summary("M/p.md");
    f2.label(&p, Sensitivity::LocalOnly);
    f2.write("M/q.md", "pub\n");
    let q = f2.summary("M/q.md");
    f2.label(&q, Sensitivity::CloudAllowed);
    fs::write(f2.root.join("M/p.md"), "v2 private edit\n").unwrap();
    fs::rename(f2.root.join("M/p.md"), f2.root.join("M/q.md")).unwrap();
    let pv = f2.preview(vec!["M/q.md"]);
    eprintln!(
        "OBS c6 edited-before-move-no-inventory eligible={} sens={:?}",
        pv.sources.len(),
        pv.sources.first().map(|s| s.sensitivity)
    );
    let f3 = F::new();
    f3.write("M/p.md", "v1\n");
    let p = f3.summary("M/p.md");
    f3.label(&p, Sensitivity::LocalOnly);
    f3.write("M/q.md", "pub\n");
    let q = f3.summary("M/q.md");
    f3.label(&q, Sensitivity::CloudAllowed);
    fs::write(f3.root.join("M/p.md"), "v2 private edit\n").unwrap();
    f3.summary("M/p.md");
    fs::rename(f3.root.join("M/p.md"), f3.root.join("M/q.md")).unwrap();
    assert!(
        f3.preview(vec!["M/q.md"]).sources.is_empty(),
        "inventoried private edit must carry lineage"
    );
}
/// N3 regression: an explicit-reviewed source later tainted by a local-only topic.
#[test]
fn c7_regression_reviewed_then_tainted_source_in_preview() {
    let f = F::new();
    f.write("M/a.md", "alpha\n");
    f.write("M/b.md", "beta\n");
    let a = f.summary("M/a.md");
    f.label(&a, Sensitivity::CloudAllowed);
    let b = f.summary("M/b.md");
    f.label(&b, Sensitivity::CloudAllowed);
    let s = f.source("M/a.md").citation;
    let t = topic("Later Private");
    let c = f.claim(&t, &s, "v");
    f.accept(vec![tc(&t), cc(&c)]);
    f.label_topic(&t, Sensitivity::LocalOnly);
    let pv = f.preview(vec!["M/a.md", "M/b.md"]);
    let names: Vec<_> = pv.sources.iter().map(|s| s.path.clone()).collect();
    let st = f.service.analysis_start(AnalysisStartRequest {
        root_id: f.root_id.clone(),
        scope_id: pv.scope_id.clone(),
        confirm_unknown: false,
    });
    eprintln!(
        "OBS c7 preview={names:?} excluded={:?} start={:?}",
        pv.excluded
            .iter()
            .map(|e| e.path.clone())
            .collect::<Vec<_>>(),
        st.as_ref().map(|_| "ok").map_err(|e| e.code.clone())
    );
    assert!(
        !names.contains(&"M/a.md".to_string()) || st.is_err(),
        "tainted source reached a run"
    );
    assert!(
        st.is_ok(),
        "one tainted source should be visibly excluded, not block the unrelated selection"
    );
}
/// N4 battery.
#[test]
fn d1_n4_all_endpoint_shapes() {
    let f = F::new();
    f.write("M/ok.md", "# Fine\nok text\n");
    f.write("M/secret.md", "# Secret Merger Heading\nsecret text\n");
    let ok = f.source("M/ok.md").citation;
    let secret = f.source("M/secret.md").citation;
    let t = topic("Deal");
    let u = topic("Other");
    let hidden = f.claim(&u, &secret, "hidden value");
    f.accept(vec![tc(&t), tc(&u), cc(&hidden)]);
    let loc = Locator {
        start_byte: 0,
        end_byte: 10,
        heading: Some("Secret Merger Heading".into()),
    };
    let source_end = Endpoint::Source {
        source_id: secret.source_id.clone(),
        locator: loc,
    };
    let tt = Endpoint::Topic { id: t.id.clone() };
    let mk = |from: Endpoint, to: Endpoint, rel: &str| Candidate::Connection {
        connection: Connection {
            id: id(),
            from,
            relation: rel.into(),
            to,
            uncertainty: None,
            sources: vec![ok.clone()],
            valid_from: None,
            valid_to: None,
        },
    };
    let visible = mk(
        tt.clone(),
        Endpoint::Topic { id: u.id.clone() },
        "visible-rel",
    );
    f.accept(vec![
        mk(tt.clone(), source_end.clone(), "to-source"),
        mk(source_end.clone(), tt.clone(), "from-source"),
        mk(
            tt.clone(),
            Endpoint::Claim {
                id: hidden.revision_id.clone(),
            },
            "to-claim-rev",
        ),
        mk(
            Endpoint::Claim {
                id: hidden.id.clone(),
            },
            tt.clone(),
            "from-claim-id",
        ),
        visible,
    ]);
    let scope = f.preview(vec!["M/ok.md"]);
    let run = f.run(&scope);
    let v = ctx(&f, &scope, &run, &t.id);
    eprintln!(
        "OBS d1 u-restricted? {:?}",
        v.as_ref().map(|v| v
            .connections
            .iter()
            .map(|c| c.relation.clone())
            .collect::<Vec<_>>())
    );
    let j = serde_json::to_string(&v.unwrap()).unwrap();
    for needle in [
        secret.source_id.as_str(),
        "Secret Merger",
        "M/secret.md",
        hidden.revision_id.as_str(),
        hidden.id.as_str(),
        "hidden value",
        "to-source",
        "from-source",
        "to-claim-rev",
        "from-claim-id",
    ] {
        assert!(!j.contains(needle), "{needle} in {j}");
    }
    assert!(j.contains("visible-rel"));
    let human = serde_json::to_string(&f.view(&t).unwrap()).unwrap();
    for needle in [
        "to-source",
        "from-source",
        "to-claim-rev",
        "from-claim-id",
        "Secret Merger",
    ] {
        assert!(human.contains(needle), "human lost {needle}");
    }
}
/// N4: a superseded in-scope revision must not make a connection to a current out-of-scope claim visible.
#[test]
fn d2_n4_claim_endpoint_by_logical_id_uses_current_revision() {
    let mut leaked = 0;
    for _ in 0..12 {
        let f = F::new();
        f.write("M/ok.md", "ok text\n");
        f.write("M/secret.md", "secret text\n");
        let ok = f.source("M/ok.md").citation;
        let secret = f.source("M/secret.md").citation;
        let t = topic("Deal");
        let first = f.claim(&t, &ok, "in scope");
        f.accept(vec![tc(&t), cc(&first)]);
        let mut second = first.clone();
        second.revision_id = id();
        second.supersedes = vec![first.revision_id.clone()];
        second.sources = vec![Evidence::Passage {
            source: secret.clone(),
        }];
        second.value = "out of scope".into();
        f.accept(vec![cc(&second)]);
        f.accept(vec![Candidate::Connection {
            connection: Connection {
                id: id(),
                from: Endpoint::Topic { id: t.id.clone() },
                relation: "logical-rel".into(),
                to: Endpoint::Claim {
                    id: first.id.clone(),
                },
                uncertainty: None,
                sources: vec![ok.clone()],
                valid_from: None,
                valid_to: None,
            },
        }]);
        let scope = f.preview(vec!["M/ok.md"]);
        let run = f.run(&scope);
        let j = serde_json::to_string(&ctx(&f, &scope, &run, &t.id).unwrap()).unwrap();
        assert!(!j.contains("out of scope"), "{j}");
        if j.contains("logical-rel") {
            leaked += 1;
        }
    }
    eprintln!("OBS d2 logical-id connection visible {leaked}/12 (ID only; value never present)");
}
/// N5 and N6, N7, N8, N10 bounded.
#[test]
fn e1_n5_variants_and_no_merge() {
    for variant in ["alex", "Alex ", "ALEX", " a\u{6c}ex", "ALEX\u{a0}"] {
        let f = F::new();
        let (t, _, s) = alex_flow(&f);
        let n = topic(variant);
        let c = f.claim(&n, &s, "he/him");
        let p = f.model_stage(vec!["M/a.md"], vec![tc(&n), cc(&c)]).unwrap();
        let has_claim = p
            .candidates
            .iter()
            .any(|x| matches!(x, Candidate::Claim { .. }));
        eprintln!(
            "OBS e1 {variant:?} status={:?} claim_candidate={has_claim}",
            p.status
        );
        assert!(!has_claim, "variant {variant:?} bypassed review");
        assert_eq!(f.service.topics_list(f.req()).unwrap().topics.len(), 1);
        assert_eq!(f.view(&t).unwrap().current_claims[0].value, "they/them");
    }
    let f = F::new();
    let (_, _, s) = alex_flow(&f);
    let mut n = topic("Alex");
    n.topic_kind = TopicKind::Thread;
    let c = f.claim(&n, &s, "he/him");
    let p = f.model_stage(vec!["M/a.md"], vec![tc(&n), cc(&c)]).unwrap();
    assert!(
        p.candidates
            .iter()
            .all(|x| !matches!(x, Candidate::Claim { .. }))
    );
}
#[test]
fn e2_n5_unicode_composition_observation() {
    let f = F::new();
    f.write("M/a.md", "x\n");
    let s = f.source("M/a.md").citation;
    let t = topic("Jos\u{e9}");
    f.accept(vec![tc(&t)]);
    let n = topic("Jose\u{301}");
    let c = f.claim(&n, &s, "v");
    let p = f.model_stage(vec!["M/a.md"], vec![tc(&n), cc(&c)]).unwrap();
    eprintln!(
        "OBS e2 NFD variant claim_pending={}",
        p.candidates
            .iter()
            .any(|x| matches!(x, Candidate::Claim { .. }))
    );
}
#[test]
fn e3_n6_semantic_dedupe_keeps_supersession() {
    let f = F::new();
    f.write("M/a.md", "x\n");
    let s = f.source("M/a.md").citation;
    let t = topic("Deal");
    let old = f.claim(&t, &s, "old value");
    f.accept(vec![tc(&t), cc(&old)]);
    let scope = f.preview(vec!["M/a.md"]);
    let run = f.run(&scope);
    let plain = f.claim(&t, &s, "new value");
    let mut rev = old.clone();
    rev.revision_id = id();
    rev.supersedes = vec![old.revision_id.clone()];
    rev.value = "new value".into();
    let p = f
        .service
        .stage_proposal(req(&f, &scope, &run, vec![cc(&plain), cc(&rev)]))
        .unwrap();
    let kept: Vec<_> = p
        .candidates
        .iter()
        .filter_map(|c| match c {
            Candidate::Claim { claim } => Some(claim.supersedes.len()),
            _ => None,
        })
        .collect();
    eprintln!("OBS e3 claims kept supersedes lens={kept:?}");
    f.approve(&p).unwrap();
    let v = f.view(&t).unwrap();
    let values: Vec<_> = v.current_claims.iter().map(|c| c.value.clone()).collect();
    eprintln!("OBS e3 current={values:?}");
    assert_eq!(
        values,
        vec!["new value".to_string()],
        "dedupe dropped the revision's supersession, old value stays current"
    );
}
#[test]
fn e4_n7_n8() {
    let f = F::new();
    f.write("M/a.md", "x\n");
    f.write("Projects/topics/meeting.md", "nested\n");
    f.write("Other/TOPICS/m2.md", "nested2\n");
    let s = f.source("M/a.md");
    let _ = s;
    let a = topic("Alex");
    f.accept(vec![tc(&a)]);
    for _ in 0..2 {
        let n = topic("Alex");
        let p = f.model_stage(vec!["M/a.md"], vec![tc(&n)]).unwrap();
        assert!(
            p.candidates
                .iter()
                .any(|c| matches!(c, Candidate::UnresolvedIdentity { .. })),
            "repeat not flagged"
        );
    }
    let pending = f
        .service
        .proposals_list(f.req())
        .unwrap()
        .proposals
        .into_iter()
        .filter(|p| p.status == ProposalStatus::Pending)
        .count();
    eprintln!("OBS e4 pending={pending}");
    let inv: Vec<_> = f
        .service
        .sources_list(f.req())
        .unwrap()
        .sources
        .into_iter()
        .map(|s| s.path)
        .collect();
    assert!(
        inv.contains(&"Projects/topics/meeting.md".to_string())
            && inv.contains(&"Other/TOPICS/m2.md".to_string())
    );
    assert!(inv.iter().all(|p| !p.to_lowercase().starts_with("topics/")));
    let pv = f
        .preview_spec(
            vec!["topics/topics/x.md", "TOPICS/operations/y.md"],
            vec!["Projects", "Other", "topics", "ToPiCs"],
        )
        .unwrap();
    let got: Vec<_> = pv.sources.iter().map(|s| s.path.clone()).collect();
    assert_eq!(got.len(), 2, "{got:?}");
    assert!(
        got.iter()
            .all(|p| p.starts_with("Projects/") || p.starts_with("Other/"))
    );
    assert!(pv.excluded.len() >= 2);
}
#[test]
fn e5_n10_orphan_correction_no_false_revive() {
    let f = F::new();
    let (t, _, _) = alex_flow(&f);
    let ops = f.service.history_list(f.req()).unwrap().operations;
    let correction = ops.last().unwrap().clone();
    fs::remove_file(
        f.root
            .join(format!("topics/operations/{}.md", correction.id)),
    )
    .unwrap();
    let data = tempfile::tempdir().unwrap();
    let mut service = WorkspaceService::open(data.path().join("rebuilt")).unwrap();
    let root_id = service
        .root_set(RootSetRequest {
            path: f.root.to_str().unwrap().into(),
            create: false,
        })
        .unwrap()
        .root
        .root_id;
    let v = service
        .topic_get(TopicGetRequest {
            root_id: root_id.clone(),
            topic_id: t.id.clone(),
        })
        .unwrap();
    eprintln!(
        "OBS e5 warnings={:?} current={:?}",
        v.warnings,
        v.current_claims
            .iter()
            .map(|c| c.value.clone())
            .collect::<Vec<_>>()
    );
    assert!(v.current_claims.is_empty() && !v.warnings.is_empty());
    assert!(
        v.history.iter().all(|c| c.value != "they/them"),
        "orphan accepted"
    );
    let p = service
        .scope_preview(ScopePreviewRequest {
            root_id: root_id.clone(),
            spec: ScopeSpec {
                files: vec!["M/a.md".into()],
                folders: vec![],
                date_from: None,
                date_to: None,
            },
        })
        .unwrap();
    let scoped_ok = p.sources.len();
    eprintln!("OBS e5 scoped_ok={scoped_ok}");
    if scoped_ok == 1 {
        let run = service
            .analysis_start(AnalysisStartRequest {
                root_id: root_id.clone(),
                scope_id: p.scope_id.clone(),
                confirm_unknown: true,
            })
            .unwrap()
            .run_id;
        let c = service.source_context(SourceContextRequest {
            root_id: root_id.clone(),
            scope_id: p.scope_id,
            run_id: run,
            topic_id: t.id.clone(),
        });
        assert!(
            c.is_err(),
            "orphaned topic context reached the model: {c:?}"
        );
    }
}
#[test]
fn e6_redaction_guard_undone_correction_statement() {
    // Undo of a value correction: old value returns as current and is out of scope again.
    let (f, t, old, _) = deal_fixture();
    let p = chat_revision(&f, &t, &old, "No, the target is Acme", |n, e| {
        n.value = "acquisition target is Acme".into();
        n.sources = vec![e];
    });
    let d = f.approve(&p).unwrap();
    f.service
        .undo(UndoRequest {
            root_id: f.root_id.clone(),
            operation_id: d.operation.unwrap().id,
        })
        .unwrap();
    let j = model_context_json(&f, &t);
    eprintln!(
        "OBS e6 after undo contains Zed={} Acme={}",
        j.contains("Zed"),
        j.contains("Acme")
    );
    assert!(!j.contains("Zed") && !j.contains("Acme"));
}

fn chat_context(f: &F, t: &Topic) -> String {
    // The exact chat sequence: confirmed unrelated scope, chat start, then its context.
    let scope = f.preview(vec!["M/ok.md"]);
    let chat = f
        .service
        .chat_start(ChatStartRequest {
            root_id: f.root_id.clone(),
            topic_id: t.id.clone(),
            scope_id: scope.scope_id.clone(),
            confirm_unknown: true,
            message: "unrelated".into(),
        })
        .unwrap();
    serde_json::to_string(&ctx(f, &scope, &chat.run_id, &t.id).unwrap()).unwrap()
}
#[test]
fn b6_n2_chat_uncertainty_plus_date_metadata_only_declassifies() {
    let (f, t, old, _) = deal_fixture();
    let p = chat_revision(
        &f,
        &t,
        &old,
        "I'm not sure about this; maybe it only applied from March",
        |n, e| {
            n.uncertainty = Some("user unsure".into());
            n.valid_from = Some("2026-03-01".into());
            n.sources = vec![e];
        },
    );
    eprintln!(
        "OBS b6 status={:?} candidates={}",
        p.status,
        p.candidates.len()
    );
    assert_eq!(p.status, ProposalStatus::Pending);
    f.approve(&p).unwrap();
    let j = chat_context(&f, &t);
    eprintln!("OBS b6 chat context={j}");
    assert!(
        !j.contains("Zed"),
        "unchanged transcript value exported in a chat scoped only to M/ok.md"
    );
}
#[test]
fn b7_n2_chat_question_new_predicate_same_transcript_value() {
    let (f, t, old, _) = deal_fixture();
    let scope = f.preview(vec!["M/secret.md"]);
    let chat = f
        .service
        .chat_start(ChatStartRequest {
            root_id: f.root_id.clone(),
            topic_id: t.id.clone(),
            scope_id: scope.scope_id.clone(),
            confirm_unknown: true,
            message: "what is this deal about?".into(),
        })
        .unwrap();
    let mut n = old.clone();
    n.id = id();
    n.revision_id = id();
    n.predicate = "summary".into();
    n.sources = vec![Evidence::UserStatement {
        statement_id: chat.message_id.clone(),
    }];
    let p = f
        .service
        .stage_proposal(StageProposalRequest {
            root_id: f.root_id.clone(),
            scope_id: Some(scope.scope_id),
            origin: Origin::Chat {
                topic_id: t.id.clone(),
                message_id: chat.message_id,
            },
            candidates: vec![cc(&n)],
        })
        .unwrap();
    assert_eq!(p.status, ProposalStatus::Pending);
    f.approve(&p).unwrap();
    let j = chat_context(&f, &t);
    let shown: Vec<_> = serde_json::from_str::<serde_json::Value>(&j).unwrap()["current_claims"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["value"].clone())
        .collect();
    eprintln!("OBS b7 values in chat context={shown:?}");
    assert!(
        !j.contains("Zed"),
        "question-only provenance exported transcript value"
    );
}
#[test]
fn b8_n2_chat_correction_candidate_with_unchanged_value() {
    let (f, t, old, _) = deal_fixture();
    let scope = f.preview(vec!["M/secret.md"]);
    let chat = f
        .service
        .chat_start(ChatStartRequest {
            root_id: f.root_id.clone(),
            topic_id: t.id.clone(),
            scope_id: scope.scope_id.clone(),
            confirm_unknown: true,
            message: "not sure".into(),
        })
        .unwrap();
    let mut n = old.clone();
    n.revision_id = id();
    n.supersedes = vec![old.revision_id.clone()];
    n.uncertainty = Some("user unsure".into());
    n.sources = vec![Evidence::UserStatement {
        statement_id: chat.message_id.clone(),
    }];
    let rule = Rule {
        id: id(),
        topic_id: t.id.clone(),
        predicate: old.predicate.clone(),
        match_value: old.value.clone(),
        replacement: old.value.clone(),
        scope: RuleScope {
            source_paths: vec![],
            context: None,
        },
        source_revision_id: n.revision_id.clone(),
    };
    let r = f.service.stage_proposal(StageProposalRequest {
        root_id: f.root_id.clone(),
        scope_id: Some(scope.scope_id),
        origin: Origin::Chat {
            topic_id: t.id.clone(),
            message_id: chat.message_id,
        },
        candidates: vec![Candidate::Correction { claim: n, rule }],
    });
    eprintln!(
        "OBS b8 stage={:?}",
        r.as_ref().map(|p| p.status).map_err(|e| e.message.clone())
    );
    if let Ok(p) = r {
        let a = f.approve(&p);
        eprintln!(
            "OBS b8 approve={:?}",
            a.as_ref()
                .map(|x| x.proposal.status)
                .map_err(|e| e.message.clone())
        );
        if a.is_ok() {
            let j = chat_context(&f, &t);
            assert!(
                !j.contains("Zed"),
                "unchanged value exported via no-op correction"
            );
        }
    }
}
#[test]
fn a1d_n1_cloud_collisions_same_failure() {
    // Non-private accepted same name, and a model-supplied visible topic whose name collides with a private one.
    let f = F::new();
    f.write("M/a.md", "selected evidence\n");
    let s = f.source("M/a.md").citation;
    let cloud = topic("Alex");
    f.accept(vec![tc(&cloud)]);
    let scope = f.preview(vec!["M/a.md"]);
    let run = f.run(&scope);
    let r =
        f.service
            .validate_proposal(req(&f, &scope, &run, claim_endpoint_batch(&f, "Alex", &s)));
    eprintln!("OBS a1d cloud collision={r:?}");
    let visible = topic("Joe");
    let mut private = topic("joe");
    private.sensitivity = Sensitivity::LocalOnly;
    f.accept(vec![tc(&visible)]);
    let other = topic("Ledger");
    let c = f.claim(&visible, &s, "fact");
    let conn = |c: &Claim| Candidate::Connection {
        connection: Connection {
            id: id(),
            from: Endpoint::Claim {
                id: c.revision_id.clone(),
            },
            relation: "r".into(),
            to: Endpoint::Topic {
                id: other.id.clone(),
            },
            uncertainty: None,
            sources: vec![s.clone()],
            valid_from: None,
            valid_to: None,
        },
    };
    let before =
        f.service
            .validate_proposal(req(&f, &scope, &run, vec![cc(&c), tc(&other), conn(&c)]));
    f.accept(vec![tc(&private)]);
    let scope2 = f.preview(vec!["M/a.md"]);
    let run2 = f.run(&scope2);
    let c2 = f.claim(&visible, &s, "fact");
    let after = f.service.validate_proposal(req(
        &f,
        &scope2,
        &run2,
        vec![cc(&c2), tc(&other), conn(&c2)],
    ));
    eprintln!("OBS a1d visible topic before private label={before:?} after={after:?}");
    assert_eq!(
        before, after,
        "a private same-name topic changes the validation of a claim on a visible topic"
    );
}

#[test]
fn r1_logical_and_revision_endpoints_keep_unrelated_facts_and_uniform_raw_errors() {
    for logical in [false, true] {
        for reverse in [false, true] {
            let (f, private, source) = private_fixture("SecretName");
            let scope = f.preview(vec!["M/a.md"]);
            let run = f.run(&scope);
            let mut candidates = claim_endpoint_batch(&f, "SecretName", &source);
            let claim = match &candidates[1] {
                Candidate::Claim { claim } => claim.clone(),
                _ => unreachable!(),
            };
            let unrelated = match &candidates[2] {
                Candidate::Topic { topic } => topic.clone(),
                _ => unreachable!(),
            };
            if let Candidate::Connection { connection } = &mut candidates[3] {
                connection.from = Endpoint::Claim {
                    id: if logical {
                        claim.id.clone()
                    } else {
                        claim.revision_id.clone()
                    },
                };
                if reverse {
                    std::mem::swap(&mut connection.from, &mut connection.to);
                }
            }
            candidates.push(cc(&f.claim(&unrelated, &source, "eligible unrelated fact")));
            assert_eq!(
                f.service
                    .validate_proposal(req(&f, &scope, &run, candidates.clone())),
                Ok(())
            );
            let mut errors = vec![];
            for name in ["SecretName", "Nobody"] {
                let mut malformed = claim_endpoint_batch(&f, name, &source);
                if let Candidate::Connection { connection } = &mut malformed[3] {
                    connection.from = Endpoint::Claim { id: id() };
                }
                errors.push(
                    f.service
                        .validate_proposal(req(&f, &scope, &run, malformed))
                        .unwrap_err(),
                );
            }
            assert_eq!(errors[0], errors[1]);
            assert_eq!(errors[0].code, "invalid_input");
            assert!(
                !serde_json::to_string(&errors)
                    .unwrap()
                    .contains(&private.id)
            );
            let before = f.service.proposals_list(f.req()).unwrap();
            let pending_files = || {
                fs::read_dir(f.partition().join("proposals"))
                    .unwrap()
                    .count()
            };
            let before_files = pending_files();
            let denied = f
                .service
                .stage_proposal_guarded(req(&f, &scope, &run, candidates.clone()), |_, _| {
                    Err(ApiError::new("cancelled", "revoked"))
                });
            assert_eq!(denied.unwrap_err().code, "cancelled");
            assert_eq!(f.service.proposals_list(f.req()).unwrap(), before);
            assert_eq!(pending_files(), before_files);
            let run = f.run(&scope);
            let proposal = f
                .service
                .stage_proposal(req(&f, &scope, &run, candidates))
                .unwrap();
            assert_eq!(proposal.status, ProposalStatus::Pending);
            assert!(
                proposal
                    .candidates
                    .iter()
                    .all(|c| !matches!(c, Candidate::Connection { .. }))
            );
            assert!(proposal.candidates.iter().any(|c|matches!(c,Candidate::UnresolvedIdentity{candidates,..} if candidates.contains(&private.id))));
            let subset = proposal
                .candidates
                .iter()
                .filter(|c| {
                    matches!(c,Candidate::Topic{topic} if topic.id==unrelated.id)
                        || matches!(c,Candidate::Claim{claim} if claim.subject_id==unrelated.id)
                })
                .cloned()
                .collect();
            f.decide(&proposal, Decision::Approve { candidates: subset })
                .unwrap();
            assert_eq!(f.view(&unrelated).unwrap().current_claims.len(), 1);
            assert!(f.view(&private).unwrap().current_claims.is_empty());
        }
    }
}

#[test]
fn r2_literal_raw_statement_value_is_usable_and_noop_correction_is_rejected() {
    let (f, t, old, _) = deal_fixture();
    let text = format!("Confirmed: {}", old.value);
    let statement = f
        .service
        .user_statement(UserStatementRequest {
            root_id: f.root_id.clone(),
            topic_id: t.id.clone(),
            text: text.clone(),
        })
        .unwrap();
    let mut literal = fresh(&old);
    literal.predicate = "confirmed summary".into();
    literal.sources = vec![Evidence::UserStatement {
        statement_id: statement.id.clone(),
    }];
    f.accept(vec![cc(&literal)]);
    let context = model_context_json(&f, &t);
    assert!(context.contains(&old.value));
    assert!(!context.contains("M/secret.md") && !context.contains("Confirmed:"));
    assert_eq!(
        f.service
            .statement_get(StatementGetRequest {
                root_id: f.root_id.clone(),
                statement_id: statement.id.clone()
            })
            .unwrap()
            .text,
        text
    );
    let mut noop = old.clone();
    noop.revision_id = id();
    noop.supersedes = vec![old.revision_id.clone()];
    noop.sources = vec![Evidence::UserStatement {
        statement_id: statement.id,
    }];
    let rule = Rule {
        id: id(),
        topic_id: t.id.clone(),
        predicate: old.predicate.clone(),
        match_value: old.value.clone(),
        replacement: old.value.clone(),
        scope: RuleScope {
            source_paths: vec![],
            context: None,
        },
        source_revision_id: noop.revision_id.clone(),
    };
    let candidates = vec![Candidate::Correction { claim: noop, rule }];
    assert_eq!(
        validate_candidate_shapes(&candidates).unwrap_err().code,
        "invalid_input"
    );
    let before = f.service.history_list(f.req()).unwrap().operations.len();
    assert_eq!(f.stage(candidates).unwrap_err().code, "invalid_input");
    assert_eq!(
        f.service.history_list(f.req()).unwrap().operations.len(),
        before
    );
}

#[test]
fn r3_superseding_identity_and_all_dependent_endpoints_survive_either_batch_order() {
    for superseding_first in [false, true] {
        let f = F::new();
        f.write("M/a.md", "source\n");
        let source = f.source("M/a.md").citation;
        let t = topic("Subject");
        let old = f.claim(&t, &source, "old value");
        f.accept(vec![tc(&t), cc(&old)]);
        let plain = f.claim(&t, &source, "new value");
        let duplicate = fresh(&plain);
        let mut revision = old.clone();
        revision.revision_id = id();
        revision.value = plain.value.clone();
        revision.supersedes = vec![old.revision_id.clone()];
        let mut candidates = if superseding_first {
            vec![cc(&revision), cc(&plain), cc(&duplicate)]
        } else {
            vec![cc(&plain), cc(&duplicate), cc(&revision)]
        };
        for (index, endpoint) in [
            plain.id.clone(),
            duplicate.revision_id.clone(),
            revision.revision_id.clone(),
        ]
        .into_iter()
        .enumerate()
        {
            candidates.push(Candidate::Connection {
                connection: Connection {
                    id: id(),
                    from: Endpoint::Claim { id: endpoint },
                    relation: format!("relation {index}"),
                    to: Endpoint::Topic { id: t.id.clone() },
                    uncertainty: None,
                    sources: vec![source.clone()],
                    valid_from: None,
                    valid_to: None,
                },
            });
        }
        let proposal = f.model_stage(vec!["M/a.md"], candidates).unwrap();
        let claim = proposal
            .candidates
            .iter()
            .find_map(|c| {
                if let Candidate::Claim { claim } = c {
                    Some(claim)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(claim.id, old.id);
        assert_eq!(claim.supersedes, vec![old.revision_id.clone()]);
        assert!(
            proposal
                .candidates
                .iter()
                .filter_map(|c| if let Candidate::Connection { connection } = c {
                    Some(&connection.from)
                } else {
                    None
                })
                .all(|e| matches!(e,Endpoint::Claim{id} if id==&old.id||id==&revision.revision_id))
        );
        f.approve(&proposal).unwrap();
        let view = f.view(&t).unwrap();
        assert_eq!(view.current_claims.len(), 1);
        assert_eq!(view.current_claims[0].value, "new value");
        assert_eq!(view.history.len(), 2);
    }
}

#[test]
fn r5_source_label_cannot_release_topic_taint_and_preview_keeps_eligible_selection() {
    let f = F::new();
    f.write("M/a.md", "alpha\n");
    f.write("M/b.md", "beta\n");
    let a = f.source("M/a.md");
    let b = f.summary("M/b.md");
    f.label(&a.summary, Sensitivity::CloudAllowed);
    f.label(&b, Sensitivity::CloudAllowed);
    let t = topic("Private topic");
    f.accept(vec![tc(&t), cc(&f.claim(&t, &a.citation, "private claim"))]);
    f.label_topic(&t, Sensitivity::LocalOnly);
    // Even a fresh source decision cannot declassify the private topic.
    f.label(&a.summary, Sensitivity::CloudAllowed);
    let preview = f.preview(vec!["M/a.md", "M/b.md"]);
    assert_eq!(preview.sources.len(), 1);
    assert_eq!(preview.sources[0].path, "M/b.md");
    assert!(preview.excluded.iter().any(|e| e.path == "M/a.md"));
    let run = f
        .service
        .analysis_start(AnalysisStartRequest {
            root_id: f.root_id.clone(),
            scope_id: preview.scope_id.clone(),
            confirm_unknown: false,
        })
        .unwrap();
    let read = f
        .service
        .scope_read(ScopeReadRequest {
            root_id: f.root_id.clone(),
            scope_id: preview.scope_id,
            run_id: run.run_id,
        })
        .unwrap();
    assert_eq!(read.sources.len(), 1);
    assert_eq!(read.sources[0].content, "beta\n");
    f.label_topic(&t, Sensitivity::CloudAllowed);
    let preview = f.preview(vec!["M/a.md", "M/b.md"]);
    assert_eq!(preview.sources.len(), 2);
}
