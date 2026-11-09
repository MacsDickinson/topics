import { test, expect } from "@playwright/test";
import { workspaceFixture } from "./fixtures/workspace";

for (const theme of ["light", "dark"] as const) {
  test(`${theme} window header reserves native controls and keeps forms interactive`, async ({ page }) => {
    await page.emulateMedia({ colorScheme: theme });
    await page.setViewportSize({ width: 560, height: 420 });
    await workspaceFixture(page, { empty: true });
    const header = page.locator(".window-chrome");
    const drag = header.locator("[data-tauri-drag-region]");
    await expect(drag).toBeVisible();
    const bounds = await drag.boundingBox();
    expect(bounds!.x).toBeGreaterThanOrEqual(80);
    const input = page.getByLabel("Notes folder path");
    await input.fill("/tmp/synthetic-notes");
    await input.selectText();
    expect(await input.evaluate((el: HTMLInputElement) => el.selectionEnd! - el.selectionStart!)).toBeGreaterThan(0);
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(560);
  });
}

test("connected navigation and setup remain below the header at minimum width", async ({ page }) => {
  await page.setViewportSize({ width: 560, height: 420 });
  await workspaceFixture(page, { populated: true });
  const header = page.locator(".window-chrome");
  const headerBox = await header.boundingBox();
  const search = page.getByRole("button", { name: "Search", exact: true });
  expect((await search.boundingBox())!.y).toBeGreaterThanOrEqual(headerBox!.y + headerBox!.height);
  await page.getByRole("button", { name: "Set up folder" }).click();
  await expect(page.getByRole("main")).toBeVisible();
  await page.keyboard.press("Meta+k");
  await expect(page.getByRole("searchbox", { name: "Search notes" })).toBeFocused();
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(560);
});

test("header remains above scrolling content and desktop sidebar fits beneath it", async ({ page }) => {
  await page.setViewportSize({ width: 1180, height: 800 });
  await workspaceFixture(page, { empty: true });
  const geometry = await page.evaluate(() => {
    const sidebar = document.querySelector('.sidebar')!.getBoundingClientRect();
    return { bottom: sidebar.bottom, viewport: innerHeight };
  });
  expect(geometry.bottom).toBeLessThanOrEqual(geometry.viewport);
  await page.evaluate(() => { (document.querySelector('.workspace-main') as HTMLElement).style.minHeight = '1800px'; window.scrollTo(0, 400); });
  await expect.poll(() => page.locator('.window-chrome').evaluate(el => el.getBoundingClientRect().top)).toBe(0);
  expect(await page.locator('.sidebar').evaluate(el => el.getBoundingClientRect().top)).toBeGreaterThanOrEqual(28);
  await page.setViewportSize({ width: 560, height: 420 });
  await page.evaluate(() => window.scrollTo(0, 400));
  await expect.poll(() => page.locator('.window-chrome').evaluate(el => el.getBoundingClientRect().top)).toBe(0);
});

for (const theme of ["light", "dark"] as const) {
  for (const width of [560, 1180]) {
    test(`${theme} ${width}px first keyboard focus clears native controls`, async ({ page }) => {
      await page.emulateMedia({ colorScheme: theme });
      await page.setViewportSize({ width, height: width === 560 ? 420 : 800 });
      await workspaceFixture(page, { empty: true });
      const skip = page.getByRole("link", { name: "Skip to content" });
      await page.keyboard.press("Tab");
      await expect(skip).toBeFocused();
      const box = await skip.boundingBox();
      expect(box).not.toBeNull();
      expect(box!.y).toBeGreaterThanOrEqual(28);
      await page.keyboard.press("Enter");
      await expect(page.getByRole("main")).toBeFocused();
    });
  }
}
