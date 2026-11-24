import { openTool } from "./interaction";
import { browser, $, expect } from '@wdio/globals';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
describe('Native accepted correction after process restart', () => {
  it('recovers the same topic, current correction and source history from real services', async () => {
    // Trigger a real disposable-index rebuild before reading authoritative records.
    await $('h1=Knowledge graph').waitForDisplayed();
    await openTool("Search notes");
    await $('input[type="search"]').waitForDisplayed(); await $('input[type="search"]').setValue('Alex');
    await expect($('h2=Synthetic project meeting')).toBeDisplayed();
    await $('button=Knowledge graph').click();
    await $('button[aria-label="Alex"]').click();
    await expect($('.topic-detail')).toHaveText(
      expect.stringContaining('they/them'),
    );
    await expect($('.user-source blockquote')).toHaveText(
      expect.stringContaining('Alex uses they/them pronouns.'),
    );
    await $('.revision-history summary').click();
    await expect($('.revision-history')).toHaveText(
      expect.stringContaining('he/him'),
    );
    const expected = JSON.parse(readFileSync(process.env.TOPICS_TEST_EXPECTED!, 'utf8'));
    const view = await browser.execute(async ({ topic_id }) => { const api = (window as any).__TAURI__.core; const { root } = await api.invoke('workspace_root_get', { request: {} }); return api.invoke('workspace_topic_get', { request: { root_id: root.root_id, topic_id } }); }, expected) as any;
    expect(view.current_claims.some((c: any) => c.revision_id === expected.revision_id && c.value === 'they/them')).toBe(true);
    await browser.saveScreenshot(resolve('evidence/native-live-restart.png'));
  });
});
