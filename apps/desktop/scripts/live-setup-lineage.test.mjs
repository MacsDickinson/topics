import { test } from 'node:test';
import assert from 'node:assert/strict';
import { baseline, corrected, verifyStatementRule, snapshot, afterRestart, findClaimArticle } from './live-setup-lineage.mjs';
const selected = {topicId:'topic',logicalId:'logical',revisionId:'old',predicate:'pronouns',value:'he/him'};
const old = {id:'logical',revision_id:'old',subject_id:'topic',predicate:'pronouns',value:'he/him',supersedes:[],sources:[{kind:'passage',source:{path:'synthetic.md'}}]};
const head = {...old,revision_id:'new',value:'updated after review',supersedes:['old'],sources:[{kind:'user_statement',statement_id:'statement'}]};
const other = {...old,id:'other',revision_id:'wrong',predicate:'role'};
const review = (record_id,state,statement_id=null) => ({record_id,state,statement_id});
const view = (current_claims,history,reviews,context=[]) => ({topic:{topic:{id:'topic'},current_claims,history},reviews,context});
const base = view([old],[],[review('old','from_notes')]);
const good = view([head],[old],[review('new','confirmed_by_you','statement')],[{id:'context',text:'Synthetic context'}]);
const statement={id:'statement',topic_id:'topic',text:'Synthetic correction'};
const rule={id:'rule',topic_id:'topic',predicate:'pronouns',match_value:'he/him',replacement:'updated after review',source_revision_id:'new'};
test('baseline, exact correction and durable restart',()=>{
  baseline(base,selected);
  const lineage=corrected(good,selected,'updated after review','Synthetic correction');
  verifyStatementRule(lineage,statement,rule,selected,'updated after review');
  const expected=snapshot(good,lineage,statement,rule);
  afterRestart(good,selected,'updated after review','Synthetic correction',statement,rule,expected);
  const broken=structuredClone(good); broken.topic.current_claims[0].supersedes=[];
  assert.throws(()=>afterRestart(broken,selected,'updated after review','Synthetic correction',statement,rule,expected));
  const lostEvidence=structuredClone(good); lostEvidence.topic.history[0].sources=[];
  assert.throws(()=>afterRestart(lostEvidence,selected,'updated after review','Synthetic correction',statement,rule,expected));
  const lostContext=structuredClone(good); lostContext.context=[];
  assert.throws(()=>afterRestart(lostContext,selected,'updated after review','Synthetic correction',statement,rule,expected));
  assert.throws(()=>afterRestart(good,selected,'updated after review','Synthetic correction',statement,{...rule,match_value:'wrong'},expected));
});
test('review counterexample passes old singleton but fails exact lineage check',()=>{
  const wrong=view([other,head],[old],[review('wrong','confirmed_by_you'),review('new','from_notes')]);
  assert.equal(wrong.reviews.filter(r=>r.state==='confirmed_by_you').length,1);
  assert.throws(()=>corrected(wrong,selected,'updated after review','Synthetic correction'));
});
test('missing evidence, incorrect statement or rule are rejected',()=>{
  const bad=structuredClone(base); bad.topic.current_claims[0].sources=[];
  assert.throws(()=>baseline(bad,selected));
  const lineage=corrected(good,selected,'updated after review','Synthetic correction');
  assert.throws(()=>verifyStatementRule(lineage,{...statement,text:'other'},rule,selected,'updated after review'));
  assert.throws(()=>verifyStatementRule(lineage,statement,{...rule,source_revision_id:'wrong'},selected,'updated after review'));
});

test('asynchronous WebdriverIO-style collection finds only the sourced selected claim', async () => {
  const other = { getText: async () => 'role: he/him From your notes' };
  const claim = { getText: async () => 'pronouns: he/him From your notes' };
  const collection = { async *[Symbol.asyncIterator]() { yield other; yield claim; } };
  assert.equal(await findClaimArticle(collection, 'pronouns', 'he/him'), claim);
  assert.equal(await findClaimArticle(collection, 'missing', 'he/him'), undefined);
});
