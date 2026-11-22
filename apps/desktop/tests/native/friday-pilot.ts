import { browser, $, expect } from '@wdio/globals';

// The preceding workspace journey configured this isolated root. A fresh app
// process should open Today and support meeting feedback without a provider.
describe('Friday pilot on a returning workspace', () => {
  it('captures an empty briefing and retrieves an omission through the native service', async () => {
    await expect($('h1=Today')).toBeDisplayed();
    const result = await browser.execute(async () => {
      const { invoke } = (window as any).__TAURI__.core;
      const call = (name: string, request: Record<string, unknown>) => invoke(name, { request });
      const { root } = await call('workspace_root_get', {});
      const rootRequest = { root_id: root.root_id };
      const briefing = await call('pilot_briefing', rootRequest);
      const snapshot = await call('pilot_snapshot_create', {
        ...rootRequest,
        briefing_token: briefing.briefing_token,
      });
      const snapshotRequest = { ...rootRequest, snapshot_id: snapshot.id };
      const feedback = await call('pilot_feedback_save', {
        ...rootRequest,
        feedback: {
          id: crypto.randomUUID(),
          snapshot_id: snapshot.id,
          item_id: null,
          kind: 'omitted',
          text: 'Friday was the first time we agreed on a handover.',
          evidence: null,
          evidence_at_cutoff: 'first_learned',
          recorded_at: '2000-01-01T00:00:00Z',
          recorded_by: 'synthetic client',
        },
      });
      const entries = await call('pilot_feedback_list', snapshotRequest);
      const frozen = await call('pilot_snapshot_get', snapshotRequest);
      return {
        items: briefing.items.length,
        sameSnapshot: frozen.id === snapshot.id && frozen.briefing.briefing_token === briefing.briefing_token,
        recovered: entries.find((entry: { id: string }) => entry.id === feedback.id)?.text,
        author: feedback.recorded_by,
        serverAssignedTime: feedback.recorded_at !== '2000-01-01T00:00:00Z',
      };
    });
    expect(result.items).toEqual(0);
    expect(result.sameSnapshot).toEqual(true);
    expect(result.author).toEqual('user');
    expect(result.serverAssignedTime).toEqual(true);
    expect(result.recovered).toEqual('Friday was the first time we agreed on a handover.');
  });

  it('reports a miss directly from Today', async () => {
    // The preceding client kept a meeting briefing while Today was already open.
    await $('button=Refresh briefing').click();
    await expect($('p*=Changes since the briefing')).toBeDisplayed();
    const sentence = 'The second meeting moved to Wednesday.';
    await $('button=Topics missed something').click();
    await $('#pilot-feedback').setValue(sentence);
    await $('button=Save feedback').click();
    await expect($(`p=${sentence}`)).toBeDisplayed();
  });
});
