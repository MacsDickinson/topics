import { type Page } from '@playwright/test';

export const ids = { root: '00000000-0000-4000-8000-000000000001', topic: '00000000-0000-4000-8000-000000000002', scope: '00000000-0000-4000-8000-000000000003', run: '00000000-0000-4000-8000-000000000004', proposal: '00000000-0000-4000-8000-000000000005', claim: '00000000-0000-4000-8000-000000000006', revision: '00000000-0000-4000-8000-000000000007', source: '00000000-0000-4000-8000-000000000008' };
const hash = 'a'.repeat(64);
export const topic = { id: ids.topic, topic_kind: 'project', name: 'Ledger', aliases: [], sensitivity: 'unknown' };
export const source = { source_id: ids.source, path: 'Meetings/one.md', document_hash: hash, locator: { start_byte: 0, end_byte: 23, heading: null }, passage_hash: hash, excerpt: 'Ledger launch is Friday' };
export const claim = { id: ids.claim, revision_id: ids.revision, subject_id: ids.topic, predicate: 'launch', value: 'Friday', uncertainty: 'Transcript; confirm the date.', sources: [{ kind: 'passage', source }], supersedes: [], recorded_at: '2026-11-03T00:00:00Z', valid_from: null, valid_to: null };
export const proposal = { id: ids.proposal, root_id: ids.root, scope_id: ids.scope, origin: { kind: 'analysis', run_id: ids.run }, status: 'pending', candidates: [{ kind: 'topic', topic }, { kind: 'claim', claim }], fingerprint: hash, created_at: '2026-11-03T00:00:00Z' };

// Browser transport fixture only. Never imported by application code or native tests.
export async function workspaceFixture(page: Page, options: { empty?: boolean; unavailable?: boolean } = {}) {
  await page.addInitScript(({ ids, topic, claim, proposal, hash, options }) => {
    const w = window as any;
    w.calls = []; w.callbacks = {}; w.nextCallback = 1; w.defer = {}; w.failures = {}; w.root = options.empty ? null : { root_id: ids.root, path: '/synthetic/notes' };
    w.topics = []; w.proposals = []; w.claims = []; w.operations = [];
    w.emitRun = (seq: number, event: unknown, root_id = ids.root, run_id = ids.run) => {
      for (const fn of Object.values(w.listeners)) (fn as Function)({ event: 'workspace_run_event', id: 1, payload: { root_id, run_id, seq, event } });
    };
    w.listeners = {};
    w.__TAURI_INTERNALS__ = {
      transformCallback: (fn: Function) => { const id = w.nextCallback++; w.callbacks[id] = fn; return id; },
      unregisterCallback: (id: number) => { delete w.callbacks[id]; },
      invoke: async (command: string, args: any) => {
        if (command === 'plugin:event|listen') { w.listeners[args.handler] = w.callbacks[args.handler]; return args.handler; }
        if (command === 'plugin:event|unlisten') { delete w.listeners[args.eventId]; return; }
        w.calls.push({ command, args });
        if (w.failures[command]) throw w.failures[command];
        if (w.defer[command]) return new Promise((resolve, reject) => { w.defer[command] = { resolve, reject }; });
        if (command.startsWith('workspace_') && (!args || Object.keys(args).join() !== 'request')) throw { code: 'invalid_input', message: 'Expected request envelope', details: null };
        const request = args?.request;
        switch (command) {
          case 'workspace_root_get': return { root: w.root };
          case 'workspace_root_set': w.root = { root_id: request.path.includes('other') ? 'other-root' : ids.root, path: request.path }; w.topics = []; w.proposals = []; return { root: w.root, index: { documents: 0, people: 0, warnings: [] } };
          case 'workspace_runtime_status': return { available: !options.unavailable, version: options.unavailable ? null : '0.166.0', message: options.unavailable ? 'Codex is missing. Install Codex 0.166.0 and restart Topics.' : 'Ready' };
          case 'workspace_scope_preview': return { scope_id: ids.scope, root_id: w.root.root_id, sources: [{ source_id: ids.source, path: 'Meetings/one.md', hash, date: '2026-11-01', sensitivity: 'unknown' }], excluded: [{ path: 'Meetings/two.md', reason: 'Outside date range' }, { path: 'Meetings/undated.md', reason: 'Unknown date' }, { path: 'Meetings/private.md', reason: 'Local-only' }] };
          case 'workspace_analysis_start': return { run_id: ids.run };
          case 'workspace_chat_start': return { run_id: ids.run, message_id: 'message-one' };
          case 'workspace_run_cancel': return { cancelled: true };
          case 'workspace_proposals_list': return { proposals: w.proposals };
          case 'workspace_topics_list': return { topics: w.topics };
          case 'workspace_topic_get': return { topic, current_claims: w.claims, connections: [], unresolved: [], history: w.history || [], warnings: [] };
          case 'workspace_proposal_decide': {
            const p = w.proposals.find((p: any) => p.id === request.proposal_id);
            p.status = request.decision.kind === 'reject' ? 'rejected' : 'accepted';
            if (p.status === 'accepted') {
              w.topics = [topic]; w.claims = request.decision.candidates.filter((c: any) => c.claim).map((c: any) => c.claim);
              w.operations.push({ id: 'operation-one', status: 'committed', description: 'Accepted Ledger launch', created_at: '2026-11-03T00:00:00Z' });
            }
            return { proposal: p, operation: w.operations.at(-1) || null };
          }
          case 'workspace_history_list': return { operations: w.operations };
          case 'workspace_undo': throw { code: 'conflict', message: 'Undo the dependent claim first.', details: { dependencies: [ids.claim] } };
          case 'workspace_sensitivity_set': return { operation: { id: 'label-one', status: 'committed', description: 'Changed sensitivity', created_at: '2026-11-03T00:00:00Z' } };
          case 'search_notes': return [];
          default: throw new Error(`Unexpected IPC: ${command}`);
        }
      },
    };
    w.showProposal = () => { w.proposals = [structuredClone(proposal)]; };
  }, { ids, topic, claim, proposal, hash, options });
  await page.goto('/');
}
