#[path = "support/setup.rs"]
mod support;
use std::collections::BTreeSet;
use support::*;
use topics_core::*;

#[test]
fn source_discovery_judgment_reopen_and_undo() {
    let mut f = Fixture::new();
    f.write("plan.md", "The Atlas launch is Friday. Robin leads Atlas.");
    f.write("minutes.md", "The Atlas launch is Monday.");
    let scope = f
        .service
        .scope_preview(ScopePreviewRequest {
            root_id: f.root_id.clone(),
            spec: ScopeSpec {
                files: vec!["plan.md".into(), "minutes.md".into()],
                folders: vec![],
                date_from: None,
                date_to: None,
            },
        })
        .unwrap();
    let run = f
        .service
        .analysis_start(AnalysisStartRequest {
            root_id: f.root_id.clone(),
            scope_id: scope.scope_id.clone(),
            confirm_unknown: true,
        })
        .unwrap();
    let sources = f
        .service
        .scope_read(ScopeReadRequest {
            root_id: f.root_id.clone(),
            scope_id: scope.scope_id.clone(),
            run_id: run.run_id.clone(),
        })
        .unwrap()
        .sources;
    let a = topic("Atlas", TopicKind::Project);
    let b = topic("Robin", TopicKind::Person);
    let mut candidates = vec![
        Candidate::Topic { topic: a.clone() },
        Candidate::Topic { topic: b.clone() },
    ];
    for source in &sources {
        let value = if source.citation.path == "plan.md" {
            "Friday"
        } else {
            "Monday"
        };
        candidates.push(Candidate::Claim {
            claim: claim(&a, "launch", value, &source.citation),
        });
    }
    candidates.push(Candidate::Connection {
        connection: relation(&b, &a, &sources[0].citation),
    });
    let proposal = f
        .service
        .stage_proposal(StageProposalRequest {
            root_id: f.root_id.clone(),
            scope_id: Some(scope.scope_id),
            origin: Origin::Analysis { run_id: run.run_id },
            candidates,
        })
        .unwrap();
    let graph = f.service.knowledge_graph(f.root_request()).unwrap();
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges[0].status, KnowledgeStatus::Pending);
    assert_eq!(graph.pending[0].decision.id, proposal.id);
    f.service
        .proposal_decide(ProposalDecideRequest {
            root_id: f.root_id.clone(),
            proposal_id: proposal.id,
            expected_fingerprint: proposal.fingerprint,
            decision: Decision::Approve {
                candidates: proposal.candidates,
            },
        })
        .unwrap();
    let root = f.root_request();
    let hit = f
        .service
        .topic_search(TopicSearchRequest {
            root_id: root.root_id.clone(),
            query: "Atlas".into(),
            limit: None,
        })
        .unwrap()
        .hits
        .remove(0);
    assert_eq!(hit.status, KnowledgeStatus::Accepted);
    let req = TopicGetRequest {
        root_id: root.root_id.clone(),
        topic_id: hit.topic.id.clone(),
    };
    let view = f.service.topic_knowledge(req.clone()).unwrap();
    assert_eq!(view.relationships[0].other.name, "Robin");
    assert_eq!(view.checks.conflicting, 1);
    assert_eq!(view.assertions.len(), 2);
    for a in &view.assertions {
        assert!(
            matches!(&a.evidence[0],AssertionEvidence::Passage{source,stale:false} if source.excerpt.contains(&a.value))
        );
    }
    let check = view.validation[0].clone();
    f.service
        .validation_defer(ValidationDeferRequest {
            root_id: f.root_id.clone(),
            topic_id: hit.topic.id.clone(),
            item_id: check.id.clone(),
            deferred: true,
        })
        .unwrap();
    f.restart();
    assert_eq!(
        f.service
            .knowledge_graph(root.clone())
            .unwrap()
            .checks
            .deferred,
        1
    );
    for action in [
        ValidationAction::Confirm {
            id: view.assertions[0].revision_id.clone(),
        },
        ValidationAction::Correct {
            value: "Thursday".into(),
        },
    ] {
        let p = f
            .service
            .validation_preview(ValidationPreviewRequest {
                root_id: f.root_id.clone(),
                topic_id: hit.topic.id.clone(),
                item_id: check.id.clone(),
                action,
                note: String::new(),
            })
            .unwrap();
        assert!(p.statement.is_some());
        assert_eq!(
            f.service
                .topic_knowledge(req.clone())
                .unwrap()
                .validation
                .len(),
            1
        );
        f.receipt = Some(p.receipt);
        f.decide();
        f.restart();
        let updated = f.service.topic_knowledge(req.clone()).unwrap();
        assert!(updated.validation.is_empty());
        assert!(
            updated
                .assertions
                .iter()
                .all(|a| a.review == ReviewState::ConfirmedByYou)
        );
        assert!(!updated.earlier.is_empty());
        assert!(updated.assertions.iter().any(|a| {
            a.evidence
                .iter()
                .any(|e| matches!(e, AssertionEvidence::Statement { .. }))
        }));
        f.service
            .undo(UndoRequest {
                root_id: f.root_id.clone(),
                operation_id: f.result.as_ref().unwrap().operations[0].id.clone(),
            })
            .unwrap();
        assert_eq!(
            f.service.topic_knowledge(req.clone()).unwrap().assertions,
            view.assertions
        );
    }
    f.unchanged();
}

#[test]
fn retrieval_requires_folder_topic_and_exact_current_released_evidence() {
    let mut f = Fixture::new();
    f.prepare_default(1);
    f.finish();
    f.all_receipt();
    f.decide();
    let topic = f.saved().into_iter().find(|t| t.name == "Alex").unwrap();
    let allow = BTreeSet::from([topic.id.clone()]);
    let q = RetrievalTopicRequest {
        topic_id: topic.id.clone(),
    };
    let get = |f: &Fixture| f.service.retrieval_topic(&f.root_id, &allow, q.clone());
    assert!(get(&f).is_err());
    f.service
        .retrieval_settings_set(RetrievalSettingsSetRequest {
            root_id: f.root_id.clone(),
            enabled: true,
        })
        .unwrap();
    assert!(get(&f).is_err());
    f.service
        .sensitivity_set(SensitivitySetRequest {
            root_id: f.root_id.clone(),
            target: Endpoint::Topic {
                id: topic.id.clone(),
            },
            sensitivity: Sensitivity::CloudAllowed,
        })
        .unwrap();
    assert!(get(&f).unwrap().assertions.is_empty());
    let s = &f.seen[0].citation;
    let source = Endpoint::Source {
        source_id: s.source_id.clone(),
        locator: s.locator.clone(),
    };
    f.service
        .sensitivity_set(SensitivitySetRequest {
            root_id: f.root_id.clone(),
            target: source.clone(),
            sensitivity: Sensitivity::CloudAllowed,
        })
        .unwrap();
    let released = get(&f).unwrap();
    assert_eq!(released.assertions[0].value, "Ledger lead");
    assert!(released.relationships.is_empty());
    let json = serde_json::to_string(&released).unwrap();
    assert!(!json.contains(&s.path) && !json.contains(&s.excerpt));
    assert!(
        f.service
            .retrieval_topic(&f.root_id, &BTreeSet::new(), q.clone())
            .is_err()
    );
    assert!(f.service.retrieval_topic(&id(), &allow, q.clone()).is_err());
    let search = f
        .service
        .retrieval_search(
            &f.root_id,
            &allow,
            RetrievalSearchRequest {
                query: "ledger lead".into(),
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(search.hits[0].topic.id, topic.id);
    f.service
        .sensitivity_set(SensitivitySetRequest {
            root_id: f.root_id.clone(),
            target: source.clone(),
            sensitivity: Sensitivity::LocalOnly,
        })
        .unwrap();
    assert!(get(&f).is_err());
    f.service
        .sensitivity_set(SensitivitySetRequest {
            root_id: f.root_id.clone(),
            target: source,
            sensitivity: Sensitivity::CloudAllowed,
        })
        .unwrap();
    std::fs::write(f.root.join(&s.path), "Synthetic changed bytes").unwrap();
    assert!(get(&f).unwrap().assertions.is_empty());
    assert_eq!(
        f.service
            .knowledge_graph(f.root_request())
            .unwrap()
            .checks
            .stale,
        2
    );
}

#[test]
fn separate_effective_periods_form_a_timeline_while_uncertainty_and_unverified_stay_distinct() {
    let mut f = Fixture::new();
    f.write(
        "reports.md",
        "Earlier owner Sam. New owner Priya. Venue to be confirmed.",
    );
    let citation = f.citation("reports.md");
    let t = topic("Orchard", TopicKind::Project);
    let mut early = claim(&t, "owner", "Sam", &citation);
    early.valid_to = Some("2026-10-31".into());
    let mut later = claim(&t, "owner", "Priya", &citation);
    later.valid_from = Some("2026-11-01".into());
    let mut venue = claim(&t, "venue", "North hall", &citation);
    venue.uncertainty = Some("Tentative".into());
    let p = f
        .service
        .manual_propose(ManualProposeRequest {
            root_id: f.root_id.clone(),
            candidates: vec![
                Candidate::Topic { topic: t.clone() },
                Candidate::Claim { claim: early },
                Candidate::Claim { claim: later },
                Candidate::Claim { claim: venue },
            ],
        })
        .unwrap()
        .proposal;
    f.service
        .proposal_decide(ProposalDecideRequest {
            root_id: f.root_id.clone(),
            proposal_id: p.id,
            expected_fingerprint: p.fingerprint,
            decision: Decision::Approve {
                candidates: p.candidates,
            },
        })
        .unwrap();
    let result = f
        .service
        .topic_knowledge(TopicGetRequest {
            root_id: f.root_id.clone(),
            topic_id: t.id,
        })
        .unwrap();
    assert_eq!(result.timelines[0].revision_ids.len(), 2);
    assert_eq!(result.checks.conflicting, 0);
    assert_eq!(result.checks.unverified, 2);
    assert_eq!(result.checks.uncertain, 1);
    f.unchanged();
}
