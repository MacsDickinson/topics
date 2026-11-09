import { test, expect } from '@playwright/test';
import { workspaceFixture } from './fixtures/workspace';

test('connecting a new folder opens local inventory before analysis', async ({ page }) => {
  await workspaceFixture(page, { empty: true });
  await page.getByLabel('Notes folder path').fill('/synthetic/new-notes');
  await page.getByRole('button', { name: 'Use notes folder' }).click();
  await expect(page.getByRole('heading', { name: 'Understand this folder' })).toBeVisible();
  await expect(page.getByText('Suggested role', { exact: false }).first()).toBeVisible();
  expect(await page.evaluate(() => (window as any).calls.filter((c: any) => c.command === 'workspace_analysis_start'))).toHaveLength(0);
});

test('setup offers tree selection and an exact manifest before building', async ({ page }) => {
  await workspaceFixture(page);
  await page.getByRole('button', { name: 'Analyse notes', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Choose notes to analyse' })).toBeVisible();
  await expect(page.getByRole('tree', { name: 'Notes folders' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Review manifest' })).toBeVisible();
});

test('topic map offers graph and list navigation', async ({ page }) => {
  await workspaceFixture(page);
  await expect(page.getByRole('button', { name: 'Graph', exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: 'List', exact: true })).toBeVisible();
});

async function showDraft(page: import('@playwright/test').Page) {
  await page.getByRole('button', { name: 'Analyse notes', exact: true }).click();
  await page.getByLabel('Folders', { exact: true }).fill('Meetings');
  await page.getByRole('button', { name: 'Preview selection' }).click();
  await page.getByLabel('Allow these unknown-sensitivity notes for this run').check();
  await page.getByRole('button', { name: 'Start analysis', exact: true }).click();
  await page.evaluate(() => {
    const w = window as any;
    w.showProposal();
    w.emitRun(1, { kind: 'completed' });
  });
  await page.getByRole('button', { name: /Inbox/ }).click();
}

test('draft map exposes one exact starting-map receipt without record checkboxes', async ({ page }) => {
  await workspaceFixture(page);
  await showDraft(page);
  await expect(page.getByRole('button', { name: 'Review starting map' })).toBeVisible();
  await page.getByRole('button', { name: 'Review starting map' }).click();
  await expect(page.getByRole('heading', { name: 'Save starting map' })).toBeVisible();
  await expect(page.getByText('Needs your input', { exact: false }).first()).toBeVisible();
  await expect(page.getByRole('checkbox', { name: /^Include (topic|claim|connection)/ })).toHaveCount(0);
});

test('a saved topic offers contextual decisions and collapsed evidence', async ({ page }) => {
  await workspaceFixture(page);
  await showDraft(page);
  await page.getByRole('button', { name: 'Approve selected changes' }).click();
  await page.getByRole('button', { name: 'Topics', exact: true }).click();
  await page.getByRole('button', { name: 'Ledger', exact: true }).click();
  await expect(page.getByText('From your notes', { exact: false }).first()).toBeVisible();
  for (const name of ['Confirm', "That’s wrong", 'Add context', 'Not now']) {
    await expect(page.getByRole('button', { name, exact: true }).first()).toBeVisible();
  }
  await expect(page.getByText('Ledger launch is Friday', { exact: true })).toHaveCount(0);
});


test('folder chooser cancellation preserves the path and selection waits for explicit use', async ({ page }) => {
  await workspaceFixture(page, { empty: true });
  await page.getByLabel('Notes folder path').fill('/synthetic/manual');
  await page.getByRole('button', { name: 'Choose folder…' }).click();
  await expect(page.getByLabel('Notes folder path')).toHaveValue('/synthetic/manual');
  await page.evaluate(() => { (window as any).chosenRoot = '/synthetic/chosen'; });
  await page.getByRole('button', { name: 'Choose folder…' }).click();
  await expect(page.getByLabel('Notes folder path')).toHaveValue('/synthetic/chosen');
  expect(await page.evaluate(() => (window as any).calls.filter((c: any) => c.command === 'workspace_root_set'))).toHaveLength(0);
});
