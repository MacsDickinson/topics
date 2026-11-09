// Only canonical synthetic job graph, closed diagnostic pairs and counters cross this boundary.
export function safeObservation(job) {
  const allowed = new Set(['turn:model_terminal_failed', 'publication:scope_not_read']);
  const batches = (job.batches || []).map((batch) => {
    const details = batch.error?.details;
    const pair = details && Object.keys(details).length === 2 && allowed.has(`${details.phase}:${details.reason}`)
      ? { phase: details.phase, reason: details.reason } : null;
    return { status: batch.status, attempt: batch.attempt, diagnostic: pair, diagnostic_status: pair ? 'known' : 'unknown' };
  });
  const coverage = job.coverage || {};
  return { status: job.status, batches, coverage: { complete: coverage.complete, selected: coverage.selected, processed: (coverage.processed || []).length, failed: (coverage.failed || []).length, outstanding: (coverage.outstanding || []).length, model_calls: coverage.model_calls, transmission_authorizations: coverage.transmission_authorizations }, graph: job.graph };
}
export async function waitForTerminal(readJob, save, { timeoutMs = 180000, intervalMs = 1000, now = Date.now, pause = ms => new Promise(resolve => setTimeout(resolve, ms)) } = {}) {
  const deadline = now() + timeoutMs;
  while (true) {
    const job = await readJob();
    if (job) {
      const observation = safeObservation(job);
      save(observation);
      if (job.status === 'finished') return job;
      if (['finished_with_gaps', 'failed', 'cancelled'].includes(job.status)) throw new Error(`Setup terminal status: ${job.status}; diagnostics saved`);
    }
    if (now() >= deadline) throw new Error('Setup terminal status unknown: observation timed out');
    await pause(intervalMs);
  }
}
