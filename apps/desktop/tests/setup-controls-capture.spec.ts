import { test, expect } from "@playwright/test";
import { workspaceFixture } from "./fixtures/workspace";
for (const variant of ["light", "dark", "narrow"] as const) {
  test(`${variant} BoardUI date picker`, async ({ page }) => {
    await page.setViewportSize(variant === "narrow" ? {width: 480, height: 800} : {width: 1280, height: 900});
    await page.emulateMedia({colorScheme: variant === "light" ? "light" : "dark"});
    await workspaceFixture(page);
    await page.getByRole("button", {name: "Set up folder"}).click();
    await page.getByRole("button", {name: "From date: Select date"}).scrollIntoViewIfNeeded();
    await page.getByRole("button", {name: "From date: Select date"}).click();
    await expect(page.locator(".board-month-panel")).toBeVisible();
    await page.screenshot({path: `evidence/setup-controls/calendar-${variant}.png`});
  });
}
