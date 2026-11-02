import { test, expect } from '@playwright/test';

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    (window as any).pending = {};
    (window as any).__TAURI_INTERNALS__ = { invoke: (command: string, args: { query: string }) => {
      if (command !== 'search_notes') throw new Error(command);
      return new Promise((resolve, reject) => { (window as any).pending[args.query] = { resolve, reject }; });
    } };
  });
  await page.goto('/');
});

test('late responses cannot replace newer results or cleared input', async ({ page }) => {
  const input = page.getByRole('searchbox', { name: 'Search notes' });
  await input.fill('old');
  await expect.poll(() => page.evaluate(() => !!(window as any).pending.old)).toBe(true);
  await expect(page.getByRole('status')).toHaveText('Searching…');
  await input.fill('new');
  await expect.poll(() => page.evaluate(() => !!(window as any).pending.new)).toBe(true);
  await page.evaluate(() => (window as any).pending.new.resolve([{ title: 'New', path: 'new.md', snippet: '<b>plain text</b>' }]));
  await expect(page.getByRole('heading', { name: 'New', exact: true })).toBeVisible();
  await page.evaluate(() => (window as any).pending.old.resolve([{ title: 'Old', path: 'old.md', snippet: '' }]));
  await expect(page.getByRole('heading', { name: 'Old', exact: true })).toHaveCount(0);
  await expect(page.getByText('<b>plain text</b>', { exact: true })).toBeVisible();
  await input.fill('pending');
  await expect.poll(() => page.evaluate(() => !!(window as any).pending.pending)).toBe(true);
  await input.fill('');
  await page.evaluate(() => (window as any).pending.pending.reject('late failure'));
  await expect(page.getByRole('status')).toHaveText('Type to search your notes.');
  await expect(page.getByRole('alert')).toHaveCount(0);
});

test('IPC errors are visible and a subsequent search recovers', async ({ page }) => {
  const input = page.getByRole('searchbox', { name: 'Search notes' });
  await input.fill('broken');
  await expect.poll(() => page.evaluate(() => !!(window as any).pending.broken)).toBe(true);
  await page.evaluate(() => (window as any).pending.broken.reject('Index unavailable'));
  await expect(page.getByRole('alert')).toHaveText('Search failed: Index unavailable');
  await expect(page.getByRole('status')).not.toHaveText('No matches');
  await input.fill('recovered');
  await expect.poll(() => page.evaluate(() => !!(window as any).pending.recovered)).toBe(true);
  await page.evaluate(() => (window as any).pending.recovered.resolve([]));
  await expect(page.getByRole('status')).toHaveText('No matches');
  await expect(page.getByRole('alert')).toHaveCount(0);
});

test('hostile result markup stays inert while FTS highlights render', async ({ page }) => {
  const baselineScripts = await page.locator('script').count();
  await page.evaluate(() => { (window as any).hostileExecutions = 0; });
  const hostile = '<img src="missing-hostile-image" onerror="window.hostileExecutions++"><script>window.hostileExecutions++</script>';
  const input = page.getByRole('searchbox', { name: 'Search notes' });
  await input.fill('hostile');
  await expect.poll(() => page.evaluate(() => !!(window as any).pending.hostile)).toBe(true);
  await page.evaluate(({ hostile }) => (window as any).pending.hostile.resolve([{
    title: `Hostile title ${hostile}`,
    path: 'synthetic/hostile.md',
    snippet: `Before ${hostile} <mark>ledger</mark> after`,
  }]), { hostile });
  await expect(page.getByRole('heading', { name: `Hostile title ${hostile}`, exact: true })).toBeVisible();
  await expect(page.locator('li p').last()).toHaveText(`Before ${hostile} ledger after`);
  await expect(page.locator('li mark')).toHaveText('ledger');
  await expect(page.locator('img')).toHaveCount(0);
  await expect(page.locator('script')).toHaveCount(baselineScripts);
  await expect(page.locator('li script')).toHaveCount(0);
  // Flush rendering and a browser task so event-handler/script execution would
  // have a chance to run if source markup had been inserted as HTML.
  await page.evaluate(() => new Promise<void>(resolve => requestAnimationFrame(() => setTimeout(resolve, 0))));
  expect(await page.evaluate(() => (window as any).hostileExecutions)).toBe(0);
});
