import { useEffect, useRef, useState } from "react";
import { ipc } from "./ipc";
import {
  EmptyState,
  Field,
  Icon,
  PageHeader,
  PrivacyBadge,
  privacyLabel,
} from "./ui";
import type {
  RootRef,
  ScopedSource,
  Sensitivity,
  SourceSummary,
} from "./workspace-types";
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
    [writing, setWriting] = useState(false),
    [inventory, setInventory] = useState<"loading" | "ready" | "error">(
      "loading",
    );
  const alive = useRef(true),
    sequence = useRef(0);
  useEffect(() => {
    alive.current = true;
    void ipc<{ sources: SourceSummary[] }>("workspace_sources_list", {
      root_id: root.root_id,
    })
      .then((r) => {
        if (alive.current) {
          setSources(r.sources);
          setInventory("ready");
        }
      })
      .catch((e) => {
        if (alive.current) {
          setInventory("error");
          onError(e);
        }
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
      const result = await ipc<ScopedSource>("workspace_source_get", {
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
      await ipc("workspace_sensitivity_set", {
        root_id: root.root_id,
        target: {
          kind: "source",
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
    <section className="sources-view">
      <PageHeader
        title="Sources"
        description="Your local Markdown library. Opening a note here never sends its text."
      />
      <div className="source-browser">
        <div className="source-list" aria-label="Source notes">
          <div className="list-heading">
            <h2>Notes</h2>
            {inventory === "ready" && (
              <span className="muted">{sources.length}</span>
            )}
          </div>
          {inventory === "loading" && (
            <p role="status">Loading source notes…</p>
          )}
          {inventory === "error" && (
            <EmptyState title="Notes could not be loaded" icon="folder">
              <p>
                Review the workspace error, then reopen Sources to try again.
              </p>
            </EmptyState>
          )}
          {inventory === "ready" && sources.length === 0 && (
            <EmptyState title="No Markdown notes found" icon="file">
              <p>Add notes inside your selected folder.</p>
            </EmptyState>
          )}
          {sources.map((s) => (
            <button
              className="library-row"
              key={s.source_id}
              aria-label={s.path}
              aria-pressed={selected?.summary.source_id === s.source_id}
              disabled={writing}
              onClick={() => void open(s.source_id)}
            >
              <Icon name="file" />
              <span className="row-content">
                <strong>{s.path.split("/").at(-1)}</strong>
                <span className="path">{s.path}</span>
                <span className="row-meta">
                  {s.date || "Undated"}
                  <span>{privacyLabel(s.sensitivity)}</span>
                </span>
              </span>
              {selected?.summary.source_id === s.source_id && (
                <Icon name="chevron" />
              )}
            </button>
          ))}
        </div>
        <div className="source-inspector" aria-busy={loading}>
          {loading ? (
            <p role="status">Opening source…</p>
          ) : selected ? (
            <>
              <header className="source-identity">
                <div className="identity-heading">
                  <span className="identity-icon">
                    <Icon name="file" />
                  </span>
                  <div>
                    <h2>{selected.summary.path.split("/").at(-1)}</h2>
                    <p className="path">{selected.summary.path}</p>
                  </div>
                </div>
                <PrivacyBadge sensitivity={selected.summary.sensitivity} />
              </header>
              <section
                className="source-policy"
                aria-label="Privacy and source location"
              >
                <p className="path">
                  {root.path}/{selected.summary.path}
                </p>
                <Field
                  id="source-sensitivity"
                  label="Source sensitivity"
                  hint="Unknown notes need explicit consent for each run. Local-only notes and their derived context are always excluded."
                >
                  <select
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
                </Field>
              </section>
              <pre className="note-content">{selected.content}</pre>
            </>
          ) : (
            <EmptyState title="Choose a source" icon="file">
              <p>
                Read the original note and review its sensitivity. Your notes
                stay on this device.
              </p>
            </EmptyState>
          )}
        </div>
      </div>
    </section>
  );
}
