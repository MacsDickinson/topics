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
