import { browser, $, $$, expect } from '@wdio/globals';
import { chooseOption } from './interaction';
import { resolve, join } from 'node:path';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';

// Real WebView + Rust IPC; runner creates only synthetic roots and app data.
describe('Topic workspace native service journey', () => {
  it('creates a fresh root, previews real notes, labels sources and reviews a durable manual proposal without Codex', async () => {
    const root = process.env.TOPICS_TEST_WORKSPACE_ROOT!;
    const rootInput = await $('#root-path');
    await rootInput.waitForDisplayed();
    await rootInput.setValue(root);
    await $('input[type="checkbox"]').click();
    await $('button=Use notes folder').click();
    await expect($('h2=No accepted topics yet')).toBeDisplayed();
    await browser.saveScreenshot(
      resolve('evidence/native-workspace-empty.png'),
    );
    const source = join(root, 'memory/projects/ledger-migration.md');
    mkdirSync(join(root, 'memory/projects'), { recursive: true });
    const original =
      '# Ledger migration\n\nThe synthetic ledger migration is ready for review.\n';
    writeFileSync(source, original);
    await $('button=Analyse notes').click();
    await $('#scope-folders').setValue('memory/projects');
    await $('button=Preview selection').click();
    await expect($('h2=Review selected notes')).toBeDisplayed();
    await expect($('.scope-review .path')).toHaveText(
      'memory/projects/ledger-migration.md',
    );
    await expect($('button=Start analysis')).toBeDisabled();
    // A real unavailable binary is configured only in the native-test build.
    await expect($('.runtime')).toHaveText(expect.stringContaining('Codex'));
    await $('button=Sources').click();
    await $('button[aria-label="memory/projects/ledger-migration.md"]').click();
    await expect($('.note-content')).toHaveText(original.trim());
    await chooseOption('select[aria-label="Source sensitivity"]', 'local_only');
    await expect($('.source-inspector .badge')).toHaveText('Local-only');
    await $('button=New topic').click();
    await $('#topic-name').setValue('Synthetic Ledger');
    await $('button=Create proposal').click();
    await expect($('input[aria-label="Topic name"]')).toHaveValue(
      'Synthetic Ledger',
    );
    await expect($('.badge.pending')).toHaveText('Needs review');
    await $('button=Approve selected changes').click();
    await expect($('.badge.accepted')).toHaveText('Accepted');
    await $('button[aria-label="Topics"]').click();
    await $('button[aria-label="Synthetic Ledger"]').click();
    await expect($('.topic-detail h1')).toHaveText('Synthetic Ledger');
    await browser.saveScreenshot(resolve('evidence/native-workspace.png'));
    await $('button=History').click();
    await expect($('.operation')).toBeDisplayed();
    for (const operation of await $$('.operation')) {
      if ((await operation.getText()).includes('Approve reviewed proposal')) {
        await operation.$('button=Undo').click();
        break;
      }
    }
    await expect($('.notice')).toHaveText('Undo recorded in history.');
    await browser.saveScreenshot(
      resolve('evidence/native-workspace-history.png'),
    );
    await $('button[aria-label="Topics"]').click();
    await expect($('button[aria-label="Synthetic Ledger"]')).not.toExist();
    expect(readFileSync(source, 'utf8')).toEqual(original);
  });
});
