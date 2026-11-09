import { Evidence, SourceEvidence } from "./Evidence";
import { endpointLabel } from "./ProposalEditor";
import { Badge, EmptyState, Field, Icon, PrivacyBadge, Section } from "./ui";
import type { Sensitivity, Topic, TopicView } from "./workspace-types";

export function TopicDetail({
  view,
  rootId,
  topics,
  busy,
  sensitivity,
}: {
  view: TopicView;
  rootId: string;
  topics: Topic[];
  busy: boolean;
  sensitivity: (s: Sensitivity) => void;
}) {
  return (
    <>
      <header className="topic-identity">
        <div className="identity-heading">
          <span className="identity-icon">
            <Icon name={view.topic.topic_kind} />
          </span>
          <div>
            <p className="muted">
              {view.topic.topic_kind === "person"
                ? "Person"
                : view.topic.topic_kind === "project"
                  ? "Project"
                  : "Thread"}
            </p>
            <h2>{view.topic.name}</h2>
            {view.topic.aliases.length > 0 && (
              <p className="muted">
                Also known as {view.topic.aliases.join(", ")}
              </p>
            )}
          </div>
        </div>
        <div className="topic-privacy">
          <PrivacyBadge sensitivity={view.topic.sensitivity} />
          <Field id="topic-sensitivity" label="Sensitivity">
            <select
              disabled={busy}
              value={view.topic.sensitivity}
              onChange={(e) => sensitivity(e.target.value as Sensitivity)}
            >
              <option value="unknown">Unknown</option>
              <option value="cloud_allowed">Cloud allowed</option>
              <option value="local_only">Local-only</option>
            </select>
          </Field>
        </div>
      </header>
      {view.warnings.map((w, i) => (
        <p className="notice" key={i}>
          {w}
        </p>
      ))}
      <Section
        title="Current understanding"
        className="understanding"
        description="Accepted statements, with their evidence and remaining uncertainty."
      >
        {view.current_claims.length === 0 && (
          <EmptyState title="No accepted claims yet" icon="thread">
            <p>
              Analyse notes or ask about this topic to propose new
              understanding.
            </p>
          </EmptyState>
        )}
        {view.current_claims.map((c) => (
          <article className="accepted-claim" key={c.revision_id}>
            <h3 className="claim-property">{c.predicate}</h3>
            <p className="claim-value">{c.value}</p>
            {c.uncertainty && (
              <p className="uncertainty">
                <Icon name="info" />
                <span>
                  <strong>Uncertainty</strong> <span>{c.uncertainty}</span>
                </span>
              </p>
            )}
            <details className="claim-provenance">
              <summary>Sources ({c.sources.length})</summary>
              <Evidence sources={c.sources} rootId={rootId} />
            </details>
          </article>
        ))}
      </Section>
      {view.connections.length > 0 && (
        <Section title="Connections">
          {view.connections.map((c) => (
            <article className="relationship" key={c.id}>
              <p>
                {endpointLabel(c.from, topics)} <strong>{c.relation}</strong>{" "}
                {endpointLabel(c.to, topics)}
              </p>
              {c.uncertainty && <p className="uncertainty">{c.uncertainty}</p>}
              <details>
                <summary>Sources ({c.sources.length})</summary>
                {c.sources.map((s, i) => (
                  <SourceEvidence source={s} key={i} />
                ))}
              </details>
            </article>
          ))}
        </Section>
      )}
      {view.unresolved.length > 0 && (
        <Section title="Unresolved mentions">
          {view.unresolved.map(
            (c, i) =>
              c.kind === "unresolved_identity" && (
                <p className="notice" key={i}>
                  “{c.name}” could refer to{" "}
                  {c.candidates
                    .map((id) => topics.find((t) => t.id === id)?.name || id)
                    .join(" or ")}
                  . More evidence is needed.
                </p>
              ),
          )}
        </Section>
      )}
      <details className="revision-history">
        <summary>
          <Icon name="history" /> Revision history ({view.history.length})
        </summary>
        {view.history.length === 0 && (
          <p className="muted">No earlier revisions.</p>
        )}
        {view.history.map((c) => (
          <article key={c.revision_id} className="accepted-claim">
            <div className="revision-meta">
              <time dateTime={c.recorded_at}>
                {new Date(c.recorded_at).toLocaleString()}
              </time>
              <Badge>
                {view.current_claims.some(
                  (head) => head.revision_id === c.revision_id,
                )
                  ? "Current"
                  : "Earlier revision"}
              </Badge>
            </div>
            <p>
              <strong>{c.predicate}</strong>: {c.value}
            </p>
            {c.uncertainty && <p className="uncertainty">{c.uncertainty}</p>}
            <details>
              <summary>Sources ({c.sources.length})</summary>
              <Evidence sources={c.sources} rootId={rootId} />
            </details>
          </article>
        ))}
      </details>
    </>
  );
}
