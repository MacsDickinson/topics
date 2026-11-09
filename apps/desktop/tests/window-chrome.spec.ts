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
