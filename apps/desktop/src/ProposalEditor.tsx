import { useState } from 'react';
import { Evidence, SourceEvidence } from './Evidence';
import type { Candidate, Endpoint, Proposal, Topic } from './workspace-types';

export function endpointLabel(endpoint: Endpoint, topics: Topic[]) {
  return endpoint.kind === 'topic'
    ? topics.find((t) => t.id === endpoint.id)?.name || `Topic ${endpoint.id}`
    : endpoint.kind === 'claim'
      ? `Claim ${endpoint.id}`
      : `${endpoint.source_id} (bytes ${endpoint.locator.start_byte}–${endpoint.locator.end_byte})`;
}
export function ProposalEditor({
  proposal,
  topics,
  busy,
  decide,
}: {
  proposal: Proposal;
  topics: Topic[];
  busy: boolean;
  decide: (candidates: Candidate[] | null) => Promise<void>;
}) {
  const [candidates, setCandidates] = useState<Candidate[]>(
    structuredClone(proposal.candidates),
  );
  const [included, setIncluded] = useState(
    proposal.candidates.map((c) => c.kind !== 'unresolved_identity'),
  );
  const pending = proposal.status === 'pending';
  const knownTopics = [
    ...topics,
    ...candidates.flatMap((c) => (c.kind === 'topic' ? [c.topic] : [])),
  ];
  const update = (index: number, candidate: Candidate) =>
    setCandidates((list) => list.map((c, i) => (index === i ? candidate : c)));
  return (
    <article className="proposal">
      <header className="proposal-heading">
        <div>
          <h2>
            {proposal.origin.kind === 'chat'
              ? 'Proposed correction'
              : proposal.origin.kind === 'manual'
                ? 'Your topic proposal'
                : 'Proposed understanding'}
          </h2>
          <p className="muted">
            {new Date(proposal.created_at).toLocaleString()} ·{' '}
            {proposal.candidates.length} changes
          </p>
        </div>
        <span className={`badge ${proposal.status}`}>
          {proposal.status === 'pending'
            ? 'Needs review'
            : proposal.status === 'accepted'
              ? 'Accepted'
              : proposal.status === 'rejected'
                ? 'Rejected'
                : 'Conflict'}
        </span>
      </header>
      <p className="muted">
        Review the evidence and uncertainty. Select related changes together;
        omitted changes are rejected when you approve.
      </p>
      <fieldset disabled={!pending || busy} className="candidate-set">
        <legend className="sr-only">Proposed changes</legend>
        {candidates.map((candidate, index) => (
          <section key={index} className="candidate">
            <label className="check include">
              <input
                type="checkbox"
                checked={included[index]}
                onChange={(e) =>
                  setIncluded((list) =>
                    list.map((v, i) => (index === i ? e.target.checked : v)),
                  )
                }
              />
              Include{' '}
              {candidate.kind === 'unresolved_identity'
                ? 'unresolved mention'
                : candidate.kind === 'correction'
                  ? 'correction and future rule'
                  : candidate.kind}
            </label>
            {candidate.kind === 'topic' && (
              <>
                <div className="form-grid">
                  <label>
                    Topic name
                    <input
                      aria-label="Topic name"
                      value={candidate.topic.name}
                      onChange={(e) =>
                        update(index, {
                          ...candidate,
                          topic: { ...candidate.topic, name: e.target.value },
                        })
                      }
                    />
                  </label>
                  <label>
                    Topic kind
                    <select
                      value={candidate.topic.topic_kind}
                      onChange={(e) =>
                        update(index, {
                          ...candidate,
                          topic: {
                            ...candidate.topic,
                            topic_kind: e.target.value as Topic['topic_kind'],
                          },
                        })
                      }
                    >
                      <option value="person">Person</option>
                      <option value="project">Project</option>
                      <option value="thread">Thread</option>
                    </select>
                  </label>
                </div>
                <label>
                  Aliases
                  <input
                    value={candidate.topic.aliases.join(', ')}
                    onChange={(e) =>
                      update(index, {
                        ...candidate,
                        topic: {
                          ...candidate.topic,
                          aliases: e.target.value
                            .split(',')
                            .map((s) => s.trim())
                            .filter(Boolean),
                        },
                      })
                    }
                  />
                </label>
                <label>Topic sensitivity<select value={candidate.topic.sensitivity} onChange={e => update(index, { ...candidate, topic: { ...candidate.topic, sensitivity: e.target.value as Topic['sensitivity'] } })}><option value="unknown">Unknown</option><option value="cloud_allowed">Cloud allowed</option><option value="local_only">Local-only</option></select></label>
              </>
            )}
            {(candidate.kind === 'claim' ||
              candidate.kind === 'correction') && (
              <>
                {(() => {
                  const original = proposal.candidates[index];
                  return (original.kind === 'claim' ||
                    original.kind === 'correction') &&
                    (original.claim.value !== candidate.claim.value ||
                      original.claim.predicate !== candidate.claim.predicate ||
                      original.claim.uncertainty !==
                        candidate.claim.uncertainty) ? (
                    <p className="badge">Your edit</p>
                  ) : null;
                })()}
                <p className="subject">
                  {knownTopics.find((t) => t.id === candidate.claim.subject_id)
                    ?.name || 'Topic'}{' '}
                  {candidate.kind === 'correction' && (
                    <span className="badge">
                      Supersedes earlier understanding
                    </span>
                  )}
                </p>
                {candidate.kind === 'correction' && (
                  <div className="diff-old">
                    <span className="muted">Earlier value</span>
                    <p>{candidate.rule.match_value}</p>
                  </div>
                )}
                <div className="form-grid">
                  <label>
                    Claim property
                    <input
                      value={candidate.claim.predicate}
                      onChange={(e) =>
                        update(index, {
                          ...candidate,
                          claim: {
                            ...candidate.claim,
                            predicate: e.target.value,
                          },
                          ...(candidate.kind === 'correction'
                            ? {
                                rule: {
                                  ...candidate.rule,
                                  predicate: e.target.value,
                                },
                              }
                            : {}),
                        })
                      }
                    />
                  </label>
                  <label>
                    {candidate.kind === 'correction'
                      ? 'New value'
                      : 'Claim value'}
                    <input
                      aria-label="Claim value"
                      value={candidate.claim.value}
                      onChange={(e) =>
                        update(index, {
                          ...candidate,
                          claim: { ...candidate.claim, value: e.target.value },
                          ...(candidate.kind === 'correction'
                            ? {
                                rule: {
                                  ...candidate.rule,
                                  replacement: e.target.value,
                                },
                              }
                            : {}),
                        })
                      }
                    />
                  </label>
                </div>
                <label>
                  Uncertainty
                  <textarea
                    rows={2}
                    value={candidate.claim.uncertainty || ''}
                    onChange={(e) =>
                      update(index, {
                        ...candidate,
                        claim: {
                          ...candidate.claim,
                          uncertainty: e.target.value || null,
                        },
                      })
                    }
                  />
                </label>
                {candidate.kind === 'correction' && (
                  <>
                    <label>
                      Reason and context
                      <textarea
                        value={candidate.rule.scope.context || ''}
                        onChange={(e) =>
                          update(index, {
                            ...candidate,
                            rule: {
                              ...candidate.rule,
                              scope: {
                                ...candidate.rule.scope,
                                context: e.target.value || null,
                              },
                            },
                          })
                        }
                      />
                    </label>
                    <label>
                      Apply future rule to these source paths
                      <textarea
                        placeholder="Empty means eligible sources for this topic and property"
                        value={candidate.rule.scope.source_paths.join('\n')}
                        onChange={(e) =>
                          update(index, {
                            ...candidate,
                            rule: {
                              ...candidate.rule,
                              scope: {
                                ...candidate.rule.scope,
                                source_paths: e.target.value
                                  .split('\n')
                                  .map((s) => s.trim())
                                  .filter(Boolean),
                              },
                            },
                          })
                        }
                      />
                    </label>
                    <p className="muted">
                      The rule applies only to this topic’s{' '}
                      {candidate.rule.predicate}. Future replacements still need
                      review.
                    </p>
                  </>
                )}
                <Evidence
                  sources={candidate.claim.sources}
                  rootId={proposal.root_id}
                />
              </>
            )}
            {candidate.kind === 'connection' && (
              <>
                <p>
                  {endpointLabel(candidate.connection.from, knownTopics)} →{' '}
                  {endpointLabel(candidate.connection.to, knownTopics)}
                </p>
                <label>
                  Relationship
                  <input
                    value={candidate.connection.relation}
                    onChange={(e) =>
                      update(index, {
                        ...candidate,
                        connection: {
                          ...candidate.connection,
                          relation: e.target.value,
                        },
                      })
                    }
                  />
                </label>
                <label>
                  Uncertainty
                  <textarea
                    value={candidate.connection.uncertainty || ''}
                    onChange={(e) =>
                      update(index, {
                        ...candidate,
                        connection: {
                          ...candidate.connection,
                          uncertainty: e.target.value || null,
                        },
                      })
                    }
                  />
                </label>
                {candidate.connection.sources.map((s, i) => (
                  <SourceEvidence source={s} key={i} />
                ))}
              </>
            )}
            {candidate.kind === 'unresolved_identity' && (
              <>
                <h3>Who is “{candidate.name}”?</h3>
                <p>
                  This mention is ambiguous. Keep it unresolved until the
                  evidence distinguishes a person.
                </p>
                <ul>
                  {candidate.candidates.map((id) => (
                    <li key={id}>
                      {knownTopics.find((t) => t.id === id)?.name ||
                        'Unknown person'}{' '}
                      <span className="path">{id}</span>
                    </li>
                  ))}
                </ul>
                {candidate.sources.map((s, i) => (
                  <SourceEvidence source={s} key={i} />
                ))}
              </>
            )}
          </section>
        ))}
      </fieldset>
      {pending && (
        <footer className="actions">
          <button
            className="primary"
            disabled={busy || !included.some(Boolean)}
            onClick={() =>
              void decide(candidates.filter((_, i) => included[i]))
            }
          >
            Approve selected changes
          </button>
          <button disabled={busy} onClick={() => void decide(null)}>
            Reject proposal
          </button>
        </footer>
      )}
      {!pending && (
        <p className="muted">
          This decision is already recorded. Repeated analysis shows the
          existing decision instead of creating accepted duplicates.
        </p>
      )}
    </article>
  );
}
