import { Badge, Button, Field, Section } from "./ui";
import { useEffect, useRef, useState } from "react";
import { ipc, apiError } from "./ipc";
import { Evidence } from "./Evidence";
import { endpointLabel } from "./ProposalEditor";
import type { Topic } from "./workspace-types";
import type { SetupTopicReview } from "./generated/SetupTopicReview";
import type { SetupReceipt } from "./generated/SetupReceipt";
import type { SetupCorrectionPreview } from "./generated/SetupCorrectionPreview";
import type { SetupDecisionResult } from "./generated/SetupDecisionResult";

type Approval =
  | { kind: "correction"; preview: SetupCorrectionPreview }
  | { kind: "statements"; receipt: SetupReceipt }
  | { kind: "connection"; receipt: SetupReceipt };

const labels = {
  from_notes: "From your notes",
  confirmed_by_you: "Confirmed by you",
  needs_input: "Needs your input",
  earlier_understanding: "Earlier understanding",
  accepted_unverified: "Accepted, not personally verified",
};
export function ContextReview({
  rootId,
  topicId,
  topics,
  onSaved,
  initialCorrection,
}: {
  rootId: string;
  topicId: string;
  topics: Topic[];
  onSaved: () => void;
  initialCorrection?: {revisionId: string; statement: string} | null;
}) {
  const [review, setReview] = useState<SetupTopicReview | null>(null),
    [error, setError] = useState("");
  const [approval, setApproval] = useState<Approval | null>(null);
  const receipt =
    approval?.kind === "correction"
      ? approval.preview.receipt
      : approval?.receipt;
  const correction = approval?.kind === "correction" ? approval.preview : null;
  const [target, setTarget] = useState<string | null>(null),
    [statement, setStatement] = useState(""),
    [replacement, setReplacement] = useState(""),
    [busy, setBusy] = useState(false),
    [saving, setSaving] = useState(false);
  const [version, setVersion] = useState(0),
    [adding, setAdding] = useState(false),
    [context, setContext] = useState("");
  const generation = useRef(0),
    requestSequence = useRef(0),
    mutation = useRef<number | null>(null);
  useEffect(() => {
    const current = ++generation.current;
    let live = true;
    setReview(null);
    ++requestSequence.current;
    mutation.current = null;
    setApproval(null);
    setBusy(false);
    setSaving(false);
    setError("");
    void ipc<SetupTopicReview>("setup_topic_review", {
      root_id: rootId,
      topic_id: topicId,
    })
      .then((value) => {
        if (live) setReview(value);
      })
      .catch((e) => {
        if (live) setError(apiError(e).message);
      });
    return () => {
      live = false;
      ++requestSequence.current;
      if (generation.current === current) generation.current++;
    };
  }, [rootId, topicId, version]);
  // Preview intent is cancelable; an already submitted human mutation is not.
  function invalidateApproval() {
    if (mutation.current !== null) return;
    ++requestSequence.current;
    setApproval(null);
    setBusy(false);
  }
  async function act<T>(
    fn: () => Promise<T>,
    done: (value: T) => void,
    mode: "preview" | "mutation" = "mutation",
  ) {
    if (mutation.current !== null) return;
    const current = generation.current,
      sequence = ++requestSequence.current;
    const active = () =>
      current === generation.current && sequence === requestSequence.current;
    if (mode === "mutation") mutation.current = sequence;
    setBusy(true);
    setSaving(mode === "mutation");
    setError("");
    try {
      const value = await fn();
      if (active()) done(value);
    } catch (e) {
      if (active()) {
        setError(apiError(e).message);
        setApproval(null);
      }
    } finally {
      if (mutation.current === sequence) mutation.current = null;
      if (active()) {
        setBusy(false);
        setSaving(false);
      }
    }
  }
  function previewApproval<T>(
    fn: () => Promise<T>,
    toApproval: (value: T) => Approval,
  ) {
    if (mutation.current !== null) return;
    invalidateApproval();
    void act(fn, (value) => setApproval(toApproval(value)), "preview");
  }
  useEffect(() => {
    if (initialCorrection && review?.topic.current_claims.some(c => c.revision_id === initialCorrection.revisionId)) {
      setTarget(initialCorrection.revisionId);
      setStatement(initialCorrection.statement);
    }
  }, [initialCorrection, review]);
  const claim = review?.topic.current_claims.find(
    (c) => c.revision_id === target,
  );
  function confirm() {
    if (mutation.current !== null) return;
    setTarget(null);
    setAdding(false);
    if (review)
      previewApproval(
        () =>
          ipc<SetupReceipt>("setup_confirm_preview", {
            root_id: rootId,
            revision_ids: review.topic.current_claims.map((c) => c.revision_id),
          }),
        (receipt) => ({ kind: "statements", receipt }),
      );
  }
  function preview() {
    if (!claim) return;
    previewApproval(
      () =>
        ipc<SetupCorrectionPreview>("setup_correction_preview", {
          root_id: rootId,
          topic_id: topicId,
          revision_ids: [claim.revision_id],
          predicate: claim.predicate,
          replacement,
          statement,
          rule_scope: {
            source_paths: claim.sources.flatMap((s) =>
              s.kind === "passage" ? [s.source.path] : [],
            ),
            context: null,
          },
        }),
      (preview) => ({ kind: "correction", preview }),
    );
  }
  function decide() {
    if (receipt && !busy)
      void act(
        () =>
          ipc<SetupDecisionResult>("setup_decide", {
            root_id: rootId,
            receipt_id: receipt.receipt_id,
            expected_fingerprint: receipt.fingerprint,
          }),
        (result) => {
          if (result.status !== "completed") {
            setApproval(null);
            setError(
              result.error?.message ||
                `Decision ${result.status}; refresh the receipt.`,
            );
            return;
          }
          setApproval(null);
          setVersion((n) => n + 1);
          setTarget(null);
          setStatement("");
          setReplacement("");
          onSaved();
        },
      );
  }
  return (
    <Section
      className="context-review"
      title="Review this understanding"
      description="Confirm what you know, correct a statement, or add your own context."
    >
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
      {!review && !error && <p role="status">Loading review state…</p>}
      {review && initialCorrection && !review.topic.current_claims.some(c => c.revision_id === initialCorrection.revisionId) && <p role="alert" className="notice">The original statement is no longer current. Your feedback is saved with the briefing; review the topic history before making a new correction.</p>}
      {busy && (
        <p role="status">
          {saving ? "Saving your change…" : "Preparing review…"}
        </p>
      )}
      {review && (
        <>
          {review.context.length > 0 && (
            <section>
              <h3>Your context</h3>
              {review.context.map((entry) => (
                <blockquote key={entry.id}>{entry.text}</blockquote>
              ))}
            </section>
          )}
          {review.topic.connections.length > 0 && (
            <section>
              <h3>Connections</h3>
              {review.topic.connections.map((connection) => (
                <article key={connection.id}>
                  <p>
                    {endpointLabel(connection.from, topics)} —{" "}
                    {connection.relation} —{" "}
                    {endpointLabel(connection.to, topics)}
                  </p>
                  <details>
                    <summary>Sources ({connection.sources.length})</summary>
                    {connection.sources.map((source, i) => (
                      <blockquote key={i}>
                        <strong>{source.path}</strong>
                        <p>{source.excerpt}</p>
                      </blockquote>
                    ))}
                  </details>
                  <Button
                    disabled={busy}
                    onClick={() => {
                      setTarget(null);
                      setAdding(false);
                      previewApproval(
                        () =>
                          ipc<SetupReceipt>(
                            "setup_connection_confirm_preview",
                            {
                              root_id: rootId,
                              connection_ids: [connection.id],
                            },
                          ),
                        (receipt) => ({ kind: "connection", receipt }),
                      );
                    }}
                  >
                    Confirm this connection
                  </Button>
                </article>
              ))}
            </section>
          )}
          {review.topic.current_claims.map((c) => {
            const state = review.reviews.find(
              (r) => r.record_id === c.revision_id,
            )?.state;
            return (
              <article key={c.revision_id}>
                <p>
                  {c.predicate}: {c.value}{" "}
                  <Badge
                    tone={
                      state === "confirmed_by_you"
                        ? "success"
                        : state === "needs_input"
                          ? "warning"
                          : "neutral"
                    }
                  >
                    {state
                      ? labels[state]
                      : "Accepted, not personally verified"}
                  </Badge>
                </p>
                {(c.valid_from || c.valid_to) && (
                  <p>
                    Reported period: {c.valid_from || "unspecified start"} to{" "}
                    {c.valid_to || "unspecified end"}
                  </p>
                )}
                <details>
                  <summary>Sources ({c.sources.length})</summary>
                  <Evidence sources={c.sources} rootId={rootId} />
                </details>
                <div className="actions">
                  <Button
                    disabled={busy}
                    onClick={() => {
                      invalidateApproval();
                      setAdding(false);
                      setTarget(c.revision_id);
                    }}
                  >
                    That’s wrong
                  </Button>
                  <Button
                    disabled={busy}
                    onClick={() => {
                      invalidateApproval();
                      setAdding(true);
                      setTarget(null);
                    }}
                  >
                    Add context
                  </Button>
                  <Button
                    disabled={saving}
                    onClick={() => {
                      invalidateApproval();
                      setTarget(null);
                      setAdding(false);
                    }}
                  >
                    Not now
                  </Button>
                </div>
              </article>
            );
          })}
          {review.topic.current_claims.length > 0 && (
            <Button disabled={busy} onClick={confirm}>
              Confirm these {review.topic.current_claims.length} statements
            </Button>
          )}
          {adding && (
            <div className="context-composer">
              <h3>Add context</h3>
              <p>
                Save your words as a note on this topic. This does not confirm
                or replace a statement.
              </p>
              <Field id="review-context" label="Your context">
                <textarea
                  autoFocus
                  disabled={saving}
                  value={context}
                  onChange={(e) => setContext(e.target.value)}
                />
              </Field>
              <Button
                disabled={busy || !context.trim()}
                onClick={() =>
                  void act(
                    () =>
                      ipc("setup_context_add", {
                        root_id: rootId,
                        topic_id: topicId,
                        text: context,
                      }),
                    () => {
                      setAdding(false);
                      setContext("");
                      setVersion((n) => n + 1);
                    },
                  )
                }
              >
                Save context
              </Button>
              <Button
                disabled={saving}
                onClick={() => {
                  invalidateApproval();
                  setAdding(false);
                }}
              >
                Cancel
              </Button>
            </div>
          )}
          {claim && (
            <div className="context-composer">
              <h3>Correct {claim.predicate}</h3>
              <p>Before: {claim.value}</p>
              <Field id="review-statement" label="Your statement">
                <textarea
                  autoFocus
                  disabled={saving}
                  value={statement}
                  onChange={(e) => {
                    invalidateApproval();
                    setStatement(e.target.value);
                  }}
                />
              </Field>
              <Field id="review-replacement" label="Replacement">
                <input
                  disabled={saving}
                  value={replacement}
                  onChange={(e) => {
                    invalidateApproval();
                    setReplacement(e.target.value);
                  }}
                />
              </Field>
              <Button
                disabled={busy || !statement.trim() || !replacement.trim()}
                onClick={preview}
              >
                Preview correction
              </Button>
              <Button
                disabled={saving}
                onClick={() => {
                  invalidateApproval();
                  setTarget(null);
                }}
              >
                Cancel correction
              </Button>
            </div>
          )}
          {receipt && (
            <section className="consent">
              <h3>
                {correction
                  ? "Review correction"
                  : approval?.kind === "connection"
                    ? "Confirm displayed connections"
                    : "Confirm displayed statements"}
              </h3>
              {correction && (
                <>
                  <p>Your statement: {correction.statement.text}</p>
                  <h4>Before</h4>
                  {correction.before.map((c) => (
                    <article key={c.revision_id}>
                      <p>
                        {c.predicate}: {c.value}
                      </p>
                      <details>
                        <summary>Sources ({c.sources.length})</summary>
                        <Evidence sources={c.sources} rootId={rootId} />
                      </details>
                    </article>
                  ))}
                  <h4>After</h4>
                  {correction.after.map((c) => (
                    <article key={c.revision_id}>
                      <p>
                        {c.predicate}: {c.value}
                      </p>
                      <details>
                        <summary>Sources ({c.sources.length})</summary>
                        <Evidence sources={c.sources} rootId={rootId} />
                      </details>
                    </article>
                  ))}
                  <p>
                    Future rule: {correction.rule.match_value} →{" "}
                    {correction.rule.replacement}; scope:{" "}
                    {correction.rule.scope.source_paths.join(", ") ||
                      "No source paths"}
                    ; {correction.rule.scope.context || "no additional context"}
                    .
                  </p>
                </>
              )}
              {!correction && (
                <ul>
                  {receipt.revisions.map((c) => (
                    <li key={c.revision_id}>
                      {c.predicate}: {c.value}
                      <details>
                        <summary>Sources ({c.sources.length})</summary>
                        <Evidence sources={c.sources} rootId={rootId} />
                      </details>
                    </li>
                  ))}
                  {receipt.connections.map((connection) => (
                    <li key={connection.id}>
                      {endpointLabel(connection.from, topics)} —{" "}
                      {connection.relation} —{" "}
                      {endpointLabel(connection.to, topics)}
                      <details>
                        <summary>Sources ({connection.sources.length})</summary>
                        {connection.sources.map((source, i) => (
                          <blockquote key={i}>
                            <strong>{source.path}</strong>
                            <p>{source.excerpt}</p>
                          </blockquote>
                        ))}
                      </details>
                    </li>
                  ))}
                </ul>
              )}
              <Button variant="primary" disabled={busy} onClick={decide}>
                {correction
                  ? "Approve correction"
                  : approval?.kind === "connection"
                    ? "Confirm connection"
                    : "Confirm statements"}
              </Button>
              <Button disabled={saving} onClick={invalidateApproval}>
                Cancel
              </Button>
            </section>
          )}
        </>
      )}
    </Section>
  );
}
