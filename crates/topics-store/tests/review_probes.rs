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
    fn undo(&self, op: &Id) -> WorkspaceResult<OperationResult> {
        self.service.undo(UndoRequest {
            root_id: self.root_id.clone(),
            operation_id: op.clone(),
        })
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

// ---- crash / recovery ----
#[test]
fn p01_crash_leftover_tmp_in_proposals_must_not_brick() {
    let f = F::new();
    f.stage(vec![tc(&topic("A"))]).unwrap();
    fs::write(
        f.partition().join("proposals/.topics-00000000-crash.tmp"),
        b"{\"proposal\":",
    )
    .unwrap();
    let r = f.service.proposals_list(f.req());
    eprintln!("OBS {:?}", r.as_ref().map(|p| p.proposals.len()));
    assert!(r.is_ok());
    assert!(f.stage(vec![tc(&topic("B"))]).is_ok());
}
#[test]
fn p02_crash_between_mkdir_and_journal_must_not_break_history() {
    let f = F::new();
    f.accept(vec![tc(&topic("A"))]);
    fs::create_dir_all(f.partition().join("journals").join(id())).unwrap();
    let r = f.service.topics_list(f.req());
    assert!(r.is_ok());
    let h = f.service.history_list(f.req());
    eprintln!("OBS {:?}", h.as_ref().map(|h| h.operations.len()));
    assert!(h.is_ok());
}
#[test]
fn p03_recovered_approval_must_show_accepted_in_proposals_list() {
    let f = F::new();
    let t = topic("A");
    let t2 = topic("B");
    let p = f.stage(vec![tc(&t), tc(&t2)]).unwrap();
    f.service.interrupt_next_operation_after(1);
    assert!(f.approve(&p).is_err());
    assert_eq!(
        f.service.topics_list(f.req()).unwrap().topics.len(),
        2,
        "recovered"
    );
    let l = f.service.proposals_list(f.req()).unwrap().proposals;
    eprintln!("OBS status {:?}", l[0].status);
    assert_eq!(l[0].status, ProposalStatus::Accepted);
}
#[test]
fn p04_recovery_conflict_must_not_hide_previously_accepted_topic() {
    let f = F::new();
    f.write("Meetings/p.md", "secret passage\n");
    let s = f.source("Meetings/p.md");
    f.label(&s.summary, Sensitivity::LocalOnly);
    let (a, b) = (topic("A"), topic("B"));
    f.accept(vec![tc(&a), tc(&b)]);
    let before: Vec<_> = [&a, &b]
        .iter()
        .map(|t| fs::read(f.root.join(format!("topics/topics/{}.md", t.id))).unwrap())
        .collect();
    let p = f
        .stage(vec![Candidate::Connection {
            connection: f.conn(&a, &b, &s.citation),
        }])
        .unwrap();
    f.service.interrupt_next_operation_after(2);
    assert!(f.approve(&p).is_err());
    // one topic rewritten (uncommitted op id), the other still old: externally edit the remaining one
    let pa = f.root.join(format!("topics/topics/{}.md", a.id));
    let pb = f.root.join(format!("topics/topics/{}.md", b.id));
    let (rewritten, remaining, rw) = if fs::read(&pa).unwrap() != before[0] {
        (a.clone(), pb.clone(), true)
    } else {
        (b.clone(), pa.clone(), false)
    };
    eprintln!("OBS rewrote first={rw}");
    let mut bytes = fs::read(&remaining).unwrap();
    bytes.extend(b"\nexternal\n");
    fs::write(&remaining, &bytes).unwrap();
    let topics = f.service.topics_list(f.req()).unwrap().topics;
    eprintln!(
        "OBS visible topics {} ; rewritten visible={}",
        topics.len(),
        topics.iter().any(|t| t.id == rewritten.id)
    );
    assert!(
        topics.iter().any(|t| t.id == rewritten.id),
        "accepted topic vanished after recovery conflict"
    );
}
#[test]
fn p05_replayed_marker_is_idempotent_positive() {
    let f = F::new();
    let r = f.accept(vec![tc(&topic("A"))]);
    let op = r.operation.unwrap();
    let jp = f
        .partition()
        .join("journals")
        .join(&op.id)
        .join("journal.json");
    let mut j: serde_json::Value = serde_json::from_slice(&fs::read(&jp).unwrap()).unwrap();
    j["status"] = "staged".into();
    fs::write(&jp, serde_json::to_vec(&j).unwrap()).unwrap();
    assert_eq!(
        f.service
            .history_list(f.req())
            .unwrap()
            .operations
            .iter()
            .filter(|o| o.id == op.id)
            .count(),
        1
    );
}
#[test]
fn p06_app_data_loss_rebuilds_knowledge_positive() {
    let f = F::new();
    f.write("M/a.md", "alpha\n");
    let s = f.source("M/a.md");
    let t = topic("A");
    let c = f.claim(&t, &s.citation, "x");
    f.accept(vec![tc(&t), cc(&c)]);
    let tmp2 = tempfile::tempdir().unwrap();
    let mut svc = WorkspaceService::open(tmp2.path().join("ad2")).unwrap();
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
            root_id: root_id.clone(),
            topic_id: t.id.clone(),
        })
        .unwrap();
    assert_eq!(v.current_claims.len(), 1);
    assert_eq!(v.current_claims[0].revision_id, c.revision_id);
    assert_eq!(fs::read(f.root.join("M/a.md")).unwrap(), b"alpha\n");
}

// ---- stale / deleted / restored committed records ----
#[test]
fn p10_edited_local_only_label_record_must_fail_closed() {
    let f = F::new();
    f.write("Meetings/private.md", "secret\n");
    let s = f.summary("Meetings/private.md");
    f.label(&s, Sensitivity::LocalOnly);
    let rec = f.root.join(format!("topics/sources/{}.md", s.source_id));
    let mut b = fs::read(&rec).unwrap();
    b.extend(b"\nA human note added by a sync tool or editor\n");
    fs::write(&rec, &b).unwrap();
    let p = f.preview(vec!["Meetings/private.md"]);
    eprintln!(
        "OBS sources={:?} excluded={:?}",
        p.sources
            .iter()
            .map(|s| (&s.path, s.sensitivity))
            .collect::<Vec<_>>(),
        p.excluded
    );
    let overwritten = fs::read(&rec).unwrap() != b;
    eprintln!("OBS external record bytes overwritten={overwritten}");
    assert!(p.sources.is_empty(), "local-only label failed open");
    assert!(!overwritten, "external bytes overwritten");
}
#[test]
fn p11_deleted_local_only_label_record_must_fail_closed() {
    let f = F::new();
    f.write("Meetings/private.md", "secret\n");
    let s = f.summary("Meetings/private.md");
    f.label(&s, Sensitivity::LocalOnly);
    fs::remove_file(f.root.join(format!("topics/sources/{}.md", s.source_id))).unwrap();
    let p = f.preview(vec!["Meetings/private.md"]);
    eprintln!(
        "OBS {:?}",
        p.sources.iter().map(|s| s.sensitivity).collect::<Vec<_>>()
    );
    assert!(p.sources.is_empty());
}
#[test]
fn p12_restoring_older_committed_bytes_must_not_downgrade_label() {
    let f = F::new();
    f.write("Meetings/private.md", "secret\n");
    let s = f.summary("Meetings/private.md");
    let rec = f.root.join(format!("topics/sources/{}.md", s.source_id));
    let older = fs::read(&rec).unwrap();
    f.label(&s, Sensitivity::LocalOnly);
    fs::write(&rec, &older).unwrap(); // e.g. a backup/sync restore
    let p = f.preview(vec!["Meetings/private.md"]);
    eprintln!(
        "OBS {:?}",
        p.sources.iter().map(|s| s.sensitivity).collect::<Vec<_>>()
    );
    assert!(
        p.sources.is_empty(),
        "stale committed label record accepted as current"
    );
}
#[test]
fn p13_restoring_older_topic_bytes_must_warn_or_keep_local_only() {
    let f = F::new();
    let t = topic("A");
    f.accept(vec![tc(&t)]);
    let rec = f.root.join(format!("topics/topics/{}.md", t.id));
    let older = fs::read(&rec).unwrap();
    f.label_topic(&t, Sensitivity::LocalOnly);
    fs::write(&rec, &older).unwrap();
    let v = f.view(&t).unwrap();
    eprintln!(
        "OBS sens={:?} warnings={:?}",
        v.topic.sensitivity, v.warnings
    );
    assert!(v.topic.sensitivity == Sensitivity::LocalOnly || !v.warnings.is_empty());
}
#[test]
fn p14_deleted_superseding_claim_must_warn_not_silently_revive_false_claim() {
    let f = F::new();
    f.write("M/a.md", "alex he/him\n");
    let s = f.source("M/a.md");
    let t = topic("Alex");
    let c = f.claim(&t, &s.citation, "he/him");
    f.accept(vec![tc(&t), cc(&c)]);
    let (n, r) = f.correct(&t, &c, "they/them");
    f.accept(vec![Candidate::Correction {
        claim: n.clone(),
        rule: r,
    }]);
    fs::remove_file(f.root.join(format!("topics/claims/{}.md", n.revision_id))).unwrap();
    let v = f.view(&t).unwrap();
    eprintln!(
        "OBS current={:?} warnings={:?}",
        v.current_claims
            .iter()
            .map(|c| &c.value)
            .collect::<Vec<_>>(),
        v.warnings
    );
    assert!(
        !v.warnings.is_empty(),
        "committed record deleted with no warning"
    );
}

// ---- privacy ----
#[test]
fn p20_renamed_local_only_source_must_stay_restricted() {
    let f = F::new();
    f.write("Meetings/private.md", "secret\n");
    let s = f.summary("Meetings/private.md");
    f.label(&s, Sensitivity::LocalOnly);
    fs::rename(
        f.root.join("Meetings/private.md"),
        f.root.join("Meetings/renamed.md"),
    )
    .unwrap();
    let p = f.preview(vec!["Meetings/renamed.md"]);
    eprintln!(
        "OBS {:?}",
        p.sources
            .iter()
            .map(|s| (&s.path, s.sensitivity))
            .collect::<Vec<_>>()
    );
    assert!(
        p.sources.is_empty(),
        "identical bytes of a local-only source became eligible after rename"
    );
}
#[test]
fn p21_case_alias_must_not_bypass_local_only_label() {
    let f = F::new();
    f.write("Meetings/private.md", "secret\n");
    let s = f.summary("Meetings/private.md");
    f.label(&s, Sensitivity::LocalOnly);
    if !f.root.join("MEETINGS").exists() {
        eprintln!("OBS case-sensitive fs; skipped");
        return;
    }
    let p = f.preview(vec!["meetings/PRIVATE.md"]);
    eprintln!(
        "OBS {:?}",
        p.sources
            .iter()
            .map(|s| (&s.path, s.sensitivity))
            .collect::<Vec<_>>()
    );
    if !p.sources.is_empty() {
        let run_id = f.run(&p);
        let r = f
            .service
            .scope_read(ScopeReadRequest {
                root_id: f.root_id.clone(),
                scope_id: p.scope_id.clone(),
                run_id,
            })
            .unwrap();
        eprintln!("OBS model-readable content={:?}", r.sources[0].content);
    }
    assert!(p.sources.is_empty());
}
#[test]
fn p22_case_alias_must_not_expose_app_records() {
    let f = F::new();
    let t = topic("SecretPerson");
    let mut t2 = t.clone();
    t2.sensitivity = Sensitivity::LocalOnly;
    f.accept(vec![tc(&t2)]);
    if !f.root.join("TOPICS").exists() {
        eprintln!("OBS case-sensitive fs; skipped");
        return;
    }
    let path = format!("Topics/topics/{}.md", t.id);
    let p = f.preview(vec![path.as_str()]);
    eprintln!(
        "OBS file: {:?}",
        p.sources.iter().map(|s| &s.path).collect::<Vec<_>>()
    );
    let q = f.preview_spec(vec![], vec!["Topics"]).unwrap();
    eprintln!("OBS folder: {} sources", q.sources.len());
    if !p.sources.is_empty() {
        let run_id = f.run(&p);
        let r = f
            .service
            .scope_read(ScopeReadRequest {
                root_id: f.root_id.clone(),
                scope_id: p.scope_id.clone(),
                run_id,
            })
            .unwrap();
        eprintln!(
            "OBS model-readable has name={}",
            r.sources[0].content.contains("SecretPerson")
        );
    }
    assert!(p.sources.is_empty() && q.sources.is_empty());
}
#[test]
fn p23_model_must_not_self_label_new_topic_cloud_allowed() {
    let f = F::new();
    f.write("M/a.md", "alex\n");
    let s = f.source("M/a.md");
    let t = topic("Alex"); // fixture sensitivity CloudAllowed = model-supplied
    let p = f
        .model_stage(
            vec!["M/a.md"],
            vec![
                tc(&t),
                Candidate::Claim {
                    claim: f.claim(&t, &s.citation, "x"),
                },
            ],
        )
        .unwrap();
    let label = p
        .candidates
        .iter()
        .find_map(|c| {
            if let Candidate::Topic { topic } = c {
                Some(topic.sensitivity)
            } else {
                None
            }
        })
        .unwrap();
    eprintln!("OBS staged label={label:?}");
    f.approve(&p).unwrap();
    let pv = f.preview(vec!["M/a.md"]);
    let run_id = f.run(&pv);
    let ctx = f.service.source_context(SourceContextRequest {
        root_id: f.root_id.clone(),
        scope_id: pv.scope_id,
        run_id,
        topic_id: t.id.clone(),
    });
    eprintln!("OBS later model context allowed={}", ctx.is_ok());
    assert_ne!(
        label,
        Sensitivity::CloudAllowed,
        "model chose the explicit cloud-allowed label"
    );
}
#[test]
fn p24_repair_onto_local_only_path_must_not_relax_label() {
    let f = F::new();
    f.write("M/p.md", "secret v1\n");
    let sp = f.summary("M/p.md");
    f.label(&sp, Sensitivity::LocalOnly);
    f.write("M/q.md", "other\n");
    let sq = f.summary("M/q.md");
    fs::remove_file(f.root.join("M/q.md")).unwrap();
    f.write("M/p.md", "secret v2\n");
    let h = format!(
        "{:x}",
        <sha2::Sha256 as sha2::Digest>::digest(b"secret v2\n")
    );
    let r = f
        .service
        .source_identity_repair(SourceRepairRequest {
            root_id: f.root_id.clone(),
            source_id: sq.source_id,
            path: "M/p.md".into(),
            expected_hash: h,
        })
        .unwrap();
    eprintln!("OBS repaired sensitivity={:?}", r.sensitivity);
    let p = f.preview(vec!["M/p.md"]);
    eprintln!("OBS eligible={}", p.sources.len());
    assert!(p.sources.is_empty());
}
#[test]
fn p25_undo_of_repair_must_keep_local_only_label_of_tombstoned_duplicate() {
    let f = F::new();
    f.write("M/a.md", "same\n");
    let sa = f.summary("M/a.md");
    f.write("M/b.md", "same\n");
    let sb = f.summary("M/b.md");
    f.label(&sb, Sensitivity::LocalOnly);
    fs::remove_file(f.root.join("M/a.md")).unwrap();
    let h = sb.hash.clone();
    let r = f
        .service
        .source_identity_repair(SourceRepairRequest {
            root_id: f.root_id.clone(),
            source_id: sa.source_id,
            path: "M/b.md".into(),
            expected_hash: h,
        })
        .unwrap();
    assert_eq!(r.sensitivity, Sensitivity::LocalOnly);
    let ops = f.service.history_list(f.req()).unwrap().operations;
    let op = ops
        .iter()
        .find(|o| o.description.starts_with("Repair"))
        .unwrap();
    f.undo(&op.id).unwrap();
    let p = f.preview(vec!["M/b.md"]);
    eprintln!(
        "OBS eligible={:?}",
        p.sources.iter().map(|s| s.sensitivity).collect::<Vec<_>>()
    );
    assert!(p.sources.is_empty());
}
#[test]
fn p26_chat_must_stay_within_authorized_topic() {
    let f = F::new();
    f.write("M/a.md", "alpha\n");
    let s = f.source("M/a.md");
    let (a, b, c) = (topic("A"), topic("B"), topic("C"));
    let cl = f.claim(&a, &s.citation, "x");
    f.accept(vec![tc(&a), tc(&b), tc(&c), cc(&cl)]);
    let pv = f.preview(vec!["M/a.md"]);
    let ch = f
        .service
        .chat_start(ChatStartRequest {
            root_id: f.root_id.clone(),
            topic_id: a.id.clone(),
            scope_id: pv.scope_id.clone(),
            confirm_unknown: true,
            message: "hi".into(),
        })
        .unwrap();
    let r = f.service.stage_proposal(StageProposalRequest {
        root_id: f.root_id.clone(),
        scope_id: Some(pv.scope_id),
        origin: Origin::Chat {
            topic_id: a.id.clone(),
            message_id: ch.message_id,
        },
        candidates: vec![Candidate::Connection {
            connection: f.conn(&b, &c, &s.citation),
        }],
    });
    eprintln!(
        "OBS {:?}",
        r.as_ref().map(|p| p.status).map_err(|e| e.code.clone())
    );
    assert!(r.is_err(), "chat about A may propose B->C connection");
}

// ---- provenance ----
#[test]
fn p30_edited_connection_must_not_keep_unqualified_transcript_citation() {
    let f = F::new();
    f.write("M/a.md", "alex leads ledger\n");
    let s = f.source("M/a.md");
    let (a, b) = (topic("A"), topic("B"));
    f.accept(vec![tc(&a), tc(&b)]);
    let cn = f.conn(&a, &b, &s.citation);
    let p = f
        .stage(vec![Candidate::Connection {
            connection: cn.clone(),
        }])
        .unwrap();
    let mut e = cn.clone();
    e.relation = "fired_by".into();
    let r = f
        .decide(
            &p,
            Decision::Approve {
                candidates: vec![Candidate::Connection { connection: e }],
            },
        )
        .unwrap();
    let got = f.view(&a).unwrap().connections;
    eprintln!(
        "OBS relation={} sources={} uncertainty={:?} prop={:?}",
        got[0].relation,
        got[0].sources.len(),
        got[0].uncertainty,
        r.proposal.status
    );
    assert!(
        got[0].uncertainty.is_some(),
        "human edit recorded as transcript-cited with no provenance"
    );
}
#[test]
fn p31_edited_claim_subject_must_record_provenance() {
    let f = F::new();
    f.write("M/a.md", "joe left\n");
    let s = f.source("M/a.md");
    let (a, b) = (topic("Joe"), topic("Joe"));
    f.accept(vec![tc(&a), tc(&b)]);
    let c = f.claim(&a, &s.citation, "left");
    let p = f.stage(vec![cc(&c)]).unwrap();
    let mut e = c.clone();
    e.subject_id = b.id.clone();
    e.uncertainty = None;
    f.decide(
        &p,
        Decision::Approve {
            candidates: vec![cc(&e)],
        },
    )
    .unwrap();
    let v = f.view(&b).unwrap();
    eprintln!(
        "OBS sources={:?} uncertainty={:?}",
        v.current_claims[0]
            .sources
            .iter()
            .map(|s| matches!(s, Evidence::UserStatement { .. }))
            .collect::<Vec<_>>(),
        v.current_claims[0].uncertainty
    );
    assert!(
        v.current_claims[0]
            .sources
            .iter()
            .any(|s| matches!(s, Evidence::UserStatement { .. })),
        "subject re-pointed with transcript citation only"
    );
}
#[test]
fn p32_citation_heading_must_be_verified() {
    let f = F::new();
    f.write("M/a.md", "# Real\nalpha\n");
    let s = f.source("M/a.md");
    let mut c = s.citation.clone();
    c.locator.heading = Some("Forged heading".into());
    let t = topic("A");
    let r = f.model_stage(
        vec!["M/a.md"],
        vec![
            tc(&t),
            Candidate::Claim {
                claim: f.claim(&t, &c, "x"),
            },
        ],
    );
    eprintln!(
        "OBS {:?}",
        r.as_ref().map(|p| p.status).map_err(|e| e.code.clone())
    );
    assert!(r.is_err());
}

// ---- undo ----
#[test]
fn p40_undo_after_decide_allows_reanalysis_to_propose_again() {
    let f = F::new();
    f.write("M/a.md", "alpha\n");
    let s = f.source("M/a.md");
    let t = topic("A");
    let c = f.claim(&t, &s.citation, "x");
    let p = f.model_stage(vec!["M/a.md"], vec![tc(&t), cc(&c)]).unwrap();
    let op = f.approve(&p).unwrap().operation.unwrap();
    f.undo(&op.id).unwrap();
    assert!(f.service.topics_list(f.req()).unwrap().topics.is_empty());
    let (t2, mut c2) = (topic("A"), c.clone());
    c2.id = id();
    c2.revision_id = id();
    c2.subject_id = t2.id.clone();
    let q = f
        .model_stage(vec!["M/a.md"], vec![tc(&t2), cc(&c2)])
        .unwrap();
    let ls = f.service.proposals_list(f.req()).unwrap().proposals;
    eprintln!(
        "OBS re-analysis status={:?} same_id={} listed_status={:?} topics={}",
        q.status,
        q.id == p.id,
        ls.iter().map(|p| p.status).collect::<Vec<_>>(),
        f.service.topics_list(f.req()).unwrap().topics.len()
    );
    assert_eq!(
        q.status,
        ProposalStatus::Pending,
        "undone decision reported as accepted; fact cannot be re-proposed"
    );
}
#[test]
fn p41_undo_claim_with_connection_on_logical_id_must_be_refused() {
    let f = F::new();
    f.write("M/a.md", "alpha\n");
    let s = f.source("M/a.md");
    let (a, b) = (topic("A"), topic("B"));
    f.accept(vec![tc(&a), tc(&b)]);
    let c = f.claim(&a, &s.citation, "x");
    let op = f.accept(vec![cc(&c)]).operation.unwrap();
    let mut cn = f.conn(&a, &b, &s.citation);
    cn.from = Endpoint::Claim { id: c.id.clone() };
    f.accept(vec![Candidate::Connection {
        connection: cn.clone(),
    }]);
    let u = f.undo(&op.id);
    eprintln!(
        "OBS undo={:?} claims={} connection_records={}",
        u.as_ref().map(|_| "ok").map_err(|e| e.code.clone()),
        f.view(&a).unwrap().current_claims.len(),
        f.view(&b).unwrap().connections.len()
    );
    assert!(u.is_err(), "dangling connection endpoint after undo");
}
#[test]
fn p42_undo_creation_must_not_skip_live_intermediate_updates() {
    let f = F::new();
    let t = topic("A");
    let mut t0 = t.clone();
    t0.sensitivity = Sensitivity::Unknown;
    let a = f.accept(vec![tc(&t0)]).operation.unwrap();
    f.label_topic(&t, Sensitivity::LocalOnly);
    f.label_topic(&t, Sensitivity::Unknown);
    let d = f.label_topic(&t, Sensitivity::LocalOnly);
    f.undo(&d.id).unwrap();
    let u = f.undo(&a.id);
    eprintln!(
        "OBS undo creation={:?}",
        u.as_ref().map(|_| "ok").map_err(|e| e.code.clone())
    );
    assert!(u.is_err(), "two un-undone label updates bypassed");
}
#[test]
fn p43_undo_correction_positive() {
    let f = F::new();
    f.write("M/a.md", "alex he/him\n");
    let s = f.source("M/a.md");
    let t = topic("Alex");
    let c = f.claim(&t, &s.citation, "he/him");
    f.accept(vec![tc(&t), cc(&c)]);
    let (n, r) = f.correct(&t, &c, "they/them");
    let op = f
        .accept(vec![Candidate::Correction { claim: n, rule: r }])
        .operation
        .unwrap();
    assert_eq!(f.view(&t).unwrap().current_claims[0].value, "they/them");
    f.undo(&op.id).unwrap();
    let v = f.view(&t).unwrap();
    assert_eq!(v.current_claims.len(), 1);
    assert_eq!(v.current_claims[0].value, "he/him");
    assert_eq!(fs::read(f.root.join("M/a.md")).unwrap(), b"alex he/him\n");
}

// ---- proposals / dedup / identity ----
#[test]
fn p50_unrelated_label_change_must_not_permanently_conflict_pending_proposal() {
    let f = F::new();
    f.write("M/a.md", "alpha\n");
    f.write("M/z.md", "zed\n");
    let s = f.source("M/a.md");
    let z = f.summary("M/z.md");
    let t = topic("A");
    let c = f.claim(&t, &s.citation, "x");
    let p = f.stage(vec![tc(&t), cc(&c)]).unwrap();
    f.label(&z, Sensitivity::CloudAllowed); // unrelated source
    let r = f.approve(&p);
    eprintln!(
        "OBS {:?}",
        r.as_ref().map(|_| "ok").map_err(|e| e.code.clone())
    );
    let st = f.service.proposals_list(f.req()).unwrap().proposals[0].status;
    eprintln!("OBS listed status={st:?}");
    assert!(
        r.is_ok(),
        "unrelated record rewrite conflicts every pending proposal"
    );
}
#[test]
fn p51_stale_unrelated_scope_file_must_not_strand_fresh_reanalysis() {
    let f = F::new();
    f.write("M/a.md", "alpha\n");
    f.write("M/b.md", "beta\n");
    let s = f.source("M/a.md");
    f.source("M/b.md");
    let t = topic("A");
    let c = f.claim(&t, &s.citation, "x");
    let p = f
        .model_stage(vec!["M/a.md", "M/b.md"], vec![tc(&t), cc(&c)])
        .unwrap();
    f.write("M/b.md", "beta changed\n");
    let r1 = f.approve(&p);
    eprintln!(
        "OBS approve after unrelated change {:?}",
        r1.as_ref().map(|_| "ok").map_err(|e| e.code.clone())
    );
    let (t2, mut c2) = (topic("A"), c.clone());
    c2.id = id();
    c2.revision_id = id();
    c2.subject_id = t2.id.clone();
    let q = f
        .model_stage(vec!["M/a.md"], vec![tc(&t2), cc(&c2)])
        .unwrap();
    eprintln!(
        "OBS re-analysis returned old proposal={} scope_same={}",
        q.id == p.id,
        q.scope_id == p.scope_id
    );
    let r2 = f.approve(&q);
    eprintln!(
        "OBS approve fresh {:?}",
        r2.as_ref().map(|_| "ok").map_err(|e| e.code.clone())
    );
    assert!(
        r2.is_ok(),
        "stale pending proposal captures re-analysis and can never be approved"
    );
}
#[test]
fn p52_equivalent_claim_must_not_duplicate_via_different_candidate_set() {
    let f = F::new();
    f.write("M/a.md", "alpha\n");
    let s = f.source("M/a.md");
    let t = topic("A");
    let c = f.claim(&t, &s.citation, "x");
    let p = f.model_stage(vec!["M/a.md"], vec![tc(&t), cc(&c)]).unwrap();
    f.approve(&p).unwrap();
    let mut c2 = c.clone();
    c2.id = id();
    c2.revision_id = id();
    let q = f.model_stage(vec!["M/a.md"], vec![cc(&c2)]).unwrap();
    eprintln!("OBS second status={:?}", q.status);
    if q.status == ProposalStatus::Pending {
        let _ = f.approve(&q);
    }
    let v = f.view(&t).unwrap();
    eprintln!("OBS current_claims={}", v.current_claims.len());
    assert_eq!(
        v.current_claims.len(),
        1,
        "same assertion+citation accepted twice"
    );
}
#[test]
fn p53_fingerprint_stable_for_connection_to_provisional_claim() {
    let f = F::new();
    f.write("M/a.md", "alpha\n");
    let s = f.source("M/a.md");
    let mk = || {
        let (t, u) = (topic("A"), topic("B"));
        let c = f.claim(&t, &s.citation, "x");
        let mut n = f.conn(&t, &u, &s.citation);
        n.from = Endpoint::Claim {
            id: c.revision_id.clone(),
        };
        vec![
            tc(&t),
            tc(&u),
            cc(&c),
            Candidate::Connection { connection: n },
        ]
    };
    let (a, b) = (f.stage(mk()).unwrap(), f.stage(mk()).unwrap());
    eprintln!("OBS equal={}", a.fingerprint == b.fingerprint);
    assert_eq!(a.fingerprint, b.fingerprint);
}
#[test]
fn p54_fingerprint_must_distinguish_same_name_identities() {
    let f = F::new();
    f.write("M/a.md", "alpha\n");
    let s = f.source("M/a.md");
    let l = topic("Ledger");
    let mk = |first: bool| {
        let (j1, j2) = (topic("Joe"), {
            let mut x = topic("Joe");
            x.id = id();
            x
        });
        let n = f.conn(if first { &j1 } else { &j2 }, &l, &s.citation);
        vec![
            tc(&j1),
            tc(&j2),
            tc(&l),
            Candidate::Connection { connection: n },
        ]
    };
    let _ = &mk;
    let a = f.stage(mk(true)).unwrap();
    let b = f.stage(mk(false)).unwrap();
    // l is shared so its id normalizes identically; j1/j2 are indistinguishable by construction here.
    eprintln!(
        "OBS equal={} (informational: both Joes hash identically)",
        a.fingerprint == b.fingerprint
    );
    assert_ne!(
        a.fingerprint, b.fingerprint,
        "ambiguous same-name identity graphs cannot be safely collapsed"
    );
}
#[test]
fn p55_second_real_person_same_name_can_still_be_proposed_by_model() {
    let f = F::new();
    f.write("M/a.md", "alpha\n");
    f.source("M/a.md");
    let j1 = topic("Joe");
    let p = f.model_stage(vec!["M/a.md"], vec![tc(&j1)]).unwrap();
    f.approve(&p).unwrap();
    let j2 = topic("Joe");
    let q = f.model_stage(vec!["M/a.md"], vec![tc(&j2)]).unwrap();
    eprintln!("OBS second Joe status={:?}", q.status);
    assert_eq!(q.status, ProposalStatus::Pending);
}
#[test]
fn p56_model_claim_on_ambiguous_same_name_must_be_unresolved() {
    let f = F::new();
    f.write("M/a.md", "joe left\n");
    let s = f.source("M/a.md");
    let (j1, j2) = (topic("Joe"), topic("Joe"));
    f.accept(vec![tc(&j1), tc(&j2)]);
    let c = f.claim(&j1, &s.citation, "left");
    let p = f.model_stage(vec!["M/a.md"], vec![cc(&c)]).unwrap();
    let kinds: Vec<_> = p
        .candidates
        .iter()
        .map(|c| matches!(c, Candidate::UnresolvedIdentity { .. }))
        .collect();
    eprintln!("OBS unresolved={kinds:?}");
    assert!(kinds.iter().all(|k| *k));
}
#[test]
fn p57_new_provisional_same_name_with_connection_must_be_unresolved() {
    let f = F::new();
    f.write("M/a.md", "joe leads ledger\n");
    let s = f.source("M/a.md");
    let (j1, j2, l) = (topic("Joe"), topic("Joe"), topic("Ledger"));
    f.accept(vec![tc(&j1), tc(&j2), tc(&l)]);
    let j3 = topic("Joe");
    let p = f
        .model_stage(
            vec!["M/a.md"],
            vec![
                tc(&j3),
                Candidate::Connection {
                    connection: f.conn(&j3, &l, &s.citation),
                },
            ],
        )
        .unwrap();
    let conn = p
        .candidates
        .iter()
        .any(|c| matches!(c, Candidate::Connection { .. }));
    eprintln!("OBS connection kept={conn}");
    assert!(!conn);
}
#[test]
fn p58_suppressed_only_output_must_not_fail_the_run() {
    let f = F::new();
    f.write("M/a.md", "alex he/him\n");
    let s = f.source("M/a.md");
    let t = topic("Alex");
    let c = f.claim(&t, &s.citation, "he/him");
    f.accept(vec![tc(&t), cc(&c)]);
    let (n, r) = f.correct(&t, &c, "they/them");
    f.accept(vec![Candidate::Correction { claim: n, rule: r }]);
    let mut again = c.clone();
    again.id = id();
    again.revision_id = id();
    again.uncertainty = Some("slightly different wording".into());
    let p = f.preview(vec!["M/a.md"]);
    let run_id = f.run(&p);
    let req = StageProposalRequest {
        root_id: f.root_id.clone(),
        scope_id: Some(p.scope_id),
        origin: Origin::Analysis { run_id },
        candidates: vec![cc(&again)],
    };
    let v = f.service.validate_proposal(req.clone());
    eprintln!(
        "OBS validate={:?}",
        v.as_ref().map_err(|e| (e.code.clone(), e.message.clone()))
    );
    let cur = f.view(&t).unwrap().current_claims;
    assert_eq!(cur[0].value, "they/them");
    assert!(v.is_ok(), "rule suppression surfaces as a model/run error");
}
#[test]
fn p59_rule_positive_other_person_and_replacement_not_auto_accepted() {
    let f = F::new();
    f.write("M/a.md", "alex he/him joe he/him\n");
    let s = f.source("M/a.md");
    let (t, o) = (topic("Alex"), topic("Joe"));
    let c = f.claim(&t, &s.citation, "he/him");
    f.accept(vec![tc(&t), tc(&o), cc(&c)]);
    let (n, r) = f.correct(&t, &c, "they/them");
    f.accept(vec![Candidate::Correction { claim: n, rule: r }]);
    let oc = f.claim(&o, &s.citation, "he/him");
    let p = f.model_stage(vec!["M/a.md"], vec![cc(&oc)]).unwrap();
    assert_eq!(p.candidates.len(), 1);
    f.approve(&p).unwrap();
    assert_eq!(f.view(&o).unwrap().current_claims[0].value, "he/him");
    assert_eq!(f.view(&t).unwrap().current_claims.len(), 1);
}

#[test]
fn a_distinct_same_name_person_with_a_new_citation_is_not_merged_by_shared_predicate_value() {
    let f = F::new();
    f.write("M/first.md", "Joe one is a lead\n");
    let one = f.source("M/first.md");
    let first = topic("Joe");
    let claim = f.claim(&first, &one.citation, "lead");
    f.accept(vec![tc(&first), cc(&claim)]);
    f.write("M/second.md", "Joe two is a lead\n");
    let two = f.source("M/second.md");
    let second = topic("Joe");
    let next = f.claim(&second, &two.citation, "lead");
    let proposal = f
        .model_stage(vec!["M/second.md"], vec![tc(&second), cc(&next)])
        .unwrap();
    assert!(
        proposal
            .candidates
            .iter()
            .any(|c| matches!(c,Candidate::Topic{topic} if topic.id==second.id))
    );
    assert!(
        !proposal
            .candidates
            .iter()
            .any(|c| matches!(c,Candidate::Claim{claim} if claim.subject_id==first.id))
    );
}
#[test]
fn shared_passage_and_relation_do_not_merge_a_new_person_through_a_different_endpoint() {
    let f = F::new();
    f.write("M/a.md", "Joe one leads A; Joe two leads B\n");
    let source = f.source("M/a.md");
    let first = topic("Joe");
    let a = topic("A");
    let b = topic("B");
    f.accept(vec![
        tc(&first),
        tc(&a),
        tc(&b),
        Candidate::Connection {
            connection: f.conn(&first, &a, &source.citation),
        },
    ]);
    let second = topic("Joe");
    let proposal = f
        .model_stage(
            vec!["M/a.md"],
            vec![
                tc(&second),
                Candidate::Connection {
                    connection: f.conn(&second, &b, &source.citation),
                },
            ],
        )
        .unwrap();
    assert!(
        proposal
            .candidates
            .iter()
            .any(|c| matches!(c,Candidate::Topic{topic} if topic.id==second.id))
    );
    assert!(!proposal.candidates.iter().any(|c|matches!(c,Candidate::Connection{connection} if connection.from==Endpoint::Topic{id:first.id.clone()})));
}

#[test]
fn approving_a_dependency_consistent_subset_ignores_changed_omitted_dependencies() {
    let f = F::new();
    f.write("M/a.md", "A\n");
    f.write("M/b.md", "B\n");
    let sa = f.source("M/a.md");
    let sb = f.source("M/b.md");
    let a = topic("A");
    let b = topic("B");
    let ca = f.claim(&a, &sa.citation, "a");
    let cb = f.claim(&b, &sb.citation, "b");
    let proposal = f.stage(vec![tc(&a), cc(&ca), tc(&b), cc(&cb)]).unwrap();
    f.label(&sb.summary, Sensitivity::CloudAllowed);
    let result = f.decide(
        &proposal,
        Decision::Approve {
            candidates: vec![tc(&a), cc(&ca)],
        },
    );
    assert!(
        result.is_ok(),
        "omitted candidate dependencies must not strand a valid human-reviewed subset"
    );
    assert_eq!(f.service.topics_list(f.req()).unwrap().topics.len(), 1);
}
#[test]
fn locator_heading_is_checked_on_source_endpoints_too() {
    let f = F::new();
    f.write("M/a.md", "# Real heading\nActual passage\n");
    let source = f.source("M/a.md");
    let a = topic("A");
    let mut connection = f.conn(&a, &a, &source.citation);
    let mut locator = source.citation.locator.clone();
    locator.heading = Some("Forged endpoint heading".into());
    connection.from = Endpoint::Source {
        source_id: source.summary.source_id,
        locator,
    };
    assert!(
        f.model_stage(
            vec!["M/a.md"],
            vec![tc(&a), Candidate::Connection { connection }]
        )
        .is_err()
    );
}
#[test]
fn human_legacy_search_excludes_app_records_and_keeps_source_search_useful() {
    let f = F::new();
    f.write("M/private.md", "HumanVisibleSourceWord\n");
    let source = f.summary("M/private.md");
    f.label(&source, Sensitivity::LocalOnly);
    let mut t = topic("GeneratedSecretWord");
    t.sensitivity = Sensitivity::LocalOnly;
    f.accept(vec![tc(&t)]);
    let human = topics_store::TopicsService::open(f.temp.path().join("app-data")).unwrap();
    assert!(human.search("GeneratedSecretWord").unwrap().is_empty());
    assert_eq!(human.search("HumanVisibleSourceWord").unwrap().len(), 1);
}

#[test]
fn recovery_conflict_invalidates_pending_dedupe_and_fresh_analysis_can_continue() {
    let f = F::new();
    f.write("M/a.md", "A source\n");
    let source = f.source("M/a.md");
    let a = topic("A");
    let claim = f.claim(&a, &source.citation, "x");
    let p = f.stage(vec![tc(&a), cc(&claim)]).unwrap();
    f.service.interrupt_next_operation_after(1);
    assert!(f.approve(&p).is_err());
    fs::write(
        f.root
            .join(format!("topics/claims/{}.md", claim.revision_id)),
        "# External conflicting draft\n",
    )
    .unwrap();
    f.service.topics_list(f.req()).unwrap();
    let old = f
        .service
        .proposals_list(f.req())
        .unwrap()
        .proposals
        .into_iter()
        .find(|v| v.id == p.id)
        .unwrap();
    assert_eq!(old.status, ProposalStatus::Conflict);
    let b = topic("A");
    let next = f.claim(&b, &source.citation, "x");
    let fresh = f
        .model_stage(vec!["M/a.md"], vec![tc(&b), cc(&next)])
        .unwrap();
    assert_ne!(fresh.id, p.id);
    assert_eq!(fresh.status, ProposalStatus::Pending);
    f.approve(&fresh).unwrap();
    assert_eq!(f.view(&b).unwrap().current_claims.len(), 1);
}

#[test]
fn inventory_registration_does_not_overwrite_an_untracked_record_target() {
    let f = F::new();
    let content = b"Synthetic source bytes\n";
    f.write("M/a.md", content);
    let source = f.summary("M/a.md");
    // Synthetic loss of ownership metadata leaves an existing target requiring review.
    for operation in f.service.history_list(f.req()).unwrap().operations {
        fs::remove_file(
            f.root
                .join(format!("topics/operations/{}.md", operation.id)),
        )
        .unwrap();
    }
    fs::remove_dir_all(f.partition().join("journals")).unwrap();
    let path = format!("topics/sources/{}.md", source.source_id);
    let external = b"# External app record requiring review\nlocal_only\n";
    f.write(&path, external);
    let result = f.preview_spec(vec!["M/a.md"], vec![]);
    assert!(result.is_err());
    assert_eq!(fs::read(f.root.join(path)).unwrap(), external);
    assert_eq!(fs::read(f.root.join("M/a.md")).unwrap(), content);
}

#[test]
fn guarded_staging_can_be_revoked_after_preparation_without_publishing() {
    use std::{
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
            mpsc,
        },
        time::Duration,
    };
    let f = F::new();
    f.write("M/a.md", "Synthetic source for guarded publication\n");
    let preview = f.preview(vec!["M/a.md"]);
    let run_id = f.run(&preview);
    let request = StageProposalRequest {
        root_id: f.root_id.clone(),
        scope_id: Some(preview.scope_id),
        origin: Origin::Analysis { run_id },
        candidates: vec![tc(&topic("Never published"))],
    };
    let service = WorkspaceService::open(f.temp.path().join("app-data")).unwrap();
    let permitted = Arc::new(AtomicBool::new(true));
    let worker_permit = permitted.clone();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    std::thread::scope(|threads| {
        let worker = threads.spawn(move || {
            service.stage_proposal_guarded(request, |proposal, action| {
                entered_tx.send(proposal.id.clone()).unwrap();
                if release_rx.recv_timeout(Duration::from_secs(3)).is_err()
                    || !worker_permit.load(Ordering::SeqCst)
                {
                    return Err(ApiError::new("cancelled", "Publication permit revoked"));
                }
                action()
            })
        });
        let proposal_id = entered_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        let directory = f.partition().join("proposals");
        assert!(!directory.join(format!("{proposal_id}.json")).exists());
        let temporary = fs::read_dir(&directory)
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| p.extension().is_some_and(|ext| ext == "tmp"))
            .unwrap();
        let staged: serde_json::Value =
            serde_json::from_slice(&fs::read(temporary).unwrap()).unwrap();
        assert_eq!(staged["proposal"]["id"], proposal_id);
        permitted.store(false, Ordering::SeqCst);
        release_tx.send(()).unwrap();
        assert_eq!(worker.join().unwrap().unwrap_err().code, "cancelled");
    });
    assert!(
        f.service
            .proposals_list(f.req())
            .unwrap()
            .proposals
            .is_empty()
    );
    assert!(f.service.topics_list(f.req()).unwrap().topics.is_empty());
    assert_eq!(
        fs::read_dir(f.partition().join("proposals"))
            .unwrap()
            .count(),
        0
    );
}
#[test]
fn guarded_success_duplicate_and_no_change_outcomes_all_use_the_publication_gate() {
    let f = F::new();
    f.write("M/a.md", "Synthetic source\n");
    let preview = f.preview(vec!["M/a.md"]);
    let topic = topic("Guarded topic");
    let request = |run_id| StageProposalRequest {
        root_id: f.root_id.clone(),
        scope_id: Some(preview.scope_id.clone()),
        origin: Origin::Analysis { run_id },
        candidates: vec![tc(&topic)],
    };
    let mut callbacks = 0;
    let first = f
        .service
        .stage_proposal_guarded(request(f.run(&preview)), |proposal, action| {
            callbacks += 1;
            assert_eq!(proposal.status, ProposalStatus::Pending);
            action()?;
            action()
        })
        .unwrap();
    assert_eq!(callbacks, 1);
    assert_eq!(
        f.service.proposals_list(f.req()).unwrap().proposals.len(),
        1
    );
    let error = f
        .service
        .stage_proposal_guarded(request(f.run(&preview)), |proposal, _| {
            callbacks += 1;
            assert_eq!(proposal.id, first.id);
            Err(ApiError::new("cancelled", "Duplicate outcome revoked"))
        })
        .unwrap_err();
    assert_eq!(error.code, "cancelled");
    assert_eq!(callbacks, 2);
    assert_eq!(
        f.service.proposals_list(f.req()).unwrap().proposals.len(),
        1
    );
    f.approve(&first).unwrap();
    let operations = f.service.history_list(f.req()).unwrap().operations.len();
    let before = f.service.proposals_list(f.req()).unwrap().proposals.len();
    let error = f
        .service
        .stage_proposal_guarded(request(f.run(&preview)), |proposal, _| {
            assert_eq!(proposal.status, ProposalStatus::Accepted);
            assert!(proposal.candidates.is_empty());
            Err(ApiError::new("cancelled", "No-change outcome revoked"))
        })
        .unwrap_err();
    assert_eq!(error.code, "cancelled");
    assert_eq!(
        f.service.proposals_list(f.req()).unwrap().proposals.len(),
        before
    );
    let mut published_ids = vec![];
    let no_change = f
        .service
        .stage_proposal_guarded(request(f.run(&preview)), |proposal, action| {
            action()?;
            published_ids.push(proposal.id.clone());
            Ok(())
        })
        .unwrap();
    assert_eq!(published_ids, vec![no_change.id.clone()]);
    assert!(no_change.candidates.is_empty());
    assert!(f.approve(&no_change).unwrap().operation.is_none());
    assert_eq!(
        f.service.history_list(f.req()).unwrap().operations.len(),
        operations
    );
}
