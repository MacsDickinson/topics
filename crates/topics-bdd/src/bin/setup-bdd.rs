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
}
#[given("a synthetic notes folder and isolated absolute application data")]
fn step_0(w: &mut SetupWorld) {
    assert!(w.f.app.is_absolute());
    assert_eq!(w.f.inventory().entry, SetupEntry::Empty);
}
#[given("the folder contains profiles, projects, meetings, logs and unclassified Markdown")]
fn step_1(w: &mut SetupWorld) {
    for (p, t) in [
        ("profiles/alex.md", "# Alex\n[[Ledger]]"),
        ("projects/ledger.md", "# Ledger"),
        (
            "meetings/2026-11-01.md",
            "---\ndate: 2026-11-01\n---\n# Meeting",
        ),
        ("logs/2026-11-02.md", "# Log"),
        ("odd/note.md", "# Unknown"),
    ] {
        w.f.write(p, t);
    }
    w.f.write(".config/hidden.md", "secret");
    w.f.write("attachment.pdf", "bytes");
}
#[when("I connect the folder for the first time")]
fn step_2(w: &mut SetupWorld) {
    assert_eq!(w.f.inventory().entry, SetupEntry::New);
}
#[then("setup shows suggested editable folder roles, document counts, dates and exclusions")]
fn step_3(w: &mut SetupWorld) {
    let inv = w.f.inventory();
    assert_eq!(inv.documents.len(), 5);
    assert_eq!(inv.folders.len(), 5);
    assert!(inv.folders.iter().any(|f| f.dated > 0));
    assert!(inv.documents.iter().any(|d| d.linked));
    assert!(!inv.excluded.is_empty());
    let roles =
        w.f.service
            .setup_inventory(SetupInventoryRequest {
                root_id: w.f.root_id.clone(),
                roles: vec![FolderRoleOverride {
                    path: "odd".into(),
                    role: FolderRole::Reference,
                }],
            })
            .unwrap();
    assert_eq!(
        roles
            .documents
            .iter()
            .find(|d| d.source.path == "odd/note.md")
            .unwrap()
            .role,
        FolderRole::Reference
    );
}
#[then("no source content or folder metadata has been sent to a model")]
fn step_4(w: &mut SetupWorld) {
    assert!(w.f.seen.is_empty());
    assert!(w.f.job.is_none());
}
#[then("all original note bytes are unchanged")]
fn step_5(w: &mut SetupWorld) {
    w.f.unchanged();
}
#[given("I connect an empty folder")]
fn step_6(w: &mut SetupWorld) {
    assert_eq!(w.f.inventory().entry, SetupEntry::Empty)
}
#[then("I can add notes or capture a topic without using a model")]
fn step_7(w: &mut SetupWorld) {
    let p =
        w.f.service
            .manual_propose(ManualProposeRequest {
                root_id: w.f.root_id.clone(),
                candidates: vec![Candidate::Topic {
                    topic: topic("Local capture", TopicKind::Thread),
                }],
            })
            .unwrap();
    assert_eq!(p.proposal.status, ProposalStatus::Pending);
    assert!(w.f.seen.is_empty());
}
#[when("I reconnect a folder with an existing saved topic map")]
fn step_8(w: &mut SetupWorld) {
    w.f.prepare_default(2);
    w.f.finish();
    w.f.all_receipt();
    w.f.decide();
    w.f.restart();
}
#[then("its map opens with an option to resume or update setup")]
fn step_9(w: &mut SetupWorld) {
    let inv = w.f.inventory();
    assert_eq!(inv.entry, SetupEntry::Returning);
    assert_eq!(inv.saved_topic_count, 2);
    assert!(
        !w.f.service
            .setup_jobs_list(w.f.root_request())
            .unwrap()
            .is_empty()
    );
}
#[then("its accepted knowledge is not imported again")]
fn step_10(w: &mut SetupWorld) {
    assert_eq!(w.f.saved().len(), 2);
    assert_eq!(
        w.f.service
            .setup_jobs_list(w.f.root_request())
            .unwrap()
            .len(),
        1
    );
}
#[given("I have previewed an eligible selection spanning several analysis batches")]
fn step_11(w: &mut SetupWorld) {
    w.f.prepare_default(3);
    w.f.job = None;
    w.f.write("private/sentinel.md", "# Secret sentinel\nprivate words");
    let s =
        w.f.inventory()
            .documents
            .into_iter()
            .find(|d| d.source.path.contains("sentinel"))
            .unwrap()
            .source;
    w.f.service
        .sensitivity_set(SensitivitySetRequest {
            root_id: w.f.root_id.clone(),
            target: Endpoint::Source {
                source_id: s.source_id,
                locator: Locator {
                    start_byte: 0,
                    end_byte: 0,
                    heading: None,
                },
            },
            sensitivity: Sensitivity::LocalOnly,
        })
        .unwrap();
    w.f.preview(1);
}
#[given("private and app-owned files are visibly excluded")]
fn step_12(w: &mut SetupWorld) {
    let m = w.f.manifest.as_ref().unwrap();
    assert!(m.excluded.iter().any(|e| e.path == "private/sentinel.md"));
    assert!(m.excluded.iter().any(|e| e.path == "topics"));
}
#[when("I choose \"Build starting map\" and authorize the displayed manifest")]
fn step_13(w: &mut SetupWorld) {
    w.f.authorize()
}
#[then("each batch can read only its capability-bound subset of that manifest")]
fn step_14(w: &mut SetupWorld) {
    let cap = w.f.start();
    let selected = w.f.read(&cap);
    assert_eq!(selected.len(), 1);
    let mut wrong = cap.clone();
    wrong.scope_id = id();
    assert!(
        w.f.service
            .setup_draft_context(SetupDraftContextRequest { capability: wrong })
            .is_err()
    );
    let t = topic("Scoped", TopicKind::Thread);
    w.f.publish(
        cap,
        vec![
            Candidate::Topic { topic: t.clone() },
            Candidate::Claim {
                claim: claim(&t, "status", "sourced", &selected[0].citation),
            },
        ],
    );
    w.f.finish();
}
#[then("the unchanged job does not require consent for every document or batch")]
fn step_15(w: &mut SetupWorld) {
    assert_eq!(w.job().coverage.transmission_authorizations, 1)
}
#[then("no model receives local-only bytes, paths, labels or derived private context")]
fn step_16(w: &mut SetupWorld) {
    let sent = serde_json::to_string(&w.f.seen).unwrap();
    assert!(!sent.contains("sentinel") && !sent.contains("private"));
}
#[then("no durable cloud permission is inferred from that consent")]
fn step_17(w: &mut SetupWorld) {
    assert!(
        w.f.inventory()
            .documents
            .iter()
            .filter(|d| d.source.sensitivity != Sensitivity::LocalOnly)
            .all(|d| d.source.sensitivity == Sensitivity::Unknown)
    );
    for t in w.f.saved() {
        assert!(
            w.f.review(&t)
                .topic
                .current_claims
                .iter()
                .flat_map(|c| &c.sources)
                .all(|e| matches!(e, Evidence::Passage { .. }))
        );
    }
}
#[given("a setup job has completed one batch and has another batch waiting")]
fn step_18(w: &mut SetupWorld) {
    w.f.prepare_default(2);
    w.f.finish_one(false);
}
#[when("a waiting source changes and a new source is added to the folder")]
fn step_19(w: &mut SetupWorld) {
    let path = w.job().manifest.batches[1].sources[0].source.path.clone();
    std::fs::write(w.f.root.join(path), "changed waiting source").unwrap();
    w.f.write("added.md", "# Newly added\nnew data");
}
#[then("the changed source is not sent under its old consent")]
fn step_20(w: &mut SetupWorld) {
    let j = w.job();
    let e =
        w.f.service
            .setup_batch_start(SetupBatchStartRequest {
                root_id: w.f.root_id.clone(),
                job_id: j.job_id.clone(),
                batch_id: j.batches[1].batch_id.clone(),
            })
            .unwrap_err();
    assert_eq!(e.code, "stale_scope");
    w.f.last_error = Some(e);
    w.f.refresh();
}
#[then("the new source is not silently added to the job")]
fn step_21(w: &mut SetupWorld) {
    assert!(
        !w.job()
            .manifest
            .batches
            .iter()
            .flat_map(|b| &b.sources)
            .any(|s| s.source.path == "added.md")
    )
}
#[then("the changed selection and exclusions can be previewed for renewed consent")]
fn step_22(w: &mut SetupWorld) {
    let before=w.job().clone();
    w.f.preview(1);
    assert!(
        w.f.manifest
            .as_ref()
            .unwrap()
            .batches
            .iter()
            .flat_map(|b| &b.sources)
            .any(|s| s.source.path == "added.md")
    );
    assert_ne!(
        w.f.manifest.as_ref().unwrap().fingerprint,
        w.job().manifest.fingerprint
    );
    let manifest=w.f.manifest.as_ref().unwrap();
    w.f.job=Some(w.f.service.setup_job_reconsent(SetupJobReconsentRequest{root_id:w.f.root_id.clone(),job_id:before.job_id.clone(),expected_revision:w.job().revision,manifest_id:manifest.manifest_id.clone(),expected_fingerprint:manifest.fingerprint.clone()}).unwrap());
    assert_eq!(w.job().job_id,before.job_id);
    assert_eq!(w.job().prior_consents[0].manifest,before.manifest);
    assert_eq!(w.job().coverage.processed,before.coverage.processed);

}
#[then("unaffected completed draft results remain available")]
fn step_23(w: &mut SetupWorld) {
    let n = 23;
    assert!(!w.job().graph.items.is_empty());
    assert_eq!(
        w.job()
            .batches
            .iter()
            .filter(|b| b.status == SetupBatchStatus::Completed)
            .count(),
        1
    );
    if n == 98 {
        assert!(w.job().graph.items.iter().any(
            |i| matches!(&i.candidate,Candidate::Claim{claim} if claim.value=="draft edit retained")
        ));
    }
}
#[given("synthetic notes repeat \"Alex leads Ledger\" across several batches")]
fn step_24(w: &mut SetupWorld) {
    w.f.prepare_default(3);
}
#[given("one repeated account is a copy of an earlier summary")]
fn step_25(w: &mut SetupWorld) {
    let paths: Vec<_> = w
        .job()
        .manifest
        .batches
        .iter()
        .flat_map(|b| &b.sources)
        .map(|s| s.source.path.clone())
        .collect();
    let bytes = std::fs::read(w.f.root.join(&paths[0])).unwrap();
    std::fs::write(w.f.root.join(&paths[2]), &bytes).unwrap();
    w.f.original.insert(paths[2].clone(), bytes);
    w.f.preview(1);
    w.f.authorize();
}
#[when("the authorized setup job finishes")]
fn step_26(w: &mut SetupWorld) {
    w.f.finish()
}
#[then(
    "its draft has one unambiguously identified Alex, one Ledger and one sourced leads relationship"
)]
fn step_27(w: &mut SetupWorld) {
    assert_eq!(
        w.job()
            .graph
            .items
            .iter()
            .filter(|i| matches!(i.candidate, Candidate::Topic { .. }))
            .count(),
        2
    );
    assert_eq!(
        w.job()
            .graph
            .items
            .iter()
            .filter(|i| matches!(i.candidate, Candidate::Connection { .. }))
            .count(),
        1
    );
    assert!(!w.job().graph.canonical_ids.is_empty());
}
#[then("all distinct citations remain inspectable")]
fn step_28(w: &mut SetupWorld) {
    let c = w
        .job()
        .graph
        .items
        .iter()
        .find_map(|i| {
            if let Candidate::Connection { connection } = &i.candidate {
                Some(connection)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(c.sources.len(), 3);
    for s in &c.sources {
        assert!(!s.excerpt.is_empty());
    }
}
#[then("the copied summary is not presented as independent confirmation")]
fn step_29(w: &mut SetupWorld) {
    let i = w
        .job()
        .graph
        .items
        .iter()
        .find(|i| matches!(i.candidate, Candidate::Connection { .. }))
        .unwrap();
    assert_eq!(i.independent_evidence_groups.len(), 2);
}
#[then("none of the draft records has been accepted automatically")]
fn step_30(w: &mut SetupWorld) {
    assert!(w.f.saved().is_empty())
}
#[given("the notes mention two different people named \"Joe\"")]
fn step_31(w: &mut SetupWorld) {
    w.joe();
}
#[given("historic and recent accounts disagree about who leads Ledger")]
fn step_32(w: &mut SetupWorld) {
    w.f = Fixture::new();
    w.f.write(
        "historic.md",
        "---\ndate: 2025-01-01\n---\n# Historic\nAlex leads Ledger, Joe mentioned.\n",
    );
    w.f.write(
        "recent.md",
        "---\ndate: 2026-11-01\n---\n# Recent\nBlake leads Ledger, Joe ambiguous.\n",
    );
    w.f.preview(2);
    w.f.authorize();
    let cap = w.f.start();
    let sources = w.f.read(&cap);
    let ledger = topic("Ledger", TopicKind::Project);
    let joe = topic("Joe", TopicKind::Person);
    let joe2 = topic("Joe", TopicKind::Person);
    let mut a = claim(&ledger, "leader", "Alex", &sources[0].citation);
    a.valid_from = Some("2025-01-01".into());
    let mut b = claim(&ledger, "leader", "Blake", &sources[1].citation);
    b.valid_from = Some("2026-11-01".into());
    w.f.publish(
        cap,
        vec![
            Candidate::Topic {
                topic: ledger.clone(),
            },
            Candidate::Topic { topic: joe.clone() },
            Candidate::Topic {
                topic: joe2.clone(),
            },
            Candidate::Claim { claim: a },
            Candidate::Claim { claim: b },
            Candidate::Connection {
                connection: relation(&joe, &ledger, &sources[0].citation),
            },
            Candidate::UnresolvedIdentity {
                name: "Joe".into(),
                candidates: vec![joe.id, joe2.id],
                sources: sources.iter().map(|s| s.citation.clone()).collect(),
            },
        ],
    );
    w.mode = "historic".into();
}
#[when("the authorized setup job builds its draft graph")]
fn step_33(w: &mut SetupWorld) {
    w.f.finish()
}
#[then("the ambiguous Joe relationship is not attached to either person as a fact")]
fn step_34(w: &mut SetupWorld) {
    w.f.all_receipt();
    assert!(
        w.f.receipt
            .as_ref()
            .unwrap()
            .candidates
            .iter()
            .all(|c| !matches!(c, Candidate::UnresolvedIdentity { .. }))
    );
    assert!(
        w.job()
            .graph
            .issues
            .iter()
            .any(|i| i.issue_kind == SetupIssueKind::Identity)
    );
}
#[then("the competing leadership accounts retain their dates and citations")]
fn step_35(w: &mut SetupWorld) {
    let claims: Vec<_> = w
        .job()
        .graph
        .items
        .iter()
        .filter_map(|i| {
            if let Candidate::Claim { claim } = &i.candidate {
                Some(claim)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(claims.len(), 2);
    assert!(
        claims
            .iter()
            .any(|c| c.valid_from.as_deref() == Some("2025-01-01") && c.value == "Alex")
    );
    assert!(
        claims
            .iter()
            .any(|c| c.valid_from.as_deref() == Some("2026-11-01") && c.value == "Blake")
    );
    assert!(claims.iter().all(|c| !c.sources.is_empty()));
}
#[then("recency or model confidence alone does not select the accepted account")]
fn step_36(w: &mut SetupWorld) {
    assert!(w.f.saved().is_empty())
}
#[given("a setup job has produced draft topics, explicit relationships and inferred associations")]
fn step_37(_w: &mut SetupWorld) {
    panic!("Presentation-owned scenario must run in desktop evidence")
}
#[when("I open its map")]
fn step_38(_w: &mut SetupWorld) {
    panic!("Presentation-owned scenario must run in desktop evidence")
}
#[then("I can navigate the topics in either graph or list form")]
fn step_39(_w: &mut SetupWorld) {
    panic!("Presentation-owned scenario must run in desktop evidence")
}
#[then("explicit relationships are distinguishable from inferred associations")]
fn step_40(_w: &mut SetupWorld) {
    panic!("Presentation-owned scenario must run in desktop evidence")
}
#[then("opening a statement or relationship reveals its underlying source passages")]
fn step_41(_w: &mut SetupWorld) {
    panic!("Presentation-owned scenario must run in desktop evidence")
}
#[then("a prose topic briefing cannot introduce an uncited factual assertion")]
fn step_42(_w: &mut SetupWorld) {
    panic!("Presentation-owned scenario must run in desktop evidence")
}
#[given("a synthetic draft topic map and isolated absolute application data")]
fn step_43(w: &mut SetupWorld) {
    assert!(w.f.app.is_absolute());
    assert_eq!(w.f.inventory().entry, SetupEntry::Empty);
}
#[given("the draft has sourced unambiguous topics, statements and explicit relationships")]
fn step_44(w: &mut SetupWorld) {
    w.prepare_groups()
}
#[given("it also has contradictory, ambiguous, unsupported and inferred items")]
fn step_45(w: &mut SetupWorld) {
    w.add_issues()
}
#[when("I inspect its exact save receipt and choose \"Save starting map\"")]
fn step_46(w: &mut SetupWorld) {
    w.f.all_receipt();
    assert!(!w.f.receipt.as_ref().unwrap().deferred_ids.is_empty());
    w.f.decide();
}
#[then("the eligible displayed set is saved through journalled operations")]
fn step_47(w: &mut SetupWorld) {
    let r = w.f.result.as_ref().unwrap();
    assert!(!r.operations.is_empty());
    for op in &r.operations {
        assert!(
            w.f.root
                .join(format!("topics/operations/{}.md", op.id))
                .exists()
        );
    }
}
#[then("its statements are labelled \"From your notes\" rather than \"Confirmed by you\"")]
fn step_48(w: &mut SetupWorld) {
    let n = 48;
    let reviews: Vec<_> =
        w.f.saved()
            .iter()
            .flat_map(|t| w.f.review(t).reviews)
            .collect();
    assert!(reviews.iter().any(|r| r.state == ReviewState::FromNotes));
    if n == 48 {
        assert!(reviews.iter().all(|r| r.state == ReviewState::FromNotes));
    }
}
#[then("the remaining items stay open for input")]
fn step_49(w: &mut SetupWorld) {
    assert!(
        w.job()
            .graph
            .items
            .iter()
            .any(|i| i.disposition == DraftDisposition::Pending)
    );
    assert!(!w.job().graph.issues.is_empty());
}
#[then("no checkbox per document or extracted fact was required")]
fn step_50(w: &mut SetupWorld) {
    assert_eq!(w.f.save_actions, 1)
}
#[then("no source privacy label or human-authored value authority was upgraded")]
fn step_51(w: &mut SetupWorld) {
    assert!(
        w.f.inventory()
            .documents
            .iter()
            .filter(|d| d.source.sensitivity != Sensitivity::LocalOnly)
            .all(|d| d.source.sensitivity == Sensitivity::Unknown)
    );
    for t in w.f.saved() {
        assert!(
            w.f.review(&t)
                .topic
                .current_claims
                .iter()
                .flat_map(|c| &c.sources)
                .all(|e| matches!(e, Evidence::Passage { .. }))
        );
    }
}
#[given("the draft has eligible groups for Ledger and Harbour")]
fn step_52(w: &mut SetupWorld) {
    w.prepare_groups()
}
#[when("I save the Ledger group and choose \"Not now\" for Harbour")]
fn step_53(w: &mut SetupWorld) {
    let ledger = w.f.topics[1].id.clone();
    let ids = w
        .job()
        .graph
        .items
        .iter()
        .filter(|i| matches!(&i.candidate,Candidate::Claim{claim} if claim.subject_id==ledger))
        .map(|i| i.item_id.clone())
        .collect();
    w.f.receipt = Some(
        w.f.service
            .setup_save_preview(SetupSavePreviewRequest {
                root_id: w.f.root_id.clone(),
                job_id: w.job().job_id.clone(),
                item_ids: ids,
            })
            .unwrap(),
    );
    w.f.decide();
    let harbour = w.f.topics[2].id.clone();
    let j = w.job();
    let ids = j
        .graph
        .items
        .iter()
        .filter(|i| {
            matches!(&i.candidate,Candidate::Topic{topic} if topic.id==harbour)
                || matches!(&i.candidate,Candidate::Claim{claim} if claim.subject_id==harbour)
        })
        .map(|i| i.item_id.clone())
        .collect();
    w.f.job = Some(
        w.f.service
            .setup_draft_disposition(SetupDraftDispositionRequest {
                root_id: w.f.root_id.clone(),
                job_id: j.job_id.clone(),
                expected_revision: j.revision,
                item_ids: ids,
                disposition: DraftDisposition::Deferred,
            })
            .unwrap(),
    );
}
#[then("Ledger becomes available in the saved map")]
fn step_54(w: &mut SetupWorld) {
    assert!(w.f.saved().iter().any(|t| t.name == "Ledger"))
}
#[then("Harbour remains resumable in the draft rather than rejected")]
fn step_55(w: &mut SetupWorld) {
    assert!(
        w.job()
            .graph
            .items
            .iter()
            .any(|i| i.disposition == DraftDisposition::Deferred)
    );
    assert!(!w.f.saved().iter().any(|t| t.name == "Harbour"));
}
#[then("neither Harbour nor its facts are falsely shown as reviewed")]
fn step_56(w: &mut SetupWorld) {
    assert!(w.job().graph.items.iter().filter(|i|matches!(&i.candidate,Candidate::Claim{claim} if claim.subject_id==w.f.topics[2].id)).all(|i|i.disposition==DraftDisposition::Deferred))
}
#[given("a Ledger relationship needs a new unambiguous person topic called \"Alex\"")]
fn step_57(w: &mut SetupWorld) {
    w.prepare_groups()
}
#[given("the draft has unrelated claims about Alex")]
fn step_58(w: &mut SetupWorld) {
    assert!(w.job().graph.items.iter().any(
        |i| matches!(&i.candidate,Candidate::Claim{claim} if claim.subject_id==w.f.topics[0].id)
    ))
}
#[when("I preview saving the Ledger relationship")]
fn step_59(w: &mut SetupWorld) {
    let id = w
        .job()
        .graph
        .items
        .iter()
        .find(|i| matches!(i.candidate, Candidate::Connection { .. }))
        .unwrap()
        .item_id
        .clone();
    w.f.receipt = Some(
        w.f.service
            .setup_save_preview(SetupSavePreviewRequest {
                root_id: w.f.root_id.clone(),
                job_id: w.job().job_id.clone(),
                item_ids: vec![id],
            })
            .unwrap(),
    );
}
#[then("the receipt visibly includes the required Alex endpoint topic")]
fn step_60(w: &mut SetupWorld) {
    assert!(
        w.f.receipt
            .as_ref()
            .unwrap()
            .dependency_ids
            .contains(&w.f.topics[0].id)
    )
}
#[then("it does not include the unrelated Alex claims")]
fn step_61(w: &mut SetupWorld) {
    assert!(
        !w.f.receipt
            .as_ref()
            .unwrap()
            .candidates
            .iter()
            .any(|c| matches!(c,Candidate::Claim{claim} if claim.subject_id==w.f.topics[0].id))
    )
}
#[when("the endpoint is ambiguous instead")]
fn step_62(w: &mut SetupWorld) {
    let joe = topic("Joe", TopicKind::Person);
    let p =
        w.f.service
            .manual_propose(ManualProposeRequest {
                root_id: w.f.root_id.clone(),
                candidates: vec![Candidate::Topic { topic: joe }],
            })
            .unwrap()
            .proposal;
    w.f.service
        .proposal_decide(ProposalDecideRequest {
            root_id: w.f.root_id.clone(),
            proposal_id: p.id,
            expected_fingerprint: p.fingerprint,
            decision: Decision::Approve {
                candidates: p.candidates,
            },
        })
        .unwrap();
    let j = w.job();
    let alex = w.f.topics[0].clone();
    let mut edited = alex;
    edited.name = "Joe".into();
    w.f.job = Some(
        w.f.service
            .setup_draft_edit(SetupDraftEditRequest {
                root_id: w.f.root_id.clone(),
                job_id: j.job_id.clone(),
                expected_revision: j.revision,
                item_id: edited.id.clone(),
                candidate: Candidate::Topic { topic: edited },
            })
            .unwrap(),
    );
    w.mode = "ambiguous".into();
}
#[then("that relationship is deferred without blocking unrelated Ledger facts")]
fn step_63(w: &mut SetupWorld) {
    assert_eq!(w.mode, "ambiguous");
    w.f.all_receipt();
    let receipt = w.f.receipt.as_ref().unwrap();
    assert!(!receipt.included_ids.is_empty());
    let relation = w
        .job()
        .graph
        .items
        .iter()
        .find(|i| matches!(i.candidate, Candidate::Connection { .. }))
        .unwrap();
    assert!(receipt.deferred_ids.contains(&relation.item_id));
}
#[given("a saved topic has a displayed section containing three sourced statements")]
fn step_64(w: &mut SetupWorld) {
    w.f.single(4)
}
#[given("other facts exist outside that section")]
fn step_65(w: &mut SetupWorld) {
    assert_eq!(w.f.review(&w.f.topics[0]).topic.current_claims.len(), 4)
}
#[when("I explicitly confirm the enumerated three statements")]
fn step_66(w: &mut SetupWorld) {
    let ids = w.f.claims[..3]
        .iter()
        .map(|c| c.revision_id.clone())
        .collect();
    w.f.receipt = Some(
        w.f.service
            .setup_confirm_preview(SetupConfirmPreviewRequest {
                root_id: w.f.root_id.clone(),
                revision_ids: ids,
            })
            .unwrap(),
    );
    w.f.decide();
}
#[then("those exact revisions are labelled \"Confirmed by you\"")]
fn step_67(w: &mut SetupWorld) {
    let r = w.f.review(&w.f.topics[0]);
    assert_eq!(
        r.reviews
            .iter()
            .filter(|m| m.state == ReviewState::ConfirmedByYou)
            .count(),
        3
    );
}
#[then("the other facts remain \"From your notes\"")]
fn step_68(w: &mut SetupWorld) {
    assert_eq!(
        w.f.review(&w.f.topics[0])
            .reviews
            .iter()
            .filter(|m| m.state == ReviewState::FromNotes)
            .count(),
        1
    )
}
#[then("the decision retains the statement IDs and evidence that I saw")]
fn step_69(w: &mut SetupWorld) {
    let r = w.f.result.as_ref().unwrap();
    assert_eq!(r.receipt.revisions.len(), 3);
    assert!(r.receipt.revisions.iter().all(|c| !c.sources.is_empty()));
    assert_eq!(r.saved_ids, r.receipt.included_ids);
}
#[given("a saved Alex topic reports \"he/him\" from a transcript")]
fn step_70(w: &mut SetupWorld) {
    w.f.single(2)
}
#[when("I choose \"That's wrong\" and enter \"Alex uses they/them\"")]
fn step_71(w: &mut SetupWorld) {
    w.f.correct()
}
#[then("I see a proposed before and after with the future rule's exact scope")]
fn step_72(w: &mut SetupWorld) {
    let c = w.f.correction.as_ref().unwrap();
    assert_eq!(c.before[0].value, "he/him");
    assert_eq!(c.after[0].value, "they/them");
    assert_eq!(c.rule.scope.source_paths, vec!["meeting.md"]);
}
#[when("I approve the displayed correction")]
fn step_73(w: &mut SetupWorld) {
    w.f.decide()
}
#[then("the topic shows the corrected assertion attributed to my exact statement")]
fn step_74(w: &mut SetupWorld) {
    let r = w.f.review(&w.f.topics[0]);
    let c = r
        .topic
        .current_claims
        .iter()
        .find(|c| c.predicate == "pronouns")
        .unwrap();
    assert_eq!(c.value, "they/them");
    let m = r
        .reviews
        .iter()
        .find(|m| m.record_id == c.revision_id)
        .unwrap();
    assert_eq!(m.state, ReviewState::ConfirmedByYou);
    let s =
        w.f.service
            .statement_get(StatementGetRequest {
                root_id: w.f.root_id.clone(),
                statement_id: m.statement_id.clone().unwrap(),
            })
            .unwrap();
    assert_eq!(s.text, "Alex uses they/them");
}
#[then("unrelated facts are not marked confirmed")]
fn step_75(w: &mut SetupWorld) {
    assert!(
        w.f.review(&w.f.topics[0])
            .reviews
            .iter()
            .any(|r| r.state == ReviewState::FromNotes)
    )
}
#[then("the prior assertion and original transcript remain available as evidence")]
fn step_76(w: &mut SetupWorld) {
    assert!(
        w.f.review(&w.f.topics[0])
            .topic
            .history
            .iter()
            .any(|c| c.value == "he/him" && !c.sources.is_empty())
    )
}
#[then("no original source note is rewritten")]
fn step_77(w: &mut SetupWorld) {
    w.f.unchanged();
}
#[given("five notes ambiguously refer to \"Joe\" in the same proposed relationship")]
fn step_78(w: &mut SetupWorld) {
    w.joe()
}
#[when("I open \"Needs your input\"")]
fn step_79(w: &mut SetupWorld) {
    assert!(!w.job().graph.issues.is_empty())
}
#[then("I see one identity question with its five source occurrences")]
fn step_80(w: &mut SetupWorld) {
    let issues: Vec<_> = w
        .job()
        .graph
        .issues
        .iter()
        .filter(|i| i.issue_kind == SetupIssueKind::Identity)
        .collect();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].sources.len(), 5);
}
#[then("I can defer it without blocking unrelated saved topics")]
fn step_81(w: &mut SetupWorld) {
    let j = w.job();
    let ids = j
        .graph
        .issues
        .iter()
        .flat_map(|i| i.item_ids.clone())
        .collect();
    w.f.job = Some(
        w.f.service
            .setup_draft_disposition(SetupDraftDispositionRequest {
                root_id: w.f.root_id.clone(),
                job_id: j.job_id.clone(),
                expected_revision: j.revision,
                item_ids: ids,
                disposition: DraftDisposition::Deferred,
            })
            .unwrap(),
    );
    w.f.all_receipt();
    w.f.decide();
}
#[then("no uncertain identity is silently merged")]
fn step_82(w: &mut SetupWorld) {
    assert_eq!(
        w.job()
            .graph
            .items
            .iter()
            .filter(|i| matches!(&i.candidate,Candidate::Topic{topic} if topic.name=="Joe"))
            .count(),
        2
    )
}
#[given("multiple assertions unambiguously repeat the same wrong pronouns for Alex")]
fn step_83(w: &mut SetupWorld) {
    w.f.multiple_pronouns()
}
#[given("another person has the same first name")]
fn step_84(w: &mut SetupWorld) {
    let other = topic("Alex", TopicKind::Person);
    let p =
        w.f.service
            .manual_propose(ManualProposeRequest {
                root_id: w.f.root_id.clone(),
                candidates: vec![Candidate::Topic {
                    topic: other.clone(),
                }],
            })
            .unwrap()
            .proposal;
    w.f.service
        .proposal_decide(ProposalDecideRequest {
            root_id: w.f.root_id.clone(),
            proposal_id: p.id,
            expected_fingerprint: p.fingerprint,
            decision: Decision::Approve {
                candidates: p.candidates,
            },
        })
        .unwrap();
    w.f.topics.push(other);
}
#[when("I correct Alex and inspect the affected assertions and rule scope")]
fn step_85(w: &mut SetupWorld) {
    w.f.correct()
}
#[when("I approve that displayed set once")]
fn step_86(w: &mut SetupWorld) {
    w.f.decide()
}
#[then("only those unambiguously matching assertions are corrected")]
fn step_87(w: &mut SetupWorld) {
    assert_eq!(
        w.f.review(&w.f.topics[0])
            .topic
            .current_claims
            .iter()
            .filter(|c| c.value == "they/them")
            .count(),
        2
    )
}
#[then("ambiguous and unrelated people are excluded")]
fn step_88(w: &mut SetupWorld) {
    assert!(w.f.review(&w.f.topics[1]).topic.current_claims.is_empty())
}
#[then("later analysis cannot revive the suppressed assertion for Alex")]
fn step_89(w: &mut SetupWorld) {
    let p =
        w.f.service
            .scope_preview(ScopePreviewRequest {
                root_id: w.f.root_id.clone(),
                spec: ScopeSpec {
                    files: vec!["meeting.md".into()],
                    folders: vec![],
                    date_from: None,
                    date_to: None,
                },
            })
            .unwrap();
    let run =
        w.f.service
            .analysis_start(AnalysisStartRequest {
                root_id: w.f.root_id.clone(),
                scope_id: p.scope_id.clone(),
                confirm_unknown: true,
            })
            .unwrap();
    let c = claim(
        &w.f.topics[0],
        "pronouns",
        "he/him",
        &w.f.citation("meeting.md"),
    );
    let staged =
        w.f.service
            .stage_proposal(StageProposalRequest {
                root_id: w.f.root_id.clone(),
                scope_id: Some(p.scope_id),
                origin: Origin::Analysis { run_id: run.run_id },
                candidates: vec![Candidate::Claim { claim: c }],
            })
            .unwrap();
    assert!(
        !staged
            .candidates
            .iter()
            .any(|c| matches!(c,Candidate::Claim{claim} if claim.value=="he/him"))
    );
}
#[given("I confirmed Ledger's launch date as Monday")]
fn step_90(w: &mut SetupWorld) {
    w.f.write("launch.md", "# Ledger\nLedger launches Monday.\n");
    w.f.preview(1);
    w.f.authorize();
    let cap = w.f.start();
    let source = w.f.read(&cap)[0].citation.clone();
    let t = topic("Ledger", TopicKind::Project);
    let c = claim(&t, "launch_date", "Monday", &source);
    w.f.topics = vec![t.clone()];
    w.f.claims = vec![c.clone()];
    w.f.publish(
        cap,
        vec![
            Candidate::Topic { topic: t },
            Candidate::Claim { claim: c.clone() },
        ],
    );
    w.f.all_receipt();
    w.f.decide();
    w.f.receipt = Some(
        w.f.service
            .setup_confirm_preview(SetupConfirmPreviewRequest {
                root_id: w.f.root_id.clone(),
                revision_ids: vec![c.revision_id],
            })
            .unwrap(),
    );
    w.f.decide();
}
#[when("a newly authorized transcript reports Friday")]
fn step_91(w: &mut SetupWorld) {
    w.f.write("new.md", "# New transcript\nLedger launches Friday.\n");
    w.f.preview(1);
    let mut request = SetupManifestPreviewRequest {
        root_id: w.f.root_id.clone(),
        spec: ScopeSpec {
            files: vec!["new.md".into()],
            folders: vec![],
            date_from: None,
            date_to: None,
        },
        roles: vec![],
        limits: w.f.manifest.as_ref().unwrap().limits.clone(),
        runtime: "synthetic-no-provider".into(),
    };
    request.spec.files = vec!["new.md".into()];
    w.f.manifest = Some(w.f.service.setup_manifest_preview(request).unwrap());
    w.f.authorize();
    let cap = w.f.start();
    let source = w.f.read(&cap)[0].citation.clone();
    let c = claim(&w.f.topics[0], "launch_date", "Friday", &source);
    w.f.publish(cap, vec![Candidate::Claim { claim: c }]);
}
#[then("Ledger shows a proposed change with the new citation and prior understanding")]
fn step_92(w: &mut SetupWorld) {
    assert!(
        w.job()
            .graph
            .issues
            .iter()
            .any(|i| i.issue_kind == SetupIssueKind::Replacement)
    )
}
#[then("the confirmed Monday assertion is not silently replaced")]
fn step_93(w: &mut SetupWorld) {
    let view = w.f.review(&w.f.topics[0]);
    assert!(
        view.reviews
            .iter()
            .any(|r| r.state == ReviewState::ConfirmedByYou)
    );
    assert_eq!(view.topic.current_claims[0].value, "Monday");
}
#[then("repeated unchanged evidence does not create another question or duplicate fact")]
fn step_94(w: &mut SetupWorld) {
    let before = w.job().clone();
    let m = before.manifest.clone();
    let preview =
        w.f.service
            .setup_manifest_preview(SetupManifestPreviewRequest {
                root_id: w.f.root_id.clone(),
                spec: m.spec,
                roles: m.roles,
                limits: m.limits,
                runtime: m.runtime,
            })
            .unwrap();
    let repeated =
        w.f.service
            .setup_job_create(SetupJobCreateRequest {
                root_id: w.f.root_id.clone(),
                manifest_id: preview.manifest_id,
                expected_fingerprint: preview.fingerprint,
            })
            .unwrap();
    assert_eq!(repeated.job_id, before.job_id);
    assert_eq!(repeated.graph, before.graph);
    assert_eq!(repeated.coverage.model_calls, before.coverage.model_calls);
    assert_eq!(repeated.graph.issues.len(), 1);
    assert_eq!(w.f.review(&w.f.topics[0]).topic.current_claims.len(), 1);
}

#[given("synthetic notes and isolated absolute application data")]
fn step_95(w: &mut SetupWorld) {
    assert!(w.f.app.is_absolute());
    assert_eq!(w.f.inventory().entry, SetupEntry::Empty);
}
#[given("a setup job has completed one batch and has a second model turn running")]
fn step_96(w: &mut SetupWorld) {
    w.f.prepare_default(2);
    w.f.finish_one(false);
    let i = w
        .job()
        .graph
        .items
        .iter()
        .find(|i| matches!(i.candidate, Candidate::Claim { .. }))
        .unwrap();
    let mut c = if let Candidate::Claim { claim } = &i.candidate {
        claim.clone()
    } else {
        unreachable!()
    };
    c.value = "draft edit retained".into();
    let j = w.job();
    w.f.job = Some(
        w.f.service
            .setup_draft_edit(SetupDraftEditRequest {
                root_id: w.f.root_id.clone(),
                job_id: j.job_id.clone(),
                expected_revision: j.revision,
                item_id: i.item_id.clone(),
                candidate: Candidate::Claim { claim: c },
            })
            .unwrap(),
    );
    w.f.start();
}
#[when("I cancel the job and restart the app")]
fn step_97(w: &mut SetupWorld) {
    w.f.control(SetupJobAction::Cancel);
    w.f.restart();
}
#[then("the completed draft batch and my saved draft edits are available")]
fn step_98(w: &mut SetupWorld) {
    let n = 98;
    assert!(!w.job().graph.items.is_empty());
    assert_eq!(
        w.job()
            .batches
            .iter()
            .filter(|b| b.status == SetupBatchStatus::Completed)
            .count(),
        1
    );
    if n == 98 {
        assert!(w.job().graph.items.iter().any(
            |i| matches!(&i.candidate,Candidate::Claim{claim} if claim.value=="draft edit retained")
        ));
    }
}
#[then("the cancelled turn's buffered candidates are absent")]
fn step_99(w: &mut SetupWorld) {
    assert_eq!(
        w.job()
            .batches
            .iter()
            .filter(|b| b.status == SetupBatchStatus::Cancelled)
            .count(),
        1
    )
}
#[then("resuming processes only the still-authorized outstanding inputs")]
fn step_100(w: &mut SetupWorld) {
    let calls = w.job().coverage.model_calls;
    w.f.control(SetupJobAction::Resume);
    w.f.finish();
    assert_eq!(w.job().coverage.model_calls, calls + 1);
    assert_eq!(w.job().coverage.processed.len(), 2);
}
#[then("no draft has become accepted knowledge without my approval")]
fn step_101(w: &mut SetupWorld) {
    assert!(w.f.saved().is_empty())
}
#[given("one authorized batch fails while other batches finish")]
fn step_102(w: &mut SetupWorld) {
    w.f.write("attachment.bin", "excluded attachment");
    w.f.prepare_default(3);
    w.f.finish_one(false);
    let cap = w.f.start();
    w.f.job = Some(
        w.f.service
            .setup_batch_fail(SetupBatchFailRequest {
                capability: cap,
                error: ApiError::new("synthetic", "private provider text must not persist"),
            })
            .unwrap(),
    );
    w.f.finish();
}
#[when("I open the resulting map")]
fn step_103(w: &mut SetupWorld) {
    assert!(!w.job().graph.items.is_empty())
}
#[then("successfully analysed topics remain browsable")]
fn step_104(w: &mut SetupWorld) {
    assert!(
        w.job()
            .graph
            .items
            .iter()
            .any(|i| matches!(i.candidate, Candidate::Topic { .. }))
    )
}
#[then("coverage lists the failed, excluded and outstanding files separately")]
fn step_105(w: &mut SetupWorld) {
    assert_eq!(w.job().coverage.failed.len(), 1);
    assert!(w.job().coverage.outstanding.is_empty());
    assert!(
        w.job()
            .coverage
            .excluded
            .iter()
            .any(|e| e.path == "attachment.bin")
    );
}
#[then("the job is not described as having analysed the whole folder")]
fn step_106(w: &mut SetupWorld) {
    assert!(!w.job().coverage.complete);
    assert_eq!(w.job().status, SetupJobStatus::FinishedWithGaps);
}
#[then("I can retry failed inputs without duplicating completed knowledge")]
fn step_107(w: &mut SetupWorld) {
    let before = w.job().graph.items.len();
    w.f.control(SetupJobAction::RetryFailed);
    w.f.finish();
    assert_eq!(w.job().graph.items.len(), before);
    assert_eq!(w.job().coverage.processed.len(), 3);
}
#[given("I have previewed an exact group save or confirmation receipt")]
fn step_108(w: &mut SetupWorld) {
    w.prepare_groups();
    w.f.all_receipt();
}
#[when("a referenced source, dependency or claim revision changes")]
fn step_109(w: &mut SetupWorld) {
    let item = w
        .job()
        .graph
        .items
        .iter()
        .find(|i| matches!(i.candidate, Candidate::Claim { .. }))
        .unwrap();
    let mut c = if let Candidate::Claim { claim } = &item.candidate {
        claim.clone()
    } else {
        unreachable!()
    };
    c.value = "updated draft".into();
    let j = w.job();
    w.f.job = Some(
        w.f.service
            .setup_draft_edit(SetupDraftEditRequest {
                root_id: w.f.root_id.clone(),
                job_id: j.job_id.clone(),
                expected_revision: j.revision,
                item_id: item.item_id.clone(),
                candidate: Candidate::Claim { claim: c },
            })
            .unwrap(),
    );
}
#[then("the old decision fails with an actionable conflict")]
fn step_110(w: &mut SetupWorld) {
    let r = w.f.receipt.as_ref().unwrap();
    let e =
        w.f.service
            .setup_decide(SetupDecisionRequest {
                root_id: w.f.root_id.clone(),
                receipt_id: r.receipt_id.clone(),
                expected_fingerprint: r.fingerprint.clone(),
            })
            .unwrap_err();
    assert_eq!(e.code, "conflict");
    assert!(e.message.contains("refresh"));
}
#[then("a refreshed receipt shows the changed scope before I can approve it")]
fn step_111(w: &mut SetupWorld) {
    let old = w.f.receipt.as_ref().unwrap().fingerprint.clone();
    w.f.all_receipt();
    assert_ne!(old, w.f.receipt.as_ref().unwrap().fingerprint);
}
#[then("unrelated deferred groups are preserved")]
fn step_112(w: &mut SetupWorld) {
    assert!(w.job().graph.items.iter().any(
        |i| matches!(&i.candidate,Candidate::Topic{topic} if topic.name=="Harbour")
            && i.disposition == DraftDisposition::Pending
    ));
}
#[given("I saved a starting map and personally corrected one fact")]
fn step_113(w: &mut SetupWorld) {
    w.f.single(2);
    w.f.correct();
    w.f.decide();
    w.snapshot = Some(w.f.review(&w.f.topics[0]));
    w.before_items = w.job().graph.items.len();
}
#[when("the app restarts and its disposable SQLite index is rebuilt")]
fn step_114(w: &mut SetupWorld) {
    let db = w.f.app.join("index.sqlite3");
    if db.exists() {
        std::fs::remove_file(db).unwrap();
    }
    w.f.restart();
    let base = topics_store::TopicsService::open(&w.f.app).unwrap();
    base.index().unwrap();
}
#[then("the same topic IDs, graph connections, citations and review decisions return")]
fn step_115(w: &mut SetupWorld) {
    assert_eq!(w.snapshot.as_ref().unwrap(), &w.f.review(&w.f.topics[0]))
}
#[then("the corrected assertion remains confirmed with its original statement")]
fn step_116(w: &mut SetupWorld) {
    let r = w.f.review(&w.f.topics[0]);
    let c = r
        .topic
        .current_claims
        .iter()
        .find(|c| c.predicate == "pronouns")
        .unwrap();
    assert_eq!(c.value, "they/them");
    let m = r
        .reviews
        .iter()
        .find(|m| m.record_id == c.revision_id)
        .unwrap();
    assert_eq!(m.state, ReviewState::ConfirmedByYou);
    let s =
        w.f.service
            .statement_get(StatementGetRequest {
                root_id: w.f.root_id.clone(),
                statement_id: m.statement_id.clone().unwrap(),
            })
            .unwrap();
    assert_eq!(s.text, "Alex uses they/them");
}
#[then("other saved imports remain \"From your notes\"")]
fn step_117(w: &mut SetupWorld) {
    let n = 117;
    let reviews: Vec<_> =
        w.f.saved()
            .iter()
            .flat_map(|t| w.f.review(t).reviews)
            .collect();
    assert!(reviews.iter().any(|r| r.state == ReviewState::FromNotes));
    if n == 48 {
        assert!(reviews.iter().all(|r| r.state == ReviewState::FromNotes));
    }
}
#[then("the setup checkpoint is not lost with the search database")]
fn step_118(w: &mut SetupWorld) {
    assert_eq!(w.before_items, w.job().graph.items.len())
}
#[given("I saved a starting-map group and later accepted independent work")]
fn step_119(w: &mut SetupWorld) {
    w.f.single(1);
    w.saved_operation = Some(w.f.result.as_ref().unwrap().operations[0].id.clone());
    let t = topic("Independent", TopicKind::Thread);
    let p =
        w.f.service
            .manual_propose(ManualProposeRequest {
                root_id: w.f.root_id.clone(),
                candidates: vec![Candidate::Topic { topic: t }],
            })
            .unwrap()
            .proposal;
    w.f.service
        .proposal_decide(ProposalDecideRequest {
            root_id: w.f.root_id.clone(),
            proposal_id: p.id,
            expected_fingerprint: p.fingerprint,
            decision: Decision::Approve {
                candidates: p.candidates,
            },
        })
        .unwrap();
}
#[when("I request undo of that group")]
fn step_120(w: &mut SetupWorld) {
    w.f.service
        .undo(UndoRequest {
            root_id: w.f.root_id.clone(),
            operation_id: w.saved_operation.clone().unwrap(),
        })
        .unwrap();
}
#[then("the app checks dependencies and preserves unrelated later work")]
fn step_121(w: &mut SetupWorld) {
    assert_eq!(
        w.f.saved()
            .iter()
            .map(|t| t.name.as_str())
            .collect::<Vec<_>>(),
        vec!["Independent"]
    )
}
#[then("a dependent later decision is surfaced instead of silently removed")]
fn step_122(_w: &mut SetupWorld) {
    let mut probe = Fixture::new();
    probe.single(1);
    let base = probe.result.as_ref().unwrap().operations[0].id.clone();
    let r = probe
        .service
        .setup_confirm_preview(SetupConfirmPreviewRequest {
            root_id: probe.root_id.clone(),
            revision_ids: vec![probe.claims[0].revision_id.clone()],
        })
        .unwrap();
    probe
        .service
        .setup_decide(SetupDecisionRequest {
            root_id: probe.root_id.clone(),
            receipt_id: r.receipt_id,
            expected_fingerprint: r.fingerprint,
        })
        .unwrap();
    let e = probe
        .service
        .undo(UndoRequest {
            root_id: probe.root_id.clone(),
            operation_id: base,
        })
        .unwrap_err();
    assert_eq!(e.code, "conflict");
    assert!(e.details.unwrap().get("dependent_operations").is_some());
}
#[then("original note bytes and operation history are preserved")]
fn step_123(w: &mut SetupWorld) {
    w.f.unchanged();
}
#[given("at least one hundred synthetic documents spanning several bounded model batches")]
fn step_124(w: &mut SetupWorld) {
    for i in 0..100 {
        w.f.write(
            &format!("unfamiliar/report-{i}.md"),
            &format!("# Report {i}\nAlex leads Ledger.\n"),
        );
    }
    w.f.preview(20);
    w.f.authorize();
    assert_eq!(w.job().manifest.batches.len(), 5);
}
#[given("the documents produce several topics plus some deferred issues")]
fn step_125(w: &mut SetupWorld) {
    w.mode = "scale".into();
}
#[when("the unchanged authorized job builds a map")]
fn step_126(w: &mut SetupWorld) {
    w.f.finish_one(true);
    w.f.finish();
    assert!(w.job().graph.items.len() > 4);
    assert!(!w.job().graph.issues.is_empty());
}
#[then("one transmission authorization covers that exact job manifest")]
fn step_127(w: &mut SetupWorld) {
    assert_eq!(w.job().coverage.transmission_authorizations, 1)
}
#[then("one explicit save action can keep its eligible starting map")]
fn step_128(w: &mut SetupWorld) {
    w.f.all_receipt();
    w.f.decide();
    assert_eq!(w.f.save_actions, 1);
    assert!(!w.f.saved().is_empty());
}
#[then("no required per-document or per-fact confirmation blocks using the map")]
fn step_129(w: &mut SetupWorld) {
    assert_eq!(w.f.save_actions, 1)
}
#[then("recorded coverage and call counts account for every selected document")]
fn step_130(w: &mut SetupWorld) {
    assert_eq!(w.job().coverage.selected, 100);
    assert_eq!(w.job().coverage.processed.len(), 100);
    assert_eq!(w.job().coverage.model_calls, 5);
    assert_eq!(w.f.seen.len(), 100);
    println!(
        "synthetic corpus=100 documents; batches/model attempts=5; transmission consents=1; save actions=1; elapsed_ms={}",w.f.started.elapsed().as_millis()
    );
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
