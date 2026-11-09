import { Field, Icon, PrivacyBadge, Section } from "./ui";
import type { ScopePreview, ScopeSpec } from "./workspace-types";
export const emptySpec: ScopeSpec = {
  files: [],
  folders: [],
  date_from: null,
  date_to: null,
};
export type ScopeDraft = Omit<ScopeSpec, "files" | "folders"> & {
  files: string;
  folders: string;
};
export const emptyDraft: ScopeDraft = { ...emptySpec, files: "", folders: "" };
export const parseLines = (value: string) =>
  value
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);
export const parseScope = (draft: ScopeDraft): ScopeSpec => ({
  ...draft,
  files: parseLines(draft.files),
  folders: parseLines(draft.folders),
});
export const hasPaths = (draft: ScopeDraft) =>
  [...draft.files.split("\n"), ...draft.folders.split("\n")].some(
    (path) => path.trim().length > 0,
  );
export function ScopeFields({
  spec,
  onChange,
  disabled,
}: {
  spec: ScopeDraft;
  onChange: (spec: ScopeDraft) => void;
  disabled: boolean;
}) {
  return (
    <fieldset disabled={disabled} className="scope-fields">
      <legend>Select notes</legend>
      <p className="muted">
        Choose at least one Markdown file or folder, one path per line. Dates
        only filter the files and folders you choose. Dates include both
        endpoints; undated notes are excluded when filtering by date.
      </p>
      <div className="form-grid">
        <Field id="scope-files" label="Markdown files">
          <textarea
            rows={2}
            placeholder="Meetings/one.md"
            value={spec.files}
            onChange={(e) => onChange({ ...spec, files: e.target.value })}
          />
        </Field>
        <Field id="scope-folders" label="Folders">
          <textarea
            rows={2}
            placeholder="Meetings"
            value={spec.folders}
            onChange={(e) => onChange({ ...spec, folders: e.target.value })}
          />
        </Field>
        <Field id="scope-from" label="From date">
          <input
            type="date"
            value={spec.date_from || ""}
            onChange={(e) =>
              onChange({ ...spec, date_from: e.target.value || null })
            }
          />
        </Field>
        <Field id="scope-through" label="Through date">
          <input
            type="date"
            value={spec.date_to || ""}
            onChange={(e) =>
              onChange({ ...spec, date_to: e.target.value || null })
            }
          />
        </Field>
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
  const unknown = preview.sources.filter((s) => s.sensitivity === "unknown");
  return (
    <Section
      title="Review selected notes"
      className="scope-review"
      description={`${preview.sources.length} ${preview.sources.length === 1 ? "note" : "notes"} eligible for this run. Only this selection may be sent.`}
    >
      <div className="scope-source-list">
        {preview.sources.map((s) => (
          <div className="source-row" key={s.source_id}>
            <Icon name="file" />
            <div>
              <p className="path">{s.path}</p>
              <span className="muted">{s.date || "Undated"}</span>
            </div>
            <PrivacyBadge sensitivity={s.sensitivity} />
          </div>
        ))}
      </div>
      {preview.excluded.length > 0 && (
        <details open className="excluded-sources">
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
    </Section>
  );
}
