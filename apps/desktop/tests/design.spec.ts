import { test, expect } from '@playwright/test';
import { workspaceFixture } from './fixtures/workspace';

// Synthetic browser transport only; these screenshots do not represent user notes.
test('theme, keyboard and narrow layouts keep review controls usable', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 850 });
  await workspaceFixture(page, { empty: true });
  await expect(page.getByRole('heading', { name: 'Start with your notes' })).toBeVisible();
  await page.screenshot({ path: 'evidence/design/setup-light.png', fullPage: true });
  await page.keyboard.press('Tab');
  await expect(page.locator(':focus-visible')).toHaveCount(1);
  await page.emulateMedia({ colorScheme: 'dark', reducedMotion: 'reduce' });
  await page.waitForTimeout(150);
  await page.screenshot({ path: 'evidence/design/setup-dark.png', fullPage: true });
  const colors = await page.locator('body').evaluate(el => ({ color: getComputedStyle(el).color, background: getComputedStyle(el).backgroundColor }));
  expect(colors.color).not.toBe(colors.background);
  await page.setViewportSize({ width: 600, height: 850 });
  await expect(page.getByLabel('Notes folder path')).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: 'evidence/design/setup-narrow.png', fullPage: true });
});

test('synthetic sources, scope review and unavailable runtime render', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 850 });
  await workspaceFixture(page, { unavailable: true });
  await expect(page.getByText(/Codex is missing/)).toBeVisible();
  await page.screenshot({ path: 'evidence/design/empty-runtime.png', fullPage: true });
  await page.getByRole('button', { name: 'Sources', exact: true }).click();
  await expect(page.getByText('Meetings/one.md', { exact: true })).toBeVisible();
  await page.screenshot({ path: 'evidence/design/sources.png', fullPage: true });
  await page.getByRole('button', { name: 'Analyse notes', exact: true }).click();
  await page.getByLabel('Folders', { exact: true }).fill('Meetings');
  await page.getByRole('button', { name: 'Preview selection' }).click();
  await expect(page.getByRole('heading', { name: 'Review selected notes' })).toBeVisible();
  await page.screenshot({ path: 'evidence/design/scope-review.png', fullPage: true });
});
