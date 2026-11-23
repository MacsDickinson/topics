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
      // Navigation can finish before React mounts or the root lookup completes.
      // Start the keyboard sequence only once the disconnected workspace is ready.
      await expect(page.getByLabel("Notes folder path")).toBeVisible();
      await page.bringToFront();
      await expect.poll(() => page.evaluate(() => document.hasFocus())).toBe(true);
      expect(await page.evaluate(() => document.activeElement === document.body)).toBe(true);
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

test("title bar follows the navigation palette and keeps status outside the drag region", async ({ page }) => {
  await page.setViewportSize({ width: 560, height: 420 });
  await workspaceFixture(page, { unavailable: true });
  const bar = page.getByRole("banner", { name: "Workspace status" });
  const status = bar.getByRole("button", { name: "Workspace status details" });
  for (const scheme of ["light", "dark"]) {
    await page.getByLabel("Appearance").selectOption(scheme);
    await expect(page.locator("html")).toHaveAttribute("data-theme", scheme);
    const surface = await page.locator(".sidebar").evaluate(el => getComputedStyle(el).backgroundColor);
    await expect(bar).toHaveCSS("background-color", surface);
  }
  const drag = await bar.locator("[data-tauri-drag-region]").boundingBox();
  const trigger = await status.boundingBox();
  expect(drag!.x + drag!.width).toBeLessThanOrEqual(trigger!.x);
  await status.focus();
  await page.keyboard.press("Enter");
  const retry = bar.getByRole("button", { name: "Check runtime again" });
  const runtimeChecks = () => page.evaluate(() =>
    (window as any).calls.filter((call: { command: string }) => call.command === "workspace_runtime_status").length,
  );
  const initialChecks = await runtimeChecks();
  for (const attempt of [1, 2]) {
    await retry.click();
    await expect.poll(runtimeChecks).toBe(initialChecks + attempt);
    await expect(status).toBeFocused();
    await expect(retry).toBeVisible();
  }
  await page.keyboard.press("Escape");
  await expect(status).toBeFocused();
  await expect(retry).toHaveCount(0);
  await status.click();
  await page.getByRole("heading").first().click();
  await expect(retry).toHaveCount(0);
});

test("status text stays selectable and the panel closes on Escape or tabbing away", async ({ page }) => {
  await workspaceFixture(page, { unavailable: true });
  const trigger = page.getByRole("button", { name: "Workspace status details" });
  await trigger.focus();
  await page.keyboard.press("Enter");
  const panel = page.getByRole("region", { name: "Status details" });
  const message = panel.getByText(/Codex is missing/);
  const text = await message.innerText();
  await message.click({ clickCount: 3 });
  await expect(panel).toBeVisible();
  expect(await page.evaluate(() => window.getSelection()?.toString().trim())).toBe(text);
  await page.keyboard.press("Escape");
  await expect(panel).toHaveCount(0);
  await expect(trigger).toBeFocused();

  await page.keyboard.press("Enter");
  await panel.getByRole("button", { name: "Check runtime again" }).focus();
  await page.keyboard.press("Tab");
  await expect(panel).toHaveCount(0);
});
