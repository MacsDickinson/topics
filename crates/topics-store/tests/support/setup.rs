//! Synthetic setup fixture shared by explicit BDD and boundary regression tests.
#![allow(dead_code)]
use std::{collections::BTreeMap, fs, path::PathBuf};
use topics_core::*;
use topics_store::WorkspaceService;
pub fn id() -> Id {
    uuid::Uuid::new_v4().to_string()
}
pub fn topic(name: &str, kind: TopicKind) -> Topic {
    Topic {
        id: id(),
        name: name.into(),
        topic_kind: kind,
        aliases: vec![],
        sensitivity: Sensitivity::Unknown,
    }
}
pub fn claim(t: &Topic, predicate: &str, value: &str, s: &SourceRef) -> Claim {
    Claim {
        id: id(),
        revision_id: id(),
        subject_id: t.id.clone(),
        predicate: predicate.into(),
        value: value.into(),
        uncertainty: None,
        sources: vec![Evidence::Passage { source: s.clone() }],
        supersedes: vec![],
        recorded_at: "2026-11-09T00:00:00Z".into(),
        valid_from: None,
        valid_to: None,
    }
}
pub fn relation(a: &Topic, b: &Topic, s: &SourceRef) -> Connection {
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
pub struct Fixture {
    pub temp: tempfile::TempDir,
    pub app: PathBuf,
    pub root: PathBuf,
    pub service: WorkspaceService,
    pub root_id: Id,
    pub manifest: Option<SetupManifest>,
    pub job: Option<SetupJob>,
    pub cap: Option<SetupBatchCapability>,
    pub receipt: Option<SetupReceipt>,
    pub result: Option<SetupDecisionResult>,
    pub original: BTreeMap<String, Vec<u8>>,
    pub topics: Vec<Topic>,
    pub claims: Vec<Claim>,
    pub last_error: Option<ApiError>,
    pub save_actions: u32,
    pub correction: Option<SetupCorrectionPreview>,
    pub seen: Vec<ScopedSource>,
}
impl Default for Fixture {
    fn default() -> Self {
        Self::new()
    }
}
impl Fixture {
    pub fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let app = temp.path().join("app-data");
        let root = temp.path().join("notes");
        assert!(app.is_absolute());
        fs::create_dir(&root).unwrap();
        let mut service = WorkspaceService::open(&app).unwrap();
        let root_id = service
            .root_set(RootSetRequest {
                path: root.to_str().unwrap().into(),
                create: false,
            })
            .unwrap()
            .root
            .root_id;
        Self {
            temp,
            app,
            root,
            service,
            root_id,
            manifest: None,
            job: None,
            cap: None,
            receipt: None,
            result: None,
            original: BTreeMap::new(),
            topics: vec![],
            claims: vec![],
            last_error: None,
            save_actions: 0,
            correction: None,
            seen: vec![],
        }
    }
    pub fn root_request(&self) -> RootRequest {
        RootRequest {
            root_id: self.root_id.clone(),
        }
    }
    pub fn write(&mut self, path: &str, text: &str) {
        let p = self.root.join(path);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
        self.original
            .entry(path.into())
            .or_insert_with(|| text.as_bytes().to_vec());
    }
    pub fn unchanged(&self) {
        for (path, bytes) in &self.original {
            assert_eq!(
                &fs::read(self.root.join(path)).unwrap(),
                bytes,
                "source {path} changed"
            );
        }
    }
    pub fn inventory(&self) -> SetupInventory {
        self.service
            .setup_inventory(SetupInventoryRequest {
                root_id: self.root_id.clone(),
                roles: vec![],
            })
            .unwrap()
    }
    pub fn citation(&self, path: &str) -> SourceRef {
        let s = self
            .inventory()
            .documents
            .into_iter()
            .find(|d| d.source.path == path)
            .unwrap()
            .source;
        self.service
            .source_get(SourceGetRequest {
                root_id: self.root_id.clone(),
                source_id: s.source_id,
            })
            .unwrap()
            .citation
    }
    pub fn preview(&mut self, files: u32) {
        self.manifest = Some(
            self.service
                .setup_manifest_preview(SetupManifestPreviewRequest {
                    root_id: self.root_id.clone(),
                    spec: ScopeSpec {
                        files: vec![],
                        folders: vec![],
                        date_from: None,
                        date_to: None,
                    },
                    roles: vec![],
                    limits: SetupLimits {
                        max_files_per_batch: files,
                        max_bytes_per_batch: 2 * 1024 * 1024,
                        max_batches: 200,
                        max_candidates_per_turn: 200,
                        max_turns_per_batch: 4,
                    },
                    runtime: "synthetic-no-provider".into(),
                })
                .unwrap(),
        );
    }
    pub fn authorize(&mut self) {
        let m = self.manifest.as_ref().unwrap();
        self.job = Some(
            self.service
                .setup_job_create(SetupJobCreateRequest {
                    root_id: self.root_id.clone(),
                    manifest_id: m.manifest_id.clone(),
                    expected_fingerprint: m.fingerprint.clone(),
                })
                .unwrap(),
        );
    }
    pub fn refresh(&mut self) {
        let id = self.job.as_ref().unwrap().job_id.clone();
        self.job = Some(
            self.service
                .setup_job_get(SetupJobRequest {
                    root_id: self.root_id.clone(),
                    job_id: id,
                })
                .unwrap(),
        );
    }
    pub fn start(&mut self) -> SetupBatchCapability {
        self.refresh();
        let j = self.job.as_ref().unwrap();
        let b = j
            .batches
            .iter()
            .find(|b| {
                matches!(
                    b.status,
                    SetupBatchStatus::Waiting | SetupBatchStatus::Cancelled
                )
            })
            .unwrap();
        let cap = self
            .service
            .setup_batch_start(SetupBatchStartRequest {
                root_id: self.root_id.clone(),
                job_id: j.job_id.clone(),
                batch_id: b.batch_id.clone(),
            })
            .unwrap();
        self.cap = Some(cap.clone());
        self.refresh();
        cap
    }
    pub fn read(&mut self, cap: &SetupBatchCapability) -> Vec<ScopedSource> {
        let sources = self
            .service
            .scope_read(ScopeReadRequest {
                root_id: cap.root_id.clone(),
                scope_id: cap.scope_id.clone(),
                run_id: cap.run_id.clone(),
            })
            .unwrap()
            .sources;
        self.seen.extend(sources.clone());
        sources
    }
    pub fn publish(&mut self, cap: SetupBatchCapability, candidates: Vec<Candidate>) {
        let request = SetupBatchStageRequest {
            capability: cap,
            candidates,
        };
        self.service.setup_batch_validate(request.clone()).unwrap();
        self.job = Some(
            self.service
                .setup_batch_stage_guarded(request, |_, action| action())
                .unwrap(),
        );
    }
    pub fn finish_one(&mut self, with_issues: bool) {
        let cap = self.start();
        let sources = self.read(&cap);
        let a = topic("Alex", TopicKind::Person);
        let b = topic("Ledger", TopicKind::Project);
        let mut candidates = vec![
            Candidate::Topic { topic: a.clone() },
            Candidate::Topic { topic: b.clone() },
        ];
        for s in &sources {
            candidates.push(Candidate::Claim {
                claim: claim(&a, "role", "Ledger lead", &s.citation),
            });
            candidates.push(Candidate::Connection {
                connection: relation(&a, &b, &s.citation),
            });
        }
        if with_issues {
            let joe = topic("Joe", TopicKind::Person);
            let other = topic("Joe", TopicKind::Person);
            candidates.push(Candidate::Topic { topic: joe.clone() });
            candidates.push(Candidate::Topic {
                topic: other.clone(),
            });
            candidates.push(Candidate::UnresolvedIdentity {
                name: "Joe".into(),
                candidates: vec![joe.id, other.id],
                sources: sources.iter().map(|s| s.citation.clone()).collect(),
            });
        }
        self.publish(cap, candidates);
    }
    pub fn finish(&mut self) {
        loop {
            self.refresh();
            if !self.job.as_ref().unwrap().batches.iter().any(|b| {
                matches!(
                    b.status,
                    SetupBatchStatus::Waiting | SetupBatchStatus::Cancelled
                )
            }) {
                break;
            }
            self.finish_one(false);
        }
    }
    pub fn prepare_default(&mut self, count: usize) {
        for i in 0..count {
            self.write(
                &format!("odd/2026-11-{:02}-{i}.md", i % 9 + 1),
                &format!("# Report {i}\nAlex leads Ledger\n"),
            );
        }
        self.preview(1);
        self.authorize();
    }
    pub fn all_receipt(&mut self) {
        self.refresh();
        let j = self.job.as_ref().unwrap();
        self.receipt = Some(
            self.service
                .setup_save_preview(SetupSavePreviewRequest {
                    root_id: self.root_id.clone(),
                    job_id: j.job_id.clone(),
                    item_ids: vec![],
                })
                .unwrap(),
        );
    }
    pub fn decide(&mut self) {
        let r = self.receipt.as_ref().unwrap();
        let result = self
            .service
            .setup_decide(SetupDecisionRequest {
                root_id: self.root_id.clone(),
                receipt_id: r.receipt_id.clone(),
                expected_fingerprint: r.fingerprint.clone(),
            })
            .unwrap();
        assert_eq!(result.status, SetupDecisionStatus::Completed, "{result:?}");
        self.result = Some(result);
        self.save_actions += 1;
        self.refresh_if_job();
    }
    fn refresh_if_job(&mut self) {
        if self.job.is_some() {
            self.refresh()
        }
    }
    pub fn saved(&self) -> Vec<Topic> {
        self.service
            .topics_list(self.root_request())
            .unwrap()
            .topics
    }
    pub fn review(&self, t: &Topic) -> SetupTopicReview {
        self.service
            .setup_topic_review(TopicGetRequest {
                root_id: self.root_id.clone(),
                topic_id: t.id.clone(),
            })
            .unwrap()
    }
    pub fn single(&mut self, claims: usize) {
        self.write(
            "meeting.md",
            "# Transcript\nAlex uses he/him and leads Ledger.\n",
        );
        self.preview(1);
        self.authorize();
        let cap = self.start();
        let s = self.read(&cap)[0].citation.clone();
        let a = topic("Alex", TopicKind::Person);
        let mut cs = vec![Candidate::Topic { topic: a.clone() }];
        self.topics = vec![a.clone()];
        self.claims.clear();
        for i in 0..claims {
            let c = claim(
                &a,
                if i == 0 {
                    "pronouns"
                } else {
                    match i {
                        1 => "team",
                        2 => "location",
                        _ => "status",
                    }
                },
                if i == 0 { "he/him" } else { "reported" },
                &s,
            );
            self.claims.push(c.clone());
            cs.push(Candidate::Claim { claim: c });
        }
        self.publish(cap, cs);
        self.all_receipt();
        self.decide();
    }
    pub fn multiple_pronouns(&mut self) {
        self.write("meeting.md","# Transcript\nAlex uses he/him, repeated across historic accounts. Alex leads Ledger.\n");
        self.preview(1);
        self.authorize();
        let cap = self.start();
        let s = self.read(&cap)[0].citation.clone();
        let a = topic("Alex", TopicKind::Person);
        let mut first = claim(&a, "pronouns", "he/him", &s);
        first.valid_from = Some("2025-01-01".into());
        let mut second = claim(&a, "pronouns", "he/him", &s);
        second.valid_from = Some("2026-01-01".into());
        let other = claim(&a, "team", "Ledger", &s);
        self.claims = vec![first.clone(), second.clone(), other.clone()];
        self.topics = vec![a.clone()];
        self.publish(
            cap,
            vec![
                Candidate::Topic { topic: a },
                Candidate::Claim { claim: first },
                Candidate::Claim { claim: second },
                Candidate::Claim { claim: other },
            ],
        );
        self.all_receipt();
        self.decide();
    }
    pub fn correct(&mut self) {
        let t = self.topics[0].clone();
        let ids = self
            .review(&t)
            .topic
            .current_claims
            .into_iter()
            .filter(|c| c.predicate == "pronouns")
            .map(|c| c.revision_id)
            .collect();
        let preview = self
            .service
            .setup_correction_preview(SetupCorrectionPreviewRequest {
                root_id: self.root_id.clone(),
                topic_id: t.id,
                revision_ids: ids,
                predicate: "pronouns".into(),
                replacement: "they/them".into(),
                statement: "Alex uses they/them".into(),
                rule_scope: RuleScope {
                    source_paths: vec!["meeting.md".into()],
                    context: None,
                },
            })
            .unwrap();
        self.receipt = Some(preview.receipt.clone());
        self.correction = Some(preview);
    }
    pub fn control(&mut self, action: SetupJobAction) {
        self.refresh();
        let j = self.job.as_ref().unwrap();
        self.job = Some(
            self.service
                .setup_job_control(SetupJobControlRequest {
                    root_id: self.root_id.clone(),
                    job_id: j.job_id.clone(),
                    expected_revision: j.revision,
                    action,
                })
                .unwrap(),
        );
    }
    pub fn restart(&mut self) {
        self.service = WorkspaceService::open(&self.app).unwrap();
        self.refresh_if_job();
    }
}
