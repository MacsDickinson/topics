import { test, expect, type Page } from '@playwright/test';
import { workspaceFixture, ids, topic, claim } from './fixtures/workspace';

async function preview(page: Page) {
  await page.getByRole('button', { name: 'Analyse notes', exact: true }).click();
  await page.getByLabel('Folders', { exact: true }).fill('Meetings');
  await page.getByLabel('From date').fill('2026-11-01');
  await page.getByLabel('Through date').fill('2026-11-02');
  await page.getByRole('button', { name: 'Preview selection' }).click();
  await expect(page.getByRole('heading', { name: 'Review selected notes' })).toBeVisible();
}
async function start(page: Page) {
  await preview(page);
  await page.getByLabel('Allow these unknown-sensitivity notes for this run').check();
  await page.getByRole('button', { name: 'Start analysis', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Stop run' })).toBeVisible();
}
async function finish(page: Page) {
  await page.evaluate(() => { (window as any).showProposal(); (window as any).emitRun(1, { kind: 'proposal', proposal_id: '00000000-0000-4000-8000-000000000005' }); (window as any).emitRun(2, { kind: 'completed' }); });
  await page.getByRole('button', { name: /Inbox/ }).click();
  await expect(page.getByLabel('Claim value')).toHaveValue('Friday');
}

test('create a fresh root and explain an honest empty workspace', async ({ page }) => {
  await workspaceFixture(page, { empty: true });
  await page.getByLabel('Notes folder path').fill('/synthetic/new-notes');
  await page.getByLabel('Create a new notes folder').check();
  await page.getByRole('button', { name: 'Use notes folder' }).click();
  await expect(page.getByText('Add Markdown notes to this folder, then analyse a selection.')).toBeVisible();
  expect(await page.evaluate(() => (window as any).calls.find((c: any) => c.command === 'workspace_root_set').args)).toEqual({ request: { path: '/synthetic/new-notes', create: true } });
});

test('bounded date preview names exclusions, requires consent and invalidates on selection edit', async ({ page }) => {
  await workspaceFixture(page);
  await preview(page);
  await expect(page.getByText('Meetings/one.md', { exact: true })).toBeVisible();
  await expect(page.getByText('Unknown date', { exact: true })).toBeVisible();
  await expect(page.getByText('Local-only', { exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Start analysis', exact: true })).toBeDisabled();
  expect(await page.evaluate(() => (window as any).calls.filter((c: any) => c.command === 'workspace_analysis_start'))).toHaveLength(0);
  await page.getByLabel('Allow these unknown-sensitivity notes for this run').check();
  await expect(page.getByRole('button', { name: 'Start analysis', exact: true })).toBeEnabled();
  await page.getByLabel('Through date').fill('2026-11-03');
  await expect(page.getByRole('heading', { name: 'Review selected notes' })).toHaveCount(0);
});

test('discovery, cited edit and human approval, accepted understanding, chat diff and history conflict', async ({ page }) => {
  await workspaceFixture(page);
  await start(page);
  await page.evaluate(() => (window as any).emitRun(0, { kind: 'text_delta', text: 'Reading selected notes…' }));
  await expect(page.getByText('Reading selected notes…', { exact: true })).toBeVisible();
  await finish(page);
  await expect(page.getByText('Ledger launch is Friday', { exact: true })).toBeVisible();
  await expect(page.getByText('Bytes 0–23', { exact: false })).toBeVisible();
  await page.getByLabel('Claim value').fill('Monday');
  await page.getByRole('button', { name: 'Approve selected changes' }).click();
  await page.getByRole('button', { name: 'Topics', exact: true }).click();
  await page.getByRole('button', { name: 'Ledger', exact: true }).click();
  await expect(page.getByText('Monday', { exact: true })).toBeVisible();
  await expect(page.getByText('Transcript; confirm the date.', { exact: true })).toBeVisible();
  await page.getByLabel('Message about this topic').fill('The launch is Tuesday; Monday was a mistake.');
  await expect(page.getByRole('button', { name: 'Send message' })).toBeDisabled();
  await page.getByRole('button', { name: 'Preview chat sources' }).click();
  await page.getByLabel('Allow these unknown-sensitivity notes for this run').check();
  await page.getByRole('button', { name: 'Send message' }).click();
  expect(await page.evaluate(() => (window as any).calls.find((c: any) => c.command === 'workspace_chat_start').args.request)).toMatchObject({ root_id: ids.root, topic_id: ids.topic, scope_id: ids.scope, confirm_unknown: true, message: 'The launch is Tuesday; Monday was a mistake.' });
  await page.getByRole('button', { name: 'Stop run' }).click();
  await page.getByRole('button', { name: 'History', exact: true }).click();
  await page.getByRole('button', { name: 'Undo', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('Undo the dependent claim first.');
  await page.screenshot({ path: 'evidence/workspace-history.png' });
});

test('approval errors preserve edited candidates and rejected proposals never become topics', async ({ page }) => {
  await workspaceFixture(page); await start(page); await finish(page);
  await page.getByLabel('Claim value').fill('Monday');
  await page.evaluate(() => { (window as any).failures.workspace_proposal_decide = { code: 'stale_evidence', message: 'This passage changed. Preview and analyse the note again.', details: null }; });
  await page.getByRole('button', { name: 'Approve selected changes' }).click();
  await expect(page.getByRole('alert')).toContainText('This passage changed.');
  await expect(page.getByLabel('Claim value')).toHaveValue('Monday');
  await page.evaluate(() => { delete (window as any).failures.workspace_proposal_decide; });
  await page.getByRole('button', { name: 'Reject proposal' }).click();
  await expect(page.getByText('Rejected', { exact: true })).toBeVisible();
  expect(await page.evaluate(() => (window as any).topics)).toEqual([]);
});

test('duplicate sequences, wrong roots and cancelled late streams cannot update workspace', async ({ page }) => {
  await workspaceFixture(page); await start(page);
  await page.evaluate(() => { const w = window as any; w.emitRun(1, { kind: 'text_delta', text: 'Once' }); w.emitRun(1, { kind: 'text_delta', text: ' duplicate' }); w.emitRun(2, { kind: 'text_delta', text: ' wrong' }, 'other-root'); });
  await expect(page.getByText('Once', { exact: true })).toBeVisible();
  await expect(page.getByText(/duplicate|wrong/, { exact: false })).toHaveCount(0);
  await page.getByRole('button', { name: 'Stop run' }).click();
  await page.evaluate(() => { const w = window as any; w.showProposal(); w.emitRun(3, { kind: 'text_delta', text: 'LATE SECRET' }); w.emitRun(4, { kind: 'proposal', proposal_id: 'late' }); w.emitRun(5, { kind: 'completed' }); });
  await expect(page.getByText('LATE SECRET', { exact: true })).toHaveCount(0);
  await page.getByRole('button', { name: /Inbox/ }).click();
  await expect(page.getByLabel('Claim value')).toHaveCount(0);
});

test('changing root cancels run and clears old topic, proposal and delayed reads', async ({ page }) => {
  await workspaceFixture(page); await start(page);
  await page.getByRole('button', { name: 'Change notes folder' }).click();
  await page.getByLabel('Notes folder path').fill('/synthetic/other-notes');
  await page.getByRole('button', { name: 'Use notes folder' }).click();
  await page.evaluate(() => { const w = window as any; w.emitRun(9, { kind: 'text_delta', text: 'OLD ROOT' }); w.emitRun(10, { kind: 'completed' }); });
  await expect(page.getByText('OLD ROOT', { exact: true })).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Ledger', exact: true })).toHaveCount(0);
  expect(await page.evaluate(() => (window as any).calls.filter((c: any) => c.command === 'workspace_run_cancel').map((c: any) => c.args.request))).toContainEqual({ root_id: ids.root, run_id: ids.run });
});

test('missing runtime leaves topic browsing and manual proposal capture available', async ({ page }) => {
  await workspaceFixture(page, { unavailable: true });
  await expect(page.getByText('Codex is missing. Install Codex 0.166.0 and restart Topics.')).toBeVisible();
  await expect(page.getByRole('button', { name: 'New topic', exact: true })).toBeEnabled();
  await page.getByRole('button', { name: 'New topic', exact: true }).click();
  await expect(page.getByLabel('Topic name', { exact: true })).toBeVisible();
});
