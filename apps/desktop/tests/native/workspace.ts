import { browser, $, expect } from '@wdio/globals';
import { resolve } from 'node:path';

// Real WebView + Rust IPC; runner creates only synthetic roots and app data.
describe('Topic workspace native service journey', () => {
  it('shows the selected synthetic root, bounded source preview and durable manual capture', async () => {
    await $('button=Analyse notes').click();
    await $('#scope-folders').setValue('memory/projects');
    await $('button=Preview selection').click();
    await expect($('h2=Review selected notes')).toBeDisplayed();
    await expect($('p=memory/projects/ledger-migration.md')).toBeDisplayed();
    await expect($('button=Start analysis')).toBeDisabled();
    await $('button=New topic').click();
    await $('#topic-name').setValue('Synthetic Ledger');
    await $('button=Create proposal').click();
    await expect($('input[aria-label="Topic name"]')).toHaveValue(
      'Synthetic Ledger',
    );
    await $('button=Approve selected changes').click();
    await $('button=Topics').click();
    await expect($('button=Synthetic Ledger')).toBeDisplayed();
    await browser.saveScreenshot(resolve('evidence/native-workspace.png'));
  });
});
