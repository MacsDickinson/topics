import { test } from 'node:test';
import assert from 'node:assert/strict';
import { safeObservation, waitForTerminal } from './live-setup-observation.mjs';
const pairs = [
  ['preflight','version_check_failed'], ['preflight','layout_denied'], ['preflight','native_denied'],
  ['startup','setup_failed'], ['discovery','discovery_failed'], ['discovery','discovery_deadline'],
  ['transport','transport_failed'], ['transport','read_deadline'], ['transport','protocol_denied'],
  ['mcp','session_failed'], ['mcp','inventory_denied'], ['mcp','request_failed'], ['mcp','request_deadline'],
  ['callback','authorization_denied'], ['candidate','schema_denied'], ['host','validation_denied'],
  ['host','call_deadline'], ['host','host_failed'], ['execution','validation_denied'], ['execution','execution_deadline'],
  ['publication','preparation_failed'], ['publication','preparation_deadline'],
  ['turn','model_terminal_failed'], ['publication','scope_not_read'],
];
const failed = { status:'finished_with_gaps', batches:[{status:'failed',attempt:1,error:{message:'PRIVATE_ERROR_SENTINEL',details:null}}], coverage:{complete:false,selected:1,processed:[],failed:['PRIVATE_PATH_SENTINEL'],outstanding:[],model_calls:1,transmission_authorizations:1}, graph:{items:[],issues:[]} };
const observe = details => safeObservation({...failed,batches:[{...failed.batches[0],error:{message:'PRIVATE_ERROR_SENTINEL',details}}]});
test('all 24 exact pairs survive without private error or source values', () => {
  assert.equal(pairs.length, 24);
  for (const [phase, reason] of pairs) {
    const result = observe({phase,reason});
    assert.deepEqual(result.batches[0].diagnostic,{phase,reason});
    assert.equal(result.batches[0].diagnostic_status,'known');
    assert.doesNotMatch(JSON.stringify(result), /PRIVATE_(ERROR|PATH)_SENTINEL/);
  }
});
test('unknown, mismatched, malformed and extra-key details remain unknown', () => {
  for (const details of [null, undefined, 'PRIVATE_SENTINEL', [], 4, {}, {phase:'turn'}, {reason:'scope_not_read'}, {phase:'turn',reason:'scope_not_read'}, {phase:'turn',reason:'model_terminal_failed',extra:'PRIVATE_SENTINEL'}, {phase:{value:'turn'},reason:'model_terminal_failed'}, {phase:'turn',reason:['model_terminal_failed']}, {phase:'PRIVATE_SENTINEL',reason:'PRIVATE_SENTINEL'}]) {
    const result = observe(details);
    assert.equal(result.batches[0].diagnostic,null);
    assert.equal(result.batches[0].diagnostic_status,'unknown');
    assert.doesNotMatch(JSON.stringify(result), /PRIVATE_/);
  }
});
test('terminal failure exits immediately and saves only safe diagnostic', async () => {
  let captured; let waits = 0;
  const job = {...failed,batches:[{...failed.batches[0],error:{message:'PRIVATE_ERROR_SENTINEL',details:{phase:'turn',reason:'model_terminal_failed'}}}]};
  await assert.rejects(waitForTerminal(async () => job, value => {captured=value}, {pause:async () => {waits++}}), /finished_with_gaps/);
  assert.equal(waits,0);
  assert.deepEqual(captured.batches[0].diagnostic,{phase:'turn',reason:'model_terminal_failed'});
  assert.doesNotMatch(JSON.stringify(captured),/PRIVATE_/);
});
