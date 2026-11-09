import {
  Badge,
  Button,
  EmptyState,
  Field,
  Icon,
  PageHeader,
  Section,
} from "./ui";
import { useEffect, useRef, useState } from "react";
import { ipc, apiError } from "./ipc";
import type { SetupInventory } from "./generated/SetupInventory";
import type { SetupManifest } from "./generated/SetupManifest";
import type { SetupJob } from "./generated/SetupJob";
import type { SetupReceipt } from "./generated/SetupReceipt";
import type { SetupDecisionResult } from "./generated/SetupDecisionResult";
import type { InventoryFolder } from "./generated/InventoryFolder";
import type { FolderRole } from "./generated/FolderRole";
import type { FolderRoleOverride } from "./generated/FolderRoleOverride";
import type { Candidate } from "./generated/Candidate";
import type { SourceRef } from "./generated/SourceRef";
import "./folder-setup.css";

const limits = {
  max_files_per_batch: 100,
  max_bytes_per_batch: 2097152,
  max_batches: 100,
  max_candidates_per_turn: 200,
  max_turns_per_batch: 1,
};
const roles: FolderRole[] = ["reference", "project", "meeting", "log", "other"];
function Evidence({ sources }: { sources: SourceRef[] }) {
  return (
    <details>
      <summary>Sources ({sources.length})</summary>
      <ul>
        {sources.map((s, i) => (
          <li key={`${s.source_id}-${i}`}>
            <strong>{s.path}</strong>
            <blockquote>{s.excerpt}</blockquote>
          </li>
        ))}
      </ul>
    </details>
  );
}
function CandidateText({ candidate }: { candidate: Candidate }) {
  switch (candidate.kind) {
    case "topic":
      return (
        <>
          {candidate.topic.name} ({candidate.topic.topic_kind})
        </>
      );
    case "claim":
    case "correction":
      return (
        <>
          {candidate.claim.predicate}: {candidate.claim.value}
        </>
      );
    case "connection":
      return (
        <>
          Relationship:{" "}
          {candidate.connection.from.kind === "topic"
            ? candidate.connection.from.id
            : candidate.connection.from.kind}{" "}
          — {candidate.connection.relation} —{" "}
          {candidate.connection.to.kind === "topic"
            ? candidate.connection.to.id
            : candidate.connection.to.kind}
        </>
      );
    case "unresolved_identity":
      return <>Identity: {candidate.name}</>;
  }
}
export function FolderSetup({
  rootId,
  runtime,
  onReturning,
  onSaved,
}: {
  rootId: string;
  runtime: string | null;
  onReturning: () => void;
  onSaved: () => void;
}) {
  const [inventory, setInventory] = useState<SetupInventory | null>(null),
    [overrides, setOverrides] = useState<FolderRoleOverride[]>([]);
  const [selected, setSelected] = useState<string[]>([]),
    [from, setFrom] = useState(""),
    [through, setThrough] = useState("");
  const [manifest, setManifest] = useState<SetupManifest | null>(null),
    [consent, setConsent] = useState(false);
  const [job, setJob] = useState<SetupJob | null>(null),
    [receipt, setReceipt] = useState<SetupReceipt | null>(null);
  const [error, setError] = useState(""),
    [busy, setBusy] = useState(false),
    [view, setView] = useState<"list" | "graph">("list"),
    [topicId, setTopicId] = useState<string | null>(null);
  const epoch = useRef(0);
  useEffect(() => {
    const generation = ++epoch.current;
    void ipc<SetupInventory>("setup_inventory", { root_id: rootId, roles: [] })
      .then((value) => {
        if (generation !== epoch.current || value.root_id !== rootId) return;
        setInventory(value);
        setSelected(
          value.documents
            .filter((d) => d.source.sensitivity !== "local_only")
            .map((d) => d.source.path),
        );
        if (value.entry === "returning") onReturning();
      })
      .catch((e) => {
        if (generation === epoch.current) setError(apiError(e).message);
      });
    return () => {
      epoch.current++;
    };
  }, [rootId]);
  useEffect(() => {
    if (!job || !["running", "ready"].includes(job.status)) return;
    const id = job.job_id,
      generation = epoch.current;
    const timer = window.setInterval(() => {
      void ipc<SetupJob>("setup_job_get", { root_id: rootId, job_id: id })
        .then((next) => {
          if (
            generation === epoch.current &&
            next.root_id === rootId &&
            next.job_id === id
          )
            setJob((old) =>
              old && next.revision >= old.revision ? next : old,
            );
        })
        .catch((e) => {
          if (generation === epoch.current) setError(apiError(e).message);
        });
    }, 1500);
    return () => window.clearInterval(timer);
  }, [rootId, job?.job_id, job?.status]);
  function invalidate() {
    setManifest(null);
    setConsent(false);
    setReceipt(null);
  }
  async function action<T>(fn: () => Promise<T>, success: (value: T) => void) {
    const generation = epoch.current;
    setBusy(true);
    setError("");
    try {
      const value = await fn();
      if (generation === epoch.current) success(value);
    } catch (e) {
      if (generation === epoch.current) {
        const problem = apiError(e);
        setError(problem.message);
        if (problem.code === "stale_scope") {
          setManifest(null);
          setConsent(false);
        }
        if (problem.code === "conflict" || problem.code === "stale_evidence")
          setReceipt(null);
      }
    } finally {
      if (generation === epoch.current) setBusy(false);
    }
  }
  function toggle(paths: string[], checked: boolean) {
    setSelected((old) =>
      checked
        ? [...new Set([...old, ...paths])]
        : old.filter((p) => !paths.includes(p)),
    );
    invalidate();
  }
  function preview() {
    void action(
      () =>
        ipc<SetupManifest>("setup_manifest_preview", {
          root_id: rootId,
          spec: {
            files: selected,
            folders: [],
            date_from: from || null,
            date_to: through || null,
          },
          roles: overrides,
          limits,
          runtime: runtime || "unavailable",
        }),
      (value) => {
        setManifest(value);
        setConsent(false);
      },
    );
  }
  function start() {
    if (!manifest || !consent) return;
    const generation = epoch.current;
    void action(async () => {
      const created = await ipc<SetupJob>("setup_job_create", {
        root_id: rootId,
        manifest_id: manifest.manifest_id,
        expected_fingerprint: manifest.fingerprint,
      });
      if (generation !== epoch.current) return created;
      setJob(created);
      return ipc<SetupJob>("setup_job_start", {
        root_id: rootId,
        job_id: created.job_id,
      });
    }, setJob);
  }
  function control(name: "pause" | "resume" | "retry_failed") {
    if (job)
      void action(
        () =>
          ipc<SetupJob>("setup_job_control", {
            root_id: rootId,
            job_id: job.job_id,
            expected_revision: job.revision,
            action: name,
          }),
        setJob,
      );
  }
  function savePreview(ids: string[] = []) {
    if (job)
      void action(
        () =>
          ipc<SetupReceipt>("setup_save_preview", {
            root_id: rootId,
            job_id: job.job_id,
            item_ids: ids,
          }),
        setReceipt,
      );
  }
  function save() {
    if (receipt)
      void action(
        () =>
          ipc<SetupDecisionResult>("setup_decide", {
            root_id: rootId,
            receipt_id: receipt.receipt_id,
            expected_fingerprint: receipt.fingerprint,
          }),
        (result) => {
          setReceipt(null);
          if (result.status !== "completed")
            setError(
              result.error?.message ||
                `Save ${result.status}; refresh the receipt.`,
            );
          onSaved();
        },
      );
  }
  const topics =
    job?.graph.items.filter((item) => item.candidate.kind === "topic") || [];
  const connections =
    job?.graph.items.filter((item) => item.candidate.kind === "connection") ||
    [];
  const active = topics.find(
    (item) =>
      item.candidate.kind === "topic" && item.candidate.topic.id === topicId,
  );
  function renderFolder(folder: InventoryFolder): React.ReactNode {
    if (!inventory) return null;
    const prefix = folder.path ? `${folder.path}/` : "";
    const documents = inventory.documents.filter(
      (d) =>
        d.source.path.startsWith(prefix) &&
        !d.source.path.slice(prefix.length).includes("/"),
    );
    const eligible = inventory.documents
      .filter(
        (d) =>
          d.source.path.startsWith(prefix) &&
          d.source.sensitivity !== "local_only",
      )
      .map((d) => d.source.path);
    const children = inventory.folders.filter(
      (child) =>
        child.path !== folder.path &&
        child.path.startsWith(prefix) &&
        !child.path.slice(prefix.length).includes("/"),
    );
    return (
      <li key={folder.path}>
        <div className="setup-folder">
          <label className="check">
            <Icon name="folder" />
            <input
              type="checkbox"
              checked={
                eligible.length > 0 &&
                eligible.every((p) => selected.includes(p))
              }
              disabled={!eligible.length || busy}
              onChange={(e) => toggle(eligible, e.target.checked)}
            />{" "}
            {folder.path || "Root"}
          </label>
          <label>
            Suggested role{" "}
            <select
              disabled={busy}
              value={
                overrides.find((r) => r.path === folder.path)?.role ||
                folder.role
              }
              onChange={(e) => {
                const next = [
                  ...overrides.filter((r) => r.path !== folder.path),
                  { path: folder.path, role: e.target.value as FolderRole },
                ];
                setOverrides(next);
                invalidate();
                void action(
                  () =>
                    ipc<SetupInventory>("setup_inventory", {
                      root_id: rootId,
                      roles: next,
                    }),
                  setInventory,
                );
              }}
            >
              {roles.map((r) => (
                <option key={r} value={r}>
                  {r}
                </option>
              ))}
            </select>
          </label>
        </div>
        <p>
          {folder.reason} {folder.documents} notes; {folder.dated} dated,{" "}
          {folder.undated} undated; {folder.linked} linked.{" "}
          {folder.date_from || "No start date"}–
          {folder.date_to || "no end date"}.
        </p>
        <details>
          <summary>Individual notes ({documents.length})</summary>
          <ul>
            {documents.map((d) => (
              <li key={d.source.source_id}>
                <label className="check">
                  <input
                    type="checkbox"
                    disabled={d.source.sensitivity === "local_only"}
                    checked={selected.includes(d.source.path)}
                    onChange={(e) => toggle([d.source.path], e.target.checked)}
                  />{" "}
                  {d.source.path}{" "}
                  {d.source.sensitivity === "local_only" && "(Local-only)"}
                </label>
              </li>
            ))}
          </ul>
        </details>
        {children.length > 0 && <ul>{children.map(renderFolder)}</ul>}
      </li>
    );
  }

  return (
    <section className="folder-setup">
      <PageHeader
        title={job ? "Starting map" : "Understand this folder"}
        description="Inventory stays on this device. Analysis starts only after you authorize the displayed selection."
      />
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
      {!inventory && !error && <p role="status">Reading folder structure…</p>}
      {inventory && !job && (
        <>
          {inventory.resumable_job_ids.map((id) => (
            <Button
              key={id}
              disabled={busy}
              onClick={() =>
                void action(
                  () =>
                    ipc<SetupJob>("setup_job_get", {
                      root_id: rootId,
                      job_id: id,
                    }),
                  setJob,
                )
              }
            >
              Resume saved setup
            </Button>
          ))}
          {inventory.entry === "empty" ? (
            <EmptyState title="No notes yet" icon="folder">
              <p>Add Markdown notes or use New topic for manual capture.</p>
            </EmptyState>
          ) : (
            <>
              <ol className="flow-steps" aria-label="Folder setup steps">
                <li aria-current={!manifest ? "step" : undefined}>
                  <span>1</span>Choose notes
                </li>
                <li aria-current={manifest ? "step" : undefined}>
                  <span>2</span>Authorize manifest
                </li>
                <li>
                  <span>3</span>Review starting map
                </li>
              </ol>
              <Section
                title="Choose notes to analyse"
                description={
                  <>
                    {inventory.documents.length} notes;{" "}
                    {inventory.eligible_bytes} eligible bytes. Suggested roles
                    do not change privacy.
                  </>
                }
              >
                <ul className="setup-tree" aria-label="Notes folders">
                  {inventory.folders
                    .filter(
                      (folder) =>
                        !inventory.folders.some(
                          (parent) =>
                            parent.path !== folder.path &&
                            (parent.path
                              ? folder.path.startsWith(`${parent.path}/`)
                              : !!folder.path),
                        ),
                    )
                    .map(renderFolder)}
                </ul>
                <div className="setup-dates">
                  <Field id="setup-from" label="From date">
                    <input
                      type="date"
                      value={from}
                      onChange={(e) => {
                        setFrom(e.target.value);
                        invalidate();
                      }}
                    />
                  </Field>
                  <Field id="setup-through" label="Through date">
                    <input
                      type="date"
                      value={through}
                      onChange={(e) => {
                        setThrough(e.target.value);
                        invalidate();
                      }}
                    />
                  </Field>
                </div>
                <h3>Excluded from discovery</h3>
                <ul>
                  {inventory.excluded.map((e, i) => (
                    <li key={i}>
                      {e.path}: {e.reason}
                    </li>
                  ))}
                </ul>
                <div className="review-footer">
                  <Button disabled={busy || !selected.length} onClick={preview}>
                    Review manifest
                  </Button>
                </div>
              </Section>
            </>
          )}
          {manifest && (
            <section className="consent">
              <h2>Review the exact manifest</h2>
              <p>
                {manifest.runtime} · {manifest.batches.length} batches · up to{" "}
                {manifest.limits.max_files_per_batch} notes and{" "}
                {manifest.limits.max_bytes_per_batch} bytes per batch.
              </p>
              <ul>
                {manifest.batches.flatMap((b) =>
                  b.sources.map((s) => (
                    <li key={s.source.source_id}>
                      {s.source.path} ({s.bytes} bytes)
                    </li>
                  )),
                )}
              </ul>
              <h3>Excluded</h3>
              <ul>
                {manifest.excluded.map((e, i) => (
                  <li key={i}>
                    {e.path}: {e.reason}
                  </li>
                ))}
              </ul>
              <label className="check">
                <input
                  type="checkbox"
                  checked={consent}
                  onChange={(e) => setConsent(e.target.checked)}
                />{" "}
                Authorize this exact manifest for this job
              </label>
              <p>This does not grant ongoing cloud permission.</p>
              <Button
                variant="primary"
                disabled={!consent || busy || !runtime}
                onClick={start}
              >
                Build starting map
              </Button>
            </section>
          )}
        </>
      )}
      {job && (
        <>
          <section className="run-panel">
            <h2>Coverage</h2>
            <p role="status">
              {job.status.replaceAll("_", " ")}: {job.coverage.processed.length}{" "}
              of {job.coverage.selected} processed,{" "}
              {job.coverage.outstanding.length} outstanding,{" "}
              {job.coverage.failed.length} failed,{" "}
              {job.coverage.excluded.length} excluded.
            </p>
            {["running", "ready"].includes(job.status) && (
              <Button disabled={busy} onClick={() => control("pause")}>
                Pause
              </Button>
            )}
            {job.status === "paused" && (
              <Button disabled={busy} onClick={() => control("resume")}>
                Resume
              </Button>
            )}
            {job.coverage.failed.length > 0 && (
              <Button disabled={busy} onClick={() => control("retry_failed")}>
                Retry failed
              </Button>
            )}
            <details>
              <summary>Coverage details</summary>
              <p>Failed: {job.coverage.failed.join(", ") || "None"}</p>
              <p>
                Outstanding: {job.coverage.outstanding.join(", ") || "None"}
              </p>
              <ul>
                {job.coverage.excluded.map((e, i) => (
                  <li key={i}>
                    {e.path}: {e.reason}
                  </li>
                ))}
              </ul>
            </details>
          </section>
          <Section
            title="Draft topics"
            description="A proposed map for your review. Saving requires a separate receipt."
          >
            <div className="actions">
              <Button
                aria-pressed={view === "list"}
                onClick={() => setView("list")}
              >
                List
              </Button>
              <Button
                aria-pressed={view === "graph"}
                onClick={() => setView("graph")}
              >
                Graph
              </Button>
            </div>
            {view === "graph" && (
              <svg
                className="setup-graph"
                role="group"
                aria-label="Draft topic map; use the topic and relationship lists to navigate"
                viewBox={`0 0 600 ${Math.max(140, topics.length * 55)}`}
              >
                {connections.map((item) => {
                  if (item.candidate.kind !== "connection") return null;
                  const c = item.candidate.connection;
                  if (c.from.kind !== "topic" || c.to.kind !== "topic")
                    return null;
                  const fromId = c.from.id,
                    toId = c.to.id;
                  const a = topics.findIndex(
                    (t) =>
                      t.candidate.kind === "topic" &&
                      t.candidate.topic.id === fromId,
                  );
                  const b = topics.findIndex(
                    (t) =>
                      t.candidate.kind === "topic" &&
                      t.candidate.topic.id === toId,
                  );
                  if (a < 0 || b < 0) return null;
                  const inferred = job.graph.issues.some(
                    (issue) =>
                      issue.issue_kind === "inferred" &&
                      issue.item_ids.includes(item.item_id),
                  );
                  const openEvidence = () => {
                    const detail = document.getElementById(
                      `edge-${item.item_id}`,
                    ) as HTMLDetailsElement | null;
                    if (detail) {
                      detail.open = true;
                      detail.scrollIntoView({ block: "nearest" });
                    }
                  };
                  return (
                    <line
                      key={item.item_id}
                      role="button"
                      tabIndex={0}
                      aria-label={`Open evidence for ${c.relation}${inferred ? ", inferred association" : ""}`}
                      onClick={openEvidence}
                      onKeyDown={(e) => {
                        if (e.key === "Enter" || e.key === " ") {
                          e.preventDefault();
                          openEvidence();
                        }
                      }}
                      className={inferred ? "inferred" : ""}
                      x1={a % 2 ? 445 : 145}
                      y1={41 + a * 55}
                      x2={b % 2 ? 445 : 145}
                      y2={41 + b * 55}
                    />
                  );
                })}
                {topics.map(
                  (item, i) =>
                    item.candidate.kind === "topic" && (
                      <g
                        key={item.item_id}
                        role="button"
                        tabIndex={0}
                        aria-label={`Open ${item.candidate.topic.name}`}
                        onClick={() =>
                          setTopicId(
                            item.candidate.kind === "topic"
                              ? item.candidate.topic.id
                              : null,
                          )
                        }
                        onKeyDown={(e) => {
                          if (e.key === "Enter" || e.key === " ") {
                            e.preventDefault();
                            if (item.candidate.kind === "topic")
                              setTopicId(item.candidate.topic.id);
                          }
                        }}
                        transform={`translate(${i % 2 ? 330 : 30}, ${20 + i * 55})`}
                      >
                        <rect width="230" height="42" rx="8" />
                        <text x="12" y="26">
                          {item.candidate.topic.name.length > 24
                            ? `${item.candidate.topic.name.slice(0, 23)}…`
                            : item.candidate.topic.name}
                        </text>
                      </g>
                    ),
                )}
              </svg>
            )}
            <ul className="draft-topic-list">
              {topics.map(
                (item) =>
                  item.candidate.kind === "topic" && (
                    <li key={item.item_id}>
                      <Button
                        onClick={() =>
                          setTopicId(
                            item.candidate.kind === "topic"
                              ? item.candidate.topic.id
                              : null,
                          )
                        }
                      >
                        {item.candidate.topic.name}
                      </Button>
                    </li>
                  ),
              )}
            </ul>
            {active && (
              <section className="draft-topic-detail">
                <h3>
                  <CandidateText candidate={active.candidate} />
                </h3>
                {job.graph.items
                  .filter(
                    (item) =>
                      (item.candidate.kind === "claim" ||
                        item.candidate.kind === "correction") &&
                      item.candidate.claim.subject_id === topicId,
                  )
                  .map((item) => {
                    if (
                      item.candidate.kind !== "claim" &&
                      item.candidate.kind !== "correction"
                    )
                      return null;
                    const claim = item.candidate.claim;
                    return (
                      <div key={item.item_id}>
                        <p>
                          {claim.predicate}: {claim.value}{" "}
                          <Badge
                            tone={
                              item.disposition === "saved"
                                ? "success"
                                : "warning"
                            }
                          >
                            {item.disposition === "saved"
                              ? "From your notes"
                              : "Draft"}
                          </Badge>
                        </p>
                        <Evidence
                          sources={claim.sources.flatMap((e) =>
                            e.kind === "passage" ? [e.source] : [],
                          )}
                        />
                      </div>
                    );
                  })}
                <Button
                  disabled={busy}
                  onClick={() =>
                    savePreview(
                      job.graph.items
                        .filter((item) => {
                          const c = item.candidate;
                          return (
                            item.item_id === active.item_id ||
                            ((c.kind === "claim" || c.kind === "correction") &&
                              c.claim.subject_id === topicId) ||
                            (c.kind === "connection" &&
                              ((c.connection.from.kind === "topic" &&
                                c.connection.from.id === topicId) ||
                                (c.connection.to.kind === "topic" &&
                                  c.connection.to.id === topicId)))
                          );
                        })
                        .map((item) => item.item_id),
                    )
                  }
                >
                  Review topic group
                </Button>
              </section>
            )}
          </Section>
          <Section title="Relationships">
            {connections.length === 0 && <p>No relationships in this draft.</p>}
            {connections.map((item) => {
              if (item.candidate.kind !== "connection") return null;
              const c = item.candidate.connection;
              const endpoint = (value: typeof c.from) =>
                value.kind === "topic"
                  ? topics.find(
                      (t) =>
                        t.candidate.kind === "topic" &&
                        t.candidate.topic.id === value.id,
                    )?.candidate
                  : null;
              const from = endpoint(c.from),
                to = endpoint(c.to);
              const inferred = job.graph.issues.some(
                (issue) =>
                  issue.issue_kind === "inferred" &&
                  issue.item_ids.includes(item.item_id),
              );
              return (
                <article key={item.item_id} className="setup-relationship">
                  <p>
                    {from ? (
                      <CandidateText candidate={from} />
                    ) : (
                      "Unresolved endpoint"
                    )}{" "}
                    — {c.relation} —{" "}
                    {to ? (
                      <CandidateText candidate={to} />
                    ) : (
                      "Unresolved endpoint"
                    )}{" "}
                    <Badge tone={inferred ? "warning" : "neutral"}>
                      {inferred
                        ? "Inferred association · Needs your input"
                        : "Relationship"}
                    </Badge>
                  </p>
                  <details id={`edge-${item.item_id}`}>
                    <summary>Sources ({c.sources.length})</summary>
                    <ul>
                      {c.sources.map((source, i) => (
                        <li key={i}>
                          <strong>{source.path}</strong>
                          <blockquote>{source.excerpt}</blockquote>
                        </li>
                      ))}
                    </ul>
                  </details>
                </article>
              );
            })}
          </Section>
          <Section title={`Needs your input (${job.graph.issues.length})`}>
            {job.graph.issues.map((issue) => (
              <article key={issue.id} className="setup-issue">
                <p>
                  {issue.key} ({issue.issue_kind}){" "}
                  {issue.deferred && "— Not now"}
                </p>
                <Evidence sources={issue.sources} />
                <Button
                  disabled={busy || issue.deferred}
                  onClick={() =>
                    void action(
                      () =>
                        ipc<SetupJob>("setup_draft_disposition", {
                          root_id: rootId,
                          job_id: job.job_id,
                          expected_revision: job.revision,
                          item_ids: issue.item_ids,
                          disposition: "deferred",
                        }),
                      setJob,
                    )
                  }
                >
                  Not now
                </Button>
              </article>
            ))}
          </Section>
          <div className="review-footer">
            <Button disabled={busy} onClick={() => savePreview()}>
              Review starting map
            </Button>
          </div>
          {receipt && (
            <section className="consent">
              <h2>Save starting map</h2>
              <p>
                {receipt.included_ids.length} items, including{" "}
                {receipt.dependency_ids.length} required dependencies;{" "}
                {receipt.deferred_ids.length} remain open.{" "}
                {receipt.operation_count} operation(s).
              </p>
              {receipt.dependency_ids.length > 0 && (
                <p>
                  Required endpoint IDs: {receipt.dependency_ids.join(", ")}
                </p>
              )}
              {receipt.deferred_ids.length > 0 && (
                <p>Deferred IDs: {receipt.deferred_ids.join(", ")}</p>
              )}
              <ul>
                {receipt.candidates.map((candidate, i) => (
                  <li key={i}>
                    <CandidateText candidate={candidate} />
                    {candidate.kind === "connection" && (
                      <Evidence sources={candidate.connection.sources} />
                    )}
                    {(candidate.kind === "claim" ||
                      candidate.kind === "correction") && (
                      <Evidence
                        sources={candidate.claim.sources.flatMap((source) =>
                          source.kind === "passage" ? [source.source] : [],
                        )}
                      />
                    )}
                  </li>
                ))}
              </ul>
              <Button variant="primary" disabled={busy} onClick={save}>
                Save starting map
              </Button>
              <Button onClick={() => setReceipt(null)}>Cancel</Button>
            </section>
          )}
        </>
      )}
    </section>
  );
}
