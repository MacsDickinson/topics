import { test, expect } from '@playwright/test';
import { hasConvergedResult } from './native/search-predicate.mjs';

test('no-matches heading and unrelated results do not satisfy watcher convergence', async ({ page }) => {
  await page.setContent('<section aria-label="Search results"><div class="empty-state"><h2>Try another word or phrase</h2></div><ul class="result-list"></ul></section>');
  expect(await page.locator('h2').count()).toBe(1); // Old predicate returns early here.
  expect(await page.evaluate(hasConvergedResult)).toBe(false);
  await page.setContent('<ul class="result-list"><li class="search-result"><article><h2>Converged ledger</h2><p class="path">wrong.md</p></article></li></ul>');
  expect(await page.evaluate(hasConvergedResult)).toBe(false);
  await page.setContent('<ul class="result-list"><li class="search-result"><article><h2>Wrong title</h2><p class="path">new.md</p></article></li></ul>');
  expect(await page.evaluate(hasConvergedResult)).toBe(false);
  await page.setContent('<ul class="result-list"><li class="search-result"><article><h2>Converged ledger</h2><p class="path">new.md</p></article></li></ul>');
  expect(await page.evaluate(hasConvergedResult)).toBe(true);
});
