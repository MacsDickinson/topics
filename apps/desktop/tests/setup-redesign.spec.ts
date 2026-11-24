import { openTool } from "./fixtures/workspace";
import { test, expect } from "@playwright/test";
import { workspaceFixture } from "./fixtures/workspace";

for (const appearance of ["light", "dark", "narrow"] as const) {
  test(`${appearance} setup selection, manifest and exact save receipt`, async ({
    page,
  }) => {
    await page.setViewportSize({
      width: appearance === "narrow" ? 600 : 1440,
      height: 960,
    });
    await page.emulateMedia({
      colorScheme: appearance === "dark" ? "dark" : "light",
      reducedMotion: "reduce",
    });
    await workspaceFixture(page);
    await openTool(page, "Set up folder");
    await expect(
      page.getByRole("heading", { name: "Choose notes to analyse" }),
    ).toBeVisible();
    await page.evaluate(() => window.scrollTo(0, 0));
      await page.screenshot({
      path: `evidence/redesign/after/${appearance}-folder-inventory.png`,
      fullPage: true,
    });
    const preview = page.getByRole("button", {
      name: "Review manifest",
      exact: true,
    });
    await preview.focus();
    await page.keyboard.press("Enter");
    await expect(
      page.getByRole("button", { name: "Build starting map", exact: true }),
    ).toBeDisabled();
    await expect(
      page.getByLabel("Authorize this exact manifest for this job"),
    ).not.toBeChecked();
    await page.evaluate(() => window.scrollTo(0, 0));
      await page.screenshot({
      path: `evidence/redesign/after/${appearance}-folder-manifest.png`,
      fullPage: true,
    });
    await page.getByLabel("Authorize this exact manifest for this job").check();
    await page
      .getByRole("button", { name: "Build starting map", exact: true })
      .click();
    await expect(
      page.getByRole("heading", { name: "Starting map" }),
    ).toBeVisible();
    await page.getByRole("button", { name: "Graph", exact: true }).click();
    const ledger = page.getByRole("button", {
      name: "Open Ledger",
      exact: true,
    });
    await ledger.focus();
    await page.keyboard.press("Enter");
    await expect(
      page.getByRole("button", { name: "Review topic group", exact: true }),
    ).toBeVisible();
    await page.evaluate(() => window.scrollTo(0, 0));
      await page.screenshot({
      path: `evidence/redesign/after/${appearance}-starting-map.png`,
      fullPage: true,
    });
    await page
      .getByRole("button", { name: "Review starting map", exact: true })
      .click();
    await expect(
      page.getByRole("heading", { name: "Save starting map", exact: true }),
    ).toBeVisible();
    await expect(page.getByRole("checkbox", { name: /^Include/ })).toHaveCount(
      0,
    );
    await page.evaluate(() => window.scrollTo(0, 0));
      await page.screenshot({
      path: `evidence/redesign/after/${appearance}-save-map.png`,
      fullPage: true,
    });
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth,
      ),
    ).toBe(true);
    expect(
      await page.evaluate(() =>
        (window as any).calls.filter((c: any) => c.command === "setup_decide"),
      ),
    ).toHaveLength(0);
  });

  test(`${appearance} contextual correction keeps statement and future rule explicit`, async ({
    page,
  }) => {
    await page.setViewportSize({
      width: appearance === "narrow" ? 600 : 1440,
      height: 960,
    });
    await page.emulateMedia({
      colorScheme: appearance === "dark" ? "dark" : "light",
      reducedMotion: "reduce",
    });
    await workspaceFixture(page, { populated: true });
    await page.getByRole("button", { name: "Knowledge graph", exact: true }).click();
    await page.getByRole("button", { name: "Ledger", exact: true }).click();
    await page
      .getByRole("button", { name: "That’s wrong", exact: true })
      .click();
    await expect(page.getByLabel("Your statement")).toBeFocused();
    await page
      .getByLabel("Your statement")
      .fill("The release calendar confirms Tuesday.");
    await page.getByLabel("Replacement", { exact: true }).fill("Tuesday");
    await page
      .getByRole("button", { name: "Preview correction", exact: true })
      .click();
    await expect(
      page.getByRole("heading", { name: "Review correction", exact: true }),
    ).toBeVisible();
    await expect(page.getByText(/Future rule:/)).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Approve correction", exact: true }),
    ).toBeEnabled();
    expect(
      await page.evaluate(() =>
        (window as any).calls.filter((c: any) => c.command === "setup_decide"),
      ),
    ).toHaveLength(0);
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth,
      ),
    ).toBe(true);
    await page.evaluate(() => window.scrollTo(0, 0));
      await page.screenshot({
      path: `evidence/redesign/after/${appearance}-context-correction.png`,
      fullPage: true,
    });
  });
}
