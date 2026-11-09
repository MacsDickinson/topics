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

// Measure resolved colors, including pseudo-element opacity; palette values are not repeated here.
async function contrastPair(locator: import('@playwright/test').Locator, kind: 'border' | 'placeholder' | 'text') {
  const colors = await locator.evaluate((el, kind) => {
    const style = getComputedStyle(el), pseudo = getComputedStyle(el, '::placeholder');
    let parent = el.parentElement;
    while (parent && getComputedStyle(parent).backgroundColor === 'rgba(0, 0, 0, 0)') parent = parent.parentElement;
    return { foreground: kind === 'border' ? style.borderTopColor : kind === 'placeholder' ? pseudo.color : style.color,
      opacity: kind === 'placeholder' ? Number(pseudo.opacity) : 1, background: style.backgroundColor,
      surrounding: parent ? getComputedStyle(parent).backgroundColor : 'rgb(255, 255, 255)' };
  }, kind);
  const parse = (value: string) => {
    const parts = value.match(/[\d.]+/g)?.map(Number);
    if (!parts || parts.length < 3) throw new Error(`Unsupported computed color: ${value}`);
    return [parts[0], parts[1], parts[2], parts[3] ?? 1];
  };
  const blend = (front: number[], back: number[], opacity = 1) => front.slice(0, 3).map((v, i) => v * front[3] * opacity + back[i] * (1 - front[3] * opacity)).concat(1);
  const surrounding = blend(parse(colors.surrounding), [255, 255, 255, 1]);
  const background = blend(parse(colors.background), surrounding);
  const foreground = blend(parse(colors.foreground), background, colors.opacity);
  const luminance = (rgb: number[]) => rgb.slice(0, 3).map(v => { const c = v / 255; return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4; }).reduce((sum, v, i) => sum + v * [0.2126, 0.7152, 0.0722][i], 0);
  const ratio = (a: number[], b: number[]) => { const [low, high] = [luminance(a), luminance(b)].sort((x, y) => x - y); return (high + 0.05) / (low + 0.05); };
  return { againstControl: ratio(foreground, background), againstSurrounding: ratio(foreground, surrounding) };
}

for (const scheme of ['light', 'dark'] as const) {
  test(`${scheme} rendered field and primary states meet contrast thresholds`, async ({ page }) => {
    await page.emulateMedia({ colorScheme: scheme });
    await workspaceFixture(page, { empty: true });
    const field = page.getByLabel('Notes folder path');
    const border = await contrastPair(field, 'border'), placeholder = await contrastPair(field, 'placeholder');
    expect(border.againstControl).toBeGreaterThanOrEqual(3);
    expect(border.againstSurrounding).toBeGreaterThanOrEqual(3);
    expect(placeholder.againstControl).toBeGreaterThanOrEqual(4.5);
    await field.fill('/synthetic/new-notes');
    const primary = page.getByRole('button', { name: 'Use notes folder' });
    await expect(primary).toBeEnabled();
    const normal = await contrastPair(primary, 'text');
    expect(normal.againstControl).toBeGreaterThanOrEqual(4.5);
    await primary.hover();
    await page.waitForTimeout(200); // Let the actual hover transition settle.
    const hover = await contrastPair(primary, 'text');
    expect(hover.againstControl).toBeGreaterThanOrEqual(4.5);
    console.log(`${scheme}: field ${border.againstControl.toFixed(2)}/${border.againstSurrounding.toFixed(2)}, placeholder ${placeholder.againstControl.toFixed(2)}, primary ${normal.againstControl.toFixed(2)}/${hover.againstControl.toFixed(2)}`);
    await workspaceFixture(page);
    await page.getByRole('button', { name: 'Analyse notes', exact: true }).click();
    const textarea = page.getByLabel('Folders', { exact: true });
    expect((await contrastPair(textarea, 'placeholder')).againstControl).toBeGreaterThanOrEqual(4.5);
    await textarea.fill('Meetings');
    await page.getByRole('button', { name: 'Preview selection' }).click();
    await page.getByLabel('Allow these unknown-sensitivity notes for this run').check();
    await page.getByRole('button', { name: 'Start analysis', exact: true }).click();
    await page.evaluate(() => { (window as any).showProposal(); (window as any).emitRun(1, { kind: 'proposal', proposal_id: '00000000-0000-4000-8000-000000000005' }); (window as any).emitRun(2, { kind: 'completed' }); });
    await page.getByRole('button', { name: /Inbox/ }).click();
    const candidateField = page.getByLabel('Claim value');
    await expect(candidateField).toBeVisible();
    const candidateBorder = await contrastPair(candidateField, 'border');
    expect(candidateBorder.againstControl).toBeGreaterThanOrEqual(3);
    expect(candidateBorder.againstSurrounding).toBeGreaterThanOrEqual(3);
    console.log(`${scheme}: review field ${candidateBorder.againstControl.toFixed(2)}/${candidateBorder.againstSurrounding.toFixed(2)}`);
  });
}

test('long synthetic source and controls remain usable at a 200% zoom-equivalent width', async ({ page }) => {
  // A 600 CSS-pixel viewport is the layout width of a 1200-pixel window at 200% zoom.
  await page.setViewportSize({ width: 600, height: 850 });
  await workspaceFixture(page);
  const longPath = `Projects/${'long-unbroken-path-segment-'.repeat(8)}notes.md`;
  const longText = `Synthetic note: ${'long-unbroken-note-content-'.repeat(20)}`;
  await page.evaluate(({ longPath, longText }) => {
    const w = window as any, invoke = w.__TAURI_INTERNALS__.invoke;
    w.__TAURI_INTERNALS__.invoke = async (command: string, args: unknown) => {
      const result = await invoke(command, args);
      if (command === 'workspace_sources_list') result.sources[0].path = longPath;
      if (command === 'workspace_source_get') { result.summary.path = longPath; result.content = longText; }
      return result;
    };
  }, { longPath, longText });
  await page.getByRole('button', { name: 'Sources', exact: true }).click();
  const source = page.getByRole('button', { name: longPath });
  await source.focus();
  await page.keyboard.press('Tab');
  await page.keyboard.press('Shift+Tab');
  await expect(source).toBeFocused();
  expect(await source.evaluate(el => getComputedStyle(el).outlineStyle)).not.toBe('none');
  await page.keyboard.press('Enter');
  await expect(page.getByText(longText, { exact: true })).toBeVisible();
  const sensitivity = page.getByLabel('Source sensitivity');
  await expect(sensitivity).toBeEnabled();
  expect((await contrastPair(sensitivity, 'border')).againstControl).toBeGreaterThanOrEqual(3);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: 'evidence/design/long-source-zoom-equivalent.png', fullPage: true });
  await page.getByRole('button', { name: 'Analyse notes', exact: true }).click();
  await page.getByLabel('Folders', { exact: true }).fill('Meetings');
  const preview = page.getByRole('button', { name: 'Preview selection' });
  await expect(preview).toBeEnabled();
  await preview.focus();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('heading', { name: 'Review selected notes' })).toBeVisible();
});
