import { useEffect, useState } from 'react';
import { ipc, apiError } from './ipc';
import type {
  Evidence as EvidenceType,
  SourceRef,
  UserStatement,
} from './workspace-types';

export function SourceEvidence({ source }: { source: SourceRef }) {
  return (
    <div className="source-evidence">
      <p className="path">{source.path}</p>
      <p className="muted">Original note passage</p>
      <blockquote>{source.excerpt}</blockquote>
      <p className="muted">
        {source.locator.heading && `${source.locator.heading} · `}Bytes{' '}
        {source.locator.start_byte}–{source.locator.end_byte}
      </p>
      <details>
        <summary>Source identity</summary>
        <p className="path">Document: {source.document_hash}</p>
        <p className="path">Passage: {source.passage_hash}</p>
      </details>
    </div>
  );
}
function StatementEvidence({ rootId, id }: { rootId: string; id: string }) {
  const [statement, setStatement] = useState<UserStatement | null>(null),
    [error, setError] = useState('');
  useEffect(() => {
    let active = true;
    setStatement(null);
    setError('');
    void ipc<UserStatement>('workspace_statement_get', {
      root_id: rootId,
      statement_id: id,
    })
      .then((result) => {
        if (active) setStatement(result);
      })
      .catch((e) => {
        if (active) setError(apiError(e).message);
      });
    return () => {
      active = false;
    };
  }, [rootId, id]);
  return (
    <div className="source-evidence user-source">
      <p>Your correction</p>
      {statement ? (
        <>
          <blockquote>{statement.text}</blockquote>
          <p className="muted">
            {new Date(statement.recorded_at).toLocaleString()}
          </p>
        </>
      ) : (
        <p className="muted">{error || 'Loading the recorded statement…'}</p>
      )}
      <details>
        <summary>Statement identity</summary>
        <p className="path">{id}</p>
      </details>
    </div>
  );
}
export function Evidence({
  sources,
  rootId,
}: {
  sources: EvidenceType[];
  rootId: string;
}) {
  return (
    <div className="evidence">
      {sources.length === 0 && (
        <p className="muted">No cited evidence. Review carefully.</p>
      )}
      {sources.map((evidence, i) =>
        evidence.kind === 'passage' ? (
          <SourceEvidence key={i} source={evidence.source} />
        ) : (
          <StatementEvidence
            key={`${rootId}-${evidence.statement_id}`}
            rootId={rootId}
            id={evidence.statement_id}
          />
        ),
      )}
    </div>
  );
}
