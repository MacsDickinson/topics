// Only canonical synthetic job graph, closed diagnostic pairs and counters cross this boundary.
export function safeObservation(job) {
  const allowed = new Set([
    'preflight:version_check_failed', 'preflight:layout_denied', 'preflight:native_denied',
    'startup:setup_failed', 'discovery:discovery_failed', 'discovery:discovery_deadline',
    'transport:transport_failed', 'transport:read_deadline', 'transport:protocol_denied',
    'mcp:session_failed', 'mcp:inventory_denied', 'mcp:request_failed', 'mcp:request_deadline',
    'callback:authorization_denied', 'candidate:schema_denied',
    'host:validation_denied', 'host:call_deadline', 'host:host_failed',
    'execution:validation_denied', 'execution:execution_deadline',
    'publication:preparation_failed', 'publication:preparation_deadline',
    'turn:model_terminal_failed', 'publication:scope_not_read',
  ]);
  const batches = (job.batches || []).map((batch) => {
    const details = batch.error?.details;
    const pair = details && typeof details === 'object' && !Array.isArray(details) && Object.keys(details).length === 2 && typeof details.phase === 'string' && typeof details.reason === 'string' && allowed.has(`${details.phase}:${details.reason}`)
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
