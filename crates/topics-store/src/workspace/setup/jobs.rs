use super::*;
pub(super) fn check_sources(ctx: &Context, sources: &[SetupManifestSource]) -> WResult<()> {
    let state = load(ctx)?;
    for s in sources {
        let bytes = scopes::source_bytes(ctx, &s.source.path).map_err(|_| {
            error(
                "stale_scope",
                "Reviewed source unavailable; preview changed selection for renewed consent",
            )
        })?;
        if hash(&bytes) != s.source.hash
            || bytes.len() as u64 != s.bytes
            || scopes::source_sensitivity(&state, &s.source.path, &s.source.hash)
                != s.source.sensitivity
            || s.source.sensitivity == Sensitivity::LocalOnly
        {
            return Err(error(
                "stale_scope",
                "Reviewed source bytes or privacy changed; preview changed selection for renewed consent",
            ));
        }
    }
    Ok(())
}
pub(super) fn outstanding(
    spec: &SetupManifestBatch,
    batch: &SetupBatch,
) -> Vec<SetupManifestSource> {
    spec.sources
        .iter()
        .filter(|s| !batch.completed_source_ids.contains(&s.source.source_id))
        .cloned()
        .collect()
}
pub(super) fn capability(
    ctx: &Context,
    cap: &SetupBatchCapability,
) -> WResult<(SetupJob, BTreeSet<Id>)> {
    let job = get_job(ctx, &cap.job_id)?;
    if cap.root_id != ctx.root_id
        || job.status != SetupJobStatus::Running
        || cap.manifest_fingerprint != job.manifest.fingerprint
    {
        return Err(error(
            "forbidden",
            "Setup capability is inactive or out of scope",
        ));
    }
    let batch = job
        .batches
        .iter()
        .find(|b| b.batch_id == cap.batch_id)
        .ok_or_else(|| error("forbidden", "Setup batch is unavailable"))?;
    if batch.status != SetupBatchStatus::Running
        || batch.scope_id.as_ref() != Some(&cap.scope_id)
        || batch.run_id.as_ref() != Some(&cap.run_id)
        || batch.attempt != cap.attempt
    {
        return Err(error(
            "forbidden",
            "Setup attempt is inactive or out of scope",
        ));
    }
    let scope = scopes::authorized(ctx, &cap.scope_id, &cap.run_id)?;
    let expected = job
        .manifest
        .batches
        .iter()
        .find(|b| b.batch_id == cap.batch_id)
        .ok_or_else(|| error("forbidden", "Manifest batch unavailable"))?;
    let remaining = outstanding(expected, batch);
    check_sources(ctx, &remaining)?;
    if scope.preview.sources
        != remaining
            .iter()
            .map(|s| s.source.clone())
            .collect::<Vec<_>>()
    {
        return Err(error(
            "forbidden",
            "Batch scope differs from reviewed manifest",
        ));
    }
    let mut ids: BTreeSet<_> = scope
        .preview
        .sources
        .iter()
        .map(|s| s.source_id.clone())
        .collect();
    // Previously transmitted, completed draft context is delegated by the same
    // immutable job consent, never by a human inventory or a fresh file scan.
    for (spec, completed) in job.manifest.batches.iter().zip(&job.batches) {
        for source in &spec.sources {
            if completed
                .completed_source_ids
                .contains(&source.source.source_id)
                && check_sources(ctx, std::slice::from_ref(source)).is_ok()
            {
                ids.insert(source.source.source_id.clone());
            }
        }
    }
    Ok((job, ids))
}
impl WorkspaceService {
    pub fn setup_job_create(&self, request: SetupJobCreateRequest) -> WResult<SetupJob> {
        self.with(&request.root_id, |ctx| {
            valid_id(&request.manifest_id)?;
            let manifest: SetupManifest = read_json(
                &setup_dir(ctx)
                    .join("manifests")
                    .join(format!("{}.json", request.manifest_id)),
            )?;
            if manifest.schema != SCHEMA
                || manifest.root_id != ctx.root_id
                || manifest.fingerprint != request.expected_fingerprint
            {
                return Err(error(
                    "conflict",
                    "Reviewed manifest changed; preview it before authorizing",
                ));
            }
            let mut original = manifest.clone();
            original.fingerprint.clear();
            if digest(&original)? != manifest.fingerprint {
                return Err(error(
                    "conflict",
                    "Reviewed manifest integrity changed; preview again",
                ));
            }
            for batch in &manifest.batches {
                check_sources(ctx, &batch.sources)?;
            }
            for job in jobs(ctx)? {
                if job.manifest.manifest_id == manifest.manifest_id {
                    return Ok(job);
                }
                let state = load(ctx)?;
                let was_undone = state.operations.iter().any(|operation| {
                    undone(&state, &operation.operation.id)
                        && operation
                            .targets
                            .iter()
                            .filter_map(|target| target.record.as_ref())
                            .any(|record| {
                                record.record_kind == "receipt"
                                    && record
                                        .data
                                        .get("receipt")
                                        .and_then(|r| r.get("job_id"))
                                        .and_then(Value::as_str)
                                        == Some(job.job_id.as_str())
                            })
                });
                // A new preview of exactly the same selection is an existing
                // job, not another copy of its unresolved review questions.
                if !was_undone
                    && job.manifest.spec == manifest.spec
                    && job.manifest.roles == manifest.roles
                    && job.manifest.limits == manifest.limits
                    && job.manifest.runtime == manifest.runtime
                    && job
                        .manifest
                        .batches
                        .iter()
                        .map(|b| &b.sources)
                        .eq(manifest.batches.iter().map(|b| &b.sources))
                    && job.manifest.excluded == manifest.excluded
                {
                    return Ok(job);
                }
            }
            if manifest.batches.is_empty() {
                return Err(error(
                    "invalid_input",
                    "No eligible documents; add notes or capture a topic locally",
                ));
            }
            let batches = manifest
                .batches
                .iter()
                .map(|b| SetupBatch {
                    batch_id: b.batch_id.clone(),
                    status: SetupBatchStatus::Waiting,
                    scope_id: None,
                    run_id: None,
                    attempt: 0,
                    error: None,
                    proposal_ids: vec![],
                    completed_source_ids: vec![],
                })
                .collect();
            let coverage = SetupCoverage {
                selected: manifest
                    .batches
                    .iter()
                    .map(|b| b.sources.len() as u64)
                    .sum(),
                processed: vec![],
                failed: vec![],
                outstanding: vec![],
                excluded: manifest.excluded.clone(),
                complete: false,
                model_calls: 0,
                transmission_authorizations: 1,
            };
            let mut job = SetupJob {
                schema: SCHEMA,
                job_id: new_id(),
                root_id: ctx.root_id.clone(),
                manifest,
                prior_consents: vec![],
                status: SetupJobStatus::Ready,
                revision: 0,
                batches,
                graph: DraftGraph {
                    items: vec![],
                    issues: vec![],
                    canonical_ids: BTreeMap::new(),
                },
                coverage,
                created_at: now(),
            };
            coverage_refresh(&mut job);
            save_job(ctx, &job)?;
            Ok(job)
        })
    }
    pub fn setup_job_get(&self, request: SetupJobRequest) -> WResult<SetupJob> {
        self.with(&request.root_id, |ctx| get_job(ctx, &request.job_id))
    }
    pub fn setup_jobs_list(&self, request: RootRequest) -> WResult<Vec<SetupJob>> {
        self.with(&request.root_id, jobs)
    }
    pub fn setup_job_control(&self, request: SetupJobControlRequest) -> WResult<SetupJob> {
        self.with(&request.root_id, |ctx| {
            let mut job = get_job(ctx, &request.job_id)?;
            revision(&job, request.expected_revision)?;
            if matches!(
                request.action,
                SetupJobAction::Resume | SetupJobAction::RetryFailed
            ) && job
                .batches
                .iter()
                .any(|b| b.status == SetupBatchStatus::Running)
            {
                return Err(error(
                    "conflict",
                    "Pause the running batch before resuming or retrying setup",
                ));
            }
            match request.action {
                SetupJobAction::Pause | SetupJobAction::Cancel => {
                    for b in &mut job.batches {
                        if b.status == SetupBatchStatus::Running {
                            if let Some(run) = &b.run_id {
                                revoke(ctx, run)?;
                            }
                            b.status = SetupBatchStatus::Cancelled;
                        }
                    }
                    job.status = if request.action == SetupJobAction::Pause {
                        SetupJobStatus::Paused
                    } else {
                        SetupJobStatus::Cancelled
                    };
                }
                SetupJobAction::Resume | SetupJobAction::RetryFailed => {
                    for (spec, b) in job.manifest.batches.iter().zip(&mut job.batches) {
                        if b.status != SetupBatchStatus::Completed {
                            if let Err(e) = check_sources(ctx, &outstanding(spec, b)) {
                                b.status = SetupBatchStatus::NeedsConsent;
                                job.status = SetupJobStatus::NeedsConsent;
                                b.error = Some(e.clone());
                                bump(&mut job)?;
                                coverage_refresh(&mut job);
                                save_job(ctx, &job)?;
                                return Err(e);
                            }
                            if b.status != SetupBatchStatus::Failed
                                || request.action == SetupJobAction::RetryFailed
                            {
                                b.status = SetupBatchStatus::Waiting;
                                b.error = None;
                            }
                        }
                    }
                    job.status = SetupJobStatus::Ready;
                }
            }
            bump(&mut job)?;
            coverage_refresh(&mut job);
            save_job(ctx, &job)?;
            Ok(job)
        })
    }
    pub fn setup_batch_start(
        &self,
        request: SetupBatchStartRequest,
    ) -> WResult<SetupBatchCapability> {
        self.with(&request.root_id,|ctx|{
  let mut job=get_job(ctx,&request.job_id)?;
  if !matches!(job.status,SetupJobStatus::Ready|SetupJobStatus::Running)||job.batches.iter().any(|b|b.status==SetupBatchStatus::Running){return Err(error("conflict","Resume setup and wait for its current bounded batch"))}
  let i=job.batches.iter().position(|b|b.batch_id==request.batch_id).ok_or_else(||error("invalid_input","Batch not in reviewed manifest"))?;
  if !matches!(job.batches[i].status,SetupBatchStatus::Waiting|SetupBatchStatus::Cancelled){return Err(error("conflict","Batch is completed, failed or needs consent; select an outstanding authorized batch"))}
  if job.batches[i].attempt>=job.manifest.limits.max_turns_per_batch{return Err(error("conflict","Batch attempt budget exhausted; review a new manifest"))}
  if let Err(e)=check_sources(ctx,&outstanding(&job.manifest.batches[i],&job.batches[i])){job.batches[i].status=SetupBatchStatus::NeedsConsent;job.batches[i].error=Some(e.clone());job.status=SetupJobStatus::NeedsConsent;bump(&mut job)?;coverage_refresh(&mut job);save_job(ctx,&job)?;return Err(e)}
  let scope_id=new_id();let run_id=new_id();
  let scope=Scope{preview:ScopePreview{scope_id:scope_id.clone(),root_id:ctx.root_id.clone(),sources:outstanding(&job.manifest.batches[i],&job.batches[i]).into_iter().map(|s|s.source).collect(),excluded:vec![]}};
  save_json(&ctx.partition.join("scopes").join(format!("{scope_id}.json")),&scope)?;
  save_json(&ctx.partition.join("runs").join(format!("{run_id}.json")),&Run{root_id:ctx.root_id.clone(),run_id:run_id.clone(),scope_id:scope_id.clone(),active:true,message_id:None,topic_id:None})?;
  let batch=&mut job.batches[i];batch.attempt+=1;batch.status=SetupBatchStatus::Running;batch.scope_id=Some(scope_id.clone());batch.run_id=Some(run_id.clone());
  let cap=SetupBatchCapability{root_id:ctx.root_id.clone(),job_id:job.job_id.clone(),batch_id:batch.batch_id.clone(),scope_id,run_id,manifest_fingerprint:job.manifest.fingerprint.clone(),attempt:batch.attempt};
  job.status=SetupJobStatus::Running;job.coverage.model_calls+=1;bump(&mut job)?;save_job(ctx,&job)?;Ok(cap)
 })
    }
    pub fn setup_batch_fail(&self, request: SetupBatchFailRequest) -> WResult<SetupJob> {
        self.with(&request.capability.root_id, |ctx| {
            // A source may already be stale when a runtime reports failure; match the exact attempt without reading it.
            let mut job = get_job(ctx, &request.capability.job_id)?;
            let b = job
                .batches
                .iter_mut()
                .find(|b| {
                    b.batch_id == request.capability.batch_id
                        && b.run_id.as_ref() == Some(&request.capability.run_id)
                        && b.scope_id.as_ref() == Some(&request.capability.scope_id)
                        && b.attempt == request.capability.attempt
                        && b.status == SetupBatchStatus::Running
                })
                .ok_or_else(|| error("forbidden", "Setup attempt is no longer running"))?;
            revoke(ctx, &request.capability.run_id)?;
            b.status = SetupBatchStatus::Failed;
            // Provider strings may contain arbitrary private bytes. Persist only
            // the generic human action and reconstructed allowlisted constants.
            let mut failure = error(
                if request.error.code == "cancelled" {
                    "cancelled"
                } else {
                    "runtime_unavailable"
                },
                "Batch did not finish; retry these inputs or defer them",
            );
            failure.details = safe_failure_details(request.error.details.as_ref());
            b.error = Some(failure);
            job.status = SetupJobStatus::Ready;
            bump(&mut job)?;
            coverage_refresh(&mut job);
            save_job(ctx, &job)?;
            Ok(job)
        })
    }
}
fn safe_failure_details(details: Option<&Value>) -> Option<Value> {
    let object = details?.as_object()?;
    if object.len() != 2 {
        return None;
    }
    match (
        object.get("phase")?.as_str()?,
        object.get("reason")?.as_str()?,
    ) {
        ("preflight", "version_check_failed") => {
            Some(serde_json::json!({"phase":"preflight","reason":"version_check_failed"}))
        }
        ("preflight", "layout_denied") => {
            Some(serde_json::json!({"phase":"preflight","reason":"layout_denied"}))
        }
        ("preflight", "native_denied") => {
            Some(serde_json::json!({"phase":"preflight","reason":"native_denied"}))
        }
        ("startup", "setup_failed") => {
            Some(serde_json::json!({"phase":"startup","reason":"setup_failed"}))
        }
        ("discovery", "discovery_failed") => {
            Some(serde_json::json!({"phase":"discovery","reason":"discovery_failed"}))
        }
        ("discovery", "discovery_deadline") => {
            Some(serde_json::json!({"phase":"discovery","reason":"discovery_deadline"}))
        }
        ("transport", "transport_failed") => {
            Some(serde_json::json!({"phase":"transport","reason":"transport_failed"}))
        }
        ("transport", "read_deadline") => {
            Some(serde_json::json!({"phase":"transport","reason":"read_deadline"}))
        }
        ("transport", "protocol_denied") => {
            Some(serde_json::json!({"phase":"transport","reason":"protocol_denied"}))
        }
        ("mcp", "session_failed") => {
            Some(serde_json::json!({"phase":"mcp","reason":"session_failed"}))
        }
        ("mcp", "inventory_denied") => {
            Some(serde_json::json!({"phase":"mcp","reason":"inventory_denied"}))
        }
        ("mcp", "request_failed") => {
            Some(serde_json::json!({"phase":"mcp","reason":"request_failed"}))
        }
        ("mcp", "request_deadline") => {
            Some(serde_json::json!({"phase":"mcp","reason":"request_deadline"}))
        }
        ("callback", "authorization_denied") => {
            Some(serde_json::json!({"phase":"callback","reason":"authorization_denied"}))
        }
        ("candidate", "schema_denied") => {
            Some(serde_json::json!({"phase":"candidate","reason":"schema_denied"}))
        }
        ("host", "validation_denied") => {
            Some(serde_json::json!({"phase":"host","reason":"validation_denied"}))
        }
        ("host", "call_deadline") => {
            Some(serde_json::json!({"phase":"host","reason":"call_deadline"}))
        }
        ("host", "host_failed") => Some(serde_json::json!({"phase":"host","reason":"host_failed"})),
        ("execution", "validation_denied") => {
            Some(serde_json::json!({"phase":"execution","reason":"validation_denied"}))
        }
        ("execution", "execution_deadline") => {
            Some(serde_json::json!({"phase":"execution","reason":"execution_deadline"}))
        }
        ("publication", "preparation_failed") => {
            Some(serde_json::json!({"phase":"publication","reason":"preparation_failed"}))
        }
        ("publication", "preparation_deadline") => {
            Some(serde_json::json!({"phase":"publication","reason":"preparation_deadline"}))
        }
        ("turn", "model_terminal_failed") => {
            Some(serde_json::json!({"phase":"turn","reason":"model_terminal_failed"}))
        }
        ("publication", "scope_not_read") => {
            Some(serde_json::json!({"phase":"publication","reason":"scope_not_read"}))
        }
        _ => None,
    }
}
fn coverage_refresh(job: &mut SetupJob) {
    coverage(job)
}

#[cfg(test)]
mod diagnostic_tests {
    use super::*;
    #[test]
    fn sanitizer_reconstructs_only_exact_contract_pairs_including_cross_mismatches() {
        let pairs: Vec<_> = include_str!("../../../../../features/folder-setup/FAILURE-ORIGINS.md")
            .lines()
            .filter(|line| line.starts_with("| `"))
            .map(|line| {
                let fields: Vec<_> = line.split('`').collect();
                (fields[1], fields[3])
            })
            .collect();
        assert_eq!(pairs.len(), 24);
        for (phase, _) in &pairs {
            for (_, reason) in &pairs {
                let value = serde_json::json!({"phase":phase,"reason":reason});
                let expected = pairs.contains(&(*phase, *reason)).then(|| value.clone());
                assert_eq!(safe_failure_details(Some(&value)), expected);
                let mut extra = value;
                extra["private"] = serde_json::json!("PRIVATE_SENTINEL");
                assert_eq!(safe_failure_details(Some(&extra)), None);
            }
        }
    }
}
