import type { Evidence as EvidenceType, SourceRef } from './workspace-types';

export function SourceEvidence({ source }: { source: SourceRef }) {
  return (
    <div className="source-evidence">
      <p className="path">{source.path}</p>
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
export function Evidence({ sources }: { sources: EvidenceType[] }) {
  return (
    <div className="evidence">
      {sources.length === 0 && (
        <p className="muted">No cited evidence. Review carefully.</p>
      )}
      {sources.map((evidence, i) =>
        evidence.kind === 'passage' ? (
          <SourceEvidence key={i} source={evidence.source} />
        ) : (
          <p key={i} className="user-source">
            Your correction{' '}
            <span className="path">Statement {evidence.statement_id}</span>
          </p>
        ),
      )}
    </div>
  );
}
