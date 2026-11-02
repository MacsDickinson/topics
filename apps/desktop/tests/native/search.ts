import { browser, $, $$, expect } from '@wdio/globals';
import { mkdirSync } from 'node:fs';
import { resolve } from 'node:path';

describe('Phase 0 real macOS app, real Rust store', () => {
  it('searches synthetic notes through IPC and shows No matches', async () => {
    const search = await $('input[type="search"]');
    await search.waitForDisplayed();
    await browser.keys(['Meta', 'k']);
    await expect(search).toBeFocused();
    await search.setValue('ledger');
    await expect($('h2')).toHaveText('Ledger migration');
    await expect($('.path')).toHaveText('memory/projects/ledger-migration.md');
    await expect($('li')).toHaveText(expect.stringContaining('ledger migration is ready for review'));
    mkdirSync(resolve('evidence'), { recursive: true });
    await browser.saveScreenshot(resolve('evidence/native-ledger.png'));
    await search.setValue('zzzz-no-match');
    await expect($('[role="status"]')).toHaveText('No matches');
    await expect($$('li')).toBeElementsArrayOfSize(0);
    await browser.saveScreenshot(resolve('evidence/native-no-matches.png'));
  });
});

// A real CLI child switches the same on-disk config used by this retained app.
// Re-query across several watcher ticks so an old watcher cannot pass by chance.
describe('running app and independent CLI coherence', () => {
  it('keeps the new root selected across watcher ticks and external edits', async () => {
    const { execFileSync } = await import('node:child_process');
    const { readFileSync, writeFileSync } = await import('node:fs');
    const cli = process.env.TOPICS_TEST_CLI!;
    const newRoot = process.env.TOPICS_TEST_NEW_ROOT!;
    const search = await $('input[type="search"]');
    execFileSync(cli, ['use', newRoot], { env: process.env });
    for (let tick = 0; tick < 5; tick++) {
      await browser.pause(250); // Deliberately cross the store's 200ms polling interval.
      await search.clearValue();
      await search.setValue('ledger');
      await expect($('h2')).toHaveText('New ledger migration');
      const results = JSON.parse(execFileSync(cli, ['search', 'ledger'], { env: process.env, encoding: 'utf8' }));
      expect(results.map((r: { title: string }) => r.title)).toEqual(['New ledger migration']);
    }
    await browser.saveScreenshot(resolve('evidence/native-root-switch.png'));
    const note = resolve(newRoot, 'new.md');
    const original = readFileSync(note);
    try {
      writeFileSync(note, '# Converged ledger\n\nExternal convergence edit.\n');
      await browser.waitUntil(async () => {
        await search.clearValue();
        await search.setValue('convergence');
        await browser.waitUntil(async () => (await $('[role="status"]').getText()) !== 'Searching…');
        return await $('h2').isExisting();
      }, { timeout: 5000, interval: 250, timeoutMsg: 'External edit did not converge through real desktop IPC' });
      await expect($('h2')).toHaveText('Converged ledger');
      await browser.saveScreenshot(resolve('evidence/native-watcher-edit.png'));
    } finally {
      writeFileSync(note, original);
    }
  });
});
