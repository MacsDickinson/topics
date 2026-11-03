import { browser, $, expect } from '@wdio/globals';
import { resolve } from 'node:path';

describe('invalid app-data startup configuration', () => {
  it('keeps a usable window and presents the resolver error without fallback', async () => {
    await expect($('.boot-error')).toBeDisplayed();
    await expect($('.boot-error')).toHaveText(
      expect.stringContaining(
        'TOPICS_APP_DATA_DIR must be a non-empty absolute path',
      ),
    );
    await expect($('.boot-error')).toHaveText(
      expect.stringContaining('then restart Topics'),
    );
    await expect($('button=Check runtime again')).not.toExist();
    await browser.saveScreenshot(
      resolve(
        `evidence/native-startup-${process.env.TOPICS_APP_DATA_DIR ? 'relative' : 'empty'}.png`,
      ),
    );
  });
});
