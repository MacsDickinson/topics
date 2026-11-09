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
