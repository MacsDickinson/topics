import { browser, $, expect } from '@wdio/globals';
import { resolve } from 'node:path';

describe('invalid app-data startup configuration', () => {
  it('keeps a usable window and presents the resolver error without fallback', async () => {
    const search = await $('input[type="search"]');
    await search.waitForDisplayed();
    await search.setValue('ledger');
    await expect($('[role="alert"]')).toHaveText(
      expect.stringContaining(
        'TOPICS_APP_DATA_DIR must be a non-empty absolute path',
      ),
    );
    await expect($('[role="status"]')).not.toHaveText('No matches');
    await browser.saveScreenshot(
      resolve(
        `evidence/native-startup-${process.env.TOPICS_APP_DATA_DIR ? 'relative' : 'empty'}.png`,
      ),
    );
  });
});
