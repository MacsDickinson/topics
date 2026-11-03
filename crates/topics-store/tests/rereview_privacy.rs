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
#[test]
fn n09_name_variant_must_not_bypass_rule() {
    let f = F::new();
    let (t, c, s) = alex_flow(&f);
    for name in ["alex", "Alex ", "ALEX"] {
        let mut t2 = topic(name);
        t2.topic_kind = t.topic_kind;
        let mut c2 = f.claim(&t2, &s, "he/him");
        c2.predicate = c.predicate.clone();
        let q = f
            .model_stage(vec!["M/a.md"], vec![tc(&t2), cc(&c2)])
            .unwrap();
        let kept = q
            .candidates
            .iter()
            .any(|x| matches!(x, Candidate::Claim { claim } if claim.value == "he/him"));
        eprintln!("OBS name={name:?} false_claim_kept={kept}");
        assert!(!kept, "name variant {name:?}");
    }
}
#[test]
fn n10_kind_variant_must_not_bypass_rule() {
    let f = F::new();
    let (_t, _c, s) = alex_flow(&f);
    let mut t2 = topic("Alex");
    t2.topic_kind = TopicKind::Thread;
    let c2 = f.claim(&t2, &s, "he/him");
    let q = f
        .model_stage(vec!["M/a.md"], vec![tc(&t2), cc(&c2)])
        .unwrap();
    let kept = q
        .candidates
        .iter()
        .any(|x| matches!(x, Candidate::Claim { .. }));
    eprintln!("OBS kept={kept}");
    assert!(!kept);
}
#[test]
fn n20_model_validation_must_not_reveal_local_only_topic_name() {
    let f = F::new();
    f.write("M/a.md", "a mention\n");
    let s = f.source("M/a.md");
    let mut secret = topic("SecretName");
    secret.sensitivity = Sensitivity::LocalOnly;
    f.accept(vec![tc(&secret)]);
    let p = f.preview(vec!["M/a.md"]);
    let run_id = f.run(&p);
    let probe = |name: &str| {
        let t = topic(name);
        let c = f.claim(&t, &s.citation, "x");
        f.service.validate_proposal(StageProposalRequest {
            root_id: f.root_id.clone(),
            scope_id: Some(p.scope_id.clone()),
            origin: Origin::Analysis {
                run_id: run_id.clone(),
            },
            candidates: vec![tc(&t), cc(&c)],
        })
    };
    let (hit, miss) = (probe("SecretName"), probe("Nobody"));
    eprintln!("OBS hit={} miss={}", code(&hit), code(&miss));
    assert_eq!(
        code(&hit),
        code(&miss),
        "tool result distinguishes a local-only topic name"
    );
}
#[test]
fn n21_local_only_name_collision_must_not_fail_whole_analysis() {
    let f = F::new();
    f.write("M/a.md", "joe and ledger\n");
    let s = f.source("M/a.md");
    let mut joe = topic("Joe");
    joe.sensitivity = Sensitivity::LocalOnly;
    f.accept(vec![tc(&joe)]);
    let (t, l) = (topic("Joe"), topic("Ledger"));
    let c = f.claim(&t, &s.citation, "x");
    let d = f.claim(&l, &s.citation, "y");
    let r = f.model_stage(vec!["M/a.md"], vec![tc(&t), cc(&c), tc(&l), cc(&d)]);
    eprintln!("OBS {}", code(&r));
    assert_eq!(code(&r), "ok");
}
#[test]
fn n22_redaction_must_not_export_transcript_value_after_uncertainty_only_edit() {
    let f = F::new();
    f.write("M/secret.md", "acquisition target zed\n");
    f.write("M/ok.md", "ok\n");
    let s = f.source("M/secret.md");
    f.source("M/ok.md");
    let t = topic("Deal");
    f.accept(vec![tc(&t)]);
    let c = f.claim(&t, &s.citation, "acquisition target is Zed");
    let p = f.stage(vec![cc(&c)]).unwrap();
    let mut e = c.clone();
    e.uncertainty = Some("low".into());
    f.decide(
        &p,
        Decision::Approve {
            candidates: vec![cc(&e)],
        },
    )
    .unwrap();
    let pv = f.preview(vec!["M/ok.md"]);
    let run_id = f.run(&pv);
    let v = f
        .service
        .source_context(SourceContextRequest {
            root_id: f.root_id.clone(),
            scope_id: pv.scope_id,
            run_id,
            topic_id: t.id.clone(),
        })
        .unwrap();
    let j = serde_json::to_string(&v).unwrap();
    eprintln!(
        "OBS contains_value={} json={}",
        j.contains("Zed"),
        &j[..j.len().min(500)]
    );
    assert!(
        !j.contains("Zed"),
        "unselected transcript-derived value exported as human-supported"
    );
}
#[test]
fn n23_redacted_view_excludes_metadata_positive() {
    let f = F::new();
    let (t, _c, _s) = alex_flow(&f);
    f.write("M/ok.md", "ok\n");
    f.source("M/ok.md");
    let pv = f.preview(vec!["M/ok.md"]);
    let run_id = f.run(&pv);
    let v = f
        .service
        .source_context(SourceContextRequest {
            root_id: f.root_id.clone(),
            scope_id: pv.scope_id,
            run_id,
            topic_id: t.id.clone(),
        })
        .unwrap();
    let j = serde_json::to_string(&v).unwrap();
    eprintln!("OBS {j}");
    assert_eq!(v.current_claims[0].value, "they/them");
    assert!(v.history.is_empty());
    assert!(
        !j.contains("M/a.md")
            && !j.contains("he/him")
            && !j.contains("correction")
            && !j.contains("document_hash")
    );
}
#[test]
fn n24_connection_source_endpoint_must_not_export_unselected_locator() {
    let f = F::new();
    f.write("M/ok.md", "ok\n");
    f.write("M/secret.md", "# Secret Merger Heading\nbody text\n");
    let ok = f.source("M/ok.md");
    let sec = f.source("M/secret.md");
    let (a, b) = (topic("A"), topic("B"));
    f.accept(vec![tc(&a), tc(&b)]);
    let mut n = f.conn(&a, &b, &ok.citation);
    n.to = Endpoint::Source {
        source_id: sec.summary.source_id.clone(),
        locator: Locator {
            start_byte: 24,
            end_byte: 28,
            heading: Some("Secret Merger Heading".into()),
        },
    };
    f.accept(vec![Candidate::Connection { connection: n }]);
    let pv = f.preview(vec!["M/ok.md"]);
    let run_id = f.run(&pv);
    let v = f
        .service
        .source_context(SourceContextRequest {
            root_id: f.root_id.clone(),
            scope_id: pv.scope_id,
            run_id,
            topic_id: a.id.clone(),
        })
        .unwrap();
    let j = serde_json::to_string(&v.connections).unwrap();
    eprintln!(
        "OBS has_heading={} has_id={}",
        j.contains("Secret Merger"),
        j.contains(&sec.summary.source_id)
    );
    assert!(!j.contains("Secret Merger") && !j.contains(&sec.summary.source_id));
}
#[test]
fn n25_local_only_moved_over_reviewed_path_must_stay_restricted() {
    let f = F::new();
    f.write("M/p.md", "secret\n");
    let p = f.summary("M/p.md");
    f.label(&p, Sensitivity::LocalOnly);
    f.write("M/q.md", "public\n");
    let q = f.summary("M/q.md");
    f.label(&q, Sensitivity::CloudAllowed);
    fs::rename(f.root.join("M/p.md"), f.root.join("M/q.md")).unwrap();
    let v = f.preview(vec!["M/q.md"]);
    eprintln!(
        "OBS {:?}",
        v.sources.iter().map(|s| s.sensitivity).collect::<Vec<_>>()
    );
    assert!(
        v.sources.is_empty(),
        "identical local-only bytes treated CloudAllowed by destination review"
    );
}
#[test]
fn n35_correction_marker_and_app_data_loss_should_warn() {
    let f = F::new();
    let (t, _c, _s) = alex_flow(&f);
    let ops = f.service.history_list(f.req()).unwrap().operations;
    let last = ops.last().unwrap().id.clone();
    fs::remove_file(f.root.join(format!("topics/operations/{last}.md"))).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let mut svc = WorkspaceService::open(tmp.path().join("ad")).unwrap();
    let root_id = svc
        .root_set(RootSetRequest {
            path: f.root.to_str().unwrap().into(),
            create: false,
        })
        .unwrap()
        .root
        .root_id;
    let v = svc
        .topic_get(TopicGetRequest {
            root_id,
            topic_id: t.id.clone(),
        })
        .unwrap();
    eprintln!(
        "OBS current={:?} warnings={:?}",
        v.current_claims
            .iter()
            .map(|c| c.value.clone())
            .collect::<Vec<_>>(),
        v.warnings
    );
    assert!(
        !v.warnings.is_empty(),
        "newest marker plus app data lost: false claim current, orphan correction and rule files on disk, no warning"
    );
    assert!(
        v.current_claims.is_empty(),
        "orphan replacement is not accepted and prior false current assertion stays withheld"
    );
}
#[test]
fn n40_guard_denied_leaves_no_visibility_and_no_temp() {
    let f = F::new();
    f.write("M/a.md", "x\n");
    f.source("M/a.md");
    let t = topic("A");
    let p = f.preview(vec!["M/a.md"]);
    let run_id = f.run(&p);
    let part = f.partition().join("proposals");
    let mut prepared_seen = false;
    let r = f.service.stage_proposal_guarded(
        StageProposalRequest {
            root_id: f.root_id.clone(),
            scope_id: Some(p.scope_id.clone()),
            origin: Origin::Analysis {
                run_id: run_id.clone(),
            },
            candidates: vec![tc(&t)],
        },
        |_, _| {
            prepared_seen = fs::read_dir(&part).map(|d| d.count() > 0).unwrap_or(false);
            Err(ApiError::new("cancelled", "revoked"))
        },
    );
    eprintln!("OBS {} prepared_seen={prepared_seen}", code(&r));
    assert_eq!(code(&r), "cancelled");
    assert!(prepared_seen);
    assert_eq!(fs::read_dir(&part).unwrap().count(), 0);
    assert!(
        f.service
            .proposals_list(f.req())
            .unwrap()
            .proposals
            .is_empty()
    );
    let late = f.service.scope_read(ScopeReadRequest {
        root_id: f.root_id.clone(),
        scope_id: p.scope_id,
        run_id,
    });
    eprintln!("OBS run consumed: {}", code(&late));
    assert_eq!(code(&late), "forbidden");
}
#[test]
fn n41_guard_no_action_and_ok_is_internal_error() {
    let f = F::new();
    let t = topic("A");
    let r = f.service.stage_proposal_guarded(
        StageProposalRequest {
            root_id: f.root_id.clone(),
            scope_id: None,
            origin: Origin::Manual,
            candidates: vec![tc(&t)],
        },
        |_, _| Ok(()),
    );
    eprintln!("OBS {}", code(&r));
    assert_eq!(code(&r), "internal");
    assert!(
        f.service
            .proposals_list(f.req())
            .unwrap()
            .proposals
            .is_empty()
    );
}
#[test]
fn n43_no_change_has_no_operation_or_knowledge_positive() {
    let f = F::new();
    let (t, c, _s) = alex_flow(&f);
    let before = f.service.history_list(f.req()).unwrap().operations.len();
    let mut again = fresh(&c);
    again.uncertainty = Some("w".into());
    let q = f.model_stage(vec!["M/a.md"], vec![cc(&again)]).unwrap();
    eprintln!("OBS status={:?} n={}", q.status, q.candidates.len());
    assert_eq!(q.status, ProposalStatus::Accepted);
    assert!(q.candidates.is_empty());
    let d = f.approve(&q).unwrap();
    assert!(d.operation.is_none());
    assert_eq!(
        f.service.history_list(f.req()).unwrap().operations.len(),
        before
    );
    assert_eq!(f.view(&t).unwrap().current_claims[0].value, "they/them");
    let e = f.decide(&q, Decision::Reject);
    eprintln!("OBS reject after no-change={}", code(&e));
    assert_eq!(code(&e), "conflict");
}
#[test]
fn n50_duplicates_inside_one_model_proposal() {
    let f = F::new();
    f.write("M/a.md", "x\n");
    let s = f.source("M/a.md");
    let t = topic("A");
    f.accept(vec![tc(&t)]);
    let c = f.claim(&t, &s.citation, "x");
    let d = fresh(&c);
    let p = f.model_stage(vec!["M/a.md"], vec![cc(&c), cc(&d)]).unwrap();
    eprintln!("OBS candidates={}", p.candidates.len());
    let _ = f.approve(&p);
    let n = f.view(&t).unwrap().current_claims.len();
    eprintln!("OBS current={n}");
    assert_eq!(n, 1);
}

#[test]
fn model_validation_has_uniform_errors_and_never_publishes_private_ambiguity() {
    let f = F::new();
    f.write("M/a.md", "selected evidence\n");
    let s = f.source("M/a.md");
    let mut private = topic("Private Person");
    private.sensitivity = Sensitivity::LocalOnly;
    f.accept(vec![tc(&private)]);
    let scope = f.preview(vec!["M/a.md"]);
    let run_id = f.run(&scope);
    for malformed in [false, true] {
        let results: Vec<_> = ["Private Person", "Absent Person"]
            .into_iter()
            .map(|name| {
                let t = topic(name);
                let mut claim = f.claim(&t, &s.citation, "a fact");
                if malformed {
                    claim.supersedes.push(id());
                }
                f.service.validate_proposal(StageProposalRequest {
                    root_id: f.root_id.clone(),
                    scope_id: Some(scope.scope_id.clone()),
                    origin: Origin::Analysis {
                        run_id: run_id.clone(),
                    },
                    candidates: vec![tc(&t), cc(&claim)],
                })
            })
            .collect();
        assert_eq!(results[0], results[1]);
        let output = serde_json::to_string(&results[0]).unwrap();
        assert!(!output.contains(&private.id) && !output.contains(&private.name));
        assert_eq!(results[0].is_ok(), !malformed);
    }
    assert!(
        f.service
            .proposals_list(f.req())
            .unwrap()
            .proposals
            .iter()
            .all(|p| p.status == ProposalStatus::Accepted)
    );
    let proposed = topic("Private Person");
    let other = topic("Unrelated Ledger");
    let claim = f.claim(&other, &s.citation, "eligible");
    let proposal = f
        .service
        .stage_proposal(StageProposalRequest {
            root_id: f.root_id.clone(),
            scope_id: Some(scope.scope_id),
            origin: Origin::Analysis { run_id },
            candidates: vec![tc(&proposed), tc(&other), cc(&claim)],
        })
        .unwrap();
    assert_eq!(proposal.status, ProposalStatus::Pending);
    assert!(proposal.candidates.iter().any(|c| matches!(c,Candidate::UnresolvedIdentity{candidates,..} if candidates.contains(&private.id))));
    assert!(
        proposal
            .candidates
            .iter()
            .any(|c| matches!(c,Candidate::Claim{claim:c} if c.revision_id==claim.revision_id))
    );
    assert_eq!(f.service.topics_list(f.req()).unwrap().topics.len(), 1);
    let approved = proposal
        .candidates
        .iter()
        .filter(|c| match c {
            Candidate::Topic { topic } => topic.id == other.id,
            Candidate::Claim { claim: c } => c.subject_id == other.id,
            _ => false,
        })
        .cloned()
        .collect();
    f.decide(
        &proposal,
        Decision::Approve {
            candidates: approved,
        },
    )
    .unwrap();
    assert_eq!(f.view(&other).unwrap().current_claims.len(), 1);
    assert!(f.view(&private).unwrap().current_claims.is_empty());
}

#[test]
fn subject_only_edit_keeps_exact_human_evidence_but_does_not_declassify_value() {
    let f = F::new();
    f.write("M/private.md", "confidential target\n");
    f.write("M/selected.md", "selected\n");
    let source = f.source("M/private.md");
    let a = topic("Original");
    let b = topic("Edited Subject");
    f.accept(vec![tc(&a), tc(&b)]);
    let claim = f.claim(&a, &source.citation, "unselected secret value");
    let proposal = f.stage(vec![cc(&claim)]).unwrap();
    let mut edited = claim.clone();
    edited.subject_id = b.id.clone();
    f.decide(
        &proposal,
        Decision::Approve {
            candidates: vec![cc(&edited)],
        },
    )
    .unwrap();
    let human = f.view(&b).unwrap();
    let statement_id = human.current_claims[0]
        .sources
        .iter()
        .find_map(|e| {
            if let Evidence::UserStatement { statement_id } = e {
                Some(statement_id.clone())
            } else {
                None
            }
        })
        .unwrap();
    let statement = f
        .service
        .statement_get(StatementGetRequest {
            root_id: f.root_id.clone(),
            statement_id,
        })
        .unwrap();
    assert!(
        statement.text.contains(&a.id)
            && statement.text.contains(&b.id)
            && statement.text.contains(&claim.value)
    );
    let p = f.preview(vec!["M/selected.md"]);
    let run_id = f.run(&p);
    let context = f
        .service
        .source_context(SourceContextRequest {
            root_id: f.root_id.clone(),
            scope_id: p.scope_id,
            run_id,
            topic_id: b.id,
        })
        .unwrap();
    assert!(
        !serde_json::to_string(&context)
            .unwrap()
            .contains(&claim.value)
    );
}

#[test]
fn a_changed_value_has_durable_authority_after_app_data_rebuild() {
    let f = F::new();
    f.write("M/original.md", "original value\n");
    f.write("M/selected.md", "selected\n");
    let source = f.source("M/original.md");
    let t = topic("Subject");
    f.accept(vec![tc(&t)]);
    let original = f.claim(&t, &source.citation, "transcript value");
    let p = f.stage(vec![cc(&original)]).unwrap();
    let mut edited = original.clone();
    edited.value = "human changed value".into();
    f.decide(
        &p,
        Decision::Approve {
            candidates: vec![cc(&edited)],
        },
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
    let p = service
        .scope_preview(ScopePreviewRequest {
            root_id: root_id.clone(),
            spec: ScopeSpec {
                files: vec!["M/selected.md".into()],
                folders: vec![],
                date_from: None,
                date_to: None,
            },
        })
        .unwrap();
    let run_id = service
        .analysis_start(AnalysisStartRequest {
            root_id: root_id.clone(),
            scope_id: p.scope_id.clone(),
            confirm_unknown: true,
        })
        .unwrap()
        .run_id;
    let view = service
        .source_context(SourceContextRequest {
            root_id,
            scope_id: p.scope_id,
            run_id,
            topic_id: t.id,
        })
        .unwrap();
    assert_eq!(view.current_claims.len(), 1);
    assert_eq!(view.current_claims[0].value, edited.value);
    let json = serde_json::to_string(&view).unwrap();
    assert!(
        !json.contains("M/original.md")
            && !json.contains("transcript value")
            && !json.contains("User edited")
    );
}

#[test]
fn moved_private_bytes_need_a_fresh_hash_bound_human_review() {
    let f = F::new();
    f.write("M/private.md", "private bytes\n");
    let private = f.summary("M/private.md");
    f.label(&private, Sensitivity::LocalOnly);
    f.write("M/public.md", "public bytes\n");
    let public = f.summary("M/public.md");
    f.label(&public, Sensitivity::CloudAllowed);
    fs::rename(f.root.join("M/private.md"), f.root.join("M/public.md")).unwrap();
    assert!(f.preview(vec!["M/public.md"]).sources.is_empty());
    // Even a fresh human decision about the stale old destination ID is not
    // consent for the new bytes. The human must inspect their current mapping.
    f.label(&public, Sensitivity::CloudAllowed);
    assert!(f.preview(vec!["M/public.md"]).sources.is_empty());
    let moved = f.summary("M/public.md");
    assert_ne!(moved.hash, public.hash);
    assert_eq!(moved.sensitivity, Sensitivity::LocalOnly);
    f.label(&moved, Sensitivity::CloudAllowed);
    let preview = f.preview(vec!["M/public.md"]);
    assert_eq!(preview.sources.len(), 1);
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
    assert_eq!(read.sources[0].content, "private bytes\n");
}

#[test]
fn source_connection_endpoint_is_human_visible_and_selected_context_visible() {
    let f = F::new();
    f.write("M/one.md", "# One\ntext\n");
    f.write("M/two.md", "# Two\ntext\n");
    let one = f.source("M/one.md");
    let two = f.source("M/two.md");
    let a = topic("A");
    let b = topic("B");
    f.accept(vec![tc(&a), tc(&b)]);
    let mut connection = f.conn(&a, &b, &one.citation);
    connection.to = Endpoint::Source {
        source_id: two.summary.source_id.clone(),
        locator: Locator {
            start_byte: 6,
            end_byte: 10,
            heading: Some("Two".into()),
        },
    };
    f.accept(vec![Candidate::Connection {
        connection: connection.clone(),
    }]);
    assert_eq!(f.view(&a).unwrap().connections, vec![connection.clone()]);
    let p = f.preview(vec!["M/one.md", "M/two.md"]);
    let run_id = f.run(&p);
    let v = f
        .service
        .source_context(SourceContextRequest {
            root_id: f.root_id.clone(),
            scope_id: p.scope_id,
            run_id,
            topic_id: a.id,
        })
        .unwrap();
    assert_eq!(v.connections, vec![connection]);
}

#[test]
fn pending_semantic_dedupe_retains_evidence_and_remaps_claim_endpoints() {
    let f = F::new();
    f.write("M/one.md", "one\n");
    f.write("M/two.md", "two\n");
    let one = f.source("M/one.md");
    let two = f.source("M/two.md");
    let a = topic("A");
    let b = topic("B");
    f.accept(vec![tc(&a), tc(&b)]);
    let first = f.claim(&a, &one.citation, "same value");
    let mut duplicate = fresh(&first);
    duplicate.sources = vec![Evidence::Passage {
        source: two.citation.clone(),
    }];
    duplicate.uncertainty = Some("additional uncertainty".into());
    let mut c = f.conn(&a, &b, &one.citation);
    c.from = Endpoint::Claim {
        id: duplicate.revision_id.clone(),
    };
    let mut d = c.clone();
    d.id = id();
    d.from = Endpoint::Claim {
        id: first.id.clone(),
    };
    d.sources = vec![two.citation.clone()];
    let proposal = f
        .model_stage(
            vec!["M/one.md", "M/two.md"],
            vec![
                cc(&first),
                cc(&duplicate),
                Candidate::Connection { connection: c },
                Candidate::Connection { connection: d },
            ],
        )
        .unwrap();
    assert_eq!(proposal.candidates.len(), 2);
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
    assert_eq!(claim.sources.len(), 2);
    assert!(
        claim
            .uncertainty
            .as_ref()
            .unwrap()
            .contains("additional uncertainty")
    );
    let connection = proposal
        .candidates
        .iter()
        .find_map(|c| {
            if let Candidate::Connection { connection } = c {
                Some(connection)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(connection.sources.len(), 2);
    assert!(
        matches!(&connection.from,Endpoint::Claim{id} if id==&first.id || id==&first.revision_id)
    );
    f.approve(&proposal).unwrap();
    assert_eq!(f.view(&a).unwrap().current_claims.len(), 1);
}

#[test]
fn topic_only_repeat_is_pending_ambiguity_and_nested_topics_are_source_notes() {
    let f = F::new();
    f.write("Projects/topics/meeting.md", "# Nested source\n");
    let inventory = f.service.sources_list(f.req()).unwrap();
    assert!(
        inventory
            .sources
            .iter()
            .any(|s| s.path == "Projects/topics/meeting.md")
    );
    assert!(
        inventory
            .sources
            .iter()
            .all(|s| !s.path.starts_with("topics/"))
    );
    let folder = f.preview_spec(vec![], vec!["Projects"]).unwrap();
    assert_eq!(folder.sources.len(), 1);
    let old = topic("Joe");
    f.accept(vec![tc(&old)]);
    let new = topic("Joe");
    let p = f
        .model_stage(vec!["Projects/topics/meeting.md"], vec![tc(&new)])
        .unwrap();
    assert_eq!(p.status, ProposalStatus::Pending);
    assert!(p.candidates.iter().any(|c|matches!(c,Candidate::UnresolvedIdentity{candidates,..} if candidates.contains(&old.id)&&candidates.contains(&new.id))));
    assert_eq!(f.service.topics_list(f.req()).unwrap().topics.len(), 1);
    f.approve(&p).unwrap();
    assert_eq!(f.service.topics_list(f.req()).unwrap().topics.len(), 2);
}

#[test]
fn n01_prestaged_provisional_duplicate_conflicts() {
    let f = F::new();
    f.write("M/a.md", "fact\n");
    let s = f.source("M/a.md");
    let (t1, t2) = (topic("Alex"), topic("Alex"));
    let c1 = f.claim(&t1, &s.citation, "x");
    let mut c2 = f.claim(&t2, &s.citation, "x");
    c2.uncertainty = Some("reworded".into());
    let p1 = f
        .model_stage(vec!["M/a.md"], vec![tc(&t1), cc(&c1)])
        .unwrap();
    let p2 = f
        .model_stage(vec!["M/a.md"], vec![tc(&t2), cc(&c2)])
        .unwrap();
    assert_ne!(p1.id, p2.id);
    f.approve(&p1).unwrap();
    let r = f.approve(&p2);
    eprintln!(
        "OBS second={} topics={} ",
        code(&r),
        f.service.topics_list(f.req()).unwrap().topics.len()
    );
    assert_eq!(code(&r), "conflict");
    assert_eq!(f.service.topics_list(f.req()).unwrap().topics.len(), 1);
    assert_eq!(f.view(&t1).unwrap().current_claims.len(), 1);
    assert_eq!(
        f.service
            .proposals_list(f.req())
            .unwrap()
            .proposals
            .iter()
            .find(|x| x.id == p2.id)
            .unwrap()
            .status,
        ProposalStatus::Conflict
    );
    let q = f
        .model_stage(vec!["M/a.md"], vec![tc(&t2), cc(&c2)])
        .unwrap();
    eprintln!("OBS refresh status={:?} n={}", q.status, q.candidates.len());
    assert!(q.candidates.is_empty());
}

#[test]
fn n02_prestaged_reused_id_duplicate_conflicts() {
    let f = F::new();
    f.write("M/a.md", "fact\n");
    let s = f.source("M/a.md");
    let t = topic("A");
    f.accept(vec![tc(&t)]);
    let c1 = f.claim(&t, &s.citation, "x");
    let mut c2 = fresh(&c1);
    c2.uncertainty = Some("reworded".into());
    let p1 = f.model_stage(vec!["M/a.md"], vec![cc(&c1)]).unwrap();
    let p2 = f.model_stage(vec!["M/a.md"], vec![cc(&c2)]).unwrap();
    f.approve(&p1).unwrap();
    let r = f.approve(&p2);
    eprintln!("OBS {}", code(&r));
    assert_eq!(code(&r), "conflict");
    assert_eq!(f.view(&t).unwrap().current_claims.len(), 1);
}

#[test]
fn n03_prestaged_connection_duplicate_conflicts() {
    let f = F::new();
    f.write("M/a.md", "fact\n");
    let s = f.source("M/a.md");
    let (a, b) = (topic("A"), topic("B"));
    f.accept(vec![tc(&a), tc(&b)]);
    let c1 = f.conn(&a, &b, &s.citation);
    let mut c2 = f.conn(&a, &b, &s.citation);
    c2.uncertainty = Some("reworded".into());
    let p1 = f
        .model_stage(
            vec!["M/a.md"],
            vec![Candidate::Connection { connection: c1 }],
        )
        .unwrap();
    let p2 = f
        .model_stage(
            vec!["M/a.md"],
            vec![Candidate::Connection { connection: c2 }],
        )
        .unwrap();
    f.approve(&p1).unwrap();
    let r = f.approve(&p2);
    eprintln!("OBS {}", code(&r));
    assert_eq!(code(&r), "conflict");
    assert_eq!(f.view(&a).unwrap().connections.len(), 1);
}

#[test]
fn n04_prestaged_provisional_cannot_bypass_later_correction() {
    let f = F::new();
    f.write("M/a.md", "alex he/him\n");
    let s = f.source("M/a.md");
    let (t1, t2) = (topic("Alex"), topic("Alex"));
    let c1 = f.claim(&t1, &s.citation, "he/him");
    let mut c2 = f.claim(&t2, &s.citation, "he/him");
    c2.uncertainty = Some("reworded".into());
    let p1 = f
        .model_stage(vec!["M/a.md"], vec![tc(&t1), cc(&c1)])
        .unwrap();
    let p2 = f
        .model_stage(vec!["M/a.md"], vec![tc(&t2), cc(&c2)])
        .unwrap();
    f.approve(&p1).unwrap();
    human_cloud(&f, &t1);
    let (n, r) = f.correct(&t1, &c1, "they/them");
    f.accept(vec![Candidate::Correction { claim: n, rule: r }]);
    let d = f.approve(&p2);
    let tops = f.service.topics_list(f.req()).unwrap().topics.len();
    eprintln!("OBS {} topics={tops}", code(&d));
    assert_eq!(code(&d), "conflict");
    assert_eq!(tops, 1);
    assert_eq!(f.view(&t1).unwrap().current_claims[0].value, "they/them");
    // reused-subject variant staged before the rule exists
}

#[test]
fn n05_prestaged_reused_subject_cannot_bypass_later_rule() {
    let f = F::new();
    f.write("M/a.md", "alex he/him\n");
    let s = f.source("M/a.md");
    let t = topic("Alex");
    let c = f.claim(&t, &s.citation, "he/him");
    f.accept(vec![tc(&t), cc(&c)]);
    f.write("M/b.md", "alex he/him again\n");
    let b = f.source("M/b.md");
    let mut c2 = fresh(&c);
    c2.sources = vec![Evidence::Passage {
        source: b.citation.clone(),
    }];
    c2.uncertainty = Some("w".into());
    // Different logical assertion text only by evidence: same_claim would dedupe it, so use a distinct date to keep it pending.
    c2.valid_from = Some("2026-01-01".into());
    let p = f.model_stage(vec!["M/b.md"], vec![cc(&c2)]).unwrap();
    assert_eq!(p.status, ProposalStatus::Pending);
    let (n, r) = f.correct(&t, &c, "they/them");
    f.accept(vec![Candidate::Correction { claim: n, rule: r }]);
    let d = f.approve(&p);
    eprintln!("OBS {}", code(&d));
    assert_eq!(code(&d), "conflict");
}

#[test]
fn n06_prestaged_subset_cannot_silently_create_duplicate_identity_with_facts() {
    let f = F::new();
    f.write("M/a.md", "alex fact one\n");
    let s = f.source("M/a.md");
    let (t1, t2) = (topic("Alex"), topic("Alex"));
    let c1 = f.claim(&t1, &s.citation, "one");
    let mut c2 = f.claim(&t2, &s.citation, "one");
    c2.uncertainty = Some("w".into());
    let mut extra = f.claim(&t2, &s.citation, "two");
    extra.predicate = "team".into();
    let p1 = f
        .model_stage(vec!["M/a.md"], vec![tc(&t1), cc(&c1)])
        .unwrap();
    let p2 = f
        .model_stage(vec!["M/a.md"], vec![tc(&t2), cc(&c2), cc(&extra)])
        .unwrap();
    f.approve(&p1).unwrap();
    let r = f.decide(
        &p2,
        Decision::Approve {
            candidates: vec![p2.candidates[0].clone(), cc(&extra)],
        },
    );
    eprintln!(
        "OBS subset={} topics={}",
        code(&r),
        f.service.topics_list(f.req()).unwrap().topics.len()
    );
    assert_eq!(code(&r), "conflict");
    assert_eq!(f.service.topics_list(f.req()).unwrap().topics.len(), 1);
}

#[test]
fn n07_unrelated_prestaged_proposals_do_not_false_conflict() {
    let f = F::new();
    f.write("M/a.md", "fact\n");
    let s = f.source("M/a.md");
    let t = topic("A");
    f.accept(vec![tc(&t)]);
    let c1 = f.claim(&t, &s.citation, "one");
    let mut c2 = f.claim(&t, &s.citation, "two");
    c2.predicate = "team".into();
    let p1 = f.model_stage(vec!["M/a.md"], vec![cc(&c1)]).unwrap();
    let p2 = f.model_stage(vec!["M/a.md"], vec![cc(&c2)]).unwrap();
    f.approve(&p1).unwrap();
    let r = f.approve(&p2);
    eprintln!("OBS {}", code(&r));
    assert_eq!(code(&r), "ok");
}

#[test]
fn n08_genuine_second_joe_is_not_merged_and_ambiguity_is_explicit() {
    let f = F::new();
    f.write("M/a.md", "joe one\n");
    f.write("M/b.md", "joe two\n");
    let (a, b) = (f.source("M/a.md"), f.source("M/b.md"));
    let j1 = topic("Joe");
    let c1 = f.claim(&j1, &a.citation, "lead");
    let p = f
        .model_stage(vec!["M/a.md"], vec![tc(&j1), cc(&c1)])
        .unwrap();
    f.approve(&p).unwrap();
    let j2 = topic("Joe");
    let c2 = f.claim(&j2, &b.citation, "intern");
    let q = f
        .model_stage(vec!["M/b.md"], vec![tc(&j2), cc(&c2)])
        .unwrap();
    let un = q.candidates.iter().find_map(|c| {
        if let Candidate::UnresolvedIdentity { candidates, .. } = c {
            Some(candidates.clone())
        } else {
            None
        }
    });
    eprintln!(
        "OBS status={:?} kinds={} unresolved={:?}",
        q.status,
        q.candidates.len(),
        un.as_ref().map(|u| u.len())
    );
    assert!(un.is_some());
    f.approve(&q).unwrap();
    let joes = f
        .service
        .topics_list(f.req())
        .unwrap()
        .topics
        .iter()
        .filter(|t| t.name == "Joe")
        .count();
    eprintln!(
        "OBS joes={joes} j1_claims={}",
        f.view(&j1).unwrap().current_claims.len()
    );
    assert_eq!(f.view(&j1).unwrap().current_claims.len(), 1);
    assert_eq!(joes, 2);
}

#[test]
fn n37_interrupted_correction_conflict_no_resurrection_positive() {
    let f = F::new();
    f.write("M/a.md", "alex he/him\n");
    let s = f.source("M/a.md");
    let t = topic("Alex");
    let c = f.claim(&t, &s.citation, "he/him");
    f.accept(vec![tc(&t), cc(&c)]);
    let (n, r) = f.correct(&t, &c, "they/them");
    let rule_path = f.root.join(format!("topics/rules/{}.md", r.id));
    let p = f
        .stage(vec![Candidate::Correction {
            claim: n.clone(),
            rule: r,
        }])
        .unwrap();
    f.service.interrupt_next_operation_after(1);
    assert!(f.approve(&p).is_err());
    fs::write(&rule_path, b"external\n").unwrap();
    let v = f.view(&t).unwrap();
    eprintln!(
        "OBS current={:?}",
        v.current_claims
            .iter()
            .map(|c| c.value.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(v.current_claims.len(), 1);
    assert_eq!(v.current_claims[0].value, "he/him");
    assert_eq!(fs::read(&rule_path).unwrap(), b"external\n");
    assert_eq!(
        f.service
            .proposals_list(f.req())
            .unwrap()
            .proposals
            .iter()
            .find(|x| x.id == p.id)
            .unwrap()
            .status,
        ProposalStatus::Conflict
    );
}

#[test]
fn maximum_raw_batch_private_name_does_not_change_tool_result_or_guard_visibility() {
    let f = F::new();
    f.write("M/a.md", "selected evidence\n");
    let private = Topic {
        sensitivity: Sensitivity::LocalOnly,
        ..topic("Private Name")
    };
    f.accept(vec![tc(&private)]);
    let p = f.preview(vec!["M/a.md"]);
    let run_id = f.run(&p);
    let mut candidates: Vec<_> = (0..200).map(|n| tc(&topic(&format!("Name {n}")))).collect();
    let request = |candidates| StageProposalRequest {
        root_id: f.root_id.clone(),
        scope_id: Some(p.scope_id.clone()),
        origin: Origin::Analysis {
            run_id: run_id.clone(),
        },
        candidates,
    };
    assert!(
        f.service
            .validate_proposal(request(candidates.clone()))
            .is_ok()
    );
    candidates[0] = tc(&topic(&private.name));
    assert!(
        f.service
            .validate_proposal(request(candidates.clone()))
            .is_ok()
    );
    let before = f.service.proposals_list(f.req()).unwrap();
    let result = f
        .service
        .stage_proposal_guarded(request(candidates), |_, _| {
            Err(ApiError::new("cancelled", "revoked"))
        });
    assert_eq!(result.unwrap_err().code, "cancelled");
    assert_eq!(f.service.proposals_list(f.req()).unwrap(), before);
    assert_eq!(
        f.service.topics_list(f.req()).unwrap().topics,
        vec![private]
    );
}

#[test]
fn orphan_rule_alone_quarantines_false_head_without_accepting_replacement() {
    let f = F::new();
    let (t, _, _) = alex_flow(&f);
    let view = f.view(&t).unwrap();
    let corrected = &view.current_claims[0];
    let operations = f.service.history_list(f.req()).unwrap().operations;
    let correction_op = operations.last().unwrap();
    fs::remove_file(
        f.root
            .join(format!("topics/operations/{}.md", correction_op.id)),
    )
    .unwrap();
    fs::remove_file(
        f.root
            .join(format!("topics/claims/{}.md", corrected.revision_id)),
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
    let view = service
        .topic_get(TopicGetRequest {
            root_id,
            topic_id: t.id,
        })
        .unwrap();
    assert!(!view.warnings.is_empty());
    assert!(view.current_claims.is_empty());
    assert!(view.history.iter().any(|c| c.value == "he/him"));
    assert!(view.history.iter().all(|c| c.value != "they/them"));
}
