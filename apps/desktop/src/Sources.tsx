import { useEffect, useRef, useState } from 'react';
import { ipc } from './ipc';
import type {
  RootRef,
  ScopedSource,
  Sensitivity,
  SourceSummary,
} from './workspace-types';
export function Sources({
  root,
  disabled,
  onError,
  onChanged,
}: {
  root: RootRef;
  disabled: boolean;
  onError: (e: unknown) => void;
  onChanged: () => void;
}) {
  const [sources, setSources] = useState<SourceSummary[]>([]),
    [selected, setSelected] = useState<ScopedSource | null>(null),
    [loading, setLoading] = useState(false),
    [writing, setWriting] = useState(false);
  const alive = useRef(true),
    sequence = useRef(0);
  useEffect(() => {
    alive.current = true;
    void ipc<{ sources: SourceSummary[] }>('workspace_sources_list', {
      root_id: root.root_id,
    })
      .then((r) => {
        if (alive.current) setSources(r.sources);
      })
      .catch((e) => {
        if (alive.current) onError(e);
      });
    return () => {
      alive.current = false;
      ++sequence.current;
    };
  }, [root.root_id]);
  async function open(id: string) {
    const seq = ++sequence.current;
    setSelected(null);
    setLoading(true);
    try {
      const result = await ipc<ScopedSource>('workspace_source_get', {
        root_id: root.root_id,
        source_id: id,
      });
      if (alive.current && seq === sequence.current) setSelected(result);
    } catch (e) {
      if (alive.current && seq === sequence.current) onError(e);
    } finally {
      if (alive.current && seq === sequence.current) setLoading(false);
    }
  }
  async function sensitivity(value: Sensitivity) {
    if (!selected || disabled || writing) return;
    setWriting(true);
    const seq = sequence.current,
      id = selected.summary.source_id;
    onChanged();
    try {
      await ipc('workspace_sensitivity_set', {
        root_id: root.root_id,
        target: {
          kind: 'source',
          source_id: id,
          locator: { start_byte: 0, end_byte: 0, heading: null },
        },
        sensitivity: value,
      });
      if (!alive.current) return;
      setSources((list) =>
        list.map((s) =>
          s.source_id === id ? { ...s, sensitivity: value } : s,
        ),
      );
      if (seq === sequence.current)
        setSelected(
          (s) => s && { ...s, summary: { ...s.summary, sensitivity: value } },
        );
    } catch (e) {
      if (alive.current) onError(e);
    } finally {
      if (alive.current) setWriting(false);
    }
  }
  return (
    <section>
      <header className="page-header">
        <h1>Sources</h1>
        <p>
          Read your notes locally and choose what may be shared. Source text is
          never sent by opening it here.
        </p>
      </header>
      <div className="source-browser">
        <div className="source-list" aria-label="Source notes">
          {sources.length === 0 && (
            <p className="empty">
              No Markdown notes found. Add notes inside your selected folder.
            </p>
          )}
          {sources.map((s) => (
            <button
              key={s.source_id}
              aria-label={s.path}
              aria-pressed={selected?.summary.source_id === s.source_id}
              disabled={writing}
              onClick={() => void open(s.source_id)}
            >
              <span>{s.path}</span>
              <span className="muted">
                {s.date || 'Undated'} ·{' '}
                {s.sensitivity === 'local_only'
                  ? 'Local-only note'
                  : s.sensitivity === 'unknown'
                    ? 'Sensitivity unknown'
                    : 'Cloud allowed'}
              </span>
            </button>
          ))}
        </div>
        <div className="source-inspector">
          {loading ? (
            <p aria-live="polite">Opening source…</p>
          ) : selected ? (
            <>
              <h2>{selected.summary.path}</h2>
              <span className="badge">
                {selected.summary.sensitivity === 'local_only'
                  ? 'Local-only'
                  : selected.summary.sensitivity === 'unknown'
                    ? 'Sensitivity unknown'
                    : 'Cloud allowed'}
              </span>
              <label className="source-sensitivity">
                Source sensitivity
                <select
                  aria-label="Source sensitivity"
                  value={selected.summary.sensitivity}
                  disabled={disabled || writing}
                  onChange={(e) =>
                    void sensitivity(e.target.value as Sensitivity)
                  }
                >
                  <option value="unknown">Unknown</option>
                  <option value="cloud_allowed">Cloud allowed</option>
                  <option value="local_only">Local-only</option>
                </select>
              </label>
              <p className="muted">
                Unknown notes need explicit consent for each run. Local-only
                notes and their derived context are always excluded.
              </p>
              <pre className="note-content">{selected.content}</pre>
            </>
          ) : (
            <p className="empty">
              Choose a source to read it and review its sensitivity.
            </p>
          )}
        </div>
      </div>
    </section>
  );
}
