import type { ScopePreview, ScopeSpec } from './workspace-types';
export const emptySpec: ScopeSpec = {
  files: [],
  folders: [],
  date_from: null,
  date_to: null,
};
export function ScopeFields({
  spec,
  onChange,
  disabled,
}: {
  spec: ScopeSpec;
  onChange: (spec: ScopeSpec) => void;
  disabled: boolean;
}) {
  const lines = (value: string) =>
    value
      .split('\n')
      .map((s) => s.trim())
      .filter(Boolean);
  return (
    <fieldset disabled={disabled} className="scope-fields">
      <legend>Select notes</legend>
      <p className="muted">
        Enter paths inside this notes folder, one per line. Dates include both
        endpoints; undated notes are excluded when filtering by date.
      </p>
      <div className="form-grid">
        <label htmlFor="scope-files">
          Markdown files
          <textarea
            aria-label="Markdown files"
            id="scope-files"
            rows={2}
            placeholder="Meetings/one.md"
            value={spec.files.join('\n')}
            onChange={(e) =>
              onChange({ ...spec, files: lines(e.target.value) })
            }
          />
        </label>
        <label htmlFor="scope-folders">
          Folders
          <textarea
            aria-label="Folders"
            id="scope-folders"
            rows={2}
            placeholder="Meetings"
            value={spec.folders.join('\n')}
            onChange={(e) =>
              onChange({ ...spec, folders: lines(e.target.value) })
            }
          />
        </label>
        <label>
          From date
          <input
            type="date"
            value={spec.date_from || ''}
            onChange={(e) =>
              onChange({ ...spec, date_from: e.target.value || null })
            }
          />
        </label>
        <label>
          Through date
          <input
            type="date"
            value={spec.date_to || ''}
            onChange={(e) =>
              onChange({ ...spec, date_to: e.target.value || null })
            }
          />
        </label>
      </div>
    </fieldset>
  );
}
export function ScopeReview({
  preview,
  confirmed,
  setConfirmed,
}: {
  preview: ScopePreview;
  confirmed: boolean;
  setConfirmed: (v: boolean) => void;
}) {
  const unknown = preview.sources.filter((s) => s.sensitivity === 'unknown');
  return (
    <section className="scope-review">
      <h2>Review selected notes</h2>
      <p>
        {preview.sources.length}{' '}
        {preview.sources.length === 1 ? 'note' : 'notes'} eligible for this run.
        Only this selection may be sent.
      </p>
      {preview.sources.map((s) => (
        <div className="source-row" key={s.source_id}>
          <div>
            <p className="path">{s.path}</p>
            <span className="muted">{s.date || 'Undated'}</span>
          </div>
          <span className="badge">
            {s.sensitivity === 'unknown'
              ? 'Sensitivity unknown'
              : s.sensitivity === 'local_only'
                ? 'Local-only'
                : 'Cloud allowed'}
          </span>
        </div>
      ))}
      {preview.excluded.length > 0 && (
        <details open>
          <summary>Excluded notes ({preview.excluded.length})</summary>
          {preview.excluded.map((s, i) => (
            <div className="source-row excluded" key={i}>
              <p className="path">{s.path}</p>
              <span>{s.reason}</span>
            </div>
          ))}
        </details>
      )}
      {unknown.length > 0 && (
        <label className="check consent">
          <input
            type="checkbox"
            checked={confirmed}
            onChange={(e) => setConfirmed(e.target.checked)}
          />
          Allow these unknown-sensitivity notes for this run
        </label>
      )}
      <p className="muted">
        Local-only notes are always excluded. A new selection or changed file
        needs a new preview.
      </p>
    </section>
  );
}
