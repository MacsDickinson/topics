import { browser, $, $$, expect } from '@wdio/globals';
import { resolve, join } from 'node:path';
import { readFileSync, writeFileSync } from 'node:fs';
import { chooseOption } from './interaction';
async function invoke<T>(command: string, request: object): Promise<T> {
  return browser.execute(
    async (command, request) =>
      (window as any).__TAURI__.core.invoke(command, { request }),
    command,
    request,
  ) as Promise<T>;
}
async function finished() {
  await browser.waitUntil(
    async () =>
      (await $('button=Stop run').isExisting()) ||
      (await $('[role="alert"]').isExisting()),
    { timeout: 15000, interval: 100, timeoutMsg: 'Run did not start' },
  );
  await browser.waitUntil(
    async () => !(await $('button=Stop run').isExisting()),
    {
      timeout: 180000,
      interval: 500,
      timeoutMsg: 'Synthetic model turn did not terminate',
    },
  );
  if (await $('[role="alert"]').isExisting())
    throw new Error(await $('[role="alert"]').getText());
}
describe('Genuine synthetic discovery and topic correction through real desktop IPC', () => {
  it('discovers cited pending knowledge, accepts it, proposes a chat correction and accepts only by human action', async () => {
    const rootPath = process.env.TOPICS_TEST_WORKSPACE_ROOT!;
    const original = readFileSync(join(rootPath, 'meeting.md'), 'utf8');
    await $('#root-path').waitForDisplayed();
    await $('#root-path').setValue(rootPath);
    await $('button=Use notes folder').click();
    await expect($('h2=No accepted topics yet')).toBeDisplayed();
    await expect($('button=Analyse notes')).toBeEnabled();
    await $('button=Analyse notes').click();
    await $('#scope-files').setValue('meeting.md');
    await $('button=Preview selection').click();
    await expect($('.scope-review .path')).toHaveText('meeting.md');
    await $('.consent input').click();
    await $('button=Start analysis').click();
    await finished();
    const { root } = await invoke<any>('workspace_root_get', {});
    expect(
      (await invoke<any>('workspace_topics_list', { root_id: root.root_id }))
        .topics,
    ).toEqual([]);
    await $('button*=Inbox').click();
    await expect($('.proposal')).toBeDisplayed();
    await expect($('.source-evidence .path')).toHaveText('meeting.md');
    await browser.saveScreenshot(resolve('evidence/native-live-discovery.png'));
    await $('button=Approve selected changes').click();
    await expect($('.badge.accepted')).toBeDisplayed();
    await $('button[aria-label="Topics"]').click();
    const { topics } = await invoke<any>('workspace_topics_list', {
      root_id: root.root_id,
    });
    const alex = topics.find((t: any) => t.name.toLowerCase() === 'alex');
    expect(alex).toBeDefined();
    await $(`button[aria-label="${alex.name}"]`).click();
    await expect($('.topic-detail')).toHaveText(
      expect.stringContaining('he/him'),
    );
    await chooseOption('.topic-header select', 'cloud_allowed');
    await browser.waitUntil(
      async () =>
        (await $('.topic-header select').getValue()) === 'cloud_allowed',
      { timeout: 10000 },
    );
    await $('#topic-message').setValue(
      'Alex uses they/them pronouns. The earlier he/him assertion is incorrect. Correct the accepted pronouns claim to exactly they/them and propose a future rule scoped to Alex and pronouns, citing this message.',
    );
    await $('button=Preview chat sources').click();
    await $('.consent input').click();
    await $('button=Send message').click();
    await finished();
    await $('button*=Inbox').click();
    await expect($('h2=Proposed correction')).toBeDisplayed();
    await expect($('.diff-old')).toHaveText(expect.stringContaining('he/him'));
    let correction;
    for (const article of await $$('.proposal')) {
      if ((await article.$('h2').getText()) === 'Proposed correction') {
        correction = article;
        break;
      }
    }
    if (!correction) throw new Error('No chat correction proposal');
    await expect(correction).toHaveText(
      expect.stringContaining('Your correction'),
    );
    await expect(correction.$('input[aria-label="Claim value"]')).toHaveValue(
      'they/them',
    );
    await browser.saveScreenshot(
      resolve('evidence/native-live-correction.png'),
    );
    const before = await invoke<any>('workspace_topic_get', {
      root_id: root.root_id,
      topic_id: alex.id,
    });
    expect(
      before.current_claims.some((c: any) => c.value === 'they/them'),
    ).toBe(false);
    await $('button=Approve selected changes').click();
    await expect($('.badge.accepted')).toBeDisplayed();
    await $('button[aria-label="Topics"]').click();
    await $(`button[aria-label="${alex.name}"]`).click();
    await expect($('.topic-detail')).toHaveText(
      expect.stringContaining('they/them'),
    );
    await expect($('.user-source blockquote')).toHaveText(
      expect.stringContaining('Alex uses they/them pronouns.'),
    );
    await browser.saveScreenshot(resolve('evidence/native-live-topic.png'));
    const accepted = await invoke<any>('workspace_topic_get', { root_id: root.root_id, topic_id: alex.id });
    const current = accepted.current_claims.find((c: any) => c.value === 'they/them');
    expect(current).toBeDefined();
    writeFileSync(process.env.TOPICS_TEST_EXPECTED!, JSON.stringify({ topic_id: alex.id, revision_id: current.revision_id }));
    // Review each discovered topic's cloud label through the actual UI, then
    // analyze the same source again with authorized accepted IDs/history.
    for (const known of topics) {
      await $(`button[aria-label="${known.name}"]`).click();
      await chooseOption('.topic-header select', 'cloud_allowed');
      await browser.waitUntil(async () => await $('.topic-header select').getValue() === 'cloud_allowed', { timeout: 10000 });
    }
    await $('button=Analyse notes').click(); await $('#scope-files').setValue('meeting.md'); await $('button=Preview selection').click();
    await $('.consent input').click(); await $('button=Start analysis').click(); await finished();
    const after = await invoke<any>('workspace_topic_get', { root_id: root.root_id, topic_id: alex.id });
    expect(after.current_claims.some((c: any) => c.revision_id === current.revision_id && c.value === 'they/them')).toBe(true);
    expect((await invoke<any>('workspace_topics_list', { root_id: root.root_id })).topics.map((t: any) => t.id).sort()).toEqual(topics.map((t: any) => t.id).sort());
    const proposals = (await invoke<any>('workspace_proposals_list', { root_id: root.root_id })).proposals;
    expect(proposals.filter((p: any) => p.status === 'pending').flatMap((p: any) => p.candidates).some((c: any) => c.kind === 'topic' && topics.some((t: any) => t.name.toLowerCase() === c.topic.name.toLowerCase()))).toBe(false);
    console.log('Synthetic acceptance: discovery reviewed, correction reviewed, same-source reanalysis preserved topic/current revision IDs and source bytes.');
    expect(readFileSync(join(rootPath, 'meeting.md'), 'utf8')).toEqual(
      original,
    );
  });
});
