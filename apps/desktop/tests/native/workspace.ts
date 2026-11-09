import { browser, $, $$, expect } from '@wdio/globals';
import { chooseOption } from './interaction';
import { resolve, join } from 'node:path';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';

// Real WebView + Rust IPC; runner creates only synthetic roots and app data.
describe('Topic workspace native service journey', () => {
  it('creates a fresh root, previews real notes, labels sources and reviews a durable manual proposal without Codex', async () => {
    const malformed = await browser.execute(async () => { try { await (window as any).__TAURI__.core.invoke('workspace_root_set', { request: { path: '/synthetic', create: false, unexpected: true } }); return null; } catch (error) { return error; } });
    expect(malformed).toEqual({ code: 'invalid_input', message: 'The request fields are invalid or missing. Check the input and retry.', details: null });
    const root = process.env.TOPICS_TEST_WORKSPACE_ROOT!;
    const rootInput = await $('#root-path');
    await rootInput.waitForDisplayed();
    await rootInput.setValue(root);
    await $('input[type="checkbox"]').click();
    await $('button=Use notes folder').click();
    await expect($('h2=No notes yet')).toBeDisplayed();
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
    await chooseOption('select#source-sensitivity', 'local_only');
    await expect($('.source-inspector .badge')).toHaveText('Local-only');
    await $('button=New topic').click();
    await $('#topic-name').setValue('Synthetic Ledger');
    await $('button=Create proposal').click();
    await expect($('input[aria-label="Topic name"]')).toHaveValue(
      'Synthetic Ledger',
    );
    await expect($('.proposal .badge.tone-warning')).toHaveText('Needs review');
    await $('button=Approve selected changes').click();
    await expect($('.proposal .badge.tone-success')).toHaveText('Accepted');
    await $('button[aria-label="Topics"]').click();
    await $('button[aria-label="Synthetic Ledger"]').click();
    await expect($('.topic-detail .topic-identity h2')).toHaveText('Synthetic Ledger');
    // Exercise the real human-only statement service without invoking a model.
    const stored = await browser.execute(async () => {
      const core = (window as any).__TAURI__.core;
      const root = (await core.invoke('workspace_root_get', { request: {} })).root;
      const topics = (await core.invoke('workspace_topics_list', { request: { root_id: root.root_id } })).topics;
      const topic = topics.find((entry: any) => entry.name === 'Synthetic Ledger');
      const statement = await core.invoke('workspace_user_statement', { request: { root_id: root.root_id, topic_id: topic.id, text: 'Synthetic context retained exactly.' } });
      const loaded = await core.invoke('workspace_statement_get', { request: { root_id: root.root_id, statement_id: statement.id } });
      return { statement, loaded };
    });
    expect(stored.loaded.text).toEqual('Synthetic context retained exactly.');
    expect(stored.loaded.id).toEqual(stored.statement.id);

    await browser.saveScreenshot(resolve('evidence/native-workspace.png'));
    await $('button=History').click();
    await browser.waitUntil(async () => {
      for (const operation of await $$('.operation')) if ((await operation.getText()).includes('Approve reviewed proposal')) return true;
      return false;
    }, { timeout: 10000, timeoutMsg: 'Approval operation did not appear in history' });
    for (const operation of await $$('.operation')) {
      if ((await operation.getText()).includes('Approve reviewed proposal')) {
        await operation.$('button=Undo').click();
        break;
      }
    }
    // The persisted statement depends on the topic, so undo must preserve it.
    await expect($('[role=alert]')).toHaveText(expect.stringContaining('Undo dependent operations first'));
    await browser.saveScreenshot(
      resolve('evidence/native-workspace-history.png'),
    );
    await $('button[aria-label="Topics"]').click();
    await expect($('button[aria-label="Synthetic Ledger"]')).toBeDisplayed();
    expect(readFileSync(source, 'utf8')).toEqual(original);
  });
});
