import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import { join, isAbsolute } from 'node:path';

const confirmed = p => p.reviews.filter(r => r.state === 'confirmed_by_you').map(r => r.record_id).sort();
const ordered = xs => [...xs].sort((a,b) => JSON.stringify(a).localeCompare(JSON.stringify(b)));
export function baseline(p, selected) {
  assert.equal(p.topic.topic.id, selected.topicId);
  const old = p.topic.current_claims.find(c => c.id === selected.logicalId && c.revision_id === selected.revisionId);
  assert.ok(old, 'selected original revision must be current');
  assert.equal(old.subject_id, selected.topicId);
  assert.equal(old.predicate, selected.predicate);
  assert.equal(old.value, selected.value);
  assert.ok(old.sources.some(s => s.kind === 'passage'), 'baseline must retain passage evidence');
  assert.equal(p.reviews.find(r => r.record_id === old.revision_id)?.state, 'from_notes');
  assert.deepEqual(confirmed(p), [], 'baseline must not confirm any claim');
  return old;
}
export function corrected(p, selected, replacement, statementText) {
  assert.equal(p.topic.topic.id, selected.topicId);
  const matches = p.topic.current_claims.filter(c => c.id === selected.logicalId && c.subject_id === selected.topicId && c.predicate === selected.predicate && c.value === replacement && c.supersedes.includes(selected.revisionId));
  assert.equal(matches.length, 1, 'one exact replacement must supersede the selected original');
  const head = matches[0];
  const refs = head.sources.filter(s => s.kind === 'user_statement');
  assert.equal(refs.length, 1, 'replacement must cite exactly one user statement');
  const review = p.reviews.find(r => r.record_id === head.revision_id);
  assert.equal(review?.state, 'confirmed_by_you');
  assert.equal(review?.statement_id, refs[0].statement_id);
  assert.deepEqual(confirmed(p), [head.revision_id], 'only the replacement head is confirmed');
  const old = p.topic.history.find(c => c.id === selected.logicalId && c.revision_id === selected.revisionId);
  assert.ok(old, 'original revision must remain in history');
  assert.equal(old.predicate, selected.predicate);
  assert.equal(old.value, selected.value);
  assert.ok(old.sources.some(s => s.kind === 'passage'));
  return { head, old, statementId: refs[0].statement_id, statementText };
}
export function verifyStatementRule(lineage, statement, rule, selected, replacement) {
  assert.equal(statement.id, lineage.statementId);
  assert.equal(statement.topic_id, selected.topicId);
  assert.equal(statement.text, lineage.statementText);
  assert.equal(rule.topic_id, selected.topicId);
  assert.equal(rule.predicate, selected.predicate);
  assert.equal(rule.match_value, selected.value);
  assert.equal(rule.replacement, replacement);
  assert.equal(rule.source_revision_id, lineage.head.revision_id);
}
export function snapshot(p, lineage, statement, rule) {
  return { topicId: p.topic.topic.id, head: lineage.head, old: lineage.old, reviews: ordered(p.reviews), current: ordered(p.topic.current_claims), history: ordered(p.topic.history), context: ordered(p.context), statement, rule };
}
export function afterRestart(p, selected, replacement, statementText, statement, rule, expected) {
  const lineage = corrected(p, selected, replacement, statementText);
  verifyStatementRule(lineage, statement, rule, selected, replacement);
  assert.deepEqual(snapshot(p, lineage, statement, rule), expected, 'claim heads, lineage, evidence, rule, context and review metadata must survive rebuild');
}

// Reads only app-owned rule records under the absolute synthetic fixture root.
export function readRule(root, headId) {
  assert.ok(isAbsolute(root));
  const folder = join(root, 'topics', 'rules');
  const rules = readdirSync(folder, { withFileTypes: true }).filter(entry => entry.isFile() && entry.name.endsWith('.md')).map(entry => {
    const data = readFileSync(join(folder, entry.name), 'utf8').split('\n').find(line => line.startsWith('data: '));
    assert.ok(data, 'rule record lacks data');
    return JSON.parse(data.slice(6));
  }).filter(rule => rule.source_revision_id === headId);
  assert.equal(rules.length, 1, 'exactly one persisted rule must refer to the replacement');
  return rules[0];
}
