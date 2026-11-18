import { $, expect } from '@wdio/globals';

describe('Friday meeting history after restarting Topics', () => {
  it('opens the most recent briefing and retrieves the saved feedback', async () => {
    await expect($('h1=Today')).toBeDisplayed();
    const latest = await $('button*=Open briefing');
    await latest.waitForDisplayed();
    await latest.click();
    await expect($('p=The second meeting moved to Wednesday.')).toBeDisplayed();
  });
});
