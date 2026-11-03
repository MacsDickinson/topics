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

fn ctx(f: &F, files: Vec<&str>, t: &Topic) -> WorkspaceResult<TopicView> {
    let pv = f.preview(files); let run_id = f.run(&pv);
    f.service.source_context(SourceContextRequest { root_id: f.root_id.clone(), scope_id: pv.scope_id, run_id, topic_id: t.id.clone() })
}
fn fresh(c: &Claim) -> Claim { let mut n = c.clone(); n.id = id(); n.revision_id = id(); n }
fn corrected(f: &F) -> (Topic, Claim, SourceRef) {
    f.write("M/a.md", "alex he/him\n"); let s = f.source("M/a.md"); let t = topic("Alex"); let c = f.claim(&t, &s.citation, "he/him");
    let p = f.model_stage(vec!["M/a.md"], vec![tc(&t), cc(&c)]).unwrap(); f.approve(&p).unwrap();
    let (n, r) = f.correct(&t, &c, "they/them"); f.accept(vec![Candidate::Correction { claim: n, rule: r }]);
    (t, c, s.citation)
}
#[test] fn r01_random_topic_id_with_identical_candidates_dedupes_positive() {
    let f = F::new(); let (t, c, _) = corrected(&f);
    let t2 = { let mut x = t.clone(); x.id = id(); x }; let mut c2 = fresh(&c); c2.subject_id = t2.id.clone();
    let q = f.model_stage(vec!["M/a.md"], vec![tc(&t2), cc(&c2)]).unwrap(); eprintln!("OBS status={:?}", q.status);
    assert_eq!(q.status, ProposalStatus::Accepted);
    assert_eq!(f.view(&t).unwrap().current_claims[0].value, "they/them");
}
#[test] fn r02_random_topic_id_bypasses_rule_when_fingerprint_differs_by_uncertainty() {
    let f = F::new(); let (t, c, _) = corrected(&f);
    let t2 = { let mut x = t.clone(); x.id = id(); x }; let mut c2 = fresh(&c); c2.subject_id = t2.id.clone(); c2.uncertainty = Some("reworded on second run".into());
    let q = f.model_stage(vec!["M/a.md"], vec![tc(&t2), cc(&c2)]).unwrap();
    let false_kept = q.candidates.iter().any(|x| matches!(x, Candidate::Claim { claim } if claim.value == "he/him"));
    eprintln!("OBS status={:?} false_claim_in_proposal={false_kept}", q.status);
    let r = f.approve(&q); eprintln!("OBS approve={:?}", r.as_ref().map(|_| "ok").map_err(|e| e.code.clone()));
    let tops = f.service.topics_list(f.req()).unwrap().topics; let alex = tops.iter().filter(|x| x.name == "Alex").count();
    let v2 = f.view(&t2); eprintln!("OBS Alex topics={alex} second_alex_current={:?}", v2.as_ref().map(|v| v.current_claims.iter().map(|c| c.value.clone()).collect::<Vec<_>>()));
    assert!(!false_kept, "correction rule bypassed by provisional topic id");
}
#[test] fn r03_random_topic_id_bypasses_rule_when_extra_fact_added() {
    let f = F::new(); let (t, c, s) = corrected(&f);
    let t2 = { let mut x = t.clone(); x.id = id(); x }; let mut c2 = fresh(&c); c2.subject_id = t2.id.clone();
    let mut extra = f.claim(&t2, &s, "new fact"); extra.predicate = "team".into();
    let q = f.model_stage(vec!["M/a.md"], vec![tc(&t2), cc(&c2), cc(&extra)]).unwrap();
    let false_kept = q.candidates.iter().any(|x| matches!(x, Candidate::Claim { claim } if claim.value == "he/him"));
    eprintln!("OBS status={:?} false_claim_in_proposal={false_kept}", q.status);
    assert!(!false_kept);
}
#[test] fn r04_reused_id_suppressed_mixed_proposal_positive() {
    let f = F::new(); let (t, c, s) = corrected(&f);
    let c2 = fresh(&c); let mut extra = f.claim(&t, &s, "new fact"); extra.predicate = "team".into();
    let q = f.model_stage(vec!["M/a.md"], vec![cc(&c2), cc(&extra)]).unwrap();
    assert_eq!(q.candidates.len(), 1); f.approve(&q).unwrap();
    let vals: Vec<_> = f.view(&t).unwrap().current_claims.iter().map(|c| c.value.clone()).collect(); eprintln!("OBS {vals:?}"); assert!(!vals.contains(&"he/him".to_string()));
}
#[test] fn r05_reused_topic_candidate_with_known_id_is_usable() {
    let f = F::new(); let (t, _, s) = corrected(&f); let mut extra = f.claim(&t, &s, "new fact"); extra.predicate = "team".into();
    let r = f.model_stage(vec!["M/a.md"], vec![tc(&t), cc(&extra)]); eprintln!("OBS {:?}", r.as_ref().map(|p| p.status).map_err(|e| (e.code.clone(), e.message.clone())));
    assert!(r.is_ok(), "echoing the known topic beside a new claim fails whole terminal stage");
}
#[test] fn r06_reusing_logical_claim_id_with_unchanged_value_is_not_terminal_error() {
    let f = F::new(); let (t, c, _) = corrected(&f); let v = f.view(&t).unwrap().current_claims[0].clone();
    let mut again = c.clone(); again.revision_id = id(); again.value = v.value.clone(); // same logical id, no supersedes
    let r = f.model_stage(vec!["M/a.md"], vec![cc(&again)]); eprintln!("OBS {:?}", r.as_ref().map(|p| p.status).map_err(|e| (e.code.clone(), e.message.clone())));
    assert!(r.is_ok());
}
#[test] fn r07_known_context_reproposal_of_corrected_value_must_not_duplicate() {
    let f = F::new(); let (t, c, _) = corrected(&f); let mut n = fresh(&c); n.value = "they/them".into();
    let q = f.model_stage(vec!["M/a.md"], vec![cc(&n)]).unwrap(); eprintln!("OBS status={:?}", q.status);
    if q.status == ProposalStatus::Pending { f.approve(&q).unwrap(); }
    let v = f.view(&t).unwrap(); eprintln!("OBS current={:?}", v.current_claims.iter().map(|c| (c.value.clone(), matches!(c.sources[0], Evidence::Passage { .. }))).collect::<Vec<_>>());
    assert_eq!(v.current_claims.len(), 1);
}
#[test] fn r08_id_reuse_connection_must_dedupe_against_first_run() {
    let f = F::new(); f.write("M/a.md", "alex leads ledger\n"); let s = f.source("M/a.md"); let (a, l) = (topic("Alex"), topic("Ledger"));
    let p = f.model_stage(vec!["M/a.md"], vec![tc(&a), tc(&l), Candidate::Connection { connection: f.conn(&a, &l, &s.citation) }]).unwrap(); f.approve(&p).unwrap();
    let q = f.model_stage(vec!["M/a.md"], vec![Candidate::Connection { connection: f.conn(&a, &l, &s.citation) }]).unwrap(); eprintln!("OBS status={:?}", q.status);
    if q.status == ProposalStatus::Pending { f.approve(&q).unwrap(); }
    let n = f.view(&a).unwrap().connections.len(); eprintln!("OBS connections={n}"); assert_eq!(n, 1);
}
#[test] fn r09_context_returns_correction_and_history_positive() {
    let f = F::new(); let (t, _, _) = corrected(&f); let v = ctx(&f, vec!["M/a.md"], &t).unwrap();
    assert_eq!(v.current_claims[0].value, "they/them"); assert_eq!(v.history.len(), 2);
}
#[test] fn r10_context_without_original_source_must_still_show_correction() {
    let f = F::new(); let (t, _, _) = corrected(&f); f.write("M/b.md", "alex he/him again\n"); f.source("M/b.md");
    let v = ctx(&f, vec!["M/b.md"], &t).unwrap(); eprintln!("OBS current={} history={}", v.current_claims.len(), v.history.len());
    assert_eq!(v.current_claims.len(), 1, "correction invisible to reanalysis of a new meeting");
}
#[test] fn r11_new_meeting_false_claim_with_reused_id_suppressed_positive() {
    let f = F::new(); let (t, c, _) = corrected(&f); f.write("M/b.md", "alex he/him again\n"); let b = f.source("M/b.md");
    let mut n = fresh(&c); n.sources = vec![Evidence::Passage { source: b.citation.clone() }]; let mut e = f.claim(&t, &b.citation, "fact"); e.predicate = "team".into();
    let q = f.model_stage(vec!["M/b.md"], vec![cc(&n), cc(&e)]).unwrap(); assert_eq!(q.candidates.len(), 1);
}
#[test] fn r12_new_meeting_random_topic_false_claim_bypasses_rule() {
    let f = F::new(); let (t, c, _) = corrected(&f); f.write("M/b.md", "alex he/him again\n"); let b = f.source("M/b.md");
    let t2 = { let mut x = t.clone(); x.id = id(); x }; let mut n = fresh(&c); n.subject_id = t2.id.clone(); n.sources = vec![Evidence::Passage { source: b.citation.clone() }];
    let q = f.model_stage(vec!["M/b.md"], vec![tc(&t2), cc(&n)]).unwrap();
    let kept = q.candidates.iter().any(|x| matches!(x, Candidate::Claim { claim } if claim.value == "he/him")); eprintln!("OBS status={:?} kept={kept}", q.status);
    assert!(!kept);
}
#[test] fn r13_human_edited_claim_must_appear_in_model_context() {
    let f = F::new(); f.write("M/a.md", "launch friday\n"); let s = f.source("M/a.md"); let t = topic("Ledger"); f.accept(vec![tc(&t)]);
    let c = f.claim(&t, &s.citation, "Friday"); let p = f.stage(vec![cc(&c)]).unwrap(); let mut e = c.clone(); e.value = "Monday".into();
    f.decide(&p, Decision::Approve { candidates: vec![cc(&e)] }).unwrap();
    let v = ctx(&f, vec!["M/a.md"], &t).unwrap(); eprintln!("OBS context current={}", v.current_claims.len());
    assert_eq!(v.current_claims.len(), 1, "edited accepted claim hidden from reanalysis context");
}
#[test] fn r14_edited_away_value_reproposal_must_not_duplicate() {
    let f = F::new(); f.write("M/a.md", "launch friday\n"); let s = f.source("M/a.md"); let t = topic("Ledger"); f.accept(vec![tc(&t)]);
    let c = f.claim(&t, &s.citation, "Friday"); let p = f.model_stage(vec!["M/a.md"], vec![cc(&c)]).unwrap(); let mut e = c.clone(); e.value = "Monday".into();
    f.decide(&p, Decision::Approve { candidates: vec![cc(&e)] }).unwrap();
    let mut n = fresh(&c); n.uncertainty = Some("again".into()); let q = f.model_stage(vec!["M/a.md"], vec![cc(&n)]).unwrap();
    eprintln!("OBS re-proposal status={:?} values={:?}", q.status, q.candidates.iter().filter_map(|x| if let Candidate::Claim { claim } = x { Some(claim.value.clone()) } else { None }).collect::<Vec<_>>());
}
#[test] fn r15_topic_named_from_newly_local_only_connection_source_denied_as_unit() {
    let f = F::new(); f.write("M/a.md", "alex\n"); f.write("M/ok.md", "ok\n"); let s = f.source("M/a.md"); f.source("M/ok.md"); let (a, l) = (topic("SecretAlex"), topic("Ledger"));
    f.accept(vec![tc(&a), tc(&l), Candidate::Connection { connection: f.conn(&a, &l, &s.citation) }]);
    f.label(&s.summary, Sensitivity::LocalOnly);
    let r = ctx(&f, vec!["M/ok.md"], &a); eprintln!("OBS {:?}", r.as_ref().map(|v| v.topic.name.clone()).map_err(|e| e.code.clone()));
    assert!(r.is_err(), "topic derived only via now-local-only connection evidence returned by name");
}
#[test] fn r16_old_revision_of_path_labelled_local_only_denied_as_unit() {
    let f = F::new(); f.write("M/a.md", "v1\n"); f.write("M/ok.md", "ok\n"); let s = f.source("M/a.md"); f.source("M/ok.md"); let t = topic("SecretAlex"); let c = f.claim(&t, &s.citation, "x"); f.accept(vec![tc(&t), cc(&c)]);
    f.write("M/a.md", "v2\n"); let s2 = f.summary("M/a.md"); assert_ne!(s2.source_id, s.summary.source_id); f.label(&s2, Sensitivity::LocalOnly);
    let r = ctx(&f, vec!["M/ok.md"], &t); eprintln!("OBS {:?}", r.as_ref().map(|v| v.topic.name.clone()).map_err(|e| e.code.clone()));
    assert!(r.is_err(), "denied-as-unit check reads per-hash record, not path label");
}
#[test] fn r17_context_must_not_include_other_topics_unresolved_identities() {
    let f = F::new(); f.write("M/a.md", "joe\n"); let s = f.source("M/a.md"); let (t, j1, j2) = (topic("Other"), topic("Joe"), topic("Joe")); f.accept(vec![tc(&t), tc(&j1), tc(&j2)]);
    f.accept(vec![Candidate::UnresolvedIdentity { name: "Joe".into(), candidates: vec![j1.id.clone(), j2.id.clone()], sources: vec![s.citation.clone()] }]);
    let v = ctx(&f, vec!["M/a.md"], &t).unwrap(); eprintln!("OBS unresolved={}", v.unresolved.len());
    assert!(v.unresolved.is_empty());
}
#[test] fn r18_context_errors_are_uniform_for_missing_and_private_topics() {
    let f = F::new(); f.write("M/a.md", "a\n"); f.source("M/a.md"); let mut t = topic("Secret"); t.sensitivity = Sensitivity::LocalOnly; f.accept(vec![tc(&t)]);
    let (e1, e2) = (ctx(&f, vec!["M/a.md"], &t).unwrap_err(), ctx(&f, vec!["M/a.md"], &topic("none")).unwrap_err()); eprintln!("OBS private={} missing={}", e1.code, e2.code);
    assert!(!e1.message.contains("Secret"));
}
