import { test } from 'node:test';
import assert from 'node:assert/strict';
import { safeObservation, waitForTerminal } from './live-setup-observation.mjs';
const failed = { status: 'finished_with_gaps', batches: [{ status: 'failed', attempt: 1, error: { message: 'SECRET', details: {phase:'turn',reason:'model_terminal_failed'} } }], coverage: { complete:false, selected:1, processed:[],failed:['synthetic'],outstanding:[], model_calls:1, transmission_authorizations:1 }, graph: {items:[],issues:[]} };
test('terminal failure exits immediately and saves only safe diagnostic', async () => {
  let captured; let waits = 0;
  await assert.rejects(waitForTerminal(async () => failed, value => {captured=value}, {pause: async () => {waits++}}), /finished_with_gaps/);
  assert.equal(waits, 0); assert.deepEqual(captured.batches[0].diagnostic, {phase:'turn',reason:'model_terminal_failed'}); assert.equal(JSON.stringify(captured).includes('SECRET'), false);
});
test('unrecognized and malformed details stay explicitly unknown', () => {
  for (const details of [null,{phase:'turn',reason:'model_terminal_failed',extra:'SECRET'},{phase:'turn',reason:'scope_not_read'}]) {
    const result = safeObservation({...failed,batches:[{...failed.batches[0],error:{details}}]});
    assert.equal(result.batches[0].diagnostic, null); assert.equal(result.batches[0].diagnostic_status,'unknown');
  }
});
