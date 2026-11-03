//! Reviewer scratch probes. Synthetic temp roots and absolute app data only. Each test asserts the
//! CORRECT behaviour, so a failure confirms a defect.
use std::{fs, path::PathBuf};
use topics_core::*;
use topics_store::WorkspaceService;
fn id() -> Id { uuid::Uuid::new_v4().to_string() }
fn topic(name: &str) -> Topic { Topic { id: id(), name: name.into(), topic_kind: TopicKind::Person, aliases: vec![], sensitivity: Sensitivity::CloudAllowed } }
struct F { temp: tempfile::TempDir, root: PathBuf, root_id: Id, service: WorkspaceService }
impl F {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("notes");
        let mut service = WorkspaceService::open(temp.path().join("app-data")).unwrap();
        let root_id = service.root_set(RootSetRequest { path: root.to_str().unwrap().into(), create: true }).unwrap().root.root_id;
        Self { temp, root, root_id, service }
    }
    fn req(&self) -> RootRequest { RootRequest { root_id: self.root_id.clone() } }
    fn write(&self, path: &str, bytes: impl AsRef<[u8]>) { let p = self.root.join(path); fs::create_dir_all(p.parent().unwrap()).unwrap(); fs::write(p, bytes).unwrap(); }
    fn summary(&self, path: &str) -> SourceSummary { self.service.sources_list(self.req()).unwrap().sources.into_iter().find(|s| s.path == path).unwrap() }
    fn source(&self, path: &str) -> ScopedSource { let s = self.summary(path); self.service.source_get(SourceGetRequest { root_id: self.root_id.clone(), source_id: s.source_id }).unwrap() }
    fn preview(&self, files: Vec<&str>) -> ScopePreview { self.preview_spec(files, vec![]).unwrap() }
    fn preview_spec(&self, files: Vec<&str>, folders: Vec<&str>) -> WorkspaceResult<ScopePreview> {
        self.service.scope_preview(ScopePreviewRequest { root_id: self.root_id.clone(), spec: ScopeSpec { files: files.into_iter().map(str::to_string).collect(), folders: folders.into_iter().map(str::to_string).collect(), date_from: None, date_to: None } })
    }
    fn run(&self, p: &ScopePreview) -> Id { self.service.analysis_start(AnalysisStartRequest { root_id: self.root_id.clone(), scope_id: p.scope_id.clone(), confirm_unknown: true }).unwrap().run_id }
    fn model_stage(&self, files: Vec<&str>, candidates: Vec<Candidate>) -> WorkspaceResult<Proposal> {
        let p = self.preview(files); let run_id = self.run(&p);
        self.service.stage_proposal(StageProposalRequest { root_id: self.root_id.clone(), scope_id: Some(p.scope_id), origin: Origin::Analysis { run_id }, candidates })
    }
    fn stage(&self, candidates: Vec<Candidate>) -> WorkspaceResult<Proposal> { self.service.manual_propose(ManualProposeRequest { root_id: self.root_id.clone(), candidates }).map(|p| p.proposal) }
    fn decide(&self, p: &Proposal, decision: Decision) -> WorkspaceResult<ProposalDecideResult> {
        self.service.proposal_decide(ProposalDecideRequest { root_id: self.root_id.clone(), proposal_id: p.id.clone(), expected_fingerprint: p.fingerprint.clone(), decision })
    }
    fn approve(&self, p: &Proposal) -> WorkspaceResult<ProposalDecideResult> { self.decide(p, Decision::Approve { candidates: p.candidates.clone() }) }
    fn accept(&self, c: Vec<Candidate>) -> ProposalDecideResult { let p = self.stage(c).unwrap(); self.approve(&p).unwrap() }
    fn label(&self, s: &SourceSummary, l: Sensitivity) -> Operation {
        self.service.sensitivity_set(SensitivitySetRequest { root_id: self.root_id.clone(), target: Endpoint::Source { source_id: s.source_id.clone(), locator: Locator { start_byte: 0, end_byte: 0, heading: None } }, sensitivity: l }).unwrap().operation
    }
    fn label_topic(&self, t: &Topic, l: Sensitivity) -> Operation {
        self.service.sensitivity_set(SensitivitySetRequest { root_id: self.root_id.clone(), target: Endpoint::Topic { id: t.id.clone() }, sensitivity: l }).unwrap().operation
    }
    fn undo(&self, op: &Id) -> WorkspaceResult<OperationResult> { self.service.undo(UndoRequest { root_id: self.root_id.clone(), operation_id: op.clone() }) }
    fn view(&self, t: &Topic) -> WorkspaceResult<TopicView> { self.service.topic_get(TopicGetRequest { root_id: self.root_id.clone(), topic_id: t.id.clone() }) }
    fn partition(&self) -> PathBuf { fs::read_dir(self.temp.path().join("app-data/workspaces")).unwrap().next().unwrap().unwrap().path() }
    fn claim(&self, t: &Topic, s: &SourceRef, v: &str) -> Claim {
        Claim { id: id(), revision_id: id(), subject_id: t.id.clone(), predicate: "role".into(), value: v.into(), uncertainty: Some("synthetic".into()), sources: vec![Evidence::Passage { source: s.clone() }], supersedes: vec![], recorded_at: "2026-11-03T00:00:00Z".into(), valid_from: None, valid_to: None }
    }
    fn conn(&self, a: &Topic, b: &Topic, s: &SourceRef) -> Connection {
        Connection { id: id(), from: Endpoint::Topic { id: a.id.clone() }, relation: "leads".into(), to: Endpoint::Topic { id: b.id.clone() }, uncertainty: None, sources: vec![s.clone()], valid_from: None, valid_to: None }
    }
    fn correct(&self, t: &Topic, old: &Claim, new_value: &str) -> (Claim, Rule) {
        let st = self.service.user_statement(UserStatementRequest { root_id: self.root_id.clone(), topic_id: t.id.clone(), text: "correction".into() }).unwrap();
        let mut c = old.clone(); c.revision_id = id(); c.value = new_value.into(); c.supersedes = vec![old.revision_id.clone()]; c.sources = vec![Evidence::UserStatement { statement_id: st.id }];
        let r = Rule { id: id(), topic_id: t.id.clone(), predicate: old.predicate.clone(), match_value: old.value.clone(), replacement: new_value.into(), scope: RuleScope { source_paths: vec![], context: None }, source_revision_id: c.revision_id.clone() };
        (c, r)
    }
}
fn tc(t: &Topic) -> Candidate { Candidate::Topic { topic: t.clone() } }
fn cc(c: &Claim) -> Candidate { Candidate::Claim { claim: c.clone() } }

fn fresh(c: &Claim) -> Claim { let mut n = c.clone(); n.id = id(); n.revision_id = id(); n }
fn code<T>(r: &WorkspaceResult<T>) -> String { match r { Ok(_) => "ok".into(), Err(e) => e.code.clone() } }
fn human_cloud(f: &F, t: &Topic) { f.label_topic(t, Sensitivity::CloudAllowed); }
fn alex_flow(f: &F) -> (Topic, Claim, SourceRef) {
    f.write("M/a.md", "alex he/him\n"); let s = f.source("M/a.md"); let t = topic("Alex"); let c = f.claim(&t, &s.citation, "he/him");
    let p = f.model_stage(vec!["M/a.md"], vec![tc(&t), cc(&c)]).unwrap(); f.approve(&p).unwrap(); human_cloud(f, &t);
    let (n, r) = f.correct(&t, &c, "they/them"); f.accept(vec![Candidate::Correction { claim: n, rule: r }]);
    (t, c, s.citation)
}
#[test] fn n09_name_variant_must_not_bypass_rule() {
    let f = F::new(); let (t, c, s) = alex_flow(&f);
    for name in ["alex", "Alex ", "ALEX"] {
        let mut t2 = topic(name); t2.topic_kind = t.topic_kind; let mut c2 = f.claim(&t2, &s, "he/him"); c2.predicate = c.predicate.clone();
        let q = f.model_stage(vec!["M/a.md"], vec![tc(&t2), cc(&c2)]).unwrap();
        let kept = q.candidates.iter().any(|x| matches!(x, Candidate::Claim { claim } if claim.value == "he/him")); eprintln!("OBS name={name:?} false_claim_kept={kept}");
        assert!(!kept, "name variant {name:?}");
    }
}
#[test] fn n10_kind_variant_must_not_bypass_rule() {
    let f = F::new(); let (_t, _c, s) = alex_flow(&f); let mut t2 = topic("Alex"); t2.topic_kind = TopicKind::Thread; let c2 = f.claim(&t2, &s, "he/him");
    let q = f.model_stage(vec!["M/a.md"], vec![tc(&t2), cc(&c2)]).unwrap(); let kept = q.candidates.iter().any(|x| matches!(x, Candidate::Claim { .. })); eprintln!("OBS kept={kept}"); assert!(!kept);
}
#[test] fn n20_model_validation_must_not_reveal_local_only_topic_name() {
    let f = F::new(); f.write("M/a.md", "a mention\n"); let s = f.source("M/a.md"); let mut secret = topic("SecretName"); secret.sensitivity = Sensitivity::LocalOnly; f.accept(vec![tc(&secret)]);
    let p = f.preview(vec!["M/a.md"]); let run_id = f.run(&p);
    let mut probe = |name: &str| { let t = topic(name); let c = f.claim(&t, &s.citation, "x"); f.service.validate_proposal(StageProposalRequest { root_id: f.root_id.clone(), scope_id: Some(p.scope_id.clone()), origin: Origin::Analysis { run_id: run_id.clone() }, candidates: vec![tc(&t), cc(&c)] }) };
    let (hit, miss) = (probe("SecretName"), probe("Nobody")); eprintln!("OBS hit={} miss={}", code(&hit), code(&miss));
    assert_eq!(code(&hit), code(&miss), "tool result distinguishes a local-only topic name");
}
#[test] fn n21_local_only_name_collision_must_not_fail_whole_analysis() {
    let f = F::new(); f.write("M/a.md", "joe and ledger\n"); let s = f.source("M/a.md"); let mut joe = topic("Joe"); joe.sensitivity = Sensitivity::LocalOnly; f.accept(vec![tc(&joe)]);
    let (t, l) = (topic("Joe"), topic("Ledger")); let c = f.claim(&t, &s.citation, "x"); let d = f.claim(&l, &s.citation, "y");
    let r = f.model_stage(vec!["M/a.md"], vec![tc(&t), cc(&c), tc(&l), cc(&d)]); eprintln!("OBS {}", code(&r)); assert_eq!(code(&r), "ok");
}
#[test] fn n22_redaction_must_not_export_transcript_value_after_uncertainty_only_edit() {
    let f = F::new(); f.write("M/secret.md", "acquisition target zed\n"); f.write("M/ok.md", "ok\n"); let s = f.source("M/secret.md"); f.source("M/ok.md"); let t = topic("Deal"); f.accept(vec![tc(&t)]);
    let c = f.claim(&t, &s.citation, "acquisition target is Zed"); let p = f.stage(vec![cc(&c)]).unwrap(); let mut e = c.clone(); e.uncertainty = Some("low".into());
    f.decide(&p, Decision::Approve { candidates: vec![cc(&e)] }).unwrap();
    let pv = f.preview(vec!["M/ok.md"]); let run_id = f.run(&pv);
    let v = f.service.source_context(SourceContextRequest { root_id: f.root_id.clone(), scope_id: pv.scope_id, run_id, topic_id: t.id.clone() }).unwrap();
    let j = serde_json::to_string(&v).unwrap(); eprintln!("OBS contains_value={} json={}", j.contains("Zed"), &j[..j.len().min(500)]);
    assert!(!j.contains("Zed"), "unselected transcript-derived value exported as human-supported");
}
#[test] fn n23_redacted_view_excludes_metadata_positive() {
    let f = F::new(); let (t, _c, _s) = alex_flow(&f); f.write("M/ok.md", "ok\n"); f.source("M/ok.md");
    let pv = f.preview(vec!["M/ok.md"]); let run_id = f.run(&pv);
    let v = f.service.source_context(SourceContextRequest { root_id: f.root_id.clone(), scope_id: pv.scope_id, run_id, topic_id: t.id.clone() }).unwrap();
    let j = serde_json::to_string(&v).unwrap(); eprintln!("OBS {j}");
    assert_eq!(v.current_claims[0].value, "they/them"); assert!(v.history.is_empty()); assert!(!j.contains("M/a.md") && !j.contains("he/him") && !j.contains("correction") && !j.contains("document_hash"));
}
#[test] fn n24_connection_source_endpoint_must_not_export_unselected_locator() {
    let f = F::new(); f.write("M/ok.md", "ok\n"); f.write("M/secret.md", "# Secret Merger Heading\nbody text\n"); let ok = f.source("M/ok.md"); let sec = f.source("M/secret.md"); let (a, b) = (topic("A"), topic("B")); f.accept(vec![tc(&a), tc(&b)]);
    let mut n = f.conn(&a, &b, &ok.citation); n.to = Endpoint::Source { source_id: sec.summary.source_id.clone(), locator: Locator { start_byte: 24, end_byte: 28, heading: Some("Secret Merger Heading".into()) } }; f.accept(vec![Candidate::Connection { connection: n }]);
    let pv = f.preview(vec!["M/ok.md"]); let run_id = f.run(&pv);
    let v = f.service.source_context(SourceContextRequest { root_id: f.root_id.clone(), scope_id: pv.scope_id, run_id, topic_id: a.id.clone() }).unwrap();
    let j = serde_json::to_string(&v.connections).unwrap(); eprintln!("OBS has_heading={} has_id={}", j.contains("Secret Merger"), j.contains(&sec.summary.source_id));
    assert!(!j.contains("Secret Merger") && !j.contains(&sec.summary.source_id));
}
#[test] fn n25_local_only_moved_over_reviewed_path_must_stay_restricted() {
    let f = F::new(); f.write("M/p.md", "secret\n"); let p = f.summary("M/p.md"); f.label(&p, Sensitivity::LocalOnly);
    f.write("M/q.md", "public\n"); let q = f.summary("M/q.md"); f.label(&q, Sensitivity::CloudAllowed);
    fs::rename(f.root.join("M/p.md"), f.root.join("M/q.md")).unwrap();
    let v = f.preview(vec!["M/q.md"]); eprintln!("OBS {:?}", v.sources.iter().map(|s| s.sensitivity).collect::<Vec<_>>());
    assert!(v.sources.is_empty(), "identical local-only bytes treated CloudAllowed by destination review");
}
#[test] fn n35_correction_marker_and_app_data_loss_should_warn() {
    let f = F::new(); let (t, _c, _s) = alex_flow(&f); let ops = f.service.history_list(f.req()).unwrap().operations; let last = ops.last().unwrap().id.clone(); fs::remove_file(f.root.join(format!("topics/operations/{last}.md"))).unwrap();
    let tmp = tempfile::tempdir().unwrap(); let mut svc = WorkspaceService::open(tmp.path().join("ad")).unwrap(); let root_id = svc.root_set(RootSetRequest { path: f.root.to_str().unwrap().into(), create: false }).unwrap().root.root_id;
    let v = svc.topic_get(TopicGetRequest { root_id, topic_id: t.id.clone() }).unwrap(); eprintln!("OBS current={:?} warnings={:?}", v.current_claims.iter().map(|c| c.value.clone()).collect::<Vec<_>>(), v.warnings);
    assert!(!v.warnings.is_empty(), "newest marker plus app data lost: false claim current, orphan correction and rule files on disk, no warning");
}
#[test] fn n40_guard_denied_leaves_no_visibility_and_no_temp() {
    let f = F::new(); f.write("M/a.md", "x\n"); f.source("M/a.md"); let t = topic("A"); let p = f.preview(vec!["M/a.md"]); let run_id = f.run(&p); let part = f.partition().join("proposals");
    let mut prepared_seen = false;
    let r = f.service.stage_proposal_guarded(StageProposalRequest { root_id: f.root_id.clone(), scope_id: Some(p.scope_id.clone()), origin: Origin::Analysis { run_id: run_id.clone() }, candidates: vec![tc(&t)] }, |_, _| { prepared_seen = fs::read_dir(&part).map(|d| d.count() > 0).unwrap_or(false); Err(ApiError::new("cancelled", "revoked")) });
    eprintln!("OBS {} prepared_seen={prepared_seen}", code(&r)); assert_eq!(code(&r), "cancelled"); assert!(prepared_seen);
    assert_eq!(fs::read_dir(&part).unwrap().count(), 0); assert!(f.service.proposals_list(f.req()).unwrap().proposals.is_empty());
    let late = f.service.scope_read(ScopeReadRequest { root_id: f.root_id.clone(), scope_id: p.scope_id, run_id }); eprintln!("OBS run consumed: {}", code(&late)); assert_eq!(code(&late), "forbidden");
}
#[test] fn n41_guard_no_action_and_ok_is_internal_error() {
    let f = F::new(); let t = topic("A"); let r = f.service.stage_proposal_guarded(StageProposalRequest { root_id: f.root_id.clone(), scope_id: None, origin: Origin::Manual, candidates: vec![tc(&t)] }, |_, _| Ok(())); eprintln!("OBS {}", code(&r)); assert_eq!(code(&r), "internal"); assert!(f.service.proposals_list(f.req()).unwrap().proposals.is_empty());
}
#[test] fn n43_no_change_has_no_operation_or_knowledge_positive() {
    let f = F::new(); let (t, c, _s) = alex_flow(&f); let before = f.service.history_list(f.req()).unwrap().operations.len();
    let mut again = fresh(&c); again.uncertainty = Some("w".into()); let q = f.model_stage(vec!["M/a.md"], vec![cc(&again)]).unwrap(); eprintln!("OBS status={:?} n={}", q.status, q.candidates.len()); assert_eq!(q.status, ProposalStatus::Accepted); assert!(q.candidates.is_empty());
    let d = f.approve(&q).unwrap(); assert!(d.operation.is_none()); assert_eq!(f.service.history_list(f.req()).unwrap().operations.len(), before); assert_eq!(f.view(&t).unwrap().current_claims[0].value, "they/them");
    let e = f.decide(&q, Decision::Reject); eprintln!("OBS reject after no-change={}", code(&e)); assert_eq!(code(&e), "conflict");
}
#[test] fn n50_duplicates_inside_one_model_proposal() {
    let f = F::new(); f.write("M/a.md", "x\n"); let s = f.source("M/a.md"); let t = topic("A"); f.accept(vec![tc(&t)]); let c = f.claim(&t, &s.citation, "x"); let d = fresh(&c);
    let p = f.model_stage(vec!["M/a.md"], vec![cc(&c), cc(&d)]).unwrap(); eprintln!("OBS candidates={}", p.candidates.len()); let _ = f.approve(&p); let n = f.view(&t).unwrap().current_claims.len(); eprintln!("OBS current={n}"); assert_eq!(n, 1);
}
