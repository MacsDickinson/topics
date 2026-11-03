import { createRoot } from 'react-dom/client';
import { useEffect, useRef, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { Search } from './Search';
import { Sources } from './Sources';
import { ipc, apiError, errorHelp } from './ipc';
import { ProposalEditor } from './ProposalEditor';
import { TopicDetail } from './TopicDetail';
import { emptySpec, ScopeFields, ScopeReview } from './Scope';
import type {
  ApiError,
  Candidate,
  Operation,
  Proposal,
  RootRef,
  RuntimeStatus,
  ScopePreview,
  ScopeSpec,
  Sensitivity,
  Topic,
  TopicView,
  WorkspaceRunEvent,
} from './workspace-types';
import './style.css';
if (import.meta.env.VITE_NATIVE_TEST === '1')
  await import('@wdio/tauri-plugin');
type Screen = 'search' | 'topics' | 'inbox' | 'history' | 'analyse' | 'sources';
type ActiveRun = { root: string; id: string; seq: number };
function validEvent(value: unknown): value is WorkspaceRunEvent {
  if (!value || typeof value !== 'object') return false;
  const e = value as WorkspaceRunEvent;
  if (
    typeof e.root_id !== 'string' ||
    typeof e.run_id !== 'string' ||
    !Number.isSafeInteger(e.seq) ||
    e.seq < 0 ||
    !e.event ||
    typeof e.event !== 'object'
  )
    return false;
  switch (e.event.kind) {
    case 'text_delta':
      return typeof e.event.text === 'string';
    case 'proposal':
      return typeof e.event.proposal_id === 'string';
    case 'error':
      return (
        !!e.event.error &&
        typeof e.event.error.message === 'string' &&
        typeof e.event.error.code === 'string'
      );
    case 'completed':
    case 'cancelled':
      return true;
    default:
      return false;
  }
}
function App() {
  const [root, setRoot] = useState<RootRef | null>();
  const rootRef = useRef<RootRef | null>(null);
  const epoch = useRef(0),
    detailSeq = useRef(0),
    refreshSeq = useRef(0),
    scopeSeq = useRef(0);
  const [screen, setScreen] = useState<Screen>('search');
  const [settings, setSettings] = useState(false),
    [path, setPath] = useState(''),
    [create, setCreate] = useState(false);
  const [topics, setTopics] = useState<Topic[]>([]),
    [proposals, setProposals] = useState<Proposal[]>([]),
    [operations, setOperations] = useState<Operation[]>([]);
  const [topic, setTopic] = useState<TopicView | null>(null),
    [topicLoading, setTopicLoading] = useState(false);
  const [runtime, setRuntime] = useState<RuntimeStatus | null>(null),
    [error, setError] = useState<ApiError | null>(null),
    [bootError, setBootError] = useState<ApiError | null>(null);
  const [busy, setBusy] = useState(false),
    [notice, setNotice] = useState('');
  const [spec, setSpec] = useState<ScopeSpec>(emptySpec),
    [preview, setPreview] = useState<ScopePreview | null>(null),
    [confirmed, setConfirmed] = useState(false);
  const [message, setMessage] = useState(''),
    [manual, setManual] = useState(false),
    [name, setName] = useState(''),
    [kind, setKind] = useState<Topic['topic_kind']>('thread'),
    [manualSensitivity, setManualSensitivity] =
      useState<Sensitivity>('unknown');
  const [run, setRun] = useState<ActiveRun | null>(null),
    [stream, setStream] = useState('');
  const runRef = useRef<ActiveRun | null>(null),
    pendingStart = useRef(false),
    earlyEvents = useRef<WorkspaceRunEvent[]>([]),
    eventReady = useRef<Promise<void>>(Promise.resolve());
  const current = (generation: number, id?: string) =>
    epoch.current === generation && (!id || rootRef.current?.root_id === id);
  const report = (e: unknown) => setError(apiError(e));
  const invalidateScope = () => {
    ++scopeSeq.current;
    setPreview(null);
    setConfirmed(false);
  };
  const clearTopic = () => {
    ++detailSeq.current;
    setTopic(null);
    setTopicLoading(false);
    setMessage('');
    invalidateScope();
  };
  async function refresh(selected: RootRef, generation = epoch.current) {
    const sequence = ++refreshSeq.current;
    const results = await Promise.allSettled([
      ipc<{ topics: Topic[] }>('workspace_topics_list', {
        root_id: selected.root_id,
      }),
      ipc<{ proposals: Proposal[] }>('workspace_proposals_list', {
        root_id: selected.root_id,
      }),
      ipc<{ operations: Operation[] }>('workspace_history_list', {
        root_id: selected.root_id,
      }),
    ]);
    if (
      !current(generation, selected.root_id) ||
      sequence !== refreshSeq.current
    )
      return;
    if (results[0].status === 'fulfilled') setTopics(results[0].value.topics);
    if (results[1].status === 'fulfilled')
      setProposals(
        results[1].value.proposals.filter(
          (p) => p.root_id === selected.root_id,
        ),
      );
    if (results[2].status === 'fulfilled')
      setOperations(results[2].value.operations);
    const failure = results.find((r) => r.status === 'rejected');
    if (failure?.status === 'rejected') report(failure.reason);
  }
  async function openTopic(id: string) {
    const selected = rootRef.current;
    if (!selected) return;
    window.scrollTo(0, 0);
    const generation = epoch.current,
      sequence = ++detailSeq.current;
    setTopic(null);
    setTopicLoading(true);
    setMessage('');
    invalidateScope();
    setError(null);
    try {
      const result = await ipc<TopicView>('workspace_topic_get', {
        root_id: selected.root_id,
        topic_id: id,
      });
      if (
        current(generation, selected.root_id) &&
        sequence === detailSeq.current
      )
        setTopic(result);
    } catch (e) {
      if (
        current(generation, selected.root_id) &&
        sequence === detailSeq.current
      )
        report(e);
    } finally {
      if (
        current(generation, selected.root_id) &&
        sequence === detailSeq.current
      )
        setTopicLoading(false);
    }
  }
  function receive(value: unknown) {
    if (!validEvent(value) || value.root_id !== rootRef.current?.root_id)
      return;
    const active = runRef.current;
    if (!active) {
      if (pendingStart.current && earlyEvents.current.length < 512)
        earlyEvents.current.push(value);
      return;
    }
    if (
      active.root !== value.root_id ||
      active.id !== value.run_id ||
      value.seq <= active.seq
    )
      return;
    active.seq = value.seq;
    if (value.event.kind === 'text_delta') {
      const delta = value.event.text;
      setStream((text) => (text + delta).slice(-100000));
    } else if (['completed', 'cancelled', 'error'].includes(value.event.kind)) {
      runRef.current = null;
      setRun(null);
      pendingStart.current = false;
      earlyEvents.current = [];
      if (value.event.kind === 'error') {
        report(value.event.error);
        setNotice('Run failed. No model changes were accepted.');
      } else if (value.event.kind === 'cancelled')
        setNotice('Run stopped. No model changes were accepted.');
      else {
        setNotice('Analysis complete. Review proposed changes in Inbox.');
        const selected = rootRef.current;
        if (selected) void refresh(selected);
      }
    }
    // Proposal events never refresh data until the successful terminal event.
  }
  const receiveRef = useRef(receive);
  receiveRef.current = receive;
  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    eventReady.current = listen<WorkspaceRunEvent>(
      'workspace_run_event',
      (event) => receiveRef.current(event.payload),
    ).then((fn) => {
      if (disposed) fn();
      else unlisten = fn;
    });
    void eventReady.current.catch(() => undefined);
    const generation = epoch.current;
    void ipc<{ root: RootRef | null }>('workspace_root_get')
      .then((result) => {
        if (disposed || !current(generation)) return;
        rootRef.current = result.root;
        setRoot(result.root);
        setPath(result.root?.path || '');
        if (result.root) void refresh(result.root, generation);
      })
      .catch((e) => {
        if (!disposed && current(generation)) {
          setBootError(apiError(e));
          setRoot(null);
        }
      });
    void checkRuntime();
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
  useEffect(() => {
    const shortcut = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        setScreen('search');
        setManual(false);
        setSettings(false);
        requestAnimationFrame(() => {
          const input = document.querySelector<HTMLInputElement>('#search');
          input?.focus();
          input?.select();
        });
      }
    };
    window.addEventListener('keydown', shortcut);
    return () => window.removeEventListener('keydown', shortcut);
  }, []);
  async function checkRuntime() {
    try {
      setRuntime(await ipc<RuntimeStatus>('workspace_runtime_status'));
    } catch (e) {
      setRuntime({
        available: false,
        version: null,
        message: apiError(e).message,
      });
    }
  }
  async function stop() {
    const active = runRef.current || run;
    if (!active) return;
    const generation = epoch.current;
    // Close events immediately, but keep a retry action until IPC confirms stop.
    runRef.current = null; pendingStart.current = false; earlyEvents.current = [];
    setError(null); setNotice('Stopping run…');
    try {
      const result = await ipc<{cancelled:boolean}>('workspace_run_cancel', { root_id: active.root, run_id: active.id });
      if (!current(generation, active.root)) return;
      setRun(null);
      setNotice(result.cancelled ? 'Run stopped. No model changes were accepted.' : 'Run is no longer active. Review Inbox for any completed proposals.');
      if (!result.cancelled && rootRef.current) await refresh(rootRef.current, generation);
    } catch (e) {
      if (current(generation, active.root)) { setNotice('Cancellation was not confirmed. Retry Stop run or change notes folder.'); report(e); }
    }
  }
  async function changeRoot() {
    if (!path.trim() || busy) return;
    setBusy(true);
    setError(null);
    setBootError(null);
    const generation = ++epoch.current;
    const active = runRef.current || run;
    runRef.current = null;
    pendingStart.current = false;
    earlyEvents.current = [];
    setRun(null);
    setStream('');
    setNotice('');
    rootRef.current = null;
    setRoot(null);
    setTopics([]);
    setSpec(emptySpec);
    setName('');
    setKind('thread');
    setManualSensitivity('unknown');
    setProposals([]);
    setOperations([]);
    clearTopic();
    setManual(false);
    try {
      if (active)
        await ipc('workspace_run_cancel', {
          root_id: active.root,
          run_id: active.id,
        });
      const result = await ipc<{
        root: RootRef;
        index: { document_count: number; warnings: string[] };
      }>('workspace_root_set', { path: path.trim(), create });
      if (!current(generation)) return;
      rootRef.current = result.root;
      setRoot(result.root);
      setPath(result.root.path);
      setSettings(false);
      setScreen('topics');
      setCreate(false);
      if (result.index.warnings?.length)
        setNotice(result.index.warnings.join('\n'));
      await refresh(result.root, generation);
    } catch (e) {
      if (current(generation)) report(e);
    } finally {
      if (current(generation)) setBusy(false);
    }
  }
  async function getPreview(chat = false) {
    const selected = rootRef.current;
    if (!selected) return;
    setError(null);
    setBusy(true);
    const generation = epoch.current,
      sequence = ++scopeSeq.current;
    setConfirmed(false);
    setPreview(null);
    const chatFiles = topic
      ? [
          ...new Set([
            ...[...topic.current_claims, ...topic.history].flatMap((c) =>
              c.sources.flatMap((s) =>
                s.kind === 'passage' ? [s.source.path] : [],
              ),
            ),
            ...topic.connections.flatMap((c) => c.sources.map((s) => s.path)),
          ]),
        ]
      : [];
    try {
      const result = await ipc<ScopePreview>('workspace_scope_preview', {
        root_id: selected.root_id,
        spec: chat ? { ...emptySpec, files: chatFiles } : spec,
      });
      if (
        current(generation, selected.root_id) &&
        sequence === scopeSeq.current
      )
        setPreview(result);
    } catch (e) {
      if (
        current(generation, selected.root_id) &&
        sequence === scopeSeq.current
      )
        report(e);
    } finally {
      if (current(generation, selected.root_id)) setBusy(false);
    }
  }
  const canSend =
    !!preview &&
    preview.root_id === root?.root_id &&
    preview.sources.every((s) => s.sensitivity !== 'local_only') &&
    (!preview.sources.some((s) => s.sensitivity === 'unknown') || confirmed);
  async function start(chat = false) {
    const selected = rootRef.current;
    if (
      !selected ||
      !preview ||
      !canSend ||
      busy ||
      runRef.current ||
      !runtime?.available
    )
      return;
    const generation = epoch.current;
    setBusy(true);
    setError(null);
    setNotice('');
    setStream('');
    pendingStart.current = true;
    earlyEvents.current = [];
    try {
      await eventReady.current;
      if (!current(generation, selected.root_id)) return;
      const request = {
        root_id: selected.root_id,
        scope_id: preview.scope_id,
        confirm_unknown: confirmed,
      };
      const result = await ipc<{ run_id: string }>(
        chat ? 'workspace_chat_start' : 'workspace_analysis_start',
        chat ? { ...request, topic_id: topic!.topic.id, message } : request,
      );
      if (!current(generation, selected.root_id)) {
        void ipc('workspace_run_cancel', {
          root_id: selected.root_id,
          run_id: result.run_id,
        }).catch(() => undefined);
        return;
      }
      const active = { root: selected.root_id, id: result.run_id, seq: -1 };
      runRef.current = active;
      setRun(active);
      pendingStart.current = false;
      for (const event of earlyEvents.current.splice(0)) receive(event);
      invalidateScope();
    } catch (e) {
      if (current(generation, selected.root_id)) {
        pendingStart.current = false;
        earlyEvents.current = [];
        report(e);
        if (apiError(e).code === 'stale_scope') invalidateScope();
      }
    } finally {
      if (current(generation, selected.root_id)) setBusy(false);
    }
  }
  async function decide(proposal: Proposal, candidates: Candidate[] | null) {
    const selected = rootRef.current;
    if (!selected || busy) return;
    const generation = epoch.current;
    setBusy(true);
    setError(null);
    try {
      await ipc('workspace_proposal_decide', {
        root_id: selected.root_id,
        proposal_id: proposal.id,
        expected_fingerprint: proposal.fingerprint,
        decision:
          candidates === null
            ? { kind: 'reject' }
            : { kind: 'approve', candidates },
      });
      if (current(generation, selected.root_id)) {
        setNotice(
          candidates === null
            ? 'Proposal rejected.'
            : 'Changes accepted. Your source notes were not rewritten.',
        );
        await refresh(selected, generation);
      }
    } catch (e) {
      if (current(generation, selected.root_id)) report(e);
    } finally {
      if (current(generation, selected.root_id)) setBusy(false);
    }
  }
  async function manualPropose() {
    const selected = rootRef.current;
    if (!selected || !name.trim() || busy) return;
    const generation = epoch.current;
    setBusy(true);
    setError(null);
    try {
      const candidates: Candidate[] = [
        {
          kind: 'topic',
          topic: {
            id: crypto.randomUUID(),
            topic_kind: kind,
            name: name.trim(),
            aliases: [],
            sensitivity: manualSensitivity,
          },
        },
      ];
      await ipc('workspace_manual_propose', {
        root_id: selected.root_id,
        candidates,
      });
      if (current(generation, selected.root_id)) {
        setManual(false);
        setName('');
        setScreen('inbox');
        await refresh(selected, generation);
      }
    } catch (e) {
      if (current(generation, selected.root_id)) report(e);
    } finally {
      if (current(generation, selected.root_id)) setBusy(false);
    }
  }
  async function undo(id: string) {
    const selected = rootRef.current;
    if (!selected || busy) return;
    const generation = epoch.current;
    setBusy(true);
    setError(null);
    try {
      await ipc('workspace_undo', {
        root_id: selected.root_id,
        operation_id: id,
      });
      if (current(generation, selected.root_id)) {
        setNotice('Undo recorded in history.');
        clearTopic();
        await refresh(selected, generation);
      }
    } catch (e) {
      if (current(generation, selected.root_id)) report(e);
    } finally {
      if (current(generation, selected.root_id)) setBusy(false);
    }
  }
  async function sensitivity(value: Sensitivity) {
    const selected = rootRef.current;
    if (!selected || !topic || busy) return;
    const generation = epoch.current,
      id = topic.topic.id;
    setBusy(true);
    setError(null);
    invalidateScope();
    try {
      await ipc('workspace_sensitivity_set', {
        root_id: selected.root_id,
        target: { kind: 'topic', id },
        sensitivity: value,
      });
      if (current(generation, selected.root_id)) {
        await refresh(selected, generation);
        await openTopic(id);
      }
    } catch (e) {
      if (current(generation, selected.root_id)) report(e);
    } finally {
      if (current(generation, selected.root_id)) setBusy(false);
    }
  }
  const pendingCount = proposals.filter((p) => p.status === 'pending').length;
  const navigate = (next: Screen) => {
    window.scrollTo(0, 0);
    setScreen(next);
    setManual(false);
    setSettings(false);
    setError(null);
    invalidateScope();
  };
  return (
    <div className="workspace">
      <aside className="sidebar">
        <div className="brand">
          Topics
          <span className="brand-mark" aria-hidden="true">
            ◫
          </span>
        </div>
        <nav aria-label="Workspace">
          <button
            aria-label="Topics"
            aria-current={screen === 'topics' ? 'page' : undefined}
            onClick={() => navigate('topics')}
          >
            Topics<span>{topics.length || ''}</span>
          </button>
          <button
            aria-current={screen === 'search' ? 'page' : undefined}
            onClick={() => navigate('search')}
          >
            Search<span>⌘K</span>
          </button>
          <button
            aria-current={screen === 'inbox' ? 'page' : undefined}
            onClick={() => navigate('inbox')}
          >
            Inbox<span>{pendingCount || ''}</span>
          </button>
          <button
            aria-current={screen === 'history' ? 'page' : undefined}
            onClick={() => navigate('history')}
          >
            History
          </button>
          <button
            aria-current={screen === 'sources' ? 'page' : undefined}
            onClick={() => navigate('sources')}
          >
            Sources
          </button>
        </nav>
        <div className="sidebar-actions">
          <button disabled={!root} onClick={() => navigate('analyse')}>
            Analyse notes
          </button>
          <button
            disabled={!root}
            onClick={() => {
              setManual(true);
              setSettings(false);
              setError(null);
            }}
          >
            New topic
          </button>
        </div>
        <div className="sidebar-bottom">
          <p className="root-path" title={root?.path}>
            {root?.path || 'No notes folder selected'}
          </p>
          <button
            className="quiet"
            onClick={() => {
              setSettings(true);
              setManual(false);
              setError(null);
            }}
          >
            Change notes folder
          </button>
          {runtime && (
            <div className={`runtime ${runtime.available ? 'ready' : ''}`}>
              <p>
                {runtime.available
                  ? `Codex ${runtime.version || ''} installed`
                  : runtime.message}
              </p>
              {!runtime.available && (
                <button className="quiet" onClick={() => void checkRuntime()}>
                  Check runtime again
                </button>
              )}
            </div>
          )}
        </div>
      </aside>
      <main className="workspace-main">
        {(settings || (root === null && !bootError)) && (
          <section className="setup">
            <header className="page-header">
              <h1>{root ? 'Choose notes folder' : 'Start with your notes'}</h1>
              <p>
                Choose an existing folder of Markdown notes, or create a new
                one. Your original notes stay yours.
              </p>
            </header>
            <form
              onSubmit={(e) => {
                e.preventDefault();
                void changeRoot();
              }}
            >
              <label htmlFor="root-path">
                Notes folder path
                <input
                  id="root-path"
                  placeholder="/absolute/path/to/notes"
                  value={path}
                  onChange={(e) => setPath(e.target.value)}
                  required
                />
              </label>
              <label className="check">
                <input
                  type="checkbox"
                  checked={create}
                  onChange={(e) => setCreate(e.target.checked)}
                />
                Create a new notes folder
              </label>
              <div className="actions">
                <button className="primary" disabled={busy || !path.trim()}>
                  Use notes folder
                </button>
                {root && (
                  <button type="button" onClick={() => setSettings(false)}>
                    Cancel
                  </button>
                )}
              </div>
            </form>
          </section>
        )}
        {bootError && settings && <p className="notice">{bootError.message}</p>}
        {error && (
          <div role="alert" className="error">
            <p>{error.message}</p>
            {errorHelp(error) && <p>{errorHelp(error)}</p>}
            {error.details != null && (
              <details>
                <summary>Related changes</summary>
                <pre>{JSON.stringify(error.details, null, 2)}</pre>
              </details>
            )}
            <button className="quiet" onClick={() => setError(null)}>
              Dismiss error
            </button>
          </div>
        )}
        {notice && (
          <p aria-live="polite" className="notice">
            {notice}
          </p>
        )}
        {manual ? (
          <section className="manual-form">
            <header className="page-header">
              <h1>New topic</h1>
              <p>
                Capture a topic by hand. You’ll review the proposal before it
                becomes accepted knowledge.
              </p>
            </header>
            <form
              onSubmit={(e) => {
                e.preventDefault();
                void manualPropose();
              }}
            >
              <label htmlFor="topic-name">
                Topic name
                <input
                  id="topic-name"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  autoFocus
                  required
                />
              </label>
              <label>
                Kind
                <select
                  value={kind}
                  onChange={(e) =>
                    setKind(e.target.value as Topic['topic_kind'])
                  }
                >
                  <option value="thread">Thread</option>
                  <option value="person">Person</option>
                  <option value="project">Project</option>
                </select>
              </label>
              <label>
                Sensitivity
                <select
                  value={manualSensitivity}
                  onChange={(e) =>
                    setManualSensitivity(e.target.value as Sensitivity)
                  }
                >
                  <option value="unknown">Unknown</option>
                  <option value="local_only">Local-only</option>
                  <option value="cloud_allowed">Cloud allowed</option>
                </select>
              </label>
              <div className="actions">
                <button className="primary" disabled={busy || !name.trim()}>
                  Create proposal
                </button>
                <button type="button" onClick={() => setManual(false)}>
                  Cancel
                </button>
              </div>
            </form>
          </section>
        ) : (
          <>
            {screen === 'search' && (
              <Search key={root?.root_id || `none-${epoch.current}`} />
            )}
            {screen === 'topics' && (
              <div className="topic-workspace">
                <section className="topic-list">
                  <header className="page-header">
                    <h1>Topics</h1>
                    <p>People, projects and ongoing threads.</p>
                  </header>
                  {topics.length === 0 ? (
                    <div className="empty">
                      <h2>No accepted topics yet</h2>
                      <p>
                        Add Markdown notes to this folder, then analyse a
                        selection.
                      </p>
                      <p>You can also create a topic proposal by hand.</p>
                    </div>
                  ) : (
                    <div className="topic-rows">
                      {topics.map((t) => (
                        <button
                          key={t.id}
                          aria-label={t.name}
                          aria-pressed={topic?.topic.id === t.id}
                          onClick={() => void openTopic(t.id)}
                        >
                          <span>{t.name}</span>
                          <span className="muted">
                            {t.topic_kind === 'person'
                              ? 'Person'
                              : t.topic_kind === 'project'
                                ? 'Project'
                                : 'Thread'}
                            {t.sensitivity === 'local_only'
                              ? ' · Local-only'
                              : ''}
                          </span>
                        </button>
                      ))}
                    </div>
                  )}
                </section>
                <section className="topic-detail" aria-label="Topic detail">
                  {topicLoading ? (
                    <p aria-live="polite">Opening topic…</p>
                  ) : topic ? (
                    <>
                      <TopicDetail
                        rootId={root!.root_id}
                        view={topic}
                        topics={topics}
                        busy={busy}
                        sensitivity={(s) => void sensitivity(s)}
                      />
                      <section className="chat">
                        <h2>Ask or correct</h2>
                        <p className="muted">
                          Explain what should change. Changes return to Inbox
                          for your review.
                        </p>
                        <label htmlFor="topic-message">
                          Message about this topic
                          <textarea
                            id="topic-message"
                            rows={3}
                            value={message}
                            onChange={(e) => setMessage(e.target.value)}
                            placeholder="What should I know about this topic?"
                          />
                        </label>
                        <button
                          disabled={
                            busy ||
                            !!run ||
                            topic.topic.sensitivity !== 'cloud_allowed'
                          }
                          onClick={() => void getPreview(true)}
                        >
                          Preview chat sources
                        </button>
                        {preview && (
                          <ScopeReview
                            preview={preview}
                            confirmed={confirmed}
                            setConfirmed={setConfirmed}
                          />
                        )}
                        <div className="actions">
                          <button
                            className="primary"
                            disabled={
                              !message.trim() ||
                              !canSend ||
                              busy ||
                              !!run ||
                              !runtime?.available ||
                              topic.topic.sensitivity !== 'cloud_allowed'
                            }
                            onClick={() => void start(true)}
                          >
                            Send message
                          </button>
                        </div>
                        {topic.topic.sensitivity !== 'cloud_allowed' && (
                          <p className="notice">
                            {topic.topic.sensitivity === 'local_only'
                              ? 'This topic is local-only. Its context cannot be sent to Codex.'
                              : 'Set this topic to Cloud allowed before chatting. Local-only sources remain excluded.'}
                          </p>
                        )}
                      </section>
                    </>
                  ) : (
                    topics.length > 0 && (
                      <div className="empty">
                        <h2>Choose a topic</h2>
                        <p>
                          Read its accepted understanding, evidence and earlier
                          revisions.
                        </p>
                      </div>
                    )
                  )}
                </section>
              </div>
            )}
            {screen === 'analyse' && (
              <section className="analysis-view">
                <header className="page-header">
                  <h1>Analyse notes</h1>
                  <p>
                    Select a small set of notes. Nothing is accepted until you
                    review it.
                  </p>
                </header>
                <ScopeFields
                  spec={spec}
                  disabled={busy || !!run}
                  onChange={(value) => {
                    setSpec(value);
                    invalidateScope();
                  }}
                />
                <button
                  disabled={
                    busy ||
                    !!run ||
                    !root ||
                    (!spec.files.length &&
                      !spec.folders.length &&
                      !spec.date_from &&
                      !spec.date_to)
                  }
                  onClick={() => void getPreview()}
                >
                  Preview selection
                </button>
                {preview && (
                  <ScopeReview
                    preview={preview}
                    confirmed={confirmed}
                    setConfirmed={setConfirmed}
                  />
                )}
                <div className="actions">
                  <button
                    className="primary"
                    disabled={
                      !canSend ||
                      !preview?.sources.length ||
                      busy ||
                      !!run ||
                      !runtime?.available
                    }
                    onClick={() => void start()}
                  >
                    Start analysis
                  </button>
                </div>
                {!runtime?.available && (
                  <p className="notice">
                    Check the runtime status in the sidebar before starting.
                    Selection previews remain available.
                  </p>
                )}
              </section>
            )}
            {screen === 'inbox' && (
              <section className="inbox">
                <header className="page-header">
                  <h1>Inbox</h1>
                  <p>
                    Proposals need your review. Approval is the only way they
                    become accepted knowledge.
                  </p>
                </header>
                {proposals.length === 0 && (
                  <div className="empty">
                    <h2>Nothing to review</h2>
                    <p>
                      Analyse a selection of notes or create a topic proposal.
                    </p>
                  </div>
                )}
                {proposals.map((p) => (
                  <ProposalEditor
                    key={`${p.id}-${p.status}-${p.fingerprint}`}
                    proposal={p}
                    topics={topics}
                    busy={busy}
                    decide={(c) => decide(p, c)}
                  />
                ))}
              </section>
            )}
            {screen === 'history' && (
              <section className="history">
                <header className="page-header">
                  <h1>History</h1>
                  <p>
                    Accepted changes and undo operations. Earlier knowledge
                    remains in the record.
                  </p>
                </header>
                {operations.length === 0 && (
                  <p className="empty">No recorded changes yet.</p>
                )}
                {operations.map((o) => (
                  <article className="operation" key={o.id}>
                    <div>
                      <h2>{o.description}</h2>
                      <p className="muted">
                        {new Date(o.created_at).toLocaleString()} · {o.status}
                      </p>
                      <details>
                        <summary>Change identity</summary>
                        <p className="path">{o.id}</p>
                      </details>
                    </div>
                    <button
                      disabled={busy || o.status !== 'committed'}
                      onClick={() => void undo(o.id)}
                    >
                      Undo
                    </button>
                  </article>
                ))}
              </section>
            )}
            {screen === 'sources' && root && (
              <Sources
                key={root.root_id}
                root={root}
                disabled={busy || !!run}
                onError={report}
                onChanged={invalidateScope}
              />
            )}
          </>
        )}
        {(run || stream) && (
          <section className="run-panel" aria-label="Analysis progress">
            <div className="run-heading">
              <h2>{run ? 'Working on your selection' : 'Run response'}</h2>
              {run && <button onClick={() => void stop()}>Stop run</button>}
            </div>
            <div role="log" aria-live="polite" className="stream">
              {stream || 'Waiting for the runtime…'}
            </div>
          </section>
        )}
      </main>
    </div>
  );
}
createRoot(document.getElementById('root')!).render(<App />);
