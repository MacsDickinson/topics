//! Explicit service bindings for all approved folder setup steps. UI rendering is mapped separately.
use cucumber::{World, given, then, when};
use topics_core::*;
#[path = "../../../topics-store/tests/support/setup.rs"]
mod support;
use support::*;
#[derive(World)]
#[world(init=Self::new)]
struct SetupWorld {
    f: Fixture,
    snapshot: Option<SetupTopicReview>,
    saved_operation: Option<Id>,
    before_items: usize,
    mode: String,
}
impl std::fmt::Debug for SetupWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Synthetic folder setup")
    }
}
impl SetupWorld {
    fn new() -> Self {
        Self {
            f: Fixture::new(),
            snapshot: None,
            saved_operation: None,
            before_items: 0,
            mode: String::new(),
        }
    }
    fn job(&self) -> &SetupJob {
        self.f.job.as_ref().unwrap()
    }
    fn prepare_groups(&mut self) {
        self.f.write(
            "meeting.md",
            "# Meeting\nAlex leads Ledger. Harbour ships. Alex uses he/him.\n",
        );
        self.f.preview(1);
        self.f.authorize();
        let cap = self.f.start();
        let s = self.f.read(&cap)[0].citation.clone();
        let alex = topic("Alex", TopicKind::Person);
        let ledger = topic("Ledger", TopicKind::Project);
        let harbour = topic("Harbour", TopicKind::Project);
        let cs = vec![
            claim(&alex, "pronouns", "he/him", &s),
            claim(&ledger, "status", "active", &s),
            claim(&harbour, "status", "ships", &s),
        ];
        let mut candidates = vec![
            Candidate::Topic {
                topic: alex.clone(),
            },
            Candidate::Topic {
                topic: ledger.clone(),
            },
            Candidate::Topic {
                topic: harbour.clone(),
            },
            Candidate::Connection {
                connection: relation(&alex, &ledger, &s),
            },
        ];
        candidates.extend(cs.iter().cloned().map(|claim| Candidate::Claim { claim }));
        self.f.topics = vec![alex, ledger, harbour];
        self.f.claims = cs;
        self.f.publish(cap, candidates);
    }
    fn add_issues(&mut self) {
        // One additional bounded batch in a fresh job, carrying eligible and disputed data together.
        self.f = Fixture::new();
        self.f.write(
            "meeting.md",
            "# Meeting\nAlex leads Ledger, Joe may attend. Conflicting dates Monday and Friday.\n",
        );
        self.f.preview(1);
        self.f.authorize();
        let cap = self.f.start();
        let s = self.f.read(&cap)[0].citation.clone();
        let alex = topic("Alex", TopicKind::Person);
        let ledger = topic("Ledger", TopicKind::Project);
        let joe = topic("Joe", TopicKind::Person);
        let joe2 = topic("Joe", TopicKind::Person);
        let mut inferred = relation(&alex, &ledger, &s);
        inferred.relation = "possibly_knows".into();
        inferred.uncertainty = Some("inferred association".into());
        let mut unknown = claim(&ledger, "rumour", "unknown", &s);
        unknown.uncertainty = Some("Unsupported assertion requiring input".into());
        let mut candidates = vec![
            Candidate::Topic {
                topic: alex.clone(),
            },
            Candidate::Topic {
                topic: ledger.clone(),
            },
            Candidate::Topic { topic: joe.clone() },
            Candidate::Topic {
                topic: joe2.clone(),
            },
            Candidate::Claim {
                claim: claim(&alex, "role", "Ledger lead", &s),
            },
            Candidate::Connection {
                connection: relation(&alex, &ledger, &s),
            },
            Candidate::Connection {
                connection: inferred,
            },
            Candidate::Claim {
                claim: claim(&ledger, "launch", "Monday", &s),
            },
            Candidate::Claim {
                claim: claim(&ledger, "launch", "Friday", &s),
            },
            Candidate::UnresolvedIdentity {
                name: "Joe".into(),
                candidates: vec![joe.id, joe2.id],
                sources: vec![s.clone()],
            },
        ];
        candidates.push(Candidate::Claim { claim: unknown });
        self.f.publish(cap, candidates);
        self.f.topics = vec![alex, ledger];
    }
    fn joe(&mut self) {
        self.f = Fixture::new();
        for i in 0..5 {
            self.f.write(
                &format!("occurrence-{i}.md"),
                &format!("# Note {i}\nJoe may lead Ledger.\n"),
            );
        }
        self.f.preview(5);
        self.f.authorize();
        self.f.finish_one(true);
    }
    fn step(&mut self, n: usize) {
        match n{
  0|43|95=>{assert!(self.f.app.is_absolute());assert_eq!(self.f.inventory().entry,SetupEntry::Empty);},
  1=>{for(p,t)in[("profiles/alex.md","# Alex\n[[Ledger]]"),("projects/ledger.md","# Ledger"),("meetings/2026-11-01.md","---\ndate: 2026-11-01\n---\n# Meeting"),("logs/2026-11-02.md","# Log"),("odd/note.md","# Unknown")]{self.f.write(p,t);}self.f.write(".config/hidden.md","secret");self.f.write("attachment.pdf","bytes");},
  2=>{assert_eq!(self.f.inventory().entry,SetupEntry::New);},
  3=>{let inv=self.f.inventory();assert_eq!(inv.documents.len(),5);assert_eq!(inv.folders.len(),5);assert!(inv.folders.iter().any(|f|f.dated>0));assert!(inv.documents.iter().any(|d|d.linked));assert!(!inv.excluded.is_empty());let roles=self.f.service.setup_inventory(SetupInventoryRequest{root_id:self.f.root_id.clone(),roles:vec![FolderRoleOverride{path:"odd".into(),role:FolderRole::Reference}]}).unwrap();assert_eq!(roles.documents.iter().find(|d|d.source.path=="odd/note.md").unwrap().role,FolderRole::Reference);},
  4=>{assert!(self.f.seen.is_empty());assert!(self.f.job.is_none());},
  5|77|123=>{self.f.unchanged();},
  6=>assert_eq!(self.f.inventory().entry,SetupEntry::Empty),
  7=>{let p=self.f.service.manual_propose(ManualProposeRequest{root_id:self.f.root_id.clone(),candidates:vec![Candidate::Topic{topic:topic("Local capture",TopicKind::Thread)}]}).unwrap();assert_eq!(p.proposal.status,ProposalStatus::Pending);assert!(self.f.seen.is_empty());},
  8=>{self.f.prepare_default(2);self.f.finish();self.f.all_receipt();self.f.decide();self.f.restart();},
  9=>{let inv=self.f.inventory();assert_eq!(inv.entry,SetupEntry::Returning);assert_eq!(inv.saved_topic_count,2);assert!(!self.f.service.setup_jobs_list(self.f.root_request()).unwrap().is_empty());},
  10=>{assert_eq!(self.f.saved().len(),2);assert_eq!(self.f.service.setup_jobs_list(self.f.root_request()).unwrap().len(),1);},
  11=>{self.f.prepare_default(3);self.f.job=None;self.f.write("private/sentinel.md","# Secret sentinel\nprivate words");let s=self.f.inventory().documents.into_iter().find(|d|d.source.path.contains("sentinel")).unwrap().source;self.f.service.sensitivity_set(SensitivitySetRequest{root_id:self.f.root_id.clone(),target:Endpoint::Source{source_id:s.source_id,locator:Locator{start_byte:0,end_byte:0,heading:None}},sensitivity:Sensitivity::LocalOnly}).unwrap();self.f.preview(1);},
  12=>{let m=self.f.manifest.as_ref().unwrap();assert!(m.excluded.iter().any(|e|e.path=="private/sentinel.md"));assert!(m.excluded.iter().any(|e|e.path=="topics"));},
  13=>self.f.authorize(),
  14=>{let cap=self.f.start();let selected=self.f.read(&cap);assert_eq!(selected.len(),1);let mut wrong=cap.clone();wrong.scope_id=id();assert!(self.f.service.setup_draft_context(SetupDraftContextRequest{capability:wrong}).is_err());let t=topic("Scoped",TopicKind::Thread);self.f.publish(cap,vec![Candidate::Topic{topic:t.clone()},Candidate::Claim{claim:claim(&t,"status","sourced",&selected[0].citation)}]);self.f.finish();},
  15|127=>assert_eq!(self.job().coverage.transmission_authorizations,1),
  16=>{let sent=serde_json::to_string(&self.f.seen).unwrap();assert!(!sent.contains("sentinel")&&!sent.contains("private"));},
  17|51=>{assert!(self.f.inventory().documents.iter().filter(|d|d.source.sensitivity!=Sensitivity::LocalOnly).all(|d|d.source.sensitivity==Sensitivity::Unknown));for t in self.f.saved(){assert!(self.f.review(&t).topic.current_claims.iter().flat_map(|c|&c.sources).all(|e|matches!(e,Evidence::Passage{..})));}},
  18=>{self.f.prepare_default(2);self.f.finish_one(false);},
  19=>{let path=self.job().manifest.batches[1].sources[0].source.path.clone();std::fs::write(self.f.root.join(path),"changed waiting source").unwrap();self.f.write("added.md","# Newly added\nnew data");},
  20=>{let j=self.job();let e=self.f.service.setup_batch_start(SetupBatchStartRequest{root_id:self.f.root_id.clone(),job_id:j.job_id.clone(),batch_id:j.batches[1].batch_id.clone()}).unwrap_err();assert_eq!(e.code,"stale_scope");self.f.last_error=Some(e);self.f.refresh();},
  21=>assert!(!self.job().manifest.batches.iter().flat_map(|b|&b.sources).any(|s|s.source.path=="added.md")),
  22=>{self.f.preview(1);assert!(self.f.manifest.as_ref().unwrap().batches.iter().flat_map(|b|&b.sources).any(|s|s.source.path=="added.md"));assert_ne!(self.f.manifest.as_ref().unwrap().fingerprint,self.job().manifest.fingerprint);},
  23|98=>{assert!(!self.job().graph.items.is_empty());assert_eq!(self.job().batches.iter().filter(|b|b.status==SetupBatchStatus::Completed).count(),1);if n==98{assert!(self.job().graph.items.iter().any(|i|matches!(&i.candidate,Candidate::Claim{claim} if claim.value=="draft edit retained")));}},
  24=>{self.f.prepare_default(3);},
  25=>{let paths:Vec<_>=self.job().manifest.batches.iter().flat_map(|b|&b.sources).map(|s|s.source.path.clone()).collect();let bytes=std::fs::read(self.f.root.join(&paths[0])).unwrap();std::fs::write(self.f.root.join(&paths[2]),&bytes).unwrap();self.f.original.insert(paths[2].clone(),bytes);self.f.preview(1);self.f.authorize();},
  26|33=>self.f.finish(),
  27=>{assert_eq!(self.job().graph.items.iter().filter(|i|matches!(i.candidate,Candidate::Topic{..})).count(),2);assert_eq!(self.job().graph.items.iter().filter(|i|matches!(i.candidate,Candidate::Connection{..})).count(),1);assert!(!self.job().graph.canonical_ids.is_empty());},
  28=>{let c=self.job().graph.items.iter().find_map(|i|if let Candidate::Connection{connection}=&i.candidate{Some(connection)}else{None}).unwrap();assert_eq!(c.sources.len(),3);for s in &c.sources{assert!(!s.excerpt.is_empty());}},
  29=>{let i=self.job().graph.items.iter().find(|i|matches!(i.candidate,Candidate::Connection{..})).unwrap();assert_eq!(i.independent_evidence_groups.len(),2);},
  30|36|101=>assert!(self.f.saved().is_empty()),
  31=>{self.joe();},
  32=>{self.f=Fixture::new();self.f.write("historic.md","---\ndate: 2025-01-01\n---\n# Historic\nAlex leads Ledger, Joe mentioned.\n");self.f.write("recent.md","---\ndate: 2026-11-01\n---\n# Recent\nBlake leads Ledger, Joe ambiguous.\n");self.f.preview(2);self.f.authorize();let cap=self.f.start();let sources=self.f.read(&cap);let ledger=topic("Ledger",TopicKind::Project);let joe=topic("Joe",TopicKind::Person);let joe2=topic("Joe",TopicKind::Person);let mut a=claim(&ledger,"leader","Alex",&sources[0].citation);a.valid_from=Some("2025-01-01".into());let mut b=claim(&ledger,"leader","Blake",&sources[1].citation);b.valid_from=Some("2026-11-01".into());self.f.publish(cap,vec![Candidate::Topic{topic:ledger.clone()},Candidate::Topic{topic:joe.clone()},Candidate::Topic{topic:joe2.clone()},Candidate::Claim{claim:a},Candidate::Claim{claim:b},Candidate::Connection{connection:relation(&joe,&ledger,&sources[0].citation)},Candidate::UnresolvedIdentity{name:"Joe".into(),candidates:vec![joe.id,joe2.id],sources:sources.iter().map(|s|s.citation.clone()).collect()}]);self.mode="historic".into();},
  34=>{self.f.all_receipt();assert!(self.f.receipt.as_ref().unwrap().candidates.iter().all(|c|!matches!(c,Candidate::UnresolvedIdentity{..})));assert!(self.job().graph.issues.iter().any(|i|i.issue_kind==SetupIssueKind::Identity));},
  35=>{let claims:Vec<_>=self.job().graph.items.iter().filter_map(|i|if let Candidate::Claim{claim}=&i.candidate{Some(claim)}else{None}).collect();assert_eq!(claims.len(),2);assert!(claims.iter().any(|c|c.valid_from.as_deref()==Some("2025-01-01")&&c.value=="Alex"));assert!(claims.iter().any(|c|c.valid_from.as_deref()==Some("2026-11-01")&&c.value=="Blake"));assert!(claims.iter().all(|c|!c.sources.is_empty()));},
  37..=42=>panic!("Presentation-owned scenario must run in desktop evidence"),
  44|52|57=>self.prepare_groups(),
  45=>self.add_issues(),
  46=>{self.f.all_receipt();assert!(!self.f.receipt.as_ref().unwrap().deferred_ids.is_empty());self.f.decide();},
  47=>{let r=self.f.result.as_ref().unwrap();assert!(!r.operations.is_empty());for op in &r.operations{assert!(self.f.root.join(format!("topics/operations/{}.md",op.id)).exists());}},
  48|117=>{let reviews:Vec<_>=self.f.saved().iter().flat_map(|t|self.f.review(t).reviews).collect();assert!(reviews.iter().any(|r|r.state==ReviewState::FromNotes));if n==48{assert!(reviews.iter().all(|r|r.state==ReviewState::FromNotes));}},
  49=>{assert!(self.job().graph.items.iter().any(|i|i.disposition==DraftDisposition::Pending));assert!(!self.job().graph.issues.is_empty());},
  50|129=>assert_eq!(self.f.save_actions,1),
  53=>{let ledger=self.f.topics[1].id.clone();let ids=self.job().graph.items.iter().filter(|i|matches!(&i.candidate,Candidate::Claim{claim} if claim.subject_id==ledger)).map(|i|i.item_id.clone()).collect();self.f.receipt=Some(self.f.service.setup_save_preview(SetupSavePreviewRequest{root_id:self.f.root_id.clone(),job_id:self.job().job_id.clone(),item_ids:ids}).unwrap());self.f.decide();let harbour=self.f.topics[2].id.clone();let j=self.job();let ids=j.graph.items.iter().filter(|i|matches!(&i.candidate,Candidate::Topic{topic} if topic.id==harbour)||matches!(&i.candidate,Candidate::Claim{claim} if claim.subject_id==harbour)).map(|i|i.item_id.clone()).collect();self.f.job=Some(self.f.service.setup_draft_disposition(SetupDraftDispositionRequest{root_id:self.f.root_id.clone(),job_id:j.job_id.clone(),expected_revision:j.revision,item_ids:ids,disposition:DraftDisposition::Deferred}).unwrap());},
  54=>assert!(self.f.saved().iter().any(|t|t.name=="Ledger")),
  55=>{assert!(self.job().graph.items.iter().any(|i|i.disposition==DraftDisposition::Deferred));assert!(!self.f.saved().iter().any(|t|t.name=="Harbour"));},
  56=>assert!(self.job().graph.items.iter().filter(|i|matches!(&i.candidate,Candidate::Claim{claim} if claim.subject_id==self.f.topics[2].id)).all(|i|i.disposition==DraftDisposition::Deferred)),
  58=>assert!(self.job().graph.items.iter().any(|i|matches!(&i.candidate,Candidate::Claim{claim} if claim.subject_id==self.f.topics[0].id))),
  59=>{let id=self.job().graph.items.iter().find(|i|matches!(i.candidate,Candidate::Connection{..})).unwrap().item_id.clone();self.f.receipt=Some(self.f.service.setup_save_preview(SetupSavePreviewRequest{root_id:self.f.root_id.clone(),job_id:self.job().job_id.clone(),item_ids:vec![id]}).unwrap());},
  60=>assert!(self.f.receipt.as_ref().unwrap().dependency_ids.contains(&self.f.topics[0].id)),
  61=>assert!(!self.f.receipt.as_ref().unwrap().candidates.iter().any(|c|matches!(c,Candidate::Claim{claim} if claim.subject_id==self.f.topics[0].id))),
  62=>{let joe=topic("Joe",TopicKind::Person);let p=self.f.service.manual_propose(ManualProposeRequest{root_id:self.f.root_id.clone(),candidates:vec![Candidate::Topic{topic:joe}]}).unwrap().proposal;self.f.service.proposal_decide(ProposalDecideRequest{root_id:self.f.root_id.clone(),proposal_id:p.id,expected_fingerprint:p.fingerprint,decision:Decision::Approve{candidates:p.candidates}}).unwrap();let j=self.job();let alex=self.f.topics[0].clone();let mut edited=alex;edited.name="Joe".into();self.f.job=Some(self.f.service.setup_draft_edit(SetupDraftEditRequest{root_id:self.f.root_id.clone(),job_id:j.job_id.clone(),expected_revision:j.revision,item_id:edited.id.clone(),candidate:Candidate::Topic{topic:edited}}).unwrap());self.mode="ambiguous".into();},
  63=>{assert_eq!(self.mode,"ambiguous");self.f.all_receipt();let receipt=self.f.receipt.as_ref().unwrap();assert!(!receipt.included_ids.is_empty());let relation=self.job().graph.items.iter().find(|i|matches!(i.candidate,Candidate::Connection{..})).unwrap();assert!(receipt.deferred_ids.contains(&relation.item_id));},
  64=>self.f.single(4),
  65=>assert_eq!(self.f.review(&self.f.topics[0]).topic.current_claims.len(),4),
  66=>{let ids=self.f.claims[..3].iter().map(|c|c.revision_id.clone()).collect();self.f.receipt=Some(self.f.service.setup_confirm_preview(SetupConfirmPreviewRequest{root_id:self.f.root_id.clone(),revision_ids:ids}).unwrap());self.f.decide();},
  67=>{let r=self.f.review(&self.f.topics[0]);assert_eq!(r.reviews.iter().filter(|m|m.state==ReviewState::ConfirmedByYou).count(),3);},
  68=>assert_eq!(self.f.review(&self.f.topics[0]).reviews.iter().filter(|m|m.state==ReviewState::FromNotes).count(),1),
  69=>{let r=self.f.result.as_ref().unwrap();assert_eq!(r.receipt.revisions.len(),3);assert!(r.receipt.revisions.iter().all(|c|!c.sources.is_empty()));assert_eq!(r.saved_ids,r.receipt.included_ids);},
  70=>self.f.single(2),
  83=>self.f.multiple_pronouns(),
  71|85=>self.f.correct(),
  72=>{let c=self.f.correction.as_ref().unwrap();assert_eq!(c.before[0].value,"he/him");assert_eq!(c.after[0].value,"they/them");assert_eq!(c.rule.scope.source_paths,vec!["meeting.md"]);},
  73|86=>self.f.decide(),
  74|116=>{let r=self.f.review(&self.f.topics[0]);let c=r.topic.current_claims.iter().find(|c|c.predicate=="pronouns").unwrap();assert_eq!(c.value,"they/them");let m=r.reviews.iter().find(|m|m.record_id==c.revision_id).unwrap();assert_eq!(m.state,ReviewState::ConfirmedByYou);let s=self.f.service.statement_get(StatementGetRequest{root_id:self.f.root_id.clone(),statement_id:m.statement_id.clone().unwrap()}).unwrap();assert_eq!(s.text,"Alex uses they/them");},
  75=>assert!(self.f.review(&self.f.topics[0]).reviews.iter().any(|r|r.state==ReviewState::FromNotes)),
  76=>assert!(self.f.review(&self.f.topics[0]).topic.history.iter().any(|c|c.value=="he/him"&&!c.sources.is_empty())),
  78=>self.joe(),
  79=>assert!(!self.job().graph.issues.is_empty()),
  80=>{let issues:Vec<_>=self.job().graph.issues.iter().filter(|i|i.issue_kind==SetupIssueKind::Identity).collect();assert_eq!(issues.len(),1);assert_eq!(issues[0].sources.len(),5);},
  81=>{let j=self.job();let ids=j.graph.issues.iter().flat_map(|i|i.item_ids.clone()).collect();self.f.job=Some(self.f.service.setup_draft_disposition(SetupDraftDispositionRequest{root_id:self.f.root_id.clone(),job_id:j.job_id.clone(),expected_revision:j.revision,item_ids:ids,disposition:DraftDisposition::Deferred}).unwrap());self.f.all_receipt();self.f.decide();},
  82=>assert_eq!(self.job().graph.items.iter().filter(|i|matches!(&i.candidate,Candidate::Topic{topic} if topic.name=="Joe")).count(),2),
  84=>{let other=topic("Alex",TopicKind::Person);let p=self.f.service.manual_propose(ManualProposeRequest{root_id:self.f.root_id.clone(),candidates:vec![Candidate::Topic{topic:other.clone()}]}).unwrap().proposal;self.f.service.proposal_decide(ProposalDecideRequest{root_id:self.f.root_id.clone(),proposal_id:p.id,expected_fingerprint:p.fingerprint,decision:Decision::Approve{candidates:p.candidates}}).unwrap();self.f.topics.push(other);},
  87=>assert_eq!(self.f.review(&self.f.topics[0]).topic.current_claims.iter().filter(|c|c.value=="they/them").count(),2),
  88=>assert!(self.f.review(&self.f.topics[1]).topic.current_claims.is_empty()),
  89=>{let p=self.f.service.scope_preview(ScopePreviewRequest{root_id:self.f.root_id.clone(),spec:ScopeSpec{files:vec!["meeting.md".into()],folders:vec![],date_from:None,date_to:None}}).unwrap();let run=self.f.service.analysis_start(AnalysisStartRequest{root_id:self.f.root_id.clone(),scope_id:p.scope_id.clone(),confirm_unknown:true}).unwrap();let c=claim(&self.f.topics[0],"pronouns","he/him",&self.f.citation("meeting.md"));let staged=self.f.service.stage_proposal(StageProposalRequest{root_id:self.f.root_id.clone(),scope_id:Some(p.scope_id),origin:Origin::Analysis{run_id:run.run_id},candidates:vec![Candidate::Claim{claim:c}]}).unwrap();assert!(!staged.candidates.iter().any(|c|matches!(c,Candidate::Claim{claim} if claim.value=="he/him")));},
  90=>{self.f.write("launch.md","# Ledger\nLedger launches Monday.\n");self.f.preview(1);self.f.authorize();let cap=self.f.start();let source=self.f.read(&cap)[0].citation.clone();let t=topic("Ledger",TopicKind::Project);let c=claim(&t,"launch_date","Monday",&source);self.f.topics=vec![t.clone()];self.f.claims=vec![c.clone()];self.f.publish(cap,vec![Candidate::Topic{topic:t},Candidate::Claim{claim:c.clone()}]);self.f.all_receipt();self.f.decide();self.f.receipt=Some(self.f.service.setup_confirm_preview(SetupConfirmPreviewRequest{root_id:self.f.root_id.clone(),revision_ids:vec![c.revision_id]}).unwrap());self.f.decide();},
  91=>{self.f.write("new.md","# New transcript\nLedger launches Friday.\n");self.f.preview(1);let mut request=SetupManifestPreviewRequest{root_id:self.f.root_id.clone(),spec:ScopeSpec{files:vec!["new.md".into()],folders:vec![],date_from:None,date_to:None},roles:vec![],limits:self.f.manifest.as_ref().unwrap().limits.clone(),runtime:"synthetic-no-provider".into()};request.spec.files=vec!["new.md".into()];self.f.manifest=Some(self.f.service.setup_manifest_preview(request).unwrap());self.f.authorize();let cap=self.f.start();let source=self.f.read(&cap)[0].citation.clone();let c=claim(&self.f.topics[0],"launch_date","Friday",&source);self.f.publish(cap,vec![Candidate::Claim{claim:c}]);},
  92=>assert!(self.job().graph.issues.iter().any(|i|i.issue_kind==SetupIssueKind::Replacement)),
  93=>{let view=self.f.review(&self.f.topics[0]);assert!(view.reviews.iter().any(|r|r.state==ReviewState::ConfirmedByYou));assert_eq!(view.topic.current_claims[0].value,"Monday");},
  94=>{let count=self.job().graph.issues.len();assert_eq!(count,1);assert_eq!(self.f.review(&self.f.topics[0]).topic.current_claims.len(),1);},
  96=>{self.f.prepare_default(2);self.f.finish_one(false);let i=self.job().graph.items.iter().find(|i|matches!(i.candidate,Candidate::Claim{..})).unwrap();let mut c=if let Candidate::Claim{claim}=&i.candidate{claim.clone()}else{unreachable!()};c.value="draft edit retained".into();let j=self.job();self.f.job=Some(self.f.service.setup_draft_edit(SetupDraftEditRequest{root_id:self.f.root_id.clone(),job_id:j.job_id.clone(),expected_revision:j.revision,item_id:i.item_id.clone(),candidate:Candidate::Claim{claim:c}}).unwrap());self.f.start();},
  97=>{self.f.control(SetupJobAction::Cancel);self.f.restart();},
  99=>assert_eq!(self.job().batches.iter().filter(|b|b.status==SetupBatchStatus::Cancelled).count(),1),
  100=>{let calls=self.job().coverage.model_calls;self.f.control(SetupJobAction::Resume);self.f.finish();assert_eq!(self.job().coverage.model_calls,calls+1);assert_eq!(self.job().coverage.processed.len(),2);},
  102=>{self.f.write("attachment.bin","excluded attachment");self.f.prepare_default(3);self.f.finish_one(false);let cap=self.f.start();self.f.job=Some(self.f.service.setup_batch_fail(SetupBatchFailRequest{capability:cap,error:ApiError::new("synthetic","private provider text must not persist")}).unwrap());self.f.finish();},
  103=>assert!(!self.job().graph.items.is_empty()),
  104=>assert!(self.job().graph.items.iter().any(|i|matches!(i.candidate,Candidate::Topic{..}))),
  105=>{assert_eq!(self.job().coverage.failed.len(),1);assert!(self.job().coverage.outstanding.is_empty());assert!(self.job().coverage.excluded.iter().any(|e|e.path=="attachment.bin"));},
  106=>{assert!(!self.job().coverage.complete);assert_eq!(self.job().status,SetupJobStatus::FinishedWithGaps);},
  107=>{let before=self.job().graph.items.len();self.f.control(SetupJobAction::RetryFailed);self.f.finish();assert_eq!(self.job().graph.items.len(),before);assert_eq!(self.job().coverage.processed.len(),3);},
  108=>{self.prepare_groups();self.f.all_receipt();},
  109=>{let item=self.job().graph.items.iter().find(|i|matches!(i.candidate,Candidate::Claim{..})).unwrap();let mut c=if let Candidate::Claim{claim}=&item.candidate{claim.clone()}else{unreachable!()};c.value="updated draft".into();let j=self.job();self.f.job=Some(self.f.service.setup_draft_edit(SetupDraftEditRequest{root_id:self.f.root_id.clone(),job_id:j.job_id.clone(),expected_revision:j.revision,item_id:item.item_id.clone(),candidate:Candidate::Claim{claim:c}}).unwrap());},
  110=>{let r=self.f.receipt.as_ref().unwrap();let e=self.f.service.setup_decide(SetupDecisionRequest{root_id:self.f.root_id.clone(),receipt_id:r.receipt_id.clone(),expected_fingerprint:r.fingerprint.clone()}).unwrap_err();assert_eq!(e.code,"conflict");assert!(e.message.contains("refresh"));},
  111=>{let old=self.f.receipt.as_ref().unwrap().fingerprint.clone();self.f.all_receipt();assert_ne!(old,self.f.receipt.as_ref().unwrap().fingerprint);},
  112=>{assert!(self.job().graph.items.iter().any(|i|matches!(&i.candidate,Candidate::Topic{topic} if topic.name=="Harbour")&&i.disposition==DraftDisposition::Pending));},
  113=>{self.f.single(2);self.f.correct();self.f.decide();self.snapshot=Some(self.f.review(&self.f.topics[0]));self.before_items=self.job().graph.items.len();},
  114=>{let db=self.f.app.join("index.sqlite3");if db.exists(){std::fs::remove_file(db).unwrap();}self.f.restart();let base=topics_store::TopicsService::open(&self.f.app).unwrap();base.index().unwrap();},
  115=>assert_eq!(self.snapshot.as_ref().unwrap(),&self.f.review(&self.f.topics[0])),
  118=>assert_eq!(self.before_items,self.job().graph.items.len()),
  119=>{self.f.single(1);self.saved_operation=Some(self.f.result.as_ref().unwrap().operations[0].id.clone());let t=topic("Independent",TopicKind::Thread);let p=self.f.service.manual_propose(ManualProposeRequest{root_id:self.f.root_id.clone(),candidates:vec![Candidate::Topic{topic:t}]}).unwrap().proposal;self.f.service.proposal_decide(ProposalDecideRequest{root_id:self.f.root_id.clone(),proposal_id:p.id,expected_fingerprint:p.fingerprint,decision:Decision::Approve{candidates:p.candidates}}).unwrap();},
  120=>{self.f.service.undo(UndoRequest{root_id:self.f.root_id.clone(),operation_id:self.saved_operation.clone().unwrap()}).unwrap();},
  121=>assert_eq!(self.f.saved().iter().map(|t|t.name.as_str()).collect::<Vec<_>>(),vec!["Independent"]),
  122=>{let mut probe=Fixture::new();probe.single(1);let base=probe.result.as_ref().unwrap().operations[0].id.clone();let r=probe.service.setup_confirm_preview(SetupConfirmPreviewRequest{root_id:probe.root_id.clone(),revision_ids:vec![probe.claims[0].revision_id.clone()]}).unwrap();probe.service.setup_decide(SetupDecisionRequest{root_id:probe.root_id.clone(),receipt_id:r.receipt_id,expected_fingerprint:r.fingerprint}).unwrap();let e=probe.service.undo(UndoRequest{root_id:probe.root_id.clone(),operation_id:base}).unwrap_err();assert_eq!(e.code,"conflict");assert!(e.details.unwrap().get("dependent_operations").is_some());},
  124=>{for i in 0..100{self.f.write(&format!("unfamiliar/report-{i}.md"),&format!("# Report {i}\nAlex leads Ledger.\n"));}self.f.preview(20);self.f.authorize();assert_eq!(self.job().manifest.batches.len(),5);},
  125=>{self.mode="scale".into();},
  126=>{self.f.finish_one(true);self.f.finish();assert!(self.job().graph.items.len()>4);assert!(!self.job().graph.issues.is_empty());},
  128=>{self.f.all_receipt();self.f.decide();assert_eq!(self.f.save_actions,1);assert!(!self.f.saved().is_empty());},
  130=>{assert_eq!(self.job().coverage.selected,100);assert_eq!(self.job().coverage.processed.len(),100);assert_eq!(self.job().coverage.model_calls,5);assert_eq!(self.f.seen.len(),100);println!("synthetic corpus=100 documents; batches/model calls=5; transmission consents=1; save actions=1");},
  _=>unreachable!("explicit approved step {n}"),
 }
    }
}
#[given("a synthetic notes folder and isolated absolute application data")]
fn step_0(w: &mut SetupWorld) {
    w.step(0);
}
#[given("the folder contains profiles, projects, meetings, logs and unclassified Markdown")]
fn step_1(w: &mut SetupWorld) {
    w.step(1);
}
#[when("I connect the folder for the first time")]
fn step_2(w: &mut SetupWorld) {
    w.step(2);
}
#[then("setup shows suggested editable folder roles, document counts, dates and exclusions")]
fn step_3(w: &mut SetupWorld) {
    w.step(3);
}
#[then("no source content or folder metadata has been sent to a model")]
fn step_4(w: &mut SetupWorld) {
    w.step(4);
}
#[then("all original note bytes are unchanged")]
fn step_5(w: &mut SetupWorld) {
    w.step(5);
}
#[given("I connect an empty folder")]
fn step_6(w: &mut SetupWorld) {
    w.step(6);
}
#[then("I can add notes or capture a topic without using a model")]
fn step_7(w: &mut SetupWorld) {
    w.step(7);
}
#[when("I reconnect a folder with an existing saved topic map")]
fn step_8(w: &mut SetupWorld) {
    w.step(8);
}
#[then("its map opens with an option to resume or update setup")]
fn step_9(w: &mut SetupWorld) {
    w.step(9);
}
#[then("its accepted knowledge is not imported again")]
fn step_10(w: &mut SetupWorld) {
    w.step(10);
}
#[given("I have previewed an eligible selection spanning several analysis batches")]
fn step_11(w: &mut SetupWorld) {
    w.step(11);
}
#[given("private and app-owned files are visibly excluded")]
fn step_12(w: &mut SetupWorld) {
    w.step(12);
}
#[when("I choose \"Build starting map\" and authorize the displayed manifest")]
fn step_13(w: &mut SetupWorld) {
    w.step(13);
}
#[then("each batch can read only its capability-bound subset of that manifest")]
fn step_14(w: &mut SetupWorld) {
    w.step(14);
}
#[then("the unchanged job does not require consent for every document or batch")]
fn step_15(w: &mut SetupWorld) {
    w.step(15);
}
#[then("no model receives local-only bytes, paths, labels or derived private context")]
fn step_16(w: &mut SetupWorld) {
    w.step(16);
}
#[then("no durable cloud permission is inferred from that consent")]
fn step_17(w: &mut SetupWorld) {
    w.step(17);
}
#[given("a setup job has completed one batch and has another batch waiting")]
fn step_18(w: &mut SetupWorld) {
    w.step(18);
}
#[when("a waiting source changes and a new source is added to the folder")]
fn step_19(w: &mut SetupWorld) {
    w.step(19);
}
#[then("the changed source is not sent under its old consent")]
fn step_20(w: &mut SetupWorld) {
    w.step(20);
}
#[then("the new source is not silently added to the job")]
fn step_21(w: &mut SetupWorld) {
    w.step(21);
}
#[then("the changed selection and exclusions can be previewed for renewed consent")]
fn step_22(w: &mut SetupWorld) {
    w.step(22);
}
#[then("unaffected completed draft results remain available")]
fn step_23(w: &mut SetupWorld) {
    w.step(23);
}
#[given("synthetic notes repeat \"Alex leads Ledger\" across several batches")]
fn step_24(w: &mut SetupWorld) {
    w.step(24);
}
#[given("one repeated account is a copy of an earlier summary")]
fn step_25(w: &mut SetupWorld) {
    w.step(25);
}
#[when("the authorized setup job finishes")]
fn step_26(w: &mut SetupWorld) {
    w.step(26);
}
#[then(
    "its draft has one unambiguously identified Alex, one Ledger and one sourced leads relationship"
)]
fn step_27(w: &mut SetupWorld) {
    w.step(27);
}
#[then("all distinct citations remain inspectable")]
fn step_28(w: &mut SetupWorld) {
    w.step(28);
}
#[then("the copied summary is not presented as independent confirmation")]
fn step_29(w: &mut SetupWorld) {
    w.step(29);
}
#[then("none of the draft records has been accepted automatically")]
fn step_30(w: &mut SetupWorld) {
    w.step(30);
}
#[given("the notes mention two different people named \"Joe\"")]
fn step_31(w: &mut SetupWorld) {
    w.step(31);
}
#[given("historic and recent accounts disagree about who leads Ledger")]
fn step_32(w: &mut SetupWorld) {
    w.step(32);
}
#[when("the authorized setup job builds its draft graph")]
fn step_33(w: &mut SetupWorld) {
    w.step(33);
}
#[then("the ambiguous Joe relationship is not attached to either person as a fact")]
fn step_34(w: &mut SetupWorld) {
    w.step(34);
}
#[then("the competing leadership accounts retain their dates and citations")]
fn step_35(w: &mut SetupWorld) {
    w.step(35);
}
#[then("recency or model confidence alone does not select the accepted account")]
fn step_36(w: &mut SetupWorld) {
    w.step(36);
}
#[given("a setup job has produced draft topics, explicit relationships and inferred associations")]
fn step_37(w: &mut SetupWorld) {
    w.step(37);
}
#[when("I open its map")]
fn step_38(w: &mut SetupWorld) {
    w.step(38);
}
#[then("I can navigate the topics in either graph or list form")]
fn step_39(w: &mut SetupWorld) {
    w.step(39);
}
#[then("explicit relationships are distinguishable from inferred associations")]
fn step_40(w: &mut SetupWorld) {
    w.step(40);
}
#[then("opening a statement or relationship reveals its underlying source passages")]
fn step_41(w: &mut SetupWorld) {
    w.step(41);
}
#[then("a prose topic briefing cannot introduce an uncited factual assertion")]
fn step_42(w: &mut SetupWorld) {
    w.step(42);
}
#[given("a synthetic draft topic map and isolated absolute application data")]
fn step_43(w: &mut SetupWorld) {
    w.step(43);
}
#[given("the draft has sourced unambiguous topics, statements and explicit relationships")]
fn step_44(w: &mut SetupWorld) {
    w.step(44);
}
#[given("it also has contradictory, ambiguous, unsupported and inferred items")]
fn step_45(w: &mut SetupWorld) {
    w.step(45);
}
#[when("I inspect its exact save receipt and choose \"Save starting map\"")]
fn step_46(w: &mut SetupWorld) {
    w.step(46);
}
#[then("the eligible displayed set is saved through journalled operations")]
fn step_47(w: &mut SetupWorld) {
    w.step(47);
}
#[then("its statements are labelled \"From your notes\" rather than \"Confirmed by you\"")]
fn step_48(w: &mut SetupWorld) {
    w.step(48);
}
#[then("the remaining items stay open for input")]
fn step_49(w: &mut SetupWorld) {
    w.step(49);
}
#[then("no checkbox per document or extracted fact was required")]
fn step_50(w: &mut SetupWorld) {
    w.step(50);
}
#[then("no source privacy label or human-authored value authority was upgraded")]
fn step_51(w: &mut SetupWorld) {
    w.step(51);
}
#[given("the draft has eligible groups for Ledger and Harbour")]
fn step_52(w: &mut SetupWorld) {
    w.step(52);
}
#[when("I save the Ledger group and choose \"Not now\" for Harbour")]
fn step_53(w: &mut SetupWorld) {
    w.step(53);
}
#[then("Ledger becomes available in the saved map")]
fn step_54(w: &mut SetupWorld) {
    w.step(54);
}
#[then("Harbour remains resumable in the draft rather than rejected")]
fn step_55(w: &mut SetupWorld) {
    w.step(55);
}
#[then("neither Harbour nor its facts are falsely shown as reviewed")]
fn step_56(w: &mut SetupWorld) {
    w.step(56);
}
#[given("a Ledger relationship needs a new unambiguous person topic called \"Alex\"")]
fn step_57(w: &mut SetupWorld) {
    w.step(57);
}
#[given("the draft has unrelated claims about Alex")]
fn step_58(w: &mut SetupWorld) {
    w.step(58);
}
#[when("I preview saving the Ledger relationship")]
fn step_59(w: &mut SetupWorld) {
    w.step(59);
}
#[then("the receipt visibly includes the required Alex endpoint topic")]
fn step_60(w: &mut SetupWorld) {
    w.step(60);
}
#[then("it does not include the unrelated Alex claims")]
fn step_61(w: &mut SetupWorld) {
    w.step(61);
}
#[when("the endpoint is ambiguous instead")]
fn step_62(w: &mut SetupWorld) {
    w.step(62);
}
#[then("that relationship is deferred without blocking unrelated Ledger facts")]
fn step_63(w: &mut SetupWorld) {
    w.step(63);
}
#[given("a saved topic has a displayed section containing three sourced statements")]
fn step_64(w: &mut SetupWorld) {
    w.step(64);
}
#[given("other facts exist outside that section")]
fn step_65(w: &mut SetupWorld) {
    w.step(65);
}
#[when("I explicitly confirm the enumerated three statements")]
fn step_66(w: &mut SetupWorld) {
    w.step(66);
}
#[then("those exact revisions are labelled \"Confirmed by you\"")]
fn step_67(w: &mut SetupWorld) {
    w.step(67);
}
#[then("the other facts remain \"From your notes\"")]
fn step_68(w: &mut SetupWorld) {
    w.step(68);
}
#[then("the decision retains the statement IDs and evidence that I saw")]
fn step_69(w: &mut SetupWorld) {
    w.step(69);
}
#[given("a saved Alex topic reports \"he/him\" from a transcript")]
fn step_70(w: &mut SetupWorld) {
    w.step(70);
}
#[when("I choose \"That's wrong\" and enter \"Alex uses they/them\"")]
fn step_71(w: &mut SetupWorld) {
    w.step(71);
}
#[then("I see a proposed before and after with the future rule's exact scope")]
fn step_72(w: &mut SetupWorld) {
    w.step(72);
}
#[when("I approve the displayed correction")]
fn step_73(w: &mut SetupWorld) {
    w.step(73);
}
#[then("the topic shows the corrected assertion attributed to my exact statement")]
fn step_74(w: &mut SetupWorld) {
    w.step(74);
}
#[then("unrelated facts are not marked confirmed")]
fn step_75(w: &mut SetupWorld) {
    w.step(75);
}
#[then("the prior assertion and original transcript remain available as evidence")]
fn step_76(w: &mut SetupWorld) {
    w.step(76);
}
#[then("no original source note is rewritten")]
fn step_77(w: &mut SetupWorld) {
    w.step(77);
}
#[given("five notes ambiguously refer to \"Joe\" in the same proposed relationship")]
fn step_78(w: &mut SetupWorld) {
    w.step(78);
}
#[when("I open \"Needs your input\"")]
fn step_79(w: &mut SetupWorld) {
    w.step(79);
}
#[then("I see one identity question with its five source occurrences")]
fn step_80(w: &mut SetupWorld) {
    w.step(80);
}
#[then("I can defer it without blocking unrelated saved topics")]
fn step_81(w: &mut SetupWorld) {
    w.step(81);
}
#[then("no uncertain identity is silently merged")]
fn step_82(w: &mut SetupWorld) {
    w.step(82);
}
#[given("multiple assertions unambiguously repeat the same wrong pronouns for Alex")]
fn step_83(w: &mut SetupWorld) {
    w.step(83);
}
#[given("another person has the same first name")]
fn step_84(w: &mut SetupWorld) {
    w.step(84);
}
#[when("I correct Alex and inspect the affected assertions and rule scope")]
fn step_85(w: &mut SetupWorld) {
    w.step(85);
}
#[when("I approve that displayed set once")]
fn step_86(w: &mut SetupWorld) {
    w.step(86);
}
#[then("only those unambiguously matching assertions are corrected")]
fn step_87(w: &mut SetupWorld) {
    w.step(87);
}
#[then("ambiguous and unrelated people are excluded")]
fn step_88(w: &mut SetupWorld) {
    w.step(88);
}
#[then("later analysis cannot revive the suppressed assertion for Alex")]
fn step_89(w: &mut SetupWorld) {
    w.step(89);
}
#[given("I confirmed Ledger's launch date as Monday")]
fn step_90(w: &mut SetupWorld) {
    w.step(90);
}
#[when("a newly authorized transcript reports Friday")]
fn step_91(w: &mut SetupWorld) {
    w.step(91);
}
#[then("Ledger shows a proposed change with the new citation and prior understanding")]
fn step_92(w: &mut SetupWorld) {
    w.step(92);
}
#[then("the confirmed Monday assertion is not silently replaced")]
fn step_93(w: &mut SetupWorld) {
    w.step(93);
}
#[then("repeated unchanged evidence does not create another question or duplicate fact")]
fn step_94(w: &mut SetupWorld) {
    w.step(94);
}
#[given("synthetic notes and isolated absolute application data")]
fn step_95(w: &mut SetupWorld) {
    w.step(95);
}
#[given("a setup job has completed one batch and has a second model turn running")]
fn step_96(w: &mut SetupWorld) {
    w.step(96);
}
#[when("I cancel the job and restart the app")]
fn step_97(w: &mut SetupWorld) {
    w.step(97);
}
#[then("the completed draft batch and my saved draft edits are available")]
fn step_98(w: &mut SetupWorld) {
    w.step(98);
}
#[then("the cancelled turn's buffered candidates are absent")]
fn step_99(w: &mut SetupWorld) {
    w.step(99);
}
#[then("resuming processes only the still-authorized outstanding inputs")]
fn step_100(w: &mut SetupWorld) {
    w.step(100);
}
#[then("no draft has become accepted knowledge without my approval")]
fn step_101(w: &mut SetupWorld) {
    w.step(101);
}
#[given("one authorized batch fails while other batches finish")]
fn step_102(w: &mut SetupWorld) {
    w.step(102);
}
#[when("I open the resulting map")]
fn step_103(w: &mut SetupWorld) {
    w.step(103);
}
#[then("successfully analysed topics remain browsable")]
fn step_104(w: &mut SetupWorld) {
    w.step(104);
}
#[then("coverage lists the failed, excluded and outstanding files separately")]
fn step_105(w: &mut SetupWorld) {
    w.step(105);
}
#[then("the job is not described as having analysed the whole folder")]
fn step_106(w: &mut SetupWorld) {
    w.step(106);
}
#[then("I can retry failed inputs without duplicating completed knowledge")]
fn step_107(w: &mut SetupWorld) {
    w.step(107);
}
#[given("I have previewed an exact group save or confirmation receipt")]
fn step_108(w: &mut SetupWorld) {
    w.step(108);
}
#[when("a referenced source, dependency or claim revision changes")]
fn step_109(w: &mut SetupWorld) {
    w.step(109);
}
#[then("the old decision fails with an actionable conflict")]
fn step_110(w: &mut SetupWorld) {
    w.step(110);
}
#[then("a refreshed receipt shows the changed scope before I can approve it")]
fn step_111(w: &mut SetupWorld) {
    w.step(111);
}
#[then("unrelated deferred groups are preserved")]
fn step_112(w: &mut SetupWorld) {
    w.step(112);
}
#[given("I saved a starting map and personally corrected one fact")]
fn step_113(w: &mut SetupWorld) {
    w.step(113);
}
#[when("the app restarts and its disposable SQLite index is rebuilt")]
fn step_114(w: &mut SetupWorld) {
    w.step(114);
}
#[then("the same topic IDs, graph connections, citations and review decisions return")]
fn step_115(w: &mut SetupWorld) {
    w.step(115);
}
#[then("the corrected assertion remains confirmed with its original statement")]
fn step_116(w: &mut SetupWorld) {
    w.step(116);
}
#[then("other saved imports remain \"From your notes\"")]
fn step_117(w: &mut SetupWorld) {
    w.step(117);
}
#[then("the setup checkpoint is not lost with the search database")]
fn step_118(w: &mut SetupWorld) {
    w.step(118);
}
#[given("I saved a starting-map group and later accepted independent work")]
fn step_119(w: &mut SetupWorld) {
    w.step(119);
}
#[when("I request undo of that group")]
fn step_120(w: &mut SetupWorld) {
    w.step(120);
}
#[then("the app checks dependencies and preserves unrelated later work")]
fn step_121(w: &mut SetupWorld) {
    w.step(121);
}
#[then("a dependent later decision is surfaced instead of silently removed")]
fn step_122(w: &mut SetupWorld) {
    w.step(122);
}
#[then("original note bytes and operation history are preserved")]
fn step_123(w: &mut SetupWorld) {
    w.step(123);
}
#[given("at least one hundred synthetic documents spanning several bounded model batches")]
fn step_124(w: &mut SetupWorld) {
    w.step(124);
}
#[given("the documents produce several topics plus some deferred issues")]
fn step_125(w: &mut SetupWorld) {
    w.step(125);
}
#[when("the unchanged authorized job builds a map")]
fn step_126(w: &mut SetupWorld) {
    w.step(126);
}
#[then("one transmission authorization covers that exact job manifest")]
fn step_127(w: &mut SetupWorld) {
    w.step(127);
}
#[then("one explicit save action can keep its eligible starting map")]
fn step_128(w: &mut SetupWorld) {
    w.step(128);
}
#[then("no required per-document or per-fact confirmation blocks using the map")]
fn step_129(w: &mut SetupWorld) {
    w.step(129);
}
#[then("recorded coverage and call counts account for every selected document")]
fn step_130(w: &mut SetupWorld) {
    w.step(130);
}
#[tokio::main]
async fn main() {
    SetupWorld::cucumber()
        .fail_on_skipped()
        .filter_run_and_exit("features/folder-setup", |_, _, s| {
            s.name != "A topic map is usable as a graph and as a list"
        })
        .await;
}
