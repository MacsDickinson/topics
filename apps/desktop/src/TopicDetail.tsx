import { Evidence, SourceEvidence } from './Evidence';
import { endpointLabel } from './ProposalEditor';
import type { Sensitivity, Topic, TopicView } from './workspace-types';
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
      <header className="page-header topic-header">
        <div>
          <p className="muted">
            {view.topic.topic_kind === 'person'
              ? 'Person'
              : view.topic.topic_kind === 'project'
                ? 'Project'
                : 'Thread'}
          </p>
          <h1>{view.topic.name}</h1>
          {view.topic.aliases.length > 0 && (
            <p>Also known as {view.topic.aliases.join(', ')}</p>
          )}
        </div>
        <label>
          Sensitivity
          <select
            disabled={busy}
            value={view.topic.sensitivity}
            onChange={(e) => sensitivity(e.target.value as Sensitivity)}
          >
            <option value="unknown">Unknown</option>
            <option value="cloud_allowed">Cloud allowed</option>
            <option value="local_only">Local-only</option>
          </select>
        </label>
      </header>
      {view.warnings.map((w, i) => (
        <p className="notice" key={i}>
          {w}
        </p>
      ))}
      <section>
        <h2>Current understanding</h2>
        {view.current_claims.length === 0 && (
          <p className="empty">
            No accepted claims yet. Analyse notes or ask about this topic to
            propose new understanding.
          </p>
        )}
        {view.current_claims.map((c) => (
          <article className="accepted-claim" key={c.revision_id}>
            <div className="claim-line">
              <h3>{c.predicate}</h3>
              <p>{c.value}</p>
            </div>
            {c.uncertainty && <p className="uncertainty">{c.uncertainty}</p>}
            <details><summary>Sources ({c.sources.length})</summary><Evidence sources={c.sources} rootId={rootId} /></details>
          </article>
        ))}
      </section>
      {view.connections.length > 0 && (
        <section>
          <h2>Connections</h2>
          {view.connections.map((c) => (
            <article className="accepted-claim" key={c.id}>
              <p>
                {endpointLabel(c.from, topics)} <strong>{c.relation}</strong>{' '}
                {endpointLabel(c.to, topics)}
              </p>
              {c.uncertainty && <p className="uncertainty">{c.uncertainty}</p>}
              <details><summary>Sources ({c.sources.length})</summary>{c.sources.map((s, i) => (
                <SourceEvidence source={s} key={i} />
              ))}</details>
            </article>
          ))}
        </section>
      )}
      {view.unresolved.length > 0 && (
        <section>
          <h2>Unresolved mentions</h2>
          {view.unresolved.map(
            (c, i) =>
              c.kind === 'unresolved_identity' && (
                <p key={i}>
                  “{c.name}” could refer to{' '}
                  {c.candidates
                    .map((id) => topics.find((t) => t.id === id)?.name || id)
                    .join(' or ')}
                  . More evidence is needed.
                </p>
              ),
          )}
        </section>
      )}
      <details className="revision-history">
        <summary>Revision history ({view.history.length})</summary>
        {view.history.length === 0 && <p>No earlier revisions.</p>}
        {view.history.map((c) => (
          <article key={c.revision_id} className="accepted-claim">
            <p>
              <strong>{c.predicate}</strong>: {c.value}
            </p>
            <p className="muted">
              {new Date(c.recorded_at).toLocaleString()}
              {view.current_claims.some(
                (head) => head.revision_id === c.revision_id,
              )
                ? ' · Current'
                : ' · Earlier revision'}
            </p>
            {c.uncertainty && <p className="uncertainty">{c.uncertainty}</p>}
            <details><summary>Sources ({c.sources.length})</summary><Evidence sources={c.sources} rootId={rootId} /></details>
          </article>
        ))}
      </details>
    </>
  );
}
