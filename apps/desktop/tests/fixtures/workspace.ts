import { type Page } from '@playwright/test';

export const ids = {
  root: '00000000-0000-4000-8000-000000000001',
  topic: '00000000-0000-4000-8000-000000000002',
  scope: '00000000-0000-4000-8000-000000000003',
  run: '00000000-0000-4000-8000-000000000004',
  proposal: '00000000-0000-4000-8000-000000000005',
  claim: '00000000-0000-4000-8000-000000000006',
  revision: '00000000-0000-4000-8000-000000000007',
  source: '00000000-0000-4000-8000-000000000008',
};
const hash = 'a'.repeat(64);
export const topic = {
  id: ids.topic,
  topic_kind: 'project',
  name: 'Ledger',
  aliases: [],
  sensitivity: 'cloud_allowed',
};
export const source = {
  source_id: ids.source,
  path: 'Meetings/one.md',
  document_hash: hash,
  locator: { start_byte: 0, end_byte: 23, heading: null },
  passage_hash: hash,
  excerpt: 'Ledger launch is Friday',
};
export const claim = {
  id: ids.claim,
  revision_id: ids.revision,
  subject_id: ids.topic,
  predicate: 'launch',
  value: 'Friday',
  uncertainty: 'Transcript; confirm the date.',
  sources: [{ kind: 'passage', source }],
  supersedes: [],
  recorded_at: '2026-11-03T00:00:00Z',
  valid_from: null,
  valid_to: null,
};
export const proposal = {
  id: ids.proposal,
  root_id: ids.root,
  scope_id: ids.scope,
  origin: { kind: 'analysis', run_id: ids.run },
  status: 'pending',
  candidates: [
    { kind: 'topic', topic },
    { kind: 'claim', claim },
  ],
  fingerprint: hash,
  created_at: '2026-11-03T00:00:00Z',
};

// Browser transport fixture only. Never imported by application code or native tests.
export async function workspaceFixture(
  page: Page,
  options: { populated?: boolean; empty?: boolean; unavailable?: boolean; inventoryEmpty?: boolean; nested?: boolean } = {},
) {
  await page.addInitScript(
    ({ ids, topic, claim, proposal, hash, options }) => {
      const w = window as any;
      w.calls = [];
      w.callbacks = {};
      w.nextCallback = 1;
      w.defer = {};
      w.failures = {};
      w.root = options.empty
        ? null
        : { root_id: ids.root, path: '/synthetic/notes' };
      w.topics = options.populated ? [structuredClone(topic)] : [];
      w.proposals = [];
      w.claims = options.populated ? [structuredClone(claim)] : [];
      w.operations = [];
      w.emitRun = (
        seq: number,
        event: unknown,
        root_id = ids.root,
        run_id = ids.run,
      ) => {
        for (const fn of Object.values(w.listeners))
          (fn as Function)({
            event: 'workspace_run_event',
            id: 1,
            payload: { root_id, run_id, seq, event },
          });
      };
      w.listeners = {};
      w.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
        unregisterListener: () => undefined,
      };
      w.__TAURI_INTERNALS__ = {
        transformCallback: (fn: Function) => {
          const id = w.nextCallback++;
          w.callbacks[id] = fn;
          return id;
        },
        unregisterCallback: (id: number) => {
          delete w.callbacks[id];
        },
        invoke: async (command: string, args: any) => {
          if (command === 'plugin:event|listen') {
            w.listeners[args.handler] = w.callbacks[args.handler];
            return args.handler;
          }
          if (command === 'plugin:event|unlisten') {
            delete w.listeners[args.eventId];
            return;
          }
          w.calls.push({ command, args });
          if (w.failures[command]) throw w.failures[command];
          if (w.defer[command])
            return new Promise((resolve, reject) => {
              w.defer[command] = { resolve, reject };
            });
          if (
            command.startsWith('workspace_') &&
            (!args || Object.keys(args).join() !== 'request')
          )
            throw {
              code: 'invalid_input',
              message: 'Expected request envelope',
              details: null,
            };
          const request = args?.request;
          switch (command) {
            case 'setup_inventory':
              if (options.nested) return { schema: 1, root_id: request.root_id, entry: 'new', folders: [{ path: 'Projects/Archive', role: 'project', reason: 'Suggested.', documents: 1, bytes: 28, dated: 1, undated: 0, linked: 0, date_from: '2026-11-01', date_to: '2026-11-01' }], documents: [{ source: { source_id: ids.source, path: 'Projects/Archive/one.md', hash, date: '2026-11-01', sensitivity: 'unknown' }, bytes: 28, title: 'One', linked: false, role: 'project', role_reason: 'Suggested.' }], excluded: [], eligible_bytes: 28, saved_topic_count: 0, resumable_job_ids: [] };
              if (options.inventoryEmpty) return { schema: 1, root_id: request.root_id, entry: 'empty', folders: [], documents: [], excluded: [], eligible_bytes: 0, saved_topic_count: 0, resumable_job_ids: [] };
              return { schema: 1, root_id: request.root_id, entry: 'new', folders: [{ path: 'Meetings', role: 'meeting', reason: 'Suggested from folder name.', documents: 1, bytes: 28, dated: 1, undated: 0, linked: 0, date_from: '2026-11-01', date_to: '2026-11-01' }], documents: [{ source: { source_id: ids.source, path: 'Meetings/one.md', hash, date: '2026-11-01', sensitivity: 'unknown' }, bytes: 28, title: 'Meeting', linked: false, role: 'meeting', role_reason: 'Suggested from folder name.' }], excluded: [{ path: 'Meetings/private.md', reason: 'Local-only' }], eligible_bytes: 28, saved_topic_count: 0, resumable_job_ids: [] };
            case 'setup_manifest_preview':
              w.setupManifest = { schema: 1, manifest_id: 'manifest-one', root_id: request.root_id, fingerprint: hash, spec: request.spec, roles: request.roles, limits: request.limits, runtime: request.runtime, created_at: '2026-11-03T00:00:00Z', batches: [{ batch_id: 'batch-one', sources: [{ source: { source_id: ids.source, path: 'Meetings/one.md', hash, date: '2026-11-01', sensitivity: 'unknown' }, bytes: 28, role: 'meeting' }] }], excluded: [{ path: 'Meetings/private.md', reason: 'Local-only' }] };
              return w.setupManifest;
            case 'setup_job_create':
              w.setupJob = { schema: 1, job_id: 'job-one', root_id: request.root_id, manifest: w.setupManifest, status: 'ready', revision: 1, batches: [], graph: { items: [{ item_id: 'item-topic', candidate: { kind: 'topic', topic }, disposition: 'pending', batch_ids: ['batch-one'], proposal_ids: [ids.proposal], independent_evidence_groups: [] }, { item_id: 'item-claim', candidate: { kind: 'claim', claim }, disposition: 'pending', batch_ids: ['batch-one'], proposal_ids: [ids.proposal], independent_evidence_groups: [] }, { item_id: 'item-alex', candidate: { kind: 'topic', topic: { ...topic, id: 'alex-id', topic_kind: 'person', name: 'Alex' } }, disposition: 'pending', batch_ids: ['batch-one'], proposal_ids: [ids.proposal], independent_evidence_groups: [] }, { item_id: 'item-edge', candidate: { kind: 'connection', connection: { id: 'edge-id', from: { kind: 'topic', id: 'alex-id' }, relation: 'works with', to: { kind: 'topic', id: ids.topic }, uncertainty: 'Association inferred', sources: [claim.sources[0].source], valid_from: null, valid_to: null } }, disposition: 'pending', batch_ids: ['batch-one'], proposal_ids: [ids.proposal], independent_evidence_groups: [] }], issues: [{ id: 'issue-one', issue_kind: 'inferred', key: 'Alex and Ledger association', item_ids: ['item-edge'], sources: [claim.sources[0].source], deferred: false }], canonical_ids: {} }, coverage: { selected: 1, processed: [], failed: [], outstanding: ['Meetings/one.md'], excluded: w.setupManifest.excluded, complete: false, model_calls: 0, transmission_authorizations: 1 }, created_at: '2026-11-03T00:00:00Z' };
              return w.setupJob;
            case 'setup_job_start':
              w.setupJob.status = 'finished'; w.setupJob.coverage.processed = ['Meetings/one.md']; w.setupJob.coverage.outstanding = []; w.setupJob.coverage.complete = true; return w.setupJob;
            case 'setup_job_get':
            case 'setup_job_watch': return w.setupJob;
            case 'setup_save_preview':
              return { schema: 1, receipt_id: 'receipt-one', root_id: request.root_id, kind: 'save', job_id: 'job-one', job_revision: 1, requested_ids: request.item_ids, included_ids: ['item-topic', 'item-claim'], dependency_ids: [], deferred_ids: [], candidates: [{ kind: 'topic', topic }, { kind: 'claim', claim }], revisions: [], expected_hashes: {}, fingerprint: hash, max_records_per_operation: 100, operation_count: 1, created_at: '2026-11-03T00:00:00Z' };
            case 'workspace_user_statement':
              w.savedContext = request.text;
              return { id: 'context-one', topic_id: request.topic_id, text: request.text, recorded_at: '2026-11-03T00:00:00Z' };
            case 'setup_confirm_preview':
              return { schema: 1, receipt_id: 'confirm-one', root_id: request.root_id, kind: 'confirm', job_id: null, job_revision: null, requested_ids: request.revision_ids, included_ids: request.revision_ids, dependency_ids: [], deferred_ids: [], candidates: [], revisions: w.claims.filter((c: any) => request.revision_ids.includes(c.revision_id)), expected_hashes: {}, fingerprint: hash, max_records_per_operation: 100, operation_count: 1, created_at: '2026-11-03T00:00:00Z' };
            case 'setup_correction_preview': {
              const before = w.claims.filter((c: any) => request.revision_ids.includes(c.revision_id));
              const after = before.map((c: any) => ({ ...c, value: request.replacement, revision_id: 'corrected-revision', supersedes: [c.revision_id] }));
              const rule = { id: 'corrected-rule', topic_id: request.topic_id, predicate: request.predicate, match_value: before[0].value, replacement: request.replacement, scope: request.rule_scope, source_revision_id: 'corrected-revision' };
              return { statement: { id: 'statement-one', topic_id: request.topic_id, text: request.statement, recorded_at: '2026-11-03T00:00:00Z' }, before, after, rule, receipt: { schema: 1, receipt_id: 'correction-receipt', root_id: request.root_id, kind: 'correction', job_id: null, job_revision: null, requested_ids: request.revision_ids, included_ids: request.revision_ids, dependency_ids: [], deferred_ids: [], candidates: after.map((c: any) => ({ kind: 'correction', claim: c, rule })), revisions: after, expected_hashes: {}, fingerprint: hash, max_records_per_operation: 100, operation_count: 1, created_at: '2026-11-03T00:00:00Z' } };
            }
            case 'setup_topic_review':
              return { topic: { topic, current_claims: w.claims, connections: [], unresolved: [], history: [], warnings: [] }, reviews: w.claims.map((c: any) => ({ id: 'review', record_id: c.revision_id, state: 'from_notes', receipt_id: 'receipt-one', statement_id: null, recorded_at: '2026-11-03T00:00:00Z' })) };
            case 'workspace_choose_root':
              return w.chosenRoot ?? null;
            case 'workspace_root_get':
              return { root: w.root };
            case 'workspace_root_set':
              w.root = {
                root_id: request.path.includes('other')
                  ? 'other-root'
                  : ids.root,
                path: request.path,
              };
              w.topics = [];
              w.proposals = [];
              return {
                root: w.root,
                index: { documents: 0, people: 0, warnings: [] },
              };
            case 'workspace_runtime_status':
              return {
                available: !options.unavailable,
                version: options.unavailable ? null : '0.166.0',
                message: options.unavailable
                  ? 'Codex is missing. Install Codex 0.166.0 and restart Topics.'
                  : 'Ready',
              };
            case 'workspace_scope_preview':
              return {
                scope_id: ids.scope,
                root_id: w.root.root_id,
                sources: [
                  {
                    source_id: ids.source,
                    path: 'Meetings/one.md',
                    hash,
                    date: '2026-11-01',
                    sensitivity: 'unknown',
                  },
                ],
                excluded: [
                  { path: 'Meetings/two.md', reason: 'Outside date range' },
                  { path: 'Meetings/undated.md', reason: 'Unknown date' },
                  { path: 'Meetings/private.md', reason: 'Local-only' },
                ],
              };
            case 'workspace_analysis_start':
              return { run_id: ids.run };
            case 'workspace_chat_start':
              return { run_id: ids.run, message_id: 'message-one' };
            case 'workspace_run_cancel':
              return { cancelled: true };
            case 'workspace_proposals_list':
              return { proposals: w.proposals };
            case 'workspace_topics_list':
              return { topics: w.topics };
            case 'workspace_statement_get':
              return {
                id: request.statement_id,
                topic_id: ids.topic,
                text: 'The launch is Tuesday; Monday was a mistake.',
                recorded_at: '2026-11-03T00:00:00Z',
              };
            case 'workspace_topic_get':
              return {
                topic:
                  w.topics.find((t: any) => t.id === request.topic_id) || topic,
                current_claims: w.claims,
                connections: [],
                unresolved: [],
                history: w.claimHistory || [],
                warnings: [],
              };
            case 'workspace_proposal_decide': {
              const p = w.proposals.find(
                (p: any) => p.id === request.proposal_id,
              );
              p.status =
                request.decision.kind === 'reject' ? 'rejected' : 'accepted';
              if (p.status === 'accepted') {
                w.claimHistory = [...(w.claimHistory || []), ...w.claims];
                w.topics = [
                  ...w.topics.filter(
                    (t: any) =>
                      !request.decision.candidates.some(
                        (c: any) => c.kind === 'topic' && c.topic.id === t.id,
                      ),
                  ),
                  ...request.decision.candidates
                    .filter((c: any) => c.kind === 'topic')
                    .map((c: any) => c.topic),
                ];
                w.claims = request.decision.candidates
                  .filter((c: any) => c.claim)
                  .map((c: any) => c.claim);
                w.operations.push({
                  id: `operation-${w.operations.length + 1}`,
                  status: 'committed',
                  description: 'Accepted Ledger launch',
                  created_at: '2026-11-03T00:00:00Z',
                });
              }
              return { proposal: p, operation: w.operations.at(-1) || null };
            }
            case 'workspace_history_list':
              return { operations: w.operations };
            case 'workspace_undo':
              throw {
                code: 'conflict',
                message: 'Undo the dependent claim first.',
                details: { dependencies: [ids.claim] },
              };
            case 'workspace_sources_list':
              return {
                sources: [
                  {
                    source_id: ids.source,
                    path: 'Meetings/one.md',
                    hash,
                    date: '2026-11-01',
                    sensitivity: w.sourceSensitivity || 'unknown',
                  },
                  {
                    source_id: 'private',
                    path: 'Meetings/private.md',
                    hash,
                    date: null,
                    sensitivity: 'local_only',
                  },
                ],
              };
            case 'workspace_source_get':
              return {
                summary: {
                  source_id: request.source_id,
                  path:
                    request.source_id === 'private'
                      ? 'Meetings/private.md'
                      : 'Meetings/one.md',
                  hash,
                  date: null,
                  sensitivity:
                    request.source_id === 'private'
                      ? 'local_only'
                      : w.sourceSensitivity || 'unknown',
                },
                content: '<img src=x onerror=alert(1)> Ledger launch is Friday',
                citation: proposal.candidates[1].claim.sources[0].source,
              };
            case 'workspace_manual_propose': {
              const p = {
                ...structuredClone(proposal),
                candidates: request.candidates,
                origin: { kind: 'manual' },
                scope_id: null,
              };
              w.proposals = [p];
              return { proposal: p };
            }
            case 'workspace_sensitivity_set':
              w.sourceSensitivity = request.sensitivity;
              return {
                operation: {
                  id: 'label-one',
                  status: 'committed',
                  description: 'Changed sensitivity',
                  created_at: '2026-11-03T00:00:00Z',
                },
              };
            case 'search_notes':
              return [];
            default:
              throw new Error(`Unexpected IPC: ${command}`);
          }
        },
      };
      w.showCorrection = () => {
        const previous = w.claims[0];
        const revised = {
          ...structuredClone(previous),
          revision_id: '00000000-0000-4000-8000-000000000009',
          value: 'Tuesday',
          supersedes: [previous.revision_id],
          sources: [{ kind: 'user_statement', statement_id: 'statement-one' }],
        };
        w.proposals = [
          {
            ...structuredClone(proposal),
            id: 'correction-one',
            origin: {
              kind: 'chat',
              topic_id: ids.topic,
              message_id: 'message-one',
            },
            candidates: [
              {
                kind: 'correction',
                claim: revised,
                rule: {
                  id: 'rule-one',
                  topic_id: ids.topic,
                  predicate: 'launch',
                  match_value: previous.value,
                  replacement: 'Tuesday',
                  scope: {
                    source_paths: ['Meetings/one.md'],
                    context: 'Monday was a mistake.',
                  },
                  source_revision_id: revised.revision_id,
                },
              },
            ],
          },
        ];
      };
      w.showProposal = () => {
        w.proposals = [structuredClone(proposal)];
      };
    },
    { ids, topic, claim, proposal, hash, options },
  );
  await page.goto('/');
}
