//! Human folder setup services; all model publications remain capability guarded.
mod context;
mod decisions;
mod draft;
mod graph;
pub(super) mod inventory;
mod jobs;
mod reconsent;
mod review;
use super::proposals as proposal;
use super::scopes::{Run, Scope};
use super::*;

const SCHEMA: u32 = 1;
const MAX_RECORDS: usize = 100;
fn digest(value: &impl Serialize) -> WResult<Hash> {
    Ok(hash(&serde_json::to_vec(value).map_err(internal)?))
}
fn setup_dir(ctx: &Context) -> PathBuf {
    ctx.partition.join("setup-v1")
}
fn job_path(ctx: &Context, id: &str) -> WResult<PathBuf> {
    valid_id(id)?;
    Ok(setup_dir(ctx).join("jobs").join(format!("{id}.json")))
}
#[derive(Clone, Serialize, Deserialize)]
struct DraftEdit {
    before: Candidate,
    after: Candidate,
    statement: UserStatement,
}
#[derive(Clone, Serialize, Deserialize)]
struct PilotRun {
    at: Timestamp,
    batch_id: Id,
    sources: Vec<SourceSummary>,
    candidates: Vec<Candidate>,
}
#[derive(Clone, Serialize, Deserialize)]
struct Checkpoint {
    #[serde(default)]
    pilot_runs: Vec<PilotRun>,
    owner: Id,
    job: SetupJob,
    edits: Vec<DraftEdit>,
}
fn checkpoint(ctx: &Context, id: &str) -> WResult<Checkpoint> {
    read_json(&job_path(ctx, id)?).map_err(|_| {
        error(
            "invalid_input",
            "Setup job unavailable; select an existing job",
        )
    })
}
fn get_job(ctx: &Context, id: &str) -> WResult<SetupJob> {
    let mut saved = checkpoint(ctx, id)?;
    if saved.job.schema != SCHEMA
        || saved.job.root_id != ctx.root_id
        || saved.job.manifest.root_id != ctx.root_id
    {
        return Err(error(
            "stale_scope",
            "Setup belongs to another root generation; preview the current folder again",
        ));
    }
    if saved.owner != ctx.instance_id
        && saved
            .job
            .batches
            .iter()
            .any(|b| b.status == SetupBatchStatus::Running)
    {
        for b in &mut saved.job.batches {
            if b.status == SetupBatchStatus::Running {
                if let Some(id) = &b.run_id {
                    revoke(ctx, id)?;
                }
                b.status = SetupBatchStatus::Cancelled;
            }
        }
        saved.job.status = SetupJobStatus::Paused;
        coverage(&mut saved.job);
        bump(&mut saved.job)?;
        saved.owner = ctx.instance_id.clone();
        save_json(&job_path(ctx, id)?, &saved)?;
    }
    // A Markdown commit can complete before the cache is updated, or undo can
    // remove a previously saved receipt. Derive only acceptance from markers.
    let state = load(ctx)?;
    let committed: BTreeSet<_> = state
        .receipts
        .values()
        .filter(|r| r.receipt.job_id.as_deref() == Some(id))
        .flat_map(|r| r.saved_ids.iter().cloned())
        .collect();
    let mut changed = false;
    for item in &mut saved.job.graph.items {
        let equivalent = match &item.candidate {
            Candidate::Topic { topic } => {
                state.topics.get(&topic.id).is_some_and(|old| old == topic)
            }
            Candidate::Claim { claim } => {
                let accepted = state.claims.values().find(|old| {
                    proposal::same_claim(old, claim)
                        && old.uncertainty == claim.uncertainty
                        && claim
                            .sources
                            .iter()
                            .all(|source| old.sources.contains(source))
                        && !state
                            .claims
                            .values()
                            .any(|next| next.supersedes.contains(&old.revision_id))
                });
                if let Some(old) = accepted {
                    link(&mut saved.job.graph.canonical_ids, &claim.id, &old.id);
                    link(
                        &mut saved.job.graph.canonical_ids,
                        &claim.revision_id,
                        &old.revision_id,
                    );
                }
                accepted.is_some()
            }
            Candidate::Connection { connection } => {
                let accepted = state.connections.values().find(|old| {
                    old.from == connection.from
                        && old.to == connection.to
                        && old.relation == connection.relation
                        && old.valid_from == connection.valid_from
                        && old.valid_to == connection.valid_to
                        && old.uncertainty == connection.uncertainty
                        && connection
                            .sources
                            .iter()
                            .all(|source| old.sources.contains(source))
                });
                if let Some(old) = accepted {
                    link(&mut saved.job.graph.canonical_ids, &connection.id, &old.id);
                }
                accepted.is_some()
            }
            _ => false,
        };
        let disposition = if committed.contains(&item.item_id) || equivalent {
            DraftDisposition::Saved
        } else if item.disposition == DraftDisposition::Saved {
            DraftDisposition::Pending
        } else {
            item.disposition
        };
        changed |= disposition != item.disposition;
        item.disposition = disposition;
    }
    for item in &mut saved.job.graph.items {
        let before = item.candidate.clone();
        proposal::remap(
            std::slice::from_mut(&mut item.candidate),
            &saved.job.graph.canonical_ids,
        )?;
        changed |= before != item.candidate;
    }
    if changed {
        bump(&mut saved.job)?;
        save_json(&job_path(ctx, id)?, &saved)?;
    }
    Ok(saved.job)
}
fn link(mapping: &mut BTreeMap<Id, Id>, from: &str, to: &str) {
    if from == to {
        return;
    }
    for value in mapping.values_mut() {
        if value == from {
            *value = to.into();
        }
    }
    mapping.insert(from.into(), to.into());
}
fn decision_chunks(
    candidates: &[Candidate],
    edits: &[DraftEdit],
    statement: bool,
) -> WResult<Vec<std::ops::Range<usize>>> {
    let mut chunks = vec![];
    let mut start = 0;
    let mut records = 1 + usize::from(statement); // receipt plus correction evidence
    for (index, candidate) in candidates.iter().enumerate() {
        let ids = proposal::candidate_ids(std::slice::from_ref(candidate));
        let history = edits
            .iter()
            .filter(|edit| {
                proposal::candidate_ids(std::slice::from_ref(&edit.after))
                    .iter()
                    .any(|id| ids.contains(id))
            })
            .count();
        let weight = history
            + match candidate {
                Candidate::Topic { .. } => 1,
                Candidate::Correction { .. } => 3,
                _ => 2,
            };
        if weight + 1 + usize::from(statement) > MAX_RECORDS {
            return Err(error(
                "invalid_input",
                "Draft edit history exceeds one bounded operation; preserve it and review a smaller new revision",
            ));
        }
        if index > start && (index - start == 30 || records + weight > MAX_RECORDS) {
            chunks.push(start..index);
            start = index;
            records = 1 + usize::from(statement);
        }
        records += weight;
    }
    if start < candidates.len() {
        chunks.push(start..candidates.len());
    }
    Ok(chunks)
}
fn save_job(ctx: &Context, job: &SetupJob) -> WResult<()> {
    let edits = if job_path(ctx, &job.job_id)?.exists() {
        checkpoint(ctx, &job.job_id)?.edits
    } else {
        vec![]
    };
    save_json(
        &job_path(ctx, &job.job_id)?,
        &Checkpoint {
            owner: ctx.instance_id.clone(),
            job: job.clone(),
            edits,
            pilot_runs: if job_path(ctx, &job.job_id)?.exists() {
                checkpoint(ctx, &job.job_id)?.pilot_runs
            } else {
                vec![]
            },
        },
    )
}
fn jobs(ctx: &Context) -> WResult<Vec<SetupJob>> {
    let dir = setup_dir(ctx).join("jobs");
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut out = vec![];
    for p in json_entries(&dir)? {
        if let Ok(saved) = read_json::<Checkpoint>(&p)
            && saved.job.schema == SCHEMA
            && saved.job.root_id == ctx.root_id
        {
            out.push(get_job(ctx, &saved.job.job_id)?);
        }
    }
    out.sort_by_key(|j| j.created_at.clone());
    Ok(out)
}
// Read only local human draft edit checkpoints; callers already hold the workspace lock.
pub(super) fn pilot_edits(ctx: &Context) -> WResult<Vec<(Candidate, Candidate, Timestamp)>> {
    let dir = setup_dir(ctx).join("jobs");
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for path in json_entries(&dir)? {
        if let Ok(saved) = read_json::<Checkpoint>(&path)
            && saved.job.schema == SCHEMA
            && saved.job.root_id == ctx.root_id
            && saved.job.manifest.root_id == ctx.root_id
        {
            out.extend(
                saved
                    .edits
                    .into_iter()
                    .map(|edit| (edit.before, edit.after, edit.statement.recorded_at)),
            );
        }
    }
    Ok(out)
}
fn revision(job: &SetupJob, expected: u64) -> WResult<()> {
    if job.revision != expected {
        Err(error(
            "conflict",
            "Draft changed; refresh the map and preview the exact decision again",
        ))
    } else {
        Ok(())
    }
}
fn bump(job: &mut SetupJob) -> WResult<()> {
    job.revision = job
        .revision
        .checked_add(1)
        .ok_or_else(|| error("conflict", "Setup revision exhausted"))?;
    Ok(())
}
fn revoke(ctx: &Context, id: &str) -> WResult<()> {
    let p = ctx.partition.join("runs").join(format!("{id}.json"));
    let mut run: Run = read_json(&p)?;
    run.active = false;
    save_json(&p, &run)
}
fn coverage(job: &mut SetupJob) {
    let mut processed = vec![];
    let mut failed = vec![];
    let mut outstanding = vec![];
    for (spec, batch) in job.manifest.batches.iter().zip(&job.batches) {
        for source in &spec.sources {
            let to = if batch
                .completed_source_ids
                .contains(&source.source.source_id)
            {
                &mut processed
            } else if batch.status == SetupBatchStatus::Failed {
                &mut failed
            } else {
                &mut outstanding
            };
            to.push(source.source.path.clone());
        }
    }
    job.coverage.processed = processed;
    job.coverage.failed = failed;
    job.coverage.outstanding = outstanding;
    job.coverage.complete = job.coverage.failed.is_empty() && job.coverage.outstanding.is_empty();
    if job.batches.iter().all(|b| {
        matches!(
            b.status,
            SetupBatchStatus::Completed | SetupBatchStatus::Failed
        )
    }) {
        job.status = if job.coverage.complete {
            SetupJobStatus::Finished
        } else {
            SetupJobStatus::FinishedWithGaps
        };
    }
}

// Local human briefing evidence, committed with the corresponding successful checkpoint.
type PilotEvidence = (
    SetupJob,
    Vec<(Timestamp, Vec<SourceSummary>, Vec<Candidate>)>,
);
pub(super) fn pilot_evidence(ctx: &Context) -> WResult<(Vec<PilotEvidence>, bool)> {
    let dir = setup_dir(ctx).join("jobs");
    if !dir.exists() {
        return Ok((vec![], false));
    }
    let mut result = Vec::new();
    let mut incomplete = false;
    for path in json_entries(&dir)? {
        let saved: Checkpoint = match read_json(&path) {
            Ok(saved) => saved,
            Err(_) => {
                incomplete = true;
                continue;
            }
        };
        if saved.job.root_id == ctx.root_id && saved.job.schema != SCHEMA {
            incomplete = true;
            continue;
        }
        if saved.job.schema == SCHEMA && saved.job.root_id == ctx.root_id {
            if saved.job.manifest.root_id != ctx.root_id
                || saved.job.manifest.batches.len() != saved.job.batches.len()
                || saved
                    .job
                    .manifest
                    .batches
                    .iter()
                    .zip(&saved.job.batches)
                    .any(|(spec, batch)| spec.batch_id != batch.batch_id)
            {
                incomplete = true;
                continue;
            }
            if saved.job.batches.iter().any(|batch| {
                batch.status == SetupBatchStatus::Completed
                    && !saved
                        .pilot_runs
                        .iter()
                        .any(|run| run.batch_id == batch.batch_id)
            }) {
                incomplete = true;
            }
            result.push((
                saved.job,
                saved
                    .pilot_runs
                    .into_iter()
                    .map(|r| (r.at, r.sources, r.candidates))
                    .collect(),
            ));
        }
    }
    result.sort_by_key(|(job, _)| job.created_at.clone());
    Ok((result, incomplete))
}
