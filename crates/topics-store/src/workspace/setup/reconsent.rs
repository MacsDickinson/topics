//! Renew one immutable manifest without rebasing original capability provenance.
use super::*;
impl WorkspaceService {
    pub fn setup_job_reconsent(&self, request: SetupJobReconsentRequest) -> WResult<SetupJob> {
        self.with(&request.root_id, |ctx| {
            let mut job = get_job(ctx, &request.job_id)?;
            revision(&job, request.expected_revision)?;
            if job
                .batches
                .iter()
                .any(|b| b.status == SetupBatchStatus::Running)
            {
                return Err(error(
                    "conflict",
                    "Pause the runtime before reviewing updated setup consent",
                ));
            }
            valid_id(&request.manifest_id)?;
            let manifest: SetupManifest = read_json(
                &setup_dir(ctx)
                    .join("manifests")
                    .join(format!("{}.json", request.manifest_id)),
            )?;
            let mut check = manifest.clone();
            check.fingerprint.clear();
            if manifest.schema != SCHEMA
                || manifest.root_id != ctx.root_id
                || manifest.fingerprint != request.expected_fingerprint
                || digest(&check)? != manifest.fingerprint
            {
                return Err(error(
                    "conflict",
                    "Updated manifest changed; preview the exact selection again",
                ));
            }
            for batch in &manifest.batches {
                jobs::check_sources(ctx, &batch.sources)?;
            }
            if manifest.manifest_id == job.manifest.manifest_id {
                return Ok(job);
            }
            let completed: Vec<_> = job
                .manifest
                .batches
                .iter()
                .zip(&job.batches)
                .flat_map(|(spec, batch)| {
                    spec.sources
                        .iter()
                        .filter(|s| batch.completed_source_ids.contains(&s.source.source_id))
                        .cloned()
                })
                .collect();
            let batches = manifest
                .batches
                .iter()
                .map(|spec| {
                    let done: Vec<_> = spec
                        .sources
                        .iter()
                        .filter(|s| {
                            completed
                                .iter()
                                .any(|old| old.source == s.source && old.bytes == s.bytes)
                        })
                        .map(|s| s.source.source_id.clone())
                        .collect();
                    SetupBatch {
                        batch_id: spec.batch_id.clone(),
                        status: if done.len() == spec.sources.len() {
                            SetupBatchStatus::Completed
                        } else {
                            SetupBatchStatus::Waiting
                        },
                        scope_id: None,
                        run_id: None,
                        attempt: 0,
                        error: None,
                        proposal_ids: vec![],
                        completed_source_ids: done,
                    }
                })
                .collect();
            job.prior_consents.push(SetupConsent {
                manifest: job.manifest.clone(),
                batches: job.batches.clone(),
            });
            job.manifest = manifest;
            job.batches = batches;
            job.status = SetupJobStatus::Ready;
            job.coverage.selected = job
                .manifest
                .batches
                .iter()
                .map(|b| b.sources.len() as u64)
                .sum();
            job.coverage.excluded = job.manifest.excluded.clone();
            job.coverage.transmission_authorizations += 1;
            bump(&mut job)?;
            coverage(&mut job);
            save_job(ctx, &job)?;
            Ok(job)
        })
    }
}
