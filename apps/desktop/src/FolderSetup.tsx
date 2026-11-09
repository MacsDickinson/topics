import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { ipc, apiError } from "./ipc";
import {
  Badge,
  Button,
  EmptyState,
  Field,
  Icon,
  PageHeader,
  Section,
} from "./ui";
import type { SetupInventory } from "./generated/SetupInventory";
import type { SetupManifest } from "./generated/SetupManifest";
import type { SetupJob } from "./generated/SetupJob";
import type { SetupReceipt } from "./generated/SetupReceipt";
import type { SetupDecisionResult } from "./generated/SetupDecisionResult";
import type { SetupTopicReview } from "./generated/SetupTopicReview";
import type { DraftItem } from "./generated/DraftItem";
import type { FolderRole } from "./generated/FolderRole";
import type { FolderRoleOverride } from "./generated/FolderRoleOverride";
import type { Candidate } from "./generated/Candidate";
import type { Endpoint } from "./generated/Endpoint";
import type { SourceRef } from "./generated/SourceRef";
import "./folder-setup.css";

const limits = {
  max_files_per_batch: 20,
  max_bytes_per_batch: 65536,
  max_batches: 100,
  max_candidates_per_turn: 64,
  max_turns_per_batch: 4,
};
function formatBytes(bytes: number): string {
  if (bytes >= 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toLocaleString(undefined, { maximumFractionDigits: 1 })} MiB`;
  if (bytes >= 1024)
    return `${(bytes / 1024).toLocaleString(undefined, { maximumFractionDigits: 1 })} KiB`;
  return `${bytes} bytes`;
}
const roles: FolderRole[] = ["reference", "project", "meeting", "log", "other"];
function batchFailureExplanation(details: unknown): string {
  if (details && typeof details === "object" && !Array.isArray(details)) {
    const record = details as Record<string, unknown>;
    const keys = Object.keys(record);
    if (
      keys.length === 2 &&
      keys.includes("phase") &&
      keys.includes("reason")
    ) {
      if (record.phase === "turn" && record.reason === "model_terminal_failed")
        return "The model reported a failed turn.";
      if (record.phase === "publication" && record.reason === "scope_not_read")
        return "The selected notes were not read, so this batch was not saved.";
    }
  }
  return "This batch could not be completed.";
}
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
function CandidateText({
  candidate,
  context = [],
}: {
  candidate: Candidate;
  context?: Candidate[];
}) {
  const endpoint = (value: Endpoint) => {
    if (value.kind === "source") return "Source passage";
    const match = context.find((item) =>
      value.kind === "topic"
        ? item.kind === "topic" && item.topic.id === value.id
        : (item.kind === "claim" || item.kind === "correction") &&
          item.claim.id === value.id,
    );
    if (!match)
      return value.kind === "topic"
        ? "Topic not available"
        : "Statement not available";
    return match.kind === "topic"
      ? match.topic.name
      : match.kind === "claim" || match.kind === "correction"
        ? `${match.claim.predicate}: ${match.claim.value}`
        : "Item not available";
  };
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
          Relationship: {endpoint(candidate.connection.from)} —{" "}
          {candidate.connection.relation} — {endpoint(candidate.connection.to)}
        </>
      );
    case "unresolved_identity":
      return <>Identity: {candidate.name}</>;
  }
}
function TopicDiff({
  rootId,
  topicId,
  incoming,
}: {
  rootId: string;
  topicId: string;
  incoming: DraftItem[];
}) {
  const [previous, setPrevious] = useState<SetupTopicReview | null>(null);
  const [error, setError] = useState("");
  useEffect(() => {
    let current = true;
    setPrevious(null);
    setError("");
    void ipc<SetupTopicReview>("setup_topic_review", {
      root_id: rootId,
      topic_id: topicId,
    })
      .then((value) => {
        if (current) setPrevious(value);
      })
      .catch((e) => {
        if (current) setError(apiError(e).message);
      });
    return () => {
      current = false;
    };
  }, [rootId, topicId]);
  return (
    <section className="topic-diff">
      <h4>Proposed change</h4>
      <p>New evidence is not saved over your existing understanding.</p>
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
      {!previous && !error && (
        <p role="status">Loading earlier understanding…</p>
      )}
      {previous && (
        <>
          <h5>Earlier understanding</h5>
          {previous.topic.current_claims
            .filter((claim) =>
              incoming.some(
                (item) =>
                  (item.candidate.kind === "claim" ||
                    item.candidate.kind === "correction") &&
                  item.candidate.claim.predicate === claim.predicate,
              ),
            )
            .map((claim) => (
              <div key={claim.revision_id}>
                <p>
                  {claim.predicate}: {claim.value}
                </p>
                <Evidence
                  sources={claim.sources.flatMap((source) =>
                    source.kind === "passage" ? [source.source] : [],
                  )}
                />
              </div>
            ))}
        </>
      )}
      <h5>New evidence</h5>
      {incoming.map((item) =>
        item.candidate.kind === "claim" ||
        item.candidate.kind === "correction" ? (
          <div key={item.item_id}>
            <p>
              {item.candidate.claim.predicate}: {item.candidate.claim.value}
            </p>
            <Evidence
              sources={item.candidate.claim.sources.flatMap((source) =>
                source.kind === "passage" ? [source.source] : [],
              )}
            />
          </div>
        ) : null,
      )}
    </section>
  );
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
    [receipt, setReceipt] = useState<SetupReceipt | null>(null),
    [receiptGroup, setReceiptGroup] = useState(false);
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
    if (!job || !runtime) return;
    const id = job.job_id,
      generation = epoch.current;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    const update = (next: SetupJob) => {
      if (
        !disposed &&
        generation === epoch.current &&
        next != null &&
        typeof next.revision === "number" &&
        Number.isSafeInteger(next.revision) &&
        next.graph != null &&
        Array.isArray(next.graph.items) &&
        Array.isArray(next.graph.issues) &&
        next.coverage != null &&
        Array.isArray(next.coverage.processed) &&
        Array.isArray(next.coverage.failed) &&
        Array.isArray(next.coverage.outstanding) &&
        Array.isArray(next.coverage.excluded) &&
        next.root_id === rootId &&
        next.job_id === id
      ) {
        setJob((old) => (old && next.revision >= old.revision ? next : old));
      }
    };
    void listen<SetupJob>("setup_job_event", (event) => update(event.payload))
      .then((remove) => {
        if (disposed) {
          remove();
          return;
        }
        unlisten = remove;
        return ipc<SetupJob>("setup_job_watch", {
          root_id: rootId,
          job_id: id,
        }).then(update);
      })
      .catch((e) => {
        if (!disposed && generation === epoch.current)
          setError(apiError(e).message);
      });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [rootId, job?.job_id, runtime]);
  function acceptJob(next: SetupJob) {
    if (next.root_id !== rootId) return;
    setJob((old) =>
      old && old.job_id === next.job_id && old.revision > next.revision
        ? old
        : next,
    );
  }
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
      setManifest(null);
      setConsent(false);
      return ipc<SetupJob>("setup_job_start", {
        root_id: rootId,
        job_id: created.job_id,
      });
    }, acceptJob);
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
        acceptJob,
      );
  }
  function previewChangedScope() {
    if (!job) return;
    void action(
      () =>
        ipc<SetupManifest>("setup_manifest_preview", {
          root_id: rootId,
          spec: job.manifest.spec,
          roles: job.manifest.roles,
          limits: job.manifest.limits,
          runtime: job.manifest.runtime,
        }),
      (updated) => {
        setManifest(updated);
        setConsent(false);
      },
    );
  }
  function reconsent() {
    if (!job || !manifest || !consent) return;
    const generation = epoch.current;
    void action(async () => {
      const renewed = await ipc<SetupJob>("setup_job_reconsent", {
        root_id: rootId,
        job_id: job.job_id,
        expected_revision: job.revision,
        manifest_id: manifest.manifest_id,
        expected_fingerprint: manifest.fingerprint,
      });
      if (generation !== epoch.current) return renewed;
      setManifest(null);
      setConsent(false);
      acceptJob(renewed);
      return ipc<SetupJob>("setup_job_start", {
        root_id: rootId,
        job_id: renewed.job_id,
      });
    }, acceptJob);
  }
  function savePreview(ids: string[] = []) {
    setReceiptGroup(ids.length > 0);
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
          if (job) {
            const generation = epoch.current;
            void ipc<SetupJob>("setup_job_get", {
              root_id: rootId,
              job_id: job.job_id,
            })
              .then((next) => {
                if (generation === epoch.current) acceptJob(next);
              })
              .catch((e) => {
                if (generation === epoch.current) setError(apiError(e).message);
              });
          }
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
  // Inventory reports folders containing notes; derive intermediate path segments
  // for navigation without scanning the filesystem or creating wire records.
  const folderPaths = new Set<string>();
  for (const folder of inventory?.folders || []) {
    if (!folder.path) {
      folderPaths.add("");
      continue;
    }
    const segments = folder.path.split("/");
    for (let i = 1; i <= segments.length; i++)
      folderPaths.add(segments.slice(0, i).join("/"));
  }
  const parentOf = (path: string) =>
    path.includes("/") ? path.slice(0, path.lastIndexOf("/")) : "";
  function renderFolder(path: string): React.ReactNode {
    if (!inventory) return null;
    const folder = inventory.folders.find((entry) => entry.path === path);
    const prefix = path ? `${path}/` : "";
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
    const children = [...folderPaths]
      .filter((child) => child !== path && parentOf(child) === path)
      .sort();
    return (
      <li key={path}>
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
            {path ? path.split("/").at(-1) : "Root"}
          </label>
          {path && (
            <label>
              Suggested role{" "}
              <select
                disabled={busy}
                value={
                  overrides.find((r) => r.path === path)?.role ||
                  folder?.role ||
                  "other"
                }
                onChange={(e) => {
                  const next = [
                    ...overrides.filter((r) => r.path !== path),
                    { path, role: e.target.value as FolderRole },
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
          )}
        </div>
        {folder ? (
          <p>
            {folder.reason} {folder.documents} notes; {folder.dated} dated,{" "}
            {folder.undated} undated; {folder.linked} linked.{" "}
            {folder.date_from || "No start date"}–
            {folder.date_to || "no end date"}.
          </p>
        ) : (
          <p>Grouping folder; roles are suggestions.</p>
        )}
        {documents.length > 0 && (
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
                      onChange={(e) =>
                        toggle([d.source.path], e.target.checked)
                      }
                    />{" "}
                    {d.source.path}{" "}
                    {d.source.sensitivity === "local_only" && "(Local-only)"}
                  </label>
                </li>
              ))}
            </ul>
          </details>
        )}
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
                  acceptJob,
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
                  {folderPaths.has("")
                    ? renderFolder("")
                    : [...folderPaths]
                        .filter((path) => !path.includes("/"))
                        .sort()
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
                {inventory.documents.some(
                  (d) =>
                    selected.includes(d.source.path) &&
                    d.bytes > limits.max_bytes_per_batch,
                ) && (
                  <div className="notice">
                    <p>
                      These selected notes exceed the{" "}
                      {formatBytes(limits.max_bytes_per_batch)} per-batch limit.
                      Exclude them before previewing.
                    </p>
                    <ul>
                      {inventory.documents
                        .filter(
                          (d) =>
                            selected.includes(d.source.path) &&
                            d.bytes > limits.max_bytes_per_batch,
                        )
                        .map((d) => (
                          <li key={d.source.source_id}>
                            {d.source.path} ({formatBytes(d.bytes)})
                          </li>
                        ))}
                    </ul>
                  </div>
                )}
                <Button
                  disabled={
                    busy ||
                    !selected.length ||
                    inventory.documents.some(
                      (d) =>
                        selected.includes(d.source.path) &&
                        d.bytes > limits.max_bytes_per_batch,
                    )
                  }
                  onClick={preview}
                >
                  Review manifest
                </Button>
              </Section>
            </>
          )}
          {manifest && (
            <section className="consent">
              <h2>Review the exact manifest</h2>
              <p>
                {manifest.runtime} · {manifest.batches.length}{" "}
                {manifest.batches.length === 1 ? "batch" : "batches"} · up to{" "}
                {manifest.limits.max_files_per_batch} notes and{" "}
                {formatBytes(manifest.limits.max_bytes_per_batch)} of notes per
                batch, with space reserved for citations and context. Up to{" "}
                {manifest.limits.max_candidates_per_turn} candidates per turn
                and {manifest.limits.max_turns_per_batch} total attempts per
                batch. Each attempt has one bounded turn.
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
              {job.coverage.complete
                ? "Selected notes analysed. Ready to save."
                : `${job.status.replaceAll("_", " ")}: selected coverage is incomplete.`}{" "}
              {job.coverage.processed.length} of {job.coverage.selected}{" "}
              selected notes processed, {job.coverage.outstanding.length}{" "}
              outstanding, {job.coverage.failed.length} failed.
            </p>
            <p>
              {job.coverage.excluded.length} excluded{" "}
              {job.coverage.excluded.length === 1 ? "source" : "sources"};
              exclusions are not part of the selected coverage.
            </p>
            {job.status === "running" && (
              <Button disabled={busy} onClick={() => control("pause")}>
                Pause
              </Button>
            )}
            {job.status === "ready" && (
              <Button
                disabled={busy || !runtime}
                onClick={() =>
                  void action(
                    () =>
                      ipc<SetupJob>("setup_job_start", {
                        root_id: rootId,
                        job_id: job.job_id,
                      }),
                    acceptJob,
                  )
                }
              >
                Resume
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
            {job.batches.some((batch) => batch.status === "failed") && (
              <ul aria-label="Batch failures">
                {job.batches
                  .filter((batch) => batch.status === "failed")
                  .map((batch) => (
                    <li key={batch.batch_id}>
                      {batchFailureExplanation(batch.error?.details)}
                    </li>
                  ))}
              </ul>
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
          {job.status === "needs_consent" && (
            <section className="consent">
              <h2>Review changed scope</h2>
              <p>
                Some unsent inputs changed. Completed work is retained; review
                the updated manifest before resuming.
              </p>
              <Button disabled={busy} onClick={previewChangedScope}>
                Preview updated manifest
              </Button>
              {manifest && (
                <>
                  <h3>Updated selection</h3>
                  <ul>
                    {manifest.batches.flatMap((batch) =>
                      batch.sources.map((entry) => {
                        const previous = job.manifest.batches
                          .flatMap((old) => old.sources)
                          .find(
                            (source) =>
                              source.source.path === entry.source.path,
                          );
                        return (
                          <li key={entry.source.source_id}>
                            {entry.source.path} —{" "}
                            {previous
                              ? previous.source.hash === entry.source.hash
                                ? "unchanged"
                                : "changed bytes"
                              : "new selection"}
                          </li>
                        );
                      }),
                    )}
                  </ul>
                  {job.manifest.batches
                    .flatMap((batch) => batch.sources)
                    .some(
                      (old) =>
                        !manifest.batches
                          .flatMap((batch) => batch.sources)
                          .some(
                            (entry) => entry.source.path === old.source.path,
                          ),
                    ) && (
                    <>
                      <h3>Removed from the updated selection</h3>
                      <ul>
                        {job.manifest.batches
                          .flatMap((batch) => batch.sources)
                          .filter(
                            (old) =>
                              !manifest.batches
                                .flatMap((batch) => batch.sources)
                                .some(
                                  (entry) =>
                                    entry.source.path === old.source.path,
                                ),
                          )
                          .map((old) => (
                            <li key={old.source.source_id}>
                              {old.source.path}
                            </li>
                          ))}
                      </ul>
                    </>
                  )}
                  <h3>Excluded</h3>
                  <ul>
                    {manifest.excluded.map((entry, i) => (
                      <li key={i}>
                        {entry.path}: {entry.reason}
                      </li>
                    ))}
                  </ul>
                  <label className="check">
                    <input
                      type="checkbox"
                      checked={consent}
                      onChange={(e) => setConsent(e.target.checked)}
                    />{" "}
                    Authorize this updated manifest for the remaining job
                  </label>
                  <Button
                    variant="primary"
                    disabled={busy || !consent || !runtime}
                    onClick={reconsent}
                  >
                    Authorize and resume
                  </Button>
                </>
              )}
            </section>
          )}
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
                  const inferred =
                    item.assertion_kind === "inferred_or_uncertain";
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
                          <Badge>
                            {item.disposition === "saved"
                              ? "From your notes"
                              : "Draft"}
                          </Badge>
                        </p>
                        {(claim.valid_from || claim.valid_to) && (
                          <p>
                            Reported period:{" "}
                            {claim.valid_from || "unspecified start"} to{" "}
                            {claim.valid_to || "unspecified end"}
                          </p>
                        )}
                        <Evidence
                          sources={claim.sources.flatMap((e) =>
                            e.kind === "passage" ? [e.source] : [],
                          )}
                        />
                      </div>
                    );
                  })}
                {(() => {
                  const itemIds = new Set(
                    job.graph.issues
                      .filter(
                        (issue) =>
                          issue.issue_kind === "replacement" ||
                          issue.issue_kind === "contradiction",
                      )
                      .flatMap((issue) => issue.item_ids),
                  );
                  const incoming = job.graph.items.filter(
                    (item) =>
                      itemIds.has(item.item_id) &&
                      (item.candidate.kind === "claim" ||
                        item.candidate.kind === "correction") &&
                      item.candidate.claim.subject_id === topicId,
                  );
                  return incoming.length > 0 && topicId ? (
                    <TopicDiff
                      rootId={rootId}
                      topicId={topicId}
                      incoming={incoming}
                    />
                  ) : null;
                })()}
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
              const inferred = item.assertion_kind === "inferred_or_uncertain";
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
                    <Badge>
                      {inferred
                        ? "Inferred association · Needs your input"
                        : item.assertion_kind === "reported_explicit"
                          ? "Explicit relationship"
                          : "Unclassified relationship"}
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
              <h2>{receiptGroup ? "Save topic group" : "Save starting map"}</h2>
              <p>
                {receipt.included_ids.length} items, including{" "}
                {receipt.dependency_ids.length} required dependencies;{" "}
                {receipt.deferred_ids.length} remain open.{" "}
                {receipt.operation_count} operation(s).
              </p>
              {receipt.dependency_ids.length > 0 && (
                <div>
                  <p>Required endpoints</p>
                  <ul>
                    {receipt.dependency_ids.map((id) => {
                      const item = job.graph.items.find(
                        (entry) => entry.item_id === id,
                      );
                      return (
                        <li key={id}>
                          {item ? (
                            <CandidateText
                              candidate={item.candidate}
                              context={job.graph.items.map(
                                (entry) => entry.candidate,
                              )}
                            />
                          ) : (
                            "Required item unavailable; refresh this receipt"
                          )}
                        </li>
                      );
                    })}
                  </ul>
                </div>
              )}
              {receipt.deferred_ids.length > 0 && (
                <div>
                  <p>Deferred items</p>
                  <ul>
                    {receipt.deferred_ids.map((id) => {
                      const item = job.graph.items.find(
                        (entry) => entry.item_id === id,
                      );
                      return (
                        <li key={id}>
                          {item ? (
                            <CandidateText
                              candidate={item.candidate}
                              context={job.graph.items.map(
                                (entry) => entry.candidate,
                              )}
                            />
                          ) : (
                            "Deferred item unavailable; refresh this receipt"
                          )}
                        </li>
                      );
                    })}
                  </ul>
                </div>
              )}
              <ul>
                {receipt.candidates.map((candidate, i) => (
                  <li key={i}>
                    <CandidateText
                      candidate={candidate}
                      context={[
                        ...job.graph.items.map((item) => item.candidate),
                        ...receipt.candidates,
                      ]}
                    />
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
                {receiptGroup ? "Save topic group" : "Save starting map"}
              </Button>
              <Button onClick={() => setReceipt(null)}>Cancel</Button>
            </section>
          )}
        </>
      )}
    </section>
  );
}
