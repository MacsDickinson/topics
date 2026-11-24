import { useEffect, useRef, useState } from "react";
import { Badge, Button, Field, Section } from "./ui";
import { apiError, ipc } from "./ipc";
import type { TopicKnowledge } from "./generated/TopicKnowledge";
import type { AssertionEvidence } from "./generated/AssertionEvidence";
import type { Assertion } from "./generated/Assertion";
import type { PendingRef } from "./generated/PendingRef";
import type { ValidationItem } from "./generated/ValidationItem";
import type { ValidationPreview } from "./generated/ValidationPreview";
import type { ValidationAction } from "./generated/ValidationAction";
import type { SetupDecisionResult } from "./generated/SetupDecisionResult";
import type { RetrievalSettings } from "./generated/RetrievalSettings";
import type { Topic } from "./workspace-types";

function Passages({ evidence }: { evidence: AssertionEvidence[] }) {
  return (
    <>
      {evidence.map((e, i) =>
        e.kind === "statement" ? (
          <blockquote key={i}>
            {e.statement.text}
            <cite>Your judgment · {e.statement.recorded_at.slice(0, 10)}</cite>
          </blockquote>
        ) : (
          <blockquote key={i}>
            {e.source.excerpt}
            <cite>
              {e.source.path}
              {e.source.locator.heading &&
                " · " + e.source.locator.heading}{" "}
              {e.stale && <Badge tone="warning">Source changed</Badge>}
            </cite>
          </blockquote>
        ),
      )}
    </>
  );
}
function ClaimCard({ claim }: { claim: Assertion }) {
  return (
    <>
      <p className="claim-value">{claim.value}</p>
      <small>
        {claim.status === "pending"
          ? "Pending discovery"
          : claim.review === "confirmed_by_you"
          ? "Confirmed by you"
          : "Accepted · unverified"}
      </small>
      {(claim.valid_from || claim.valid_to) && (
        <p>
          Effective {claim.valid_from || "until"}{" "}
          {claim.valid_to && "through " + claim.valid_to}
        </p>
      )}
      {claim.uncertainty && <p>{claim.uncertainty}</p>}
      <Passages evidence={claim.evidence} />
    </>
  );
}

function Check({
  detail,
  item,
  rootId,
  onSaved,
  review,
}: {
  detail: TopicKnowledge;
  item: ValidationItem;
  rootId: string;
  onSaved: () => void;
  review: (ref: PendingRef) => void;
}) {
  const [preview, setPreview] = useState<ValidationPreview | null>(null);
  const [value, setValue] = useState(""),
    [busy, setBusy] = useState(false),
    [error, setError] = useState("");
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);
  const claims = detail.assertions.filter((c) =>
    item.revision_ids.includes(c.revision_id),
  );
  const relationship = detail.relationships.find(
    (r) => r.id === item.connection_id,
  );
  const pending =
    claims.find((c) => c.pending)?.pending || relationship?.pending;
  const params = {
    root_id: rootId,
    topic_id: detail.topic.id,
    item_id: item.id,
  };
  async function act(fn: () => Promise<void>) {
    setBusy(true);
    setError("");
    try {
      await fn();
    } catch (e) {
      if (mounted.current) setError(apiError(e).message);
    } finally {
      if (mounted.current) setBusy(false);
    }
  }
  function inspect(action: ValidationAction) {
    setPreview(null);
    void act(async () => {
      const p = await ipc<ValidationPreview>("validation_preview", {
        ...params,
        action,
      });
      if (mounted.current) setPreview(p);
    });
  }
  return (
    <article className="validation-item">
      <div className="check-heading">
        <Badge tone={item.deferred ? "neutral" : "warning"}>
          {item.deferred ? "Deferred · " : ""}
          {item.kind}
        </Badge>
        <h3>{item.predicate}</h3>
      </div>
      <div className="evidence-pair">
        {claims.map((c) => (
          <div key={c.revision_id}>
            <ClaimCard claim={c} />
            {!pending && !preview && (
              <Button
                disabled={busy}
                onClick={() => inspect({ kind: "confirm", id: c.revision_id })}
              >
                Confirm {c.value}
              </Button>
            )}
          </div>
        ))}
      </div>
      {relationship && (
        <>
          <p>
            {detail.topic.name} → {relationship.relation} →{" "}
            {relationship.other.name}
          </p>
          <p>{relationship.uncertainty}</p>
          <Passages evidence={relationship.evidence} />
          {!preview && !pending && (
            <Button
              disabled={busy}
              onClick={() => inspect({ kind: "confirm", id: relationship.id })}
            >
              Confirm relationship
            </Button>
          )}
        </>
      )}
      {!preview &&
        (pending ? (
          <Button onClick={() => review(pending)}>
            Review pending discovery
          </Button>
        ) : (
          claims.length > 0 && (
            <details>
              <summary>Correct the value</summary>
              <Field label="Correct value" id={"correction-" + item.id}>
                <input
                  disabled={busy}
                  value={value}
                  onChange={(e) => setValue(e.target.value)}
                />
              </Field>
              <Button
                disabled={!value.trim() || busy}
                onClick={() => inspect({ kind: "correct", value })}
              >
                Preview correction
              </Button>
            </details>
          )
        ))}
      {preview ? (
        <section className="consent" aria-label="Judgment receipt">
          <h4>Review your judgment</h4>
          {preview.statement && <p>{preview.statement.text}</p>}
          {preview.after.map((c) => (
            <p key={c.revision_id}>
              {c.predicate} ·{" "}
              {
                preview.before.find((b) => c.supersedes.includes(b.revision_id))
                  ?.value
              }{" "}
              → <strong>{c.value}</strong>
            </p>
          ))}
          {preview.confirmed.map((c) => (
            <p key={c.revision_id}>
              Confirm {c.predicate} · {c.value}
            </p>
          ))}
          {preview.receipt.connections.map((c) => (
            <p key={c.id}>
              Confirm {detail.topic.name} → {c.relation} →{" "}
              {relationship?.other.name}
            </p>
          ))}
          <div className="actions">
            <Button
              variant="primary"
              disabled={busy}
              onClick={() =>
                void act(async () => {
                  const result = await ipc<SetupDecisionResult>(
                    "setup_decide",
                    {
                      root_id: rootId,
                      receipt_id: preview.receipt.receipt_id,
                      expected_fingerprint: preview.receipt.fingerprint,
                    },
                  );
                  if (result.status !== "completed")
                    throw new Error(
                      result.error?.message ||
                        "Refresh to review the current judgment.",
                    );
                  if (mounted.current) {
                    setPreview(null);
                    onSaved();
                  }
                })
              }
            >
              Approve judgment
            </Button>
            <Button disabled={busy} onClick={() => setPreview(null)}>
              Cancel
            </Button>
          </div>
        </section>
      ) : (
        <Button
          disabled={busy}
          variant="quiet"
          onClick={() =>
            void act(async () => {
              await ipc("validation_defer", {
                ...params,
                deferred: !item.deferred,
              });
              if (mounted.current) onSaved();
            })
          }
        >
          {item.deferred ? "Bring back" : "Defer"}
        </Button>
      )}
      {error && <p role="alert">{error}</p>}
    </article>
  );
}

export function KnowledgeDetail({
  detail,
  rootId,
  onSaved,
  review,
  onSelect,
}: {
  detail: TopicKnowledge;
  rootId: string;
  onSaved: () => void;
  review: (ref: PendingRef) => void;
  onSelect: (id: string) => void;
}) {
  const pending = detail.assertions.filter((c) => c.status === "pending");
  return (
    <>
      {detail.status === "pending" && (
        <header className="topic-identity">
          <h2 tabIndex={-1}>{detail.topic.name}</h2>
          <Badge>Pending topic</Badge>
          {detail.pending && (
            <Button onClick={() => review(detail.pending!)}>
              Review discovery
            </Button>
          )}
        </header>
      )}
      {detail.validation.length > 0 && (
        <Section title="Claims to check">
          {detail.validation.map((i) => (
            <Check
              key={i.id + "-" + i.deferred}
              item={i}
              detail={detail}
              rootId={rootId}
              onSaved={onSaved}
              review={review}
            />
          ))}
        </Section>
      )}
      {detail.timelines.map((t) => (
        <details key={t.predicate}>
          <summary>{t.predicate} over time</summary>
          {t.revision_ids.map((id) => {
            const c = detail.assertions.find((c) => c.revision_id === id);
            return (
              c && (
                <div key={id}>
                  <ClaimCard claim={c} />
                </div>
              )
            );
          })}
        </details>
      ))}
      {pending.length > 0 && (
        <Section title="Pending assertions">
          {pending.map((c) => (
            <article key={c.revision_id}>
              <strong>{c.predicate}</strong>
              <ClaimCard claim={c} />
              {c.pending && (
                <Button onClick={() => review(c.pending!)}>
                  Review discovery
                </Button>
              )}
            </article>
          ))}
        </Section>
      )}
      {detail.relationships.length > 0 && (
        <div className="related-topics">
          <h3>Explore relationships</h3>
          {detail.relationships.map((r) => (
            <div key={r.id}>
              <Button variant="quiet" onClick={() => onSelect(r.other.id)}>
                {r.outgoing ? "→" : "←"} {r.relation} · {r.other.name}
              </Button>
              <small>
                {r.status}
                {r.uncertainty && " · inferred"}
                {r.review === "confirmed_by_you" && " · confirmed"}
              </small>
              {r.status === "pending" && (
                <details>
                  <summary>Relationship evidence</summary>
                  <Passages evidence={r.evidence} />
                  {r.pending && (
                    <Button onClick={() => review(r.pending!)}>
                      Review discovery
                    </Button>
                  )}
                </details>
              )}
            </div>
          ))}
        </div>
      )}
    </>
  );
}

export function RetrievalAccess({
  rootId,
  topics,
}: {
  rootId: string;
  topics: Topic[];
}) {
  const [settings, setSettings] = useState<RetrievalSettings | null>(null),
    [busy, setBusy] = useState(false),
    [error, setError] = useState("");
  useEffect(() => {
    let live = true;
    void ipc<RetrievalSettings>("retrieval_settings_get", { root_id: rootId })
      .then((s) => {
        if (live) setSettings(s);
      })
      .catch((e) => {
        if (live) setError(apiError(e).message);
      });
    return () => {
      live = false;
    };
  }, [rootId]);
  return (
    <details className="retrieval-access">
      <summary>Local MCP access</summary>
      <p>
        Allow a local client to search selected Cloud allowed topics. Claims and
        relationships require current citations whose exact source bytes you
        marked Cloud allowed.
      </p>
      <Button
        disabled={!settings || busy}
        onClick={async () => {
          setBusy(true);
          setError("");
          try {
            setSettings(
              await ipc<RetrievalSettings>("retrieval_settings_set", {
                root_id: rootId,
                enabled: !settings?.enabled,
              }),
            );
          } catch (e) {
            setError(apiError(e).message);
          } finally {
            setBusy(false);
          }
        }}
      >
        {settings?.enabled
          ? "Turn off retrieval"
          : "Enable retrieval for this folder"}
      </Button>
      {settings?.enabled && (
        <>
          <p>
            In your client’s stdio settings, use the installed{" "}
            <code>topics</code> CLI. Pass <code>mcp</code>, this folder ID and
            the topic IDs you choose as separate arguments.
          </p>
          <p>
            Folder <code>{rootId}</code>
          </p>
          {topics
            .filter((t) => t.sensitivity === "cloud_allowed")
            .map((t) => (
              <p key={t.id}>
                {t.name} <code>{t.id}</code>
              </p>
            ))}
        </>
      )}
      {error && <p role="alert">{error}</p>}
    </details>
  );
}
