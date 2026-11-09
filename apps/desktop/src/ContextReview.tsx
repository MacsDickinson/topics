import { useEffect, useRef, useState } from "react";
import { ipc, apiError } from "./ipc";
import { Evidence } from "./Evidence";
import { Badge, Button } from "./ui";
import type { SetupTopicReview } from "./generated/SetupTopicReview";
import type { SetupReceipt } from "./generated/SetupReceipt";
import type { SetupCorrectionPreview } from "./generated/SetupCorrectionPreview";
import type { SetupDecisionResult } from "./generated/SetupDecisionResult";

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
  onSaved,
}: {
  rootId: string;
  topicId: string;
  onSaved: () => void;
}) {
  const [review, setReview] = useState<SetupTopicReview | null>(null),
    [error, setError] = useState("");
  const [receipt, setReceipt] = useState<SetupReceipt | null>(null),
    [correction, setCorrection] = useState<SetupCorrectionPreview | null>(null);
  const [target, setTarget] = useState<string | null>(null),
    [statement, setStatement] = useState(""),
    [replacement, setReplacement] = useState(""),
    [busy, setBusy] = useState(false);
  const [version, setVersion] = useState(0),
    [adding, setAdding] = useState(false),
    [context, setContext] = useState("");
  const generation = useRef(0);
  useEffect(() => {
    const current = ++generation.current;
    let live = true;
    setReview(null);
    setReceipt(null);
    setCorrection(null);
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
      if (generation.current === current) generation.current++;
    };
  }, [rootId, topicId, version]);
  async function act<T>(fn: () => Promise<T>, done: (value: T) => void) {
    const current = generation.current;
    setBusy(true);
    setError("");
    try {
      const value = await fn();
      if (current === generation.current) done(value);
    } catch (e) {
      if (current === generation.current) setError(apiError(e).message);
    } finally {
      if (current === generation.current) setBusy(false);
    }
  }
  const claim = review?.topic.current_claims.find(
    (c) => c.revision_id === target,
  );
  function confirm() {
    if (review)
      void act(
        () =>
          ipc<SetupReceipt>("setup_confirm_preview", {
            root_id: rootId,
            revision_ids: review.topic.current_claims.map((c) => c.revision_id),
          }),
        setReceipt,
      );
  }
  function preview() {
    if (!claim) return;
    void act(
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
      (value) => {
        setCorrection(value);
        setReceipt(value.receipt);
      },
    );
  }
  function decide() {
    if (receipt)
      void act(
        () =>
          ipc<SetupDecisionResult>("setup_decide", {
            root_id: rootId,
            receipt_id: receipt.receipt_id,
            expected_fingerprint: receipt.fingerprint,
          }),
        (result) => {
          if (result.status !== "completed") {
            setError(
              result.error?.message ||
                `Decision ${result.status}; refresh the receipt.`,
            );
            return;
          }
          setVersion((n) => n + 1);
          setTarget(null);
          setStatement("");
          setReplacement("");
          onSaved();
        },
      );
  }
  return (
    <section className="context-review">
      <h2>Review this understanding</h2>
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
      {!review && !error && <p role="status">Loading review state…</p>}
      {review && (
        <>
          {review.topic.current_claims.map((c) => {
            const state = review.reviews.find(
              (r) => r.record_id === c.revision_id,
            )?.state;
            return (
              <article key={c.revision_id}>
                <p>
                  {c.predicate}: {c.value}{" "}
                  <Badge>
                    {state
                      ? labels[state]
                      : "Accepted, not personally verified"}
                  </Badge>
                </p>
                <details>
                  <summary>Sources ({c.sources.length})</summary>
                  <Evidence sources={c.sources} rootId={rootId} />
                </details>
                <button
                  disabled={busy}
                  onClick={() => {
                    setTarget(c.revision_id);
                    setReceipt(null);
                    setCorrection(null);
                  }}
                >
                  That’s wrong
                </button>
                <button
                  disabled={busy}
                  onClick={() => {
                    setAdding(true);
                    setTarget(null);
                  }}
                >
                  Add context
                </button>
                <button onClick={() => setTarget(null)}>Not now</button>
              </article>
            );
          })}
          {review.topic.current_claims.length > 0 && (
            <button disabled={busy} onClick={confirm}>
              Confirm these {review.topic.current_claims.length} statements
            </button>
          )}
          {adding && (
            <div>
              <h3>Add context</h3>
              <p>
                Save your words as a note on this topic. This does not confirm
                or replace a statement.
              </p>
              <label>
                Your context{" "}
                <textarea
                  value={context}
                  onChange={(e) => setContext(e.target.value)}
                />
              </label>
              <button
                disabled={busy || !context.trim()}
                onClick={() =>
                  void act(
                    () =>
                      ipc("workspace_user_statement", {
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
              </button>
              <button onClick={() => setAdding(false)}>Cancel</button>
            </div>
          )}
          {claim && (
            <div>
              <h3>Correct {claim.predicate}</h3>
              <p>Before: {claim.value}</p>
              <label>
                Your statement{" "}
                <textarea
                  value={statement}
                  onChange={(e) => setStatement(e.target.value)}
                />
              </label>
              <label>
                Replacement{" "}
                <input
                  value={replacement}
                  onChange={(e) => setReplacement(e.target.value)}
                />
              </label>
              <button
                disabled={busy || !statement.trim() || !replacement.trim()}
                onClick={preview}
              >
                Preview correction
              </button>
            </div>
          )}
          {receipt && (
            <section className="consent">
              <h3>
                {correction
                  ? "Review correction"
                  : "Confirm displayed statements"}
              </h3>
              {correction && (
                <>
                  <p>Your statement: {correction.statement.text}</p>
                  <h4>Before</h4>
                  {correction.before.map((c) => (
                    <p key={c.revision_id}>
                      {c.predicate}: {c.value}
                    </p>
                  ))}
                  <h4>After</h4>
                  {correction.after.map((c) => (
                    <p key={c.revision_id}>
                      {c.predicate}: {c.value}
                    </p>
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
                    </li>
                  ))}
                </ul>
              )}
              <Button variant="primary" disabled={busy} onClick={decide}>
                {correction ? "Approve correction" : "Confirm statements"}
              </Button>
              <button
                onClick={() => {
                  setReceipt(null);
                  setCorrection(null);
                }}
              >
                Cancel
              </button>
            </section>
          )}
        </>
      )}
    </section>
  );
}
